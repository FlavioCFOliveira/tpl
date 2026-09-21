//! The parser tree: every node of `FR-CLI-002`, the seven aliases of
//! `FR-CLI-011`, the six group nodes of `FR-CLI-008`, and the seven global
//! flags of `FR-GLOB-001`.
//!
//! The tree is **closed**, per `FR-CLI-002`, and closing it is what this module
//! is for. Five settings hold it closed, and [`closed`] sets every one of them
//! at every node:
//!
//! | Setting | State | Required by |
//! |---|---|---|
//! | `infer_subcommands` | off | `FR-CLI-004` — `tpl sch tables` is not `tpl schema tables` |
//! | `infer_long_args` | off | `FR-CLI-005` — `--data` is not `--database` |
//! | `allow_external_subcommands` | off | `FR-CLI-006` — an undeclared token is never handed on for a `tpl-<token>` lookup |
//! | `disable_help_flag` | on | `OD-07` — `tpl` renders all seven sections of `FR-HELP-006` itself, so `-h/--help` is the global flag `FR-GLOB-001` declares |
//! | `disable_help_subcommand` | on | `OD-07` again — `help` is a real subcommand of the tree, per `FR-HELP-004` |
//!
//! The first three are `clap`'s defaults today. They are set all the same, and
//! asserted by a test: a default is the one property of a dependency that can
//! change without anything in this project changing, and `FR-CLI-002` is a
//! guarantee this project makes rather than one it inherits.
//!
//! **Where the global flags come from.** They are declared once, in
//! [`globals`], with `global = true`, and read from the root of the parsed
//! invocation. That is what makes `FR-CLI-024` hold: a global argument is
//! propagated down to every node when the parser is built, and the value it
//! matched at any depth is propagated back up to the root. `-d shop` before the
//! command, between a command and its subcommand, and after the positional
//! arguments are therefore the same invocation, which is the way a calling
//! agent builds a command line.
//!
//! **Where the local arguments come from.** Every positional argument and
//! every local flag of the twenty-nine leaves is declared on the node that
//! owns it, in the module that declares that node, and read from the
//! specification module that owns the command. A flag that more than one node
//! declares is written once and flattened by each of them — the four of
//! `FR-GLOB-021` and the three object flags in [`local`], `--pattern` in
//! [`schema`], the nine entry flags in [`cfg`] — so that one row of a
//! requirement's table remains one declaration in the tree. Nothing is global
//! but the seven, per `FR-GLOB-001`, and nothing carries a short form but their
//! five, per `FR-GLOB-024`.
//!
//! Each argument carries its own value type, default, requiredness, enumerated
//! values and repeatability, because `FR-HELP-013` obliges the help to state
//! all five and `FR-HELP-021` derives the JSON command tree by introspecting
//! this tree rather than from a second source. What binds one argument to
//! another — `FR-OUT-009`, `FR-RND-005`, `FR-CFG-016`, `FR-CFG-029` — is not
//! declared here, for the reason [`globals`] gives for `FR-CLI-015`: a refusal
//! written in the parser's words is a refusal the caller never reads in the
//! four labelled lines of `FR-ERR-008`. The first of the four is applied by
//! [`rules`], over the declarations of this tree, and reaches every node that
//! declares both flags without naming one of them.
//!
//! # The interim arrangement
//!
//! One thing in this module is deliberately provisional, and it is named here
//! so that it is not mistaken for finished work.
//!
//! **A leaf whose work is a later sprint reports `70`.** [`route`] gives each
//! such leaf an arm that returns [`Error::InternalInvariant`] naming its
//! command path, which `FR-ERR-001` makes exit `70` — the code for a condition
//! the caller cannot have caused and cannot correct, which is exactly what a
//! parsed command with no implementation is. The arrangement is an arm per leaf
//! rather than one catch-all so that each later sprint replaces **its own**
//! entry, and so that the arm it must replace is named by its path rather than
//! found by reading. What remains under it is `tpl cfg database test`, the one
//! `cfg` subcommand that contacts a server.
//!
//! Nothing else is provisional here.
//!
//! - `tpl help` and `tpl version` are the two leaves that act: [`help::command`]
//!   resolves a command path of any depth and writes either the seven sections
//!   of `FR-HELP-006` or the JSON command tree of `FR-HELP-016`, and [`version`]
//!   writes the line of `FR-HELP-005`.
//! - `FR-CLI-007` and `FR-HELP-025` require a group node invoked with no child
//!   to print exactly the text `tpl help <node>` would print, and to exit `0`:
//!   [`node_help`] renders that text through [`help::text`] and writes it
//!   through [`output`].
//! - `FR-GLOB-019` and `FR-GLOB-020` answer `-h/--help` and `-V/--version` at
//!   whatever node they were given at, through those same two functions.
//!
//! **The six forms of `FR-HELP-001` therefore reach two functions between
//! them**, which is what makes the three equivalences of `FR-HELP-002` hold by
//! construction rather than by comparison. A test walks the tree and asserts
//! the bytes all the same, because a property held by construction is one
//! nobody notices losing.
//!
//! [`parse`] is the whole of the route from the process to the tree, and
//! [`crate::run`] takes it: it builds the tree, hands it the vector, and turns
//! whatever comes back into the one thing the caller reads — an [`Invocation`],
//! or an [`Error`] the four labelled lines of `FR-ERR-008` are composed from.
//!
//! # The parsing rules
//!
//! `FR-ERR-006` makes argument parsing step 1 of the validation order, for
//! every command without exception, and the step is three things in one:
//! whatever the parser refuses, which [`intercept`] re-renders per `OD-08`;
//! then the repetition of `FR-CLI-014`; then the pair of `FR-CLI-015`; then
//! `--pretty` without `--format json`, per `FR-OUT-009`. [`rules`] owns the
//! last three and states the order among the four.
//!
//! Two of the six rules are the parser's own behaviour, and are therefore
//! asserted here rather than implemented: `FR-CLI-017` terminates the arguments
//! at `--` on every command, and `FR-CLI-020` matches a flag and its value byte
//! for byte. A default is the one property of a dependency that can change
//! without anything in this project changing, which is why each is a test.

mod cache;
mod cfg;
mod globals;
mod help;
mod intercept;
mod layout;
mod local;
mod render;
mod rules;
mod schema;
mod source;
mod template;

use std::ffi::OsString;
use std::io::Write;
use std::path::PathBuf;

use clap::error::ErrorKind;
use clap::{ArgAction, CommandFactory, FromArgMatches, Parser, Subcommand};

use cache::Cache;
use cfg::Cfg;
use globals::Globals;
use schema::Schema;
use template::Template;

use crate::diagnostics::verbosity::Level;
use crate::error::{self, Error};
use crate::output;
use crate::project;

/// The interim outcome of a leaf whose implementation is a later sprint.
///
/// It expands to the guard of `FR-ERR-031`, which is the one place that decides
/// a violated invariant is a `70` and the one that names where it was detected.
/// The path is written once and carries into both the invariant and the arm a
/// later sprint replaces.
macro_rules! not_yet_implemented {
    ($path:literal) => {
        error::ensure_invariant(
            false,
            concat!("the command '", $path, "' has an implementation"),
        )
    };
}

/// One parsed invocation: the seven global flags, and the node they were given
/// at.
///
/// [`Cli::command`] is optional because `tpl` is itself a group node, per
/// `FR-CLI-008`: a bare `tpl` is not a usage error but a request for the
/// top-level help, per `FR-CLI-007`.
#[derive(Debug, Clone, PartialEq, Eq, Parser)]
#[command(name = "tpl")]
pub(crate) struct Cli {
    /// The seven flags of `FR-GLOB-001`, wherever on the command line they
    /// appeared.
    #[command(flatten)]
    pub(crate) globals: Globals,

    /// The command invoked, or [`None`] for a bare `tpl`.
    #[command(subcommand)]
    pub(crate) command: Option<Command>,
}

/// What one parsed invocation resolved to: the seven global flags, and the
/// form the rest of the vector names.
///
/// [`Cli`] is what the parser produces and this is what the process acts on.
/// They differ because two of the six forms of `FR-HELP-001` carry no command
/// at all: `-h/--help` and `-V/--version` are answered at whatever node they
/// were given at, per `FR-GLOB-019` and `FR-GLOB-020`, and a node that declares
/// a required operand has none to supply when the caller is asking for the help
/// in order to learn what the operand is.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct Invocation {
    /// The seven flags of `FR-GLOB-001`, wherever on the command line they
    /// appeared.
    pub(crate) globals: Globals,

    /// What the invocation asks for.
    pub(crate) form: Form,
}

/// The three things an invocation the parser accepted can be.
///
/// The first two are the flag forms of `FR-HELP-001`, and neither reaches a
/// command: they are answered from the tree and the typed table alone, which is
/// what `FR-PROJ-025` requires of them.
// Exactly one value of this type exists per process, built once by `parse` and
// read once by `route`, so the size difference between the two flag forms and
// the command they are alternatives to is never multiplied by anything. Boxing
// the command to level the variants would add a heap allocation and an
// indirection to the path every ordinary invocation takes, against a saving of
// a couple of hundred bytes of stack on a value that is constructed once — which
// is the trade this project's own measurement rule refuses to make without
// evidence.
#[allow(
    clippy::large_enum_variant,
    reason = "one value per process: levelling the variants would buy stack this process does \
              not want and cost an allocation on the ordinary path"
)]
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum Form {
    /// `-h/--help` was given: the canonical path of the node it was answered
    /// at, as the segments below `tpl` (`FR-GLOB-019`).
    Help(Vec<String>),

    /// `-V/--version` was given (`FR-GLOB-020`).
    Version,

    /// Neither flag was given: the command the vector names, or [`None`] for a
    /// bare `tpl`, which `FR-CLI-007` answers with the top-level help.
    Command(Option<Command>),
}

/// The eight top-level commands of `FR-CLI-010`, and no others.
///
/// None of them carries an alias: `FR-CLI-012` refuses every top-level alias,
/// so `tpl s`, `tpl t` and `tpl r` are `64`.
#[derive(Debug, Clone, PartialEq, Eq, Subcommand)]
pub(crate) enum Command {
    /// Reads the structure of the selected database.
    Schema(Schema),

    /// Reads the templates the project carries.
    Template(Template),

