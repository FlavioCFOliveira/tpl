//! The JSON command tree of `FR-HELP-016`: the whole surface, or one subtree of
//! it, as a single document.
//!
//! It is the other half of [`super`]. The text renderer of [`super::render`]
//! and this module read the **same two sources** — the parser tree
//! [`crate::cli::tree`] declares and the typed table of [`super`] — and part
//! only in what they compose from them. Nothing here parses rendered help,
//! which `FR-HELP-022` forbids, and nothing here is maintained beside the tree,
//! which `FR-HELP-021` forbids: every node, alias, argument and flag of the
//! document is introspected from the tree the binary parses with.
//!
//! # The shape, key by key
//!
//! | Key of `data` | Content | Fixed by |
//! |---|---|---|
//! | `tpl_version` | The package version, read at compile time | `FR-HELP-017` |
//! | `global_flags` | The seven flags of `FR-GLOB-001`, **once** | `FR-HELP-017`, `FR-HELP-018` |
//! | `commands` | A flat array, one entry per node **below** `tpl` | `FR-HELP-019` |
//! | `template_surface` | The three groups of the template surface | `FR-HELP-017`, `FR-ENV-005` |
//!
//! `data` is an **open** set of keys, per `FR-HELP-017`: a later edition adds a
//! key after the last, so the position of every key already present is
//! unchanged. The envelope itself stays closed at three keys, per `FR-OUT-028`,
//! and `source` is `binary` because the document is produced by the binary from
//! itself with nothing read, per `FR-OUT-026`.
//!
//! # Why `commands` is flat, and what a subtree is under that
//!
//! `FR-HELP-019` settles it: `data.commands` is a flat array and an entry never
//! carries its children, because the `path` every entry carries **is** the
//! parent-child relation written out and a nested array would carry it twice.
//! A subtree is therefore a **selection** over that array — the entry whose
//! `path` is the path given, together with every entry whose `path` extends it
//! segment by segment, in the order they hold in the unreduced document, per
//! `FR-HELP-029`. [`collect`] produces exactly that by walking from the node the
//! path reached, so the selection is the walk rather than a filter applied
//! after one.
//!
//! The root is not an entry: `FR-HELP-019` says *every node below `tpl`*, and
//! what the root's `OPTIONS` would carry is `data.global_flags`, listed once
//! per `FR-GLOB-003` and `FR-HELP-018`.
//!
//! # Order, and the absence of a map
//!
//! `FR-HELP-023` and `FR-OUT-013` require declaration order throughout and
//! forbid an unordered map on the emitting path. Both hold **structurally**:
//! every type here is a struct whose field declaration order is its key order,
//! per `OD-18`, and every collection is a [`Vec`] filled by a walk in
//! declaration order — of the tree for the nodes and their arguments, and of
//! the typed table for the examples and the exit codes. No map of any kind is
//! constructed, ordered or otherwise.
//!
//! # Where the template surface comes from
//!
//! `FR-HELP-017` fixes the **name and the position** of `data.template_surface`
//! and `FR-ENV-005` fixes what it holds: three sibling objects of one shape,
//! each carrying `guarantee`, `filters`, `tests` and `functions` in that order.
//! The shape is published here, in that position. Its **content** is `render/`'s:
//! that requirement derives `registered` from the registrations the environment
//! actually performs, for the reason `FR-HELP-021` gives for the command tree,
//! so the four arrays that carry names are read from that module's constants
//! and no name is written here. [`TemplateSurface::PUBLISHED`] states which of
//! the twelve values are permanent and which are read from the environment.

use std::io::Write;

use clap::{Arg, Command};
use serde::Serialize;

use super::{Example as Declared, Line as Written, Outcome, render};
use crate::cli::rules;
use crate::error::{self, Error};
use crate::output::{self, Document, Form, Source};

/// The version the document publishes, per `FR-HELP-017`.
///
/// It is the package's own, read at compile time, so it cannot disagree with
/// the binary that emitted the document or with the line `FR-HELP-005` prints.
const VERSION: &str = env!("CARGO_PKG_VERSION");

/// What `FR-HELP-018` puts on every command instead of the seven flags.
///
/// It is `true` for every node without exception, because `FR-GLOB-002` gives
/// every node of the tree every global flag. It is carried as data rather than
/// left implied so that a caller reads one document shape whatever it later
/// comes to mean.
const INHERITS_GLOBALS: bool = true;

