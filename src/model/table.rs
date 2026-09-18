//! A table, and the invariant every key it carries obeys (`FR-CAT-001`,
//! `FR-CAT-002`, `FR-CAT-009` … `FR-CAT-015`, `FR-CAT-043`, `FR-CAT-044`).
//!
//! | Requirement | What the table does with it |
//! |---|---|
//! | `FR-CAT-001`, `FR-CAT-031` — two covered table types | [`TableType`] has two variants, and [`TableType::from_catalogue`] is the coverage predicate: it answers [`None`] for `VIEW`, `SEQUENCE`, `SYSTEM VIEW` and `TEMPORARY` |
//! | `FR-CAT-002` — every table states which of the two it is | [`Table::table_type`] |
//! | `FR-CAT-009` — columns, generated and invisible ones included | [`Table::columns`] |
//! | `FR-CAT-010` — an index is one object with an ordered column list | [`Table::indexes`], folded in [`super::index`] |
//! | `FR-CAT-011`, `FR-CAT-043` — the primary key, from the index catalogue and no other source | [`Table::primary_key`], which **is** the index named `PRIMARY` in that same collection |
//! | `FR-CAT-012`, `FR-CAT-013` — both directions of a foreign key | [`Table::foreign_keys`] and [`Table::referenced_by`] |
//! | `FR-CAT-014`, `FR-CAT-015` — triggers and `CHECK` constraints | [`Table::triggers`] and [`Table::check_constraints`] |
//! | `FR-CAT-044` — no key names a column the table does not carry | [`Table::assemble`] is the only constructor and it refuses one |
//! | `FR-SCH-009` — engine, collation and comment | Three fields, and **no character set**: the table catalogue has no such field on any of the four series |
//!
//! *`FR-CAT-032` is why the coverage predicate is a predicate.* After a
//! `CREATE TEMPORARY TABLE`, `10.11` returns no row for it at all while the
//! other three return one of table type `TEMPORARY`. A reader developed against
//! `10.11` alone would pass every test and then present temporary tables —
//! without columns or indexes, because those tables reach neither of the other
//! catalogue tables — against `11.4` and later. Excluding by filtering on the
//! type is cheap; relying on the catalogue to omit them is wrong on three
//! series out of four.
//!
//! *The primary key is not a second copy.* `FR-CAT-043` presents the index
//! named `PRIMARY` both in the index collection and as the table's primary key,
//! so that a template need not find it by matching a name. Storing it twice
//! would make the two able to disagree; [`Table::primary_key`] reads it out of
//! the collection instead, which is also the strongest form of *from the index
//! catalogue table and from no other source* — there is no other field for
//! another source to be written into.

use std::borrow::Cow;

use serde::{Deserialize, Serialize};
use thiserror::Error;

use super::check_constraint::CheckConstraint;
use super::column::Column;
use super::foreign_key::{ForeignKey, IncomingForeignKey};
use super::index::Index;
use super::restricted::Restricted;
use super::trigger::Trigger;

/// The catalogue's table type for an ordinary table.
const BASE_TABLE: &str = "BASE TABLE";

/// The catalogue's table type for a system-versioned one.
const SYSTEM_VERSIONED: &str = "SYSTEM VERSIONED";

/// Which of the two covered table types a table is (`FR-CAT-002`).
///
/// `FR-CAT-031` closes the set of table types a supported series can emit at
/// six, and `FR-CAT-001` covers two of them. The other four have no variant
/// here, which is what makes the exclusions of `FR-CAT-003` through
/// `FR-CAT-006` structural: a view, a sequence, a system view or a temporary
/// table cannot be carried as a table, because there is no value that would say
/// it was one.
///
/// The two are covered together because both are ordinary tables to a
/// generator, and both carry columns, keys and indexes. A template that must
/// treat a system-versioned table differently needs the distinction stated
/// rather than inferred from a column name.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[non_exhaustive]
pub enum TableType {
    /// The catalogue's `BASE TABLE`.
    #[serde(rename = "BASE TABLE")]
    Base,

    /// The catalogue's `SYSTEM VERSIONED`.
    #[serde(rename = "SYSTEM VERSIONED")]
    SystemVersioned,
}

