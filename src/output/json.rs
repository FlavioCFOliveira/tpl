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
use serde_json::ser::Formatter;

use super::envelope::Document;

/// The byte every JSON document ends with, per `FR-OUT-007`.
const TERMINATOR: u8 = b'\n';

/// The indent of one nesting level, per `FR-OUT-008`.
const INDENT: &[u8] = b"  ";

/// The nesting levels [`SEPARATOR`] covers in one slice. Deeper levels are
/// still written correctly, in more than one call.
const COVERED_LEVELS: usize = 32;

/// A comma, a newline and the indent of [`COVERED_LEVELS`] levels.
///
/// Every separator the indented form writes is a prefix of this run, or of it
/// without its first byte: `",\n"` or `"\n"` followed by two spaces per level.
const SEPARATOR: &[u8; 2 + COVERED_LEVELS * INDENT.len()] = &{
    let mut run = [b' '; 2 + COVERED_LEVELS * INDENT.len()];
    run[0] = b',';
    run[1] = b'\n';
    run
};

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
/// The indented form is written through [`Indented`], the project's own
/// formatter, which lays out `serde_json`'s pretty form — the two-space indent
/// with one key per line that `FR-OUT-008` fixes — byte for byte. The layout is
/// therefore the project's to keep, and a test pins it to `serde_json`'s
/// `PrettyFormatter` at every depth.
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
        Form::Indented => write_indented(&mut *writer, document),
    }
    .map_err(io::Error::from)?;

    writer.write_all(&[TERMINATOR])
}

/// Serialises `value` to `writer` in the indented form of `FR-OUT-008`,
/// without the terminator.
fn write_indented<W, T>(writer: W, value: &T) -> serde_json::Result<()>
where
    W: Write,
    T: Serialize + ?Sized,
{
    value.serialize(&mut serde_json::Serializer::with_formatter(
        writer,
        Indented::default(),
    ))
}

/// The indented form of `FR-OUT-008`: `serde_json`'s `PrettyFormatter` with its
/// default two-space indent, byte for byte.
///
/// PERF: `PrettyFormatter` writes an indent with one `write_all` per nesting
/// level, and the comma before it with another. This writes the comma, the
/// newline and the whole indent as one slice of [`SEPARATOR`]. #243 measured
/// the pretty `schema dump` of `WL-001` at 17.70 → 15.03 ms with the same
/// bytes; see `BENCHMARKS.md`, 2026-09-23, task #245.
#[derive(Debug, Default)]
struct Indented {
    /// How many containers are open around the next value.
    level: usize,
    /// Whether the innermost open container has written a value yet, which
    /// decides whether its closing bracket goes on a line of its own.
    has_value: bool,
}

impl Indented {
    /// Writes an optional comma, a newline, and the indent of the current
    /// level, in one call where the level is within [`COVERED_LEVELS`].
    fn separate<W>(&self, writer: &mut W, comma: bool) -> io::Result<()>
    where
        W: ?Sized + Write,
    {
        let start = usize::from(!comma);
        let covered = self.level.min(COVERED_LEVELS);
        writer.write_all(&SEPARATOR[start..2 + covered * INDENT.len()])?;

        let mut remaining = self.level - covered;
        while remaining > 0 {
            let chunk = remaining.min(COVERED_LEVELS);
            writer.write_all(&SEPARATOR[2..2 + chunk * INDENT.len()])?;
            remaining -= chunk;
        }

        Ok(())
    }

    /// Opens a container.
    fn open<W>(&mut self, writer: &mut W, bracket: &[u8]) -> io::Result<()>
    where
        W: ?Sized + Write,
    {
        self.level += 1;
        self.has_value = false;
        writer.write_all(bracket)
    }

    /// Closes a container, on a line of its own unless it is empty.
    fn close<W>(&mut self, writer: &mut W, bracket: &[u8]) -> io::Result<()>
    where
        W: ?Sized + Write,
    {
        self.level = self.level.saturating_sub(1);
        if self.has_value {
            self.separate(writer, false)?;
        }
        writer.write_all(bracket)
    }
}

impl Formatter for Indented {
    fn begin_array<W>(&mut self, writer: &mut W) -> io::Result<()>
    where
        W: ?Sized + Write,
    {
        self.open(writer, b"[")
    }

    fn end_array<W>(&mut self, writer: &mut W) -> io::Result<()>
    where
        W: ?Sized + Write,
    {
        self.close(writer, b"]")
    }

    fn begin_array_value<W>(&mut self, writer: &mut W, first: bool) -> io::Result<()>
    where
        W: ?Sized + Write,
    {
        self.separate(writer, !first)
    }

    fn end_array_value<W>(&mut self, _writer: &mut W) -> io::Result<()>
    where
        W: ?Sized + Write,
    {
        self.has_value = true;
        Ok(())
    }

    fn begin_object<W>(&mut self, writer: &mut W) -> io::Result<()>
    where
        W: ?Sized + Write,
    {
        self.open(writer, b"{")
    }

    fn end_object<W>(&mut self, writer: &mut W) -> io::Result<()>
    where
        W: ?Sized + Write,
    {
        self.close(writer, b"}")
    }

