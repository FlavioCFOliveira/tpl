//! The caller's own command, written back with one correction.
//!
//! A `hint` that corrects a command the caller ran has to reproduce the whole
//! of it: a global flag, a `--context` document, a `--set` value and every
//! other flag given, changing only the faulty token (finding T-01 of the third
//! re-audit of rmp `#263`). A hint that wrote the command path alone and the
//! corrected token read `core.database` where the caller had named `-d shop`,
//! or the server where the caller had named a `--context` document, and could
//! exit `0` with output from the wrong source when copied verbatim.
//!
//! The argument vector is recorded once, by [`record`], before it is parsed.
//! [`restated`] walks it against the command tree, classifying each token as
//! a literal of `FR-ERR-022` — a command, an alias, a flag — or as a value, and
//! tests every value by the character set that governs it. A value its set
//! refuses is written as a placeholder, and [`Restated::replacing`] says in
//! words what each placeholder stands for, so nothing the caller gave is
//! dropped in silence and nothing refused reaches the line.
//!
//! Where nothing was recorded — a unit test that renders an [`Error`] it built
//! itself — [`restated`] answers [`None`] and the caller of this module keeps
//! the command path it already carries.
//!
//! [`Error`]: crate::error::Error

use std::borrow::Cow;
use std::ffi::OsString;
use std::sync::OnceLock;

use super::hint::{admits, admits_key, admits_template};

/// The argument vector of this process, `argv[0]` included, as it was parsed.
static RECORDED: OnceLock<Vec<OsString>> = OnceLock::new();

/// Records the argument vector the process is about to parse.
///
/// The first call wins; a second is ignored, because one process parses one
/// vector.
pub(crate) fn record(argv: &[OsString]) {
    let _ = RECORDED.set(argv.to_vec());
}

/// A correction to apply to the recorded command.
#[derive(Debug, Clone, Copy)]
pub(super) enum Edit<'a> {
    /// Write `to` in place of the value of the argument whose identifier is
    /// one of `ids` — a flag's value or a positional operand.
    Value {
        /// The clap identifiers the value may be carried under.
        ids: &'a [&'a str],
        /// What is written in its place.
        to: Replacement<'a>,
    },
    /// Make the format JSON: the value of `--format` becomes `json`, or
    /// `--format json` is written before the first `--pretty`.
    FormatJson,
    /// Append these literal words at the end of the command.
    Append(&'a [&'a str]),
    /// Remove the flag whose value is carried under one of these clap
    /// identifiers, with its value: the `-d shop` a node that takes no `-d`
    /// was given (finding V-08 of the fifth re-audit of rmp `#263`).
    Remove(&'a [&'a str]),
    /// Write the command word `to` in place of the command word `from`: the
    /// caller's `cfg database add`, whole, as the `update` that changes the
    /// entry it could not create (finding U-07 of the fourth re-audit of rmp
    /// `#263`). Both words are literals of `FR-ERR-022`.
    Command {
        /// The command word written.
        from: &'a str,
        /// The command word written in its place.
        to: &'a str,
    },
}

/// What an [`Edit::Value`] writes.
#[derive(Debug, Clone, Copy)]
pub(super) enum Replacement<'a> {
    /// A value the hint has already tested and admits.
    Literal(&'a str),
    /// A placeholder, and the words that say what it stands for.
    Placeholder {
        /// The placeholder, angle brackets included.
        text: &'a str,
        /// What the caller writes in its place.
        meaning: &'a str,
    },
}

/// The recorded command with the correction applied.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct Restated {
    /// The whole command, beginning with `tpl`.
    pub(super) command: String,
    /// Each placeholder written into it, with what it stands for.
    pub(super) placeholders: Vec<(String, String)>,
}

impl Restated {
    /// The words that say what each placeholder stands for, as a clause that
    /// follows the command: `; replace <context> with the value you gave
    /// --context`. Empty where the command carries no placeholder.
    pub(super) fn replacing(&self) -> String {
        let mut clause = String::new();
        for (index, (placeholder, meaning)) in self.placeholders.iter().enumerate() {
            clause.push_str(match index {
                0 => "; replace ",
                _ if index + 1 == self.placeholders.len() => ", and ",
                _ => ", ",
            });
            clause.push_str(placeholder);
            clause.push_str(" with ");
            clause.push_str(meaning);
        }
        clause
    }
}

/// The recorded command with `edits` applied, or [`None`] where no vector was
/// recorded, a token is not valid UTF-8 where a literal was expected, or an
/// edit found nothing to act on.
pub(super) fn restated(edits: &[Edit<'_>]) -> Option<Restated> {
    let argv = RECORDED.get()?;
    let words: Vec<Option<&str>> = argv.iter().skip(1).map(|word| word.to_str()).collect();
    restate(&crate::cli::tree(), &words, edits)
}

/// The node an invocation names when every flag is set aside wherever it is
/// written, and the index in `words` just after the word that names it.
///
/// `words` is the vector without `argv[0]`. A flag the path reached so far
/// does not declare is stepped over, with the next word as its value where
/// some node on or below the path declares it as taking one and that word
/// names no child: `tpl --format json schema tables` names `schema tables`,
/// although the parser refused `--format` at the root (finding Y-02 of the
/// eighth re-audit of rmp `#263`).
pub(crate) fn destination<'t>(
    tree: &'t clap::Command,
    words: &[Option<&str>],
) -> (Vec<&'t clap::Command>, usize) {
    let mut path: Vec<&clap::Command> = vec![tree];
    let mut end = 0_usize;
    let mut index = 0_usize;

    while let Some(&Some(text)) = words.get(index) {
        if text == "--" {
            break;
        }
        let node = path.last().copied().unwrap_or(tree);
        index += 1;
        if text.len() > 1 && text.starts_with('-') {
            let next = words.get(index).copied().flatten();
            let consumes = next
                .is_some_and(|next| !next.starts_with('-') && node.find_subcommand(next).is_none())
                && takes_the_next(&path, text);
            index += usize::from(consumes);
            continue;
        }
        let Some(child) = node.find_subcommand(text) else {
            break;
        };
        path.push(child);
        end = index;
    }

    (path, end)
}