impl TableType {
    /// The coverage predicate of `FR-CAT-001` through `FR-CAT-006`.
    ///
    /// [`Some`] for the two types the model covers and [`None`] for every other
    /// value of the closed set of `FR-CAT-031`, which is the filter
    /// `FR-CAT-032` requires rather than a reliance on the catalogue omitting
    /// anything.
    #[must_use]
    pub fn from_catalogue(field: &str) -> Option<Self> {
        match field {
            BASE_TABLE => Some(Self::Base),
            SYSTEM_VERSIONED => Some(Self::SystemVersioned),
            _ => None,
        }
    }

    /// The spelling the catalogue writes, which is contract surface once
    /// `table_type` reaches the document under `FR-CAT-002`.
    #[must_use]
    pub const fn name(self) -> &'static str {
        match self {
            Self::Base => BASE_TABLE,
            Self::SystemVersioned => SYSTEM_VERSIONED,
        }
    }
}

/// A key naming a column its table does not carry (`FR-CAT-044`).
///
/// The error is neutral about what should happen next, and deliberately so. The
/// same violation means two different things at the two places a model is
/// built: from a live read it is an internal invariant, which
/// [`Error::InternalInvariant`](crate::Error::InternalInvariant) reports at
/// `70`; from a `--context` document it is caller data, which
/// [`Error::ContextDocumentMalformed`](crate::Error::ContextDocumentMalformed)
/// reports at `65`. A variant of its own would have to choose one of the two,
/// so this type chooses neither and each caller maps it.
#[derive(Debug, Clone, PartialEq, Eq, Error)]
#[error(
    "the key '{key}' of table '{table}' names the column '{column}', which the table does not carry"
)]
#[non_exhaustive]
pub struct UnknownKeyColumn {
    /// The table the key was carried on.
    pub table: String,

    /// The key that named the column.
    pub key: String,

    /// The column name that is absent from the table's column list.
    pub column: String,
}

/// Everything a table is assembled from (`FR-CAT-009` … `FR-CAT-015`).
///
/// *This type is not `#[non_exhaustive]`, for the reason
/// [`CatalogueType`](super::column_type::CatalogueType) is not: it is an
/// **input**, and a caller has to be able to write it down.* The value it
/// produces, [`Table`], carries the attribute for both.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TableParts<'a> {
    /// The table's name.
    pub name: Cow<'a, str>,

    /// Which of the two covered types it is.
    pub table_type: TableType,

    /// The storage engine, per `FR-SCH-009`.
    pub engine: Option<Cow<'a, str>>,

    /// The table's collation, per `FR-SCH-009`.
    pub collation: Option<Cow<'a, str>>,

    /// The table comment, which is the empty string where none was given.
    pub comment: Cow<'a, str>,

    /// The columns, in ordinal position order.
    pub columns: Vec<Column<'a>>,

    /// The indexes, including the one named `PRIMARY`.
    pub indexes: Vec<Index<'a>>,

    /// The outgoing foreign keys.
    pub foreign_keys: Vec<ForeignKey<'a>>,

    /// The incoming ones.
    pub referenced_by: Vec<IncomingForeignKey<'a>>,

    /// The triggers.
    pub triggers: Vec<Trigger<'a>>,

    /// The `CHECK` constraints.
    pub check_constraints: Vec<CheckConstraint<'a>>,

    /// The properties that could not be read, or [`None`] where the table is
    /// complete.
    pub restricted: Option<Restricted<'a>>,
}

impl<'a> TableParts<'a> {
    /// The parts of a table that carries nothing yet.
    ///
    /// Every collection is empty and every optional field absent, so a caller
    /// — and a test — states only what it is about, through a struct update.
    #[must_use]
    pub fn new(name: Cow<'a, str>, table_type: TableType) -> Self {
        Self {
            name,
            table_type,
            engine: None,
            collation: None,
            comment: Cow::Borrowed(""),
            columns: Vec::new(),
            indexes: Vec::new(),
            foreign_keys: Vec::new(),
            referenced_by: Vec::new(),
            triggers: Vec::new(),
            check_constraints: Vec::new(),
            restricted: None,
        }
    }
}

