//! A column's type: the raw string of `FR-CTX-014`, and the eight decomposed
//! parts of `FR-CTX-015` that are read from it and from the fields beside it.
//!
//! The catalogue states a column's type twice — once as a string a human wrote
//! and the server rewrote, and once as a set of discrete fields — and neither
//! statement is sufficient on its own. The string carries the member list of an
//! `ENUM` and the `unsigned` attribute, which no field carries; the fields
//! carry the precision, the scale and the length, which the string states only
//! by position. The decomposition is therefore a **merge**, and `FR-CTX-040`
//! names the one source of each part so that the merge is checkable.
//!
//! | Requirement | What the type does with it |
//! |---|---|
//! | `FR-CTX-014` — the type exactly as the server writes it | The raw string is carried borrowed and unaltered, whatever else is read from it |
//! | `FR-CTX-015`, `FR-CTX-040` — eight parts, each from one named field | [`ColumnType::decompose`] reads each part from the field the requirement names, and from no other |
//! | `FR-CTX-016`, `FR-CTX-039` — the member list of an `ENUM` or a `SET` | `values` is scanned by **quote state**, never by splitting on the comma |
//! | `FR-CTX-017` — a part that does not apply is `null` | Every part is an [`Option`], and a part read from the catalogue is [`None`] exactly where its field is SQL `NULL` |
//! | `FR-CTX-018` — an unrecognised type keeps the raw string | A `data_type` outside the observed set nulls all eight parts and leaves the raw string as the template's safety net |
//! | `FR-CTX-038` — `unsigned` lives in the raw string alone | It is derived from that string, and never from `data_type`, which does not carry it |
//! | `FR-CTX-041` — `charset` and `collation` are one pair | They are read from their two fields, passed through verbatim, and never inferred from the type |
//!
//! **The decomposition cannot fail.** A type the system does not recognise is
//! an outcome of `FR-CTX-018` rather than an error, so this module reports
//! none.
//!
//! *[`ColumnType`] is the one type of the graph whose read-back does not
//! re-establish its own invariant, and the reason is `FR-CTX-033`.* The
//! document carries the **decomposed** parts, not the catalogue fields
//! [`ColumnType::decompose`] reads them from, so there is nothing to
//! re-decompose; and that requirement bars the system from validating a
//! supplied document beyond the three keys of `FR-CTX-031`. A hand-written
//! document may therefore state parts that no read of a supported series would
//! have produced together. `FR-CTX-018` keeps the consequence bounded:
//! `column_type` is carried unaltered whatever the parts say, so the template's
//! safety net survives a document the system did not write.
//!
//! *The failure this module exists to prevent* is the naive one. The separator
//! of a member list and a member's own content are the same byte, `2C`, so
//! nothing but the quote state tells them apart: a reader that splits the
//! fixture's four-member `imdg_class` on the comma reports seven members, three
//! of them fragments, at exit `0`.

use std::borrow::Cow;

use serde::{Deserialize, Serialize};

use super::collapse_doubled_apostrophes;

/// The `data_type` of a column whose raw string carries a member list.
const ENUM: &str = "enum";

/// The other one.
const SET: &str = "set";

/// The attribute of `FR-CTX-038`, in the lower case the catalogue writes it in.
const UNSIGNED: &str = "unsigned";

/// The quote each member of a list is delimited by, per `FR-CTX-039`.
const QUOTE: char = '\'';

/// The character the member list, and a display width, open with.
const OPEN: char = '(';

/// The character they close with.
const CLOSE: char = ')';

