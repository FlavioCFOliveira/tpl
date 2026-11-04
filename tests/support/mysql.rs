//! The MySQL fixture of `scripts/mysql/`, as a test reaches it.
//!
//! It is `fixture.rs` for the `mysql` provider: the **gate** that answers
//! whether the fixture is up, the **inventory** that says which servers
//! answered, where and in which role, the two **server-side instruments** of
//! `NFR-PERF-007` that `FR-MYSQL-044` obliges the fixture to provide, and the
//! **authentication cache** `FR-MYSQL-045` depends on. It is reached by
//! `#[path]` for the reason `fixture.rs` records, and allows dead code for the
//! same reason: more than one test binary includes it, and each uses its part.
//!
//! # The fixture is asked, never restated
//!
//! No port, no server name, no credential and no schema name is written in
//! Rust:
//!
//! | Question | Asked of |
//! |---|---|
//! | Is the fixture up? | `status.sh --quiet`, whose exit code is the gate |
//! | Where did each server answer? | `status.sh --export`, **parsed, never `eval`ed** |
//! | Which server has which role, and which series? | `series.env`, through its own `tpl_mysql_each` iterator and its filters |
//! | The accounts and the schemas | `series.env`, its own variables |
//!
//! **No `docker` command is issued from Rust.** Every fixture operation goes
//! through `status.sh`, `observe.sh`, `auth-cache.sh` and `series.env`.

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

/// How long a probe of a fixture port may take.
const PROBE_TIMEOUT: Duration = Duration::from_secs(5);

/// The connections the observer's own closing reading accounts for, measured
/// on all five servers and recorded in `scripts/mysql/README.md`.
const OBSERVER_BASELINE: i64 = 1;

/// The role a server plays in the fixture, as `series.env` filters them.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Role {
    /// A server of a series of `FR-MYSQL-003`, presenting the fixture
    /// certificate: the filter `tls`.
    Series,
    /// A server of a supported series offering no TLS: the filter `notls`.
    NoTls,
    /// The server of a series outside `FR-MYSQL-003` and older than its
    /// newest: the filter `refused`.
    Refused,
}

impl Role {
    /// The filter of `tpl_mysql_each` that selects this role.
    const fn filter(self) -> &'static str {
        match self {
            Self::Series => "tls",
            Self::NoTls => "notls",
            Self::Refused => "refused",
        }
    }
}

/// What `status.sh` answered, as the three-valued gate its header defines.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Gate {
    /// Exit `0`: every server answered.
    Ready,
    /// Exit `1`: no server answered. Skip, and say why.
    Absent,
    /// Exit `2`, or anything else: a broken fixture, never skipped over.
    Broken(Option<i32>),
}

/// Maps the gate's exit code, and nothing else.
pub fn gate_of(code: Option<i32>) -> Gate {
    match code {
        Some(0) => Gate::Ready,
        Some(1) => Gate::Absent,
        other => Gate::Broken(other),
    }
}

/// One server of the fixture.
#[derive(Debug, Clone)]
pub struct Server {
    /// The name `series.env` gives it.
    name: String,
    /// The series it runs, as `series.env` states it.
    series: String,
    /// `host:port`, exactly as `status.sh --export` printed it.
    address: String,
}

impl Server {
    /// The name the harness knows it by.
    pub fn name(&self) -> &str {
        &self.name
    }

    /// The series it runs.
    pub fn series(&self) -> &str {
        &self.series
    }

    /// The address it answered on.
    pub fn address(&self) -> &str {
        &self.address
    }

    /// The host part of the address.
    pub fn host(&self) -> &str {
        self.address
            .rsplit_once(':')
            .map_or(self.address.as_str(), |(host, _)| host)
    }

    /// The port part of the address.
    pub fn port(&self) -> &str {
        self.address.rsplit_once(':').map_or("", |(_, port)| port)
    }

