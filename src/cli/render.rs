//! The third arm of the tree: `tpl render`, which joins the other two.
//!
//! It reads the catalogue, reads a template, renders it, and writes the result
//! to stdout. The pipeline is fixed and has no other steps, and one invocation
//! produces exactly one rendered result, per `FR-RND-002`.
//!
//! # The two context sources, and the one shape they produce
//!
//! `FR-RND-023` binds the model to the `database` variable, and the model
//! arrives one of two ways:
//!
//! | Source | What runs | Requirement |
//! |---|---|---|
//! | The selected database | [`super::source::Reader`], cache first, each object file read when the template first reaches it | `FR-RND-026`, `FR-CACHE-006`, `FR-CACHE-038` |
//! | A `--context` document | The file, or stdin for `-`, read back through `crate::model::document` | `FR-RND-016`, `FR-RND-017`, `FR-RND-022` |
//!
//! **The result must not betray which.** That is not a rule this module has to
//! remember: both paths produce a
//! [`DatabaseDocument`] and hand it
//! to the same [`produce`], so the context a template sees is assembled once,
//! from one type, by [`context::assemble`]. The one exception is a render
//! served from the cache, whose members are read from their files as the
//! template reaches them: [`lazily`] binds the object and renders over
//! [`context::assemble_shelved`], which writes the same five variables and
//! answers every read of `database` as the whole document would. What differs
//! between the sources is carried where it is true and nowhere else — the
//! population an object was sought in, which `FR-ERR-034` obliges a `66` and a
//! `64` to name, and which [`named::Sought`] makes a parameter.
//!
//! # The order the steps run in
//!
//! `FR-ERR-006` fixes it, and every step of it is reached here:
//!
//! | Step | What runs | Requirement |
//! |---|---|---|
//! | 1 | `--set`, the object flags, and `--context` beside `-d/--database` | `FR-RND-005`, `FR-RND-011` … `FR-RND-014`, `FR-RND-018` |
//! | 2, 3 | Project discovery, the trust checks, and `.tpl/.cfg` | `FR-PROJ-004` … `FR-PROJ-011`, `FR-CONF-001` … `FR-CONF-022` |
//! | 4 | The template name | `FR-RND-029`, `FR-TMPL-027` |
//! | 5, 6 | The entry, the cache, and the server on a miss — **skipped** on the `--context` path | `FR-RND-019`, `FR-RND-022`, `FR-RND-026`, `FR-CACHE-038` |
//! | 7 | The object the invocation bound | `FR-RND-032` |
//! | 8 | The render | `FR-RND-030`, `FR-RND-031`, `FR-RND-033` |
//!
//! Step 1 is this module's own and runs before anything is discovered or
//! opened, which is what makes a malformed `--set` a `64` on a machine with no
//! project. The order among its three refusals is the order [`super::rules`]
//! states for its own four: a refusal that is a property of **one** argument
//! precedes a refusal that is a property of **two**, so a `--set` that is not a
//! pair is reported however the object flags read.
//!
//! Step 4 is performed explicitly, on both context paths, by resolving the
//! template name as soon as the project has been read and before anything else
//! is reached. `tpl render` is the only command that reaches both sides of that
//! step, and what follows it is everything an invocation that cannot render has
//! no use for: the entry, with the `${VAR}` expansion of `FR-CONF-015` and the
//! `password_command` child of `FR-CONF-024`; the one connection of
//! `NFR-PERF-004`; the catalogue read of `NFR-PERF-001`; and the store written
//! under `FR-CACHE-030`. `OD-15` already splits the resolution by who asks and
//! makes the loader resolve again, so the cost of stating the step is one
//! `realpath` on a path already in the page cache.
//!
//! Two visible consequences follow from the position, and `FR-ERR-006` records
//! both as accepted: an invocation carrying two faults reports the template
//! rather than the entry or the document, because `FR-ERR-007` reports the
//! first condition that fails; and under `--context -` the document is no
//! longer read, so a producer at the other end of the pipe is cut off instead
//! of drained.
//!
//! # What this command does not have
//!
//! `FR-RND-027` withholds `--format` and `--pretty`: the result is the rendered
//! text and has no alternative representation. `FR-RND-028` withholds
//! `--output`, `--output-dir`, `--output-name`, `--no-clobber` and `--dry-run`:
//! the result goes to stdout and where it lands is the caller's business,
//! through a redirection the caller writes. None of the seven is declared, so
//! each is the ordinary unknown-flag `64` of `FR-CLI-019` rather than a case of
//! its own.
//!
//! **A miss found during the render returns the invocation to step 6.** A
//! render served from the cache reads an object file when the template first
//! reaches it, and a file that is then a miss abandons the render, per
//! `FR-CACHE-039`: the server is read, the cache written, and steps 7 and 8
//! run again over the server's document, so a condition raised after that
//! carries the code of the step that raised it. A condition the abandoned
//! render raised before it reached the miss is the invocation's, and no
//! connection is opened for it. So is a render bound it crosses, before the
//! miss or after it: the abandoned render keeps all four bounds until it has
//! returned, and crossing one ends the invocation with `65` and that bound's
//! `cause`, with no server read (`FR-CACHE-039`, `FR-RND-038`). Only an
//! abandoned render that returned within every bound returns the invocation to
//! step 6.
//!
//! # A failed render leaves stdout empty
//!
//! `FR-RND-034` admits **at most one** incomplete result on stdout when a
//! render fails, and what this module leaves is none:
//! [`crate::render::Environment::render`] returns the whole text or the
//! condition, and nothing is written until it has returned the text. A render
//! abandoned under `FR-CACHE-039` writes nothing either: its text is dropped
//! once it has returned, and so is its condition unless that is a render
//! bound. The two paths that can interrupt a render in progress are the
//! deadline and the memory limit of [`bounded`], which terminate the process
//! through
//! [`std::process::exit`] — running no destructor, so a buffered stdout is
//! discarded rather than flushed, which is the outcome `FR-ERR-033` requires.

mod context;

use std::collections::BTreeMap;
use std::io::Read as _;
use std::path::{Path, PathBuf};
use std::sync::mpsc::{self, RecvTimeoutError};

use minijinja::Value;

use super::Ending;
use super::globals::Globals;
use super::local::{Caching, Object};
use super::schema::named::{self, Sought};
use super::source::{self, Reader, Served, leak};
use crate::cache::Shelved;
use context::Store;
use std::sync::Arc;
use std::time::{Duration, Instant};

use crate::deadline::{Bound, Phase};
use crate::error::{CatalogueObjectKind, ContextFault, Error, Position};
use crate::mariadb::catalogue::completeness;
use crate::model::document::{self, DatabaseDocument};
use crate::output;
use crate::project::settings;
use crate::render::{Environment, RenderMemoryLimit};

/// The context variable a bound table is read through (`FR-RND-023`).
const TABLE: &str = "table";

/// The context variable a bound view is read through.
const VIEW: &str = "view";

/// The context variable a bound routine is read through.
const ROUTINE: &str = "routine";

/// `--table`, as `FR-RND-003` spells it.
const TABLE_FLAG: &str = "--table";

/// `--view`, as `FR-RND-003` spells it.
const VIEW_FLAG: &str = "--view";

/// `--routine`, as `FR-RND-003` spells it.
const ROUTINE_FLAG: &str = "--routine";

/// `--context`, as `FR-RND-016` spells it.
const CONTEXT_FLAG: &str = "--context";

/// `-d/--database`, in the long form the tree declares it under.
const DATABASE_FLAG: &str = "--database";

/// The value of `--context` that names standard input (`FR-RND-017`).
const STDIN: &str = "-";

/// The invocation of `FR-ERR-022` the routine token of this arm belongs to.
///
/// It carries the flag and the operand's placeholder as well as the command
/// path, because `FR-RND-003` names the object by flag and `FR-RND-001` makes
/// the template a required operand: the hint of a refusal is the same
/// invocation corrected, per `FR-ERR-009`, and an invocation missing either is
/// not one a caller can run.
const ROUTINE_INVOCATION: &str = "render <template> --routine";

/// What one `tpl render` invocation supplies, beside the global flags.
///
/// It is `cfg`'s own `Supplied` under another name and exists for the same
/// reason: a command takes one argument rather than five the caller of
/// [`super::route`] would otherwise have to thread through every function
/// here.
#[derive(Debug, Clone, Copy)]
pub(super) struct Supplied<'a> {
    /// The template to render (`FR-RND-001`).
    pub(super) template: &'a str,

    /// `--table`, `--view` and `--routine` (`FR-RND-003`).
    pub(super) object: &'a Object,

    /// Every `--set` occurrence, in the order written (`FR-RND-008`).
    pub(super) set: &'a [String],

    /// Every `--context` occurrence, which `FR-CLI-014` has already reduced to
    /// at most one (`FR-RND-016`).
    pub(super) context: &'a [PathBuf],

    /// `--direct` and `--no-cache` (`FR-RND-025`).
    pub(super) caching: &'a Caching,

    /// Whether the process exits when the render returns, which decides
    /// whether the context source's buffers and the render context are freed
    /// (see [`Served`] and [`produce`]).
    pub(super) ending: Ending,
}