/// Writes the command tree, or the subtree `path` roots, to `out`.
///
/// `tree` is the whole parser tree, which is where the seven global flags are
/// declared and therefore where `data.global_flags` is read from. `node` is the
/// node `path` resolved to, and `path` its canonical segments below `tpl`;
/// together they are the reduction of `FR-HELP-029`, and for the unreduced
/// document they are the tree itself and the empty path.
///
/// # Errors
///
/// Returns [`Error::InternalInvariant`] where a node of the tree carries no
/// entry of the typed table — a disagreement between the two sources, which is
/// a defect in `tpl` and which a test of [`super`] pins — and
/// [`Error::StdoutClosedMidDocument`] or [`Error::StdoutUnwritable`] where the
/// stream refused the write.
pub(super) fn emit<W: Write>(
    out: &mut W,
    tree: &Command,
    node: &Command,
    path: &[&str],
    form: Form,
) -> Result<(), Error> {
    let surface = surface(tree, node, path)?;

    output::emit_to(out, &Document::new(Source::Binary, &surface), form)
}

/// The `data` of the document, per `FR-HELP-017`.
///
/// The four keys are the four fields, in the order the requirement fixes them,
/// and `OD-18` makes that declaration order the emitted key order.
#[derive(Debug, Serialize)]
struct Surface<'a> {
    /// The version of the binary that produced the document.
    tpl_version: &'static str,

    /// The seven global flags of `FR-GLOB-001`, carried once for the whole
    /// document, per `FR-HELP-018`.
    global_flags: Vec<Flag<'a>>,

    /// One entry per node below `tpl`, flat and in declaration order, per
    /// `FR-HELP-019`.
    commands: Vec<Entry<'a>>,

    /// The three groups of the template surface, per `FR-ENV-005`.
    template_surface: TemplateSurface,
}

/// One command of the tree, per `FR-HELP-019`.
///
/// The seven members that requirement names come first, in the order it names
/// them, and `inherits_globals` follows them: it is not one of the seven, and
/// placing it after the last leaves every one of them where the requirement
/// puts it. Placing it beside `options`, which is the member it qualifies, was
/// rejected for that reason — it reads well and it interleaves a member the
/// requirement does not list into a sequence the requirement fixes.
#[derive(Debug, Serialize)]
struct Entry<'a> {
    /// The full path of the node, as the canonical segments below `tpl`.
    ///
    /// An array rather than one joined string, because `FR-HELP-029` extends a
    /// path **segment by segment** and because this is the vector a caller
    /// hands straight back to `tpl help`, per `FR-HELP-026`.
    path: Vec<&'a str>,

    /// The aliases of `FR-CLI-011` this node answers to, per `FR-CLI-013`.
    aliases: Vec<&'a str>,

    /// The `DESCRIPTION` the typed table carries, unwrapped.
    description: &'static str,

    /// The positional arguments the node declares, in declaration order.
    ///
    /// A child is never a member of this array: it is an entry of
    /// `data.commands` of its own, which is what the rationale of `FR-HELP-008`
    /// means by separating the two.
    arguments: Vec<Argument<'a>>,

    /// Every **local** flag the node declares, per `FR-HELP-020` and
    /// `FR-GLOB-022`, in declaration order. The seven global ones are in
    /// `data.global_flags` and are never repeated here.
    options: Vec<Flag<'a>>,

    /// The examples of the typed table, per `FR-HELP-012` and `FR-HELP-022`.
    examples: Vec<Example<'a>>,

    /// Only the codes this command can produce, per `FR-HELP-011`, ascending.
    exit_codes: Vec<ExitCode>,

    /// Always `true`: every node accepts every global flag, per `FR-GLOB-002`.
    inherits_globals: bool,
}

