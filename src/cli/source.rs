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
//! `FR-ERR-006` fixes it, and it is the order of [`project`] and
//! [`Reader::open_from`] followed by [`Reader::fetch`]:
//!
//! | Step | What runs | Requirement |
//! |---|---|---|
//! | 2 | Project discovery and the two trust checks | `FR-PROJ-004` … `FR-PROJ-011` |
//! | 3 | `.tpl/.cfg` is read and validated | `FR-CONF-001` … `FR-CONF-022` |
//! | 5 | The entry is selected and resolved, and the two keys a read needs are checked | `FR-GLOB-004` … `FR-GLOB-008`, `FR-CONF-040`, `FR-CONF-041` |
//! | 6 | The cache is consulted, and the server read only where it misses | `FR-CACHE-006`, `FR-CACHE-007` |
//!
//! **Step 4 is missing from that table because it is not this module's.** It is
//! template resolution, which `FR-ERR-006` evaluates immediately after `.tpl/.cfg`
//! and which only `tpl render` reaches. That command therefore takes steps 2 and
//! 3 from [`project`], resolves its template name, and enters step 5 through
//! [`Reader::open_from`]; every other caller has no step 4 to make room for and
//! calls [`Reader::open`], which is the two joined.
//!
//! Step 5's two checks are made **here** and not below: `FR-CONF-040` rejects
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
//!
//! # Where the process ends, the read's buffers are leaked
//!
//! A document borrows the buffers it was decoded from: the cache files, or the
//! rows of a server read and the model folded from them. Under
//! [`Ending::Process`] those owners are leaked rather than dropped, and the
//! document borrows `'static` data ([`Served::leaked`]). That removes two costs
//! (`BENCHMARKS.md`, 2026-09-22, rows 3 and 4 of the waste register): the
//! destructor of the decoded document and its buffers, 0.41 ms of a cached
//! render of `WL-001`, which the exiting process has no use for; and the copy
//! `tpl render` took of the whole document, 0.81 to 1.08 ms and 3.65 MB,
//! because `minijinja` holds the `database` object under a `'static` bound.
//! Under [`Ending::Caller`] — a test that carries on — the owners are dropped
//! as before, and the render copies ([`Served::borrowed`]).

use std::ops::Deref;
use std::path::Path;

use crate::cache::paths::Collection;
use crate::cache::{Cache, Covered, Listed, Look, Summary};
use crate::deadline::{Clock, Deadlines, Seconds};
use crate::error::{Error, KeyAbsence};
use crate::mariadb::{self, Target, catalogue};
use crate::model::document::{self, DatabaseDocument};
use crate::output::Source;
use crate::project::Project;
use crate::project::config::Configuration;
use crate::project::config::expand;
use crate::project::config::keys::EntryKey;
use crate::project::settings::{self, Settings};

use super::Ending;
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
/// the two cache flags, and whether the process exits when the command
/// returns.
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

    /// Whether the process exits when the command returns, which decides
    /// whether a read's buffers are leaked or dropped (see [`Served`]).
    ending: Ending,
}

/// The document a read produced, as a presentation receives it, and whether
/// it lives for the rest of the process.
///
/// It dereferences to the [`DatabaseDocument`], so a presentation that only
/// reads the document cannot tell a [`Served::leaked`] one from a
/// [`Served::borrowed`] one. The one caller that must keep the document beyond
/// the presentation — the `database` variable of `tpl render`, which
/// `minijinja` holds under a `'static` bound — keeps a leaked document as it is
/// ([`Served::leaked_document`]) and copies a borrowed one.
///
/// It is a struct and not a two-variant enum because a document is invariant
/// in its lifetime (a `Cow` of a slice is), so no one reference type can hold
/// both a `'static` document and one that borrows the caller's buffers.
#[derive(Debug, Clone, Copy)]
pub(super) struct Served<'a, 'd> {
    /// The document.
    document: &'a DatabaseDocument<'d>,

    /// The same document where it borrows buffers leaked for the rest of the
    /// process, under [`Ending::Process`]; [`None`] where the owners are
    /// dropped once the presentation returns, under [`Ending::Caller`].
    leaked: Option<&'static DatabaseDocument<'static>>,
}

