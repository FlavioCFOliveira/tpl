//! The second arm of the tree: `tpl template`, which reads the project's
//! templates.
//!
//! The node is a group, per `FR-CLI-008`: no action of its own
//! (`FR-CLI-009`), an optional child, and its own help at exit `0` when
//! invoked bare (`FR-CLI-007`). None of its four children carries an alias —
//! `FR-CLI-011` declares seven aliases and none of them is here.
//!
//! # The surface of the four
//!
//! `FR-GLOB-021` gives `--format`, and with it `--pretty`, to `list` and
//! `path` and to neither of the other two: `show` prints template source
//! unaltered, per `FR-TMPL-015`, and `check` reports through its exit code, per
//! `FR-TMPL-020`. Neither has a second representation to choose between, so
//! `tpl template show x --format json` is the ordinary unknown-flag `64` of
//! `FR-CLI-019`.
//!
//! The four positional arguments differ in arity, and each difference is a
//! requirement rather than a convenience: `show` takes exactly one
//! (`FR-TMPL-016`); `check` takes any number, checking every template of the
//! project when given none (`FR-TMPL-018`, `FR-TMPL-019`); `path` takes at most
//! one, printing the template root when given none (`FR-TMPL-021`,
//! `FR-TMPL-022`); `list` takes none.
//!
//! A name is resolved by `FR-TMPL-006` and `FR-TMPL-007` throughout: it is the
//! path of the file relative to `.tpl/templates/`, with or without the `.jinja`
//! extension. That resolution is the command's work, not the parser's, so every
//! name here is a string.

use clap::{Args, Subcommand};

use super::local;

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
    List {
        /// `--format` and `--pretty`, per `FR-GLOB-021`.
        #[command(flatten)]
        output: local::Output,
    },

    /// Prints the source of one template.
    Show {
        /// The template to print, with or without its `.jinja` extension
        /// (`FR-TMPL-007`, `FR-TMPL-016`).
        ///
        /// A name that does not resolve is `66`, with a nearest-match
        /// suggestion over the template names that do exist, per
        /// `FR-TMPL-027`.
        #[arg(value_name = "NAME")]
        name: String,
    },

    /// Checks that one template compiles.
    Check {
        /// The templates to check, with or without their `.jinja` extension.
        ///
        /// Repeatable. Given none, every template of the project is checked,
        /// per `FR-TMPL-018`; given one or more, exactly those are, per
        /// `FR-TMPL-019`. The check is syntax analysis only and evaluates
        /// nothing, per `FR-TMPL-017`.
        #[arg(value_name = "NAME")]
        names: Vec<String>,
    },

    /// Reports where a template name resolves to.
    Path {
        /// The template whose path to print, with or without its `.jinja`
        /// extension (`FR-TMPL-022`).
        ///
        /// Optional: given none, the absolute path of the template root is
        /// printed instead, per `FR-TMPL-021`.
        #[arg(value_name = "NAME")]
        name: Option<String>,

        /// `--format` and `--pretty`, per `FR-GLOB-021`.
        #[command(flatten)]
        output: local::Output,
    },
}
