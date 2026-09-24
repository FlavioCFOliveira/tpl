//! The catalogue cache arm: `tpl cache`.
//!
//! The node is a group, per `FR-CLI-008`: no action of its own
//! (`FR-CLI-009`), an optional child, and its own help at exit `0` when
//! invoked bare (`FR-CLI-007`). None of its three children carries an alias.
//!
//! `FR-CACHE-021` fixes the membership as well as the kind: the group carries
//! **exactly** `load`, `clean` and `status`, and [`Command`] is that list. A
//! fourth child would be a fourth variant, so the requirement is the shape of
//! the enumeration rather than a rule applied to it.
//!
//! # The surface of the three
//!
//! `FR-CACHE-024` gives `load` and `clean` the three object flags of
//! [`local::Object`], in the spellings `tpl render` uses, and `FR-CACHE-022`
//! and `FR-CACHE-023` make the absence of all three mean *the whole catalogue
//! of the selected entry*. `status` names no object: it reports on the cache
//! rather than acting on one of its members, per `FR-CACHE-025`.
//!
//! The two cache flags are split between the three, and the split is exact.
//! `FR-CACHE-017` declares `--direct` and `--no-cache` on `load` alone, and
//! `FR-CACHE-020` withholds both from `clean` and `status`, where supplying
//! either is the ordinary unknown-flag `64` of `FR-CLI-019`. On `load` itself
//! the two are declared and then treated differently by the command:
//! `FR-CACHE-018` accepts `--direct` and ignores it, because reading the server
//! is what the command does, and `FR-CACHE-019` makes `--no-cache` a `64`,
//! because loading without storing is a contradiction. Both are refusals the
//! command makes about a flag it declares — the alternative, not declaring
//! `--no-cache`, would report the contradiction as an unknown flag and so say
//! the wrong thing.
//!
//! `FR-CACHE-027` gives `status` `--format` and `--pretty`, and gives them to
//! neither of the other two.

use std::borrow::Cow;
use std::io::Write;
use std::path::PathBuf;

use clap::{Args, Subcommand};
use serde::Serialize;

use super::Ending;
use super::globals::Globals;
use super::layout;
use super::local;
use super::schema::named;
use super::source::{self, Opened, Reader};
use crate::cache::paths::Collection;
use crate::cache::{Cache as Store, Covered, Held, Status};
use crate::diagnostics::suggest::{self, Population};
use crate::error::{CatalogueObjectKind, Error};
use crate::model::document::shape::TableDocument;
use crate::model::document::{self, DatabaseDocument};
use crate::model::routine::{Routine, RoutineKind};
use crate::model::view::View;
use crate::output::{Document, Form, Order, Source, Table};
use crate::project::config::keys::is_entry_name;

/// The `tpl cache` group node.
#[derive(Debug, Clone, PartialEq, Eq, Args)]
pub(crate) struct Cache {
    /// The subcommand invoked, or [`None`] where the group node was invoked
    /// bare.
    #[command(subcommand)]
    pub(crate) command: Option<Command>,
}

/// The three children of `tpl cache`, and `FR-CACHE-021` admits no fourth.
#[derive(Debug, Clone, PartialEq, Eq, Subcommand)]
pub(crate) enum Command {
    /// Reads from the server and stores the result in the cache.
    Load {
        /// `--table`, `--view` and `--routine`, per `FR-CACHE-024`.
        #[command(flatten)]
        object: local::Object,

        /// `--direct` and `--no-cache`, per `FR-CACHE-017`.
        #[command(flatten)]
        caching: local::Caching,
    },

    /// Deletes cached data.
    Clean {
        /// `--table`, `--view` and `--routine`, per `FR-CACHE-024`.
        #[command(flatten)]
        object: local::Object,
    },

    /// Reports what the cache holds.
    Status {
        /// `--format` and `--pretty`, per `FR-CACHE-027`.
        #[command(flatten)]
        output: local::Output,
    },
}

