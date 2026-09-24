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

use super::ToStatic;

catalogued! {
    /// The level a `CHECK` constraint is declared at (`FR-CAT-046`).
    ///
    /// `FR-CAT-046` fixes the level as admitting **exactly two values** and
    /// records that no third was observed over the fixture's 24 constraints on
    /// any of the four series. Note the case: the catalogue writes them mixed,
    /// and the spelling is contract surface once the level reaches the
    /// document.
    ///
    /// *The set is closed by that observation and not by the catalogue.* The
    /// field is declared `varchar(6)` and `NOT NULL` on all four series, never
    /// an `ENUM`, so a third value is carried verbatim under `FR-CAT-055`.
    ConstraintLevel from "LEVEL" {
        /// Declared on the table, with a name the DDL gave. 22 of the
        /// fixture's 24.
        Table = "Table",
        /// Declared on a column, and therefore named after that column. 2 of
        /// 24.
        Column = "Column",
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
    pub level: ConstraintLevel<'a>,

    /// The clause, as the catalogue rewrote it: identifiers backtick-quoted
    /// and operators lower-cased.
    #[serde(borrow)]
    pub clause: Cow<'a, str>,
}

/// A copy that borrows nothing, for the render context of `FR-RND-023`.
impl ToStatic for CheckConstraint<'_> {
    type Static = CheckConstraint<'static>;

    fn to_static(&self) -> Self::Static {
        CheckConstraint {
            name: self.name.to_static(),
            level: self.level.to_static(),
            clause: self.clause.to_static(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{CheckConstraint, ConstraintLevel};
    use std::borrow::Cow;

    #[test]
    fn fr_cat_046_the_level_admits_exactly_the_two_values_the_catalogue_writes() {
        // FR-CAT-046: `Table` on 22 of 24 and `Column` on 2, mixed case, and
        // no third value on any of the four series.
        assert_eq!(
            ConstraintLevel::from_catalogue("Table"),
            ConstraintLevel::Table
        );
        assert_eq!(
            ConstraintLevel::from_catalogue("Column"),
            ConstraintLevel::Column
        );
        assert_eq!(ConstraintLevel::Table.name(), "Table");
        assert_eq!(ConstraintLevel::Column.name(), "Column");
        assert_eq!(ConstraintLevel::Table.recorded(), Some("Table"));
    }

    #[test]
    fn fr_cat_055_a_level_outside_the_recorded_two_is_carried_and_is_not_a_refusal() {
        // FR-CAT-055: the catalogue declares this field `varchar(6)` and never
        // an `ENUM`, so nothing but the observation of FR-CAT-046 closes the
        // set. A third value is carried as the catalogue wrote it, case
        // included — the reading is exact, so `TABLE` is not `Table`.
        for outside in ["TABLE", "column", "", "Row"] {
            let carried = ConstraintLevel::from_catalogue(outside);

            assert_eq!(carried, ConstraintLevel::Unrecorded(Cow::Borrowed(outside)));
            assert_eq!(carried.name(), outside);
            assert_eq!(
                carried.recorded(),
                None,
                "an unrecorded level has no spelling a requirement fixes"
            );
        }
    }

    #[test]
    fn fr_cat_055_a_level_round_trips_through_the_document_whether_recorded_or_not() {
        // FR-CAT-055: the document carries the catalogue's own string, so a
        // value a server newer than the window returned survives the cache.
        for level in [
            ConstraintLevel::Table,
            ConstraintLevel::Column,
            ConstraintLevel::Unrecorded(Cow::Borrowed("Assertion")),
        ] {
            let written = serde_json::to_string(&level).expect("a level serialises");
            let read: ConstraintLevel<'_> =
                serde_json::from_str(&written).expect("a level reads back");

            assert_eq!(written, format!("\"{}\"", level.name()));
            assert_eq!(read, level);
        }
    }

    #[test]
    fn fr_cat_037_a_column_level_constraint_is_carried_under_the_column_name() {
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
    fn fr_cat_046_the_clause_is_carried_as_the_catalogue_rewrote_it() {
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