/// One table (`FR-CAT-001`).
///
/// The fields are private and [`Table::assemble`] is the only constructor,
/// because `FR-CAT-044` is an invariant over the whole value rather than a
/// property of any one field: it relates every key to the column list beside
/// it, so it can only be established where both are in hand.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub struct Table<'a> {
    /// The table's name, returned unescaped.
    name: Cow<'a, str>,

    /// Which of the two covered types it is (`FR-CAT-002`).
    table_type: TableType,

    /// The storage engine (`FR-SCH-009`).
    engine: Option<Cow<'a, str>>,

    /// The table's collation (`FR-SCH-009`). There is no character set beside
    /// it: the table catalogue carries none, on any of the four series.
    collation: Option<Cow<'a, str>>,

    /// The comment (`FR-CAT-039`), empty where none was given.
    comment: Cow<'a, str>,

    /// The columns, in ordinal position order (`FR-CAT-009`).
    columns: Vec<Column<'a>>,

    /// The indexes (`FR-CAT-010`), including the primary key.
    indexes: Vec<Index<'a>>,

    /// The outgoing foreign keys (`FR-CAT-012`).
    foreign_keys: Vec<ForeignKey<'a>>,

    /// The incoming foreign keys (`FR-CAT-013`).
    referenced_by: Vec<IncomingForeignKey<'a>>,

    /// The triggers (`FR-CAT-014`).
    triggers: Vec<Trigger<'a>>,

    /// The `CHECK` constraints (`FR-CAT-015`).
    check_constraints: Vec<CheckConstraint<'a>>,

    /// The properties that could not be read (`FR-PRIV-005`).
    restricted: Option<Restricted<'a>>,
}

impl<'a> Table<'a> {
    /// Assembles a table, enforcing `FR-CAT-044`.
    ///
    /// # Errors
    ///
    /// Returns [`UnknownKeyColumn`] for the first key that names a column
    /// absent from the table's own column list. The requirement states the
    /// invariant over the primary key, a unique key, an index and **both
    /// directions of a foreign key**, and each direction is checked on the half
    /// that points at this table: the referencing columns of an outgoing key,
    /// and the referenced columns of an incoming one. The other half of each
    /// belongs to the table at the far end, whose column list is not this
    /// table's to check — and checking it here would make the invariant
    /// unsatisfiable for every key that crosses a table boundary.
    ///
    /// The primary key needs no check of its own: `FR-CAT-043` makes it an
    /// element of the index collection, so the check over the indexes is the
    /// check over it.
    ///
    /// *Why the invariant is enforced rather than assumed.* The catalogue does
    /// not supply it. On a system-versioned table the key-column-usage table
    /// names `row_end`, the implicit period column system versioning adds, and
    /// the column catalogue carries **no row for it on any table**;
    /// `FR-CAT-043` chose the index catalogue as the primary key's source for
    /// exactly this reason, and this check is what keeps a future source change
    /// from reintroducing the state through another door.
    ///
    /// The check is a linear scan of the column list per key column, which
    /// allocates nothing. A table is scanned once, when it is assembled, and
    /// the product of a realistic column count and key-column count is small
    /// enough that a set would cost more in allocation than it saved in
    /// comparisons.
    pub fn assemble(parts: TableParts<'a>) -> Result<Self, UnknownKeyColumn> {
        for index in &parts.indexes {
            for member in &index.columns {
                Self::require_column(&parts, &index.name, &member.name)?;
            }
        }

        for key in &parts.foreign_keys {
            for member in &key.columns {
                Self::require_column(&parts, &key.name, &member.column)?;
            }
        }

        for incoming in &parts.referenced_by {
            for member in &incoming.key.columns {
                Self::require_column(&parts, &incoming.key.name, &member.referenced_column)?;
            }
        }

        let TableParts {
            name,
            table_type,
            engine,
            collation,
            comment,
            columns,
            indexes,
            foreign_keys,
            referenced_by,
            triggers,
            check_constraints,
            restricted,
        } = parts;

        Ok(Self {
            name,
            table_type,
            engine,
            collation,
            comment,
            columns,
            indexes,
            foreign_keys,
            referenced_by,
            triggers,
            check_constraints,
            restricted,
        })
    }

    /// Fails unless the column list carries `column`.
    fn require_column(
        parts: &TableParts<'a>,
        key: &str,
        column: &str,
    ) -> Result<(), UnknownKeyColumn> {
        if parts.columns.iter().any(|carried| carried.name == *column) {
            return Ok(());
        }

        Err(UnknownKeyColumn {
            table: parts.name.as_ref().to_owned(),
            key: key.to_owned(),
            column: column.to_owned(),
        })
    }