/// Which object the invocation bound, or the whole database (`FR-RND-003`
/// … `FR-RND-006`).
///
/// The absence of all three flags is not an error: it is the whole-database
/// form `FR-RND-006` gives it, and it binds **no** object variable — which is
/// what `BR-RND-001` relies on when it says a template written for tables never
/// receives a view.
///
/// It is [`Clone`] and not [`Copy`], for the reason [`named::Wanted`] is.
#[derive(Debug, Clone)]
enum Binding<'a> {
    /// No object flag: the whole database, and no object variable.
    Whole,
    /// `--table <name>`.
    Table(&'a str),
    /// `--view <name>`.
    View(&'a str),
    /// `--routine <name>`, in either the bare or a qualified form.
    Routine(named::Wanted<'a>),
}

impl<'a> Binding<'a> {
    /// What `object` names.
    ///
    /// `tpl cache` carries the same reduction over the same three flags and
    /// the two are written once per arm rather than shared, on the terms
    /// [`super::schema`] and [`super::template`] state for `representation`:
    /// each states the requirement that gave its command the flags, which is
    /// `FR-RND-003` and `FR-RND-005` here and `FR-CACHE-024` there.
    ///
    /// # Errors
    ///
    /// Returns [`Error::MutuallyExclusiveFlags`] — `64` — where more than one
    /// kind was given, per `FR-RND-005`, and
    /// [`Error::RoutinePrefixNotLowerCase`] for the token shape `FR-SCH-008`
    /// refuses, which is decided from the token alone and therefore precedes
    /// everything this command reads.
    fn of(object: &'a Object) -> Result<Self, Error> {
        let given: [(&'static str, Option<&'a String>); 3] = [
            (TABLE_FLAG, object.table.first()),
            (VIEW_FLAG, object.view.first()),
            (ROUTINE_FLAG, object.routine.first()),
        ];
        let mut named = given.iter().filter(|(_, value)| value.is_some());

        let Some((first, value)) = named.next() else {
            return Ok(Self::Whole);
        };

        if let Some((second, _)) = named.next() {
            return Err(Error::MutuallyExclusiveFlags {
                first: (*first).to_owned(),
                second: (*second).to_owned(),
            });
        }

        let name = value.map_or("", String::as_str);

        Ok(match *first {
            TABLE_FLAG => Self::Table(name),
            VIEW_FLAG => Self::View(name),
            _ => Self::Routine(named::routine_token(name, ROUTINE_INVOCATION)?),
        })
    }

    /// The object variable this binding contributes to the context, or
    /// [`None`] for the whole-database form (`FR-RND-023`, `FR-RND-032`).
    ///
    /// This is step 7 of `FR-ERR-006`, and it runs over whichever context
    /// source produced `document`: `at` carries which that was, so the `66` and
    /// the `64` name the population they were actually sought in.
    ///
    /// # Errors
    ///
    /// Returns what [`named::table`], [`named::view`] and [`named::routine`]
    /// return: the `66` of `FR-RND-032` with the nearest matches among the
    /// objects of that kind the source does carry, the `64` of `FR-SCH-010`
    /// for a bare routine name that reaches both namespaces, and the `77` of
    /// `FR-PRIV-003` for an object that came back short.
    fn bind(
        &self,
        document: &DatabaseDocument<'_>,
        at: Sought<'_>,
    ) -> Result<Option<(&'static str, Value)>, Error> {
        Ok(match self {
            Self::Whole => None,
            Self::Table(name) => Some((
                TABLE,
                Value::from_serialize(named::table(document, name, at)?),
            )),
            Self::View(name) => Some((
                VIEW,
                Value::from_serialize(named::view(document, name, at)?),
            )),
            Self::Routine(wanted) => Some((
                ROUTINE,
                Value::from_serialize(named::routine(document, wanted, at, ROUTINE_INVOCATION)?),
            )),
        })
    }
}

impl Binding<'_> {
    /// Step 7 of `FR-ERR-006` over a render served from the cache under
    /// `FR-CACHE-038`: the object is sought among the members the listing
    /// names, and its file is the one object file read before the render
    /// starts.
    ///
    /// The lookup is [`named::member`]'s and [`named::routine_member`]'s,
    /// which [`Binding::bind`] reaches through [`named::table`],
    /// [`named::view`] and [`named::routine`], over the same names in the same
    /// order — so the `66` and the `64` are the conditions an up-front read of
    /// the same files would have raised. Completeness is checked on the member
    /// read, as those three check it.
    ///
    /// # Errors
    ///
    /// Returns what [`Binding::bind`] returns.
    fn bind_shelved(&self, store: &Store, at: Sought<'_>) -> Result<Chosen, Error> {
        let shelved = store.shelved();

        Ok(match self {
            Self::Whole => Chosen::Bound(None),
            Self::Table(name) => {
                let (index, _) =
                    named::member(shelved.tables(), CatalogueObjectKind::Table, name, at)?;
                let Some(table) = store.table(index) else {
                    return Ok(Chosen::Missed);
                };
                completeness::of_table(&table)?;

                Chosen::Bound(Some((TABLE, Value::from_serialize(&*table))))
            }
            Self::View(name) => {
                let (index, _) =
                    named::member(shelved.views(), CatalogueObjectKind::View, name, at)?;
                let Some(view) = store.view(index) else {
                    return Ok(Chosen::Missed);
                };
                completeness::of_view(&view)?;

                Chosen::Bound(Some((VIEW, Value::from_serialize(&*view))))
            }
            Self::Routine(wanted) => {
                let (index, _) =
                    named::routine_member(shelved.routines(), wanted, at, ROUTINE_INVOCATION)?;
                let Some(routine) = store.routine(index) else {
                    return Ok(Chosen::Missed);
                };
                completeness::of_routine(&routine)?;

                Chosen::Bound(Some((ROUTINE, Value::from_serialize(&*routine))))
            }
        })
    }
}

/// What [`Binding::bind_shelved`] found.
#[derive(Debug)]
enum Chosen {
    /// The object variable, or [`None`] for the whole-database form.
    Bound(Option<(&'static str, Value)>),
    /// The bound object's file is a miss (`FR-CACHE-033`), decided before the
    /// render starts.
    Missed,
}

/// How a render served from the cache under `FR-CACHE-038` ended.
#[derive(Debug)]
enum Lazily {
    /// It rendered, and its result is on stdout.
    Served,
    /// A file read before the render started is a miss, so no render started.
    Missed,
    /// A file the render reached is a miss, so the render was abandoned under
    /// `FR-CACHE-039`, and it returned within every render bound: nothing of
    /// it reached stdout. It carries the `now` the render that follows uses.
    Abandoned(String),
}

/// Everything one render needs once its context source has answered.
///
/// It is a value rather than five parameters because [`produce`] is reached
/// from two places — the read-through path and the document path — and a
/// parameter list threaded through both is a list the two can come to disagree
/// about.
#[derive(Debug)]
struct Assembly<'a> {
    /// The environment of the resolved project (`FR-TMPL-023`).
    environment: &'a Environment,

    /// The template to render, as the caller named it (`FR-RND-001`).
    template: &'a str,

    /// What the invocation bound (`FR-RND-003` … `FR-RND-006`).
    binding: &'a Binding<'a>,

    /// The `--set` entries of this invocation (`FR-CTX-026`).
    defined: &'a BTreeMap<&'a str, &'a str>,

    /// What bounds the render (`FR-RND-033`, `FR-GLOB-012`).
    deadline: Bound,

    /// What follows the render once it returns.
    ending: Ending,
}

/// Runs `tpl render` (`FR-RND-001` … `FR-RND-034`, `FR-CACHE-038`,
/// `FR-CACHE-039`).
///
/// # Errors
///
/// Returns the condition of the first step of `FR-ERR-006` that fails, in the
/// order this module's own documentation states: the `64` of `FR-RND-005`,
/// `FR-RND-011` through `FR-RND-014` and `FR-RND-018`; the `78` of a project
/// or a configuration that does not describe a read; the `66` of `FR-RND-029`
/// for a template name that resolves to nothing; the `78` of an entry that
/// does not describe a read; the `65` of `FR-RND-020` for a `--context`
/// document that does not match the contract, and the `69` and `77` of a
/// server that did not answer or did not permit the read; the `66` of
/// `FR-RND-032`; and the `65` of `FR-RND-030`, `FR-RND-031`, `FR-RND-033`,
/// `FR-RND-036` and `FR-RND-037`.
pub(super) fn run<W: std::io::Write>(
    out: &mut W,
    globals: &Globals,
    supplied: &Supplied<'_>,
) -> Result<(), Error> {
    run_body(out, globals, supplied).map_err(|refused| with_template(refused, supplied.template))
}

/// A refusal whose hint writes the corrected `tpl render` invocation, given
/// the template this invocation named in place of the `<template>`
/// placeholder of [`ROUTINE_INVOCATION`]: the template is known here, and
/// `BR-ERR-004` asks for the value rather than a placeholder.
fn with_template(refused: Error, named: &str) -> Error {
    match refused {
        Error::RoutinePrefixNotLowerCase {
            token,
            prefix,
            name,
            invocation,
            ..
        } => Error::RoutinePrefixNotLowerCase {
            token,
            prefix,
            name,
            invocation,
            template: Some(named.to_owned()),
        },
        Error::AmbiguousRoutineName {
            name,
            entry,
            database,
            invocation,
            ..
        } => Error::AmbiguousRoutineName {
            name,
            entry,
            database,
            invocation,
            template: Some(named.to_owned()),
        },
        Error::AmbiguousRoutineInContext {
            name,
            path,
            database,
            invocation,
            ..
        } => Error::AmbiguousRoutineInContext {
            name,
            path,
            database,
            invocation,
            template: Some(named.to_owned()),
        },
        other => other,
    }
}

/// The body of [`run`].
fn run_body<W: std::io::Write>(
    out: &mut W,
    globals: &Globals,
    supplied: &Supplied<'_>,
) -> Result<(), Error> {
    // Step 1 of FR-ERR-006, in the order this module's documentation states.
    let defined = context::vars(supplied.set)?;
    let binding = Binding::of(supplied.object)?;
    let document = document_flag(globals, supplied.context)?;

    match document {
        Some(path) => from_document(out, globals, supplied, &binding, &defined, path),
        None => from_catalogue(out, globals, supplied, &binding, &defined),
    }
}

/// The `--context` document this invocation names, or [`None`]
/// (`FR-RND-016`, `FR-RND-018`, `FR-RND-019`).
///
/// # Errors
///
/// Returns [`Error::MutuallyExclusiveFlags`] — `64` — where `--context` was
/// given together with `-d/--database` **on the command line**, per
/// `FR-RND-018`. The test is the flag and never the resolved entry, because
/// `FR-RND-019` makes `core.database` no conflict at all: a project that names
/// a default entry renders from a document without the file having to be
/// changed.
fn document_flag<'a>(globals: &Globals, context: &'a [PathBuf]) -> Result<Option<&'a Path>, Error> {
    let Some(path) = context.first() else {
        return Ok(None);
    };

    if !globals.database.is_empty() {
        return Err(Error::MutuallyExclusiveFlags {
            first: CONTEXT_FLAG.to_owned(),
            second: DATABASE_FLAG.to_owned(),
        });
    }

    Ok(Some(path))
}