/// Whether the flag `text` takes the next word as its value, where a node of
/// `path`, or a node below its last, declares it.
fn takes_the_next(path: &[&clap::Command], text: &str) -> bool {
    fn below(node: &clap::Command, matches: &impl Fn(&clap::Arg) -> bool) -> bool {
        node.get_arguments()
            .any(|argument| matches(argument) && takes_value(argument))
            || node.get_subcommands().any(|child| below(child, matches))
    }
    let anywhere = |matches: &dyn Fn(&clap::Arg) -> bool| {
        path.iter().any(|node| {
            node.get_arguments()
                .any(|argument| matches(argument) && takes_value(argument))
        }) || path
            .last()
            .is_some_and(|node| below(node, &|argument| matches(argument)))
    };

    if text.contains('=') {
        return false;
    }
    if let Some(long) = text.strip_prefix("--") {
        return anywhere(&|argument| argument.get_long() == Some(long));
    }
    // A cluster: the first element that takes a value takes the rest of the
    // word, and the next word only where it is the last element.
    let cluster = &text[1..];
    for (at, short) in cluster.char_indices() {
        if anywhere(&|argument| argument.get_short() == Some(short)) {
            return at + short.len_utf8() == cluster.len();
        }
    }
    false
}

/// The recorded command with every flag written before the command that
/// declares it moved to just after that command's path, each with its value
/// (finding Y-02 of the eighth re-audit of rmp `#263`); [`None`] where no
/// vector was recorded or the restatement fails.
pub(super) fn relocated() -> Option<Restated> {
    let argv = RECORDED.get()?;
    let words: Vec<Option<&str>> = argv.iter().skip(1).map(|word| word.to_str()).collect();
    let tree = crate::cli::tree();
    let reordered = relocate(&tree, &words)?;
    restate(&tree, &reordered, &[])
}

/// The words of [`relocated`], reordered.
fn relocate<'w>(tree: &clap::Command, words: &[Option<&'w str>]) -> Option<Vec<Option<&'w str>>> {
    let (target, end) = destination(tree, words);
    let mut path: Vec<&clap::Command> = vec![tree];
    let mut kept: Vec<Option<&str>> = Vec::with_capacity(words.len());
    let mut moved: Vec<Option<&str>> = Vec::new();
    let mut index = 0_usize;

    while index < end {
        let text = words[index]?;
        index += 1;
        if text.len() > 1 && text.starts_with('-') {
            let name = text.split('=').next().unwrap_or(text);
            let here = if let Some(long) = name.strip_prefix("--") {
                declared(&path, |argument| argument.get_long() == Some(long)).is_some()
            } else {
                let short = name[1..].chars().next()?;
                declared(&path, |argument| argument.get_short() == Some(short)).is_some()
            };
            let next = words.get(index).copied().flatten();
            let consumes = next.is_some_and(|next| {
                !next.starts_with('-')
                    && path
                        .last()
                        .is_some_and(|node| node.find_subcommand(next).is_none())
            }) && takes_the_next(&path, text);
            let bucket = if here { &mut kept } else { &mut moved };
            bucket.push(Some(text));
            if consumes {
                bucket.push(words[index]);
                index += 1;
            }
            continue;
        }
        let child = path.last()?.find_subcommand(text)?;
        path.push(child);
        kept.push(Some(text));
    }

    if moved.is_empty() || path.len() != target.len() {
        return None;
    }
    kept.extend(moved);
    kept.extend_from_slice(&words[end..]);
    Some(kept)
}

/// The recorded command with the one argument `token` written as its words,
/// each its own argument, and every other argument and flag kept (finding
/// Y-03 of the eighth re-audit of rmp `#263`); [`None`] where no vector was
/// recorded, `token` is not in it, or the restatement fails.
pub(super) fn split(token: &str) -> Option<Restated> {
    let argv = RECORDED.get()?;
    let words: Vec<Option<&str>> = argv.iter().skip(1).map(|word| word.to_str()).collect();
    let at = words.iter().position(|word| *word == Some(token))?;
    let mut spread: Vec<Option<&str>> = Vec::with_capacity(words.len() + 4);
    spread.extend_from_slice(&words[..at]);
    spread.extend(token.split_whitespace().map(Some));
    spread.extend_from_slice(&words[at + 1..]);
    restate(&crate::cli::tree(), &spread, &[])
}

/// Whether the recorded vector holds a word after `token`.
pub(super) fn follows(token: &str) -> bool {
    RECORDED.get().is_some_and(|argv| {
        argv.iter()
            .skip(1)
            .position(|word| word.to_str() == Some(token))
            .is_some_and(|at| argv.len() > at + 2)
    })
}

/// The command path of the recorded invocation and whether it carried an
/// operand.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct Shape {
    /// The nodes from `tpl` to the one reached, as their names are spelt in
    /// the tree, separated by spaces and without `tpl`.
    pub(super) path: String,
    /// Whether any positional operand was given to the node reached.
    pub(super) operand: bool,
}

/// The [`Shape`] of the recorded invocation, or [`None`] where no vector was
/// recorded or a flag it holds is not one the tree declares.
///
/// `FR-PROJ-027` writes the command path back where the invocation carried no
/// operand, and says in words to run it again where it carried one.
pub(super) fn shape() -> Option<Shape> {
    let argv = RECORDED.get()?;
    let words: Vec<Option<&str>> = argv.iter().skip(1).map(|word| word.to_str()).collect();
    shaped(&crate::cli::tree(), &words)
}

/// The walk behind [`shape`], over a vector without `argv[0]`.
fn shaped(tree: &clap::Command, words: &[Option<&str>]) -> Option<Shape> {
    let mut path: Vec<&clap::Command> = vec![tree];
    let mut operand = false;
    let mut words = words.iter().copied();

    while let Some(word) = words.next() {
        let Some(text) = word else {
            operand = true;
            continue;
        };
        if text == "--" {
            operand |= words.next().is_some();
            break;
        }
        if let Some(long) = text.strip_prefix("--").filter(|long| !long.is_empty()) {
            let (name, attached) = match long.split_once('=') {
                Some((name, value)) => (name, Some(value)),
                None => (long, None),
            };
            let argument = declared(&path, |argument| argument.get_long() == Some(name))?;
            if takes_value(argument) && attached.is_none() {
                words.next();
            }
            continue;
        }
        if let Some(cluster) = text.strip_prefix('-').filter(|cluster| !cluster.is_empty()) {
            for (at, short) in cluster.char_indices() {
                let argument = declared(&path, |argument| argument.get_short() == Some(short))?;
                if takes_value(argument) {
                    if cluster[at + short.len_utf8()..].is_empty() {
                        words.next();
                    }
                    break;
                }
            }
            continue;
        }
        let node = *path.last()?;
        match node.find_subcommand(text) {
            Some(child) if !operand => path.push(child),
            _ => operand = true,
        }
    }

    Some(Shape {
        path: path
            .iter()
            .skip(1)
            .map(|node| node.get_name())
            .collect::<Vec<_>>()
            .join(" "),
        operand,
    })
}

