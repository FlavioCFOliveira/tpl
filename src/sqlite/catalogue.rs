//! The catalogue plan of a `sqlite` read, the values it owns, and the per-view
//! fallback (`FR-LITE-015`, `FR-LITE-035`, `FR-LITE-037`).
//!
//! **The plan is a constant**: seven statements follow the probe, fixed before
//! the first is sent, each joining `pragma_table_list` to the function it reads
//! through, so the count does not depend on the number of objects
//! (`FR-LITE-037`, `NFR-PERF-001`). The one path that adds statements is the
//! fallback of `FR-LITE-035`: where the statement reading every view's columns
//! fails with `SQLITE_ERROR`, one statement per view reads that view's
//! columns, so that exactly the views that do not compile are marked.
//!
//! **The read owns its values.** A row borrows its statement, so every value is
//! copied out as it is read, through its storage class and never coerced, a
//! SQL `NULL` kept apart from every value (`FR-LITE-004`, step 2); a field of
//! a class the library does not document for it is the `70` of `FR-ERR-030`.
//! Text is converted with U+FFFD substituted at this boundary and nowhere else
//! (`FR-OUT-017`).
//!
//! **A view's columns are read and not carried.** Statement 7 is what
//! `FR-LITE-035` decides a view's completeness by, and what the count of
//! `FR-LITE-037` includes; the model's view has no field for its columns
//! (`FR-CAT-047`), and the document contract gives it none, so what the
//! statement returns is validated and dropped. That the specification asks
//! for the columns to be carried as well is reported against it rather than
//! implemented against the document contract.

pub(crate) mod statements;

mod fold;
mod trigger;

use std::path::PathBuf;

use rusqlite::Row;

use super::fault::{self, Class, Stage};
use super::{Failed, Session, integer, text, text_or_null};
use crate::error::{Error, LibraryReport};
use crate::model::database::Database;
use crate::model::document::shape::UnreadableView;
use crate::model::server::Server;

/// One object `pragma_table_list` reports in `main`.
#[derive(Debug, Clone)]
struct Object {
    /// Its name.
    name: String,
    /// Its type: `table`, `view`, `virtual`, `shadow`, or another the carried
    /// library reports.
    kind: String,
    /// Whether it is a `WITHOUT ROWID` table — the `wr` field.
    without_rowid: bool,
    /// Whether it is a `STRICT` table.
    strict: bool,
}

/// One row of `pragma_table_xinfo`, for a table.
#[derive(Debug, Clone)]
struct ColumnRow {
    /// The table it belongs to.
    table: String,
    /// The `cid` field: the declaration order, from zero.
    cid: i64,
    /// The `name` field.
    name: String,
    /// The `type` field, exactly as stored.
    declared: String,
    /// The `notnull` field.
    not_null: bool,
    /// The `dflt_value` field, SQL `NULL` kept apart.
    default: Option<String>,
    /// The `pk` field.
    pk: i64,
    /// The `hidden` field.
    hidden: i64,
}

/// One row of `pragma_index_list` joined to `pragma_index_xinfo`.
#[derive(Debug, Clone)]
struct IndexRow {
    /// The table it belongs to.
    table: String,
    /// The index's name.
    index: String,
    /// The `unique` field.
    unique: bool,
    /// The `origin` field: `c`, `u` or `pk`.
    origin: String,
    /// The member's `seqno`.
    seqno: i64,
    /// The member's `cid`: `-1` for the rowid, `-2` for an expression.
    cid: i64,
    /// The member's `name`, SQL `NULL` for the rowid and an expression.
    name: Option<String>,
    /// The `desc` field.
    descending: bool,
    /// The `key` field: whether the row is a member of the key.
    key: bool,
}

/// One row of `pragma_foreign_key_list`.
#[derive(Debug, Clone)]
struct KeyRow {
    /// The referencing table.
    table: String,
    /// The `id` field, one value per key.
    id: i64,
    /// The `seq` field, the position of the pair.
    seq: i64,
    /// The `table` field: the referenced table, as written.
    referenced: String,
    /// The `from` field.
    from: String,
    /// The `to` field, SQL `NULL` where the key names no parent column.
    to: Option<String>,
    /// The `on_update` field.
    on_update: String,
    /// The `on_delete` field.
    on_delete: String,
    /// The `match` field.
    matched: String,
}

