//! The document that carries the model, in both directions (`FR-CTX-001`,
//! `FR-SCH-017`, `FR-SCH-022`, `FR-RND-020`, `FR-OUT-016`).
//!
//! One document carries the model. `tpl schema dump` emits it, `tpl render
//! --context` consumes it, and a template sees the same material. This module
//! is the junction: [`dump`] builds the document from a model and [`read`]
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
//! either: [`read`] is given the bytes, so the one path that opens a file is
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

pub(crate) use shape::ContextData;

/// The rule a document that is JSON and not this contract has failed.
///
/// `ContextFault::Structure` carries a literal, so this is one rule for every
/// shape fault the decoder reports. It names the contract rather than the key,
/// which is the most the fault type can carry.
const DOCUMENT_CONTRACT: &str =
    "the document carries every key context-document.md fixes, each with the type it fixes";

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
/// (`FR-RND-020`, `FR-ERR-029`).
pub(crate) fn read<'a>(bytes: &'a str, path: &Path) -> Result<Database<'a>, Error> {
    let malformed = |fault| Error::ContextDocumentMalformed {
        path: path.to_owned(),
        fault,
    };

    // FR-SCH-036: the whole envelope, and not a bare `data` object in its
    // place. A document that omits `schema_version` is refused here, by the
    // three fields of `Document` having no defaults.
    let document: Document<ContextData<'a>> =
        serde_json::from_str(bytes).map_err(|reported| malformed(fault(&reported)))?;

    read::database(document.into_data().database).map_err(malformed)
}

/// Classifies what the decoder reported, per the `65` row of `FR-ERR-034`.
///
/// The row obliges the `cause` line to name the path and *either* the position
/// of the malformed JSON *or* the structural rule the document failed, and the
/// decoder's own classification is what separates the two.
fn fault(reported: &serde_json::Error) -> ContextFault {
    match reported.classify() {
        Category::Data => ContextFault::Structure {
            rule: DOCUMENT_CONTRACT,
        },
        // Syntax and Eof are both a document that is not well-formed JSON. Io
        // is unreachable: the bytes are already in memory, and a decode from a
        // string performs no read.
        Category::Syntax | Category::Eof | Category::Io => ContextFault::NotJson(Position {
            line: reported.line(),
            column: reported.column(),
        }),
    }
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
            Error::ContextDocumentMalformed { path: named, fault } => {
                assert_eq!(
                    named,
                    path(),
                    "FR-ERR-034 obliges the cause to name the path"
                );
                fault
            }
            other => panic!("a malformed context document is FR-RND-020's 65, not {other}"),
        }
    }

    #[test]
    fn a_document_serialised_and_read_back_yields_an_equal_model() {
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
    fn an_empty_collection_is_an_empty_array_and_is_never_null_and_never_omitted() {
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
    fn a_complete_object_carries_no_restricted_key_and_an_incomplete_one_carries_an_array() {
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
    fn the_compact_and_the_indented_form_are_both_accepted() {
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
    fn a_series_outside_the_supported_window_is_accepted() {
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
    fn a_document_missing_a_key_of_the_server_object_is_refused() {
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
    fn a_document_that_is_not_well_formed_json_is_refused_with_the_position() {
        // FR-RND-020 and the 65 row of FR-ERR-034, whose cause line carries
        // either the position of the malformed JSON or the structural rule.
        let fault = refused("{\"schema_version\":1,\n  \"source\": oops}");

        let ContextFault::NotJson(position) = fault else {
            panic!("bytes that are not JSON are the position half of FR-ERR-034");
        };

        assert_eq!(position.line, 2);
    }

    #[test]
    fn a_bare_data_object_is_refused_in_place_of_the_whole_envelope() {
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
    fn a_restricted_marking_that_names_no_property_is_refused_rather_than_repaired() {
        // FR-PRIV-016: the array is never empty. Reading it as a complete
        // object would turn a document the requirement forbids into one it
        // permits, silently.
        let document =
            compact(&fixture::database()).replace(r#""restricted":["body"]"#, r#""restricted":[]"#);

        assert!(matches!(refused(&document), ContextFault::Structure { .. }));
    }

    #[test]
    fn a_table_whose_key_names_a_column_it_does_not_carry_is_refused() {
        // FR-CAT-044, through Table::assemble, which is the model's only
        // constructor for a table. The same violation is an internal invariant
        // on the server path and caller data here, which is why the model
        // reports it neutrally and each caller maps it.
        let document = compact(&fixture::database()).replace(
            r#"{"name":"leg_id","direction":"A","prefix_length":null}"#,
            r#"{"name":"row_end","direction":"A","prefix_length":null}"#,
        );

        let ContextFault::Structure { rule } = refused(&document) else {
            panic!("a key naming an absent column is a structural fault");
        };

        assert!(rule.contains("names a column that table carries"), "{rule}");
    }

    #[test]
    fn the_dump_is_the_envelope_of_the_requirement_carrying_one_data_key() {
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
    fn a_column_default_is_emitted_in_exactly_the_three_forms_the_requirement_lists() {
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
    fn a_restricted_marking_is_ordered_by_name_ascending_byte_wise() {
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
    fn the_primary_key_is_read_back_from_the_index_collection_and_not_from_its_own_key() {
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
    fn the_path_a_document_was_read_from_is_carried_into_the_diagnostic() {
        // FR-ERR-034 obliges the cause line of this 65 to name the path.
        let reported = read("{", Path::new("/tmp/freight.json"))
            .expect_err("an unterminated object is not well-formed JSON");

        assert!(
            reported.to_string().contains("/tmp/freight.json"),
            "{reported}"
        );
        assert_eq!(reported.exit_code(), 65);
    }
}
