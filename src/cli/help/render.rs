//! The renderer of `FR-HELP-006`: seven sections, in one order, laid out at the
//! fixed width of `FR-HELP-009`.
//!
//! `OD-07` settles that `tpl` renders help itself and that the parser's own
//! renderer is never invoked. This module is that renderer. It reads two
//! sources and no third: the **parser tree** [`super::super::tree`] builds, for
//! everything a declaration carries, and the **typed table** of [`super`], for
//! the two sections no argument parser holds.
//!
//! | Section | Read from | Omitted when empty |
//! |---|---|---|
//! | `USAGE` | the node, its children and its positional arguments | never (`FR-HELP-007`) |
//! | `DESCRIPTION` | [`Entry::description`], [`Entry::blocks`], [`Entry::touches`] | never (`FR-HELP-007`) |
//! | `ARGUMENTS` | the node's children (`FR-HELP-008`), then its positional arguments | yes |
//! | `OPTIONS` | the node's own flags | yes |
//! | `EXAMPLES` | [`Entry::examples`] | never (`FR-HELP-007`) |
//! | `EXIT CODES` | [`Entry::exit_codes`] | never (`FR-HELP-007`) |
//! | `SEE ALSO` | [`Entry::see_also`] | yes |
//!
//! The order is the array in [`compose`] and nowhere else, so an eighth section
//! cannot be added by writing a heading somewhere: it has to be written into
//! that array, in a position. The four sections `FR-HELP-007` always requires
//! are omitted by no rule of their own — each of the four is empty for no node
//! of the tree, and a test pins that rather than the renderer special-casing
//! it.
//!
//! # The 80 columns, and where they are decided
//!
//! `FR-HELP-009` fixes the width and `FR-HELP-010` forbids reading `COLUMNS`,
//! querying the terminal and reflowing. [`WIDTH`] is therefore a constant, and
//! **nothing on this path reads the environment**: the only inputs are the tree
//! and the table.
//!
//! A **column is one Unicode scalar value**, counted with
//! [`str::chars`]. The one non-ASCII character the table uses is U+2014 EM
//! DASH, which occupies one column; a test holds the rendered text to that
//! alphabet, so a character whose width is not one forces a decision rather
//! than silently overflowing a line.
//!
//! Two kinds of text are laid out, and only one of them is wrapped:
//!
//! - **Prose** — a description, a caption, the meaning of an exit code, the
//!   one-line summary of a child — is stored unwrapped in the table or in the
//!   tree, and [`wrap`] breaks it on spaces at the width left by its indent.
//! - **An example's lines are never wrapped.** They are shell, and a break on a
//!   space would cut a quoted operand or an invocation that `BR-HELP-003`
//!   parses. They are emitted as written, at [`NESTED`]. A line that does not
//!   fit even so is **folded** by [`folded`], between the tokens of its own
//!   argument vector and with the shell's own continuation, which is the one
//!   break that leaves the example copyable.
//!
//! # What `FR-HELP-013` can be told, and by what
//!
//! Five of the six facts are properties of an argument and are introspected
//! from the declarations of `cli/`:
//!
//! | Fact | Read from |
//! |---|---|
//! | Type | `clap::Arg::get_value_parser`, by the type it produces |
//! | Permitted values | `clap::Arg::get_possible_values`, which a `ValueEnum` fills |
//! | Default | `clap::Arg::get_default_values` |
//! | Required | `clap::Arg::is_required_set` |
//! | Repeatable | [`rules::repeats`], which is the rule [`rules::refuse_repetition`] applies |
//!
//! Repeatability is read from [`rules`] rather than from the action, and the
//! reason is `OD-08`: a flag that carries one value is **declared**
//! `ArgAction::Append` so that both values reach the message of `FR-CLI-014`,
//! and is refused on its second occurrence all the same. Reading the action
//! here would tell the caller the opposite of what the tool does.
//!
//! Repeatability is **three-valued** since `FR-CLI-025`: refused, idempotent,
//! or accumulated. [`rules::repetition`] is that answer and [`facts`] states it,
//! in the terms that requirement fixes for the middle case — accepted more than
//! once, with further occurrences having the effect of the first.
//!
//! **The sixth fact, mutual exclusion, is read from the typed table**, because
//! the tree does not carry it. `FR-OUT-009`, `FR-RND-005`, `FR-CFG-016` and
//! `FR-CFG-029` are refusals *between* two arguments, and
//! [`super::super::globals`], [`super::super::local`] and
//! [`super::super::cfg`] each state why no `conflicts_with` declares them: a
//! refusal written in the parser's words is a refusal the caller never reads in
//! the four labelled lines of `FR-ERR-008`. There is therefore nothing to
//! introspect, and `FR-HELP-013` obliges help to state it all the same — so it
//! is stated **on the argument's own entry**, out of [`super::Documented`],
//! which is where `FR-HELP-022` puts a fact neither channel may derive from the
//! other.
//!
//! **The pairs are stated in one place and not two.** They were named in the
//! prose of the `EXIT CODES` section of the command that refuses each, which
//! answered `FR-HELP-013` for every local pair and for the global pair of
//! `FR-CLI-015` not at all: the root's `64` line carries an unknown command, an
//! unknown flag and a repeated flag value, and never that pair. An obligation
//! met in two places for one population and in neither for the other is met
//! inconsistently, so it is met on the argument — where the requirement puts
//! it, beside the other five facts, and where it is addressable per argument in
//! the JSON document as well. The `EXIT CODES` prose is untouched: it says what
//! produces a code, which is a different question from what an argument may be
//! written with.
//!
//! **One sentence accompanies every flag and every argument**, per
//! `FR-HELP-030`, and it comes from the same typed table. It cannot come from
//! the declarations: the only per-argument text the tree carries is the doc
//! comment `clap`'s derive lifts, and those name requirement identifiers and are
//! written in backticks, which `FR-HELP-014` bars from help — it is
//! self-contained and refers to no document outside the help system.
//!
//! # Where the global flags are, and are not
//!
//! `FR-GLOB-003` lists the seven **once**, in the `OPTIONS` of the root. That
//! holds here by construction rather than by a filter: the seven are declared
//! with `global = true` on the root alone, and `clap` propagates a global
//! argument into the subcommands when the tree is **built**. This renderer
//! reads the tree as [`super::super::tree`] returns it, unbuilt, where every
//! node carries exactly the arguments it declares. A test pins both halves.
//!
//! `USAGE` is the one place the seven are acknowledged below the root, as
//! `[options]`: `FR-GLOB-002` gives every node all seven, so the token is true
//! of every node, and it names no flag.

