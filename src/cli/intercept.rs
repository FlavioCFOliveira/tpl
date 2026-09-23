//! What a `clap::Error` becomes, per `OD-08`.
//!
//! The parser's own renderer is never invoked, so **no byte it composes reaches
//! a caller**: `FR-ERR-008` fixes four labelled lines and `FR-ERR-033` makes
//! them the whole of what a failure is reported as. This module reads the
//! refusal as typed API — an `ErrorKind`, and the contexts `error-context`
//! populates — and produces the [`Error`] the renderer of `diagnostics/`
//! composes those four lines from.
//!
//! | `ErrorKind` | Becomes | Reached by |
//! |---|---|---|
//! | `InvalidSubcommand` | [`Error::UnknownCommand`], with the nearest matches of `FR-CLI-003` | `tpl sch tables` |
//! | `UnknownArgument` | [`Error::UnknownFlag`], [`Error::SeparateTokenValue`] or [`Error::UnexpectedArgument`] | `tpl schema tables -x`, `tpl -d -x …`, `tpl version foo` |
//! | `MissingRequiredArgument` | [`Error::MissingArgument`] | `tpl schema table` |
//! | `InvalidValue` | [`Error::ValueOutsideEnumeration`], [`Error::SeparateTokenValue`] or [`Error::FlagValueMissing`] | `tpl schema tables --format xml`, `tpl --timeout` |
//! | `ValueValidation` | [`Error::MalformedValue`] | `tpl --timeout soon` |
//! | `ArgumentConflict` | [`Error::RepeatedFlag`] or [`Error::MutuallyExclusiveFlags`] | no invocation of this tree, since `FR-CLI-025`; the arm stays because `ErrorKind` is `#[non_exhaustive]` |
//! | anything else | [`Error::InvocationRejected`] | a value that is not valid UTF-8 |
//!
//! **Every row of the table is `64`**, and so is the last: `ErrorKind` and
//! `ContextKind` are both `#[non_exhaustive]`, so the two matches carry a
//! wildcard arm by force of the language, and `OD-08` fixes what that arm
//! produces — a `64` whose `cause` names the token and states that the
//! invocation was rejected. No refusal the parser can invent moves a caller
//! onto a different branch.
//!
//! # What the argument vector is read for
//!
//! Three of the conditions above are not decided by the refusal alone, and the
//! vector answers each by **position**, never by re-parsing it:
//!
//! - `FR-CLI-018` asks whether a rejected token was written as the separate
//!   value of a flag. The parser reports `-x` in `tpl -d -x` as an unknown
//!   argument, exactly as it reports `-x` in `tpl schema tables -x`; what
//!   separates them is the token before it. The vector also carries the value
//!   **as written**, which the refusal does not: a single-dash token is a
//!   cluster of short flags to the parser, reported at its first element alone
//!   — `-foo` as `-f` — and the corrected form `FR-CLI-018` obliges has to show
//!   the caller what they wrote.
//! - `FR-CLI-017` makes every token after `--` a positional argument, so a
//!   token rejected there is an unexpected argument and never an unknown flag.
//! - The `cause` and `hint` lines of a missing argument name the node the
//!   invocation reached, and the nearest-match populations of `FR-ERR-021` are
//!   that node's children and that node's flags. [`reached`] walks the tree
//!   over the leading tokens to find it.
//!
//! None of the three decides a **code**: each is `64` either way, and a walk
//! that stops early degrades a message rather than an outcome. That is the
//! whole of what this module reads the vector for, and `FR-GLOB-018` is
//! untouched: one rejected token is not the argument vector it bars, and it is
//! `FR-ERR-034` row `64` that requires the token to be named.

use std::borrow::Cow;
use std::collections::BTreeSet;
use std::ffi::OsString;

use clap::error::{ContextKind, ContextValue, ErrorKind};

use crate::diagnostics::suggest::{self, Population};
use crate::error::Error;

/// The argument terminator of `FR-CLI-017`.
const TERMINATOR: &str = "--";

/// The `cause` line's phrase for a value whose expected type this tree does not
/// name.
///
/// Every flag whose value is parsed into something other than a string or a
/// path is named by [`expected`], and a test walks the tree to prove it, so
/// this phrase is a total function's floor rather than a message anyone reads.
const UNNAMED_TYPE: &str = "a value of the type its declaration fixes";

/// The node an invocation had reached, and the path that names it.
struct Reached<'a> {
    /// The deepest node the leading tokens of the vector name.
    node: &'a clap::Command,
    /// That node's path, in canonical spelling and without the program name.
    /// Empty at the root.
    path: String,
}

/// The [`Error`] a refusal of the parser is reported to the caller as.
///
/// `tree` is the built tree the refusal came from and `argv` the vector it was
/// given, `argv[0]` included.
pub(super) fn intercepted(refused: &clap::Error, tree: &clap::Command, argv: &[OsString]) -> Error {
    // The vector is read for position alone, so a token that is not UTF-8 is
    // compared and reported as it renders; the parse has already refused it by
    // the time any such token reaches a message.
    let written: Vec<Cow<'_, str>> = argv.iter().map(|token| token.to_string_lossy()).collect();
    let reached = reached(tree, &written);

    match refused.kind() {
        // FR-CLI-003: the token, and the nearest matches among the children of
        // the node it was written at. BR-CLI-001 routes a mistyped alias here
        // too, which is why an alias is a candidate beside a canonical name.
        ErrorKind::InvalidSubcommand => match one(refused, ContextKind::InvalidSubcommand) {
            Some(token) => {
                let nearest = nearest_command(reached.node, &token);
                Error::UnknownCommand {
                    token,
                    node: reached.path.clone(),
                    nearest,
                }
            }
            None => rejected(refused, &reached),
        },

        ErrorKind::UnknownArgument => match one(refused, ContextKind::InvalidArg) {
            Some(token) => unknown_argument(tree, &reached, &written, token),
            None => rejected(refused, &reached),
        },

        ErrorKind::MissingRequiredArgument => match one(refused, ContextKind::InvalidArg) {
            Some(argument) => Error::MissingArgument {
                command: reached.path,
                argument,
            },
            None => rejected(refused, &reached),
        },

        ErrorKind::InvalidValue => match one(refused, ContextKind::InvalidArg) {
            Some(argument) => invalid_value(tree, &reached, &written, refused, &argument),
            None => rejected(refused, &reached),
        },

        ErrorKind::ValueValidation => match one(refused, ContextKind::InvalidArg) {
            Some(argument) => {
                let flag = named(&argument);
                Error::MalformedValue {
                    value: one(refused, ContextKind::InvalidValue).unwrap_or_default(),
                    expected: expected(&flag),
                    command: declaring(tree, &reached, &flag),
                    parameter: flag,
                }
            }
            None => rejected(refused, &reached),
        },

        ErrorKind::ArgumentConflict => conflict(refused, &reached),

        // OD-08's wildcard arm, which `ErrorKind` being `#[non_exhaustive]`
        // obliges. It produces `64` and never another code.
        _ => rejected(refused, &reached),
    }
}

