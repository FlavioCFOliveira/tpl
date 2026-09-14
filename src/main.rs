//! The `tpl` executable.
//!
//! It calls the library and returns what the library decided. It classifies
//! nothing of its own: the exit status comes from the error type in the
//! library, per `OD-06`.
//!
//! The one thing it does before that is process setup rather than work: the
//! panic hook of `ADR-004` is installed here because a panic hook is a property
//! of the **process**, and this is the process. Everything it then writes is
//! composed in the library, on the same path every other diagnostic takes.

#![forbid(unsafe_code)]

use std::process::ExitCode;

fn main() -> ExitCode {
    tpl::install_panic_hook();

    match tpl::run() {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => ExitCode::from(error.exit_code()),
    }
}