use std::num::NonZeroU64;

use clap::builder::{PossibleValue, ValueParser};
use clap::{Arg, Command, value_parser};

use super::surface::{self, Family, Item};
use super::{Block, Entry, Example, Line, Outcome, Row, entry};
use crate::cli::rules;

/// The fixed width every help text is laid out to (`FR-HELP-009`).
///
/// A constant, and the only width there is: `FR-HELP-010` forbids reading
/// `COLUMNS`, querying the terminal and reflowing, so that the same help is
/// byte-identical across machines and across every target.
const WIDTH: usize = 80;

/// The indent of a section's body.
const BODY: usize = 2;

/// The indent of what sits under one line of a section's body: the facts of an
/// argument, the children of a node, the lines of an example.
///
/// It is `4` and not more because the longest example line the table holds that
/// is not folded reaches column 79 at this indent. An example is shell, so this
/// indent is fixed by the content rather than chosen.
const NESTED: usize = 4;

/// The blank columns between a label and the text beside it.
const GUTTER: usize = 2;

/// The width of the exit-code number column, `NN` and one space.
const NUMBER: usize = 4;

/// The shell's own line continuation, written at the end of a folded example
/// line.
const CONTINUED: &str = " \\";

/// The extra indent of the remainder of a folded example line.
const FOLD: usize = 2;

/// The help text of the node `path` names, or [`None`] where it names no node
/// of the tree or no entry of the table.
///
/// `path` is the segments below `tpl`, and is empty for the root. A segment is
/// resolved against the tree, so an alias reaches the node it names and the
/// table is indexed by the canonical path, per `FR-HELP-027`.
///
/// The text ends with a single newline and carries no trailing blank line.
pub(crate) fn text(tree: &Command, path: &[&str]) -> Option<String> {
    let (node, canonical) = walk(tree, path)?;

    entry(&canonical).map(|found| compose(node, &canonical, found))
}

/// The node `path` reaches, and the canonical spelling of that path.
fn walk<'a>(tree: &'a Command, path: &[&str]) -> Option<(&'a Command, Vec<&'a str>)> {
    let mut node = tree;
    let mut canonical = Vec::with_capacity(path.len());

    for segment in path {
        let child = node.find_subcommand(segment)?;
        canonical.push(child.get_name());
        node = child;
    }

    Some((node, canonical))
}

/// The seven sections of `FR-HELP-006`, in that order, with the empty ones
/// omitted per `FR-HELP-007`.
fn compose(node: &Command, path: &[&str], found: &Entry) -> String {
    let sections = [
        ("USAGE", usage(node, path)),
        ("DESCRIPTION", description(found)),
        ("ARGUMENTS", arguments(node, path)),
        ("OPTIONS", options(node, path)),
        ("EXAMPLES", examples(found.examples)),
        ("EXIT CODES", exit_codes(found.exit_codes)),
        ("SEE ALSO", see_also(found.see_also)),
    ];

    let mut rendered = String::new();

    for (title, lines) in sections {
        if lines.is_empty() {
            continue;
        }

        if !rendered.is_empty() {
            rendered.push('\n');
        }

        rendered.push_str(title);
        rendered.push('\n');

        for line in lines {
            rendered.push_str(&line);
            rendered.push('\n');
        }
    }

    rendered
}

/// The widest label a table of rows keeps its text beside; a wider label takes
/// a line of its own, with its text beneath it at this column.
const LABEL: usize = 26;

/// The `DESCRIPTION` section: the first paragraph, every block after it, and
/// the four statements of `FR-HELP-031` last, each separated from the one
/// before by a blank line.
fn description(found: &Entry) -> Vec<String> {
    let mut paragraphs: Vec<Vec<String>> = vec![wrapped(found.description, BODY)];

    for block in found.blocks {
        match block {
            Block::Prose(text) => paragraphs.push(wrapped(text, BODY)),
            Block::Rows { heading, rows } => paragraphs.push(table(heading, rows)),
            Block::Surface => paragraphs.extend(template_surface()),
        }
    }

    if let Some(touches) = &found.touches {
        paragraphs.push(wrapped(&touches.sentence(), BODY));
    }

    let mut lines = Vec::new();

    for paragraph in paragraphs {
        if !lines.is_empty() {
            lines.push(String::new());
        }

        lines.extend(paragraph);
    }

    lines
}

/// A heading and its rows, the labels in a column of their own.
fn table(heading: &str, rows: &[Row]) -> Vec<String> {
    let listed: Vec<(&str, &str)> = rows.iter().map(|row| (row.label, row.text)).collect();

    labelled(heading, &listed)
}

/// A heading and its rows, from `(label, text)` pairs.
///
/// The labels share one column, as wide as the widest label up to [`LABEL`];
/// a label wider than that is written on a line of its own and its text starts
/// on the next line, at the column.
fn labelled(heading: &str, rows: &[(&str, &str)]) -> Vec<String> {
    labelled_at(heading, rows, column(rows.iter().map(|(label, _)| *label)))
}

/// The column the text of rows labelled by `labels` starts at: the widest
/// label up to [`LABEL`], and the gutter.
fn column<'a>(labels: impl Iterator<Item = &'a str>) -> usize {
    widest(labels.filter(|label| count(label) <= LABEL)) + GUTTER
}

/// A heading and its rows, their text starting at `column`.
fn labelled_at(heading: &str, rows: &[(&str, &str)], column: usize) -> Vec<String> {
    let mut lines = wrapped(heading, BODY);

    for (label, text) in rows {
        if count(label) + GUTTER > column {
            lines.push(indented(label, NESTED));
            lines.extend(wrapped(text, NESTED + column));
        } else {
            lines.extend(tabulated(label, text, NESTED, column));
        }
    }

    lines
}