/// One positional argument of a node.
///
/// The five facts `FR-HELP-013` obliges help to state about an argument are the
/// five members beside its name. The sixth, mutual exclusion, is absent for the
/// reason [`super::render`] gives at length: the tree does not carry it, and a
/// second source for it would be a table of exclusions maintained beside the
/// commands that enforce them.
#[derive(Debug, Serialize)]
struct Argument<'a> {
    /// The name the tree gives the value, as the help spells it.
    name: &'a str,

    /// The type of the value.
    value_type: &'static str,

    /// The values the argument enumerates, or `null` where it enumerates none.
    permitted: Option<Vec<String>>,

    /// What the argument is worth when it is not given, or `null` where it has
    /// no default.
    default: Option<Vec<String>>,

    /// Whether the invocation must supply it.
    required: bool,

    /// Whether the invocation may supply it more than once.
    repeatable: bool,
}

/// One flag, whether global or local.
///
/// Both populations are the same shape, because `FR-HELP-018` splits them by
/// **position in the document** and not by kind: a caller reads the seven of
/// `data.global_flags` and the locals of a command's `options` with one
/// routine.
#[derive(Debug, Serialize)]
struct Flag<'a> {
    /// The long form, with its two leading dashes.
    long: String,

    /// The short form, with its one leading dash, or `null` where the flag has
    /// none. `FR-GLOB-024` gives a short form to exactly five flags of the
    /// whole tool.
    short: Option<String>,

    /// The name the tree gives the value, or `null` where the flag takes none.
    value_name: Option<&'a str>,

    /// The type of the value, or `null` where the flag takes none.
    value_type: Option<&'static str>,

    /// The values the flag enumerates, or `null` where it enumerates none.
    permitted: Option<Vec<String>>,

    /// What the flag is worth when it is not given, or `null` where it has no
    /// default.
    default: Option<Vec<String>>,

    /// Whether the invocation must supply it.
    required: bool,

    /// Whether the invocation may supply it more than once.
    repeatable: bool,
}

/// One example of a command, per `FR-HELP-012`.
#[derive(Debug, Serialize)]
struct Example<'a> {
    /// What the example does, in one line.
    caption: &'a str,

    /// The example, line by line, in the order it is written.
    lines: Vec<Line<'a>>,
}

/// One line of an example.
///
/// A line carries what a caller **copies** and, where the line is a `tpl` call,
/// what a caller **executes**: `invocation` is the argument vector, token by
/// token, so an agent runs the example without a shell, and it is `null` on a
/// line that is shell alone — a pipe into `jq`, a `while` loop. Publishing the
/// table's own prefix, invocation and suffix instead was rejected: it exposes
/// the layout the renderer needs and obliges every consumer to reassemble the
/// line the caller actually types.
#[derive(Debug, Serialize)]
struct Line<'a> {
    /// The line as the caller would type it, shell included.
    text: String,

    /// The argument vector of the `tpl` call on this line, beginning with
    /// `tpl`, or `null` where the line carries none.
    invocation: Option<&'a [&'a str]>,
}

/// One row of a command's `EXIT CODES`, per `FR-HELP-011` and `FR-HELP-022`.
#[derive(Debug, Serialize)]
struct ExitCode {
    /// The number the process exits with.
    code: u8,

    /// The `sysexits.h` name `FR-ERR-001` gives it.
    name: &'static str,

    /// What produces it in this command.
    meaning: &'static str,
}

/// The template surface of `FR-ENV-005`, in the three groups of `FR-ENV-001`.
#[derive(Debug, Serialize)]
struct TemplateSurface {
    /// Group 1 — what `tpl` registers itself, guaranteed as contract.
    registered: Group,

    /// Group 2 — what the pinned engine and its contrib crate supply, and which
    /// `tpl` names without owning.
    inherited: Group,

    /// Group 3 — everything else the engine offers, unguaranteed and not
    /// enumerable.
    other: Group,
}

/// One group of the template surface.
///
/// The four members are the four keys `FR-ENV-005` fixes, in the order it fixes
/// them. An array is `null` where the group cannot be enumerated, which is what
/// that requirement and `FR-OUT-012` together give for a value that is absent
/// rather than empty.
#[derive(Debug, Serialize)]
struct Group {
    /// What the group promises: `contract`, `pinned` or `none`.
    guarantee: &'static str,

    /// The filter names, or `null`.
    filters: Option<&'static [&'static str]>,

    /// The test names, or `null`.
    tests: Option<&'static [&'static str]>,

    /// The global function names, or `null`.
    functions: Option<&'static [&'static str]>,
}