impl<'a, 'd> Served<'a, 'd> {
    /// A document whose owners the caller drops once the presentation
    /// returns.
    pub(super) const fn borrowed(document: &'a DatabaseDocument<'d>) -> Self {
        Self {
            document,
            leaked: None,
        }
    }

    /// The document, where it lives for the rest of the process.
    pub(super) const fn leaked_document(&self) -> Option<&'static DatabaseDocument<'static>> {
        self.leaked
    }
}

impl Served<'static, 'static> {
    /// A document whose owners are leaked for the rest of the process:
    /// neither it nor anything it borrows is ever freed.
    pub(super) const fn leaked(document: &'static DatabaseDocument<'static>) -> Self {
        Self {
            document,
            leaked: Some(document),
        }
    }
}

impl<'d> Deref for Served<'_, 'd> {
    type Target = DatabaseDocument<'d>;

    fn deref(&self) -> &Self::Target {
        self.document
    }
}

/// `value`, moved to the heap and leaked for the rest of the process.
///
/// It is safe and runs no destructor, so it is reserved to
/// [`Ending::Process`], where the operating system reclaims the memory when the
/// process exits. What is leaked must own memory only — no buffered writer, no
/// lock, no child process — for the same reason [`Ending::release`] states.
pub(super) fn leak<T>(value: T) -> &'static T {
    Box::leak(Box::new(value))
}

/// The entry and the store one invocation acts on.
///
/// It is steps 2, 3 and 5 of `FR-ERR-006`, settled: a value of this type
/// exists only where the project was found and trusted, `.tpl/.cfg` validated,
/// the entry selected and resolved, and the two keys of `FR-CONF-040` and
/// `FR-CONF-041` found.
///
/// The project itself is **not** kept. A command that reaches `.tpl/` as well
/// as a catalogue holds it already: [`project`] is what produced it, and
/// [`Reader::open_from`] takes it by reference precisely so that one discovery
/// serves both.
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

    /// The four phase deadlines of `FR-CONF-004`, as `.tpl/.cfg` resolved
    /// them.
    ///
    /// `tpl render` reads `core.render_timeout` from here rather than from a
    /// second load of the file, which `FR-CONF-004` resolves once per
    /// invocation.
    pub(super) const fn deadlines(&self) -> Deadlines {
        self.settings.deadlines()
    }
}

