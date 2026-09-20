//! Where a catalogue read comes from: the cache first, the server on a miss
//! (`FR-CACHE-006`, `FR-CACHE-007`, `FR-SCH-025`).
//!
//! Every command that reads a catalogue goes through this module: the eight
//! `schema` subcommands, per `FR-SCH-025`, and `tpl cache load`, which is the
//! same pipeline with the lookup skipped and the store narrowed. What differs
//! between them is what happens to the `database` object at the end, so that is
//! the parameter — [`Reader::serve`] takes the presentation and supplies the
//! document.
//!
//! # The order the steps run in
//!
//! `FR-ERR-006` fixes it, and it is the order of [`Reader::open`] followed by
//! [`Reader::fetch`]:
//!
//! | Step | What runs | Requirement |
//! |---|---|---|
//! | 2 | Project discovery and the two trust checks | `FR-PROJ-004` … `FR-PROJ-011` |
//! | 3 | `.tpl/.cfg` is read and validated | `FR-CONF-001` … `FR-CONF-022` |
//! | 4 | The entry is selected and resolved, and the two keys a read needs are checked | `FR-GLOB-004` … `FR-GLOB-008`, `FR-CONF-040`, `FR-CONF-041` |
//! | 5 | The cache is consulted, and the server read only where it misses | `FR-CACHE-006`, `FR-CACHE-007` |
//!
//! Step 4's two checks are made **here** and not below: `FR-CONF-040` rejects
//! composing either refusal where the connection is assembled, because the `78`
//! row of `FR-ERR-034` obliges the `cause` to name the file and the key, and
//! neither is a thing that layer holds.
//!
//! # One read shape, and why
//!
//! Every server read this module makes is
//! [`Scope::Everything`](catalogue::Scope::Everything). The narrowed plans of
//! that enumeration cannot produce the document a named read must answer with:
//! `FR-CTX-006` and `FR-CTX-010` embed, in full, the table at each end of every
//! foreign key, and a plan that reads one table row returns that table's keys
//! without the tables they name — which `FR-CTX-023` requires to be present and
//! which [`crate::model::document`] detects as a dangling reference rather than
//! emitting. The population `FR-SCH-010` draws a nearest-match suggestion from
//! is in hand for the same reason, and `FR-CACHE-007` then warms the whole
//! cache, so the invocation after a miss opens no connection at all, per
//! `FR-CACHE-006`.
//!
//! `NFR-PERF-001` and `NFR-PERF-002` are satisfied as they are stated: the plan
//! is eleven statements for a database of one object and for a database of two
//! hundred alike, so no count here depends on what the database holds.
//!
//! # A cache that cannot be read or written is never an error
//!
//! [`crate::cache`] states both directions. Nothing in this module reports a
//! cache failure: a lookup that finds nothing, a file that will not decode and a
//! write that will not land are each the miss or the missed optimisation their
//! own requirement makes them, and the exit code and every byte of stdout are
//! what they would have been.

use std::path::Path;

use crate::cache::{Cache, Covered, Look};
use crate::deadline::{Clock, Seconds};
use crate::error::Error;
use crate::mariadb::{self, Target, catalogue};
use crate::model::document::{self, DatabaseDocument};
use crate::output::Source;
use crate::project::Project;
use crate::project::config::Configuration;
use crate::project::config::expand;
use crate::project::config::keys::EntryKey;
use crate::project::settings::{self, Settings};

use super::globals::Globals;
use super::local::Caching;

/// The flag of `FR-CFG-027` that writes `database.<name>.host`.
const HOST_FLAG: &str = "--host";

/// The placeholder the hint of `FR-CONF-040` writes after it.
const HOST_PLACEHOLDER: &str = "<host>";

/// The flag of `FR-CFG-027` that writes `database.<name>.database`.
const SCHEMA_FLAG: &str = "--schema";

