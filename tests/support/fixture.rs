//! The MariaDB fixture of `scripts/mariadb/`, as a test reaches it.
//!
//! Three things live here, and nothing else: the **gate** that answers whether
//! the fixture is up, the **inventory** that says which series answered and
//! where, and the three **server-side instruments** of `NFR-PERF-007`. The
//! fourth instrument of that requirement, the differential run, needs no
//! server and lives in [`differential`](../differential/index.html).
//!
//! # Why this file is under `tests/support/` and is reached by `#[path]`
//!
//! Cargo makes an integration-test target of `tests/*.rs` and of
//! `tests/*/main.rs`, and of nothing else. A file under `tests/support/` whose
//! name is not `main.rs` is therefore never a target of its own: it is compiled
//! into the test binaries that name it, and `cargo test` never runs it as a
//! test with no tests in it.
//!
//! The project's module style — `foo.rs` beside `foo/`, never `mod.rs` — cannot
//! be used to reach it, because `tests/support.rs` beside `tests/support/`
//! would be exactly the target that must not exist. `#[path]` is what replaces
//! the pair: the including file writes
//!
//! ```ignore
//! #[path = "support/fixture.rs"]
//! mod fixture;
//! ```
//!
//! which keeps the module name `fixture`, keeps `mod.rs` out of the tree, and
//! keeps the file off the target list.
//!
//! # The fixture is asked, never restated
//!
//! No port, no server name, no credential and no schema name is written in
//! Rust. The inventory is read from the harness at run time:
//!
//! | Question | Asked of |
//! |---|---|
//! | Is the fixture up? | `status.sh --quiet`, whose exit code is the gate |
//! | Where did each server answer? | `status.sh --export`, whose output is **parsed, never `eval`ed** |
//! | Which of them are the four series of `FR-SRV-015`? | `series.env`, through its own `tpl_mariadb_each … tls` iterator |
//!
//! The third question is asked of `series.env` because `--export` cannot answer
//! it: it prints a name and an address per server that answered, and drops the
//! column that distinguishes the four series from the `--skip-ssl` server,
//! which `series.env` states is not a fifth series. `series.env` is part of the
//! harness's published interface, and its iterator is documented as *"the four
//! of `FR-SRV-015` only"*.
//!
//! **No `docker` command is issued from Rust.** Every fixture operation goes
//! through `up.sh`, `down.sh`, `status.sh`, `observe.sh` and `series.env`.
//!
//! # Why this module allows dead code
//!
//! It is included by more than one test binary, and each of them uses the part
//! of the harness its own subject needs: the observations of `NFR-PERF-007`
//! want the three instruments, and a test that reads a catalogue wants the
//! gate, the inventory and an address. An item unused in one of those binaries
//! is not dead code, and without this allowance `cargo clippy --all-targets`
//! would reject the file for being complete.

#![allow(
    dead_code,
    reason = "this module is included by more than one test binary and each uses the part of the \
              harness its own subject needs; an item unused in one of them is not dead code"
)]

use std::collections::BTreeMap;
use std::io::{Read as _, Write as _};
use std::net::{SocketAddr, TcpStream, ToSocketAddrs as _};
use std::path::{Path, PathBuf};
use std::process::{Command, Output};
use std::sync::{Mutex, MutexGuard, OnceLock};
use std::time::Duration;

/// How long a probe of a fixture port may take before it is treated as an
/// absent server.
const PROBE_TIMEOUT: Duration = Duration::from_secs(5);

/// The connections the observer's own second reading accounts for.
///
/// `observe.sh connections` reaches the server to read its counter, so the
/// reading that closes a bracket is itself counted. The value is `1`, measured
/// on all five servers rather than assumed, and recorded in
/// `scripts/mariadb/README.md` under *The connections a server accepts*.
const OBSERVER_BASELINE: i64 = 1;

/// The shell that reads `series.env`.
///
/// It sources the inventory and calls the harness's own iterator with the
/// `tls` filter, which `series.env` documents as the four series of
/// `FR-SRV-015`. Nothing of the inventory is decided here.
const SERIES_SCRIPT: &str = "set -eu; . ./series.env; \
     emit() { printf '%s\\n' \"$1\"; }; tpl_mariadb_each emit tls";