/// The `data_type` values the system recognises, per `FR-CTX-018`.
///
/// These are the 39 values the fixture's 301 columns yielded, byte-identical on
/// all four series of `FR-SRV-015`. `FR-CTX-040` assigns every one of them a
/// row in its table of populated parts, which is what makes a value outside
/// this set a type whose parts the system has no rule for — and therefore the
/// case `FR-CTX-018` sends to the raw string. `FR-ENV-046` enumerates the same
/// 39 and partitions them into families, which is where the eight geometry
/// types `FR-CTX-040` names by count are named individually.
///
/// Two values that a DDL can write are deliberately absent, and both are
/// absent because the catalogue does not report them: a column declared `JSON`
/// reports `longtext`, per `FR-CAT-038`, and one declared `NUMERIC` reports
/// `decimal`.
///
/// The array is sorted, which [`recognised`] relies on and a test enforces.
const RECOGNISED_DATA_TYPES: [&str; 39] = [
    "bigint",
    "binary",
    "bit",
    "blob",
    "char",
    "date",
    "datetime",
    "decimal",
    "double",
    "enum",
    "float",
    "geometry",
    "geometrycollection",
    "inet4",
    "inet6",
    "int",
    "linestring",
    "longblob",
    "longtext",
    "mediumblob",
    "mediumint",
    "mediumtext",
    "multilinestring",
    "multipoint",
    "multipolygon",
    "point",
    "polygon",
    "set",
    "smallint",
    "text",
    "time",
    "timestamp",
    "tinyblob",
    "tinyint",
    "tinytext",
    "uuid",
    "varbinary",
    "varchar",
    "year",
];

/// The catalogue's statement of one column's type.
///
/// This is the input side of the decomposition: the raw type string of
/// `FR-CTX-014` together with the six catalogue fields `FR-CTX-040` names as
/// the sources of the parts read rather than derived. Every field is carried
/// exactly as the catalogue returned it, [`None`] standing for SQL `NULL`, so
/// that the mapping from field to part stays checkable against the requirement.
///
/// *This is the one type of the module that is **not** `#[non_exhaustive]`,
/// and the reason is that it is an input.* A caller has to be able to write it
/// down, and the attribute would make that impossible outside this crate. The
/// value it produces, [`ColumnType`], carries the attribute for both.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct CatalogueType<'a> {
    /// The type exactly as the server writes it, per `FR-CTX-014`.
    pub column_type: &'a str,

    /// The data-type field, which never carries the unsigned attribute.
    pub data_type: &'a str,

    /// The numeric-precision field.
    pub numeric_precision: Option<u64>,

    /// The datetime-precision field, which is SQL `NULL` for every type that
    /// populates the numeric one.
    pub datetime_precision: Option<u64>,

    /// The numeric-scale field.
    pub numeric_scale: Option<u64>,

    /// The character-maximum-length field, which for an `ENUM` or a `SET` is
    /// the length of the longest member.
    pub character_maximum_length: Option<u64>,

    /// The character-set field, set for the eight textual types and SQL `NULL`
    /// for every other, per `FR-CTX-041`.
    pub charset: Option<&'a str>,

    /// The collation field, which is set and null with [`Self::charset`] and
    /// never apart from it.
    pub collation: Option<&'a str>,
}

