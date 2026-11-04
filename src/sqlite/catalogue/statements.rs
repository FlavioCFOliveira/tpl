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
//! | `BEGIN DEFERRED` and `COMMIT`, the read transaction of `FR-LITE-042` | [`BEGIN`], [`COMMIT`] |
//! | `SELECT` against `sqlite_schema` and the pragma functions, restricted to `main` | [`CHECK`], [`ENCODING`] … [`STORED`] |
//!
//! **No statement invokes `pragma_table_list`** (`FR-LITE-015`, as amended in
//! the sixty-ninth edition). Every catalogue statement reads the objects from
//! `main.sqlite_schema`, and **a pragma function that takes a name is passed
//! the schema `main` and only the name of a row of type `table` whose root page
//! is not `0`**, the row restricted in the statement before the function is
//! reached: each join is a `CROSS JOIN`, which keeps the schema table in the
//! outer loop, and the `WHERE` terms on the outer table are evaluated before
//! the table-valued function of the inner loop is called for a row. So no
//! view's name and no virtual table's name is passed to one, no view is
//! compiled and no virtual table is connected (`FR-LITE-015`, `FR-LITE-019`,
//! `FR-LITE-020`). A view is read from [`OBJECTS`], which names it, and from
//! [`STORED`], which holds its stored text, and from nothing else.
//!
//! **Every statement after the check that reads the schema table selects the
//! row's `type` first and its `name` second**, so that a value of the schema
//! table of a class the check refuses — which, inside the read transaction of
//! `FR-LITE-042`, only a process that writes the file without SQLite's locks
//! can put there — is reported with the row it belongs to (`FR-LITE-040`, its
//! paragraph on values).
//!
//! **No statement orders its rows.** `FR-LITE-015` fixes which statements are
//! issued, not an order of their rows, and every order the model carries —
//! the objects by name, a table's columns by `cid`, an index's members by
//! `seqno`, a key's pairs by `seq` — is produced by the fold whatever order
//! the rows arrive in (`catalogue::fold`). PERF: an `ORDER BY` here sorted
//! every row in the library's sorter before the first was returned, for an
//! order the fold produces again.

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

/// The start of the read transaction of `FR-LITE-042`, immediately before the
/// check. It reads nothing; the snapshot is taken by the check.
pub(crate) const BEGIN: &str = "BEGIN DEFERRED";

/// The end of the read transaction, immediately after the last catalogue
/// statement, issued only where every one of them completed (`FR-LITE-042`).
pub(crate) const COMMIT: &str = "COMMIT";

/// The check of `FR-LITE-041`: every row of the schema table, each of its
/// five values as stored, judged in `tpl` by `check::verdict`. It is also the
/// catalogue probe of `tpl cfg database test` (`FR-CFG-056`).
pub(crate) const CHECK: &str = "SELECT s.type, s.name, s.tbl_name, s.rootpage, s.sql \
     FROM main.sqlite_schema AS s";

/// Statement 1 of the plan: the encoding of `main` (`FR-LITE-023`).
pub(crate) const ENCODING: &str = "SELECT encoding FROM pragma_encoding";

/// Statement 2: the rows of type `table` and `view`, with their root page,
/// and for a table whose root page is not `0` whether `pragma_index_info`
/// returns a row for its name — a `WITHOUT ROWID` table (`FR-LITE-019` …
/// `FR-LITE-021`, `FR-LITE-028`), in no stated order: the fold orders the
/// objects by name.
pub(crate) const OBJECTS: &str = "SELECT s.type, s.name, s.rootpage, \
     EXISTS (SELECT 1 FROM pragma_index_info(s.name, 'main')) \
     FROM main.sqlite_schema AS s \
     WHERE s.type = 'table' AND s.rootpage IS NOT 0 \
     UNION ALL \
     SELECT s.type, s.name, s.rootpage, 0 \
     FROM main.sqlite_schema AS s \
     WHERE s.type = 'view' OR (s.type = 'table' AND s.rootpage IS 0)";

/// Statement 3: the columns of every table (`FR-LITE-025` … `FR-LITE-029`),
/// in no stated order: the fold orders a table's columns by `cid`.
pub(crate) const TABLE_COLUMNS: &str = "SELECT s.type, s.name, c.cid, c.name, c.type, \
     c.\"notnull\", c.dflt_value, c.pk, c.hidden \
     FROM main.sqlite_schema AS s \
     CROSS JOIN pragma_table_xinfo(s.name, 'main') AS c \
     WHERE s.type = 'table' AND s.rootpage IS NOT 0";

