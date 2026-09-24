//! The one type a credential is carried in, and the one place it can be read
//! from.
//!
//! `FR-ERR-013` and `BR-ERR-003` bar a credential from every message at every
//! verbosity, and `FR-GLOB-018` bars it from every diagnostic stream. Both are
//! prohibitions on **printing**, and the way to hold a prohibition on printing
//! is to deny the value a printing implementation: [`Secret`] has a
//! [`Debug`](fmt::Debug) that writes a placeholder, no
//! [`Display`](std::fmt::Display), and no [`Serialize`](serde::Serialize). A
//! `{secret}` therefore does not compile, a `{secret:?}` inside a `format!`
//! that reaches a diagnostic writes nothing of the value, and the emitting path
//! of `output/` cannot take one at all.
//!
//! [`Secret::expose`] is the single deliberate exit. It is named to be
//! greppable, and the whole of what may call it is the code that hands the
//! credential to the database driver.
//!
//! [`Redacted`] is the same prohibition for a value that is **not** a
//! [`Secret`] and carries one all the same — the file's own bytes, a `password`
//! or `dsn` as the file wrote it, a field of a parsed DSN. Those are read as
//! text by the paths that validate and redact them, so they cannot be moved
//! into a [`Secret`]; what they can be denied is a derived
//! [`Debug`](fmt::Debug), and this is what the hand-written one writes in their
//! place.

use std::fmt;

/// What [`Secret`]'s [`Debug`](fmt::Debug) writes in place of the value.
const REDACTED: &str = "Secret(***)";

/// What a hand-written [`Debug`](fmt::Debug) writes in place of a field that
/// may carry a credential.
const REDACTED_FIELD: &str = "***";

/// A stand-in a hand-written [`Debug`](fmt::Debug) prints for a field it will
/// not disclose.
///
/// `debug_struct` takes a `&dyn Debug` per field, so a field that must not be
/// printed needs a value whose own [`Debug`](fmt::Debug) is the placeholder.
/// This is that value, held once so that every redacting implementation writes
/// the same bytes as [`crate::project::config::redact`] does on the printing
/// path.
pub(crate) struct Redacted;

impl fmt::Debug for Redacted {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(REDACTED_FIELD)
    }
}

/// A credential: a password from `.tpl/.cfg`, or the standard output of a
/// `password_command`.
///
/// It is deliberately poor at being printed. The derived [`Debug`](fmt::Debug)
/// is replaced, [`Display`](std::fmt::Display) is not implemented, and neither
/// is [`Serialize`](serde::Serialize) — so the type cannot reach a diagnostic
/// line, a `text` layout or a JSON document by any route the crate offers.
///
/// Equality is not implemented either: comparing two secrets is not something
/// this crate does, and an ordinary [`PartialEq`] would be a timing oracle
/// nobody asked for.
#[derive(Clone)]
pub(crate) struct Secret(String);

impl Secret {
    /// Takes ownership of a credential.
    pub(crate) const fn new(value: String) -> Self {
        Self(value)
    }

    /// The credential itself.
    ///
    /// The one route out of this type, named so that every use of a credential
    /// is one `grep` away. Nothing but the code that authenticates against the
    /// server may call it, and nothing that writes to a stream may.
    pub(crate) fn expose(&self) -> &str {
        &self.0
    }
}

impl fmt::Debug for Secret {
    /// Writes [`REDACTED`], whatever the value is.
    ///
    /// `FR-SEC-006` admits no credential in a message at any verbosity, and a
    /// derived [`Debug`](fmt::Debug) would put one in the first `{:?}` that
    /// formats a struct holding one.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(REDACTED)
    }
}

#[cfg(test)]
mod tests {
    use super::{REDACTED, REDACTED_FIELD, Redacted, Secret};

    #[test]
    fn fr_err_013_the_placeholder_a_hand_written_debug_writes_carries_no_value() {
        // The stand-in every redacting Debug of `project/config` prints. It
        // holds nothing, so there is nothing for it to disclose, and it writes
        // what the printing path of FR-CFG-021 writes.
        assert_eq!(format!("{Redacted:?}"), REDACTED_FIELD);
        assert_eq!(format!("{Redacted:#?}"), REDACTED_FIELD);
    }

    #[test]
    fn fr_err_013_the_debug_of_a_secret_carries_none_of_it() {
        // FR-ERR-013, FR-GLOB-018: a credential reaches no message and no
        // diagnostic stream, at any verbosity.
        let secret = Secret::new("hunter2".to_owned());

        assert_eq!(format!("{secret:?}"), REDACTED);
        assert!(!format!("{secret:?}").contains("hunter2"));
        assert!(!format!("{secret:#?}").contains("hunter2"));
    }

    #[test]
    fn fr_err_013_a_struct_holding_a_secret_carries_none_of_it_either() {
        // The derived Debug of a containing struct delegates to this one, so
        // the prohibition travels with the value rather than with the caller.
        #[derive(Debug)]
        struct Settings {
            #[allow(dead_code, reason = "the derived Debug is what this test reads")]
            user: &'static str,
            #[allow(dead_code, reason = "the derived Debug is what this test reads")]
            password: Secret,
        }

        let rendered = format!(
            "{:?}",
            Settings {
                user: "alice",
                password: Secret::new("hunter2".to_owned()),
            }
        );

        assert!(rendered.contains("alice"), "{rendered}");
        assert!(!rendered.contains("hunter2"), "{rendered}");
    }

    #[test]
    fn the_one_exit_returns_what_was_stored() {
        assert_eq!(Secret::new("hunter2".to_owned()).expose(), "hunter2");
    }
}
