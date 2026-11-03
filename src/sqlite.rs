//! The reader of a `sqlite` entry: the hardened open, the catalogue read, and
//! the fold into the model (`FR-LITE-001` … `FR-LITE-040`, `OD-35`).
//!
//! It is the second reader, beside `mariadb/` and shaped like it, and the only
//! module that sends a statement for a `sqlite` entry (`FR-LITE-015`,
//! `FR-LITE-016`). It shares nothing with `mariadb/`: both write `model/`, and
//! both are bounded through `deadline.rs`.
//!
//! | Submodule | What it owns | Forced by |
//! |---|---|---|
//! | `open` | The kind of the file, and the read-only open | `FR-LITE-005`, `FR-LITE-006` |
//! | `harden` | Each control and limit, set and read back, and the comparison of the two | `FR-LITE-010` … `FR-LITE-012` |
//! | `watch` | The watchdog that bounds every statement | `FR-LITE-009`, `OD-36` |
//! | `fault` | Where the library's result becomes a condition of `FR-ERR-001` | `FR-LITE-040`, `OD-38` |
//! | `catalogue` | The plan, the read that owns its values, and the fold | `FR-LITE-015`, `FR-LITE-019` … `FR-LITE-037` |
//!
//! **The one route to a connection is [`open`]**, and it runs the six stages
//! of `FR-LITE-017` in order and behind no condition: the kind of the file,
//! the read-only open, the statement bound, the build, the other controls and
//! the limits, and the version probe — the first statement that reads the
//! file. No flag, key or environment condition reaches any of them
//! (`FR-LITE-014`).
//!
//! **No asynchronous runtime is built.** `rusqlite` is synchronous, so a
//! command that reads a `sqlite` entry builds no runtime and reaches nothing
//! in `mariadb/` (`ADR-013`, *Consequences*; `ADR-005`).
//!
//! **What is alive is counted per thread**, as `mariadb/` counts its own: the
//! open connections and the live watchdogs, so that [`quiescent`] answers
//! `FR-RND-040` for this reader.

mod catalogue;
mod fault;
mod harden;
mod open;
mod watch;

use std::cell::Cell;
use std::marker::PhantomData;
use std::path::{Path, PathBuf};
use std::thread::LocalKey;
use std::time::Instant;

use rusqlite::config::DbConfig;
use rusqlite::limits::Limit;
use rusqlite::types::ValueRef;
use rusqlite::{Connection, Row};

use crate::deadline::{Clock, Phase, Seconds};
use crate::diagnostics::emit::{self, CatalogueSource};
use crate::error::Error;
use crate::model::server::{Server, Standing};
use crate::project::settings::FileSettings;

pub(crate) use catalogue::{Catalogue, read};
pub(crate) use fault::name as code_name;

use catalogue::statements;
use fault::{Place, Stage, Timing};
use watch::Watchdog;

thread_local! {
    /// The connections to a database file this thread holds open.
    static CONNECTIONS: Cell<usize> = const { Cell::new(0) };

    /// The statement watchdogs this thread has started and not yet joined.
    static WATCHDOGS: Cell<usize> = const { Cell::new(0) };
}

/// Whether this thread holds no open connection and no live watchdog
/// (`FR-RND-040`).
///
/// The counts are per thread for the reason `mariadb::quiescent` gives: every
/// statement runs on the thread that opened the connection, and a
/// process-wide count would see another test's connection in the suite.
pub(crate) fn quiescent() -> bool {
    CONNECTIONS.with(Cell::get) == 0 && WATCHDOGS.with(Cell::get) == 0
}

/// One unit of a thread's count, held for as long as what it counts exists.
#[derive(Debug)]
struct Alive {
    /// The count this unit belongs to.
    count: &'static LocalKey<Cell<usize>>,
    /// Keeps the unit on the thread it was counted on.
    here: PhantomData<*const ()>,
}

impl Alive {
    /// Counts one more on this thread.
    fn new(count: &'static LocalKey<Cell<usize>>) -> Self {
        count.with(|value| value.set(value.get() + 1));
        Self {
            count,
            here: PhantomData,
        }
    }
}

impl Drop for Alive {
    fn drop(&mut self) {
        self.count
            .with(|value| value.set(value.get().saturating_sub(1)));
    }
}