/// Renders from the selected database, through the cache (`FR-RND-026`).
///
/// `FR-RND-023` binds `database` whatever object the invocation named, so a
/// template may walk every collection even when one object is bound, and
/// `FR-CTX-023` requires every object a carried reference names to be present.
/// The whole `database` is therefore always bound, and what differs between
/// the two sources is **when** its members are read:
///
/// | Source | Read before the render | Read during it | Requirement |
/// |---|---|---|---|
/// | The cache | `meta.json`, `database.json`, each collection's listing, the bound object's file | every other object file, when the template first reaches it | `FR-CACHE-038` |
/// | The server, on a miss | the whole catalogue, then written to the cache | nothing | `FR-CACHE-007` |
///
/// A miss among the files read before the render is answered as any miss is.
/// A miss among the files read during it abandons the render, which has then
/// written nothing, and the invocation is a miss: the server is read once, the
/// cache written, and the render made again from the server's document, with
/// the first render's `now` and a render deadline of its own (`FR-CACHE-039`).
///
/// # Errors
///
/// Returns what [`source::project`], [`Environment::resolve`],
/// [`Reader::open_from`] and [`Reader::read_through`] return, and what
/// [`produce`] and [`lazily`] return.
fn from_catalogue<W: std::io::Write>(
    out: &mut W,
    globals: &Globals,
    supplied: &Supplied<'_>,
    binding: &Binding<'_>,
    defined: &BTreeMap<&str, &str>,
) -> Result<(), Error> {
    let reader = Reader::new(globals, Some(supplied.caching), supplied.ending);
    // Steps 2 and 3, made once: the template root of FR-TMPL-023, the render
    // deadline of FR-CONF-004 and the render bounds of FR-CONF-045 are all
    // properties of the project this read is made through, and a second walk
    // could resolve a second project between the steps.
    let (project, configuration) = source::project(reader.tpl_dir())?;
    let environment = Environment::bounded(project.root(), settings::render_bounds(&configuration));

    // Step 4, before an entry is resolved: FR-RND-029 through FR-TMPL-027. An
    // invocation refused here has spared itself the ${VAR} expansion of
    // FR-CONF-015, the password_command child of FR-CONF-024, the one
    // connection of NFR-PERF-004, the catalogue read and the store write.
    environment.resolve(supplied.template)?;

    // Steps 5 and 6.
    let opened = reader.open_from(&project, configuration)?;
    let render_timeout = opened.deadlines().of(Phase::Render);
    let mut assembly = Assembly {
        environment: &environment,
        template: supplied.template,
        binding,
        defined,
        deadline: reader.clock().bound(render_timeout),
        ending: supplied.ending,
    };
    let mut now = None;

    // FR-CACHE-013: `--direct` skips the lookup outright.
    if !supplied.caching.direct
        && let Some(shelved) = opened.cache.shelved()
    {
        match lazily(out, &assembly, shelved, opened.entry())? {
            Lazily::Served => return Ok(()),
            Lazily::Missed => {}
            Lazily::Abandoned(at) => {
                // FR-CACHE-039 and FR-CONF-005: the render that produces the
                // result has the whole render deadline, still composed with
                // the overall budget of FR-GLOB-011, and the abandoned
                // render's `now`.
                assembly.deadline = reader.clock().bound(render_timeout);
                now = Some(at);
            }
        }
    }

    reader.read_through(&opened, |served, _, entry| {
        produce(
            out,
            &assembly,
            served,
            Sought::Catalogue {
                entry,
                database: &served.name,
            },
            now,
        )
    })
}

/// Serves one render from the cache files `shelved` names, reading each
/// object file when the template first reaches it (`FR-CACHE-038`,
/// `FR-CACHE-039`).
///
/// It is steps 7 and 8 of `FR-ERR-006`, as [`produce`] makes them, over a
/// `database` whose members are read on demand. A condition either step raises
/// is answered as [`produce`] answers it, and no connection is opened for it,
/// **unless** the render reached a miss first: once a member is a miss, the
/// render is abandoned whatever it then did, because what it rendered is not
/// the document the cache could serve.
///
/// # Errors
///
/// Returns what [`Binding::bind_shelved`] returns for step 7, and for step 8
/// what [`produce`] returns, where no member the render reached was a miss.
fn lazily<W: std::io::Write>(
    out: &mut W,
    assembly: &Assembly<'_>,
    shelved: Shelved,
    entry: &str,
) -> Result<Lazily, Error> {
    let Some(store) = Store::open(shelved, assembly.ending) else {
        return Ok(Lazily::Missed);
    };
    let store = Arc::new(store);
    let ended = attempt(out, assembly, &store, entry);

    match ended {
        // PERF: as for the context in `produce`, the store owns memory only,
        // and an exiting process has no use for freeing it block by block.
        Ok(Lazily::Served) => assembly.ending.release(store),
        // FR-RND-038: a render that did not produce the result frees what it
        // read, so the render that follows is not charged for it under
        // FR-RND-039.
        _ => drop(store),
    }

    ended
}

/// The render [`lazily`] makes, once its store is open.
///
/// # Errors
///
/// Returns what [`lazily`] returns.
fn attempt<W: std::io::Write>(
    out: &mut W,
    assembly: &Assembly<'_>,
    store: &Arc<Store>,
    entry: &str,
) -> Result<Lazily, Error> {
    // 7 — FR-RND-032, over the members the listing names.
    let at = Sought::Catalogue {
        entry,
        database: store.name(),
    };
    let bound = match assembly.binding.bind_shelved(store, at)? {
        Chosen::Bound(bound) => bound,
        Chosen::Missed => return Ok(Lazily::Missed),
    };

    // 8 — as in `produce`, with `now` read at render time and once.
    let now = context::now();
    let context = context::assemble_shelved(store, bound, assembly.defined, &now);
    let produced = bounded(
        assembly.deadline,
        assembly.environment.bounds().memory_limit,
        || assembly.environment.render(assembly.template, &context),
    );

    // FR-CACHE-039 and FR-RND-034: a render that reached a miss is abandoned,
    // and nothing of it is written — its text, or its condition, is dropped.
    // FR-RND-038: its values are **freed**, never leaked as a finished
    // render's are, so the heap the following render is held to under
    // FR-RND-039 does not carry them.
    //
    // FR-CACHE-039 and FR-RND-038: an abandoned render that crossed a render
    // bound, before the miss or after it, ends the invocation with 65 and
    // that bound's cause, and the server is not read. The deadline and the
    // memory limit end it from the watchdog of `bounded`; render fuel and the
    // output limit end it from inside, and are answered here. Any other
    // condition of the abandoned render is its reading of the miss, and is
    // dropped with it.
    if store.missed() {
        drop(context);
        return match produced {
            Err(
                crossed @ (Error::RenderFuelExhausted { .. }
                | Error::RenderOutputLimitExceeded { .. }),
            ) => Err(crossed),
            _ => Ok(Lazily::Abandoned(now)),
        };
    }

    assembly.ending.release(context);

    output::emit_verbatim(out, &produced?)?;

    Ok(Lazily::Served)
}

/// Renders from a `--context` document (`FR-RND-016`, `FR-RND-022`).
///
/// Steps 5 and 6 of `FR-ERR-006` are skipped entirely: no entry is selected,
/// so `password_command` does not run, and no connection is opened and no file
/// of the cache is read or written. Steps 2, 3 and 4 are **not** skipped —
/// `FR-TMPL-023` makes the template root a property of the resolved project and
/// `FR-CONF-004` resolves the render deadline from `[core]`, and neither is
/// reachable without the first two; the third is the template name, which
/// `FR-ERR-006` evaluates on this path exactly where it evaluates it on the
/// other.
///
/// **The document is read after step 4 and not before it**, which is what
/// `FR-ERR-006` records as an accepted cost: a document that does not match the
/// contract is reported only where the template resolves, and under
/// `--context -` a producer at the other end of the pipe is cut off rather than
/// drained.
///
/// # Errors
///
/// Returns what [`source::project`] and [`Environment::resolve`] return, what
/// [`read_document`] returns for a document that cannot be read, the `65` of
/// `FR-RND-020` for one that does not match the contract, and what [`produce`]
/// returns.
fn from_document<W: std::io::Write>(
    out: &mut W,
    globals: &Globals,
    supplied: &Supplied<'_>,
    binding: &Binding<'_>,
    defined: &BTreeMap<&str, &str>,
    path: &Path,
) -> Result<(), Error> {
    let reader = Reader::new(globals, Some(supplied.caching), supplied.ending);
    // Steps 2 and 3, and the render bounds of FR-CONF-045 from the same file.
    let (project, configuration) = source::project(reader.tpl_dir())?;
    let environment = Environment::bounded(project.root(), settings::render_bounds(&configuration));

    // Step 4, before the document is read: FR-RND-029 through FR-TMPL-027.
    environment.resolve(supplied.template)?;

    let assembly = Assembly {
        environment: &environment,
        template: supplied.template,
        binding,
        defined,
        deadline: reader
            .clock()
            .bound(settings::deadlines(&configuration).of(Phase::Render)),
        ending: supplied.ending,
    };

    let bytes = read_document(path)?;
    let mut hand = |served: Served<'_, '_>| {
        produce(
            out,
            &assembly,
            served,
            Sought::Document {
                path,
                database: &served.name,
            },
            None,
        )
    };

    // PERF: under Ending::Process the bytes and the model are leaked, so the
    // document borrows them for the rest of the process and the render keeps
    // it without a copy (`super::source`'s documentation).
    match assembly.ending {
        Ending::Process => {
            let model = leak(document::read(bytes.leak(), path)?);

            hand(Served::leaked(leak(document::context(model)?)))
        }
        Ending::Caller => {
            let model = document::read(&bytes, path)?;

            hand(Served::borrowed(&document::context(&model)?))
        }
    }
}

/// Steps 7 and 8 of `FR-ERR-006`, over whichever source produced `document`.
///
/// This is the one place a render happens, which is what makes the promise of
/// this sprint structural: the result cannot betray which source the model came
/// from, because the only thing either source contributes is this argument.
///
/// The template name is **not** resolved here. `FR-ERR-006` evaluates it at
/// step 4, which both callers reach before they resolve an entry or read a
/// document, so what arrives here is a name already known to resolve.
///
/// # Errors
///
/// Returns what [`Binding::bind`] returns for step 7, and for step 8 the `65`
/// of `FR-RND-030` for a syntax error, of `FR-RND-031` for an evaluation
/// failure, of `FR-RND-033` for the deadline, and of `FR-RND-036` and
/// `FR-RND-037` for the render fuel and the render output limit, and
/// [`Error::StdoutUnwritable`] where the stream refused the write.
fn produce<W: std::io::Write>(
    out: &mut W,
    assembly: &Assembly<'_>,
    document: Served<'_, '_>,
    at: Sought<'_>,
    now: Option<String>,
) -> Result<(), Error> {
    // 7 — FR-RND-032.
    let bound = assembly.binding.bind(&document, at)?;

    // 8 — FR-RND-030, FR-RND-031, FR-RND-033. `now` is read here, which is
    // render time, and once, which is FR-CTX-029 — unless a render abandoned
    // under FR-CACHE-039 already read it, whose value this render keeps.
    let at = now.unwrap_or_else(context::now);
    let context = context::assemble(document, bound, assembly.defined, &at);
    let produced = bounded(
        assembly.deadline,
        assembly.environment.bounds().memory_limit,
        || assembly.environment.render(assembly.template, &context),
    );

    // PERF: the context holds every member the template read, and under
    // Ending::Caller a copy of the catalogue; when it held the whole catalogue
    // converted, freeing it block by block cost 2.35 ms of a 21.8 ms render of
    // `WL-001` (`BENCHMARKS.md`, 2026-09-22). Where the process exits as soon
    // as this command returns, the operating system reclaims that memory
    // anyway, so the value is leaked instead; a caller that carries on — a
    // test — still frees it. It is released on the failure path as well, and
    // it owns memory only, so stdout, its flush and the exit code are
    // untouched either way.
    assembly.ending.release(context);
    let produced = produced?;

    // FR-RND-028, and FR-OUT-019, which exempts a render's result from the
    // escaping of FR-OUT-018: the bytes the template produced, and no others.
    output::emit_verbatim(out, &produced)
}

