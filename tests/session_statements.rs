//! The statements of an open session, observed from outside the process
//! against a scripted server on the loopback interface.
//!
//! | Requirement | What is observed |
//! |---|---|
//! | `FR-CONF-005`, `FR-ERR-027`, `FR-SRV-010` | A session statement that does not answer ends at its own deadline of `core.query_timeout`, at `69`, named as the statement it is — the version probe, the read-only session statement — never as a catalogue query and never as `78` |
//! | `FR-CFG-044`, `FR-ERR-036` | The privilege probe answered with an error packet carrying `1142` is its `false` at `0`; one carrying `1105` is not, and exits `69` naming the number |
//! | `FR-GLOB-017`, `NFR-PERF-008` | At `-v`, one line per connection phase that ran and per session statement, each naming it; no line for the TLS handshake a `disabled` entry does not negotiate; and the catalogue-query lines unchanged in number |
//!
//! The server is scripted and not a database: it speaks exactly the part of
//! the client-server protocol these bodies reach — a greeting naming
//! `mysql_native_password` with neither TLS nor `CLIENT_DEPRECATE_EOF`, an OK
//! packet for any login, and per statement the answer the body scripts — so
//! that the statement a body is about is the one that stalls or fails, and
//! the outcome does not depend on a fixture. Both providers are run, because
//! a `mariadb` and a `mysql` entry issue the same three session statements
//! (`FR-SRV-042`, `FR-MYSQL-017`).

#[path = "support/sandbox.rs"]
mod sandbox;

use std::io::{Read as _, Write as _};
use std::net::{TcpListener, TcpStream};
use std::thread::{self, JoinHandle};

use sandbox::Sandbox;

/// The entry every body names.
const ENTRY: &str = "shop";

