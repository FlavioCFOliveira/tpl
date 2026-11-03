//! The text of every statement a `sqlite` entry issues, one constant each
//! (`FR-LITE-015`, `FR-LITE-016`).
//!
//! The list is closed by construction: no other module of `sqlite/` composes
//! SQL, no value read from the file enters a statement's text, and
//! [`CLOSED_LIST`] is the list the statement record of `FR-LITE-018` is
//! checked against.
//!
//! | Row of `FR-LITE-015` | Constants |
//! |---|---|
//! | The five controls applied by a statement, each set and read back | [`SET_QUERY_ONLY`] … [`READ_TEMP_STORE`] |
//! | `PRAGMA compile_options` | [`COMPILE_OPTIONS`] |
//! | The version probe | [`PROBE`] |
//! | `SELECT` against `sqlite_schema` and the pragma functions, restricted to `main` | [`ENCODING`] … [`STORED`], [`CATALOGUE_PROBE`] |
//!
//! **Each join restricts `pragma_table_list` to `main` and to the covered
//! types before the function it joins is reached** (architecture, *The SQLite
//! reader*): the `WHERE` terms on the outer table are evaluated before the
//! table-valued function of the inner loop is called for a row, so a virtual
//! table whose module the library lacks is never asked for its columns
//! (`FR-LITE-021`, observed there).
//!
//! **No statement reads from a view or compiles one** (`FR-LITE-015`,
//! `FR-LITE-033`). Every join to a pragma function that takes a table's name
//! restricts `pragma_table_list` to the type `table`, so no view's name is
//! passed to one; a view is read from [`OBJECTS`], which names it, and from
//! [`STORED`], which holds its stored text, and from nothing else.

/// The confirmation of the extension-loading control, and of the build the
/// temporary-storage control depends on (`FR-LITE-010`).
pub(crate) const COMPILE_OPTIONS: &str = "PRAGMA compile_options";

/// `query_only` on (`FR-LITE-010`).
pub(crate) const SET_QUERY_ONLY: &str = "PRAGMA query_only = 1";

/// Its read-back.
pub(crate) const READ_QUERY_ONLY: &str = "PRAGMA query_only";

/// `cell_size_check` on (`FR-LITE-010`).
pub(crate) const SET_CELL_SIZE_CHECK: &str = "PRAGMA cell_size_check = 1";

/// Its read-back.
pub(crate) const READ_CELL_SIZE_CHECK: &str = "PRAGMA cell_size_check";

/// `mmap_size` `0` (`FR-LITE-010`).
pub(crate) const SET_MMAP_SIZE: &str = "PRAGMA mmap_size = 0";

/// Its read-back.
pub(crate) const READ_MMAP_SIZE: &str = "PRAGMA mmap_size";

/// `temp_store` `MEMORY` (`FR-LITE-010`).
pub(crate) const SET_TEMP_STORE: &str = "PRAGMA temp_store = 2";

/// Its read-back.
pub(crate) const READ_TEMP_STORE: &str = "PRAGMA temp_store";

/// The version probe of `FR-LITE-002`, the first statement that reads the
/// file.
pub(crate) const PROBE: &str = "SELECT sqlite_version()";

/// Statement 1: the encoding of `main` (`FR-LITE-023`).
pub(crate) const ENCODING: &str = "SELECT encoding FROM pragma_encoding";

/// Statement 2: the objects of `main`, with their type, `wr` and `strict`
/// (`FR-LITE-019` … `FR-LITE-021`, `FR-LITE-028`).
pub(crate) const OBJECTS: &str = "SELECT t.name, t.type, t.wr, t.strict \
     FROM pragma_table_list AS t \
     WHERE t.schema = 'main' \
     ORDER BY t.name";

/// Statement 3: the columns of every table (`FR-LITE-025` … `FR-LITE-029`).
pub(crate) const TABLE_COLUMNS: &str = "SELECT t.name, c.cid, c.name, c.type, c.\"notnull\", \
     c.dflt_value, c.pk, c.hidden \
     FROM pragma_table_list AS t \
     JOIN pragma_table_xinfo(t.name, t.schema) AS c \
     WHERE t.schema = 'main' AND t.type = 'table' \
     ORDER BY t.name, c.cid";

/// Statement 4: the indexes of every table, and their members
/// (`FR-LITE-028` … `FR-LITE-030`).
pub(crate) const INDEXES: &str = "SELECT t.name, l.name, l.\"unique\", l.origin, x.seqno, \
     x.cid, x.name, x.\"desc\", x.\"key\" \
     FROM pragma_table_list AS t \
     JOIN pragma_index_list(t.name, t.schema) AS l \
     JOIN pragma_index_xinfo(l.name, t.schema) AS x \
     WHERE t.schema = 'main' AND t.type = 'table' \
     ORDER BY t.name, l.name, x.seqno";

/// Statement 5: the foreign keys of every table (`FR-LITE-031`).
pub(crate) const FOREIGN_KEYS: &str = "SELECT t.name, f.id, f.seq, f.\"table\", f.\"from\", \
     f.\"to\", f.on_update, f.on_delete, f.\"match\" \
     FROM pragma_table_list AS t \
     JOIN pragma_foreign_key_list(t.name, t.schema) AS f \
     WHERE t.schema = 'main' AND t.type = 'table' \
     ORDER BY t.name, f.id, f.seq";