    /// Renders one template, once.
    Render {
        /// The template to render, with or without its `.jinja` extension
        /// (`FR-RND-001`).
        ///
        /// Exactly one, because one invocation produces exactly one render,
        /// per `FR-RND-002`. It is resolved by the rules of `FR-TMPL-006` and
        /// `FR-TMPL-007`, and a name that does not resolve is `66`, per
        /// `FR-RND-029`.
        #[arg(value_name = "TEMPLATE")]
        template: String,

        /// `--table`, `--view` and `--routine`, per `FR-RND-003`.
        #[command(flatten)]
        object: local::Object,

        /// One entry of the `vars` context variable (`FR-RND-008`).
        ///
        /// Repeatable, with distinct keys: a repeated key is `64`, per
        /// `FR-RND-014`. The argument is split on its first `=`, per
        /// `FR-RND-009`, the key must match `[A-Za-z_][A-Za-z0-9_]*`, per
        /// `FR-RND-012`, and the value is always a string, never a number or a
        /// boolean inferred from it, per `FR-RND-015`. All three are the
        /// command's to apply, so the argument is carried here as written.
        #[arg(long = "set", value_name = "KEY=VALUE", action = ArgAction::Append)]
        set: Vec<String>,

        /// The JSON document supplying the server-derived context, instead of
        /// a database (`FR-RND-016`).
        ///
        /// `-` reads the document from stdin, per `FR-RND-017`. It is
        /// incompatible with `-d/--database` given explicitly on the command
        /// line, per `FR-RND-018`, which is a refusal between two arguments
        /// and so the command's rather than the parser's.
        ///
        /// Every occurrence, for the reason [`globals`] gives: it carries a
        /// value, so `FR-CLI-014` refuses a second one over the occurrences
        /// this declaration accumulates.
        #[arg(long = "context", value_name = "PATH", action = ArgAction::Append)]
        context: Vec<PathBuf>,

        /// `--direct` and `--no-cache`, per `FR-RND-025`.
        ///
        /// There is no `--format` and no `--pretty` beside them: the result is
        /// the rendered text, which has no alternative representation, per
        /// `FR-RND-027`.
        #[command(flatten)]
        caching: local::Caching,
    },

    /// The catalogue cache.
    Cache(Cache),

    /// The `.tpl/.cfg` file.
    Cfg(Cfg),

    /// Creates a `.tpl` project.
    Init {
        /// Where to create the project (`FR-PROJ-012`).
        ///
        /// Optional, defaulting to the current directory. The destination and
        /// any missing parent of it are created, per `FR-PROJ-013`, and a
        /// destination that already holds a `.tpl` folder is `73` with nothing
        /// changed, per `FR-PROJ-014`.
        #[arg(value_name = "PATH", default_value = ".")]
        path: PathBuf,
    },

    /// Prints help, and the whole command tree as JSON.
    Help {
        /// The node whose help to print, as the sequence of segments that
        /// reaches it (`FR-HELP-026`).
        ///
        /// Any depth of the tree, so `tpl help cfg database add` is valid.
        /// Each segment is resolved by the rules of the tree — an alias
        /// resolves to its canonical node, and no segment is inferred from a
        /// prefix — per `FR-HELP-027`, and a segment naming no child of the
        /// node reached is `64`, per `FR-HELP-028`. Given none, the top-level
        /// help is printed, per `FR-HELP-001`.
        #[arg(value_name = "COMMAND_PATH")]
        command_path: Vec<String>,

        /// `--format` and `--pretty`, per `FR-GLOB-021`.
        ///
        /// `--format json` is what turns this command into the JSON command
        /// tree of `FR-HELP-016`, reduced to the subtree the path names, per
        /// `FR-HELP-029`.
        #[command(flatten)]
        output: local::Output,
    },

    /// Prints the version.
    ///
    /// It declares nothing: `FR-HELP-005` fixes the output as exactly
    /// `tpl <version>` and a single newline, which admits no argument and no
    /// choice of representation.
    Version,
}

/// The parser tree, closed at every node by [`closed`].
///
/// This is the single construction site of the tree, and the only one [`parse`]
/// and the tests use: a setting applied here is applied to every node, and a
/// node added to the tree acquires it without anyone remembering to.
pub(crate) fn tree() -> clap::Command {
    closed(Cli::command())
}

/// Applies the five settings of this module's table to `command` and to every
/// node beneath it.
///
/// They are applied by recursion over the declared tree rather than by an
/// attribute on each node, because an attribute is a thing a new node can be
/// written without, and a node written without one would accept what
/// `FR-CLI-002` closes the tree against.
fn closed(command: clap::Command) -> clap::Command {
    let children: Vec<String> = command
        .get_subcommands()
        .map(|child| child.get_name().to_owned())
        .collect();

    children.into_iter().fold(
        command
            .infer_subcommands(false)
            .infer_long_args(false)
            .allow_external_subcommands(false)
            .disable_help_flag(true)
            .disable_help_subcommand(true),
        |command, name| command.mut_subcommand(name, closed),
    )
}

/// Parses one argument vector against the tree, and applies the parsing rules.
///
/// The vector is the whole invocation, `argv[0]` included, as the process
/// receives it. This is the only route from the process to the parser, and it
/// is what makes `OD-08` hold: the `clap::Error` is intercepted here and never
/// rendered, so no byte the parser composes reaches either stream.
///
/// # Errors
///
/// Returns the [`Error`] of the first condition of step 1 of `FR-ERR-006` that
/// fails, in the order [`rules`] states: what the parser itself refused, then
/// the repetition of `FR-CLI-014`, then the pair of `FR-CLI-015`. Every one of
/// them exits `64`.
pub(crate) fn parse<I, T>(arguments: I) -> Result<Invocation, Error>
where
    I: IntoIterator<Item = T>,
    T: Into<OsString>,
{
    let argv: Vec<OsString> = arguments.into_iter().map(Into::into).collect();

    // The tree is built once and kept: `intercept` reads the node an invocation
    // reached from it, and `rules` reads the declarations from it.
    let mut tree = tree();
    let refused = match tree.try_get_matches_from_mut(&argv) {
        Ok(matches) => return interpret(&tree, &matches, &argv),
        Err(refused) => refused,
    };

    if refused.kind() == ErrorKind::MissingRequiredArgument
        && let Some(flagged) = flagged(&argv)?
    {
        return Ok(flagged);
    }

    Err(intercept::intercepted(&refused, &tree, &argv))
}

/// Applies the parsing rules to an accepted invocation and says what it is.
///
/// The three rules of [`rules`] run before the invocation is built, and in the
/// order that module states, so `FR-ERR-006` step 1 is complete for every form
/// alike — including the two that never reach a command.
///
/// # Errors
///
/// Returns the [`Error`] of the first rule that refuses, and the intercepted
/// refusal where the matched vector does not satisfy the declarations.
fn interpret(
    tree: &clap::Command,
    matches: &clap::ArgMatches,
    argv: &[OsString],
) -> Result<Invocation, Error> {
    rules::refuse_repetition(tree, matches)?;

    let globals = Globals::from_arg_matches(matches)
        .map_err(|refused| intercept::intercepted(&refused, tree, argv))?;

    rules::refuse_both_verbosities(&globals)?;

    // Before the two flag forms are answered, because `FR-ERR-006` runs step 1
    // "for every command without exception": `tpl schema tables --help
    // --pretty` is `64` for the same reason an unknown flag on `tpl --help`
    // is.
    rules::refuse_pretty_without_json(tree, matches)?;

    if globals.help || globals.version {
        return Ok(requested(globals, matches));
    }

    let Cli { globals, command } = Cli::from_arg_matches(matches)
        .map_err(|refused| intercept::intercepted(&refused, tree, argv))?;

    Ok(Invocation {
        globals,
        form: Form::Command(command),
    })
}

/// The invocation `-h/--help` or `-V/--version` names, with help winning where
/// both were given.
///
/// `FR-GLOB-019` and `FR-GLOB-020` give each flag an outcome and the corpus
/// states no precedence between them, so one is chosen here: help, because it
/// is the form that names the other and a caller that asked for both asked for
/// the larger answer. A refusal of the pair was rejected — `FR-CLI-015` is the
/// only pair this corpus refuses, and inventing a second would make an
/// invocation fail that every requirement in force accepts.
fn requested(globals: Globals, matches: &clap::ArgMatches) -> Invocation {
    let form = if globals.help {
        Form::Help(reached(matches))
    } else {
        Form::Version
    };

    Invocation { globals, form }
}

/// The help or version form a vector the parser refused for a missing argument
/// nonetheless names, or [`None`] where it names neither.
///
/// `OD-07` turns the parser's own help flag off, so `-h`, `--help`, `-V` and
/// `--version` are ordinary global arguments of [`globals`] and the parser
/// validates **required arguments first**. At the thirteen leaves that declare
/// one of the fourteen required operands of the tree, the strict parse
/// therefore refuses `tpl <node> --help` before the flag is ever read, which
/// contradicts `FR-HELP-002` — the three help forms are byte-identical at every
/// depth — and `FR-GLOB-019` and `FR-GLOB-020`, which give both flags an
/// outcome at every node.
///
/// The vector is parsed a second time, against **the same tree** with the
/// requirement waived, and the result is honoured only where one of the two
/// flags is present. Nothing else is decided by it: a vector that names neither
/// flag falls through to the refusal the strict parse produced, with its
/// message intact.
///
/// This is not the argument pre-scan `FR-ERR-017` and `FR-ERR-018` withdrew,
/// and not the second set of rules `OD-08` rejects: it is the project's own
/// parser, over the project's own tree, differing in one validation setting,
/// and it runs on a path that has already failed.
///
/// # Errors
///
/// Returns the [`Error`] of the first rule of [`rules`] that refuses the
/// vector, because step 1 of `FR-ERR-006` runs for every command without
/// exception and a help form is no exception to it.
fn flagged(argv: &[OsString]) -> Result<Option<Invocation>, Error> {
    let mut waived = waived(tree());

    // Either refusal below means the waiver did not turn the vector into a
    // help form, so nothing here classifies it: the strict parse's own refusal
    // is returned by the caller, which is the message the caller composed for
    // exactly this vector.
    let Ok(matches) = waived.try_get_matches_from_mut(argv) else {
        return Ok(None);
    };

    let Ok(globals) = Globals::from_arg_matches(&matches) else {
        return Ok(None);
    };

    if !globals.help && !globals.version {
        return Ok(None);
    }

    rules::refuse_repetition(&waived, &matches)?;
    rules::refuse_both_verbosities(&globals)?;
    rules::refuse_pretty_without_json(&waived, &matches)?;

    Ok(Some(requested(globals, &matches)))
}

/// `command`, and every node beneath it, with the requirement waived from every
/// argument that declares one.
///
/// The waiver exists for [`flagged`] alone and reaches nothing a caller can
/// observe: [`tree`] is unchanged, so `Arg::is_required_set` still reports the
/// requirement to the help of `FR-HELP-013` and to the JSON command tree of
/// `FR-HELP-021`, and a vector that supplies neither the operand nor one of the
/// two flags is refused by the strict parse with the message it already had.
fn waived(command: clap::Command) -> clap::Command {
    let children: Vec<String> = command
        .get_subcommands()
        .map(|child| child.get_name().to_owned())
        .collect();

    children.into_iter().fold(
        command.mut_args(|argument| argument.required(false)),
        |command, name| command.mut_subcommand(name, waived),
    )
}