/// The bytes of the `--context` document (`FR-RND-016`, `FR-RND-017`).
///
/// `-` is standard input, which is what makes `tpl schema dump | tpl render x
/// --context -` the pipeline `FR-RND-017` writes.
///
/// A document that is not valid UTF-8 is the `65` of `FR-RND-020` and not a
/// read failure: RFC 8259 makes UTF-8 the encoding of a JSON text, so bytes
/// that are not UTF-8 are bytes that are not well-formed JSON. The position
/// reported is the first byte the decoder could not read, which is what the
/// `65` row of `FR-ERR-034` obliges beside the path.
///
/// # Errors
///
/// Returns [`Error::ContextDocumentUnreadable`] — `74` — where the file or the
/// stream refused the read, and [`Error::ContextDocumentMalformed`] — `65` —
/// where the bytes are not UTF-8.
fn read_document(path: &Path) -> Result<String, Error> {
    let unreadable = |returned| Error::ContextDocumentUnreadable {
        path: path.to_owned(),
        returned,
    };

    let bytes = if path == Path::new(STDIN) {
        let mut read = Vec::new();
        std::io::stdin()
            .lock()
            .read_to_end(&mut read)
            .map_err(unreadable)?;

        read
    } else {
        std::fs::read(path).map_err(unreadable)?
    };

    String::from_utf8(bytes).map_err(|reported| {
        let read = reported.utf8_error().valid_up_to();

        Error::ContextDocumentMalformed {
            path: path.to_owned(),
            fault: ContextFault::NotJson(position(&reported.as_bytes()[..read])),
        }
    })
}

/// The one-based line and column immediately after `read`.
fn position(read: &[u8]) -> Position {
    let line = read.iter().filter(|byte| **byte == b'\n').count() + 1;
    let column = read.len()
        - read
            .iter()
            .rposition(|byte| *byte == b'\n')
            .map_or(0, |at| at + 1)
        + 1;

    Position { line, column }
}

/// How often the watchdog of [`bounded`] reads the heap count (`FR-RND-039`,
/// `ADR-011`).
///
/// The overshoot `FR-RND-039` admits is what a render allocates within one
/// interval; reading the count is three atomic loads and allocates nothing.
/// `ADR-011` proposes the value, for the technical specification to fix.
const HEAP_POLL: Duration = Duration::from_millis(10);

/// Runs one render under the deadline of `FR-RND-033`, per `OD-12`, and under
/// the render memory limit of `FR-RND-039`, per `ADR-011`.
///
/// The render runs on the calling thread and a watchdog thread waits on a
/// channel with `recv_timeout`; the render signals the channel when it
/// completes. The watchdog wakes at the deadline, and every [`HEAP_POLL`]
/// before it, to read the heap count the process's allocator keeps. If the
/// deadline arrives first, or the count is observed above `memory`, the
/// watchdog writes the four labelled lines of `FR-ERR-008` for that `65` and
/// terminates the process with that status — which is what `FR-GLOB-013` asks
/// for the deadline, the phase in progress being the one the watchdog was
/// created for, and what `ADR-011` fixes for the memory limit. Render fuel and
/// the render output limit end the render from inside it, so whichever of the
/// four bounds is crossed first is the one reported (`FR-RND-038`).
///
/// Where the binary installed no heap counter — an in-process test — the
/// watchdog waits for the deadline alone, as it did before `FR-RND-039`.
///
/// Three requirements make that admissible rather than merely convenient.
/// `FR-RND-034` already admits at most one incomplete result on stdout when a
/// render fails, so an interrupted render breaks no promise about stdout;
/// `NFR-DET-001` puts stderr outside the contract, so the watchdog writing to
/// it interleaves nothing that is contract; and [`std::process::exit`] runs no
/// destructor, so a buffered stdout is discarded rather than emitted.
///
/// **It is not the speculative parallelism this project forbids.** The thread
/// performs no work of the invocation and makes nothing faster: it waits, and
/// the render is exactly as fast as it would have been without it. `OD-12`
/// records the same reasoning for the `password_command` child.
///
/// A bound already spent does not start the phase at all, on the same terms
/// `OD-12` records for that child: a phase started in order to be abandoned at
/// once is a thread and a process termination bought for nothing.
///
/// **Nothing of a catalogue read is alive when the render starts**
/// (`FR-RND-040`): the render is refused, as a violated invariant, while this
/// thread holds an open connection or a driver runtime. Every caller reaches
/// here with the read already closed, so the check is the requirement's
/// in-process observation rather than a path a caller can take.
///
/// **A render abandoned under `FR-CACHE-039` is watched like any other**,
/// until it has returned. `FR-CACHE-039` and `FR-RND-038` keep all four bounds
/// on it, before the miss and after it: a render that reaches the miss through
/// a lookup function of `FR-ENV-020` can go on evaluating, and a watchdog that
/// stood down at the miss would leave its deadline and its memory unbounded.
/// Crossing either ends the invocation here with `65` and that bound's
/// `cause`, before any server read — which is the outcome those requirements
/// fix for an abandoned render.
///
/// # Errors
///
/// Returns [`Error::InternalInvariant`] — `70` — where a connection or a
/// driver runtime is alive, [`Error::RenderDeadlineExceeded`] — `65` — where
/// the bound was already spent, and whatever `render` returns.
fn bounded<T>(
    deadline: Bound,
    memory: RenderMemoryLimit,
    render: impl FnOnce() -> Result<T, Error>,
) -> Result<T, Error> {
    crate::error::ensure_invariant(
        crate::mariadb::quiescent(),
        "no connection is open and no runtime of the database driver is alive while a template \
         is evaluated",
    )?;

    let expired = Error::RenderDeadlineExceeded {
        bound: deadline.bound(),
        limit: deadline.limit(),
    };

    if deadline.expired() {
        return Err(expired);
    }

    let (finished, waiting) = mpsc::channel::<()>();
    let ends = Instant::now() + deadline.remaining();
    let timer = std::thread::spawn(move || {
        loop {
            let left = ends.saturating_duration_since(Instant::now());
            let wait = match crate::heap::allocated() {
                Some(_) => left.min(HEAP_POLL),
                None => left,
            };

            if !matches!(waiting.recv_timeout(wait), Err(RecvTimeoutError::Timeout)) {
                // The render finished and dropped the sender.
                return;
            }

            if Instant::now() >= ends {
                terminate(&expired);
            }

            if let Some(held) = crate::heap::allocated()
                && memory.crossed_by(held)
            {
                terminate(&Error::RenderMemoryLimitExceeded {
                    limit: memory.get(),
                });
            }
        }
    });

    let began = Instant::now();
    let produced = render();

    // The phase is over whichever way it ended, so the watchdog is woken and
    // joined: nothing of this invocation outlives the invocation.
    drop(finished);
    let _ = timer.join();

    // FR-GLOB-017: the phase ran, and this is how long it took. It is written
    // after the watchdog is joined so that the line cannot interleave with the
    // four labelled lines a bound would have written from that thread.
    //
    // A render a bound of the watchdog **did** interrupt reports nothing, and
    // cannot: that path leaves the process from inside the watchdog, which is
    // what FR-RND-033 and FR-RND-039 require of it.
    crate::diagnostics::emit::phase_ran(Phase::Render, began.elapsed());

    produced
}

/// Reports `condition` and ends the process with its code, from the watchdog
/// of [`bounded`].
///
/// [`std::process::exit`] runs no destructor, so stdout, still buffered, is
/// discarded and nothing further reaches it (`FR-RND-034`).
fn terminate(condition: &Error) -> ! {
    crate::diagnostics::report(condition);

    std::process::exit(i32::from(condition.exit_code()));
}

#[cfg(test)]
mod tests {
    use super::{Binding, Supplied, bounded, document_flag, position, run};
    use crate::cli::Ending;
    use crate::cli::globals::Globals;
    use crate::cli::local::{Caching, Object};
    use crate::cli::schema::named::Sought;
    use crate::cli::source::Served;
    use crate::deadline::{Bound, Seconds};
    use crate::error::Error;
    use crate::model::document;
    use crate::model::routine::{Routine, RoutineKind};
    use crate::project::scratch::Scratch;
    use crate::render::RenderMemoryLimit;
    use std::num::NonZeroU64;
    use std::path::{Path, PathBuf};

    /// The globals a test invocation carries.
    fn globals(tpl_dir: Option<&Path>, database: Option<&str>) -> Globals {
        Globals {
            database: database.map(str::to_owned).into_iter().collect(),
            tpl_dir: tpl_dir.map(Path::to_path_buf).into_iter().collect(),
            timeout: Vec::new(),
            verbose: 0,
            quiet: false,
            help: false,
            version: false,
        }
    }

    /// The object flags a test invocation carries.
    fn object(table: &[&str], view: &[&str], routine: &[&str]) -> Object {
        let owned = |values: &[&str]| values.iter().map(|value| (*value).to_owned()).collect();

        Object {
            table: owned(table),
            view: owned(view),
            routine: owned(routine),
        }
    }

    /// A project carrying `templates`, with no configuration file.
    ///
    /// An absent `.tpl/.cfg` is a project with an empty configuration, per
    /// `FR-PROJ-001`, so the four `[core]` deadlines are the built-in defaults
    /// of `FR-CONF-002` and nothing has to be chmodded for the trust checks.
    fn project(scratch: &Scratch, templates: &[(&str, &str)]) -> PathBuf {
        scratch.directory("project/.tpl/templates");

        for (name, source) in templates {
            scratch.file(&format!("project/.tpl/templates/{name}"), source);
        }

        scratch.canonical("project/.tpl")
    }

    /// The document `tpl schema dump` would emit for the fixture model.
    fn dumped() -> String {
        let model = document::fixture::database();
        let document =
            document::dump(&model, crate::output::Source::Server).expect("the fixture is coherent");
        let mut bytes = Vec::new();

        crate::output::emit_to(&mut bytes, &document, crate::output::Form::Compact)
            .expect("a buffer accepts every write");

        String::from_utf8(bytes).expect("the encoder emits UTF-8")
    }