/// A column's type, decomposed (`FR-CTX-014`, `FR-CTX-015`).
///
/// The raw string and the eight parts, in one value. The fields are private
/// and [`ColumnType::decompose`] is the only constructor, which is what makes
/// three requirements structural rather than conventional: the parts cannot be
/// populated from a field other than the one `FR-CTX-040` names, the two
/// precision fields cannot be carried as two parts, and a type the system does
/// not recognise cannot reach a caller with a part filled in.
///
/// Every part borrows the catalogue strings it was read from. A member of a
/// list is copied in exactly one case — where it carries a doubled apostrophe,
/// and one byte has to be dropped.
///
/// *The four string parts are [`Cow<'a, str>`](Cow) and not `&'a str`, and the
/// reason is the document rather than the catalogue.* A live read always
/// borrows, so `&'a str` was sufficient while the model was only ever built
/// from catalogue rows. `FR-SCH-022` makes the document a second source:
/// `serde_json` borrows a JSON string only where it needed no unescaping, and
/// a raw type string such as `enum('8''6"')` carries a quote that JSON escapes,
/// so it arrives **owned**. The four fields therefore hold what
/// [`Cow`] holds everywhere else in the graph — borrowed from the
/// source where the source allows it, owned where it does not — and one type
/// serves both producers.
///
/// [`CatalogueType`] keeps `&'a str`, because it is the catalogue's statement
/// and a catalogue row is always borrowed.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct ColumnType<'a> {
    /// The raw type string, carried unaltered under every outcome.
    #[serde(borrow)]
    column_type: Cow<'a, str>,

    /// The data-type field, or [`None`] under `FR-CTX-018`.
    #[serde(borrow)]
    data_type: Option<Cow<'a, str>>,

    /// The two precision fields of `FR-CTX-040`, merged.
    precision: Option<u64>,

    /// The numeric-scale field.
    scale: Option<u64>,

    /// The character-maximum-length field.
    length: Option<u64>,

    /// Derived from the raw string, per `FR-CTX-038`.
    unsigned: Option<bool>,

    /// The character-set field.
    #[serde(borrow)]
    charset: Option<Cow<'a, str>>,

    /// The collation field.
    #[serde(borrow)]
    collation: Option<Cow<'a, str>>,

    /// The member list of an `ENUM` or a `SET`, per `FR-CTX-016`. Its order is
    /// the stored ordinal and is one of the six exceptions of `NFR-DET-002`.
    #[serde(borrow)]
    values: Option<Vec<Cow<'a, str>>>,
}

impl<'a> ColumnType<'a> {
    /// Decomposes the catalogue's statement of a type into the parts a
    /// template reads.
    ///
    /// The parts are read as `FR-CTX-040` fixes: `data_type`, `scale`, `length`
    /// and the pair `charset`/`collation` from their own fields; `precision`
    /// from the numeric-precision field, falling back to the datetime-precision
    /// field, which is lossless because **no type populates both**; and
    /// `unsigned` and `values` from the raw string, which is the only place
    /// either of them exists.
    ///
    /// A `data_type` the system does not recognise takes the whole value down
    /// the path of `FR-CTX-018`: the raw string is kept and all eight parts are
    /// [`None`], so a type introduced by a later server reaches the template
    /// intact rather than being guessed at.
    ///
    /// Two parts behave unlike the others, and both do so because they are
    /// **derived** rather than read. `FR-CTX-017` makes a read part `null`
    /// exactly where its field is SQL `NULL`, and `FR-CTX-040` forbids the
    /// system to decide applicability on its own account; there is no field
    /// behind `unsigned` to be SQL `NULL`, so a recognised type that does not
    /// carry the attribute reports `false` rather than `null`. `values` is the
    /// one part with a rule of its own: `FR-CTX-016` makes it `null` for every
    /// type but `ENUM` and `SET`.
    #[must_use]
    pub fn decompose(catalogue: &CatalogueType<'a>) -> Self {
        if !recognised(catalogue.data_type) {
            return Self::unrecognised(catalogue.column_type);
        }

        Self {
            column_type: Cow::Borrowed(catalogue.column_type),
            data_type: Some(Cow::Borrowed(catalogue.data_type)),
            precision: catalogue.numeric_precision.or(catalogue.datetime_precision),
            scale: catalogue.numeric_scale,
            length: catalogue.character_maximum_length,
            unsigned: Some(carries_unsigned(catalogue.column_type)),
            charset: catalogue.charset.map(Cow::Borrowed),
            collation: catalogue.collation.map(Cow::Borrowed),
            values: members(catalogue.data_type, catalogue.column_type),
        }
    }

    /// The raw string of `FR-CTX-018`: every part `null`, the type unchanged.
    fn unrecognised(column_type: &'a str) -> Self {
        Self {
            column_type: Cow::Borrowed(column_type),
            data_type: None,
            precision: None,
            scale: None,
            length: None,
            unsigned: None,
            charset: None,
            collation: None,
            values: None,
        }
    }

    /// The type exactly as the server writes it, per `FR-CTX-014`.
    ///
    /// This is the part that is never absent, and the part `FR-CTX-018` makes
    /// the fallback: a template can always match on it.
    #[must_use]
    pub fn raw(&self) -> &str {
        &self.column_type
    }

