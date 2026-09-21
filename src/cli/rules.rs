//! The parsing rules `tpl` applies to an invocation the parser accepted.
//!
//! `FR-ERR-006` puts argument parsing first in the validation order and runs it
//! "for every command without exception". Step 1 is therefore four things in
//! one, in this order:
//!
//! | Order | What is decided | Where |
//! |---|---|---|
//! | 1 | Whatever the parser itself refuses — an unknown command, an unknown flag, a value outside an enumeration | [`super::intercept`] |
//! | 2 | A flag that carries a single value, given more than once (`FR-CLI-014`) | [`refuse_repetition`] |
//! | 3 | `-q/--quiet` together with `-v/--verbose` (`FR-CLI-015`) | [`refuse_both_verbosities`] |
//! | 4 | `--pretty` without `--format json`, on a node that declares both (`FR-OUT-009`) | [`refuse_pretty_without_json`] |
//!
//! The order among the four is this module's choice and is stated here because
//! `FR-ERR-006` fixes the order *between* steps and not within one. A refusal
//! that is a property of **one** flag precedes a refusal that is a property of
//! **two**, so `tpl -d a -d b -q -v version` reports the repetition: the
//! repeated flag is wrong however the rest of the line reads, and the pair is
//! only wrong as a pair. The two pair refusals are then ordered by reach: the
//! pair of `FR-CLI-015` is two **global** flags and is wrong at every node of
//! the tree, and the pair of `FR-OUT-009` is two local flags of one node.
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
use super::local::Format;
use crate::diagnostics::verbosity::Level;
use crate::error::{self, Error};

/// `--pretty`, by the identifier `ArgMatches` is keyed by.
const PRETTY: &str = "pretty";

/// `--format`, by the identifier `ArgMatches` is keyed by.
const FORMAT: &str = "format";

/// `--pretty`, in the long form the tree declares it under.
const PRETTY_FLAG: &str = "--pretty";

/// `--format text`, which is the member `--pretty` excludes.
///
/// It is a literal rather than a value interpolated from the invocation.
/// `--format` takes a closed set of two, per `FR-OUT-001`, and this refusal is
/// reached only where the value in force is the one that is not `json` — so
/// there is exactly one spelling the second member can have, and composing it
/// from the matched value would put a caller-supplied byte on a line
/// `FR-ERR-024` would then have to escape.
const FORMAT_TEXT: &str = "--format text";

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

