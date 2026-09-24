//! The document that carries the model, in both directions (`FR-CTX-001`,
//! `FR-SCH-017`, `FR-SCH-022`, `FR-RND-020`, `FR-OUT-016`).
//!
//! One document carries the model. `tpl schema dump` emits it, `tpl render
//! --context` consumes it, and a template sees the same material. This module
//! is the junction: [`dump`] builds the document from a model and [`read()`]
//! builds a model from a document, and the two are inverse because they are two
//! directions over **one** set of types rather than two hand-written routines
//! that would have to be kept inverse.
//!
//! | Submodule | What it owns |
//! |---|---|
//! | `shape` | The document's types and its key order, per `FR-OUT-013` and `OD-18` |
//! | `order` | The default rule of `NFR-DET-002` and its six exceptions, byte-wise |
//! | `build` | The outward direction: the embedding of `ADR-009`, and the ordering applied where each collection is built |
//! | `read` | The inward direction: exactly the checks `FR-CTX-033` and `FR-RND-020` admit, and no repair |
//!
//! # The four rules the document is
//!
//! **Every collection is an array**, `[]` when empty and never `null` and never
//! omitted (`FR-CTX-003` … `FR-CTX-005`). It holds because a collection is a
//! slice or a [`Vec`] in every type, so there is no state in which a key is
//! absent or its value is not an array. `null` is left to an absent scalar, and
//! a consumer may test a collection for emptiness without first testing it for
//! nullity.
//!
//! **A reference goes one hop, then names**, in both directions
//! (`FR-CTX-006` … `FR-CTX-010`). `shape` states it as a type parameter, so a
//! cycle terminates by construction: an embedded table's reference collections
//! hold strings, and a string carries no table. There is no depth counter and
//! no visited set, because there is nothing for one to guard.
//!
//! **Every collection is ordered, and the six exceptions cannot be reached by
//! the default rule** (`NFR-DET-002`). `order` is bounded by a trait that the
//! excepted member types do not implement, so sorting a primary key's columns,
//! a foreign key's positional pairing or an `ENUM` member list is a compile
//! error rather than a silent corruption at exit `0`.
//!
//! **Key order is a property of the type** (`FR-OUT-013`, `OD-18`).
//! Serialisation is derived throughout, so a struct's field declaration order
//! **is** its key order, stated once and unable to drift; and no unordered map
//! appears on the emitting path.
//!
//! # What this module does not do
//!
//! It does not emit. [`crate::output`] owns the envelope of `FR-OUT-024`, the
//! two forms of `FR-OUT-007` and `FR-OUT-008`, and the buffered writer the
//! bytes are aggregated through; [`dump`] hands that module a
//! [`Document`] and composes no byte of its own. It does not read a file
//! either: [`read()`] is given the bytes, so the one path that opens a file is
//! the caller's.

pub(crate) mod build;

#[cfg(test)]
pub(crate) mod fixture;

pub(crate) mod order;
pub(crate) mod read;
pub(crate) mod shape;

use std::path::Path;

use serde_json::error::Category;

use crate::error::{ContextFault, Error, Position};
use crate::model::database::Database;
use crate::output::{Document, Source};

pub(crate) use shape::{ContextData, DatabaseDocument};

/// Builds the document that carries `database`, ready to be emitted.
///
/// `source` is the value of `FR-OUT-026` the read was served by — `server` or
/// `cache` on a `schema` document, per `FR-SCH-035`. The envelope is
/// [`Document`], so `FR-OUT-032` holds on the way out: what this returns is the
/// only shape [`crate::output::emit`] accepts.
///
/// # Errors
///
/// Returns [`Error::InternalInvariant`] where a foreign key names a table the
/// model does not carry, which `FR-CTX-023` makes unreachable for a model
/// produced by a server read.
pub(crate) fn dump<'a>(
    database: &'a Database<'a>,
    source: Source,
) -> Result<Document<ContextData<'a>>, Error> {
    Ok(Document::new(source, build::context(database)?))
}

/// Builds the `database` object a read produced, without enveloping it.
///
/// It is the half of [`dump`] that the seven `schema` subcommands which are not
/// `dump` need: each of them presents a **part** of this object — one
/// collection, per `FR-SCH-032`, or one member of one, per `FR-SCH-033` — and
/// each part is the same shape here as it is in the dump, which is what makes
/// `BR-SCH-001` hold across the eight. The envelope those commands put round
/// their own part is [`Document`], composed where the part is chosen.
///
/// `cache` is the other caller: `FR-CACHE-030` writes each member of this
/// object to its own file, so the object is what a write is taken from and
/// what a read reassembles.
///
/// # Errors
///
/// Returns what [`dump`] returns, for the same condition.
pub(crate) fn context<'a>(database: &'a Database<'a>) -> Result<DatabaseDocument<'a>, Error> {
    Ok(build::context(database)?.database)
}

