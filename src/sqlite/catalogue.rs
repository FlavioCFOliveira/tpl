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
//! read as its row is stepped, through its storage class and never coerced, a
//! SQL `NULL` kept apart from every value (`FR-LITE-004`, step 2); a field of
//! a pragma function of a class the library does not document for it is the
//! `70` of `FR-ERR-030`, and a value of the schema table of a class the check
//! refuses is the `78` of the check (`FR-LITE-040`, its paragraph on values).
//! Text is converted with U+FFFD substituted at this boundary and nowhere else
//! (`FR-OUT-017`). PERF: what is kept is copied out once: a table's name once
//! per run of its consecutive rows ([`Tabled`]), an index's fields once per run
//! of its consecutive members ([`Indexes`]), and a field whose values come
//! from a closed set of spellings as that spelling, with no copy; the model
//! then borrows from these values ([`Catalogue::model`]).
//!
//! **No order is taken from the statements.** No statement carries an
//! `ORDER BY`, so the rows come in whatever order the library's plan yields
//! them; every order the model carries is produced by the fold (`fold`).

pub(crate) mod statements;

pub(super) mod check;
mod fold;
mod trigger;

use std::borrow::Cow;
use std::ops::Range;
use std::path::PathBuf;

use rusqlite::Row;

use super::fault::Stage;
use super::{
    Misread, Session, integer, schema_named, schema_rootpage, schema_sql, schema_text, schema_type,
    spelled, text, text_named, text_or_null,
};
use crate::error::Error;
use crate::model::database::Database;
use crate::model::server::Server;

/// The spellings of the `origin` field of `pragma_index_list`, as
/// `FR-LITE-030` records them.
const ORIGINS: [&str; 3] = ["c", "u", "pk"];

/// The spellings of the `on_update` and `on_delete` fields of
/// `pragma_foreign_key_list`, as `FR-LITE-031` records them.
const ACTIONS: [&str; 5] = [
    "NO ACTION",
    "RESTRICT",
    "SET NULL",
    "SET DEFAULT",
    "CASCADE",
];

/// The spelling of the `match` field of `pragma_foreign_key_list`, as
/// `FR-LITE-031` records it.
const MATCHES: [&str; 1] = ["NONE"];

/// The bytes a name was converted from, kept only where the conversion
/// replaced invalid UTF-8 (`FR-OUT-017`); elsewhere the name's text is those
/// bytes.
///
/// Ordered by these bytes, names fall in the order SQLite's `BINARY`
/// collation gives a `UTF-8` file, the order the statements' `ORDER BY` gave
/// before it was removed: two names that differ only in invalid UTF-8 convert
/// to one text, and only their bytes still tell their order (`fold`, *Every
/// order is produced here*). A `UTF-16` file cannot hold two such names: the
/// library converts its names to valid UTF-8 as it loads the schema, and
/// refuses two that convert alike.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
struct Raw(Option<Box<[u8]>>);

impl Raw {
    /// The bytes `text` was converted from, kept where they differ from it.
    fn of(text: &str, bytes: &[u8]) -> Self {
        Self((text.as_bytes() != bytes).then(|| Box::from(bytes)))
    }

    /// The bytes of the name whose text is `text`.
    fn bytes<'n>(&'n self, text: &'n str) -> &'n [u8] {
        self.0.as_deref().unwrap_or(text.as_bytes())
    }
}

/// One row of `main.sqlite_schema` of type `table` or `view`.
#[derive(Debug, Clone)]
struct Object {
    /// Its name.
    name: String,
    /// The bytes its name was converted from, where they differ from it.
    raw: Raw,
    /// Its type: `table` or `view`.
    kind: Cow<'static, str>,
    /// Its root page: `0` for a virtual table, which `FR-LITE-021` excludes.
    rootpage: Option<i64>,
    /// Whether `pragma_index_info` returned a row for its name: a
    /// `WITHOUT ROWID` table (`FR-LITE-020`).
    without_rowid: bool,
}

/// The rows one statement returned for the tables it joined to, and the table
/// each belongs to.
///
/// A statement joins each row of the schema table to the rows a pragma
/// function returns for it, so the rows of one table come consecutively, and
/// each run of them carries its table's name once rather than on every row.
/// Nothing relies on that: a table whose rows came apart has a run for each
/// part, and the fold groups the runs by name (`fold`).
#[derive(Debug)]
struct Tabled<T> {
    /// Every row, in the order the statement returned them.
    rows: Vec<T>,
    /// Each run of consecutive rows of one table.
    runs: Vec<Run>,
}