/// The invocation of `FR-ERR-022` the routine token of `tpl cache load` belongs
/// to.
///
/// It carries the flag as well as the command path, because `FR-CACHE-024`
/// names the object by flag here where `FR-SCH-005` names it positionally: the
/// hint of a refusal is the same invocation corrected, per `FR-ERR-009`, and an
/// invocation without its flag is not the same one.
const LOAD: &str = "cache load --routine";

/// The invocation the routine token of `tpl cache clean` belongs to.
const CLEAN: &str = "cache clean --routine";

/// The columns of the collection part of `tpl cache status`.
const COLLECTIONS: [&str; 3] = ["NAME", "COUNT", "WHOLE"];

/// The heading that part is written under.
const COLLECTIONS_HEADING: &str = "COLLECTIONS";

/// The `data` of `tpl cache status` (`FR-CACHE-034`).
///
/// Exactly the three keys that requirement fixes, in the order it writes them;
/// `OD-18` makes the field order the key order, so the shape is stated once, in
/// this type.
#[derive(Debug, Serialize)]
struct StatusData<'a> {
    /// The name of the selected database entry.
    entry: &'a str,

    /// The load time from `meta.json`, per `FR-CDOC-013`, or `null` where the
    /// cache is empty, per `FR-CACHE-035`.
    loaded_at: Option<&'a str>,

    /// One object per collection, carrying its name, its count and whether it
    /// was loaded whole, per `FR-CDOC-006`.
    collections: &'a [Held],
}

