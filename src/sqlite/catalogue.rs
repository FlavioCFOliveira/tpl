//! The catalogue plan of a `sqlite` read, and the values it owns
//! (`FR-LITE-015`, `FR-LITE-037`, `FR-LITE-041`, `FR-LITE-042`).
//!
//! **The read is one read transaction.** `BEGIN DEFERRED` is issued
//! immediately before the check of `FR-LITE-041`, and `COMMIT` immediately
//! after the last catalogue statement, so that the check and every catalogue
//! statement read one snapshot of the file: a table or a view another
//! connection adds after the check is never read through (`FR-LITE-042`). A
//! read that ends early — the check refuses the file, a statement fails, a
//! deadline ends one — issues no `COMMIT`; the transaction ends with the
//! connection, which the library rolls back when it closes.
//!
//! **The plan is a constant**: the check, then six statements, fixed before
//! the first is sent, each joining `main.sqlite_schema` to the function it
//! reads through, so the count does not depend on the number of objects, and
//! no path adds a statement (`FR-LITE-037`, `NFR-PERF-001`). No statement
//! invokes `pragma_table_list`.
//!
//! **The read compiles no view and connects no virtual table.** A view is
//! read from statement 2, which names it, and statement 6, which holds its
//! stored text; no statement reads from a view or passes a view's or a virtual
//! table's name to a pragma function, so a view whose definition does not
//! compile is read as every other view is, and nothing is marked
//! (`FR-LITE-015`, `FR-LITE-033`, `FR-LITE-036`).
//!
//! **The read owns its values.** A row borrows its statement, so every value is
//! copied out as it is read, through its storage class and never coerced, a
//! SQL `NULL` kept apart from every value (`FR-LITE-004`, step 2); a field of
//! a pragma function of a class the library does not document for it is the
//! `70` of `FR-ERR-030`, and a value of the schema table of a class the check
//! refuses is the `78` of the check (`FR-LITE-040`, its paragraph on values).
//! Text is converted with U+FFFD substituted at this boundary and nowhere else
//! (`FR-OUT-017`).

pub(crate) mod statements;

pub(super) mod check;
mod fold;
mod trigger;

use std::path::PathBuf;

use rusqlite::Row;

use super::fault::Stage;
use super::{
    Misread, Session, integer, schema_rootpage, schema_sql, schema_text, text, text_or_null,
};
use crate::error::Error;
use crate::model::database::Database;
use crate::model::server::Server;

/// One row of `main.sqlite_schema` of type `table` or `view`.
#[derive(Debug, Clone)]
struct Object {
    /// Its name.
    name: String,
    /// Its type: `table` or `view`.
    kind: String,
    /// Its root page: `0` for a virtual table, which `FR-LITE-021` excludes.
    rootpage: Option<i64>,
    /// Whether `pragma_index_info` returned a row for its name: a
    /// `WITHOUT ROWID` table (`FR-LITE-020`).
    without_rowid: bool,
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

/// What a `sqlite` read produced: every value the plan read, owned.
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
}

/// How each statement of the read is named in a `cause`.
const BEGIN: &str = "the BEGIN DEFERRED of the read transaction";
/// See [`BEGIN`].
const COMMIT: &str = "the COMMIT of the read transaction";
/// See [`BEGIN`].
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