/// Which of the two flags of `FR-ERR-043` a hint may carry: `false` where the
/// hint exists to change or to remove that flag, and so carries what it
/// proposes instead.
#[derive(Debug, Clone, Copy)]
pub(super) struct Carry {
    /// Whether `--tpl-dir` may be carried.
    pub(super) tpl_dir: bool,
    /// Whether `-d/--database` may be carried.
    pub(super) database: bool,
}

/// A value of one of the two flags, as it may be written into a command.
#[derive(Debug, Clone, PartialEq, Eq)]
struct Given {
    /// The value, or the placeholder that stands for it.
    text: String,
    /// The placeholder and what it stands for, where the value's set refused
    /// the value.
    placeholder: Option<(&'static str, &'static str)>,
}

/// The `--tpl-dir` placeholder and its meaning; the same words [`valued`]
/// gives the flag, so that a restated command and a carried one agree.
const TPL_DIR_PLACEHOLDER: (&str, &str) = ("<tpl-dir>", "the value you gave --tpl-dir");

/// The `-d/--database` placeholder and its meaning.
const DATABASE_PLACEHOLDER: (&str, &str) = ("<database>", "the value you gave --database");

/// The `--tpl-dir` and the `-d/--database` the caller wrote, each tested by
/// the set that governs it.
#[derive(Debug, Default, PartialEq, Eq)]
struct Globals {
    tpl_dir: Option<Given>,
    database: Option<Given>,
}