/// The template surface of `FR-HELP-033`, as paragraphs: the context
/// variables, each family of groups 1 and 2 with its signatures and purposes,
/// and the statement about group 3 that `FR-ENV-004` requires.
///
/// Every line is read from [`surface`], which the JSON document reads too, and
/// every name from the registrations `render/` performs.
fn template_surface() -> Vec<Vec<String>> {
    let variables: Vec<(&str, &str)> = surface::VARIABLES
        .iter()
        .map(|variable| (variable.name, variable.purpose))
        .collect();

    let groups: [(&str, Family, &[&str]); 4] = [
        (
            "Filters of tpl (contract):",
            Family::Filter,
            crate::render::REGISTERED_FILTERS,
        ),
        (
            "Filters of the template engine (pinned):",
            Family::Filter,
            crate::render::INHERITED_FILTERS,
        ),
        (
            "Tests of tpl (contract). Each takes a column; anything else fails the render:",
            Family::Test,
            crate::render::REGISTERED_TESTS,
        ),
        (
            "Functions of tpl (contract):",
            Family::Function,
            crate::render::REGISTERED_FUNCTIONS,
        ),
    ];

    let mut paragraphs = vec![labelled("A template sees these variables:", &variables)];

    // One column for the four families, so that every signature starts its
    // purpose at the same place whichever family it belongs to.
    let families: Vec<(&str, Vec<(&str, &str)>)> = groups
        .iter()
        .map(|(heading, family, names)| {
            let rows = names
                .iter()
                .filter_map(|name| surface::item(*family, name))
                .map(|item: &Item| (item.signature, item.purpose))
                .collect();

            (*heading, rows)
        })
        .collect();
    let shared = column(
        families
            .iter()
            .flat_map(|(_, rows)| rows.iter().map(|(label, _)| *label)),
    );

    for (heading, rows) in &families {
        paragraphs.push(labelled_at(heading, rows, shared));
    }

    paragraphs.push(wrapped(
        "Contract: guaranteed by tpl. Pinned: guaranteed to behave as in the template engine \
         version tpl is built with. Everything else the template engine offers also works, but \
         carries no guarantee: it can change or disappear when tpl moves to a new engine \
         version.",
        BODY,
    ));

    paragraphs
}

/// The one line of `USAGE`: the path, the subcommand where the node has
/// children, its positional arguments, and the flags every node accepts.
fn usage(node: &Command, path: &[&str]) -> Vec<String> {
    let mut line = written(path);

    if node.get_subcommands().next().is_some() {
        line.push_str(" <subcommand>");
    }

    for argument in positionals(node) {
        line.push(' ');
        line.push_str(&placeholder(argument));
    }

    line.push_str(" [options]");

    vec![indented(&line, BODY)]
}

/// `ARGUMENTS`: the node's children first, as its first positional argument per
/// `FR-HELP-008`, then the positional arguments it declares.
fn arguments(node: &Command, path: &[&str]) -> Vec<String> {
    let mut lines = children(node);

    for argument in positionals(node) {
        lines.push(indented(&placeholder(argument), BODY));
        lines.extend(stated(path, value_name(argument)));
        lines.extend(wrapped(
            &facts(argument, path, value_name(argument)),
            NESTED,
        ));
    }

    lines
}

/// The sentence of `FR-HELP-030` for the argument `name` at `path`, laid out.
///
/// It precedes the facts of `FR-HELP-013` because it is the fact a caller came
/// to the help for: the six say what shape the value has, and an agent choosing
/// between two flags learns from them nothing about which to write.
///
/// A table with no row for the argument contributes no line. The table is
/// complete over the tree and a test walks the tree to hold it so, which is
/// where an argument added without a sentence is caught — rather than here,
/// where the only thing a renderer could do about it is invent one.
fn stated(path: &[&str], name: &str) -> Vec<String> {
    super::documented(path, name)
        .map(|stated| wrapped(stated.purpose, NESTED))
        .unwrap_or_default()
}

/// The children of a node, in the shape `FR-HELP-008` gives them: one short
/// line each, under a first positional argument named `<subcommand>`.
///
/// The line is the `about` the tree already carries, and the aliases of
/// `FR-CLI-011` are spelled beside the name because `FR-CLI-013` shows every
/// one of them in the help of its parent node.
fn children(node: &Command) -> Vec<String> {
    let listed: Vec<(String, String)> = node
        .get_subcommands()
        .map(|child| (named(child), about(child)))
        .collect();

    if listed.is_empty() {
        return Vec::new();
    }

    let column = widest(listed.iter().map(|(name, _)| name.as_str())) + GUTTER;

    let mut lines = vec![indented("<subcommand>   one of:", BODY)];

    for (name, summary) in &listed {
        lines.extend(tabulated(name, summary, NESTED, column));
    }

    lines
}

/// `OPTIONS`: every flag the node itself declares, in declaration order.
///
/// Below the root that is the node's own flags alone, which is what
/// `FR-GLOB-003` requires and what this module's own documentation explains.
fn options(node: &Command, path: &[&str]) -> Vec<String> {
    node.get_arguments()
        .filter(|argument| !argument.is_positional())
        .flat_map(|argument| {
            let named = long_form(argument);
            let mut lines = vec![indented(&spelling(argument), BODY)];
            lines.extend(stated(path, &named));
            lines.extend(wrapped(&facts(argument, path, &named), NESTED));

            lines
        })
        .collect()
}

/// A flag's long form with both dashes, which is how the typed table names it.
///
/// Every flag of the tree declares a long form, which a test pins; the
/// identifier stands in for one that did not, so a flag is looked up by
/// something rather than by nothing.
fn long_form(argument: &Arg) -> String {
    format!(
        "--{}",
        argument
            .get_long()
            .unwrap_or_else(|| argument.get_id().as_str())
    )
}

/// `EXAMPLES`: each caption, then the lines of the example, as written.
///
/// The lines are emitted as the table holds them, and [`folded`] is the whole
/// of what may happen to one: an example is shell, and a break on a space would
/// produce an invocation that is not the one `BR-HELP-003` parses through the
/// command parser.
fn examples(listed: &[Example]) -> Vec<String> {
    let mut lines = Vec::new();

    for (position, example) in listed.iter().enumerate() {
        if position > 0 {
            lines.push(String::new());
        }

        lines.extend(wrapped(example.caption, BODY));

        for line in example.lines {
            lines.extend(folded(line, NESTED));
        }
    }

    lines
}

/// `EXIT CODES`: the number, the `sysexits.h` name, and what produces it in
/// this command.
fn exit_codes(listed: &[Outcome]) -> Vec<String> {
    let column = NUMBER + widest(listed.iter().map(|outcome| outcome.code.name())) + GUTTER;

    listed
        .iter()
        .flat_map(|outcome| {
            let label = format!(
                "{number:<width$}{name}",
                number = outcome.code.code(),
                width = NUMBER,
                name = outcome.code.name()
            );

            tabulated(&label, outcome.meaning, BODY, column)
        })
        .collect()
}

/// `SEE ALSO`: one command path per line, per `FR-HELP-014`.
fn see_also(listed: &[&[&str]]) -> Vec<String> {
    listed
        .iter()
        .map(|path| indented(&written(path), BODY))
        .collect()
}

