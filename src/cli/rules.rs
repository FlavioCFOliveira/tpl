//! The parsing rules `tpl` applies to an invocation the parser accepted.
//!
//! `FR-ERR-006` puts argument parsing first in the validation order and runs it
//! "for every command without exception". Step 1 is therefore three things in
//! one, in this order:
//!
//! | Order | What is decided | Where |
//! |---|---|---|
//! | 1 | Whatever the parser itself refuses — an unknown command, an unknown flag, a value outside an enumeration | [`super::intercept`] |
//! | 2 | A flag that carries a single value, given more than once (`FR-CLI-014`) | [`refuse_repetition`] |
//! | 3 | `-q/--quiet` together with `-v/--verbose` (`FR-CLI-015`) | [`refuse_both_verbosities`] |
//!
//! The order among the three is this module's choice and is stated here because
//! `FR-ERR-006` fixes the order *between* steps and not within one. A refusal
//! that is a property of **one** flag precedes a refusal that is a property of
//! **two**, so `tpl -d a -d b -q -v version` reports the repetition: the
//! repeated flag is wrong however the rest of the line reads, and the pair is
//! only wrong as a pair.
//!
//! `FR-CLI-016` needs no rule of its own. The count saturates twice over — at
//! `255` in the parser, which the declaration in [`super::globals`] records,
//! and at `TRACE` in [`level`] — so "further occurrences saturate at `TRACE`
//! without error" holds at the fourth occurrence and at the three-hundredth
//! alike.
//!
//! `FR-CLI-017` and `FR-CLI-020` need no rule either, and neither is therefore
//! implemented here: the parser terminates the arguments at `--` and matches a
//! flag and a value byte for byte. Both are properties this project guarantees
//! rather than inherits, so both are asserted by tests in [`super`].

use clap::{ArgAction, ArgMatches};

use super::globals::Globals;
use crate::diagnostics::verbosity::Level;
use crate::error::{self, Error};

/// The one flag of the tree that is repeatable by requirement.
///
/// `FR-RND-008` makes `--set` repeatable with distinct keys, so it is outside
/// `FR-CLI-014`, which governs a flag that carries **a single value**. The set
/// is written as identifiers rather than as spellings because that is what
/// `ArgMatches` is keyed by, and it is asserted against the tree by a test: a
/// second repeatable flag added to the tree without an entry here would be
/// refused on its second occurrence.
const REPEATABLE: [&str; 1] = ["set"];

/// `-q/--quiet`, in the long form the tree declares it under.
const QUIET: &str = "--quiet";

/// `-v/--verbose`, in the long form the tree declares it under.
const VERBOSE: &str = "--verbose";

/// Whether the tree accepts `argument` more than once.
///
/// It is the rule [`refuse_repetition`] applies, read the other way round, and
/// it is here rather than in the renderer of [`super::help`] so that what the
/// help states and what the invocation meets cannot drift apart. Reading the
/// action instead would state the opposite of both: `OD-08` declares a flag
/// that carries **one** value with `ArgAction::Append` so that both values
/// reach the message of `FR-CLI-014`, and that flag is refused on its second
/// occurrence all the same.
///
/// Three populations are repeatable, and no other:
///
/// | Population | Why |
/// |---|---|
/// | `-v/--verbose` | `FR-CLI-016` counts the occurrences and saturates them |
/// | A positional argument that takes many values | `FR-TMPL-018` and `FR-HELP-026` are the sequence, not a repetition |
/// | The flags of [`REPEATABLE`] | `FR-RND-008` makes `--set` repeatable with distinct keys |
pub(super) fn repeats(argument: &clap::Arg) -> bool {
    match argument.get_action() {
        ArgAction::Count => true,
        ArgAction::Append => {
            argument.get_long().is_none() || REPEATABLE.contains(&argument.get_id().as_str())
        }
        _ => false,
    }
}

/// Refuses a flag that carries a single value and was given more than once
/// (`FR-CLI-014`).
///
/// The rule is applied over the **declarations** rather than over a list of
/// flags: every flag declared with `ArgAction::Append` carries one value and is
/// refused on its second occurrence, except the one [`REPEATABLE`] names. A
/// flag added to the tree is therefore governed without anything here changing,
/// which is the property `OD-08` bought by declaring these flags repeatable in
/// the first place.
///
/// Both values reach the message, which is what `FR-CLI-014` obliges and what
/// the parser's own `ArgumentConflict` cannot supply: the values are read from
/// [`ArgMatches::get_raw`], so they are the bytes the caller wrote, in the
/// order written and with their case untouched, per `FR-CLI-020`.
///
/// A global flag is declared at the root and its occurrences are propagated
/// back to the root's matches from whatever depth they were written at, so the
/// root is where a repetition of one is found, whichever node carried it.
///
/// # Errors
///
/// Returns [`Error::RepeatedValueFlag`] naming the flag and its first two
/// values, or [`Error::InternalInvariant`] where the matched tree and the
/// matches disagree about which nodes exist, which is a defect in `tpl` rather
/// than in the invocation.
pub(super) fn refuse_repetition(
    command: &clap::Command,
    matches: &ArgMatches,
) -> Result<(), Error> {
    for argument in command.get_arguments() {
        let Some(long) = argument.get_long() else {
            // FR-CLI-014 governs a flag. A positional argument that takes many
            // values — `template check`, `help` — is the requirement the
            // sequence serves, not a repetition.
            continue;
        };

        if !matches!(argument.get_action(), ArgAction::Append) || repeats(argument) {
            continue;
        }

        let Some(mut written) = matches.get_raw(argument.get_id().as_str()) else {
            continue;
        };

        if let (Some(first), Some(second)) = (written.next(), written.next()) {
            return Err(Error::RepeatedValueFlag {
                flag: format!("--{long}"),
                first: first.to_string_lossy().into_owned(),
                second: second.to_string_lossy().into_owned(),
            });
        }
    }

    let Some((name, inner)) = matches.subcommand() else {
        return Ok(());
    };
    let Some(child) = command.find_subcommand(name) else {
        return error::ensure_invariant(
            false,
            "a matched subcommand is a node of the tree it was matched against",
        );
    };

    refuse_repetition(child, inner)
}