/// `hint` with every runnable `tpl` command it writes carrying the caller's
/// `--tpl-dir` and `-d/--database`, immediately after `tpl` and in that order
/// (`FR-ERR-043`).
///
/// A flag is carried only where the caller wrote it — a `.tpl` found by
/// discovery, or an entry selected by `core.database`, is never in the
/// argument vector — and
/// only onto a command on which it has an effect, per the table of
/// `BR-GLOB-001`; a command the hint already writes with the flag keeps the
/// value the hint proposes, and one on which the flag has no effect loses it.
/// A value its set refuses is a placeholder, and a clause at the end of the
/// line says what the placeholder stands for.
///
/// Where no vector was recorded, or the caller wrote neither flag, `hint` is
/// returned unchanged.
pub(super) fn carried(hint: Cow<'static, str>, carry: Carry) -> Cow<'static, str> {
    let Some(argv) = RECORDED.get() else {
        return hint;
    };
    let words: Vec<Option<&str>> = argv.iter().skip(1).map(|word| word.to_str()).collect();
    let tree = crate::cli::tree();
    let given = globals(&tree, &words);
    carry_into(&tree, &hint, &given, carry).map_or(hint, Cow::Owned)
}

/// The two flags as the caller wrote them in `words`, the vector without
/// `argv[0]`, up to the first `--`.
///
/// The scan does not need the whole vector to parse: a hint is written for an
/// invocation the parser may have refused. A token that is the value of
/// another flag is skipped, so that value is never read as one of the two.
fn globals(tree: &clap::Command, words: &[Option<&str>]) -> Globals {
    let mut found = Globals::default();
    let mut words = words.iter().copied();

    while let Some(word) = words.next() {
        let Some(text) = word else { continue };
        if text == "--" {
            break;
        }

        if let Some(long) = text.strip_prefix("--") {
            let (name, attached) = match long.split_once('=') {
                Some((name, value)) => (name, Some(value)),
                None => (long, None),
            };
            match name {
                "tpl-dir" => {
                    let value = attached.or_else(|| words.next().flatten());
                    found.tpl_dir.get_or_insert_with(|| tpl_dir_given(value));
                }
                "database" => {
                    let value = attached.or_else(|| words.next().flatten());
                    found.database.get_or_insert_with(|| database_given(value));
                }
                _ if attached.is_none()
                    && takes_value_anywhere(tree, |a| a.get_long() == Some(name)) =>
                {
                    words.next();
                }
                _ => {}
            }
            continue;
        }

        let Some(cluster) = text.strip_prefix('-').filter(|cluster| !cluster.is_empty()) else {
            continue;
        };
        for (at, short) in cluster.char_indices() {
            let known = |argument: &clap::Arg| argument.get_short() == Some(short);
            if !takes_value_anywhere(tree, known) {
                if any_argument(tree, &known) {
                    continue;
                }
                break;
            }
            let rest = &cluster[at + short.len_utf8()..];
            let rest = rest.strip_prefix('=').unwrap_or(rest);
            let value = if rest.is_empty() {
                words.next().flatten()
            } else {
                Some(rest)
            };
            if short == 'd' {
                found.database.get_or_insert_with(|| database_given(value));
            }
            break;
        }
    }

    found
}

/// The `-d/--database` value the caller wrote, where the set of `FR-ERR-022`
/// admits it.
pub(super) fn database() -> Option<String> {
    let argv = RECORDED.get()?;
    let words: Vec<Option<&str>> = argv.iter().skip(1).map(|word| word.to_str()).collect();
    let given = globals(&crate::cli::tree(), &words).database?;

    given.placeholder.is_none().then_some(given.text)
}

/// The `--tpl-dir` the caller wrote on the command line: [`None`] where it
/// wrote none, `Some(None)` where the set of `FR-ERR-041` refuses the value,
/// and `Some(Some(value))` otherwise. A value from `TPL_DIR` is never in the
/// vector, so it is never returned.
pub(super) fn tpl_dir() -> Option<Option<String>> {
    let argv = RECORDED.get()?;
    let words: Vec<Option<&str>> = argv.iter().skip(1).map(|word| word.to_str()).collect();
    let given = globals(&crate::cli::tree(), &words).tpl_dir?;

    Some(given.placeholder.is_none().then_some(given.text))
}

/// The `--tpl-dir` value, tested by the set of `FR-ERR-041`.
fn tpl_dir_given(value: Option<&str>) -> Given {
    match value {
        Some(path) if admits_template(path) => Given {
            text: path.to_owned(),
            placeholder: None,
        },
        _ => Given {
            text: TPL_DIR_PLACEHOLDER.0.to_owned(),
            placeholder: Some(TPL_DIR_PLACEHOLDER),
        },
    }
}

/// The `-d/--database` value, tested by the set of `FR-ERR-022`.
fn database_given(value: Option<&str>) -> Given {
    match value {
        Some(entry) if admits(entry) => Given {
            text: entry.to_owned(),
            placeholder: None,
        },
        _ => Given {
            text: DATABASE_PLACEHOLDER.0.to_owned(),
            placeholder: Some(DATABASE_PLACEHOLDER),
        },
    }
}

/// Whether some node of `tree` declares an argument `matches` selects.
fn any_argument(tree: &clap::Command, matches: &impl Fn(&clap::Arg) -> bool) -> bool {
    tree.get_arguments().any(matches)
        || tree
            .get_subcommands()
            .any(|child| any_argument(child, matches))
}

/// Whether the argument `matches` selects, wherever the tree declares it,
/// takes a value.
fn takes_value_anywhere(tree: &clap::Command, matches: impl Fn(&clap::Arg) -> bool) -> bool {
    fn walk(node: &clap::Command, matches: &impl Fn(&clap::Arg) -> bool) -> bool {
        node.get_arguments()
            .any(|argument| matches(argument) && takes_value(argument))
            || node.get_subcommands().any(|child| walk(child, matches))
    }
    walk(tree, &matches)
}

/// The words that open a command in a hint and precede `tpl` in prose that
/// only names one: "the tpl render command", "no tpl cfg command runs".
const PROSE: &[&str] = &["the ", "no ", "No "];

/// The line with `given` carried into each command, or [`None`] where no
/// command changed.
fn carry_into(tree: &clap::Command, hint: &str, given: &Globals, carry: Carry) -> Option<String> {
    if given.tpl_dir.is_none() && given.database.is_none() {
        return None;
    }

    let mut line = String::with_capacity(hint.len() + 64);
    let mut placeholders: Vec<(&'static str, &'static str)> = Vec::new();
    let mut changed = false;
    let mut copied = 0_usize;
    let mut searched = 0_usize;

    while let Some(found) = hint[searched..].find("tpl ") {
        let at = searched + found;
        let flags_at = at + "tpl ".len();
        let before = &hint[..at];
        let opens = (before.is_empty() || before.ends_with(' '))
            && !PROSE.iter().any(|word| before.ends_with(word));

        if opens && let Some(rewritten) = rewrite(tree, &hint[flags_at..], given, carry) {
            line.push_str(&hint[copied..flags_at]);
            line.push_str(&rewritten.flags);
            copied = flags_at + rewritten.consumed;
            for placeholder in rewritten.placeholders {
                if !placeholders.contains(&placeholder) {
                    placeholders.push(placeholder);
                }
            }
            changed = true;
        }
        searched = flags_at.max(copied);
    }

    if !changed {
        return None;
    }
    line.push_str(&hint[copied..]);

    // The words for each placeholder carried, unless the line already says
    // them — a restated command names its own.
    let unexplained: Vec<(&str, &str)> = placeholders
        .into_iter()
        .filter(|(placeholder, _)| !line.contains(&format!("{placeholder} with ")))
        .collect();
    for (index, (placeholder, meaning)) in unexplained.iter().enumerate() {
        line.push_str(match index {
            0 => "; replace ",
            _ if index + 1 == unexplained.len() => ", and ",
            _ => ", ",
        });
        line.push_str(placeholder);
        line.push_str(" with ");
        line.push_str(meaning);
    }

    Some(line)
}

/// The leading flags of one command, rewritten under `FR-ERR-043`.
#[derive(Debug)]
struct Rewritten {
    /// The flags to write after `tpl `, each followed by a space.
    flags: String,
    /// How many bytes of the original command the flags replace.
    consumed: usize,
    /// The placeholders written into `flags` from the invocation.
    placeholders: Vec<(&'static str, &'static str)>,
}

/// Which of the two flags a leading flag of a command is.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Leading {
    TplDir,
    Database,
    Other,
}

