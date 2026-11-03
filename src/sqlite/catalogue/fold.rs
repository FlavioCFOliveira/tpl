//! The values a `sqlite` read owns, folded into the model (`FR-LITE-019` …
//! `FR-LITE-034`, `BR-LITE-002`).
//!
//! Coverage is applied before any object is folded, and every table is built
//! through the one constructor a table has, so `FR-CAT-044` is checked here as
//! on every other producer.
//!
//! | Fact | Rule | Forced by |
//! |---|---|---|
//! | The covered set | `main`; `table` and `view` as `pragma_table_list` reports them; internal tables excluded by the `sqlite_` prefix, ignoring ASCII case | `FR-LITE-019` … `FR-LITE-021` |
//! | `data_type` | The five affinity rules, in order, over the declared type; `blob` for a `STRICT` column declared `ANY` | `FR-LITE-026` |
//! | A column default | The eight ordered rows over `dflt_value` and the column's nullability | `FR-LITE-027` |
//! | The rowid alias | Not `WITHOUT ROWID`, one `pk` column, and no index of origin `pk` | `FR-LITE-028` |
//! | The primary key | The `pk` columns in order of `pk`, named after the index of origin `pk` or unnamed | `FR-LITE-029` |
//! | The indexes | Those `pragma_index_list` reports, less an index with an expression member | `FR-LITE-030`, `FR-LITE-021` |
//! | The foreign keys | One per `id`, unnamed; `to` `NULL` read as the parent's primary key; a key whose parent is not columns of a covered table excluded | `FR-LITE-031`, `FR-LITE-021` |
//! | The triggers | The rows of type `trigger` on a covered table, their timing and event from the header | `FR-LITE-032` |
//! | The views | Their name and their whole stored statement; a view that does not compile marked on `columns` | `FR-LITE-033`, `FR-LITE-035` |
//!
//! **Names are matched as SQLite matches them.** SQLite compares identifiers
//! ignoring ASCII case, and a foreign key or a trigger names its table as its
//! author wrote it. Each reference is therefore resolved against the covered
//! tables ignoring ASCII case and carried under the name the table itself
//! carries, so that the byte-wise lookups of the document find it
//! (`NFR-DET-002`).

use std::borrow::Cow;

use super::trigger::{self, Event, Timing};
use super::{Catalogue, ColumnRow, IndexRow, KeyRow, Object};
use crate::error::Error;
use crate::model::column::{Column, Generated, GeneratedStorage};
use crate::model::column_default::ColumnDefault;
use crate::model::column_type::ColumnType;
use crate::model::database::{Database, Provider};
use crate::model::document::order;
use crate::model::foreign_key::{
    ForeignKey, ForeignKeyColumn, IncomingForeignKey, ReferentialAction,
};
use crate::model::index::{Index, IndexColumn, PrimaryKey, SortDirection};
use crate::model::restricted::Restricted;
use crate::model::table::{Table, TableParts, TableType};
use crate::model::trigger::{Trigger, TriggerEvent, TriggerTiming};
use crate::model::view::View;
use crate::sqlite::fault::{self, Place, Stage};

/// The prefix of an internal table, compared ignoring ASCII case
/// (`FR-LITE-021`).
const INTERNAL: &str = "sqlite_";

/// The type `pragma_table_list` gives an ordinary table.
const TABLE: &str = "table";

/// The type it gives a view.
const VIEW: &str = "view";

/// The type `sqlite_schema` gives a trigger.
const TRIGGER: &str = "trigger";

/// The origin `pragma_index_list` gives the index of a primary key.
const PRIMARY_ORIGIN: &str = "pk";

/// The column rank `pragma_index_xinfo` gives an expression member.
const EXPRESSION: i64 = -2;

/// The `hidden` value of a virtual generated column.
const VIRTUAL_GENERATED: i64 = 2;

/// The `hidden` value of a stored generated column.
const STORED_GENERATED: i64 = 3;

/// The property of a view that could not be read (`FR-LITE-035`).
const COLUMNS: &str = "columns";

/// The invariant a trigger header that yields no timing and event breaks.
const HEADER: &str =
    "the header of every trigger the library accepted yields one timing and one event";

/// The invariant a key naming a column its table does not carry breaks.
const KEY_COLUMN: &str = "every key a sqlite read folds names columns of its own table";

/// The stage a fold failure is reported at.
const FOLD: Stage = Stage::Catalogue("the fold of the catalogue the statements returned");