/// How a scripted statement is answered.
#[derive(Debug, Clone, Copy)]
enum Answer {
    /// An OK packet.
    Ok,
    /// A one-row, one-column result whose column is an integer.
    Integer(&'static str),
    /// A one-row, one-column result whose column is text.
    Text(&'static str),
    /// An error packet carrying this number.
    Error(u16),
    /// Nothing, until the client closes the connection.
    Stall,
}

/// The answers of one connection, by the statement they answer.
#[derive(Debug, Clone, Copy)]
struct Script {
    /// The read-only session statement.
    read_only: Answer,
    /// Its read-back.
    read_back: Answer,
    /// The version probe.
    probe: Answer,
    /// Any statement prepared after them: the privilege probe of
    /// `FR-CFG-044` and the catalogue statements, which travel as prepared
    /// statements because they carry a bind parameter.
    prepared: Answer,
}

impl Script {
    /// A session that opens and settles for `provider`, whose prepared
    /// statements are answered with `prepared`.
    const fn settled(provider: &str, prepared: Answer) -> Self {
        Self {
            read_only: Answer::Ok,
            read_back: Answer::Integer("1"),
            probe: Answer::Text(version(provider)),
            prepared,
        }
    }
}

/// A version the window of `provider` supports.
const fn version(provider: &str) -> &'static str {
    match provider.as_bytes() {
        b"mariadb" => "11.4.13-MariaDB",
        _ => "8.4.13",
    }
}

/// The greeting the scripted server writes.
///
/// Protocol version `10`; a server version; a connection id; the first eight
/// bytes of the scramble; the filler; the lower capability flags, `0xa30f`:
/// `CLIENT_LONG_PASSWORD`, `CLIENT_FOUND_ROWS`, `CLIENT_LONG_FLAG`,
/// `CLIENT_CONNECT_WITH_DB`, `CLIENT_IGNORE_SPACE`, `CLIENT_PROTOCOL_41`,
/// `CLIENT_TRANSACTIONS` and `CLIENT_SECURE_CONNECTION`, and no `CLIENT_SSL`;
/// the collation; the status flags; the upper capability flags, `0x002f`:
/// `CLIENT_MULTI_STATEMENTS`, `CLIENT_MULTI_RESULTS`,
/// `CLIENT_PS_MULTI_RESULTS`, `CLIENT_PLUGIN_AUTH` and
/// `CLIENT_PLUGIN_AUTH_LENENC_CLIENT_DATA`, and no `CLIENT_DEPRECATE_EOF`; the
/// scramble's length, `21`; ten reserved bytes; the scramble's last twelve
/// bytes and their NUL; the authentication method.
const GREETING: &[u8] = b"\x0a\
    8.4.13\x00\
    \x07\x00\x00\x00\
    abcdefgh\
    \x00\
    \x0f\xa3\
    \x21\
    \x02\x00\
    \x2f\x00\
    \x15\
    \x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\
    ijklmnopqrst\x00\
    mysql_native_password\x00";

/// An OK packet: no rows affected, no insert id, autocommit, no warnings.
const OK: &[u8] = b"\x00\x00\x00\x02\x00\x00\x00";

/// An EOF packet: no warnings, autocommit.
const EOF: &[u8] = b"\xfe\x00\x00\x02\x00";

/// `value` as a length-encoded string, for a value shorter than 251 bytes.
fn lenenc(value: &[u8]) -> Vec<u8> {
    let mut encoded = vec![u8::try_from(value.len()).expect("a scripted value is short")];
    encoded.extend_from_slice(value);

    encoded
}

/// A column definition named `v`: an integer column, or a text one.
fn column(integer: bool) -> Vec<u8> {
    let mut definition = Vec::new();
    for part in [&b"def"[..], b"", b"", b"", b"v", b""] {
        definition.extend(lenenc(part));
    }
    definition.push(0x0c);
    if integer {
        // binary collation, length 1, LONGLONG, NOT_NULL | BINARY, 0 decimals
        definition.extend_from_slice(b"\x3f\x00\x01\x00\x00\x00\x08\x81\x00\x00");
    } else {
        // utf8 collation, length 765, VAR_STRING, no flags, 31 decimals
        definition.extend_from_slice(b"\x21\x00\xfd\x02\x00\x00\xfd\x00\x00\x1f");
    }
    definition.extend_from_slice(b"\x00\x00");

    definition
}

/// An error packet carrying `number`, with a SQL state and a message.
fn error(number: u16) -> Vec<u8> {
    let mut packet = vec![0xff];
    packet.extend_from_slice(&number.to_le_bytes());
    packet.extend_from_slice(b"#HY000scripted error");

    packet
}

/// Writes one packet carrying `payload` under sequence id `sequence`.
fn write(connection: &mut TcpStream, sequence: u8, payload: &[u8]) -> std::io::Result<()> {
    let length = u32::try_from(payload.len()).expect("a scripted packet is short");
    let mut packet = length.to_le_bytes().to_vec();
    packet[3] = sequence;
    packet.extend_from_slice(payload);

    connection.write_all(&packet)
}

/// Reads one packet's payload, or [`None`] once the client has closed.
fn read(connection: &mut TcpStream) -> Option<Vec<u8>> {
    let mut header = [0_u8; 4];
    connection.read_exact(&mut header).ok()?;
    let length =
        usize::from(header[0]) | usize::from(header[1]) << 8 | usize::from(header[2]) << 16;
    let mut payload = vec![0_u8; length];
    connection.read_exact(&mut payload).ok()?;

    Some(payload)
}

/// Writes `answer` to the command just read, or stalls; [`None`] once the
/// client has closed.
fn answer(connection: &mut TcpStream, answer: Answer) -> Option<()> {
    let written = match answer {
        Answer::Ok => write(connection, 1, OK),
        Answer::Error(number) => write(connection, 1, &error(number)),
        Answer::Integer(value) | Answer::Text(value) => {
            let integer = matches!(answer, Answer::Integer(_));
            write(connection, 1, b"\x01")
                .and_then(|()| write(connection, 2, &column(integer)))
                .and_then(|()| write(connection, 3, EOF))
                .and_then(|()| write(connection, 4, &lenenc(value.as_bytes())))
                .and_then(|()| write(connection, 5, EOF))
        }
        Answer::Stall => {
            // The client's deadline ends the statement; the connection is
            // then closed, which ends this read.
            while read(connection).is_some() {}
            return None;
        }
    };

    written.ok()
}

/// A scripted server on a port of the loopback interface that accepts one
/// connection and answers it as `script` says.
fn server(script: Script) -> (u16, JoinHandle<()>) {
    let bound = TcpListener::bind("127.0.0.1:0").expect("the loopback interface takes a listener");
    let port = bound
        .local_addr()
        .expect("a bound listener has an address")
        .port();

    let serving = thread::spawn(move || {
        let (mut connection, _) = bound.accept().expect("tpl connects once");
        if write(&mut connection, 0, GREETING).is_err() || read(&mut connection).is_none() {
            return;
        }
        if write(&mut connection, 2, OK).is_err() {
            return;
        }

        while let Some(command) = read(&mut connection) {
            let text = String::from_utf8_lossy(command.get(1..).unwrap_or_default());
            let scripted = match command.first() {
                Some(0x03) if text.starts_with("SET SESSION TRANSACTION") => script.read_only,
                Some(0x03) if text.contains("read_only") => script.read_back,
                Some(0x03) if text.contains("VERSION()") => script.probe,
                Some(0x16) => script.prepared,
                // COM_QUIT, or anything this server does not script.
                _ => return,
            };
            if answer(&mut connection, scripted).is_none() {
                return;
            }
        }
    });

    (port, serving)
}

/// An entry of `provider` reaching the scripted server at `port`, whose
/// statements take a deadline of one second.
fn local_entry(provider: &str, port: u16) -> String {
    format!(
        "[core]\ndatabase = \"{ENTRY}\"\nquery_timeout = 1\n\n\
         [database.{ENTRY}]\nprovider = \"{provider}\"\nhost = \"127.0.0.1\"\nport = {port}\n\
         user = \"reader\"\npassword = \"irrelevant\"\n\
         database = \"freight\"\ntls = \"disabled\"\n"
    )
}

/// What one run produced.
struct Outcome {
    /// The exit code.
    code: Option<i32>,
    /// Standard output.
    out: String,
    /// Standard error.
    err: String,
}

/// Runs `arguments` for an entry of `provider` against a scripted server.
fn against(provider: &str, script: Script, arguments: &[&str]) -> (u16, Outcome) {
    let (port, serving) = server(script);
    let sandbox = Sandbox::new();
    sandbox.project(&local_entry(provider, port));
    let printed = sandbox.run(arguments);
    serving.join().expect("the server ran");

    (
        port,
        Outcome {
            code: printed.status.code(),
            out: String::from_utf8_lossy(&printed.stdout).into_owned(),
            err: String::from_utf8_lossy(&printed.stderr).into_owned(),
        },
    )
}

/// The four lines of an expired statement deadline of one second.
fn statement_expired(statement: &str, port: u16) -> String {
    format!(
        "error: {statement} for 127.0.0.1:{port} exceeded its own deadline of 1s, for database \
         entry '{ENTRY}'\n\
         cause: {statement} for 127.0.0.1:{port} had not finished when its own deadline of 1s \
         expired; core.query_timeout sets that deadline, which bounds each statement on its own\n\
         hint:  raise the deadline with: tpl cfg set core.query_timeout <seconds>\n\
         exit:  69 (EX_UNAVAILABLE)\n"
    )
}

/// The command every body that reads runs: one connection, no cache.
const READ: [&str; 4] = ["schema", "tables", "--direct", "--no-cache"];

// -------------------------------------------------------------- FR-ERR-027 ---

#[test]
fn fr_err_027_a_version_probe_that_does_not_answer_is_reported_as_the_version_probe() {
    // FR-CONF-005 and FR-ERR-027, seventy-ninth edition: the version probe
    // has its own deadline of core.query_timeout, and its expiry names it —
    // not "a query reading the database structure", which was the line
    // before rmp #339.
    for provider in ["mariadb", "mysql"] {
        let script = Script {
            probe: Answer::Stall,
            ..Script::settled(provider, Answer::Error(1105))
        };
        let (port, outcome) = against(provider, script, &READ);

        assert_eq!(outcome.code, Some(69), "{provider}: {}", outcome.err);
        assert_eq!(
            outcome.err,
            statement_expired("the version probe", port),
            "{provider}"
        );
    }
}

#[test]
fn fr_srv_010_a_read_only_statement_that_does_not_answer_is_69_and_not_78() {
    // FR-SRV-010, note of the seventy-ninth edition: an expiry of either
    // statement of the read-only pair is the 69 of FR-ERR-027, named as the
    // statement that was running, and not the 78 of a setting not applied.
    for provider in ["mariadb", "mysql"] {
        let read_only = Script {
            read_only: Answer::Stall,
            ..Script::settled(provider, Answer::Error(1105))
        };
        let (port, outcome) = against(provider, read_only, &READ);
        assert_eq!(outcome.code, Some(69), "{provider}: {}", outcome.err);
        assert_eq!(
            outcome.err,
            statement_expired("the read-only session statement", port),
            "{provider}"
        );

        let read_back = Script {
            read_back: Answer::Stall,
            ..Script::settled(provider, Answer::Error(1105))
        };
        let (port, outcome) = against(provider, read_back, &READ);
        assert_eq!(outcome.code, Some(69), "{provider}: {}", outcome.err);
        assert_eq!(
            outcome.err,
            statement_expired("the read-back of the read-only session", port),
            "{provider}"
        );
    }
}

// -------------------------------------------------------------- FR-CFG-044 ---

#[test]
fn fr_cfg_044_another_error_packet_is_not_read_as_false() {
    // FR-CFG-044 and FR-ERR-036, seventy-ninth edition: 1105 is not one of
    // the four privilege errors, so the probe has not been answered: exit 69,
    // the third way of FR-ERR-036 naming the number, and no document.
    for provider in ["mariadb", "mysql"] {
        let (port, outcome) = against(
            provider,
            Script::settled(provider, Answer::Error(1105)),
            &["cfg", "database", "test", ENTRY, "--format", "json"],
        );

        assert_eq!(outcome.code, Some(69), "{provider}: {}", outcome.err);
        assert_eq!(outcome.out, "", "{provider}: no document is written");
        assert_eq!(
            outcome.err,
            format!(
                "error: a query reading the database structure from 127.0.0.1:{port} did not \
                 complete, for database entry '{ENTRY}'\n\
                 cause: the session with 127.0.0.1:{port} was open, and the server answered a \
                 query reading the database structure with server error 1105\n\
                 hint:  the operation is read-only and can be run again; check the connection \
                 with: tpl cfg database test {ENTRY}\n\
                 exit:  69 (EX_UNAVAILABLE)\n"
            ),
            "{provider}"
        );
    }
}

#[test]
fn fr_cfg_044_a_privilege_error_is_the_probe_false_at_zero() {
    // The control of the body above: the same exchange answered with 1142,
    // ER_TABLEACCESS_DENIED_ERROR, is the probe's false, exit 0.
    for provider in ["mariadb", "mysql"] {
        let (_, outcome) = against(
            provider,
            Script::settled(provider, Answer::Error(1142)),
            &["cfg", "database", "test", ENTRY, "--format", "json"],
        );

        assert_eq!(outcome.code, Some(0), "{provider}: {}", outcome.err);
        assert!(
            outcome.out.contains("\"can_read_catalogue\":false"),
            "{provider}: {}",
            outcome.out
        );
    }
}

// -------------------------------------------------------------- FR-GLOB-017 ---

/// The phase a `phase:` line names: what follows the token, up to ` took`.
fn phases(err: &str) -> Vec<&str> {
    err.lines()
        .filter_map(|line| line.strip_prefix("phase: "))
        .filter_map(|rest| rest.split_once(" took ").map(|(phase, _)| phase))
        .collect()
}

#[test]
fn fr_glob_017_each_connection_phase_and_session_statement_has_one_line_naming_it() {
    // FR-GLOB-017, seventy-ninth edition: DNS resolution, the TCP connect and
    // the login, each on a line of its own; no line for a TLS handshake the
    // `disabled` mode does not negotiate; one line for each session
    // statement, named; and the line of the catalogue query apart, so the
    // count of NFR-PERF-008 is one.
    for provider in ["mariadb", "mysql"] {
        let (_, outcome) = against(
            provider,
            Script::settled(provider, Answer::Error(1142)),
            &["-v", "cfg", "database", "test", ENTRY],
        );

        assert_eq!(outcome.code, Some(0), "{provider}: {}", outcome.err);
        assert_eq!(
            phases(&outcome.err),
            [
                "dns resolution",
                "tcp connect",
                "login",
                "read-only session statement",
                "read-back of the read-only session",
                "version probe",
                "server query",
            ],
            "{provider}: {}",
            outcome.err
        );
        assert_eq!(
            outcome
                .err
                .lines()
                .filter(|line| line.starts_with("query:"))
                .count(),
            1,
            "{provider}: NFR-PERF-008 counts the catalogue query alone: {}",
            outcome.err
        );
    }
}

#[test]
fn fr_glob_017_a_statement_that_expired_keeps_its_line_and_none_follows_it() {
    // FR-GLOB-017: a phase in which a deadline expired ran, and has its line;
    // a phase the connection did not reach has none.
    for provider in ["mariadb", "mysql"] {
        let script = Script {
            read_back: Answer::Stall,
            ..Script::settled(provider, Answer::Error(1142))
        };
        let (_, outcome) = against(
            provider,
            script,
            &["-v", "schema", "tables", "--direct", "--no-cache"],
        );

        assert_eq!(outcome.code, Some(69), "{provider}: {}", outcome.err);
        assert_eq!(
            phases(&outcome.err),
            [
                "dns resolution",
                "tcp connect",
                "login",
                "read-only session statement",
                "read-back of the read-only session",
            ],
            "{provider}: {}",
            outcome.err
        );
    }
}

#[test]
fn fr_glob_017_a_login_that_fails_keeps_its_line() {
    // FR-GLOB-017: the login fails — the server closes the connection after
    // the greeting — and its line is still written, after the TCP connect's.
    for provider in ["mariadb", "mysql"] {
        let bound =
            TcpListener::bind("127.0.0.1:0").expect("the loopback interface takes a listener");
        let port = bound
            .local_addr()
            .expect("a bound listener has an address")
            .port();
        let serving = thread::spawn(move || {
            let (mut connection, _) = bound.accept().expect("tpl connects once");
            let _ = write(&mut connection, 0, GREETING);
            let _ = read(&mut connection);
        });
        let sandbox = Sandbox::new();
        sandbox.project(&local_entry(provider, port));
        let printed = sandbox.run(&["-v", "schema", "tables", "--direct", "--no-cache"]);
        serving.join().expect("the server ran");
        let err = String::from_utf8_lossy(&printed.stderr);

        assert_eq!(printed.status.code(), Some(69), "{provider}: {err}");
        assert_eq!(
            phases(&err),
            ["dns resolution", "tcp connect", "login"],
            "{provider}: {err}"
        );
    }
}