    /// The data-type field, without the unsigned attribute it never carries.
    #[must_use]
    pub fn data_type(&self) -> Option<&str> {
        self.data_type.as_deref()
    }

    /// The numeric precision, or the datetime precision where that is the field
    /// the catalogue populates.
    ///
    /// Three of its values were written by the server and not by any author: a
    /// `FLOAT` reports `12`, a `DOUBLE` reports `22`, and a `DATETIME` with no
    /// fractional part reports `0` rather than [`None`]. A caller testing for
    /// absence must test for [`None`] and not for a falsy zero.
    #[must_use]
    pub const fn precision(&self) -> Option<u64> {
        self.precision
    }

    /// The numeric scale, which every integer type reports as `0`.
    #[must_use]
    pub const fn scale(&self) -> Option<u64> {
        self.scale
    }

    /// The character maximum length, which for an `ENUM` or a `SET` is the
    /// length of the longest member rather than of the type string.
    #[must_use]
    pub const fn length(&self) -> Option<u64> {
        self.length
    }

    /// Whether the raw string carries the unsigned attribute, per
    /// `FR-CTX-038`.
    ///
    /// [`None`] only under `FR-CTX-018`: a recognised type that does not carry
    /// the attribute reports `false`.
    #[must_use]
    pub const fn unsigned(&self) -> Option<bool> {
        self.unsigned
    }

    /// The column's character set, per `FR-CTX-041`.
    ///
    /// Set for the eight textual types and [`None`] for every other, together
    /// with [`Self::collation`] and never apart from it. An inherited value is
    /// reported explicitly and is indistinguishable from a declared one.
    #[must_use]
    pub fn charset(&self) -> Option<&str> {
        self.charset.as_deref()
    }

    /// The column's collation, per `FR-CTX-041`.
    #[must_use]
    pub fn collation(&self) -> Option<&str> {
        self.collation.as_deref()
    }

    /// The member list of an `ENUM` or a `SET`, in the order the catalogue
    /// states, and [`None`] for every other type (`FR-CTX-016`).
    ///
    /// The order is the declaration order and carries the ordinal each member
    /// is stored as, so it is meaning rather than presentation and is one of
    /// the collections `NFR-DET-002` does not sort.
    #[must_use]
    pub fn values(&self) -> Option<&[Cow<'a, str>]> {
        self.values.as_deref()
    }
}

/// Whether a `data_type` is one the system recognises, per `FR-CTX-018`.
fn recognised(data_type: &str) -> bool {
    RECOGNISED_DATA_TYPES.binary_search(&data_type).is_ok()
}

/// Whether a raw type string carries the unsigned attribute, per `FR-CTX-038`.
///
/// The attribute is a word of the string, in lower case, after the display
/// width. The search is therefore made over what **follows the last `)`**, and
/// that restriction is not decoration: the only place the observed grammar
/// admits an authored space inside the string is a member list, and a member
/// reading `Class 3, unsigned cargo` would otherwise report an `ENUM` as
/// unsigned. A string with no parentheses is searched whole, which no observed
/// type without a width carries a space in.
fn carries_unsigned(column_type: &str) -> bool {
    let attributes = match column_type.rfind(CLOSE) {
        // `)` is one ASCII byte, so the index after it is a character boundary,
        // and it is the end of the string where the type carries nothing else.
        Some(close) => &column_type[close + CLOSE.len_utf8()..],
        None => column_type,
    };

    attributes
        .split_whitespace()
        .any(|attribute| attribute == UNSIGNED)
}

