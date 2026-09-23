//! Where the driver's own error stops, and a condition of `FR-ERR-001` begins.
//!
//! `FR-GLOB-018` bars the raw driver error from every diagnostic stream at
//! every verbosity, and `OD-06` makes that structural by refusing it a home
//! inside [`Error`]: no variant carries it and no conversion from it exists.
//! This module is the boundary those two describe. A driver failure arrives
//! here, is **classified** into a phase, a host, a port and an outcome, and the
//! original value is dropped — so it is not present in the process to be
//! emitted even by a function that would take one.
//!
//! | What the driver returned | What it becomes | Forced by |
//! |---|---|---|
//! | An error packet during the handshake, about the **database** | [`Error::PropertyNotReadable`], `77` | `FR-PRIV-021`: the reader was not shown the database the entry names |
//! | Any other error packet during the handshake | [`Error::AuthenticationRefused`], `77` | The server refused this connection for these credentials |
//! | A TLS failure | [`Error::TlsHandshakeFailed`], `69` | `FR-ERR-034`, the `69` row, which names the TLS handshake as a phase |
//! | Anything else, once the address is known | [`Error::ConnectionRefused`], `69` | The session did not open, and `FR-ERR-001` puts an unavailable server on `69` |
//! | A phase that outlived its bound | [`Error::NetworkDeadlineExceeded`], `69` | `FR-ERR-027`, `FR-GLOB-012`, `FR-GLOB-013` |
//!
//! *The driver's error is read for its discriminants and for nothing else.*
//! Point 3 of `OD-12` separates the TCP connect from the TLS handshake by the
//! driver's own variant rather than by the call site, because
//! `MySqlConnectOptions` has no method that accepts an already-connected
//! socket and one call therefore covers both phases. Three discriminants are
//! read in all — the driver's variant, the I/O kind on one of them, and the
//! server's **error number** on an error packet — and nothing else crosses
//! this boundary: no message, no SQLSTATE string and no chain is carried out
//! of this module, which is what `FR-GLOB-018` and `OD-06` require.
//!
//! # The two `77`s of the handshake, and what separates them
//!
//! An error packet during the handshake used to become
//! [`Error::AuthenticationRefused`] whatever it said, which told a caller whose
//! entry names a database the server does not hold that **its credentials were
//! refused**. They were accepted. `FR-PRIV-021` gives that condition its own
//! `77`, naming the database and stating that its metadata could not be read,
//! and the two are separated here because the handshake is where the condition
//! arrives: `FR-CONF-041` puts the database on the connection, so the server
//! refuses the connection and the schema catalogue is never reached.
//!
//! *Observed against `scripts/mariadb/` on 2026-09-18, on all four series of
//! `FR-SRV-015` — `10.11.19`, `11.4.13`, `11.8.9` and `12.3.3` — with the
//! reading identical on every one of them:*
//!
//! | What was attempted | Error number | SQLSTATE |
//! |---|---|---|
//! | A user the server does not know | `1045` | `28000` |
//! | A user it knows, with the wrong password | `1045` | `28000` |
//! | A database the server does not hold | `1049` | `42000` |
//! | A database the reader holds no grant on | `1044` | `42000` |
//!
//! **The two numbers this module reads are `1044` and `1049`, and they are
//! exactly the two explanations `FR-PRIV-021` declines to separate** — a reader
//! who may not see the database, and a database that is not there. That is why
//! one condition carries both: the requirement states that the catalogue offers
//! no second view to tell them apart, and neither does the handshake.
//!
//! *Rejected: separating on SQLSTATE `42000`.* It covers both numbers and is
//! the coarser reading — `42000` is the server's general class for *syntax
//! error or access rule violation*, so a future handshake packet in that class
//! that is about neither the database nor the credentials would be reported as
//! a database the reader could not see. `FR-ERR-034` bans a `cause` that would
//! read identically for a different failure, and the two numbers are the
//! narrowest discriminant that covers the observation and nothing beyond it.
//!
//! *Rejected: reading the driver's message for the database name.* The name is
//! already in hand — it is what the entry named, per `FR-CONF-041` — and
//! `FR-GLOB-018` bars the message from every stream at every verbosity.
//!
//! **The variant alone is not enough, and the observation that settles it was
//! owed.** `OD-12` recorded that whether a TLS failure reaches `tpl` as
//! `sqlx::Error::Tls` was not confirmed in the driver's documentation, and
//! that `FR-ERR-034`'s `69` row could not be met as written if such a failure
//! arrived as `Error::Io`. It does, for one of the two failure modes. Observed
//! against `scripts/mariadb/` on 2026-09-18, on the `11.8` server and the
//! `--skip-ssl` one:
//!
//! | Failure | What the driver returned |
//! |---|---|
//! | A server offering no TLS, under `required` | `Error::Tls` |
//! | A certificate the trust material does not vouch for, under `verify-ca` and under `verify-identity` | `Error::Io`, of kind `InvalidData` |
//!
//! The second is the driver completing the TLS handshake's own I/O and
//! propagating what the TLS implementation reported, which reaches `tpl` as an
//! I/O error of that kind and of no other. The phase is therefore read from
//! the variant **and** from that kind, which is what keeps `FR-ERR-034`'s `69`
//! row satisfiable: a caller told the TCP connect failed would check that the
//! server is listening, and it is.
//!
//! *Bounded claim.* Two failure modes were observed, on one server of one
//! series and on the `--skip-ssl` server, at the driver version `ADR-003`
//! pins. A certificate that does not name the host was **not** observed — the
//! fixture's certificate names every spelling of the loopback a test can write
//! — and is classified with the first on the ground that it is the same
//! rejection by the same implementation on the same call; that is reasoning,
//! not an observation, and it is written down as such. No message and no code
//! from the driver is read by any of it.
//!
//! **A recorded limit, and the only one this module has.** A failure that is
//! neither a handshake refusal, nor TLS, nor a deadline — a connection that
//! drops while a statement is in flight, a protocol error — is reported as
//! [`Error::ConnectionRefused`]. Its exit code is right, `69`, and a caller's
//! next step is right, because the server is unreachable either way; its
//! `cause` line names the TCP connect, which such a failure is not. The four
//! variants of `69` are the whole of what `FR-ERR-001` affords, and
//! `FR-ERR-034` enumerates four phases without giving the catalogue-query
//! phase a variant of its own outside a deadline. The gap is reported rather
//! than filled: a fifth condition is a change to `FR-ERR-001` and to
//! `FR-ERR-034` together, which this module may not make.

