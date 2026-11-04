//! The connection of a server entry, from the TCP connect to the open session,
//! observed from outside the process.
//!
//! | Requirement | What is observed |
//! |---|---|
//! | `FR-CONF-055`, `ADR-014` point 5 | An account with no password logs in on every MariaDB series and every MySQL server, with `password = ""` and with no `password` key |
//! | `FR-ERR-049` | A refused TCP connect carries the first outcome, word for word, for both providers |
//! | `FR-ERR-047`, `FR-CONF-005`, `ADR-014` point 6 | A connection that closes in the login, and an answer that is not the protocol, are the login's first two outcomes |
//! | `FR-ERR-047` | A `mariadb` entry whose login needs the server's RSA public key ends at `69` with the facts of `FR-MYSQL-045` |
//!
//! The passwordless account is `tpl_nopass`, which both fixtures create with
//! no password and `SELECT, EXECUTE ON freight.*`, per their READMEs. The
//! bodies that reach a fixture server skip with their reason where that
//! fixture is not up; the bodies of `FR-ERR-049` and of the first two
//! outcomes of `FR-ERR-047` need no server: what answers at the address is
//! exactly what they are about, a closed port or a listener that is not a
//! database server.

#[path = "support/fixture.rs"]
mod mariadb;
#[path = "support/mysql.rs"]
mod mysql;
#[path = "support/sandbox.rs"]
mod sandbox;

use std::io::Write as _;
use std::net::TcpListener;
use std::thread::{self, JoinHandle};

use sandbox::Sandbox;

/// The entry every body names.
const ENTRY: &str = "shop";

/// The account both fixtures create with no password.
const NOPASS: &str = "tpl_nopass";

/// The schema both fixtures grant `NOPASS` on.
const SCHEMA: &str = "freight";

/// What one run produced.
struct Outcome {
    /// The exit code.
    code: Option<i32>,
    /// Standard error.
    err: String,
}

/// Runs `tpl schema info --direct --no-cache` against the project in
/// `sandbox`: one connection, no cache.
fn read(sandbox: &Sandbox) -> Outcome {
    let printed = sandbox.run(&["schema", "info", "--direct", "--no-cache"]);

    Outcome {
        code: printed.status.code(),
        err: String::from_utf8_lossy(&printed.stderr).into_owned(),
    }
}

/// A sandbox holding `configuration` as its project.
fn project(configuration: &str) -> Sandbox {
    let sandbox = Sandbox::new();
    sandbox.project(configuration);

    sandbox
}

/// `configuration` with its `password` key removed.
fn without_password(configuration: &str) -> String {
    let removed = configuration.replace("password = \"\"\n", "");
    assert_ne!(
        removed, configuration,
        "the configuration carried a password key"
    );

    removed
}

/// An entry of `provider` reaching `127.0.0.1:port`, as the reader account.
fn local_entry(provider: &str, port: u16) -> String {
    format!(
        "[core]\ndatabase = \"{ENTRY}\"\n\n\
         [database.{ENTRY}]\nprovider = \"{provider}\"\nhost = \"127.0.0.1\"\nport = {port}\n\
         user = \"reader\"\npassword = \"irrelevant\"\n\
         database = \"{SCHEMA}\"\ntls = \"disabled\"\n"
    )
}

/// A listener on a port of the loopback interface that accepts one
/// connection, writes `greeting` to it, and closes it.
///
/// It is not a database server, and stands in for none: it is the thing at the
/// address that the outcome under test is about.
fn listener(greeting: &'static [u8]) -> (u16, JoinHandle<()>) {
    let bound = TcpListener::bind("127.0.0.1:0").expect("the loopback interface takes a listener");
    let port = bound
        .local_addr()
        .expect("a bound listener has an address")
        .port();

    let accepting = thread::spawn(move || {
        let (mut connection, _) = bound.accept().expect("tpl connects once");
        // The peer may already have gone; what it read is what it read.
        let _ = connection.write_all(greeting);
    });

    (port, accepting)
}

/// The name the diagnostics give the server of `provider`.
fn server_name(provider: &str) -> &'static str {
    match provider {
        "mariadb" => "MariaDB",
        "mysql" => "MySQL",
        other => panic!("no server provider {other}"),
    }
}

// -------------------------------------------------------------- FR-CONF-055 ---

