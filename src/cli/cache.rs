//! The catalogue cache arm: `tpl cache`.
//!
//! The node is a group, per `FR-CLI-008`: no action of its own
//! (`FR-CLI-009`), an optional child, and its own help at exit `0` when
//! invoked bare (`FR-CLI-007`). None of its three children carries an alias.
//!
//! # The surface of the three
//!
//! `FR-CACHE-024` gives `load` and `clean` the three object flags of
//! [`local::Object`], in the spellings `tpl render` uses, and `FR-CACHE-022`
//! and `FR-CACHE-023` make the absence of all three mean *the whole catalogue
//! of the selected entry*. `status` names no object: it reports on the cache
//! rather than acting on one of its members, per `FR-CACHE-025`.
//!
//! The two cache flags are split between the three, and the split is exact.
//! `FR-CACHE-017` declares `--direct` and `--no-cache` on `load` alone, and
//! `FR-CACHE-020` withholds both from `clean` and `status`, where supplying
//! either is the ordinary unknown-flag `64` of `FR-CLI-019`. On `load` itself
//! the two are declared and then treated differently by the command:
//! `FR-CACHE-018` accepts `--direct` and ignores it, because reading the server
//! is what the command does, and `FR-CACHE-019` makes `--no-cache` a `64`,
//! because loading without storing is a contradiction. Both are refusals the
//! command makes about a flag it declares — the alternative, not declaring
//! `--no-cache`, would report the contradiction as an unknown flag and so say
//! the wrong thing.
//!
//! `FR-CACHE-027` gives `status` `--format` and `--pretty`, and gives them to
//! neither of the other two.

use clap::{Args, Subcommand};

use super::local;

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
    Load {
        /// `--table`, `--view` and `--routine`, per `FR-CACHE-024`.
        #[command(flatten)]
        object: local::Object,

        /// `--direct` and `--no-cache`, per `FR-CACHE-017`.
        #[command(flatten)]
        caching: local::Caching,
    },

    /// Removes the cached catalogue.
    Clean {
        /// `--table`, `--view` and `--routine`, per `FR-CACHE-024`.
        #[command(flatten)]
        object: local::Object,
    },

    /// Reports what the cache holds.
    Status {
        /// `--format` and `--pretty`, per `FR-CACHE-027`.
        #[command(flatten)]
        output: local::Output,
    },
}