/// One line of an example, broken at a token boundary where it does not fit.
///
/// A line is emitted as written wherever it fits, which is every line but one
/// in the table today. A line that does not fit is folded **between the tokens
/// of its own argument vector** — never inside the shell its [`Line::prefix`]
/// and [`Line::suffix`] carry, whose spacing may be significant — and each
/// continued line ends with the shell's own continuation, so what the caller
/// copies is still the one invocation `BR-HELP-003` parses.
///
/// This is `FR-HELP-009` and not the reflow `FR-HELP-010` forbids: the break is
/// decided once, at the fixed width, and is written into the text.
fn folded(line: &Line, indent: usize) -> Vec<String> {
    let whole = shell(line);

    if indent + count(&whole) <= WIDTH {
        return vec![indented(&whole, indent)];
    }

    let mut lines = Vec::new();
    let mut current = line.prefix.to_owned();
    let mut margin = indent;

    for (position, token) in line.invocation.iter().enumerate() {
        let tail = if position + 1 == line.invocation.len() {
            count(line.suffix)
        } else {
            count(CONTINUED)
        };
        let separator = usize::from(!current.is_empty() && !current.ends_with(' '));
        let reached = margin + count(&current) + separator + count(token) + tail;

        if !current.trim().is_empty() && reached > WIDTH {
            current.push_str(CONTINUED);
            lines.push(indented(&current, margin));

            current = String::new();
            margin = indent + FOLD;
        } else if separator == 1 {
            current.push(' ');
        }

        current.push_str(token);
    }

    current.push_str(line.suffix);
    lines.push(indented(&current, margin));

    lines
}

/// The positional arguments a node declares, in declaration order.
fn positionals(node: &Command) -> impl Iterator<Item = &Arg> {
    node.get_arguments()
        .filter(|argument| argument.is_positional())
}

/// The path a caller writes to reach a node, `tpl` included.
fn written(path: &[&str]) -> String {
    let mut spelled = String::from("tpl");

    for segment in path {
        spelled.push(' ');
        spelled.push_str(segment);
    }

    spelled
}

/// One line of an example, as the caller would type it.
///
/// It is `pub(super)` because the JSON command tree publishes the same line:
/// `FR-HELP-022` makes the typed table the one source of `examples` for both
/// consumers, so the line a caller copies is composed once, here.
pub(super) fn shell(line: &Line) -> String {
    let mut spelled = String::from(line.prefix);
    spelled.push_str(&line.invocation.join(" "));
    spelled.push_str(line.suffix);

    spelled
}

/// A child's name, with the visible aliases of `FR-CLI-011` beside it.
fn named(child: &Command) -> String {
    let mut spelled = child.get_name().to_owned();

    for alias in child.get_visible_aliases() {
        spelled.push_str(", ");
        spelled.push_str(alias);
    }

    spelled
}

/// The one-line summary a node carries, as the tree declares it.
///
/// The backtick is removed. What the tree carries is the node's own doc
/// comment, where a backtick is `rustdoc` markup; help is read by a caller and
/// `FR-HELP-015` admits no decorative character in it. Nothing else is altered:
/// the wording is the tree's.
fn about(child: &Command) -> String {
    child
        .get_about()
        .map(ToString::to_string)
        .unwrap_or_default()
        .replace('`', "")
}

/// A flag as a caller writes it: its short form where it has one, its long
/// form, and the value it takes.
fn spelling(argument: &Arg) -> String {
    let mut spelled = String::new();

    if let Some(short) = argument.get_short() {
        spelled.push('-');
        spelled.push(short);
        spelled.push_str(", ");
    }

    if let Some(long) = argument.get_long() {
        spelled.push_str("--");
        spelled.push_str(long);
    }

    if argument.get_action().takes_values() {
        spelled.push_str(" <");
        spelled.push_str(value_name(argument));
        spelled.push('>');
    }

    spelled
}

/// A positional argument as it appears in `USAGE` and in `ARGUMENTS`: angle
/// brackets where it is required, square brackets where it is not, and an
/// ellipsis where more than one is accepted.
fn placeholder(argument: &Arg) -> String {
    let name = value_name(argument);
    let repeats = if rules::repeats(argument) { "..." } else { "" };

    if argument.is_required_set() {
        format!("<{name}>{repeats}")
    } else {
        format!("[{name}]{repeats}")
    }
}

/// The name the tree gives an argument's value.
///
/// It is `pub(super)` for the reason [`shell`] is: the JSON command tree states
/// the same fact about the same argument, and introspecting it twice would be
/// two readings of one declaration.
pub(super) fn value_name(argument: &Arg) -> &str {
    argument
        .get_value_names()
        .and_then(<[clap::builder::Str]>::first)
        .map_or("VALUE", clap::builder::Str::as_str)
}

/// The six facts `FR-HELP-013` obliges help to state about one flag or
/// argument: its type and permitted values, its default, whether it is
/// required, whether it is repeatable, and the arguments it excludes.
///
/// **Repeatability is three-valued**, because `FR-CLI-025` made it so: a flag
/// that carries a value is refused on its second occurrence, a flag that
/// carries none is accepted and further occurrences have the effect of the
/// first, and a flag or argument that accumulates means something by each. That
/// requirement states the fact for the middle case in its own words, and this
/// is it.
///
/// **The sixth fact is read from the typed table**, and not from the
/// declarations: the tree carries no `conflicts_with` to introspect, because
/// every exclusion this corpus obliges is refused away from the parser so that
/// the caller reads it in the four labelled lines of `FR-ERR-008`. The sentence
/// is omitted where the argument excludes nothing, which is what *any* mutual
/// exclusion means for an argument that has none.
fn facts(argument: &Arg, path: &[&str], name: &str) -> String {
    let mut stated = format!(
        "{kind} {default} {required} {repetition}",
        kind = kind(argument),
        default = default(argument, path, name),
        required = if argument.is_required_set() {
            "Required."
        } else {
            "Optional."
        },
        repetition = match rules::repetition(argument) {
            rules::Repetition::Refused => "Not repeatable.",
            rules::Repetition::Idempotent => "Repeating it changes nothing.",
            rules::Repetition::Accumulated => "Repeatable.",
        }
    );

    if let Some(excluded) = excluded(path, name) {
        stated.push(' ');
        stated.push_str(&excluded);
    }

    stated
}

