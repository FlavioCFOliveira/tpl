//! Where the driver's error stops for a `mysql` entry (`FR-MYSQL-045`,
//! `FR-ERR-047`, `FR-ERR-048`, `ADR-014`).
//!
//! A `mysql` entry is connected exactly as a `mariadb` entry
//! (`FR-MYSQL-020`), so every failure of the connection is classified by
//! `crate::mariadb::fault::connecting`, which this module calls with the
//! provider: the table of `FR-ERR-048` is read for a `mysql` entry, the
//! `cause` of `FR-ERR-047` names a MySQL server, the `hint` of a refused TLS
//! handshake is `FR-MYSQL-021`'s, and the refusal of `ADR-014` is
//! `FR-MYSQL-045`'s `78`.
//!
//! **The refusal is recognised by its type, never by its text** (`ADR-014`,
//! point 4), in the shared module. For a `mariadb` entry the same refusal is
//! the third outcome of `FR-ERR-047`: the same facts and command, at exit
//! `69`, per the note on `FR-SEC-021`.

use sqlx_mysql::ConnectPhase;

use crate::error::Error;
use crate::mariadb::connect::Target;
use crate::mariadb::fault::{self as shared, Reached};
use crate::model::database::Provider;

/// The condition a failure to open a `mysql` session is reported as.
///
/// `user` is the user actually presented to the server, as the caller read it
/// back from the options it composed, and `phase` the phase the connection had
/// reached (`FR-CONF-005`).
pub(crate) fn connecting(
    target: &Target<'_>,
    driver: &sqlx_core::Error,
    user: &str,
    phase: ConnectPhase,
) -> Error {
    shared::connecting(&Reached::of(target, user, Provider::Mysql), driver, phase)
}

#[cfg(test)]
mod tests {
    use crate::mariadb::fault::public_key_request;
    use std::io;

    #[test]
    fn adr_014_a_configuration_error_of_another_origin_is_not_the_refusal() {
        // ADR-014, point 4: the variant alone is not the condition, because
        // the TLS set-up returns `Error::Configuration` too.
        let other = sqlx_core::Error::Configuration("client certificate rejected".into());
        let io = sqlx_core::Error::Io(io::Error::from(io::ErrorKind::ConnectionRefused));

        assert!(public_key_request(&other).is_none());
        assert!(public_key_request(&io).is_none());
    }
}