/// A run of consecutive rows of [`Tabled`] that belong to one table.
#[derive(Debug, Clone)]
struct Run {
    /// The table's name, the schema table's `name` field.
    table: String,
    /// The bytes that name was converted from, where they differ from it.
    raw: Raw,
    /// Where the run's rows stand among the statement's.
    rows: Range<usize>,
}

impl Run {
    /// The bytes of the table's name, as the file holds them.
    fn key(&self) -> &[u8] {
        self.raw.bytes(&self.table)
    }
}

impl Object {
    /// The bytes of its name, as the file holds them.
    fn key(&self) -> &[u8] {
        self.raw.bytes(&self.name)
    }
}

impl<T> Default for Tabled<T> {
    fn default() -> Self {
        Self {
            rows: Vec::new(),
            runs: Vec::new(),
        }
    }
}

impl<T> Tabled<T> {
    /// Adds `row`, of the table named `table`, converted from `bytes`, to the
    /// last run where that run is the same table's, and to a run of its own
    /// otherwise. Two tables are one where their bytes are, so two names that
    /// convert alike keep runs apart.
    fn push(&mut self, table: &str, bytes: &[u8], row: T) {
        let at = self.rows.len();
        self.rows.push(row);
        match self.runs.last_mut() {
            Some(run) if run.key() == bytes => run.rows.end = at + 1,
            _ => self.runs.push(Run {
                table: table.to_owned(),
                raw: Raw::of(table, bytes),
                rows: at..at + 1,
            }),
        }
    }

    /// The last row, where it belongs to the table named by `bytes`.
    fn last_of(&mut self, bytes: &[u8]) -> Option<&mut T> {
        match self.runs.last() {
            Some(run) if run.key() == bytes => self.rows.last_mut(),
            _ => None,
        }
    }

    /// The rows of `run`.
    fn rows_of(&self, run: &Run) -> &[T] {
        self.rows.get(run.rows.clone()).unwrap_or_default()
    }
}

/// One row of `pragma_table_xinfo`, for a table.
#[derive(Debug, Clone)]
struct ColumnRow {
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

/// The rows of `pragma_index_list` joined to `pragma_index_xinfo`: the
/// fields of each index once per run of its members, and every member.
#[derive(Debug, Default)]
struct Indexes {
    /// The runs of consecutive members of one index, by table.
    indexes: Tabled<IndexRun>,
    /// Every member row, in the order the statement returned them.
    members: Vec<IndexMember>,
}

/// A run of consecutive rows of one index that agree on the fields of
/// `pragma_index_list`, which the join repeats on each of its members.
#[derive(Debug, Clone)]
struct IndexRun {
    /// The index's name.
    name: String,
    /// The bytes that name was converted from, where they differ from it.
    raw: Raw,
    /// The `unique` field.
    unique: bool,
    /// The `origin` field: `c`, `u` or `pk`.
    origin: Cow<'static, str>,
    /// Where its members stand among [`Indexes::members`].
    members: Range<usize>,
}

/// The fields of `pragma_index_xinfo` of one row: one member of an index.
#[derive(Debug, Clone)]
struct IndexMember {
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

impl IndexRun {
    /// The bytes of the index's name, as the file holds them.
    fn key(&self) -> &[u8] {
        self.raw.bytes(&self.name)
    }
}

impl Indexes {
    /// Adds the member row of index `index` of `table`, each a name and the
    /// bytes it was converted from, to the last run where it is of the same
    /// index and agrees with it, and to a run of its own otherwise.
    fn push(
        &mut self,
        (table, table_bytes): (&str, &[u8]),
        (index, index_bytes): (&str, &[u8]),
        unique: bool,
        origin: Cow<'static, str>,
        member: IndexMember,
    ) {
        let at = self.members.len();
        self.members.push(member);
        if let Some(run) = self.indexes.last_of(table_bytes)
            && run.key() == index_bytes
            && run.unique == unique
            && run.origin == origin
        {
            run.members.end = at + 1;
            return;
        }
        self.indexes.push(
            table,
            table_bytes,
            IndexRun {
                name: index.to_owned(),
                raw: Raw::of(index, index_bytes),
                unique,
                origin,
                members: at..at + 1,
            },
        );
    }