/// The flags `command` — the text after `tpl ` — should lead with, or
/// [`None`] where it names no command of the tree or needs no change.
fn rewrite(
    tree: &clap::Command,
    command: &str,
    given: &Globals,
    carry: Carry,
) -> Option<Rewritten> {
    // The leading flags, each with its value, and where the node begins.
    let mut leading: Vec<(Leading, &str)> = Vec::new();
    let mut offset = 0_usize;
    loop {
        let rest = &command[offset..];
        let token = rest.split(' ').next().unwrap_or_default();
        if token.len() < 2 || !token.starts_with('-') || token == "--" {
            break;
        }
        let (name, attached) = match token.split_once('=') {
            Some((name, _)) => (name, true),
            None => (token, false),
        };
        let (kind, valued) = match name {
            "--tpl-dir" => (Leading::TplDir, true),
            "-d" | "--database" => (Leading::Database, true),
            _ if name.starts_with("-d") && !name.starts_with("--") => (Leading::Database, false),
            _ => {
                let argument = tree.get_arguments().find(|argument| {
                    name.strip_prefix("--").map_or_else(
                        || name.chars().nth(1) == argument.get_short(),
                        |long| argument.get_long() == Some(long),
                    )
                });
                (Leading::Other, argument.is_some_and(takes_value))
            }
        };
        let mut length = token.len();
        if valued && !attached {
            let value = rest[length..].strip_prefix(' ')?;
            length += 1 + value.split(' ').next().unwrap_or_default().len();
        }
        leading.push((kind, &rest[..length]));
        offset += length;
        offset += usize::from(command[offset..].starts_with(' '));
    }

    // Where the command ends in the hint: at the first clause that follows it.
    let end = command[offset..]
        .find("; ")
        .into_iter()
        .chain(command[offset..].find(", "))
        .min()
        .map_or(command.len(), |end| offset + end);

    // The node the command names, which decides where each flag has effect.
    // The words are read within the command, and the punctuation of the
    // sentence around it is not part of a word: `load,` in "tpl cache load,
    // or …" names `load` (finding AA-01 of the tenth re-audit of rmp `#263`).
    let mut words = command[offset..end]
        .split(' ')
        .map(|word| word.trim_end_matches(|c: char| c.is_ascii_punctuation() && c != '-'));
    let top = tree.find_subcommand(words.next()?)?;
    let child = words
        .next()
        .and_then(|word| top.find_subcommand(word))
        .map(clap::Command::get_name);
    let tpl_dir_applies = !matches!(top.get_name(), "init" | "help" | "version");
    let database_applies = match top.get_name() {
        "schema" => true,
        "render" => !command[offset..end].contains("--context"),
        "cache" => matches!(child, Some("load" | "clean" | "status")),
        _ => false,
    };

    let written = |kind: Leading| {
        leading
            .iter()
            .find(|(held, _)| *held == kind)
            .map(|(_, text)| *text)
    };
    let mut flags = String::new();
    let mut placeholders = Vec::new();
    for (kind, applies, allowed, value, flag) in [
        (
            Leading::TplDir,
            tpl_dir_applies,
            carry.tpl_dir,
            &given.tpl_dir,
            "--tpl-dir",
        ),
        (
            Leading::Database,
            database_applies,
            carry.database,
            &given.database,
            "-d",
        ),
    ] {
        if !applies {
            continue;
        }
        if let Some(text) = written(kind) {
            flags.push_str(text);
            flags.push(' ');
        } else if let (true, Some(value)) = (allowed, value) {
            flags.push_str(flag);
            flags.push(' ');
            flags.push_str(&value.text);
            flags.push(' ');
            placeholders.extend(value.placeholder);
        }
    }
    for (_, text) in leading.iter().filter(|(kind, _)| *kind == Leading::Other) {
        flags.push_str(text);
        flags.push(' ');
    }

    (flags != command[..offset]).then_some(Rewritten {
        flags,
        consumed: offset,
        placeholders,
    })
}

/// One word of the restated command.
#[derive(Debug)]
struct Piece {
    /// The text written, including a `--flag=` it is glued to.
    text: String,
    /// The identifier of the argument this word is the value of, if any.
    value_of: Option<String>,
    /// The flag spelling the word is glued to (`--flag=`), if any.
    glued: Option<String>,
    /// The placeholder the word carries, and what it stands for.
    placeholder: Option<(String, String)>,
}

impl Piece {
    fn literal(text: &str) -> Self {
        Self {
            text: text.to_owned(),
            value_of: None,
            glued: None,
            placeholder: None,
        }
    }
}

/// The walk behind [`restated`], over a vector without `argv[0]`.
fn restate(tree: &clap::Command, words: &[Option<&str>], edits: &[Edit<'_>]) -> Option<Restated> {
    let mut path: Vec<&clap::Command> = vec![tree];
    let mut pieces: Vec<Piece> = Vec::with_capacity(words.len() + 2);
    let mut positional = 0_usize;
    let mut operands_only = false;
    let mut words = words.iter().copied();

    while let Some(word) = words.next() {
        let node = *path.last()?;
        let Some(text) = word else {
            // Not UTF-8: it can only be a value, and it is never written.
            pieces.push(operand(node, &mut positional, None));
            continue;
        };

        if !operands_only && text == "--" {
            operands_only = true;
            pieces.push(Piece::literal(text));
            continue;
        }

        if !operands_only && text.len() > 2 && text.starts_with("--") {
            let (name, attached) = match text[2..].split_once('=') {
                Some((name, value)) => (name, Some(value)),
                None => (&text[2..], None),
            };
            let argument = declared(&path, |argument| argument.get_long() == Some(name))?;
            let flag = format!("--{name}");
            if !takes_value(argument) {
                pieces.push(Piece::literal(&flag));
            } else if let Some(value) = attached {
                let mut piece = valued(argument, Some(value));
                piece.glued = Some(flag);
                pieces.push(piece);
            } else {
                pieces.push(Piece::literal(&flag));
                pieces.push(valued(argument, words.next().flatten()));
            }
            continue;
        }

        if !operands_only && text.len() > 1 && text.starts_with('-') {
            let cluster = &text[1..];
            let mut plain = String::new();
            for (at, short) in cluster.char_indices() {
                let argument = declared(&path, |argument| argument.get_short() == Some(short))?;
                if !takes_value(argument) {
                    plain.push(short);
                    continue;
                }
                if !plain.is_empty() {
                    pieces.push(Piece::literal(&format!("-{plain}")));
                    plain.clear();
                }
                pieces.push(Piece::literal(&format!("-{short}")));
                let rest = &cluster[at + short.len_utf8()..];
                let rest = rest.strip_prefix('=').unwrap_or(rest);
                let value = if rest.is_empty() {
                    words.next().flatten()
                } else {
                    Some(rest)
                };
                pieces.push(valued(argument, value));
                break;
            }
            if !plain.is_empty() {
                pieces.push(Piece::literal(&format!("-{plain}")));
            }
            continue;
        }

        if !operands_only
            && positional == 0
            && let Some(child) = node.find_subcommand(text)
        {
            pieces.push(Piece::literal(text));
            path.push(child);
            continue;
        }

        pieces.push(operand(node, &mut positional, Some(text)));
    }

    for edit in edits {
        apply(&mut pieces, *edit)?;
    }
    hoist(&mut pieces);

    let mut command = String::from("tpl");
    let mut placeholders: Vec<(String, String)> = Vec::new();
    for piece in &pieces {
        command.push(' ');
        if let Some(flag) = &piece.glued {
            command.push_str(flag);
            command.push('=');
        }
        command.push_str(&piece.text);
        if let Some(placeholder) = &piece.placeholder
            && !placeholders.contains(placeholder)
        {
            placeholders.push(placeholder.clone());
        }
    }

    Some(Restated {
        command,
        placeholders,
    })
}

/// Moves `--tpl-dir` and `-d/--database`, each with its value, to the front of
/// the command, in that order, as `FR-ERR-043` writes them.
fn hoist(pieces: &mut Vec<Piece>) {
    let mut front: Vec<Piece> = Vec::with_capacity(4);
    for id in ["tpl_dir", "database"] {
        let Some(at) = pieces
            .iter()
            .position(|piece| piece.value_of.as_deref() == Some(id))
        else {
            continue;
        };
        // A value glued to its flag is one word; otherwise the flag is the
        // word before it.
        let start = if pieces[at].glued.is_some() {
            at
        } else {
            at.saturating_sub(1)
        };
        front.extend(pieces.drain(start..=at));
    }
    pieces.splice(0..0, front);
}

/// Applies one edit, or answers [`None`] where it found nothing to act on.
fn apply(pieces: &mut Vec<Piece>, edit: Edit<'_>) -> Option<()> {
    match edit {
        Edit::Value { ids, to } => {
            let piece = pieces.iter_mut().rev().find(|piece| {
                piece
                    .value_of
                    .as_deref()
                    .is_some_and(|id| ids.contains(&id))
            })?;
            match to {
                Replacement::Literal(text) => {
                    piece.text = text.to_owned();
                    piece.placeholder = None;
                }
                Replacement::Placeholder { text, meaning } => {
                    piece.text = text.to_owned();
                    piece.placeholder = Some((text.to_owned(), meaning.to_owned()));
                }
            }
        }
        Edit::FormatJson => {
            let mut found = false;
            for piece in pieces.iter_mut() {
                if piece.value_of.as_deref() == Some("format") {
                    piece.text = "json".to_owned();
                    piece.placeholder = None;
                    found = true;
                }
            }
            if !found {
                let at = pieces.iter().position(|piece| piece.text == "--pretty")?;
                pieces.insert(at, Piece::literal("json"));
                pieces.insert(at, Piece::literal("--format"));
            }
        }
        Edit::Append(words) => pieces.extend(words.iter().map(|word| Piece::literal(word))),
        Edit::Remove(ids) => {
            let at = pieces.iter().position(|piece| {
                piece
                    .value_of
                    .as_deref()
                    .is_some_and(|id| ids.contains(&id))
            })?;
            // A value glued to its flag is one word; otherwise the flag is the
            // word before it.
            let start = if pieces[at].glued.is_some() {
                at
            } else {
                at.saturating_sub(1)
            };
            pieces.drain(start..=at);
        }
        Edit::Command { from, to } => {
            let piece = pieces.iter_mut().find(|piece| {
                piece.value_of.is_none() && piece.glued.is_none() && piece.text == from
            })?;
            to.clone_into(&mut piece.text);
        }
    }
    Some(())
}

/// The flag of `path` — the node reached, or one of its ancestors, whose
/// global flags reach it — that `matches` selects.
fn declared<'t>(
    path: &[&'t clap::Command],
    matches: impl Fn(&clap::Arg) -> bool,
) -> Option<&'t clap::Arg> {
    path.iter()
        .rev()
        .find_map(|node| node.get_arguments().find(|argument| matches(argument)))
}