    /// The table's name.
    #[must_use]
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Which of the two covered types the table is (`FR-CAT-002`).
    #[must_use]
    pub const fn table_type(&self) -> TableType {
        self.table_type
    }

    /// The storage engine (`FR-SCH-009`).
    #[must_use]
    pub fn engine(&self) -> Option<&str> {
        self.engine.as_deref()
    }

    /// The table's collation (`FR-SCH-009`), passed through verbatim per
    /// `FR-SRV-039`.
    ///
    /// There is no character set beside it. `FR-SCH-009` named one until the
    /// seventh edition and the catalogue has no such field: the three places a
    /// character set is reachable are the database and each individual column,
    /// not the table between them, and deriving one from the collation's
    /// leading segment would be an inference the catalogue does not state.
    #[must_use]
    pub fn collation(&self) -> Option<&str> {
        self.collation.as_deref()
    }

    /// The table comment, which is the **empty string** where none was given
    /// and is never absent (`FR-CAT-039`).
    ///
    /// It carries no promise of being the text the author wrote: a
    /// supplementary-plane character is replaced by `?` in the server before
    /// any read, per `FR-CAT-036`.
    #[must_use]
    pub fn comment(&self) -> &str {
        &self.comment
    }

    /// The columns, in ordinal position order (`FR-CAT-009`).
    #[must_use]
    pub fn columns(&self) -> &[Column<'a>] {
        &self.columns
    }

    /// The indexes (`FR-CAT-010`), the primary key among them.
    #[must_use]
    pub fn indexes(&self) -> &[Index<'a>] {
        &self.indexes
    }

    /// The primary key — the index named `PRIMARY` (`FR-CAT-011`,
    /// `FR-CAT-043`).
    ///
    /// [`None`] where the table has none. The answer is an element of
    /// [`Table::indexes`] and not a copy of one, so the two cannot disagree.
    #[must_use]
    pub fn primary_key(&self) -> Option<&Index<'a>> {
        self.indexes.iter().find(|index| index.is_primary_key())
    }

    /// The outgoing foreign keys (`FR-CAT-012`).
    #[must_use]
    pub fn foreign_keys(&self) -> &[ForeignKey<'a>] {
        &self.foreign_keys
    }

    /// The incoming foreign keys (`FR-CAT-013`).
    #[must_use]
    pub fn referenced_by(&self) -> &[IncomingForeignKey<'a>] {
        &self.referenced_by
    }

    /// The triggers (`FR-CAT-014`).
    ///
    /// An empty collection does not mean the table has none: `FR-PRIV-020` is
    /// the stated limit that a reader without the privilege receives zero rows,
    /// which is byte-for-byte what a table with no triggers returns.
    #[must_use]
    pub fn triggers(&self) -> &[Trigger<'a>] {
        &self.triggers
    }

    /// The `CHECK` constraints (`FR-CAT-015`).
    #[must_use]
    pub fn check_constraints(&self) -> &[CheckConstraint<'a>] {
        &self.check_constraints
    }

    /// The properties of this table that could not be read, or [`None`] where
    /// it is complete (`FR-PRIV-005` … `FR-PRIV-007`).
    #[must_use]
    pub const fn restricted(&self) -> Option<&Restricted<'a>> {
        self.restricted.as_ref()
    }
}

#[cfg(test)]
mod tests {
    use super::{Table, TableParts, TableType};
    use crate::model::column::Column;
    use crate::model::column_type::{CatalogueType, ColumnType};
    use crate::model::foreign_key::{
        ForeignKey, ForeignKeyColumn, IncomingForeignKey, ReferentialAction,
    };
    use crate::model::index::{Index, IndexColumn, PRIMARY_KEY_NAME, SortDirection};
    use crate::model::restricted::Restricted;
    use std::borrow::Cow;

    /// The fixture's system-versioned table, which is the one `FR-CAT-043`
    /// turned on: its three primary-key sources disagree.
    const TARIFF: &str = "tariff";