// The harness is asked rather than restated, for the reason
// `super::catalogue` gives where it declares the same module: the gate, the
// inventory and the address of each server come from `scripts/mariadb/`, and
// `#[path]` is what reaches a file that must not become a test target of its
// own. It is **used** from there rather than declared a second time: one file
// reached by two `#[path]` items is two modules over one file, which
// `clippy::duplicate_mod` refuses and which would give the harness's own
// `OnceLock` gate two instances in one binary.
#[cfg(test)]
use super::catalogue::fixture;

use sqlx::mysql::MySqlDatabaseError;

use crate::deadline::Bound;
use crate::error::{CatalogueObjectKind, Error, NetworkPhase, TlsFault};

/// The server's error number for a database the reader holds no grant on.
///
/// Observed on all four series on 2026-09-18, under this module's own table.
const DATABASE_ACCESS_DENIED: u16 = 1044;

/// The server's error number for a database the server does not hold.
///
/// Observed on all four series on 2026-09-18, under this module's own table.
const UNKNOWN_DATABASE: u16 = 1049;

/// The property `FR-PRIV-021` reports unreadable, as its `cause` line names it.
///
/// It is `pub(crate)` because two paths reach that one condition and the
/// requirement gives it one `cause`: the handshake classifies the `1049` and
/// `1044` packets here, and [`crate::mariadb::catalogue`]'s fold reports a
/// schema catalogue that returned no row. A second literal would be a second
/// thing that can be wrong.
pub(crate) const METADATA: &str = "metadata";

