//! `tpl` reads the structure of a MariaDB database and applies MiniJinja
//! templates to it.
//!
//! The crate is split in two halves, per `ADR-006`: this library holds the
//! logic, so that it is addressable from a test without launching a process,
//! and the binary parses the invocation, dispatches, and maps the resulting
//! error to an exit status.
//!
//! At this commit the tool knows **where** it would connect, and still connects
//! to nothing. [`run`] is the entry point the binary calls,
//! [`install_panic_hook`] is the process setup it performs first, and [`Error`]
//! is the value every module reports failure through.
//!
//! | Module | What it owns |
//! |---|---|
//! | `cli` | The closed command tree of `FR-CLI-002`, the seven global flags of `FR-GLOB-001` every node accepts, the parsing rules of `FR-CLI-014` through `FR-CLI-020`, the six help and version forms of `FR-HELP-001`, and the `cfg` arm that maintains `.tpl/.cfg` |
//! | `project` | Where a project is found and why it is trusted (`FR-PROJ-001` … `FR-PROJ-011`), what `tpl init` creates (`FR-PROJ-012` … `FR-PROJ-024`), how `.tpl/.cfg` is read, validated and rewritten (`FR-CONF-001` … `FR-CONF-036`), and the settings a connection will need (`FR-CONF-004`, `FR-CONF-029`) |
//! | `deadline` | The clock every blocking phase is bounded by: the four `[core]` deadlines of `FR-CONF-005` and the overall budget of `FR-GLOB-011`, composed as `FR-GLOB-012` composes them |
//! | `diagnostics` | The four labelled lines of `FR-ERR-008` a failure reaches the caller as |
//! | `output` | The two formats a result reaches the caller through — the envelope of `FR-OUT-024` and the aligned columns of `FR-OUT-006` |
//!
//! The catalogue reader and the render environment are added by the tasks that
//! follow.

#![forbid(unsafe_code)]
#![warn(missing_docs)]

pub mod error;

pub(crate) mod cli;

pub(crate) mod deadline;

pub(crate) mod diagnostics;

pub(crate) mod project;

// Four items of this module have no caller yet, and all four wait on the same
// sprint: `emit` and `emit_table`, which take standard output where the `cfg`
// arm takes the stream its caller supplies; the `server` and `cache` values of
// `FR-OUT-026`; and the excepted order of `NFR-DET-002`, which only a catalogue
// collection carries. One fact explains every one of them, so it is stated once
// here rather than once per item.
#[allow(
    dead_code,
    reason = "the commands that read a server are a later sprint, and OD-05 places the envelope, \
              the emitter, the text layout and the writer here, so every one of those commands \
              depends on them"
)]
pub(crate) mod output;

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