#[test]
fn fr_conf_055_an_account_with_no_password_logs_in_on_every_mariadb_series() {
    // FR-CONF-055 and ADR-014 point 5: the empty password is presented as the
    // server's own client presents it, so `password = ""` and an entry with no
    // `password` key both log in. Before the patch the first ended at 77 on
    // every series.
    let Some(series) =
        mariadb::series("fr_conf_055_an_account_with_no_password_logs_in_on_every_mariadb_series")
    else {
        return;
    };
    let _exclusive = mariadb::exclusive();

    for server in series {
        let with_empty = mariadb::configuration(server, ENTRY, SCHEMA, (NOPASS, ""));
        let with_none = without_password(&with_empty);

        for (form, configuration) in [
            ("password = \"\"", &with_empty),
            ("no password", &with_none),
        ] {
            let outcome = read(&project(configuration));
            assert_eq!(
                outcome.code,
                Some(0),
                "{} {form}: {}",
                server.name(),
                outcome.err
            );
        }
    }
}

#[test]
fn fr_conf_055_an_account_with_no_password_logs_in_on_every_mysql_server() {
    // FR-CONF-055 and ADR-014 point 5, on MySQL: over TLS, and over a session
    // TLS does not protect with the authentication cache emptied first, which
    // shows the empty password needs no RSA public key (FR-MYSQL-045). Before
    // the patch the plaintext logins ended at 78.
    let test = "fr_conf_055_an_account_with_no_password_logs_in_on_every_mysql_server";
    let _exclusive = mysql::exclusive();
    let (Some(tls), Some(plain)) = (
        mysql::servers(test, mysql::Role::Series),
        mysql::servers(test, mysql::Role::NoTls),
    ) else {
        return;
    };

    let cases = tls
        .iter()
        .flat_map(|server| [(server, "verify-identity"), (server, "disabled")])
        .chain(plain.iter().map(|server| (server, "disabled")));

    for (server, mode) in cases {
        let with_empty = mysql::configuration(server, ENTRY, SCHEMA, (NOPASS, ""), mode);
        let with_none = without_password(&with_empty);

        for (form, configuration) in [
            ("password = \"\"", &with_empty),
            ("no password", &with_none),
        ] {
            mysql::flush_authentication_cache(server);
            let outcome = read(&project(configuration));
            assert_eq!(
                outcome.code,
                Some(0),
                "{} {mode} {form}: {}",
                server.name(),
                outcome.err
            );
        }
    }
}

// -------------------------------------------------------------- FR-ERR-049 ---

#[test]
fn fr_err_049_a_refused_tcp_connect_is_the_first_outcome_for_both_providers() {
    // FR-ERR-049, first outcome: nothing listens on port 1, so the operating
    // system reports ECONNREFUSED. The four lines are the specification's
    // example, with the address and the entry of this body.
    for provider in ["mariadb", "mysql"] {
        let outcome = read(&project(&local_entry(provider, 1)));

        assert_eq!(outcome.code, Some(69), "{provider}: {}", outcome.err);
        assert_eq!(
            outcome.err,
            format!(
                "error: the server at 127.0.0.1:1 refused the connection, for database entry '{ENTRY}'\n\
                 cause: the TCP connect to 127.0.0.1:1 did not open a session: the host refused it, \
                 or nothing is listening on that port\n\
                 hint:  check that the server is running and listening on port 1, or change the \
                 address with: tpl cfg database update {ENTRY} --host <host> --port <port>\n\
                 exit:  69 (EX_UNAVAILABLE)\n"
            ),
            "{provider}"
        );
    }
}

// -------------------------------------------------------------- FR-ERR-047 ---

#[test]
fn fr_err_047_a_connection_closed_in_the_login_is_the_first_outcome() {
    // FR-ERR-047, first outcome, and FR-CONF-005: the TCP connect completed,
    // so the driver recorded the login, and the connection closed before the
    // greeting. The cause names neither the TCP connect as failed nor a
    // refusal.
    for provider in ["mariadb", "mysql"] {
        let (port, accepting) = listener(b"");
        let outcome = read(&project(&local_entry(provider, port)));
        accepting.join().expect("the listener accepted once");

        assert_eq!(outcome.code, Some(69), "{provider}: {}", outcome.err);
        assert_eq!(
            outcome.err,
            format!(
                "error: the login to 127.0.0.1:{port} did not complete, for database entry '{ENTRY}'\n\
                 cause: the TCP connect to 127.0.0.1:{port} succeeded, and the connection failed \
                 before the login completed: the server closed it, it was reset, or the operating \
                 system abandoned it\n\
                 hint:  check that 127.0.0.1:{port} is the address of a running {} server, then run \
                 the command again, or change the address with: tpl cfg database update {ENTRY} \
                 --host <host> --port <port>\n\
                 exit:  69 (EX_UNAVAILABLE)\n",
                server_name(provider)
            ),
            "{provider}"
        );
    }
}