    /// The address, resolved.
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

/// The accounts and schemas the fixture carries, as `series.env` states them.
#[derive(Debug, Clone)]
pub struct Names {
    /// The privileged account: user and password.
    pub root: (String, String),
    /// The reduced-grant reader of item 16 of `FR-MYSQL-044`.
    pub reader: (String, String),
    /// The main schema.
    pub schema: String,
    /// The schema that holds no object.
    pub empty_schema: String,
}

/// What one ask of the gate and the inventory established.
#[derive(Debug)]
struct State {
    /// What `status.sh --quiet` answered.
    gate: Gate,
    /// Every server that answered, by role, in the order `series.env` lists
    /// them.
    servers: Vec<(Role, Server)>,
    /// The accounts and schemas.
    names: Option<Names>,
}

/// Asks the gate and reads the inventory, once per test binary.
fn state() -> &'static State {
    static STATE: OnceLock<State> = OnceLock::new();

    STATE.get_or_init(probe)
}

/// The servers of `role`, or `None` when the fixture is absent.
///
/// A broken fixture **panics** rather than skipping, for the reason
/// `fixture.rs` records.
pub fn servers(test: &str, role: Role) -> Option<Vec<Server>> {
    let state = state();

    match state.gate {
        Gate::Ready => Some(
            state
                .servers
                .iter()
                .filter(|(of, _)| *of == role)
                .map(|(_, server)| server.clone())
                .collect(),
        ),
        Gate::Absent => {
            notice(&format!(
                "skipped {test}: the MySQL fixture is not up \
                 (scripts/mysql/status.sh --quiet exited 1: no server answered). \
                 Start it with ./scripts/mysql/up.sh."
            ));
            None
        }
        Gate::Broken(code) => panic!(
            "the MySQL fixture is broken, not absent: scripts/mysql/status.sh --quiet {}. \
             Run ./scripts/mysql/status.sh to see which servers answered, then \
             ./scripts/mysql/down.sh and ./scripts/mysql/up.sh.",
            match code {
                Some(code) => format!("exited {code}"),
                None => "was signalled".to_owned(),
            }
        ),
    }
}

/// The servers of the series of `FR-MYSQL-003`, one each, or `None` when the
/// fixture is absent.
pub fn series(test: &str) -> Option<Vec<Server>> {
    servers(test, Role::Series)
}

/// The accounts and schemas of the fixture, once the gate has admitted it.
pub fn names() -> &'static Names {
    state()
        .names
        .as_ref()
        .expect("the gate admitted the fixture before its names were asked")
}

/// Serialises every test that touches a server of this fixture, for the
/// reason `fixture::exclusive` gives: the connection record is a monotonic
/// counter read before and after, and the statement record is emptied and
/// read per window.
pub fn exclusive() -> MutexGuard<'static, ()> {
    static LOCK: Mutex<()> = Mutex::new(());

    LOCK.lock().unwrap_or_else(|poisoned| poisoned.into_inner())
}

/// Writes `text` where a plain `cargo test` run shows it, past the capture,
/// as `fixture::notice` does.
pub fn notice(text: &str) {
    let line = format!("tpl tests: {text}\n");

    match std::fs::OpenOptions::new().append(true).open("/dev/stderr") {
        Ok(mut stderr) => {
            let _ = stderr.write_all(line.as_bytes());
        }
        #[allow(
            clippy::print_stderr,
            reason = "the fallback of a skip notice, which is worse than silence only if it is \
                      not written at all"
        )]
        Err(_) => eprint!("{line}"),
    }
}

/// A `.tpl/.cfg` naming one `mysql` entry that reaches `server` as `account`,
/// selecting `schema`, under the TLS mode `tls`, with `extra` appended to the
/// entry.
///
/// Under `verify-ca` and `verify-identity` the entry names the fixture's own
/// root certificate as `ca_file`, the file `scripts/mysql/README.md` names for
/// it.
pub fn configuration(
    server: &Server,
    entry: &str,
    schema: &str,
    account: (&str, &str),
    tls: &str,
) -> String {
    let (user, password) = account;
    let trust = if matches!(tls, "verify-ca" | "verify-identity") {
        format!("ca_file = \"{}\"\n", harness().join("tls/ca.pem").display())
    } else {
        String::new()
    };

    format!(
        "[core]\ndatabase = \"{entry}\"\n\n\
         [database.{entry}]\nprovider = \"mysql\"\nhost = \"{}\"\nport = {}\n\
         user = \"{user}\"\npassword = \"{password}\"\n\
         database = \"{schema}\"\ntls = \"{tls}\"\n{trust}",
        server.host(),
        server.port(),
    )
}

