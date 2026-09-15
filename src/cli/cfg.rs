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
//!
//! # The surface of the ten
//!
//! `FR-GLOB-021` gives `--format`, and with it `--pretty`, to the five
//! subcommands that answer with a document — `get`, `list`, `database list`,
//! `database show` and `database test`, the last by `FR-CFG-026`. The five that
//! change the file or name a key for removal have neither, so
//! `tpl cfg set core.database shop --format json` is the ordinary unknown-flag
//! `64` of `FR-CLI-019`.
//!
//! The nine flags of [`Entry`] are `FR-CFG-027` in full, and they are declared
//! by `database add` and `database update` and by nothing else. Two rules
//! between them are **not** declared on the arguments and are the command's:
//! `FR-CFG-016`, which requires `add` to be given either `--dsn` or at least
//! one discrete connection flag, and `FR-CFG-029`, which makes the two groups
//! mutually exclusive in one invocation. Both are refusals between arguments
//! rather than properties of one, and are left where [`super::local`] leaves
//! the two of its own.
//!
//! What is absent is a requirement in its own right. `FR-CFG-030` forbids a
//! `--password` or `-p` flag on any command, and `FR-GLOB-023` generalises it
//! to any flag whose purpose is to carry a password. A literal password reaches
//! the file through `tpl cfg set` or inside a `--dsn`, per `FR-CFG-031` and
//! `FR-CFG-032`, and `BR-CFG-003` states the position that makes those two
//! acceptable: `tpl` warns, and does not prevent.

use std::path::PathBuf;

use clap::{ArgAction, Args, Subcommand, ValueEnum};

use super::local;

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
    Get {
        /// The key to read, in the dotted form of `FR-CONF-002`.
        ///
        /// A key absent from the file is `66`, with a nearest-match suggestion
        /// over the keys that do exist, per `FR-CFG-007`.
        #[arg(value_name = "KEY")]
        key: String,

        /// `--format` and `--pretty`, per `FR-GLOB-021`.
        #[command(flatten)]
        output: local::Output,
    },

    /// Writes one configuration key.
    Set {
        /// The key to write, which `FR-CFG-009` restricts to the enumerated
        /// key space of `FR-CONF-002`.
        #[arg(value_name = "KEY")]
        key: String,

        /// The value to write, validated against the type `FR-CONF-002`
        /// declares for the key, per `FR-CFG-010`.
        ///
        /// A value given here is visible in the process table for the life of
        /// the invocation, per `FR-CFG-033`.
        #[arg(value_name = "VALUE")]
        value: String,
    },

    /// Removes one configuration key.
    Unset {
        /// The key or block to remove — either a leaf, such as
        /// `database.shop.host`, or a whole block, such as `database.shop`,
        /// per `FR-CFG-011`.
        #[arg(value_name = "KEY")]
        key: String,
    },

    /// Lists the configuration keys the project carries.
    List {
        /// `--format` and `--pretty`, per `FR-GLOB-021`.
        #[command(flatten)]
        output: local::Output,
    },

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

/// The mode of `--tls`, per `FR-CONF-013`.
///
/// The five are a closed set and are normative over the driver, per
/// `FR-CONF-036`, so they are a type rather than a string checked after
/// parsing: `FR-HELP-013` obliges the help to state the permitted values of
/// every flag, and `FR-HELP-021` derives them by introspecting this tree.
#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
pub(crate) enum TlsMode {
    /// No TLS.
    Disabled,

    /// Encrypt if the server allows it.
    Preferred,

    /// Always encrypt, without validating.
    Required,

    /// Also validate the certificate chain.
    VerifyCa,

    /// Also validate the hostname.
    VerifyIdentity,
}

/// The nine flags of `FR-CFG-027`, each mapping to one key of a
/// `[database.<name>]` block.
///
/// They are declared once and flattened into `add` and `update`, which is what
/// keeps one flag from meaning one thing to the command that creates an entry
/// and another to the command that changes it. None of them carries a short
/// form: `FR-GLOB-024` closes the short-flag space at five, and `FR-CFG-047`
/// says so again for the three the fifth edition added.
///
/// Every flag is optional here. `add` and `update` require different
/// combinations — `FR-CFG-016` and `FR-CFG-029` — and both are refusals
/// between arguments, left to the commands that make them.
///
/// Each of the nine carries a value, so each is declared repeatable and the
/// repetition is refused by [`super::rules`], for the reason `OD-08` gives and
/// [`super::globals`] states: `FR-CLI-014` obliges the message to name **both
/// values**, and only the accumulated occurrences put both in hand. A field
/// here is every occurrence in the order written, reduced to at most one
/// before `add` or `update` reads it.
#[derive(Debug, Clone, PartialEq, Eq, Args)]
pub(crate) struct Entry {
    /// The whole connection as one URL, written to `database.<name>.dsn`.
    ///
    /// Mutually exclusive with the discrete connection flags, per
    /// `FR-CFG-029`. It is stored verbatim, a literal password included, per
    /// `FR-CFG-031`, and a value given here is visible in the process table
    /// for the life of the invocation, per `FR-CFG-033`.
    #[arg(long = "dsn", value_name = "URL", action = ArgAction::Append)]
    pub(crate) dsn: Vec<String>,

