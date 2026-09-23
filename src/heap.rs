//! The heap count the render memory limit of `FR-RND-039` bounds (`ADR-011`).
//!
//! The count comes from the counting global allocator of `ADR-011`, and that
//! allocator is **installed by the binary**, not by this library. A global
//! allocator is process-wide state, exactly as the panic hook of `ADR-004` is,
//! and it is left to the binary for the reason [`crate::install_panic_hook`]
//! states: a library that imposed it would impose it on every program that
//! links the library, including each test binary. The technical
//! specification records the arrangement in its refinement of `OD-05`. The library therefore owns no allocator. It owns the one place the
//! count is read from, and the binary hands it the reading through
//! [`crate::install_heap_counter`] before anything else runs.
//!
//! *Rejected: the library declaring the `#[global_allocator]` itself.* Every
//! test binary of the package would then run under it, and the choice of the
//! process's one allocator — which `ADR-011` notes excludes an in-tree `dhat`
//! profiler — would be made for every program the library is linked into
//! rather than for the one that ships.
//!
//! Where no counter was installed — an in-process test — [`allocated`]
//! answers [`None`] and the render memory limit observes nothing. Every
//! other bound applies unchanged. The limit is verified end to end, through
//! the binary, where the counter is always installed.

use std::sync::OnceLock;

/// The function the binary installed that reads the process's heap count.
static COUNTER: OnceLock<fn() -> usize> = OnceLock::new();

/// Records `counter` as the reading of the process's heap count.
///
/// The first installation wins; a later one is ignored, so the count a render
/// is bounded by cannot change under it.
pub(crate) fn install(counter: fn() -> usize) {
    let _ = COUNTER.set(counter);
}

/// The heap bytes the process holds allocated, as its allocator counts them,
/// or [`None`] where no counter was installed.
///
/// Reading it allocates nothing, which is what lets the render's watchdog
/// thread call it while the render it bounds is allocating.
pub(crate) fn allocated() -> Option<usize> {
    COUNTER.get().map(|counter| counter())
}
