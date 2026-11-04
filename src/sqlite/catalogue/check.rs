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
//! | A type SQLite does not write | A `type` that is not exactly one of the four texts `table`, `index`, `view` and `trigger`, compared on its whole value — a NUL character included — with no letter case folded |
//! | A row with no text that is not an index's | An `sql` that is `NULL` or the empty text on a row whose `type` is not exactly `index` |
//! | A NUL character in a name or a text | A `name`, `tbl_name` or `sql` holding a NUL character anywhere in its value |
//! | A trigger's header SQLite does not write | A row of type `trigger` whose `sql`, under SQLite's comment syntax and ignoring ASCII letter case, does not begin with the words `CREATE` and `TRIGGER`, or whose next word is an unquoted `IF` |
//!
//! A row refused under more than one kind is reported under the first, in the
//! order of that table. An integer or a real is written in digits and cannot
//! begin with `pragma_`; a `NULL` root page, which the library refuses when it
//! loads the schema, is not `0`.
//!
//! **Every comparison of `type` the reader makes is exact**, here and in the
//! statements and the fold after the check: the fourth kind refuses every
//! other spelling before any of them runs, so an exact comparison misses no
//! row of a file the check admits (`FR-LITE-041`, `FR-LITE-020`).

use std::borrow::Cow;

use rusqlite::types::ValueRef;

use super::trigger;
use crate::error::{FileValue, RefusedKind, RefusedRow};

/// The prefix SQLite gives the name of every pragma table-valued function.
const PRAGMA: &[u8] = b"pragma_";

/// The keyword whose place after `CREATE` makes a text create a virtual table.
const VIRTUAL: &str = "VIRTUAL";

/// The four texts SQLite writes in the `type` column of the schema table, and
/// no other (`https://sqlite.org/schematab.html`).
const TYPES: [&[u8]; 4] = [b"table", b"index", b"view", b"trigger"];

