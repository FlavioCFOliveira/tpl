//! `tpl` reads the structure of a MariaDB database and applies MiniJinja
//! templates to it.
//!
//! The crate is split in two halves, per `ADR-006`: this library holds the
//! logic, so that it is addressable from a test without launching a process,
//! and the binary parses the invocation, dispatches, and maps the resulting
//! error to an exit status.
//!
//! At this commit the **first arm reads**: the eight `schema` subcommands and
//! the three of `tpl cache` resolve a database entry, consult the store under
//! `.tpl/.cache/`, open the one connection they are allowed where the store
//! does not answer, read the catalogue into the model, and present it. [`run`]
//! is the entry point the binary calls, [`install_panic_hook`] is the process
//! setup it performs first, and [`Error`] is the value every module reports
//! failure through.
//!
//! | Module | What it owns |
//! |---|---|
//! | `cli` | The closed command tree of `FR-CLI-002`, the seven global flags of `FR-GLOB-001` every node accepts, the parsing rules of `FR-CLI-014` through `FR-CLI-020`, the six help and version forms of `FR-HELP-001`, and the `cfg` arm that maintains `.tpl/.cfg` |
//! | `project` | Where a project is found and why it is trusted (`FR-PROJ-001` … `FR-PROJ-011`), what `tpl init` creates (`FR-PROJ-012` … `FR-PROJ-024`), how `.tpl/.cfg` is read, validated and rewritten (`FR-CONF-001` … `FR-CONF-036`), and the settings a connection will need (`FR-CONF-004`, `FR-CONF-029`) |
//! | `deadline` | The clock every blocking phase is bounded by: the four `[core]` deadlines of `FR-CONF-005` and the overall budget of `FR-GLOB-011`, composed as `FR-GLOB-012` composes them |
//! | `diagnostics` | The four labelled lines of `FR-ERR-008` a failure reaches the caller as |
//! | `output` | The two formats a result reaches the caller through — the envelope of `FR-OUT-024` and the aligned columns of `FR-OUT-006` |
//! | `model` | The structure a database is read as: the covered object kinds of `FR-CAT-001`, `FR-CAT-007` and `FR-CAT-008` with the field lists of `FR-CAT-042` and `FR-CAT-045` … `FR-CAT-051`, the per-column decomposition of `FR-CTX-011` … `FR-CTX-018` and `FR-CTX-037` … `FR-CTX-041`, the `server` and `database` objects of `FR-CTX-031` … `FR-CTX-036`, the `restricted` marking of `FR-PRIV-016`, and the refusals of `FR-CAT-024` and `FR-CTX-021` |
//! | `model::document` | The one document that carries the model in both directions: the collection shape of `FR-CTX-003` … `FR-CTX-005`, the one-hop embedding of `FR-CTX-006` … `FR-CTX-010`, the orderings of `NFR-DET-002`, and the read-back `FR-CTX-033` admits |
//! | `mariadb` | The one connection of `NFR-PERF-004`, the TLS mode of `FR-CONF-037` and `ADR-002`, the read-only session of `FR-SRV-008` … `FR-SRV-011`, the version probe of `FR-SRV-002` with the window of `FR-SRV-015`, and the classification `OD-06` drops the driver's error at |
//! | `mariadb::catalogue` | The fixed repertoire of catalogue queries — one per object kind, whose count `NFR-PERF-001` and `NFR-PERF-002` fix — the common column lists of `FR-SRV-037`, the fold that turns their rows into the model, and the completeness verdict of `FR-PRIV-001` … `FR-PRIV-019` it takes as it folds |
//! | `cache` | The store of `FR-CACHE-001` … `FR-CACHE-037`: one folder per entry, one file per object written through a rename, the two versions and the completeness record of `FR-CDOC-001` … `FR-CDOC-007`, and a failure in either direction that is a miss rather than a condition |
//! | `render` | The engine of `ADR-001`, built lazily and from disk at render time; the one template-name resolution of `FR-TMPL-023` … `FR-TMPL-027`; and the registered surface of `FR-ENV-005` … `FR-ENV-046` with the semantics of `FR-SEM-001` … `FR-SEM-019` |
//!
//! A read is honest about what a reader's privileges did not reach: an object
//! that came back short carries the `restricted` marking of `FR-PRIV-016`, a
//! caller that named one receives the `77` of `FR-PRIV-003`, and `FR-CACHE-037`
//! keeps a marked object out of the store. The render environment exists and
//! registers, so `tpl help --format json` publishes the whole template surface;
//! the two arms that consume it — `tpl template …` and `tpl render` — are added
//! by the tasks that follow.