/// The member list of an `ENUM` or a `SET`, or [`None`] for any other type.
///
/// The list begins after `enum(` or `set(` and ends at the **final** `)`, per
/// `FR-CTX-039`, which is what admits a member carrying either character.
///
/// A raw string that carries no parenthesised list yields an empty list rather
/// than a failure. The shape was not observed — every `ENUM` and `SET` of the
/// fixture carries its members — and `FR-CTX-018` reaches only an unrecognised
/// `data_type`, which `enum` and `set` are not.
fn members<'a>(data_type: &str, column_type: &'a str) -> Option<Vec<Cow<'a, str>>> {
    if data_type != ENUM && data_type != SET {
        return None;
    }

    let (Some(open), Some(close)) = (column_type.find(OPEN), column_type.rfind(CLOSE)) else {
        return Some(Vec::new());
    };

    if close <= open {
        return Some(Vec::new());
    }

    // Both delimiters are one ASCII byte, so both indices are character
    // boundaries and the slice cannot split a multi-byte member.
    Some(scan(&column_type[open + OPEN.len_utf8()..close]))
}

/// Reads a member list by quote state, per `FR-CTX-039`.
///
/// Each member is delimited by a single quote on each side, and the members are
/// separated by a comma **between** a closing quote and an opening one. The
/// scanner therefore never looks at the separator: it finds an opening quote,
/// reads to the matching closing quote, and repeats. A comma, a parenthesis and
/// a multi-byte character inside a member are bytes like any other.
fn scan(list: &str) -> Vec<Cow<'_, str>> {
    let mut members = Vec::new();
    let mut rest = list;

    while let Some(open) = rest.find(QUOTE) {
        let body = &rest[open + QUOTE.len_utf8()..];

        let Some(close) = closing(body) else {
            // An unterminated member: the string ends inside a quoted run,
            // which the catalogue was not observed to produce. What has been
            // read is kept, and the fragment is dropped rather than reported as
            // a member the server never wrote.
            break;
        };

        members.push(collapse_doubled_apostrophes(&body[..close]));
        rest = &body[close + QUOTE.len_utf8()..];
    }

    members
}

/// The index of the quote that closes a member, given the text just after the
/// quote that opened it.
///
/// A doubled apostrophe is one apostrophe **of the member**, per `FR-CTX-039`,
/// so a quote followed by a quote continues the member rather than ending it.
/// There is no backslash escape to consider: the catalogue was observed never
/// to write one.
fn closing(body: &str) -> Option<usize> {
    let mut from = 0;

    while let Some(found) = body[from..].find(QUOTE) {
        let at = from + found;
        let after = at + QUOTE.len_utf8();

        if body[after..].starts_with(QUOTE) {
            from = after + QUOTE.len_utf8();
            continue;
        }

        return Some(at);
    }

    None
}

#[cfg(test)]
mod tests {
    use std::borrow::Cow;

    use super::{CatalogueType, ColumnType, RECOGNISED_DATA_TYPES};

    /// The four-member `ENUM` of the fixture's `cargo_item.imdg_class`, whose
    /// second member carries a bare comma.
    const IMDG_CLASS: &str = "enum('Not regulated','Class 3, Flammable liquids',\
                              'Class 8, Corrosive substances','Class 9, Miscellaneous')";

    /// The `ENUM` of the fixture's `vessel.class_society`, whose first member
    /// carries a doubled apostrophe.
    const CLASS_SOCIETY: &str = "enum('Lloyd''s Register','DNV','Bureau Veritas')";