/// Which object `tpl cache load` and `tpl cache clean` were given
/// (`FR-CACHE-022`, `FR-CACHE-023`, `FR-CACHE-024`).
///
/// The absence of all three flags is not an error: it is the whole-catalogue
/// form both requirements give it.
///
/// It is [`Clone`] and not [`Copy`], for the reason [`named::Wanted`] is.
#[derive(Debug, Clone)]
enum Wanted<'a> {
    /// No object flag: the whole catalogue of the selected entry.
    Everything,
    /// `--table <name>`.
    Table(&'a str),
    /// `--view <name>`.
    View(&'a str),
    /// `--routine <name>`, in either the bare or a qualified form.
    Routine(named::Wanted<'a>),
}

impl<'a> Wanted<'a> {
    /// The collections the command reaches for what it names: all three for
    /// the whole catalogue, and the object's own collection otherwise
    /// (`FR-CACHE-042`, `FR-CACHE-044`).
    const fn collections(&self) -> &'static [Collection] {
        match self {
            Self::Everything => &Collection::ALL,
            Self::Table(_) => &[Collection::Tables],
            Self::View(_) => &[Collection::Views],
            Self::Routine(_) => &[Collection::Routines],
        }
    }

    /// What `object` names, for the command at `invocation`.
    ///
    /// # Errors
    ///
    /// Returns [`Error::MutuallyExclusiveFlags`] — `64` — where more than one
    /// kind was given: `FR-CACHE-024` names **an** individual object, and two
    /// kinds in one invocation name none. Returns
    /// [`Error::RoutinePrefixNotLowerCase`] for the token shape `FR-SCH-008`
    /// refuses, which is decided without a server and therefore precedes
    /// everything else this command does.
    fn of(object: &'a local::Object, invocation: &'static str) -> Result<Self, Error> {
        let given: [(&'static str, Option<&'a String>); 3] = [
            ("--table", object.table.first()),
            ("--view", object.view.first()),
            ("--routine", object.routine.first()),
        ];
        let mut named = given.iter().filter(|(_, value)| value.is_some());

        let Some((first, value)) = named.next() else {
            return Ok(Self::Everything);
        };

        if let Some((second, _)) = named.next() {
            return Err(Error::MutuallyExclusiveFlags {
                first: (*first).to_owned(),
                second: (*second).to_owned(),
            });
        }

        let name = value.map_or("", String::as_str);

        Ok(match *first {
            "--table" => Self::Table(name),
            "--view" => Self::View(name),
            _ => Self::Routine(named::routine_token(name, invocation)?),
        })
    }
}

/// Runs one `cache` subcommand.
///
/// # Errors
///
/// Returns what each subcommand's own function returns.
pub(crate) fn run<W: Write>(
    out: &mut W,
    globals: &Globals,
    command: &Command,
    ending: Ending,
) -> Result<(), Error> {
    match command {
        Command::Load { object, caching } => load(globals, object, caching, ending),
        Command::Clean { object } => clean(globals, object, ending),
        Command::Status { output } => status(out, globals, output, ending),
    }
}

/// Reads the catalogue and stores it (`FR-CACHE-022`).
///
/// With no object flag the whole catalogue of the selected entry is stored;
/// with one, that object alone. The command always reaches the server, which is
/// why `FR-CACHE-018` accepts `--direct` and ignores it, and `FR-CACHE-019`
/// refuses `--no-cache` outright.
///
/// It writes nothing to standard output. `FR-CACHE-027` gives `--format` to
/// `status` and to neither of the other two, so this command has no
/// representation to answer in and `BR-CLI-004` leaves stdout empty.
///
/// # A read that does not complete stores nothing
///
/// `FR-CACHE-032` requires an unreachable server to exit `69` and to leave
/// everything already stored **unchanged**, and the order of the body is what
/// establishes it: every step that can fail — the project, the configuration,
/// the entry, the connection, the catalogue read, the fold, the document build
/// and the resolution of a named object — runs **before** the single call to
/// [`Store::write`], so a failure returns with no file of the store created,
/// truncated or removed. There is no partial-load path: the store is written
/// once, from a document that is already whole, and [`Store::of`] composes
/// paths without creating a directory, so even reaching the failing step leaves
/// nothing behind.
///
/// # Errors
///
/// Returns [`Error::LoadWithoutStoring`] — `64` — for `--no-cache`, per
/// `FR-CACHE-019`; what [`Wanted::of`] returns for the object flags; what the
/// read returns, including the `69` of `FR-CACHE-032` where the server could
/// not be reached; and the `66` of `FR-SCH-010` where the named object does not
/// exist.
fn load(
    globals: &Globals,
    object: &local::Object,
    caching: &local::Caching,
    ending: Ending,
) -> Result<(), Error> {
    // FR-CACHE-019, and FR-ERR-006 step 1: a contradiction between a flag the
    // command declares and what the command does is decided from the
    // invocation alone, before anything is discovered or opened.
    if caching.no_cache {
        let named =
            |kind: &'static str, names: &[String]| names.first().map(|name| (kind, name.clone()));
        return Err(Error::LoadWithoutStoring {
            object: named("table", &object.table)
                .or_else(|| named("view", &object.view))
                .or_else(|| named("routine", &object.routine)),
        });
    }

    let wanted = Wanted::of(object, LOAD)?;
    // FR-CACHE-018: `--direct` is accepted and ignored, because reading the
    // server is what the command does — so the lookup is suppressed whatever
    // the invocation said.
    let reader = Reader::new(globals, None, ending);
    // FR-CACHE-044: the collections the store write reaches are checked for a
    // link before the connection is opened.
    let opened = reader.open(wanted.collections())?;
    let catalogue = reader.fetch(&opened)?;
    let model = catalogue.model()?;
    let document = document::context(&model)?;
    match wanted {
        Wanted::Everything => opened.cache.write(&document, Covered::Everything),
        Wanted::Table(name) => {
            let found = named::table(&document, name, sought(&opened, &document))?;

            opened.cache.write(
                &only(&document, vec![found.clone()], Vec::new(), Vec::new()),
                Covered::One(Collection::Tables),
            );
        }
        Wanted::View(name) => {
            let found = named::view(&document, name, sought(&opened, &document))?;

            opened.cache.write(
                &only(&document, Vec::new(), vec![found.clone()], Vec::new()),
                Covered::One(Collection::Views),
            );
        }
        Wanted::Routine(token) => {
            let found = named::routine(&document, &token, sought(&opened, &document), LOAD)?;

            opened.cache.write(
                &only(&document, Vec::new(), Vec::new(), vec![found.clone()]),
                Covered::One(Collection::Routines),
            );
        }
    }

    Ok(())
}

/// Removes cached data for the selected entry (`FR-CACHE-023`).
///
/// It reaches no server: `FR-CACHE-009` does not list it among the commands
/// that read through the cache, and `FR-CACHE-020` gives it neither cache flag.
/// The entry is therefore selected and not resolved, which is also what keeps
/// the `password_command` of `FR-CONF-023` from running for a command that
/// authenticates to nothing.
///
/// # Errors
///
/// Returns what the project, the configuration and the selection return; what
/// [`Wanted::of`] returns for the object flags; [`Error::AmbiguousRoutineName`]
/// — `64` — where a bare routine name reaches two stored objects, per
/// `FR-CACHE-024`; and [`Error::ProjectFileUnwritable`] where the removal
/// failed.
fn clean(globals: &Globals, object: &local::Object, ending: Ending) -> Result<(), Error> {
    let wanted = Wanted::of(object, CLEAN)?;
    let reader = Reader::new(globals, None, ending);
    let (project, configuration) = source::project(reader.tpl_dir())?;

    // FR-CACHE-041: the cache of a name no entry declares, where the name
    // matches FR-CONF-048, no entry has it under another ASCII case, and the
    // path exists. Any other outcome falls through to the resolution below,
    // which answers exactly as before: `66` for a name that does not resolve.
    if matches!(wanted, Wanted::Everything)
        && let Some(name) = reader
            .requested()
            .or(configuration.core().database.as_deref())
        && is_entry_name(name)
        && !configuration
            .names()
            .any(|declared| declared.eq_ignore_ascii_case(name))
    {
        let orphan = Store::of(project.root(), name);

        // FR-CACHE-042 item 1: a link at `.tpl/.cache` is refused before
        // condition 3 of FR-CACHE-041 looks through it. A link at the orphan's
        // own folder is removed as a link by `clean_orphan`, per item 2.
        orphan.refuse_links(false, &[], true)?;

        if let Some(recorded) = orphan.clean_orphan()? {
            crate::diagnostics::emit::orphan_cache_removed(name, recorded.name.as_deref());
            return Ok(());
        }
    }

    let entry = source::entry_of(&configuration, reader.requested())?;
    let cache = Store::of(project.root(), entry);

    // FR-CACHE-042 item 1, at step 6 of FR-ERR-006 and before the condition of
    // FR-CACHE-040: with no object flag only `.tpl/.cache` is refused, since
    // a link at the entry's folder is the thing removed and is removed as a
    // link (item 2); with one, the entry's folder and the object's collection
    // folder are refused as well.
    match &wanted {
        Wanted::Everything => cache.refuse_links(false, &[], true)?,
        other => cache.refuse_links(true, other.collections(), true)?,
    }

    // FR-CACHE-040: an object flag that names nothing the cache holds deletes
    // nothing and is 66, with the nearest cached names of that kind.
    let absent = |collection: Collection,
                  kind: CatalogueObjectKind,
                  name: &str,
                  qualified: Option<&'static str>,
                  held_as: Option<&'static str>| {
        let names = cache.names(collection);
        let nearest =
            suggest::suggestions(name, names.iter().map(String::as_str), Population::Names)
                .names()
                .map(str::to_owned)
                .collect();
        Error::NothingCachedNamed {
            kind,
            name: name.to_owned(),
            entry: entry.to_owned(),
            nearest,
            qualified,
            held_as,
        }
    };
    let held =
        |collection: Collection, kind: CatalogueObjectKind, name: &str, file: Option<PathBuf>| {
            if cache.holds(file.as_deref()) {
                cache.clean_one(collection, file)
            } else {
                Err(absent(collection, kind, name, None, None))
            }
        };

    match wanted {
        Wanted::Everything => cache.clean(),
        Wanted::Table(name) => held(
            Collection::Tables,
            CatalogueObjectKind::Table,
            name,
            cache.table_file(name),
        ),
        Wanted::View(name) => held(
            Collection::Views,
            CatalogueObjectKind::View,
            name,
            cache.view_file(name),
        ),
        Wanted::Routine(named::Wanted::Qualified(kind, name)) => {
            let file = cache.routine_file(&kind, name);
            if cache.holds(file.as_deref()) {
                return cache.clean_one(Collection::Routines, file);
            }
            // Y-05 of the eighth re-audit of rmp `#263`: a routine of the
            // other kind under the same name is cached, and the lines say so
            // rather than that no routine of that name is.
            let (qualified, other, other_kind) = match kind {
                RoutineKind::Procedure => (
                    Some("procedure"),
                    Some(RoutineKind::Function),
                    Some("function"),
                ),
                RoutineKind::Function => (
                    Some("function"),
                    Some(RoutineKind::Procedure),
                    Some("procedure"),
                ),
                _ => (None, None, None),
            };
            let held_as = other
                .filter(|other| cache.holds(cache.routine_file(other, name).as_deref()))
                .and(other_kind);
            Err(absent(
                Collection::Routines,
                CatalogueObjectKind::Routine,
                name,
                qualified,
                held_as,
            ))
        }
        Wanted::Routine(named::Wanted::Bare(name)) => {
            let procedure = cache.routine_file(&RoutineKind::Procedure, name);
            let function = cache.routine_file(&RoutineKind::Function, name);

            // FR-CACHE-024 and FR-SCH-010: a bare name that reaches both
            // namespaces is refused rather than resolved in favour of either.
            // The population here is what the store holds, because that is
            // what the command acts on.
            if cache.holds(procedure.as_deref()) && cache.holds(function.as_deref()) {
                return Err(Error::AmbiguousRoutineName {
                    name: name.to_owned(),
                    entry: entry.to_owned(),
                    // The store says which database it was read from. A store
                    // that holds two routine files and no readable metadata
                    // has been altered by hand, and the entry name is the one
                    // instance left to name.
                    database: cache.database().unwrap_or_else(|| entry.to_owned()),
                    invocation: CLEAN,
                    template: None,
                });
            }

            let file = if cache.holds(procedure.as_deref()) {
                procedure
            } else {
                function
            };

            held(
                Collection::Routines,
                CatalogueObjectKind::Routine,
                name,
                file,
            )
        }
    }
}

