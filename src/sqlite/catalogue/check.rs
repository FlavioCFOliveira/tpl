//! The verdict of the check over one row of the schema table (`FR-LITE-041`).
//!
//! The check is one statement against `main.sqlite_schema`, issued after the
//! version probe and before every other catalogue statement, inside the read
//! transaction of `FR-LITE-042`; it is also the catalogue probe of
//! `tpl cfg database test` (`FR-CFG-056`). Its statement returns every row of
//! the schema table, each of its five values as stored, and the verdict is
//! decided here, in `tpl`, by [`verdict`] — a pure function over one row's
//! values, testable as a vector (architecture, *The SQLite reader*).
//!
//! | Kind refused | How it is recognised |
//! |---|---|
//! | A table or a view named in the namespace of the pragma table-valued functions | A `type` of `table` or `view` and a `name` whose first seven bytes are `pragma_`, ignoring ASCII case, whether the name is a text or a BLOB |
//! | A value of a storage class the schema table's contract excludes | A `type`, `name` or `tbl_name` that is not text, a `rootpage` that is neither an integer nor `NULL`, an `sql` that is neither text nor `NULL` |
//! | A table whose root page contradicts its text | A row of type `table` whose `rootpage` is `0` and whose text does not create a virtual table, or whose `rootpage` is not `0` and whose text creates one — read from the first word after `CREATE`, under SQLite's comment syntax |
//!
//! A row refused under more than one kind is reported under the first, in the
//! order of that table. An integer or a real is written in digits and cannot
//! begin with `pragma_`; a `NULL` root page, which the library refuses when it
//! loads the schema, is not `0`.

use std::borrow::Cow;

use rusqlite::types::ValueRef;

use super::trigger;
use crate::error::{FileValue, RefusedKind, RefusedRow};

/// The prefix SQLite gives the name of every pragma table-valued function.
const PRAGMA: &[u8] = b"pragma_";

/// The keyword whose place after `CREATE` makes a text create a virtual table.
const VIRTUAL: &str = "VIRTUAL";

/// The five columns of the schema table, in the order the check's statement
/// selects them and the order a value of an excluded class is reported in.
pub(crate) const COLUMNS: [&str; 5] = ["type", "name", "tbl_name", "rootpage", "sql"];