/// One connection to a database file, every control in force and the probe
/// answered.
///
/// The field order is the drop order, and it is deliberate: the watchdog is
/// stopped and joined before the connection is dropped, so no interrupt is
/// issued against a closed connection (`OD-36`, point 4).
#[derive(Debug)]
pub(crate) struct Session {
    /// The watchdog of `OD-36`, and its count.
    watchdog: Option<(Watchdog, Alive)>,
    /// The connection, and its count.
    connection: Option<(Connection, Alive)>,
    /// The entry the read is made through.
    entry: String,
    /// The database file, as resolved.
    path: PathBuf,
    /// `core.query_timeout`, the deadline of every statement (`FR-LITE-009`).
    query: Seconds,
    /// The clock the deadline composes with (`FR-GLOB-012`).
    clock: Clock,
    /// The library that answered the probe (`FR-LITE-002`).
    server: Server<'static>,
}

/// Why a statement did not answer.
#[derive(Debug)]
enum Failed {
    /// A condition already classified: a bound spent before it began, or a
    /// value of a class the library does not document for its field.
    Condition(Error),
    /// The library's own failure, not yet classified.
    Library(rusqlite::Error, Timing),
}

/// Why a row could not be read.
#[derive(Debug)]
enum Unread {
    /// The library failed while stepping.
    Library(rusqlite::Error),
    /// A field held a value of a class the library does not document for it.
    Shape(&'static str),
}

impl From<rusqlite::Error> for Unread {
    fn from(error: rusqlite::Error) -> Self {
        Self::Library(error)
    }
}

/// The invariant a value of an undocumented class breaks (`FR-LITE-004`,
/// step 2, `FR-ERR-030`).
const STORAGE_CLASS: &str =
    "every field a statement of FR-LITE-015 reads holds the storage class SQLite documents for it";

/// The invariant a version without two dot-separated parts breaks.
const VERSION_FORM: &str =
    "the version the carried library reports has the form <major>.<minor>.<patch>";

/// Opens the database file `settings` names, through the six stages of
/// `FR-LITE-017`, in their order and behind no condition.
///
/// # Errors
///
/// Returns the condition of the first stage that refuses: `78` or `74` for
/// the kind of the file (`FR-LITE-005`); the code `FR-LITE-040` gives what the
/// open returned (`FR-LITE-006`, `FR-LITE-007`); the `70` of a control or a
/// limit that did not take effect (`FR-LITE-012`); and the code of whatever
/// the version probe returned.
pub(crate) fn open(settings: &FileSettings, clock: &Clock) -> Result<Session, Error> {
    let entry = settings.entry();
    let path = settings.path();
    let place = Place { entry, path };

    // Stage 1: the kind of the file, before anything opens it.
    open::kind(entry, path)?;

    // Stage 2: the read-only open.
    #[cfg_attr(
        not(test),
        allow(unused_mut, reason = "the trace hook of OD-37 takes it mutably")
    )]
    let mut connection =
        open::open(path).map_err(|error| fault::condition(place, Stage::Open, &error, None))?;
    #[cfg(test)]
    record::install(&mut connection);
    let counted = Alive::new(&CONNECTIONS);

    // Stage 3: the statement bound — the watchdog, and the busy timeout each
    // statement sets before it runs (OD-36).
    let watchdog = Watchdog::start(connection.get_interrupt_handle()).map_err(|_| {
        fault::defect(
            place,
            Stage::Open,
            "the statement watchdog of a SQLite connection starts",
            None,
        )
    })?;

    let mut session = Session {
        watchdog: Some((watchdog, Alive::new(&WATCHDOGS))),
        connection: Some((connection, counted)),
        entry: entry.to_owned(),
        path: path.to_owned(),
        query: settings.deadlines().of(Phase::CatalogueQuery),
        clock: *clock,
        server: Server::stated(
            std::borrow::Cow::Borrowed(""),
            std::borrow::Cow::Borrowed(""),
            Standing::Supported,
        ),
    };

    // Stages 4 and 5: every control and limit, confirmed.
    session.harden()?;

    // Stage 6: the version probe, the first statement that reads the file.
    let version = session.rows(Stage::Probe, statements::PROBE, false, |row| text(row, 0));
    let version = session.answered(Stage::Probe, version)?;
    let version = version
        .into_iter()
        .next()
        .ok_or_else(|| fault::defect(session.place(), Stage::Probe, VERSION_FORM, None))?;
    // FR-LITE-002 and FR-LITE-003: `series` from the version alone, and the
    // standing `supported` with no window consulted.
    session.server = Server::probed(std::borrow::Cow::Owned(version), Standing::Supported)
        .ok_or_else(|| fault::defect(session.place(), Stage::Probe, VERSION_FORM, None))?;

    Ok(session)
}