/// Reports what the cache holds (`FR-CACHE-025`, `FR-CACHE-034`).
///
/// `source` is `project` and not `cache`, per `FR-CACHE-034`: the command
/// reports **on** the store rather than being served **from** it. An empty
/// store is a success, per `FR-CACHE-026` and `FR-OUT-033`, and carries
/// `loaded_at` `null` with an empty collection array, per `FR-CACHE-035`.
///
/// # Errors
///
/// Returns what the project, the configuration and the selection return, and
/// [`Error::StdoutUnwritable`] where the stream refused the write.
fn status<W: Write>(
    out: &mut W,
    globals: &Globals,
    output: &local::Output,
    ending: Ending,
) -> Result<(), Error> {
    let reader = Reader::new(globals, None, ending);
    let (project, configuration) = source::project(reader.tpl_dir())?;
    let entry = source::entry_of(&configuration, reader.requested())?;
    let store = Store::of(project.root(), entry);

    // FR-CACHE-044: the report reads `meta.json` and counts every collection,
    // so all three folders are checked for a link first.
    store.refuse_links(true, &Collection::ALL, false)?;
    let held = store.status();

    let format = output
        .format
        .first()
        .copied()
        .unwrap_or(local::Format::Text);
    let form = if output.pretty.pretty {
        Form::Indented
    } else {
        Form::Compact
    };

    match format {
        local::Format::Text => text(out, entry, &held),
        local::Format::Json => crate::output::emit_to(
            out,
            &Document::new(
                Source::Project,
                StatusData {
                    entry,
                    loaded_at: held.loaded_at.as_deref(),
                    collections: &held.collections,
                },
            ),
            form,
        ),
    }
}