/// Reads a `--context` document back as a model.
///
/// `bytes` is the document as read from `path`, in either the compact or the
/// indented form: `FR-OUT-016` and `FR-RND-021` accept both, and neither the
/// decoder nor this function distinguishes them. `path` is carried only so that
/// a failure can name it, per `FR-ERR-034`.
///
/// The model borrows `bytes` wherever the encoding allowed it, and owns the
/// value where a string needed unescaping — which is the case
/// [`ColumnType`](crate::model::column_type::ColumnType) carries
/// [`Cow`](std::borrow::Cow) for.
///
/// # Errors
///
/// Returns [`Error::ContextDocumentMalformed`] for a document that is not
/// well-formed JSON, carrying the position the decoder stopped at, or for one
/// that is JSON and does not match the contract, carrying the rule it failed
/// (`FR-RND-020`, `FR-ERR-029`) — including a foreign key naming a table the
/// document's `tables` does not carry, which `FR-CTX-042` makes such a rule
/// and which would otherwise reach [`context`] as the `70` of a violated
/// invariant.
pub(crate) fn read<'a>(bytes: &'a str, path: &Path) -> Result<Database<'a>, Error> {
    let malformed = |fault| Error::ContextDocumentMalformed {
        path: path.into(),
        fault,
        default_entry: false,
    };

    // FR-SCH-036: the whole envelope, and not a bare `data` object in its
    // place. A document that omits `schema_version` is refused here, by the
    // three fields of `Document` having no defaults.
    let document: Document<ContextData<'a>> =
        serde_json::from_str(bytes).map_err(|reported| malformed(fault(bytes, &reported)))?;

    read::database(document.into_data().database).map_err(malformed)
}

/// Classifies what the decoder reported, per the `65` row of `FR-ERR-034`.
///
/// The row obliges the `cause` line to name the path and *either* the position
/// of the malformed JSON *or* the structural rule the document failed, and the
/// decoder's own classification is what separates the two.
///
/// The forty-third edition asks the second half for the key path of the member
/// that fails and what the contract expects there, and forbids citing a file
/// of the specification. The decoder states what it expected and where it
/// stopped, and [`path_at`] turns where it stopped into the key path.
fn fault(bytes: &str, reported: &serde_json::Error) -> ContextFault {
    // No JSON text at all is its own fault: "the parser stopped at line 1,
    // column 0" would state a position that tells the reader nothing.
    if bytes.trim_start().is_empty() {
        return ContextFault::Empty;
    }

    match reported.classify() {
        Category::Data => structure(bytes, reported),
        // Syntax and Eof are both a document that is not well-formed JSON. Io
        // is unreachable: the bytes are already in memory, and a decode from a
        // string performs no read.
        // The decoder reports column 0 for an end of input at the start of a
        // line, before any character of it; every other position this system
        // writes is one-based, so it is the first column (finding T-07).
        Category::Syntax | Category::Eof | Category::Io => ContextFault::NotJson(Position {
            line: reported.line(),
            column: reported.column().max(1),
        }),
    }
}

/// The structural fault the decoder reported, as a key path and what was
/// expected there.
///
/// `expected` is written as a clause that follows the path — "is missing",
/// "must be an array, and it is an object" — in the vocabulary of JSON and
/// not of the decoder: `map`, `sequence` and `struct TableShape` are the
/// decoder's words for what a reader of the document calls an object and an
/// array.
fn structure(bytes: &str, reported: &serde_json::Error) -> ContextFault {
    let offset = offset(bytes, reported.line(), reported.column());
    let (mut at, closed) = path_at(bytes, offset);
    let message = reported.to_string();
    let message = message
        .rsplit_once(" at line ")
        .map_or(message.as_str(), |(head, _)| head);

    // "missing field `name`" is reported where the object that lacks it
    // closes, so the path is that object's, and the key is appended to it.
    let expected = match message.strip_prefix("missing field `") {
        Some(rest) if closed => {
            let field = rest.trim_end_matches('`');
            at = if at.is_empty() {
                field.to_owned()
            } else {
                format!("{at}.{field}")
            };
            MISSING.to_owned()
        }
        // The top level is the envelope, whatever the decoder says it found.
        _ if at.is_empty() => ENVELOPE.to_owned(),
        _ => expectation(message),
    };

    ContextFault::Structure { at, expected }
}