/// One row of `sqlite_schema` of type `view` or `trigger`.
#[derive(Debug, Clone)]
struct StoredRow {
    /// `view` or `trigger`.
    kind: String,
    /// The object's name.
    name: String,
    /// The table or view it is on, for a trigger; itself, for a view.
    table: String,
    /// The stored `CREATE` statement.
    sql: Option<String>,
}

/// What a `sqlite` read produced: every value the plan read, owned, and the
/// views whose columns could not be read with the library's reason
/// (`FR-LITE-035`).
#[derive(Debug)]
pub(crate) struct Catalogue {
    /// The entry the read was made through.
    entry: String,
    /// The database file, as resolved.
    path: PathBuf,
    /// The library that answered the probe.
    server: Server<'static>,
    /// The encoding of `main`.
    encoding: String,
    /// The objects of `main`.
    objects: Vec<Object>,
    /// The columns of every table.
    columns: Vec<ColumnRow>,
    /// The indexes of every table, member by member.
    indexes: Vec<IndexRow>,
    /// The foreign keys of every table, pair by pair.
    keys: Vec<KeyRow>,
    /// The stored text of every view and trigger.
    stored: Vec<StoredRow>,
    /// The views whose columns could not be read, and why.
    unreadable: Vec<UnreadableView>,
}

impl Catalogue {
    /// The model this read yields (`FR-LITE-019` … `FR-LITE-034`).
    ///
    /// # Errors
    ///
    /// Returns the `70` of `FR-LITE-032` for a trigger header that yields no
    /// one timing and one event, and of `FR-CAT-044` where a key the fold
    /// built names a column its table does not carry.
    pub(crate) fn model(&self) -> Result<Database<'static>, Error> {
        fold::database(self)
    }

    /// The views whose columns could not be read, each with the reason the
    /// library gave (`FR-LITE-035`).
    pub(crate) fn unreadable(&self) -> &[UnreadableView] {
        &self.unreadable
    }
}

/// How each catalogue statement is named in a `cause`.
const ENCODING: &str = "the catalogue statement reading the encoding of main";
/// See [`ENCODING`].
const OBJECTS: &str = "the catalogue statement reading the objects of main";
/// See [`ENCODING`].
const TABLE_COLUMNS: &str = "the catalogue statement reading the columns of every table";
/// See [`ENCODING`].
const INDEXES: &str = "the catalogue statement reading the indexes of every table";
/// See [`ENCODING`].
const FOREIGN_KEYS: &str = "the catalogue statement reading the foreign keys of every table";
/// See [`ENCODING`].
const STORED: &str = "the catalogue statement reading the stored text of every view and trigger";

/// Reads the catalogue of the open session: the seven statements of the plan,
/// and the per-view fallback where the seventh fails (`FR-LITE-015`,
/// `FR-LITE-035`, `FR-LITE-037`).
///
/// # Errors
///
/// Returns the code `FR-LITE-040` gives the first statement that fails, other
/// than a view that does not compile.
pub(crate) fn read(session: &Session) -> Result<Catalogue, Error> {
    let encoding = issue(
        session,
        Stage::Catalogue(ENCODING),
        statements::ENCODING,
        |row| text(row, 0),
    )?;
    let objects = issue(
        session,
        Stage::Catalogue(OBJECTS),
        statements::OBJECTS,
        |row| {
            Ok(Object {
                name: text(row, 0)?,
                kind: text(row, 1)?,
                without_rowid: integer(row, 2)? != 0,
                strict: integer(row, 3)? != 0,
            })
        },
    )?;
    let columns = issue(
        session,
        Stage::Catalogue(TABLE_COLUMNS),
        statements::TABLE_COLUMNS,
        |row| column_row(row, Some(0)),
    )?;
    let indexes = issue(
        session,
        Stage::Catalogue(INDEXES),
        statements::INDEXES,
        |row| {
            Ok(IndexRow {
                table: text(row, 0)?,
                index: text(row, 1)?,
                unique: integer(row, 2)? != 0,
                origin: text(row, 3)?,
                seqno: integer(row, 4)?,
                cid: integer(row, 5)?,
                name: text_or_null(row, 6)?,
                descending: integer(row, 7)? != 0,
                key: integer(row, 8)? != 0,
            })
        },
    )?;
    let keys = issue(
        session,
        Stage::Catalogue(FOREIGN_KEYS),
        statements::FOREIGN_KEYS,
        |row| {
            Ok(KeyRow {
                table: text(row, 0)?,
                id: integer(row, 1)?,
                seq: integer(row, 2)?,
                referenced: text(row, 3)?,
                from: text(row, 4)?,
                to: text_or_null(row, 5)?,
                on_update: text(row, 6)?,
                on_delete: text(row, 7)?,
                matched: text(row, 8)?,
            })
        },
    )?;
    let stored = issue(
        session,
        Stage::Catalogue(STORED),
        statements::STORED,
        |row| {
            Ok(StoredRow {
                kind: text(row, 0)?,
                name: text(row, 1)?,
                table: text(row, 2)?,
                sql: text_or_null(row, 3)?,
            })
        },
    )?;
    let unreadable = views(session, &objects)?;
    let (entry, path) = session.origin();

    Ok(Catalogue {
        entry: entry.to_owned(),
        path: path.to_owned(),
        server: session.server().clone(),
        encoding: encoding.into_iter().next().unwrap_or_default(),
        objects,
        columns,
        indexes,
        keys,
        stored,
        unreadable,
    })
}

