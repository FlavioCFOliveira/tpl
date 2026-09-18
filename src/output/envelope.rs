//! The three keys of `FR-OUT-024`, the four values of `FR-OUT-026`, and the
//! collection shape of `FR-OUT-030`.
//!
//! Each of the three is a **type**, not a convention, because each requirement
//! it carries is a prohibition that a convention cannot enforce:
//!
//! | Requirement | What the type does with it |
//! |---|---|
//! | `FR-OUT-024` — exactly three keys, in one order | [`Document`] declares three fields, and `OD-18` makes declaration order the key order |
//! | `FR-OUT-025` — `schema_version` is the first key | It is the first field, and [`Document::new`] is the only constructor, so no caller supplies the value |
//! | `FR-OUT-026` — `source` is one of exactly four values | [`Source`] is a closed enum of four variants |
//! | `FR-OUT-028` — nothing beside `data`, and no fourth key | The struct is closed and its fields are private; there is no route to a fourth key |
//! | `FR-OUT-029` — `source` is never a boolean, and `restricted` is never on the envelope | [`Source`] is an enumerated string, and the envelope has no field a per-object marking could occupy |
//! | `FR-OUT-030`, `FR-OUT-035` — one key in the plural, carrying an array that is empty rather than absent | [`Collection`] holds a slice, so the key always exists and its value is always an array |

use serde::ser::SerializeStruct as _;
use serde::{Deserialize, Serialize, Serializer};

/// The version of the document contract, per `FR-OUT-011` and `FR-OUT-025`.
///
/// It versions the documents independently of the binary: what moves it is a
/// change on one of the three breaking rows of `FR-OUT-014` — removing a field,
/// renaming one, or changing a field's type. Adding a field, or adding a value
/// to [`Source`], does not.
///
/// It is written in one place and read by [`Document::new`] alone, so a
/// document cannot be emitted carrying a different one.
const SCHEMA_VERSION: u32 = 1;

/// The name `serialize_struct` is given for [`Collection`].
///
/// A self-describing format ignores it — JSON writes the field name and not
/// this — and JSON is the only format this crate serialises to.
const COLLECTION: &str = "Collection";

/// Where the bytes of a document's payload came from (`FR-OUT-026`).
///
/// The set is closed and the value answers one question: is this result live,
/// or may it be stale. Every document carries it, so a caller never has to know
/// which commands have the field before it can read one.
///
/// `FR-CDOC-010` requires an enumerated string rather than a boolean, because
/// `FR-OUT-014` lets an enumerated field gain a value without breaking the
/// contract and a boolean cannot gain one. Two of the four values arrived that
/// way.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub(crate) enum Source {
    /// Read from a server on this invocation.
    Server,
    /// Served from `.tpl/.cache/`, per `FR-CACHE-006`.
    Cache,
    /// Read from `.tpl/` alone, with no catalogue involved.
    Project,
    /// Produced by the binary from itself, with nothing read.
    Binary,
}

/// One JSON document: the envelope of `FR-OUT-024` around a payload.
///
/// ```json
/// {"schema_version":1,"source":"server","data":{…}}
/// ```
///
/// The three fields are private and [`Document::new`] is the only constructor,
/// which is what makes `FR-OUT-025` and `FR-OUT-028` structural: the version is
/// not a caller's to supply, and there is no fourth key for one command's
/// benefit because there is no fourth field.
///
/// `T` is the payload, whose shape belongs to the module that owns the command
/// producing it (`FR-OUT-027`, `BR-OUT-002`). A listing passes a
/// [`Collection`]; a single named object passes a struct of its own with one
/// field, per `FR-OUT-031`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct Document<T> {
    /// The version of the document contract. First, per `FR-OUT-025`.
    schema_version: u32,
    /// Where the bytes of `data` came from.
    source: Source,
    /// The whole of the command's result, per `FR-OUT-027`.
    data: T,
}

impl<T> Document<T> {
    /// Envelopes `data`, declaring where its bytes came from.
    ///
    /// The version is [`SCHEMA_VERSION`] and is not a parameter: `FR-OUT-025`
    /// fixes the key and `FR-OUT-014` fixes what moves its value, so a caller
    /// has no occasion to choose one.
    pub(crate) const fn new(source: Source, data: T) -> Self {
        Self {
            schema_version: SCHEMA_VERSION,
            source,
            data,
        }
    }

    /// The payload, taken out of the envelope.
    ///
    /// It is the counterpart of [`Document::new`] on the one path that reads a
    /// document rather than writing one: `FR-SCH-036` requires a `--context`
    /// document to carry the whole envelope and refuses a bare `data` object in
    /// its place, so the envelope is what is decoded and this is what a reader
    /// continues from.
    ///
    /// `schema_version` and `source` are not returned. `FR-OUT-025` keeps the
    /// version out of a caller's hands, and `FR-CDOC-016` — not this module —
    /// owns what the value `cache` withdraws from a document that was read.
    pub(crate) fn into_data(self) -> T {
        self.data
    }
}

