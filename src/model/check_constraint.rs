//! A `CHECK` constraint of a table (`FR-CAT-015`, `FR-CAT-046`).
//!
//! Three fields, from a catalogue field list of six: the other three are the
//! catalogue and schema names, which are row identity, and the table name,
//! under which the constraint is already carried.
//!
//! *Two consequences of `FR-CAT-015` are carried here rather than filtered
//! away, because filtering either would be a change to that requirement.* A
//! constraint declared at column level is named **after the column it
//! qualifies**, so a constraint name is not guaranteed distinct from a column
//! name of the same table, and a template that keys a map on the one can
//! collide with the other. And MariaDB attaches an implicit `json_valid`
//! constraint to every column declared `JSON` and reports it exactly as it
//! reports an authored column-level check, so the model carries constraints
//! nobody wrote — distinguishable from an authored one only by reading the
//! clause.

use std::borrow::Cow;

use serde::{Deserialize, Serialize};

/// The catalogue's value for a constraint declared on the table.
const TABLE_LEVEL: &str = "Table";

/// The catalogue's value for one declared on a column.
const COLUMN_LEVEL: &str = "Column";

/// The level a `CHECK` constraint is declared at (`FR-CAT-046`).
///
/// `FR-CAT-046` fixes the level as admitting **exactly two values** and records
/// that no third was observed over the fixture's 24 constraints on any of the
/// four series. Note the case: the catalogue writes them mixed, and the
/// spelling is contract surface once the level reaches the document.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[non_exhaustive]
pub enum ConstraintLevel {
    /// Declared on the table, with a name the DDL gave. 22 of the fixture's 24.
    #[serde(rename = "Table")]
    Table,

    /// Declared on a column, and therefore named after that column. 2 of 24.
    #[serde(rename = "Column")]
    Column,
}

impl ConstraintLevel {
    /// Reads the level from the catalogue's level field.
    ///
    /// [`None`] for any other value, which `FR-CAT-046` observed none of.
    #[must_use]
    pub fn from_catalogue(field: &str) -> Option<Self> {
        match field {
            TABLE_LEVEL => Some(Self::Table),
            COLUMN_LEVEL => Some(Self::Column),
            _ => None,
        }
    }

    /// The spelling the catalogue writes, in the catalogue's own case.
    #[must_use]
    pub const fn name(self) -> &'static str {
        match self {
            Self::Table => TABLE_LEVEL,
            Self::Column => COLUMN_LEVEL,
        }
    }
}

/// One `CHECK` constraint (`FR-CAT-046`).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct CheckConstraint<'a> {
    /// The name the DDL gave, or the column name for a column-level
    /// constraint, per `FR-CAT-037`.
    #[serde(borrow)]
    pub name: Cow<'a, str>,

    /// Whether the constraint is declared on the table or on a column.
    pub level: ConstraintLevel,

    /// The clause, as the catalogue rewrote it: identifiers backtick-quoted
    /// and operators lower-cased.
    #[serde(borrow)]
    pub clause: Cow<'a, str>,
}

#[cfg(test)]
mod tests {
    use super::{CheckConstraint, ConstraintLevel};
    use std::borrow::Cow;

    #[test]
    fn the_level_admits_exactly_the_two_values_the_catalogue_writes() {
        // FR-CAT-046: `Table` on 22 of 24 and `Column` on 2, mixed case, and
        // no third value on any of the four series.
        assert_eq!(
            ConstraintLevel::from_catalogue("Table"),
            Some(ConstraintLevel::Table)
        );
        assert_eq!(
            ConstraintLevel::from_catalogue("Column"),
            Some(ConstraintLevel::Column)
        );
        assert_eq!(ConstraintLevel::from_catalogue("TABLE"), None);
        assert_eq!(ConstraintLevel::from_catalogue("column"), None);
        assert_eq!(ConstraintLevel::from_catalogue(""), None);
        assert_eq!(ConstraintLevel::Table.name(), "Table");
        assert_eq!(ConstraintLevel::Column.name(), "Column");
    }

    #[test]
    fn a_column_level_constraint_is_carried_under_the_column_name() {
        // FR-CAT-037: the catalogue names it after the column it qualifies, so
        // a constraint name is not guaranteed distinct from a column name of
        // the same table. The model carries what the catalogue gave.
        let implicit = CheckConstraint {
            name: Cow::Borrowed("manifest_payload"),
            level: ConstraintLevel::Column,
            clause: Cow::Borrowed("json_valid(`manifest_payload`)"),
        };

        assert_eq!(implicit.name, "manifest_payload");
        assert_eq!(implicit.level, ConstraintLevel::Column);
    }

    #[test]
    fn the_clause_is_carried_as_the_catalogue_rewrote_it() {
        // FR-CAT-046: identifiers backtick-quoted and operators lower-cased.
        // The DDL wrote `un_number IS NULL OR (un_number >= 1 AND ...)`.
        let table_level = CheckConstraint {
            name: Cow::Borrowed("ck_cargo_item_un_number"),
            level: ConstraintLevel::Table,
            clause: Cow::Borrowed(
                "`un_number` is null or `un_number` >= 1 and `un_number` <= 3550",
            ),
        };

        assert!(table_level.clause.contains("is null"));
        assert!(table_level.clause.contains('`'));
    }
}
