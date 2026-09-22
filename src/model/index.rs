//! An index, folded from the one-row-per-column form the catalogue reports it
//! in (`FR-CAT-010`, `FR-CAT-042`).
//!
//! `FR-CAT-010` is the whole shape of this module: an index is **one object
//! with an ordered column list**, not several rows. The catalogue reports one
//! row per column and states the order in a sequence field; presenting those
//! rows would make every template perform the same grouping, and each one would
//! order the columns differently.
//!
//! The fold decides where each fact lives, and the division is not arbitrary:
//! a fact the catalogue repeats on every row of one index belongs to the index,
//! and a fact that differs row by row belongs to the member. The sort direction
//! and the prefix length are the two that differ, so they are fields of
//! [`IndexColumn`] and not of [`Index`].
//!
//! | Catalogue field | Where it lands |
//! |---|---|
//! | index name | [`Index::name`] |
//! | non-unique | [`Index::unique`], true WHEN the field is `0` |
//! | sequence in index | the order of [`Index::columns`] — the column the fold runs on |
//! | column name | [`IndexColumn::name`] |
//! | collation | [`IndexColumn::direction`] |
//! | sub-part | [`IndexColumn::prefix_length`] |
//! | index type | [`Index::index_type`] |
//! | index comment | [`Index::comment`] |
//! | ignored | [`Index::ignored`], true WHEN the field is not `NO` |
//!
//! *The five fields `FR-CAT-042` excludes have no field here, and each is
//! excluded on a stated ground*: the catalogue, schema and index-schema fields
//! and the table name are row identity; `CARDINALITY` is volatile
//! (`FR-CAT-024`); the packed field was never observed populated; the nullable
//! field restates the column's own nullability (`FR-CTX-021`); and the comment
//! field — which is **not** the index comment — was never observed populated
//! either. A reader that takes the comment field instead of the index-comment
//! field beside it reports every index as uncommented.

use std::borrow::Cow;

use serde::{Deserialize, Serialize};

use super::ToStatic;

/// The name the catalogue reports a primary key under (`FR-CAT-043`).
///
/// This is the whole of how a primary key is told from any other unique index.
/// Over the fixture the indexes so named are exactly the seventeen the
/// constraint table reports as primary keys, on all four series.
pub const PRIMARY_KEY_NAME: &str = "PRIMARY";

/// The catalogue's value for an ascending index column.
const ASCENDING: &str = "A";

/// The catalogue's value for a descending one.
const DESCENDING: &str = "D";

/// The direction an index column is sorted in (`FR-CAT-042`).
///
/// The catalogue reports it in the collation field of the index row, which
/// reads `A` on 70 of the fixture's 77 rows, `D` on 6, and SQL `NULL` on the
/// one full-text row. The null case is [`None`] on [`IndexColumn::direction`]
/// rather than a third variant: it is an absent scalar, which `FR-CTX-005`
/// reserves `null` for.
/// *The document carries the catalogue's own two letters.* The spelling is
/// contract surface once the direction reaches the document, and `A` and `D`
/// are what the catalogue writes; a word the catalogue does not write would be
/// vocabulary this system invented, which no requirement asks for.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[non_exhaustive]
pub enum SortDirection {
    /// The column is indexed ascending.
    #[serde(rename = "A")]
    Ascending,

    /// The column is indexed descending.
    #[serde(rename = "D")]
    Descending,
}

impl SortDirection {
    /// Reads the direction from the collation field of an index row.
    ///
    /// [`None`] for any other value, which is the full-text row's SQL `NULL`.
    #[must_use]
    pub fn from_catalogue(field: &str) -> Option<Self> {
        match field {
            ASCENDING => Some(Self::Ascending),
            DESCENDING => Some(Self::Descending),
            _ => None,
        }
    }

    /// The catalogue value this direction was read from.
    #[must_use]
    pub const fn field(self) -> &'static str {
        match self {
            Self::Ascending => ASCENDING,
            Self::Descending => DESCENDING,
        }
    }
}

/// One member of an index's column list (`FR-CAT-042`).
///
/// The two facts beside the name are the ones that differ from row to row of
/// the same index, which is why they are here and not on [`Index`].
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct IndexColumn<'a> {
    /// The column's name, returned unescaped. `FR-CAT-044` is the invariant
    /// that it is a column of the same table, and
    /// [`Table::assemble`](super::table::Table::assemble) is where it is
    /// enforced.
    #[serde(borrow)]
    pub name: Cow<'a, str>,

    /// The direction this column is indexed in, or [`None`] where the
    /// catalogue states none.
    pub direction: Option<SortDirection>,

    /// The prefix length in characters, or [`None`] for a whole-column index.
    ///
    /// *It is what the catalogue states about the index, not what the DDL
    /// said.* The fixture's spatial index declares no prefix and its row
    /// nonetheless reads `32`, so a template that presents this as *the author
    /// asked for a prefix* is wrong for that row.
    pub prefix_length: Option<u64>,
}