impl Session {
    /// The library that answered the probe (`FR-LITE-002`).
    pub(crate) const fn server(&self) -> &Server<'static> {
        &self.server
    }

    /// Closes the connection, the watchdog first (`OD-36`, point 4,
    /// `FR-RND-040`).
    ///
    /// A failed close is not a condition: the answer is already produced, and
    /// `close` is "functionally equivalent to the `Drop` implementation" apart
    /// from returning the error (docs.rs, `rusqlite` 0.41.1).
    pub(crate) fn close(mut self) {
        if let Some((watchdog, counted)) = self.watchdog.take() {
            watchdog.stop();
            drop(counted);
        }
        if let Some((connection, counted)) = self.connection.take() {
            let _ = connection.close();
            drop(counted);
        }
    }

    /// The catalogue probe of `FR-CFG-056`, step 4: one `SELECT` against
    /// `sqlite_schema` of `main`, whose rows are not read as model content.
    ///
    /// # Errors
    ///
    /// Returns the code `FR-LITE-040` gives what the library returned.
    pub(crate) fn probe(&self) -> Result<(), Error> {
        let answered = self.rows(
            Stage::Catalogue("the catalogue probe of tpl cfg database test"),
            statements::CATALOGUE_PROBE,
            true,
            |_| Ok(()),
        );

        self.answered(
            Stage::Catalogue("the catalogue probe of tpl cfg database test"),
            answered,
        )
        .map(drop)
    }

    /// Where a failure on this connection arose.
    fn place(&self) -> Place<'_> {
        Place {
            entry: &self.entry,
            path: &self.path,
        }
    }

    /// Stages 4 and 5 of the connection start (`FR-LITE-010`,
    /// `FR-LITE-011`).
    fn harden(&self) -> Result<(), Error> {
        let place = self.place();
        let refused = |mismatch: harden::Mismatch| mismatch.into_error(&self.entry, &self.path);
        let control =
            |error: rusqlite::Error| fault::condition(place, Stage::Control, &error, None);

        // Stage 4: the build.
        let options = self.rows(Stage::Build, statements::COMPILE_OPTIONS, false, |row| {
            text(row, 0)
        });
        let options = self.answered(Stage::Build, options)?;
        harden::build(&options).map_err(refused)?;

        let connection = self.connection()?;

        // Stage 5: the read-only open, read back on `main`.
        let readonly = connection.is_readonly("main").map_err(control)?;
        harden::flag("read-only open", "read-only", true, readonly).map_err(refused)?;

        // Defensive mode on and `trusted_schema` off, through the
        // configuration interface (FR-LITE-015 admits the pragma without
        // obliging it).
        connection
            .set_db_config(DbConfig::SQLITE_DBCONFIG_DEFENSIVE, true)
            .map_err(control)?;
        let defensive = connection
            .db_config(DbConfig::SQLITE_DBCONFIG_DEFENSIVE)
            .map_err(control)?;
        harden::flag("defensive mode", "on", true, defensive).map_err(refused)?;

        connection
            .set_db_config(DbConfig::SQLITE_DBCONFIG_TRUSTED_SCHEMA, false)
            .map_err(control)?;
        let trusted = connection
            .db_config(DbConfig::SQLITE_DBCONFIG_TRUSTED_SCHEMA)
            .map_err(control)?;
        harden::flag("trusted_schema", "off", false, trusted).map_err(refused)?;

        // The four controls a statement applies, each set and read back.
        for (name, set, set_statement, read_statement, expected) in [
            (
                "query_only",
                "on (1)",
                statements::SET_QUERY_ONLY,
                statements::READ_QUERY_ONLY,
                1,
            ),
            (
                "cell_size_check",
                "on (1)",
                statements::SET_CELL_SIZE_CHECK,
                statements::READ_CELL_SIZE_CHECK,
                1,
            ),
            (
                "mmap_size",
                "0",
                statements::SET_MMAP_SIZE,
                statements::READ_MMAP_SIZE,
                0,
            ),
            (
                "temp_store",
                "MEMORY (2)",
                statements::SET_TEMP_STORE,
                statements::READ_TEMP_STORE,
                2,
            ),
        ] {
            let applied = self.rows(Stage::Control, set_statement, false, |_| Ok(()));
            self.answered(Stage::Control, applied)?;

            let read = self.rows(Stage::Control, read_statement, false, |row| integer(row, 0));
            let read = self.answered(Stage::Control, read)?;
            let read_back = read.first().copied().ok_or_else(|| {
                refused(harden::Mismatch {
                    control: name,
                    set,
                    read_back: "no row".to_owned(),
                })
            })?;
            harden::number(name, set, expected, read_back).map_err(refused)?;
        }

        // The three limits of FR-LITE-011.
        for (limit, name, set, value) in [
            (
                Limit::SQLITE_LIMIT_LENGTH,
                "SQLITE_LIMIT_LENGTH",
                "1000000",
                harden::LENGTH_LIMIT,
            ),
            (
                Limit::SQLITE_LIMIT_SQL_LENGTH,
                "SQLITE_LIMIT_SQL_LENGTH",
                "1000000",
                harden::LENGTH_LIMIT,
            ),
            (
                Limit::SQLITE_LIMIT_ATTACHED,
                "SQLITE_LIMIT_ATTACHED",
                "0",
                harden::ATTACHED_LIMIT,
            ),
        ] {
            connection.set_limit(limit, value).map_err(control)?;
            let read_back = connection.limit(limit).map_err(control)?;
            harden::number(name, set, i64::from(value), i64::from(read_back)).map_err(refused)?;
        }

        Ok(())
    }

    /// The connection, while it is open.
    fn connection(&self) -> Result<&Connection, Error> {
        self.connection
            .as_ref()
            .map(|(connection, _)| connection)
            .ok_or_else(|| {
                fault::defect(
                    self.place(),
                    Stage::Open,
                    "a statement is issued only on an open connection",
                    None,
                )
            })
    }

    /// The rows of a statement that answered, or the condition its failure
    /// is under `FR-LITE-040`.
    fn answered<T>(&self, stage: Stage, answered: Result<Vec<T>, Failed>) -> Result<Vec<T>, Error> {
        answered.map_err(|failed| match failed {
            Failed::Condition(error) => error,
            Failed::Library(error, timing) => {
                fault::condition(self.place(), stage, &error, Some(timing))
            }
        })
    }

    /// Runs one statement under its bound and reads every row through `read`
    /// (`FR-LITE-009`, `OD-36`, `FR-GLOB-017`).
    ///
    /// The bound is the phase clock's bound over `core.query_timeout`, composed
    /// with the overall budget; a bound already spent starts nothing and is the
    /// `69` of `FR-ERR-027`. The busy timeout is set to what remains of it, the
    /// watchdog armed with the same, and both apply to this statement alone.
    /// `catalogue` says whether the statement is one of the last row of
    /// `FR-LITE-015`, which is what the line of `FR-GLOB-017` counts.
    ///
    /// Every row is copied out as it is read, so the statement is finalised
    /// before this returns, and no value borrows the connection.
    fn rows<T, F>(
        &self,
        stage: Stage,
        sql: &str,
        catalogue: bool,
        mut read: F,
    ) -> Result<Vec<T>, Failed>
    where
        F: FnMut(&Row<'_>) -> Result<T, &'static str>,
    {
        let bound = self.clock.bound(self.query);
        if bound.expired() {
            return Err(Failed::Condition(fault::expired(
                self.place(),
                stage,
                bound,
            )));
        }

        let connection = self.connection().map_err(Failed::Condition)?;
        let started = Instant::now();
        let timing = |spent: bool| Timing { bound, spent };

        if let Err(error) = connection.busy_timeout(bound.remaining()) {
            return Err(Failed::Library(error, timing(false)));
        }

        // FR-GLOB-017: one line per catalogue query, written before the
        // statement runs, so a statement that runs and fails is counted too.
        if catalogue {
            emit::catalogue_query(CatalogueSource::SqliteSchema);
        }

        let watchdog = self.watchdog.as_ref().map(|(watchdog, _)| watchdog);
        if let Some(watchdog) = watchdog {
            watchdog.arm(bound.remaining());
        }

        let outcome = (|| -> Result<Vec<T>, Unread> {
            let mut statement = connection.prepare(sql)?;
            let mut rows = statement.query([])?;
            let mut collected = Vec::new();
            while let Some(row) = rows.next()? {
                collected.push(read(row).map_err(Unread::Shape)?);
            }
            Ok(collected)
        })();

        if let Some(watchdog) = watchdog {
            let _ = watchdog.disarm();
        }
        let took = started.elapsed();
        emit::phase_ran(Phase::CatalogueQuery, took);

        outcome.map_err(|unread| match unread {
            Unread::Library(error) => Failed::Library(error, timing(took >= bound.remaining())),
            Unread::Shape(invariant) => {
                Failed::Condition(fault::defect(self.place(), stage, invariant, None))
            }
        })
    }

    /// The entry and the path, for a condition raised outside a statement.
    pub(crate) fn origin(&self) -> (&str, &Path) {
        (&self.entry, &self.path)
    }
}