/// The condition a failure to open the session is reported as.
///
/// `user` is the user actually presented to the server, which the caller reads
/// back from the options it composed rather than from the entry, so that a
/// refusal names what was sent. `database` is the one the selected entry names,
/// per `FR-CONF-041`, and is [`None`] where the entry names none — in which
/// case no packet about a database can arrive, because none was sent.
///
/// The two `77`s this produces are separated by the server's error number, for
/// the reason and on the evidence this module's own documentation records.
pub(crate) fn connecting(
    entry: &str,
    error: &sqlx::Error,
    host: &str,
    port: u16,
    user: &str,
    database: Option<&str>,
) -> Error {
    // An error packet a server sends during the handshake refuses the
    // connection, and the two things it can refuse it over are the credentials
    // presented and the database named. Both are `77` (`EX_NOPERM`) and
    // neither is an availability failure; which of the two it was decides
    // which `cause` the caller reads.
    if let sqlx::Error::Database(returned) = error {
        if let (Some(database), true) = (database, about_the_database(returned.as_ref())) {
            return Error::PropertyNotReadable {
                kind: CatalogueObjectKind::Database,
                object: database.to_owned(),
                property: METADATA,
            };
        }

        return Error::AuthenticationRefused {
            entry: entry.to_owned(),
            user: user.to_owned(),
            host: host.to_owned(),
        };
    }

    speaking(entry, error, host, port)
}

/// Whether an error packet refuses the **database** rather than the
/// credentials (`FR-PRIV-021`).
///
/// The two numbers are the two explanations that requirement declines to
/// separate, and they are read from the driver's MySQL-specific error type
/// because the number is the discriminant and the trait above it carries only
/// SQLSTATE — which this module's documentation records as the coarser
/// reading. A packet the driver reports without a number of this type is not
/// one of the two, and falls to the credential refusal.
fn about_the_database(returned: &dyn sqlx::error::DatabaseError) -> bool {
    returned
        .try_downcast_ref::<MySqlDatabaseError>()
        .is_some_and(|packet| matches!(packet.number(), DATABASE_ACCESS_DENIED | UNKNOWN_DATABASE))
}

/// The condition a failure of a statement on an open session is reported as.
///
/// The name resolution and the handshake are behind it, so a `Database` error
/// here is the server answering the statement rather than refusing the
/// connection, and the caller that knows which statement it issued classifies
/// that outcome itself — the read-only pair of `FR-SRV-008` and `FR-SRV-009`
/// is `78`, not `69`, whatever the server said.
pub(crate) fn speaking(entry: &str, error: &sqlx::Error, host: &str, port: u16) -> Error {
    if let Some(fault) = tls(error) {
        return Error::TlsHandshakeFailed {
            entry: entry.to_owned(),
            host: host.to_owned(),
            port,
            fault,
        };
    }

    Error::ConnectionRefused {
        entry: entry.to_owned(),
        host: host.to_owned(),
        port,
    }
}

/// Whether the failure arose in the TLS handshake.
///
/// Two discriminants and no third, for the reason this module's own
/// documentation records: the driver reports a server that offers no TLS on
/// its own variant, and a certificate the trust material does not vouch for as
/// an I/O error of kind [`InvalidData`](std::io::ErrorKind::InvalidData) —
/// which is the TLS implementation's verdict travelling through the call that
/// completes the handshake's I/O. No kind the operating system produces for a
/// socket is `InvalidData`, and the driver reports a malformed protocol packet
/// on a variant of its own, so the kind separates the two phases without
/// reading a message.
///
/// The same two discriminants say what the handshake returned, which the `69`
/// row of `FR-ERR-034` obliges the `cause` to name: the driver's own variant is
/// the TLS layer refusing before a certificate was judged — a server offering
/// none among them — and `InvalidData` is the TLS implementation's verdict on
/// the certificate the server presented.
fn tls(error: &sqlx::Error) -> Option<TlsFault> {
    match error {
        sqlx::Error::Tls(_) => Some(TlsFault::Refused),
        sqlx::Error::Io(returned) if returned.kind() == std::io::ErrorKind::InvalidData => {
            Some(TlsFault::CertificateRejected)
        }
        _ => None,
    }
}