/// Folds the catalogue into the model.
///
/// # Errors
///
/// Returns the `70` of `FR-LITE-032` and of `FR-CAT-044`.
pub(super) fn database(catalogue: &Catalogue) -> Result<Database<'static>, Error> {
    let place = Place {
        entry: &catalogue.entry,
        path: &catalogue.path,
    };

    // FR-LITE-019 … FR-LITE-021: coverage, before anything is folded.
    let tables: Vec<&Object> = catalogue
        .objects
        .iter()
        .filter(|object| object.kind == TABLE && !internal(&object.name))
        .collect();
    let names: Vec<&str> = tables.iter().map(|table| table.name.as_str()).collect();

    let mut parts: Vec<TableParts<'static>> = Vec::with_capacity(tables.len());
    for table in &tables {
        parts.push(self::table(catalogue, table));
    }

    // FR-LITE-031: the keys, once every table's columns and key are known.
    let mut incoming: Vec<(usize, IncomingForeignKey<'static>)> = Vec::new();
    for (at, table) in tables.iter().enumerate() {
        for key in keys(catalogue, &table.name, &names, &parts) {
            if let Some(referenced) = key
                .referenced_table
                .as_deref()
                .and_then(|referenced| position(&names, referenced))
            {
                incoming.push((
                    referenced,
                    IncomingForeignKey {
                        table: Cow::Owned(table.name.clone()),
                        key: key.clone(),
                    },
                ));
            }
            if let Some(part) = parts.get_mut(at) {
                part.foreign_keys.push(key);
            }
        }
    }
    for (at, key) in incoming {
        if let Some(part) = parts.get_mut(at) {
            part.referenced_by.push(key);
        }
    }

    // FR-LITE-032: the triggers on covered tables.
    for row in catalogue.stored.iter().filter(|row| row.kind == TRIGGER) {
        let Some(at) = position(&names, &row.table) else {
            // A trigger on a view, or on an object the model does not cover.
            continue;
        };
        let sql = row.sql.clone().unwrap_or_default();
        let header =
            trigger::header(&sql).ok_or_else(|| fault::defect(place, FOLD, HEADER, None))?;
        if let Some(part) = parts.get_mut(at) {
            part.triggers.push(Trigger {
                name: Cow::Owned(row.name.clone()),
                event: match header.event {
                    Event::Delete => TriggerEvent::Delete,
                    Event::Insert => TriggerEvent::Insert,
                    Event::Update => TriggerEvent::Update,
                },
                timing: match header.timing {
                    Timing::Before => TriggerTiming::Before,
                    Timing::After => TriggerTiming::After,
                },
                action_order: None,
                statement: Some(Cow::Owned(sql)),
                orientation: Cow::Borrowed("ROW"),
                old_row_alias: Cow::Borrowed("OLD"),
                new_row_alias: Cow::Borrowed("NEW"),
                sql_mode: None,
                definer: None,
                character_set_client: None,
                collation_connection: None,
                database_collation: None,
            });
        }
    }

    let mut assembled = Vec::with_capacity(parts.len());
    for mut part in parts {
        order::sort_by_name(&mut part.indexes);
        order::sort_by_name(&mut part.triggers);
        let name = part.name.clone();
        part.foreign_keys.sort_by(|left, right| {
            order::compare_by_content(
                (&name, &left.columns, left.referenced_table.as_deref()),
                (&name, &right.columns, right.referenced_table.as_deref()),
            )
        });
        part.referenced_by.sort_by(|left, right| {
            order::compare_by_content(
                (&left.table, &left.key.columns, Some(&name)),
                (&right.table, &right.key.columns, Some(&name)),
            )
        });
        assembled
            .push(Table::assemble(part).map_err(|_| fault::defect(place, FOLD, KEY_COLUMN, None))?);
    }
    order::sort_by_name(&mut assembled);

    Ok(Database {
        // FR-LITE-019, FR-LITE-023.
        name: Cow::Borrowed("main"),
        charset: Cow::Owned(catalogue.encoding.clone()),
        collation: None,
        provider: Provider::Sqlite,
        server: catalogue.server.clone(),
        tables: assembled,
        views: views(catalogue),
        // FR-LITE-022: SQLite has no stored routines.
        routines: Vec::new(),
    })
}

/// Whether `name` is an internal table's (`FR-LITE-021`).
fn internal(name: &str) -> bool {
    name.get(..INTERNAL.len())
        .is_some_and(|prefix| prefix.eq_ignore_ascii_case(INTERNAL))
}