    fn column(name: &str) -> Column<'_> {
        Column {
            name: Cow::Borrowed(name),
            table_name: Cow::Borrowed(TARIFF),
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
            auto_increment: false,
            invisible: false,
            generated: None,
            on_update: None,
        }
    }

    fn index<'a>(name: &'a str, columns: &[&'a str]) -> Index<'a> {
        Index {
            name: Cow::Borrowed(name),
            unique: true,
            columns: columns
                .iter()
                .map(|column| IndexColumn {
                    name: Cow::Borrowed(column),
                    direction: Some(SortDirection::Ascending),
                    prefix_length: None,
                })
                .collect(),
            index_type: Cow::Borrowed("BTREE"),
            comment: Cow::Borrowed(""),
            ignored: false,
        }
    }

    fn foreign_key<'a>(name: &'a str, column: &'a str, referenced: &'a str) -> ForeignKey<'a> {
        ForeignKey {
            name: Cow::Borrowed(name),
            columns: vec![ForeignKeyColumn {
                column: Cow::Borrowed(column),
                referenced_column: Cow::Borrowed(referenced),
            }],
            referenced_table: Cow::Borrowed("consignment"),
            referenced_key: Cow::Borrowed(PRIMARY_KEY_NAME),
            match_option: Cow::Borrowed("NONE"),
            on_update: ReferentialAction::Restrict,
            on_delete: ReferentialAction::Cascade,
        }
    }

    /// `tariff`, carrying the one column the index catalogue names in its
    /// primary key.
    fn parts() -> TableParts<'static> {
        TableParts {
            columns: vec![column("tariff_id")],
            indexes: vec![index(PRIMARY_KEY_NAME, &["tariff_id"])],
            ..TableParts::new(Cow::Borrowed(TARIFF), TableType::SystemVersioned)
        }
    }

    #[test]
    fn fr_cat_001_the_coverage_predicate_admits_two_table_types_and_refuses_the_other_four() {
        // FR-CAT-001 covers two of the six types FR-CAT-031 closes the set at.
        // FR-CAT-032 requires the exclusion of a temporary table to be this
        // filter and not a reliance on the catalogue omitting it — which only
        // `10.11` does.
        assert_eq!(
            TableType::from_catalogue("BASE TABLE"),
            Some(TableType::Base)
        );
        assert_eq!(
            TableType::from_catalogue("SYSTEM VERSIONED"),
            Some(TableType::SystemVersioned)
        );
        assert_eq!(TableType::from_catalogue("VIEW"), None);
        assert_eq!(TableType::from_catalogue("SEQUENCE"), None);
        assert_eq!(TableType::from_catalogue("SYSTEM VIEW"), None);
        assert_eq!(TableType::from_catalogue("TEMPORARY"), None);
        assert_eq!(TableType::Base.name(), "BASE TABLE");
        assert_eq!(TableType::SystemVersioned.name(), "SYSTEM VERSIONED");
    }

    #[test]
    fn fr_cat_044_a_table_whose_keys_name_its_own_columns_assembles() {
        let table = Table::assemble(parts()).expect("every key names a column the table carries");

        assert_eq!(table.name(), TARIFF);
        assert_eq!(table.table_type(), TableType::SystemVersioned);
        assert_eq!(table.columns().len(), 1);
        assert_eq!(table.indexes().len(), 1);
    }

    #[test]
    fn fr_cat_043_the_primary_key_is_the_index_named_primary_and_is_not_a_second_copy_of_it() {
        // FR-CAT-043: the index catalogue table is the authoritative source,
        // and the primary key is the index it reports under the name PRIMARY.
        // The answer is an element of the index collection, so the two cannot
        // disagree.
        let table = Table::assemble(parts()).expect("the fixture assembles");
        let primary = table.primary_key().expect("tariff has a primary key");

        assert_eq!(primary.name, PRIMARY_KEY_NAME);
        assert!(std::ptr::eq(primary, &table.indexes()[0]));
    }

    #[test]
    fn fr_cat_043_a_table_with_no_index_named_primary_has_no_primary_key() {
        let table = Table::assemble(TableParts {
            indexes: vec![index("uq_tariff_code", &["tariff_id"])],
            ..parts()
        })
        .expect("the key names a column the table carries");

        assert_eq!(table.primary_key(), None);
        assert_eq!(table.indexes().len(), 1);
    }

    #[test]
    fn fr_cat_044_an_index_naming_a_column_the_table_does_not_carry_is_refused() {
        // FR-CAT-044, and it is the case FR-CAT-043 was decided by: key column
        // usage reports the primary key of `tariff` as (tariff_id, row_end),
        // and the column catalogue carries no row for `row_end` on any table.
        // A model built from that source would carry an internally
        // inconsistent table; this is what stops it.
        let refused = Table::assemble(TableParts {
            indexes: vec![index(PRIMARY_KEY_NAME, &["tariff_id", "row_end"])],
            ..parts()
        })
        .expect_err("row_end is absent from the column list");

        assert_eq!(refused.table, TARIFF);
        assert_eq!(refused.key, PRIMARY_KEY_NAME);
        assert_eq!(refused.column, "row_end");
    }

    #[test]
    fn fr_cat_044_an_outgoing_foreign_key_naming_an_absent_referencing_column_is_refused() {
        // FR-CAT-044 applies to both directions of a foreign key. The
        // referencing columns of an outgoing key are this table's.
        let refused = Table::assemble(TableParts {
            foreign_keys: vec![foreign_key(
                "fk_tariff_consignment",
                "absent",
                "consignment_id",
            )],
            ..parts()
        })
        .expect_err("the referencing column is absent from the column list");

        assert_eq!(refused.key, "fk_tariff_consignment");
        assert_eq!(refused.column, "absent");
    }

    #[test]
    fn fr_cat_044_an_outgoing_foreign_keys_referenced_column_belongs_to_the_other_table_and_is_not_checked()
     {
        // The far half of a key belongs to the table at the far end. Checking
        // it here would make the invariant unsatisfiable for every key that
        // crosses a table boundary, which is every foreign key that is not
        // self-referential.
        let table = Table::assemble(TableParts {
            foreign_keys: vec![foreign_key(
                "fk_tariff_consignment",
                "tariff_id",
                "consignment_id",
            )],
            ..parts()
        })
        .expect("the referencing column is carried; the referenced one is not this table's");

        assert_eq!(table.foreign_keys().len(), 1);
    }

    #[test]
    fn fr_cat_044_an_incoming_foreign_key_naming_an_absent_referenced_column_is_refused() {
        // The incoming direction is the mirror: the referenced columns point
        // at this table, so they are the half this table's column list must
        // carry.
        let refused = Table::assemble(TableParts {
            referenced_by: vec![IncomingForeignKey {
                table: Cow::Borrowed("tariff_band"),
                key: foreign_key("fk_band_tariff", "tariff_id", "absent"),
            }],
            ..parts()
        })
        .expect_err("the referenced column is absent from the column list");

        assert_eq!(refused.key, "fk_band_tariff");
        assert_eq!(refused.column, "absent");
    }

    #[test]
    fn fr_cat_044_an_incoming_foreign_keys_referencing_column_belongs_to_the_referencing_table() {
        let table = Table::assemble(TableParts {
            referenced_by: vec![IncomingForeignKey {
                table: Cow::Borrowed("tariff_band"),
                key: foreign_key("fk_band_tariff", "band_tariff_id", "tariff_id"),
            }],
            ..parts()
        })
        .expect("the referenced column is carried; the referencing one is not this table's");

        assert_eq!(table.referenced_by()[0].table, "tariff_band");
    }

    #[test]
    fn fr_priv_007_a_complete_table_carries_no_marking_and_an_incomplete_one_names_a_property() {
        // FR-PRIV-007 and FR-PRIV-016.
        let complete = Table::assemble(parts()).expect("the fixture assembles");
        assert_eq!(complete.restricted(), None);

        let incomplete = Table::assemble(TableParts {
            restricted: Restricted::new(vec![Cow::Borrowed("columns")]),
            ..parts()
        })
        .expect("the fixture assembles");

        assert_eq!(
            incomplete
                .restricted()
                .expect("the table was built incomplete")
                .properties(),
            [Cow::Borrowed("columns")]
        );
    }

    #[test]
    fn fr_sch_009_a_table_carries_a_collation_and_no_character_set() {
        // FR-SCH-009 as amended: the table catalogue carries a collation and
        // no character set, on all four series. The three places a character
        // set is reachable are the database and each individual column.
        let table = Table::assemble(TableParts {
            engine: Some(Cow::Borrowed("InnoDB")),
            collation: Some(Cow::Borrowed("utf8mb4_unicode_520_ci")),
            ..parts()
        })
        .expect("the fixture assembles");

        assert_eq!(table.engine(), Some("InnoDB"));
        assert_eq!(table.collation(), Some("utf8mb4_unicode_520_ci"));
    }
}