    fn begin_object_key<W>(&mut self, writer: &mut W, first: bool) -> io::Result<()>
    where
        W: ?Sized + Write,
    {
        self.separate(writer, !first)
    }

    fn begin_object_value<W>(&mut self, writer: &mut W) -> io::Result<()>
    where
        W: ?Sized + Write,
    {
        writer.write_all(b": ")
    }

    fn end_object_value<W>(&mut self, _writer: &mut W) -> io::Result<()>
    where
        W: ?Sized + Write,
    {
        self.has_value = true;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use serde_json::{Value, json};

    use super::{COVERED_LEVELS, Form, write_document, write_indented};
    use crate::output::envelope::{Collection, Document, Source};

    /// `value` in the indented form, and in `serde_json`'s `PrettyFormatter`.
    fn both(value: &Value) -> (String, String) {
        let mut ours = Vec::new();
        write_indented(&mut ours, value).expect("a buffer accepts every write");

        let theirs = serde_json::to_vec_pretty(value).expect("a buffer accepts every write");

        (
            String::from_utf8(ours).expect("the encoder emits UTF-8"),
            String::from_utf8(theirs).expect("the encoder emits UTF-8"),
        )
    }

    /// Wraps `leaf` in `depth` containers, alternating objects and arrays,
    /// each holding a sibling before and after so every separator is reached.
    fn nested(depth: usize, leaf: Value) -> Value {
        (0..depth).fold(leaf, |inner, level| {
            if level % 2 == 0 {
                json!({ "before": level, "inner": inner, "after": [], "empty": {} })
            } else {
                json!([level, inner, "a \"quoted\"\n\ttext", null, true, 1.5])
            }
        })
    }

    /// Writes a document to a buffer and returns it as text.
    fn emitted(form: Form) -> String {
        let empty: [&str; 0] = [];
        let document = Document::new(Source::Server, Collection::new("tables", &empty));

        let mut out = Vec::new();
        write_document(&mut out, &document, form).expect("a buffer accepts every write");

        String::from_utf8(out).expect("the encoder emits UTF-8")
    }

    #[test]
    fn fr_out_007_compact_is_the_default_form() {
        // FR-OUT-007: compact by default, and the type says so.
        assert_eq!(Form::default(), Form::Compact);
    }

    #[test]
    fn fr_out_007_a_compact_document_is_one_line_with_one_terminating_newline() {
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
    fn fr_out_008_an_indented_document_has_a_two_space_indent_and_one_key_per_line() {
        // FR-OUT-008: a two-space indent with one key per line.
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
    fn fr_out_007_the_indented_form_ends_with_one_terminating_newline_too() {
        // FR-OUT-007 fixes the terminator for the compact form; the indented
        // form is the same document and ends the same way, so a consumer
        // reading line by line sees a complete document in both.
        let document = emitted(Form::Indented);

        assert!(document.ends_with("}\n"));
        assert!(!document.ends_with("}\n\n"));
    }

    #[test]
    fn fr_out_013_both_forms_carry_the_same_keys_in_the_same_order() {
        // FR-OUT-013, FR-OUT-024: key order is a property of the type, so
        // --pretty changes the whitespace and nothing else.
        for document in [emitted(Form::Compact), emitted(Form::Indented)] {
            let at = |key: &str| document.find(key).expect("the key is present");

            assert!(at("\"schema_version\"") < at("\"source\""), "{document}");
            assert!(at("\"source\"") < at("\"data\""), "{document}");
            assert!(at("\"data\"") < at("\"tables\""), "{document}");
        }
    }

    #[test]
    fn fr_out_008_the_indented_form_is_pretty_formatter_byte_for_byte_at_every_depth() {
        // FR-OUT-008: the layout is the project's own formatter, so a test and
        // not the encoder's default is what keeps it serde_json's two-space
        // form. Every depth up to well past the covered run, where the indent
        // is written in more than one call.
        for depth in 0..=COVERED_LEVELS * 3 + 1 {
            for leaf in [
                json!(0),
                json!([]),
                json!({}),
                json!(["x"]),
                json!({ "k": "v" }),
            ] {
                let (ours, theirs) = both(&nested(depth, leaf));
                assert_eq!(ours, theirs, "depth {depth}");
            }
        }
    }

    #[test]
    fn fr_out_008_a_single_deep_chain_matches_pretty_formatter() {
        // One container per level and nothing beside it: only the opening
        // separators and the closing indents, at every level past the run.
        let deep = (0..COVERED_LEVELS * 4 + 3).fold(json!("leaf"), |inner, level| {
            if level % 2 == 0 {
                json!([inner])
            } else {
                json!({ "k": inner })
            }
        });

        let (ours, theirs) = both(&deep);
        assert_eq!(ours, theirs);
    }

    #[test]
    fn fr_out_008_scalars_and_empty_containers_match_pretty_formatter() {
        for value in [
            json!(null),
            json!(true),
            json!(-7),
            json!(2.25),
            json!("s"),
            json!([]),
            json!({}),
            json!([[], {}, [[]], { "a": {} }]),
        ] {
            let (ours, theirs) = both(&value);
            assert_eq!(ours, theirs, "{value}");
        }
    }
}
