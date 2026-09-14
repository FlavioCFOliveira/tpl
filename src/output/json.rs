//! The two forms of `FR-OUT-007` and `FR-OUT-008`, and the one call into
//! `serde_json`.
//!
//! Serialisation is **derived**, per `OD-18`: the emitted types are the
//! model's, their field declaration order is the key order, and the order is
//! therefore stated once — in the type — rather than a second time in a writer
//! that would eventually disagree with it. `preserve_order` is off and
//! `indexmap` is not in the dependency graph, which is how `FR-OUT-013` is met
//! when it forbids an unordered map on the emitting path.
//!
//! C0 escaping needs nothing here. JSON admits no raw control character inside
//! a string, so the encoder emits the escape the format defines and that is
//! what satisfies `FR-OUT-018` on this path; the tab exception belongs to the
//! `text` layouts, where the aligned columns of `FR-OUT-006` are laid out with
//! it.
//!
//! Nothing is flushed here. The buffer belongs to [`super::writer`], which owns
//! it because it also owns the consequence of a failed flush.

use std::io::{self, Write};

use serde::Serialize;

use super::envelope::Document;

/// The byte every JSON document ends with, per `FR-OUT-007`.
const TERMINATOR: u8 = b'\n';

/// Which of the two forms a document is written in.
///
/// [`Form::Compact`] is the default the type itself states, because
/// `FR-OUT-007` makes it the default of the contract. [`Form::Indented`] is
/// requested by `--pretty` and by nothing else.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub(crate) enum Form {
    /// One line, no superfluous whitespace, one terminating newline
    /// (`FR-OUT-007`).
    #[default]
    Compact,
    /// A two-space indent with one key per line (`FR-OUT-008`).
    Indented,
}

/// Writes one document to `writer`, in `form`, terminated by a single newline.
///
/// The parameter is a [`Document`] rather than any [`Serialize`] value, so
/// `FR-OUT-032` holds by construction on the way in: the encoder is never
/// handed a document that is not enveloped.
///
/// The indented form is `serde_json`'s own pretty printer, whose default indent
/// is the two spaces `FR-OUT-008` fixes, so the requirement is met by the
/// encoder's default rather than by a setting that could be changed without
/// noticing.
///
/// # Errors
///
/// Returns what `writer` returned. A serialisation failure that is not the
/// writer's — which the payload types cannot produce, having no non-string map
/// key and no non-finite number — arrives as [`io::ErrorKind::InvalidData`].
pub(super) fn write_document<W, T>(
    writer: &mut W,
    document: &Document<T>,
    form: Form,
) -> io::Result<()>
where
    W: Write,
    T: Serialize,
{
    match form {
        Form::Compact => serde_json::to_writer(&mut *writer, document),
        Form::Indented => serde_json::to_writer_pretty(&mut *writer, document),
    }
    .map_err(io::Error::from)?;

    writer.write_all(&[TERMINATOR])
}

#[cfg(test)]
mod tests {
    use super::{Form, write_document};
    use crate::output::envelope::{Collection, Document, Source};

    /// Writes a document to a buffer and returns it as text.
    fn emitted(form: Form) -> String {
        let empty: [&str; 0] = [];
        let document = Document::new(Source::Server, Collection::new("tables", &empty));

        let mut out = Vec::new();
        write_document(&mut out, &document, form).expect("a buffer accepts every write");

        String::from_utf8(out).expect("the encoder emits UTF-8")
    }

    #[test]
    fn compact_is_the_default_form() {
        // FR-OUT-007: compact by default, and the type says so.
        assert_eq!(Form::default(), Form::Compact);
    }

    #[test]
    fn a_compact_document_is_one_line_with_one_terminating_newline() {
        // FR-OUT-007, FR-OUT-024: one line, no superfluous whitespace, the
        // three keys in order, terminated by a single newline.
        let document = emitted(Form::Compact);

        assert_eq!(
            document,
            "{\"schema_version\":1,\"source\":\"server\",\"data\":{\"tables\":[]}}\n"
        );
        assert_eq!(
            document.matches('\n').count(),
            1,
            "a compact document carries exactly one newline, and it terminates it"
        );
        assert!(
            document.ends_with('\n'),
            "the one newline is the terminator"
        );
    }

    #[test]
    fn an_indented_document_has_a_two_space_indent_and_one_key_per_line() {
        // FR-OUT-008: a two-space indent with one key per line, which is
        // serde_json's own default.
        let document = emitted(Form::Indented);

        assert_eq!(
            document,
            concat!(
                "{\n",
                "  \"schema_version\": 1,\n",
                "  \"source\": \"server\",\n",
                "  \"data\": {\n",
                "    \"tables\": []\n",
                "  }\n",
                "}\n"
            )
        );
    }

    #[test]
    fn the_indented_form_ends_with_one_terminating_newline_too() {
        // FR-OUT-007 fixes the terminator for the compact form; the indented
        // form is the same document and ends the same way, so a consumer
        // reading line by line sees a complete document in both.
        let document = emitted(Form::Indented);

        assert!(document.ends_with("}\n"));
        assert!(!document.ends_with("}\n\n"));
    }

    #[test]
    fn both_forms_carry_the_same_keys_in_the_same_order() {
        // FR-OUT-013, FR-OUT-024: key order is a property of the type, so
        // --pretty changes the whitespace and nothing else.
        for document in [emitted(Form::Compact), emitted(Form::Indented)] {
            let at = |key: &str| document.find(key).expect("the key is present");

            assert!(at("\"schema_version\"") < at("\"source\""), "{document}");
            assert!(at("\"source\"") < at("\"data\""), "{document}");
            assert!(at("\"data\"") < at("\"tables\""), "{document}");
        }
    }
}