/// What `scripts/mariadb/status.sh` answered, read as the three-valued gate its
/// own header defines.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Gate {
    /// Exit `0`: every server answered. Run the server-dependent tests.
    Ready,
    /// Exit `1`: no server answered. Skip them, and say why.
    Absent,
    /// Exit `2`, or anything else: a broken fixture, or a gate that could not
    /// be asked. Neither is an absent fixture, and neither is skipped over.
    Broken(Option<i32>),
}

/// Maps the gate's exit code, and nothing else.
///
/// It is a free function over an `Option<i32>` so that the mapping can be
/// exercised without a fixture: a test feeds it `0`, `1` and `2` and reads the
/// three values back. `None` is a gate that was signalled rather than exited,
/// and every code the header does not define joins exit `2` in
/// [`Gate::Broken`] — a gate that answered something unforeseen is a gate whose
/// answer is unknown, and an unknown answer is never read as *absent*.
pub fn gate_of(code: Option<i32>) -> Gate {
    match code {
        Some(0) => Gate::Ready,
        Some(1) => Gate::Absent,
        other => Gate::Broken(other),
    }
}

/// One server of the fixture: the name the harness knows it by, and the
/// address it answered on.
#[derive(Debug, Clone)]
pub struct Server {
    /// The name `series.env` gives it, which is also the argument `status.sh`
    /// and `observe.sh` take.
    name: String,
    /// `host:port`, exactly as `status.sh --export` printed it.
    address: String,
}

impl Server {
    /// The name the harness knows it by.
    pub fn name(&self) -> &str {
        &self.name
    }

    /// The address it answered on, as `status.sh --export` printed it.
    ///
    /// It is `host:port`, and it is the harness's answer rather than this
    /// file's: no port is written in Rust.
    pub fn address(&self) -> &str {
        &self.address
    }

    /// The address it answered on, as `status.sh --export` printed it,
    /// resolved.
    fn socket_address(&self) -> SocketAddr {
        self.address
            .to_socket_addrs()
            .unwrap_or_else(|failure| panic!("status.sh exported {:?}: {failure}", self.address))
            .next()
            .unwrap_or_else(|| {
                panic!(
                    "status.sh exported {:?}, which resolved to nothing",
                    self.address
                )
            })
    }
}

/// What one ask of the gate and the inventory established.
#[derive(Debug)]
struct State {
    /// What `status.sh --quiet` answered.
    gate: Gate,
    /// The four series of `FR-SRV-015`, in the order `series.env` lists them.
    /// Empty unless the gate is [`Gate::Ready`].
    series: Vec<Server>,
    /// How many servers `status.sh --export` named, which is every server of
    /// the fixture and not only the series among them.
    answered: usize,
}

/// Asks the gate and reads the inventory, once per test binary.
fn state() -> &'static State {
    static STATE: OnceLock<State> = OnceLock::new();

    STATE.get_or_init(probe)
}

/// The four series of `FR-SRV-015`, or `None` when the fixture is absent.
///
/// `test` names the test in the skip notice, so that a reader of a run knows
/// which body did not execute.
///
/// A broken fixture — some servers answered and some did not — **panics**
/// rather than skipping. The gate has three values precisely so that half a
/// fixture is not read as none, and a runner that skipped over it would report
/// a real failure as a pass.
pub fn series(test: &str) -> Option<&'static [Server]> {
    let state = state();

    match state.gate {
        Gate::Ready => Some(&state.series),
        Gate::Absent => {
            notice(&format!(
                "skipped {test}: the MariaDB fixture is not up \
                 (scripts/mariadb/status.sh --quiet exited 1: no server answered). \
                 Start it with ./scripts/mariadb/up.sh."
            ));
            None
        }
        Gate::Broken(code) => panic!(
            "the MariaDB fixture is broken, not absent: \
             scripts/mariadb/status.sh --quiet {}. \
             Some servers answered and some did not, or the gate could not be asked; \
             either way this is a failure and is not skipped over. \
             Run ./scripts/mariadb/status.sh to see which, then ./scripts/mariadb/down.sh \
             and ./scripts/mariadb/up.sh.",
            match code {
                Some(code) => format!("exited {code}"),
                None => "was signalled".to_owned(),
            }
        ),
    }
}