/// The condition a phase that outlived its bound is reported as.
///
/// `FR-ERR-034` obliges the `cause` line to name which of the two bounds of
/// `FR-GLOB-012` expired and its resolved value, and [`Bound`] carries both
/// because the composition was made where the two were known.
pub(crate) fn expired(
    entry: &str,
    phase: NetworkPhase,
    host: &str,
    port: u16,
    bound: Bound,
) -> Error {
    Error::NetworkDeadlineExceeded {
        entry: entry.to_owned(),
        phase,
        host: host.to_owned(),
        port,
        bound: bound.bound(),
        limit: bound.limit(),
    }
}

#[cfg(test)]
mod tests {
    use super::fixture;
    use super::{connecting, expired, speaking};
    use crate::deadline::{Bound, Seconds};
    use crate::error::{CatalogueObjectKind, DeadlineBound, Error, NetworkPhase};
    use std::io;
    use std::num::NonZeroU64;

    /// The host and port every condition below names.
    const HOST: &str = "db.example.com";
    const PORT: u16 = 3306;

    /// The database the selected entry names (`FR-CONF-041`), which every
    /// condition below is classified against.
    const DATABASE: &str = "freight";

    /// A driver error of the kind an unreachable server produces.
    fn unreachable() -> sqlx::Error {
        sqlx::Error::Io(io::Error::from(io::ErrorKind::ConnectionRefused))
    }

    /// A driver error of the kind a refused TLS negotiation produces.
    fn refused_tls() -> sqlx::Error {
        sqlx::Error::Tls("server does not support TLS".into())
    }

    /// A driver error of the kind a certificate the trust material does not
    /// vouch for produces, as observed on 2026-09-18.
    fn untrusted_certificate() -> sqlx::Error {
        sqlx::Error::Io(io::Error::new(
            io::ErrorKind::InvalidData,
            "invalid peer certificate: UnknownIssuer",
        ))
    }

    #[test]
    fn fr_err_034_a_tls_failure_is_reported_as_the_tls_handshake_phase() {
        // FR-ERR-034, the 69 row: the cause names the phase that failed, and
        // OD-12 point 3 derives it from the driver's discriminant.
        let condition = connecting("shop", &refused_tls(), HOST, PORT, "alice", Some(DATABASE));

        assert!(matches!(condition, Error::TlsHandshakeFailed { .. }));
        assert_eq!(condition.exit_code(), 69);
    }

    #[test]
    fn fr_err_034_a_certificate_the_trust_material_does_not_vouch_for_is_the_tls_phase() {
        // Observed on 2026-09-18 against scripts/mariadb: `verify-ca` and
        // `verify-identity` against a certificate this client cannot chain
        // return an I/O error of kind InvalidData rather than the driver's own
        // TLS variant. Reported as the TCP connect it would send a caller to
        // check a server that is listening.
        let condition = connecting(
            "shop",
            &untrusted_certificate(),
            HOST,
            PORT,
            "alice",
            Some(DATABASE),
        );

        assert!(matches!(condition, Error::TlsHandshakeFailed { .. }));
        assert_eq!(condition.exit_code(), 69);
    }

    #[test]
    fn fr_err_034_an_ordinary_socket_failure_is_not_read_as_the_tls_phase() {
        // The kind is the discriminant, and no kind the operating system
        // produces for a socket is InvalidData.
        for kind in [
            io::ErrorKind::ConnectionRefused,
            io::ErrorKind::ConnectionReset,
            io::ErrorKind::TimedOut,
            io::ErrorKind::UnexpectedEof,
            io::ErrorKind::BrokenPipe,
        ] {
            let driver = sqlx::Error::Io(io::Error::from(kind));

            assert!(
                matches!(
                    connecting("shop", &driver, HOST, PORT, "alice", Some(DATABASE)),
                    Error::ConnectionRefused { .. }
                ),
                "{kind:?}"
            );
        }
    }