/// A token the parser did not accept as a flag of the node it was written at.
///
/// Three conditions wear one `ErrorKind`, and the vector separates them:
/// `FR-CLI-017` makes a token after `--` a positional argument, `FR-CLI-018`
/// makes a token written after a value-carrying flag that flag's value, and
/// what remains is the unknown flag of `FR-CLI-019`.
fn unknown_argument(
    tree: &clap::Command,
    reached: &Reached<'_>,
    written: &[Cow<'_, str>],
    token: String,
) -> Error {
    if !token.starts_with('-') || after_the_terminator(written, &token) {
        return Error::UnexpectedArgument {
            command: reached.path.clone(),
            token,
        };
    }

    match flag_before(tree, reached.node, written, &token) {
        Some((flag, value)) => Error::SeparateTokenValue { flag, value },
        None => {
            let nearest = nearest_flag(tree, reached.node, &token);
            Error::UnknownFlag {
                token,
                command: reached.path.clone(),
                positional: reached.node.get_arguments().any(clap::Arg::is_positional),
                nearest,
            }
        }
    }
}

/// A value the parser did not accept for the flag it was written after.
///
/// An empty value is the parser saying that the flag received none, which
/// `FR-CLI-018` governs where the token that followed it begins with `-`.
fn invalid_value(
    tree: &clap::Command,
    reached: &Reached<'_>,
    written: &[Cow<'_, str>],
    refused: &clap::Error,
    argument: &str,
) -> Error {
    let flag = named(argument);
    let value = one(refused, ContextKind::InvalidValue).unwrap_or_default();

    if value.is_empty() {
        return match value_after(tree, reached.node, written, &flag) {
            Some(value) => Error::SeparateTokenValue { flag, value },
            None => Error::FlagValueMissing { flag },
        };
    }

    let permitted = many(refused, ContextKind::ValidValue);
    let command = declaring(tree, reached, &flag);
    if permitted.is_empty() {
        Error::MalformedValue {
            value,
            expected: expected(&flag),
            command,
            parameter: flag,
        }
    } else {
        Error::ValueOutsideEnumeration {
            flag,
            command,
            value,
            permitted,
        }
    }
}

/// A refusal the parser reports as a conflict between two arguments.
///
/// The tree declares no conflict of its own — `FR-CLI-015`, `FR-OUT-009`,
/// `FR-RND-005`, `FR-CFG-016` and `FR-CFG-029` are each refused away from the
/// parser — so the pair the parser names is one argument twice: a flag that
/// carries no value, given more than once. The two-argument reading is kept
/// because the kind admits it and costs a line.
fn conflict(refused: &clap::Error, reached: &Reached<'_>) -> Error {
    match (
        one(refused, ContextKind::InvalidArg),
        one(refused, ContextKind::PriorArg),
    ) {
        (Some(first), Some(second)) if first == second => Error::RepeatedFlag {
            flag: named(&first),
        },
        (Some(first), Some(second)) => Error::MutuallyExclusiveFlags {
            first: named(&first),
            second: named(&second),
        },
        _ => rejected(refused, reached),
    }
}

/// The wildcard outcome of `OD-08`: a `64` naming the token, where the parser
/// named one, and what the parser refused, in plain words.
///
/// `FR-ERR-034` bars a `cause` that would fit any failure, so the kind the
/// parser reported is translated into the fact it stands for rather than
/// reported as "rejected". The parser's own message is not carried: it is
/// composed for its own renderer, which `OD-07` never invokes.
fn rejected(refused: &clap::Error, reached: &Reached<'_>) -> Error {
    Error::InvocationRejected {
        reason: reason(refused.kind()),
        command: reached.path.clone(),
        token: one(refused, ContextKind::InvalidArg)
            .or_else(|| one(refused, ContextKind::InvalidSubcommand))
            .or_else(|| one(refused, ContextKind::InvalidValue)),
    }
}

/// What a kind of refusal means, in the words a `cause` line uses.
fn reason(kind: ErrorKind) -> &'static str {
    match kind {
        ErrorKind::InvalidUtf8 => "a token is not valid UTF-8, and tpl reads text arguments only",
        ErrorKind::TooManyValues => "more values were given than the argument takes",
        ErrorKind::TooFewValues | ErrorKind::WrongNumberOfValues => {
            "the argument was given the wrong number of values"
        }
        ErrorKind::NoEquals => "the flag takes its value joined by '='",
        ErrorKind::MissingSubcommand => "a subcommand is required here",
        ErrorKind::InvalidSubcommand => "no such command at this position",
        ErrorKind::UnknownArgument => "the token is not an argument this command takes",
        ErrorKind::InvalidValue | ErrorKind::ValueValidation => {
            "the value is not one the argument accepts"
        }
        ErrorKind::MissingRequiredArgument => "a required argument is missing",
        ErrorKind::ArgumentConflict => "two arguments cannot be given together",
        _ => "the parser refused it for a reason tpl does not name",
    }
}

/// The command path whose help states what `flag` takes: empty for a global
/// flag, and the node the invocation reached for any other.
fn declaring(tree: &clap::Command, reached: &Reached<'_>, flag: &str) -> String {
    let long = flag.strip_prefix(TERMINATOR).unwrap_or(flag);
    let global = tree
        .get_arguments()
        .any(|argument| argument.get_long() == Some(long));

    if global {
        String::new()
    } else {
        reached.path.clone()
    }
}

/// One value of `kind`, where the refusal carries one in a shape this crate
/// reads.
///
/// `ContextValue` is `#[non_exhaustive]`, so the match carries a wildcard arm
/// by force of the language. A shape this crate does not read yields no token,
/// and the caller degrades to [`rejected`] rather than inventing one.
fn one(refused: &clap::Error, kind: ContextKind) -> Option<String> {
    match refused.get(kind) {
        Some(ContextValue::String(value)) => Some(value.clone()),
        Some(ContextValue::Strings(values)) => values.first().cloned(),
        _ => None,
    }
}

/// Every value of `kind`, in the order the refusal carries them.
fn many(refused: &clap::Error, kind: ContextKind) -> Vec<String> {
    match refused.get(kind) {
        Some(ContextValue::String(value)) => vec![value.clone()],
        Some(ContextValue::Strings(values)) => values.clone(),
        _ => Vec::new(),
    }
}

/// The argument as the caller names it, without the value name the parser
/// appends to it.
///
/// The contexts spell an option `--format <FORMAT>` and a positional `<NAME>`.
/// The first word is the flag, and a positional has none — it survives as its
/// own value name, which no runnable command is built from because
/// [`crate::diagnostics`] admits no such spelling into one.
fn named(argument: &str) -> String {
    argument
        .split_once(' ')
        .map_or(argument, |(flag, _)| flag)
        .to_owned()
}

/// The type a flag's value is parsed into, in the words the `64` row of
/// `FR-ERR-034` obliges the `cause` line to carry beside the value.
///
/// Only a flag whose value is parsed into something other than a string or a
/// path can reach this: a string accepts every byte the shell delivers. The two
/// that can are named, and a test walks the tree feeding every flag a value no
/// parser accepts, so a third added later fails that test rather than reaching
/// [`UNNAMED_TYPE`].
fn expected(flag: &str) -> &'static str {
    match flag {
        // FR-GLOB-011: a positive integer number of seconds.
        "--timeout" => "a whole number of seconds, greater than zero",
        // FR-CONF-002: the port of a `[database.<name>]` entry.
        "--port" => "a whole number from 1 to 65535",
        _ => UNNAMED_TYPE,
    }
}