/// Whether a flag takes a value, rather than being counted or set.
fn takes_value(argument: &clap::Arg) -> bool {
    argument.get_action().takes_values()
}

/// The next positional operand of `node`.
fn operand(node: &clap::Command, positional: &mut usize, text: Option<&str>) -> Piece {
    let positionals: Vec<&clap::Arg> = node.get_positionals().collect();
    let argument = positionals
        .get(*positional)
        .or_else(|| positionals.last())
        .copied();
    *positional += 1;
    match argument {
        Some(argument) => valued(argument, text),
        None => placeholder(None, "<value>".to_owned(), "the value you gave".to_owned()),
    }
}

/// The value `text` of `argument`, tested by the set that governs it.
fn valued(argument: &clap::Arg, text: Option<&str>) -> Piece {
    let id = argument.get_id().as_str();
    let name = argument.get_long().map_or_else(
        || {
            argument
                .get_value_names()
                .and_then(|names| names.first())
                .map_or_else(|| id.to_owned(), |name| name.to_lowercase())
        },
        str::to_owned,
    );
    let meaning = if argument.is_positional() {
        format!("the {name} you gave")
    } else {
        format!("the value you gave --{name}")
    };
    let refused = || placeholder(Some(id), format!("<{name}>"), meaning.clone());

    let Some(text) = text else {
        return refused();
    };

    // FR-CONF-050: an empty host or database is refused, so "the value you
    // gave" would stand for nothing; the placeholder says what to write.
    match id {
        "host" if crate::project::config::keys::is_blank(text) => {
            return placeholder(
                Some(id),
                "<host>".to_owned(),
                "the host of the server".to_owned(),
            );
        }
        "schema" if crate::project::config::keys::is_blank(text) => {
            return placeholder(
                Some(id),
                "<database>".to_owned(),
                "the name of the database on the server".to_owned(),
            );
        }
        _ => {}
    }

    let admitted = match id {
        // BR-ERR-003 bars a DSN from every message, and a value of `cfg set`
        // may be a password; neither is written back whatever it holds.
        "dsn" | "value" => false,
        // FR-ERR-041 governs a template name and a path of the project.
        "template" | "tpl_dir" | "context" => admits_template(text),
        // A key of FR-CONF-002 is a literal, tested segment by segment.
        "key" => admits_key(text),
        // FR-RND-012 confines the key; the value is a value like any other.
        "set" => {
            return match text.split_once('=') {
                Some((key, value)) if admits(key) && admits(value) => Piece {
                    text: text.to_owned(),
                    value_of: Some(id.to_owned()),
                    glued: None,
                    placeholder: None,
                },
                Some((key, _)) if admits(key) => Piece {
                    text: format!("{key}=<{key}>"),
                    value_of: Some(id.to_owned()),
                    glued: None,
                    placeholder: Some((
                        format!("<{key}>"),
                        format!("the value you gave --set {key}"),
                    )),
                },
                _ => refused(),
            };
        }
        _ => admits(text),
    };

    if admitted {
        Piece {
            text: text.to_owned(),
            value_of: Some(id.to_owned()),
            glued: None,
            placeholder: None,
        }
    } else {
        refused()
    }
}

fn placeholder(id: Option<&str>, text: String, meaning: String) -> Piece {
    Piece {
        text: text.clone(),
        value_of: id.map(str::to_owned),
        glued: None,
        placeholder: Some((text, meaning)),
    }
}

#[cfg(test)]
mod tests {
    use super::{Carry, Edit, Globals, Replacement, Shape, carry_into, globals, restate, shaped};