/// The placeholder the hint of `FR-CONF-041` writes after it.
const SCHEMA_PLACEHOLDER: &str = "<database>";

/// What one invocation supplies to a read: the project, the entry, the budget,
/// and the two cache flags.
///
/// It is the counterpart of `cfg`'s own `Supplied`, and it exists for the same
/// reason: a command takes one argument rather than four vectors it must reduce
/// itself. `FR-CLI-014` has already reduced each of the three global flags to at
/// most one occurrence by the time this is built.
#[derive(Debug, Clone, Copy)]
pub(super) struct Reader<'a> {
    /// `--tpl-dir`, which names the `.tpl` folder and suppresses the walk
    /// (`FR-GLOB-009`).
    tpl_dir: Option<&'a Path>,

    /// `-d/--database`, which names the entry (`FR-GLOB-004`).
    requested: Option<&'a str>,

    /// `--timeout`, the overall budget of `FR-GLOB-011`.
    budget: Option<Seconds>,

    /// `--direct`: read the database, ignoring whatever is cached
    /// (`FR-CACHE-013`).
    direct: bool,

    /// `--no-cache`: store nothing (`FR-CACHE-014`).
    no_cache: bool,
}

/// The project, the entry and the store one invocation acts on.
///
/// It is steps 2 through 4 of `FR-ERR-006`, settled: a value of this type
/// exists only where the project was found and trusted, `.tpl/.cfg` validated,
/// the entry selected and resolved, and the two keys of `FR-CONF-040` and
/// `FR-CONF-041` found.
#[derive(Debug)]
pub(super) struct Opened {
    /// The entry's own store (`FR-CACHE-001`, `FR-CACHE-002`).
    pub(super) cache: Cache,

    /// The validated configuration, kept so that a later condition can name
    /// the file the `78` row of `FR-ERR-034` obliges.
    configuration: Configuration,

    /// Everything the connection will need, already resolved.
    settings: Settings,
}

impl Opened {
    /// The database entry this invocation selected (`FR-GLOB-008`).
    pub(super) fn entry(&self) -> &str {
        self.settings.entry()
    }
}

impl<'a> Reader<'a> {
    /// What `globals` and `caching` supply.
    ///
    /// `caching` is [`None`] for a command that declares neither flag, which
    /// `FR-CACHE-020` leaves without either behaviour to choose.
    pub(super) fn new(globals: &'a Globals, caching: Option<&Caching>) -> Self {
        Self {
            tpl_dir: globals.tpl_dir.first().map(std::path::PathBuf::as_path),
            requested: globals.database.first().map(String::as_str),
            budget: globals.timeout.first().copied().map(Seconds::new),
            direct: caching.is_some_and(|caching| caching.direct),
            no_cache: caching.is_some_and(|caching| caching.no_cache),
        }
    }

