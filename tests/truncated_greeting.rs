//! A greeting that ends early, observed from outside the process.
//!
//! | Requirement | What is observed |
//! |---|---|
//! | `FR-ERR-047` outcome 2 | A well-framed packet that begins as a greeting and ends before the fields the protocol gives a greeting, or declares a field longer than the packet, is the second outcome: exit `69`, the second example's four lines, never the `70` of `FR-ERR-030` |
//! | `FR-ERR-047` outcome 1 | The same greeting whole is read, and the login goes on: the connection the listener then closes is the first outcome |
//! | `FR-ERR-047` outcome 1 | A greeting whose header announces more than the listener sends before it closes is cut short by the connection, and is the first outcome, not the second |
//! | `FR-ERR-048`, `FR-ERR-047` outcome 2 | An error packet cut short before its error number or its SQL state, in the greeting's place or in answer to the login, is the second outcome, never `70` |
//! | `FR-ERR-047` outcome 2 | A one-byte `0x01` packet in answer to a `caching_sha2_password` login is the second outcome, never `70` |
//!
//! Each case is a listener on the loopback interface that accepts one
//! connection, writes one packet whose four-byte header announces exactly the
//! payload that follows, and closes. The payload is a prefix of a real
//! greeting, cut where the decoder reads its next field, so every field read
//! of the decoder that can find fewer bytes than it needs is reached once.
//! Both providers are run, because a `mariadb` and a `mysql` entry log in
//! through the same driver (`FR-MYSQL-020`). The bodies need no server: what
//! answers at the address is exactly what they are about.
//!
//! The two greetings are the ones the driver's own decoder tests carry, the
//! MySQL one with the version of the fixture's `8.4` server: a MySQL greeting,
//! whose capability flags set `CLIENT_LONG_PASSWORD` and so carry four
//! reserved bytes, and a MariaDB one, whose flags clear it and so carry its
//! extended capabilities in their place.

#[path = "support/sandbox.rs"]
mod sandbox;

use std::fmt::Write as _;
use std::io::Write as _;
use std::net::TcpListener;
use std::thread::{self, JoinHandle};

use sandbox::Sandbox;

/// The entry every body names.
const ENTRY: &str = "shop";

/// A greeting of a MySQL `8.4` server, field by field.
///
/// Protocol version `10`; server version `8.4.13`; connection id; the first
/// eight bytes of the scramble; the filler; the lower capability flags; the
/// collation; the status flags; the upper capability flags, with
/// `CLIENT_PLUGIN_AUTH`; the scramble's length, `21`; ten reserved bytes; the
/// scramble's last twelve bytes and their NUL; the authentication method.
const MYSQL: &[u8] = b"\x0a\
    8.4.13\x00\
    \x19\x00\x00\x00\
    \x114aB0c\x06g\
    \x00\
    \xff\xff\
    \xff\
    \x02\x00\
    \xff\xc7\
    \x15\
    \x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\
    tL\x03s\x0f[4\rl4. \x00\
    caching_sha2_password\x00";

/// A greeting of a MariaDB server: its lower capability flags clear
/// `CLIENT_LONG_PASSWORD`, so the four bytes after the six reserved ones are
/// its extended capabilities.
const MARIADB: &[u8] = b"\x0a\
    5.5.5-10.4.7-MariaDB-1:10.4.7+maria~bionic\x00\
    \x0b\x00\x00\x00\
    t6L\\j\"dS\
    \x00\
    \xfe\xf7\
    \x08\
    \x02\x00\
    \xff\x81\
    \x15\
    \x00\x00\x00\x00\x00\x00\
    \x07\x00\x00\x00\
    U14Oph9\"<H5n\x00\
    mysql_native_password\x00";

