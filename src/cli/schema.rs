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
//! The arguments and local flags of these eight commands are not here: this
//! module declares the nodes, and each command's own parameters arrive with the
//! module that owns them.

use clap::{Args, Subcommand};

/// The `tpl schema` group node.
#[derive(Debug, Clone, PartialEq, Eq, Args)]
pub(crate) struct Schema {
    /// The subcommand invoked, or [`None`] where the group node was invoked
    /// bare.
    #[command(subcommand)]
    pub(crate) command: Option<Command>,
}

/// The eight children of `tpl schema`.
#[derive(Debug, Clone, PartialEq, Eq, Subcommand)]
pub(crate) enum Command {
    /// Reports the server and the selected database.
    Info,

    /// Lists the tables of the selected database.
    #[command(visible_alias = "tbls")]
    Tables,

    /// Describes one table.
    #[command(visible_alias = "tbl")]
    Table,

    /// Lists the views of the selected database.
    #[command(visible_alias = "vws")]
    Views,

    /// Describes one view.
    #[command(visible_alias = "vw")]
    View,

    /// Lists the routines — procedures and functions — of the selected
    /// database.
    #[command(visible_alias = "rtns")]
    Routines,

    /// Describes one routine.
    #[command(visible_alias = "rtn")]
    Routine,

    /// Writes the whole catalogue model of the selected database.
    Dump,
}