    /// `--tpl-dir`, for a command that reaches the project and no catalogue.
    pub(super) const fn tpl_dir(&self) -> Option<&'a Path> {
        self.tpl_dir
    }

    /// `-d/--database`, for the same commands.
    pub(super) const fn requested(&self) -> Option<&'a str> {
        self.requested
    }

    /// Discovers the project, reads `.tpl/.cfg`, and resolves the entry —
    /// steps 2 through 4 of `FR-ERR-006`.
    ///
    /// # Errors
    ///
    /// Returns what [`Project::current`], [`Project::configuration`] and
    /// [`settings::resolve`] return, and [`Error::EntryKeyMissing`] where the
    /// entry names no host or no database, per `FR-CONF-040` and
    /// `FR-CONF-041`.
    pub(super) fn open(&self) -> Result<Opened, Error> {
        let project = Project::current(self.tpl_dir)?;
        let configuration = project.configuration()?;
        let settings = settings::resolve(
            &configuration,
            self.requested,
            &self.clock(),
            &expand::environment,
        )?;

        // FR-CONF-040 and FR-CONF-041, in the order FR-ERR-006 reads the
        // connection before the catalogue: an entry that describes no
        // connection is refused before one that describes no read.
        let missing = |key: EntryKey, flag, placeholder| Error::EntryKeyMissing {
            entry: settings.entry().to_owned(),
            key: format!("database.{}.{key}", settings.entry()),
            file: configuration.file().to_owned(),
            flag,
            placeholder,
        };

        if settings.host().is_none() {
            return Err(missing(EntryKey::Host, HOST_FLAG, HOST_PLACEHOLDER));
        }
        if settings.database().is_none() {
            return Err(missing(EntryKey::Database, SCHEMA_FLAG, SCHEMA_PLACEHOLDER));
        }

        let cache = Cache::of(project.root(), settings.entry());

        Ok(Opened {
            cache,
            configuration,
            settings,
        })
    }

    /// Opens the one connection of `NFR-PERF-004`, reads the whole catalogue,
    /// and closes it.
    ///
    /// The session is closed as soon as the read ends, which is what lets the
    /// answer outlive it: a [`catalogue::Catalogue`] owns its rows.
    ///
    /// # Errors
    ///
    /// Returns [`Error::EntryKeyMissing`] where the resolved settings name no
    /// host or no database — which [`Reader::open`] has already refused, so
    /// each arm is the type's own totality rather than a reachable condition —
    /// and what [`mariadb::open`] and [`catalogue::read`] return.
    pub(super) fn fetch(&self, opened: &Opened) -> Result<catalogue::Catalogue, Error> {
        let settings = &opened.settings;
        let file = opened.configuration.file();
        let missing = |key: &str, flag, placeholder| Error::EntryKeyMissing {
            entry: settings.entry().to_owned(),
            key: format!("database.{}.{key}", settings.entry()),
            file: file.to_owned(),
            flag,
            placeholder,
        };

        let Some(target) = Target::of(settings) else {
            return Err(missing("host", HOST_FLAG, HOST_PLACEHOLDER));
        };
        // FR-CONF-041: the database a read covers is the one the entry names.
        let Some(database) = settings.database() else {
            return Err(missing("database", SCHEMA_FLAG, SCHEMA_PLACEHOLDER));
        };

        let clock = self.clock();
        let mut session = mariadb::open(&target, &clock)?;
        let read = catalogue::read(
            &mut session,
            &target,
            &clock,
            database,
            catalogue::Scope::Everything,
        );

        session.close();

        read
    }

    /// Reads the catalogue and hands the `database` object to `present`.
    ///
    /// `present` receives the document, the value of `FR-OUT-026` that served
    /// it — `cache` on a hit and `server` on a miss, per `FR-SCH-035` — and the
    /// entry the read was made through. It is a parameter rather than a
    /// returned document because the document borrows whichever of the two
    /// owners produced it, and a caller that took it out would have to name
    /// that owner.
    ///
    /// `look` is what the cache is asked for, and is the command's own: a
    /// listing asks for its collection and a named object for itself, per
    /// `FR-CDOC-007` and `FR-CDOC-008`. A miss reads the **whole** catalogue
    /// whatever was asked for, for the reason this module's own documentation
    /// gives.
    ///
    /// The cache is written **before** `present` runs, which is `FR-CACHE-007`:
    /// the write happens before the answer. `FR-CACHE-016` makes
    /// `--direct --no-cache` the pure read — neither the lookup nor the write
    /// runs, so no file of the store is touched.
    ///
    /// # Errors
    ///
    /// Returns what [`Reader::open`] and [`Reader::fetch`] return, what the
    /// fold and the document build return, and whatever `present` returns.
    pub(super) fn serve<T, P>(&self, look: Look<'_>, present: P) -> Result<T, Error>
    where
        P: FnOnce(&DatabaseDocument<'_>, Source, &str) -> Result<T, Error>,
    {
        let opened = self.open()?;

        // FR-CACHE-006: the cache is consulted first, and a hit opens no
        // connection. FR-CACHE-013 skips the lookup outright, and
        // FR-CACHE-033 makes a file that will not decode a miss rather than a
        // condition — which is why the decode is part of the hit.
        if !self.direct
            && let Some(loaded) = opened.cache.look(look)
            && let Some(document) = loaded.document()
        {
            return present(&document, Source::Cache, opened.entry());
        }

        let catalogue = self.fetch(&opened)?;
        let model = catalogue.model()?;
        let document = document::context(&model)?;

        if !self.no_cache {
            opened.cache.write(&document, Covered::Everything);
        }

        present(&document, Source::Server, opened.entry())
    }

    /// The clock this invocation's blocking phases are bounded by
    /// (`FR-GLOB-011`, `FR-GLOB-012`).
    fn clock(&self) -> Clock {
        settings::clock(self.budget)
    }
}

/// The project and its validated configuration, for a command that reaches
/// `.tpl/` and no catalogue.
///
/// `tpl cache clean` and `tpl cache status` name the store by entry and never
/// reach a server, so they stop at the **selection** of `FR-GLOB-004` through
/// `FR-GLOB-008`: `FR-CONF-040` and `FR-CONF-041` govern an invocation that
/// opens a connection or reads the catalogue, and neither does. Stopping there
/// is also what keeps `password_command` from running for a command that has
/// nothing to authenticate to.
///
/// # Errors
///
/// Returns what [`Project::current`] and [`Project::configuration`] return.
pub(super) fn project(tpl_dir: Option<&Path>) -> Result<(Project, Configuration), Error> {
    let project = Project::current(tpl_dir)?;
    let configuration = project.configuration()?;

    Ok((project, configuration))
}

/// The entry such an invocation selects (`FR-GLOB-004` … `FR-GLOB-008`).
///
/// # Errors
///
/// Returns what [`settings::select`] returns: the `78` of `FR-GLOB-006` where
/// nothing names an entry, and the `66` of `FR-GLOB-007` where the name reaches
/// no entry of the file.
pub(super) fn entry_of<'c>(
    configuration: &'c Configuration,
    requested: Option<&str>,
) -> Result<&'c str, Error> {
    settings::select(configuration, requested).map(|(name, _, _)| name)
}