/// The position of the covered table `name` names, ignoring ASCII case.
fn position(names: &[&str], name: &str) -> Option<usize> {
    names
        .iter()
        .position(|carried| *carried == name)
        .or_else(|| {
            names
                .iter()
                .position(|carried| carried.eq_ignore_ascii_case(name))
        })
}

/// One table's parts: its columns, its primary key and its indexes
/// (`FR-LITE-024` … `FR-LITE-030`).
fn table(catalogue: &Catalogue, object: &Object) -> TableParts<'static> {
    let rows: Vec<&ColumnRow> = catalogue
        .columns
        .iter()
        .filter(|row| row.table == object.name)
        .collect();
    let index_rows: Vec<&IndexRow> = catalogue
        .indexes
        .iter()
        .filter(|row| row.table == object.name)
        .collect();

    // FR-LITE-029: the primary key's columns, in ascending order of `pk`.
    let mut key_rows: Vec<&ColumnRow> = rows.iter().copied().filter(|row| row.pk > 0).collect();
    key_rows.sort_by_key(|row| row.pk);
    let origin = index_rows
        .iter()
        .find(|row| row.origin == PRIMARY_ORIGIN)
        .map(|row| row.index.as_str());

    // FR-LITE-028: the rowid alias.
    let alias = (!object.without_rowid && key_rows.len() == 1 && origin.is_none())
        .then(|| key_rows.first().map(|row| row.name.as_str()))
        .flatten();

    let columns: Vec<Column<'static>> = rows
        .iter()
        .map(|row| {
            column(
                row,
                &object.name,
                object.strict,
                alias == Some(row.name.as_str()),
            )
        })
        .collect();

    let primary_key = (!key_rows.is_empty()).then(|| PrimaryKey {
        name: origin.map(|name| Cow::Owned(name.to_owned())),
        unique: origin.is_none_or(|name| {
            index_rows
                .iter()
                .find(|row| row.index == name)
                .is_none_or(|row| row.unique)
        }),
        columns: key_rows
            .iter()
            .map(|row| IndexColumn {
                name: Cow::Owned(row.name.clone()),
                direction: Some(match origin {
                    Some(name) => direction_of(&index_rows, name, &row.name),
                    // A rowid alias is the table's B-tree key, ascending.
                    None => SortDirection::Ascending,
                }),
                prefix_length: None,
            })
            .collect(),
        index_type: None,
        comment: Cow::Borrowed(""),
        ignored: false,
    });

    TableParts {
        engine: None,
        collation: None,
        columns,
        indexes: indexes(&index_rows),
        primary_key,
        ..TableParts::new(Cow::Owned(object.name.clone()), TableType::Base)
    }
}

/// The direction of the member `column` of the index `index`.
fn direction_of(rows: &[&IndexRow], index: &str, column: &str) -> SortDirection {
    let descending = rows
        .iter()
        .find(|row| row.index == index && row.key && row.name.as_deref() == Some(column))
        .is_some_and(|row| row.descending);

    if descending {
        SortDirection::Descending
    } else {
        SortDirection::Ascending
    }
}

/// The indexes of one table, less those with an expression member
/// (`FR-LITE-030`, `FR-LITE-021`).
fn indexes(rows: &[&IndexRow]) -> Vec<Index<'static>> {
    let mut names: Vec<&str> = Vec::new();
    for row in rows {
        if !names.contains(&row.index.as_str()) {
            names.push(&row.index);
        }
    }

    names
        .into_iter()
        .filter_map(|name| {
            let mut members: Vec<&&IndexRow> = rows
                .iter()
                .filter(|row| row.index == name && row.key)
                .collect();
            if members
                .iter()
                .any(|row| row.cid == EXPRESSION || row.name.is_none())
            {
                return None;
            }
            members.sort_by_key(|row| row.seqno);
            let unique = members.first().is_some_and(|row| row.unique);

            Some(Index {
                name: Cow::Owned(name.to_owned()),
                unique,
                columns: members
                    .iter()
                    .map(|row| IndexColumn {
                        name: Cow::Owned(row.name.clone().unwrap_or_default()),
                        direction: Some(if row.descending {
                            SortDirection::Descending
                        } else {
                            SortDirection::Ascending
                        }),
                        prefix_length: None,
                    })
                    .collect(),
                index_type: None,
                comment: Cow::Borrowed(""),
                ignored: false,
            })
        })
        .collect()
}

