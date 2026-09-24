//! The `tpl` executable.
//!
//! It calls the library and returns what the library decided. It classifies
//! nothing of its own: the exit status comes from the error type in the
//! library, per `OD-06`.
//!
//! The one thing it does before that is process setup rather than work: the
//! panic hook of `ADR-004` and the counting allocator of `ADR-011` are
//! installed here because each is a property of the **process**, and this is
//! the process. Everything it then writes is composed in the library, on the
//! same path every other diagnostic takes.

#![forbid(unsafe_code)]

use std::alloc::System;
use std::process::ExitCode;

/// The counting allocator of `ADR-011`, whose count the render memory limit of
/// `FR-RND-039` bounds.
///
/// Its own hard limit is `usize::MAX` and `set_limit` is never called
/// (`ADR-011`, point 3): it refuses no allocation of its own accord, so the
/// only allocation failure left is one the operating system produces.
#[global_allocator]
static ALLOCATOR: cap::Cap<System> = cap::Cap::new(System, usize::MAX);

fn main() -> ExitCode {
    tpl::install_panic_hook();
    tpl::install_heap_counter(|| ALLOCATOR.allocated());

    match tpl::run() {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => ExitCode::from(error.exit_code()),
    }
}