/// The payload shape of a command that returns a collection (`FR-OUT-030`).
///
/// ```json
/// {"tables":[…]}
/// ```
///
/// One key, named for the collection in the plural, carrying a JSON array of
/// its members. The members are borrowed rather than owned, so enveloping a
/// listing copies nothing.
///
/// `FR-OUT-035` is satisfied by the type rather than by a branch: the key is a
/// field and cannot be omitted, and the value is a slice and is therefore an
/// array whatever its length — `[]` on an empty result, never `null` and never
/// absent. `FR-OUT-033` makes that result a success, so an empty listing is the
/// ordinary first state of a project rather than a case to special-case.
///
/// The one key's name varies per collection and is therefore a value rather
/// than a literal in a derive, which is why this is the crate's one hand-written
/// [`Serialize`]. `OD-18` rejects a bespoke writer on the ground that it states
/// a key order the types already state; a shape with exactly one key states no
/// order, so the ground does not reach it. The implementation serialises a
/// **struct** of one field rather than a map, so no map — ordered or otherwise
/// — appears on the emitting path, per `FR-OUT-013`.
#[derive(Debug)]
pub(crate) struct Collection<'a, T> {
    /// The collection's name, in the plural.
    key: &'static str,
    /// Its members, in the order `NFR-DET-002` fixes for them.
    members: &'a [T],
}

impl<'a, T> Collection<'a, T> {
    /// Names a collection and its members.
    ///
    /// `key` is the plural `FR-OUT-030` requires — `tables`, `views`,
    /// `routines`, `templates`. `members` are already in the order
    /// `NFR-DET-002` fixes; this type applies none of its own.
    pub(crate) const fn new(key: &'static str, members: &'a [T]) -> Self {
        Self { key, members }
    }
}

impl<T: Serialize> Serialize for Collection<'_, T> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut payload = serializer.serialize_struct(COLLECTION, 1)?;
        payload.serialize_field(self.key, &self.members)?;
        payload.end()
    }
}

#[cfg(test)]
mod tests {
    use super::{Collection, Document, Source};
    use serde::Serialize;

    /// A payload with an absent value, for `FR-OUT-012`.
    #[derive(Serialize)]
    struct Table {
        name: &'static str,
        comment: Option<&'static str>,
    }

    #[test]
    fn fr_out_024_the_envelope_carries_the_three_keys_of_the_requirement_in_order() {
        // FR-OUT-024, FR-OUT-025, FR-OUT-028: exactly three keys, in this
        // order, with nothing beside `data`.
        let document = Document::new(Source::Server, Collection::new("tables", &["orders"]));

        assert_eq!(
            serde_json::to_string(&document).expect("the envelope serialises"),
            r#"{"schema_version":1,"source":"server","data":{"tables":["orders"]}}"#
        );
    }

    #[test]
    fn fr_out_026_source_takes_exactly_the_four_values_of_the_requirement() {
        // FR-OUT-026, FR-CDOC-010: four enumerated strings, never a boolean.
        let values: Vec<String> = [
            Source::Server,
            Source::Cache,
            Source::Project,
            Source::Binary,
        ]
        .into_iter()
        .map(|source| serde_json::to_string(&source).expect("a value serialises"))
        .collect();

        assert_eq!(
            values,
            vec![
                r#""server""#.to_owned(),
                r#""cache""#.to_owned(),
                r#""project""#.to_owned(),
                r#""binary""#.to_owned(),
            ]
        );
    }

    #[test]
    fn fr_out_030_a_collection_is_one_key_in_the_plural_carrying_an_array() {
        // FR-OUT-030: `data` is an object of one key, named for the collection
        // in the plural, whose value is an array of its members.
        let document = Document::new(
            Source::Cache,
            Collection::new("views", &["daily_sales", "stock_on_hand"]),
        );

        assert_eq!(
            serde_json::to_string(&document).expect("the document serialises"),
            r#"{"schema_version":1,"source":"cache","data":{"views":["daily_sales","stock_on_hand"]}}"#
        );
    }

    #[test]
    fn fr_out_035_an_empty_collection_carries_its_key_with_an_empty_array() {
        // FR-OUT-035: the key is not omitted and its value is not `null`.
        let empty: [&str; 0] = [];
        let document = Document::new(Source::Project, Collection::new("templates", &empty));

        assert_eq!(
            serde_json::to_string(&document).expect("the document serialises"),
            r#"{"schema_version":1,"source":"project","data":{"templates":[]}}"#
        );
    }

    #[test]
    fn fr_out_012_an_absent_value_is_null_and_is_not_omitted() {
        // FR-OUT-012: the shape of a document is constant, so an absent value
        // is emitted as `null`. OD-18 confines `skip_serializing_if` to one
        // field of one type, which is not written yet.
        let tables = [Table {
            name: "orders",
            comment: None,
        }];
        let document = Document::new(Source::Server, Collection::new("tables", &tables));

        assert_eq!(
            serde_json::to_string(&document).expect("the document serialises"),
            r#"{"schema_version":1,"source":"server","data":{"tables":[{"name":"orders","comment":null}]}}"#
        );
    }

    #[test]
    fn fr_out_011_the_schema_version_is_the_one_the_corpus_publishes() {
        // FR-OUT-011, FR-OUT-025: the number every worked example in the
        // specification carries. A release gate checks that it moved only on a
        // breaking row of FR-OUT-014.
        let document = Document::new(Source::Binary, Collection::new("commands", &["init"]));

        assert!(
            serde_json::to_string(&document)
                .expect("the document serialises")
                .starts_with(r#"{"schema_version":1,"#)
        );
    }
}