/// Where each field of [`MYSQL`] begins.
mod at {
    /// The server version, after the protocol version.
    pub const VERSION: usize = 1;
    /// The connection id, after `8.4.13` and its NUL.
    pub const CONNECTION_ID: usize = 8;
    /// The first eight bytes of the scramble.
    pub const SCRAMBLE_1: usize = 12;
    /// The filler byte.
    pub const FILLER: usize = 20;
    /// The lower half of the capability flags.
    pub const CAPABILITIES_LOW: usize = 21;
    /// The collation.
    pub const COLLATION: usize = 23;
    /// The status flags.
    pub const STATUS: usize = 24;
    /// The upper half of the capability flags.
    pub const CAPABILITIES_HIGH: usize = 26;
    /// The length of the scramble.
    pub const SCRAMBLE_LENGTH: usize = 28;
    /// The six reserved bytes.
    pub const RESERVED_6: usize = 29;
    /// The four reserved bytes a MySQL greeting carries.
    pub const RESERVED_4: usize = 35;
    /// The scramble's last twelve bytes.
    pub const SCRAMBLE_2: usize = 39;
    /// The NUL after them.
    pub const SCRAMBLE_2_NUL: usize = 51;
    /// The authentication method.
    pub const METHOD: usize = 52;
}

/// Where the extended capabilities of [`MARIADB`] begin: after its protocol
/// version, its 42-byte server version and NUL, and the 27 bytes that follow
/// up to the end of the six reserved bytes.
const MARIADB_EXTENDED_CAPABILITIES: usize = 1 + 42 + 1 + 27;

/// One packet the listener writes: the four-byte header announcing exactly
/// `payload`, then `payload`.
fn framed(payload: &[u8]) -> Vec<u8> {
    let length = u32::try_from(payload.len()).expect("a test payload is short");
    let mut packet = length.to_le_bytes().to_vec();
    packet[3] = 0; // the sequence id of the greeting
    packet.extend_from_slice(payload);

    packet
}

/// [`MYSQL`] with the byte at `offset` replaced by `value`.
fn mysql_with(offset: usize, value: u8) -> Vec<u8> {
    let mut greeting = MYSQL.to_vec();
    greeting[offset] = value;

    greeting
}

/// One greeting that ends early, and the field the decoder reads next.
struct Case {
    /// What the case is, for the failure report.
    name: &'static str,
    /// The payload of the one packet written.
    payload: Vec<u8>,
}

/// Every case: a prefix of a greeting cut at each field read the decoder
/// makes, and two greetings whose own fields send that read past the packet.
fn cases() -> Vec<Case> {
    // The offsets name the fields they say they name.
    assert_eq!(MYSQL.len(), 74, "the MySQL greeting is 74 bytes");
    assert_eq!(&MYSQL[at::CONNECTION_ID - 1..][..1], b"\x00");
    assert_eq!(&MYSQL[at::METHOD..], b"caching_sha2_password\x00");
    assert_eq!(
        &MARIADB[MARIADB_EXTENDED_CAPABILITIES..][..4],
        b"\x07\x00\x00\x00",
        "the MariaDB greeting's extended capabilities"
    );

    let prefix = |name, length: usize| Case {
        name,
        payload: MYSQL[..length].to_vec(),
    };

    let mut cases = vec![
        prefix("an empty packet, before the protocol version", 0),
        prefix(
            "just after the protocol version, before the server version",
            at::VERSION,
        ),
        prefix("inside the server version string", at::VERSION + 3),
        prefix(
            "after the server version, before the connection id",
            at::CONNECTION_ID,
        ),
        prefix("inside the connection id", at::CONNECTION_ID + 2),
        prefix("inside the scramble's first part", at::SCRAMBLE_1 + 3),
        prefix("before the filler", at::FILLER),
        prefix(
            "inside the lower capability flags",
            at::CAPABILITIES_LOW + 1,
        ),
        prefix("before the collation", at::COLLATION),
        prefix("inside the status flags", at::STATUS + 1),
        prefix(
            "before the upper half of the capability flags",
            at::CAPABILITIES_HIGH,
        ),
        prefix(
            "inside the upper half of the capability flags",
            at::CAPABILITIES_HIGH + 1,
        ),
        prefix("before the scramble's length", at::SCRAMBLE_LENGTH),
        prefix("inside the six reserved bytes", at::RESERVED_6 + 2),
        prefix(
            "inside the four reserved bytes of a MySQL greeting",
            at::RESERVED_4 + 1,
        ),
        prefix("inside the scramble's second part", at::SCRAMBLE_2 + 5),
        prefix(
            "before the NUL after the scramble's second part",
            at::SCRAMBLE_2_NUL,
        ),
        prefix("before the authentication method name", at::METHOD),
        prefix(
            "inside the authentication method name, no NUL",
            at::METHOD + 7,
        ),
    ];

    cases.push(Case {
        name: "inside the extended capabilities of a MariaDB greeting",
        payload: MARIADB[..MARIADB_EXTENDED_CAPABILITIES + 2].to_vec(),
    });

    // `CLIENT_PLUGIN_AUTH` cleared (bit 19, bit 3 of the upper flags' first
    // byte): the decoder skips the scramble's length byte instead of reading
    // it, a read of its own.
    let mut without_plugin_auth = mysql_with(at::CAPABILITIES_HIGH, 0xf7);
    without_plugin_auth.truncate(at::SCRAMBLE_LENGTH);
    cases.push(Case {
        name: "before the scramble's length, CLIENT_PLUGIN_AUTH cleared",
        payload: without_plugin_auth,
    });

    // Malformed rather than truncated: every field is present, and the
    // scramble's length, 255, declares a second part of 246 bytes the packet
    // does not hold.
    cases.push(Case {
        name: "a scramble length of 255, longer than the packet",
        payload: mysql_with(at::SCRAMBLE_LENGTH, 0xff),
    });

    cases
}