    /// The server host, written to `database.<name>.host`.
    #[arg(long = "host", value_name = "HOST", action = ArgAction::Append)]
    pub(crate) host: Vec<String>,

    /// The server port, written to `database.<name>.port`.
    ///
    /// Absent, the entry carries no port and the default of `FR-CONF-002`,
    /// `3306`, applies when the entry is read.
    #[arg(long = "port", value_name = "PORT", action = ArgAction::Append)]
    pub(crate) port: Vec<u16>,

    /// The user to authenticate as, written to `database.<name>.user`.
    #[arg(long = "user", value_name = "USER", action = ArgAction::Append)]
    pub(crate) user: Vec<String>,

    /// The database on the server, written to `database.<name>.database`.
    ///
    /// It is the one flag whose name differs from the key it writes, per
    /// `FR-CFG-028`: `schema` is the catalogue's own word for it, and it leaves
    /// the global `-d/--database` free to mean the entry label everywhere.
    #[arg(long = "schema", value_name = "NAME", action = ArgAction::Append)]
    pub(crate) schema: Vec<String>,

    /// The TLS mode, written to `database.<name>.tls`.
    ///
    /// Absent, the entry carries no mode and the default of `FR-CONF-013`,
    /// `verify-identity`, applies when the entry is read.
    #[arg(long = "tls", value_name = "MODE", value_enum, action = ArgAction::Append)]
    pub(crate) tls: Vec<TlsMode>,

    /// The command that produces the password, written to
    /// `database.<name>.password_command`.
    ///
    /// One string, split into the stored array by the POSIX quoting rules of
    /// `FR-CONF-025`. It accepts no array on the command line and is not
    /// repeatable, per `FR-CFG-046`. A value given here is visible in the
    /// process table for the life of the invocation, and `FR-CONF-017` admits
    /// no `${VAR}` expansion in this key, per `FR-CFG-033`.
    #[arg(
        long = "password-command",
        value_name = "COMMAND",
        action = ArgAction::Append
    )]
    pub(crate) password_command: Vec<String>,

    /// The certificate file trusted by `verify-ca` and `verify-identity`,
    /// written to `database.<name>.ca_file` (`FR-CONF-014`).
    #[arg(long = "ca-file", value_name = "PATH", action = ArgAction::Append)]
    pub(crate) ca_file: Vec<PathBuf>,

    /// The certificate directory trusted by `verify-ca` and `verify-identity`,
    /// written to `database.<name>.ca_path` (`FR-CONF-014`).
    #[arg(long = "ca-path", value_name = "PATH", action = ArgAction::Append)]
    pub(crate) ca_path: Vec<PathBuf>,
}

/// The six children of `tpl cfg database`.
#[derive(Debug, Clone, PartialEq, Eq, Subcommand)]
pub(crate) enum DatabaseCommand {
    /// Adds a database entry.
    Add {
        /// The name of the entry to create, which becomes the
        /// `[database.<name>]` block (`FR-CFG-015`).
        ///
        /// A name that is already taken is `64`, with a hint pointing at
        /// `tpl cfg database update`, per `FR-CFG-017`.
        #[arg(value_name = "NAME")]
        name: String,

        /// The nine flags of `FR-CFG-027`.
        #[command(flatten)]
        entry: Entry,
    },

    /// Lists the database entries.
    List {
        /// `--format` and `--pretty`, per `FR-GLOB-021`.
        #[command(flatten)]
        output: local::Output,
    },

    /// Shows one database entry.
    Show {
        /// The entry to print, with passwords redacted and `${VAR}` left
        /// exactly as written, per `FR-CFG-019` and `FR-CFG-021`.
        #[arg(value_name = "NAME")]
        name: String,

        /// `--format` and `--pretty`, per `FR-GLOB-021`.
        #[command(flatten)]
        output: local::Output,
    },

    /// Changes one database entry.
    Update {
        /// The entry to change. Only the fields named by the flags supplied
        /// are changed, per `FR-CFG-020`.
        #[arg(value_name = "NAME")]
        name: String,

        /// The nine flags of `FR-CFG-027`.
        #[command(flatten)]
        entry: Entry,
    },

    /// Removes one database entry.
    Remove {
        /// The entry to delete (`FR-CFG-022`).
        #[arg(value_name = "NAME")]
        name: String,
    },

    /// Opens a connection with one database entry and reports the outcome.
    Test {
        /// The entry to test, over the four steps of `FR-CFG-024`.
        #[arg(value_name = "NAME")]
        name: String,

        /// `--format` and `--pretty`, per `FR-CFG-026`.
        #[command(flatten)]
        output: local::Output,
    },
}