/// How many servers `status.sh --export` named.
///
/// It is the whole fixture, series and otherwise: `series.env` states that the
/// `--skip-ssl` record `FR-CONF-038` obliges **is not a fifth series**, so this
/// count is larger than the set [`series`] drives, and a driver that walked the
/// fixture instead of the series would be caught by the difference.
pub fn servers_that_answered() -> usize {
    state().answered
}

/// Serialises every test that touches a fixture server.
///
/// `libtest` runs the tests of one binary on parallel threads, and the
/// connection record is a **monotonic counter read before and after** the
/// invocation under test. Two brackets that overlap measure each other's
/// connections, so every body that reaches a server — including one that only
/// opens a socket — holds this guard for its whole duration.
///
/// A poisoned lock is taken anyway: the panic that poisoned it has already
/// failed its own test, and the counter it left behind is not shared state a
/// later bracket depends on.
pub fn exclusive() -> MutexGuard<'static, ()> {
    static LOCK: Mutex<()> = Mutex::new(());

    LOCK.lock().unwrap_or_else(|poisoned| poisoned.into_inner())
}

/// Writes `text` where a plain `cargo test` run shows it.
///
/// `libtest` captures `print!` and `eprint!` and prints neither for a test that
/// passes, so a skip reported through them is invisible unless the run is made
/// with `--nocapture` — and a skip a caller cannot see is a skip a caller reads
/// as a pass, which is the failure the three-valued gate exists to prevent.
///
/// `/dev/stderr` is a second handle on the process's own file descriptor `2`,
/// which the capture does not reach, so a line written through it appears in
/// the ordinary output of `cargo test`. The handle is opened for **append**, so
/// a run whose stderr is a file is not rewound. Where it cannot be opened at
/// all the notice falls back to `eprint!`, which is captured but is better than
/// silence.
pub fn notice(text: &str) {
    let line = format!("tpl tests: {text}\n");

    match std::fs::OpenOptions::new().append(true).open("/dev/stderr") {
        // One `write_all` of one line, because the tests that call this run on
        // parallel threads and a line assembled in pieces would interleave.
        Ok(mut stderr) => {
            let _ = stderr.write_all(line.as_bytes());
        }
        // The one legitimate print in this repository: it is the fallback of
        // a notice that must reach a plain `cargo test` run, and it is test
        // support rather than a path the binary can reach.
        #[allow(
            clippy::print_stderr,
            reason = "the fallback of a skip notice, which is worse than silence only if it is \
                      not written at all"
        )]
        Err(_) => eprint!("{line}"),
    }
}

/// A `.tpl/.cfg` naming one database entry that reaches `server` as `account`.
///
/// `entry` is the entry name and `schema` the server-side database it selects,
/// per `FR-CONF-041`. The transport is `disabled` because the five modes of
/// `FR-CONF-013` are exercised where the connection is made, and a body that
/// is about something else asks for the one mode that adds nothing to what is
/// under test.
///
/// The address is the harness's own answer, split here rather than written:
/// no port is written in Rust anywhere in this module, and this composes the
/// file from what `status.sh --export` printed.
pub fn configuration(server: &Server, entry: &str, schema: &str, account: (&str, &str)) -> String {
    let (host, port) = server
        .address()
        .rsplit_once(':')
        .expect("status.sh --export prints host:port");
    let (user, password) = account;

    format!(
        "[core]\ndatabase = \"{entry}\"\n\n\
         [database.{entry}]\nhost = \"{host}\"\nport = {port}\n\
         user = \"{user}\"\npassword = \"{password}\"\n\
         database = \"{schema}\"\ntls = \"disabled\"\n"
    )
}

// ------------------------------------------------------- the three instruments ---

/// The server's connection record: the count of connections it has accepted
/// since it started.
///
/// This is `observe.sh connections <server> --value`, which reads
/// `Connections` from `GLOBAL_STATUS`. The counter is monotonic, so it is read
/// as a difference and never as a level.
pub fn connections(server: &Server) -> i64 {
    let printed = observe(&["connections", server.name(), "--value"]);
    let value = String::from_utf8_lossy(&printed.stdout);

    value
        .trim()
        .parse()
        .unwrap_or_else(|failure| panic!("observe.sh connections printed {value:?}: {failure}"))
}