/// The canonical path of the node a matched invocation reached, as the segments
/// below `tpl`.
///
/// It is read from the matches rather than from the built [`Cli`], which is
/// what lets a help form name its node without the node's own operands being
/// present. The segments are the parser's own names, so an alias of
/// `FR-CLI-011` has already resolved to the node it names, per `FR-HELP-027`.
fn reached(matches: &clap::ArgMatches) -> Vec<String> {
    let mut path = Vec::new();
    let mut node = matches;

    while let Some((name, inner)) = node.subcommand() {
        path.push(name.to_owned());
        node = inner;
    }

    path
}

/// The diagnostic level an invocation resolves to (`FR-GLOB-014`,
/// `FR-GLOB-015`, `FR-CLI-016`).
///
/// `OD-17` resolves it once, during argument handling; the process fixes it for
/// the rest of the run, and nothing else reads the two flags.
pub(crate) fn level(invocation: &Invocation) -> Level {
    rules::level(&invocation.globals)
}

/// Parses one argument vector against the tree, and applies no rule of `tpl`.
///
/// It is the raw parse, and it exists for the tests that observe **what the
/// parser itself refuses** — the closure of `FR-CLI-002`, the two inferences of
/// `FR-CLI-004` and `FR-CLI-005`, the unknown flag of `FR-CLI-019`. Every other
/// test, and the process, go through [`parse`].
///
/// # Errors
///
/// Returns the `clap::Error` the parser raised, unclassified.
#[cfg(test)]
pub(crate) fn parse_from<I, T>(arguments: I) -> Result<Cli, clap::Error>
where
    I: IntoIterator<Item = T>,
    T: Into<OsString> + Clone,
{
    let matches = tree().try_get_matches_from(arguments)?;

    Cli::from_arg_matches(&matches)
}

/// Runs the form one parsed invocation names.
///
/// # Errors
///
/// Returns whatever the form reports. Until a node is implemented that is
/// [`Error::InternalInvariant`], per the interim arrangement this module's
/// documentation states.
pub(crate) fn dispatch(invocation: &Invocation) -> Result<(), Error> {
    route(&mut std::io::stdout().lock(), invocation)
}

/// The one match from a parsed invocation to what it asks for.
///
/// The two flag forms are answered first and by the same two functions every
/// other form reaches them through, which is what makes the equivalences of
/// `FR-HELP-002` hold by construction rather than by comparison: there is one
/// help renderer and one version line, and `tpl help <path>`, `tpl <path>
/// --help`, `tpl <path> -h` and a bare group node all arrive at the first,
/// while `tpl version`, `tpl --version` and `tpl -V` all arrive at the second.
///
/// Below them every arm is either a group node, which prints its own help and
/// succeeds per `FR-CLI-007`, or a leaf, which reports the interim `70` until
/// the sprint that owns it replaces the arm.
///
/// The writer is a parameter rather than standard output taken directly, so
/// that what a node prints is observable from a test without a process.
///
/// # Errors
///
/// Returns [`Error::InternalInvariant`] for a leaf with no implementation,
/// whatever `tpl help` reports for a path that names no node, and
/// [`Error::StdoutUnwritable`] where the text could not be written.
fn route<W: Write>(out: &mut W, invocation: &Invocation) -> Result<(), Error> {
    let command = match &invocation.form {
        // FR-GLOB-019 and FR-GLOB-020: both flags are answered at the node
        // they were given at, whatever that node is, and neither reaches a
        // command.
        Form::Help(path) => {
            let path: Vec<&str> = path.iter().map(String::as_str).collect();

            return node_help(out, &path);
        }
        Form::Version => return version(out),
        Form::Command(command) => command,
    };

    match command {
        None => node_help(out, &[]),

        Some(Command::Schema(read)) => match &read.command {
            None => node_help(out, &["schema"]),
            Some(command) => schema::run(out, &invocation.globals, command),
        },

        Some(Command::Template(read)) => match &read.command {
            None => node_help(out, &["template"]),
            Some(command) => template::run(out, &invocation.globals, command),
        },

        Some(Command::Render {
            template,
            object,
            set,
            context,
            caching,
        }) => render::run(
            out,
            &invocation.globals,
            &render::Supplied {
                template,
                object,
                set,
                context,
                caching,
            },
        ),

        Some(Command::Cache(store)) => match &store.command {
            None => node_help(out, &["cache"]),
            Some(command) => cache::run(out, &invocation.globals, command),
        },

        Some(Command::Cfg(config)) => {
            let globals = &invocation.globals;

            match &config.command {
                None => node_help(out, &["cfg"]),
                Some(cfg::Command::Get { key, output }) => {
                    cfg::keys::get(out, &cfg::Supplied::new(globals, Some(output)), key)
                }
                Some(cfg::Command::Set { key, value }) => {
                    cfg::keys::set(&cfg::Supplied::new(globals, None), key, value)
                }
                Some(cfg::Command::Unset { key }) => {
                    cfg::keys::unset(&cfg::Supplied::new(globals, None), key)
                }
                Some(cfg::Command::List { output }) => {
                    cfg::keys::list(out, &cfg::Supplied::new(globals, Some(output)))
                }
                Some(cfg::Command::Database(database)) => match &database.command {
                    None => node_help(out, &["cfg", "database"]),
                    Some(cfg::DatabaseCommand::Add { name, entry }) => {
                        cfg::entries::add(&cfg::Supplied::new(globals, None), name, &entry.flags())
                    }
                    Some(cfg::DatabaseCommand::List { output }) => {
                        cfg::entries::list(out, &cfg::Supplied::new(globals, Some(output)))
                    }
                    Some(cfg::DatabaseCommand::Show { name, output }) => {
                        cfg::entries::show(out, &cfg::Supplied::new(globals, Some(output)), name)
                    }
                    Some(cfg::DatabaseCommand::Update { name, entry }) => cfg::entries::update(
                        &cfg::Supplied::new(globals, None),
                        name,
                        &entry.flags(),
                    ),
                    Some(cfg::DatabaseCommand::Remove { name }) => {
                        cfg::entries::remove(&cfg::Supplied::new(globals, None), name)
                    }
                    // The one cfg subcommand that contacts a server, per
                    // FR-CFG-005 and FR-CACHE-010, and therefore the one the
                    // sprint that opens a connection owns. FR-CACHE-010 also
                    // keeps the store out of it — the command reaches a server
                    // and reads nothing into the model — and FR-CACHE-011 keeps
                    // the store and the server out of every other arm of this
                    // match and of every `template` subcommand: none of them
                    // constructs a `Cache` or a session.
                    Some(cfg::DatabaseCommand::Test { .. }) => {
                        not_yet_implemented!("tpl cfg database test")
                    }
                },
            }
        }

        Some(Command::Init { path }) => project::init::create(path),

        // The two commands of the tree this sprint implements. Both reach the
        // same two functions the flag forms above reach, which is the whole of
        // why `FR-HELP-002` holds: one renderer, one version line, six forms.
        Some(Command::Help {
            command_path,
            output,
        }) => help::command(out, command_path, output),
        Some(Command::Version) => version(out),
    }
}

/// Writes the help of one node, for every form that reaches it by a path the
/// tree has already resolved.
///
/// Three of the six forms of `FR-HELP-001` arrive here: `-h/--help` at any
/// node, per `FR-GLOB-019`; a group node invoked with no child, per
/// `FR-CLI-007` and `FR-HELP-025`; and `tpl help <path>` in its `text` form,
/// which [`help::command`] resolves and then hands on. The fourth,
/// `tpl help <path> --format json`, is the same path resolved by the same
/// resolver and rendered as the document of `FR-HELP-016` instead.
///
/// `path` is the canonical segments below `tpl`, and is empty for the root. The
/// text is the one [`help::text`] composes, which is what `FR-HELP-002` and
/// `FR-HELP-025` require: the one renderer, reached by every form, so the
/// equivalences are a property of the code rather than of a comparison.
///
/// It is written through [`output`], which is the route `FR-OUT-020`
/// gives stdout and which aggregates the bytes behind one buffer.
/// `BR-CLI-005` makes this a legitimate stdout payload, and the caller of this
/// function exits `0`.
///
/// # Errors
///
/// Returns [`Error::StdoutUnwritable`] where the stream refused the write, and
/// [`Error::InternalInvariant`] where the path names no node of the tree or no
/// entry of the table — a disagreement between the tree and the table, which is
/// a defect in `tpl` rather than in the invocation and which a test of
/// [`help`] pins.
fn node_help<W: Write>(out: &mut W, path: &[&str]) -> Result<(), Error> {
    let Some(text) = help::text(path) else {
        return error::ensure_invariant(
            false,
            "every node of the tree carries an entry of the help table",
        );
    };

    output::emit_help(out, &text)
}