#[cfg(test)]
mod tests {
    use super::Reader;
    use crate::cli::globals::Globals;
    use crate::cli::local::Caching;

    /// The globals a test invocation carries.
    fn globals(entry: Option<&str>) -> Globals {
        Globals {
            database: entry.map(str::to_owned).into_iter().collect(),
            tpl_dir: Vec::new(),
            timeout: Vec::new(),
            verbose: 0,
            quiet: false,
            help: false,
            version: false,
        }
    }

    #[test]
    fn fr_cache_015_the_two_flags_are_read_independently_and_compose() {
        // FR-CACHE-015: the two are orthogonal, so the four rows of its table
        // are four values of this type rather than one enumerated setting.
        let supplied = globals(Some("shop"));

        for (direct, no_cache) in [(false, false), (true, false), (false, true), (true, true)] {
            let carried = Caching { direct, no_cache };
            let reader = Reader::new(&supplied, Some(&carried));

            assert_eq!(reader.direct, direct);
            assert_eq!(reader.no_cache, no_cache);
        }
    }

    #[test]
    fn fr_cache_020_a_command_that_declares_neither_flag_reads_neither() {
        // FR-CACHE-020 withholds both from `cache clean` and `cache status`,
        // and the absence is carried rather than defaulted from a flag those
        // commands never parsed.
        let supplied = globals(Some("shop"));
        let reader = Reader::new(&supplied, None);

        assert!(!reader.direct && !reader.no_cache);
        assert_eq!(reader.requested(), Some("shop"));
        assert_eq!(reader.tpl_dir(), None);
    }
}