impl TemplateSurface {
    /// The surface as this binary states it.
    ///
    /// Eight of the twelve values are permanent and fixed by `FR-ENV-005`: the
    /// three guarantees; the three `null` arrays of `other`, because group 3 is
    /// everything the engine offers that the other two do not name and `tpl`
    /// cannot enumerate it; and the two empty arrays of `inherited`, because
    /// group 2 enumerates filters alone, per `FR-ENV-018` and `FR-ENV-019`.
    ///
    /// The remaining four — the three arrays of `registered` and the filters of
    /// `inherited` — are `render/`'s, which `FR-ENV-005` requires them to be
    /// derived from: "the registrations the environment actually performs".
    /// They are taken from that module's own constants and are **not restated
    /// here**, on `FR-HELP-021`'s ground: a second statement of one truth is
    /// the statement that stops being true without saying so, and the document
    /// would then be able to assert a surface the binary does not have. The
    /// three of `registered` are written by the same declaration that installs
    /// the names on the engine; the filters of `inherited` are the closed list
    /// of `FR-ENV-019`, which `tpl` does not register and `ADR-001` guarantees
    /// against the engine pin.
    const PUBLISHED: Self = Self {
        registered: Group {
            guarantee: "contract",
            filters: Some(crate::render::REGISTERED_FILTERS),
            tests: Some(crate::render::REGISTERED_TESTS),
            functions: Some(crate::render::REGISTERED_FUNCTIONS),
        },
        inherited: Group {
            guarantee: "pinned",
            filters: Some(crate::render::INHERITED_FILTERS),
            tests: Some(&[]),
            functions: Some(&[]),
        },
        other: Group {
            guarantee: "none",
            filters: None,
            tests: None,
            functions: None,
        },
    };
}

/// The `data` of the document `emit` writes.
fn surface<'a>(
    tree: &'a Command,
    node: &'a Command,
    path: &[&'a str],
) -> Result<Surface<'a>, Error> {
    // The whole table is the upper bound of any subtree, and the table is
    // thirty-five entries: one allocation, whatever the reduction.
    let mut commands = Vec::with_capacity(super::entries().len());
    let mut segments: Vec<&str> = path.to_vec();

    collect(node, &mut segments, &mut commands)?;

    Ok(Surface {
        tpl_version: VERSION,
        global_flags: flags(tree),
        commands,
        template_surface: TemplateSurface::PUBLISHED,
    })
}

/// Appends `node` and every node beneath it to `into`, in the order
/// `FR-HELP-019` fixes.
///
/// The walk is pre-order over the declared children, so each node is followed
/// by its own children before the next node at its level, and declaration order
/// is preserved throughout — which is the order requirement met by the walk
/// rather than by a sort.
///
/// The root is skipped, and it is the only node that can be: `path` is empty
/// for the root alone, and `FR-HELP-019` carries one entry per node **below**
/// `tpl`.
///
/// # Errors
///
/// Returns [`Error::InternalInvariant`] where a node carries no entry of the
/// typed table.
fn collect<'a>(
    node: &'a Command,
    path: &mut Vec<&'a str>,
    into: &mut Vec<Entry<'a>>,
) -> Result<(), Error> {
    if !path.is_empty() {
        into.push(entry(node, path)?);
    }

    for child in node.get_subcommands() {
        path.push(child.get_name());
        collect(child, path, into)?;
        path.pop();
    }

    Ok(())
}

/// One entry of `data.commands`.
///
/// # Errors
///
/// Returns [`Error::InternalInvariant`] where the typed table carries no entry
/// for `path`.
fn entry<'a>(node: &'a Command, path: &[&'a str]) -> Result<Entry<'a>, Error> {
    let Some(declared) = super::entry(path) else {
        error::ensure_invariant(
            false,
            "every node of the tree carries an entry of the help table",
        )?;

        unreachable!("ensure_invariant returns the error when the invariant does not hold")
    };

    Ok(Entry {
        path: path.to_vec(),
        aliases: node.get_all_aliases().collect(),
        description: declared.description,
        arguments: node
            .get_arguments()
            .filter(|argument| argument.is_positional())
            .map(argument)
            .collect(),
        options: flags(node),
        examples: declared.examples.iter().map(example).collect(),
        exit_codes: declared.exit_codes.iter().map(exit_code).collect(),
        inherits_globals: INHERITS_GLOBALS,
    })
}

/// Every flag `node` declares, in declaration order.
///
/// Below the root that is the node's own local flags alone, which is exactly
/// what `FR-HELP-020` requires of `options`: the seven global flags are
/// declared on the root with `global = true` and are propagated into the
/// subcommands only when the parser is **built**, and this reads the tree as
/// [`crate::cli::tree`] returns it, unbuilt. The root's own call is therefore
/// the seven of `FR-GLOB-001` and nothing else, which is `data.global_flags`.
fn flags(node: &Command) -> Vec<Flag<'_>> {
    node.get_arguments()
        .filter(|argument| !argument.is_positional())
        .map(flag)
        .collect()
}