/// What [`structure`] says of a required key the document does not carry.
const MISSING: &str = "is missing; a context document requires it";

/// What [`structure`] says of a top level that is not the envelope.
const ENVELOPE: &str =
    "the top level must be an object with the keys schema_version, source and data";

/// The decoder's message for a value it refused, as a clause that follows the
/// value's path.
fn expectation(message: &str) -> String {
    if let Some(rest) = message.strip_prefix("invalid type: ")
        && let Some((found, wanted)) = rest.split_once(", expected ")
    {
        return format!(
            "must be {}, but it is {}",
            json_term(wanted),
            json_term(found)
        );
    }

    // "invalid length 0, expected struct TableShape with 13 elements" is an
    // array given where an object is expected.
    if let Some(rest) = message.strip_prefix("invalid length ")
        && rest.contains("expected struct ")
    {
        return "must be an object, but it is an array".to_owned();
    }

    // "unknown variant `bogus`, expected one of `server`, `cache`".
    if let Some(rest) = message.strip_prefix("unknown variant `")
        && let Some((found, allowed)) = rest.split_once("`, expected ")
    {
        let allowed = allowed.trim_start_matches("one of ").replace('`', "");
        return format!("must be one of {allowed}, but it is '{found}'");
    }

    format!("does not match the contract: {message}")
}

/// A decoder's name for a kind of value, as JSON calls it.
fn json_term(decoder: &str) -> String {
    match decoder {
        "map" | "a map" => "an object".to_owned(),
        "sequence" | "a sequence" => "an array".to_owned(),
        "null" => "null".to_owned(),
        other if other.starts_with("struct ") => "an object".to_owned(),
        other => other.to_owned(),
    }
}

/// The byte offset of a one-based line and column the decoder reported.
fn offset(bytes: &str, line: usize, column: usize) -> usize {
    let start: usize = bytes
        .split_inclusive('\n')
        .take(line.saturating_sub(1))
        .map(str::len)
        .sum();

    (start + column).min(bytes.len())
}

/// One level of the path [`path_at`] tracks.
enum Level {
    /// Inside an object, with the key most recently read.
    Object(Option<String>),
    /// Inside an array, with the index of the element being read.
    Array(usize),
}

/// The key path of the value that ends at or just before `offset`, in a
/// document known to be well-formed JSON, and whether that value is an object
/// or an array that closed there.
///
/// The decoder reports a fault where it stops, which is just after the value it
/// refused, or just after the object that lacks a required key; the path of
/// that value is the path of the member at fault.
fn path_at(bytes: &str, offset: usize) -> (String, bool) {
    let mut stack: Vec<Level> = Vec::new();
    let mut last = (String::new(), false);
    let text = bytes.as_bytes();
    let mut index = 0;

    while index < text.len() && index < offset {
        match text[index] {
            b'{' => stack.push(Level::Object(None)),
            b'[' => stack.push(Level::Array(0)),
            b'}' | b']' => {
                stack.pop();
                last = (render(&stack), true);
            }
            b',' => {
                if let Some(Level::Array(position)) = stack.last_mut() {
                    *position += 1;
                }
            }
            b'"' => {
                let end = string_end(text, index);
                let read = bytes.get(index + 1..end).unwrap_or_default();

                let is_key = text[end + 1..]
                    .iter()
                    .find(|byte| !byte.is_ascii_whitespace())
                    == Some(&b':');

                match stack.last_mut() {
                    Some(Level::Object(key)) if is_key => *key = Some(read.to_owned()),
                    _ => last = (render(&stack), false),
                }
                index = end;
            }
            b':' => {}
            byte if !byte.is_ascii_whitespace() => {
                // A number or a literal: read to its end.
                while index + 1 < text.len()
                    && !matches!(text[index + 1], b',' | b'}' | b']')
                    && !text[index + 1].is_ascii_whitespace()
                {
                    index += 1;
                }
                last = (render(&stack), false);
            }
            _ => {}
        }
        index += 1;
    }

    // The decoder refuses an object or an array where it peeks the opening
    // bracket, and reports the offset before that bracket rather than after a
    // value: the member at fault is then the one whose value opens there — the
    // key just read, or the array element being read — and not the value
    // before it.
    let opens = text
        .get(index..)
        .and_then(|rest| rest.iter().find(|byte| !byte.is_ascii_whitespace()));
    if matches!(opens, Some(b'{' | b'[')) {
        last = (render(&stack), false);
    }

    last
}