    fn shape_of(words: &[&str]) -> Shape {
        let words: Vec<Option<&str>> = words.iter().copied().map(Some).collect();
        shaped(&crate::cli::tree(), &words).expect("every flag is declared")
    }

    #[test]
    fn fr_proj_027_the_shape_is_the_command_path_and_whether_an_operand_was_given() {
        let shape = shape_of(&["--tpl-dir", "shop", "-d", "a", "cfg", "database", "list"]);
        assert_eq!(shape.path, "cfg database list");
        assert!(!shape.operand);

        let shape = shape_of(&["--tpl-dir=shop", "schema", "tables", "--format", "json"]);
        assert_eq!(shape.path, "schema tables");
        assert!(!shape.operand);

        let shape = shape_of(&[
            "--tpl-dir",
            "shop",
            "cfg",
            "database",
            "add",
            "zz",
            "--host",
            "h",
        ]);
        assert_eq!(shape.path, "cfg database add");
        assert!(shape.operand);

        let shape = shape_of(&["template", "show", "--", "example"]);
        assert_eq!(shape.path, "template show");
        assert!(shape.operand);
    }

    #[test]
    fn v_08_remove_drops_the_flag_and_its_value_in_either_spelling() {
        for words in [
            &["-d", "shop", "cfg", "database", "test"][..],
            &["--database=shop", "cfg", "database", "test"][..],
        ] {
            let (command, _) = restated(
                words,
                &[Edit::Remove(&["database"]), Edit::Append(&["shop"])],
            );
            assert_eq!(command, "tpl cfg database test shop");
        }
    }

    fn restated(words: &[&str], edits: &[Edit<'_>]) -> (String, String) {
        let words: Vec<Option<&str>> = words.iter().copied().map(Some).collect();
        let restated = restate(&crate::cli::tree(), &words, edits).expect("restates");
        let replacing = restated.replacing();
        (restated.command, replacing)
    }

    const FUNCTION_F: Edit<'static> = Edit::Value {
        ids: &["routine", "name"],
        to: Replacement::Literal("function:f"),
    };

    #[test]
    fn t_01_keeps_every_flag_and_changes_the_faulty_token() {
        let (command, replacing) = restated(
            &[
                "-d",
                "shop",
                "render",
                "rust/x",
                "--routine",
                "FUNCTION:f",
                "--set",
                "a=b",
            ],
            &[FUNCTION_F],
        );
        assert_eq!(
            command,
            "tpl -d shop render rust/x --routine function:f --set a=b"
        );
        assert_eq!(replacing, "");
    }

    #[test]
    fn t_01_keeps_the_context_document_and_the_project() {
        let (command, _) = restated(
            &[
                "--tpl-dir",
                ".tpl",
                "render",
                "example",
                "--context",
                "rt.json",
                "--routine=r",
            ],
            &[FUNCTION_F],
        );
        assert_eq!(
            command,
            "tpl --tpl-dir .tpl render example --context rt.json --routine=function:f"
        );
    }

    #[test]
    fn t_01_positional_routine_is_the_one_replaced() {
        let (command, _) = restated(
            &["-d", "r", "schema", "routine", "Function:x", "--direct"],
            &[FUNCTION_F],
        );
        assert_eq!(command, "tpl -d r schema routine function:f --direct");
    }

    #[test]
    fn t_01_refused_values_become_named_placeholders() {
        let (command, replacing) = restated(
            &[
                "-vv",
                "-dshop",
                "schema",
                "tables",
                "--pattern",
                "ord%",
                "--pretty",
            ],
            &[Edit::FormatJson],
        );
        assert_eq!(
            command,
            "tpl -d shop -vv schema tables --pattern <pattern> --format json --pretty"
        );
        assert_eq!(
            replacing,
            "; replace <pattern> with the value you gave --pattern"
        );
    }

    #[test]
    fn t_01_format_text_becomes_json() {
        let (command, _) = restated(
            &["schema", "table", "orders", "--format", "text", "--pretty"],
            &[Edit::FormatJson],
        );
        assert_eq!(command, "tpl schema table orders --format json --pretty");
    }

    #[test]
    fn t_01_set_values_are_named_by_key() {
        let (command, replacing) = restated(
            &[
                "render",
                "t",
                "--routine",
                "x",
                "--set",
                "title=a b",
                "--context",
                "/a b/c.json",
            ],
            &[FUNCTION_F],
        );
        assert_eq!(
            command,
            "tpl render t --routine function:f --set title=<title> --context <context>"
        );
        assert_eq!(
            replacing,
            "; replace <title> with the value you gave --set title, and <context> with the value \
             you gave --context"
        );
    }

    #[test]
    fn t_01_a_dsn_and_a_cfg_value_are_never_written_back() {
        let (command, replacing) = restated(
            &[
                "cfg",
                "database",
                "update",
                "f",
                "--dsn",
                "mysql://u:p@h/d",
                "--tls",
                "required",
            ],
            &[],
        );
        assert_eq!(
            command,
            "tpl cfg database update f --dsn <dsn> --tls required"
        );
        assert_eq!(replacing, "; replace <dsn> with the value you gave --dsn");

        let (command, _) = restated(&["cfg", "set", "database.a.password", "secret"], &[]);
        assert_eq!(command, "tpl cfg set database.a.password <value>");
    }

    #[test]
    fn t_01_an_edit_with_nothing_to_act_on_restates_nothing() {
        let words = [Some("version")];
        assert!(restate(&crate::cli::tree(), &words, &[FUNCTION_F]).is_none());
    }

    const BOTH: Carry = Carry {
        tpl_dir: true,
        database: true,
    };

    /// What `hint` becomes for an invocation written as `words`.
    fn carried(words: &[&str], hint: &str, carry: Carry) -> String {
        let tree = crate::cli::tree();
        let words: Vec<Option<&str>> = words.iter().copied().map(Some).collect();
        let given = globals(&tree, &words);
        carry_into(&tree, hint, &given, carry).unwrap_or_else(|| hint.to_owned())
    }

    const SHOP: &[&str] = &[
        "--tpl-dir",
        "/srv/shop/.tpl",
        "-d",
        "shop",
        "schema",
        "table",
        "ordrs",
    ];

