//! The values a `sqlite` read owns, folded into the model (`FR-LITE-019` …
//! `FR-LITE-034`, `BR-LITE-002`).
//!
//! Coverage is applied before any object is folded, and every table is built
//! through the one constructor a table has, so `FR-CAT-044` is checked here as
//! on every other producer.
//!
//! | Fact | Rule | Forced by |
//! |---|---|---|
//! | The covered set | `main`; the rows of `main.sqlite_schema` of type `table` whose root page is not `0` — a shadow table among them — and of type `view`; a virtual table, at root page `0`, excluded; internal tables excluded by the `sqlite_` prefix, ignoring ASCII case | `FR-LITE-019` … `FR-LITE-021` |
//! | `data_type` | The five affinity rules, in order, over the declared type, for every column — a `STRICT` column declared `ANY` among them, which takes `numeric` | `FR-LITE-026` |
//! | A column default | The eight ordered rows over `dflt_value` and the column's nullability | `FR-LITE-027` |
//! | The rowid alias | Not `WITHOUT ROWID` — `pragma_index_info` returned no row for the table's name — one `pk` column, and no index of origin `pk` | `FR-LITE-028` |
//! | The primary key | The `pk` columns in order of `pk`, named after the index of origin `pk` or unnamed | `FR-LITE-029` |
//! | The indexes | Those `pragma_index_list` reports, less an index with an expression member | `FR-LITE-030`, `FR-LITE-021` |
//! | The foreign keys | One per `id`, unnamed; `to` `NULL` read as the parent's primary key; a key whose parent is not columns of a covered table — or that names no parent column of a parent whose primary key has another number of columns — excluded | `FR-LITE-031`, `FR-LITE-021` |
//! | The triggers | The rows of type `trigger` on a covered table, their timing and event from the header; a trigger on a view excluded | `FR-LITE-032`, `FR-LITE-021` |
//! | The views | Their name and their whole stored statement, and `null` for the seven server fields; no columns, and no marking | `FR-LITE-033`, `FR-LITE-036` |
//!
//! **Names are matched as SQLite matches them** (`BR-LITE-003`). SQLite writes
//! a trigger's `tbl_name`, and reports a foreign key's parent table and
//! columns, as the statement wrote them, and resolves each by folding the 26
//! ASCII letters. Each of these three names is therefore matched — a trigger's
//! table against the covered tables and the views, a key's parent table
//! against the covered tables, a key's parent columns against that table's
//! columns — after folding the 26 ASCII letters over the whole name, and no
//! other character; and the object found is carried under its own name, so
//! that the byte-wise lookups of the document find it (`NFR-DET-002`,
//! `FR-CTX-023`). An index's members and a key's own columns are not matched:
//! the library reports each as its table declares it.
//!
//! **The fold indexes before it looks up.** Every collection the fold reads
//! by a name — the column, index and key rows by their table, an index's
//! members by their index, the stored rows by their name, the covered tables,
//! the views and a table's columns by their folded name — is grouped
//! once into a map keyed by that name, and no object is found by scanning
//! another collection. The fold runs after the last statement, outside every
//! statement's bound, so a lookup that scanned would let a file of many
//! objects cost time growing with their square where the statements' grows
//! with their number (architecture, *The SQLite reader*; `FR-LITE-009`,
//! *Consequence*; `FR-LITE-020`, *Provenance*). The maps are only looked up,
//! never iterated, so no order of a document depends on them.

use std::borrow::Cow;
use std::collections::{BTreeMap, HashMap};

use super::trigger::{self, Event, Timing};
use super::{Catalogue, ColumnRow, IndexRow, KeyRow, Object, StoredRow};
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
use crate::model::table::{Table, TableParts, TableType};
use crate::model::trigger::{Trigger, TriggerEvent, TriggerTiming};
use crate::model::view::View;
use crate::sqlite::fault::{self, Place, Stage};

/// The prefix of an internal table, compared ignoring ASCII case
/// (`FR-LITE-021`).
const INTERNAL: &str = "sqlite_";

/// The type the schema table gives a table, ordinary, shadow or virtual.
const TABLE: &str = "table";

/// The type it gives a view.
const VIEW: &str = "view";