/// The index of the quotation mark that closes the string opening at `start`.
fn string_end(text: &[u8], start: usize) -> usize {
    let mut index = start + 1;

    while index < text.len() {
        match text[index] {
            b'\\' => index += 2,
            b'"' => return index,
            _ => index += 1,
        }
    }

    text.len().saturating_sub(1)
}

/// The path the stack describes, as a caller reads it in the document.
fn render(stack: &[Level]) -> String {
    let mut path = String::new();

    for level in stack {
        match level {
            Level::Object(Some(key)) => {
                if !path.is_empty() {
                    path.push('.');
                }
                path.push_str(key);
            }
            Level::Object(None) => {}
            Level::Array(position) => {
                path.push('[');
                path.push_str(&position.to_string());
                path.push(']');
            }
        }
    }

    path
}

#[cfg(test)]
mod tests {
    use super::{dump, fixture, read};
    use crate::error::{ContextFault, Error};
    use crate::model::database::Database;
    use crate::model::restricted::Restricted;
    use crate::output::{Form, Source, emit_to};
    use std::borrow::Cow;
    use std::path::{Path, PathBuf};

    /// The `server` object the fixture emits, as one literal, so that a test
    /// that removes or rewrites part of it fails loudly if the shape moves.
    const SERVER: &str =
        r#""server":{"version":"11.4.13-MariaDB-ubu2404","series":"11.4","standing":"supported"}"#;

    /// The path a supplied document is named by in a diagnostic.
    fn path() -> PathBuf {
        PathBuf::from("context.json")
    }

    /// Emits a model through the writer every result reaches stdout through.
    ///
    /// It is [`emit_to`] and not a call into `serde_json`, so the bytes a test
    /// reads are the bytes a caller would: one buffered writer, the envelope of
    /// `FR-OUT-024`, and the terminating newline of `FR-OUT-007`.
    fn emitted(database: &Database<'_>, form: Form) -> String {
        let document = dump(database, Source::Server).expect("the fixture carries every reference");

        let mut bytes = Vec::new();
        emit_to(&mut bytes, &document, form).expect("a buffer accepts every write");

        String::from_utf8(bytes).expect("the encoder emits UTF-8")
    }

    fn compact(database: &Database<'_>) -> String {
        emitted(database, Form::Compact)
    }

    /// The fault a supplied document failed on.
    fn refused(document: &str) -> ContextFault {
        match read(document, &path()).expect_err("the document does not match the contract") {
            Error::ContextDocumentMalformed {
                path: named, fault, ..
            } => {
                assert_eq!(
                    &*named,
                    path(),
                    "FR-ERR-034 obliges the cause to name the path"
                );
                fault
            }
            other => panic!("a malformed context document is FR-RND-020's 65, not {other}"),
        }
    }

    #[test]
    fn fr_sch_022_a_document_serialised_and_read_back_yields_an_equal_model() {
        // FR-SCH-022 with OD-18: the dump and its re-emission are inverse
        // because they are two directions over one set of types. The embedding
        // of FR-CTX-006 and FR-CTX-010 is undone by taking the name back out of
        // each embedded object, which is the only field of it the model holds.
        let model = fixture::database();
        let document = compact(&model);

        let read_back =
            read(&document, &path()).expect("the document tpl wrote matches its own contract");

        assert_eq!(read_back, model);
        assert_eq!(
            compact(&read_back),
            document,
            "and the bytes are the same bytes"
        );
    }

