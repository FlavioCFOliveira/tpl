//! The first arm of the tree: `tpl schema`, which reads the database structure.
//!
//! The node is a group, per `FR-CLI-008`, so it carries no action of its own
//! (`FR-CLI-009`) and its child is optional — a bare `tpl schema` prints its
//! own help and exits `0`, per `FR-CLI-007`.
//!
//! Six of the seven aliases of `FR-CLI-011` are declared here. They are
//! declared as **visible** aliases because `FR-CLI-013` shows every alias in
//! the help of its parent node and in the JSON command tree, and because the
//! tree is what `FR-HELP-021` introspects to build that document.
//!
//! # The surface of the eight
//!
//! Three requirements fix it, and between them they leave no subcommand
//! undecided:
//!
//! | Requirement | What it gives, and to which of the eight |
//! |---|---|
//! | `FR-SCH-024` | `--direct` and `--no-cache`, to all eight |
//! | `FR-SCH-023` | `--format` and `--pretty`, to the seven that are not `dump` |
//! | `FR-SCH-020` | `--pretty` alone, to `dump` |
//!
//! `dump` is the exception in both directions and deliberately so:
//! `FR-SCH-019` withholds `--format` because a flag with one permitted value is
//! not a choice, and `FR-SCH-021` withholds `--pattern` because a partial
//! context would fail at render time on a missing object. Neither is declared,
//! so each is the ordinary unknown-flag `64` of `FR-CLI-019` rather than a case
//! of its own.
//!
//! `FR-SCH-005` gives `table`, `view` and `routine` one positional argument
//! each, and states the asymmetry it creates with `render` and `cache`, which
//! name an object by flag: here the subcommand already carries the object's
//! type, so the name needs no flag to disambiguate it.

use clap::{ArgAction, Args, Subcommand};

use super::local;

/// The `tpl schema` group node.
#[derive(Debug, Clone, PartialEq, Eq, Args)]
pub(crate) struct Schema {
    /// The subcommand invoked, or [`None`] where the group node was invoked
    /// bare.
    #[command(subcommand)]
    pub(crate) command: Option<Command>,
}

/// `--pattern`, declared by exactly the three listing subcommands of
/// `FR-SCH-011`.
///
/// It is declared once and flattened three times for the reason
/// [`local`] gives for the flags it holds. `FR-SCH-015` forbids it on every
/// other command, `dump` included.
#[derive(Debug, Clone, PartialEq, Eq, Args)]
pub(crate) struct Pattern {
    /// Filter the listing by object name, in MariaDB `LIKE` syntax
    /// (`FR-SCH-012`).
    ///
    /// `%` matches any sequence of characters including the empty one, `_`
    /// matches exactly one character, and `\%` and `\_` match those two
    /// literally. The pattern is evaluated in memory and is never sent to the
    /// server, per `FR-SCH-013`, and it folds case over ASCII `A-Z` and `a-z`
    /// only, per `FR-SCH-014`.
    ///
    /// Every occurrence, for the reason [`super::globals`] gives: `FR-CLI-014`
    /// refuses a second one over the occurrences this declaration
    /// accumulates.
    #[arg(long = "pattern", value_name = "PATTERN", action = ArgAction::Append)]
    pub(crate) pattern: Vec<String>,
}

/// The eight children of `tpl schema`.
#[derive(Debug, Clone, PartialEq, Eq, Subcommand)]
pub(crate) enum Command {
    /// Reports the server and the selected database.
    Info {
        /// `--format` and `--pretty`, per `FR-SCH-023`.
        #[command(flatten)]
        output: local::Output,

        /// `--direct` and `--no-cache`, per `FR-SCH-024`.
        #[command(flatten)]
        caching: local::Caching,
    },

    /// Lists the tables of the selected database.
    #[command(visible_alias = "tbls")]
    Tables {
        /// `--pattern`, per `FR-SCH-011`.
        #[command(flatten)]
        filter: Pattern,

        /// `--format` and `--pretty`, per `FR-SCH-023`.
        #[command(flatten)]
        output: local::Output,

        /// `--direct` and `--no-cache`, per `FR-SCH-024`.
        #[command(flatten)]
        caching: local::Caching,
    },

    /// Describes one table.
    #[command(visible_alias = "tbl")]
    Table {
        /// The table to describe (`FR-SCH-005`).
        ///
        /// A name absent from the selected database is `66`, with a
        /// nearest-match suggestion over the tables that do exist, per
        /// `FR-SCH-010`.
        #[arg(value_name = "NAME")]
        name: String,

        /// `--format` and `--pretty`, per `FR-SCH-023`.
        #[command(flatten)]
        output: local::Output,

        /// `--direct` and `--no-cache`, per `FR-SCH-024`.
        #[command(flatten)]
        caching: local::Caching,
    },

    /// Lists the views of the selected database.
    #[command(visible_alias = "vws")]
    Views {
        /// `--pattern`, per `FR-SCH-011`.
        #[command(flatten)]
        filter: Pattern,

        /// `--format` and `--pretty`, per `FR-SCH-023`.
        #[command(flatten)]
        output: local::Output,

        /// `--direct` and `--no-cache`, per `FR-SCH-024`.
        #[command(flatten)]
        caching: local::Caching,
    },

    /// Describes one view.
    #[command(visible_alias = "vw")]
    View {
        /// The view to describe (`FR-SCH-005`).
        ///
        /// A name absent from the selected database is `66`, with a
        /// nearest-match suggestion over the views that do exist, per
        /// `FR-SCH-010`.
        #[arg(value_name = "NAME")]
        name: String,

        /// `--format` and `--pretty`, per `FR-SCH-023`.
        #[command(flatten)]
        output: local::Output,

        /// `--direct` and `--no-cache`, per `FR-SCH-024`.
        #[command(flatten)]
        caching: local::Caching,
    },

    /// Lists the routines — procedures and functions — of the selected
    /// database.
    #[command(visible_alias = "rtns")]
    Routines {
        /// `--pattern`, per `FR-SCH-011`.
        ///
        /// There is no `--type` beside it: `FR-SCH-008` refuses one, and
        /// selecting one kind from a listing is done downstream from
        /// `--format json`.
        #[command(flatten)]
        filter: Pattern,

        /// `--format` and `--pretty`, per `FR-SCH-023`.
        #[command(flatten)]
        output: local::Output,

        /// `--direct` and `--no-cache`, per `FR-SCH-024`.
        #[command(flatten)]
        caching: local::Caching,
    },

    /// Describes one routine.
    #[command(visible_alias = "rtn")]
    Routine {
        /// The routine to describe (`FR-SCH-005`).
        ///
        /// It accepts the qualified forms `procedure:<name>` and
        /// `function:<name>` as well as a bare name, per `FR-SCH-008`. A bare
        /// name matching both a procedure and a function is `64`, and a name
        /// matching neither is `66`, per `FR-SCH-010`.
        #[arg(value_name = "NAME")]
        name: String,

        /// `--format` and `--pretty`, per `FR-SCH-023`.
        #[command(flatten)]
        output: local::Output,

        /// `--direct` and `--no-cache`, per `FR-SCH-024`.
        #[command(flatten)]
        caching: local::Caching,
    },

    /// Writes the whole catalogue model of the selected database.
    Dump {
        /// `--pretty`, standing alone because the result is JSON and nothing
        /// else, per `FR-SCH-020` and `FR-OUT-010`.
        #[command(flatten)]
        pretty: local::Pretty,

        /// `--direct` and `--no-cache`, per `FR-SCH-024`.
        #[command(flatten)]
        caching: local::Caching,
    },
}