/// One column of a table or a view (`FR-LITE-025` … `FR-LITE-028`).
fn column(row: &ColumnRow, table: &str, strict: bool, alias: bool) -> Column<'static> {
    let nullable = !row.not_null;

    Column {
        name: Cow::Owned(row.name.clone()),
        table_name: Cow::Owned(table.to_owned()),
        position: u64::try_from(row.cid).unwrap_or_default() + 1,
        column_type: ColumnType::affinity(
            Cow::Owned(row.declared.clone()),
            affinity(&row.declared, strict),
        ),
        nullable,
        default: default(row.default.as_deref(), nullable),
        comment: Cow::Borrowed(""),
        auto_increment: alias,
        invisible: false,
        generated: match row.hidden {
            VIRTUAL_GENERATED => Some(Generated {
                expression: None,
                storage: GeneratedStorage::Virtual,
            }),
            STORED_GENERATED => Some(Generated {
                expression: None,
                storage: GeneratedStorage::Stored,
            }),
            _ => None,
        },
        on_update: None,
    }
}

/// The type affinity of a declared type, by the five rules of section 3.1 of
/// `https://sqlite.org/datatype3.html`, in their order (`FR-LITE-026`).
pub(super) fn affinity(declared: &str, strict: bool) -> &'static str {
    let upper = declared.to_ascii_uppercase();

    if strict && upper == "ANY" {
        return "blob";
    }
    if upper.contains("INT") {
        "integer"
    } else if upper.contains("CHAR") || upper.contains("CLOB") || upper.contains("TEXT") {
        "text"
    } else if upper.contains("BLOB") || upper.is_empty() {
        "blob"
    } else if upper.contains("REAL") || upper.contains("FLOA") || upper.contains("DOUB") {
        "real"
    } else {
        "numeric"
    }
}

/// The default of a column, classified by the first row of `FR-LITE-027`
/// that matches.
pub(super) fn default(written: Option<&str>, nullable: bool) -> Option<ColumnDefault<'static>> {
    let Some(written) = written else {
        // The first two rows: SQL NULL is no default for a column that is not
        // nullable, and the default NULL for one that is.
        return nullable.then_some(ColumnDefault::Null);
    };

    if written.eq_ignore_ascii_case("NULL") {
        return Some(ColumnDefault::Null);
    }
    if let Some(inner) = quoted(written) {
        return Some(ColumnDefault::Literal(Cow::Owned(
            crate::model::collapse_doubled_apostrophes(inner).into_owned(),
        )));
    }
    if number(written)
        || blob(written)
        || written.eq_ignore_ascii_case("TRUE")
        || written.eq_ignore_ascii_case("FALSE")
    {
        return Some(ColumnDefault::Literal(Cow::Owned(written.to_owned())));
    }

    Some(ColumnDefault::Expression(Cow::Owned(written.to_owned())))
}

/// The text between the quotes of a single quoted string, `'([^']|'')*'`.
fn quoted(written: &str) -> Option<&str> {
    let inner = written.strip_prefix('\'')?.strip_suffix('\'')?;
    let mut characters = inner.chars().peekable();
    while let Some(character) = characters.next() {
        if character == '\'' {
            // An apostrophe inside is admitted only doubled.
            if characters.next() != Some('\'') {
                return None;
            }
        }
    }
    Some(inner)
}

/// Whether `written` is a number of `FR-LITE-027`:
/// `[+-]?([0-9]+(\.[0-9]*)?|\.[0-9]+)([eE][+-]?[0-9]+)?` or
/// `[+-]?0[xX][0-9A-Fa-f]+`.
fn number(written: &str) -> bool {
    let unsigned = written.strip_prefix(['+', '-']).unwrap_or(written);

    if let Some(hex) = unsigned
        .strip_prefix("0x")
        .or_else(|| unsigned.strip_prefix("0X"))
    {
        return !hex.is_empty() && hex.bytes().all(|byte| byte.is_ascii_hexdigit());
    }

    let (mantissa, exponent) = match unsigned.find(['e', 'E']) {
        Some(at) => (&unsigned[..at], Some(&unsigned[at + 1..])),
        None => (unsigned, None),
    };
    let digits = |text: &str| !text.is_empty() && text.bytes().all(|byte| byte.is_ascii_digit());
    let mantissa_ok = match mantissa.split_once('.') {
        Some((whole, fraction)) => {
            (digits(whole) && (fraction.is_empty() || digits(fraction)))
                || (whole.is_empty() && digits(fraction))
        }
        None => digits(mantissa),
    };
    let exponent_ok = exponent
        .is_none_or(|exponent| digits(exponent.strip_prefix(['+', '-']).unwrap_or(exponent)));

    mantissa_ok && exponent_ok
}

