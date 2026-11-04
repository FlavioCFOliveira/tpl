//! Where the driver's error stops for a `mysql` entry (`FR-MYSQL-045`,
//! `ADR-014`).
//!
//! A `mysql` entry is connected exactly as a `mariadb` entry
//! (`FR-MYSQL-020`), so every failure of the connection is classified as
//! `crate::mariadb::fault::connecting` classifies it — every row of that
//! module's table holds for a MySQL server — save one: the refusal the patch of
//! `ADR-014` adds to the driver, which the published crate does not have.
//!
//! | What the driver returned | What it becomes | Forced by |
//! |---|---|---|
//! | `Error::Configuration` boxing the driver's `PublicKeyRequestRefused` | [`Error::PublicKeyRequestRefused`], `78` | `FR-MYSQL-045` |
//! | Anything else | What `crate::mariadb::fault::connecting` makes of it | `FR-MYSQL-020` |
//!
//! **The refusal is recognised by its type, never by its text** (`ADR-014`,
//! point 4). The driver returns `Error::Configuration` for other conditions
//! too — the TLS set-up returns it for a client-certificate error — so the
//! variant alone is not the condition: the value it boxes is downcast to the
//! one type the patch declared for this refusal alone, and nothing of the
//! driver's message is read. The method's name is the one field read, and it
//! is the field `FR-MYSQL-045` obliges the `cause` to name.
//!
//! **A `mariadb` entry never reaches this module.** `ADR-014` holds a
//! `mariadb` entry to exactly the outcome the same login had before the patch,
//! `Error::ConnectionRefused`, exit `69`, with the same message; the
//! classification of `crate::mariadb::fault` reaches that row for the refusal
//! already, and a test there pins it.

use sqlx_mysql::PublicKeyRequestRefused;

use crate::error::Error;
use crate::mariadb::connect::Target;
use crate::mariadb::fault as shared;
use crate::project::config::entry::TlsMode;

/// The condition a failure to open a `mysql` session is reported as.
///
/// `user` is the user actually presented to the server, as the caller read it
/// back from the options it composed.
pub(crate) fn connecting(target: &Target<'_>, driver: &sqlx_core::Error, user: &str) -> Error {
    if let Some(refused) = public_key_request(driver) {
        return Error::PublicKeyRequestRefused {
            entry: target.entry().to_owned(),
            user: user.to_owned(),
            host: target.host().to_owned(),
            port: target.port(),
            tls: target.tls().name(),
            // The refusal is raised only on a stream that is not TLS, and
            // `preferred` leaves one so only where the server offers none
            // (ADR-014, point 4); under `disabled` nothing says whether it
            // does.
            no_tls_offered: target.tls() == TlsMode::Preferred,
            method: refused.plugin(),
        };
    }

    shared::connecting(
        target.entry(),
        driver,
        target.host(),
        target.port(),
        user,
        target.database(),
    )
}

/// The refusal of `ADR-014`, where `driver` is it.
///
/// The variant is read first and the boxed value is then downcast to the
/// patch's type, so a configuration error of any other origin is not this
/// condition.
pub(crate) fn public_key_request(driver: &sqlx_core::Error) -> Option<&PublicKeyRequestRefused> {
    match driver {
        sqlx_core::Error::Configuration(boxed) => boxed.downcast_ref::<PublicKeyRequestRefused>(),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::public_key_request;
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