/// The verdict of the check over one row: [`None`] where the row is admitted,
/// and the [`RefusedRow`] of the first kind that refuses it otherwise.
///
/// `values` are the row's `type`, `name`, `tbl_name`, `rootpage` and `sql`,
/// as stored.
#[must_use]
pub(crate) fn verdict(values: [ValueRef<'_>; 5]) -> Option<RefusedRow> {
    let [kind, name, _, rootpage, sql] = values;
    let refused = |refused: RefusedKind| {
        Some(RefusedRow {
            row_type: written(kind),
            name: written(name),
            kind: refused,
        })
    };

    // The first kind: a table or a view that takes the place of a function.
    if (spelled(kind, "table") || spelled(kind, "view")) && pragma_named(name) {
        return refused(RefusedKind::PragmaName);
    }

    // The second kind: a value of a class the contract excludes.
    for (column, value) in COLUMNS.into_iter().zip(values) {
        if !admitted(column, value) {
            return refused(RefusedKind::StorageClass {
                column,
                class: class(value),
            });
        }
    }

    // The third kind: a table whose root page contradicts its text.
    if matches!(kind, ValueRef::Text(b"table")) {
        let creates_virtual = creates_virtual(sql);
        let zero = matches!(rootpage, ValueRef::Integer(0));
        if zero != creates_virtual {
            return refused(RefusedKind::RootPage {
                rootpage: match rootpage {
                    ValueRef::Integer(page) => Some(page),
                    _ => None,
                },
                creates_virtual,
            });
        }
    }

    None
}

/// The refusal of a value of the schema table that a statement after the
/// check reads in a class the check refuses — which only a file changed after
/// the check can hold (`FR-LITE-040`, its paragraph on values).
///
/// `kind` and `name` are the row's `type` and `name` where the statement read
/// them, and `column` and `value` the value refused.
#[must_use]
pub(crate) fn excluded(
    kind: Option<ValueRef<'_>>,
    name: Option<ValueRef<'_>>,
    column: &'static str,
    value: ValueRef<'_>,
) -> RefusedRow {
    RefusedRow {
        row_type: kind.map_or_else(|| FileValue::new("NULL"), written),
        name: name.map_or_else(|| FileValue::new("NULL"), written),
        kind: RefusedKind::StorageClass {
            column,
            class: class(value),
        },
    }
}

/// Whether `value` is of a class the schema table's contract admits in
/// `column` (`https://sqlite.org/schematab.html`).
fn admitted(column: &str, value: ValueRef<'_>) -> bool {
    match column {
        "rootpage" => matches!(value, ValueRef::Integer(_) | ValueRef::Null),
        "sql" => matches!(value, ValueRef::Text(_) | ValueRef::Null),
        _ => matches!(value, ValueRef::Text(_)),
    }
}

/// Whether `value`, a text or a BLOB, holds exactly `expected`.
fn spelled(value: ValueRef<'_>, expected: &str) -> bool {
    match value {
        ValueRef::Text(bytes) | ValueRef::Blob(bytes) => bytes == expected.as_bytes(),
        _ => false,
    }
}

/// Whether `name`, a text or a BLOB, begins with `pragma_`, ignoring ASCII
/// case and compared on its bytes, whatever its storage class.
fn pragma_named(name: ValueRef<'_>) -> bool {
    match name {
        ValueRef::Text(bytes) | ValueRef::Blob(bytes) => bytes
            .get(..PRAGMA.len())
            .is_some_and(|prefix| prefix.eq_ignore_ascii_case(PRAGMA)),
        _ => false,
    }
}

/// Whether `sql` creates a virtual table: its first word after `CREATE`,
/// under SQLite's comment syntax, is `VIRTUAL`, ignoring ASCII case.
fn creates_virtual(sql: ValueRef<'_>) -> bool {
    let ValueRef::Text(bytes) = sql else {
        return false;
    };
    let text: Cow<'_, str> = String::from_utf8_lossy(bytes);

    trigger::word_after_create(&text).is_some_and(|word| word.eq_ignore_ascii_case(VIRTUAL))
}

/// The storage class of `value`, as SQLite names it.
const fn class(value: ValueRef<'_>) -> &'static str {
    match value {
        ValueRef::Null => "NULL",
        ValueRef::Integer(_) => "INTEGER",
        ValueRef::Real(_) => "REAL",
        ValueRef::Text(_) => "TEXT",
        ValueRef::Blob(_) => "BLOB",
    }
}

/// `value` as a value of the file: a text or a BLOB as its bytes read as text
/// with U+FFFD substituted, an integer or a real in digits, `NULL` as `NULL`.
fn written(value: ValueRef<'_>) -> FileValue {
    FileValue::new(match value {
        ValueRef::Null => "NULL".to_owned(),
        ValueRef::Integer(number) => number.to_string(),
        ValueRef::Real(number) => number.to_string(),
        ValueRef::Text(bytes) | ValueRef::Blob(bytes) => {
            String::from_utf8_lossy(bytes).into_owned()
        }
    })
}

#[cfg(test)]
mod tests {
    use rusqlite::types::ValueRef;

    use super::verdict;
    use crate::error::RefusedKind;

    const TABLE: ValueRef<'static> = ValueRef::Text(b"table");
    const VIEW: ValueRef<'static> = ValueRef::Text(b"view");
    const INDEX: ValueRef<'static> = ValueRef::Text(b"index");
    const TRIGGER: ValueRef<'static> = ValueRef::Text(b"trigger");

    /// A row of `kind` named `name`, on `name`, at root page `page`, created
    /// by `sql`.
    fn row<'v>(
        kind: ValueRef<'v>,
        name: ValueRef<'v>,
        page: ValueRef<'v>,
        sql: ValueRef<'v>,
    ) -> [ValueRef<'v>; 5] {
        [kind, name, name, page, sql]
    }

    fn kind_of(values: [ValueRef<'_>; 5]) -> Option<RefusedKind> {
        verdict(values).map(|refused| refused.kind)
    }

    #[test]
    fn fr_lite_041_a_table_or_a_view_named_pragma_is_refused_in_every_case_and_class() {
        for name in [
            &b"pragma_table_list"[..],
            b"PRAGMA_TABLE_XINFO",
            b"Pragma_Encoding",
            b"pragma_",
            b"pragma_settings",
        ] {
            for kind in [
                TABLE,
                VIEW,
                ValueRef::Blob(b"table"),
                ValueRef::Blob(b"view"),
            ] {
                for stored in [ValueRef::Text(name), ValueRef::Blob(name)] {
                    let sql = ValueRef::Text(b"CREATE VIEW x AS SELECT 1");
                    assert_eq!(
                        kind_of(row(kind, stored, ValueRef::Integer(0), sql)),
                        Some(RefusedKind::PragmaName),
                        "{kind:?} {stored:?}"
                    );
                }
            }
        }
    }

    #[test]
    fn fr_lite_041_an_index_or_a_trigger_named_pragma_and_names_near_the_prefix_pass() {
        let sql = ValueRef::Text(b"CREATE INDEX pragma_table_list ON t(a)");
        assert_eq!(
            kind_of(row(
                INDEX,
                ValueRef::Text(b"pragma_table_list"),
                ValueRef::Integer(3),
                sql
            )),
            None
        );
        let sql = ValueRef::Text(b"CREATE TRIGGER pragma_index_list AFTER INSERT ON t BEGIN END");
        assert_eq!(
            kind_of(row(
                TRIGGER,
                ValueRef::Text(b"pragma_index_list"),
                ValueRef::Integer(0),
                sql
            )),
            None
        );
        // The comparison is on the ASCII letters of the prefix alone: a
        // letter outside ASCII in its place is not folded to one.
        let sql = ValueRef::Text(b"CREATE VIEW x AS SELECT 1");
        for name in [
            "pr\u{e1}gma_table_list",
            "PR\u{c1}GMA_TABLE_LIST",
            "pragma",
            "pragmatable",
            "xpragma_table_list",
            "pragma table",
        ] {
            assert_eq!(
                kind_of(row(
                    VIEW,
                    ValueRef::Text(name.as_bytes()),
                    ValueRef::Integer(0),
                    sql
                )),
                None,
                "{name}"
            );
        }
    }

    #[test]
    fn fr_lite_041_each_excluded_storage_class_in_each_column_is_refused() {
        let good = [
            TABLE,
            ValueRef::Text(b"carrier"),
            ValueRef::Text(b"carrier"),
            ValueRef::Integer(2),
            ValueRef::Text(b"CREATE TABLE carrier(a)"),
        ];
        assert_eq!(kind_of(good), None);

        let excluded: [(usize, &str, &[ValueRef<'static>]); 5] = [
            (
                0,
                "type",
                &[
                    ValueRef::Blob(b"table"),
                    ValueRef::Null,
                    ValueRef::Integer(1),
                    ValueRef::Real(1.5),
                ],
            ),
            (
                1,
                "name",
                &[
                    ValueRef::Blob(b"carrier"),
                    ValueRef::Null,
                    ValueRef::Integer(7),
                    ValueRef::Real(0.5),
                ],
            ),
            (
                2,
                "tbl_name",
                &[
                    ValueRef::Blob(b"carrier"),
                    ValueRef::Null,
                    ValueRef::Integer(7),
                ],
            ),
            (
                3,
                "rootpage",
                &[
                    ValueRef::Blob(b"2"),
                    ValueRef::Text(b"2"),
                    ValueRef::Real(2.0),
                ],
            ),
            (
                4,
                "sql",
                &[
                    ValueRef::Blob(b"CREATE TABLE carrier(a)"),
                    ValueRef::Integer(1),
                    ValueRef::Real(1.0),
                ],
            ),
        ];
        let class = |value: ValueRef<'_>| match value {
            ValueRef::Null => "NULL",
            ValueRef::Integer(_) => "INTEGER",
            ValueRef::Real(_) => "REAL",
            ValueRef::Text(_) => "TEXT",
            ValueRef::Blob(_) => "BLOB",
        };
        for (at, column, values) in excluded {
            for value in values {
                let mut refused = good;
                refused[at] = *value;
                assert_eq!(
                    kind_of(refused),
                    Some(RefusedKind::StorageClass {
                        column,
                        class: class(*value)
                    }),
                    "{column} as {value:?}"
                );
            }
        }

        // The classes the contract admits: a NULL root page and a NULL sql.
        let mut admitted = good;
        admitted[3] = ValueRef::Null;
        admitted[4] = ValueRef::Null;
        assert_eq!(kind_of(admitted), None);
    }

    #[test]
    fn fr_lite_041_a_root_page_that_contradicts_the_text_is_refused_in_each_direction() {
        let name = ValueRef::Text(b"cargo_note");
        let ordinary = ValueRef::Text(b"CREATE TABLE cargo_note(body)");
        let virtual_table = ValueRef::Text(b"CREATE VIRTUAL TABLE cargo_note USING fts5(body)");

        assert_eq!(
            kind_of(row(TABLE, name, ValueRef::Integer(0), ordinary)),
            Some(RefusedKind::RootPage {
                rootpage: Some(0),
                creates_virtual: false
            })
        );
        assert_eq!(
            kind_of(row(TABLE, name, ValueRef::Integer(2), virtual_table)),
            Some(RefusedKind::RootPage {
                rootpage: Some(2),
                creates_virtual: true
            })
        );
        assert_eq!(
            kind_of(row(TABLE, name, ValueRef::Null, virtual_table)),
            Some(RefusedKind::RootPage {
                rootpage: None,
                creates_virtual: true
            })
        );
        // A NULL text creates nothing, so a root page of 0 contradicts it.
        assert_eq!(
            kind_of(row(TABLE, name, ValueRef::Integer(0), ValueRef::Null)),
            Some(RefusedKind::RootPage {
                rootpage: Some(0),
                creates_virtual: false
            })
        );

        // What agrees passes: an ordinary table on its page, a virtual table
        // at 0 whatever comment stands between the two words, a table named
        // "virtual", and a view or a trigger at 0.
        for (kind, page, sql) in [
            (TABLE, 2, &b"CREATE TABLE cargo_note(body)"[..]),
            (
                TABLE,
                0,
                b"CREATE VIRTUAL TABLE cargo_note USING fts5(body)",
            ),
            (
                TABLE,
                0,
                b"CREATE /* note */ VIRTUAL TABLE cargo_note USING fts5(body)",
            ),
            (
                TABLE,
                0,
                b"CREATE -- note\nVIRTUAL TABLE cargo_note USING fts5(body)",
            ),
            (
                TABLE,
                0,
                b"create virtual table cargo_note using fts5(body)",
            ),
            (TABLE, 5, b"CREATE TABLE \"virtual\"(body)"),
            (VIEW, 0, b"CREATE VIEW v AS SELECT 1"),
            (TRIGGER, 0, b"CREATE TRIGGER t AFTER INSERT ON a BEGIN END"),
            (INDEX, 4, b"CREATE INDEX i ON a(b)"),
        ] {
            assert_eq!(
                kind_of(row(
                    kind,
                    name,
                    ValueRef::Integer(page),
                    ValueRef::Text(sql)
                )),
                None,
                "{}",
                String::from_utf8_lossy(sql)
            );
        }
    }

    #[test]
    fn fr_lite_041_the_refused_row_names_its_type_and_name_as_values_of_the_file() {
        let refused = verdict([
            ValueRef::Text(b"view"),
            ValueRef::Blob(b"pragma_\xffx"),
            ValueRef::Text(b"t"),
            ValueRef::Integer(0),
            ValueRef::Text(b"CREATE VIEW v AS SELECT 1"),
        ])
        .expect("refused");
        assert_eq!(refused.row_type.as_str(), "view");
        assert_eq!(refused.name.as_str(), "pragma_\u{fffd}x");

        let refused = verdict([
            ValueRef::Real(1.5),
            ValueRef::Integer(42),
            ValueRef::Text(b"t"),
            ValueRef::Integer(0),
            ValueRef::Null,
        ])
        .expect("refused");
        assert_eq!(refused.row_type.as_str(), "1.5");
        assert_eq!(refused.name.as_str(), "42");
    }
}