    #[test]
    fn fr_err_001_a_server_that_does_not_answer_is_unavailable_rather_than_misconfigured() {
        let condition = connecting("shop", &unreachable(), HOST, PORT, "alice", Some(DATABASE));

        assert!(matches!(condition, Error::ConnectionRefused { .. }));
        assert_eq!(condition.exit_code(), 69);
    }

    #[test]
    fn fr_glob_018_no_driver_message_survives_the_classification() {
        // FR-GLOB-018 and OD-06: the driver's own text reaches no stream,
        // because the value it lived on is dropped at this boundary.
        let driver = sqlx::Error::Tls("a message the driver composed".into());
        let condition = connecting("shop", &driver, HOST, PORT, "alice", Some(DATABASE));

        assert!(!format!("{condition}").contains("a message the driver composed"));
        assert!(!format!("{condition:?}").contains("a message the driver composed"));
    }

    #[test]
    fn fr_err_034_a_statement_that_fails_is_not_read_as_a_refused_credential() {
        // On an open session the handshake is behind us, so nothing here is a
        // 77: the session did not hold, which is 69.
        let condition = speaking("shop", &unreachable(), HOST, PORT);

        assert_eq!(condition.exit_code(), 69);
    }

    #[test]
    fn fr_err_034_an_expired_phase_names_the_bound_that_expired_and_its_value() {
        // FR-ERR-034, the 69 row, with FR-GLOB-012: which of the two bounds
        // applies and its resolved value.
        let spent = Bound::spent(Seconds::new(NonZeroU64::new(7).expect("7 is positive")));
        let condition = expired("shop", NetworkPhase::TcpConnect, HOST, PORT, spent);

        match condition {
            Error::NetworkDeadlineExceeded {
                phase,
                bound,
                limit,
                ..
            } => {
                assert_eq!(phase, NetworkPhase::TcpConnect);
                assert_eq!(bound, DeadlineBound::Overall);
                assert_eq!(limit.as_secs(), 7);
            }
            other => panic!("expected a deadline, got {other:?}"),
        }
    }

    // ------------------------------------------- against the fixture ---

    /// The privileged account of the fixture, which every database is visible
    /// to.
    const ROOT: (&str, &str) = ("root", "tpl-root");

    /// The reduced-grant reader, which holds `SELECT, EXECUTE ON freight.*`
    /// and nothing else — so `mysql` is a database it may not see.
    const REDUCED: (&str, &str) = ("tpl_reader", "tpl-reader-pw");

    /// A database no server of the fixture holds.
    const ABSENT: &str = "a_database_no_server_of_the_fixture_holds";

    /// A database every server holds and the reduced reader holds no grant on.
    const UNGRANTED: &str = "mysql";

    /// The schema the fixture carries, which every server shows to both
    /// accounts above.
    const PRESENT: &str = "freight";

    /// The condition opening a connection to `server` produces, as the whole
    /// path produces it.
    ///
    /// It goes through [`crate::mariadb::open`] rather than through
    /// [`connecting`] directly, because the subject is the classification
    /// **reached from a real handshake**: a unit test over a hand-built driver
    /// error would assert what this module does with a value this module also
    /// invented.
    fn opening(
        server: &fixture::Server,
        account: (&str, &str),
        database: &str,
    ) -> Result<(), Error> {
        use crate::mariadb::connect::Target;
        use crate::project::config;
        use crate::project::scratch::Scratch;
        use crate::project::settings;

        let (host, port) = server
            .address()
            .rsplit_once(':')
            .expect("status.sh --export prints host:port");
        let (user, password) = account;

        let scratch = Scratch::new();
        let file = scratch.file(
            ".cfg",
            &format!(
                "[database.fixture]\nhost = \"{host}\"\nport = {port}\n\
                 user = \"{user}\"\npassword = \"{password}\"\n\
                 database = \"{database}\"\ntls = \"disabled\"\n"
            ),
        );
        let configuration = config::load(&file).expect("the document is valid");
        let resolved = settings::resolve(
            &configuration,
            Some("fixture"),
            &settings::clock(None),
            &|_: &str| None,
        )
        .expect("the entry resolves");
        let target = Target::of(&resolved).expect("the entry names a host");

        crate::mariadb::open(&target, &settings::clock(None)).map(super::super::Session::close)
    }