/// Statement 6: the stored text of every view and trigger (`FR-LITE-032`,
/// `FR-LITE-033`).
pub(crate) const STORED: &str = "SELECT s.type, s.name, s.tbl_name, s.sql \
     FROM main.sqlite_schema AS s \
     WHERE s.type IN ('view', 'trigger') \
     ORDER BY s.name";

/// The catalogue probe of `FR-CFG-056`, step 4: one `SELECT` against
/// `sqlite_schema` of `main`, whose rows are not read as model content.
pub(crate) const CATALOGUE_PROBE: &str = "SELECT count(*) FROM main.sqlite_schema";

/// Every statement of the list `FR-LITE-015` closes, in the order
/// `FR-LITE-017` and the plan fix them: the connection start, then the
/// catalogue.
#[cfg(test)]
pub(crate) const CLOSED_LIST: [&str; 17] = [
    COMPILE_OPTIONS,
    SET_QUERY_ONLY,
    READ_QUERY_ONLY,
    SET_CELL_SIZE_CHECK,
    READ_CELL_SIZE_CHECK,
    SET_MMAP_SIZE,
    READ_MMAP_SIZE,
    SET_TEMP_STORE,
    READ_TEMP_STORE,
    PROBE,
    ENCODING,
    OBJECTS,
    TABLE_COLUMNS,
    INDEXES,
    FOREIGN_KEYS,
    STORED,
    CATALOGUE_PROBE,
];

/// The statements of the connection start, in the order `FR-LITE-017` fixes:
/// the build, the controls, then the probe.
#[cfg(test)]
pub(crate) const CONNECTION_START: [&str; 10] = [
    COMPILE_OPTIONS,
    SET_QUERY_ONLY,
    READ_QUERY_ONLY,
    SET_CELL_SIZE_CHECK,
    READ_CELL_SIZE_CHECK,
    SET_MMAP_SIZE,
    READ_MMAP_SIZE,
    SET_TEMP_STORE,
    READ_TEMP_STORE,
    PROBE,
];

/// The six statements of the catalogue plan, in the order they are issued
/// (`FR-LITE-015`, `FR-LITE-037`).
#[cfg(test)]
pub(crate) const PLAN: [&str; 6] = [
    ENCODING,
    OBJECTS,
    TABLE_COLUMNS,
    INDEXES,
    FOREIGN_KEYS,
    STORED,
];

#[cfg(test)]
mod tests {
    use super::{CLOSED_LIST, CONNECTION_START, PLAN};

    #[test]
    fn fr_lite_016_no_statement_writes_attaches_or_names_another_schema() {
        // FR-LITE-016: no DDL, no DML, no ATTACH or DETACH, no VACUUM, ANALYZE
        // or REINDEX, no transaction statement, and nothing against a schema
        // other than `main`.
        for statement in CLOSED_LIST {
            let upper = statement.to_ascii_uppercase();
            let words: Vec<&str> = upper
                .split(|character: char| !(character.is_ascii_alphanumeric() || character == '_'))
                .filter(|word| !word.is_empty())
                .collect();
            for refused in [
                "INSERT",
                "UPDATE",
                "DELETE",
                "CREATE",
                "DROP",
                "ALTER",
                "ATTACH",
                "DETACH",
                "VACUUM",
                "ANALYZE",
                "REINDEX",
                "BEGIN",
                "COMMIT",
                "ROLLBACK",
                "SAVEPOINT",
                "TEMP",
                "INTEGRITY_CHECK",
                "QUICK_CHECK",
            ] {
                assert!(!words.contains(&refused), "{statement} carries {refused}");
            }
        }
    }

    #[test]
    fn fr_lite_015_the_connection_start_and_the_plan_are_members_of_the_closed_list() {
        for statement in CONNECTION_START.iter().chain(PLAN.iter()) {
            assert!(CLOSED_LIST.contains(statement), "{statement}");
        }
        assert_eq!(PLAN.len(), 6);
    }

    #[test]
    fn fr_lite_015_no_statement_passes_a_view_to_a_pragma_function() {
        // FR-LITE-015 as amended in the sixty-eighth edition: no statement
        // reads from a view or passes a view's name to a pragma function, so
        // none makes the library compile a view's definition. Every function
        // that takes a table's name is reached only through a row of
        // `pragma_table_list` restricted to the type `table`.
        for statement in CLOSED_LIST {
            for function in [
                "pragma_table_info(",
                "pragma_table_xinfo(",
                "pragma_index_list(",
                "pragma_foreign_key_list(",
            ] {
                if statement.contains(function) {
                    assert!(
                        statement.contains("t.type = 'table'"),
                        "{statement} reaches {function} for an object that is not a table"
                    );
                }
            }
            assert!(
                !statement.contains("t.type = 'view'"),
                "{statement} selects the views of pragma_table_list for a join"
            );
        }
    }
}