// ------------------------------------------------------- the instruments ---

/// The server's connection record (`observe.sh connections <server> --value`).
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
pub fn connections_attributable_to(server: &Server, body: impl FnOnce()) -> i64 {
    let before = connections(server);
    body();
    let after = connections(server);

    after - before - OBSERVER_BASELINE
}

/// The statement record, emptied and switched on.
pub fn statements_on(server: &Server) {
    observe(&["statements", "on", server.name()]);
}

/// The statement record, switched off.
pub fn statements_off(server: &Server) {
    observe(&["statements", "off", server.name()]);
}

/// How many statements the record holds, under `filters`.
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

/// The statements the record holds, as text, under `filters`: one line per
/// statement, with a header line, carrying the thread, the command type and
/// the statement.
pub fn statements_text(server: &Server, filters: &[&str]) -> String {
    let mut arguments = vec!["statements", "dump", server.name()];
    arguments.extend_from_slice(filters);

    String::from_utf8_lossy(&observe(&arguments).stdout).into_owned()
}

/// The command type and the statement of each row the record holds for
/// `user`, in the order the server received them.
pub fn received(server: &Server, user: &str) -> Vec<(String, String)> {
    statements_text(server, &["--user", user])
        .lines()
        .skip(1)
        .filter_map(|row| {
            let mut fields = row.splitn(3, '\t');
            let _thread = fields.next()?;
            let kind = fields.next()?.to_owned();
            let statement = fields.next().unwrap_or_default().trim().to_owned();

            Some((kind, statement))
        })
        .collect()
}

/// Empties the server's `caching_sha2_password` cache
/// (`auth-cache.sh flush <server>`), which `FR-MYSQL-045` depends on.
pub fn flush_authentication_cache(server: &Server) {
    run_checked("auth-cache.sh", &["flush", server.name()]);
}

/// Fills the server's `caching_sha2_password` cache for `user` by one login
/// (`auth-cache.sh prime <server> <user>`), so that a login over a session TLS
/// does not protect succeeds without the server's public key.
pub fn prime_authentication_cache(server: &Server, user: &str) {
    run_checked("auth-cache.sh", &["prime", server.name(), user]);
}

/// Opens one TCP connection to `server`, reads the first byte of its greeting,
/// and closes: the control of the connection record.
pub fn connect_once(server: &Server) {
    let mut socket = TcpStream::connect_timeout(&server.socket_address(), PROBE_TIMEOUT)
        .unwrap_or_else(|failure| {
            panic!("{} did not accept a connection: {failure}", server.name())
        });

    socket
        .set_read_timeout(Some(PROBE_TIMEOUT))
        .expect("a read timeout is set on a connected socket");

    let mut first = [0u8; 1];
    let _ = socket.read(&mut first);
}

// ------------------------------------------------------------ the harness ---

/// The harness directory.
fn harness() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("scripts/mysql")
}

/// Runs a harness script from the harness directory.
fn script(name: &str, arguments: &[&str]) -> Output {
    Command::new(harness().join(name))
        .args(arguments)
        .current_dir(harness())
        .output()
        .unwrap_or_else(|failure| panic!("scripts/mysql/{name} did not run: {failure}"))
}

/// Runs a harness script and refuses anything but success.
fn run_checked(name: &str, arguments: &[&str]) -> Output {
    let printed = script(name, arguments);

    assert!(
        printed.status.success(),
        "scripts/mysql/{name} {} exited {:?}: {}",
        arguments.join(" "),
        printed.status.code(),
        String::from_utf8_lossy(&printed.stderr)
    );

    printed
}

/// Runs `observe.sh`, refusing anything but success.
fn observe(arguments: &[&str]) -> Output {
    run_checked("observe.sh", arguments)
}

