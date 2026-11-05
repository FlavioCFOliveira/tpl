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
//! **Rows are grouped, and names matched, on the bytes the file holds**
//! (`FR-LITE-043`). SQLite holds a name as the bytes a statement wrote, valid
//! UTF-8 or not, and `FR-OUT-017` converts those bytes to text with U+FFFD
//! substituted, so two names that differ only in invalid UTF-8 convert to one
//! text. Every row is therefore found by the bytes of its object's name
//! ([`super::Raw`]), never by the text, so each row stays with its object and
//! every reference with the object SQLite resolves it to. Where two names of
//! one set — the covered tables and the views together, or the columns, the
//! indexes or the triggers of one covered table — differ in their bytes and
//! convert to one text, the document could not tell them apart, and the read
//! is refused with the `78` of `FR-LITE-043` before anything is folded
//! ([`alike`]).
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
//! `FR-CTX-023`). The comparison is of the bytes the file holds for each name,
//! with the 26 ASCII letters folded (`FR-LITE-043`). An index's members and a
//! key's own columns are not matched: the library reports each as its table
//! declares it.
//!
//! **The fold indexes before it looks up.** Every collection the fold reads
//! by a name is grouped once and then looked up, and no object is found by
//! scanning another collection. The column, index and key rows are grouped by
//! their table: each statement's runs of rows ([`Tabled`]) are sorted once by
//! the bytes of their table's name and a table's are found by binary search; a
//! table's index runs are sorted by the bytes of the index's name, which
//! brings an index's members together; the stored rows of the views are
//! sorted by the bytes of their name and found the same way. The covered tables, the views and a table's columns are
//! looked up by the name `BR-LITE-003` matches, through a map keyed by that
//! name with the 26 ASCII letters folded ([`Names`]) — a table's columns only
//! where a key names its parent columns. The fold runs after the last
//! statement, outside every statement's bound, so a lookup that scanned would
//! let a file of many objects cost time growing with their square where the
//! statements' grows with their number (architecture, *The SQLite reader*;
//! `FR-LITE-009`, *Consequence*; `FR-LITE-020`, *Provenance*). The maps are
//! only looked up, never iterated, so no order of a document depends on them.
//!
//! **Every order is produced here.** No statement orders its rows
//! (`statements`), and grouping is correct whatever order they come in. The
//! objects are taken in ascending order of the bytes of their names, so a
//! position among the covered tables, and the first of two names a map keeps,
//! do not depend on the order of the rows; a table's columns in ascending
//! order of `cid` (`FR-LITE-025`); an index's members in ascending order of
//! `seqno` (`FR-LITE-030`); the primary key's members in ascending order of
//! `pk` (`FR-LITE-029`); a key's pairs in ascending order of `seq`, its keys
//! one per `id` in ascending order of `id`, then the keys in both directions by
//! content (`FR-LITE-031`); and the tables, the indexes, the triggers and the
//! views by name (`NFR-DET-002`).
//!
//! **No two names of one set tie.** [`alike`] has refused a file in which two
//! names of one set convert to one text, so every order by name above is
//! total. The objects are taken in the order of their names' bytes before they
//! are sorted by text, so that whatever order the rows came in, the fold is
//! the same. Each name keeps its bytes only where the conversion changed them
//! ([`super::Raw`]); elsewhere the text is the bytes.
//!
//! PERF: the model borrows every name and text from the rows the read owns,
//! and copies none of them.

use std::borrow::Cow;
use std::collections::HashMap;
use std::hash::{Hash, Hasher};

use super::trigger::{self, Event, Timing};
use super::{Catalogue, ColumnRow, IndexMember, IndexRun, KeyRow, Object, Run, StoredRow, Tabled};
use crate::error::{AlikeKind, AlikeNames, Error, FileValue};
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

/// A name as `BR-LITE-003` matches it: equal to another once the 26 ASCII
/// upper-case letters of each are folded to lower case, over the whole name,
/// and no other character folded.
///
/// It is taken over the bytes the file holds for the name, not over the text
/// they convert to (`FR-LITE-043`). PERF: it hashes the folded bytes and
/// compares ignoring ASCII case, so a map keyed by it holds the name itself
/// and never a folded copy of it.
#[derive(Debug, Clone, Copy)]
struct Folded<'n>(&'n [u8]);

impl PartialEq for Folded<'_> {
    fn eq(&self, other: &Self) -> bool {
        self.0.eq_ignore_ascii_case(other.0)
    }
}

impl Eq for Folded<'_> {}

impl Hash for Folded<'_> {
    /// The bytes [`slice::to_ascii_lowercase`] gives, in chunks of a fixed
    /// size, then a terminator, so that two names equal under
    /// [`PartialEq`] — which have one length — hash alike.
    fn hash<H: Hasher>(&self, state: &mut H) {
        let mut folded = [0_u8; 32];
        for chunk in self.0.chunks(folded.len()) {
            let folded = folded.get_mut(..chunk.len()).unwrap_or_default();
            folded.copy_from_slice(chunk);
            folded.make_ascii_lowercase();
            state.write(folded);
        }
        state.write_u8(0xff);
    }
}