/// Statement 4: the indexes of every table, and their members
/// (`FR-LITE-028` … `FR-LITE-030`), in no stated order: the fold orders an
/// index's members by `seqno`.
pub(crate) const INDEXES: &str = "SELECT s.type, s.name, l.name, l.\"unique\", l.origin, \
     x.seqno, x.cid, x.name, x.\"desc\", x.\"key\" \
     FROM main.sqlite_schema AS s \
     CROSS JOIN pragma_index_list(s.name, 'main') AS l \
     CROSS JOIN pragma_index_xinfo(l.name, 'main') AS x \
     WHERE s.type = 'table' AND s.rootpage IS NOT 0";

/// Statement 5: the foreign keys of every table (`FR-LITE-031`), in no stated
/// order: the fold orders a key's pairs by `seq`.
pub(crate) const FOREIGN_KEYS: &str = "SELECT s.type, s.name, f.id, f.seq, f.\"table\", \
     f.\"from\", f.\"to\", f.on_update, f.on_delete, f.\"match\" \
     FROM main.sqlite_schema AS s \
     CROSS JOIN pragma_foreign_key_list(s.name, 'main') AS f \
     WHERE s.type = 'table' AND s.rootpage IS NOT 0";

/// Statement 6: the stored text of every view and trigger (`FR-LITE-032`,
/// `FR-LITE-033`), in no stated order.
pub(crate) const STORED: &str = "SELECT s.type, s.name, s.tbl_name, s.sql \
     FROM main.sqlite_schema AS s \
     WHERE s.type IN ('view', 'trigger')";