/// Whether `written` is a BLOB literal, `[xX]'[0-9A-Fa-f]*'`.
fn blob(written: &str) -> bool {
    written
        .strip_prefix(['x', 'X'])
        .and_then(|rest| rest.strip_prefix('\''))
        .and_then(|rest| rest.strip_suffix('\''))
        .is_some_and(|hex| hex.bytes().all(|byte| byte.is_ascii_hexdigit()))
}

/// The outgoing keys of `table`, less those `FR-LITE-021` excludes
/// (`FR-LITE-031`).
fn keys(
    catalogue: &Catalogue,
    table: &str,
    names: &[&str],
    parts: &[TableParts<'static>],
) -> Vec<ForeignKey<'static>> {
    let rows: Vec<&KeyRow> = catalogue
        .keys
        .iter()
        .filter(|row| row.table == table)
        .collect();
    let mut ids: Vec<i64> = rows.iter().map(|row| row.id).collect();
    ids.sort_unstable();
    ids.dedup();

    let own = position(names, table).and_then(|at| parts.get(at));

    ids.into_iter()
        .filter_map(|id| {
            let mut pairs: Vec<&&KeyRow> = rows.iter().filter(|row| row.id == id).collect();
            pairs.sort_by_key(|row| row.seq);
            let first = pairs.first()?;

            // FR-LITE-021: the parent must be a covered table.
            let parent_at = position(names, &first.referenced)?;
            let parent = parts.get(parent_at)?;
            let parent_name = names.get(parent_at)?;

            // `to` NULL on every pair: the parent's primary key, in its order.
            let referenced: Vec<String> = if pairs.iter().all(|row| row.to.is_none()) {
                let key = parent.primary_key.as_ref()?;
                key.columns
                    .iter()
                    .map(|member| member.name.to_string())
                    .collect()
            } else {
                pairs
                    .iter()
                    .map(|row| row.to.as_deref().and_then(|to| carried(parent, to)))
                    .collect::<Option<Vec<String>>>()?
            };
            if referenced.len() != pairs.len() {
                return None;
            }

            let columns = pairs
                .iter()
                .zip(referenced)
                .map(|(row, referenced)| {
                    let column = own
                        .and_then(|own| carried(own, &row.from))
                        .unwrap_or_else(|| row.from.clone());
                    Some(ForeignKeyColumn {
                        column: Cow::Owned(column),
                        referenced_column: Cow::Owned(referenced),
                    })
                })
                .collect::<Option<Vec<_>>>()?;

            Some(ForeignKey {
                name: None,
                columns,
                referenced_table: Some(Cow::Owned((*parent_name).to_owned())),
                referenced_key: None,
                match_option: Cow::Owned(first.matched.clone()),
                on_update: ReferentialAction::from_catalogue_owned(first.on_update.clone()),
                on_delete: ReferentialAction::from_catalogue_owned(first.on_delete.clone()),
            })
        })
        .collect()
}

/// The name a column of `table` carries, matched as SQLite matches it.
fn carried(table: &TableParts<'_>, column: &str) -> Option<String> {
    table
        .columns
        .iter()
        .find(|carried| carried.name == column)
        .or_else(|| {
            table
                .columns
                .iter()
                .find(|carried| carried.name.eq_ignore_ascii_case(column))
        })
        .map(|carried| carried.name.to_string())
}

/// The views, with their stored statement, marked where their columns could
/// not be read (`FR-LITE-033`, `FR-LITE-035`).
fn views(catalogue: &Catalogue) -> Vec<View<'static>> {
    let mut views: Vec<View<'static>> = catalogue
        .objects
        .iter()
        .filter(|object| object.kind == VIEW)
        .map(|object| {
            let definition = catalogue
                .stored
                .iter()
                .find(|row| row.kind == VIEW && row.name == object.name)
                .and_then(|row| row.sql.clone())
                .unwrap_or_default();
            let marked = catalogue
                .unreadable
                .iter()
                .any(|unreadable| unreadable.name == object.name);

            View {
                name: Cow::Owned(object.name.clone()),
                definition: Cow::Owned(definition),
                check_option: None,
                is_updatable: None,
                definer: None,
                security_type: None,
                character_set_client: None,
                collation_connection: None,
                algorithm: None,
                restricted: if marked {
                    Restricted::new(vec![Cow::Borrowed(COLUMNS)])
                } else {
                    None
                },
            }
        })
        .collect();

    order::sort_by_name(&mut views);
    views
}

