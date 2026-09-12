//! The verbosity gate of `FR-GLOB-014` and `FR-GLOB-015`.
//!
//! `-v/--verbose` raises the diagnostic level — one occurrence is `INFO`, two
//! `DEBUG`, three `TRACE`, and further occurrences saturate at `TRACE` without
//! error. `-q/--quiet` lowers it to errors only. Neither touches stdout, per
//! `FR-GLOB-016`, and neither reaches the four labelled lines of `FR-ERR-008`:
//! an error is emitted at every level, because "errors only" is the *floor*.
//!
//! `OD-17` fixes the shape: the level is resolved once, during argument
//! handling, and read from an ordinary shared value. No subscriber is
//! installed, no facade is used, and nothing a dependency emits can reach the
//! stream — the raw driver error `FR-GLOB-018` names first among the six
//! therefore has no route to stderr at all.

use std::sync::atomic::{AtomicU8, Ordering};

/// The diagnostic level a run emits at.
///
/// The five are ordered, and the ordering is the gate: an emission at
/// [`Level::Info`] is written when the resolved level is at or above it.
/// [`Level::Warnings`] is the level of a run that supplies neither flag, and is
/// where the warning of `FR-PROJ-016` is written.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Default)]
#[repr(u8)]
pub(crate) enum Level {
    /// `-q/--quiet`: the four labelled lines of a failure, and nothing else.
    Errors = 0,
    /// Neither flag: failures, and the warnings the specification obliges.
    #[default]
    Warnings = 1,
    /// One `-v`: which phases ran and how long each took, and one line per
    /// catalogue query (`FR-GLOB-017`).
    Info = 2,
    /// Two `-v`: additionally each cache hit and miss (`FR-GLOB-017`).
    Debug = 3,
    /// Three or more `-v`: internal detail (`FR-GLOB-017`).
    Trace = 4,
}

impl Level {
    /// Resolves the level from the two flags, once, per `OD-17`.
    ///
    /// `verbose` is the number of occurrences of `-v/--verbose`, which
    /// `FR-CLI-016` counts to three and saturates above; `quiet` is whether
    /// `-q/--quiet` was supplied.
    ///
    /// Where both are supplied, `quiet` decides. The specification declares
    /// the two neither exclusive nor ordered, and the narrower outcome is the
    /// one a caller can recover from by dropping a flag.
    #[allow(
        dead_code,
        reason = "the flags that supply these two arguments belong to `cli/`, a later \
                  sprint; OD-17 resolves the level during argument handling and this is that \
                  resolution"
    )]
    pub(crate) const fn resolve(verbose: u8, quiet: bool) -> Self {
        if quiet {
            return Self::Errors;
        }

        match verbose {
            0 => Self::Warnings,
            1 => Self::Info,
            2 => Self::Debug,
            // FR-GLOB-014: further occurrences saturate at TRACE without error.
            _ => Self::Trace,
        }
    }

    /// The level a stored discriminant denotes.
    ///
    /// Only [`set_level`] writes the shared value and it writes a discriminant
    /// of this enum, so no other value can arise; one would degrade to the
    /// default rather than panic on the path that reports a failure.
    const fn from_repr(value: u8) -> Self {
        match value {
            0 => Self::Errors,
            2 => Self::Info,
            3 => Self::Debug,
            4 => Self::Trace,
            _ => Self::Warnings,
        }
    }
}

/// The resolved level, as an ordinary shared value.
///
/// `Relaxed` is the weakest correct ordering here and is sufficient: the value
/// is a single scalar that publishes no other memory, and it is written once
/// during argument handling — before any work that emits, and before any thread
/// that might emit exists, so the spawn itself carries the edge.
static LEVEL: AtomicU8 = AtomicU8::new(Level::Warnings as u8);

/// Fixes the level for the rest of the run.
///
/// Called once, from argument handling. Calling it later does not corrupt the
/// value; it merely changes which emissions are written from that point on.
#[allow(
    dead_code,
    reason = "argument handling, which calls this once, belongs to `cli/`, a later sprint"
)]
pub(crate) fn set_level(level: Level) {
    LEVEL.store(level as u8, Ordering::Relaxed);
}

/// The level this run emits at.
pub(crate) fn level() -> Level {
    Level::from_repr(LEVEL.load(Ordering::Relaxed))
}

/// Whether an emission at `at` is written at the resolved level.
pub(crate) fn emits(at: Level) -> bool {
    level() >= at
}

#[cfg(test)]
mod tests {
    use super::{Level, emits, level, set_level};

    #[test]
    fn one_v_is_info_two_is_debug_three_is_trace() {
        // FR-GLOB-014.
        assert_eq!(Level::resolve(0, false), Level::Warnings);
        assert_eq!(Level::resolve(1, false), Level::Info);
        assert_eq!(Level::resolve(2, false), Level::Debug);
        assert_eq!(Level::resolve(3, false), Level::Trace);
    }

    #[test]
    fn further_occurrences_saturate_without_error() {
        // FR-GLOB-014.
        for verbose in 3u8..=u8::MAX {
            assert_eq!(Level::resolve(verbose, false), Level::Trace);
        }
    }

    #[test]
    fn quiet_lowers_the_level_to_errors_only() {
        // FR-GLOB-015.
        assert_eq!(Level::resolve(0, true), Level::Errors);
        assert_eq!(Level::resolve(9, true), Level::Errors);
    }

    #[test]
    fn the_levels_are_ordered_so_that_the_gate_is_a_comparison() {
        assert!(Level::Errors < Level::Warnings);
        assert!(Level::Warnings < Level::Info);
        assert!(Level::Info < Level::Debug);
        assert!(Level::Debug < Level::Trace);
        assert_eq!(Level::default(), Level::Warnings);
    }

    #[test]
    fn a_stored_discriminant_round_trips() {
        for expected in [
            Level::Errors,
            Level::Warnings,
            Level::Info,
            Level::Debug,
            Level::Trace,
        ] {
            assert_eq!(Level::from_repr(expected as u8), expected);
        }
    }

    /// The one test that touches the shared value. No other test reads it, so
    /// running in parallel with them observes nothing.
    #[test]
    fn the_gate_opens_at_the_level_that_was_set() {
        let restore = level();

        set_level(Level::Errors);
        assert!(!emits(Level::Warnings));
        assert!(!emits(Level::Info));

        set_level(Level::Info);
        assert!(emits(Level::Warnings));
        assert!(emits(Level::Info));
        assert!(!emits(Level::Debug));

        set_level(Level::Trace);
        assert!(emits(Level::Trace));

        set_level(restore);
    }
}
