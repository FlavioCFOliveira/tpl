//! The twelve statements of a `mysql` read, one per table of the closed list
//! (`FR-MYSQL-015`, `FR-MYSQL-042`).
//!
//! **One statement per `INFORMATION_SCHEMA` table, and the plan is a constant.**
//! [`plan`] is given the schema name and nothing else, so the number of
//! statements a read issues cannot depend on what the database holds: twelve
//! for a full read, whatever the database, which is `NFR-PERF-001` and
//! `NFR-PERF-002` as `FR-MYSQL-042` applies them. Every command reads the whole
//! catalogue, so there is no narrowed plan; `NFR-PERF-002` counts statements,
//! and a read that names one object issues the same twelve.
//!
//! | # | [`Read`] | Table | Mapped by |
//! |---|---|---|---|
//! | 1 | [`Read::Schema`] | `SCHEMATA` | `FR-MYSQL-025` |
//! | 2 | [`Read::Tables`] | `TABLES` | `FR-MYSQL-026` |
//! | 3 | [`Read::Columns`] | `COLUMNS` | `FR-MYSQL-027` … `FR-MYSQL-031` |
//! | 4 | [`Read::Indexes`] | `STATISTICS` | `FR-MYSQL-024`, `FR-MYSQL-032` |
//! | 5 | [`Read::ForeignKeyRules`] | `REFERENTIAL_CONSTRAINTS` | `FR-MYSQL-033` |
//! | 6 | [`Read::KeyColumns`] | `KEY_COLUMN_USAGE` | `FR-MYSQL-033` |
//! | 7 | [`Read::CheckOwners`] | `TABLE_CONSTRAINTS` | `FR-MYSQL-034` |
//! | 8 | [`Read::Checks`] | `CHECK_CONSTRAINTS` | `FR-MYSQL-034` |
//! | 9 | [`Read::Triggers`] | `TRIGGERS` | `FR-MYSQL-038` |
//! | 10 | [`Read::Views`] | `VIEWS` | `FR-MYSQL-035` |
//! | 11 | [`Read::Routines`] | `ROUTINES` | `FR-MYSQL-036` |
//! | 12 | [`Read::Parameters`] | `PARAMETERS` | `FR-MYSQL-037` |
//!
//! **The `CHECK` constraint is related to its table by its name, in the fold,
//! and not by a statement per table.** MySQL's check-constraint table carries
//! no table name (`FR-MYSQL-034`); the table-constraint table does, and the two
//! are joined on the schema and the constraint name, which MySQL makes unique
//! in a schema. Two statements, one per table, keep the closed list's ceiling
//! and the rationale of `FR-MYSQL-042` in one reading: each statement reads one
//! table, restricted to the database alone.
//!
//! **Every value travels as a bind parameter**, so each text is a
//! `&'static str` and nothing a caller supplies is composed into SQL. The one
//! literal, `'CHECK'`, is the constraint type `FR-MYSQL-034` selects on.
//!
//! **No field outside the mapping is named**, so every field a statement reads
//! is one `FR-MYSQL-025` … `FR-MYSQL-038` maps to the model or uses as a
//! discriminant. Each field named exists by that name on both series, per the
//! observation `FR-MYSQL-015` records.

/// The schema row (`FR-MYSQL-025`): the three fields the model carries.
const SCHEMA: &str = "SELECT SCHEMA_NAME, DEFAULT_CHARACTER_SET_NAME, DEFAULT_COLLATION_NAME \
     FROM INFORMATION_SCHEMA.SCHEMATA WHERE SCHEMA_NAME = ?";

/// The table rows (`FR-MYSQL-026`). The twelve volatile fields and the two
/// `FR-CAT-053` names no property for are not named.
const TABLES: &str = "SELECT TABLE_NAME, TABLE_TYPE, ENGINE, TABLE_COLLATION, TABLE_COMMENT \
     FROM INFORMATION_SCHEMA.TABLES WHERE TABLE_SCHEMA = ? ORDER BY TABLE_NAME";

/// The column rows (`FR-MYSQL-027`). `CHARACTER_OCTET_LENGTH`, `COLUMN_KEY`,
/// `PRIVILEGES` and `SRS_ID` are not carried and not named.
const COLUMNS: &str = "SELECT TABLE_NAME, COLUMN_NAME, ORDINAL_POSITION, COLUMN_DEFAULT, \
     IS_NULLABLE, DATA_TYPE, CHARACTER_MAXIMUM_LENGTH, NUMERIC_PRECISION, NUMERIC_SCALE, \
     DATETIME_PRECISION, CHARACTER_SET_NAME, COLLATION_NAME, COLUMN_TYPE, EXTRA, COLUMN_COMMENT, \
     GENERATION_EXPRESSION FROM INFORMATION_SCHEMA.COLUMNS WHERE TABLE_SCHEMA = ? \
     ORDER BY TABLE_NAME, ORDINAL_POSITION";

