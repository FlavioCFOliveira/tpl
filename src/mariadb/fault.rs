//! Where the driver's own error stops, and a condition of `FR-ERR-001` begins,
//! for both server providers.
//!
//! `FR-GLOB-018` bars the raw driver error from every diagnostic stream at
//! every verbosity, and `OD-06` makes that structural by refusing it a home
//! inside [`Error`]: no variant carries it and no conversion from it exists.
//! This module is the boundary those two describe. A driver failure arrives
//! here, is **classified**, and the original value is dropped. Three numbers
//! cross the boundary, and only where a requirement obliges a `cause` to name
//! one: the error number of a server's error packet (`FR-ERR-036`,
//! `FR-ERR-048`) and the error number the operating system reports for the
//! TCP connect (`FR-ERR-049`), per `FR-GLOB-018` as the seventy-seventh
//! edition amended it. No message, no SQLSTATE and no chain of the driver's
//! crosses it.
//!
//! `crate::mysql::fault` shares it: a `mysql` entry is connected exactly as a
//! `mariadb` entry (`FR-MYSQL-020`), and the provider is passed in where a
//! condition differs by it — the table of `FR-ERR-048`, the server the `cause`
//! of `FR-ERR-047` names, the code of the refusal of `ADR-014`, and the `hint`
//! of a refused TLS handshake.
//!
//! # Opening the session: by the phase the connection reached
//!
//! `FR-CONF-005` decides the phase by what the connection had completed, and
//! not by the call that was running, and one call to the driver opens and logs
//! in. The driver records the phase it reaches in the handle `ADR-014` point 6
//! adds — the TCP connect, the TLS handshake, the login — and the caller hands
//! that reading to [`connecting`] with the driver's error.
//!
//! | What the driver returned | Phase | Condition | Forced by |
//! |---|---|---|---|
//! | The refusal of `ADR-014` (by type) | the login | [`Error::PublicKeyRequestRefused`]: `78` for `mysql`, `69` for `mariadb` | `FR-MYSQL-045`, `FR-ERR-047` outcome 3 |
//! | An error packet | any | By its number and the provider | `FR-ERR-048` |
//! | `Error::Tls` | any | [`Error::TlsHandshakeFailed`], refused | `FR-ERR-034` |
//! | `Error::Io` | the TCP connect | By the operating system's error number | `FR-ERR-049` |
//! | `Error::Io` of kind `InvalidData` | the TLS handshake | [`Error::TlsHandshakeFailed`], certificate rejected | `FR-ERR-034` |
//! | Any other error | the TLS handshake | [`Error::TlsHandshakeFailed`], refused | `FR-ERR-034` |
//! | `Error::Io` | the login | [`Error::LoginNotCompleted`], the connection closed | `FR-ERR-047` outcome 1 |
//! | Any other error | the login | [`Error::LoginNotCompleted`], an answer not accepted | `FR-ERR-047` outcome 2 |
//!
//! *The `InvalidData` kind still separates a certificate the trust material
//! does not vouch for from a handshake refused before any certificate*, as it
//! did on the observation of 2026-09-18 against `scripts/mariadb/`, where
//! `verify-ca` and `verify-identity` against an untrusted certificate returned
//! that kind and a server offering no TLS under `required` returned
//! `Error::Tls`. The phase now says that the TLS handshake was running, so the
//! kind no longer has to say it alone.
//!
//! # A statement on an open session
//!
//! [`speaking`] classifies a failure of the version probe or of a catalogue
//! statement into the three ways of `FR-ERR-036`: an `Error::Io` of any kind,
//! or an `Error::Tls`, is the connection failing; an error packet is its
//! number; and anything else is an answer outside the protocol. The read-only
//! pair (`78`) and the privilege probe's `false` are classified by their
//! callers before this is reached.

// The harness is asked rather than restated, for the reason
// `super::catalogue` gives where it declares the same module.
#[cfg(test)]
use super::catalogue::fixture;

use rustix::io::Errno;
use sqlx_mysql::{ConnectPhase, MySqlDatabaseError, PublicKeyRequestRefused};