/// One positional argument.
fn argument(declared: &Arg) -> Argument<'_> {
    Argument {
        name: render::value_name(declared),
        value_type: render::type_name(declared),
        permitted: permitted(declared),
        default: default(declared),
        required: declared.is_required_set(),
        repeatable: rules::repeats(declared),
    }
}

/// One flag.
fn flag(declared: &Arg) -> Flag<'_> {
    let takes_value = declared.get_action().takes_values();

    Flag {
        // Every flag of the tree declares a long form, which a test pins; the
        // identifier stands in for one that did not, so a flag is named by
        // something rather than by nothing.
        long: format!(
            "--{}",
            declared
                .get_long()
                .unwrap_or_else(|| declared.get_id().as_str())
        ),
        short: declared.get_short().map(|short| format!("-{short}")),
        value_name: takes_value.then(|| render::value_name(declared)),
        value_type: takes_value.then(|| render::type_name(declared)),
        // A flag that takes no value enumerates none either. `ArgAction::SetTrue`
        // carries `true` and `false` as possible values of its own, which is the
        // parser's internal representation of a switch and not a set of values
        // this tree accepts — the text help says `Takes no value.` of the same
        // flag, and the two consumers of one declaration may not disagree.
        permitted: takes_value.then(|| permitted(declared)).flatten(),
        default: default(declared),
        required: declared.is_required_set(),
        repeatable: rules::repeats(declared),
    }
}

/// The values an argument enumerates, or [`None`] where it enumerates none.
fn permitted(declared: &Arg) -> Option<Vec<String>> {
    let values = declared.get_possible_values();

    (!values.is_empty()).then(|| {
        values
            .iter()
            .map(|value| value.get_name().to_owned())
            .collect()
    })
}

/// What an argument is worth when it is not given, or [`None`] where it has no
/// default.
fn default(declared: &Arg) -> Option<Vec<String>> {
    let values = declared.get_default_values();

    (!values.is_empty()).then(|| {
        values
            .iter()
            .map(|value| value.to_string_lossy().into_owned())
            .collect()
    })
}

/// One example, as the typed table carries it.
fn example<'a>(declared: &'a Declared) -> Example<'a> {
    Example {
        caption: declared.caption,
        lines: declared.lines.iter().map(line).collect(),
    }
}

/// One line of an example.
fn line<'a>(written: &'a Written) -> Line<'a> {
    Line {
        text: render::shell(written),
        invocation: (!written.invocation.is_empty()).then_some(written.invocation),
    }
}

/// One row of an `EXIT CODES` section.
fn exit_code(declared: &Outcome) -> ExitCode {
    ExitCode {
        code: declared.code.code(),
        name: declared.code.name(),
        meaning: declared.meaning,
    }
}

#[cfg(test)]
mod tests {
    use super::{Surface, TemplateSurface, surface};
    use crate::cli::{parse, tree};
    use crate::output::{Document, Source};
    use clap::Command;

    /// The seven global flags of `FR-GLOB-001`, in the order the tree declares
    /// them, as `data.global_flags` spells them.
    const GLOBALS: [&str; 7] = [
        "--database",
        "--tpl-dir",
        "--timeout",
        "--verbose",
        "--quiet",
        "--help",
        "--version",
    ];