/// A listener on a port of the loopback interface that accepts one
/// connection, writes `packet` to it, and closes it.
fn listener(packet: Vec<u8>) -> (u16, JoinHandle<()>) {
    let bound = TcpListener::bind("127.0.0.1:0").expect("the loopback interface takes a listener");
    let port = bound
        .local_addr()
        .expect("a bound listener has an address")
        .port();

    let accepting = thread::spawn(move || {
        let (mut connection, _) = bound.accept().expect("tpl connects once");
        // The peer may already have gone; what it read is what it read.
        let _ = connection.write_all(&packet);
    });

    (port, accepting)
}

/// An entry of `provider` reaching `127.0.0.1:port`, as the reader account.
fn local_entry(provider: &str, port: u16) -> String {
    format!(
        "[core]\ndatabase = \"{ENTRY}\"\n\n\
         [database.{ENTRY}]\nprovider = \"{provider}\"\nhost = \"127.0.0.1\"\nport = {port}\n\
         user = \"reader\"\npassword = \"irrelevant\"\n\
         database = \"freight\"\ntls = \"disabled\"\n"
    )
}

/// The sequence id of the server's answer to the login: the greeting is `0`
/// and the client's handshake response `1`.
const ANSWER_SEQUENCE: u8 = 2;

/// One packet carrying `payload` under sequence id `sequence`.
fn framed_as(payload: &[u8], sequence: u8) -> Vec<u8> {
    let mut packet = framed(payload);
    packet[3] = sequence;

    packet
}

/// A listener that accepts one connection, writes `greeting` whole, reads the
/// client's handshake response, writes `answer`, and closes.
///
/// The response is read whole — its header, then the payload it announces —
/// so that `answer` reaches a client that has sent its login and waits for
/// the server's reply, which is where the driver reads it.
fn answering(greeting: &'static [u8], answer: Vec<u8>) -> (u16, JoinHandle<()>) {
    use std::io::Read as _;

    let bound = TcpListener::bind("127.0.0.1:0").expect("the loopback interface takes a listener");
    let port = bound
        .local_addr()
        .expect("a bound listener has an address")
        .port();

    let accepting = thread::spawn(move || {
        let (mut connection, _) = bound.accept().expect("tpl connects once");
        if connection.write_all(&framed(greeting)).is_err() {
            return;
        }

        let mut header = [0_u8; 4];
        if connection.read_exact(&mut header).is_err() {
            return;
        }
        let length =
            usize::from(header[0]) | usize::from(header[1]) << 8 | usize::from(header[2]) << 16;
        let mut response = vec![0_u8; length];
        if connection.read_exact(&mut response).is_err() {
            return;
        }

        // The peer may already have gone; what it read is what it read.
        let _ = connection.write_all(&answer);
    });

    (port, accepting)
}

/// Runs `tpl schema tables --direct --no-cache` for an entry of `provider`
/// against a listener that answers the login of `greeting` with `answer`.
fn answered(provider: &str, greeting: &'static [u8], answer: Vec<u8>) -> (u16, Outcome) {
    let (port, accepting) = answering(greeting, answer);
    let sandbox = Sandbox::new();
    sandbox.project(&local_entry(provider, port));
    let printed = sandbox.run(&["schema", "tables", "--direct", "--no-cache"]);
    accepting.join().expect("the listener accepted once");

    (
        port,
        Outcome {
            code: printed.status.code(),
            err: String::from_utf8_lossy(&printed.stderr).into_owned(),
        },
    )
}

