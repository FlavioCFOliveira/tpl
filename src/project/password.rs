//! `password_command`: a child process that supplies a password without it
//! being written to disk.
//!
//! Six rules govern one child, and every one of them is a security property
//! rather than a convenience:
//!
//! | Rule | Requirement |
//! |---|---|
//! | Executed **directly**, from an argument array, with no shell | `FR-CONF-024`, `FR-SEC-011` |
//! | Shell metacharacters are literal arguments | `FR-CONF-026` |
//! | Started as the leader of a process group of its own | `FR-CONF-028` |
//! | Bounded by a deadline over the whole phase — until the child has exited **and** its standard output has ended — exceeding which ends the whole group and is `78` | `FR-CONF-028`, `FR-SEC-012` |
//! | At most 4096 bytes read from its standard output, beyond which the whole group is terminated and the invocation is `78` | `FR-CONF-031`, `FR-SEC-024` |
//! | Its standard error goes to the null device: neither inherited nor captured | `FR-CONF-032` |
//!
//! **The cap is applied before the bytes are in memory, and it terminates the
//! child rather than truncating the read.** A password truncated at the cap
//! would be sent to the server, refused, and reported as a `77` naming the
//! credentials — a wrong diagnosis of a configuration fault, which is the
//! outcome `FR-CONF-031` is written to prevent. The read is therefore bounded
//! at the pipe, by a reader that stops one byte past the cap, and a child that
//! reaches that byte is killed, with its group, without waiting for it to exit.
//!
//! **The group, and why the reader is never waited on past the deadline.** A
//! helper may exit `0` after starting a descendant that inherits its standard
//! output (`printf secret; sleep 120 &`). The child exits at once, but the
//! pipe does not reach end of file until the descendant lets go of it, so a
//! read that waited for the end would wait past every deadline — finding
//! SEC-01 of `SECURITY-AUDIT.md`. `FR-CONF-028` therefore makes the phase end
//! only when both the exit and the end of file have been seen, and every way
//! this module ends the helper signals the whole group with `SIGKILL`, through
//! [`rustix::process::kill_process_group`]. The exit is observed without
//! reaping the child, so its pid — the group's id — stays reserved until the
//! group has been signalled and cannot name anyone else's group. It then leaves
//! the reader thread behind rather than joining it: a descendant that left the
//! group cannot be ended from here, and the invocation must not wait for it.
//! The abandoned thread holds nothing but the read end of the pipe and ends at
//! that pipe's end of file, or with the process.
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
use std::os::unix::process::CommandExt as _;
use std::process::{Command, Stdio};
use std::sync::mpsc;
use std::thread;
use std::time::{Duration, Instant};

