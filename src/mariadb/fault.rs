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
//! | An error packet during the handshake | [`Error::AuthenticationRefused`], `77` | The server refused this connection for these credentials |
//! | A TLS failure | [`Error::TlsHandshakeFailed`], `69` | `FR-ERR-034`, the `69` row, which names the TLS handshake as a phase |
//! | Anything else, once the address is known | [`Error::ConnectionRefused`], `69` | The session did not open, and `FR-ERR-001` puts an unavailable server on `69` |
//! | A phase that outlived its bound | [`Error::NetworkDeadlineExceeded`], `69` | `FR-ERR-027`, `FR-GLOB-012`, `FR-GLOB-013` |
//!
//! *The driver's error is read for its discriminant and for nothing else.*
//! Point 3 of `OD-12` separates the TCP connect from the TLS handshake by the
//! driver's own variant rather than by the call site, because
//! `MySqlConnectOptions` has no method that accepts an already-connected
//! socket and one call therefore covers both phases. The discriminant is the
//! whole of what is read: no message, no code and no chain crosses this
//! boundary.
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

use crate::deadline::Bound;
use crate::error::{Error, NetworkPhase};

/// The condition a failure to open the session is reported as.
///
/// `user` is the user actually presented to the server, which the caller reads
/// back from the options it composed rather than from the entry, so that a
/// refusal names what was sent.
pub(crate) fn connecting(error: &sqlx::Error, host: &str, port: u16, user: &str) -> Error {
    // Every error packet a server sends during the handshake refuses the
    // connection for the credentials presented — access denied for the user,
    // for the database it named, for the host it came from, or for the
    // authentication plugin it offered. All four are `77` (`EX_NOPERM`) and
    // none of them is an availability failure.
    if matches!(error, sqlx::Error::Database(_)) {
        return Error::AuthenticationRefused {
            user: user.to_owned(),
            host: host.to_owned(),
        };
    }

    speaking(error, host, port)
}

/// The condition a failure of a statement on an open session is reported as.
///
/// The name resolution and the handshake are behind it, so a `Database` error
/// here is the server answering the statement rather than refusing the
/// connection, and the caller that knows which statement it issued classifies
/// that outcome itself — the read-only pair of `FR-SRV-008` and `FR-SRV-009`
/// is `78`, not `69`, whatever the server said.
pub(crate) fn speaking(error: &sqlx::Error, host: &str, port: u16) -> Error {
    if tls(error) {
        return Error::TlsHandshakeFailed {
            host: host.to_owned(),
            port,
        };
    }

    Error::ConnectionRefused {
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
fn tls(error: &sqlx::Error) -> bool {
    match error {
        sqlx::Error::Tls(_) => true,
        sqlx::Error::Io(returned) => returned.kind() == std::io::ErrorKind::InvalidData,
        _ => false,
    }
}

/// The condition a phase that outlived its bound is reported as.
///
/// `FR-ERR-034` obliges the `cause` line to name which of the two bounds of
/// `FR-GLOB-012` expired and its resolved value, and [`Bound`] carries both
/// because the composition was made where the two were known.
pub(crate) fn expired(phase: NetworkPhase, host: &str, port: u16, bound: Bound) -> Error {
    Error::NetworkDeadlineExceeded {
        phase,
        host: host.to_owned(),
        port,
        bound: bound.bound(),
        limit: bound.limit(),
    }
}

#[cfg(test)]
mod tests {
    use super::{connecting, expired, speaking};
    use crate::deadline::{Bound, Seconds};
    use crate::error::{DeadlineBound, Error, NetworkPhase};
    use std::io;
    use std::num::NonZeroU64;

    /// The host and port every condition below names.
    const HOST: &str = "db.example.com";
    const PORT: u16 = 3306;

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
        let condition = connecting(&refused_tls(), HOST, PORT, "alice");

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
        let condition = connecting(&untrusted_certificate(), HOST, PORT, "alice");

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
                    connecting(&driver, HOST, PORT, "alice"),
                    Error::ConnectionRefused { .. }
                ),
                "{kind:?}"
            );
        }
    }

    #[test]
    fn fr_err_001_a_server_that_does_not_answer_is_unavailable_rather_than_misconfigured() {
        let condition = connecting(&unreachable(), HOST, PORT, "alice");

        assert!(matches!(condition, Error::ConnectionRefused { .. }));
        assert_eq!(condition.exit_code(), 69);
    }

    #[test]
    fn fr_glob_018_no_driver_message_survives_the_classification() {
        // FR-GLOB-018 and OD-06: the driver's own text reaches no stream,
        // because the value it lived on is dropped at this boundary.
        let driver = sqlx::Error::Tls("a message the driver composed".into());
        let condition = connecting(&driver, HOST, PORT, "alice");

        assert!(!format!("{condition}").contains("a message the driver composed"));
        assert!(!format!("{condition:?}").contains("a message the driver composed"));
    }

    #[test]
    fn fr_err_034_a_statement_that_fails_is_not_read_as_a_refused_credential() {
        // On an open session the handshake is behind us, so nothing here is a
        // 77: the session did not hold, which is 69.
        let condition = speaking(&unreachable(), HOST, PORT);

        assert_eq!(condition.exit_code(), 69);
    }

    #[test]
    fn fr_err_034_an_expired_phase_names_the_bound_that_expired_and_its_value() {
        // FR-ERR-034, the 69 row, with FR-GLOB-012: which of the two bounds
        // applies and its resolved value.
        let spent = Bound::spent(Seconds::new(NonZeroU64::new(7).expect("7 is positive")));
        let condition = expired(NetworkPhase::TcpConnect, HOST, PORT, spent);

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
}
