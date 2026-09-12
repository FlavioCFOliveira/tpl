//! `tpl` reads the structure of a MariaDB database and applies MiniJinja
//! templates to it.
//!
//! The crate is split in two halves, per `ADR-006`: this library holds the
//! logic, so that it is addressable from a test without launching a process,
//! and the binary parses the invocation, dispatches, and maps the resulting
//! error to an exit status.
//!
//! At this commit the library carries no command. [`run`] is the single entry
//! point the binary calls, and [`Error`] is the value every module will report
//! failure through; the parser tree, the catalogue reader and the render
//! environment are added by the tasks that follow.

#![forbid(unsafe_code)]
#![warn(missing_docs)]

pub mod error;

pub use error::Error;

/// Runs `tpl`.
///
/// The exit status is the library's to decide, never the binary's: `OD-06`
/// puts the derivation on [`Error::exit_code`] so that the assignment is made
/// in one exhaustive match inside this crate, and `main` returns what that
/// yields without classifying anything of its own.
///
/// # Errors
///
/// Returns the [`Error`] of the first condition that fails, in the order
/// `FR-ERR-006` fixes. At this commit no command is wired up, so no condition
/// can arise and the function succeeds without reading or writing anything.
pub fn run() -> Result<(), Error> {
    Ok(())
}
