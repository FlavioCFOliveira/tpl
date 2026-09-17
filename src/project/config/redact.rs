//! The redaction of `FR-CFG-021`, applied on the printing path and nowhere
//! else.
//!
//! | Stored value | Printed as |
//! |---|---|
//! | a literal password | `***` |
//! | the password inside a DSN | `***`, with user, host, port and database left visible |
//! | `${VAR}` in any field | `${VAR}`, exactly as written |
//!
//! The third row is what keeps a password living in an environment variable off
//! stdout: printing the reference unexpanded discloses the name of a variable
//! and never its value. It applies to a value that **is** a reference; a value
//! that merely contains one is redacted, because printing `secret${SUFFIX}` as
//! written would disclose the literal half.
//!
//! **Redaction is a property of the printing path.** `FR-SEC-003` names
//! `tpl cfg list` and `tpl cfg database show` and `BR-CFG-002` names the one
//! deliberate exception, `tpl cfg get`, which is a directed read of a named
//! key. The two printers therefore call this module and the third does not; no
//! caller decides whether a value is a secret, and no value carries a flag
//! saying so.
//!
//! Two shapes are redacted, because the two commands print two shapes.
//! [`value`] redacts one value of one key, which is what the structured output
//! of `tpl cfg database show` and the document of `tpl cfg list` are built
//! from. [`document`] redacts the **file's own text**, which is what
//! `FR-CFG-013` prints literally: the spans the reader recorded are spliced,
//! so every byte the file carries outside a password — its comments, its key
//! order, its spacing — reaches the reader untouched.

use std::borrow::Cow;
use std::ops::Range;

use toml::de::DeTable;

use super::entry::Written;
use super::expand;
use super::keys::EntryKey;

/// What a redacted value is printed as (`FR-CFG-021`).
pub(crate) const REDACTED: &str = "***";

/// One span of the file's text, and what is printed in its place.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct Redaction {
    /// The byte range of the value token, quotes included.
    span: Range<usize>,
    /// The TOML token printed in its place.
    replacement: String,
}

/// `written` as `FR-CFG-021` prints it for `field`.
///
/// A key that cannot hold a credential is returned unchanged, which is every
/// key but `password` and `dsn`.
pub(crate) fn value<'a>(field: EntryKey, written: Written<'a>) -> Cow<'a, str> {
    match (field, written) {
        (EntryKey::Password, Written::Text(text)) => {
            if expand::is_whole_reference(text) {
                Cow::Borrowed(text)
            } else {
                Cow::Borrowed(REDACTED)
            }
        }
        (EntryKey::Dsn, Written::Text(text)) => match dsn(text) {
            Some(redacted) => Cow::Owned(redacted),
            None => Cow::Borrowed(text),
        },
        // A credential-bearing key whose value is not a string cannot be read
        // from a validated document: FR-CONF-002 declares both as strings and
        // the reader refuses anything else. The value is redacted whole rather
        // than printed, which is the safe direction.
        (EntryKey::Password | EntryKey::Dsn, _) => Cow::Borrowed(REDACTED),
        _ => written.line(),
    }
}

/// A DSN as `FR-CFG-021` prints it: the password replaced, the rest visible.
///
/// A value the grammar does not accept cannot reach this function through a
/// loaded document, because `FR-CONF-009` refuses it at the read; a value that
/// somehow did is redacted whole rather than printed, which is the safe
/// direction for a field that may carry a credential.
fn dsn(raw: &str) -> Option<String> {
    let Ok(parsed) = super::dsn::parse(raw, "", std::path::Path::new("")) else {
        return Some(REDACTED.to_owned());
    };

    let password = parsed.password()?;

    if expand::is_whole_reference(password.raw()) {
        return None;
    }

    Some(
        parsed
            .with_password(REDACTED)
            .unwrap_or_else(|| REDACTED.to_owned()),
    )
}

/// The spans of `text` that carry a credential, and what replaces each.
///
/// Called once, by the reader, with the table it has just validated: every DSN
/// it meets is therefore one the grammar accepted. A value that needs no
/// redaction contributes no span, so a file with no password is spliced not at
/// all.
pub(crate) fn spans(root: &DeTable<'_>, text: &str) -> Vec<Redaction> {
    let mut redactions = Vec::new();

    let Some(section) = root
        .get("database")
        .and_then(|section| section.get_ref().as_table())
    else {
        return redactions;
    };

    for block in section.values() {
        let Some(block) = block.get_ref().as_table() else {
            continue;
        };

        for field in [EntryKey::Password, EntryKey::Dsn] {
            let Some(found) = block.get(field.leaf()) else {
                continue;
            };
            let Some(written) = found.get_ref().as_str() else {
                continue;
            };

            let printed = value(field, Written::Text(written));
            if printed == written {
                continue;
            }

            let span = found.span();
            let quoted = text
                .get(span.clone())
                .is_some_and(|token| token.starts_with('\'') || token.starts_with('"'));

            redactions.push(Redaction {
                span,
                replacement: if quoted {
                    format!("\"{printed}\"")
                } else {
                    printed.into_owned()
                },
            });
        }
    }

    redactions.sort_by_key(|redaction| redaction.span.start);

    redactions
}