/// The index rows (`FR-MYSQL-032`). `EXPRESSION` is read as the mark of a
/// functional key part (`FR-MYSQL-024`) and not carried; `CARDINALITY`,
/// `PACKED`, `NULLABLE` and `COMMENT` are not named.
const INDEXES: &str = "SELECT TABLE_NAME, INDEX_NAME, NON_UNIQUE, COLUMN_NAME, COLLATION, \
     SUB_PART, INDEX_TYPE, INDEX_COMMENT, IS_VISIBLE, EXPRESSION \
     FROM INFORMATION_SCHEMA.STATISTICS WHERE TABLE_SCHEMA = ? \
     ORDER BY TABLE_NAME, INDEX_NAME, SEQ_IN_INDEX";

/// The rules half of a foreign key (`FR-MYSQL-033`), the fields `FR-CAT-045`
/// reads under the same names.
const RULES: &str = "SELECT TABLE_NAME, CONSTRAINT_NAME, UNIQUE_CONSTRAINT_NAME, MATCH_OPTION, \
     UPDATE_RULE, DELETE_RULE, REFERENCED_TABLE_NAME \
     FROM INFORMATION_SCHEMA.REFERENTIAL_CONSTRAINTS WHERE CONSTRAINT_SCHEMA = ? \
     ORDER BY TABLE_NAME, CONSTRAINT_NAME";

/// The columns half of a foreign key (`FR-MYSQL-033`), restricted to the rows
/// that name a referenced table, as `FR-CAT-045` restricts them; the
/// referenced schema is selected for `FR-CAT-057`.
const KEY_COLUMNS: &str = "SELECT TABLE_NAME, CONSTRAINT_NAME, COLUMN_NAME, \
     REFERENCED_TABLE_SCHEMA, REFERENCED_TABLE_NAME, REFERENCED_COLUMN_NAME \
     FROM INFORMATION_SCHEMA.KEY_COLUMN_USAGE \
     WHERE TABLE_SCHEMA = ? AND REFERENCED_TABLE_NAME IS NOT NULL \
     ORDER BY TABLE_NAME, CONSTRAINT_NAME, ORDINAL_POSITION";

/// The table of each `CHECK` constraint (`FR-MYSQL-034`): the rows of the
/// table-constraint table whose type is `CHECK`, and no other row.
const CHECK_OWNERS: &str = "SELECT TABLE_NAME, CONSTRAINT_NAME \
     FROM INFORMATION_SCHEMA.TABLE_CONSTRAINTS \
     WHERE CONSTRAINT_SCHEMA = ? AND CONSTRAINT_TYPE = 'CHECK' \
     ORDER BY TABLE_NAME, CONSTRAINT_NAME";

/// The clause of each `CHECK` constraint (`FR-MYSQL-034`).
const CHECKS: &str = "SELECT CONSTRAINT_NAME, CHECK_CLAUSE \
     FROM INFORMATION_SCHEMA.CHECK_CONSTRAINTS WHERE CONSTRAINT_SCHEMA = ? \
     ORDER BY CONSTRAINT_NAME";

/// The trigger rows (`FR-MYSQL-038`), the fields `FR-CAT-050` reads.
const TRIGGERS: &str = "SELECT EVENT_OBJECT_TABLE, TRIGGER_NAME, EVENT_MANIPULATION, \
     ACTION_TIMING, ACTION_ORDER, ACTION_STATEMENT, ACTION_ORIENTATION, \
     ACTION_REFERENCE_OLD_ROW, ACTION_REFERENCE_NEW_ROW, SQL_MODE, DEFINER, \
     CHARACTER_SET_CLIENT, COLLATION_CONNECTION, DATABASE_COLLATION \
     FROM INFORMATION_SCHEMA.TRIGGERS WHERE EVENT_OBJECT_SCHEMA = ? \
     ORDER BY EVENT_OBJECT_TABLE, TRIGGER_NAME";

