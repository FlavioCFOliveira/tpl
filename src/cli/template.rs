//! The second arm of the tree: `tpl template`, which reads the project's
//! templates.
//!
//! The node is a group, per `FR-CLI-008`: no action of its own
//! (`FR-CLI-009`), an optional child, and its own help at exit `0` when
//! invoked bare (`FR-CLI-007`). None of its four children carries an alias —
//! `FR-CLI-011` declares seven aliases and none of them is here.

use clap::{Args, Subcommand};

/// The `tpl template` group node.
#[derive(Debug, Clone, PartialEq, Eq, Args)]
pub(crate) struct Template {
    /// The subcommand invoked, or [`None`] where the group node was invoked
    /// bare.
    #[command(subcommand)]
    pub(crate) command: Option<Command>,
}

/// The four children of `tpl template`.
#[derive(Debug, Clone, PartialEq, Eq, Subcommand)]
pub(crate) enum Command {
    /// Lists the templates the project carries.
    List,

    /// Prints the source of one template.
    Show,

    /// Checks that one template compiles.
    Check,

    /// Reports where a template name resolves to.
    Path,
}
