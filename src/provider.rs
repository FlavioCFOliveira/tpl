//! The dispatch over an entry's provider (`FR-CONF-051`, `OD-35`).
//!
//! `FR-CONF-051` makes the provider of the selected entry decide which
//! contract reads it, and this module is where that decision is taken once: one
//! exhaustive match sends a `mariadb` entry to `mariadb/` and a `sqlite` entry
//! to `sqlite/`, and both readers yield the same `model::Database`. Three
//! callers reach it and no other module names a reader: `cli/source.rs`,
//! through which every catalogue read goes; `cli/cfg/connectivity.rs`, for the
//! four steps of `tpl cfg database test` (`FR-CFG-024`, `FR-CFG-056`); and
//! `bounded()` in `cli/render.rs`, which asks [`quiescent`] whether anything of
//! either reader's read is alive (`FR-RND-040`).
//!
//! *Rejected, in `OD-35`: a trait with two implementations.* The two cases are
//! closed and known when the crate is compiled, and the two sessions differ in
//! kind — one owns the asynchronous runtime of `ADR-005`, the other is
//! synchronous.

use crate::deadline::Clock;
use crate::error::Error;
use crate::mariadb::{self, Target, catalogue};
use crate::model::database::Database;
use crate::model::document::shape::UnreadableView;
use crate::model::server::Server;
use crate::project::settings::FileSettings;
use crate::sqlite;

/// What a read is made against, as the provider of its entry decides.
#[derive(Debug)]
pub(crate) enum Source<'a> {
    /// A MariaDB server: the connection target and the database the entry
    /// names (`FR-CONF-041`).
    Server {
        /// Where the connection goes.
        target: Target<'a>,
        /// The server-side database the read covers.
        database: &'a str,
    },
    /// A SQLite database file.
    File(&'a FileSettings),
}

/// What a read produced, by reader; both yield the same model.
#[derive(Debug)]
pub(crate) enum Read {
    /// A `mariadb` read.
    Server(catalogue::Catalogue),
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
            Self::Server(catalogue) => catalogue.model(),
            Self::File(catalogue) => catalogue.model(),
        }
    }

    /// The views whose columns a `sqlite` read could not read, and why
    /// (`FR-LITE-035`); none for a `mariadb` read.
    pub(crate) fn unreadable(&self) -> &[UnreadableView] {
        match self {
            Self::Server(_) => &[],
            Self::File(catalogue) => catalogue.unreadable(),
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
        Source::Server { target, database } => {
            let mut session = mariadb::open(&target, clock)?;
            let read = catalogue::read(
                &mut session,
                &target,
                clock,
                database,
                &catalogue::Scope::Everything,
            );
            session.close();
            read.map(Read::Server)
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

/// The four steps of `tpl cfg database test`, for either provider.
///
/// For a `mariadb` entry they are those of `FR-CFG-024`, the connection
/// carrying no default schema and the probe restricted to `database`; for a
/// `sqlite` entry, those of `FR-CFG-056`: the kind check and the open, every
/// control and limit, the version probe, and one `SELECT` against
/// `sqlite_schema` whose rows are not read — `can_read_catalogue` is true
/// whenever it is answered.
///
/// # Errors
///
/// Returns the condition of the first step that refuses.
pub(crate) fn test(source: Source<'_>, clock: &Clock) -> Result<Tested, Error> {
    match source {
        Source::Server { target, database } => {
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
        Source::File(settings) => {
            let session = sqlite::open(settings, clock)?;
            let probed = session.probe();
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

/// Whether the calling thread holds nothing of either reader's read: no
/// connection, no driver runtime and no statement watchdog (`FR-RND-040`).
pub(crate) fn quiescent() -> bool {
    mariadb::quiescent() && sqlite::quiescent()
}