use super::config::entry::PasswordCommand;
use super::secret::Secret;
use crate::deadline::{Bound, Phase};
use crate::error::{self, ChildEnd, Error, PasswordCommandFault};

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
/// Returns [`Error::PasswordCommandNotExecutable`] where no exit status is
/// obtained — the child could not be started, or its status could not be read
/// after it had started, which `FR-CONF-042` obliges the `cause` line to
/// separate — [`Error::PasswordCommandDeadlineExceeded`] where `bound` expired
/// before the phase ended, which is before the child had exited **and** its
/// standard output had reached end of file (`FR-CONF-028`), a helper that
/// exits `0` while a descendant holds its standard output included,
/// [`Error::PasswordCommandOutputCapExceeded`] where it wrote more than
/// [`OUTPUT_CAP`] bytes (`FR-CONF-031`), and [`Error::PasswordCommandFailed`]
/// where it exited non-zero (`FR-CONF-033`) or was ended by a signal `tpl` did
/// not send (`FR-CONF-043`).
pub(crate) fn obtain(
    entry: &str,
    command: &PasswordCommand,
    bound: Bound,
) -> Result<Secret, Error> {
    let arguments = command.arguments();

    if bound.expired() {
        return Err(expired(entry, arguments, bound));
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
        // FR-CONF-028: the leader of a process group of its own, so that the
        // deadline and the cap can end every descendant still in it.
        .process_group(0)
        .spawn()
        .map_err(|returned| Error::PasswordCommandNotExecutable {
            entry: entry.to_owned(),
            command: arguments.to_vec(),
            fault: PasswordCommandFault::NotStarted,
            returned,
        })?;

    let Some(stdout) = child.stdout.take() else {
        terminate(&mut child);

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
    let mut exited = false;

    // FR-GLOB-017 reports how long the phase took, whichever way it ended, so
    // the report is written by every exit of the loop below rather than by the
    // successful one alone.
    let report = || crate::diagnostics::emit::phase_ran(Phase::PasswordCommand, started.elapsed());

    // FR-CONF-028: the phase ends when the child has exited **and** its
    // standard output has reached end of file, and not at the first of the
    // two. A child that exited while a descendant still holds the pipe is a
    // phase still running, and the deadline below bounds it.
    //
    // The child's exit is **observed without reaping it**, by `exited` below:
    // an unreaped child keeps its pid, and so its group's id, reserved, which
    // is what makes the group kill of `terminate` reach the helper's group and
    // no other. The child is reaped only once the phase is over — after the
    // group kill, or after the pipe has closed.
    loop {
        if captured.is_none() {
            match receiver.try_recv() {
                Ok(bytes) if bytes.len() > OUTPUT_CAP => {
                    report();

                    return Err(abandon(
                        &mut child,
                        reader,
                        Error::PasswordCommandOutputCapExceeded {
                            entry: entry.to_owned(),
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

        if !exited {
            match self::exited(&child) {
                Ok(now) => exited = now,
                // FR-CONF-042's second condition, and the reason the fault is
                // carried: the child **had** started, so a line saying it
                // could not be started would be false of this path.
                Err(returned) => {
                    report();

                    return Err(abandon(
                        &mut child,
                        reader,
                        Error::PasswordCommandNotExecutable {
                            entry: entry.to_owned(),
                            command: arguments.to_vec(),
                            fault: PasswordCommandFault::StatusUnreadable,
                            returned,
                        },
                    ));
                }
            }
        }

        if exited && captured.is_some() {
            break;
        }

        if started.elapsed() >= bound.remaining() {
            report();

            return Err(abandon(
                &mut child,
                reader,
                expired(entry, arguments, bound),
            ));
        }

        thread::sleep(POLL);
    }

    report();

    // The phase is over: the child has exited and the pipe has closed, so the
    // wait reaps a zombie and returns at once. Nothing is signalled after it.
    let status = match child.wait() {
        Ok(status) => status,
        Err(returned) => {
            return Err(abandon(
                &mut child,
                reader,
                Error::PasswordCommandNotExecutable {
                    entry: entry.to_owned(),
                    command: arguments.to_vec(),
                    fault: PasswordCommandFault::StatusUnreadable,
                    returned,
                },
            ));
        }
    };

    // The reader has sent, so it has returned or is about to: joining it
    // waits for nothing but the thread's own exit.
    let _ = reader.join();
    let bytes = captured.unwrap_or_default();

    if bytes.len() > OUTPUT_CAP {
        return Err(Error::PasswordCommandOutputCapExceeded {
            entry: entry.to_owned(),
            command: arguments.to_vec(),
            cap: OUTPUT_CAP,
        });
    }

    if !status.success() {
        return Err(Error::PasswordCommandFailed {
            entry: entry.to_owned(),
            command: arguments.to_vec(),
            end: ended(&status),
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

/// How a child that did not exit zero ended.
///
/// `FR-CONF-033` owns the exit status and `FR-CONF-043` the signal, and the two
/// are read in that order because a status is the ordinary outcome. The signal
/// is read through [`ExitStatusExt`](std::os::unix::process::ExitStatusExt),
/// which every target of `NFR-PERF-018` carries: `FR-CONF-043` obliges the
/// `cause` line to name the number, and `FR-ERR-034` bans naming the category
/// instead where the instance is available.
///
/// **A signal `tpl` itself sent never reaches here.** `FR-CONF-028` and
/// `FR-CONF-031` each terminate the child's process group and return their own
/// condition before the status is read, which is what `FR-CONF-043` requires of
/// the two it excludes.
fn ended(status: &std::process::ExitStatus) -> ChildEnd {
    use std::os::unix::process::ExitStatusExt as _;

    if let Some(code) = status.code() {
        return ChildEnd::Exited(code);
    }

    status
        .signal()
        .map_or(ChildEnd::Unreported, ChildEnd::Signalled)
}

/// Ends the helper's process group, reaps the child, leaves the reader behind,
/// and returns `condition`.
///
/// `FR-CONF-028` and `FR-CONF-031` require every process of the group to be
/// terminated where the deadline expires or the cap is passed, and
/// `FR-CONF-028` forbids waiting on the child's standard output past the
/// deadline. The reader is therefore **dropped rather than joined**: once the
/// group is dead its pipe reaches end of file and the thread returns on its
/// own, and where a descendant that left the group still holds the pipe, the
/// thread stays blocked until the process exits — which is the one outcome the
/// requirement accepts for such a descendant.
fn abandon(
    child: &mut std::process::Child,
    reader: thread::JoinHandle<()>,
    condition: Error,
) -> Error {
    terminate(child);
    drop(reader);

    condition
}

/// Sends `SIGKILL` to the child's process group, then to the child, and reaps
/// it.
///
/// **The child is never reaped before this is called**: [`obtain`] observes
/// its exit through [`exited`], which leaves it a zombie. A zombie keeps its
/// pid, and a pid that is held cannot be handed to another process, nor become
/// the id of another group — so the group signalled here is the helper's own,
/// whether the child is still running or has exited while a descendant holds
/// the pipe. The child is reaped by the `wait` below, after the signal.
///
/// That guarantee rests on one condition outside this function: nothing else
/// in the process reaps the child. `tpl` installs no `SIGCHLD` disposition —
/// ignoring it would make the kernel reap children on exit — and nothing else
/// waits on an arbitrary pid.
///
/// A pid that does not fit the platform's type, or that is `1`, is not
/// signalled as a group: `kill(-1, …)` would reach every process this user
/// may signal. No child of this process can hold either, so the guard costs
/// nothing and keeps the call total.
fn terminate(child: &mut std::process::Child) {
    use rustix::process::{Signal, kill_process_group};

    if let Some(group) = pid_of(child) {
        // ESRCH — the group has no member left — is the state being asked
        // for, and every other error leaves the child to the kill below.
        let _ = kill_process_group(group, Signal::KILL);
    }

    let _ = child.kill();
    let _ = child.wait();
}

/// The child's pid, as the type the system calls take, or [`None`] where it
/// is one no group kill may target.
fn pid_of(child: &std::process::Child) -> Option<rustix::process::Pid> {
    i32::try_from(child.id())
        .ok()
        .filter(|pid| *pid > 1)
        .and_then(rustix::process::Pid::from_raw)
}

/// Whether the child has exited, observed **without reaping it**.
///
/// `waitid` with `WNOWAIT` reports the exit and leaves the child waitable, so
/// it stays a zombie holding its pid — and its group's id — until
/// [`terminate`] or the success path of [`obtain`] reaps it. `WNOHANG` makes
/// the call answer at once; an exit by a signal is an exit here too, because
/// `WEXITED` covers both.
///
/// # Errors
///
/// Returns what `waitid` returns, and an error of its own where the child's
/// pid cannot be expressed — which no child of this process can have.
fn exited(child: &std::process::Child) -> std::io::Result<bool> {
    use rustix::process::{WaitId, WaitIdOptions, waitid};

    let pid = pid_of(child)
        .ok_or_else(|| std::io::Error::other("the child's pid cannot be waited on"))?;
    let options = WaitIdOptions::EXITED | WaitIdOptions::NOHANG | WaitIdOptions::NOWAIT;

    waitid(WaitId::Pid(pid), options)
        .map(|status| status.is_some())
        .map_err(|errno| std::io::Error::from_raw_os_error(errno.raw_os_error()))
}

/// The condition of `FR-CONF-028`, naming which bound expired.
fn expired(entry: &str, arguments: &[String], bound: Bound) -> Error {
    Error::PasswordCommandDeadlineExceeded {
        entry: entry.to_owned(),
        command: arguments.to_vec(),
        bound: bound.bound(),
        limit: bound.limit(),
    }
}

#[cfg(test)]
mod tests {
    use super::{OUTPUT_CAP, exited, obtain, pid_of, terminate};
    use crate::deadline::{Clock, Seconds};
    use crate::error::{ChildEnd, DeadlineBound, Error};
    use crate::project::config::entry::PasswordCommand;
    use crate::project::scratch::Scratch;
    use std::num::NonZeroU64;
    use std::os::unix::process::CommandExt as _;

    /// The database entry every command here is declared by.
    const ENTRY: &str = "shop";
    use std::path::Path;
    use std::time::{Duration, Instant};

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
        let produced = obtain(ENTRY, &command(&[&echo(), "  hunter2  "]), bound(10))
            .expect("the child produced a password");

        assert_eq!(produced.expose(), "hunter2");
    }

    #[test]
    fn fr_conf_026_a_shell_metacharacter_reaches_the_child_as_a_literal_argument() {
        // FR-CONF-026, FR-SEC-011. Every one of these would differ under a
        // shell: `;` would start a second command, `$(…)` would substitute,
        // `|` would pipe, and `*` would glob.
        for hostile in ["a; echo b", "$(echo b)", "a | echo b", "*", "a && echo b"] {
            let produced = obtain(ENTRY, &command(&[&echo(), hostile]), bound(10))
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
        let condition = obtain(ENTRY, &command(&[&sleep, "30"]), bound(1))
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
            obtain(ENTRY, &command(&[&yes]), bound(10)).expect_err("the child writes without end");

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
            ENTRY,
            &command(&[&head, "-c", &OUTPUT_CAP.to_string(), zero]),
            bound(10),
        );

        assert!(produced.is_ok(), "{produced:?}");
    }

    #[test]
    fn fr_conf_033_a_child_that_exits_non_zero_is_refused_with_the_status_it_returned() {
        // FR-CONF-033: the cause names the command as stored and the status.
        let no = tool(&["/usr/bin/false", "/bin/false"]);
        let condition = obtain(ENTRY, &command(&[&no]), bound(10)).expect_err("the child fails");

        match condition {
            Error::PasswordCommandFailed {
                ref command, end, ..
            } => {
                assert_eq!(command, &[no]);
                assert_eq!(end, ChildEnd::Exited(1));
            }
            other => panic!("expected a non-zero exit, got {other:?}"),
        }
        assert_eq!(condition.exit_code(), 78);
    }

    #[test]
    fn fr_conf_033_a_program_that_does_not_exist_is_refused_as_a_configuration_fault() {
        let condition = obtain(
            ENTRY,
            &command(&["/nonexistent/tpl-password-helper"]),
            bound(10),
        )
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
            ENTRY,
            &command(&[&cat, "/nonexistent/tpl-file-that-is-not-there"]),
            bound(10),
        )
        .expect_err("the child fails");

        let rendered = format!("{condition}");
        assert!(!rendered.contains("No such file"), "{rendered}");
        assert!(matches!(condition, Error::PasswordCommandFailed { .. }));
    }

    /// The interpreter every script below is run by.
    const SH: &str = "/bin/sh";

    /// An `sh` script at `name` in `scratch`, running `body`, to be run as
    /// `[SH, script]`.
    ///
    /// The script is handed to the shell rather than executed itself. A file
    /// just written and then executed races every other test that spawns a
    /// child meanwhile: a sibling forked in that instant inherits the writing
    /// descriptor until its own `exec`, and executing the script while it is
    /// held was observed to stall the child before its first line, which made
    /// a deadline test pass for the wrong reason. The shell only reads the
    /// file, which no held descriptor delays.
    fn script(scratch: &Scratch, name: &str, body: &str) -> String {
        let path = scratch.file(name, body);

        path.to_str().expect("the scratch path is UTF-8").to_owned()
    }

    /// The pid a script wrote to `file`.
    fn written_pid(file: &Path) -> rustix::process::Pid {
        let text = std::fs::read_to_string(file).expect("the script wrote the descendant's pid");
        let raw: i32 = text.trim().parse().expect("the file holds a pid");

        rustix::process::Pid::from_raw(raw).expect("a pid is positive")
    }

    /// Whether the process `pid` is gone, waiting up to two seconds for it.
    ///
    /// A descendant killed with its group is re-parented once its parent has
    /// exited and reaped by the new parent, not by this process, so its
    /// disappearance lags the signal by however long that parent takes.
    fn gone(pid: rustix::process::Pid) -> bool {
        let until = Instant::now() + Duration::from_secs(2);

        loop {
            if rustix::process::test_kill_process(pid).is_err() {
                return true;
            }
            if Instant::now() >= until {
                return false;
            }
            std::thread::sleep(Duration::from_millis(10));
        }
    }

    #[test]
    fn fr_conf_028_a_descendant_holding_the_output_after_the_child_exits_ends_at_the_deadline() {
        // FR-CONF-028, FR-SEC-012, finding SEC-01: the child writes a password,
        // leaves a descendant holding its standard output, and exits 0. The
        // phase has not ended, so the deadline ends it: 78 within the bound,
        // and the descendant, still in the group, is dead afterwards.
        let scratch = Scratch::new();
        let pid_file = scratch.path("descendant.pid");
        let helper = script(
            &scratch,
            "holder.sh",
            &format!(
                "printf secret\nsleep 120 &\necho $! > '{}'\nexit 0\n",
                pid_file.display()
            ),
        );

        let began = Instant::now();
        let condition = obtain(ENTRY, &command(&[SH, &helper]), bound(1))
            .expect_err("the output never ends in time");
        let took = began.elapsed();

        match condition {
            Error::PasswordCommandDeadlineExceeded { bound, .. } => {
                assert_eq!(bound, DeadlineBound::Phase);
            }
            ref other => panic!("expected a deadline, got {other:?}"),
        }
        assert_eq!(condition.exit_code(), 78);
        assert!(took < Duration::from_secs(2), "took {took:?}");
        assert!(
            gone(written_pid(&pid_file)),
            "the descendant outlived the group"
        );
    }

    #[test]
    fn fr_conf_028_the_child_is_still_unreaped_when_its_group_is_killed_at_the_deadline() {
        // The exit is observed without reaping, so the pid — the group's id —
        // is still held when the group is signalled. A descendant in the group
        // records the child's process state every 20 ms until the group kill
        // ends it: the last record is a zombie (`Z`). Had the child been
        // reaped before the kill, `ps` would find no such process and the last
        // records would carry no state at all.
        let scratch = Scratch::new();
        let states = scratch.path("states");
        let helper = script(
            &scratch,
            "watched.sh",
            &format!(
                "printf secret\n\
                 ( while :; do echo \"$(ps -o stat= -p $$)|\" >> '{}'; sleep 0.02; done ) &\n\
                 exit 0\n",
                states.display()
            ),
        );

        let condition = obtain(ENTRY, &command(&[SH, &helper]), bound(1))
            .expect_err("the output never ends in time");
        assert!(matches!(
            condition,
            Error::PasswordCommandDeadlineExceeded { .. }
        ));

        // The watcher is dead once `obtain` returns: nothing appends after.
        let recorded = std::fs::read_to_string(&states).expect("the watcher wrote");
        let last = recorded
            .lines()
            .last()
            .expect("the watcher wrote at least one record");
        assert!(
            last.trim_start().starts_with('Z'),
            "the child was not a zombie when its group was killed; last record {last:?} of \
             {recorded:?}"
        );
    }

    #[test]
    fn fr_conf_028_an_exit_is_observed_without_reaping_and_the_group_kill_precedes_the_reap() {
        // The two functions the deadline path is made of, driven directly.
        let scratch = Scratch::new();
        let pid_file = scratch.path("descendant.pid");
        let helper = script(
            &scratch,
            "leader.sh",
            &format!("sleep 120 &\necho $! > '{}'\nexit 0\n", pid_file.display()),
        );
        let mut child = std::process::Command::new(SH)
            .arg(&helper)
            .stdin(std::process::Stdio::null())
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null())
            .process_group(0)
            .spawn()
            .expect("the script starts");
        let pid = pid_of(&child).expect("a child's pid is above 1");

        let until = Instant::now() + Duration::from_secs(5);
        while !exited(&child).expect("waitid answers") {
            assert!(Instant::now() < until, "the child did not exit");
            std::thread::sleep(Duration::from_millis(5));
        }

        // Observed, and not reaped: the pid still names a process (the
        // zombie), and the exit is still there to be observed.
        assert!(rustix::process::test_kill_process(pid).is_ok());
        assert!(exited(&child).expect("waitid answers"));

        terminate(&mut child);

        assert!(
            rustix::process::test_kill_process(pid).is_err(),
            "terminate reaps the child"
        );
        assert!(
            gone(written_pid(&pid_file)),
            "the group kill reached the descendant"
        );
    }

    #[test]
    fn fr_conf_031_passing_the_cap_ends_the_whole_group_descendants_included() {
        // FR-CONF-031, FR-SEC-024: the cap terminates the group the deadline
        // does, so a descendant holding the pipe does not survive it.
        let scratch = Scratch::new();
        let pid_file = scratch.path("descendant.pid");
        let helper = script(
            &scratch,
            "flood.sh",
            &format!(
                "sleep 120 &\necho $! > '{}'\nhead -c {} /dev/zero\nexit 0\n",
                pid_file.display(),
                OUTPUT_CAP * 2
            ),
        );

        let began = Instant::now();
        let condition = obtain(ENTRY, &command(&[SH, &helper]), bound(10))
            .expect_err("the child passes the cap");
        let took = began.elapsed();

        match condition {
            Error::PasswordCommandOutputCapExceeded { cap, .. } => assert_eq!(cap, OUTPUT_CAP),
            ref other => panic!("expected the output cap, got {other:?}"),
        }
        assert_eq!(condition.exit_code(), 78);
        assert!(took < Duration::from_secs(10), "took {took:?}");
        assert!(
            gone(written_pid(&pid_file)),
            "the descendant outlived the group"
        );
    }

    #[test]
    fn fr_conf_028_a_descendant_that_releases_the_output_in_time_leaves_the_password_usable() {
        // FR-CONF-028: the phase ends at the exit and the end of file, so a
        // descendant that closes its copy of the pipe before the deadline
        // leaves an ordinary success.
        let scratch = Scratch::new();
        let helper = script(
            &scratch,
            "brief.sh",
            "printf secret\n(sleep 0.2) &\nexit 0\n",
        );

        let produced =
            obtain(ENTRY, &command(&[SH, &helper]), bound(5)).expect("the output ended in time");

        assert_eq!(produced.expose(), "secret");
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

        let condition = obtain(
            ENTRY,
            &command(&["/nonexistent/tpl-password-helper"]),
            spent,
        )
        .expect_err("the budget is spent");

        match condition {
            Error::PasswordCommandDeadlineExceeded { bound, .. } => {
                assert_eq!(bound, DeadlineBound::Overall);
            }
            other => panic!("expected a deadline, got {other:?}"),
        }
    }
}
