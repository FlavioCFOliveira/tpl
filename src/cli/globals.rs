//! The seven global flags of `FR-GLOB-001`, declared once and accepted
//! everywhere.
//!
//! `FR-GLOB-002` gives every node of the tree every one of the seven, and
//! `FR-CLI-024` frees their **position**: before the command, between a command
//! and its subcommand, after the positional arguments, and among the local
//! flags. The mechanism is `clap`'s `global = true`, set on each of the seven
//! and on nothing else: a global argument is propagated to every subcommand
//! when the parser is built, and the value it matched at any depth is
//! propagated back up to the root, which is where this struct reads it from.
//!
//! The alternative — flattening this struct into every node and merging the
//! seven copies afterwards — was rejected. It declares the same flag once per
//! node, so `FR-GLOB-024`'s promise that a short form means one thing wherever
//! it appears would hold by review rather than by construction, and a node
//! whose copy was forgotten would refuse a flag `FR-GLOB-002` requires it to
//! accept.
//!
//! What is deliberately **not** here: the refusal of `-q` together with `-v`
//! (`FR-CLI-015`), the repetition rule of `FR-CLI-014`, and the saturation of
//! `FR-CLI-016`. All three are parsing rules, owned by the module that
//! intercepts the parser's own failures, and a `conflicts_with` written here
//! would decide one of them in the parser's words rather than in the four
//! labelled lines of `FR-ERR-008`.

use std::num::NonZeroU64;
use std::path::PathBuf;

use clap::{ArgAction, Args};

/// The seven flags every node accepts, in any position.
///
/// Each field carries the flag as `FR-GLOB-001` declares it, with the short
/// form that table gives it and no other: `FR-GLOB-024` makes `-d`, `-v`, `-q`,
/// `-h` and `-V` the complete short-flag set of the tool, and `--tpl-dir` and
/// `--timeout` have none.
#[derive(Debug, Clone, PartialEq, Eq, Args)]
pub(crate) struct Globals {
    /// The `[database.<name>]` entry of `.tpl/.cfg` this invocation uses
    /// (`FR-GLOB-004`).
    ///
    /// Absent, the entry named by `core.database` applies (`FR-GLOB-005`), and
    /// `FR-GLOB-008` requires the two to stay distinguishable — which is what
    /// [`Option`] carries here, rather than a resolved name.
    #[arg(short = 'd', long = "database", value_name = "NAME", global = true)]
    pub(crate) database: Option<String>,

    /// The `.tpl` folder to use, naming it explicitly and suppressing
    /// discovery (`FR-GLOB-009`).
    #[arg(long = "tpl-dir", value_name = "PATH", global = true)]
    pub(crate) tpl_dir: Option<PathBuf>,

    /// The overall wall-clock budget for the invocation, in seconds
    /// (`FR-GLOB-011`).
    ///
    /// It has no default: absent, the invocation carries no overall budget and
    /// is bounded only by the per-phase deadlines of `FR-CONF-005`. The type is
    /// [`NonZeroU64`] because the requirement's value is a **positive**
    /// integer, so a budget of zero seconds is refused where it is written
    /// rather than where it would expire.
    #[arg(long = "timeout", value_name = "SECONDS", global = true)]
    pub(crate) timeout: Option<NonZeroU64>,

    /// How many times `-v/--verbose` was given (`FR-GLOB-014`).
    ///
    /// One occurrence is `INFO`, two `DEBUG`, three `TRACE`. The count is
    /// carried raw: `FR-CLI-016` saturates it above three, and that is a
    /// parsing rule rather than a property of the flag.
    #[arg(short = 'v', long = "verbose", action = ArgAction::Count, global = true)]
    pub(crate) verbose: u8,

    /// Whether `-q/--quiet` was given, lowering the diagnostic level to errors
    /// only (`FR-GLOB-015`).
    #[arg(short = 'q', long = "quiet", action = ArgAction::SetTrue, global = true)]
    pub(crate) quiet: bool,

    /// Whether `-h/--help` was given, at whichever node it appeared
    /// (`FR-GLOB-019`).
    ///
    /// It is a flag of this tree and not the parser's own: `OD-07` renders all
    /// seven sections of `FR-HELP-006` in `tpl`, so `disable_help_flag` is set
    /// on every node and this is the only `--help` the tree declares.
    #[arg(short = 'h', long = "help", action = ArgAction::SetTrue, global = true)]
    pub(crate) help: bool,

    /// Whether `-V/--version` was given (`FR-GLOB-020`).
    #[arg(short = 'V', long = "version", action = ArgAction::SetTrue, global = true)]
    pub(crate) version: bool,
}
