//! The local flags that more than one command declares.
//!
//! `FR-GLOB-021` keeps four flags deliberately **out** of the global set:
//! `--format`, `--pretty`, `--direct` and `--no-cache` are declared only by the
//! commands its table names, and rejected as unknown flags everywhere else, per
//! `FR-CLI-019`. `FR-RND-003` and `FR-CACHE-024` add a fifth family: the three
//! object flags `--table`, `--view` and `--routine`, which `render`,
//! `cache load` and `cache clean` spell identically.
//!
//! Each family is declared **once**, here, and flattened into every command the
//! requirement names. That is the arrangement [`super::globals`] uses for the
//! seven flags of `FR-GLOB-001`, and it holds for the same reason: a flag
//! written out once per command is a flag whose spelling, value type, default
//! and help can drift between two nodes that one row of `FR-GLOB-021` names
//! together. What does **not** carry over is `global = true`: nothing here is
//! global, and the set of nodes that flattens each struct is exactly the set the
//! requirement names.
//!
//! None of these flags carries a short form, per `FR-GLOB-024`.
//!
//! What is deliberately **not** here: `FR-OUT-009`, which makes `--pretty`
//! without `--format json` a `64` on a command that declares both; and
//! `FR-RND-005`, which refuses more than one kind of object flag in one
//! invocation. Neither is a property of an argument — each is a refusal
//! *between* two of them — and a `conflicts_with` written here would reach the
//! caller in the parser's words rather than in the four labelled lines of
//! `FR-ERR-008`. It is the same line [`super::globals`] draws for
//! `FR-CLI-015`.

use clap::{ArgAction, Args, ValueEnum};

/// The value of `--format`, per `FR-OUT-001`.
///
/// The two are a closed set, so they are a type rather than a string checked
/// after parsing: `FR-HELP-013` obliges the help to state the permitted values
/// of every flag, and `FR-HELP-021` derives them by introspecting this tree.
#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
pub(crate) enum Format {
    /// Aligned columns laid out for a person, per `FR-OUT-006`. Not a
    /// contract: anything parsing output must use `json`, per `FR-OUT-004`.
    Text,

    /// The envelope of `FR-OUT-024`, on one line unless `--pretty` is given.
    Json,
}

/// `--pretty`, declared by the seventeen commands of `FR-GLOB-021` — every
/// command that declares `--format`, plus `tpl schema dump`.
///
/// It is a struct of its own, flattened into [`Output`] as well as into
/// `tpl schema dump`, so that the flag is declared exactly once for both
/// populations. `FR-SCH-020` and `FR-OUT-010` are why the smaller one exists:
/// `dump` emits JSON and nothing else, so `--pretty` stands there without any
/// accompanying `--format`.
#[derive(Debug, Clone, PartialEq, Eq, Args)]
pub(crate) struct Pretty {
    /// Indent JSON output by two spaces, one key per line (`FR-OUT-008`).
    #[arg(long = "pretty", action = ArgAction::SetTrue)]
    pub(crate) pretty: bool,
}

/// `--format` and `--pretty` together, declared by the sixteen commands the
/// first row of the `FR-GLOB-021` table names.
#[derive(Debug, Clone, PartialEq, Eq, Args)]
pub(crate) struct Output {
    /// The representation of the result (`FR-OUT-001`).
    ///
    /// The default is fixed at `text` and is not conditioned on anything:
    /// `FR-OUT-002` forbids consulting `isatty()`, `TERM`, or any other
    /// terminal property, so a command and the same command in a pipeline
    /// produce identical bytes.
    #[arg(long = "format", value_name = "FORMAT", value_enum, default_value_t = Format::Text)]
    pub(crate) format: Format,

    /// `--pretty`, which `FR-OUT-009` admits only alongside `--format json`.
    #[command(flatten)]
    pub(crate) pretty: Pretty,
}

/// `--direct` and `--no-cache`, declared by exactly the ten commands of
/// `FR-CACHE-017`: the eight `schema` subcommands, `tpl render`, and
/// `tpl cache load`.
///
/// The two are orthogonal and compose, per `FR-CACHE-015`, so they are two
/// independent flags rather than one enumerated value: `--direct --no-cache` is
/// the pure read of `FR-CACHE-016`, which touches no file at all.
#[derive(Debug, Clone, PartialEq, Eq, Args)]
pub(crate) struct Caching {
    /// Read from the database, ignoring whatever is cached (`FR-CACHE-013`).
    #[arg(long = "direct", action = ArgAction::SetTrue)]
    pub(crate) direct: bool,

    /// Do not store the result of this invocation in the cache
    /// (`FR-CACHE-014`).
    #[arg(long = "no-cache", action = ArgAction::SetTrue)]
    pub(crate) no_cache: bool,
}

/// `--table`, `--view` and `--routine`, the flags that name one catalogue
/// object.
///
/// `FR-RND-003` declares them on `tpl render` and `FR-CACHE-024` gives
/// `tpl cache load` and `tpl cache clean` the same three spellings. Each takes
/// exactly one value and is given at most once, per `FR-RND-004`; a repetition
/// is `64` under `FR-CLI-014`, which is a parsing rule and not declared here.
///
/// The three are mutually exclusive in one invocation, per `FR-RND-005`, and
/// that refusal is the command's rather than the parser's, for the reason this
/// module's own documentation gives. Absence of all three is not an error: it
/// is the whole-catalogue form of `FR-RND-006` and `FR-CACHE-022`.
#[derive(Debug, Clone, PartialEq, Eq, Args)]
pub(crate) struct Object {
    /// The table to bind (`FR-RND-003`).
    #[arg(long = "table", value_name = "NAME")]
    pub(crate) table: Option<String>,

    /// The view to bind (`FR-RND-003`).
    #[arg(long = "view", value_name = "NAME")]
    pub(crate) view: Option<String>,

    /// The routine to bind (`FR-RND-003`).
    ///
    /// It accepts the qualified forms `procedure:<name>` and `function:<name>`
    /// as well as a bare name, per `FR-SCH-008`, because procedures and
    /// functions occupy distinct namespaces on the server. Resolving the name
    /// is the command's, and a bare name matching both is `64` under
    /// `FR-SCH-010`.
    #[arg(long = "routine", value_name = "NAME")]
    pub(crate) routine: Option<String>,
}