/// Refuses `-q/--quiet` given together with `-v/--verbose` (`FR-CLI-015`).
///
/// The pair is refused here rather than declared as a `conflicts_with` on
/// either flag, for the reason [`super::globals`] gives: a refusal written in
/// the parser's words is a refusal the caller never reads in the four labelled
/// lines of `FR-ERR-008`. `FR-GLOB-015` names the same refusal where both flags
/// are declared, and encodes no precedence between them — neither does this.
///
/// # Errors
///
/// Returns [`Error::MutuallyExclusiveFlags`] naming both members of the pair,
/// which is what the `64` row of `FR-ERR-034` obliges the `cause` line to name.
pub(super) fn refuse_both_verbosities(globals: &Globals) -> Result<(), Error> {
    if globals.quiet && globals.verbose > 0 {
        return Err(Error::MutuallyExclusiveFlags {
            first: QUIET.to_owned(),
            second: VERBOSE.to_owned(),
        });
    }

    Ok(())
}

/// The diagnostic level the two flags resolve to (`FR-GLOB-014`,
/// `FR-GLOB-015`, `FR-CLI-016`).
///
/// `OD-17` resolves the level once, during argument handling, and this is that
/// resolution; the process fixes it for the rest of the run. The saturation of
/// `FR-CLI-016` is the resolution's own: three occurrences and three hundred
/// both reach [`Level::Trace`], and neither is an error.
pub(super) fn level(globals: &Globals) -> Level {
    Level::resolve(globals.verbose, globals.quiet)
}

#[cfg(test)]
mod tests {
    use super::{QUIET, REPEATABLE, VERBOSE, level, refuse_both_verbosities};
    use crate::cli::globals::Globals;
    use crate::diagnostics::verbosity::Level;

    /// The seven flags, with neither verbosity flag given.
    fn globals(verbose: u8, quiet: bool) -> Globals {
        Globals {
            database: Vec::new(),
            tpl_dir: Vec::new(),
            timeout: Vec::new(),
            verbose,
            quiet,
            help: false,
            version: false,
        }
    }

    #[test]
    fn fr_cli_015_the_two_verbosity_flags_are_refused_together_and_alone_are_not() {
        // FR-CLI-015 and FR-GLOB-015.
        let refused = refuse_both_verbosities(&globals(1, true)).expect_err("the pair is refused");

        assert_eq!(refused.exit_code(), 64);
        assert!(refused.to_string().contains(QUIET), "{refused}");
        assert!(refused.to_string().contains(VERBOSE), "{refused}");

        refuse_both_verbosities(&globals(0, true)).expect("quiet alone is not a refusal");
        refuse_both_verbosities(&globals(3, false)).expect("verbose alone is not a refusal");
        refuse_both_verbosities(&globals(0, false)).expect("neither flag is not a refusal");
    }

    #[test]
    fn fr_cli_016_the_verbosity_count_saturates_at_trace_without_error() {
        // FR-CLI-016 and FR-GLOB-014: one INFO, two DEBUG, three TRACE, and
        // every further occurrence TRACE. The count reaching this function is
        // itself saturated at 255 by the parser, which `super::super` asserts
        // over an argument vector.
        assert_eq!(level(&globals(0, false)), Level::Warnings);
        assert_eq!(level(&globals(1, false)), Level::Info);
        assert_eq!(level(&globals(2, false)), Level::Debug);
        assert_eq!(level(&globals(3, false)), Level::Trace);
        assert_eq!(level(&globals(4, false)), Level::Trace);
        assert_eq!(level(&globals(u8::MAX, false)), Level::Trace);

        assert_eq!(level(&globals(0, true)), Level::Errors);
    }

    #[test]
    fn fr_rnd_008_the_only_repeatable_flag_is_the_one_the_requirement_names() {
        // FR-RND-008 makes `--set` repeatable and nothing else in the tree is.
        // The identifiers are what `ArgMatches` is keyed by, so a flag renamed
        // without this constant following it fails here rather than silently
        // stopping being governed.
        assert_eq!(REPEATABLE, ["set"]);

        let tree = crate::cli::tree();
        let render = tree.find_subcommand("render").expect("render is a node");
        let set = render
            .get_arguments()
            .find(|argument| argument.get_id() == "set")
            .expect("render declares --set");

        assert_eq!(set.get_long(), Some("set"));
    }
}