/// The type it gives a trigger.
const TRIGGER: &str = "trigger";

/// The root page the schema table gives a virtual table (`FR-LITE-020`).
const VIRTUAL_ROOT: i64 = 0;

/// The origin `pragma_index_list` gives the index of a primary key.
const PRIMARY_ORIGIN: &str = "pk";

/// The column rank `pragma_index_xinfo` gives an expression member.
const EXPRESSION: i64 = -2;

/// The `hidden` value of a virtual generated column.
const VIRTUAL_GENERATED: i64 = 2;

/// The `hidden` value of a stored generated column.
const STORED_GENERATED: i64 = 3;

/// The invariant a trigger header that yields no timing and event breaks.
const HEADER: &str =
    "the header of every trigger the library accepted yields one timing and one event";

/// The invariant a key naming a column its table does not carry breaks.
const KEY_COLUMN: &str = "every key a sqlite read folds names columns of its own table";

/// The stage a fold failure is reported at.
const FOLD: Stage = Stage::Catalogue("the fold of the catalogue the statements returned");

/// Names matched as `BR-LITE-003` matches them: equal once the 26 ASCII
/// upper-case letters of each are folded to lower case, over the whole name,
/// and no other character folded.
///
/// SQLite refuses two tables or views, and two columns of one table, whose
/// names differ only in ASCII letter case, so among the names of a file whose
/// schema loads at most one matches; the first is kept all the same, so that
/// a lookup is defined whatever it is given.
#[derive(Debug, Default)]
struct Names {
    /// Each name with its ASCII letters lower-cased, and its position.
    folded: HashMap<String, usize>,
}

impl Names {
    /// The names, each at its position in `names`.
    fn of<'c>(names: impl Iterator<Item = &'c str>) -> Self {
        let mut found = Self::default();
        for (at, name) in names.enumerate() {
            found.folded.entry(name.to_ascii_lowercase()).or_insert(at);
        }
        found
    }

    /// The position of the name `name` matches.
    fn position(&self, name: &str) -> Option<usize> {
        self.folded.get(&name.to_ascii_lowercase()).copied()
    }
}

/// Every row the fold reads by a table's or an index's name, grouped once.
#[derive(Debug, Default)]
struct Grouped<'c> {
    /// The column rows of each table, in the order of `cid`.
    columns: HashMap<&'c str, Vec<&'c ColumnRow>>,
    /// The index rows of each table, member by member.
    indexes: HashMap<&'c str, Vec<&'c IndexRow>>,
    /// The key rows of each table, pair by pair.
    keys: HashMap<&'c str, Vec<&'c KeyRow>>,
    /// The stored row of each view, by its name.
    views: HashMap<&'c str, &'c StoredRow>,
}

impl<'c> Grouped<'c> {
    /// Groups every row of `catalogue` by the name it is read by.
    fn of(catalogue: &'c Catalogue) -> Self {
        let mut grouped = Self::default();
        for row in &catalogue.columns {
            grouped.columns.entry(&row.table).or_default().push(row);
        }
        for row in &catalogue.indexes {
            grouped.indexes.entry(&row.table).or_default().push(row);
        }
        for row in &catalogue.keys {
            grouped.keys.entry(&row.table).or_default().push(row);
        }
        for row in catalogue.stored.iter().filter(|row| row.kind == VIEW) {
            grouped.views.entry(&row.name).or_insert(row);
        }
        grouped
    }

