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
    /// The two flags are mutually exclusive, so `quiet && verbose > 0` cannot
    /// reach this function: `FR-CLI-015` refuses the pair with exit `64`
    /// (`EX_USAGE`) while the arguments are parsed, before any level is
    /// resolved, and `FR-GLOB-015` names that refusal where both flags are
    /// declared. No precedence between the two is encoded here, because the
    /// corpus has none to encode.
    ///
    /// The function is nevertheless total, and does not panic in any build
    /// profile. The pair the parser refuses resolves to [`Level::Warnings`] —
    /// the level of a run that supplied neither flag, and this enum's default
    /// — which is the one outcome that favours neither flag. It is the same
    /// degradation [`Level::from_repr`] applies to a discriminant that cannot
    /// arise.
    pub(crate) const fn resolve(verbose: u8, quiet: bool) -> Self {
        match (verbose, quiet) {
            // FR-GLOB-015.
            (0, true) => Self::Errors,
            (0, false) => Self::Warnings,
            (1, false) => Self::Info,
            (2, false) => Self::Debug,
            // FR-GLOB-014: further occurrences saturate at TRACE without error.
            (_, false) => Self::Trace,
            // Unreachable per FR-CLI-015, and resolved in neither flag's
            // favour: the default, which `Default::default` cannot yield in a
            // `const fn`.
            (_, true) => Self::Warnings,
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
    fn fr_glob_014_one_v_is_info_two_is_debug_three_is_trace() {
        // FR-GLOB-014.
        assert_eq!(Level::resolve(0, false), Level::Warnings);
        assert_eq!(Level::resolve(1, false), Level::Info);
        assert_eq!(Level::resolve(2, false), Level::Debug);
        assert_eq!(Level::resolve(3, false), Level::Trace);
    }

    #[test]
    fn fr_glob_014_further_occurrences_saturate_without_error() {
        // FR-GLOB-014.
        for verbose in 3u8..=u8::MAX {
            assert_eq!(Level::resolve(verbose, false), Level::Trace);
        }
    }

    #[test]
    fn fr_glob_015_quiet_lowers_the_level_to_errors_only() {
        // FR-GLOB-015.
        assert_eq!(Level::resolve(0, true), Level::Errors);
    }

    /// `FR-CLI-015` exits `64` on `-q` with `-v`, so the pair never reaches
    /// `resolve`. The function is total regardless, and neither flag wins.
    #[test]
    fn fr_cli_015_the_pair_the_parser_refuses_favours_neither_flag() {
        for verbose in 1u8..=u8::MAX {
            assert_eq!(Level::resolve(verbose, true), Level::default());
        }
        assert_eq!(Level::default(), Level::Warnings);
    }

    #[test]
    fn fr_glob_014_the_levels_are_ordered_so_that_the_gate_is_a_comparison() {
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
    fn fr_glob_014_the_gate_opens_at_the_level_that_was_set() {
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
