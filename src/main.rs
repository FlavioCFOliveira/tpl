//! The `tpl` executable.
//!
//! It calls the library and returns what the library decided. It classifies
//! nothing of its own: the exit status comes from the error type in the
//! library, per `OD-06`.

#![forbid(unsafe_code)]

use std::process::ExitCode;

fn main() -> ExitCode {
    tpl::run()
}