    /// The four parts the catalogue states as discrete fields, as one value a
    /// test can compare in a line.
    fn parts<'a>(
        column: &'a ColumnType<'a>,
    ) -> (Option<&'a str>, Option<u64>, Option<u64>, Option<u64>) {
        (
            column.data_type(),
            column.precision(),
            column.scale(),
            column.length(),
        )
    }

    #[test]
    fn the_unsigned_attribute_is_read_from_the_raw_string_and_the_data_type_is_shared() {
        // FR-CTX-038: `unsigned` appears inside the raw string only, after the
        // display width; `data_type` reads the same for the signed and the
        // unsigned form, so it cannot be the source.
        let unsigned = ColumnType::decompose(&CatalogueType {
            column_type: "bigint(20) unsigned",
            data_type: "bigint",
            numeric_precision: Some(20),
            numeric_scale: Some(0),
            ..CatalogueType::default()
        });
        let signed = ColumnType::decompose(&CatalogueType {
            column_type: "bigint(20)",
            data_type: "bigint",
            numeric_precision: Some(19),
            numeric_scale: Some(0),
            ..CatalogueType::default()
        });

        assert_eq!(unsigned.unsigned(), Some(true));
        assert_eq!(unsigned.data_type(), Some("bigint"));
        assert_eq!(signed.unsigned(), Some(false));
        assert_eq!(
            unsigned.data_type(),
            signed.data_type(),
            "the two forms share one data_type"
        );
    }

    #[test]
    fn a_member_carrying_the_attribute_does_not_make_an_enum_unsigned() {
        // The attribute follows the display width, so the search runs over what
        // follows the final `)` and never over the member list.
        let column = ColumnType::decompose(&CatalogueType {
            column_type: "enum('unsigned cargo','signed cargo')",
            data_type: "enum",
            character_maximum_length: Some(15),
            ..CatalogueType::default()
        });

        assert_eq!(column.unsigned(), Some(false));
    }

    #[test]
    fn a_member_list_is_read_by_quote_state_and_a_bare_comma_stays_inside_its_member() {
        // FR-CTX-039: the separator and a member's own content are the same
        // byte. Splitting on the comma reports seven members, three of them
        // fragments, at exit 0.
        let column = ColumnType::decompose(&CatalogueType {
            column_type: IMDG_CLASS,
            data_type: "enum",
            character_maximum_length: Some(38),
            charset: Some("utf8mb4"),
            collation: Some("utf8mb4_unicode_520_ci"),
            ..CatalogueType::default()
        });

        assert_eq!(
            column.values(),
            Some(
                [
                    "Not regulated",
                    "Class 3, Flammable liquids",
                    "Class 8, Corrosive substances",
                    "Class 9, Miscellaneous",
                ]
                .map(Cow::Borrowed)
                .as_slice()
            ),
            "four members, not the seven a comma split reports"
        );
    }

    #[test]
    fn a_doubled_apostrophe_inside_a_member_collapses_to_one() {
        // FR-CTX-039: an apostrophe inside a member is doubled and is never
        // backslash-escaped, and the member ends at the quote that is not.
        let column = ColumnType::decompose(&CatalogueType {
            column_type: CLASS_SOCIETY,
            data_type: "enum",
            character_maximum_length: Some(16),
            ..CatalogueType::default()
        });

        assert_eq!(
            column.values(),
            Some(
                ["Lloyd's Register", "DNV", "Bureau Veritas"]
                    .map(Cow::Borrowed)
                    .as_slice()
            )
        );
    }

    #[test]
    fn a_member_list_passes_every_other_byte_through_including_multi_byte_ones() {
        // FR-CTX-039: the em dash of the fixture survives intact, and so does a
        // member that is the empty string.
        let column = ColumnType::decompose(&CatalogueType {
            column_type: "set('Method 1 — weighbridge','','a)b')",
            data_type: "set",
            character_maximum_length: Some(22),
            ..CatalogueType::default()
        });

        assert_eq!(
            column.values(),
            Some(
                ["Method 1 — weighbridge", "", "a)b"]
                    .map(Cow::Borrowed)
                    .as_slice()
            )
        );
    }

    #[test]
    fn a_type_that_is_neither_an_enum_nor_a_set_carries_no_member_list() {
        // FR-CTX-016: `values` is null for every other type.
        let column = ColumnType::decompose(&CatalogueType {
            column_type: "varchar(255)",
            data_type: "varchar",
            character_maximum_length: Some(255),
            charset: Some("utf8mb4"),
            collation: Some("utf8mb4_unicode_520_ci"),
            ..CatalogueType::default()
        });

        assert_eq!(column.values(), None);
        assert_eq!(column.charset(), Some("utf8mb4"));
        assert_eq!(column.collation(), Some("utf8mb4_unicode_520_ci"));
    }

    #[test]
    fn an_unrecognised_type_keeps_the_raw_string_and_nulls_every_part() {
        // FR-CTX-018: the raw string is the safety net, so a type a later
        // server introduces reaches the template intact rather than guessed at.
        let column = ColumnType::decompose(&CatalogueType {
            column_type: "vector(1536)",
            data_type: "vector",
            numeric_precision: Some(1536),
            numeric_scale: Some(0),
            character_maximum_length: Some(6144),
            charset: Some("binary"),
            collation: Some("binary"),
            ..CatalogueType::default()
        });

        assert_eq!(column.raw(), "vector(1536)");
        assert_eq!(parts(&column), (None, None, None, None));
        assert_eq!(column.unsigned(), None);
        assert_eq!(column.charset(), None);
        assert_eq!(column.collation(), None);
        assert_eq!(column.values(), None);
    }

    #[test]
    fn the_two_precision_fields_merge_into_one_part() {
        // FR-CTX-040: no type populates both, so the merge is lossless and
        // `datetime(6)`'s 6 stays reachable through a part list naming
        // `precision` once.
        let datetime = ColumnType::decompose(&CatalogueType {
            column_type: "datetime(6)",
            data_type: "datetime",
            datetime_precision: Some(6),
            ..CatalogueType::default()
        });
        let decimal = ColumnType::decompose(&CatalogueType {
            column_type: "decimal(12,3)",
            data_type: "decimal",
            numeric_precision: Some(12),
            numeric_scale: Some(3),
            ..CatalogueType::default()
        });

        assert_eq!(parts(&datetime), (Some("datetime"), Some(6), None, None));
        assert_eq!(parts(&decimal), (Some("decimal"), Some(12), Some(3), None));
    }

    #[test]
    fn a_zero_precision_is_a_value_and_not_an_absence() {
        // FR-CTX-040: a DATETIME with no fractional part reports 0, where a
        // DATE reports SQL NULL. A caller testing absence must test for None.
        let datetime = ColumnType::decompose(&CatalogueType {
            column_type: "datetime",
            data_type: "datetime",
            datetime_precision: Some(0),
            ..CatalogueType::default()
        });
        let date = ColumnType::decompose(&CatalogueType {
            column_type: "date",
            data_type: "date",
            ..CatalogueType::default()
        });

        assert_eq!(datetime.precision(), Some(0));
        assert_eq!(date.precision(), None);
    }

    #[test]
    fn a_type_equal_to_its_data_type_carries_no_part_but_its_own_name() {
        // FR-CTX-038 and FR-CTX-040: the geometry types, inet4, inet6 and uuid
        // return a string equal to `data_type`, and populate no size field.
        let column = ColumnType::decompose(&CatalogueType {
            column_type: "multipolygon",
            data_type: "multipolygon",
            ..CatalogueType::default()
        });

        assert_eq!(column.raw(), "multipolygon");
        assert_eq!(parts(&column), (Some("multipolygon"), None, None, None));
        assert_eq!(column.unsigned(), Some(false));
        assert_eq!(column.values(), None);
    }

    #[test]
    fn the_recognised_set_is_the_thirty_nine_observed_values_and_is_sorted() {
        // FR-CTX-018 recognises what FR-CTX-040 has a rule for, and
        // `recognised` binary-searches, which a list out of order would break.
        assert!(
            RECOGNISED_DATA_TYPES.is_sorted(),
            "the list is binary-searched"
        );
        assert_eq!(RECOGNISED_DATA_TYPES.len(), 39);

        for data_type in ["bigint", "decimal", "datetime", "longtext", "uuid"] {
            assert!(super::recognised(data_type), "{data_type}");
        }

        // Neither of the first two is a value the catalogue reports: a JSON
        // column reads `longtext`, per FR-CAT-038, and a NUMERIC column reads
        // `decimal`. The case of the third is the case the catalogue writes.
        for data_type in ["json", "numeric", "", "BIGINT"] {
            assert!(!super::recognised(data_type), "{data_type}");
        }
    }
}