#[cfg(test)]
mod tests {
    use super::{affinity, default, internal, number};
    use crate::model::column_default::ColumnDefault;
    use std::borrow::Cow;

    #[test]
    fn fr_lite_026_the_five_rules_apply_in_their_order() {
        // FR-LITE-026 and section 3.1 of datatype3: the first rule that
        // matches decides, so `CHARINT` is integer and `FLOATING POINT`
        // integer too ("POINT" holds "INT").
        for (declared, expected) in [
            ("INTEGER", "integer"),
            ("BIGINT", "integer"),
            ("CHARINT", "integer"),
            ("FLOATING POINT", "integer"),
            ("VARCHAR(80)", "text"),
            ("NCHAR(55)", "text"),
            ("CLOB", "text"),
            ("text", "text"),
            ("BLOB", "blob"),
            ("", "blob"),
            ("REAL", "real"),
            ("DOUBLE PRECISION", "real"),
            ("FLOAT", "real"),
            ("DECIMAL(12,3)", "numeric"),
            ("NUMERIC(5,2)", "numeric"),
            ("BOOLEAN", "numeric"),
            ("DATETIME", "numeric"),
            ("STRING", "numeric"),
            ("ANY", "numeric"),
        ] {
            assert_eq!(affinity(declared, false), expected, "{declared}");
        }
        // A STRICT column declared ANY stores a value as received.
        assert_eq!(affinity("ANY", true), "blob");
        assert_eq!(affinity("INTEGER", true), "integer");
    }

    #[test]
    fn fr_lite_027_every_row_of_the_default_classification() {
        let literal = |value: &'static str| Some(ColumnDefault::Literal(Cow::Borrowed(value)));
        let expression =
            |value: &'static str| Some(ColumnDefault::Expression(Cow::Borrowed(value)));

        assert_eq!(default(None, false), None);
        assert_eq!(default(None, true), Some(ColumnDefault::Null));
        assert_eq!(default(Some("NULL"), false), Some(ColumnDefault::Null));
        assert_eq!(default(Some("null"), true), Some(ColumnDefault::Null));
        assert_eq!(default(Some("'unnamed'"), true), literal("unnamed"));
        assert_eq!(default(Some("''"), true), literal(""));
        assert_eq!(
            default(Some("'Lloyd''s \"A1\"'"), true),
            literal("Lloyd's \"A1\"")
        );
        for number in ["0.0", "-1.5", "+5", "1e3", "0x1F", ".5", "7.", "2E-3"] {
            assert_eq!(default(Some(number), true), literal(number), "{number}");
        }
        assert_eq!(default(Some("x'00ff'"), true), literal("x'00ff'"));
        assert_eq!(default(Some("X'AB'"), true), literal("X'AB'"));
        assert_eq!(default(Some("TRUE"), true), literal("TRUE"));
        assert_eq!(default(Some("false"), true), literal("false"));
        for other in [
            "CURRENT_TIMESTAMP",
            "current_time",
            "date('now')",
            "1+2",
            "\"dq\"",
            "lower( 'X' )",
            "'a' || 'b'",
            "0x",
            "1e",
        ] {
            assert_eq!(default(Some(other), true), expression(other), "{other}");
        }
    }

    #[test]
    fn fr_lite_027_the_number_shapes_are_exactly_the_two_patterns() {
        assert!(number("12"));
        assert!(number("-0x0a"));
        assert!(!number("."));
        assert!(!number("1.2.3"));
        assert!(!number("+"));
        assert!(!number("1e+"));
    }

    #[test]
    fn fr_lite_021_an_internal_table_is_recognised_by_its_prefix_in_any_case() {
        for name in [
            "sqlite_sequence",
            "sqlite_stat1",
            "SQLITE_upper",
            "Sqlite_mixed",
        ] {
            assert!(internal(name), "{name}");
        }
        for name in ["sqlit", "my_sqlite_table", "sqlitex"] {
            assert!(!internal(name), "{name}");
        }
    }
}