/// A text field, read through its storage class and converted with U+FFFD
/// substituted, at this boundary and nowhere else (`FR-OUT-017`).
fn text(row: &Row<'_>, at: usize) -> Result<String, &'static str> {
    match row.get_ref(at) {
        Ok(ValueRef::Text(bytes)) => Ok(String::from_utf8_lossy(bytes).into_owned()),
        _ => Err(STORAGE_CLASS),
    }
}

/// A text field that may be SQL `NULL`, kept distinct from every value
/// (`FR-LITE-004`, step 2).
fn text_or_null(row: &Row<'_>, at: usize) -> Result<Option<String>, &'static str> {
    match row.get_ref(at) {
        Ok(ValueRef::Text(bytes)) => Ok(Some(String::from_utf8_lossy(bytes).into_owned())),
        Ok(ValueRef::Null) => Ok(None),
        _ => Err(STORAGE_CLASS),
    }
}

/// An integer field, never coerced from another class.
fn integer(row: &Row<'_>, at: usize) -> Result<i64, &'static str> {
    match row.get_ref(at) {
        Ok(ValueRef::Integer(value)) => Ok(value),
        _ => Err(STORAGE_CLASS),
    }
}

/// The statement record of `FR-LITE-018`, taken in process through the
/// library's trace (`OD-37`, point 4).
///
/// The hook is compiled under the test configuration alone — the seam
/// `FR-ERR-031` names — and the feature `trace` it calls is enabled by a
/// `[dev-dependencies]` entry alone, so the distributed binary carries neither
/// the call nor the binding (`ADR-013`, point 1). Each thread keeps its own
/// record, so tests running in parallel never share one.
#[cfg(test)]
pub(crate) mod record {
    use std::cell::RefCell;

    use rusqlite::Connection;
    use rusqlite::trace::{TraceEvent, TraceEventCodes};

    thread_local! {
        /// The texts the trace reported on this thread while recording.
        static RECORD: RefCell<Option<Vec<String>>> = const { RefCell::new(None) };
    }

    /// The prefix the trace gives a nested statement (`FR-LITE-018`).
    pub(crate) const NESTED: &str = "-- ";

    /// Installs the trace on a connection just opened, before its first
    /// statement.
    pub(super) fn install(connection: &mut Connection) {
        let installed = connection.trace_v2(
            TraceEventCodes::SQLITE_TRACE_STMT,
            Some(|event: TraceEvent<'_>| {
                if let TraceEvent::Stmt(_, text) = event {
                    RECORD.with(|record| {
                        if let Some(record) = record.borrow_mut().as_mut() {
                            record.push(text.to_owned());
                        }
                    });
                }
            }),
        );
        assert!(installed.is_ok(), "the statement trace installs");
    }

    /// Runs `run` while recording, and answers what it answered and every
    /// text the trace reported on this thread meanwhile.
    pub(crate) fn recorded<T>(run: impl FnOnce() -> T) -> (T, Vec<String>) {
        RECORD.with(|record| *record.borrow_mut() = Some(Vec::new()));
        let answered = run();
        let texts = RECORD.with(|record| record.borrow_mut().take().unwrap_or_default());

        (answered, texts)
    }

    /// The texts the system issued: every text reported without the prefix
    /// of a nested statement, which is neither checked nor counted.
    pub(crate) fn issued(texts: &[String]) -> Vec<&str> {
        texts
            .iter()
            .map(String::as_str)
            .filter(|text| !text.starts_with(NESTED))
            .collect()
    }
}

/// The SQLite fixture of `FR-LITE-038`, built by the shared test support.
#[cfg(test)]
#[path = "../tests/support/sqlite.rs"]
pub(crate) mod fixture;

#[cfg(test)]
mod tests;