/// The NUL character, which the library reads a value of the schema table
/// only up to (`FR-LITE-041`, its sixth kind).
const NUL: u8 = 0;

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
    let [kind, name, tbl_name, rootpage, sql] = values;
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

    // The second kind admitted only a text `type`, so `kind` is one here.
    // The fourth kind: a type SQLite does not write, compared on the whole
    // value — `table` followed by a NUL character is not `table` — and in its
    // case.
    if let ValueRef::Text(bytes) = kind
        && !TYPES.contains(&bytes)
    {
        return refused(RefusedKind::Type);
    }

    // The fifth kind: a row with no text, which the library reads as an
    // index's whatever its type, on a row that is not an index's.
    let null = matches!(sql, ValueRef::Null);
    if (null || matches!(sql, ValueRef::Text(b""))) && !matches!(kind, ValueRef::Text(b"index")) {
        return refused(RefusedKind::NoText { null });
    }

    // The sixth kind: a NUL character the library would read a value only up
    // to, in the first of the three columns that holds one.
    for (column, value) in [("name", name), ("tbl_name", tbl_name), ("sql", sql)] {
        if let ValueRef::Text(bytes) = value
            && bytes.contains(&NUL)
        {
            return refused(RefusedKind::Nul { column });
        }
    }

    // The seventh kind: a trigger's header SQLite does not write — a text
    // that does not begin `CREATE TRIGGER`, or whose next word is an unquoted
    // `IF`, as `TEMP`, `TEMPORARY` and `IF NOT EXISTS`, which SQLite removes,
    // would make it.
    if matches!(kind, ValueRef::Text(b"trigger"))
        && let ValueRef::Text(bytes) = sql
        && !trigger::header_written(&String::from_utf8_lossy(bytes))
    {
        return refused(RefusedKind::TriggerHeader);
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

        // The classes the contract admits: a NULL root page, and a NULL sql,
        // which the fifth kind admits on an index's row alone.
        let mut admitted = good;
        admitted[3] = ValueRef::Null;
        assert_eq!(kind_of(admitted), None);
        let autoindex = [
            INDEX,
            ValueRef::Text(b"sqlite_autoindex_carrier_1"),
            ValueRef::Text(b"carrier"),
            ValueRef::Integer(3),
            ValueRef::Null,
        ];
        assert_eq!(kind_of(autoindex), None);
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

    /// A row SQLite writes for each of the four types.
    fn written_by_sqlite() -> [[ValueRef<'static>; 5]; 6] {
        [
            [
                TABLE,
                ValueRef::Text(b"vessel"),
                ValueRef::Text(b"vessel"),
                ValueRef::Integer(2),
                ValueRef::Text(b"CREATE TABLE vessel(imo INTEGER PRIMARY KEY)"),
            ],
            [
                INDEX,
                ValueRef::Text(b"ix_vessel"),
                ValueRef::Text(b"vessel"),
                ValueRef::Integer(3),
                ValueRef::Text(b"CREATE INDEX ix_vessel ON vessel(imo)"),
            ],
            [
                INDEX,
                ValueRef::Text(b"sqlite_autoindex_port_1"),
                ValueRef::Text(b"port"),
                ValueRef::Integer(4),
                ValueRef::Null,
            ],
            [
                VIEW,
                ValueRef::Text(b"v_fleet"),
                ValueRef::Text(b"v_fleet"),
                ValueRef::Integer(0),
                ValueRef::Text(b"CREATE VIEW v_fleet AS SELECT imo FROM vessel"),
            ],
            [
                TRIGGER,
                ValueRef::Text(b"trg_vessel"),
                ValueRef::Text(b"VESSEL"),
                ValueRef::Integer(0),
                ValueRef::Text(
                    b"CREATE TRIGGER trg_vessel after insert on VESSEL BEGIN SELECT 1; END",
                ),
            ],
            [
                TRIGGER,
                ValueRef::Text(b"temp"),
                ValueRef::Text(b"vessel"),
                ValueRef::Integer(0),
                ValueRef::Text(b"CREATE TRIGGER temp AFTER DELETE ON vessel BEGIN SELECT 1; END"),
            ],
        ]
    }

    #[test]
    fn fr_lite_041_rows_sqlite_writes_pass_the_four_new_kinds() {
        for row in written_by_sqlite() {
            assert_eq!(kind_of(row), None, "{row:?}");
        }
    }

    #[test]
    fn fr_lite_041_a_type_sqlite_does_not_write_is_refused_whole_and_in_its_case() {
        // The fourth kind: every spelling the library loads — upper, title
        // and mixed case, and the type followed by a NUL character — and
        // every other text.
        for row in written_by_sqlite() {
            let ValueRef::Text(written) = row[0] else {
                panic!("a type SQLite writes is a text");
            };
            let upper = written.to_ascii_uppercase();
            let mut title = written.to_vec();
            title[0] = title[0].to_ascii_uppercase();
            let mixed: Vec<u8> = written
                .iter()
                .enumerate()
                .map(|(at, byte)| {
                    if at % 2 == 1 {
                        byte.to_ascii_uppercase()
                    } else {
                        *byte
                    }
                })
                .collect();
            let nul = [written, &b"\0"[..]].concat();
            let nul_letter = [written, &b"\0x"[..]].concat();
            for spelling in [&upper, &title, &mixed, &nul, &nul_letter] {
                let mut refused = row;
                refused[0] = ValueRef::Text(spelling);
                assert_eq!(
                    kind_of(refused),
                    Some(RefusedKind::Type),
                    "{}",
                    String::from_utf8_lossy(spelling)
                );
            }
        }
        for other in [&b""[..], b"garbage", b" table", b"tables", b"virtual"] {
            let mut refused = written_by_sqlite()[0];
            refused[0] = ValueRef::Text(other);
            assert_eq!(kind_of(refused), Some(RefusedKind::Type), "{other:?}");
        }

        // The table named like the encoding function, typed `TABLE`, which
        // the first kind alone let forge the encoding at exit 0, is refused
        // by this one.
        assert_eq!(
            kind_of([
                ValueRef::Text(b"TABLE"),
                ValueRef::Text(b"pragma_encoding"),
                ValueRef::Text(b"pragma_encoding"),
                ValueRef::Integer(2),
                ValueRef::Text(b"CREATE TABLE pragma_encoding(encoding)"),
            ]),
            Some(RefusedKind::Type)
        );
    }

    #[test]
    fn fr_lite_041_a_row_with_no_text_is_refused_unless_it_is_an_index_s() {
        // The fifth kind: NULL or the empty text, on every type but `index`.
        for kind in [TABLE, VIEW, TRIGGER] {
            for (sql, null) in [(ValueRef::Null, true), (ValueRef::Text(b""), false)] {
                assert_eq!(
                    kind_of([
                        kind,
                        ValueRef::Text(b"ix_vessel"),
                        ValueRef::Text(b"vessel"),
                        ValueRef::Integer(3),
                        sql,
                    ]),
                    Some(RefusedKind::NoText { null }),
                    "{kind:?} {sql:?}"
                );
            }
        }
        for sql in [ValueRef::Null, ValueRef::Text(b"")] {
            assert_eq!(
                kind_of([
                    INDEX,
                    ValueRef::Text(b"ix_vessel"),
                    ValueRef::Text(b"vessel"),
                    ValueRef::Integer(3),
                    sql,
                ]),
                None,
                "{sql:?}"
            );
        }
    }

    #[test]
    fn fr_lite_041_a_nul_character_in_a_name_or_a_text_is_refused_naming_its_column() {
        // The sixth kind: anywhere in `name`, `tbl_name` or `sql`, reported
        // under the first of the three that holds one.
        for row in written_by_sqlite() {
            for (at, column) in [(1, "name"), (2, "tbl_name"), (4, "sql")] {
                let ValueRef::Text(value) = row[at] else {
                    continue;
                };
                for placed in [
                    [&b"\0"[..], value].concat(),
                    [value, &b"\0"[..]].concat(),
                    [value, &b"\0; DROP TABLE vessel"[..]].concat(),
                ] {
                    let mut refused = row;
                    refused[at] = ValueRef::Text(&placed);
                    assert_eq!(
                        kind_of(refused),
                        Some(RefusedKind::Nul { column }),
                        "{column}: {}",
                        String::from_utf8_lossy(&placed)
                    );
                }
            }
        }
        let mut both = written_by_sqlite()[0];
        both[2] = ValueRef::Text(b"vessel\0");
        both[4] = ValueRef::Text(b"CREATE TABLE vessel(imo)\0x");
        assert_eq!(kind_of(both), Some(RefusedKind::Nul { column: "tbl_name" }));
    }

    #[test]
    fn fr_lite_041_a_trigger_header_sqlite_does_not_write_is_refused() {
        // The seventh kind: TEMP, TEMPORARY and IF NOT EXISTS, which SQLite
        // removes, and any text that does not begin CREATE TRIGGER.
        let trigger = |sql: &'static [u8]| {
            [
                TRIGGER,
                ValueRef::Text(b"trg_vessel"),
                ValueRef::Text(b"vessel"),
                ValueRef::Integer(0),
                ValueRef::Text(sql),
            ]
        };
        for sql in [
            &b"CREATE TEMP TRIGGER trg_vessel AFTER INSERT ON vessel BEGIN SELECT 1; END"[..],
            b"CREATE TEMPORARY TRIGGER trg_vessel AFTER INSERT ON vessel BEGIN SELECT 1; END",
            b"CREATE TRIGGER IF NOT EXISTS trg_vessel AFTER INSERT ON vessel BEGIN SELECT 1; END",
            b"CREATE TRIGGER if not exists trg_vessel AFTER INSERT ON vessel BEGIN END",
            b"CREATE TRIGGER /* x */ If NOT EXISTS trg_vessel AFTER INSERT ON vessel BEGIN END",
            b"CREATE VIEW trg_vessel AS SELECT 1",
            b"SELECT 1",
            b" ",
            b"CREATE",
            b"CREATE \"TRIGGER\" trg_vessel AFTER INSERT ON vessel BEGIN END",
        ] {
            assert_eq!(
                kind_of(trigger(sql)),
                Some(RefusedKind::TriggerHeader),
                "{}",
                String::from_utf8_lossy(sql)
            );
        }
        // Admitted: lower case, a comment inside the header, and names SQLite
        // writes after `CREATE TRIGGER`, `IF` among them when quoted.
        for sql in [
            &b"create trigger trg_vessel after insert on vessel BEGIN SELECT 1; END"[..],
            b"CREATE /* note */ TRIGGER trg_vessel AFTER INSERT ON vessel BEGIN END",
            b"CREATE -- note\nTRIGGER trg_vessel AFTER INSERT ON vessel BEGIN END",
            b"CREATE TRIGGER \"IF\" AFTER INSERT ON vessel BEGIN END",
            b"CREATE TRIGGER [if] AFTER INSERT ON vessel BEGIN END",
            b"CREATE TRIGGER temp AFTER INSERT ON vessel BEGIN END",
            b"CREATE TRIGGER after AFTER INSERT ON vessel BEGIN END",
            b"CREATE TRIGGER iffy AFTER INSERT ON vessel BEGIN END",
        ] {
            assert_eq!(
                kind_of(trigger(sql)),
                None,
                "{}",
                String::from_utf8_lossy(sql)
            );
        }
        // A view or a table whose text begins otherwise is not a trigger's.
        assert_eq!(
            kind_of([
                VIEW,
                ValueRef::Text(b"v"),
                ValueRef::Text(b"v"),
                ValueRef::Integer(0),
                ValueRef::Text(b"CREATE TEMP VIEW v AS SELECT 1"),
            ]),
            None
        );
    }

    #[test]
    fn fr_lite_041_a_row_refused_under_several_kinds_is_reported_under_the_first() {
        // A NUL type with no text: the fourth before the fifth.
        assert_eq!(
            kind_of([
                ValueRef::Text(b"table\0"),
                ValueRef::Text(b"ix\0"),
                ValueRef::Text(b"vessel"),
                ValueRef::Integer(3),
                ValueRef::Null,
            ]),
            Some(RefusedKind::Type)
        );
        // No text and a NUL name: the fifth before the sixth.
        assert_eq!(
            kind_of([
                VIEW,
                ValueRef::Text(b"ix\0"),
                ValueRef::Text(b"vessel"),
                ValueRef::Integer(3),
                ValueRef::Null,
            ]),
            Some(RefusedKind::NoText { null: true })
        );
        // A NUL in a trigger's TEMP header: the sixth before the seventh.
        assert_eq!(
            kind_of([
                TRIGGER,
                ValueRef::Text(b"t"),
                ValueRef::Text(b"vessel"),
                ValueRef::Integer(0),
                ValueRef::Text(b"CREATE TEMP TRIGGER t AFTER INSERT ON vessel BEGIN END\0"),
            ]),
            Some(RefusedKind::Nul { column: "sql" })
        );
    }
}