    /// What `tpl render` does for one invocation over a `--context` document,
    /// and what it wrote while doing it.
    fn rendered(
        tpl_dir: &Path,
        context: &Path,
        template: &str,
        object: &Object,
        set: &[&str],
    ) -> (Result<(), Error>, String) {
        let caching = Caching {
            direct: false,
            no_cache: false,
        };
        let set: Vec<String> = set.iter().map(|entry| (*entry).to_owned()).collect();
        let context = [context.to_path_buf()];
        let supplied = Supplied {
            template,
            object,
            set: &set,
            context: &context,
            caching: &caching,
            ending: Ending::Caller,
        };
        let mut written = Vec::new();
        let result = run(&mut written, &globals(Some(tpl_dir), None), &supplied);

        (
            result,
            String::from_utf8(written).expect("a template of these tests writes UTF-8"),
        )
    }

    #[test]
    fn fr_rnd_005_more_than_one_kind_of_object_flag_is_64() {
        // FR-RND-005: two kinds in one invocation name no object, and the
        // refusal names both members of the pair, per the `64` row of
        // FR-ERR-034.
        for (table, view, routine) in [
            (&["orders"][..], &["v_sales"][..], &[][..]),
            (&["orders"][..], &[][..], &["calc_vat"][..]),
            (&[][..], &["v_sales"][..], &["calc_vat"][..]),
            (&["orders"][..], &["v_sales"][..], &["calc_vat"][..]),
        ] {
            let supplied = object(table, view, routine);
            let condition = Binding::of(&supplied).expect_err("two kinds name no object");

            assert_eq!(condition.exit_code(), 64);
            assert!(matches!(condition, Error::MutuallyExclusiveFlags { .. }));
        }
    }

    #[test]
    fn fr_rnd_006_no_object_flag_binds_the_whole_database_and_no_object_variable() {
        // FR-RND-006: the absence of all three is the whole-database form and
        // not a condition, and BR-RND-001 rests on nothing being bound.
        let supplied = object(&[], &[], &[]);

        assert!(matches!(
            Binding::of(&supplied).expect("no flag is no condition"),
            Binding::Whole
        ));
    }

    #[test]
    fn fr_rnd_018_the_context_flag_and_an_explicit_database_flag_are_64() {
        // FR-RND-018: two conflicting context sources. FR-RND-019 is the other
        // half — `core.database` in the file is not a conflict, which is why
        // the test below supplies no flag and reaches `Some`.
        let context = [PathBuf::from("context.json")];

        let condition =
            document_flag(&globals(None, Some("shop")), &context).expect_err("two context sources");

        assert_eq!(condition.exit_code(), 64);
        assert!(
            matches!(
                &condition,
                Error::MutuallyExclusiveFlags { first, second }
                    if first == "--context" && second == "--database"
            ),
            "{condition}"
        );

        assert_eq!(
            document_flag(&globals(None, None), &context).expect("one source"),
            Some(Path::new("context.json"))
        );
        assert_eq!(
            document_flag(&globals(None, Some("shop")), &[]).expect("one source"),
            None
        );
    }

    #[test]
    fn fr_rnd_023_a_render_from_a_document_carries_the_five_variables() {
        // FR-RND-023 and FR-RND-024 end to end, over the document path, which
        // FR-RND-022 keeps free of a connection and of the cache: the template
        // reads all five and the result is what it wrote.
        let scratch = Scratch::new();
        let tpl_dir = project(
            &scratch,
            &[(
                "all.jinja",
                "{{ database.name }}|{{ table.name }}|{{ vars.title }}|{{ tpl.version }}|\
                 {{ now }}|{{ now }}",
            )],
        );
        let context = scratch.file("context.json", &dumped());

        let (result, written) = rendered(
            &tpl_dir,
            &context,
            "all",
            &object(&["consignment"], &[], &[]),
            &["title=Orders"],
        );

        result.expect("the render succeeds");

        let fields: Vec<&str> = written.split('|').collect();

        assert_eq!(fields[0], "freight");
        assert_eq!(fields[1], "consignment");
        assert_eq!(fields[2], "Orders");
        assert_eq!(fields[3], env!("CARGO_PKG_VERSION"));
        // FR-CTX-029: one instant, at every reference.
        assert_eq!(fields[4], fields[5]);
        assert!(fields[4].ends_with('Z'), "{written}");
    }

    #[test]
    fn fr_rnd_024_a_document_that_carries_the_three_injected_variables_does_not_supply_them() {
        // FR-RND-024: the value is ignored, and it is ignored structurally —
        // the document contributes the `database` object and nothing else, so
        // the three at its top level reach nothing that could read them.
        let scratch = Scratch::new();
        let tpl_dir = project(
            &scratch,
            &[("three.jinja", "{{ vars }}|{{ tpl.version }}|{{ now }}")],
        );
        let supplied = dumped().replace(
            r#""source":"server""#,
            r#""source":"server","vars":{"title":"forged"},"tpl":{"version":"9.9.9"},"now":"1999-01-01T00:00:00Z""#,
        );
        let context = scratch.file("forged.json", &supplied);

        let (result, written) = rendered(&tpl_dir, &context, "three", &object(&[], &[], &[]), &[]);

        result.expect("the render succeeds");

        let fields: Vec<&str> = written.split('|').collect();

        assert_eq!(fields[0], "{}", "FR-CTX-026: no --set is an empty object");
        assert_eq!(fields[1], env!("CARGO_PKG_VERSION"));
        assert_ne!(fields[2], "1999-01-01T00:00:00Z");
    }

    #[test]
    fn fr_rnd_021_a_document_is_accepted_in_either_form_and_is_read_from_no_connection() {
        // FR-RND-021 and FR-RND-022: compact and indented are one contract,
        // and neither opens anything. The project here declares no database
        // entry at all, so an invocation that reached step 5 would be the `78`
        // of FR-GLOB-006 rather than a render.
        let scratch = Scratch::new();
        let tpl_dir = project(&scratch, &[("name.jinja", "{{ database.name }}")]);
        let model = document::fixture::database();
        let document =
            document::dump(&model, crate::output::Source::Server).expect("the fixture is coherent");
        let mut indented = Vec::new();
        crate::output::emit_to(&mut indented, &document, crate::output::Form::Indented)
            .expect("a buffer accepts every write");
        let context = scratch.file(
            "indented.json",
            &String::from_utf8(indented).expect("the encoder emits UTF-8"),
        );

        let (result, written) = rendered(&tpl_dir, &context, "name", &object(&[], &[], &[]), &[]);

        result.expect("the render succeeds");
        assert_eq!(written, "freight");
    }

    #[test]
    fn fr_rnd_020_a_document_that_is_not_the_contract_is_65() {
        // FR-RND-020, in both halves: bytes that are not JSON, and JSON that
        // is not the document contract.
        let scratch = Scratch::new();
        let tpl_dir = project(&scratch, &[("name.jinja", "{{ database.name }}")]);

        for (file, bytes) in [("broken.json", "{ this is not json"), ("bare.json", "{}")] {
            let context = scratch.file(file, bytes);
            let (result, written) =
                rendered(&tpl_dir, &context, "name", &object(&[], &[], &[]), &[]);
            let condition = result.expect_err("the document is refused");

            assert_eq!(condition.exit_code(), 65, "{file}");
            assert!(
                matches!(condition, Error::ContextDocumentMalformed { .. }),
                "{file}"
            );
            assert!(written.is_empty(), "{file} wrote {written:?}");
        }
    }

    #[test]
    fn fr_ctx_042_a_context_document_with_a_dangling_reference_is_65_and_not_70() {
        // FR-CTX-042, FR-RND-020, FR-ERR-029, finding SEC-02: one table kept
        // from a whole dump, every table its keys name dropped. The render is
        // refused as a malformed document, never as a defect of tpl.
        let scratch = Scratch::new();
        let tpl_dir = project(&scratch, &[("name.jinja", "{{ database.name }}")]);
        let mut document: serde_json::Value =
            serde_json::from_str(&dumped()).expect("the dump is JSON");
        let tables = document["data"]["database"]["tables"]
            .as_array_mut()
            .expect("tables is an array");
        let kept = tables
            .iter()
            .find(|table| {
                table["foreign_keys"]
                    .as_array()
                    .is_some_and(|keys| keys.iter().any(|key| key["referenced_table"].is_object()))
            })
            .cloned()
            .expect("the fixture carries a table with a key that names a table");
        *tables = vec![kept];
        let context = scratch.file("dangling.json", &document.to_string());

        let (result, written) = rendered(&tpl_dir, &context, "name", &object(&[], &[], &[]), &[]);
        let condition = result.expect_err("the document is refused");

        assert_eq!(condition.exit_code(), 65);
        assert!(
            matches!(
                condition,
                Error::ContextDocumentMalformed {
                    fault: crate::error::ContextFault::DanglingReference { .. },
                    ..
                }
            ),
            "{condition:?}"
        );
        assert!(written.is_empty(), "wrote {written:?}");
    }

    /// Writes `.tpl/.cfg` into the project `tpl_dir`, with the mode
    /// `FR-SEC-004` requires.
    fn configured(scratch: &Scratch, text: &str) {
        let file = scratch.file("project/.tpl/.cfg", text);
        scratch.chmod(&file, 0o600);
    }

    /// A template writing 1000 bytes.
    const THOUSAND_BYTES: &str = "{% for i in range(100) %}0123456789{% endfor %}";

    #[test]
    fn fr_rnd_036_a_render_that_exhausts_the_fuel_the_cfg_declares_is_65_and_writes_nothing() {
        // FR-RND-036, FR-CONF-045: `core.render_fuel` from `.tpl/.cfg`.
        let scratch = Scratch::new();
        let tpl_dir = project(
            &scratch,
            &[(
                "loop.jinja",
                "{% for i in range(10000) %}{% for j in range(10000) %}{% endfor %}{% endfor %}",
            )],
        );
        configured(&scratch, "[core]\nrender_fuel = 5000\n");
        let context = scratch.file("dump.json", &dumped());

        let (result, written) = rendered(&tpl_dir, &context, "loop", &object(&[], &[], &[]), &[]);
        let condition = result.expect_err("the loop outruns its fuel");

        assert!(
            matches!(condition, Error::RenderFuelExhausted { fuel: 5000 }),
            "{condition:?}"
        );
        assert_eq!(condition.exit_code(), 65);
        assert!(written.is_empty(), "wrote {} bytes", written.len());
    }

    #[test]
    fn fr_rnd_037_a_render_past_the_limit_the_cfg_declares_is_65_and_stdout_holds_at_most_it() {
        // FR-RND-037, FR-CONF-045: stdout receives no byte beyond the limit —
        // here none at all, which FR-RND-034 admits.
        let scratch = Scratch::new();
        let tpl_dir = project(&scratch, &[("wide.jinja", THOUSAND_BYTES)]);
        configured(&scratch, "[core]\nrender_output_limit = 64\n");
        let context = scratch.file("dump.json", &dumped());

        let (result, written) = rendered(&tpl_dir, &context, "wide", &object(&[], &[], &[]), &[]);
        let condition = result.expect_err("a thousand bytes pass a limit of 64");

        assert!(
            matches!(condition, Error::RenderOutputLimitExceeded { limit: 64 }),
            "{condition:?}"
        );
        assert_eq!(condition.exit_code(), 65);
        assert!(written.len() <= 64, "wrote {} bytes", written.len());
    }

