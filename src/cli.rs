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
//! # The interim arrangement
//!
//! Two things in this module are deliberately provisional, and both are named
//! here so that neither is mistaken for finished work.
//!
//! **A leaf whose work is a later sprint reports `70`.** Every leaf of the tree
//! parses today; none of them acts. [`route`] gives each one an arm that
//! returns [`Error::InternalInvariant`] naming its command path, which
//! `FR-ERR-001` makes exit `70` — the code for a condition the caller cannot
//! have caused and cannot correct, which is exactly what a parsed command with
//! no implementation is. The arrangement is an arm per leaf rather than one
//! catch-all so that each later sprint replaces **its own** entry, and so that
//! the arm it must replace is named by its path rather than found by reading.
//!
//! **A group node's help is a placeholder.** `FR-CLI-007` and `FR-HELP-025`
//! require a group node invoked with no child to print exactly the text
//! `tpl help <node>` would print, and to exit `0`. The renderer that composes
//! that text is a later task; [`group_help`] is the call site it replaces, and
//! it says so in its own documentation.
//!
//! Nothing in this module is reached from [`crate::run`] yet. The route from
//! the process to [`parse_from`] carries one decision this module does not
//! make — what a `clap::Error` becomes, per `OD-08` — and it is made where the
//! parsing rules of `FR-CLI-014` through `FR-CLI-020` are.

mod cache;
mod cfg;
mod globals;
mod schema;
mod template;

use std::ffi::OsString;
use std::io::Write;

use clap::{CommandFactory, FromArgMatches, Parser, Subcommand};

use cache::Cache;
use cfg::Cfg;
use globals::Globals;
use schema::Schema;
use template::Template;

use crate::error::{self, Error};

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
    Render,

    /// The catalogue cache.
    Cache(Cache),

    /// The `.tpl/.cfg` file.
    Cfg(Cfg),

    /// Creates a `.tpl` project.
    Init,

    /// Prints help, and the whole command tree as JSON.
    Help,

    /// Prints the version.
    Version,
}

/// The parser tree, closed at every node by [`closed`].
///
/// This is the single construction site of the tree, and the only one
/// [`parse_from`] and the tests use: a setting applied here is applied to every
/// node, and a node added to the tree acquires it without anyone remembering
/// to.
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

/// Parses one argument vector against the tree.
///
/// The vector is the whole invocation, `argv[0]` included, as the process
/// receives it.
///
/// # Errors
///
/// Returns the `clap::Error` the parser raised, unclassified. What it becomes —
/// which of the conditions of `FR-CLI-003`, `FR-CLI-019` or `FR-ERR-006` it is,
/// and which exit code that carries — is decided where `OD-08` places it, and
/// deliberately not here.
pub(crate) fn parse_from<I, T>(arguments: I) -> Result<Cli, clap::Error>
where
    I: IntoIterator<Item = T>,
    T: Into<OsString> + Clone,
{
    let matches = tree().try_get_matches_from(arguments)?;

    Cli::from_arg_matches(&matches)
}

/// Runs the node one parsed invocation names.
///
/// # Errors
///
/// Returns whatever the node reports. Until a node is implemented that is
/// [`Error::InternalInvariant`], per the interim arrangement this module's
/// documentation states.
pub(crate) fn dispatch(cli: &Cli) -> Result<(), Error> {
    route(&mut std::io::stdout().lock(), cli)
}