/// What the text report writes for `loaded_at` when the cache is empty.
const NEVER_LOADED: &str = "never (the cache is empty; fill it with tpl cache load)";

/// What the text report writes for `loaded_at` when the cache holds files but
/// no usable load record, per `FR-CACHE-034`: the collections below carry the
/// counts, so the cache is not called empty.
const UNRECORDED: &str = "not recorded (no usable load record; the counts below are the files present; \
     reload with tpl cache load)";

/// Writes the `text` form of `tpl cache status`.
///
/// Two parts: the entry and the load time, which are properties of the store,
/// and the collections, which are its contents. The second is written even when
/// it is empty, because it **is** the result of this command and `FR-OUT-034`
/// requires an empty result to print its header row and nothing beneath it.
///
/// # Errors
///
/// Returns [`Error::StdoutUnwritable`] where the stream refused the write.
fn text<W: Write>(out: &mut W, entry: &str, held: &Status) -> Result<(), Error> {
    // AB-01 of the eleventh re-audit of rmp `#263`: the command the row names
    // carries the caller's -d and --tpl-dir, as a hint's does, so that copied
    // it fills this entry's cache and not the default entry's.
    let absent = if held.collections.is_empty() {
        NEVER_LOADED
    } else {
        UNRECORDED
    };
    let loaded_at = held
        .loaded_at
        .as_deref()
        .map_or_else(|| crate::diagnostics::carried(absent), Cow::Borrowed);
    let own: Vec<[layout::Cell<'_>; 2]> = vec![
        [layout::text("entry"), layout::text(entry)],
        [
            layout::text("loaded_at"),
            // An empty cell here read as a value nobody printed; the text says
            // what the absence means and what fills it. The JSON path keeps
            // the `null` of FR-CACHE-035.
            layout::text(&loaded_at),
        ],
    ];
    let collections: Vec<[layout::Cell<'_>; 3]> = held
        .collections
        .iter()
        .map(|collection| {
            [
                layout::text(collection.name),
                layout::count(collection.count),
                layout::flag(collection.whole),
            ]
        })
        .collect();

    let mut sections = layout::Sections::default();
    sections.table(&Table::new(layout::PROPERTY, &own, Order::AsGiven))?;
    sections.heading(COLLECTIONS_HEADING);
    sections.table(&Table::new(COLLECTIONS, &collections, Order::AsGiven))?;

    layout::emit(out, &sections)
}

/// The `database` object narrowed to the members a named load stores.
///
/// `FR-CACHE-022` and `FR-CACHE-024` make `tpl cache load --table orders` store
/// that table and nothing else, and [`Store::write`] stores every member the
/// document it is given carries — so the narrowing is done here, on the
/// document, rather than by a second store path.
///
/// The metadata travels with it because `FR-SCH-025` obliges `tpl schema info`
/// to read through the cache too, and the database's own three fields and the
/// `server` object of `FR-CTX-031` belong to no collection.
fn only<'d>(
    document: &DatabaseDocument<'d>,
    tables: Vec<TableDocument<'d>>,
    views: Vec<View<'d>>,
    routines: Vec<Routine<'d>>,
) -> DatabaseDocument<'d> {
    DatabaseDocument {
        name: document.name.clone(),
        charset: document.charset.clone(),
        collation: document.collation.clone(),
        server: document.server.clone(),
        tables,
        views: Cow::Owned(views),
        routines: Cow::Owned(routines),
    }
}

/// Where a named load was made, for the conditions whose `cause` names the
/// population.
fn sought<'a>(opened: &'a Opened, document: &'a DatabaseDocument<'_>) -> named::Sought<'a> {
    named::Sought::Catalogue {
        entry: opened.entry(),
        database: &document.name,
    }
}