#![forbid(unsafe_code)]
#![warn(missing_docs)]

pub mod error;

pub mod model;

pub(crate) mod cache;

pub(crate) mod cli;

pub(crate) mod deadline;

pub(crate) mod diagnostics;

pub(crate) mod project;

pub(crate) mod mariadb;

pub(crate) mod output;

// The module is complete and two of its three consumers are later blocks: the
// four `tpl template` subcommands and `tpl render` reach the engine and the
// resolution, and `cli/help.rs` already reaches the registered surface. One
// fact explains every item, so it is stated once here rather than once per
// item.
#[allow(
    dead_code,
    reason = "the commands that build a render environment — the four of `tpl template` and \
              `tpl render` — are a later block, and the block that owns the engine owns the \
              surface it registers"
)]
pub(crate) mod render;

pub use error::Error;

/// Installs the panic hook of `ADR-004`.
///
/// `FR-ERR-030` gives `70` two producing conditions, and this is the one that
/// is not an [`Error`] value: WHEN a panic occurs, the process writes the four
/// labelled lines of `FR-ERR-008` — with the `hint` of `FR-ERR-032` and a
/// `cause` naming where the panic arose — and terminates with `70`, leaving
/// stdout empty as `FR-ERR-033` requires.
///
/// The binary calls this once, before anything else, and it is the binary's to
/// call rather than [`run`]'s: a panic hook is process-wide state, and a
/// library function that installed one as a side effect would impose it on
/// every caller of [`run`], including a test binary that must be free to panic.
///
/// Calling it twice replaces the hook with an equivalent one. Not calling it
/// leaves the standard library's default hook in place, which prints the panic
/// payload `FR-GLOB-018` bars and exits by aborting rather than with `70`.
pub fn install_panic_hook() {
    diagnostics::panic::install();
}

/// Runs `tpl`.
///
/// The four labelled lines of `FR-ERR-008` are written here, on the way out:
/// `FR-ERR-033` puts them on stderr and leaves stdout empty, and `OD-05`
/// reduces `main.rs` to parsing, dispatching and mapping the error to an exit
/// status. The exit status itself is the library's to decide, never the
/// binary's: `OD-06` puts the derivation on [`Error::exit_code`] so that the
/// assignment is made in one exhaustive match inside this crate, and `main`
/// returns what that yields without classifying anything of its own.
///
/// # Errors
///
/// Returns the [`Error`] of the first condition that fails, in the order
/// `FR-ERR-006` fixes, after it has been reported. At this commit step 1 of
/// that order is the whole of it: the invocation is parsed, and a command that
/// parses reports the interim `70` its own module documents.
pub fn run() -> Result<(), Error> {
    dispatch().inspect_err(diagnostics::report)
}

/// Parses the invocation and runs the command it names.
///
/// This is where the parser tree of `cli/` is reached, and it is reached once:
/// [`cli::parse`] is the only route from the process to the parser, so the
/// interception `OD-08` requires cannot be bypassed and no byte the parser's
/// own renderer composes can reach either stream.
///
/// The instant `--timeout` is measured from is recorded first, and the
/// diagnostic level of `FR-GLOB-014` and `FR-GLOB-015` is fixed between the
/// two, which is where `OD-17` places it — after the invocation has been parsed
/// and before anything that emits. A failure to parse leaves it at the level of
/// a run that supplied neither flag, which costs nothing: the four labelled
/// lines of `FR-ERR-008` are written at every level.
///
/// It reports nothing: the diagnostic is written once, by [`run`], for whatever
/// condition reaches it first.
fn dispatch() -> Result<(), Error> {
    // FR-GLOB-011 measures the overall budget from process start, and this is
    // the earliest instant a library function can record: the argument vector
    // has not been read, so nothing blocking can have run.
    deadline::mark_process_start();

    let invocation = cli::parse(std::env::args_os())?;

    diagnostics::verbosity::set_level(cli::level(&invocation));

    cli::dispatch(&invocation)
}