/// The sentence stating the mutual exclusions of `FR-HELP-013`, or [`None`]
/// where the argument has none.
fn excluded(path: &[&str], name: &str) -> Option<String> {
    let listed = super::documented(path, name)?.excludes;

    match listed {
        [] => None,
        [only] => Some(format!("Not to be given with {only}.")),
        [rest @ .., last] => Some(format!(
            "Not to be given with {} or {last}.",
            rest.join(", ")
        )),
    }
}

/// The type of an argument's value, and its permitted values where they are
/// enumerated.
///
/// The type is read from the value parser the declaration produced, by the Rust
/// type it yields, so a flag that changes type states the new one without
/// anything here changing. A type this function does not name is a type the
/// tree does not use, which a test asserts.
fn kind(argument: &Arg) -> String {
    if !argument.get_action().takes_values() {
        return "Takes no value.".to_owned();
    }

    let permitted = argument.get_possible_values();
    if !permitted.is_empty() {
        let names: Vec<&str> = permitted.iter().map(PossibleValue::get_name).collect();

        return format!("Type: one of {}.", names.join(", "));
    }

    format!("Type: {}.", type_name(argument))
}

/// The type of an argument's value, named.
///
/// The type is read from the value parser the declaration produced, by the Rust
/// type it yields, so a flag that changes type states the new one without
/// anything here changing. A type this function does not name is a type the
/// tree does not use, which a test asserts.
///
/// An argument whose values are enumerated is a **string** drawn from a closed
/// set: the set is stated beside the type — as `one of …` by [`kind`] and as the
/// `permitted` member by the JSON command tree — and not in place of it, so a
/// consumer reads one type vocabulary for every argument of the tree.
///
/// It is `pub(super)` for the reason [`shell`] is.
pub(super) fn type_name(argument: &Arg) -> &'static str {
    if !argument.get_possible_values().is_empty() {
        return "string";
    }

    let produced = argument.get_value_parser().type_id();

    if produced == ValueParser::string().type_id() {
        "string"
    } else if produced == ValueParser::path_buf().type_id() {
        "path"
    } else if produced == ValueParser::from(value_parser!(u16)).type_id() {
        "integer"
    } else if produced == ValueParser::from(value_parser!(NonZeroU64)).type_id() {
        "positive integer"
    } else {
        "value"
    }
}

/// What an argument is worth when it is not given.
///
/// A default the parser supplies is read from the declaration; a default the
/// configuration applies to a key the command leaves unwritten is read from
/// [`super::implied`], because the declaration cannot carry it.
fn default(argument: &Arg, path: &[&str], name: &str) -> String {
    if let Some(implied) = super::implied(path, name) {
        return format!("Default: \"{implied}\".");
    }

    let declared: Vec<String> = argument
        .get_default_values()
        .iter()
        .map(|value| format!("\"{}\"", value.to_string_lossy()))
        .collect();

    if declared.is_empty() {
        if argument.get_action().takes_values() {
            "No default.".to_owned()
        } else {
            "Off unless given.".to_owned()
        }
    } else {
        format!("Default: {}.", declared.join(", "))
    }
}

/// `text`, broken at the width `indent` leaves, every line indented by it.
fn wrapped(text: &str, indent: usize) -> Vec<String> {
    wrap(text, WIDTH.saturating_sub(indent))
        .iter()
        .map(|line| indented(line, indent))
        .collect()
}

/// `label` in a column of its own, with `text` wrapped in the space left
/// beside it and its continuation lines aligned under it.
fn tabulated(label: &str, text: &str, indent: usize, column: usize) -> Vec<String> {
    let broken = wrap(text, WIDTH.saturating_sub(indent + column));
    let padding = column.saturating_sub(count(label));

    let mut lines = Vec::with_capacity(broken.len().max(1));

    for (position, line) in broken.iter().enumerate() {
        if position == 0 {
            lines.push(format!(
                "{}{label}{}{line}",
                spaces(indent),
                spaces(padding)
            ));
        } else {
            lines.push(indented(line, indent + column));
        }
    }

    if lines.is_empty() {
        lines.push(indented(label, indent));
    }

    lines
}

/// `text` broken on spaces so that no line is wider than `width` columns.
///
/// A word wider than `width` takes a line of its own rather than being cut: a
/// break inside a flag spelling or a command path would produce text that reads
/// as something the tree does not declare.
fn wrap(text: &str, width: usize) -> Vec<String> {
    let mut lines = Vec::new();
    let mut line = String::new();

    for word in text.split_whitespace() {
        if line.is_empty() {
            line.push_str(word);
        } else if count(&line) + 1 + count(word) <= width {
            line.push(' ');
            line.push_str(word);
        } else {
            lines.push(std::mem::take(&mut line));
            line.push_str(word);
        }
    }

    if !line.is_empty() {
        lines.push(line);
    }

    lines
}

/// `text`, indented by `indent` columns.
fn indented(text: &str, indent: usize) -> String {
    format!("{}{text}", spaces(indent))
}

/// A run of `count` spaces.
fn spaces(count: usize) -> String {
    " ".repeat(count)
}

/// The width of `text`, in columns.
///
/// A column is one Unicode scalar value, for the reason this module's own
/// documentation gives.
fn count(text: &str) -> usize {
    text.chars().count()
}