    #[test]
    fn fr_ctx_004_an_empty_collection_is_an_empty_array_and_is_never_null_and_never_omitted() {
        // FR-CTX-004 and FR-CTX-005: a consumer may test a collection for
        // emptiness without first testing it for nullity, so `null` is left to
        // an absent scalar. FR-OUT-035 says the same for an empty result.
        let empty = Database {
            tables: Vec::new(),
            views: Vec::new(),
            routines: Vec::new(),
            ..fixture::database()
        };
        let document = compact(&empty);

        assert!(
            document.contains(r#""tables":[],"views":[],"routines":[]"#),
            "{document}"
        );
        assert!(!document.contains("null"), "{document}");

        // And on a table, over every collection it carries.
        let bare = compact(&fixture::database_of(vec![fixture::carrier()]));
        for key in [
            r#""columns":[{"#,
            r#""indexes":[{"#,
            r#""foreign_keys":[]"#,
            r#""referenced_by":[]"#,
            r#""triggers":[]"#,
            r#""check_constraints":[]"#,
        ] {
            assert!(bare.contains(key), "{key} is absent from {bare}");
        }
    }

    #[test]
    fn fr_priv_005_a_complete_object_carries_no_restricted_key_and_an_incomplete_one_carries_an_array()
     {
        // FR-PRIV-005 through FR-PRIV-007 and FR-PRIV-016, which is the one
        // exception OD-18 admits to FR-OUT-012: the key is absent rather than
        // `null` where there is nothing to report.
        let document = compact(&fixture::database());

        assert!(
            document.contains(r#""restricted":["definition"]"#),
            "{document}"
        );
        assert!(document.contains(r#""restricted":["body"]"#), "{document}");
        assert!(!document.contains(r#""restricted":null"#), "{document}");

        let complete = compact(&fixture::database_of(vec![fixture::carrier()]));
        assert!(!complete.contains("restricted"), "{complete}");
    }

    #[test]
    fn fr_out_016_the_compact_and_the_indented_form_are_both_accepted() {
        // FR-OUT-016 and FR-RND-021. The two differ in whitespace and in
        // nothing else, so one contract reads both and neither path knows
        // which it was given.
        let model = fixture::database();
        let indented = emitted(&model, Form::Indented);

        assert!(
            indented.contains("\n  \"schema_version\": 1,"),
            "{indented}"
        );
        assert_eq!(
            read(&indented, &path()).expect("the indented form is the same document"),
            model
        );
        assert_eq!(
            read(&compact(&model), &path()).expect("the compact form is the same document"),
            model
        );
    }

    #[test]
    fn fr_ctx_033_a_series_outside_the_supported_window_is_accepted() {
        // FR-CTX-033: on this path the system SHALL NOT validate `series`
        // against FR-SRV-015 and SHALL NOT check `standing` against `series`.
        // The document is a record of a read that already happened, and
        // FR-RND-022 opens no connection, so there is no server to vouch for.
        // Validating the window would make every committed dump expire on a
        // calendar date as the window moved.
        let document = compact(&fixture::database()).replace(
            SERVER,
            r#""server":{"version":"5.5.68-MariaDB","series":"5.5","standing":"supported"}"#,
        );

        let model = read(&document, &path()).expect("FR-CTX-033 forbids the window check");

        assert_eq!(model.server.series(), "5.5");
        assert_eq!(model.server.version(), "5.5.68-MariaDB");
    }

    #[test]
    fn br_ctx_006_a_document_missing_a_key_of_the_server_object_is_refused() {
        // BR-CTX-006: `server` is a structural rule of context-document.md, so
        // a document that omits it does not match the contract and is 65 under
        // FR-RND-020. The same holds for one of its three keys, which
        // FR-CTX-033 requires to be present.
        let whole = compact(&fixture::database());
        assert!(whole.contains(SERVER), "{whole}");

        for absent in [
            whole.replace(&format!("{SERVER},"), ""),
            whole.replace(r#""series":"11.4","#, ""),
            whole.replace(r#""standing":"supported""#, r#""standing":"retired""#),
            whole.replace(r#""version":"11.4.13-MariaDB-ubu2404""#, r#""version":114"#),
        ] {
            assert!(
                matches!(refused(&absent), ContextFault::Structure { .. }),
                "a document that is JSON and not this contract names the rule it failed"
            );
        }
    }

    #[test]
    fn fr_rnd_020_a_document_that_is_not_well_formed_json_is_refused_with_the_position() {
        // FR-RND-020 and the 65 row of FR-ERR-034, whose cause line carries
        // either the position of the malformed JSON or the structural rule.
        let fault = refused("{\"schema_version\":1,\n  \"source\": oops}");

        let ContextFault::NotJson(position) = fault else {
            panic!("bytes that are not JSON are the position half of FR-ERR-034");
        };

        assert_eq!(position.line, 2);
    }

    #[test]
    fn fr_sch_036_a_bare_data_object_is_refused_in_place_of_the_whole_envelope() {
        // FR-SCH-036: a caller that had stripped the envelope would have
        // discarded the `source` field FR-CDOC-016 makes the signal of what the
        // document does not promise.
        let whole = compact(&fixture::database());
        let bare = whole
            .trim_end()
            .strip_prefix(r#"{"schema_version":1,"source":"server","data":"#)
            .and_then(|rest| rest.strip_suffix('}'))
            .expect("the envelope the emitter wrote is the one FR-OUT-024 fixes");

        assert!(bare.starts_with(r#"{"database":"#), "{bare}");
        assert!(matches!(refused(bare), ContextFault::Structure { .. }));
    }

    #[test]
    fn fr_priv_016_a_restricted_marking_that_names_no_property_is_refused_rather_than_repaired() {
        // FR-PRIV-016: the array is never empty. Reading it as a complete
        // object would turn a document the requirement forbids into one it
        // permits, silently.
        let document =
            compact(&fixture::database()).replace(r#""restricted":["body"]"#, r#""restricted":[]"#);

        assert!(matches!(refused(&document), ContextFault::Structure { .. }));
    }

    #[test]
    fn fr_cat_044_a_table_whose_key_names_a_column_it_does_not_carry_is_refused() {
        // FR-CAT-044, through Table::assemble, which is the model's only
        // constructor for a table. The same violation is an internal invariant
        // on the server path and caller data here, which is why the model
        // reports it neutrally and each caller maps it.
        let document = compact(&fixture::database()).replace(
            r#"{"name":"leg_id","direction":"A","prefix_length":null}"#,
            r#"{"name":"row_end","direction":"A","prefix_length":null}"#,
        );

        let ContextFault::Structure { at, expected } = refused(&document) else {
            panic!("a key naming an absent column is a structural fault");
        };

        assert!(at.starts_with("data.database.tables["), "{at}");
        assert!(
            expected.contains("must be one of its own columns"),
            "{expected}"
        );
    }

    /// The fixture's dump as a JSON value, for a test that edits it.
    fn editable() -> serde_json::Value {
        serde_json::from_str(&compact(&fixture::database())).expect("the dump is JSON")
    }

    /// The `tables` collection of an editable dump.
    fn tables_of(document: &mut serde_json::Value) -> &mut Vec<serde_json::Value> {
        document["data"]["database"]["tables"]
            .as_array_mut()
            .expect("tables is an array")
    }

    #[test]
    fn fr_ctx_042_a_foreign_key_naming_a_table_the_document_does_not_carry_is_65() {
        // FR-CTX-042, finding SEC-02, in the shape the audit's document had:
        // one table kept, and every table its keys name dropped. The builder
        // would report the 70 of an invariant; the document is the caller's
        // to correct, so it is FR-RND-020's 65 naming the key.
        let mut document = editable();
        let tables = tables_of(&mut document);
        let kept = tables
            .iter()
            .find(|table| {
                table["foreign_keys"]
                    .as_array()
                    .is_some_and(|keys| keys.iter().any(|key| key["referenced_table"].is_object()))
            })
            .cloned()
            .expect("the fixture carries a table with a key that names a table");
        let first = kept["foreign_keys"]
            .as_array()
            .and_then(|keys| keys.iter().find(|key| key["referenced_table"].is_object()))
            .cloned()
            .expect("found above");
        *tables = vec![kept.clone()];

        let fault = refused(&document.to_string());

        let ContextFault::DanglingReference {
            table,
            collection,
            key,
            names,
        } = fault
        else {
            panic!("a dangling key is FR-CTX-042's fault, not {fault:?}");
        };
        assert_eq!(table, kept["name"].as_str().expect("a name"));
        assert_eq!(collection, "foreign_keys");
        assert_eq!(key, first["name"].as_str().expect("a name"));
        assert_eq!(
            names,
            first["referenced_table"]["name"].as_str().expect("a name")
        );
    }

    #[test]
    fn fr_ctx_042_a_referenced_by_entry_naming_a_table_the_document_does_not_carry_is_65() {
        // FR-CTX-042's second direction: the referencing table of an entry
        // under `referenced_by`.
        let mut document = editable();
        let tables = tables_of(&mut document);
        let at = tables
            .iter()
            .position(|table| {
                table["referenced_by"]
                    .as_array()
                    .is_some_and(|entries| !entries.is_empty())
            })
            .expect("the fixture carries a referenced table");
        let carrier = tables[at]["name"].as_str().expect("a name").to_owned();
        let entry = &mut tables[at]["referenced_by"][0];
        let key = entry["key"]["name"].as_str().expect("a name").to_owned();
        entry["table"]["name"] = serde_json::Value::from("ghost_table");

        let fault = refused(&document.to_string());

        assert_eq!(
            fault,
            ContextFault::DanglingReference {
                table: carrier,
                collection: "referenced_by",
                key,
                names: "ghost_table".to_owned(),
            }
        );
    }

    #[test]
    fn fr_ctx_042_a_column_whose_table_name_is_absent_is_not_a_reference() {
        // FR-CTX-042, what it does not reach: FR-SEM-017, FR-SEM-018 and
        // FR-ENV-017 make this case reachable on purpose, and the test that
        // resolves it fails the render when it is applied.
        let mut document = editable();
        tables_of(&mut document)[0]["columns"][0]["table_name"] =
            serde_json::Value::from("elsewhere");

        let text = document.to_string();
        let read_back = read(&text, &path());

        assert!(read_back.is_ok(), "{read_back:?}");
    }

    #[test]
    fn fr_sch_017_the_dump_is_the_envelope_of_the_requirement_carrying_one_data_key() {
        // FR-SCH-017 and FR-OUT-024: three keys in one order, and a `data` of
        // one key named for the kind in the singular. FR-OUT-007 terminates it
        // with a single newline.
        let document = compact(&fixture::database());

        assert!(
            document.starts_with(r#"{"schema_version":1,"source":"server","data":{"database":{"name":"freight","charset":"utf8mb4","collation":"utf8mb4_unicode_520_ci","server":{"#),
            "{document}"
        );
        assert_eq!(document.matches('\n').count(), 1);
        assert!(document.ends_with("}\n"));
    }

    #[test]
    fn the_read_back_model_borrows_the_document_where_the_encoding_allowed_it() {
        // The reason ColumnType carries Cow: serde_json borrows a JSON string
        // only where it needed no unescaping, and a raw type string such as
        // `enum('8''6"')` is escaped in JSON and arrives owned. One shape
        // carries both, so no document-side buffer has to outlive the model.
        let document = compact(&fixture::database());
        let model = read(&document, &path()).expect("the document matches the contract");

        assert!(
            matches!(model.name, Cow::Borrowed(_)),
            "an unescaped name is borrowed"
        );
        assert_eq!(
            model.tables[0].columns()[2].column_type.raw(),
            fixture::STATUS,
            "and the raw type string is carried unaltered whichever way it arrived"
        );
    }

    #[test]
    fn fr_ctx_012_a_column_default_is_emitted_in_exactly_the_three_forms_the_requirement_lists() {
        // FR-CTX-012 and FR-CTX-013: `kind` is a three-way discriminant, and
        // the `null` form carries no `value` — which is why the shape is a
        // tagged enumeration and not a struct with an optional field. A form
        // the requirement does not list has no value that would produce it.
        let document = compact(&fixture::database());

        assert!(
            document.contains(r#""default":{"kind":"literal","value":"EUR"}"#),
            "{document}"
        );
        assert!(
            document.contains(r#""default":{"kind":"null"}"#),
            "{document}"
        );
        assert!(
            document.contains(r#""default":{"kind":"expression","value":"current_timestamp()"}"#),
            "{document}"
        );
        assert!(!document.contains(r#""kind":"null","value""#), "{document}");
        assert!(
            document.contains(r#""default":null"#),
            "and a NOT NULL column with no DEFAULT is the bare null of FR-CTX-011: {document}"
        );
    }

    #[test]
    fn fr_ctx_015_the_decomposed_parts_are_siblings_of_the_raw_type_and_not_nested_under_it() {
        // FR-CTX-014 gives a column `column_type`; FR-CTX-015 gives it the
        // eight parts **additionally**. Both therefore read as fields of the
        // column: `col.data_type`, never `col.column_type.data_type`. The
        // literal below is the whole of one column, so a part that moved back
        // under the raw string fails here rather than in a template.
        let document = compact(&fixture::database());

        assert!(
            document.contains(
                r#""position":1,"column_type":"bigint(20) unsigned","data_type":"bigint","#
            ),
            "{document}"
        );
        assert!(
            !document.contains(r#""column_type":{"#),
            "the raw type is a string and carries nothing: {document}"
        );

        // And the eight parts of FR-CTX-015, each present on the column and
        // `null` where FR-CTX-017 makes it null.
        for part in [
            r#""data_type":"bigint""#,
            r#""precision":20"#,
            r#""scale":0"#,
            r#""length":null"#,
            r#""unsigned":true"#,
            r#""charset":null"#,
            r#""collation":null"#,
            r#""values":null"#,
        ] {
            assert!(document.contains(part), "{part} is absent from {document}");
        }
    }

    #[test]
    fn fr_priv_016_a_restricted_marking_is_ordered_by_name_ascending_byte_wise() {
        // FR-PRIV-016 requires the array to be ordered, and the marking keeps
        // the order it was given until it becomes a document collection, which
        // is where every other collection is ordered too.
        let view = crate::model::view::View {
            restricted: Restricted::new(vec![
                Cow::Borrowed("definition"),
                Cow::Borrowed("algorithm"),
            ]),
            ..fixture::view("v_consignment_manifest")
        };
        let model = Database {
            views: vec![view],
            ..fixture::database()
        };

        assert!(
            compact(&model).contains(r#""restricted":["algorithm","definition"]"#),
            "{}",
            compact(&model)
        );
    }

    #[test]
    fn fr_cat_043_the_primary_key_is_read_back_from_the_index_collection_and_not_from_its_own_key()
    {
        // FR-CAT-043 makes the index catalogue table the authoritative source
        // and bars every other. The document presents the primary key a second
        // time so that a template need not match on a name; reading it back
        // would give the model a second place the two could disagree from, so
        // the read-back path takes it from the indexes and this key is ignored.
        let model = fixture::database();
        let document = compact(&model).replace(
            r#""primary_key":{"name":"PRIMARY","unique":true,"columns":[{"name":"consignment_id","direction":"A","prefix_length":null}]"#,
            r#""primary_key":{"name":"PRIMARY","unique":true,"columns":[{"name":"reference","direction":"A","prefix_length":null}]"#,
        );

        assert_ne!(
            document,
            compact(&model),
            "the rewrite reached the key it was aimed at"
        );

        let read_back = read(&document, &path()).expect("the document matches the contract");
        let key = read_back.tables[0]
            .primary_key()
            .expect("consignment has a primary key");

        assert_eq!(key.columns[0].name, "consignment_id");
        assert_eq!(read_back, model);
    }

    #[test]
    fn fr_err_034_the_path_a_document_was_read_from_is_carried_into_the_diagnostic() {
        // FR-ERR-034 obliges the cause line of this 65 to name the path.
        let reported = read("{", Path::new("/tmp/freight.json"))
            .expect_err("an unterminated object is not well-formed JSON");

        assert!(
            reported.to_string().contains("/tmp/freight.json"),
            "{reported}"
        );
        assert_eq!(reported.exit_code(), 65);
    }

    #[test]
    fn r_01_an_object_or_an_array_refused_at_its_bracket_is_named_by_its_own_path() {
        // Finding R-01 of the re-audit for rmp #269: the decoder reports the
        // offset before the bracket, and the path was the member before it.
        for collection in ["tables", "views", "routines"] {
            let mut document = editable();
            document["data"]["database"][collection] = serde_json::json!({});
            let ContextFault::Structure { at, expected } = refused(&document.to_string()) else {
                panic!("an object in place of an array is a structural fault");
            };
            assert_eq!(at, format!("data.database.{collection}"));
            assert_eq!(expected, "must be an array, but it is an object");
        }

        let mut document = editable();
        document["data"]["database"]["tables"] = serde_json::json!([[]]);
        let ContextFault::Structure { at, expected } = refused(&document.to_string()) else {
            panic!("an array in place of an object is a structural fault");
        };
        assert_eq!(at, "data.database.tables[0]");
        assert_eq!(expected, "must be an object, but it is an array");
    }

    #[test]
    fn r_06_no_json_text_at_all_is_its_own_fault() {
        assert_eq!(refused(""), ContextFault::Empty);
        assert_eq!(refused(" \n"), ContextFault::Empty);
    }

    #[test]
    fn fr_err_034_a_missing_key_is_named_by_its_path_and_a_wrong_type_by_what_was_expected() {
        let absent = refused(r#"{"schema_version":1,"source":"server","data":{}}"#);
        let ContextFault::Structure { at, expected } = absent else {
            panic!("an absent key is a structural fault");
        };
        assert_eq!(at, "data.database");
        assert_eq!(expected, "is missing; a context document requires it");

        let mut document = editable();
        document["data"]["database"]["name"] = serde_json::json!(7);
        let ContextFault::Structure { at, expected } = refused(&document.to_string()) else {
            panic!("a wrong type is a structural fault");
        };
        assert_eq!(at, "data.database.name");
        assert!(
            expected.starts_with("must be a string, but it is"),
            "{expected}"
        );
    }
}