/// The one match from a parsed invocation to the node's action.
///
/// Every arm is either a group node, which prints its own help and succeeds per
/// `FR-CLI-007`, or a leaf, which reports the interim `70` until the sprint
/// that owns it replaces the arm.
///
/// The writer is a parameter rather than standard output taken directly, so
/// that what a group node prints is observable from a test without a process.
///
/// # Errors
///
/// Returns [`Error::InternalInvariant`] for a leaf with no implementation, and
/// [`Error::StdoutUnwritable`] where a group node's help could not be written.
fn route<W: Write>(out: &mut W, cli: &Cli) -> Result<(), Error> {
    match &cli.command {
        None => group_help(out, "tpl"),

        Some(Command::Schema(schema)) => match &schema.command {
            None => group_help(out, "tpl schema"),
            Some(schema::Command::Info) => not_yet_implemented!("tpl schema info"),
            Some(schema::Command::Tables) => not_yet_implemented!("tpl schema tables"),
            Some(schema::Command::Table) => not_yet_implemented!("tpl schema table"),
            Some(schema::Command::Views) => not_yet_implemented!("tpl schema views"),
            Some(schema::Command::View) => not_yet_implemented!("tpl schema view"),
            Some(schema::Command::Routines) => not_yet_implemented!("tpl schema routines"),
            Some(schema::Command::Routine) => not_yet_implemented!("tpl schema routine"),
            Some(schema::Command::Dump) => not_yet_implemented!("tpl schema dump"),
        },

        Some(Command::Template(template)) => match &template.command {
            None => group_help(out, "tpl template"),
            Some(template::Command::List) => not_yet_implemented!("tpl template list"),
            Some(template::Command::Show) => not_yet_implemented!("tpl template show"),
            Some(template::Command::Check) => not_yet_implemented!("tpl template check"),
            Some(template::Command::Path) => not_yet_implemented!("tpl template path"),
        },

        Some(Command::Render) => not_yet_implemented!("tpl render"),

        Some(Command::Cache(cache)) => match &cache.command {
            None => group_help(out, "tpl cache"),
            Some(cache::Command::Load) => not_yet_implemented!("tpl cache load"),
            Some(cache::Command::Clean) => not_yet_implemented!("tpl cache clean"),
            Some(cache::Command::Status) => not_yet_implemented!("tpl cache status"),
        },

        Some(Command::Cfg(config)) => match &config.command {
            None => group_help(out, "tpl cfg"),
            Some(cfg::Command::Get) => not_yet_implemented!("tpl cfg get"),
            Some(cfg::Command::Set) => not_yet_implemented!("tpl cfg set"),
            Some(cfg::Command::Unset) => not_yet_implemented!("tpl cfg unset"),
            Some(cfg::Command::List) => not_yet_implemented!("tpl cfg list"),
            Some(cfg::Command::Database(database)) => match &database.command {
                None => group_help(out, "tpl cfg database"),
                Some(cfg::DatabaseCommand::Add) => not_yet_implemented!("tpl cfg database add"),
                Some(cfg::DatabaseCommand::List) => not_yet_implemented!("tpl cfg database list"),
                Some(cfg::DatabaseCommand::Show) => not_yet_implemented!("tpl cfg database show"),
                Some(cfg::DatabaseCommand::Update) => {
                    not_yet_implemented!("tpl cfg database update")
                }
                Some(cfg::DatabaseCommand::Remove) => {
                    not_yet_implemented!("tpl cfg database remove")
                }
                Some(cfg::DatabaseCommand::Test) => not_yet_implemented!("tpl cfg database test"),
            },
        },

        Some(Command::Init) => not_yet_implemented!("tpl init"),
        Some(Command::Help) => not_yet_implemented!("tpl help"),
        Some(Command::Version) => not_yet_implemented!("tpl version"),
    }
}

/// Writes the help of a group node invoked with no child, per `FR-CLI-007` and
/// `FR-HELP-025`.
///
/// **This body is a placeholder, and the call site is the point of it.**
/// `FR-HELP-025` requires exactly the text `tpl help <node>` would print, and
/// `BR-HELP-001` requires that text to be byte-identical to
/// `tpl <node> --help`; the renderer that composes all seven sections of
/// `FR-HELP-006` is a later task, and it replaces what is written here without
/// moving where it is written from. What the placeholder prints names the node
/// whose help is owed and claims to be nothing else.
///
/// # Errors
///
/// Returns [`Error::StdoutUnwritable`] where the stream refused the write. The
/// `0`-or-`74` distinction of `FR-ERR-025` and `FR-ERR-026` belongs to the
/// writer of `output`, and the renderer emits through it.
fn group_help<W: Write>(out: &mut W, path: &str) -> Result<(), Error> {
    writeln!(out, "the help of '{path}' is not rendered yet")
        .map_err(|returned| Error::StdoutUnwritable { returned })
}

#[cfg(test)]
mod tests {
    use clap::error::ErrorKind;

    use super::{Cli, Command, Error, cfg, parse_from, route, schema, tree};