/// Runs `body` bracketed by two readings of the connection record, and returns
/// the connections attributable to it.
///
/// The closing reading opens a connection of its own, which is the
/// [`OBSERVER_BASELINE`] the difference is reduced by.
pub fn connections_attributable_to(server: &Server, body: impl FnOnce()) -> i64 {
    let before = connections(server);
    body();
    let after = connections(server);

    after - before - OBSERVER_BASELINE
}

/// The server's statement record, emptied and switched on.
///
/// This is `observe.sh statements on <server>`, which truncates the general log
/// before enabling it, so that a dump covers exactly the window under test.
pub fn statements_on(server: &Server) {
    observe(&["statements", "on", server.name()]);
}

/// The server's statement record, switched off.
pub fn statements_off(server: &Server) {
    observe(&["statements", "off", server.name()]);
}

/// How many statements the record holds, under `filters`.
///
/// This is `observe.sh statements dump <server> [filters] --count`. With no
/// filter the dump drops the connections made over the container's Unix socket,
/// which is how the observer itself reaches the server and is not how anything
/// under test reaches it; `--all` keeps them, and is what makes the instrument
/// its own control.
pub fn statements_count(server: &Server, filters: &[&str]) -> i64 {
    let mut arguments = vec!["statements", "dump", server.name()];
    arguments.extend_from_slice(filters);
    arguments.push("--count");

    let printed = observe(&arguments);
    let value = String::from_utf8_lossy(&printed.stdout);

    value
        .trim()
        .parse()
        .unwrap_or_else(|failure| panic!("observe.sh statements dump printed {value:?}: {failure}"))
}

/// The statements the record holds, as text, under `filters`.
///
/// This is `observe.sh statements dump <server> [filters]` without `--count`:
/// one line per statement, carrying the thread, the command type and the
/// statement itself with its line breaks flattened. It is what a test reads
/// when the **content** of what the server received is the subject —
/// `FR-SCH-013`, which forbids a pattern to be sent at all, is the case the
/// count cannot answer.
pub fn statements_text(server: &Server, filters: &[&str]) -> String {
    let mut arguments = vec!["statements", "dump", server.name()];
    arguments.extend_from_slice(filters);

    String::from_utf8_lossy(&observe(&arguments).stdout).into_owned()
}

/// The syscall trace of a process: the files it opens and the sockets it
/// connects.
///
/// This is `observe.sh opens --backend strace -- <command…>`. The backend is
/// named rather than left at `auto`, which falls back to tracing a **Linux
/// build inside a container** on a host that has no `strace`:
/// `NFR-PERF-005` forbids crediting that observation to either Darwin target,
/// because the artefact observed would not be the artefact distributed, so a
/// fallback that happened silently would produce a trace no caller may use.
///
/// Returns what the trace carried, or `None` when the host has no `strace` and
/// the instrument therefore does not exist here.
pub fn opens(command: &[&str]) -> Option<String> {
    if Command::new("strace").arg("-V").output().is_err() {
        return None;
    }

    let mut arguments = vec!["opens", "--backend", "strace", "--"];
    arguments.extend_from_slice(command);

    let printed = observe(&arguments);

    Some(String::from_utf8_lossy(&printed.stdout).into_owned())
}

/// Opens one TCP connection to `server`, reads the byte its greeting begins
/// with, and closes.
///
/// This is the control for the connection record: an assertion that a command
/// opened no connection is worth nothing until the same instrument is shown to
/// count one that was opened. It is the shape of the gate's own probe, which
/// `scripts/mariadb/README.md` records as raising `Connections` by one per run.
pub fn connect_once(server: &Server) {
    let mut socket = TcpStream::connect_timeout(&server.socket_address(), PROBE_TIMEOUT)
        .unwrap_or_else(|failure| {
            panic!("{} did not accept a connection: {failure}", server.name())
        });

    socket
        .set_read_timeout(Some(PROBE_TIMEOUT))
        .expect("a read timeout is set on a connected socket");

    // A server that accepts and then says nothing is a half-started container,
    // and must not block the control. One byte of the greeting is enough to
    // establish that the connection was accepted and handled.
    let mut first = [0u8; 1];
    let _ = socket.read(&mut first);
}

// ------------------------------------------------------------------ the harness ---

/// The harness directory, which is the only way a test reaches the fixture.
fn harness() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("scripts/mariadb")
}