    /// Runs `body` against every series of `FR-SRV-015`, or reports the skip.
    fn on_every_series(test: &str, body: impl Fn(&fixture::Server)) {
        let Some(series) = fixture::series(test) else {
            return;
        };
        let _exclusive = fixture::exclusive();

        for server in series {
            body(server);
        }
    }

    #[test]
    fn fr_priv_021_a_database_the_server_does_not_hold_is_not_a_refused_credential() {
        // FR-PRIV-021, observed on all four series on 2026-09-18: the server
        // answers error 1049 and the credentials were accepted, so the caller
        // is told which database it could not read rather than that its
        // credentials were refused.
        on_every_series(
            "fr_priv_021_a_database_the_server_does_not_hold_is_not_a_refused_credential",
            |server| {
                let condition = opening(server, ROOT, ABSENT).expect_err("the database is absent");

                assert_eq!(condition.exit_code(), 77, "{}", server.name());
                match &condition {
                    Error::PropertyNotReadable {
                        kind,
                        object,
                        property,
                    } => {
                        assert_eq!(*kind, CatalogueObjectKind::Database, "{}", server.name());
                        assert_eq!(object, ABSENT, "{}", server.name());
                        assert_eq!(*property, "metadata", "{}", server.name());
                    }
                    other => panic!("{}: expected FR-PRIV-021, got {other:?}", server.name()),
                }
            },
        );
    }

    #[test]
    fn fr_priv_021_a_database_the_reader_may_not_see_reaches_the_same_condition() {
        // FR-PRIV-021 declines to separate the two explanations, and the
        // handshake offers no way to: error 1044 for a database the reader
        // holds no grant on, 1049 for one that is not there. Observed on all
        // four series on 2026-09-18.
        on_every_series(
            "fr_priv_021_a_database_the_reader_may_not_see_reaches_the_same_condition",
            |server| {
                let condition =
                    opening(server, REDUCED, UNGRANTED).expect_err("the grant is absent");

                assert_eq!(condition.exit_code(), 77, "{}", server.name());
                match &condition {
                    Error::PropertyNotReadable { kind, object, .. } => {
                        assert_eq!(*kind, CatalogueObjectKind::Database, "{}", server.name());
                        assert_eq!(object, UNGRANTED, "{}", server.name());
                    }
                    other => panic!("{}: expected FR-PRIV-021, got {other:?}", server.name()),
                }
            },
        );
    }

    #[test]
    fn fr_err_034_a_refused_credential_is_still_reported_as_one() {
        // The 77 row of FR-ERR-034: for authentication, the user and the host
        // the server refused. Error 1045, observed on all four series on
        // 2026-09-18, for a password that is wrong and for a user that does
        // not exist alike — and the separation of FR-PRIV-021 does not reach
        // either.
        on_every_series(
            "fr_err_034_a_refused_credential_is_still_reported_as_one",
            |server| {
                for account in [("root", "not-the-password"), ("nobody_at_all", "x")] {
                    let condition =
                        opening(server, account, PRESENT).expect_err("the credentials are wrong");

                    assert_eq!(condition.exit_code(), 77, "{}", server.name());
                    match &condition {
                        Error::AuthenticationRefused { user, .. } => {
                            assert_eq!(user, account.0, "{}", server.name());
                        }
                        other => panic!(
                            "{}: expected a refused credential, got {other:?}",
                            server.name()
                        ),
                    }
                }
            },
        );
    }

    #[test]
    fn fr_conf_041_a_database_the_entry_names_and_the_reader_can_see_opens() {
        // The control for the three above: the same path, with the database
        // FR-CONF-041 puts on the connection present and visible, opens and
        // classifies nothing.
        on_every_series(
            "fr_conf_041_a_database_the_entry_names_and_the_reader_can_see_opens",
            |server| {
                opening(server, ROOT, PRESENT).expect("the fixture schema is there");
                opening(server, REDUCED, PRESENT).expect("the reduced reader holds SELECT on it");
            },
        );
    }
}