    /// The rows of `table` in `map`, or none.
    fn of_table<'m, T>(map: &'m HashMap<&'c str, Vec<&'c T>>, table: &str) -> &'m [&'c T] {
        map.get(table).map_or(&[], Vec::as_slice)
    }
}

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
    let grouped = Grouped::of(catalogue);

    // FR-LITE-019 … FR-LITE-021: coverage, before anything is folded.
    let tables: Vec<&Object> = catalogue
        .objects
        .iter()
        .filter(|object| covered_table(object))
        .collect();
    let names = Names::of(tables.iter().map(|table| table.name.as_str()));
    let columns: Vec<Names> = tables
        .iter()
        .map(|table| {
            Names::of(
                Grouped::of_table(&grouped.columns, &table.name)
                    .iter()
                    .map(|row| row.name.as_str()),
            )
        })
        .collect();

    let mut parts: Vec<TableParts<'static>> = Vec::with_capacity(tables.len());
    for table in &tables {
        parts.push(self::table(&grouped, table));
    }

    // FR-LITE-031: the keys, once every table's columns and key are known;
    // each incoming key is listed under the table its parent matched.
    let mut incoming: Vec<(usize, IncomingForeignKey<'static>)> = Vec::new();
    for (at, table) in tables.iter().enumerate() {
        let rows = Grouped::of_table(&grouped.keys, &table.name);
        for (parent, key) in keys(rows, &tables, &names, &columns, &parts) {
            incoming.push((
                parent,
                IncomingForeignKey {
                    table: Cow::Owned(table.name.clone()),
                    key: key.clone(),
                },
            ));
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

    // FR-LITE-032 and FR-LITE-021: the triggers whose `tbl_name` matches a
    // covered table under BR-LITE-003; one that matches a view is excluded.
    let view_names = Names::of(
        catalogue
            .objects
            .iter()
            .filter(|object| object.kind == VIEW)
            .map(|object| object.name.as_str()),
    );
    for row in catalogue.stored.iter().filter(|row| row.kind == TRIGGER) {
        if view_names.position(&row.table).is_some() {
            // A trigger on a view: the model carries triggers on tables.
            continue;
        }
        let Some(at) = names.position(&row.table) else {
            // A trigger on no covered table — an object the model does not
            // cover.
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
        views: views(catalogue, &grouped),
        // FR-LITE-022: SQLite has no stored routines.
        routines: Vec::new(),
    })
}

/// Whether `object` is a covered table: a row of type `table` whose root page
/// is not `0` — a shadow table among them — and whose name is no internal
/// table's (`FR-LITE-020`, `FR-LITE-021`).
fn covered_table(object: &Object) -> bool {
    object.kind == TABLE && object.rootpage != Some(VIRTUAL_ROOT) && !internal(&object.name)
}

/// Whether `name` is an internal table's (`FR-LITE-021`).
fn internal(name: &str) -> bool {
    name.get(..INTERNAL.len())
        .is_some_and(|prefix| prefix.eq_ignore_ascii_case(INTERNAL))
}

/// One table's parts: its columns, its primary key and its indexes
/// (`FR-LITE-024` … `FR-LITE-030`).
fn table(grouped: &Grouped<'_>, object: &Object) -> TableParts<'static> {
    let rows = Grouped::of_table(&grouped.columns, &object.name);
    let indexes = by_index(Grouped::of_table(&grouped.indexes, &object.name));

    // FR-LITE-029: the primary key's columns, in ascending order of `pk`.
    let mut key_rows: Vec<&ColumnRow> = rows.iter().copied().filter(|row| row.pk > 0).collect();
    key_rows.sort_by_key(|row| row.pk);
    let origin = indexes
        .iter()
        .find(|(_, members)| members.iter().any(|row| row.origin == PRIMARY_ORIGIN))
        .map(|(name, members)| (*name, members.as_slice()));

    // FR-LITE-028: the rowid alias.
    let alias = (!object.without_rowid && key_rows.len() == 1 && origin.is_none())
        .then(|| key_rows.first().map(|row| row.name.as_str()))
        .flatten();

    let columns: Vec<Column<'static>> = rows
        .iter()
        .map(|row| column(row, &object.name, alias == Some(row.name.as_str())))
        .collect();

    // The direction of each member of the index of origin `pk`, by its name.
    let descending: HashMap<&str, bool> = origin
        .map(|(_, members)| {
            members
                .iter()
                .filter(|row| row.key)
                .filter_map(|row| row.name.as_deref().map(|name| (name, row.descending)))
                .collect()
        })
        .unwrap_or_default();

    let primary_key = (!key_rows.is_empty()).then(|| PrimaryKey {
        name: origin.map(|(name, _)| Cow::Owned(name.to_owned())),
        unique: origin.is_none_or(|(_, members)| members.first().is_none_or(|row| row.unique)),
        columns: key_rows
            .iter()
            .map(|row| IndexColumn {
                name: Cow::Owned(row.name.clone()),
                direction: Some(match origin {
                    Some(_) if descending.get(row.name.as_str()) == Some(&true) => {
                        SortDirection::Descending
                    }
                    // A rowid alias is the table's B-tree key, ascending.
                    _ => SortDirection::Ascending,
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
        indexes: self::indexes(&indexes),
        primary_key,
        ..TableParts::new(Cow::Owned(object.name.clone()), TableType::Base)
    }
}

/// The rows of one table's indexes, grouped by index, each index in the order
/// it first appears and its rows in theirs.
fn by_index<'c>(rows: &[&'c IndexRow]) -> Vec<(&'c str, Vec<&'c IndexRow>)> {
    let mut at: HashMap<&'c str, usize> = HashMap::new();
    let mut grouped: Vec<(&'c str, Vec<&'c IndexRow>)> = Vec::new();
    for row in rows {
        let slot = *at.entry(row.index.as_str()).or_insert_with(|| {
            grouped.push((row.index.as_str(), Vec::new()));
            grouped.len() - 1
        });
        if let Some((_, members)) = grouped.get_mut(slot) {
            members.push(row);
        }
    }
    grouped
}

/// The indexes of one table, less those with an expression member
/// (`FR-LITE-030`, `FR-LITE-021`).
fn indexes(grouped: &[(&str, Vec<&IndexRow>)]) -> Vec<Index<'static>> {
    grouped
        .iter()
        .filter_map(|(name, rows)| {
            let mut members: Vec<&IndexRow> = rows.iter().copied().filter(|row| row.key).collect();
            if members
                .iter()
                .any(|row| row.cid == EXPRESSION || row.name.is_none())
            {
                return None;
            }
            members.sort_by_key(|row| row.seqno);
            let unique = members.first().is_some_and(|row| row.unique);

            Some(Index {
                name: Cow::Owned((*name).to_owned()),
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

/// One column of a table (`FR-LITE-025` … `FR-LITE-028`).
fn column(row: &ColumnRow, table: &str, alias: bool) -> Column<'static> {
    let nullable = !row.not_null;

    Column {
        name: Cow::Owned(row.name.clone()),
        table_name: Cow::Owned(table.to_owned()),
        position: u64::try_from(row.cid).unwrap_or_default() + 1,
        column_type: ColumnType::affinity(
            Cow::Owned(row.declared.clone()),
            affinity(&row.declared),
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
/// `https://sqlite.org/datatype3.html`, in their order, for every column — a
/// column of a `STRICT` table included (`FR-LITE-026`, as amended in the
/// sixty-ninth edition): no statement reports that a table is `STRICT`, so a
/// column declared `ANY` takes `numeric`, the fifth rule's.
pub(super) fn affinity(declared: &str) -> &'static str {
    let upper = declared.to_ascii_uppercase();

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

/// The outgoing keys of one covered table, from its key rows, less those
/// `FR-LITE-021` excludes, each with the position of the covered table its
/// parent matched (`FR-LITE-031`).
fn keys(
    rows: &[&KeyRow],
    tables: &[&Object],
    names: &Names,
    columns: &[Names],
    parts: &[TableParts<'static>],
) -> Vec<(usize, ForeignKey<'static>)> {
    // One key per `id`, in ascending order of `id`, its pairs in order of
    // `seq`.
    let mut by_id: BTreeMap<i64, Vec<&KeyRow>> = BTreeMap::new();
    for row in rows {
        by_id.entry(row.id).or_default().push(row);
    }

    by_id
        .into_values()
        .filter_map(|mut pairs| {
            pairs.sort_by_key(|row| row.seq);
            let first = pairs.first()?;

            // FR-LITE-021: the parent table must match a covered table under
            // BR-LITE-003, and the key carries that table's own name.
            let parent_at = names.position(&first.referenced)?;
            let parent = parts.get(parent_at)?;
            let parent_name = &tables.get(parent_at)?.name;
            let parent_columns = columns.get(parent_at)?;

            // `to` NULL on every pair: the parent's primary key, in its order;
            // a parent with none is excluded. Otherwise each `to` must match a
            // column of the parent under BR-LITE-003, carried under the
            // column's own name.
            let referenced: Vec<String> = if pairs.iter().all(|row| row.to.is_none()) {
                let key = parent.primary_key.as_ref()?;
                key.columns
                    .iter()
                    .map(|member| member.name.to_string())
                    .collect()
            } else {
                pairs
                    .iter()
                    .map(|row| {
                        row.to
                            .as_deref()
                            .and_then(|to| carried(parent, parent_columns, to))
                    })
                    .collect::<Option<Vec<String>>>()?
            };
            // FR-LITE-021: a key naming no parent column of a parent whose
            // primary key has another number of columns than the key pairs
            // nothing, and is excluded as a dangling key is. A key naming its
            // parent columns has one per pair already.
            if referenced.len() != pairs.len() {
                return None;
            }

            // A key's own columns are not matched: the `from` field is the
            // column's name as the referencing table declares it
            // (BR-LITE-003), carried as reported.
            let columns = pairs
                .iter()
                .zip(referenced)
                .map(|(row, referenced)| ForeignKeyColumn {
                    column: Cow::Owned(row.from.clone()),
                    referenced_column: Cow::Owned(referenced),
                })
                .collect();

            Some((
                parent_at,
                ForeignKey {
                    name: None,
                    columns,
                    referenced_table: Some(Cow::Owned(parent_name.clone())),
                    referenced_key: None,
                    match_option: Cow::Owned(first.matched.clone()),
                    on_update: ReferentialAction::from_catalogue_owned(first.on_update.clone()),
                    on_delete: ReferentialAction::from_catalogue_owned(first.on_delete.clone()),
                },
            ))
        })
        .collect()
}

/// The name a column of `table` carries, matched under `BR-LITE-003` through
/// the table's column names grouped once.
fn carried(table: &TableParts<'_>, names: &Names, column: &str) -> Option<String> {
    names
        .position(column)
        .and_then(|at| table.columns.get(at))
        .map(|carried| carried.name.to_string())
}

/// The views: each one's name and whole stored statement, and nothing a view
/// whose definition does not compile could lack (`FR-LITE-033`,
/// `FR-LITE-036`).
fn views(catalogue: &Catalogue, grouped: &Grouped<'_>) -> Vec<View<'static>> {
    let mut views: Vec<View<'static>> = catalogue
        .objects
        .iter()
        .filter(|object| object.kind == VIEW)
        .map(|object| {
            let definition = grouped
                .views
                .get(object.name.as_str())
                .and_then(|row| row.sql.clone())
                .unwrap_or_default();

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
                restricted: None,
            }
        })
        .collect();

    order::sort_by_name(&mut views);
    views
}

#[cfg(test)]
mod tests {
    use super::{Names, affinity, default, internal, number};
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
            ("any", "numeric"),
        ] {
            assert_eq!(affinity(declared), expected, "{declared}");
        }
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

    #[test]
    fn br_lite_003_names_match_with_the_26_ascii_letters_folded_and_no_other_character() {
        let names =
            Names::of(["Vessel", "port_call", "\u{c9}mbarque", "\u{e9}mbarque"].into_iter());

        // Folded over the whole name, in either direction.
        assert_eq!(names.position("Vessel"), Some(0));
        assert_eq!(names.position("VESSEL"), Some(0));
        assert_eq!(names.position("vessel"), Some(0));
        assert_eq!(names.position("vEsSeL"), Some(0));
        assert_eq!(names.position("PORT_CALL"), Some(1));
        // No character outside ASCII is folded: É and é are two names.
        assert_eq!(names.position("\u{c9}MBARQUE"), Some(2));
        assert_eq!(names.position("\u{e9}MBARQUE"), Some(3));
        assert_eq!(names.position("\u{c9}mbarque"), Some(2));
        // The whole name, and nothing near it.
        for other in ["Vesse", "Vessels", "v\u{e9}ssel", "port call", "", "berth"] {
            assert_eq!(names.position(other), None, "{other}");
        }
        // A file whose schema loads holds no two names that fold alike; were
        // it given them, the first is kept.
        let clashing = Names::of(["a", "A"].into_iter());
        assert_eq!(clashing.position("A"), Some(0));
    }
}