use super::connect::Target;
use crate::deadline::Bound;
use crate::error::{
    AccountRefusal, CatalogueObjectKind, Error, LoginFault, LoginRefusal, NetworkPhase,
    SessionStatement, StatementFault, TcpFault, TlsFault,
};
use crate::model::database::Provider;
use crate::project::config::entry::TlsMode;

/// The property `FR-PRIV-021` reports unreadable, as its `cause` line names it.
///
/// It is `pub(crate)` because two paths reach that one condition and the
/// requirement gives it one `cause`: the login classifies the `1049` and
/// `1044` packets here, and the folds report a schema catalogue that returned
/// no row.
pub(crate) const METADATA: &str = "metadata";

/// What a connection attempt reached, for the classification of its failure.
#[derive(Debug, Clone, Copy)]
pub(crate) struct Reached<'a> {
    /// The database entry the connection was opened for.
    pub(crate) entry: &'a str,
    /// The host attempted.
    pub(crate) host: &'a str,
    /// The port attempted.
    pub(crate) port: u16,
    /// The user actually presented to the server.
    pub(crate) user: &'a str,
    /// The database the connection named (`FR-CONF-041`), where it named one.
    pub(crate) database: Option<&'a str>,
    /// The TLS mode in force (`FR-CONF-013`).
    pub(crate) tls: TlsMode,
    /// The entry's provider.
    pub(crate) provider: Provider,
}

impl<'a> Reached<'a> {
    /// What `target` reached, as `user`, for an entry of `provider`.
    pub(crate) fn of(target: &Target<'a>, user: &'a str, provider: Provider) -> Self {
        Self {
            entry: target.entry(),
            host: target.host(),
            port: target.port(),
            user,
            database: target.database(),
            tls: target.tls(),
            provider,
        }
    }
}

/// The network phase the driver's phase handle names (`FR-CONF-005`).
pub(crate) const fn phase_of(reached: ConnectPhase) -> NetworkPhase {
    match reached {
        ConnectPhase::TcpConnect => NetworkPhase::TcpConnect,
        ConnectPhase::TlsHandshake => NetworkPhase::TlsHandshake,
        ConnectPhase::Login => NetworkPhase::Login,
    }
}

/// The refusal of `ADR-014`, where `driver` is it.
///
/// The variant is read first and the boxed value is then downcast to the
/// patch's type, so a configuration error of any other origin is not this
/// condition (`ADR-014`, point 4).
pub(crate) fn public_key_request(driver: &sqlx_core::Error) -> Option<&PublicKeyRequestRefused> {
    match driver {
        sqlx_core::Error::Configuration(boxed) => boxed.downcast_ref::<PublicKeyRequestRefused>(),
        _ => None,
    }
}

/// The condition a failure to open the session is reported as, in the phase
/// the connection had reached (`FR-ERR-047`, `FR-ERR-048`, `FR-ERR-049`).
pub(crate) fn connecting(
    reached: &Reached<'_>,
    driver: &sqlx_core::Error,
    phase: ConnectPhase,
) -> Error {
    if let Some(refused) = public_key_request(driver) {
        return Error::PublicKeyRequestRefused {
            provider: reached.provider,
            entry: reached.entry.to_owned(),
            user: reached.user.to_owned(),
            host: reached.host.to_owned(),
            port: reached.port,
            tls: reached.tls.name(),
            // Raised only on a stream that is not TLS, which `preferred`
            // leaves so only where the server offers none (ADR-014, point 4).
            no_tls_offered: reached.tls == TlsMode::Preferred,
            method: refused.plugin(),
        };
    }

    match (driver, phase) {
        (sqlx_core::Error::Database(packet), _) => {
            match packet.try_downcast_ref::<MySqlDatabaseError>() {
                Some(packet) => refused_login(reached, packet.number()),
                None => not_completed(reached, LoginFault::NotAccepted),
            }
        }
        (sqlx_core::Error::Tls(_), _) => tls_failed(reached, TlsFault::Refused),
        (sqlx_core::Error::Io(returned), ConnectPhase::TcpConnect) => tcp(reached, returned),
        (sqlx_core::Error::Io(returned), ConnectPhase::TlsHandshake)
            if returned.kind() == std::io::ErrorKind::InvalidData =>
        {
            tls_failed(reached, TlsFault::CertificateRejected)
        }
        (_, ConnectPhase::TlsHandshake) => tls_failed(reached, TlsFault::Refused),
        (sqlx_core::Error::Io(_), ConnectPhase::Login) => {
            not_completed(reached, LoginFault::ConnectionClosed)
        }
        (_, ConnectPhase::Login) => not_completed(reached, LoginFault::NotAccepted),
        // The driver opens the socket and returns an I/O error alone before
        // the login begins; anything else here names no number, and is the
        // fourth outcome of FR-ERR-049 without one.
        (_, ConnectPhase::TcpConnect) => tcp_failed(reached, TcpFault::Other(None)),
    }
}