    #[test]
    fn fr_err_043_the_two_flags_follow_tpl_in_order() {
        assert_eq!(
            carried(
                SHOP,
                "did you mean 'orders'? list the available tables with: tpl schema tables",
                BOTH
            ),
            "did you mean 'orders'? list the available tables with: tpl --tpl-dir \
             /srv/shop/.tpl -d shop schema tables"
        );
        // Written in any order and any spelling, carried as --tpl-dir first.
        assert_eq!(
            carried(
                &[
                    "--database=shop",
                    "schema",
                    "tables",
                    "--tpl-dir=/srv/shop/.tpl"
                ],
                "list them with: tpl schema views",
                BOTH
            ),
            "list them with: tpl --tpl-dir /srv/shop/.tpl -d shop schema views"
        );
        assert_eq!(
            carried(
                &["-vdshop", "schema", "tables"],
                "run: tpl schema views",
                BOTH
            ),
            "run: tpl -d shop schema views"
        );
    }

    #[test]
    fn fr_err_043_a_refused_value_is_a_placeholder_the_line_explains() {
        assert_eq!(
            carried(
                &[
                    "--tpl-dir",
                    "/srv/my shop/.tpl",
                    "-d",
                    "shop",
                    "schema",
                    "table",
                    "x"
                ],
                "list the available tables with: tpl schema tables",
                BOTH
            ),
            "list the available tables with: tpl --tpl-dir <tpl-dir> -d shop schema tables; \
             replace <tpl-dir> with the value you gave --tpl-dir"
        );
        assert_eq!(
            carried(
                &["-d", "my-shop", "schema", "table", "x"],
                "list them with: tpl schema tables",
                BOTH
            ),
            "list them with: tpl -d <database> schema tables; replace <database> with the value \
             you gave --database"
        );
        // A restated command already says what its placeholder stands for.
        assert_eq!(
            carried(
                &["--tpl-dir", "/a b", "cfg", "list"],
                "run: tpl --tpl-dir <tpl-dir> cfg list; replace <tpl-dir> with the value you gave \
                 --tpl-dir",
                BOTH
            ),
            "run: tpl --tpl-dir <tpl-dir> cfg list; replace <tpl-dir> with the value you gave \
             --tpl-dir"
        );
    }

    #[test]
    fn fr_err_043_a_flag_is_carried_only_where_it_has_an_effect() {
        let cases = [
            (
                "see: tpl cfg list",
                "see: tpl --tpl-dir /srv/shop/.tpl cfg list",
            ),
            (
                "see: tpl template show t",
                "see: tpl --tpl-dir /srv/shop/.tpl template show t",
            ),
            (
                "see: tpl cache load",
                "see: tpl --tpl-dir /srv/shop/.tpl -d shop cache load",
            ),
            (
                "see: tpl render t --table x",
                "see: tpl --tpl-dir /srv/shop/.tpl -d shop render t --table x",
            ),
            (
                "see: tpl render t --context c.json; or not",
                "see: tpl --tpl-dir /srv/shop/.tpl render t --context c.json; or not",
            ),
            ("see: tpl help cfg set", "see: tpl help cfg set"),
            ("see: tpl init <path>", "see: tpl init <path>"),
            ("see: tpl version", "see: tpl version"),
        ];
        for (hint, expected) in cases {
            assert_eq!(carried(SHOP, hint, BOTH), expected, "{hint}");
        }

        // A restated command loses a flag the node it names ignores.
        assert_eq!(
            carried(
                &[
                    "-d", "shop", "cfg", "database", "update", "f", "--port", "1"
                ],
                "write: tpl -d shop cfg database update f --dsn <dsn>",
                BOTH
            ),
            "write: tpl cfg database update f --dsn <dsn>"
        );
    }

    #[test]
    fn fr_err_043_a_hint_that_changes_a_flag_keeps_what_it_proposes() {
        // A candidate entry stands in place of the one -d gave.
        assert_eq!(
            carried(SHOP, "try: tpl -d shops schema tables", BOTH),
            "try: tpl --tpl-dir /srv/shop/.tpl -d shops schema tables"
        );
        assert_eq!(
            carried(
                SHOP,
                "list: tpl cfg database list",
                Carry {
                    tpl_dir: true,
                    database: false
                }
            ),
            "list: tpl --tpl-dir /srv/shop/.tpl cfg database list"
        );
        assert_eq!(
            carried(
                SHOP,
                "name one with: tpl --tpl-dir <path>/.tpl <command>",
                BOTH
            ),
            "name one with: tpl --tpl-dir <path>/.tpl <command>"
        );
        assert_eq!(
            carried(
                SHOP,
                "run: tpl schema tables",
                Carry {
                    tpl_dir: false,
                    database: true
                }
            ),
            "run: tpl -d shop schema tables"
        );
    }

    #[test]
    fn fr_err_043_every_command_of_the_line_is_carried_and_prose_is_not() {
        assert_eq!(
            carried(
                SHOP,
                "tpl cfg unset a.b; tpl cfg unset c.d; then tpl cfg set e.f <v>",
                BOTH
            ),
            "tpl --tpl-dir /srv/shop/.tpl cfg unset a.b; tpl --tpl-dir /srv/shop/.tpl cfg unset \
             c.d; then tpl --tpl-dir /srv/shop/.tpl cfg set e.f <v>"
        );
        for prose in [
            "add --set title=<value> to the tpl render command",
            "no tpl cfg command runs until the file is valid",
            "tpl discards the command's standard error",
            "the file .tpl/.cfg does not set it",
        ] {
            assert_eq!(carried(SHOP, prose, BOTH), prose);
        }
    }

    #[test]
    fn fr_err_043_only_what_the_caller_wrote_is_carried() {
        let tree = crate::cli::tree();
        let scan = |words: &[&str]| {
            let words: Vec<Option<&str>> = words.iter().copied().map(Some).collect();
            globals(&tree, &words)
        };

        assert_eq!(scan(&["schema", "tables"]), Globals::default());
        // After `--`, and as the value of another flag, `-d` is not the flag.
        assert_eq!(
            scan(&["render", "t", "--", "-d", "shop"]),
            Globals::default()
        );
        assert_eq!(
            scan(&["schema", "tables", "--pattern", "-d"]),
            Globals::default()
        );
        assert_eq!(
            carried(&["schema", "tables"], "run: tpl schema views", BOTH),
            "run: tpl schema views"
        );
    }
}