/// The view rows (`FR-MYSQL-035`): the eight fields that fill eight of the
/// nine properties; MySQL's view catalogue has no algorithm field.
const VIEWS: &str = "SELECT TABLE_NAME, VIEW_DEFINITION, CHECK_OPTION, IS_UPDATABLE, DEFINER, \
     SECURITY_TYPE, CHARACTER_SET_CLIENT, COLLATION_CONNECTION \
     FROM INFORMATION_SCHEMA.VIEWS WHERE TABLE_SCHEMA = ? ORDER BY TABLE_NAME";

/// The routine rows (`FR-MYSQL-036`), the fields `FR-CAT-048` reads.
const ROUTINES: &str = "SELECT ROUTINE_NAME, ROUTINE_TYPE, DTD_IDENTIFIER, DATA_TYPE, \
     CHARACTER_MAXIMUM_LENGTH, NUMERIC_PRECISION, NUMERIC_SCALE, DATETIME_PRECISION, \
     CHARACTER_SET_NAME, COLLATION_NAME, ROUTINE_BODY, ROUTINE_DEFINITION, PARAMETER_STYLE, \
     IS_DETERMINISTIC, SQL_DATA_ACCESS, SECURITY_TYPE, SQL_MODE, ROUTINE_COMMENT, DEFINER, \
     CHARACTER_SET_CLIENT, COLLATION_CONNECTION, DATABASE_COLLATION \
     FROM INFORMATION_SCHEMA.ROUTINES WHERE ROUTINE_SCHEMA = ? \
     ORDER BY ROUTINE_NAME, ROUTINE_TYPE";

/// The parameter rows (`FR-MYSQL-037`). `ORDINAL_POSITION > 0` excludes a
/// function's return row, which is not a parameter.
const PARAMETERS: &str = "SELECT SPECIFIC_NAME, ROUTINE_TYPE, PARAMETER_NAME, PARAMETER_MODE, \
     DTD_IDENTIFIER, DATA_TYPE, CHARACTER_MAXIMUM_LENGTH, NUMERIC_PRECISION, NUMERIC_SCALE, \
     DATETIME_PRECISION, CHARACTER_SET_NAME, COLLATION_NAME \
     FROM INFORMATION_SCHEMA.PARAMETERS WHERE SPECIFIC_SCHEMA = ? AND ORDINAL_POSITION > 0 \
     ORDER BY SPECIFIC_NAME, ROUTINE_TYPE, ORDINAL_POSITION";

/// What one statement reads, which is how the fold finds its rows again.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum Read {
    /// `SCHEMATA`.
    Schema,
    /// `TABLES`.
    Tables,
    /// `COLUMNS`.
    Columns,
    /// `STATISTICS`.
    Indexes,
    /// `REFERENTIAL_CONSTRAINTS`.
    ForeignKeyRules,
    /// `KEY_COLUMN_USAGE`.
    KeyColumns,
    /// `TABLE_CONSTRAINTS`, its `CHECK` rows.
    CheckOwners,
    /// `CHECK_CONSTRAINTS`.
    Checks,
    /// `TRIGGERS`.
    Triggers,
    /// `VIEWS`.
    Views,
    /// `ROUTINES`.
    Routines,
    /// `PARAMETERS`.
    Parameters,
}

impl Read {
    /// Every member, in the order a read issues them.
    pub(super) const ALL: [Self; 12] = [
        Self::Schema,
        Self::Tables,
        Self::Columns,
        Self::Indexes,
        Self::ForeignKeyRules,
        Self::KeyColumns,
        Self::CheckOwners,
        Self::Checks,
        Self::Triggers,
        Self::Views,
        Self::Routines,
        Self::Parameters,
    ];

    /// The number of statements of a read, and of slots their rows occupy.
    pub(super) const COUNT: usize = Self::ALL.len();

    /// The slot this read's rows are kept in.
    pub(super) const fn slot(self) -> usize {
        match self {
            Self::Schema => 0,
            Self::Tables => 1,
            Self::Columns => 2,
            Self::Indexes => 3,
            Self::ForeignKeyRules => 4,
            Self::KeyColumns => 5,
            Self::CheckOwners => 6,
            Self::Checks => 7,
            Self::Triggers => 8,
            Self::Views => 9,
            Self::Routines => 10,
            Self::Parameters => 11,
        }
    }

