//! `password_command`: a child process that supplies a password without it
//! being written to disk.
//!
//! Five rules govern one child, and every one of them is a security property
//! rather than a convenience:
//!
//! | Rule | Requirement |
//! |---|---|
//! | Executed **directly**, from an argument array, with no shell | `FR-CONF-024`, `FR-SEC-011` |
//! | Shell metacharacters are literal arguments | `FR-CONF-026` |
//! | Bounded by a deadline, exceeding which is `78` | `FR-CONF-028`, `FR-SEC-012` |
//! | At most 4096 bytes read from its standard output, beyond which the child is terminated and the invocation is `78` | `FR-CONF-031` |
//! | Its standard error goes to the null device: neither inherited nor captured | `FR-CONF-032` |
//!
//! **The cap is applied before the bytes are in memory, and it terminates the
//! child rather than truncating the read.** A password truncated at the cap
//! would be sent to the server, refused, and reported as a `77` naming the
//! credentials — a wrong diagnosis of a configuration fault, which is the
//! outcome `FR-CONF-031` is written to prevent. The read is therefore bounded
//! at the pipe, by a reader that stops one byte past the cap, and a child that
//! reaches that byte is killed without waiting for it to exit.
//!
//! **Why a reader thread and a polling loop.** The three obligations — a
//! deadline, a bounded read, and a child that may never exit — cannot be met by
//! a blocking `wait` or by a blocking `read`, because either would hold the
//! process past the deadline. A child writing more than a pipe holds also
//! blocks on its own write until something drains it, so the drain has to
//! happen while the deadline is being watched. The runtime of `ADR-005` is
//! scoped to the database driver and is not started for a `cfg` invocation, so
//! the watcher is a thread and a sleep of a millisecond rather than a task.

use std::io::Read as _;
use std::process::{Command, Stdio};
use std::sync::mpsc;
use std::thread;
use std::time::{Duration, Instant};

use super::config::entry::PasswordCommand;
use super::secret::Secret;
use crate::deadline::Bound;
use crate::error::{self, Error};

/// The most the system reads from the child's standard output, in bytes
/// (`FR-CONF-031`).
///
/// Fixed here rather than made configurable, because a key that raised it would
/// be a key that reopened the denial-of-service surface an untrusted `.cfg`
/// reaches.
pub(crate) const OUTPUT_CAP: usize = 4096;

/// How often the deadline and the child are looked at.
const POLL: Duration = Duration::from_millis(1);

/// Runs `command` and returns the password it produced.
///
/// The password is the **trimmed** standard output of the child, per
/// `FR-CONF-027`, and it is returned inside a [`Secret`], which has no
/// [`Display`](std::fmt::Display) and a [`Debug`](std::fmt::Debug) that prints
/// nothing of it.
///
/// # Errors
///
/// Returns [`Error::PasswordCommandNotExecutable`] where the child could not be
/// started, [`Error::PasswordCommandDeadlineExceeded`] where `bound` expired
/// before it finished (`FR-CONF-028`),
/// [`Error::PasswordCommandOutputCapExceeded`] where it wrote more than
/// [`OUTPUT_CAP`] bytes (`FR-CONF-031`), and [`Error::PasswordCommandFailed`]
/// where it exited non-zero (`FR-CONF-033`).
pub(crate) fn obtain(command: &PasswordCommand, bound: Bound) -> Result<Secret, Error> {
    let arguments = command.arguments();

    if bound.expired() {
        return Err(expired(arguments, bound));
    }

    let mut child = Command::new(command.program())
        .args(&arguments[1..])
        // FR-SEC-023: `tpl` never reads stdin except for an explicitly
        // requested `--context -`, and a child inheriting it could.
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        // FR-CONF-032: neither inherited nor captured. There is nothing to
        // leak because there is nothing held.
        .stderr(Stdio::null())
        .spawn()
        .map_err(|returned| Error::PasswordCommandNotExecutable {
            command: arguments.to_vec(),
            returned,
        })?;

    let Some(stdout) = child.stdout.take() else {
        let _ = child.kill();
        let _ = child.wait();

        return error::ensure_invariant(
            false,
            "a child spawned with a piped standard output has one",
        )
        .map(|()| Secret::new(String::new()));
    };

    let (sender, receiver) = mpsc::channel();
    let reader = thread::spawn(move || {
        let mut captured = Vec::with_capacity(OUTPUT_CAP + 1);
        // One byte past the cap is enough to know the cap was passed, and is
        // the whole of what is ever held in memory.
        // A read that fails part-way sends nothing rather than a prefix: a
        // truncated password is the outcome FR-CONF-031 refuses, and an empty
        // capture leaves the child's own exit status to diagnose the run.
        if stdout
            .take(OUTPUT_CAP as u64 + 1)
            .read_to_end(&mut captured)
            .is_err()
        {
            captured.clear();
        }

        let _ = sender.send(captured);
    });

    let started = Instant::now();
    let mut captured: Option<Vec<u8>> = None;

    let status = loop {
        if captured.is_none() {
            match receiver.try_recv() {
                Ok(bytes) if bytes.len() > OUTPUT_CAP => {
                    return Err(reap(
                        &mut child,
                        reader,
                        Error::PasswordCommandOutputCapExceeded {
                            command: arguments.to_vec(),
                            cap: OUTPUT_CAP,
                        },
                    ));
                }
                Ok(bytes) => captured = Some(bytes),
                Err(mpsc::TryRecvError::Empty) => {}
                Err(mpsc::TryRecvError::Disconnected) => captured = Some(Vec::new()),
            }
        }

        match child.try_wait() {
            Ok(Some(status)) => break status,
            Ok(None) => {}
            Err(returned) => {
                return Err(reap(
                    &mut child,
                    reader,
                    Error::PasswordCommandNotExecutable {
                        command: arguments.to_vec(),
                        returned,
                    },
                ));
            }
        }

        if started.elapsed() >= bound.remaining() {
            return Err(reap(&mut child, reader, expired(arguments, bound)));
        }

        thread::sleep(POLL);
    };

    let _ = reader.join();
    let bytes = match captured {
        Some(bytes) => bytes,
        None => receiver.recv().unwrap_or_default(),
    };

    if bytes.len() > OUTPUT_CAP {
        return Err(Error::PasswordCommandOutputCapExceeded {
            command: arguments.to_vec(),
            cap: OUTPUT_CAP,
        });
    }

    if !status.success() {
        return Err(Error::PasswordCommandFailed {
            command: arguments.to_vec(),
            status: status.code(),
        });
    }

    // FR-OUT-017 is the corpus's posture on a byte sequence that is not valid
    // UTF-8: replace it and continue. A helper that prints one has produced a
    // password the server will refuse, which is a `77` the caller can read,
    // rather than a condition this module can diagnose.
    Ok(Secret::new(
        String::from_utf8_lossy(&bytes).trim().to_owned(),
    ))
}

