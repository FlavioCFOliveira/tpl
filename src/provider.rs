//! The dispatch over an entry's provider (`FR-CONF-051`, `OD-35`).
//!
//! `FR-CONF-051` makes the provider of the selected entry decide which
//! contract reads it, and this module is where that decision is taken once: one
//! exhaustive match sends a `mariadb` entry to `mariadb/`, a `mysql` entry to
//! `mysql/` and a `sqlite` entry to `sqlite/`, and the three readers yield the
//! same `model::Database`. Three callers reach it and no other module names a
//! reader: `cli/source.rs`, through which every catalogue read goes;
//! `cli/cfg/connectivity.rs`, for the four steps of `tpl cfg database test`
//! (`FR-CFG-024`, `FR-CFG-056`); and `bounded()` in `cli/render.rs`, which asks
//! [`quiescent`] whether anything of any reader's read is alive
//! (`FR-RND-040`).
//!
//! *Rejected, in `OD-35`: a trait with one implementation per provider.* The
//! cases are closed and known when the crate is compiled, and the sessions
//! differ in kind — the two server readers each own the asynchronous runtime of
//! `ADR-005`, and the file reader is synchronous.

use crate::deadline::Clock;
use crate::error::Error;
use crate::mariadb::{self, Target, catalogue};
use crate::model::database::Database;
use crate::model::server::Server;
use crate::mysql;
use crate::project::settings::FileSettings;
use crate::sqlite;

/// What a read is made against, as the provider of its entry decides.
#[derive(Debug)]
pub(crate) enum Source<'a> {
    /// A MariaDB server: the connection target and the database the entry
    /// names (`FR-CONF-041`).
    Mariadb {
        /// Where the connection goes.
        target: Target<'a>,
        /// The server-side database the read covers.
        database: &'a str,
    },
    /// A MySQL server, reached exactly as a MariaDB server is
    /// (`FR-MYSQL-020`) and read under `mysql-contract.md`.
    Mysql {
        /// Where the connection goes.
        target: Target<'a>,
        /// The server-side database the read covers (`FR-MYSQL-023`).
        database: &'a str,
    },
    /// A SQLite database file.
    File(&'a FileSettings),
}

/// What a read produced, by reader; all three yield the same model.
#[derive(Debug)]
pub(crate) enum Read {
    /// A `mariadb` read.
    Mariadb(catalogue::Catalogue),
    /// A `mysql` read.
    Mysql(mysql::catalogue::Catalogue),
    /// A `sqlite` read.
    File(sqlite::Catalogue),
}

impl Read {
    /// The model this read yields.
    ///
    /// # Errors
    ///
    /// Returns what the reader's fold returns.
    pub(crate) fn model(&self) -> Result<Database<'_>, Error> {
        match self {
            Self::Mariadb(catalogue) => catalogue.model(),
            Self::Mysql(catalogue) => catalogue.model(),
            Self::File(catalogue) => catalogue.model(),
        }
    }
}

/// Opens the one connection of `NFR-PERF-004`, reads the whole catalogue, and
/// closes it before returning, whether the read succeeded or not
/// (`FR-RND-040`).
///
/// # Errors
///
/// Returns what the reader of the source's provider returns.
pub(crate) fn read(source: Source<'_>, clock: &Clock) -> Result<Read, Error> {
    match source {
        Source::Mariadb { target, database } => {
            let mut session = mariadb::open(&target, clock)?;
            let read = catalogue::read(
                &mut session,
                &target,
                clock,
                database,
                &catalogue::Scope::Everything,
            );
            session.close();
            read.map(Read::Mariadb)
        }
        Source::Mysql { target, database } => {
            let mut session = mysql::open(&target, clock)?;
            let read = mysql::catalogue::read(&mut session, &target, clock, database);
            session.close();
            read.map(Read::Mysql)
        }
        Source::File(settings) => {
            let session = sqlite::open(settings, clock)?;
            let read = sqlite::read(&session);
            session.close();
            read.map(Read::File)
        }
    }
}

/// What the four steps of `tpl cfg database test` found (`FR-CFG-024`,
/// `FR-CFG-056`): the steps that succeeded are what reached this value.
#[derive(Debug)]
pub(crate) struct Tested {
    /// Step 3: what answered the version probe.
    pub(crate) server: Server<'static>,
    /// Step 4: what the catalogue probe answered.
    pub(crate) can_read_catalogue: bool,
}

/// The four steps of `tpl cfg database test`, for every provider.
///
/// For a `mariadb` entry they are those of `FR-CFG-024`, the connection
/// carrying no default schema and the probe restricted to `database`; for a
/// `mysql` entry, the same four with the MySQL counterpart of each — the
/// read-only session of `FR-MYSQL-018`, the product and series checks of
/// `FR-MYSQL-011`, and the probe of `FR-CFG-044` against a table
/// `FR-MYSQL-015` closes — in the order `FR-MYSQL-017` fixes; for a `sqlite`
/// entry, those of `FR-CFG-056`: the kind check and the open, every control
/// and limit, the version probe, and the check of `FR-LITE-041` — one
/// `SELECT` against `sqlite_schema` whose rows are judged and never read as
/// model content, issued alone and outside any transaction (`FR-LITE-042`);
/// `can_read_catalogue` is true whenever it is answered and refuses no row,
/// and a row it refuses is its `78`.
///
/// # Errors
///
/// Returns the condition of the first step that refuses.
pub(crate) fn test(source: Source<'_>, clock: &Clock) -> Result<Tested, Error> {
    match source {
        Source::Mariadb { target, database } => {
            let target = target.without_database();
            let mut session = mariadb::open(&target, clock)?;
            let can_read_catalogue = catalogue::probe(&mut session, &target, clock, database);
            let server = session.server().clone();
            session.close();

            Ok(Tested {
                server,
                can_read_catalogue: can_read_catalogue?,
            })
        }
        Source::Mysql { target, database } => {
            let target = target.without_database();
            let mut session = mysql::open(&target, clock)?;
            let can_read_catalogue =
                mysql::catalogue::probe(&mut session, &target, clock, database);
            let server = session.server().clone();
            session.close();

            Ok(Tested {
                server,
                can_read_catalogue: can_read_catalogue?,
            })
        }
        Source::File(settings) => {
            let session = sqlite::open(settings, clock)?;
            let probed = session.check();
            let server = session.server().clone();
            session.close();
            probed?;

            Ok(Tested {
                server,
                can_read_catalogue: true,
            })
        }
    }
}

/// Whether the calling thread holds nothing of any reader's read: no
/// connection, no driver runtime and no statement watchdog (`FR-RND-040`).
pub(crate) fn quiescent() -> bool {
    mariadb::quiescent() && mysql::quiescent() && sqlite::quiescent()
}