/// The condition of an error packet the server ended the login with, by its
/// number read for the entry's provider (`FR-ERR-048`).
pub(crate) fn refused_login(reached: &Reached<'_>, number: u16) -> Error {
    let account = |reason| Error::AccountRefused {
        entry: reached.entry.to_owned(),
        by_dsn: false,
        user: reached.user.to_owned(),
        host: reached.host.to_owned(),
        reason,
    };
    let limit = |reason| Error::LoginRefused {
        entry: reached.entry.to_owned(),
        host: reached.host.to_owned(),
        port: reached.port,
        number,
        reason,
    };

    match (reached.provider, number) {
        // FR-PRIV-021: the two explanations it declines to separate. A
        // connection that names no database receives neither.
        (_, 1044 | 1049) => match reached.database {
            Some(database) => Error::PropertyNotReadable {
                kind: CatalogueObjectKind::Database,
                object: database.to_owned(),
                property: METADATA,
            },
            None => limit(LoginRefusal::Unclassified),
        },
        (_, 1045 | 1698) => Error::AuthenticationRefused {
            entry: reached.entry.to_owned(),
            user: reached.user.to_owned(),
            host: reached.host.to_owned(),
        },
        (_, 1130) => account(AccountRefusal::HostNotPrivileged),
        (_, 1862) => account(AccountRefusal::PasswordExpired),
        (Provider::Mariadb, 4151) | (Provider::Mysql, 3118) => account(AccountRefusal::Locked),
        (Provider::Mariadb, 4150) | (Provider::Mysql, 3955) => account(AccountRefusal::Blocked),
        (_, 3159) => Error::SecureTransportRequired {
            entry: reached.entry.to_owned(),
            host: reached.host.to_owned(),
            port: reached.port,
            tls: reached.tls.name(),
        },
        (_, 1040) => limit(LoginRefusal::ServerConnectionLimit),
        (_, 1203) => limit(LoginRefusal::UserConnectionLimit),
        (_, 1226) => limit(LoginRefusal::ResourceLimit),
        (_, 1129) => limit(LoginRefusal::HostBlocked),
        _ => limit(LoginRefusal::Unclassified),
    }
}

/// The condition of a TCP connect the operating system ended, by its error
/// number, or by the kind where it gave none (`FR-ERR-049`).
fn tcp(reached: &Reached<'_>, returned: &std::io::Error) -> Error {
    use std::io::ErrorKind;

    let fault = match returned.raw_os_error() {
        Some(number) => match Errno::from_raw_os_error(number) {
            Errno::CONNREFUSED => return refused(reached),
            Errno::HOSTUNREACH | Errno::HOSTDOWN => TcpFault::HostUnreachable,
            Errno::NETUNREACH | Errno::NETDOWN => TcpFault::NetworkUnreachable,
            Errno::TIMEDOUT => TcpFault::NoAnswer,
            _ => TcpFault::Other(Some(number)),
        },
        None => match returned.kind() {
            ErrorKind::ConnectionRefused => return refused(reached),
            ErrorKind::HostUnreachable => TcpFault::HostUnreachable,
            ErrorKind::NetworkUnreachable | ErrorKind::NetworkDown => TcpFault::NetworkUnreachable,
            ErrorKind::TimedOut => TcpFault::NoAnswer,
            _ => TcpFault::Other(None),
        },
    };

    tcp_failed(reached, fault)
}