/// Terminates the child, waits for the reader, and returns `condition`.
///
/// `FR-CONF-031` requires the child to be terminated where the cap is passed,
/// and a child abandoned at a deadline must not be left running behind the
/// process either.
fn reap(
    child: &mut std::process::Child,
    reader: thread::JoinHandle<()>,
    condition: Error,
) -> Error {
    let _ = child.kill();
    let _ = child.wait();
    let _ = reader.join();

    condition
}

/// The condition of `FR-CONF-028`, naming which bound expired.
fn expired(arguments: &[String], bound: Bound) -> Error {
    Error::PasswordCommandDeadlineExceeded {
        command: arguments.to_vec(),
        bound: bound.bound(),
        limit: bound.limit(),
    }
}

#[cfg(test)]
mod tests {
    use super::{OUTPUT_CAP, obtain};
    use crate::deadline::{Clock, Seconds};
    use crate::error::{DeadlineBound, Error};
    use crate::project::config::entry::PasswordCommand;
    use std::num::NonZeroU64;

    /// A bound of `limit` seconds with no overall budget in force.
    fn bound(limit: u64) -> crate::deadline::Bound {
        Clock::new(None).bound(Seconds::new(
            NonZeroU64::new(limit).expect("the test writes a positive value"),
        ))
    }

    /// The first of `candidates` that exists on this system.
    ///
    /// The two supported families put these programs in one of two places, and
    /// a test that silently skipped would report nothing at all.
    fn tool(candidates: &[&str]) -> String {
        candidates
            .iter()
            .find(|path| std::path::Path::new(path).exists())
            .map(|path| (*path).to_owned())
            .unwrap_or_else(|| panic!("none of {candidates:?} exists on this system"))
    }

    fn echo() -> String {
        tool(&["/bin/echo", "/usr/bin/echo"])
    }

    /// The command `arguments` names.
    fn command(arguments: &[&str]) -> PasswordCommand {
        PasswordCommand::new(arguments.iter().map(|word| (*word).to_owned()).collect())
            .expect("the array carries a program")
    }

    #[test]
    fn fr_conf_027_the_password_is_the_trimmed_standard_output_of_the_child() {
        // FR-CONF-027.
        let produced = obtain(&command(&[&echo(), "  hunter2  "]), bound(10))
            .expect("the child produced a password");

        assert_eq!(produced.expose(), "hunter2");
    }

    #[test]
    fn fr_conf_026_a_shell_metacharacter_reaches_the_child_as_a_literal_argument() {
        // FR-CONF-026, FR-SEC-011. Every one of these would differ under a
        // shell: `;` would start a second command, `$(…)` would substitute,
        // `|` would pipe, and `*` would glob.
        for hostile in ["a; echo b", "$(echo b)", "a | echo b", "*", "a && echo b"] {
            let produced = obtain(&command(&[&echo(), hostile]), bound(10))
                .expect("the child produced a password");

            assert_eq!(
                produced.expose(),
                hostile,
                "{hostile:?} was interpreted rather than passed through"
            );
        }
    }