/// The width of the widest of `texts`, in columns, or `0` where there are none.
fn widest<'a>(texts: impl Iterator<Item = &'a str>) -> usize {
    texts.map(count).max().unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use super::{WIDTH, count, text, walk, wrap};
    use crate::cli::{parse, tree};

    /// The seven sections of `FR-HELP-006`, in the one order it fixes.
    const SECTIONS: [&str; 7] = [
        "USAGE",
        "DESCRIPTION",
        "ARGUMENTS",
        "OPTIONS",
        "EXAMPLES",
        "EXIT CODES",
        "SEE ALSO",
    ];

    /// The four sections `FR-HELP-007` prints whatever the node.
    const ALWAYS: [&str; 4] = ["USAGE", "DESCRIPTION", "EXAMPLES", "EXIT CODES"];

    /// The seven global flags of `FR-GLOB-001`, as `OPTIONS` spells them.
    const GLOBALS: [&str; 7] = [
        "-d, --database <NAME>",
        "--tpl-dir <PATH>",
        "--timeout <SECONDS>",
        "-v, --verbose",
        "-q, --quiet",
        "-h, --help",
        "-V, --version",
    ];

    /// Applies `visitor` to every node of `command`, with the path it is
    /// reached by, in declaration order.
    fn visit(
        command: &clap::Command,
        path: &[&str],
        visitor: &mut impl FnMut(&[&str], &clap::Command),
    ) {
        visitor(path, command);

        for child in command.get_subcommands() {
            let mut below: Vec<&str> = path.to_vec();
            below.push(child.get_name());
            visit(child, &below, visitor);
        }
    }

    /// Every node of the tree, by the path it is reached by.
    fn node_paths() -> Vec<Vec<String>> {
        let tree = tree();
        let mut paths = Vec::new();
        visit(&tree, &[], &mut |path, _| {
            paths.push(path.iter().map(|&segment| segment.to_owned()).collect());
        });

        paths
    }

    /// The help of the node `path` names.
    fn rendered(path: &[&str]) -> String {
        text(&tree(), path).expect("every node of the tree has an entry")
    }

    /// The path of a node as a caller writes it, for a failure message.
    fn spelled(path: &[&str]) -> String {
        std::iter::once("tpl")
            .chain(path.iter().copied())
            .collect::<Vec<&str>>()
            .join(" ")
    }

    /// The help of every node, by the path it is reached by.
    fn every_help() -> Vec<(Vec<String>, String)> {
        node_paths()
            .into_iter()
            .map(|path| {
                let segments: Vec<&str> = path.iter().map(String::as_str).collect();
                let help = rendered(&segments);

                (path, help)
            })
            .collect()
    }

    /// A rendered help text with its layout collapsed: every run of
    /// whitespace, line breaks included, reduced to one space.
    fn collapsed(help: &str) -> String {
        help.split_whitespace().collect::<Vec<&str>>().join(" ")
    }

    /// The headings of a rendered help text, in the order it carries them.
    ///
    /// A heading is a line that begins in column 0, which is what separates a
    /// section from its body: every line of a body is indented.
    fn headings(help: &str) -> Vec<&str> {
        help.lines()
            .filter(|line| !line.is_empty() && !line.starts_with(' '))
            .collect()
    }

    /// The body of one section of a rendered help text, or [`None`] where the
    /// text omits that section.
    fn section<'a>(help: &'a str, title: &str) -> Option<Vec<&'a str>> {
        let mut lines = help.lines().skip_while(|line| *line != title);
        lines.next()?;

        Some(
            lines
                .take_while(|line| line.is_empty() || line.starts_with(' '))
                .collect(),
        )
    }

    #[test]
    fn fr_help_006_every_node_carries_the_seven_sections_in_order_and_no_eighth() {
        // FR-HELP-006 fixes seven sections, "in this order and no other". The
        // headings of every node are therefore a subsequence of the seven, and
        // a heading the requirement does not name is an eighth section however
        // it is spelled.
        for (path, help) in every_help() {
            let mut expected = SECTIONS.iter();

            for heading in headings(&help) {
                assert!(
                    expected.any(|section| *section == heading),
                    "{path:?} carries '{heading}' out of order, or as an eighth section"
                );
            }
        }
    }

    #[test]
    fn fr_help_007_the_four_mandatory_sections_appear_on_every_node() {
        // FR-HELP-007: USAGE, DESCRIPTION, EXAMPLES and EXIT CODES always
        // appear.
        for (path, help) in every_help() {
            for title in ALWAYS {
                assert!(headings(&help).contains(&title), "{path:?} omits {title}");
            }
        }
    }

    #[test]
    fn fr_help_007_an_empty_section_is_omitted_and_a_filled_one_is_not() {
        // FR-HELP-007: the other three appear exactly when the node has
        // something to put in them. ARGUMENTS holds two kinds of thing, per
        // FR-HELP-008, so it appears for a node with children as well as for
        // one with a positional argument of its own.
        let tree = tree();

        for (path, help) in every_help() {
            let segments: Vec<&str> = path.iter().map(String::as_str).collect();
            let (node, _) = walk(&tree, &segments).expect("the path names a node");

            let arguments = node.get_subcommands().next().is_some()
                || node.get_arguments().any(clap::Arg::is_positional);
            let options = node
                .get_arguments()
                .any(|argument| !argument.is_positional());

            assert_eq!(
                headings(&help).contains(&"ARGUMENTS"),
                arguments,
                "{path:?} disagrees with the tree about ARGUMENTS"
            );
            assert_eq!(
                headings(&help).contains(&"OPTIONS"),
                options,
                "{path:?} disagrees with the tree about OPTIONS"
            );
        }

        // `tpl version` declares no argument and no flag of its own, so it is
        // the node that proves the omission rather than the presence.
        let help = rendered(&["version"]);
        let version = headings(&help);
        assert!(!version.contains(&"ARGUMENTS"));
        assert!(!version.contains(&"OPTIONS"));
    }

    #[test]
    fn fr_help_009_no_rendered_line_exceeds_eighty_columns() {
        // FR-HELP-009, over every node of the tree. A column is one Unicode
        // scalar value, which the alphabet test below holds the text to.
        for (path, help) in every_help() {
            for line in help.lines() {
                assert!(
                    count(line) <= WIDTH,
                    "{path:?} writes a line of {} columns: {line:?}",
                    count(line)
                );
            }
        }
    }

    #[test]
    fn fr_help_008_the_children_of_a_node_are_its_first_positional_argument() {
        // FR-HELP-008: a subcommand is formally the node's first positional
        // argument, so the children are listed inside ARGUMENTS, under
        // `<subcommand>`, and nothing precedes them there.
        let tree = tree();

        for (path, help) in every_help() {
            let segments: Vec<&str> = path.iter().map(String::as_str).collect();
            let (node, _) = walk(&tree, &segments).expect("the path names a node");

            let names: Vec<&str> = node
                .get_subcommands()
                .map(clap::Command::get_name)
                .collect();

            if names.is_empty() {
                continue;
            }

            let listed = section(&help, "ARGUMENTS").expect("a node with children lists them");

            assert_eq!(
                listed.first(),
                Some(&"  <subcommand>   one of:"),
                "{path:?} does not open ARGUMENTS with its children"
            );

            for name in names {
                assert!(
                    listed
                        .iter()
                        .any(|line| line.trim_start().starts_with(name)),
                    "{path:?} does not list the child {name}"
                );
            }
        }

        // The aliases of FR-CLI-011 are shown beside the name they resolve to,
        // per FR-CLI-013.
        let help = rendered(&["schema"]);
        let schema = section(&help, "ARGUMENTS").expect("schema has children");
        assert!(schema.iter().any(|line| line.contains("tables, tbls")));
    }

    #[test]
    fn fr_glob_003_the_seven_global_flags_are_listed_at_the_root_and_at_no_other_node() {
        // FR-GLOB-003, both halves. The root lists the seven in the order
        // FR-GLOB-001 declares them, and no other node names one of them in
        // its OPTIONS.
        let help = rendered(&[]);
        let root = section(&help, "OPTIONS").expect("the root lists the global flags");
        let spellings: Vec<String> = root
            .iter()
            .filter(|line| line.starts_with("  -"))
            .map(|line| line.trim().to_owned())
            .collect();

        assert_eq!(spellings, GLOBALS);

        for (path, help) in every_help() {
            if path.is_empty() {
                continue;
            }

            let Some(listed) = section(&help, "OPTIONS") else {
                continue;
            };

            for flag in GLOBALS {
                assert!(
                    !listed.iter().any(|line| line.trim() == flag),
                    "{path:?} repeats the global flag {flag}"
                );
            }
        }
    }

    #[test]
    fn fr_help_013_every_flag_and_argument_states_the_facts_of_the_requirement() {
        // FR-HELP-013, over all six facts: five are introspected from the
        // declarations and the sixth is read from the typed table, which is
        // where the tree's exclusions are stated because the tree does not
        // declare them.
        let tree = tree();

        for (path, help) in every_help() {
            let segments: Vec<&str> = path.iter().map(String::as_str).collect();
            let (node, _) = walk(&tree, &segments).expect("the path names a node");

            // The facts of an argument are wrapped like any other prose, so
            // they are asserted over the text with its layout collapsed: what
            // is pinned is that every argument of every node is named, that its
            // sentence of FR-HELP-030 follows it, and that the facts follow
            // that — not the column a break fell in.
            let collapsed = collapsed(&help);

            for argument in node.get_arguments() {
                let named = if argument.is_positional() {
                    super::placeholder(argument)
                } else {
                    super::spelling(argument)
                };
                let key = if argument.is_positional() {
                    super::value_name(argument).to_owned()
                } else {
                    super::long_form(argument)
                };
                let purpose = crate::cli::help::documented(&segments, &key)
                    .unwrap_or_else(|| panic!("{path:?} states no purpose for {key}"))
                    .purpose;
                let stated = super::facts(argument, &segments, &key);

                assert!(
                    collapsed.contains(&format!("{named} {purpose} {stated}")),
                    "{path:?} does not state '{named} {purpose} {stated}'"
                );

                assert!(
                    stated.contains("Required.") || stated.contains("Optional."),
                    "{path:?} states no requiredness for {named}"
                );
                // FR-CLI-025 made the fact three-valued, so the assertion is
                // over the three wordings and not over two.
                assert!(
                    stated.contains("Not repeatable.")
                        || stated.contains("Repeatable.")
                        || stated.contains("Repeating it changes nothing."),
                    "{path:?} states no repeatability for {named}"
                );
                assert!(
                    stated.contains("Default:")
                        || stated.contains("No default.")
                        || stated.contains("Off unless given."),
                    "{path:?} states no default for {named}"
                );
            }
        }
    }

    #[test]
    fn fr_help_013_every_mutual_exclusion_the_tree_declares_is_named_in_the_help_of_its_node() {
        // FR-HELP-013's sixth fact, walked over the tree rather than over a
        // list of pairs written by hand: for every node and every argument that
        // node declares, whatever the typed table states about its exclusions
        // appears in that node's own help, on that argument's entry. A pair
        // added to the table without reaching the reader fails here.
        let tree = tree();
        let mut met = 0usize;

        for (path, help) in every_help() {
            let segments: Vec<&str> = path.iter().map(String::as_str).collect();
            let (node, _) = walk(&tree, &segments).expect("the path names a node");
            let collapsed = collapsed(&help);

            for argument in node.get_arguments() {
                let key = if argument.is_positional() {
                    super::value_name(argument).to_owned()
                } else {
                    super::long_form(argument)
                };
                let Some(sentence) = super::excluded(&segments, &key) else {
                    continue;
                };

                met += 1;

                let named = if argument.is_positional() {
                    super::placeholder(argument)
                } else {
                    super::spelling(argument)
                };
                let stated = super::facts(argument, &segments, &key);

                // The sentence sits inside the argument's own entry, which is
                // what `{named} … {stated}` being one run asserts.
                assert!(
                    collapsed.contains(&format!("{named} ")),
                    "{path:?} does not name {named}"
                );
                assert!(
                    stated.ends_with(&sentence),
                    "{path:?} states {key}'s exclusions away from its facts"
                );
                assert!(
                    collapsed.contains(&stated),
                    "{path:?} does not state '{sentence}' on the entry of {key}"
                );

                for excluded in super::super::documented(&segments, &key)
                    .expect("the argument has a row")
                    .excludes
                {
                    assert!(
                        sentence.contains(excluded),
                        "{path:?} states {key}'s exclusions without naming {excluded}"
                    );
                }
            }
        }

        // A floor, so that a table stating no exclusion at all would not pass
        // this walk vacuously.
        assert!(met >= 20, "only {met} exclusions were met over the tree");
    }

    #[test]
    fn fr_help_013_the_global_pair_and_a_local_pair_are_stated_in_the_same_place() {
        // The obligation was met in two places for one population and in
        // neither for the other: every local pair was named in the prose of the
        // EXIT CODES section of the command that refuses it, and the one global
        // pair — -q/--quiet with -v/--verbose — appeared in no help text of the
        // tree at all. Both are now stated on the argument's own entry, in
        // OPTIONS, which is the single place this asserts.
        let root = collapsed(&rendered(&[]));
        let render = collapsed(&rendered(&["render"]));

        for (path, help, entry, excluded) in [
            (&[][..], &root, "-q, --quiet", "--verbose"),
            (&[][..], &root, "-v, --verbose", "--quiet"),
            (&["render"][..], &render, "--table <NAME>", "--view"),
        ] {
            assert!(
                help.contains(&format!("{entry} ")),
                "{entry} is not an entry of this help"
            );

            let node = rendered(path);
            let listed = section(&node, "OPTIONS").expect("the node carries an OPTIONS section");
            let body = listed
                .join(" ")
                .split_whitespace()
                .collect::<Vec<&str>>()
                .join(" ");

            assert!(
                body.contains(&format!("{entry} ")),
                "{entry} is not in the OPTIONS section"
            );
            assert!(
                body.contains(&format!("Not to be given with {excluded}")),
                "{entry} does not state its exclusion in OPTIONS"
            );
        }

        // And it is not also stated in EXIT CODES, which is what "one place"
        // means: that section says what produces a code.
        for path in [&[][..], &["render"][..]] {
            let help = rendered(path);
            let codes = section(&help, "EXIT CODES").expect("every node lists its codes");

            assert!(
                !codes.join(" ").contains("Not to be given with"),
                "{path:?} states an exclusion in EXIT CODES as well"
            );
        }
    }

    #[test]
    fn fr_help_030_every_flag_and_argument_of_every_node_carries_its_sentence() {
        // FR-HELP-030 over the rendered text, walked node by node: the sentence
        // of the typed table appears in the help of the node that declares the
        // argument, between the argument and its facts. The population is the
        // tree's, so an argument added to a node is covered without this test
        // changing.
        let tree = tree();

        for (path, help) in every_help() {
            let segments: Vec<&str> = path.iter().map(String::as_str).collect();
            let (node, _) = walk(&tree, &segments).expect("the path names a node");
            let collapsed = collapsed(&help);

            for argument in node.get_arguments() {
                let key = if argument.is_positional() {
                    super::value_name(argument).to_owned()
                } else {
                    super::long_form(argument)
                };
                let purpose = super::super::documented(&segments, &key)
                    .unwrap_or_else(|| panic!("{path:?} states no purpose for {key}"))
                    .purpose;

                assert!(
                    collapsed.contains(purpose),
                    "{path:?} does not carry the sentence of {key}"
                );
            }
        }
    }

    #[test]
    fn fr_help_013_the_type_of_every_argument_of_the_tree_is_named() {
        // `kind` falls back to the word "value" for a type it does not name,
        // and no argument of the tree reaches it. A new value type therefore
        // fails here rather than telling a caller nothing.
        for (path, help) in every_help() {
            assert!(
                !help.contains("Type: value."),
                "{path:?} carries an argument whose type is not named"
            );
        }
    }

    #[test]
    fn fr_rnd_008_the_repeatability_stated_is_the_repeatability_enforced() {
        // The one flag FR-RND-008 makes repeatable says so, and a flag that
        // carries a single value says the opposite — which is what FR-CLI-014
        // enforces over the very declarations this help is read from.
        let render = rendered(&["render"]);

        assert!(
            render.contains("Type: string. No default. Optional. Repeatable."),
            "{render}"
        );
        assert!(
            render.contains(
                "Type: string. No default. Optional. Not repeatable. Not to be given with\n    \
                 --view or --routine."
            ),
            "{render}"
        );

        assert!(
            parse(["tpl", "render", "t", "--set", "a=1", "--set", "b=2"]).is_ok(),
            "--set is refused on its second occurrence"
        );
        assert!(
            parse(["tpl", "render", "t", "--table", "a", "--table", "b"]).is_err(),
            "--table is accepted twice"
        );
    }

    #[test]
    fn fr_help_015_no_markup_reaches_the_help() {
        // FR-HELP-015: help is read by a caller, not rendered by a documentation
        // tool. The one text lifted from the tree is a node's `about`, which is
        // a doc comment and carries backticks; three of them do.
        for (path, help) in every_help() {
            assert!(!help.contains('`'), "{path:?} carries rustdoc markup");
        }
    }

    #[test]
    fn fr_help_015_nothing_decorative_reaches_the_help() {
        // FR-HELP-015 and NFR-DET-004: no colour, no emoji, no ANSI escape
        // sequence, no decorative character. The alphabet is pinned rather
        // than the absence of an escape alone, because it is also what makes
        // counting columns in Unicode scalar values exact: U+2014 and U+2026
        // are the two non-ASCII characters the table uses, and each occupies
        // one column. U+2026 is not decoration: FR-ENV-047 writes a keyword
        // argument with no default as `name=…` in a signature.
        for (path, help) in every_help() {
            for character in help.chars() {
                let admitted = character == '\n'
                    || character == '\u{2014}'
                    || character == '\u{2026}'
                    || ('\u{20}'..='\u{7e}').contains(&character);

                assert!(
                    admitted,
                    "{path:?} carries {character:?}, which is outside the help alphabet"
                );
            }
        }
    }

    #[test]
    fn fr_help_006_the_layout_is_the_one_the_sections_fix() {
        // One node, written out, so that a change to the layout is a change to
        // this test rather than something noticed downstream. `tpl version`
        // is the smallest node of the tree: no argument, no flag of its own,
        // and so no ARGUMENTS and no OPTIONS.
        assert_eq!(
            rendered(&["version"]),
            "\
USAGE
  tpl version [options]

DESCRIPTION
  Prints tpl, a space and the version, such as tpl 0.1.0, and nothing else.

  Does not contact the server. Needs no database entry and no project. Writes no
  file. Prints the version line.

EXAMPLES
  Print the version.
    tpl version

EXIT CODES
  0   EX_OK     The version line was written to stdout.
  64  EX_USAGE  An argument, a flag other than the global ones (this command
                takes none), or a global flag that takes a value given twice. On
                any command, also -v with -q, or a global flag given a value it
                does not take, such as --timeout 0.
  74  EX_IOERR  stdout could not be written.

SEE ALSO
  tpl help
"
        );
    }

    #[test]
    fn fr_help_009_a_word_wider_than_the_line_takes_a_line_of_its_own() {
        // Wrapping never cuts a word: a break inside a flag spelling or a
        // command path would read as something the tree does not declare.
        assert_eq!(wrap("--no-cache", 4), vec!["--no-cache"]);
        assert_eq!(wrap("a bb ccc", 5), vec!["a bb", "ccc"]);
        assert_eq!(wrap("   ", 10), Vec::<String>::new());
    }

    #[test]
    fn fr_help_028_a_path_that_names_no_node_has_no_help() {
        assert_eq!(text(&tree(), &["schema", "nowhere"]), None);
        assert_eq!(text(&tree(), &["nowhere"]), None);
    }

    #[test]
    fn fr_help_027_an_alias_reaches_the_help_of_the_node_it_names() {
        // FR-HELP-027: an alias resolves to its canonical node, so the table
        // is reached by the canonical path however the caller spelled it.
        assert_eq!(rendered(&["cfg", "db"]), rendered(&["cfg", "database"]));
        assert_eq!(
            rendered(&["schema", "tbls"]),
            rendered(&["schema", "tables"])
        );
    }

    #[test]
    fn fr_help_006_the_help_of_a_node_names_the_node() {
        // USAGE opens with the path a caller writes, at every depth.
        for (path, help) in every_help() {
            let segments: Vec<&str> = path.iter().map(String::as_str).collect();

            assert!(
                help.starts_with(&format!("USAGE\n  {}", spelled(&segments))),
                "{path:?} does not open with its own path"
            );
        }
    }
}