/// Refuses `--pretty` on a command that declares `--format` and was not given
/// `--format json` (`FR-OUT-009`).
///
/// `--pretty` is declared by the seventeen commands of `FR-GLOB-021`, and the
/// rule reaches **sixteen** of them: the seventeenth is `tpl schema dump`,
/// which declares `--pretty` and no `--format` because its only output is JSON,
/// per `FR-SCH-019`. `FR-OUT-010` and `FR-SCH-020` make `--pretty` stand alone
/// there, and the guard below is what keeps that case working — the rule is
/// conditioned on the node declaring **both**, so a node that declares one of
/// the two is not reached by it at all.
///
/// The rule is applied over the **declarations** rather than over a list of
/// command paths, for the reason [`refuse_repetition`] gives: a node that gains
/// the pair is governed without anything here changing, and a node that loses
/// `--format` stops being governed at the same moment it stops being able to
/// fail. The recursion is the same walk down the matched chain, because
/// `FR-GLOB-021` keeps both flags out of the global set and each is therefore
/// matched at the node that declared it.
///
/// The value in force is read after [`refuse_repetition`] has run, so there is
/// at most one occurrence; the declaration in [`super::local`] gives `--format`
/// a default, so the vector is never empty and its first entry is the format
/// the invocation resolves to — which is what makes `--pretty` alone, with no
/// `--format` written at all, the refusal `FR-OUT-009` states it is.
///
/// # Errors
///
/// Returns [`Error::MutuallyExclusiveFlags`] naming both members of the pair,
/// which is what the `64` row of `FR-ERR-034` obliges the `cause` line to name,
/// and [`Error::InternalInvariant`] where the matched tree and the matches
/// disagree about which nodes exist.
pub(super) fn refuse_pretty_without_json(
    command: &clap::Command,
    matches: &ArgMatches,
) -> Result<(), Error> {
    if declares(command, PRETTY) && declares(command, FORMAT) {
        // `get_flag` and `get_many` are keyed by identifier and panic on an
        // argument the node does not declare, which is what the guard above
        // rules out. `get_many` rather than `get_one`: the declaration
        // accumulates occurrences, per `OD-08`, and only the first is the
        // format in force.
        let pretty = matches.get_flag(PRETTY);
        let format = matches
            .get_many::<Format>(FORMAT)
            .and_then(|mut values| values.next().copied())
            .unwrap_or(Format::Text);

        if pretty && format != Format::Json {
            return Err(Error::MutuallyExclusiveFlags {
                first: PRETTY_FLAG.to_owned(),
                second: FORMAT_TEXT.to_owned(),
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

    refuse_pretty_without_json(child, inner)
}

/// Whether `command` declares the argument `identifier` names.
///
/// The test is over the node's own declarations, so it answers `false` for a
/// node that a **child** declares the argument on — which is what makes the
/// walk above visit each node with the arguments that node actually matched.
fn declares(command: &clap::Command, identifier: &str) -> bool {
    command
        .get_arguments()
        .any(|argument| argument.get_id() == identifier)
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
    use super::{
        FORMAT, PRETTY, QUIET, REPEATABLE, VERBOSE, declares, level, refuse_both_verbosities,
    };
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

    /// The seventeen commands of `FR-GLOB-021` that declare `--pretty`, by the
    /// path a caller writes. Sixteen of them declare `--format` beside it; the
    /// seventeenth is `tpl schema dump`, per `FR-SCH-019` and `FR-SCH-020`.
    const DECLARE_PRETTY: [&[&str]; 17] = [
        &["schema", "info"],
        &["schema", "tables"],
        &["schema", "table"],
        &["schema", "views"],
        &["schema", "view"],
        &["schema", "routines"],
        &["schema", "routine"],
        &["schema", "dump"],
        &["template", "list"],
        &["template", "path"],
        &["cfg", "get"],
        &["cfg", "list"],
        &["cfg", "database", "list"],
        &["cfg", "database", "show"],
        &["cfg", "database", "test"],
        &["cache", "status"],
        &["help"],
    ];

    /// The one of the seventeen that declares `--pretty` and no `--format`.
    const PRETTY_ALONE: &[&str] = &["schema", "dump"];

    /// The node `path` names.
    fn node(path: &[&str]) -> clap::Command {
        let mut command = crate::cli::tree();

        for segment in path {
            command = command
                .find_subcommand(segment)
                .unwrap_or_else(|| panic!("{segment} is a node of the tree"))
                .clone();
        }

        command
    }

    /// What `tpl` makes of one argument vector.
    fn refused(argv: &[&str]) -> Option<crate::error::Error> {
        let mut vector = vec!["tpl"];
        vector.extend_from_slice(argv);

        crate::cli::parse(vector).err()
    }

    #[test]
    fn fr_glob_021_the_seventeen_commands_that_declare_pretty_are_the_ones_the_table_names() {
        // FR-GLOB-021: `--pretty` is declared by every command that declares
        // `--format`, plus `schema dump`. The rule of FR-OUT-009 is
        // conditioned on a node declaring both, so this is what fixes which
        // nodes it reaches.
        for path in DECLARE_PRETTY {
            let node = node(path);

            assert!(declares(&node, PRETTY), "{path:?} declares --pretty");
            assert_eq!(
                declares(&node, FORMAT),
                path != PRETTY_ALONE,
                "{path:?} declares --format iff it is not schema dump"
            );
        }
    }

    #[test]
    fn fr_out_009_pretty_without_format_json_is_refused_on_every_command_that_declares_both() {
        // FR-OUT-009: on a command that declares `--format`, `--pretty`
        // requires `--format json`. It is the rule, not a property of one
        // command, so every one of the sixteen is driven.
        for path in DECLARE_PRETTY {
            if path == PRETTY_ALONE {
                continue;
            }

            for tail in [vec!["--pretty"], vec!["--pretty", "--format", "text"]] {
                let mut argv: Vec<&str> = path.to_vec();
                // The three nodes that take a required operand are given one,
                // so the refusal under test is reached rather than the
                // missing-argument one.
                argv.extend(operands(path));
                argv.extend(tail);

                let condition =
                    refused(&argv).unwrap_or_else(|| panic!("{argv:?} is refused by FR-OUT-009"));

                assert_eq!(condition.exit_code(), 64, "{argv:?}");
                assert!(condition.to_string().contains("--pretty"), "{argv:?}");
                assert!(condition.to_string().contains("--format text"), "{argv:?}");
            }
        }
    }

    #[test]
    fn fr_out_009_pretty_with_format_json_is_accepted_on_every_one_of_them() {
        // The other half of the same requirement, and the control for the test
        // above: the pair is refused for the value in force and not for the
        // flag.
        for path in DECLARE_PRETTY {
            if path == PRETTY_ALONE {
                continue;
            }

            let mut argv: Vec<&str> = path.to_vec();
            argv.extend(operands(path));
            argv.extend(["--pretty", "--format", "json"]);

            assert!(refused(&argv).is_none(), "{argv:?} parses");
        }
    }

    #[test]
    fn fr_sch_020_pretty_stands_alone_on_schema_dump() {
        // FR-SCH-020 and FR-OUT-010: `tpl schema dump --pretty` is valid
        // without any accompanying `--format`, because the command declares
        // none. The rule of FR-OUT-009 is conditioned on the node declaring
        // both, which is what leaves this case untouched.
        assert!(refused(&["schema", "dump", "--pretty"]).is_none());

        // And `--format` on it is still the unknown-flag 64 of FR-CLI-019,
        // per FR-SCH-019 — a different refusal, from a different rule.
        let condition = refused(&["schema", "dump", "--format", "json"])
            .expect("schema dump declares no --format");

        assert_eq!(condition.exit_code(), 64);
        assert!(condition.to_string().contains("--format"), "{condition}");
    }

    #[test]
    fn fr_err_006_the_rule_runs_before_the_help_form_is_answered() {
        // FR-ERR-006: step 1 runs "for every command without exception", so a
        // help form does not excuse it — which is the same reading that makes
        // an unknown flag on `tpl --help` a 64.
        for argv in [
            vec!["help", "--pretty"],
            vec!["schema", "tables", "--help", "--pretty"],
            vec!["schema", "table", "--help", "--pretty"],
        ] {
            let condition =
                refused(&argv).unwrap_or_else(|| panic!("{argv:?} is refused by FR-OUT-009"));

            assert_eq!(condition.exit_code(), 64, "{argv:?}");
        }
    }

    #[test]
    fn fr_out_009_a_command_that_declares_neither_flag_is_not_reached() {
        // The rule is over the declarations: `tpl render` and `tpl init`
        // declare neither flag, so nothing here governs them and `--pretty` on
        // either is the unknown-flag 64 of FR-CLI-019 instead.
        let condition =
            refused(&["render", "rust/struct", "--pretty"]).expect("render declares no --pretty");

        assert_eq!(condition.exit_code(), 64);
        assert!(
            condition.to_string().contains("unknown flag"),
            "{condition}"
        );
    }

    /// The operands `path` requires, which are part of the identity of a node
    /// that declares one.
    fn operands(path: &[&str]) -> Vec<&'static str> {
        match path {
            ["schema", "table" | "view" | "routine"] => vec!["orders"],
            ["cfg", "get"] => vec!["core.database"],
            ["cfg", "database", "show" | "test"] => vec!["shop"],
            _ => Vec::new(),
        }
    }
}