    #[test]
    fn fr_conf_045_raising_the_limit_in_the_cfg_lets_the_same_render_pass() {
        let scratch = Scratch::new();
        let tpl_dir = project(&scratch, &[("wide.jinja", THOUSAND_BYTES)]);
        configured(&scratch, "[core]\nrender_output_limit = 1000\n");
        let context = scratch.file("dump.json", &dumped());

        let (result, written) = rendered(&tpl_dir, &context, "wide", &object(&[], &[], &[]), &[]);

        result.expect("a thousand bytes fit a limit of 1000");
        assert_eq!(written.len(), 1000);
    }

    #[test]
    fn fr_conf_045_a_render_bound_outside_its_range_in_the_cfg_is_78() {
        let scratch = Scratch::new();
        let tpl_dir = project(&scratch, &[("wide.jinja", THOUSAND_BYTES)]);
        configured(&scratch, "[core]\nrender_fuel = 0\n");
        let context = scratch.file("dump.json", &dumped());

        let (result, written) = rendered(&tpl_dir, &context, "wide", &object(&[], &[], &[]), &[]);
        let condition = result.expect_err("0 is outside the range");

        assert!(
            matches!(condition, Error::ConfigurationValueMalformed { .. }),
            "{condition:?}"
        );
        assert_eq!(condition.exit_code(), 78);
        assert!(written.is_empty());
    }

    #[test]
    fn fr_rnd_029_a_template_that_does_not_exist_is_66_with_a_suggestion() {
        // FR-RND-029 through FR-TMPL-027, and the order of FR-ERR-006: step 4
        // is reached because steps 2 and 3 passed, and the condition is the
        // missing template rather than anything the document or the render
        // would have raised. The document supplied here is well formed and
        // carries the whole contract, so nothing downstream could have
        // produced this refusal.
        let scratch = Scratch::new();
        let tpl_dir = project(&scratch, &[("example.jinja", "hello\n")]);
        let context = scratch.file("context.json", &dumped());

        let (result, written) = rendered(&tpl_dir, &context, "exmple", &object(&[], &[], &[]), &[]);
        let condition = result.expect_err("no such template");

        assert_eq!(condition.exit_code(), 66);
        assert!(
            matches!(&condition, Error::TemplateNotFound { nearest, .. } if nearest == &["example"]),
            "{condition}"
        );
        assert!(written.is_empty(), "{written:?}");
    }

    #[test]
    fn fr_rnd_032_an_object_absent_from_the_document_is_66_with_a_suggestion() {
        // FR-RND-032 over the document source, and the `66` row of FR-ERR-034:
        // the population named is the document and the database it describes,
        // because no INFORMATION_SCHEMA was read and no entry was resolved.
        let scratch = Scratch::new();
        let tpl_dir = project(&scratch, &[("name.jinja", "{{ table.name }}")]);
        let context = scratch.file("context.json", &dumped());

        let (result, written) = rendered(
            &tpl_dir,
            &context,
            "name",
            &object(&["consignmen"], &[], &[]),
            &[],
        );
        let condition = result.expect_err("no such table");

        assert_eq!(condition.exit_code(), 66);
        assert!(
            matches!(
                &condition,
                Error::ContextObjectNotFound { name, nearest, .. }
                    if name == "consignmen" && nearest.contains(&"consignment".to_owned())
            ),
            "{condition}"
        );
        assert!(written.is_empty(), "{written:?}");
    }

    #[test]
    fn fr_rnd_032_a_bare_routine_name_reaching_both_namespaces_is_64_in_a_document_too() {
        // FR-RND-032's second sentence: FR-SCH-010 "applies under any
        // circumstance", and a document is one of them. The fixture carries no
        // such pair, so one is built.
        let scratch = Scratch::new();
        let tpl_dir = project(&scratch, &[("name.jinja", "{{ routine.name }}")]);
        let model = document::fixture::database();
        let twinned = {
            let mut routines: Vec<Routine<'_>> = model.routines.clone();
            let mut second = routines
                .iter()
                .find(|routine| routine.kind == RoutineKind::Procedure)
                .expect("the fixture carries a procedure")
                .clone();
            second.kind = RoutineKind::Function;
            routines.push(second);

            crate::model::database::Database { routines, ..model }
        };
        let document = document::dump(&twinned, crate::output::Source::Server)
            .expect("the fixture is coherent");
        let mut bytes = Vec::new();
        crate::output::emit_to(&mut bytes, &document, crate::output::Form::Compact)
            .expect("a buffer accepts every write");
        let name = twinned
            .routines
            .first()
            .expect("the fixture carries a routine")
            .name
            .to_string();
        let context = scratch.file(
            "twinned.json",
            &String::from_utf8(bytes).expect("the encoder emits UTF-8"),
        );

        let (result, written) =
            rendered(&tpl_dir, &context, "name", &object(&[], &[], &[&name]), &[]);
        let condition = result.expect_err("the bare name reaches both namespaces");

        assert_eq!(condition.exit_code(), 64);
        assert!(
            matches!(condition, Error::AmbiguousRoutineInContext { .. }),
            "a document is not a catalogue"
        );
        assert!(written.is_empty(), "{written:?}");
    }

    #[test]
    fn fr_rnd_034_a_render_that_fails_leaves_stdout_empty() {
        // FR-RND-034 admits at most one incomplete result, and what this
        // command leaves is none: the text is produced whole or not at all,
        // and nothing is written until it has been. FR-RND-030 and FR-RND-031
        // are the two ways it fails.
        let scratch = Scratch::new();
        let tpl_dir = project(
            &scratch,
            &[
                ("broken.jinja", "kept\n{% if %}\n"),
                ("absent.jinja", "kept\n{{ database.missing }}\n"),
                ("stopped.jinja", "kept\n{{ fail('no mapping') }}\n"),
            ],
        );
        let context = scratch.file("context.json", &dumped());

        for template in ["broken", "absent", "stopped"] {
            let (result, written) =
                rendered(&tpl_dir, &context, template, &object(&[], &[], &[]), &[]);
            let condition = result.expect_err("the render fails");

            assert_eq!(condition.exit_code(), 65, "{template}");
            assert!(written.is_empty(), "{template} wrote {written:?}");
        }
    }

    #[test]
    fn fr_rnd_033_a_deadline_already_spent_is_65_and_starts_no_render() {
        // FR-RND-033 and FR-GLOB-012: the overall budget expiring before the
        // phase begins ends it before it starts, and nothing of the render
        // runs. The process-terminating half is outside a process's own test
        // and is verified from outside it.
        let spent = Bound::spent(Seconds::new(NonZeroU64::new(30).expect("30 is positive")));
        let mut ran = false;

        let condition = bounded(spent, RenderMemoryLimit::DEFAULT, || {
            ran = true;
            Ok::<(), Error>(())
        })
        .expect_err("the bound is spent");

        assert_eq!(condition.exit_code(), 65);
        assert!(matches!(condition, Error::RenderDeadlineExceeded { .. }));
        assert!(!ran, "a spent bound starts no render");
    }

    #[test]
    fn a_render_inside_its_deadline_returns_what_it_produced() {
        // The other half of the same function, and the property that keeps the
        // timer from outliving the render: the call returns.
        let clock = crate::project::settings::clock(None);
        let bound = clock.bound(Seconds::new(NonZeroU64::new(30).expect("30 is positive")));

        assert_eq!(
            bounded(bound, RenderMemoryLimit::DEFAULT, || {
                Ok::<&str, Error>("produced")
            })
            .expect("it is inside the deadline"),
            "produced"
        );
    }

    #[test]
    fn fr_cache_039_a_render_abandoned_on_a_miss_reads_no_further_file() {
        // Once a member is a miss, every member reached after it fails without
        // its file being read, so the abandoned render ends at the next one.
        // `carrier` is damaged and `consignment_leg` intact, so the second
        // member reads as a table unless the miss stops its read.
        let scratch = Scratch::new();
        let tpl_dir = cached(
            &scratch,
            &[(
                "after.jinja",
                "{{ database.tables[0].name }}{{ database.tables[2].name }}",
            )],
        );
        std::fs::write(stored(&tpl_dir, "tables/carrier.json"), "{ torn").expect("ours");
        let listed = crate::cache::Cache::of(&tpl_dir, ENTRY)
            .shelved()
            .expect("the store is whole");
        let store = super::Store::open(listed, Ending::Caller).expect("database.json decodes");

        assert!(store.table(0).is_none(), "the damaged file is a miss");
        assert!(
            !store.missed(),
            "the flag is the value's, set where it is read"
        );

        let value = super::context::assemble_shelved(
            &std::sync::Arc::new(store),
            None,
            &std::collections::BTreeMap::new(),
            "1970-01-01T00:00:00Z",
        );
        let tables = value
            .get_attr("database")
            .and_then(|database| database.get_attr("tables"))
            .expect("the collection is readable");

        assert_eq!(
            tables
                .get_item_by_index(0)
                .expect("an index answers")
                .kind(),
            minijinja::value::ValueKind::Invalid
        );
        assert_eq!(
            tables
                .get_item_by_index(2)
                .expect("an index answers")
                .kind(),
            minijinja::value::ValueKind::Invalid,
            "a member reached after the miss is not read"
        );
    }

    #[test]
    fn fr_cache_039_an_abandoned_render_keeps_evaluating_after_a_miss_a_lookup_function_reached() {
        // What an abandoned render can still do. A member reached through the
        // collection reads as an invalid value, and any use of it unwinds the
        // render. A member reached through one of the four lookup functions of
        // FR-ENV-020 does not: the function's answer is tolerated by
        // assignment, `is defined`, `is none` and `default`, so the render
        // carries on after the miss — here doubling a string to 1 MiB and
        // looping — and returns what it produced. The store records the miss,
        // so the render is still abandoned; what it does between the miss and
        // its return is bounded only by what bounds a render while it runs.
        let after = "{% set ns = namespace(s='x', n=0) %}\
                     {% for i in range(20) %}{% set ns.s = ns.s ~ ns.s %}{% endfor %}\
                     {% for i in range(1000) %}{% set ns.n = ns.n + 1 %}{% endfor %}\
                     AFTER {{ ns.s | length }} {{ ns.n }}";

        for reach in [
            "{% set t = table('carrier') %}",
            "{% if table('carrier') is defined %}{% endif %}",
            "{{ table('carrier') is none }}",
            "{{ column('carrier', 'carrier_id') is defined }}",
        ] {
            let scratch = Scratch::new();
            let source = format!("{reach}{after}");
            let tpl_dir = cached(&scratch, &[("escape.jinja", source.as_str())]);
            std::fs::write(stored(&tpl_dir, "tables/carrier.json"), "{ torn").expect("ours");
            let listed = crate::cache::Cache::of(&tpl_dir, ENTRY)
                .shelved()
                .expect("the store is whole");
            let store = std::sync::Arc::new(
                super::Store::open(listed, Ending::Caller).expect("database.json decodes"),
            );
            let value = super::context::assemble_shelved(
                &store,
                None,
                &std::collections::BTreeMap::new(),
                "1970-01-01T00:00:00Z",
            );

            let produced = crate::render::Environment::new(&tpl_dir)
                .render("escape", &value)
                .unwrap_or_else(|failure| panic!("{reach}: the render unwound: {failure:?}"));

            assert!(
                store.missed(),
                "{reach}: the lookup reached the damaged file"
            );
            assert!(
                produced.ends_with("AFTER 1048576 1000"),
                "{reach}: {produced:?}"
            );
        }
    }