/// Statement 7, and the per-view fallback where it fails (`FR-LITE-035`).
///
/// The views' columns are validated and dropped, for the reason this module
/// states; what is kept is which views could not be read, and why.
fn views(session: &Session, objects: &[Object]) -> Result<Vec<UnreadableView>, Error> {
    let every = session.rows(Stage::Views, statements::VIEW_COLUMNS, &[], true, |row| {
        column_row(row, Some(0)).map(drop)
    });

    match every {
        Ok(_) => return Ok(Vec::new()),
        Err(Failed::Library(error, timing)) if marks(Stage::Views, &error) => {
            // The single statement failed because a view does not compile:
            // every view is read on its own, so exactly those that fail are
            // marked (FR-PRIV-006, FR-PRIV-007).
            let _ = timing;
        }
        Err(failed) => {
            return session
                .answered(Stage::Views, Err::<Vec<()>, _>(failed))
                .map(|_| Vec::new());
        }
    }

    let mut unreadable = Vec::new();
    for view in objects.iter().filter(|object| object.kind == VIEW) {
        let one = session.rows(
            Stage::View,
            statements::ONE_VIEW_COLUMNS,
            &[&view.name],
            true,
            |row| column_row(row, None).map(drop),
        );

        match one {
            Ok(_) => {}
            Err(Failed::Library(error, _)) if marks(Stage::View, &error) => {
                if let Some(reason) = fault::report(&error) {
                    unreadable.push(UnreadableView {
                        name: view.name.clone(),
                        reason,
                    });
                }
            }
            Err(failed) => {
                session.answered(Stage::View, Err::<Vec<()>, _>(failed))?;
            }
        }
    }

    Ok(unreadable)
}

/// Whether a failure at `stage` marks a view rather than failing the read.
fn marks(stage: Stage, error: &rusqlite::Error) -> bool {
    fault::report(error)
        .is_some_and(|report: LibraryReport| fault::classify(stage, report.code) == Class::MarkView)
}

/// The type `pragma_table_list` gives a view.
const VIEW: &str = "view";

/// Runs one statement of the plan, counted under `FR-GLOB-017`.
fn issue<T>(
    session: &Session,
    stage: Stage,
    sql: &str,
    read: impl FnMut(&Row<'_>) -> Result<T, &'static str>,
) -> Result<Vec<T>, Error> {
    let answered = session.rows(stage, sql, &[], true, read);
    session.answered(stage, answered)
}

/// One row of `pragma_table_xinfo`; `table` is the position of the table's
/// name, which the per-view statement does not select.
fn column_row(row: &Row<'_>, table: Option<usize>) -> Result<ColumnRow, &'static str> {
    let (owner, from) = match table {
        Some(at) => (text(row, at)?, at + 1),
        None => (String::new(), 0),
    };

    Ok(ColumnRow {
        table: owner,
        cid: integer(row, from)?,
        name: text(row, from + 1)?,
        declared: text(row, from + 2)?,
        not_null: integer(row, from + 3)? != 0,
        default: text_or_null(row, from + 4)?,
        pk: integer(row, from + 5)?,
        hidden: integer(row, from + 6)?,
    })
}
