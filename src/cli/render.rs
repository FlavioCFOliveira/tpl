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
//! | The selected database | [`super::source::Reader`], cache first | `FR-RND-026`, `FR-CACHE-006` |
//! | A `--context` document | The file, or stdin for `-`, read back through `crate::model::document` | `FR-RND-016`, `FR-RND-017`, `FR-RND-022` |
//!
//! **The result must not betray which.** That is not a rule this module has to
//! remember: both paths produce a
//! [`DatabaseDocument`](crate::model::document::DatabaseDocument) and hand it
//! to the same [`produce`], so the context a template sees is assembled once,
//! from one type, by [`context::assemble`]. What differs between the two is
//! carried where it is true and nowhere else — the population an object was
//! sought in, which `FR-ERR-034` obliges a `66` and a `64` to name, and which
//! [`named::Sought`] makes a parameter.
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
//! | 5, 6 | The entry, the cache, and the server on a miss — **skipped** on the `--context` path | `FR-RND-019`, `FR-RND-022`, `FR-RND-026` |
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
//! # A failed render leaves stdout empty
//!
//! `FR-RND-034` admits **at most one** incomplete result on stdout when a
//! render fails, and what this module leaves is none:
//! [`crate::render::Environment::render`] returns the whole text or the
//! condition, and nothing is written until it has returned the text. The one
//! path that can interrupt a render in progress is the deadline of
//! [`bounded`], which terminates the process through
//! [`std::process::exit`] — running no destructor, so a buffered stdout is
//! discarded rather than flushed, which is the outcome `FR-ERR-033` requires.

mod context;

use std::collections::BTreeMap;
use std::io::Read as _;
use std::path::{Path, PathBuf};
use std::sync::mpsc::{self, RecvTimeoutError};

use minijinja::Value;

use super::globals::Globals;
use super::local::{Caching, Object};
use super::schema::named::{self, Sought};
use super::source::{self, Reader};
use crate::cache::Look;
use crate::deadline::{Bound, Phase};
use crate::error::{ContextFault, Error, Position};
use crate::model::document::{self, DatabaseDocument};
use crate::output;
use crate::project::settings;
use crate::render::Environment;

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
}