/// One index of a table (`FR-CAT-010`, `FR-CAT-042`).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct Index<'a> {
    /// The index's name, returned unescaped — the fixture's hostile names
    /// include one containing a space and one containing a backtick.
    #[serde(borrow)]
    pub name: Cow<'a, str>,

    /// Whether the index is unique, read from the non-unique field: there is
    /// **no** is-unique field in the catalogue.
    ///
    /// A primary key reports `true` here, because its rows carry the
    /// non-unique field `0`. `primary_key` and `unique` are not disjoint and
    /// nothing requires them to be, per `FR-CAT-043`.
    pub unique: bool,

    /// The index's columns, in the order the catalogue's sequence field states
    /// (`FR-CAT-010`). The order is meaning rather than presentation, and it is
    /// one of the six exceptions of `NFR-DET-002`.
    #[serde(borrow)]
    pub columns: Vec<IndexColumn<'a>>,

    /// The index type, carried verbatim — `BTREE`, `FULLTEXT` and `SPATIAL`
    /// were observed, and no `HASH` row appeared.
    ///
    /// The document key is `type`, which is not a name a Rust field can take.
    #[serde(rename = "type", borrow)]
    pub index_type: Cow<'a, str>,

    /// The text written with `COMMENT` on the key, and the **empty string**
    /// where none was given.
    #[serde(borrow)]
    pub comment: Cow<'a, str>,

    /// Whether the index is ignored, true WHEN the catalogue's ignored field
    /// is not `NO`. It read `NO` on all 77 rows of the fixture.
    pub ignored: bool,
}

impl Index<'_> {
    /// Whether this is the index `FR-CAT-043` presents as the primary key.
    ///
    /// The name is the whole test, and the index catalogue table is the whole
    /// source: `FR-CAT-043` bars reading a primary key from the
    /// key-column-usage table, which on a system-versioned table names the
    /// implicit period column the model does not carry, and from the
    /// constraint table, which a reduced-privilege reader loses entirely.
    #[must_use]
    pub fn is_primary_key(&self) -> bool {
        self.name == PRIMARY_KEY_NAME
    }
}

/// A copy that borrows nothing, for the render context of `FR-RND-023`.
impl ToStatic for IndexColumn<'_> {
    type Static = IndexColumn<'static>;

    fn to_static(&self) -> Self::Static {
        IndexColumn {
            name: self.name.to_static(),
            direction: self.direction,
            prefix_length: self.prefix_length,
        }
    }
}

/// A copy that borrows nothing, for the render context of `FR-RND-023`.
impl ToStatic for Index<'_> {
    type Static = Index<'static>;

    fn to_static(&self) -> Self::Static {
        Index {
            name: self.name.to_static(),
            unique: self.unique,
            columns: self.columns.to_static(),
            index_type: self.index_type.to_static(),
            comment: self.comment.to_static(),
            ignored: self.ignored,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{Index, IndexColumn, PRIMARY_KEY_NAME, SortDirection};
    use std::borrow::Cow;

    fn member(name: &str) -> IndexColumn<'_> {
        IndexColumn {
            name: Cow::Borrowed(name),
            direction: Some(SortDirection::Ascending),
            prefix_length: None,
        }
    }

    fn index<'a>(name: &'a str, columns: Vec<IndexColumn<'a>>) -> Index<'a> {
        Index {
            name: Cow::Borrowed(name),
            unique: true,
            columns,
            index_type: Cow::Borrowed("BTREE"),
            comment: Cow::Borrowed(""),
            ignored: false,
        }
    }

    #[test]
    fn fr_cat_010_an_index_is_one_object_carrying_an_ordered_column_list() {
        // FR-CAT-010: the catalogue reports one row per column and the model
        // folds them into one object. The order is the sequence field's.
        let composite = index(
            "uq_consignment_leg",
            vec![member("consignment_id"), member("leg_number")],
        );

        let names: Vec<&str> = composite
            .columns
            .iter()
            .map(|column| column.name.as_ref())
            .collect();

        assert_eq!(names, ["consignment_id", "leg_number"]);
    }

    #[test]
    fn fr_cat_042_the_sort_direction_and_the_prefix_length_belong_to_the_member() {
        // Both differ from row to row of one index, so neither can be a field
        // of the index. FR-CAT-042 reads them from the collation field and the
        // sub-part field of each row.
        let prefixed = IndexColumn {
            name: Cow::Borrowed("description"),
            direction: Some(SortDirection::Descending),
            prefix_length: Some(32),
        };

        assert_eq!(prefixed.direction, Some(SortDirection::Descending));
        assert_eq!(prefixed.prefix_length, Some(32));
        assert_eq!(member("leg_number").prefix_length, None);
    }

    #[test]
    fn fr_cat_042_the_direction_is_read_from_the_two_values_observed_and_the_null_row_has_none() {
        // 70 rows read `A`, 6 read `D`, and the one full-text row is SQL NULL.
        assert_eq!(
            SortDirection::from_catalogue("A"),
            Some(SortDirection::Ascending)
        );
        assert_eq!(
            SortDirection::from_catalogue("D"),
            Some(SortDirection::Descending)
        );
        assert_eq!(SortDirection::from_catalogue(""), None);
        assert_eq!(SortDirection::from_catalogue("ASC"), None);
        assert_eq!(SortDirection::Ascending.field(), "A");
        assert_eq!(SortDirection::Descending.field(), "D");
    }

    #[test]
    fn fr_cat_043_the_primary_key_is_the_index_named_primary_and_nothing_else_names_it() {
        // FR-CAT-043: the name is the whole test.
        assert!(index(PRIMARY_KEY_NAME, vec![member("consignment_id")]).is_primary_key());
        assert!(!index("uq_consignment_reference", vec![member("reference")]).is_primary_key());
        assert!(!index("primary", vec![member("reference")]).is_primary_key());
    }
}