/// Names matched as `BR-LITE-003` matches them, through [`Folded`].
///
/// SQLite refuses two tables or views, and two columns of one table, whose
/// names differ only in ASCII letter case, so among the names of a file whose
/// schema loads at most one matches; the first is kept all the same, so that
/// a lookup is defined whatever it is given.
#[derive(Debug, Default)]
struct Names<'n> {
    /// Each name, and its position.
    folded: HashMap<Folded<'n>, usize>,
}

impl<'n> Names<'n> {
    /// The names, by their bytes, each at its position in `names`.
    fn of(names: impl Iterator<Item = &'n [u8]>) -> Self {
        let mut folded = HashMap::with_capacity(names.size_hint().0);
        for (at, name) in names.enumerate() {
            folded.entry(Folded(name)).or_insert(at);
        }
        Self { folded }
    }

    /// The position of the name the bytes `name` match.
    fn position(&self, name: &[u8]) -> Option<usize> {
        let folded: &HashMap<Folded<'_>, usize> = &self.folded;
        folded.get(&Folded(name)).copied()
    }
}

/// The rows of one statement, found by their table's name.
#[derive(Debug)]
struct ByTable<'c, T> {
    /// The rows.
    tabled: &'c Tabled<T>,
    /// Their runs in ascending order of the bytes of their table's name; the
    /// sort is stable, so the runs of one table keep the order the statement
    /// returned them in.
    runs: Vec<&'c Run>,
}

impl<'c, T> ByTable<'c, T> {
    /// Groups the runs of `tabled` by the bytes of their table's name, once.
    fn of(tabled: &'c Tabled<T>) -> Self {
        let mut runs: Vec<&'c Run> = tabled.runs.iter().collect();
        runs.sort_by(|left, right| left.key().cmp(right.key()));
        Self { tabled, runs }
    }

    /// The rows of the table whose name the file holds as `table`, each with
    /// its run, run by run, and each run's rows in the order the statement
    /// returned them.
    fn rows(&self, table: &[u8]) -> Vec<(&'c Run, &'c T)> {
        let start = self.runs.partition_point(|run| run.key() < table);
        let after = self.runs.get(start..).unwrap_or_default();
        let runs = after
            .get(..after.partition_point(|run| run.key() == table))
            .unwrap_or_default();

        let mut rows = Vec::with_capacity(runs.iter().map(|run| run.rows.len()).sum());
        for run in runs {
            rows.extend(self.tabled.rows_of(run).iter().map(|row| (*run, row)));
        }
        rows
    }
}

/// Every row the fold reads by a table's or a view's name, grouped once.
#[derive(Debug)]
struct Grouped<'c> {
    /// The column rows, by their table.
    columns: ByTable<'c, ColumnRow>,
    /// The index runs, by their table.
    indexes: ByTable<'c, IndexRun>,
    /// The key rows, by their table.
    keys: ByTable<'c, KeyRow>,
    /// The stored rows of the views, in ascending order of the bytes of
    /// their name.
    views: Vec<&'c StoredRow>,
}

impl<'c> Grouped<'c> {
    /// Groups every row of `catalogue` by the name it is read by.
    fn of(catalogue: &'c Catalogue) -> Self {
        let mut views: Vec<&'c StoredRow> = catalogue
            .stored
            .iter()
            .filter(|row| row.kind == VIEW)
            .collect();
        views.sort_by(|left, right| left.key().cmp(right.key()));

        Self {
            columns: ByTable::of(&catalogue.columns),
            indexes: ByTable::of(&catalogue.indexes.indexes),
            keys: ByTable::of(&catalogue.keys),
            views,
        }
    }

    /// The stored row of the view whose name the file holds as `name`.
    fn view(&self, name: &[u8]) -> Option<&'c StoredRow> {
        let at = self.views.partition_point(|row| row.key() < name);
        self.views.get(at).copied().filter(|row| row.key() == name)
    }
}