    /// Every leaf of the tree of `FR-CLI-002`, by the path a caller writes.
    const LEAVES: [&[&str]; 29] = [
        &["schema", "info"],
        &["schema", "tables"],
        &["schema", "table"],
        &["schema", "views"],
        &["schema", "view"],
        &["schema", "routines"],
        &["schema", "routine"],
        &["schema", "dump"],
        &["template", "list"],
        &["template", "show"],
        &["template", "check"],
        &["template", "path"],
        &["render"],
        &["cache", "load"],
        &["cache", "clean"],
        &["cache", "status"],
        &["cfg", "get"],
        &["cfg", "set"],
        &["cfg", "unset"],
        &["cfg", "list"],
        &["cfg", "database", "add"],
        &["cfg", "database", "list"],
        &["cfg", "database", "show"],
        &["cfg", "database", "update"],
        &["cfg", "database", "remove"],
        &["cfg", "database", "test"],
        &["init"],
        &["help"],
        &["version"],
    ];

    /// The six group nodes of `FR-CLI-008`, by the path a caller writes.
    const GROUPS: [&[&str]; 6] = [
        &[],
        &["schema"],
        &["template"],
        &["cache"],
        &["cfg"],
        &["cfg", "database"],
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

    /// The invocation `path` parses to, which every node of the tree must.
    fn parsed(path: &[&str]) -> Cli {
        parse_from(argv(path, &[])).expect("the tree declares this node")
    }

    /// What the node `path` names does, and what it wrote while doing it.
    fn outcome(path: &[&str]) -> (Result<(), Error>, String) {
        let invocation = parsed(path);
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

    #[test]
    fn every_node_of_the_tree_parses() {
        // FR-CLI-002: the tree is closed, and these are what it is closed
        // around. The two constants are the tree of the requirement written
        // out, and the count is what makes an unreachable node a failure here
        // rather than a discovery later.
        let declared = node_paths();

        assert_eq!(declared.len(), LEAVES.len() + GROUPS.len());

        for path in LEAVES.iter().chain(GROUPS.iter()) {
            let reached = node_path(path);

            assert!(declared.contains(&reached), "{reached} is not a node");
            parse_from(argv(path, &[])).unwrap_or_else(|error| panic!("{reached}: {error}"));
        }
    }

    #[test]
    fn every_leaf_reports_the_interim_seventy_naming_its_command_path() {
        // The interim arrangement this module documents: a leaf parses, has no
        // implementation, and says so as FR-ERR-030 does — never as a success
        // and never as a usage error the caller could act on.
        for path in LEAVES {
            let (result, written) = outcome(path);
            let reported = result.expect_err("no leaf is implemented yet");

            assert_eq!(reported.exit_code(), 70, "{path:?}");
            assert!(
                reported.to_string().contains(&node_path(path)),
                "{path:?} is not named by: {reported}"
            );
            assert!(written.is_empty(), "{path:?} wrote {written:?}");
        }
    }

    #[test]
    fn a_group_node_invoked_with_no_child_prints_its_own_help_and_succeeds() {
        // FR-CLI-007 and FR-HELP-025: help on stdout, exit 0. What the
        // placeholder writes is not the help text, which is a later task; that
        // it is written, at this call site, for exactly these six nodes, is
        // this task's.
        for path in GROUPS {
            let (result, written) = outcome(path);

            result.unwrap_or_else(|error| panic!("{path:?}: {error}"));
            assert!(
                written.contains(&node_path(path)),
                "{path:?} is not named by: {written:?}"
            );
        }
    }

    #[test]
    fn the_group_nodes_are_exactly_the_six_of_the_requirement() {
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
    fn the_top_level_commands_are_exactly_the_eight_of_the_requirement() {
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
    fn each_of_the_seven_aliases_resolves_to_its_canonical_node() {
        // FR-CLI-011, the whole table.
        for (alias, canonical) in [
            (["schema", "tbls"], ["schema", "tables"]),
            (["schema", "tbl"], ["schema", "table"]),
            (["schema", "vws"], ["schema", "views"]),
            (["schema", "vw"], ["schema", "view"]),
            (["schema", "rtns"], ["schema", "routines"]),
            (["schema", "rtn"], ["schema", "routine"]),
            (["cfg", "db"], ["cfg", "database"]),
        ] {
            assert_eq!(parsed(&alias), parsed(&canonical), "{alias:?}");
        }

        // The seventh alias reaches the level beneath it too, since it names a
        // group node rather than a leaf.
        assert_eq!(
            parsed(&["cfg", "db", "add"]),
            parsed(&["cfg", "database", "add"])
        );
    }

    #[test]
    fn no_alias_other_than_the_seven_is_accepted() {
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
    fn a_command_is_not_inferred_from_a_prefix_of_its_name() {
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
    fn a_long_flag_is_not_inferred_from_a_prefix_of_its_name() {
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
    }

    #[test]
    fn an_undeclared_token_is_never_handed_to_an_executable_on_the_path() {
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
    fn a_global_flag_is_accepted_in_every_position_of_the_command_line() {
        // FR-CLI-024 and FR-GLOB-002: the three lines of the requirement are
        // one invocation.
        let before = parse_from(argv(&[], &["-d", "shop", "schema", "tables"])).expect("parses");
        let after = parse_from(argv(&["schema", "tables"], &["-d", "shop"])).expect("parses");
        let between = parse_from(argv(&["schema"], &["-d", "shop", "tables"])).expect("parses");

        assert_eq!(before, after);
        assert_eq!(before, between);
        assert_eq!(before.globals.database.as_deref(), Some("shop"));

        // And at the deepest node of the tree, where the flag sits between two
        // subcommands rather than between a command and its subcommand.
        assert_eq!(
            parse_from(argv(&["cfg"], &["-d", "shop", "database", "add"])).expect("parses"),
            parse_from(argv(&["cfg", "database", "add"], &["-d", "shop"])).expect("parses")
        );
    }

    #[test]
    fn every_node_accepts_every_one_of_the_seven_global_flags() {
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

        for path in LEAVES.iter().chain(GROUPS.iter()) {
            let invocation =
                parse_from(argv(path, &seven)).unwrap_or_else(|error| panic!("{path:?}: {error}"));
            let globals = &invocation.globals;

            assert_eq!(globals.database.as_deref(), Some("shop"), "{path:?}");
            assert_eq!(
                globals.tpl_dir.as_deref(),
                Some(std::path::Path::new("/srv/project/.tpl")),
                "{path:?}"
            );
            assert_eq!(
                globals.timeout.map(std::num::NonZeroU64::get),
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
    fn the_seven_global_flags_are_the_whole_of_the_global_set() {
        // FR-GLOB-001 declares exactly seven and no others.
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
    }

    #[test]
    fn the_short_flag_set_of_the_whole_tree_is_the_five_of_the_requirement() {
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
    fn the_parser_generates_no_help_of_its_own_at_any_node() {
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
        assert_eq!(parsed(&["help"]).command, Some(Command::Help));
    }

    #[test]
    fn a_flag_a_node_does_not_declare_is_refused_there() {
        // FR-CLI-019, to the extent this task's surface can reach it: no node
        // declares a local flag yet, so every flag but the seven is unknown
        // everywhere.
        for trailing in [&["--format", "json"][..], &["--pretty"][..], &["-x"][..]] {
            let refused =
                parse_from(argv(&["schema", "tables"], trailing)).expect_err("not declared");

            assert_eq!(refused.kind(), ErrorKind::UnknownArgument, "{trailing:?}");
        }
    }

    #[test]
    fn a_flag_and_its_value_are_case_sensitive() {
        // FR-CLI-020: neither is normalised.
        assert_eq!(
            parse_from(argv(&["schema", "tables"], &["-d", "Shop"]))
                .expect("parses")
                .globals
                .database
                .as_deref(),
            Some("Shop")
        );

        assert_eq!(
            parse_from(argv(&["schema", "tables"], &["-D", "shop"]))
                .expect_err("no -D")
                .kind(),
            ErrorKind::UnknownArgument
        );
    }

    #[test]
    fn the_nested_node_of_the_third_level_is_reached_through_its_parent() {
        // The one three-level path of FR-CLI-002, and the one place a group
        // node is a child of a group node.
        let invocation = parsed(&["cfg", "database", "remove"]);

        let Some(Command::Cfg(config)) = &invocation.command else {
            panic!("`cfg` did not parse to its own node: {invocation:?}");
        };
        let Some(cfg::Command::Database(database)) = &config.command else {
            panic!("`database` did not parse to its own node: {config:?}");
        };

        assert_eq!(database.command, Some(cfg::DatabaseCommand::Remove));
    }

    #[test]
    fn an_alias_and_its_canonical_name_reach_the_same_variant() {
        let invocation = parsed(&["schema", "rtns"]);

        let Some(Command::Schema(read)) = &invocation.command else {
            panic!("`schema` did not parse to its own node: {invocation:?}");
        };

        assert_eq!(read.command, Some(schema::Command::Routines));
    }
}