/// Runs `tpl render` (`FR-RND-001` … `FR-RND-034`).
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
/// `FR-RND-032`; and the `65` of `FR-RND-030`, `FR-RND-031` and `FR-RND-033`.
pub(super) fn run<W: std::io::Write>(
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
/// The read is [`Look::Everything`], for the reason [`super::source`] gives and
/// one of this command's own: `FR-RND-023` binds `database` whatever object the
/// invocation named, so a template may walk every collection even when one
/// object is bound, and `FR-CTX-023` requires every object a carried reference
/// names to be present.
///
/// # Errors
///
/// Returns what [`source::project`], [`Environment::resolve`],
/// [`Reader::open_from`] and [`Reader::serve_from`] return, and what
/// [`produce`] returns.
fn from_catalogue<W: std::io::Write>(
    out: &mut W,
    globals: &Globals,
    supplied: &Supplied<'_>,
    binding: &Binding<'_>,
    defined: &BTreeMap<&str, &str>,
) -> Result<(), Error> {
    let reader = Reader::new(globals, Some(supplied.caching));
    // Steps 2 and 3, made once: the template root of FR-TMPL-023 and the
    // render deadline of FR-CONF-004 are both properties of the project this
    // read is made through, and a second walk could resolve a second project
    // between the two steps.
    let (project, configuration) = source::project(reader.tpl_dir())?;
    let environment = Environment::new(project.root());

    // Step 4, before an entry is resolved: FR-RND-029 through FR-TMPL-027. An
    // invocation refused here has spared itself the ${VAR} expansion of
    // FR-CONF-015, the password_command child of FR-CONF-024, the one
    // connection of NFR-PERF-004, the catalogue read and the store write.
    environment.resolve(supplied.template)?;

    // Steps 5 and 6.
    let opened = reader.open_from(&project, configuration)?;
    let assembly = Assembly {
        environment: &environment,
        template: supplied.template,
        binding,
        defined,
        deadline: reader.clock().bound(opened.deadlines().of(Phase::Render)),
    };

    reader.serve_from(&opened, &Look::Everything, |document, _, entry| {
        produce(
            out,
            &assembly,
            document,
            Sought::Catalogue {
                entry,
                database: &document.name,
            },
        )
    })
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
    let reader = Reader::new(globals, Some(supplied.caching));
    // Steps 2 and 3.
    let (project, configuration) = source::project(reader.tpl_dir())?;
    let environment = Environment::new(project.root());

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
    };

    let bytes = read_document(path)?;
    let model = document::read(&bytes, path)?;
    let built = document::context(&model)?;

    produce(
        out,
        &assembly,
        &built,
        Sought::Document {
            path,
            database: &built.name,
        },
    )
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
/// failure and of `FR-RND-033` for the deadline, and
/// [`Error::StdoutUnwritable`] where the stream refused the write.
fn produce<W: std::io::Write>(
    out: &mut W,
    assembly: &Assembly<'_>,
    document: &DatabaseDocument<'_>,
    at: Sought<'_>,
) -> Result<(), Error> {
    // 7 — FR-RND-032.
    let bound = assembly.binding.bind(document, at)?;

    // 8 — FR-RND-030, FR-RND-031, FR-RND-033. `now` is read here, which is
    // render time, and once, which is FR-CTX-029.
    let at = context::now();
    let context = context::assemble(document, bound, assembly.defined, &at);
    let produced = bounded(assembly.deadline, || {
        assembly.environment.render(assembly.template, &context)
    })?;

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
/// Returns [`Error::ProjectFileUnreadable`] — `74` — where the file or the
/// stream refused the read, and [`Error::ContextDocumentMalformed`] — `65` —
/// where the bytes are not UTF-8.
fn read_document(path: &Path) -> Result<String, Error> {
    let unreadable = |returned| Error::ProjectFileUnreadable {
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

/// Runs one render under the deadline of `FR-RND-033`, per `OD-12`.
///
/// The render runs on the calling thread and a timer thread waits on a channel
/// with `recv_timeout`; the render signals the channel when it completes. If
/// the deadline arrives first the timer writes the four labelled lines of
/// `FR-ERR-008` for the `65` and terminates the process with that status — which
/// is what `FR-GLOB-013` asks for, the phase in progress being the one the
/// timer was created for.
///
/// Three requirements make that admissible rather than merely convenient.
/// `FR-RND-034` already admits at most one incomplete result on stdout when a
/// render fails, so an interrupted render breaks no promise about stdout;
/// `NFR-DET-001` puts stderr outside the contract, so the timer writing to it
/// interleaves nothing that is contract; and [`std::process::exit`] runs no
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
/// # Errors
///
/// Returns [`Error::RenderDeadlineExceeded`] — `65` — where the bound was
/// already spent, and whatever `render` returns.
fn bounded<T>(deadline: Bound, render: impl FnOnce() -> Result<T, Error>) -> Result<T, Error> {
    let expired = Error::RenderDeadlineExceeded {
        bound: deadline.bound(),
        limit: deadline.limit(),
    };

    if deadline.expired() {
        return Err(expired);
    }

    let (finished, waiting) = mpsc::channel::<()>();
    let remaining = deadline.remaining();
    let timer = std::thread::spawn(move || {
        if matches!(
            waiting.recv_timeout(remaining),
            Err(RecvTimeoutError::Timeout)
        ) {
            let status = expired.exit_code();
            crate::diagnostics::report(&expired);

            std::process::exit(i32::from(status));
        }
    });

    let produced = render();

    // The phase is over whichever way it ended, so the timer is woken and
    // joined: nothing of this invocation outlives the invocation.
    drop(finished);
    let _ = timer.join();

    produced
}

#[cfg(test)]
mod tests {
    use super::{Binding, Supplied, bounded, document_flag, position, run};
    use crate::cli::globals::Globals;
    use crate::cli::local::{Caching, Object};
    use crate::deadline::{Bound, Seconds};
    use crate::error::Error;
    use crate::model::document;
    use crate::model::routine::{Routine, RoutineKind};
    use crate::project::scratch::Scratch;
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

        let condition = bounded(spent, || {
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
            bounded(bound, || Ok::<&str, Error>("produced")).expect("it is inside the deadline"),
            "produced"
        );
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
}