/// Runs a harness script from the harness directory.
fn script(name: &str, arguments: &[&str]) -> Output {
    Command::new(harness().join(name))
        .args(arguments)
        .current_dir(harness())
        .output()
        .unwrap_or_else(|failure| panic!("scripts/mariadb/{name} did not run: {failure}"))
}

/// Runs `observe.sh` and refuses anything but success: an instrument that
/// failed reports nothing, and reading its silence as an absence is the error
/// the whole of `NFR-PERF-007` exists to prevent.
fn observe(arguments: &[&str]) -> Output {
    let printed = script("observe.sh", arguments);

    assert!(
        printed.status.success(),
        "observe.sh {} exited {:?}: {}",
        arguments.join(" "),
        printed.status.code(),
        String::from_utf8_lossy(&printed.stderr)
    );

    printed
}

/// Asks the gate, and reads the inventory when the gate says there is one.
fn probe() -> State {
    let gate = gate_of(script("status.sh", &["--quiet"]).status.code());

    if gate != Gate::Ready {
        return State {
            gate,
            series: Vec::new(),
            answered: 0,
        };
    }

    let exported = script("status.sh", &["--export"]);
    let inventory = parse_export(&String::from_utf8_lossy(&exported.stdout));

    // `TPL_MARIADB_READY` carries the gate as data. It is read back against the
    // exit code, because the two are written by the same script from the same
    // two counters and a disagreement between them would mean the inventory
    // below describes a fixture that is not the one the gate admitted.
    assert_eq!(
        inventory.get("TPL_MARIADB_READY").map(String::as_str),
        Some("all"),
        "status.sh --quiet exited 0 while --export reported {:?}",
        inventory.get("TPL_MARIADB_READY")
    );

    let series = series_names()
        .into_iter()
        .map(|name| {
            let variable = exported_variable(&name);
            let address = inventory.get(&variable).unwrap_or_else(|| {
                panic!("status.sh --quiet exited 0 while --export named no {variable}")
            });

            Server {
                name,
                address: address.clone(),
            }
        })
        .collect();

    // Every assignment but the one that carries the gate as data is a server
    // that answered.
    let answered = inventory.len() - 1;

    State {
        gate,
        series,
        answered,
    }
}

/// The names of the four series of `FR-SRV-015`, from `series.env`.
fn series_names() -> Vec<String> {
    let printed = Command::new("bash")
        .arg("-c")
        .arg(SERIES_SCRIPT)
        .current_dir(harness())
        .output()
        .unwrap_or_else(|failure| {
            panic!("scripts/mariadb/series.env could not be read: {failure}")
        });

    assert!(
        printed.status.success(),
        "reading scripts/mariadb/series.env exited {:?}: {}",
        printed.status.code(),
        String::from_utf8_lossy(&printed.stderr)
    );

    let names: Vec<String> = String::from_utf8_lossy(&printed.stdout)
        .lines()
        .map(str::trim)
        .filter(|name| !name.is_empty())
        .map(str::to_owned)
        .collect();

    assert!(
        !names.is_empty(),
        "scripts/mariadb/series.env listed no TLS server"
    );

    names
}

/// Reads the shell assignments `status.sh --export` writes.
///
/// They are **parsed, not `eval`ed**: a test runner that evaluated them would
/// execute whatever the script emitted, and would read a fixture's inventory by
/// running it.
fn parse_export(printed: &str) -> BTreeMap<String, String> {
    printed
        .lines()
        .filter_map(|line| {
            // `NAME=value; export NAME` — the assignment is what precedes the
            // first `;`, and the export that follows it is for a shell.
            let assignment = line.split(';').next()?;
            let (name, value) = assignment.split_once('=')?;

            Some((name.trim().to_owned(), value.trim().to_owned()))
        })
        .collect()
}

/// The variable `status.sh --export` writes for a server of this name.
///
/// The transform is `status.sh`'s own — `tr 'a-z.' 'A-Z_'` over the name,
/// behind the shared prefix — applied here in the same direction. Nothing of
/// the inventory is reproduced: the name arrives from `series.env` and the
/// address from the export.
fn exported_variable(name: &str) -> String {
    format!(
        "TPL_MARIADB_{}",
        name.to_ascii_uppercase().replace('.', "_")
    )
}