/// The deepest node the leading tokens of `written` name, and its path.
///
/// The walk descends on an exact child name or alias, which is the tree's own
/// rule — `FR-CLI-004` infers no command from a prefix — and steps over a flag,
/// and over the token that carries its value where the two were written apart.
/// It stops at the first token that is neither, which is the token the parser
/// refused or the first operand.
///
/// The path is spelled canonically, so an alias names the node it resolves to,
/// per `FR-HELP-027`.
fn reached<'a>(tree: &'a clap::Command, written: &[Cow<'_, str>]) -> Reached<'a> {
    let mut node = tree;
    let mut path = String::new();
    let mut index = 1;

    while let Some(token) = written.get(index) {
        if token == TERMINATOR {
            break;
        }

        if token.starts_with('-') {
            let next = written.get(index + 1);

            index += 1 + usize::from(carries_the_next(tree, node, token, next));
            continue;
        }

        let Some(child) = node.find_subcommand(token.as_ref()) else {
            break;
        };

        if !path.is_empty() {
            path.push(' ');
        }
        path.push_str(child.get_name());
        node = child;
        index += 1;
    }

    Reached { node, path }
}

/// Whether the flag `token` names takes its value from the token after it,
/// which `next` is where the vector carries one.
///
/// The argument terminator is never a value. `FR-CLI-017` makes `--` end the
/// flags, and the parser reads it as the terminator rather than as the value of
/// the flag it follows — `tpl -d --` is refused for a flag given without a
/// value, not accepted with `--` as the value of `-d`. A walk that stepped over
/// it would continue past the end of the flags and resolve a **positional**
/// argument as a command, so the `cause` line would name a node the invocation
/// never reached and the `hint` would be a runnable command for that node.
fn carries_the_next(
    tree: &clap::Command,
    node: &clap::Command,
    token: &str,
    next: Option<&Cow<'_, str>>,
) -> bool {
    next.is_some_and(|next| next != TERMINATOR)
        && !token.contains('=')
        && declared(tree, node, token).is_some_and(|argument| argument.get_action().takes_values())
}

/// The argument `token` names — the one that would take a value written after
/// it — at the node reached or among the seven globals the root declares.
///
/// The root is consulted beside the node because a global argument is
/// propagated into a subcommand when that subcommand is built, and the parser
/// builds the tree only as far as the invocation reached.
///
/// A token of one `-` may be a **cluster** of short flags, and the parser reads
/// one left to right: the first element that carries a value takes the rest of
/// the token as that value, and takes the next token only when no rest remains.
/// The argument a cluster names is therefore its **last** element, and only
/// when every element before it carries no value. `-vd` names `--database`,
/// exactly as `-d` does; `-dv` is `-d` with the value `v` and names nothing
/// that a following token could belong to; `-vq` names `--quiet`, which carries
/// no value, and the caller of this function is what refuses it.
///
/// A single-element token is the same rule with no earlier element, so `-d` and
/// `-x` resolve exactly as they did before clusters were read at all.
fn declared<'a>(
    tree: &'a clap::Command,
    node: &'a clap::Command,
    token: &str,
) -> Option<&'a clap::Arg> {
    if let Some(long) = token.strip_prefix(TERMINATOR) {
        if long.is_empty() {
            return None;
        }

        return by_long(tree, node, long);
    }

    let mut elements = token.strip_prefix('-')?.chars();
    let last = elements.next_back()?;

    for element in elements {
        // An element that carries a value consumes the rest of the token, so
        // nothing after the token belongs to this cluster; an element the tree
        // does not declare makes the token no cluster of this tree's.
        if by_short(tree, node, element)?.get_action().takes_values() {
            return None;
        }
    }

    by_short(tree, node, last)
}