#[test]
fn fr_err_047_an_answer_outside_the_protocol_is_the_second_outcome() {
    // FR-ERR-047, second outcome: one well-framed packet whose payload is not
    // a greeting — no NUL ends the server version — so the driver reports a
    // protocol error while the login runs.
    const NOT_A_GREETING: &[u8] = &[3, 0, 0, 0, 0x0a, b'x', b'y'];

    for provider in ["mariadb", "mysql"] {
        let (port, accepting) = listener(NOT_A_GREETING);
        let outcome = read(&project(&local_entry(provider, port)));
        accepting.join().expect("the listener accepted once");

        let server = server_name(provider);
        assert_eq!(outcome.code, Some(69), "{provider}: {}", outcome.err);
        assert_eq!(
            outcome.err,
            format!(
                "error: the login to 127.0.0.1:{port} did not complete, for database entry '{ENTRY}'\n\
                 cause: what answered the login at 127.0.0.1:{port} did not answer as a {server} \
                 server does: it does not speak the {server} protocol, or it asked for a step tpl \
                 does not implement, such as an authentication method\n\
                 hint:  check that 127.0.0.1:{port} is the address of a {server} server, or change \
                 it with: tpl cfg database update {ENTRY} --host <host> --port <port>\n\
                 exit:  69 (EX_UNAVAILABLE)\n"
            ),
            "{provider}"
        );
    }
}

#[test]
fn fr_err_047_a_mariadb_entry_whose_login_needs_the_key_ends_at_69() {
    // FR-ERR-047, third outcome: a `mariadb` entry pointed at a MySQL server
    // that offers no TLS, with its authentication cache emptied, needs the
    // server's RSA public key, which tpl does not request. Under `disabled`
    // the four lines are the specification's example; under `preferred` the
    // cause also states that the server offered no TLS, and the hint that it
    // must offer TLS.
    let test = "fr_err_047_a_mariadb_entry_whose_login_needs_the_key_ends_at_69";
    let _exclusive = mysql::exclusive();
    let Some(plain) = mysql::servers(test, mysql::Role::NoTls) else {
        return;
    };
    let names = mysql::names();
    let (user, password) = (names.reader.0.as_str(), names.reader.1.as_str());

    for server in &plain {
        let name = server.name();
        let address = server.address();

        for mode in ["disabled", "preferred"] {
            let configuration =
                mysql::configuration(server, ENTRY, &names.schema, (user, password), mode)
                    .replace("provider = \"mysql\"\n", "");
            mysql::flush_authentication_cache(server);
            let outcome = read(&project(&configuration));

            assert_eq!(outcome.code, Some(69), "{name} {mode}: {}", outcome.err);
            let lines: Vec<&str> = outcome.err.lines().collect();
            assert_eq!(lines.len(), 4, "{name} {mode}: {}", outcome.err);
            assert_eq!(
                lines[0],
                format!(
                    "error: cannot authenticate user '{user}' over an unencrypted session, \
                     for database entry '{ENTRY}'"
                ),
                "{name} {mode}"
            );
            assert_eq!(lines[3], "exit:  69 (EX_UNAVAILABLE)", "{name} {mode}");

            if mode == "disabled" {
                assert_eq!(
                    lines[1],
                    format!(
                        "cause: connected to {address} as user '{user}' under tls 'disabled', so \
                         the session is not encrypted; the authentication was not completed: the \
                         server asks for the full caching_sha2_password exchange, which needs its \
                         RSA public key, and tpl does not request that key over a session TLS \
                         does not protect"
                    ),
                    "{name}"
                );
                assert_eq!(
                    lines[2],
                    format!(
                        "hint:  encrypt the session: tpl cfg database update {ENTRY} --tls verify-identity"
                    ),
                    "{name}"
                );
            } else {
                for fact in [
                    address,
                    user,
                    "'preferred'",
                    "offered no TLS",
                    "caching_sha2_password",
                    "tpl does not request that key over a session TLS does not protect",
                ] {
                    assert!(
                        lines[1].contains(fact),
                        "{name} {mode}: {fact}: {}",
                        lines[1]
                    );
                }
                assert!(
                    lines[2].contains(&format!(
                        "tpl cfg database update {ENTRY} --tls verify-identity"
                    )),
                    "{name} {mode}: {}",
                    lines[2]
                );
                // FR-ERR-047 item 3: the hint also states that the server
                // must offer TLS before that mode can connect.
                for fact in [
                    "offered no TLS",
                    "must offer it before that mode can connect",
                ] {
                    assert!(
                        lines[2].contains(fact),
                        "{name} {mode}: {fact}: {}",
                        lines[2]
                    );
                }
            }
        }
    }
}