    /// A cached project whose `.tpl/.cfg` is [`UNREACHABLE`] with `core` added
    /// to its `[core]` section, carrying `templates`, with `carrier`'s file
    /// damaged so that reaching it is a miss under `FR-CACHE-039`.
    ///
    /// The entry names a port nothing listens on, so a server read the
    /// invocation made would end with the `69` of a refused connection: a
    /// `65` naming a render bound is therefore also the proof that no
    /// connection was attempted.
    fn abandoning(scratch: &Scratch, core: &str, templates: &[(&str, &str)]) -> PathBuf {
        let tpl_dir = cached(scratch, templates);
        let file = scratch.file(
            "project/.tpl/.cfg",
            &UNREACHABLE.replacen("[core]\n", &format!("[core]\n{core}"), 1),
        );
        scratch.chmod(&file, 0o600);
        std::fs::write(stored(&tpl_dir, "tables/carrier.json"), "{ torn").expect("ours");

        tpl_dir
    }

    /// The reach of `carrier` through a lookup function, which the render
    /// survives, per the test above.
    const REACH: &str = "{% set t = table('carrier') %}";

    #[test]
    fn fr_cache_039_an_abandoned_render_that_exhausts_its_fuel_ends_the_invocation_with_65() {
        // FR-CACHE-039, second paragraph, and FR-RND-038: fuel crossed after
        // the miss is the invocation's 65, and the server is not read.
        let scratch = Scratch::new();
        let source = format!(
            "{REACH}{{% for i in range(10000) %}}{{% for j in range(10000) %}}{{% endfor %}}{{% endfor %}}"
        );
        let tpl_dir = abandoning(&scratch, "render_fuel = 5000\n", &[("loop.jinja", &source)]);

        let (result, written) = from_store(&tpl_dir, "loop", &object(&[], &[], &[]));
        let condition = result.expect_err("the abandoned render outruns its fuel");

        assert!(
            matches!(condition, Error::RenderFuelExhausted { fuel: 5000 }),
            "{condition:?}"
        );
        assert!(written.is_empty(), "wrote {written:?}");
    }

    #[test]
    fn fr_cache_039_an_abandoned_render_past_its_output_limit_ends_the_invocation_with_65() {
        let scratch = Scratch::new();
        let source = format!("{REACH}{{% for i in range(100) %}}0123456789{{% endfor %}}");
        let tpl_dir = abandoning(
            &scratch,
            "render_output_limit = 64\n",
            &[("wide.jinja", &source)],
        );

        let (result, written) = from_store(&tpl_dir, "wide", &object(&[], &[], &[]));
        let condition = result.expect_err("the abandoned render passes its output limit");

        assert!(
            matches!(condition, Error::RenderOutputLimitExceeded { limit: 64 }),
            "{condition:?}"
        );
        assert!(written.is_empty(), "wrote {written:?}");
    }

    #[test]
    fn fr_cache_039_an_abandoned_render_within_every_bound_still_reads_the_server() {
        // The control: the same miss, no bound crossed, and the invocation is
        // a miss like any other — it reads the server, which here refuses the
        // connection, so the condition is the 69 of that read and not a 65.
        let scratch = Scratch::new();
        let source = format!("{REACH}within");
        let tpl_dir = abandoning(&scratch, "", &[("calm.jinja", &source)]);

        let (result, written) = from_store(&tpl_dir, "calm", &object(&[], &[], &[]));
        let condition = result.expect_err("the unreachable server refuses the read");

        assert_eq!(condition.exit_code(), 69, "{condition:?}");
        assert!(written.is_empty(), "wrote {written:?}");
    }

    #[test]
    fn the_reported_position_of_a_byte_that_is_not_utf_8_is_one_based() {
        // The `65` row of FR-ERR-034 obliges a position beside the path, and a
        // position a caller can act on counts from one in both axes.
        assert_eq!(position(b"").line, 1);
        assert_eq!(position(b"").column, 1);
        assert_eq!(position(b"{\n  \"a\"").line, 2);
        assert_eq!(position(b"{\n  \"a\"").column, 6);
    }

    /// The entry every catalogue render below reads through.
    const ENTRY: &str = "shop";

    /// A `.tpl/.cfg` whose one entry reaches a port nothing listens on, so a
    /// render that opened a connection would be the `69` of `FR-ERR-034` and
    /// never a render.
    const UNREACHABLE: &str = "[core]\ndatabase = \"shop\"\n\n[database.shop]\n\
         host = \"127.0.0.1\"\nport = 1\nuser = \"reader\"\npassword = \"secret\"\n\
         database = \"freight\"\ntls = \"disabled\"\n";

    /// The template the equivalence below renders over both sources: the whole
    /// `database` as `json`, every collection in order with its length, and
    /// the two tests that resolve a column's table against the context.
    const WIDE: &str = "{{ database | json }}\n{{ database.tables | length }} {{ database.views | length }} \
         {{ database.routines | length }}\n{% for t in database.tables %}{{ t.name }}:\
         {% for c in t.columns %}{{ c.name }}={{ c is primary_key }}/{{ c is unique }} {% endfor %}\
         {% endfor %}\n{% for r in database.routines %}{{ r.kind }}.{{ r.name }} {% endfor %}\n\
         {{ table.name }} {{ table | json }}";

    /// A project reaching [`UNREACHABLE`], carrying `templates`, with the store
    /// of its entry written from [`document::fixture::whole`] — every
    /// collection recorded whole.
    fn cached(scratch: &Scratch, templates: &[(&str, &str)]) -> PathBuf {
        let tpl_dir = project(scratch, templates);
        let file = scratch.file("project/.tpl/.cfg", UNREACHABLE);
        scratch.chmod(&file, 0o600);

        let model = document::fixture::whole();
        let built = document::context(&model).expect("the fixture is coherent");
        crate::cache::Cache::of(&tpl_dir, ENTRY).write(&built, crate::cache::Covered::Everything);

        tpl_dir
    }

    /// The object file of one member of the store [`cached`] wrote.
    fn stored(tpl_dir: &Path, relative: &str) -> PathBuf {
        tpl_dir.join(".cache").join(ENTRY).join(relative)
    }

    /// What `tpl render` does over the selected database, and what it wrote.
    fn from_store(tpl_dir: &Path, template: &str, object: &Object) -> (Result<(), Error>, String) {
        let caching = Caching {
            direct: false,
            no_cache: false,
        };
        let supplied = Supplied {
            template,
            object,
            set: &[],
            context: &[],
            caching: &caching,
            ending: Ending::Caller,
        };
        let mut written = Vec::new();
        let result = run(&mut written, &globals(Some(tpl_dir), None), &supplied);

        (
            result,
            String::from_utf8(written).expect("a template of these tests writes UTF-8"),
        )
    }

    #[test]
    fn fr_cache_038_a_render_from_the_store_is_the_render_of_an_up_front_read() {
        // FR-CACHE-038 end to end: the lazily served `database` renders the
        // bytes the pipeline before the fortieth edition rendered — every file
        // decoded up front by `Cache::everything`, bound and assembled whole —
        // for the whole of it as `json`, each collection's order and length,
        // and the tests that reach back into the context. No connection is
        // made, which the unreachable entry would have refused.
        let scratch = Scratch::new();
        let tpl_dir = cached(&scratch, &[("wide.jinja", WIDE)]);
        let loaded = crate::cache::Cache::of(&tpl_dir, ENTRY)
            .everything()
            .expect("the store is whole");
        let up_front = loaded.document().expect("every file decodes");
        let environment = crate::render::Environment::new(&tpl_dir);
        let defined = std::collections::BTreeMap::new();

        for table in ["carrier", "consignment", "consignment_leg"] {
            let bound = object(&[table], &[], &[]);
            let binding = Binding::of(&bound).expect("one kind");
            let sought = Sought::Catalogue {
                entry: ENTRY,
                database: &up_front.name,
            };
            let context = super::context::assemble(
                Served::borrowed(&up_front),
                binding
                    .bind(&up_front, sought)
                    .expect("the table is stored"),
                &defined,
                "1970-01-01T00:00:00Z",
            );
            let expected = environment
                .render("wide", &context)
                .expect("the up-front read renders");

            let (lazily, served) = from_store(&tpl_dir, "wide", &bound);

            lazily.expect("the store serves the render");
            assert_eq!(served, expected, "{table}");
        }
    }

    #[test]
    fn fr_cache_033_a_damaged_file_the_render_never_reaches_is_not_a_miss() {
        // FR-CACHE-033 as the fortieth edition amended it: the render reads
        // the bound table and never reaches `carrier`, so its damaged file is
        // not a miss — no connection is attempted, which would be `69` — and
        // it is left as it was.
        let scratch = Scratch::new();
        let tpl_dir = cached(&scratch, &[("bound.jinja", "{{ table.name }}")]);
        let damaged = stored(&tpl_dir, "tables/carrier.json");
        std::fs::write(&damaged, "{ torn").expect("the store is ours");

        let (result, written) = from_store(&tpl_dir, "bound", &object(&["consignment"], &[], &[]));

        result.expect("the damaged file is never reached");
        assert_eq!(written, "consignment");
        assert_eq!(
            std::fs::read_to_string(&damaged).expect("the store is ours"),
            "{ torn"
        );
    }