    /// The members of `run`.
    fn members_of(&self, run: &IndexRun) -> &[IndexMember] {
        self.members.get(run.members.clone()).unwrap_or_default()
    }
}

/// One row of `pragma_foreign_key_list`, for a table.
#[derive(Debug, Clone)]
struct KeyRow {
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
    on_update: Cow<'static, str>,
    /// The `on_delete` field.
    on_delete: Cow<'static, str>,
    /// The `match` field.
    matched: Cow<'static, str>,
}

/// One row of `sqlite_schema` of type `view` or `trigger`.
#[derive(Debug, Clone)]
struct StoredRow {
    /// `view` or `trigger`.
    kind: Cow<'static, str>,
    /// The object's name.
    name: String,
    /// The bytes that name was converted from, where they differ from it.
    raw: Raw,
    /// The table or view it is on, for a trigger; itself, for a view.
    table: String,
    /// The stored `CREATE` statement.
    sql: Option<String>,
}

impl StoredRow {
    /// The bytes of the object's name, as the file holds them.
    fn key(&self) -> &[u8] {
        self.raw.bytes(&self.name)
    }
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
    columns: Tabled<ColumnRow>,
    /// The indexes of every table, member by member.
    indexes: Indexes,
    /// The foreign keys of every table, pair by pair.
    keys: Tabled<KeyRow>,
    /// The stored text of every view and trigger.
    stored: Vec<StoredRow>,
}

impl Catalogue {
    /// The model this read yields (`FR-LITE-019` … `FR-LITE-034`).
    ///
    /// PERF: the model borrows every name and text it carries from the rows
    /// of this read rather than copying them, so it lives no longer than the
    /// read; `provider::Read::model` already hands out a model that borrows
    /// its read.
    ///
    /// # Errors
    ///
    /// Returns the `70` of `FR-LITE-032` for a trigger header that yields no
    /// one timing and one event, and of `FR-CAT-044` where a key the fold
    /// built names a column its table does not carry.
    pub(crate) fn model(&self) -> Result<Database<'_>, Error> {
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
            let kind = schema_type(row, 0)?;
            let (name, bytes) = schema_named(row, 1, "name")?;
            Ok(Object {
                raw: Raw::of(&name, bytes),
                name: name.into_owned(),
                kind,
                rootpage: schema_rootpage(row, 2)?,
                without_rowid: integer(row, 3)? != 0,
            })
        },
    )?;
    let mut columns = Tabled::default();
    fill(
        session,
        Stage::Catalogue(TABLE_COLUMNS),
        statements::TABLE_COLUMNS,
        |row| {
            let (table, bytes) = schema_named(row, 1, "name")?;
            columns.push(&table, bytes, column_row(row)?);
            Ok(())
        },
    )?;
    let mut indexes = Indexes::default();
    fill(
        session,
        Stage::Catalogue(INDEXES),
        statements::INDEXES,
        |row| {
            let (table, table_bytes) = schema_named(row, 1, "name")?;
            let (index, index_bytes) = text_named(row, 2)?;
            let unique = integer(row, 3)? != 0;
            let origin = spelled(row, 4, &ORIGINS)?;
            let member = IndexMember {
                seqno: integer(row, 5)?,
                cid: integer(row, 6)?,
                name: text_or_null(row, 7)?,
                descending: integer(row, 8)? != 0,
                key: integer(row, 9)? != 0,
            };
            indexes.push(
                (&table, table_bytes),
                (&index, index_bytes),
                unique,
                origin,
                member,
            );
            Ok(())
        },
    )?;
    let mut keys = Tabled::default();
    fill(
        session,
        Stage::Catalogue(FOREIGN_KEYS),
        statements::FOREIGN_KEYS,
        |row| {
            let (table, bytes) = schema_named(row, 1, "name")?;
            let key = KeyRow {
                id: integer(row, 2)?,
                seq: integer(row, 3)?,
                referenced: text(row, 4)?,
                from: text(row, 5)?,
                to: text_or_null(row, 6)?,
                on_update: spelled(row, 7, &ACTIONS)?,
                on_delete: spelled(row, 8, &ACTIONS)?,
                matched: spelled(row, 9, &MATCHES)?,
            };
            keys.push(&table, bytes, key);
            Ok(())
        },
    )?;
    let stored = issue(
        session,
        Stage::Catalogue(STORED),
        statements::STORED,
        |row| {
            let kind = schema_type(row, 0)?;
            let (name, bytes) = schema_named(row, 1, "name")?;
            Ok(StoredRow {
                raw: Raw::of(&name, bytes),
                name: name.into_owned(),
                kind,
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

/// Runs one statement of the plan, counted under `FR-GLOB-017`, handing
/// every row to `read`, which keeps what it copies out.
fn fill(
    session: &Session,
    stage: Stage,
    sql: &str,
    read: impl FnMut(&Row<'_>) -> Result<(), Misread>,
) -> Result<(), Error> {
    let answered = session.each(stage, sql, true, read);
    session.answered(stage, answered)
}

/// The fields of `pragma_table_xinfo` of one row of statement 3, whose
/// table's type and name precede them.
fn column_row(row: &Row<'_>) -> Result<ColumnRow, Misread> {
    Ok(ColumnRow {
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

#[cfg(test)]
mod tests {
    //! The fold over rows returned in an order no statement states: the
    //! statements carry no `ORDER BY`, so every order the model carries must
    //! come from the fold, and grouping must hold whatever order the rows
    //! arrive in.

    use std::path::Path;

    use rusqlite::config::DbConfig;
    use rusqlite::{Connection, OpenFlags};

    use super::{Catalogue, IndexMember, IndexRun, Indexes, Run, Tabled};
    use crate::deadline::Deadlines;
    use crate::model::database::Database;
    use crate::model::trigger::TriggerEvent;
    use crate::project::scratch::Scratch;
    use crate::project::settings::{self, FileSettings};
    use crate::sqlite::fixture;

    /// The seeds each test permutes the rows with.
    const SEEDS: [u64; 5] = [1, 2, 3, 0x5eed, 0xdead_beef];

    /// A permutation of `0..len`, drawn from `seed` by xorshift64* and the
    /// Fisher–Yates shuffle, so that a failure reproduces.
    fn permutation(len: usize, seed: u64) -> Vec<usize> {
        let mut state = seed.max(1);
        let mut next = || {
            state ^= state >> 12;
            state ^= state << 25;
            state ^= state >> 27;
            state.wrapping_mul(0x2545_f491_4f6c_dd1d)
        };
        let mut order: Vec<usize> = (0..len).collect();
        for last in (1..len).rev() {
            let bound = u64::try_from(last + 1).expect("a length fits in u64");
            let pick = usize::try_from(next() % bound).expect("below a length");
            order.swap(last, pick);
        }
        order
    }

    /// The rows of `tabled`, each with its table, pushed again in the order
    /// `seed` draws: the rows of one table come apart into many runs.
    fn shuffled_tabled<T: Clone>(tabled: &Tabled<T>, seed: u64) -> Tabled<T> {
        let flat: Vec<(&Run, &T)> = tabled
            .runs
            .iter()
            .flat_map(|run| tabled.rows_of(run).iter().map(move |row| (run, row)))
            .collect();
        let mut shuffled = Tabled::default();
        for at in permutation(flat.len(), seed) {
            let (run, row) = flat[at];
            shuffled.push(&run.table, run.key(), row.clone());
        }
        shuffled
    }

    /// `catalogue` with every collection of rows in the order `seed` draws.
    fn shuffled(catalogue: &Catalogue, seed: u64) -> Catalogue {
        let members: Vec<(&Run, &IndexRun, &IndexMember)> = catalogue
            .indexes
            .indexes
            .runs
            .iter()
            .flat_map(|run| {
                catalogue
                    .indexes
                    .indexes
                    .rows_of(run)
                    .iter()
                    .flat_map(move |index| {
                        catalogue
                            .indexes
                            .members_of(index)
                            .iter()
                            .map(move |member| (run, index, member))
                    })
            })
            .collect();
        let mut indexes = Indexes::default();
        for at in permutation(members.len(), seed) {
            let (run, index, member) = members[at];
            indexes.push(
                (&run.table, run.key()),
                (&index.name, index.key()),
                index.unique,
                index.origin.clone(),
                member.clone(),
            );
        }
        let objects = permutation(catalogue.objects.len(), seed)
            .into_iter()
            .map(|at| catalogue.objects[at].clone())
            .collect();
        let stored = permutation(catalogue.stored.len(), seed)
            .into_iter()
            .map(|at| catalogue.stored[at].clone())
            .collect();

        Catalogue {
            entry: catalogue.entry.clone(),
            path: catalogue.path.clone(),
            server: catalogue.server.clone(),
            encoding: catalogue.encoding.clone(),
            objects,
            columns: shuffled_tabled(&catalogue.columns, seed),
            indexes,
            keys: shuffled_tabled(&catalogue.keys, seed),
            stored,
        }
    }

    /// The catalogue of the file at `path`.
    fn read(path: &Path) -> Catalogue {
        let clock = settings::clock(None);
        let session = crate::sqlite::open(
            &FileSettings::new("ledger", path, Deadlines::default()),
            &clock,
        )
        .expect("the file opens");
        let read = super::read(&session);
        session.close();
        read.expect("the file reads")
    }

    /// The texts a query of one parameter returns, in its own order.
    fn texts(connection: &Connection, sql: &str, parameter: &str) -> Vec<String> {
        let mut statement = connection.prepare(sql).expect("the oracle prepares");
        statement
            .query_map([parameter], |row| row.get(0))
            .expect("the oracle runs")
            .collect::<Result<_, _>>()
            .expect("every value is text")
    }

    /// Asserts every order the specification requires of `database`, each
    /// against the library's own answer, ordered by the oracle's `ORDER BY`
    /// on a connection of its own.
    fn assert_ordered(database: &Database<'_>, connection: &Connection, seed: u64) {
        // NFR-DET-002: the tables and the views by name, byte-wise.
        assert!(
            database
                .tables
                .windows(2)
                .all(|pair| pair[0].name().as_bytes() < pair[1].name().as_bytes()),
            "seed {seed}: tables"
        );
        assert!(
            database
                .views
                .windows(2)
                .all(|pair| pair[0].name.as_bytes() < pair[1].name.as_bytes()),
            "seed {seed}: views"
        );

        for table in &database.tables {
            let name = table.name();

            // FR-LITE-025: the columns in ascending order of `cid`.
            let columns: Vec<&str> = table.columns().iter().map(|column| &*column.name).collect();
            let expected = texts(
                connection,
                "SELECT name FROM pragma_table_xinfo(?1) ORDER BY cid",
                name,
            );
            assert_eq!(columns, expected, "seed {seed}: the columns of {name}");
            for (at, column) in table.columns().iter().enumerate() {
                assert_eq!(
                    column.position,
                    u64::try_from(at + 1).expect("a position fits"),
                    "seed {seed}: {name}"
                );
            }

            // FR-LITE-029: the primary key's members in ascending order of `pk`.
            if let Some(key) = table.primary_key() {
                let members: Vec<&str> = key.columns.iter().map(|member| &*member.name).collect();
                let expected = texts(
                    connection,
                    "SELECT name FROM pragma_table_xinfo(?1) WHERE pk > 0 ORDER BY pk",
                    name,
                );
                assert_eq!(members, expected, "seed {seed}: the key of {name}");
            }

            // FR-LITE-030: each index's members in ascending order of `seqno`,
            // and the indexes by name.
            for index in table.indexes() {
                let members: Vec<&str> = index.columns.iter().map(|member| &*member.name).collect();
                let expected = texts(
                    connection,
                    "SELECT name FROM pragma_index_xinfo(?1) WHERE key = 1 ORDER BY seqno",
                    &index.name,
                );
                assert_eq!(members, expected, "seed {seed}: index {}", index.name);
            }
            assert!(
                table
                    .indexes()
                    .windows(2)
                    .all(|pair| pair[0].name.as_bytes() < pair[1].name.as_bytes()),
                "seed {seed}: the indexes of {name}"
            );
            assert!(
                table
                    .triggers()
                    .windows(2)
                    .all(|pair| pair[0].name.as_bytes() < pair[1].name.as_bytes()),
                "seed {seed}: the triggers of {name}"
            );

            // FR-LITE-031: each key's pairs in ascending order of `seq`: its
            // referencing columns are one key's `from` fields in that order.
            let mut statement = connection
                .prepare(
                    "SELECT id, group_concat(\"from\", ',') FROM \
                     (SELECT id, seq, \"from\" FROM pragma_foreign_key_list(?1) ORDER BY id, seq) \
                     GROUP BY id",
                )
                .expect("the oracle prepares");
            let declared: Vec<String> = statement
                .query_map([name], |row| row.get(1))
                .expect("the oracle runs")
                .collect::<Result<_, _>>()
                .expect("every value is text");
            for key in table.foreign_keys() {
                let from: Vec<&str> = key.columns.iter().map(|pair| &*pair.column).collect();
                assert!(
                    declared.contains(&from.join(",")),
                    "seed {seed}: a key of {name} pairs {from:?}, the library {declared:?}"
                );
            }
        }
    }

    #[test]
    fn fr_lite_025_029_030_031_the_fold_orders_rows_returned_in_any_order() {
        // The statements state no order, so the fold is given the rows of the
        // main fixture database in five drawn orders, a table's rows and an
        // index's members coming apart into many runs: the model folded is
        // the one the rows as returned fold to, and every order the
        // specification requires holds, each checked against the library.
        let scratch = Scratch::new();
        let artefacts = fixture::build(&scratch.directory("fixture"));
        let catalogue = read(&artefacts.main);
        let expected = catalogue.model().expect("the rows as returned fold");
        let connection =
            Connection::open_with_flags(&artefacts.main, OpenFlags::SQLITE_OPEN_READ_ONLY)
                .expect("the oracle's connection opens");
        assert_ordered(&expected, &connection, 0);

        for seed in SEEDS {
            let shuffled = shuffled(&catalogue, seed);
            // The shuffle splits the rows of a table across runs.
            assert!(shuffled.columns.runs.len() > catalogue.columns.runs.len());
            assert!(shuffled.indexes.indexes.runs.len() > catalogue.indexes.indexes.runs.len());
            assert!(shuffled.keys.runs.len() > catalogue.keys.runs.len());

            let database = shuffled.model().expect("the shuffled rows fold");
            assert_ordered(&database, &connection, seed);
            assert_eq!(database, expected, "seed {seed}");
        }
    }

    /// The objects of [`colliding`], each created under an ASCII name and then
    /// renamed, under `writable_schema`, to the bytes beside it.
    const RENAMED: [(&str, &str); 8] = [
        ("tbl_ff", "74ff"),
        ("tbl_fe", "74fe"),
        ("idx_ff", "69ff"),
        ("idx_fe", "69fe"),
        ("trg_ff", "67ff"),
        ("trg_fe", "67fe"),
        ("vw_ff", "76ff"),
        ("vw_fe", "76fe"),
    ];

    /// Builds, at `path`, a database whose tables `t\xFF` and `t\xFE`, their
    /// indexes `i\xFF` and `i\xFE`, their triggers `g\xFF` and `g\xFE` and the
    /// views `v\xFF` and `v\xFE` differ only in a byte that is not UTF-8, so
    /// that each pair converts to one text with U+FFFD substituted
    /// (`FR-OUT-017`). Each is created first, so that its row stands before
    /// its pair's in the schema table: the reverse of the order of their
    /// bytes. SQLite writes no such name, so they are written under
    /// `writable_schema`, as `tests/support/sqlite.rs` writes its artefacts,
    /// by a connection whose defensive mode is off.
    fn colliding(path: &Path) {
        let connection = Connection::open(path).expect("the database opens");
        connection
            .set_db_config(DbConfig::SQLITE_DBCONFIG_DEFENSIVE, false)
            .expect("the builder sets defensive mode off");
        connection
            .execute_batch(
                "CREATE TABLE parent (id INTEGER PRIMARY KEY, label TEXT NOT NULL);
                 CREATE TABLE tbl_ff (id INTEGER PRIMARY KEY, ff_name TEXT NOT NULL,
                     ff_qty INTEGER DEFAULT 0,
                     parent_id INTEGER REFERENCES parent (id) ON DELETE CASCADE);
                 CREATE TABLE tbl_fe (id INTEGER PRIMARY KEY, fe_rate REAL,
                     parent_id INTEGER REFERENCES parent (id) ON DELETE SET NULL);
                 CREATE TABLE child (id INTEGER PRIMARY KEY,
                     ref_fe INTEGER REFERENCES tbl_fe (id),
                     ref_ff INTEGER REFERENCES tbl_ff (id));
                 CREATE INDEX idx_ff ON tbl_ff (ff_name, ff_qty DESC);
                 CREATE INDEX idx_fe ON tbl_fe (fe_rate);
                 CREATE TRIGGER trg_ff AFTER INSERT ON tbl_ff BEGIN SELECT 1; END;
                 CREATE TRIGGER trg_fe BEFORE DELETE ON tbl_fe BEGIN SELECT 1; END;
                 CREATE VIEW vw_ff AS SELECT id, ff_name FROM tbl_ff;
                 CREATE VIEW vw_fe AS SELECT id, fe_rate FROM tbl_fe;",
            )
            .expect("the schema builds");
        let renamed = |column: &str| {
            RENAMED
                .iter()
                .fold(column.to_owned(), |text, (name, bytes)| {
                    format!("replace({text}, '{name}', CAST(x'{bytes}' AS TEXT))")
                })
        };
        connection
            .execute_batch(&format!(
                "PRAGMA writable_schema = ON;
                 UPDATE sqlite_schema SET name = {}, tbl_name = {}, sql = {};
                 PRAGMA writable_schema = OFF;",
                renamed("name"),
                renamed("tbl_name"),
                renamed("sql"),
            ))
            .expect("the names are rewritten");
        connection.close().expect("the database closes");
    }

    #[test]
    fn fr_out_017_names_that_convert_alike_keep_the_order_of_their_bytes() {
        // The statements carried `ORDER BY` until task #323, which ordered two
        // names that convert to one text by their bytes, and the fold kept
        // that order wherever the names tied. The model of the database of
        // [`colliding`] is pinned here as the read with `ORDER BY` gave it:
        // the rows of both `t\u{fffd}` tables grouped under the one text, those
        // of `t\xFE` first; the members of both `i\u{fffd}` indexes, `i\xFE`'s
        // first among those of one `seqno`; the stored text of `v\xFE`; and the
        // triggers, `g\xFE`'s first.
        let scratch = Scratch::new();
        let path = scratch.directory("colliding").join("colliding.sqlite");
        colliding(&path);
        let catalogue = read(&path);
        let database = catalogue.model().expect("the model folds");
        let replaced = |prefix: &str| format!("{prefix}\u{fffd}");

        let names: Vec<&str> = database.tables.iter().map(|table| table.name()).collect();
        assert_eq!(
            names,
            [
                "child",
                "parent",
                replaced("t").as_str(),
                replaced("t").as_str()
            ]
        );
        let (first, second) = (&database.tables[2], &database.tables[3]);
        for table in [first, second] {
            let columns: Vec<&str> = table.columns().iter().map(|column| &*column.name).collect();
            assert_eq!(
                columns,
                [
                    "id",
                    "fe_rate",
                    "parent_id",
                    "id",
                    "ff_name",
                    "ff_qty",
                    "parent_id"
                ]
            );
            let indexes: Vec<(&str, Vec<&str>)> = table
                .indexes()
                .iter()
                .map(|index| {
                    (
                        &*index.name,
                        index.columns.iter().map(|member| &*member.name).collect(),
                    )
                })
                .collect();
            assert_eq!(
                indexes,
                [(replaced("i").as_str(), vec!["fe_rate", "ff_name", "ff_qty"])]
            );
        }

        // Both triggers attach to the first table of the name, `g\xFE` first.
        let events: Vec<&TriggerEvent> = first
            .triggers()
            .iter()
            .map(|trigger| &trigger.event)
            .collect();
        assert_eq!(events, [&TriggerEvent::Delete, &TriggerEvent::Insert]);
        assert!(second.triggers().is_empty());
        let incoming: Vec<&str> = first
            .referenced_by()
            .iter()
            .map(|incoming| &*incoming.table)
            .collect();
        assert_eq!(incoming, ["child", "child"]);
        assert!(second.referenced_by().is_empty());

        // Both views carry the stored text of `v\xFE`.
        let definitions: Vec<&str> = database
            .views
            .iter()
            .map(|view| &*view.definition)
            .collect();
        let fe = format!(
            "CREATE VIEW {} AS SELECT id, fe_rate FROM {}",
            replaced("v"),
            replaced("t")
        );
        assert_eq!(definitions, [fe.as_str(), fe.as_str()]);
    }

    #[test]
    fn fr_out_017_names_that_convert_alike_fold_alike_whatever_order_their_rows_come_in() {
        // The database of [`colliding`] in five drawn orders folds to the
        // model its rows fold to as returned.
        let scratch = Scratch::new();
        let path = scratch.directory("colliding").join("colliding.sqlite");
        colliding(&path);
        let catalogue = read(&path);
        let expected = catalogue.model().expect("the model folds");

        for seed in SEEDS {
            let shuffled = shuffled(&catalogue, seed);
            assert_eq!(
                shuffled.model().expect("the shuffled rows fold"),
                expected,
                "seed {seed}"
            );
        }
    }
}