    /// The text of this read's statement.
    pub(super) const fn sql(self) -> &'static str {
        match self {
            Self::Schema => SCHEMA,
            Self::Tables => TABLES,
            Self::Columns => COLUMNS,
            Self::Indexes => INDEXES,
            Self::ForeignKeyRules => RULES,
            Self::KeyColumns => KEY_COLUMNS,
            Self::CheckOwners => CHECK_OWNERS,
            Self::Checks => CHECKS,
            Self::Triggers => TRIGGERS,
            Self::Views => VIEWS,
            Self::Routines => ROUTINES,
            Self::Parameters => PARAMETERS,
        }
    }

    /// The `INFORMATION_SCHEMA` table this read's statement reads.
    #[cfg(test)]
    pub(super) const fn table(self) -> &'static str {
        match self {
            Self::Schema => "SCHEMATA",
            Self::Tables => "TABLES",
            Self::Columns => "COLUMNS",
            Self::Indexes => "STATISTICS",
            Self::ForeignKeyRules => "REFERENTIAL_CONSTRAINTS",
            Self::KeyColumns => "KEY_COLUMN_USAGE",
            Self::CheckOwners => "TABLE_CONSTRAINTS",
            Self::Checks => "CHECK_CONSTRAINTS",
            Self::Triggers => "TRIGGERS",
            Self::Views => "VIEWS",
            Self::Routines => "ROUTINES",
            Self::Parameters => "PARAMETERS",
        }
    }
}

/// The statements of a read, in the order they are issued (`NFR-PERF-001`).
///
/// It is the constant [`Read::ALL`]: the schema name is bound to each, and
/// nothing else varies.
pub(super) const fn plan() -> [Read; Read::COUNT] {
    Read::ALL
}

/// The one statement of the privilege probe of `FR-CFG-044`: the schema row a
/// read issues first, which is a `SELECT` against one of the twelve tables
/// `FR-MYSQL-015` closes, restricted to the database the entry names.
pub(super) const fn probe() -> Read {
    Read::Schema
}

#[cfg(test)]
mod tests {
    use super::{Read, plan, probe};

    /// The twelve tables of `FR-MYSQL-015`, in its order.
    const CLOSED_LIST: [&str; 12] = [
        "SCHEMATA",
        "TABLES",
        "COLUMNS",
        "STATISTICS",
        "KEY_COLUMN_USAGE",
        "REFERENTIAL_CONSTRAINTS",
        "TABLE_CONSTRAINTS",
        "CHECK_CONSTRAINTS",
        "VIEWS",
        "ROUTINES",
        "PARAMETERS",
        "TRIGGERS",
    ];

    #[test]
    fn fr_mysql_015_each_statement_reads_one_table_of_the_closed_list_and_no_other() {
        let mut read: Vec<&str> = plan().iter().map(|statement| statement.table()).collect();
        let mut closed = CLOSED_LIST.to_vec();
        read.sort_unstable();
        closed.sort_unstable();

        // Every table of the list is read once, and no thirteenth.
        assert_eq!(read, closed);

        for statement in plan() {
            let sql = statement.sql();
            let named = format!("FROM INFORMATION_SCHEMA.{} ", statement.table());

            assert!(sql.starts_with("SELECT "), "{sql}");
            assert!(sql.contains(&named), "{sql}");
            assert_eq!(sql.matches("INFORMATION_SCHEMA.").count(), 1, "{sql}");
            // FR-MYSQL-016: no other kind of statement and no other schema.
            for barred in [
                "SHOW ", "INSERT ", "UPDATE ", "DELETE ", "CREATE ", "SET ", "@@",
            ] {
                assert!(!sql.contains(barred), "{barred} in {sql}");
            }
            // One bind, the schema; nothing is composed into the text.
            assert_eq!(sql.matches('?').count(), 1, "{sql}");
        }
    }

    #[test]
    fn nfr_perf_001_the_plan_is_twelve_statements_whatever_the_database_holds() {
        assert_eq!(plan().len(), 12);

        for (slot, statement) in plan().into_iter().enumerate() {
            assert_eq!(statement.slot(), slot);
        }
    }

    #[test]
    fn fr_cfg_044_the_probe_is_the_schema_row_every_read_issues_first() {
        assert_eq!(probe(), Read::Schema);
        assert_eq!(plan()[0], Read::Schema);
    }

    #[test]
    fn fr_mysql_035_the_view_statement_names_no_algorithm_field() {
        // MySQL's view catalogue has none, and naming it would fail.
        assert!(!Read::Views.sql().contains("ALGORITHM"));
        // FR-MYSQL-027: no is-generated field exists to be named.
        assert!(!Read::Columns.sql().contains("IS_GENERATED"));
    }
}