    #[test]
    fn fr_conf_028_a_child_that_exceeds_its_deadline_is_terminated_and_refused() {
        // FR-CONF-028, FR-SEC-012: the deadline is what keeps a command
        // waiting on a FIFO from hanging the caller with no diagnosis.
        let sleep = tool(&["/bin/sleep", "/usr/bin/sleep"]);
        let condition = obtain(&command(&[&sleep, "30"]), bound(1))
            .expect_err("the child outlives its deadline");

        match condition {
            Error::PasswordCommandDeadlineExceeded { bound, .. } => {
                assert_eq!(bound, DeadlineBound::Phase);
            }
            other => panic!("expected a deadline, got {other:?}"),
        }
        assert_eq!(condition.exit_code(), 78);
    }

    #[test]
    fn fr_conf_031_a_child_that_writes_more_than_the_cap_is_terminated_and_refused() {
        // FR-CONF-031: the bound is a refusal rather than a truncation, and
        // the child is terminated.
        let yes = tool(&["/usr/bin/yes", "/bin/yes"]);
        let condition =
            obtain(&command(&[&yes]), bound(10)).expect_err("the child writes without end");

        match condition {
            Error::PasswordCommandOutputCapExceeded { cap, .. } => assert_eq!(cap, OUTPUT_CAP),
            other => panic!("expected the output cap, got {other:?}"),
        }
        assert_eq!(condition.exit_code(), 78);
    }

    #[test]
    fn fr_conf_031_output_exactly_at_the_cap_is_accepted() {
        // The cap is the most that is read, so a helper printing exactly that
        // many bytes is not over it.
        let head = tool(&["/usr/bin/head", "/bin/head"]);
        let zero = "/dev/zero";
        if !std::path::Path::new(zero).exists() {
            return;
        }

        let produced = obtain(
            &command(&[&head, "-c", &OUTPUT_CAP.to_string(), zero]),
            bound(10),
        );

        assert!(produced.is_ok(), "{produced:?}");
    }

    #[test]
    fn fr_conf_033_a_child_that_exits_non_zero_is_refused_with_the_status_it_returned() {
        // FR-CONF-033: the cause names the command as stored and the status.
        let no = tool(&["/usr/bin/false", "/bin/false"]);
        let condition = obtain(&command(&[&no]), bound(10)).expect_err("the child fails");

        match condition {
            Error::PasswordCommandFailed {
                ref command,
                status,
            } => {
                assert_eq!(command, &[no]);
                assert_eq!(status, Some(1));
            }
            other => panic!("expected a non-zero exit, got {other:?}"),
        }
        assert_eq!(condition.exit_code(), 78);
    }

    #[test]
    fn fr_conf_033_a_program_that_does_not_exist_is_refused_as_a_configuration_fault() {
        let condition = obtain(&command(&["/nonexistent/tpl-password-helper"]), bound(10))
            .expect_err("the child cannot be started");

        assert!(matches!(
            condition,
            Error::PasswordCommandNotExecutable { .. }
        ));
        assert_eq!(condition.exit_code(), 78);
    }

    #[test]
    fn fr_conf_032_the_standard_error_of_a_failing_child_reaches_no_message() {
        // FR-CONF-032, FR-SEC-005: it goes to the null device, so there is
        // nothing to name and the diagnosis is the exit status alone.
        let cat = tool(&["/bin/cat", "/usr/bin/cat"]);
        let condition = obtain(
            &command(&[&cat, "/nonexistent/tpl-file-that-is-not-there"]),
            bound(10),
        )
        .expect_err("the child fails");

        let rendered = format!("{condition}");
        assert!(!rendered.contains("No such file"), "{rendered}");
        assert!(matches!(condition, Error::PasswordCommandFailed { .. }));
    }

    #[test]
    fn fr_glob_012_a_bound_that_has_already_expired_never_starts_a_child() {
        // FR-GLOB-012: a phase ends at the first of the two bounds, and an
        // overall budget already spent is the first. The program named here
        // does not exist, so a run that reached the spawn would report that
        // instead.
        let spent = crate::deadline::Bound::spent(Seconds::new(
            NonZeroU64::new(10).expect("the test writes a positive value"),
        ));

        let condition = obtain(&command(&["/nonexistent/tpl-password-helper"]), spent)
            .expect_err("the budget is spent");

        match condition {
            Error::PasswordCommandDeadlineExceeded { bound, .. } => {
                assert_eq!(bound, DeadlineBound::Overall);
            }
            other => panic!("expected a deadline, got {other:?}"),
        }
    }
}
