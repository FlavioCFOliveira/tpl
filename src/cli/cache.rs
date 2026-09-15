//! The catalogue cache arm: `tpl cache`.
//!
//! The node is a group, per `FR-CLI-008`: no action of its own
//! (`FR-CLI-009`), an optional child, and its own help at exit `0` when
//! invoked bare (`FR-CLI-007`). None of its three children carries an alias.

use clap::{Args, Subcommand};

/// The `tpl cache` group node.
#[derive(Debug, Clone, PartialEq, Eq, Args)]
pub(crate) struct Cache {
    /// The subcommand invoked, or [`None`] where the group node was invoked
    /// bare.
    #[command(subcommand)]
    pub(crate) command: Option<Command>,
}

/// The three children of `tpl cache`.
#[derive(Debug, Clone, PartialEq, Eq, Subcommand)]
pub(crate) enum Command {
    /// Reads the catalogue and writes it to the cache.
    Load,

    /// Removes the cached catalogue.
    Clean,

    /// Reports what the cache holds.
    Status,
}