impl<'a> Reader<'a> {
    /// What `globals` and `caching` supply.
    ///
    /// `caching` is [`None`] for a command that declares neither flag, which
    /// `FR-CACHE-020` leaves without either behaviour to choose.
    pub(super) fn new(globals: &'a Globals, caching: Option<&Caching>, ending: Ending) -> Self {
        Self {
            tpl_dir: globals.tpl_dir.first().map(std::path::PathBuf::as_path),
            requested: globals.database.first().map(String::as_str),
            budget: globals.timeout.first().copied().map(Seconds::new),
            direct: caching.is_some_and(|caching| caching.direct),
            no_cache: caching.is_some_and(|caching| caching.no_cache),
            ending,
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
    /// steps 2, 3 and 5 of `FR-ERR-006`.
    ///
    /// Every command but `tpl render` reaches the entry straight from the
    /// project, because `FR-ERR-006` gives it nothing to evaluate in between.
    ///
    /// # Errors
    ///
    /// Returns what [`project`] and [`Reader::open_from`] return.
    pub(super) fn open(&self) -> Result<Opened, Error> {
        let (project, configuration) = project(self.tpl_dir)?;

        self.open_from(&project, configuration)
    }

    /// Step 5 of `FR-ERR-006` over a project already discovered: the entry is
    /// selected and resolved, and the two keys a read needs are checked.
    ///
    /// It is separate from [`Reader::open`] because `FR-ERR-006` evaluates
    /// template resolution at step 4, between the configuration and the entry,
    /// and `tpl render` is the one command that reaches both sides of it. The
    /// entry is where the `${VAR}` expansion of `FR-CONF-015` happens and where
    /// the `password_command` child of `FR-CONF-024` runs, so an invocation
    /// whose template does not resolve must never arrive here.
    ///
    /// # Errors
    ///
    /// Returns what [`settings::resolve`] returns, and
    /// [`Error::EntryKeyMissing`] where the entry names no host or no
    /// database, per `FR-CONF-040` and `FR-CONF-041`.
    pub(super) fn open_from(
        &self,
        project: &Project,
        configuration: Configuration,
    ) -> Result<Opened, Error> {
        let settings = settings::resolve(
            &configuration,
            self.requested,
            &self.clock(),
            &expand::environment,
        )?;

        connection_keys(&settings, &configuration)?;

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
    /// answer outlive it: a [`catalogue::Catalogue`] owns its rows. The close
    /// ends the session, closes the socket and shuts the driver's runtime
    /// down before this returns, whether the read succeeded or not, so every
    /// caller holds the owned answer and nothing of the read — which is what
    /// `FR-RND-040` requires of `tpl render` before its template evaluates.
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
            absence: KeyAbsence::Absent,
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
            &catalogue::Scope::Everything,
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
    /// Handing the value in is what makes `FR-CACHE-012` structural rather than
    /// a rule each command applies: the one place that decides which source
    /// served the read is the one place that states it, so no presentation can
    /// answer from the store without saying so.
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
    pub(super) fn serve<T, P>(&self, look: &Look<'_>, present: P) -> Result<T, Error>
    where
        P: FnOnce(&DatabaseDocument<'_>, Source, &str) -> Result<T, Error>,
    {
        let opened = self.open()?;

        self.serve_from(&opened, look, |served, source, entry| {
            present(&served, source, entry)
        })
    }

    /// [`Reader::serve`] over a project already opened, handing the
    /// presentation the document as [`Served`].
    ///
    /// `tpl render` does not come through here: it needs steps 2, 3 and 5 in
    /// hand before the read, and it reads the cache under `FR-CACHE-038`
    /// rather than whole, so it opens through [`Reader::open_from`] and
    /// reaches the server through [`Reader::read_through`].
    ///
    /// # Errors
    ///
    /// Returns what [`Reader::fetch`] returns, what the fold and the document
    /// build return, and whatever `present` returns.
    fn serve_from<T, P>(&self, opened: &Opened, look: &Look<'_>, present: P) -> Result<T, Error>
    where
        P: FnOnce(Served<'_, '_>, Source, &str) -> Result<T, Error>,
    {
        // FR-CACHE-006: the cache is consulted first, and a hit opens no
        // connection. FR-CACHE-013 skips the lookup outright, and
        // FR-CACHE-033 makes a file that will not decode a miss rather than a
        // condition — which is why the decode is part of the hit.
        //
        // FR-CDOC-011 is discharged in this module and nowhere else: it
        // obliges `source` to satisfy FR-CACHE-012 — a cached read states that
        // it was cached — and this call, `serve_summary` and `serve_listing`
        // are the three that present what the store served. The value is
        // passed rather than defaulted, so a hit cannot reach the presentation
        // carrying the value a live read would have carried.
        if !self.direct
            && let Some(loaded) = opened.cache.look(look)
        {
            // PERF: under Ending::Process the files' bytes are leaked, so the
            // document borrows them for the rest of the process and is never
            // freed or copied (this module's documentation). A miss leaks the
            // bytes it could not decode, which the server read then outlives.
            match self.ending {
                Ending::Process => {
                    if let Some(document) = leak(loaded).document() {
                        return present(
                            Served::leaked(leak(document)),
                            Source::Cache,
                            opened.entry(),
                        );
                    }
                }
                Ending::Caller => {
                    if let Some(document) = loaded.document() {
                        return present(Served::borrowed(&document), Source::Cache, opened.entry());
                    }
                }
            }
        }

        self.read_through(opened, present)
    }

    /// Serves `tpl schema info`: the metadata and the size of each collection,
    /// from the cache where [`Cache::summary`] hits and from the server
    /// otherwise (`FR-SCH-031`).
    ///
    /// It is [`Reader::serve`] with a narrower hit: the cache is asked for
    /// `database.json` and three counts rather than for every object file, and
    /// a miss is the same whole read, the same write and the same presentation
    /// of what the server returned. `source` is set here, on both paths, for
    /// the reason [`Reader::serve_from`] gives.
    ///
    /// # Errors
    ///
    /// Returns what [`Reader::serve`] returns.
    pub(super) fn serve_summary<T, P>(&self, present: P) -> Result<T, Error>
    where
        P: FnOnce(&Summary<'_>, Source) -> Result<T, Error>,
    {
        let opened = self.open()?;

        if !self.direct
            && let Some(held) = opened.cache.summary()
            && let Some(summary) = held.summary()
        {
            return present(&summary, Source::Cache);
        }

        self.read_through(&opened, |served, source, _| {
            present(&Summary::of(&served), source)
        })
    }

    /// Serves the `text` listing of `tpl schema tables`: four members of each
    /// table, from the cache where the table collection is recorded whole and
    /// every file decodes as a [`Listed`], and from the server otherwise
    /// (`FR-SCH-026`, `FR-CDOC-007`).
    ///
    /// It is [`Reader::serve`] over [`Look::Collection`] with a reduced decode
    /// on the hit: the same files are read and every one must decode, and a
    /// miss is the same whole read, write and presentation. The `json` form
    /// carries every table in full and is served by [`Reader::serve`].
    ///
    /// # Errors
    ///
    /// Returns what [`Reader::serve`] returns.
    pub(super) fn serve_listing<T, P>(&self, present: P) -> Result<T, Error>
    where
        P: FnOnce(&[Listed<'_>], Source) -> Result<T, Error>,
    {
        let opened = self.open()?;

        if !self.direct
            && let Some(loaded) = opened.cache.collection(Collection::Tables)
            && let Some(listed) = loaded.listing()
        {
            return present(&listed, Source::Cache);
        }

        self.read_through(&opened, |served, source, _| {
            let listed: Vec<Listed<'_>> = served.tables.iter().map(Listed::of).collect();
            present(&listed, source)
        })
    }

    /// The server half of every read: the whole catalogue is read, the cache
    /// written unless `--no-cache` was given, and the document presented with
    /// `source` set to `server` (`FR-CACHE-007`, `FR-SCH-035`).
    ///
    /// `tpl render` calls it directly on a miss, including the miss
    /// `FR-CACHE-039` finds during a render; that is the one connection of
    /// `NFR-PERF-004`, because nothing before it opened one.
    ///
    /// `present` runs only after [`Reader::fetch`] has returned, so the
    /// connection is closed and the runtime shut down before a template is
    /// evaluated from what it is handed (`FR-RND-040`).
    ///
    /// # Errors
    ///
    /// Returns what [`Reader::fetch`] returns, what the fold and the document
    /// build return, and whatever `present` returns.
    pub(super) fn read_through<T, P>(&self, opened: &Opened, present: P) -> Result<T, Error>
    where
        P: FnOnce(Served<'_, '_>, Source, &str) -> Result<T, Error>,
    {
        let catalogue = self.fetch(opened)?;
        let hand = |served: Served<'_, '_>| {
            if !self.no_cache {
                opened.cache.write(&served, Covered::Everything);
            }

            present(served, Source::Server, opened.entry())
        };

        // PERF: under Ending::Process the rows and the model are leaked, so
        // the document borrows them for the rest of the process and is never
        // freed or copied (this module's documentation).
        match self.ending {
            Ending::Process => {
                let model = leak(leak(catalogue).model()?);

                hand(Served::leaked(leak(document::context(model)?)))
            }
            Ending::Caller => {
                let model = catalogue.model()?;

                hand(Served::borrowed(&document::context(&model)?))
            }
        }
    }

    /// The clock this invocation's blocking phases are bounded by
    /// (`FR-GLOB-011`, `FR-GLOB-012`).
    ///
    /// It is read outside this module by `tpl render`, whose own phase is
    /// bounded by the same budget: `FR-GLOB-012` composes `--timeout` with
    /// every phase deadline, and the render is one of the six `FR-CONF-005`
    /// names.
    pub(super) fn clock(&self) -> Clock {
        settings::clock(self.budget)
    }
}

/// The project and its validated configuration — steps 2 and 3 of
/// `FR-ERR-006`.
///
/// `tpl cache clean` and `tpl cache status` name the store by entry and never
/// reach a server, so they stop at the **selection** of `FR-GLOB-004` through
/// `FR-GLOB-008`: `FR-CONF-040` and `FR-CONF-041` govern an invocation that
/// opens a connection or reads the catalogue, and neither does. Stopping there
/// is also what keeps `password_command` from running for a command that has
/// nothing to authenticate to.
///
/// `tpl render` reaches it for a second reason: step 4 of `FR-ERR-006` resolves
/// its template name from the root `FR-TMPL-023` makes a property of this very
/// project, and it does so before [`Reader::open_from`] resolves the entry.
///
/// # Errors
///
/// Returns what [`Project::current`] and [`Project::configuration`] return.
pub(super) fn project(tpl_dir: Option<&Path>) -> Result<(Project, Configuration), Error> {
    let project = Project::current(tpl_dir)?;
    let configuration = project.configuration()?;

    Ok((project, configuration))
}

/// The two keys an invocation that opens a connection needs, checked in the
/// order `FR-ERR-006` fixes (`FR-CONF-040`, `FR-CONF-041`).
///
/// An entry that describes no connection is refused before one that describes
/// no read: `FR-CONF-040` is the host and `FR-CONF-041` the server-side
/// database, and each `cause` names the file and the key the entry does not
/// carry, per the `78` row of `FR-ERR-034`.
///
/// It is a free function so that the two callers apply **one** refusal rather
/// than two that can drift: the cached read of [`Reader::open_from`], and
/// `tpl cfg database test`, which `FR-CONF-041` names in as many words — the
/// probe of `FR-CFG-044` "has nothing to restrict itself to" without a
/// database, so that command exits `78` on such an entry and reports none of
/// the four steps of `FR-CFG-024`.
///
/// # Errors
///
/// Returns [`Error::EntryKeyMissing`] naming whichever of the two keys the
/// entry does not carry.
pub(super) fn connection_keys(
    settings: &Settings,
    configuration: &Configuration,
) -> Result<(), Error> {
    let raw = configuration.entry(settings.entry());
    let missing = |key: EntryKey, flag, placeholder, value: Option<&str>| {
        // FR-CONF-050: an empty value, as written or after expansion, is an
        // absent key, and the cause says which of the three it is.
        let written = raw.and_then(|entry| match (&entry.dsn, key) {
            (Some(dsn), _) => Some(dsn.value.as_str()),
            (None, EntryKey::Host) => entry.host.as_deref(),
            (None, _) => entry.database.as_deref(),
        });
        let absence = match (value, written) {
            (None, _) => KeyAbsence::Absent,
            (Some(_), Some(written)) if written.contains("${") => KeyAbsence::ExpandsToEmpty,
            (Some(_), _) => KeyAbsence::Empty,
        };

        Error::EntryKeyMissing {
            entry: settings.entry().to_owned(),
            key: format!("database.{}.{key}", settings.entry()),
            file: configuration.file().to_owned(),
            flag,
            placeholder,
            absence,
        }
    };

    match settings.host() {
        None => return Err(missing(EntryKey::Host, HOST_FLAG, HOST_PLACEHOLDER, None)),
        Some("") => {
            return Err(missing(
                EntryKey::Host,
                HOST_FLAG,
                HOST_PLACEHOLDER,
                Some(""),
            ));
        }
        Some(_) => {}
    }
    match settings.database() {
        None => Err(missing(
            EntryKey::Database,
            SCHEMA_FLAG,
            SCHEMA_PLACEHOLDER,
            None,
        )),
        Some("") => Err(missing(
            EntryKey::Database,
            SCHEMA_FLAG,
            SCHEMA_PLACEHOLDER,
            Some(""),
        )),
        Some(_) => Ok(()),
    }
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
    use crate::cli::Ending;
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
            let reader = Reader::new(&supplied, Some(&carried), Ending::Caller);

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
        let reader = Reader::new(&supplied, None, Ending::Caller);

        assert!(!reader.direct && !reader.no_cache);
        assert_eq!(reader.requested(), Some("shop"));
        assert_eq!(reader.tpl_dir(), None);
    }
}