/// The error packets that end before the fields an error packet carries:
/// before its error number, inside it, and inside its SQL state.
const SHORT_ERROR_PACKETS: [(&str, &[u8]); 3] = [
    ("the error packet's marker alone", b"\xff"),
    ("inside the error number", b"\xff\x15"),
    ("inside the SQL state", b"\xff\x15\x04#42"),
];

/// Records a run that did not end with the second outcome.
fn unless_second(failures: &mut String, provider: &str, case: &str, port: u16, outcome: &Outcome) {
    if outcome.code != Some(69) || outcome.err != second_outcome(provider, port) {
        let _ = write!(
            failures,
            "\n--- {provider}, {case}: exit {:?}\n{}",
            outcome.code, outcome.err
        );
    }
}

/// The name the diagnostics give the server of `provider`.
fn server_name(provider: &str) -> &'static str {
    match provider {
        "mariadb" => "MariaDB",
        "mysql" => "MySQL",
        other => panic!("no server provider {other}"),
    }
}

/// What one run produced.
struct Outcome {
    /// The exit code.
    code: Option<i32>,
    /// Standard error.
    err: String,
}

/// Runs `tpl schema tables --direct --no-cache` for an entry of `provider`
/// against a listener that writes `packet`.
fn against(provider: &str, packet: Vec<u8>) -> (u16, Outcome) {
    let (port, accepting) = listener(packet);
    let sandbox = Sandbox::new();
    sandbox.project(&local_entry(provider, port));
    let printed = sandbox.run(&["schema", "tables", "--direct", "--no-cache"]);
    accepting.join().expect("the listener accepted once");

    (
        port,
        Outcome {
            code: printed.status.code(),
            err: String::from_utf8_lossy(&printed.stderr).into_owned(),
        },
    )
}

/// The second outcome's four lines, for an entry of `provider` at `port`.
fn second_outcome(provider: &str, port: u16) -> String {
    let server = server_name(provider);
    format!(
        "error: the login to 127.0.0.1:{port} did not complete, for database entry '{ENTRY}'\n\
         cause: what answered the login at 127.0.0.1:{port} did not answer as a {server} \
         server does: it does not speak the {server} protocol, or it asked for a step tpl \
         does not implement, such as an authentication method\n\
         hint:  check that 127.0.0.1:{port} is the address of a {server} server, or change \
         it with: tpl cfg database update {ENTRY} --host <host> --port <port>\n\
         exit:  69 (EX_UNAVAILABLE)\n"
    )
}

/// The first outcome's four lines, for an entry of `provider` at `port`.
fn first_outcome(provider: &str, port: u16) -> String {
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
    )
}

// -------------------------------------------------------------- FR-ERR-047 ---

#[test]
fn fr_err_047_a_whole_greeting_is_read_and_the_login_goes_on() {
    // The control: both greetings, whole, are read by the decoder, so the
    // login goes on to its response, and the listener's close is the first
    // outcome. Every failure of the next body is therefore the truncation's.
    for (provider, greeting) in [
        ("mysql", MYSQL),
        ("mariadb", MYSQL),
        ("mysql", MARIADB),
        ("mariadb", MARIADB),
    ] {
        let (port, outcome) = against(provider, framed(greeting));

        assert_eq!(outcome.code, Some(69), "{provider}: {}", outcome.err);
        assert_eq!(outcome.err, first_outcome(provider, port), "{provider}");
    }
}

#[test]
fn fr_err_047_a_greeting_that_ends_early_is_the_second_outcome() {
    // FR-ERR-047, second outcome: a truncated or malformed greeting, in a
    // well-framed packet, is an answer the system does not accept — exit 69
    // with the second example's four lines, and never the 70 of FR-ERR-030.
    // Every case is run for both providers before the verdict, so that one
    // failure reports all of them.
    let mut failures = String::new();
    let mut runs = 0_usize;

    for case in cases() {
        for provider in ["mysql", "mariadb"] {
            let (port, outcome) = against(provider, framed(&case.payload));
            runs += 1;

            if outcome.code != Some(69) || outcome.err != second_outcome(provider, port) {
                let _ = write!(
                    failures,
                    "\n--- {provider}, {} ({} bytes): exit {:?}\n{}",
                    case.name,
                    case.payload.len(),
                    outcome.code,
                    outcome.err
                );
            }
        }
    }

    assert!(
        failures.is_empty(),
        "of {runs} runs, these did not end with the second outcome:{failures}"
    );
}