    #[test]
    fn fr_cache_039_a_miss_reached_during_the_render_is_answered_as_one_found_before_it() {
        // FR-CACHE-039: a render that reaches a damaged file is abandoned and
        // the invocation is a miss, answered as today's up-front miss is — one
        // server read, which here is refused, so both invocations are the same
        // `69` and neither wrote a byte of the abandoned render. The template
        // writes before it reaches the member, so a byte that escaped would be
        // seen.
        let template = "written first {{ database.tables[0].name }}";
        let bound = object(&["consignment"], &[], &[]);

        let scratch = Scratch::new();
        let tpl_dir = cached(&scratch, &[("reach.jinja", template)]);
        std::fs::write(stored(&tpl_dir, "tables/carrier.json"), "{ torn").expect("ours");
        let (during, during_written) = from_store(&tpl_dir, "reach", &bound);

        let scratch = Scratch::new();
        let tpl_dir = cached(&scratch, &[("reach.jinja", template)]);
        std::fs::write(stored(&tpl_dir, "tables/consignment.json"), "{ torn").expect("ours");
        let (before, before_written) = from_store(&tpl_dir, "reach", &bound);

        let during = during.expect_err("the server is unreachable");
        let before = before.expect_err("the server is unreachable");

        assert_eq!(during.exit_code(), 69, "{during}");
        assert_eq!(during.exit_code(), before.exit_code());
        assert_eq!(during.to_string(), before.to_string());
        assert!(during_written.is_empty(), "{during_written:?}");
        assert!(before_written.is_empty(), "{before_written:?}");
    }

    #[test]
    fn fr_cache_039_a_condition_raised_before_the_miss_is_reached_is_the_invocations() {
        // FR-CACHE-039's consequence and FR-ERR-006: a render that fails
        // before it reaches the damaged file reports its own `65` and opens no
        // connection; the same failure after the miss is not reached, because
        // the render was abandoned at the miss.
        let scratch = Scratch::new();
        let tpl_dir = cached(
            &scratch,
            &[
                (
                    "first.jinja",
                    "{{ fail('the template stops') }}{{ database.tables[0].name }}",
                ),
                (
                    "second.jinja",
                    "{{ database.tables[0].name }}{{ fail('the template stops') }}",
                ),
            ],
        );
        std::fs::write(stored(&tpl_dir, "tables/carrier.json"), "{ torn").expect("ours");
        let bound = object(&["consignment"], &[], &[]);

        let (first, written) = from_store(&tpl_dir, "first", &bound);
        let first = first.expect_err("the template stops");
        assert_eq!(first.exit_code(), 65, "{first}");
        assert!(written.is_empty(), "{written:?}");

        let (second, written) = from_store(&tpl_dir, "second", &bound);
        let second = second.expect_err("the server is unreachable");
        assert_eq!(second.exit_code(), 69, "{second}");
        assert!(written.is_empty(), "{written:?}");
    }

    #[test]
    fn fr_cache_038_an_object_is_bound_from_the_listing_as_from_the_whole_store() {
        // Step 7 over the listing: an absent name is the `66` with the
        // nearest match, a bare routine name reaching both kinds is the `64`,
        // and the qualified form binds one — with no connection, and the same
        // condition the whole store would raise.
        let scratch = Scratch::new();
        let tpl_dir = cached(&scratch, &[("kind.jinja", "{{ routine.kind }}")]);

        let (absent, _) = from_store(&tpl_dir, "kind", &object(&["consignmen"], &[], &[]));
        let absent = absent.expect_err("no such table");
        assert_eq!(absent.exit_code(), 66);
        assert!(
            matches!(
                &absent,
                Error::CatalogueObjectNotFound { nearest, .. }
                    if nearest.contains(&"consignment".to_owned())
            ),
            "{absent}"
        );

        let (bare, _) = from_store(
            &tpl_dir,
            "kind",
            &object(&[], &[], &["sp_book_consignment"]),
        );
        let bare = bare.expect_err("the bare name reaches both kinds");
        assert_eq!(bare.exit_code(), 64);
        assert!(matches!(bare, Error::AmbiguousRoutineName { .. }), "{bare}");

        let (qualified, written) = from_store(
            &tpl_dir,
            "kind",
            &object(&[], &[], &["function:sp_book_consignment"]),
        );
        qualified.expect("the qualified name binds one");
        assert_eq!(written, "FUNCTION");
    }

    /// Removes the object files of the store [`cached`] wrote, but `kept`.
    fn keep_only(tpl_dir: &Path, kept: &[&str]) {
        for collection in ["tables", "views", "routines"] {
            let directory = stored(tpl_dir, collection);
            for entry in std::fs::read_dir(&directory)
                .expect("the store is ours")
                .flatten()
            {
                let name = entry.file_name().to_string_lossy().into_owned();
                if !kept.contains(&format!("{collection}/{name}").as_str()) {
                    std::fs::remove_file(entry.path()).expect("the store is ours");
                }
            }
        }
    }

    #[test]
    fn fr_cache_038_a_lookup_by_name_opens_only_the_file_it_returns() {
        // FR-CACHE-038 as the forty-first edition amended it, for each of the
        // six lookups: every object file but the one returned is removed, so a
        // lookup that opened any other would find a miss, and the miss would
        // reach the unreachable entry as `69`. Each name sorts after members
        // the lookup must pass over.
        let whole = object(&[], &[], &[]);
        let bound = object(&["consignment_leg"], &[], &[]);
        for (template, source, object, kept, expected) in [
            (
                "table",
                "{{ table('consignment_leg').name }}",
                &whole,
                &["tables/consignment_leg.json"][..],
                "consignment_leg",
            ),
            (
                "column",
                "{{ column('consignment_leg', 'leg_id').name }}",
                &whole,
                &["tables/consignment_leg.json"][..],
                "leg_id",
            ),
            (
                "view",
                "{{ view('v_consignment_manifest').name }}",
                &whole,
                &["views/v_consignment_manifest.json"][..],
                "v_consignment_manifest",
            ),
            (
                "tests",
                "{% for c in table.columns %}{{ c.name }}={{ c is primary_key }}/{{ c is unique }} {% endfor %}",
                &bound,
                &["tables/consignment_leg.json"][..],
                "leg_id=true/true consignment_id=false/false ",
            ),
        ] {
            let scratch = Scratch::new();
            let tpl_dir = cached(&scratch, &[(&format!("{template}.jinja"), source)]);
            keep_only(&tpl_dir, kept);

            let (result, written) = from_store(&tpl_dir, template, object);

            result.unwrap_or_else(|condition| panic!("{template}: {condition}"));
            assert_eq!(written, expected, "{template}");
        }

        // `routine`: the first routine of the name in the listing's order is
        // the one returned, so the other of the two is removed.
        let scratch = Scratch::new();
        let tpl_dir = cached(
            &scratch,
            &[("routine.jinja", "{{ routine('sp_book_consignment').kind }}")],
        );
        let listed = crate::cache::Cache::of(&tpl_dir, ENTRY)
            .shelved()
            .expect("the store is whole");
        let first =
            crate::cache::paths::lower(listed.routines()[0].kind().expect("a routine has a kind"))
                .expect("a recorded kind");
        keep_only(
            &tpl_dir,
            &[&format!("routines/{first}.sp_book_consignment.json")],
        );

        let (result, written) = from_store(&tpl_dir, "routine", &whole);

        result.expect("only the returned routine is read");
        assert_eq!(written, first.to_uppercase());
    }

    #[test]
    fn fr_cache_038_a_procedure_and_a_function_of_one_name_resolve_as_the_scan_did() {
        // The tie of NFR-DET-002 is kept as it was: `routine(name)` returns the
        // routine a scan of the up-front read's collection meets first.
        let scratch = Scratch::new();
        let tpl_dir = cached(
            &scratch,
            &[("routine.jinja", "{{ routine('sp_book_consignment').kind }}")],
        );
        let loaded = crate::cache::Cache::of(&tpl_dir, ENTRY)
            .everything()
            .expect("the store is whole");
        let up_front = loaded.document().expect("every file decodes");

        let (result, written) = from_store(&tpl_dir, "routine", &object(&[], &[], &[]));

        result.expect("the routine is stored");
        assert_eq!(written, up_front.routines[0].kind.name());
    }

    #[test]
    fn fr_cache_033_a_damaged_file_a_lookup_passes_over_is_not_a_miss() {
        // FR-CACHE-033 as amended: `carrier` and `consignment` sort before the
        // table sought, and neither is consulted, so neither is a miss.
        let scratch = Scratch::new();
        let tpl_dir = cached(
            &scratch,
            &[("found.jinja", "{{ table('consignment_leg').name }}")],
        );
        for damaged in ["tables/carrier.json", "tables/consignment.json"] {
            std::fs::write(stored(&tpl_dir, damaged), "{ torn").expect("ours");
        }

        let (result, written) = from_store(&tpl_dir, "found", &object(&[], &[], &[]));

        result.expect("the damaged files are passed over");
        assert_eq!(written, "consignment_leg");
    }

    #[test]
    fn fr_cache_039_a_damaged_file_a_lookup_returns_restarts_the_render() {
        // FR-CACHE-039: the file returned is reached, so a damaged one is a
        // miss and the render is abandoned; the one server read is refused
        // here, so the invocation is `69`, with nothing on stdout.
        let scratch = Scratch::new();
        let tpl_dir = cached(
            &scratch,
            &[("found.jinja", "written first {{ table('carrier').name }}")],
        );
        std::fs::write(stored(&tpl_dir, "tables/carrier.json"), "{ torn").expect("ours");

        let (result, written) = from_store(&tpl_dir, "found", &object(&[], &[], &[]));
        let condition = result.expect_err("the server is unreachable");

        assert_eq!(condition.exit_code(), 69, "{condition}");
        assert!(written.is_empty(), "{written:?}");
    }

    #[test]
    fn fr_cache_038_a_lookup_that_finds_nothing_opens_no_object_file() {
        // No object file is left, so any file a lookup opened would be a miss
        // and a `69`. What each lookup answers is today's: `undefined`, which
        // the guard sees as absent and whose use is the `65` of FR-SEM-012.
        let whole = object(&[], &[], &[]);
        let scratch = Scratch::new();
        let tpl_dir = cached(
            &scratch,
            &[
                (
                    "guarded.jinja",
                    "{{ table('nosuch') is defined }} {{ view('nosuch') is defined }} \
                     {{ routine('nosuch') is defined }} {{ column('nosuch', 'id') is defined }} \
                     {{ column('carrier', 'nosuch') is defined }}",
                ),
                ("used.jinja", "{{ table('nosuch').name }}"),
            ],
        );
        keep_only(&tpl_dir, &["tables/carrier.json"]);

        let (result, written) = from_store(&tpl_dir, "guarded", &whole);
        result.expect("no lookup opened a file it did not return");
        assert_eq!(written, "false false false false false");

        keep_only(&tpl_dir, &[]);
        let (result, written) = from_store(&tpl_dir, "used", &whole);
        let condition = result.expect_err("an undefined value is used");
        assert_eq!(condition.exit_code(), 65, "{condition}");
        assert!(written.is_empty(), "{written:?}");
    }
}