/// `text` with every redaction spliced into it (`FR-CFG-013`, `FR-CFG-021`).
///
/// Everything outside a redacted span is copied byte for byte, which is what
/// makes the output the file "literally": its comments, its key order and its
/// spacing survive, and only the value tokens that carry a credential do not.
pub(crate) fn document(text: &str, redactions: &[Redaction]) -> String {
    if redactions.is_empty() {
        return text.to_owned();
    }

    let mut printed = String::with_capacity(text.len());
    let mut copied = 0;

    for redaction in redactions {
        let Some(before) = text.get(copied..redaction.span.start) else {
            continue;
        };
        printed.push_str(before);
        printed.push_str(&redaction.replacement);
        copied = redaction.span.end;
    }

    printed.push_str(text.get(copied..).unwrap_or_default());

    printed
}

#[cfg(test)]
mod tests {
    use super::super::entry::Written;
    use super::super::keys::EntryKey;
    use super::{REDACTED, document, spans, value};
    use toml::de::DeTable;

    /// The redacted text of a document, as `tpl cfg list` writes it.
    fn printed(text: &str) -> String {
        let parsed = DeTable::parse(text).expect("the document parses");

        document(text, &spans(parsed.get_ref(), text))
    }

    #[test]
    fn a_literal_password_prints_as_three_asterisks() {
        // FR-CFG-021, first row.
        assert_eq!(
            value(EntryKey::Password, Written::Text("hunter2")),
            REDACTED
        );
    }

    #[test]
    fn the_password_inside_a_dsn_prints_redacted_and_the_rest_visible() {
        // FR-CFG-021, second row.
        let printed = value(
            EntryKey::Dsn,
            Written::Text("mysql://alice:hunter2@db.example.com:3306/shop"),
        );

        assert_eq!(printed, "mysql://alice:***@db.example.com:3306/shop");
        assert!(!printed.contains("hunter2"));
    }

    #[test]
    fn a_reference_prints_exactly_as_written() {
        // FR-CFG-021, third row: a password living in an environment variable
        // never reaches stdout through these two commands.
        assert_eq!(
            value(EntryKey::Password, Written::Text("${SHOP_DB_PASSWORD}")),
            "${SHOP_DB_PASSWORD}"
        );
        assert_eq!(
            value(
                EntryKey::Dsn,
                Written::Text("mysql://alice:${SHOP_DB_PASSWORD}@db.example.com/shop")
            ),
            "mysql://alice:${SHOP_DB_PASSWORD}@db.example.com/shop"
        );
    }

    #[test]
    fn a_value_only_partly_a_reference_is_redacted() {
        // Printing `secret${SUFFIX}` as written would disclose the literal
        // half, which is the disclosure the third row exists to prevent.
        assert_eq!(
            value(EntryKey::Password, Written::Text("secret${SUFFIX}")),
            REDACTED
        );
    }

    #[test]
    fn a_key_that_cannot_hold_a_credential_is_printed_unchanged() {
        assert_eq!(
            value(EntryKey::Host, Written::Text("db.example.com")),
            "db.example.com"
        );
        assert_eq!(value(EntryKey::Port, Written::Number(3306)), "3306");
    }

    #[test]
    fn the_document_keeps_its_comments_and_its_order_and_loses_its_password() {
        // FR-CFG-013: the contents of .tpl/.cfg, literally, with passwords
        // redacted.
        let text = concat!(
            "# the project's own note\n",
            "[core]\n",
            "database = \"shop\"   # the default entry\n",
            "\n",
            "[database.shop]\n",
            "user = \"alice\"\n",
            "password = \"hunter2\"\n",
            "host = \"db.example.com\"\n",
        );

        let printed = printed(text);

        assert_eq!(
            printed,
            concat!(
                "# the project's own note\n",
                "[core]\n",
                "database = \"shop\"   # the default entry\n",
                "\n",
                "[database.shop]\n",
                "user = \"alice\"\n",
                "password = \"***\"\n",
                "host = \"db.example.com\"\n",
            )
        );
    }

    #[test]
    fn a_document_with_nothing_to_redact_is_copied_byte_for_byte() {
        let text = "# a note\n[core]\ndatabase = \"shop\"\n";

        assert_eq!(printed(text), text);
    }

    #[test]
    fn every_password_of_every_entry_is_redacted() {
        let text = concat!(
            "[database.a]\n",
            "password = \"one\"\n",
            "[database.b]\n",
            "dsn = \"mysql://u:two@h/d\"\n",
            "[database.c]\n",
            "password = \"${FROM_ENV}\"\n",
        );

        let printed = printed(text);

        assert!(!printed.contains("one"), "{printed}");
        assert!(!printed.contains("two"), "{printed}");
        assert!(printed.contains("${FROM_ENV}"), "{printed}");
        assert!(printed.contains("mysql://u:***@h/d"), "{printed}");
    }
}