#[test]
fn fr_err_047_a_greeting_cut_short_by_the_connection_is_the_first_outcome() {
    // FR-ERR-047, the boundary of the second outcome: the header announces
    // the whole greeting and the listener closes after part of it, so the
    // packet's announced length never arrives. That is the connection
    // closing, the first outcome, wherever the cut falls.
    for cut in [1, at::CONNECTION_ID + 2, at::CAPABILITIES_HIGH, at::METHOD] {
        let mut packet = framed(MYSQL);
        packet.truncate(4 + cut);

        for provider in ["mysql", "mariadb"] {
            let (port, outcome) = against(provider, packet.clone());

            assert_eq!(
                outcome.code,
                Some(69),
                "{provider}, cut at {cut}: {}",
                outcome.err
            );
            assert_eq!(
                outcome.err,
                first_outcome(provider, port),
                "{provider}, cut at {cut}"
            );
        }
    }
}

// -------------------------------------------------------------- FR-ERR-048 ---

#[test]
fn fr_err_048_a_short_error_packet_in_the_greeting_place_is_the_second_outcome() {
    // FR-ERR-048, eightieth edition: an error packet the system cannot read as
    // the server's — one that ends before its error number, or before the
    // fields an error packet carries — has no number to classify, and is the
    // second outcome of FR-ERR-047 at 69, never the 70 of FR-ERR-030.
    let mut failures = String::new();

    for (case, payload) in SHORT_ERROR_PACKETS {
        for provider in ["mysql", "mariadb"] {
            let (port, outcome) = against(provider, framed(payload));
            unless_second(&mut failures, provider, case, port, &outcome);
        }
    }

    assert!(
        failures.is_empty(),
        "these did not end with the second outcome:{failures}"
    );
}

#[test]
fn fr_err_048_a_short_error_packet_in_answer_to_the_login_is_the_second_outcome() {
    // FR-ERR-048 and FR-ERR-047, as above, for the packet the server answers
    // the client's handshake response with, after a whole greeting of either
    // product.
    let mut failures = String::new();

    for (case, payload) in SHORT_ERROR_PACKETS {
        for greeting in [MYSQL, MARIADB] {
            for provider in ["mysql", "mariadb"] {
                let (port, outcome) =
                    answered(provider, greeting, framed_as(payload, ANSWER_SEQUENCE));
                unless_second(&mut failures, provider, case, port, &outcome);
            }
        }
    }

    assert!(
        failures.is_empty(),
        "these did not end with the second outcome:{failures}"
    );
}

#[test]
fn fr_err_047_a_one_byte_0x01_packet_in_a_caching_sha2_password_login_is_the_second_outcome() {
    // FR-ERR-047, eightieth edition: the greeting names caching_sha2_password,
    // and the server answers the login with a packet holding the byte 0x01
    // alone — the start of that method's exchange, without the byte that says
    // which step. It is an answer outside the protocol: 69, never 70.
    let mut failures = String::new();

    for provider in ["mysql", "mariadb"] {
        let (port, outcome) = answered(provider, MYSQL, framed_as(b"\x01", ANSWER_SEQUENCE));
        unless_second(
            &mut failures,
            provider,
            "a one-byte 0x01 packet",
            port,
            &outcome,
        );
    }

    assert!(
        failures.is_empty(),
        "these did not end with the second outcome:{failures}"
    );
}

#[test]
fn fr_err_047_a_whole_error_packet_in_answer_to_the_login_is_classified_by_its_number() {
    // The control for the two bodies above: the same exchange, with an error
    // packet that carries every field, is read as the server's and classified
    // by its number under FR-ERR-048 — 1045, a refused authentication, 77 —
    // so every 69 above is the shortness of the packet.
    for provider in ["mysql", "mariadb"] {
        let (_, outcome) = answered(
            provider,
            MYSQL,
            framed_as(
                b"\xff\x15\x04#28000Access denied for user 'reader'",
                ANSWER_SEQUENCE,
            ),
        );

        assert_eq!(outcome.code, Some(77), "{provider}: {}", outcome.err);
    }
}