    /// Every node of `command`, with the path it is reached by, in declaration
    /// order: each node followed by its own children before the next node at
    /// its level.
    fn visit(command: &Command, path: &[String], into: &mut Vec<Vec<String>>) {
        if !path.is_empty() {
            into.push(path.to_vec());
        }

        for child in command.get_subcommands() {
            let mut below = path.to_vec();
            below.push(child.get_name().to_owned());
            visit(child, &below, into);
        }
    }

    /// Every node of the tree below `tpl`, in the order `FR-HELP-019` requires.
    fn nodes() -> Vec<Vec<String>> {
        let mut paths = Vec::new();
        visit(&tree(), &[], &mut paths);

        paths
    }

    /// The node `path` reaches.
    fn at<'a>(tree: &'a Command, path: &[&str]) -> &'a Command {
        let mut node = tree;

        for segment in path {
            node = node
                .find_subcommand(segment)
                .unwrap_or_else(|| panic!("{path:?} names a node of the tree"));
        }

        node
    }

    /// The path of an entry, as a caller writes it, for a failure message.
    fn written(path: &[&str]) -> String {
        std::iter::once("tpl")
            .chain(path.iter().copied())
            .collect::<Vec<&str>>()
            .join(" ")
    }

    /// The document for the whole tree, serialised in the compact form.
    fn compact(surface: &Surface<'_>) -> String {
        serde_json::to_string(&Document::new(Source::Binary, surface))
            .expect("the document serialises")
    }

    #[test]
    fn br_help_003_every_command_alias_and_flag_of_the_tree_appears_in_the_document() {
        // BR-HELP-003, first property, and the one that makes FR-HELP-021 a
        // contract rather than an intention: the document is compared against
        // the tree the binary parses with, node by node, so a node, an alias
        // or a flag added to the tree and missing from the document fails
        // here rather than leaving a caller unable to discover it.
        let tree = tree();
        let published = surface(&tree, &tree, &[]).expect("every node has an entry");

        assert_eq!(
            published
                .commands
                .iter()
                .map(|entry| entry.path.iter().map(|&s| s.to_owned()).collect())
                .collect::<Vec<Vec<String>>>(),
            nodes(),
            "the document does not carry exactly the nodes of the tree, in order"
        );

        for entry in &published.commands {
            let node = at(&tree, &entry.path);
            let at = written(&entry.path);

            assert_eq!(
                entry.aliases,
                node.get_all_aliases().collect::<Vec<&str>>(),
                "{at} does not carry its aliases"
            );

            assert_eq!(
                entry
                    .options
                    .iter()
                    .map(|flag| flag.long.clone())
                    .collect::<Vec<String>>(),
                node.get_arguments()
                    .filter(|argument| !argument.is_positional())
                    .map(|argument| format!("--{}", argument.get_long().expect("a long form")))
                    .collect::<Vec<String>>(),
                "{at} does not carry its flags"
            );

            assert_eq!(
                entry
                    .arguments
                    .iter()
                    .map(|argument| argument.name)
                    .collect::<Vec<&str>>(),
                node.get_arguments()
                    .filter(|argument| argument.is_positional())
                    .map(super::render::value_name)
                    .collect::<Vec<&str>>(),
                "{at} does not carry its positional arguments"
            );
        }

        assert_eq!(
            published
                .global_flags
                .iter()
                .map(|flag| flag.long.as_str())
                .collect::<Vec<&str>>(),
            GLOBALS
        );

        // The comparison above is against the tree, which is the source of
        // both sides; this pins the count against the requirement, so a tree
        // that lost an alias would fail here rather than agreeing with itself.
        assert_eq!(
            published
                .commands
                .iter()
                .map(|entry| entry.aliases.len())
                .sum::<usize>(),
            7,
            "FR-CLI-011 declares seven aliases and no others"
        );
    }

    #[test]
    fn br_help_003_every_command_of_the_document_carries_at_least_one_example() {
        // BR-HELP-003, second property, asserted on the document rather than
        // on the table it is read from: the table's own test covers the root,
        // which FR-HELP-019 does not publish, and this one covers exactly what
        // a caller loading the surface receives.
        let tree = tree();
        let published = surface(&tree, &tree, &[]).expect("every node has an entry");

        for entry in &published.commands {
            assert!(
                !entry.examples.is_empty(),
                "{} carries no example",
                written(&entry.path)
            );
        }
    }

    #[test]
    fn br_help_003_every_example_of_the_document_parses_through_the_command_parser() {
        // BR-HELP-003, third property: an example that does not parse is worse
        // than no example. The vector published as `invocation` is the one an
        // agent runs, so it is that vector that is parsed — not the `text`
        // beside it, which carries the shell the example is written in.
        let tree = tree();
        let published = surface(&tree, &tree, &[]).expect("every node has an entry");

        for entry in &published.commands {
            for example in &entry.examples {
                for line in &example.lines {
                    let Some(invocation) = line.invocation else {
                        continue;
                    };

                    assert_eq!(
                        invocation.first(),
                        Some(&"tpl"),
                        "an example of {} does not begin with tpl",
                        written(&entry.path)
                    );
                    assert!(
                        parse(invocation.iter().copied()).is_ok(),
                        "the example '{}' of {} does not parse",
                        line.text,
                        written(&entry.path)
                    );
                }
            }
        }
    }

    #[test]
    fn fr_help_018_the_global_flags_are_carried_once_and_no_command_repeats_them() {
        // FR-HELP-018 and FR-GLOB-003: the seven are listed once, and each
        // command says `inherits_globals` instead of repeating them. The
        // property holds by construction — the seven are declared on the root
        // and propagated only when the parser is built — and is pinned here
        // because a property held by construction is one nobody notices
        // losing.
        let tree = tree();
        let published = surface(&tree, &tree, &[]).expect("every node has an entry");

        for entry in &published.commands {
            assert!(
                entry.inherits_globals,
                "{} does not inherit the global flags",
                written(&entry.path)
            );

            for flag in &entry.options {
                assert!(
                    !GLOBALS.contains(&flag.long.as_str()),
                    "{} repeats the global flag {}",
                    written(&entry.path),
                    flag.long
                );
            }
        }
    }

    #[test]
    fn fr_help_029_a_subtree_is_the_node_and_its_descendants_in_the_order_the_whole_document_holds()
    {
        // FR-HELP-029: `data.commands` reduced to the entry whose path is the
        // path given, together with every entry whose path extends it segment
        // by segment, in the order those entries hold unreduced.
        let tree = tree();
        let node = at(&tree, &["cfg", "database"]);
        let reduced = surface(&tree, node, &["cfg", "database"]).expect("every node has an entry");

        assert_eq!(
            reduced
                .commands
                .iter()
                .map(|entry| entry.path.clone())
                .collect::<Vec<Vec<&str>>>(),
            vec![
                vec!["cfg", "database"],
                vec!["cfg", "database", "add"],
                vec!["cfg", "database", "list"],
                vec!["cfg", "database", "show"],
                vec!["cfg", "database", "update"],
                vec!["cfg", "database", "remove"],
                vec!["cfg", "database", "test"],
            ]
        );

        // And the other three keys of `data` are emitted unreduced, which is
        // the whole of what FR-HELP-029 leaves alone and what FR-HELP-017
        // requires of every document of this form.
        let whole = surface(&tree, &tree, &[]).expect("every node has an entry");

        assert_eq!(reduced.tpl_version, whole.tpl_version);
        assert_eq!(reduced.global_flags.len(), GLOBALS.len());
        assert_eq!(
            serde_json::to_string(&reduced.template_surface).expect("it serialises"),
            serde_json::to_string(&whole.template_surface).expect("it serialises")
        );
    }

    #[test]
    fn fr_out_024_the_document_carries_the_keys_of_the_requirement_in_the_order_it_fixes() {
        // FR-OUT-024 for the envelope, FR-HELP-017 for the four keys of `data`,
        // and FR-HELP-019 for the eight members of an entry. Asserting the
        // emitted bytes rather than the types is what covers FR-HELP-023 and
        // FR-OUT-013 as well: a map on this path would reorder them, and
        // nothing here is a map.
        let tree = tree();
        let published = surface(&tree, &tree, &[]).expect("every node has an entry");
        let document = compact(&published);

        assert!(
            document.starts_with(concat!(
                r#"{"schema_version":1,"source":"binary","data":{"tpl_version":""#,
                env!("CARGO_PKG_VERSION"),
                r#"","global_flags":[{"long":"--database","#
            )),
            "{document}"
        );

        assert!(
            document.contains(r#""commands":[{"path":["schema"],"aliases":[],"description":"#),
            "an entry does not carry its members in the order of FR-HELP-019"
        );

        assert!(
            document.contains(r#""examples":[{"caption":"Print this help.","#),
            "an entry does not carry its examples after its options"
        );
        assert!(
            document.contains(r#""exit_codes":[{"code":0,"name":"EX_OK","#),
            "an entry does not carry its exit codes after its examples"
        );
        assert!(
            document.contains(r#""inherits_globals":true"#),
            "an entry does not carry the marker of FR-HELP-018"
        );

        let at = |key: &str| document.find(key).expect("the key is present");

        assert!(at(r#""tpl_version""#) < at(r#""global_flags""#));
        assert!(at(r#""global_flags""#) < at(r#""commands""#));
        assert!(at(r#""commands""#) < at(r#""template_surface""#));
    }

    #[test]
    fn fr_env_005_the_template_surface_is_the_three_groups_of_the_requirement() {
        // FR-ENV-005: three sibling objects of one shape, each carrying
        // `guarantee`, `filters`, `tests` and `functions` in that order, with
        // the guarantee its table gives the group. The four arrays `render/`
        // fills carry the names of FR-ENV-006 followed by those of FR-ENV-007,
        // then those of FR-ENV-014, FR-ENV-020 and FR-ENV-018, each in the
        // order its requirement states them, per FR-HELP-023. The two
        // FR-ENV-018 and FR-ENV-019 fix as empty are empty in every document,
        // and the three of group 3 are `null` in every document.
        assert_eq!(
            serde_json::to_string(&TemplateSurface::PUBLISHED).expect("it serialises"),
            concat!(
                r#"{"registered":{"guarantee":"contract","filters":["pascal","camel","snake","#,
                r#""upper_snake","kebab","quote","sql_type","json","indent","comment","escape"],"#,
                r#""tests":["nullable","primary_key","auto_increment","unique","numeric",""#,
                r#"temporal","textual"],"functions":["table","view","routine","column","fail"]},"#,
                r#""inherited":{"guarantee":"pinned","filters":["default","join","length","map","#,
                r#""select","reject","first","last","reverse","sort","trim","upper","lower","#,
                r#""replace"],"tests":[],"functions":[]},"other":{"guarantee":"none","#,
                r#""filters":null,"tests":null,"functions":null}}"#
            )
        );
    }

    #[test]
    fn fr_env_005_the_published_arrays_are_the_ones_the_environment_registers() {
        // FR-ENV-005 derives `registered` from the registrations the
        // environment actually performs. The document holds no copy of the
        // names, so this asserts the derivation rather than the names — which
        // `render/` asserts against their requirements.
        let published = TemplateSurface::PUBLISHED;

        assert_eq!(
            published.registered.filters,
            Some(crate::render::REGISTERED_FILTERS)
        );
        assert_eq!(
            published.registered.tests,
            Some(crate::render::REGISTERED_TESTS)
        );
        assert_eq!(
            published.registered.functions,
            Some(crate::render::REGISTERED_FUNCTIONS)
        );
        assert_eq!(
            published.inherited.filters,
            Some(crate::render::INHERITED_FILTERS)
        );
    }

    #[test]
    fn fr_glob_024_every_flag_of_the_tree_declares_a_long_form_and_every_value_a_named_type() {
        // Two properties the document asserts about the tree rather than about
        // itself. FR-GLOB-024 gives five flags of the whole tool a short form
        // and gives none of them a spelling without a long one, so the
        // identifier this module falls back to is a fallback that is never
        // taken. And a value whose type this module cannot name would reach a
        // caller as the bare word `value`, which says nothing it can act on.
        let tree = tree();
        let published = surface(&tree, &tree, &[]).expect("every node has an entry");

        let flags = published
            .commands
            .iter()
            .flat_map(|entry| entry.options.iter())
            .chain(published.global_flags.iter());

        for flag in flags {
            assert!(flag.long.starts_with("--"), "{flag:?}");
            assert_ne!(flag.value_type, Some("value"), "{flag:?}");
        }

        for argument in published
            .commands
            .iter()
            .flat_map(|entry| entry.arguments.iter())
        {
            assert_ne!(argument.value_type, "value", "{argument:?}");
        }
    }
}