/// Writes the version line of `FR-HELP-005`.
///
/// Exactly `tpl <version>` and a single newline, and nothing else. The number
/// is the package's own, read at compile time, so the line cannot disagree with
/// the binary it came from and no second place records it.
///
/// Both forms of `FR-GLOB-020` and the `tpl version` command reach this one
/// function, which is what makes the third equivalence of `FR-HELP-002` hold
/// by construction.
///
/// It is written through [`output`] for the reason [`node_help`] gives: it is a
/// legitimate stdout payload that is not a JSON document, so a consumer that
/// cuts it short is the silent `0` of `FR-ERR-025`.
///
/// # Errors
///
/// Returns [`Error::StdoutUnwritable`] where the stream refused the write.
fn version<W: Write>(out: &mut W) -> Result<(), Error> {
    output::emit_help(out, concat!("tpl ", env!("CARGO_PKG_VERSION"), "\n"))
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use clap::error::ErrorKind;

    use super::{
        Cli, Command, Error, Form, Invocation, Level, cfg, help, local, parse, parse_from, route,
        schema, template, tree,
    };

    /// Every leaf of the tree of `FR-CLI-002`, by the path a caller writes and
    /// the operands that path requires.
    ///
    /// A leaf whose specification module gives it a required positional
    /// argument does not parse without one, so the operands are part of the
    /// identity of the node here. They are the minimal vector: nothing
    /// optional is supplied by any entry.
    const LEAVES: [(&[&str], &[&str]); 29] = [
        (&["schema", "info"], &[]),
        (&["schema", "tables"], &[]),
        (&["schema", "table"], &["orders"]),
        (&["schema", "views"], &[]),
        (&["schema", "view"], &["v_sales"]),
        (&["schema", "routines"], &[]),
        (&["schema", "routine"], &["calc_vat"]),
        (&["schema", "dump"], &[]),
        (&["template", "list"], &[]),
        (&["template", "show"], &["rust/struct"]),
        (&["template", "check"], &[]),
        (&["template", "path"], &[]),
        (&["render"], &["rust/struct"]),
        (&["cache", "load"], &[]),
        (&["cache", "clean"], &[]),
        (&["cache", "status"], &[]),
        (&["cfg", "get"], &["core.database"]),
        (&["cfg", "set"], &["core.database", "shop"]),
        (&["cfg", "unset"], &["core.database"]),
        (&["cfg", "list"], &[]),
        (&["cfg", "database", "add"], &["shop"]),
        (&["cfg", "database", "list"], &[]),
        (&["cfg", "database", "show"], &["shop"]),
        (&["cfg", "database", "update"], &["shop"]),
        (&["cfg", "database", "remove"], &["shop"]),
        (&["cfg", "database", "test"], &["shop"]),
        (&["init"], &[]),
        (&["help"], &[]),
        (&["version"], &[]),
    ];

    /// The leaves that have left the interim `70`.
    ///
    /// `tpl help` prints the help of the node its path names, in either of the
    /// two representations of `FR-HELP-001`, and `tpl version` prints the line
    /// of `FR-HELP-005`. `tpl init` creates the five artefacts of
    /// `FR-PROJ-017`, and the nine `cfg` subcommands that do not contact a
    /// server maintain `.tpl/.cfg`. The eight `schema` subcommands read the
    /// catalogue through the cache, per `FR-SCH-025`, and the three of
    /// `tpl cache` load, clean and report on it. The four of `tpl template`
    /// list, print, check and locate the project's templates, per
    /// `FR-TMPL-002`. `tpl render` joins the two arms: it assembles the
    /// context of `FR-RND-023` from a catalogue read or a `--context`
    /// document and renders one template, once. Every other leaf is still the
    /// arrangement this module's own documentation describes.
    const IMPLEMENTED: [&[&str]; 28] = [
        &["schema", "info"],
        &["schema", "tables"],
        &["schema", "table"],
        &["schema", "views"],
        &["schema", "view"],
        &["schema", "routines"],
        &["schema", "routine"],
        &["schema", "dump"],
        &["cache", "load"],
        &["cache", "clean"],
        &["cache", "status"],
        &["template", "list"],
        &["template", "show"],
        &["template", "check"],
        &["template", "path"],
        &["render"],
        &["help"],
        &["version"],
        &["init"],
        &["cfg", "get"],
        &["cfg", "set"],
        &["cfg", "unset"],
        &["cfg", "list"],
        &["cfg", "database", "add"],
        &["cfg", "database", "list"],
        &["cfg", "database", "show"],
        &["cfg", "database", "update"],
        &["cfg", "database", "remove"],
    ];

    /// The six group nodes of `FR-CLI-008`, by the path a caller writes. None
    /// of them declares an argument of its own, per `FR-CLI-009`.
    const GROUPS: [&[&str]; 6] = [
        &[],
        &["schema"],
        &["template"],
        &["cache"],
        &["cfg"],
        &["cfg", "database"],
    ];

    /// The whole declared surface: every node of the tree, the positional
    /// arguments it declares in declaration order, and the long flags it
    /// declares, sorted.
    ///
    /// This is the requirement corpus written out, node by node, and it is the
    /// statement `FR-CLI-019` turns into a contract: a command rejects every
    /// flag it does not declare, so the declaration **is** the surface. A
    /// positional argument is named by its identifier and a flag by its long
    /// form, which is the whole of what either has — `FR-GLOB-024` gives no
    /// local flag a short one.
    ///
    /// The root carries the seven global flags of `FR-GLOB-001`, which is
    /// where `FR-GLOB-002` and `FR-CLI-024` require them to be declared and the
    /// only place they appear in the unbuilt tree.
    #[rustfmt::skip]
    const SURFACE: [(&[&str], &[&str], &[&str]); 35] = [
        (&[], &[], &["database", "help", "quiet", "timeout", "tpl-dir", "verbose", "version"]),

        (&["schema"], &[], &[]),
        (&["schema", "info"], &[], &["direct", "format", "no-cache", "pretty"]),
        (&["schema", "tables"], &[], &["direct", "format", "no-cache", "pattern", "pretty"]),
        (&["schema", "table"], &["name"], &["direct", "format", "no-cache", "pretty"]),
        (&["schema", "views"], &[], &["direct", "format", "no-cache", "pattern", "pretty"]),
        (&["schema", "view"], &["name"], &["direct", "format", "no-cache", "pretty"]),
        (&["schema", "routines"], &[], &["direct", "format", "no-cache", "pattern", "pretty"]),
        (&["schema", "routine"], &["name"], &["direct", "format", "no-cache", "pretty"]),
        (&["schema", "dump"], &[], &["direct", "no-cache", "pretty"]),

        (&["template"], &[], &[]),
        (&["template", "list"], &[], &["format", "pretty"]),
        (&["template", "show"], &["name"], &[]),
        (&["template", "check"], &["names"], &[]),
        (&["template", "path"], &["name"], &["format", "pretty"]),

        (&["render"], &["template"], &["context", "direct", "no-cache", "routine", "set", "table", "view"]),

        (&["cache"], &[], &[]),
        (&["cache", "load"], &[], &["direct", "no-cache", "routine", "table", "view"]),
        (&["cache", "clean"], &[], &["routine", "table", "view"]),
        (&["cache", "status"], &[], &["format", "pretty"]),

        (&["cfg"], &[], &[]),
        (&["cfg", "get"], &["key"], &["format", "pretty"]),
        (&["cfg", "set"], &["key", "value"], &[]),
        (&["cfg", "unset"], &["key"], &[]),
        (&["cfg", "list"], &[], &["format", "pretty"]),
        (&["cfg", "database"], &[], &[]),
        (&["cfg", "database", "add"], &["name"], &["ca-file", "ca-path", "dsn", "host", "password-command", "port", "schema", "tls", "user"]),
        (&["cfg", "database", "list"], &[], &["format", "pretty"]),
        (&["cfg", "database", "show"], &["name"], &["format", "pretty"]),
        (&["cfg", "database", "update"], &["name"], &["ca-file", "ca-path", "dsn", "host", "password-command", "port", "schema", "tls", "user"]),
        (&["cfg", "database", "remove"], &["name"], &[]),
        (&["cfg", "database", "test"], &["name"], &["format", "pretty"]),

        (&["init"], &["path"], &[]),
        (&["help"], &["command_path"], &["format", "pretty"]),
        (&["version"], &[], &[]),
    ];

    /// The sixteen commands the first row of the `FR-GLOB-021` table names as
    /// the declarers of `--format`.
    #[rustfmt::skip]
    const FORMAT: [&[&str]; 16] = [
        &["schema", "info"], &["schema", "tables"], &["schema", "table"],
        &["schema", "views"], &["schema", "view"], &["schema", "routines"],
        &["schema", "routine"],
        &["template", "list"], &["template", "path"],
        &["cfg", "get"], &["cfg", "list"],
        &["cfg", "database", "list"], &["cfg", "database", "show"],
        &["cfg", "database", "test"],
        &["cache", "status"],
        &["help"],
    ];

    /// The seventeen commands the second row of the `FR-GLOB-021` table names
    /// as the declarers of `--pretty`: every command that declares `--format`,
    /// plus `schema dump`.
    #[rustfmt::skip]
    const PRETTY: [&[&str]; 17] = [
        &["schema", "info"], &["schema", "tables"], &["schema", "table"],
        &["schema", "views"], &["schema", "view"], &["schema", "routines"],
        &["schema", "routine"], &["schema", "dump"],
        &["template", "list"], &["template", "path"],
        &["cfg", "get"], &["cfg", "list"],
        &["cfg", "database", "list"], &["cfg", "database", "show"],
        &["cfg", "database", "test"],
        &["cache", "status"],
        &["help"],
    ];

    /// The ten commands the third and fourth rows of the `FR-GLOB-021` table
    /// name: the eight `schema` subcommands, `render`, and `cache load`. The
    /// two rows name the same ten, which `FR-CACHE-017` states as one
    /// sentence, so one constant carries both.
    #[rustfmt::skip]
    const CACHED: [&[&str]; 10] = [
        &["schema", "info"], &["schema", "tables"], &["schema", "table"],
        &["schema", "views"], &["schema", "view"], &["schema", "routines"],
        &["schema", "routine"], &["schema", "dump"],
        &["render"],
        &["cache", "load"],
    ];

    /// The argument vector a caller writes for `path`, `argv[0]` included.
    fn argv(path: &[&str], trailing: &[&str]) -> Vec<String> {
        std::iter::once("tpl")
            .chain(path.iter().copied())
            .chain(trailing.iter().copied())
            .map(str::to_owned)
            .collect()
    }

    /// The node `path` names, written as the tree writes it.
    fn node_path(path: &[&str]) -> String {
        std::iter::once("tpl")
            .chain(path.iter().copied())
            .collect::<Vec<&str>>()
            .join(" ")
    }

    /// The invocation `path` parses to when given `operands`, which every node
    /// of the tree must.
    fn parsed(path: &[&str], operands: &[&str]) -> Cli {
        parse_from(argv(path, operands)).expect("the tree declares this node")
    }

    /// The command one invocation names, for a test that reads it.
    ///
    /// Every vector these tests parse names a command rather than one of the
    /// two flag forms of `FR-HELP-001`, so anything else is a failure of the
    /// test's own premise rather than an outcome to assert on.
    fn commanded(invocation: &Invocation) -> Option<&Command> {
        match &invocation.form {
            Form::Command(command) => command.as_ref(),
            form => panic!("the vector named {form:?} rather than a command"),
        }
    }

    /// The invocation `path` names, as the process would act on it.
    fn invoked(path: &[&str], operands: &[&str]) -> Invocation {
        let Cli { globals, command } = parsed(path, operands);

        Invocation {
            globals,
            form: Form::Command(command),
        }
    }

    /// What the node `path` names does, and what it wrote while doing it.
    fn outcome(path: &[&str], operands: &[&str]) -> (Result<(), Error>, String) {
        let invocation = invoked(path, operands);
        let mut written = Vec::new();
        let result = route(&mut written, &invocation);

        (
            result,
            String::from_utf8(written).expect("the placeholder writes UTF-8"),
        )
    }

    /// Applies `visitor` to every node of `command`, with the path it is
    /// reached by.
    fn visit(command: &clap::Command, path: &str, visitor: &mut impl FnMut(&str, &clap::Command)) {
        visitor(path, command);

        for child in command.get_subcommands() {
            visit(child, &format!("{path} {}", child.get_name()), visitor);
        }
    }

    /// Every node of the built tree, by the path it is reached by.
    fn node_paths() -> Vec<String> {
        let tree = tree();
        let mut paths = Vec::new();
        visit(&tree, "tpl", &mut |path, _| paths.push(path.to_owned()));

        paths
    }

    /// The node of `tree` that `path` names.
    fn node<'a>(tree: &'a clap::Command, path: &[&str]) -> &'a clap::Command {
        path.iter().fold(tree, |command, name| {
            command
                .find_subcommand(name)
                .unwrap_or_else(|| panic!("{} is a node", node_path(path)))
        })
    }

    /// The argument `long` of the node `path` names.
    fn argument<'a>(tree: &'a clap::Command, path: &[&str], long: &str) -> &'a clap::Arg {
        node(tree, path)
            .get_arguments()
            .find(|argument| argument.get_long() == Some(long))
            .unwrap_or_else(|| panic!("{} declares --{long}", node_path(path)))
    }

    /// The values `argument` permits, in the order it declares them.
    fn permitted(argument: &clap::Arg) -> Vec<String> {
        argument
            .get_possible_values()
            .iter()
            .map(|value| value.get_name().to_owned())
            .collect()
    }

    #[test]
    fn fr_cli_002_every_node_of_the_tree_parses() {
        // FR-CLI-002: the tree is closed, and these are what it is closed
        // around. The two constants are the tree of the requirement written
        // out, and the count is what makes an unreachable node a failure here
        // rather than a discovery later.
        let declared = node_paths();

        assert_eq!(declared.len(), LEAVES.len() + GROUPS.len());

        for (path, operands) in LEAVES
            .iter()
            .copied()
            .chain(GROUPS.iter().map(|path| (*path, &[][..])))
        {
            let reached = node_path(path);

            assert!(declared.contains(&reached), "{reached} is not a node");
            parse_from(argv(path, operands)).unwrap_or_else(|error| panic!("{reached}: {error}"));
        }
    }

    #[test]
    fn fr_cli_019_every_node_declares_exactly_the_arguments_and_flags_of_its_requirement() {
        // FR-CLI-019: a command rejects every flag it does not declare, so the
        // declaration is the contract and this is the whole of it. The count
        // makes an argument added to a node a failure here rather than a
        // surface nobody asked for.
        let tree = tree();
        let mut declared = BTreeMap::new();
        visit(&tree, "tpl", &mut |path, command| {
            let positionals: Vec<String> = command
                .get_positionals()
                .map(|argument| argument.get_id().as_str().to_owned())
                .collect();
            let mut flags: Vec<String> = command
                .get_arguments()
                .filter_map(|argument| argument.get_long().map(str::to_owned))
                .collect();
            flags.sort();

            declared.insert(path.to_owned(), (positionals, flags));
        });

        assert_eq!(declared.len(), SURFACE.len());

        for (path, positionals, flags) in SURFACE {
            let reached = node_path(path);
            let (found_positionals, found_flags) = declared
                .get(&reached)
                .unwrap_or_else(|| panic!("{reached} is not a node"));

            assert_eq!(found_positionals, positionals, "{reached}: arguments");
            assert_eq!(found_flags, flags, "{reached}: local flags");
        }
    }

    #[test]
    fn fr_glob_021_each_of_the_four_local_flags_is_declared_by_exactly_the_nodes_its_row_names() {
        // FR-GLOB-021, the whole table. The set is asserted in both
        // directions: a node the row names and does not declare the flag
        // fails, and so does a node that declares it and the row does not
        // name.
        let tree = tree();
        let mut declared: BTreeMap<String, Vec<String>> = BTreeMap::new();
        visit(&tree, "tpl", &mut |path, command| {
            for argument in command.get_arguments() {
                if let Some(long) = argument.get_long() {
                    declared
                        .entry(long.to_owned())
                        .or_default()
                        .push(path.to_owned());
                }
            }
        });

        for (flag, row) in [
            ("format", &FORMAT[..]),
            ("pretty", &PRETTY[..]),
            ("direct", &CACHED[..]),
            ("no-cache", &CACHED[..]),
        ] {
            let mut found = declared.get(flag).cloned().unwrap_or_default();
            found.sort();

            let mut expected: Vec<String> = row.iter().map(|path| node_path(path)).collect();
            expected.sort();

            assert_eq!(found, expected, "--{flag}");
        }
    }

    #[test]
    fn fr_glob_023_no_node_declares_a_flag_that_carries_a_password() {
        // FR-GLOB-023 and FR-CFG-030: no flag named `password`, and no `-p`,
        // at any node. `--password-command` names a command rather than a
        // password, per FR-CFG-027, and is the nearest thing the tree has.
        let tree = tree();
        visit(&tree, "tpl", &mut |path, command| {
            for argument in command.get_arguments() {
                assert_ne!(argument.get_long(), Some("password"), "{path}");
                assert_ne!(argument.get_id().as_str(), "password", "{path}");
                assert_ne!(argument.get_short(), Some('p'), "{path}");
            }
        });
    }

    #[test]
    fn fr_help_013_an_enumerated_value_is_a_type_the_parser_holds() {
        // FR-HELP-013 obliges the help to state the permitted values of every
        // flag and FR-HELP-021 derives them from this tree, so a closed value
        // set is carried on the argument rather than checked after parsing.
        let tree = tree();

        let format = argument(&tree, &["schema", "tables"], "format");
        assert_eq!(permitted(format), ["text", "json"]);
        assert_eq!(format.get_default_values(), ["text"]);

        let tls = argument(&tree, &["cfg", "database", "add"], "tls");
        assert_eq!(
            permitted(tls),
            [
                "disabled",
                "preferred",
                "required",
                "verify-ca",
                "verify-identity"
            ]
        );
        assert!(
            tls.get_default_values().is_empty(),
            "the default of FR-CONF-013 belongs to the key, not to the flag"
        );

        // And the refusal is the parser's, at the argument, rather than a
        // string accepted here and rejected somewhere later.
        for (path, operands, trailing) in [
            (&["schema", "tables"][..], &[][..], &["--format", "xml"][..]),
            (
                &["cfg", "database", "add"][..],
                &["shop"][..],
                &["--tls", "verify"][..],
            ),
        ] {
            let mut vector = operands.to_vec();
            vector.extend_from_slice(trailing);
            let refused = parse_from(argv(path, &vector)).expect_err("not a permitted value");

            assert_eq!(refused.kind(), ErrorKind::InvalidValue, "{trailing:?}");
        }
    }

    #[test]
    fn fr_out_001_the_default_of_format_is_fixed_at_text_wherever_it_is_declared() {
        // FR-OUT-001 and FR-OUT-002: the default is `text` and is conditioned
        // on nothing, so it is declared on the argument and is the same at
        // each of the sixteen nodes that carry it.
        let tree = tree();

        for path in FORMAT {
            assert_eq!(
                argument(&tree, path, "format").get_default_values(),
                ["text"],
                "{}",
                node_path(path)
            );
        }

        let Some(Command::Schema(read)) = parsed(&["schema", "tables"], &[]).command else {
            panic!("`schema tables` did not parse to its own node");
        };
        let Some(schema::Command::Tables { output, .. }) = read.command else {
            panic!("`tables` did not parse to its own node");
        };

        assert_eq!(output.format, [local::Format::Text]);
        assert!(!output.pretty.pretty);
    }

    #[test]
    fn fr_help_013_a_required_argument_is_required_and_an_optional_one_is_not() {
        // FR-HELP-013 obliges the help to state whether each argument is
        // required, and FR-HELP-021 derives that from this tree.
        let tree = tree();

        for (path, identifier) in [
            (&["schema", "table"][..], "name"),
            (&["schema", "view"][..], "name"),
            (&["schema", "routine"][..], "name"),
            (&["template", "show"][..], "name"),
            (&["render"][..], "template"),
            (&["cfg", "get"][..], "key"),
            (&["cfg", "set"][..], "key"),
            (&["cfg", "set"][..], "value"),
            (&["cfg", "unset"][..], "key"),
            (&["cfg", "database", "add"][..], "name"),
            (&["cfg", "database", "show"][..], "name"),
            (&["cfg", "database", "update"][..], "name"),
            (&["cfg", "database", "remove"][..], "name"),
            (&["cfg", "database", "test"][..], "name"),
        ] {
            let found = node(&tree, path)
                .get_positionals()
                .find(|argument| argument.get_id() == identifier)
                .unwrap_or_else(|| panic!("{} declares <{identifier}>", node_path(path)));

            assert!(
                found.is_required_set(),
                "{} <{identifier}>",
                node_path(path)
            );

            let refused = parse_from(argv(path, &[])).expect_err("the operand is required");
            assert_eq!(
                refused.kind(),
                ErrorKind::MissingRequiredArgument,
                "{}",
                node_path(path)
            );
        }

        // The three optional positionals, each of which means something
        // specific by its absence.
        for (path, identifier) in [
            (&["template", "check"][..], "names"),
            (&["template", "path"][..], "name"),
            (&["help"][..], "command_path"),
            (&["init"][..], "path"),
        ] {
            let found = node(&tree, path)
                .get_positionals()
                .find(|argument| argument.get_id() == identifier)
                .unwrap_or_else(|| panic!("{} declares <{identifier}>", node_path(path)));

            assert!(
                !found.is_required_set(),
                "{} <{identifier}>",
                node_path(path)
            );
        }

        // FR-PROJ-012: absent, the path of `tpl init` is the current
        // directory, which is the one optional argument with a default rather
        // than a meaning of its own.
        assert_eq!(
            node(&tree, &["init"])
                .get_positionals()
                .find(|argument| argument.get_id() == "path")
                .expect("`init` declares <path>")
                .get_default_values(),
            ["."]
        );
    }

    #[test]
    fn fr_rnd_008_a_repeatable_argument_collects_every_occurrence() {
        // FR-RND-008 makes `--set` repeatable with distinct keys, and
        // FR-TMPL-019 and FR-HELP-026 make two positionals sequences rather
        // than single values. FR-RND-014 refuses a repeated key, which is the
        // command's and not declared here.
        let Some(Command::Render { set, .. }) =
            parsed(&["render"], &["x", "--set", "a=1", "--set", "b=2"]).command
        else {
            panic!("`render` did not parse to its own node");
        };

        assert_eq!(set, ["a=1", "b=2"]);

        let Some(Command::Template(read)) =
            parsed(&["template", "check"], &["a", "b", "c"]).command
        else {
            panic!("`template` did not parse to its own node");
        };
        let Some(template::Command::Check { names }) = read.command else {
            panic!("`check` did not parse to its own node");
        };

        assert_eq!(names, ["a", "b", "c"]);

        let Some(Command::Help { command_path, .. }) =
            parsed(&["help"], &["cfg", "database", "add"]).command
        else {
            panic!("`help` did not parse to its own node");
        };

        assert_eq!(command_path, ["cfg", "database", "add"]);
    }

    #[test]
    fn fr_proj_022_init_writes_nothing_to_stdout_and_creates_the_project_at_the_path_it_was_given()
    {
        // FR-PROJ-022 and BR-CLI-004: nothing on stdout, and exit 0. The path
        // is an operand rather than the working directory, so the test never
        // moves the process — which `cargo test` shares between threads.
        let scratch = crate::project::scratch::Scratch::new();
        let destination = scratch.path("project");
        let named = destination.to_string_lossy().into_owned();

        let (result, written) = outcome(&["init"], &[&named]);

        assert!(result.is_ok(), "{result:?}");
        assert_eq!(written, "");
        assert!(destination.join(".tpl").join(".cfg").is_file());
    }

    #[test]
    fn fr_err_030_every_unimplemented_leaf_reports_the_interim_seventy_naming_its_command_path() {
        // The interim arrangement this module documents: a leaf parses, has no
        // implementation, and says so as FR-ERR-030 does — never as a success
        // and never as a usage error the caller could act on. IMPLEMENTED now
        // holds twenty-eight of the twenty-nine leaves and this test ranges
        // over the one that is left, `tpl cfg database test`; what each
        // implemented leaf does instead is held to its own requirements by
        // the tests of the module that owns it, which is where its behaviour
        // is written.
        for (path, operands) in LEAVES {
            if IMPLEMENTED.contains(&path) {
                continue;
            }

            let (result, written) = outcome(path, operands);
            let reported = result.expect_err("no leaf is implemented yet");

            assert_eq!(reported.exit_code(), 70, "{path:?}");
            assert!(
                reported.to_string().contains(&node_path(path)),
                "{path:?} is not named by: {reported}"
            );
            assert!(written.is_empty(), "{path:?} wrote {written:?}");
        }
    }

    /// The form one vector parses to.
    fn form(path: &[&str], trailing: &[&str]) -> Form {
        parse(argv(path, trailing))
            .unwrap_or_else(|error| panic!("{path:?} {trailing:?}: {error}"))
            .form
    }

    /// The help form for one canonical path.
    fn help_of(path: &[&str]) -> Form {
        Form::Help(path.iter().map(|&segment| segment.to_owned()).collect())
    }

    #[test]
    fn fr_glob_019_both_flag_forms_are_answered_at_the_node_the_vector_reached() {
        // FR-GLOB-019 and FR-GLOB-020 give each flag an outcome at every node,
        // and FR-CLI-024 frees its position. The node answered is therefore
        // the node the vector reached, and the path it carries is canonical,
        // per FR-HELP-027 — which is what lets the renderer be reached by the
        // same path `tpl help <path>` resolves to.
        assert_eq!(form(&[], &["-h"]), help_of(&[]));
        assert_eq!(form(&[], &["--help"]), help_of(&[]));
        assert_eq!(
            form(&["schema", "tables"], &["--help"]),
            help_of(&["schema", "tables"])
        );

        // A node with a required operand, which is the defect this effort
        // resolves: the flag is answered although the operand is absent.
        assert_eq!(
            form(&["schema", "table"], &["-h"]),
            help_of(&["schema", "table"])
        );
        assert_eq!(form(&["cfg", "set"], &["--help"]), help_of(&["cfg", "set"]));

        // FR-CLI-024: written before the command, which is the same
        // invocation.
        assert_eq!(
            form(&[], &["-h", "schema", "table"]),
            help_of(&["schema", "table"])
        );

        // FR-HELP-027: an alias resolves to the node it names, on this path as
        // on `tpl help`'s.
        assert_eq!(
            form(&["schema", "tbl"], &["--help"]),
            help_of(&["schema", "table"])
        );
        assert_eq!(
            form(&["cfg", "db", "add"], &["-h"]),
            help_of(&["cfg", "database", "add"])
        );

        // FR-GLOB-020, at a node with an operand and at one without.
        assert_eq!(form(&[], &["-V"]), Form::Version);
        assert_eq!(form(&["version"], &["--version"]), Form::Version);
        assert_eq!(form(&["cfg", "database", "add"], &["-V"]), Form::Version);

        // Both flags together: help wins, whichever order they were written
        // in. The corpus states no precedence, so the choice is `requested`'s
        // and is pinned here rather than left to the order of a match.
        assert_eq!(form(&[], &["--help", "--version"]), help_of(&[]));
        assert_eq!(form(&[], &["--version", "--help"]), help_of(&[]));
        assert_eq!(
            form(&["schema", "table"], &["-V", "-h"]),
            help_of(&["schema", "table"])
        );
    }

    #[test]
    fn fr_err_034_the_waiver_reaches_the_two_flags_and_nothing_else() {
        // The other half of the defect's resolution. A vector that supplies
        // neither the operand nor one of the two flags is refused exactly as
        // it was, with the message FR-ERR-034 row 64 obliges — so the waiver
        // is a property of the two flags rather than a weakening of
        // FR-SCH-005, FR-TMPL-016, FR-RND-001 or the cfg entry commands.
        for (path, operands) in LEAVES {
            if operands.is_empty() {
                continue;
            }

            let refused = parse(argv(path, &[])).expect_err("the operand is required");

            assert_eq!(refused.exit_code(), 64, "{path:?}");
            assert!(
                matches!(&refused, Error::MissingArgument { command, .. } if command == &path.join(" ")),
                "{path:?}: {refused}"
            );
        }
    }

    #[test]
    fn fr_cli_007_a_group_node_invoked_with_no_child_prints_its_own_help_and_succeeds() {
        // FR-CLI-007 and FR-HELP-025: exactly the text `tpl help <node>` would
        // print, on stdout, at exit 0. "Exactly" is asserted byte for byte
        // against the renderer both forms reach, which is what keeps the two
        // from drifting when one of them grows a caller.
        for path in GROUPS {
            let (result, written) = outcome(path, &[]);

            result.unwrap_or_else(|error| panic!("{path:?}: {error}"));

            let expected = help::text(path).expect("every group node has an entry");

            assert_eq!(written, expected, "{path:?}");
            assert!(
                written.starts_with(&format!("USAGE\n  {}", node_path(path))),
                "{path:?} did not print its own help"
            );
        }
    }

    #[test]
    fn fr_cli_008_the_group_nodes_are_exactly_the_six_of_the_requirement() {
        // FR-CLI-008 names six, and FR-CLI-009 gives none of them an action of
        // its own. A node with children is a group by construction here: the
        // child is optional and the arm for None prints help, so a seventh
        // group would be a seventh node with children.
        let tree = tree();
        let mut groups = Vec::new();
        visit(&tree, "tpl", &mut |path, command| {
            if command.get_subcommands().next().is_some() {
                groups.push(path.to_owned());
            }
        });
        groups.sort();

        let mut expected: Vec<String> = GROUPS.iter().map(|path| node_path(path)).collect();
        expected.sort();

        assert_eq!(groups, expected);
    }

    #[test]
    fn fr_cli_010_the_top_level_commands_are_exactly_the_eight_of_the_requirement() {
        // FR-CLI-010.
        let tree = tree();
        let mut top_level: Vec<&str> = tree
            .get_subcommands()
            .map(clap::Command::get_name)
            .collect();
        top_level.sort_unstable();

        assert_eq!(
            top_level,
            [
                "cache", "cfg", "help", "init", "render", "schema", "template", "version"
            ]
        );
    }

    #[test]
    fn fr_cli_011_each_of_the_seven_aliases_resolves_to_its_canonical_node() {
        // FR-CLI-011, the whole table.
        for (alias, canonical, operands) in [
            (["schema", "tbls"], ["schema", "tables"], &[][..]),
            (["schema", "tbl"], ["schema", "table"], &["orders"][..]),
            (["schema", "vws"], ["schema", "views"], &[][..]),
            (["schema", "vw"], ["schema", "view"], &["v_sales"][..]),
            (["schema", "rtns"], ["schema", "routines"], &[][..]),
            (["schema", "rtn"], ["schema", "routine"], &["calc_vat"][..]),
            (["cfg", "db"], ["cfg", "database"], &[][..]),
        ] {
            assert_eq!(
                parsed(&alias, operands),
                parsed(&canonical, operands),
                "{alias:?}"
            );
        }

        // The seventh alias reaches the level beneath it too, since it names a
        // group node rather than a leaf.
        assert_eq!(
            parsed(&["cfg", "db", "add"], &["shop"]),
            parsed(&["cfg", "database", "add"], &["shop"])
        );
    }

    #[test]
    fn fr_cli_011_no_alias_other_than_the_seven_is_accepted() {
        // FR-CLI-011 declares seven "and no others"; FR-CLI-012 refuses every
        // top-level alias, of which `tpl s`, `tpl t` and `tpl r` are the three
        // the requirement names.
        for path in [
            &["s"][..],
            &["t"][..],
            &["r"][..],
            &["c"][..],
            &["schema", "tbles"][..],
            &["schema", "rtn s"][..],
            &["schema", "procs"][..],
            &["template", "ls"][..],
            &["cache", "st"][..],
            &["cfg", "dbs"][..],
            &["cfg", "d"][..],
        ] {
            let refused = parse_from(argv(path, &[])).expect_err("no such node");

            assert_eq!(refused.kind(), ErrorKind::InvalidSubcommand, "{path:?}");
        }
    }

    #[test]
    fn fr_cli_004_a_command_is_not_inferred_from_a_prefix_of_its_name() {
        // FR-CLI-004: `tpl sch tables` is 64, not `tpl schema tables`.
        //
        // `closed` sets `infer_subcommands` off at every node, and `clap`
        // exposes no getter for it, so it is observed rather than read — which
        // is what the requirement needs anyway: an **unambiguous** prefix is
        // the case inference would accept, and every token below is one.
        for path in [
            &["sch", "tables"][..],
            &["schem"][..],
            &["rend"][..],
            &["ini"][..],
            &["vers"][..],
            &["schema", "inf"][..],
            &["schema", "dum"][..],
            &["template", "che"][..],
            &["cache", "stat"][..],
            &["cfg", "data"][..],
            &["cfg", "database", "rem"][..],
        ] {
            let refused = parse_from(argv(path, &[])).expect_err("a prefix is not a command");

            assert_eq!(refused.kind(), ErrorKind::InvalidSubcommand, "{path:?}");
        }
    }

    #[test]
    fn fr_cli_005_a_long_flag_is_not_inferred_from_a_prefix_of_its_name() {
        // FR-CLI-005: `tpl --data shop schema tables` is 64. `infer_long_args`
        // has no getter either, and every prefix below is unambiguous for the
        // same reason as above.
        let refused =
            parse_from(argv(&[], &["--data", "shop", "schema", "tables"])).expect_err("no --data");

        assert_eq!(refused.kind(), ErrorKind::UnknownArgument);

        // At the root, where the seven are declared, and at a leaf, where they
        // are the propagated globals rather than the declared arguments.
        for prefix in ["--datab", "--tpl", "--tpl-di", "--time", "--verb", "--qui"] {
            for path in [&[][..], &["schema", "tables"][..]] {
                let refused =
                    parse_from(argv(path, &[prefix])).expect_err("a prefix is not a flag");

                assert_eq!(
                    refused.kind(),
                    ErrorKind::UnknownArgument,
                    "{prefix} at {path:?}"
                );
            }
        }

        // And over a local flag, at the node that declares it: a prefix of
        // `--pattern` or of `--no-cache` is no more a flag than a prefix of a
        // global one.
        for prefix in ["--patt", "--form", "--no-cac", "--dir", "--pret"] {
            let refused = parse_from(argv(&["schema", "tables"], &[prefix]))
                .expect_err("a prefix is not a flag");

            assert_eq!(refused.kind(), ErrorKind::UnknownArgument, "{prefix}");
        }
    }

    #[test]
    fn fr_cli_006_an_undeclared_token_is_never_handed_to_an_executable_on_the_path() {
        // FR-CLI-006: a token that is not a declared command SHALL NOT cause a
        // lookup of a `tpl-<token>` executable on PATH. The parser performs no
        // process lookup of any kind; the one setting that would make an
        // undeclared token something other than an error — and so give a caller
        // somewhere to write that lookup — is off, at every node, and an
        // undeclared token is refused instead of collected.
        let tree = tree();
        visit(&tree, "tpl", &mut |path, command| {
            assert!(!command.is_allow_external_subcommands_set(), "{path}");
        });

        for path in [
            &["frobnicate"][..],
            &["tpl-frobnicate"][..],
            &["schema", "tpl-x"][..],
        ] {
            let refused = parse_from(argv(path, &[])).expect_err("no such command");

            assert_eq!(refused.kind(), ErrorKind::InvalidSubcommand, "{path:?}");
        }

        // PATH itself is not manipulated to prove the absence of the lookup:
        // `std::env::set_var` is `unsafe` in edition 2024 and this crate
        // forbids `unsafe_code`, so the observation is made where the
        // behaviour is decided — the setting above, and the refusal beside it.
    }

    #[test]
    fn fr_cli_024_a_global_flag_is_accepted_in_every_position_of_the_command_line() {
        // FR-CLI-024 and FR-GLOB-002: the three lines of the requirement are
        // one invocation.
        let before = parse_from(argv(&[], &["-d", "shop", "schema", "tables"])).expect("parses");
        let after = parse_from(argv(&["schema", "tables"], &["-d", "shop"])).expect("parses");
        let between = parse_from(argv(&["schema"], &["-d", "shop", "tables"])).expect("parses");

        assert_eq!(before, after);
        assert_eq!(before, between);
        assert_eq!(before.globals.database, ["shop"]);

        // And at the deepest node of the tree, where the flag sits between two
        // subcommands rather than between a command and its subcommand.
        assert_eq!(
            parse_from(argv(
                &["cfg"],
                &["-d", "shop", "database", "add", "reporting"]
            ))
            .expect("parses"),
            parse_from(argv(
                &["cfg", "database", "add"],
                &["reporting", "-d", "shop"]
            ))
            .expect("parses")
        );

        // And among the local flags of a node that declares some, which is the
        // fourth position FR-CLI-024 names and the one this task made
        // reachable.
        assert_eq!(
            parse_from(argv(
                &["schema", "tables"],
                &["--pattern", "order%", "-d", "shop", "--format", "json"]
            ))
            .expect("parses"),
            parse_from(argv(
                &["schema", "tables"],
                &["-d", "shop", "--pattern", "order%", "--format", "json"]
            ))
            .expect("parses")
        );
    }

    #[test]
    fn fr_glob_002_every_node_accepts_every_one_of_the_seven_global_flags() {
        // FR-GLOB-002: every node, every flag. The vector puts all seven after
        // the node, which is the position FR-CLI-024 had to free and the one an
        // agent appending to a line it has already built writes.
        let seven = [
            "-d",
            "shop",
            "--tpl-dir",
            "/srv/project/.tpl",
            "--timeout",
            "30",
            "-v",
            "-q",
            "-h",
            "-V",
        ];

        for (path, operands) in LEAVES
            .iter()
            .copied()
            .chain(GROUPS.iter().map(|path| (*path, &[][..])))
        {
            let mut trailing = operands.to_vec();
            trailing.extend_from_slice(&seven);

            let invocation = parse_from(argv(path, &trailing))
                .unwrap_or_else(|error| panic!("{path:?}: {error}"));
            let globals = &invocation.globals;

            assert_eq!(globals.database, ["shop"], "{path:?}");
            assert_eq!(
                globals.tpl_dir,
                [std::path::Path::new("/srv/project/.tpl")],
                "{path:?}"
            );
            assert_eq!(
                globals.timeout.first().map(|seconds| seconds.get()),
                Some(30),
                "{path:?}"
            );
            assert_eq!(globals.verbose, 1, "{path:?}");
            assert!(globals.quiet, "{path:?}");
            assert!(globals.help, "{path:?}");
            assert!(globals.version, "{path:?}");
        }
    }

    #[test]
    fn fr_glob_001_the_seven_global_flags_are_the_whole_of_the_global_set() {
        // FR-GLOB-001 declares exactly seven and no others, and FR-GLOB-021
        // keeps four flags out of that set deliberately — none of the local
        // flags this tree declares is global.
        let tree = tree();
        let mut global: Vec<&str> = tree
            .get_arguments()
            .filter(|argument| argument.is_global_set())
            .filter_map(clap::Arg::get_long)
            .collect();
        global.sort_unstable();

        assert_eq!(
            global,
            [
                "database", "help", "quiet", "timeout", "tpl-dir", "verbose", "version"
            ]
        );

        visit(&tree, "tpl", &mut |path, command| {
            if path == "tpl" {
                return;
            }

            for argument in command.get_arguments() {
                assert!(
                    !argument.is_global_set(),
                    "{path} declares a global {:?}",
                    argument.get_id()
                );
            }
        });
    }

    #[test]
    fn fr_glob_024_the_short_flag_set_of_the_whole_tree_is_the_five_of_the_requirement() {
        // FR-GLOB-024: `-d`, `-v`, `-q`, `-h` and `-V` are the complete
        // short-flag set of the tool, at any node, global or local.
        let tree = tree();
        let mut shorts = Vec::new();
        visit(&tree, "tpl", &mut |path, command| {
            for argument in command.get_arguments() {
                if let Some(short) = argument.get_short() {
                    shorts.push((path.to_owned(), short));
                }
            }
        });

        let declared: Vec<char> = shorts.iter().map(|(_, short)| *short).collect();

        assert_eq!(declared, ['d', 'v', 'q', 'h', 'V']);
        assert!(
            shorts.iter().all(|(path, _)| path == "tpl"),
            "a short form is declared away from the root: {shorts:?}"
        );

        // The two flags FR-GLOB-024 leaves without one, asserted where the
        // five are, so that acquiring one is a failure here.
        for long in ["tpl-dir", "timeout"] {
            let argument = tree
                .get_arguments()
                .find(|argument| argument.get_long() == Some(long))
                .unwrap_or_else(|| panic!("--{long} is declared"));

            assert_eq!(argument.get_short(), None, "--{long}");
        }
    }

    #[test]
    fn fr_help_006_the_parser_generates_no_help_of_its_own_at_any_node() {
        // OD-07: `tpl` renders all seven sections of FR-HELP-006 itself, so
        // `-h/--help` is the global flag of FR-GLOB-001 and `help` is the real
        // subcommand of FR-HELP-004 — neither is the parser's.
        let tree = tree();
        visit(&tree, "tpl", &mut |path, command| {
            assert!(command.is_disable_help_flag_set(), "{path}");
            assert!(command.is_disable_help_subcommand_set(), "{path}");
        });

        // Observed rather than inferred: were the parser's own help flag live,
        // this would be an error of kind `DisplayHelp` rather than a parse.
        let invocation = parse_from(argv(&["schema", "tables"], &["--help"])).expect("parses");

        assert!(invocation.globals.help);

        // And `help` is a node of the tree, not the parser's subcommand.
        assert!(matches!(
            parsed(&["help"], &[]).command,
            Some(Command::Help { .. })
        ));
    }

    #[test]
    fn fr_cli_019_a_flag_a_node_does_not_declare_is_refused_there() {
        // FR-CLI-019: there is no "known but inapplicable" category. The first
        // two rows are the two invocations the requirement's own rationale
        // names; the rest are every other place the corpus withholds a flag
        // that a neighbouring node declares.
        for (path, operands, trailing) in [
            (&["init"][..], &[][..], &["-o", "/tmp/x"][..]),
            (&["schema", "dump"][..], &[][..], &["--format", "text"][..]),
            (
                &["schema", "dump"][..],
                &[][..],
                &["--pattern", "order%"][..],
            ),
            (
                &["schema", "info"][..],
                &[][..],
                &["--pattern", "order%"][..],
            ),
            (&["render"][..], &["x"][..], &["--format", "json"][..]),
            (&["render"][..], &["x"][..], &["--pretty"][..]),
            (&["render"][..], &["x"][..], &["--pattern", "order%"][..]),
            (&["cache", "clean"][..], &[][..], &["--direct"][..]),
            (&["cache", "clean"][..], &[][..], &["--no-cache"][..]),
            (&["cache", "status"][..], &[][..], &["--direct"][..]),
            (
                &["cache", "status"][..],
                &[][..],
                &["--table", "orders"][..],
            ),
            (
                &["template", "show"][..],
                &["x"][..],
                &["--format", "json"][..],
            ),
            (&["template", "check"][..], &[][..], &["--pretty"][..]),
            (
                &["cfg", "set"][..],
                &["core.database", "shop"][..],
                &["--format", "json"][..],
            ),
            (
                &["cfg", "database", "remove"][..],
                &["shop"][..],
                &["--format", "json"][..],
            ),
            (
                &["cfg", "database", "test"][..],
                &["shop"][..],
                &["--host", "db.example.com"][..],
            ),
            (&["schema", "tables"][..], &[][..], &["-x"][..]),
            (&["version"][..], &[][..], &["--format", "json"][..]),
        ] {
            let mut vector = operands.to_vec();
            vector.extend_from_slice(trailing);
            let refused = parse_from(argv(path, &vector)).expect_err("not declared");

            assert_eq!(
                refused.kind(),
                ErrorKind::UnknownArgument,
                "{path:?} {trailing:?}"
            );
        }
    }

    #[test]
    fn fr_cli_020_a_flag_and_its_value_are_case_sensitive() {
        // FR-CLI-020: neither is normalised.
        assert_eq!(
            parse_from(argv(&["schema", "tables"], &["-d", "Shop"]))
                .expect("parses")
                .globals
                .database,
            ["Shop"]
        );

        assert_eq!(
            parse_from(argv(&["schema", "tables"], &["-D", "shop"]))
                .expect_err("no -D")
                .kind(),
            ErrorKind::UnknownArgument
        );

        // And over a local flag and an enumerated value, which this task added:
        // `--Format` is not `--format`, and `JSON` is not `json`.
        assert_eq!(
            parse_from(argv(&["schema", "tables"], &["--Format", "json"]))
                .expect_err("no --Format")
                .kind(),
            ErrorKind::UnknownArgument
        );

        assert_eq!(
            parse_from(argv(&["schema", "tables"], &["--format", "JSON"]))
                .expect_err("no such value")
                .kind(),
            ErrorKind::InvalidValue
        );
    }

    #[test]
    fn fr_cli_002_the_nested_node_of_the_third_level_is_reached_through_its_parent() {
        // The one three-level path of FR-CLI-002, and the one place a group
        // node is a child of a group node.
        let invocation = parsed(&["cfg", "database", "remove"], &["shop"]);

        let Some(Command::Cfg(config)) = &invocation.command else {
            panic!("`cfg` did not parse to its own node: {invocation:?}");
        };
        let Some(cfg::Command::Database(database)) = &config.command else {
            panic!("`database` did not parse to its own node: {config:?}");
        };
        let Some(cfg::DatabaseCommand::Remove { name }) = &database.command else {
            panic!("`remove` did not parse to its own node: {database:?}");
        };

        assert_eq!(name, "shop");
    }

    #[test]
    fn fr_cli_011_an_alias_and_its_canonical_name_reach_the_same_variant() {
        let invocation = parsed(&["schema", "rtns"], &[]);

        let Some(Command::Schema(read)) = &invocation.command else {
            panic!("`schema` did not parse to its own node: {invocation:?}");
        };

        assert!(matches!(
            read.command,
            Some(schema::Command::Routines { .. })
        ));
    }

    #[test]
    fn fr_cli_014_a_flag_that_carries_a_single_value_is_refused_on_its_second_occurrence() {
        // FR-CLI-014, and the whole of what OD-08 bought by declaring these
        // flags repeatable: the message names **both values**, which the
        // parser's own ArgumentConflict cannot, because it names the argument
        // twice and neither value.
        for (path, operands, trailing, flag, first, second) in [
            (
                &[][..],
                &[][..],
                &["-d", "a", "-d", "b", "schema", "tables"][..],
                "--database",
                "a",
                "b",
            ),
            (
                &["schema", "tables"][..],
                &[][..],
                &["--pattern", "order%", "--pattern", "invoice%"][..],
                "--pattern",
                "order%",
                "invoice%",
            ),
            (
                &["schema", "tables"][..],
                &[][..],
                &["--format", "json", "--format", "text"][..],
                "--format",
                "json",
                "text",
            ),
            (
                &["cfg", "database", "add"][..],
                &["shop"][..],
                &["--host", "a.example.com", "--host", "b.example.com"][..],
                "--host",
                "a.example.com",
                "b.example.com",
            ),
            (
                &["render"][..],
                &["rust/struct"][..],
                &["--table", "orders", "--table", "invoices"][..],
                "--table",
                "orders",
                "invoices",
            ),
        ] {
            let mut vector = operands.to_vec();
            vector.extend_from_slice(trailing);
            let refused = parse(argv(path, &vector)).expect_err("the repetition is refused");

            assert_eq!(refused.exit_code(), 64, "{path:?} {trailing:?}");

            let Error::RepeatedValueFlag {
                flag: named,
                first: written,
                second: again,
            } = &refused
            else {
                panic!("{path:?} {trailing:?} was refused as {refused}");
            };

            assert_eq!(named, flag);
            assert_eq!(written, first);
            assert_eq!(again, second);
        }
    }

    #[test]
    fn fr_rnd_008_the_one_repeatable_flag_and_the_two_repeatable_arguments_are_not_refused() {
        // FR-RND-008 makes `--set` repeatable with distinct keys, and
        // FR-TMPL-019 and FR-HELP-026 make two positionals sequences. None of
        // the three is the repetition FR-CLI-014 refuses.
        let invocation = parse(argv(
            &["render"],
            &["rust/struct", "--set", "a=1", "--set", "b=2"],
        ))
        .expect("--set is repeatable");

        let Some(Command::Render { set, .. }) = commanded(&invocation) else {
            panic!("`render` did not parse to its own node");
        };
        assert_eq!(set, &["a=1", "b=2"]);

        parse(argv(&["template", "check"], &["a", "b", "c"])).expect("names is a sequence");
        parse(argv(&["help"], &["cfg", "database", "add"])).expect("the path is a sequence");
    }

    #[test]
    fn fr_cli_024_a_global_flag_is_refused_on_its_second_occurrence_at_whatever_depth() {
        // FR-CLI-024 frees the position of a global flag, and FR-CLI-014
        // continues to refuse a repetition "wherever the two occurrences
        // appear" — including one written before the command and one after it.
        let refused = parse(argv(
            &["cfg", "database", "add"],
            &["-d", "a", "shop", "-d", "b"],
        ))
        .expect_err("the repetition is refused");

        assert_eq!(refused.exit_code(), 64);
        assert!(refused.to_string().contains("--database"), "{refused}");
    }

    #[test]
    fn fr_cli_015_quiet_and_verbose_together_are_refused_and_the_repetition_is_decided_first() {
        // FR-CLI-015 and FR-GLOB-015, then the order `rules` states: a refusal
        // that is a property of one flag precedes one that is a property of
        // two.
        let refused = parse(argv(&["version"], &["-q", "-v"])).expect_err("the pair is refused");

        assert_eq!(refused.exit_code(), 64);
        assert!(
            matches!(refused, Error::MutuallyExclusiveFlags { .. }),
            "{refused}"
        );

        let refused = parse(argv(&["version"], &["-d", "a", "-d", "b", "-q", "-v"]))
            .expect_err("both conditions hold");

        assert!(
            matches!(refused, Error::RepeatedValueFlag { .. }),
            "the repetition is decided first, and this was {refused}"
        );
    }

    #[test]
    fn fr_cli_016_the_verbosity_count_saturates_past_three_and_past_two_hundred_and_fifty_five() {
        // FR-CLI-016: the count reaches three levels and saturates above three
        // **without error**. The second vector is the one the count's own type
        // could have refused: `ArgAction::Count` accumulates into a `u8`, and
        // what this asserts is that the parser saturates there rather than
        // overflowing or refusing.
        for (occurrences, counted, level) in [
            (1, 1, Level::Info),
            (2, 2, Level::Debug),
            (3, 3, Level::Trace),
            (4, 4, Level::Trace),
            (300, u8::MAX, Level::Trace),
        ] {
            let repeated = vec!["-v"; occurrences];
            let invocation =
                parse(argv(&["version"], &repeated)).unwrap_or_else(|error| panic!("{error}"));

            assert_eq!(invocation.globals.verbose, counted, "{occurrences} of -v");
            assert_eq!(super::level(&invocation), level, "{occurrences} of -v");
        }
    }

    #[test]
    fn fr_cli_017_the_argument_terminator_is_accepted_on_every_command() {
        // FR-CLI-017, at every node of FR-CLI-002 and not only at the ones
        // that take an operand after it.
        for (path, operands) in LEAVES
            .iter()
            .copied()
            .chain(GROUPS.iter().map(|path| (*path, &[][..])))
        {
            let mut vector = operands.to_vec();
            vector.push("--");

            parse(argv(path, &vector)).unwrap_or_else(|error| panic!("{path:?}: {error}"));
        }
    }

    #[test]
    fn fr_cli_017_a_token_after_the_terminator_is_a_positional_argument() {
        // FR-CLI-017 and the note FR-CLI-024 makes of it: `tpl render x -- -d`
        // passes `-d` to the command as an argument and does not select a
        // database entry.
        let invocation = parse(argv(&["render"], &["--", "-d"])).expect("parses");
        let Some(Command::Render { template, .. }) = commanded(&invocation) else {
            panic!("`render` did not parse to its own node");
        };

        assert_eq!(template, "-d");
        assert!(
            invocation.globals.database.is_empty(),
            "the token selected a database entry"
        );

        let invocation =
            parse(argv(&["template", "check"], &["--", "-d", "--format"])).expect("parses");
        let Some(Command::Template(read)) = commanded(&invocation) else {
            panic!("`template` did not parse to its own node");
        };
        let Some(template::Command::Check { names }) = &read.command else {
            panic!("`check` did not parse to its own node");
        };

        assert_eq!(names, &["-d", "--format"]);
        assert!(invocation.globals.database.is_empty());

        // And where the command has no place for it, the token is refused as
        // an argument rather than read as the flag it resembles.
        let refused = parse(argv(&["render"], &["x", "--", "-d"])).expect_err("no second operand");

        assert_eq!(refused.exit_code(), 64);
        assert!(
            matches!(&refused, Error::UnexpectedArgument { token, .. } if token == "-d"),
            "{refused}"
        );
    }

    #[test]
    fn fr_cli_018_a_value_beginning_with_a_dash_is_accepted_in_the_two_forms_that_carry_it() {
        // FR-CLI-018, second sentence: the value is accepted in the
        // `--flag=value` form, and as a positional argument after `--`.
        for written in [&["-d=-x"][..], &["--database=-x"][..]] {
            let invocation = parse(argv(&["schema", "tables"], written)).expect("parses");

            assert_eq!(invocation.globals.database, ["-x"], "{written:?}");
        }

        let invocation =
            parse(argv(&["cfg", "set"], &["core.database", "--", "-x"])).expect("parses");
        let Some(Command::Cfg(config)) = commanded(&invocation) else {
            panic!("`cfg` did not parse to its own node");
        };
        let Some(cfg::Command::Set { value, .. }) = &config.command else {
            panic!("`set` did not parse to its own node");
        };

        assert_eq!(value, "-x");
    }

    #[test]
    fn fr_cli_020_a_flag_and_its_value_keep_their_case_in_the_message() {
        // FR-CLI-020: neither is normalised, and the refusal reproduces both
        // exactly as they were written.
        let refused =
            parse(argv(&["schema", "tables"], &["--Format", "JSON"])).expect_err("no --Format");

        assert_eq!(refused.exit_code(), 64);
        assert!(refused.to_string().contains("--Format"), "{refused}");

        let refused =
            parse(argv(&["schema", "tables"], &["--format", "JSON"])).expect_err("no such value");

        assert!(refused.to_string().contains("JSON"), "{refused}");

        let refused = parse(argv(&["schema", "tables"], &["-d", "Shop", "-d", "SHOP"]))
            .expect_err("the repetition is refused");

        let Error::RepeatedValueFlag { first, second, .. } = &refused else {
            panic!("{refused}");
        };
        assert_eq!(first, "Shop");
        assert_eq!(second, "SHOP");
    }

    #[test]
    fn fr_cli_002_every_node_of_the_tree_is_reached_through_parse_as_it_is_through_the_parser() {
        // `parse` is the one route from the process to the tree, and it adds
        // rules rather than changing what parses: every node of FR-CLI-002
        // still parses through it, and reaches the same invocation.
        for (path, operands) in LEAVES
            .iter()
            .copied()
            .chain(GROUPS.iter().map(|path| (*path, &[][..])))
        {
            let through_rules =
                parse(argv(path, operands)).unwrap_or_else(|error| panic!("{path:?}: {error}"));
            let raw = parsed(path, operands);

            assert_eq!(through_rules.globals, raw.globals, "{path:?}");
            assert_eq!(through_rules.form, Form::Command(raw.command), "{path:?}");
        }
    }
}
