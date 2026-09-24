//! The panic hook of `ADR-004`, and one of the two calls to
//! `std::process::exit` in the crate.
//!
//! `FR-ERR-030` gives `70` two producing conditions — a panic in the process,
//! and a detected invariant violation — and requires both to exist in the
//! binary the project distributes, or a code `FR-ERR-001` makes contract is one
//! that binary cannot return. The invariant violation is an [`Error`] value and
//! travels the ordinary path; the panic does not, and this module is how it
//! reaches the caller as the same four labelled lines and the same status.
//!
//! `ADR-004` fixes the mechanism, because the release profile sets
//! `panic = "abort"` and no frame above the panic site runs under it. A panic
//! hook does: it "is invoked when a thread panics, but before the panic runtime
//! is invoked", so it "will run with both the aborting and unwinding runtimes"
//! (`std::panic::set_hook`). The hook writes the message and terminates through
//! [`std::process::exit`], and the aborting runtime is never reached.
//!
//! Nothing is written to stdout on this path, and nothing is flushed to it:
//! [`std::process::exit`] runs no destructor, so a buffered stdout is
//! discarded rather than emitted, which is the outcome `FR-ERR-033` requires.
//!
//! The other call is the render deadline of `FR-RND-033`, in
//! [`crate::cli`]'s third arm. `OD-12` gives it the same mechanism for the same
//! reason: a timer thread holds a condition no frame of the render could
//! return, so it writes the four labelled lines itself and terminates the
//! process with the status they named.
//!
//! [`Error`]: crate::error::Error

use std::panic::set_hook;
use std::process::exit;

use super::render;

/// Installs the panic hook, so that a panic produces the outcome `FR-ERR-030`
/// requires rather than whatever the default hook prints.
///
/// The binary calls this once, before anything else it does. It costs one boxed
/// closure and reads nothing: no configuration, no environment, no stream — the
/// verbosity level of `FR-GLOB-014` included, because `-q` lowers the stream to
/// errors only and a panic is one at every level.
///
/// The hook replaces the default one for the whole process, which is what makes
/// the four labelled lines the only thing a panic can write: the default hook's
/// own message, which carries the panic payload `FR-GLOB-018` keeps off this
/// stream, is no longer installed to produce it.
pub(crate) fn install() {
    set_hook(Box::new(|info| {
        render::report_panic(info.location());

        // ADR-004: the status is the `70` the `exit` line just named, and
        // `std::process::exit` passes it through to the operating system.
        exit(i32::from(render::SOFTWARE));
    }));
}
