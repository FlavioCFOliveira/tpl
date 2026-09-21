//! A column of a table or a view (`FR-SCH-009`, `FR-CTX-019` … `FR-CTX-021`).
//!
//! The column is where the model refuses most visibly. `FR-CTX-021` forbids
//! materialising on a column any fact its table already states, and names two:
//! `is_primary_key` and `is_unique`. Neither is a field here, and the refusal
//! is structural — `BR-CTX-003` is the reason, and it is a reason about
//! documents rather than about bytes. A column that said it was not part of the
//! primary key while its table said it was would be a document that contradicts
//! itself, and nothing downstream could repair it once both were written.
//! `FR-CTX-022` is where those two facts are answered instead: the tests
//! `primary_key` and `unique` resolve [`Column::table_name`] against the render
//! context and ask the table.
//!
//! | Requirement | What the column does with it |
//! |---|---|
//! | `FR-SCH-009` — position, type, nullability, default, comment, generated status | One field each, and the type and the default are the decompositions of [`super::column_type`] and [`super::column_default`] |
//! | `FR-CTX-019` — every column names its table | [`Column::table_name`], which is what makes a column reached through an index or a key self-locating |
//! | `FR-CTX-020`, `FR-CAT-041` — the static attributes the catalogue states | Four fields, read from the one attribute field `FR-CAT-041` fixes |
//! | `FR-CAT-027` — the auto-increment attribute stays | [`Column::auto_increment`], which is **not** the table counter `FR-CAT-024` excludes |
//! | `FR-CAT-051` — a generated column carries its expression and storage kind | [`Generated`], one value carrying both, so neither can be present without the other |

use std::borrow::Cow;

use serde::{Deserialize, Serialize};

use super::column_default::ColumnDefault;
use super::column_type::ColumnType;

/// The attribute-field value of a column generated and not stored.
const VIRTUAL_GENERATED: &str = "VIRTUAL GENERATED";

/// The attribute-field value of a column generated and stored.
const STORED_GENERATED: &str = "STORED GENERATED";

/// Whether a generated column's value is stored or recomputed (`FR-CAT-051`).
///
/// `FR-CAT-051` reads the storage kind from the column attribute field and
/// forbids reading it from the is-generated field, which says only *whether* a
/// column is generated and never *how*. The two values above are the two that
/// field was observed to take for a generated column, on all four series.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[non_exhaustive]
pub enum GeneratedStorage {
    /// The value is recomputed on every read.
    #[serde(rename = "VIRTUAL GENERATED")]
    Virtual,

    /// The value is written to the table.
    #[serde(rename = "STORED GENERATED")]
    Stored,
}

impl GeneratedStorage {
    /// Reads the storage kind from the column attribute field of
    /// `FR-CAT-041`.
    ///
    /// [`None`] for every other value the field takes, which is the column
    /// that is not generated at all.
    #[must_use]
    pub fn from_attribute(attribute: &str) -> Option<Self> {
        match attribute {
            VIRTUAL_GENERATED => Some(Self::Virtual),
            STORED_GENERATED => Some(Self::Stored),
            _ => None,
        }
    }

    /// The attribute-field value this kind was read from.
    #[must_use]
    pub const fn attribute(self) -> &'static str {
        match self {
            Self::Virtual => VIRTUAL_GENERATED,
            Self::Stored => STORED_GENERATED,
        }
    }
}

/// What a generated column adds to an ordinary one (`FR-CAT-051`).
///
/// The expression and the storage kind are one value because a column has
/// either both or neither: an expression without a storage kind would not say
/// whether the value is written, and a storage kind without an expression would
/// describe nothing. [`Column::generated`] is therefore an [`Option`] of this
/// rather than two independent optional fields, and the half-populated state
/// has no representation.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct Generated<'a> {
    /// The generation expression, as the catalogue rewrote it: identifiers
    /// backtick-quoted, function names lower-cased, carrying no `AS`, no
    /// enclosing parentheses, and neither the `VIRTUAL` nor the `STORED`
    /// keyword.
    #[serde(borrow)]
    pub expression: Cow<'a, str>,

    /// Whether the value is stored, read from the attribute field and never
    /// from the is-generated field.
    pub storage: GeneratedStorage,
}

