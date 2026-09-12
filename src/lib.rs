//! `tpl` reads the structure of a MariaDB database and applies MiniJinja
//! templates to it.
//!
//! The crate is split in two halves, per `ADR-006`: this library holds the
//! logic, so that it is addressable from a test without launching a process,
//! and the binary parses the invocation, dispatches, and maps the resulting
//! error to an exit status.
//!
//! At this commit the library carries no command. [`run`] is the single entry
//! point the binary calls, [`Error`] is the value every module will report
//! failure through, and `diagnostics` writes the four labelled lines of
//! `FR-ERR-008` that a failure reaches the caller as; the parser tree, the
//! catalogue reader and the render environment are added by the tasks that
//! follow.

#![forbid(unsafe_code)]
#![warn(missing_docs)]

pub mod error;

pub(crate) mod diagnostics;

pub use error::Error;

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