/// The argument declared under the long form `long`.
fn by_long<'a>(
    tree: &'a clap::Command,
    node: &'a clap::Command,
    long: &str,
) -> Option<&'a clap::Arg> {
    node.get_arguments()
        .chain(tree.get_arguments())
        .find(|argument| argument.get_long() == Some(long))
}

/// The argument declared under the short form `short`.
fn by_short<'a>(
    tree: &'a clap::Command,
    node: &'a clap::Command,
    short: char,
) -> Option<&'a clap::Arg> {
    node.get_arguments()
        .chain(tree.get_arguments())
        .find(|argument| argument.get_short() == Some(short))
}

/// The long spelling of an argument, which is what a message names it by.
fn long_form(argument: &clap::Arg, written: &str) -> String {
    argument
        .get_long()
        .map_or_else(|| written.to_owned(), |long| format!("--{long}"))
}

/// The flag `token` was written as the separate value of, and the value **as
/// the caller wrote it**, where `FR-CLI-018` governs the pair.
///
/// The reported token and the written one are not always the same string. A
/// token of a single `-` is a cluster of short flags to the parser, which
/// refuses it at its **first** element and reports that element alone: a caller
/// who wrote `-foo` is told about `-f`, and one who wrote `-42` about `-4`. A
/// long token is reported whole, and so is a short one that is a whole cluster
/// by itself.
///
/// The lookback therefore matches a written token that the reported one
/// **heads**, per [`heads`], and the value it returns is that written token —
/// the hint of `FR-CLI-018` shows the caller the line they wrote, corrected,
/// and `--pattern=-f` would correct it into something else.
fn flag_before(
    tree: &clap::Command,
    node: &clap::Command,
    written: &[Cow<'_, str>],
    token: &str,
) -> Option<(String, String)> {
    let index = written
        .iter()
        .position(|candidate| candidate == token || heads(token, candidate))?;
    let value = written.get(index)?.clone().into_owned();
    let previous = written.get(index.checked_sub(1)?)?;

    if previous.contains('=') {
        return None;
    }

    let argument = declared(tree, node, previous)?;

    // The guard that keeps the rule off a genuine cluster: `--pretty -foo`
    // reports `-f` exactly as `--pattern -foo` does, and what separates them is
    // that one flag carries a value and the other carries none.
    argument
        .get_action()
        .takes_values()
        .then(|| (long_form(argument, previous), value))
}

/// Whether `written` is the single-dash token the short form `token` heads.
///
/// The test is deliberately narrow, and the narrowing is the point: it applies
/// only where the parser reports a **short** form, which is the only shape it
/// decomposes, and only over a token of one `-`, which is the only shape it
/// decomposes into. `--nope` is reported as `--nope` and matches by equality;
/// `-foo` is reported as `-f` and matches here.
///
/// Rejected: matching any written token that begins with the reported one,
/// whatever either looks like. It would make `--pattern` the head of a written
/// `--patternx`, which is a different flag and not a decomposition of this one,
/// and it would put a token that was never rejected into the message.
fn heads(token: &str, written: &str) -> bool {
    short(token) && !written.starts_with(TERMINATOR) && written.starts_with(token)
}

/// Whether `token` is a short form: one `-`, and one character after it.
fn short(token: &str) -> bool {
    token.starts_with('-') && !token.starts_with(TERMINATOR) && token.chars().count() == 2
}

/// The token written after `flag`, where it begins with `-` and is therefore
/// the separate-token value `FR-CLI-018` refuses.
///
/// The flag is sought in **every** spelling that names it — its long form, its
/// short form, and a cluster ending in that short form — because the parser
/// names it by its long form however the caller wrote it. It is sought at its
/// **last** occurrence, because that is the one the parser was reading when it
/// ran out of tokens.
fn value_after(
    tree: &clap::Command,
    node: &clap::Command,
    written: &[Cow<'_, str>],
    flag: &str,
) -> Option<String> {
    let argument = declared(tree, node, flag)?;

    let index = written.iter().rposition(|candidate| {
        declared(tree, node, candidate).is_some_and(|named| named.get_id() == argument.get_id())
    })?;
    let value = written.get(index + 1)?;

    (value.starts_with('-') && value != TERMINATOR).then(|| value.clone().into_owned())
}

/// Whether `token` was written after the argument terminator, which
/// `FR-CLI-017` makes a positional argument of it.
///
/// The refusal names the token and not the place it was written at, so the
/// place is recovered from the vector. The parser reads left to right and
/// refuses the **first** token it cannot accept, so the occurrence it refused
/// is the first one and the question is whether **that** occurrence lies after
/// the terminator.
///
/// Rejected: asking whether the token occurs anywhere after the terminator. A
/// token written on both sides of it — `tpl schema tables -x -- -x` — was
/// refused at the occurrence **before** it, where it is the unknown flag of
/// `FR-CLI-019`, and answering from the later occurrence reported it as the
/// unexpected argument of `FR-CLI-017` instead.
fn after_the_terminator(written: &[Cow<'_, str>], token: &str) -> bool {
    let Some(terminator) = written.iter().position(|candidate| candidate == TERMINATOR) else {
        return false;
    };

    written
        .iter()
        .position(|candidate| candidate == token)
        .is_some_and(|refused| refused > terminator)
}

/// The nearest matches to `token` among the children of `node`, per
/// `FR-ERR-019`.
///
/// The population is the node's children **and their aliases**, because
/// `BR-CLI-001` routes a mistyped alias through this rule rather than through
/// inference, and `tbl` and `tbls` differ by one character.
fn nearest_command(node: &clap::Command, token: &str) -> Vec<String> {
    let population: Vec<&str> = node
        .get_subcommands()
        .flat_map(|child| {
            std::iter::once(child.get_name()).chain(child.get_all_aliases().map(str::trim))
        })
        .collect();

    kept(token, population.iter().copied(), Population::Commands)
}

/// The nearest matches to `token` among the flags the invocation could have
/// written at `node`, per `FR-ERR-019`.
///
/// The population is that node's own flags together with the seven globals of
/// `FR-GLOB-001`, in both spellings a caller may write: the long form every
/// flag carries, and the five short forms `FR-GLOB-024` closes the space at.
/// The set is deduplicated because the root is both the node and the home of
/// the globals when the refusal happened there.
fn nearest_flag(tree: &clap::Command, node: &clap::Command, token: &str) -> Vec<String> {
    let mut population: BTreeSet<String> = BTreeSet::new();

    // A one-letter token is one edit from every other one-letter flag, so
    // offering them is noise: `-5` would suggest `-V`, `-d` and `-h`. Only
    // the long forms are offered for it.
    let offer_short = !short(token);

    for argument in node.get_arguments().chain(tree.get_arguments()) {
        if let Some(long) = argument.get_long() {
            population.insert(format!("--{long}"));
        }
        if let Some(short) = argument.get_short().filter(|_| offer_short) {
            population.insert(format!("-{short}"));
        }
    }

    kept(
        token,
        population.iter().map(String::as_str),
        Population::Flags,
    )
}

/// The candidates the selection of `FR-ERR-019` kept, owned so that they travel
/// on the error value to the line [`crate::diagnostics`] composes.
fn kept<'a>(
    supplied: &str,
    population: impl IntoIterator<Item = &'a str>,
    class: Population,
) -> Vec<String> {
    suggest::suggestions(supplied, population, class)
        .names()
        .map(str::to_owned)
        .collect()
}

#[cfg(test)]
mod tests {
    use std::ffi::OsString;
    use std::os::unix::ffi::OsStringExt;

    use clap::error::ErrorKind;

    use crate::cli::{parse, parse_from};
    use crate::diagnostics::rendered;
    use crate::error::Error;

    /// The four labels of `FR-ERR-008`, in the order it fixes.
    const LABELS: [&str; 4] = ["error: ", "cause: ", "hint:  ", "exit:  "];

    /// Two spellings `clap`'s own renderer composes and this one never does.
    ///
    /// They are what a caller would see if a byte of that renderer reached a
    /// stream, which `OD-08` and `FR-ERR-033` forbid outright.
    const CLAP_RENDERER: [&str; 4] = [
        "Usage:",
        "For more information",
        "tip:",
        "--help' for more information",
    ];

    /// The invocation `vector` names, refused.
    ///
    /// The kind is asserted beside the mapping so that the pair is a fact about
    /// this tree rather than about `clap`: a version that reclassified one of
    /// these refusals would fail here, where it is mapped, rather than silently
    /// producing the wildcard's message.
    fn refused(vector: &[&str], kind: ErrorKind) -> Error {
        assert_eq!(
            parse_from(vector.iter().copied())
                .expect_err("the parser refuses this invocation")
                .kind(),
            kind,
            "{vector:?}"
        );

        parse(vector.iter().copied()).expect_err("the invocation is refused")
    }

    /// The four labelled lines the caller reads, as one block, checked against
    /// the shape `FR-ERR-008` fixes.
    fn four_lines(error: &Error) -> Vec<String> {
        let block = rendered(error);
        let lines: Vec<String> = block.lines().map(str::to_owned).collect();

        assert_eq!(lines.len(), 4, "{block}");
        for (line, label) in lines.iter().zip(LABELS) {
            assert!(line.starts_with(label), "{line:?} does not carry {label:?}");
        }
        for composed in CLAP_RENDERER {
            assert!(!block.contains(composed), "{block}");
        }

        lines
    }

    /// The content of one labelled line, without its label.
    fn line(lines: &[String], label: &str) -> String {
        let found = lines
            .iter()
            .find(|line| line.starts_with(label))
            .unwrap_or_else(|| panic!("no line carries {label:?}"));

        found[label.len()..].to_owned()
    }

    #[test]
    fn fr_cli_003_an_unknown_subcommand_names_the_token_and_the_nearest_matches() {
        // FR-CLI-003 and BR-CLI-001: the nearest-match rule resolves a
        // mistyped alias, `tbl` and `tbls` differing by one character.
        let error = refused(&["tpl", "schema", "tbles"], ErrorKind::InvalidSubcommand);
        let lines = four_lines(&error);

        assert_eq!(error.exit_code(), 64);
        assert_eq!(
            line(&lines, "error: "),
            "unknown command 'tbles' under 'tpl schema'"
        );
        assert!(line(&lines, "cause: ").contains("'tbles'"));
        assert_eq!(
            line(&lines, "hint:  "),
            "did you mean 'tables', 'tbls' or 'table'? list what it takes with: tpl help schema"
        );
        assert_eq!(line(&lines, "exit:  "), "64 (EX_USAGE)");
    }

    #[test]
    fn fr_cli_019_an_unknown_flag_names_the_token_and_the_nearest_matches() {
        // FR-CLI-019: a command rejects every flag it does not declare, and
        // FR-ERR-021 applies the nearest-match rule to a flag name.
        let error = refused(
            &["tpl", "schema", "tables", "--pattrn", "x"],
            ErrorKind::UnknownArgument,
        );
        let lines = four_lines(&error);

        assert_eq!(error.exit_code(), 64);
        assert_eq!(line(&lines, "error: "), "unknown flag '--pattrn'");
        assert!(line(&lines, "cause: ").contains("'--pattrn'"));
        assert_eq!(
            line(&lines, "hint:  "),
            "did you mean '--pattern'? list the flags of this command with: tpl help schema tables"
        );
        assert_eq!(line(&lines, "exit:  "), "64 (EX_USAGE)");
    }

    #[test]
    fn fr_cli_017_a_token_the_command_takes_no_argument_for_is_named_as_written() {
        // The same ErrorKind, and not a flag: `foo` names none, and FR-CLI-017
        // has already made `-d` a positional argument by the time it is
        // refused.
        for (vector, token, command) in [
            (&["tpl", "version", "foo"][..], "foo", "version"),
            (&["tpl", "render", "x", "--", "-d"][..], "-d", "render"),
        ] {
            let error = refused(vector, ErrorKind::UnknownArgument);
            let lines = four_lines(&error);

            assert_eq!(error.exit_code(), 64);
            assert_eq!(
                line(&lines, "error: "),
                format!("unexpected argument '{token}'")
            );
            assert!(line(&lines, "cause: ").contains(&format!("'tpl {command}'")));
            assert_eq!(
                line(&lines, "hint:  "),
                format!("show what it takes with: tpl help {command}")
            );
        }
    }

    #[test]
    fn fr_cli_018_a_separate_token_value_beginning_with_a_dash_shows_the_corrected_form() {
        // FR-CLI-018, over both of the kinds the parser reports it as: an
        // unknown flag where the value names none, and an empty value where it
        // names a flag the node declares.
        for (vector, kind, flag, value) in [
            (
                &["tpl", "-d", "-x", "schema", "tables"][..],
                ErrorKind::UnknownArgument,
                "--database",
                "-x",
            ),
            (
                &["tpl", "schema", "tables", "--pattern", "--format", "json"][..],
                ErrorKind::InvalidValue,
                "--pattern",
                "--format",
            ),
        ] {
            let error = refused(vector, kind);
            let lines = four_lines(&error);

            assert_eq!(error.exit_code(), 64);
            assert_eq!(
                line(&lines, "error: "),
                format!("'{value}' was read as a flag rather than as the value of '{flag}'")
            );
            assert!(line(&lines, "cause: ").contains("token of its own"));
            assert_eq!(
                line(&lines, "hint:  "),
                format!("write the value in one token: {flag}={value}")
            );
            assert_eq!(line(&lines, "exit:  "), "64 (EX_USAGE)");
        }
    }

    #[test]
    fn fr_cli_018_a_separate_token_value_shows_the_corrected_form_whatever_shape_it_has() {
        // FR-CLI-018 draws no line at a value's length or shape, and neither
        // does this: the parser decomposes a single-dash token of more than one
        // character into short flags and reports only its head, so `-foo`
        // arrives as `-f` and `-42` as `-4`. The written token is what the
        // corrected form has to show.
        for (path, operands, flag, value) in [
            // The two shapes the parser reports whole.
            (&["schema", "tables"][..], &[][..], "--pattern", "-f"),
            (&["schema", "tables"][..], &[][..], "--pattern", "-5"),
            // The class the parser reports by its head alone.
            (&["schema", "tables"][..], &[][..], "--pattern", "-foo"),
            (&["schema", "tables"][..], &[][..], "--pattern", "-42"),
            (&[][..], &[][..], "--tpl-dir", "-abc"),
            (
                &["cfg", "database", "add"][..],
                &["shop"][..],
                "--host",
                "-dbhost",
            ),
            // A long token that names no flag of the tree.
            (&["schema", "tables"][..], &[][..], "--pattern", "--nope"),
        ] {
            let mut vector = vec!["tpl"];
            vector.extend_from_slice(path);
            vector.extend_from_slice(operands);
            vector.extend_from_slice(&[flag, value]);

            let error = parse(vector.iter().copied()).expect_err("the value is refused");
            let lines = four_lines(&error);

            assert_eq!(error.exit_code(), 64, "{vector:?}");
            assert_eq!(
                line(&lines, "error: "),
                format!("'{value}' was read as a flag rather than as the value of '{flag}'"),
                "{vector:?}"
            );
            assert_eq!(
                line(&lines, "hint:  "),
                format!("write the value in one token: {flag}={value}"),
                "{vector:?}"
            );
        }
    }

    #[test]
    fn fr_cli_018_a_short_flag_written_in_a_cluster_refuses_a_value_exactly_as_it_does_alone() {
        // FR-CLI-018 conditions on the **value**, not on how the flag that
        // takes it was written. `-d` is the one short form of FR-GLOB-024 that
        // carries a value, so the shapes are the clusters that end in it, and
        // each must read as `-d` does: the two invocations are the same one
        // written two ways.
        let alone = parse(["tpl", "-d", "-x", "schema", "tables"]).expect_err("refused");
        let alone = four_lines(&alone);

        for cluster in ["-vd", "-qd", "-vvd", "-hd", "-Vd"] {
            let error = parse(["tpl", cluster, "-x", "schema", "tables"]).expect_err("refused");
            let clustered = four_lines(&error);

            assert_eq!(error.exit_code(), 64, "{cluster}");
            assert_eq!(
                line(&clustered, "error: "),
                line(&alone, "error: "),
                "{cluster} and -d differ in the error line"
            );
            assert_eq!(
                line(&clustered, "hint:  "),
                line(&alone, "hint:  "),
                "{cluster} and -d differ in the hint line"
            );
            assert_eq!(clustered, alone, "{cluster} and -d differ");
        }

        // The class of values reaches a clustered flag whole, and so does the
        // other kind of refusal the parser reports it as: a value that names a
        // flag the node declares arrives as an empty value rather than as an
        // unknown argument.
        for (vector, value) in [
            (&["tpl", "-vvd", "-foo", "schema", "tables"][..], "-foo"),
            (
                &["tpl", "schema", "tables", "-vd", "--format", "json"][..],
                "--format",
            ),
        ] {
            let error = parse(vector.iter().copied()).expect_err("refused");
            let lines = four_lines(&error);

            assert_eq!(
                line(&lines, "error: "),
                format!("'{value}' was read as a flag rather than as the value of '--database'"),
                "{vector:?}"
            );
            assert_eq!(
                line(&lines, "hint:  "),
                format!("write the value in one token: --database={value}"),
                "{vector:?}"
            );
        }
    }

    #[test]
    fn fr_cli_018_a_cluster_that_carries_its_value_is_still_the_invocation_it_was() {
        // The control the rule above needs on the other side: reading a cluster
        // must not make one refuse a value it accepted. Both forms the parser
        // admits — the value in the next token, and the value attached to the
        // cluster — parse, and reach the flags they name.
        for vector in [
            &["tpl", "-vd", "shop", "version"][..],
            &["tpl", "-vdshop", "version"][..],
        ] {
            let invocation =
                parse(vector.iter().copied()).unwrap_or_else(|error| panic!("{vector:?}: {error}"));

            assert_eq!(invocation.globals.database, ["shop"], "{vector:?}");
            assert_eq!(invocation.globals.verbose, 1, "{vector:?}");
        }
    }

    #[test]
    fn fr_err_022_a_value_the_character_set_refuses_still_shows_the_corrected_form() {
        // FR-ERR-022 governs what may be written into the line and FR-ERR-023
        // drops a value outside the set, so a path-shaped value leaves the form
        // standing with its placeholder — which is still the correction
        // FR-CLI-018 obliges the hint to show.
        for (vector, flag) in [
            (&["tpl", "--tpl-dir", "-/srv/x", "version"][..], "--tpl-dir"),
            (
                &[
                    "tpl",
                    "cfg",
                    "database",
                    "add",
                    "shop",
                    "--host",
                    "-db.example.com",
                ][..],
                "--host",
            ),
        ] {
            let error = parse(vector.iter().copied()).expect_err("refused");
            let lines = four_lines(&error);

            assert_eq!(error.exit_code(), 64, "{vector:?}");
            assert_eq!(
                line(&lines, "hint:  "),
                format!("write the value in one token: {flag}=<value>"),
                "{vector:?}"
            );
        }
    }

    #[test]
    fn fr_cli_018_a_cluster_written_after_a_flag_that_carries_no_value_stays_an_unknown_flag() {
        // The control the rule above needs: `--pretty -foo` is reported exactly
        // as `--pattern -foo` is, and what separates them is that one flag
        // carries a value and the other carries none. A rule that read the
        // token alone would turn every mistyped short flag into a value.
        for (trailing, token) in [
            (&["--pretty", "-foo"][..], "-f"),
            (&["--no-cache", "-abc"][..], "-a"),
            (&["-q", "-foo"][..], "-f"),
            // And the same over a cluster: what decides is its **last**
            // element, which carries no value in either of these.
            (&["-qv", "-foo"][..], "-f"),
            (&["-vq", "-abc"][..], "-a"),
        ] {
            let mut vector = vec!["tpl", "schema", "tables"];
            vector.extend_from_slice(trailing);

            let error = parse(vector.iter().copied()).expect_err("no such flag");
            let lines = four_lines(&error);

            assert_eq!(error.exit_code(), 64, "{vector:?}");
            assert_eq!(
                line(&lines, "error: "),
                format!("unknown flag '{token}'"),
                "{vector:?}"
            );
        }
    }

    #[test]
    fn fr_err_034_a_flag_given_without_any_value_says_so_rather_than_naming_a_value() {
        let error = refused(&["tpl", "--timeout"], ErrorKind::InvalidValue);
        let lines = four_lines(&error);

        assert_eq!(error.exit_code(), 64);
        assert_eq!(
            line(&lines, "error: "),
            "the flag '--timeout' was given without a value"
        );
        assert_eq!(
            line(&lines, "cause: "),
            "'--timeout' carries one value and the invocation supplied none for it"
        );
        assert_eq!(
            line(&lines, "hint:  "),
            "give the value after the flag, or in one token: --timeout=<value>"
        );
    }

    #[test]
    fn fr_err_034_a_value_outside_an_enumeration_names_the_value_and_the_values_accepted() {
        // FR-ERR-034, the `64` row: the value that did not conform, together
        // with the type expected — which for an enumerated flag is the set of
        // spellings it declares.
        let error = refused(
            &["tpl", "schema", "tables", "--format", "xml"],
            ErrorKind::InvalidValue,
        );
        let lines = four_lines(&error);

        assert_eq!(error.exit_code(), 64);
        assert_eq!(
            line(&lines, "error: "),
            "'xml' is not a value '--format' accepts"
        );
        assert_eq!(
            line(&lines, "cause: "),
            "'xml' was supplied for '--format', which takes text or json"
        );
    }

    #[test]
    fn fr_err_034_a_value_of_the_wrong_type_names_the_value_and_the_type_expected() {
        for (vector, flag, value, expected) in [
            (
                &["tpl", "--timeout", "soon", "version"][..],
                "--timeout",
                "soon",
                "a whole number of seconds, greater than zero",
            ),
            (
                &["tpl", "cfg", "database", "add", "shop", "--port", "abc"][..],
                "--port",
                "abc",
                "a whole number from 1 to 65535",
            ),
        ] {
            let error = refused(vector, ErrorKind::ValueValidation);
            let lines = four_lines(&error);

            assert_eq!(error.exit_code(), 64);
            assert_eq!(
                line(&lines, "error: "),
                format!("'{value}' is not a valid value for '{flag}'")
            );
            assert_eq!(
                line(&lines, "cause: "),
                format!("'{value}' was supplied for '{flag}', which takes {expected}")
            );
        }
    }

    #[test]
    fn fr_err_034_a_missing_required_argument_names_the_command_and_the_argument() {
        let error = refused(
            &["tpl", "cfg", "set", "core.database"],
            ErrorKind::MissingRequiredArgument,
        );
        let lines = four_lines(&error);

        assert_eq!(error.exit_code(), 64);
        assert_eq!(
            line(&lines, "error: "),
            "'tpl cfg set' needs the argument <VALUE>"
        );
        assert_eq!(
            line(&lines, "hint:  "),
            "show the usage with: tpl help cfg set"
        );
    }

    #[test]
    fn fr_cli_025_a_flag_that_carries_no_value_reaches_this_module_at_all_no_longer() {
        // FR-CLI-025 makes the repetition idempotent, so the parser raises no
        // `ArgumentConflict` for it and there is nothing here to classify. It
        // used to: `tpl -q -q version` was the one invocation of this tree that
        // reached the arm, and the `64` it produced is the outcome that
        // requirement rejects. The arm itself stays, because `ErrorKind` is
        // `#[non_exhaustive]` and this crate may not assume the set is closed.
        //
        // The six flags the requirement names are driven in `cli::rules`; what
        // is asserted here is that the parser accepts what this module would
        // otherwise have had to reject.
        assert!(parse_from(["tpl", "-q", "-q", "version"].into_iter()).is_ok());
        assert!(parse(["tpl", "-q", "-q", "version"].into_iter()).is_ok());
    }

    #[test]
    fn fr_err_034_a_refusal_this_crate_does_not_classify_is_a_sixty_four_naming_what_it_can() {
        // OD-08's wildcard arm, reached by a value that is not valid UTF-8:
        // `clap` reports `InvalidUtf8` and populates no context this crate
        // reads, so no token can be named and the arm says so.
        let vector: Vec<OsString> = vec![
            OsString::from("tpl"),
            OsString::from("schema"),
            OsString::from("table"),
            OsString::from_vec(vec![0x6f, 0xff, 0x73]),
        ];

        assert_eq!(
            parse_from(vector.clone())
                .expect_err("not valid UTF-8")
                .kind(),
            ErrorKind::InvalidUtf8
        );

        let error = parse(vector).expect_err("not valid UTF-8");
        let lines = four_lines(&error);

        assert_eq!(error.exit_code(), 64, "the wildcard arm produces 64");
        assert_eq!(
            line(&lines, "error: "),
            "the invocation was rejected: a token is not valid UTF-8, and tpl reads text \
             arguments only"
        );
        assert_eq!(line(&lines, "exit:  "), "64 (EX_USAGE)");

        // And the arm names the token wherever the refusal carries one.
        let reached = super::Reached {
            node: &crate::cli::tree(),
            path: String::new(),
        };
        let named = super::rejected(&clap::Error::raw(ErrorKind::Io, "unreachable"), &reached);
        assert_eq!(named.exit_code(), 64);
        assert!(matches!(
            named,
            Error::InvocationRejected { token: None, .. }
        ));
    }

    #[test]
    fn fr_err_033_no_byte_the_parsers_own_renderer_composes_reaches_the_caller() {
        // OD-08 and FR-ERR-033: the parser keeps `error-context`, which is read
        // as typed API, and its renderer is never invoked. The comparison is
        // made against what that renderer would have written for the same
        // refusal, line by line: nothing it composes survives into the four
        // labelled lines, and the two agree on nothing but the word `error`.
        for vector in [
            &["tpl", "schema", "tbles"][..],
            &["tpl", "schema", "tables", "--pattrn", "x"][..],
            &["tpl", "schema", "tables", "--format", "xml"][..],
            &["tpl", "cfg", "set", "core.database"][..],
            // `tpl -q -q version` stood here until FR-CLI-025 made it an
            // invocation the parser accepts; it refuses nothing now, so there
            // is no refusal of it to compare two renderings of.
            &["tpl", "--timeout", "soon", "version"][..],
            &["tpl", "-d", "-x", "schema", "tables"][..],
            &["tpl", "version", "foo"][..],
        ] {
            let composed = parse_from(vector.iter().copied())
                .expect_err("the parser refuses this invocation")
                .to_string();
            let ours = rendered(&parse(vector.iter().copied()).expect_err("refused"));

            for line in composed.lines().map(str::trim) {
                // A line of one or two words is a token rather than a
                // composition — `<VALUE>` is the argument this tree declares,
                // and both messages name it because the context carries it.
                // What this asserts is that nothing the parser **composed**
                // survives.
                if line.split_whitespace().count() < 3 {
                    continue;
                }

                assert!(
                    !ours.contains(line),
                    "{vector:?}: the parser's own line {line:?} reached the caller"
                );
            }
        }
    }

    #[test]
    fn fr_err_034_every_flag_whose_value_is_parsed_names_the_type_it_expects() {
        // `expected` is a total function with a floor nobody should read, and
        // this is what keeps the floor unreachable: every flag of the tree is
        // given a value no parser of a number accepts, at the node that
        // declares it. A flag that refuses the value must be named by
        // `expected`, or carry an enumeration of its own, which the cause line
        // names instead.
        let tree = crate::cli::tree();
        let mut refused_by_type = 0;
        walk(&tree, &[], &mut |vector, flag| {
            let Err(error) = parse(vector.iter().map(String::as_str)) else {
                return;
            };

            match error {
                Error::MalformedValue { expected, .. } => {
                    refused_by_type += 1;
                    assert_ne!(
                        expected,
                        super::UNNAMED_TYPE,
                        "{flag} is parsed into a type no cause line can name"
                    );
                }
                Error::ValueOutsideEnumeration { .. } => refused_by_type += 1,
                other => panic!("{vector:?} was refused as {other}"),
            }
        });

        assert!(
            refused_by_type >= 3,
            "the tree declares --timeout, --port and two enumerated flags; \
             {refused_by_type} refused the value"
        );
    }

    /// Applies `visitor` to an invocation of every flag of the tree that
    /// carries a value, at the node that declares it.
    ///
    /// The vector is the node's path, one placeholder for each argument the
    /// node requires, and the flag with a value no number accepts.
    fn walk(command: &clap::Command, path: &[String], visitor: &mut impl FnMut(&[String], &str)) {
        for argument in command.get_arguments() {
            let Some(long) = argument.get_long() else {
                continue;
            };
            if !argument.get_action().takes_values() {
                continue;
            }

            let mut vector = vec!["tpl".to_owned()];
            vector.extend_from_slice(path);
            vector.extend(
                command
                    .get_positionals()
                    .filter(|positional| positional.is_required_set())
                    .map(|_| "x".to_owned()),
            );
            vector.push(format!("--{long}"));
            vector.push("soon".to_owned());

            visitor(&vector, &format!("--{long}"));
        }

        for child in command.get_subcommands() {
            let mut deeper = path.to_vec();
            deeper.push(child.get_name().to_owned());
            walk(child, &deeper, visitor);
        }
    }
}