/// One column of a table or a view.
///
/// The fields are public and the type is `#[non_exhaustive]`, so the model can
/// be read anywhere and built only inside this crate. There is no invariant for
/// a constructor to enforce: every field is a fact read from one catalogue
/// field, and the two that are decompositions rather than readings —
/// [`Column::column_type`] and [`Column::default`] — enforce their own.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct Column<'a> {
    /// The column's name, returned unescaped and unquoted. Quoting it for a
    /// target dialect is the template's job, per `FR-ENV-045`.
    #[serde(borrow)]
    pub name: Cow<'a, str>,

    /// The table the column belongs to (`FR-CTX-019`).
    #[serde(borrow)]
    pub table_name: Cow<'a, str>,

    /// The ordinal position, one-based, which is also the order `FR-CAT-009`
    /// presents the columns in.
    pub position: u64,

    /// The type: the raw string of `FR-CTX-014` and the eight decomposed parts
    /// of `FR-CTX-015`, carried as **siblings** on the column.
    ///
    /// `FR-CTX-014` gives a column `column_type`; `FR-CTX-015` gives it the
    /// decomposed parts *additionally*. Both read as fields of the column, so
    /// a serialised column reads `col.data_type` and never
    /// `col.column_type.data_type`, and `#[serde(flatten)]` is what makes the
    /// document say so. The decomposition remains a type of its own, because
    /// [`ColumnType::decompose`](super::column_type::ColumnType::decompose)
    /// being its only constructor is what makes `FR-CTX-040` structural — the
    /// nesting was an artefact of that type, not a rule of the document.
    ///
    /// The nine keys are emitted where this field sits, so `OD-18` still holds
    /// and the key order is the field order of this struct with
    /// [`ColumnType`]'s own order spliced in at this position.
    #[serde(flatten)]
    pub column_type: ColumnType<'a>,

    /// Whether the column admits `NULL`, which is what the `nullable` test of
    /// `FR-ENV-041` answers from.
    pub nullable: bool,

    /// The default, classified per `FR-CTX-037`. [`None`] is the one case with
    /// no default at all: a `NOT NULL` column that declares none.
    pub default: Option<ColumnDefault<'a>>,

    /// The comment, which is the **empty string** where none was given and is
    /// never absent, per `FR-CAT-039`. It carries no promise of being the text
    /// the author wrote, per `FR-CAT-036`.
    #[serde(borrow)]
    pub comment: Cow<'a, str>,

    /// Whether the column is auto-incremental (`FR-CAT-027`, `FR-CTX-020`).
    ///
    /// This is the **column attribute**. The table-level counter of the same
    /// name is excluded outright by `FR-CAT-024` and has no field anywhere.
    pub auto_increment: bool,

    /// Whether the column is invisible (`FR-CAT-035`, `FR-CTX-020`). An
    /// invisible column is carried like any other, per `FR-CAT-009`.
    pub invisible: bool,

    /// The generation expression and storage kind, or [`None`] for a column
    /// that is not generated (`FR-CAT-051`).
    pub generated: Option<Generated<'a>>,

    /// The `ON UPDATE` default the attribute field states, where it states one
    /// (`FR-CAT-041`, `FR-CTX-020`).
    ///
    /// It is carried verbatim — `on update current_timestamp()` and
    /// `on update current_timestamp(3)` were both observed — because it is
    /// reported nowhere else: the column-default field of `FR-CTX-037` carries
    /// the `DEFAULT` clause and says nothing about `ON UPDATE`.
    #[serde(borrow)]
    pub on_update: Option<Cow<'a, str>>,
}

#[cfg(test)]
mod tests {
    use super::{Column, Generated, GeneratedStorage};
    use crate::model::column_type::{CatalogueType, ColumnType};
    use std::borrow::Cow;

    /// A column of the fixture's `consignment`, built with the defaults a test
    /// that is not about them does not want to restate.
    fn column<'a>(name: &'a str) -> Column<'a> {
        Column {
            name: Cow::Borrowed(name),
            table_name: Cow::Borrowed("consignment"),
            position: 1,
            column_type: ColumnType::decompose(&CatalogueType {
                column_type: "bigint(20) unsigned",
                data_type: "bigint",
                numeric_precision: Some(20),
                numeric_scale: Some(0),
                ..CatalogueType::default()
            }),
            nullable: false,
            default: None,
            comment: Cow::Borrowed(""),
            auto_increment: true,
            invisible: false,
            generated: None,
            on_update: None,
        }
    }

    #[test]
    fn fr_ctx_019_a_column_names_the_table_it_belongs_to() {
        // FR-CTX-019: a column reached through an index or a key must find its
        // own table without the template having carried a reference to it.
        assert_eq!(column("consignment_id").table_name, "consignment");
    }

    #[test]
    fn fr_cat_051_the_storage_kind_is_read_from_the_attribute_field_and_from_no_other() {
        // FR-CAT-051: the is-generated field reads `ALWAYS` for both kinds and
        // cannot be the source. These are the two values the attribute field
        // was observed to take over the fixture's nine generated columns.
        assert_eq!(
            GeneratedStorage::from_attribute("VIRTUAL GENERATED"),
            Some(GeneratedStorage::Virtual)
        );
        assert_eq!(
            GeneratedStorage::from_attribute("STORED GENERATED"),
            Some(GeneratedStorage::Stored)
        );
        assert_eq!(GeneratedStorage::from_attribute("ALWAYS"), None);
        assert_eq!(GeneratedStorage::from_attribute("auto_increment"), None);
        assert_eq!(GeneratedStorage::from_attribute(""), None);
    }

    #[test]
    fn fr_cat_051_the_attribute_spelling_round_trips() {
        // The spelling is contract surface once the attribute it reports
        // reaches the document, and the case of each value is the catalogue's.
        for storage in [GeneratedStorage::Virtual, GeneratedStorage::Stored] {
            assert_eq!(
                GeneratedStorage::from_attribute(storage.attribute()),
                Some(storage)
            );
        }
    }

    #[test]
    fn fr_cat_051_a_generated_column_carries_the_expression_and_the_kind_together() {
        // FR-CAT-051: one value, so neither half can be present alone. The
        // expression is the fixture's, as the catalogue rewrote it.
        let generated = Column {
            generated: Some(Generated {
                expression: Cow::Borrowed(
                    "round(`declared_value` * coalesce(`insurance_rate`,0),2)",
                ),
                storage: GeneratedStorage::Stored,
            }),
            ..column("insured_value")
        };

        let carried = generated
            .generated
            .as_ref()
            .expect("the column was built generated");

        assert_eq!(carried.storage, GeneratedStorage::Stored);
        assert!(carried.expression.contains("coalesce"));
    }

    #[test]
    fn fr_cat_027_the_auto_increment_a_column_carries_is_the_attribute_and_not_the_counter() {
        // FR-CAT-027 keeps the column attribute; FR-CAT-024 excludes the
        // table-level counter of the same name outright. The two are one word
        // and two different things, and only one of them is a field.
        assert!(column("consignment_id").auto_increment);
    }
}