/// The first outcome of `FR-ERR-049`, the message the implementation has
/// printed for `ECONNREFUSED` since before it.
fn refused(reached: &Reached<'_>) -> Error {
    Error::ConnectionRefused {
        by_dsn: false,
        entry: reached.entry.to_owned(),
        host: reached.host.to_owned(),
        port: reached.port,
    }
}

/// Outcomes 2 to 4 of `FR-ERR-049`.
fn tcp_failed(reached: &Reached<'_>, fault: TcpFault) -> Error {
    Error::TcpConnectFailed {
        entry: reached.entry.to_owned(),
        by_dsn: false,
        host: reached.host.to_owned(),
        port: reached.port,
        fault,
    }
}

/// Outcomes 1 and 2 of `FR-ERR-047`.
fn not_completed(reached: &Reached<'_>, fault: LoginFault) -> Error {
    Error::LoginNotCompleted {
        entry: reached.entry.to_owned(),
        by_dsn: false,
        host: reached.host.to_owned(),
        port: reached.port,
        provider: reached.provider,
        fault,
    }
}

/// A TLS handshake that did not complete.
fn tls_failed(reached: &Reached<'_>, fault: TlsFault) -> Error {
    Error::TlsHandshakeFailed {
        provider: reached.provider,
        entry: reached.entry.to_owned(),
        host: reached.host.to_owned(),
        port: reached.port,
        fault,
    }
}

