//! The configuration arm: `tpl cfg`, and the database entries beneath it.
//!
//! This arm carries the only three-level path of the tree, and both of its
//! interior nodes are groups, per `FR-CLI-008`: `tpl cfg` and
//! `tpl cfg database` each have no action of their own (`FR-CLI-009`), an
//! optional child, and their own help at exit `0` when invoked bare
//! (`FR-CLI-007`).
//!
//! The seventh alias of `FR-CLI-011` — `db` for `database` — is declared here,
//! and it is the only alias of the arm. It is visible for the reason every
//! other alias is: `FR-CLI-013` shows each one in the help of its parent node
//! and in the JSON command tree.

use clap::{Args, Subcommand};

/// The `tpl cfg` group node.
#[derive(Debug, Clone, PartialEq, Eq, Args)]
pub(crate) struct Cfg {
    /// The subcommand invoked, or [`None`] where the group node was invoked
    /// bare.
    #[command(subcommand)]
    pub(crate) command: Option<Command>,
}

/// The five children of `tpl cfg`.
#[derive(Debug, Clone, PartialEq, Eq, Subcommand)]
pub(crate) enum Command {
    /// Reads one configuration key.
    Get,

    /// Writes one configuration key.
    Set,

    /// Removes one configuration key.
    Unset,

    /// Lists the configuration keys the project carries.
    List,

    /// The database entries of `.tpl/.cfg`.
    #[command(visible_alias = "db")]
    Database(Database),
}

/// The `tpl cfg database` group node.
#[derive(Debug, Clone, PartialEq, Eq, Args)]
pub(crate) struct Database {
    /// The subcommand invoked, or [`None`] where the group node was invoked
    /// bare.
    #[command(subcommand)]
    pub(crate) command: Option<DatabaseCommand>,
}

/// The six children of `tpl cfg database`.
#[derive(Debug, Clone, PartialEq, Eq, Subcommand)]
pub(crate) enum DatabaseCommand {
    /// Adds a database entry.
    Add,

    /// Lists the database entries.
    List,

    /// Shows one database entry.
    Show,

    /// Changes one database entry.
    Update,

    /// Removes one database entry.
    Remove,

    /// Opens a connection with one database entry and reports the outcome.
    Test,
}
