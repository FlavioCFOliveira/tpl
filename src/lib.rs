//! `tpl` reads the structure of a MariaDB database and applies MiniJinja
//! templates to it.
//!
//! The crate is split in two halves, per `ADR-006`: this library holds the
//! logic, so that it is addressable from a test without launching a process,
//! and the binary parses the invocation, dispatches, and maps the resulting
//! error to an exit status.
//!
//! At this commit no command of the library acts. [`run`] is the entry point
//! the binary calls, [`install_panic_hook`] is the process setup it performs
//! first, [`Error`] is the value every module will report failure through,
//! `cli` declares the closed command tree of `FR-CLI-002` and the seven global
//! flags of `FR-GLOB-001` every node of it accepts, `diagnostics` writes the
//! four labelled lines of `FR-ERR-008` that a failure reaches the caller as,
//! and `output` holds the two formats every result reaches the caller through
//! — the envelope of `FR-OUT-024` and the aligned columns of `FR-OUT-006`; the
//! catalogue reader and the render environment are added by the tasks that
//! follow.

#![forbid(unsafe_code)]
#![warn(missing_docs)]

pub mod error;

// The tree parses today and no node of it acts: `route` gives every leaf the
// interim `70` its own module documents, and the route from the process to the
// parser carries the one decision `OD-08` places outside this module. One fact
// explains every item the lint names, so it is stated once here.
#[allow(
    dead_code,
    reason = "no command is wired to the process yet; the parser tree is declared here because \
              FR-CLI-002 closes it, and the interception of the parser's own failures that \
              reaches it is a later task"
)]
pub(crate) mod cli;

pub(crate) mod diagnostics;

// Nothing emits a result yet: `OD-05` places the envelope, the emitter, the
// `text` layout and the writer in this module, and the commands that reach them
// are later sprints. One fact explains every constructor and enumerated value
// the lint names, so it is stated once here rather than once per item, and the
// attribute goes with the first command that emits.
#[allow(
    dead_code,
    reason = "the commands that emit a result are later sprints; OD-05 places the envelope, the \
              emitter, the text layout and the writer here, and every one of those commands \
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
/// `FR-ERR-006` fixes, after it has been reported. At this commit no command is
/// wired up, so no condition can arise and the function succeeds without
/// reading or writing anything.
pub fn run() -> Result<(), Error> {
    dispatch().inspect_err(diagnostics::report)
}

/// Parses the invocation and runs the command it names.
///
/// This is where the parser tree of `cli/` is reached. It reports nothing: the
/// diagnostic is written once, by [`run`], for whatever condition reaches it
/// first.
fn dispatch() -> Result<(), Error> {
    Ok(())
}