/// Every statement of the list `FR-LITE-015` closes, in the order
/// `FR-LITE-017` and the plan fix them: the connection start, then the read.
#[cfg(test)]
pub(crate) const CLOSED_LIST: [&str; 19] = [
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
    BEGIN,
    CHECK,
    ENCODING,
    OBJECTS,
    TABLE_COLUMNS,
    INDEXES,
    FOREIGN_KEYS,
    STORED,
    COMMIT,
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

/// The six statements of the catalogue plan that follow the check, in the
/// order they are issued (`FR-LITE-015`, `FR-LITE-037`).
#[cfg(test)]
pub(crate) const PLAN: [&str; 6] = [
    ENCODING,
    OBJECTS,
    TABLE_COLUMNS,
    INDEXES,
    FOREIGN_KEYS,
    STORED,
];

/// The seven catalogue statements of a full read: the check, then the plan
/// (`FR-LITE-037`, as amended in the sixty-ninth edition).
#[cfg(test)]
pub(crate) const CATALOGUE: [&str; 7] = [
    CHECK,
    ENCODING,
    OBJECTS,
    TABLE_COLUMNS,
    INDEXES,
    FOREIGN_KEYS,
    STORED,
];

/// Every statement a full read issues after the connection start, in the
/// order of `FR-LITE-017`: `BEGIN DEFERRED`, the check, the plan, `COMMIT`
/// (`FR-LITE-042`).
#[cfg(test)]
pub(crate) const READ: [&str; 9] = [
    BEGIN,
    CHECK,
    ENCODING,
    OBJECTS,
    TABLE_COLUMNS,
    INDEXES,
    FOREIGN_KEYS,
    STORED,
    COMMIT,
];

#[cfg(test)]
mod tests {
    use super::{BEGIN, CATALOGUE, CHECK, CLOSED_LIST, COMMIT, CONNECTION_START, PLAN, READ};

    /// The words of `statement`, upper-cased.
    fn words(statement: &str) -> Vec<String> {
        statement
            .to_ascii_uppercase()
            .split(|character: char| !(character.is_ascii_alphanumeric() || character == '_'))
            .filter(|word| !word.is_empty())
            .map(str::to_owned)
            .collect()
    }

    #[test]
    fn fr_lite_016_no_statement_writes_attaches_or_names_another_schema() {
        // FR-LITE-016: no DDL, no DML, no ATTACH or DETACH, no VACUUM, ANALYZE
        // or REINDEX, nothing against a schema other than `main`, and no
        // transaction statement but the BEGIN DEFERRED and the COMMIT of
        // FR-LITE-042.
        for statement in CLOSED_LIST {
            let words = words(statement);
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
                "ROLLBACK",
                "SAVEPOINT",
                "RELEASE",
                "END",
                "IMMEDIATE",
                "EXCLUSIVE",
                "TEMP",
                "INTEGRITY_CHECK",
                "QUICK_CHECK",
            ] {
                assert!(
                    !words.iter().any(|word| word == refused),
                    "{statement} carries {refused}"
                );
            }
            if statement != BEGIN {
                assert!(!words.iter().any(|word| word == "BEGIN"), "{statement}");
            }
            if statement != COMMIT {
                assert!(!words.iter().any(|word| word == "COMMIT"), "{statement}");
            }
        }
        assert_eq!(BEGIN, "BEGIN DEFERRED");
        assert_eq!(COMMIT, "COMMIT");
    }

    #[test]
    fn fr_lite_015_the_connection_start_and_the_read_are_members_of_the_closed_list() {
        for statement in CONNECTION_START.iter().chain(READ.iter()) {
            assert!(CLOSED_LIST.contains(statement), "{statement}");
        }
        assert_eq!(PLAN.len(), 6);
        assert_eq!(CATALOGUE.len(), 7);
        assert_eq!(CATALOGUE[0], CHECK);
        assert_eq!(&CATALOGUE[1..], PLAN);
        assert_eq!(READ.first(), Some(&BEGIN));
        assert_eq!(&READ[1..8], CATALOGUE);
        assert_eq!(READ.last(), Some(&COMMIT));
    }

    #[test]
    fn fr_lite_015_no_statement_invokes_pragma_table_list_or_passes_a_view_to_a_pragma_function() {
        // FR-LITE-015 as amended in the sixty-ninth edition: no statement
        // invokes `pragma_table_list`, and every pragma function that takes a
        // table's name is reached only through a row of `main.sqlite_schema`
        // restricted to the type `table` and a root page other than 0, so no
        // view's or virtual table's name is passed to one. `pragma_index_xinfo`
        // takes an index's name, from a row of `pragma_index_list`.
        const RESTRICTED: &str = "WHERE s.type = 'table' AND s.rootpage IS NOT 0";

        for statement in CLOSED_LIST {
            assert!(
                !statement.to_ascii_lowercase().contains("pragma_table_list"),
                "{statement} invokes pragma_table_list"
            );
            for function in [
                "pragma_table_info(",
                "pragma_table_xinfo(",
                "pragma_index_list(",
                "pragma_index_info(",
                "pragma_foreign_key_list(",
            ] {
                if statement.contains(function) {
                    assert!(
                        statement.contains(&format!("{function}s.name, 'main')")),
                        "{statement} passes {function} something other than a schema row's name"
                    );
                    assert!(
                        statement.contains("FROM main.sqlite_schema AS s"),
                        "{statement} reaches {function} without the schema table"
                    );
                    assert!(
                        statement.contains(RESTRICTED),
                        "{statement} reaches {function} for a row that is not a table"
                    );
                }
            }
            if statement.contains("pragma_index_xinfo(") {
                assert!(statement.contains("pragma_index_xinfo(l.name, 'main')"));
                assert!(statement.contains("pragma_index_list(s.name, 'main')"));
            }
            // Every join is a CROSS JOIN, which keeps the schema table outer.
            let words = words(statement);
            for (at, word) in words.iter().enumerate() {
                if word == "JOIN" {
                    assert_eq!(
                        words.get(at.wrapping_sub(1)).map(String::as_str),
                        Some("CROSS"),
                        "{statement}"
                    );
                }
            }
        }
    }

    #[test]
    fn fr_lite_040_every_statement_after_the_check_that_reads_the_schema_table_selects_type_then_name()
     {
        // The paragraph on values of FR-LITE-040: a value of the schema table
        // of a class the check refuses is reported with its row, which the
        // reader finds at these two columns.
        for statement in PLAN {
            if statement.contains("sqlite_schema") {
                assert!(
                    statement.starts_with("SELECT s.type, s.name, "),
                    "{statement}"
                );
            }
        }
        assert!(CHECK.starts_with("SELECT s.type, s.name, s.tbl_name, s.rootpage, s.sql"));
    }
}