/// Folds the catalogue into the model, which borrows from it.
///
/// # Errors
///
/// Returns the `78` of `FR-LITE-043` where two names of one set convert to
/// one text from different bytes, before anything is folded, and the `70` of
/// `FR-LITE-032` and of `FR-CAT-044`.
pub(super) fn database(catalogue: &Catalogue) -> Result<Database<'_>, Error> {
    let place = Place {
        entry: &catalogue.entry,
        path: &catalogue.path,
    };
    let grouped = Grouped::of(catalogue);

    // The objects in ascending order of the bytes their names were converted
    // from, whatever order the statement returned them in.
    let mut objects: Vec<&Object> = catalogue.objects.iter().collect();
    objects.sort_by(|left, right| left.key().cmp(right.key()));

    // FR-LITE-019 … FR-LITE-021: coverage, before anything is folded.
    let tables: Vec<&Object> = objects
        .iter()
        .copied()
        .filter(|object| covered_table(object))
        .collect();
    let names = Names::of(tables.iter().map(|table| table.key()));
    let view_names = Names::of(
        objects
            .iter()
            .filter(|object| object.kind == VIEW)
            .map(|object| object.key()),
    );

    // FR-LITE-032 and FR-LITE-021: the triggers whose `tbl_name` matches a
    // covered table under BR-LITE-003, each with that table's position; one
    // that matches a view is excluded, and one on no covered table is an
    // object the model does not cover.
    let mut triggers: Vec<&StoredRow> = catalogue
        .stored
        .iter()
        .filter(|row| row.kind == TRIGGER)
        .collect();
    triggers.sort_by(|left, right| left.key().cmp(right.key()));
    let triggers: Vec<(usize, &StoredRow)> = triggers
        .into_iter()
        .filter(|row| view_names.position(row.table_key()).is_none())
        .filter_map(|row| Some((names.position(row.table_key())?, row)))
        .collect();

    // FR-LITE-043: before anything is folded, and so before any model or
    // cache write.
    alike(place, &objects, &tables, &grouped, &triggers)?;

    let (mut parts, column_keys): (Vec<TableParts<'_>>, Vec<Vec<&[u8]>>) = tables
        .iter()
        .map(|table| self::table(catalogue, &grouped, table))
        .unzip();

    // FR-LITE-031: the keys, once every table's columns and key are known.
    // A table's columns are matched by name only for a key that names its
    // parent columns, so their names are indexed only for such a parent.
    let mut outgoing = Vec::new();
    {
        let mut folded: HashMap<usize, Names<'_>> = HashMap::new();
        for (at, table) in tables.iter().enumerate() {
            let rows = grouped.keys.rows(table.key());
            let parents = Parents {
                tables: &tables,
                names: &names,
                parts: &parts,
                columns: &column_keys,
            };
            for (parent, key) in keys(rows, &parents, &mut folded) {
                outgoing.push((at, table.name.as_str(), parent, key));
            }
        }
    }
    // Each incoming key is listed under the table its parent matched.
    let mut incoming: Vec<(usize, IncomingForeignKey<'_>)> = Vec::with_capacity(outgoing.len());
    for (at, table, parent, key) in outgoing {
        incoming.push((
            parent,
            IncomingForeignKey {
                table: Cow::Borrowed(table),
                key: key.clone(),
            },
        ));
        if let Some(part) = parts.get_mut(at) {
            part.foreign_keys.push(key);
        }
    }
    for (at, key) in incoming {
        if let Some(part) = parts.get_mut(at) {
            part.referenced_by.push(key);
        }
    }

    for (at, row) in triggers {
        let sql = row.sql.as_deref().unwrap_or_default();
        let header =
            trigger::header(sql).ok_or_else(|| fault::defect(place, FOLD, HEADER, None))?;
        if let Some(part) = parts.get_mut(at) {
            part.triggers.push(Trigger {
                name: Cow::Borrowed(&row.name),
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
                statement: Some(Cow::Borrowed(sql)),
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
        charset: Cow::Borrowed(&catalogue.encoding),
        collation: None,
        provider: Provider::Sqlite,
        server: catalogue.server.clone(),
        tables: assembled,
        views: views(&objects, &grouped),
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
///
/// The bytes of each column's name are returned beside the parts, in the order
/// of their columns, for the match of a key's parent columns.
fn table<'c>(
    catalogue: &'c Catalogue,
    grouped: &Grouped<'c>,
    object: &'c Object,
) -> (TableParts<'c>, Vec<&'c [u8]>) {
    // FR-LITE-025: the columns in ascending order of `cid`.
    let mut rows = grouped.columns.rows(object.key());
    rows.sort_by_key(|(_, row)| row.cid);
    // FR-LITE-030: the index runs in ascending order of the bytes of the
    // index's name, so that the runs of one index stand together whatever
    // order its members came in.
    let mut entries = grouped.indexes.rows(object.key());
    entries.sort_by(|(_, left), (_, right)| left.key().cmp(right.key()));
    let members = &catalogue.indexes;

    // FR-LITE-029: the primary key's columns, in ascending order of `pk`.
    let mut key_rows: Vec<&ColumnRow> = rows
        .iter()
        .map(|(_, row)| *row)
        .filter(|row| row.pk > 0)
        .collect();
    key_rows.sort_by_key(|row| row.pk);
    // The index of origin `pk`: of several, the one whose bytes come first.
    let origin = by_index(&entries)
        .filter(|index| index.iter().any(|(_, run)| run.origin == PRIMARY_ORIGIN))
        .min_by(|left, right| bytes_of(left).cmp(&bytes_of(right)));
    // Its members, in the order of their bytes, then of `seqno`.
    let origin_members = origin
        .map(|index| ordered_members(members, index))
        .unwrap_or_default();

    // FR-LITE-028: the rowid alias.
    let alias = (!object.without_rowid && key_rows.len() == 1 && origin.is_none())
        .then(|| key_rows.first().map(|row| row.name.as_str()))
        .flatten();

    let columns: Vec<Column<'c>> = rows
        .iter()
        .map(|(_, row)| column(row, &object.name, alias == Some(row.name.as_str())))
        .collect();
    let column_keys = rows.iter().map(|(_, row)| row.key()).collect();

    // The direction of each member of the index of origin `pk`, by its name.
    let descending: HashMap<&str, bool> = origin_members
        .iter()
        .filter(|(member, _, _)| member.key)
        .filter_map(|(member, _, _)| member.name.as_deref().map(|name| (name, member.descending)))
        .collect();

    let primary_key = (!key_rows.is_empty()).then(|| PrimaryKey {
        name: origin
            .and_then(|index| index.first())
            .map(|(_, run)| Cow::Borrowed(run.name.as_str())),
        unique: origin_members.first().is_none_or(|(_, _, run)| run.unique),
        columns: key_rows
            .iter()
            .map(|row| IndexColumn {
                name: Cow::Borrowed(&row.name),
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

    let parts = TableParts {
        engine: None,
        collation: None,
        columns,
        indexes: self::indexes(members, &entries),
        primary_key,
        ..TableParts::new(Cow::Borrowed(&object.name), TableType::Base)
    };

    (parts, column_keys)
}

/// One index run of a table, with the run of its table.
type Entry<'c> = (&'c Run, &'c IndexRun);

/// One table's index runs, ordered by the bytes of the index's name, as one
/// slice per index.
fn by_index<'r, 'c>(entries: &'r [Entry<'c>]) -> impl Iterator<Item = &'r [Entry<'c>]> {
    entries.chunk_by(|(_, left), (_, right)| left.key() == right.key())
}

/// The bytes of the table's name and of the index's name of the first run of
/// `index`, which order the indexes of one name as the statement's
/// `ORDER BY` ordered them.
fn bytes_of<'c>(index: &[Entry<'c>]) -> Option<(&'c [u8], &'c [u8])> {
    index.first().map(|(table, run)| (table.key(), run.key()))
}

/// Every member of the index whose runs are `index`, each with the runs that
/// carry its table and its index's fields, run by run.
fn index_members<'c>(
    members: &'c super::Indexes,
    index: &[Entry<'c>],
) -> impl Iterator<Item = (&'c IndexMember, &'c Run, &'c IndexRun)> {
    index.iter().flat_map(move |(table, run)| {
        members
            .members_of(run)
            .iter()
            .map(move |member| (member, *table, *run))
    })
}

/// Every member of the index whose runs are `index`, in ascending order of
/// the bytes of its table's name and of its index's name, then of `seqno`.
fn ordered_members<'c>(
    members: &'c super::Indexes,
    index: &[Entry<'c>],
) -> Vec<(&'c IndexMember, &'c Run, &'c IndexRun)> {
    let mut ordered: Vec<_> = index_members(members, index).collect();
    ordered.sort_by(
        |(left, left_table, left_run), (right, right_table, right_run)| {
            left_table
                .key()
                .cmp(right_table.key())
                .then_with(|| left_run.key().cmp(right_run.key()))
                .then(left.seqno.cmp(&right.seqno))
        },
    );
    ordered
}

/// The indexes of one table, from its index runs ordered by the bytes of the
/// index's name, less those with an expression member (`FR-LITE-030`,
/// `FR-LITE-021`).
fn indexes<'c>(members: &'c super::Indexes, entries: &[Entry<'c>]) -> Vec<Index<'c>> {
    by_index(entries)
        .filter_map(|index| {
            let (_, first) = index.first()?;
            let mut keyed: Vec<(&IndexMember, &IndexRun)> = index_members(members, index)
                .filter(|(member, _, _)| member.key)
                .map(|(member, _, run)| (member, run))
                .collect();
            if keyed
                .iter()
                .any(|(member, _)| member.cid == EXPRESSION || member.name.is_none())
            {
                return None;
            }
            // FR-LITE-030: the members in ascending order of `seqno`; the
            // sort is stable, so members of one `seqno` keep the order of
            // their bytes.
            keyed.sort_by_key(|(member, _)| member.seqno);
            let unique = keyed.first().is_some_and(|(_, run)| run.unique);

            Some(Index {
                name: Cow::Borrowed(&first.name),
                unique,
                columns: keyed
                    .iter()
                    .map(|(member, _)| IndexColumn {
                        name: Cow::Borrowed(member.name.as_deref().unwrap_or_default()),
                        direction: Some(if member.descending {
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
fn column<'c>(row: &'c ColumnRow, table: &'c str, alias: bool) -> Column<'c> {
    let nullable = !row.not_null;

    Column {
        name: Cow::Borrowed(&row.name),
        table_name: Cow::Borrowed(table),
        position: u64::try_from(row.cid).unwrap_or_default() + 1,
        column_type: ColumnType::affinity(Cow::Borrowed(&row.declared), affinity(&row.declared)),
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
///
/// Each rule asks whether the declared type holds a word in any ASCII letter
/// case, which is what upper-casing the type and searching it for the word
/// in upper case answers. PERF: it is asked of the type as declared, so no
/// upper-cased copy is made.
pub(super) fn affinity(declared: &str) -> &'static str {
    let holds = |word: &str| holds_ignoring_ascii_case(declared, word);

    if holds("INT") {
        "integer"
    } else if holds("CHAR") || holds("CLOB") || holds("TEXT") {
        "text"
    } else if holds("BLOB") || declared.is_empty() {
        "blob"
    } else if holds("REAL") || holds("FLOA") || holds("DOUB") {
        "real"
    } else {
        "numeric"
    }
}

/// Whether `text` holds `word`, its ASCII letters compared ignoring case and
/// every other byte compared exactly.
///
/// A match found among the bytes of `text` begins on a character, because
/// `word` is ASCII and no byte of a multi-byte character is.
fn holds_ignoring_ascii_case(text: &str, word: &str) -> bool {
    word.is_empty()
        || text
            .as_bytes()
            .windows(word.len())
            .any(|window| window.eq_ignore_ascii_case(word.as_bytes()))
}

/// The default of a column, classified by the first row of `FR-LITE-027`
/// that matches.
pub(super) fn default(written: Option<&str>, nullable: bool) -> Option<ColumnDefault<'_>> {
    let Some(written) = written else {
        // The first two rows: SQL NULL is no default for a column that is not
        // nullable, and the default NULL for one that is.
        return nullable.then_some(ColumnDefault::Null);
    };

    if written.eq_ignore_ascii_case("NULL") {
        return Some(ColumnDefault::Null);
    }
    if let Some(inner) = quoted(written) {
        return Some(ColumnDefault::Literal(
            crate::model::collapse_doubled_apostrophes(inner),
        ));
    }
    if number(written)
        || blob(written)
        || written.eq_ignore_ascii_case("TRUE")
        || written.eq_ignore_ascii_case("FALSE")
    {
        return Some(ColumnDefault::Literal(Cow::Borrowed(written)));
    }

    Some(ColumnDefault::Expression(Cow::Borrowed(written)))
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

/// What a key's parent is matched against: the covered tables, their names,
/// their parts, and the bytes of each one's column names in column order.
struct Parents<'r, 'c> {
    /// The covered tables.
    tables: &'r [&'c Object],
    /// Their names, by their bytes.
    names: &'r Names<'r>,
    /// Their parts, at the tables' positions.
    parts: &'r [TableParts<'c>],
    /// The bytes of each one's column names, at the tables' positions.
    columns: &'r [Vec<&'c [u8]>],
}

/// The outgoing keys of one covered table, from its key rows, less those
/// `FR-LITE-021` excludes, each with the position of the covered table its
/// parent matched (`FR-LITE-031`).
///
/// `folded` holds the column names of every parent already matched by name,
/// indexed on first need.
fn keys<'c, 'p>(
    mut rows: Vec<(&'c Run, &'c KeyRow)>,
    parents: &Parents<'p, 'c>,
    folded: &mut HashMap<usize, Names<'p>>,
) -> Vec<(usize, ForeignKey<'c>)> {
    // One key per `id`, in ascending order of `id`, its pairs in ascending
    // order of `seq` and then of the bytes of their table's name, whatever
    // order the rows came in.
    rows.sort_by(|(left_table, left), (right_table, right)| {
        (left.id, left.seq)
            .cmp(&(right.id, right.seq))
            .then_with(|| left_table.key().cmp(right_table.key()))
    });

    rows.chunk_by(|(_, left), (_, right)| left.id == right.id)
        .filter_map(|pairs| {
            let (_, first) = pairs.first()?;

            // FR-LITE-021: the parent table must match a covered table under
            // BR-LITE-003, on the bytes the file holds (FR-LITE-043), and the
            // key carries that table's own name.
            let parent_at = parents.names.position(first.referenced_key())?;
            let parent = parents.parts.get(parent_at)?;
            let parent_name = &parents.tables.get(parent_at)?.name;

            // `to` NULL on every pair: the parent's primary key, in its order;
            // a parent with none is excluded. Otherwise each `to` must match a
            // column of the parent under BR-LITE-003, carried under the
            // column's own name.
            let referenced: Vec<Cow<'c, str>> = if pairs.iter().all(|(_, row)| row.to.is_none()) {
                let key = parent.primary_key.as_ref()?;
                key.columns
                    .iter()
                    .map(|member| member.name.clone())
                    .collect()
            } else {
                let keys = parents.columns.get(parent_at)?;
                let columns = folded
                    .entry(parent_at)
                    .or_insert_with(|| Names::of(keys.iter().copied()));
                pairs
                    .iter()
                    .map(|(_, row)| row.to_key().and_then(|to| carried(parent, columns, to)))
                    .collect::<Option<Vec<Cow<'c, str>>>>()?
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
                .map(|((_, row), referenced)| ForeignKeyColumn {
                    column: Cow::Borrowed(&row.from),
                    referenced_column: referenced,
                })
                .collect();

            Some((
                parent_at,
                ForeignKey {
                    name: None,
                    columns,
                    referenced_table: Some(Cow::Borrowed(parent_name)),
                    referenced_key: None,
                    match_option: Cow::Borrowed(&first.matched),
                    on_update: ReferentialAction::from_catalogue(&first.on_update),
                    on_delete: ReferentialAction::from_catalogue(&first.on_delete),
                },
            ))
        })
        .collect()
}

/// The name a column of `table` carries, matched under `BR-LITE-003` on the
/// bytes `column` through the table's column names grouped once.
fn carried<'c>(table: &TableParts<'c>, names: &Names<'_>, column: &[u8]) -> Option<Cow<'c, str>> {
    names
        .position(column)
        .and_then(|at| table.columns.get(at))
        .map(|carried| carried.name.clone())
}

/// Refuses a file in which two names of one set of `FR-LITE-043` differ in
/// their bytes and convert to one text: the covered tables and the views
/// together, then, table by table in the order of their bytes, the columns,
/// the indexes and the triggers of one covered table.
///
/// `objects` are in the order of their names' bytes, and `triggers` are each
/// with the position of the covered table they matched.
///
/// # Errors
///
/// Returns [`Error::SqliteNamesAlike`] for the first pair found.
fn alike(
    place: Place<'_>,
    objects: &[&Object],
    tables: &[&Object],
    grouped: &Grouped<'_>,
    triggers: &[(usize, &StoredRow)],
) -> Result<(), Error> {
    let refused = |kind, table: Option<&str>, text: &str| Error::SqliteNamesAlike {
        entry: place.entry.to_owned(),
        path: place.path.to_owned(),
        names: Box::new(AlikeNames {
            kind,
            table: table.map(FileValue::new),
            text: FileValue::new(text),
        }),
    };

    // The first set: the covered tables and the views, taken together.
    let named = objects
        .iter()
        .filter(|object| object.kind == VIEW || covered_table(object))
        .map(|object| (object.name.as_str(), object.key(), object.kind == VIEW));
    if let Some(((text, _, left), (_, _, right))) = first_alike(named) {
        let kind = match (left, right) {
            (false, false) => AlikeKind::Tables,
            (true, true) => AlikeKind::Views,
            _ => AlikeKind::TableAndView,
        };
        return Err(refused(kind, None, text));
    }

    for (at, table) in tables.iter().enumerate() {
        let held = Some(table.name.as_str());

        let columns = grouped.columns.rows(table.key());
        let columns = columns
            .iter()
            .map(|(_, row)| (row.name.as_str(), row.key(), ()));
        if let Some(((text, _, ()), _)) = first_alike(columns) {
            return Err(refused(AlikeKind::Columns, held, text));
        }

        let indexes = grouped.indexes.rows(table.key());
        let indexes = indexes
            .iter()
            .map(|(_, run)| (run.name.as_str(), run.key(), ()));
        if let Some(((text, _, ()), _)) = first_alike(indexes) {
            return Err(refused(AlikeKind::Indexes, held, text));
        }

        let on_table = triggers
            .iter()
            .filter(|(of, _)| *of == at)
            .map(|(_, row)| (row.name.as_str(), row.key(), ()));
        if let Some(((text, _, ()), _)) = first_alike(on_table) {
            return Err(refused(AlikeKind::Triggers, held, text));
        }
    }

    Ok(())
}

/// One name of a set of `FR-LITE-043`: its text, the bytes it was converted
/// from, and a tag.
type Named<'n, T> = (&'n str, &'n [u8], T);

/// The first two of `names` whose texts are equal and whose bytes differ, in
/// the order of their texts and then of their bytes.
///
/// A name that appears more than once with the same bytes — the runs of one
/// index, or the rows of one object — is one name, and is not a pair.
fn first_alike<'n, T: Copy>(
    names: impl Iterator<Item = Named<'n, T>>,
) -> Option<(Named<'n, T>, Named<'n, T>)> {
    let mut sorted: Vec<Named<'n, T>> = names.collect();
    sorted.sort_by(|(left, left_bytes, _), (right, right_bytes, _)| {
        order::compare(left, right).then_with(|| left_bytes.cmp(right_bytes))
    });

    sorted.windows(2).find_map(|pair| match pair {
        [left, right] if left.0 == right.0 && left.1 != right.1 => Some((*left, *right)),
        _ => None,
    })
}

/// The views: each one's name and whole stored statement, and nothing a view
/// whose definition does not compile could lack (`FR-LITE-033`,
/// `FR-LITE-036`), from the objects in ascending order of name.
fn views<'c>(objects: &[&'c Object], grouped: &Grouped<'c>) -> Vec<View<'c>> {
    let mut views: Vec<View<'c>> = objects
        .iter()
        .filter(|object| object.kind == VIEW)
        .map(|object| {
            let definition = grouped
                .view(object.key())
                .and_then(|row| row.sql.as_deref())
                .unwrap_or_default();

            View {
                name: Cow::Borrowed(&object.name),
                definition: Cow::Borrowed(definition),
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
    use super::{ByTable, Names, affinity, default, indexes, internal, number};
    use crate::model::column_default::ColumnDefault;
    use crate::model::index::SortDirection;
    use crate::sqlite::catalogue::{IndexMember, Indexes, Tabled};
    use std::borrow::Cow;

    /// The type affinity as it was decided before the declared type was
    /// searched in place: over an upper-cased copy of it.
    fn affinity_of_an_upper_cased_copy(declared: &str) -> &'static str {
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

    /// Every spelling of `word` in which each ASCII letter is upper or lower
    /// case.
    fn every_case(word: &str) -> Vec<String> {
        let letters = word.bytes().filter(u8::is_ascii_alphabetic).count();
        (0_u32..1 << letters)
            .map(|mask| {
                let mut at = 0;
                word.chars()
                    .map(|character| {
                        if !character.is_ascii_alphabetic() {
                            return character;
                        }
                        let upper = mask & (1 << at) != 0;
                        at += 1;
                        if upper {
                            character.to_ascii_uppercase()
                        } else {
                            character.to_ascii_lowercase()
                        }
                    })
                    .collect()
            })
            .collect()
    }

    #[test]
    fn fr_lite_026_the_type_searched_in_place_decides_as_its_upper_cased_copy_did() {
        // FR-LITE-026: the affinity of every declared type is unchanged by
        // searching the type ignoring ASCII case rather than an upper-cased
        // copy of it: every letter case of every word the five rules search
        // for, alone and inside other text, and texts whose letters outside
        // ASCII look like those words or sit beside them.
        let mut declared: Vec<String> = Vec::new();
        for word in [
            "INT", "CHAR", "CLOB", "TEXT", "BLOB", "REAL", "FLOA", "DOUB",
        ] {
            for spelling in every_case(word) {
                for (before, after) in [
                    ("", ""),
                    ("x", ""),
                    ("", "y"),
                    ("var", "(80)"),
                    ("\u{e9}", "\u{e9}"),
                    ("big ", " unsigned"),
                    ("\u{0}", "\u{0}"),
                ] {
                    declared.push(format!("{before}{spelling}{after}"));
                }
            }
        }
        declared.extend(
            [
                "",
                " ",
                "ANY",
                "any",
                "NUMERIC(5,2)",
                "DECIMAL(12,3)",
                "BOOLEAN",
                "DateTime",
                "STRING",
                "FLOATING POINT",
                "CharInt",
                "\u{131}nt",
                "\u{130}NT",
                "\u{ff49}\u{ff4e}\u{ff54}",
                "ch\u{e4}r",
                "TEXT\u{e9}",
                "\u{c9}TEXT",
                "flo\u{301}at",
                "D\u{d6}UB",
                "IN T",
                "I\u{0}NT",
                "REA",
                "FLO",
                "DOU",
                "BLO",
                "CLO",
                "TEX",
                "CHA",
            ]
            .map(str::to_owned),
        );

        for declared in &declared {
            assert_eq!(
                affinity(declared),
                affinity_of_an_upper_cased_copy(declared),
                "{declared:?}"
            );
        }
    }

    #[test]
    fn br_lite_003_a_name_longer_than_a_hash_chunk_matches_in_any_letter_case() {
        // The folded key hashes a name in chunks of 32 bytes: a name across
        // several chunks still matches in every letter case, and a name that
        // differs from it in one letter outside ASCII, or in its length,
        // does not.
        let long = "Consignment_Customs_Declaration_Line_Item_Archive_2026";
        let names = Names::of([long, "other"].into_iter().map(str::as_bytes));

        assert_eq!(names.position(long.as_bytes()), Some(0));
        assert_eq!(
            names.position(long.to_ascii_uppercase().as_bytes()),
            Some(0)
        );
        assert_eq!(
            names.position(long.to_ascii_lowercase().as_bytes()),
            Some(0)
        );
        assert_eq!(names.position(long.replace('A', "\u{c0}").as_bytes()), None);
        assert_eq!(names.position(&long.as_bytes()[..long.len() - 1]), None);
        assert_eq!(names.position(format!("{long}x").as_bytes()), None);
    }

    #[test]
    fn a_table_s_rows_are_grouped_whatever_runs_they_came_in() {
        // Rows of three tables interleaved: each table's rows are found, run
        // by run, in the order they were pushed; a name no row carries finds
        // none, and a name is matched whole and byte for byte.
        let mut tabled = Tabled::default();
        for (table, row) in [
            ("b", 1),
            ("b", 2),
            ("a", 3),
            ("c", 4),
            ("b", 5),
            ("a", 6),
            ("a", 7),
            ("B", 8),
        ] {
            tabled.push(table, table.as_bytes(), row);
        }
        assert_eq!(tabled.runs.len(), 6);

        let grouped = ByTable::of(&tabled);
        let rows = |table: &str| -> Vec<i32> {
            grouped
                .rows(table.as_bytes())
                .into_iter()
                .map(|(_, row)| *row)
                .collect()
        };
        assert_eq!(rows("a"), [3, 6, 7]);
        assert_eq!(rows("b"), [1, 2, 5]);
        assert_eq!(rows("B"), [8]);
        assert_eq!(rows("c"), [4]);
        for none in ["", "d", "aa", " a"] {
            assert!(rows(none).is_empty(), "{none:?}");
        }
    }

    #[test]
    fn fr_lite_030_an_index_whose_members_came_apart_is_one_index_in_seqno_order() {
        // FR-LITE-030 over member rows in no order: the members of `ix_b`
        // come in three runs, interleaved with `ix_a`'s and in descending
        // `seqno`; each index is one index, its key members in ascending
        // `seqno`, a member that is not of the key left out, and the indexes
        // in order of name.
        let member = |seqno: i64, name: Option<&str>, descending: bool, key: bool| IndexMember {
            seqno,
            cid: seqno,
            name: name.map(str::to_owned),
            descending,
            key,
        };
        let mut read = Indexes::default();
        for (index, member) in [
            ("ix_b", member(2, Some("third"), false, true)),
            ("ix_a", member(1, Some("y"), true, true)),
            ("ix_b", member(1, Some("second"), true, true)),
            ("ix_a", member(0, Some("x"), false, true)),
            ("ix_b", member(3, None, false, false)),
            ("ix_b", member(0, Some("first"), false, true)),
        ] {
            read.push(
                ("t", b"t"),
                (index, index.as_bytes()),
                true,
                Cow::Borrowed("c"),
                member,
            );
        }
        // One table's run; five runs of an index, the last two members of
        // `ix_b` consecutive and so one run.
        assert_eq!(read.indexes.runs.len(), 1);
        assert_eq!(read.indexes.rows.len(), 5);

        let grouped = ByTable::of(&read.indexes);
        let mut entries = grouped.rows(b"t");
        entries.sort_by(|(_, left), (_, right)| left.name.cmp(&right.name));
        let built = indexes(&read, &entries);

        /// An index as the test reads it: its name, whether it is unique, and
        /// each member's name and direction.
        type Shown<'i> = (&'i str, bool, Vec<(&'i str, Option<SortDirection>)>);
        let shown: Vec<Shown<'_>> = built
            .iter()
            .map(|index| {
                (
                    &*index.name,
                    index.unique,
                    index
                        .columns
                        .iter()
                        .map(|column| (&*column.name, column.direction))
                        .collect(),
                )
            })
            .collect();
        assert_eq!(
            shown,
            [
                (
                    "ix_a",
                    true,
                    vec![
                        ("x", Some(SortDirection::Ascending)),
                        ("y", Some(SortDirection::Descending))
                    ]
                ),
                (
                    "ix_b",
                    true,
                    vec![
                        ("first", Some(SortDirection::Ascending)),
                        ("second", Some(SortDirection::Descending)),
                        ("third", Some(SortDirection::Ascending))
                    ]
                ),
            ]
        );
    }

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
    fn fr_lite_043_names_that_convert_alike_are_two_names_to_the_match() {
        // FR-LITE-043: the match is made on the bytes the file holds, so two
        // names that differ only in a byte that is not UTF-8 — both of which
        // convert to `r\u{fffd}f` — are two names, each found by its own
        // bytes and by its ASCII letters folded, and neither by the other's.
        let names = Names::of([b"r\xffF".as_slice(), b"r\xfeF"].into_iter());

        assert_eq!(names.position(b"r\xffF"), Some(0));
        assert_eq!(names.position(b"R\xfef"), Some(1));
        assert_eq!(names.position("r\u{fffd}F".as_bytes()), None);
        assert_eq!(names.position(b"r\xfdF"), None);
    }

    #[test]
    fn fr_lite_043_the_first_pair_that_converts_alike_is_found_and_no_other() {
        // FR-LITE-043: two names of one set whose texts are equal and whose
        // bytes differ are a pair; one name repeated with its own bytes — the
        // runs of one index — is not.
        let pair = super::first_alike(
            [
                ("b", b"b".as_slice(), ()),
                ("r\u{fffd}f", b"r\xfef".as_slice(), ()),
                ("a", b"a".as_slice(), ()),
                ("r\u{fffd}f", b"r\xfff".as_slice(), ()),
            ]
            .into_iter(),
        );
        let ((text, left, ()), (_, right, ())) = pair.expect("the pair is found");
        assert_eq!(text, "r\u{fffd}f");
        assert_eq!((left, right), (b"r\xfef".as_slice(), b"r\xfff".as_slice()));

        let repeated = super::first_alike(
            [
                ("ix", b"ix".as_slice(), ()),
                ("ix", b"ix".as_slice(), ()),
                ("r\u{fffd}f", b"r\xfff".as_slice(), ()),
            ]
            .into_iter(),
        );
        assert!(repeated.is_none());
    }

    #[test]
    fn br_lite_003_names_match_with_the_26_ascii_letters_folded_and_no_other_character() {
        let names = Names::of(
            ["Vessel", "port_call", "\u{c9}mbarque", "\u{e9}mbarque"]
                .map(str::as_bytes)
                .into_iter(),
        );

        // Folded over the whole name, in either direction.
        assert_eq!(names.position("Vessel".as_bytes()), Some(0));
        assert_eq!(names.position("VESSEL".as_bytes()), Some(0));
        assert_eq!(names.position("vessel".as_bytes()), Some(0));
        assert_eq!(names.position("vEsSeL".as_bytes()), Some(0));
        assert_eq!(names.position("PORT_CALL".as_bytes()), Some(1));
        // No character outside ASCII is folded: É and é are two names.
        assert_eq!(names.position("\u{c9}MBARQUE".as_bytes()), Some(2));
        assert_eq!(names.position("\u{e9}MBARQUE".as_bytes()), Some(3));
        assert_eq!(names.position("\u{c9}mbarque".as_bytes()), Some(2));
        // The whole name, and nothing near it.
        for other in ["Vesse", "Vessels", "v\u{e9}ssel", "port call", "", "berth"] {
            assert_eq!(names.position(other.as_bytes()), None, "{other}");
        }
        // A file whose schema loads holds no two names that fold alike; were
        // it given them, the first is kept.
        let clashing = Names::of([b"a".as_slice(), b"A"].into_iter());
        assert_eq!(clashing.position(b"A"), Some(0));
    }
}