/// Runs `script` under `bash` with `series.env` sourced, and returns its
/// standard output.
fn sourced(script: &str) -> String {
    let printed = Command::new("bash")
        .arg("-c")
        .arg(format!("set -eu; . ./series.env; {script}"))
        .current_dir(harness())
        .output()
        .unwrap_or_else(|failure| panic!("scripts/mysql/series.env could not be read: {failure}"));

    assert!(
        printed.status.success(),
        "reading scripts/mysql/series.env exited {:?}: {}",
        printed.status.code(),
        String::from_utf8_lossy(&printed.stderr)
    );

    String::from_utf8_lossy(&printed.stdout).into_owned()
}

/// The name and series of each server of `role`, from `series.env`'s own
/// iterator.
fn role_names(role: Role) -> Vec<(String, String)> {
    sourced(&format!(
        "emit() {{ printf '%s %s\\n' \"$1\" \"$2\"; }}; tpl_mysql_each emit {}",
        role.filter()
    ))
    .lines()
    .filter_map(|line| {
        let (name, series) = line.trim().split_once(' ')?;
        Some((name.to_owned(), series.to_owned()))
    })
    .collect()
}

/// The accounts and schemas, from `series.env`'s own variables.
fn read_names() -> Names {
    let printed = sourced(
        "printf '%s\\n' \"$TPL_MYSQL_ROOT_PASSWORD\" \"$TPL_MYSQL_READER_USER\" \
         \"$TPL_MYSQL_READER_PASSWORD\" \"$TPL_MYSQL_SCHEMA\" \"$TPL_MYSQL_EMPTY_SCHEMA\"",
    );
    let lines: Vec<String> = printed.lines().map(str::to_owned).collect();
    let [root_password, reader, reader_password, schema, empty_schema] = lines.as_slice() else {
        panic!(
            "series.env stated {} values, expected 5: {printed:?}",
            lines.len()
        );
    };

    Names {
        root: ("root".to_owned(), root_password.clone()),
        reader: (reader.clone(), reader_password.clone()),
        schema: schema.clone(),
        empty_schema: empty_schema.clone(),
    }
}

/// Asks the gate, and reads the inventory when the gate says there is one.
fn probe() -> State {
    let gate = gate_of(script("status.sh", &["--quiet"]).status.code());

    if gate != Gate::Ready {
        return State {
            gate,
            servers: Vec::new(),
            names: None,
        };
    }

    let exported = script("status.sh", &["--export"]);
    let inventory = parse_export(&String::from_utf8_lossy(&exported.stdout));

    assert_eq!(
        inventory.get("TPL_MYSQL_READY").map(String::as_str),
        Some("all"),
        "status.sh --quiet exited 0 while --export reported {:?}",
        inventory.get("TPL_MYSQL_READY")
    );

    let mut servers = Vec::new();
    for role in [Role::Series, Role::NoTls, Role::Refused] {
        for (name, series) in role_names(role) {
            let variable = exported_variable(&name);
            let address = inventory.get(&variable).unwrap_or_else(|| {
                panic!("status.sh --quiet exited 0 while --export named no {variable}")
            });

            servers.push((
                role,
                Server {
                    name,
                    series,
                    address: address.clone(),
                },
            ));
        }
    }

    State {
        gate,
        servers,
        names: Some(read_names()),
    }
}

/// Reads the shell assignments `status.sh --export` writes, without
/// evaluating them.
fn parse_export(printed: &str) -> BTreeMap<String, String> {
    printed
        .lines()
        .filter_map(|line| {
            let assignment = line.split(';').next()?;
            let (name, value) = assignment.split_once('=')?;

            Some((name.trim().to_owned(), value.trim().to_owned()))
        })
        .collect()
}

/// The variable `status.sh --export` writes for a server of this name: upper
/// case, every dot and hyphen an underscore, behind the shared prefix.
fn exported_variable(name: &str) -> String {
    format!(
        "TPL_MYSQL_{}",
        name.to_ascii_uppercase().replace(['.', '-'], "_")
    )
}
