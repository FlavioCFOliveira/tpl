//! `tpl` reads the structure of a MariaDB database and applies MiniJinja
//! templates to it.
//!
//! The crate is split in two halves, per `ADR-006`: this library holds the
//! logic, so that it is addressable from a test without launching a process,
//! and the binary parses the invocation, dispatches, and maps the resulting
//! error to an exit status.
//!
//! At this commit the library carries no command. [`run`] is the single entry
//! point the binary calls; the parser tree, the catalogue reader, the render
//! environment and the error taxonomy are added by the tasks that follow.

#![forbid(unsafe_code)]
#![warn(missing_docs)]

use std::process::ExitCode;

/// Runs `tpl` and yields the process's exit status.
///
/// The status is the library's to decide, never the binary's: `OD-06` puts the
/// exit-code derivation beside the error type so that the assignment is made in
/// one exhaustive match inside this crate. Until that type exists there is no
/// condition to classify, and the function succeeds without reading or writing
/// anything.
#[must_use]
pub fn run() -> ExitCode {
    ExitCode::SUCCESS
}