/// Reads the catalogue of the open session in one read transaction:
/// `BEGIN DEFERRED`, the check, the six statements of the plan, and `COMMIT`,
/// and no other statement (`FR-LITE-015`, `FR-LITE-017`, `FR-LITE-037`,
/// `FR-LITE-041`, `FR-LITE-042`).
///
/// # Errors
///
/// Returns the `78` of the check for a file it refuses, and the code
/// `FR-LITE-040` gives the first statement that fails. Either way no `COMMIT`
/// is issued, and the transaction ends with the connection.
pub(crate) fn read(session: &Session) -> Result<Catalogue, Error> {
    session.transaction(Stage::Transaction(BEGIN), statements::BEGIN)?;
    session.check()?;
    #[cfg(test)]
    race::after_check();

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
                kind: schema_text(row, 0, "type")?,
                name: schema_text(row, 1, "name")?,
                rootpage: schema_rootpage(row, 2)?,
                without_rowid: integer(row, 3)? != 0,
            })
        },
    )?;
    let columns = issue(
        session,
        Stage::Catalogue(TABLE_COLUMNS),
        statements::TABLE_COLUMNS,
        column_row,
    )?;
    let indexes = issue(
        session,
        Stage::Catalogue(INDEXES),
        statements::INDEXES,
        |row| {
            Ok(IndexRow {
                table: schema_text(row, 1, "name")?,
                index: text(row, 2)?,
                unique: integer(row, 3)? != 0,
                origin: text(row, 4)?,
                seqno: integer(row, 5)?,
                cid: integer(row, 6)?,
                name: text_or_null(row, 7)?,
                descending: integer(row, 8)? != 0,
                key: integer(row, 9)? != 0,
            })
        },
    )?;
    let keys = issue(
        session,
        Stage::Catalogue(FOREIGN_KEYS),
        statements::FOREIGN_KEYS,
        |row| {
            Ok(KeyRow {
                table: schema_text(row, 1, "name")?,
                id: integer(row, 2)?,
                seq: integer(row, 3)?,
                referenced: text(row, 4)?,
                from: text(row, 5)?,
                to: text_or_null(row, 6)?,
                on_update: text(row, 7)?,
                on_delete: text(row, 8)?,
                matched: text(row, 9)?,
            })
        },
    )?;
    let stored = issue(
        session,
        Stage::Catalogue(STORED),
        statements::STORED,
        |row| {
            Ok(StoredRow {
                kind: schema_text(row, 0, "type")?,
                name: schema_text(row, 1, "name")?,
                table: schema_text(row, 2, "tbl_name")?,
                sql: schema_sql(row, 3)?,
            })
        },
    )?;

    // Every catalogue statement completed: the transaction ends here, and on
    // no other path (FR-LITE-042).
    session.transaction(Stage::Transaction(COMMIT), statements::COMMIT)?;
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
    })
}

/// Runs one statement of the plan, counted under `FR-GLOB-017`.
fn issue<T>(
    session: &Session,
    stage: Stage,
    sql: &str,
    read: impl FnMut(&Row<'_>) -> Result<T, Misread>,
) -> Result<Vec<T>, Error> {
    let answered = session.rows(stage, sql, true, read);
    session.answered(stage, answered)
}

/// One row of statement 3: the table's type and name, then the fields of
/// `pragma_table_xinfo`.
fn column_row(row: &Row<'_>) -> Result<ColumnRow, Misread> {
    Ok(ColumnRow {
        table: schema_text(row, 1, "name")?,
        cid: integer(row, 2)?,
        name: text(row, 3)?,
        declared: text(row, 4)?,
        not_null: integer(row, 5)? != 0,
        default: text_or_null(row, 6)?,
        pk: integer(row, 7)?,
        hidden: integer(row, 8)?,
    })
}

/// A seam of the test configuration alone, between the check and the first
/// statement after it, where a test makes another connection write the file
/// (`FR-LITE-042`). It is compiled into no other build, like the statement
/// record of `FR-LITE-018` (`FR-ERR-031`, `OD-21`).
#[cfg(test)]
pub(crate) mod race {
    use std::cell::RefCell;

    thread_local! {
        /// What runs on this thread once the check has read.
        static AFTER_CHECK: RefCell<Option<Box<dyn FnOnce()>>> = RefCell::new(None);
    }

    /// Runs `writer` on this thread after the check of the next read, inside
    /// its read transaction.
    pub(crate) fn after_the_check(writer: impl FnOnce() + 'static) {
        AFTER_CHECK.with(|slot| *slot.borrow_mut() = Some(Box::new(writer)));
    }

    /// Runs what [`after_the_check`] set, once.
    pub(super) fn after_check() {
        let writer = AFTER_CHECK.with(|slot| slot.borrow_mut().take());
        if let Some(writer) = writer {
            writer();
        }
    }
}