/// The condition a failure of a statement on an open session is reported as
/// (`FR-ERR-036`).
///
/// The callers classify the conditions routed elsewhere before this is
/// reached: the read-only pair is `78`, and an error packet the privilege
/// probe receives is its `false`.
pub(crate) fn speaking(
    entry: &str,
    error: &sqlx_core::Error,
    host: &str,
    port: u16,
    statement: SessionStatement,
) -> Error {
    let fault = match error {
        sqlx_core::Error::Io(_) | sqlx_core::Error::Tls(_) => StatementFault::ConnectionFailed,
        sqlx_core::Error::Database(packet) => {
            match packet.try_downcast_ref::<MySqlDatabaseError>() {
                Some(packet) => StatementFault::ServerError(packet.number()),
                None => StatementFault::OutsideProtocol,
            }
        }
        _ => StatementFault::OutsideProtocol,
    };

    Error::StatementFailed {
        entry: entry.to_owned(),
        host: host.to_owned(),
        port,
        statement,
        fault,
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
        by_dsn: false,
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
    use super::{Reached, connecting, expired, speaking};
    use crate::deadline::{Bound, Seconds};
    use crate::error::{
        CatalogueObjectKind, DeadlineBound, Error, NetworkPhase, SessionStatement, TcpFault,
    };
    use crate::model::database::Provider;
    use crate::project::config::entry::TlsMode;
    use sqlx_mysql::ConnectPhase;
    use std::io;
    use std::num::NonZeroU64;

    /// The host and port every condition below names.
    const HOST: &str = "db.example.com";
    const PORT: u16 = 3306;

    /// The database the selected entry names (`FR-CONF-041`), which every
    /// condition below is classified against.
    const DATABASE: &str = "freight";

    /// What a `mariadb` entry `shop` reached, as `alice`, under `tls`.
    fn reached(tls: TlsMode) -> Reached<'static> {
        Reached {
            entry: "shop",
            host: HOST,
            port: PORT,
            user: "alice",
            database: Some(DATABASE),
            tls,
            provider: Provider::Mariadb,
        }
    }

    /// A driver error of the kind an unreachable server produces.
    fn unreachable() -> sqlx_core::Error {
        sqlx_core::Error::Io(io::Error::from(io::ErrorKind::ConnectionRefused))
    }

    /// A driver error of the kind a refused TLS negotiation produces.
    fn refused_tls() -> sqlx_core::Error {
        sqlx_core::Error::Tls("server does not support TLS".into())
    }

    /// A driver error of the kind a certificate the trust material does not
    /// vouch for produces, as observed on 2026-09-18.
    fn untrusted_certificate() -> sqlx_core::Error {
        sqlx_core::Error::Io(io::Error::new(
            io::ErrorKind::InvalidData,
            "invalid peer certificate: UnknownIssuer",
        ))
    }

    #[test]
    fn fr_err_034_a_tls_failure_is_reported_as_the_tls_handshake_phase() {
        // FR-ERR-034, the 69 row: the cause names the phase that failed, and
        // OD-12 point 3 derives it from the driver's discriminant.
        let condition = connecting(
            &reached(TlsMode::VerifyIdentity),
            &refused_tls(),
            ConnectPhase::TlsHandshake,
        );

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
            &reached(TlsMode::VerifyIdentity),
            &untrusted_certificate(),
            ConnectPhase::TlsHandshake,
        );

        assert!(matches!(condition, Error::TlsHandshakeFailed { .. }));
        assert_eq!(condition.exit_code(), 69);
    }

    #[test]
    fn fr_err_034_an_ordinary_socket_failure_is_not_read_as_the_tls_phase() {
        // The kind is the discriminant, and no kind the operating system
        // produces for a socket is InvalidData. Amended for FR-ERR-049 in the
        // seventy-seventh edition: a TCP connect that fails is the refusal of
        // the first outcome only for ECONNREFUSED; every other report is its
        // own outcome, and none is the TLS phase.
        for (kind, expected) in [
            (io::ErrorKind::ConnectionRefused, None),
            (io::ErrorKind::ConnectionReset, Some(TcpFault::Other(None))),
            (io::ErrorKind::TimedOut, Some(TcpFault::NoAnswer)),
            (io::ErrorKind::UnexpectedEof, Some(TcpFault::Other(None))),
            (io::ErrorKind::BrokenPipe, Some(TcpFault::Other(None))),
        ] {
            let driver = sqlx_core::Error::Io(io::Error::from(kind));
            let condition = connecting(
                &reached(TlsMode::VerifyIdentity),
                &driver,
                ConnectPhase::TcpConnect,
            );

            assert_eq!(condition.exit_code(), 69, "{kind:?}");
            match (expected, &condition) {
                (None, Error::ConnectionRefused { .. }) => {}
                (Some(fault), Error::TcpConnectFailed { fault: found, .. }) => {
                    assert_eq!(*found, fault, "{kind:?}");
                }
                (_, other) => panic!("{kind:?}: {other:?}"),
            }
        }
    }

    #[test]
    fn fr_err_001_a_server_that_does_not_answer_is_unavailable_rather_than_misconfigured() {
        let condition = connecting(
            &reached(TlsMode::VerifyIdentity),
            &unreachable(),
            ConnectPhase::TcpConnect,
        );

        assert!(matches!(condition, Error::ConnectionRefused { .. }));
        assert_eq!(condition.exit_code(), 69);
    }

    #[test]
    fn fr_glob_018_no_driver_message_survives_the_classification() {
        // FR-GLOB-018 and OD-06: the driver's own text reaches no stream,
        // because the value it lived on is dropped at this boundary.
        let driver = sqlx_core::Error::Tls("a message the driver composed".into());
        let condition = connecting(
            &reached(TlsMode::VerifyIdentity),
            &driver,
            ConnectPhase::TlsHandshake,
        );

        assert!(!format!("{condition}").contains("a message the driver composed"));
        assert!(!format!("{condition:?}").contains("a message the driver composed"));
    }

    #[test]
    fn fr_err_034_a_statement_that_fails_is_not_read_as_a_refused_credential() {
        // On an open session the handshake is behind us, so nothing here is a
        // 77: the session did not hold, which is 69.
        let condition = speaking(
            "shop",
            &unreachable(),
            HOST,
            PORT,
            SessionStatement::CatalogueQuery,
        );

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
        .expect("the entry resolves")
        .into_mariadb()
        .expect("the entry is a mariadb entry");
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

    // ------------------------------------ the seventy-seventh edition, in process ---

    /// What a `mysql` entry `shop` reached, as `alice`, under `disabled`.
    fn reached_mysql() -> Reached<'static> {
        Reached {
            provider: Provider::Mysql,
            ..reached(TlsMode::Disabled)
        }
    }

    /// A driver error carrying the operating system's error `errno`.
    fn os(errno: rustix::io::Errno) -> sqlx_core::Error {
        sqlx_core::Error::Io(io::Error::from_raw_os_error(errno.raw_os_error()))
    }

    #[test]
    fn fr_conf_005_the_phase_is_the_one_the_driver_recorded() {
        // FR-CONF-005: the phase is decided by what the connection had
        // completed, read from the handle of ADR-014 point 6.
        assert_eq!(
            super::phase_of(ConnectPhase::TcpConnect),
            NetworkPhase::TcpConnect
        );
        assert_eq!(
            super::phase_of(ConnectPhase::TlsHandshake),
            NetworkPhase::TlsHandshake
        );
        assert_eq!(super::phase_of(ConnectPhase::Login), NetworkPhase::Login);

        // A new handle holds the TCP connect.
        assert_eq!(
            sqlx_mysql::ConnectPhaseHandle::new().phase(),
            ConnectPhase::TcpConnect
        );
    }

    #[test]
    fn fr_err_049_each_report_of_the_operating_system_is_its_own_outcome() {
        use rustix::io::Errno;

        let classify = |driver: &sqlx_core::Error| {
            connecting(
                &reached(TlsMode::Disabled),
                driver,
                ConnectPhase::TcpConnect,
            )
        };

        assert!(matches!(
            classify(&os(Errno::CONNREFUSED)),
            Error::ConnectionRefused { .. }
        ));
        for (errno, expected) in [
            (Errno::HOSTUNREACH, TcpFault::HostUnreachable),
            (Errno::HOSTDOWN, TcpFault::HostUnreachable),
            (Errno::NETUNREACH, TcpFault::NetworkUnreachable),
            (Errno::NETDOWN, TcpFault::NetworkUnreachable),
            (Errno::TIMEDOUT, TcpFault::NoAnswer),
            (
                Errno::ACCESS,
                TcpFault::Other(Some(Errno::ACCESS.raw_os_error())),
            ),
        ] {
            let condition = classify(&os(errno));

            assert_eq!(condition.exit_code(), 69, "{errno:?}");
            match condition {
                Error::TcpConnectFailed { fault, .. } => assert_eq!(fault, expected, "{errno:?}"),
                other => panic!("{errno:?}: {other:?}"),
            }
        }
    }

    #[test]
    fn fr_err_049_an_io_error_outside_the_tcp_connect_is_not_classified_as_one() {
        // FR-ERR-049 classifies the operating system's report only while the
        // handle says the TCP connect is in progress: the same ECONNREFUSED
        // during the login is a connection that closed (FR-ERR-047).
        use rustix::io::Errno;

        let condition = connecting(
            &reached(TlsMode::Disabled),
            &os(Errno::CONNREFUSED),
            ConnectPhase::Login,
        );

        assert!(matches!(
            condition,
            Error::LoginNotCompleted {
                fault: crate::error::LoginFault::ConnectionClosed,
                ..
            }
        ));
    }

    #[test]
    fn fr_err_047_the_two_outcomes_a_driver_error_reaches_in_the_login() {
        use crate::error::LoginFault;

        for (provider_reached, driver, expected) in [
            (
                reached(TlsMode::Disabled),
                sqlx_core::Error::Io(io::Error::from(io::ErrorKind::UnexpectedEof)),
                LoginFault::ConnectionClosed,
            ),
            (
                reached(TlsMode::Disabled),
                sqlx_core::Error::Io(io::Error::from(io::ErrorKind::ConnectionReset)),
                LoginFault::ConnectionClosed,
            ),
            (
                reached(TlsMode::Disabled),
                sqlx_core::Error::Protocol("an answer the driver does not read".to_owned()),
                LoginFault::NotAccepted,
            ),
            (
                reached_mysql(),
                sqlx_core::Error::Protocol("an answer the driver does not read".to_owned()),
                LoginFault::NotAccepted,
            ),
        ] {
            let condition = connecting(&provider_reached, &driver, ConnectPhase::Login);

            assert_eq!(condition.exit_code(), 69);
            match condition {
                Error::LoginNotCompleted {
                    fault, provider, ..
                } => {
                    assert_eq!(fault, expected);
                    assert_eq!(provider, provider_reached.provider);
                }
                other => panic!("{other:?}"),
            }
        }
    }

    #[test]
    fn fr_err_034_every_failure_while_the_tls_handshake_runs_is_the_tls_phase() {
        use crate::error::TlsFault;

        for (driver, expected) in [
            (refused_tls(), TlsFault::Refused),
            (untrusted_certificate(), TlsFault::CertificateRejected),
            (
                sqlx_core::Error::Io(io::Error::from(io::ErrorKind::ConnectionReset)),
                TlsFault::Refused,
            ),
        ] {
            match connecting(
                &reached(TlsMode::VerifyIdentity),
                &driver,
                ConnectPhase::TlsHandshake,
            ) {
                Error::TlsHandshakeFailed {
                    fault, provider, ..
                } => {
                    assert_eq!(fault, expected);
                    assert_eq!(provider, Provider::Mariadb);
                }
                other => panic!("{other:?}"),
            }
        }
    }

    #[test]
    fn fr_err_048_every_row_of_the_table_for_each_provider() {
        use super::refused_login;
        use crate::error::{AccountRefusal, LoginRefusal};

        #[derive(Debug, PartialEq)]
        enum Row {
            Database,
            Credentials,
            Account(AccountRefusal),
            Encrypted,
            Login(LoginRefusal),
        }

        let row = |reached: &Reached<'_>, number| match refused_login(reached, number) {
            Error::PropertyNotReadable { .. } => (Row::Database, 77),
            Error::AuthenticationRefused { .. } => (Row::Credentials, 77),
            Error::AccountRefused { reason, .. } => (Row::Account(reason), 77),
            Error::SecureTransportRequired { .. } => (Row::Encrypted, 78),
            Error::LoginRefused {
                reason,
                number: carried,
                ..
            } => {
                assert_eq!(carried, number);
                (Row::Login(reason), 69)
            }
            other => panic!("{number}: {other:?}"),
        };

        let mariadb = reached(TlsMode::Disabled);
        let mysql = reached_mysql();

        for (number, expected) in [
            (1044, Row::Database),
            (1049, Row::Database),
            (1045, Row::Credentials),
            (1698, Row::Credentials),
            (1130, Row::Account(AccountRefusal::HostNotPrivileged)),
            (1862, Row::Account(AccountRefusal::PasswordExpired)),
            (3159, Row::Encrypted),
            (1040, Row::Login(LoginRefusal::ServerConnectionLimit)),
            (1203, Row::Login(LoginRefusal::UserConnectionLimit)),
            (1226, Row::Login(LoginRefusal::ResourceLimit)),
            (1129, Row::Login(LoginRefusal::HostBlocked)),
            (1053, Row::Login(LoginRefusal::Unclassified)),
        ] {
            for reached in [&mariadb, &mysql] {
                let (found, code) = row(reached, number);
                assert_eq!(found, expected, "{number} for {}", reached.provider);
                let expected_code = match expected {
                    Row::Encrypted => 78,
                    Row::Login(_) => 69,
                    _ => 77,
                };
                assert_eq!(code, expected_code, "{number}");
            }
        }

        // Read for the entry's provider: the two products give these numbers
        // different meanings.
        assert_eq!(row(&mariadb, 4151).0, Row::Account(AccountRefusal::Locked));
        assert_eq!(row(&mariadb, 4150).0, Row::Account(AccountRefusal::Blocked));
        assert_eq!(row(&mysql, 3118).0, Row::Account(AccountRefusal::Locked));
        assert_eq!(row(&mysql, 3955).0, Row::Account(AccountRefusal::Blocked));
        assert_eq!(
            row(&mariadb, 3118).0,
            Row::Login(LoginRefusal::Unclassified)
        );
        assert_eq!(row(&mysql, 4151).0, Row::Login(LoginRefusal::Unclassified));
    }

    #[test]
    fn fr_err_036_a_statement_fails_in_one_of_three_ways() {
        use crate::error::StatementFault;

        for (driver, expected) in [
            (
                sqlx_core::Error::Io(io::Error::from(io::ErrorKind::ConnectionReset)),
                StatementFault::ConnectionFailed,
            ),
            (
                sqlx_core::Error::Io(io::Error::from(io::ErrorKind::InvalidData)),
                StatementFault::ConnectionFailed,
            ),
            (refused_tls(), StatementFault::ConnectionFailed),
            (
                sqlx_core::Error::Protocol("out of sequence".to_owned()),
                StatementFault::OutsideProtocol,
            ),
            (
                sqlx_core::Error::RowNotFound,
                StatementFault::OutsideProtocol,
            ),
        ] {
            for statement in [
                SessionStatement::VersionProbe,
                SessionStatement::CatalogueQuery,
            ] {
                match speaking("shop", &driver, HOST, PORT, statement) {
                    Error::StatementFailed {
                        fault,
                        statement: carried,
                        ..
                    } => {
                        assert_eq!(fault, expected, "{driver:?}");
                        assert_eq!(carried, statement);
                    }
                    other => panic!("{other:?}"),
                }
            }
        }
    }

    #[test]
    fn fr_err_048_a_second_login_beyond_the_account_limit_is_69_on_every_series() {
        // FR-ERR-048, observed: the fixture's tpl_limited holds
        // MAX_USER_CONNECTIONS 1, so a second login while the first is open is
        // refused with a limit on connections, which is 69 and not the 77 of a
        // refused credential.
        use crate::mariadb::connect::Target;
        use crate::project::config;
        use crate::project::scratch::Scratch;
        use crate::project::settings;

        on_every_series(
            "fr_err_048_a_second_login_beyond_the_account_limit_is_69_on_every_series",
            |server| {
                let (host, port) = server
                    .address()
                    .rsplit_once(':')
                    .expect("status.sh --export prints host:port");
                let scratch = Scratch::new();
                let file = scratch.file(
                    ".cfg",
                    &format!(
                        "[database.fixture]\nhost = \"{host}\"\nport = {port}\n\
                         user = \"tpl_limited\"\npassword = \"tpl-limited-pw\"\n\
                         database = \"freight\"\ntls = \"disabled\"\n"
                    ),
                );
                let configuration = config::load(&file).expect("the document is valid");
                let resolved = settings::resolve(
                    &configuration,
                    Some("fixture"),
                    &settings::clock(None),
                    &|_: &str| None,
                )
                .expect("the entry resolves")
                .into_mariadb()
                .expect("the entry is a mariadb entry");
                let target = Target::of(&resolved).expect("the entry names a host");
                let clock = settings::clock(None);

                let first = crate::mariadb::open(&target, &clock).expect("the first login opens");
                let second =
                    crate::mariadb::open(&target, &clock).map(super::super::Session::close);
                first.close();

                let condition = second.expect_err("the second login is beyond the limit");
                assert_eq!(
                    condition.exit_code(),
                    69,
                    "{}: {condition:?}",
                    server.name()
                );
                match condition {
                    Error::LoginRefused { number, reason, .. } => {
                        crate::mariadb::catalogue::fixture::notice(&format!(
                            "{}: the second login of tpl_limited was refused with server error \
                             {number}, classified {reason:?}",
                            server.name()
                        ));
                        // Observed on 2026-11-04 on all four series: 1226,
                        // ER_USER_LIMIT_REACHED, a resource limit of the
                        // account.
                        assert_eq!(number, 1226, "{}", server.name());
                        assert_eq!(
                            reason,
                            crate::error::LoginRefusal::ResourceLimit,
                            "{}",
                            server.name()
                        );
                    }
                    other => panic!("{}: {other:?}", server.name()),
                }
            },
        );
    }
}
