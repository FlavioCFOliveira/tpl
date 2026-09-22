//! The typed help table of `FR-HELP-022`: one entry per node of the tree,
//! indexed by command path.
//!
//! Five of the seven sections of `FR-HELP-006` are introspectable from the
//! parser tree `cli` declares — `USAGE`, `ARGUMENTS` and `OPTIONS` come from
//! the nodes, the arguments and the flags tasks 86 and 87 built, and
//! `FR-HELP-021` derives the JSON command tree from that same tree. The other
//! two are not: no argument parser holds an example or an exit code. This
//! module is where they are written, once, for the text help and the JSON
//! document alike, and `FR-HELP-022` forbids recovering either by parsing
//! rendered help.
//!
//! `OD-05` places the table here, beside the tree whose vocabulary indexes it.
//!
//! # The shape, and why declaration order holds without a map
//!
//! [`ENTRIES`] is a `const` array, and an array is ordered by construction. It
//! is written in the order `FR-HELP-019` requires of `data.commands` — each
//! node followed by its own children before the next node at its level, in the
//! order the tree declares them — so a consumer that walks it emits that order
//! without sorting anything, and `FR-HELP-023` and `FR-OUT-013` are satisfied
//! by the container rather than by the consumer. [`entry`] finds a path by
//! scanning it. **No unordered map appears anywhere on this path**, and a
//! `HashMap` introduced between this table and either consumer would be a
//! defect: thirty-five entries are not a lookup problem, and the one property
//! a map would cost is the one the requirement names.
//!
//! # What an entry carries, and what it does not
//!
//! An entry carries the node's `DESCRIPTION`, its `EXAMPLES`, its `EXIT CODES`
//! and its `SEE ALSO` references, and nothing else. Four things are
//! deliberately absent:
//!
//! - **The layout.** Prose is stored unwrapped and examples are stored as
//!   lines rather than as a rendered block. The seven sections, their order,
//!   the omission rule of `FR-HELP-007` and the fixed 80 columns of
//!   `FR-HELP-009` belong to the renderer.
//! - **The one-line summary of a node.** `FR-HELP-008` lists a node's children
//!   inside its `ARGUMENTS`, one short line each; that line is the `about` the
//!   tree already carries, introspected from the node. [`Entry::description`]
//!   is the `DESCRIPTION` section — what the command is and what it does — and
//!   is the longer of the two.
//! - **Every property of a flag or an argument.** `FR-HELP-013` obliges help to
//!   state a type, a default, requiredness, the enumerated values and
//!   repeatability, and all five are introspected from the tree.
//! - **`70` (`EX_SOFTWARE`), anywhere but the root.** See below.
//!
//! # Which codes an entry lists
//!
//! `FR-HELP-011` admits only the codes a command can produce, and the sets
//! below are read from `FR-ERR-001`, the validation order of `FR-ERR-006`, and
//! the module that owns each command. Four rulings shape them, and each is
//! applied to every entry:
//!
//! | Ruling | Consequence |
//! |---|---|
//! | Step 1 of `FR-ERR-006` runs for every command without exception | every entry lists `64` |
//! | `FR-PROJ-025` exempts `init`, `help` and `version` from discovery | those three omit `78`, as that requirement's accepted cost states |
//! | `FR-HELP-025` makes a group node invoked bare print its help and exit `0`, unconditionally | the six group nodes omit `78` for the same reason: nothing is discovered to fail |
//! | `FR-ERR-003` reserves `73` to `tpl init` | no other entry lists it |
//! | `FR-ERR-030` makes `70` a defect in `tpl` rather than a condition of any command under it | it is stated once, on the root entry |
//!
//! **`70` is listed once, on the root entry, and on no node beneath it.**
//! `FR-ERR-030` gives it exactly two producing conditions — a panic, and a
//! violated internal invariant — and neither is a condition of a command: both
//! are defects in `tpl`, reachable from every invocation and correctable by no
//! caller. Two readings are therefore wrong in opposite directions. Writing it
//! into all thirty-five entries repeats one fact at a level where it says
//! nothing about the command it sits under, which `BR-HELP-002` refuses.
//! Leaving it out altogether names a code the binary can return in no help
//! text at all, where help is one of only three channels a calling agent has
//! for learning the surface — and `BR-HELP-002` refuses what is repeated at a
//! level where it has *already been said*, which presumes it was said
//! somewhere.
//!
//! `FR-GLOB-003` settles this shape already: the seven global flags are listed
//! **once**, in the `OPTIONS` of `tpl --help`, and repeated at no other node,
//! because a property of the tool is not a property of each command under it.
//! `70` is stated the same way — once, at the root, where `FR-HELP-011` admits
//! it because a panic is reachable from any invocation, the root's own
//! included. A test pins where it appears, so that moving it is a deliberate
//! act.
//!
//! # `SEE ALSO`
//!
//! `FR-HELP-014` makes help self-contained: a reference names another node of
//! this tree and never a website, a manual page or a README. Every reference
//! is therefore a command path, checked against the tree by a test rather than
//! written as free text.
//!
//! # Who reads the table
//!
//! The table has exactly two consumers, and they read the same entries from the
//! same array:
//!
//! - [`render`] composes the seven sections of `FR-HELP-006` from this table and
//!   from the parser tree. [`text`] is the whole of the route to it, and every
//!   form of `FR-HELP-001` that prints help arrives there.
//! - [`document`] composes the JSON command tree of `FR-HELP-016` from the same
//!   two sources, reading the table whole through [`entries`], in the order
//!   `FR-HELP-019` requires.
//!
//! `FR-HELP-022` is what makes that one array rather than two: `examples` and
//! `exit_codes` feed the text help and the JSON document alike, and neither is
//! ever recovered by parsing rendered help.
//!
//! # The path both consumers are reached by
//!
//! [`command`] is `tpl help` itself, and [`resolve`] is the one resolver under
//! it. A command path of any depth is a sequence of positional arguments, each
//! segment resolved against the children of the node the preceding segments
//! reached: an alias of `FR-CLI-011` resolves to its canonical node and no
//! segment is inferred from a prefix, per `FR-HELP-026`, `FR-HELP-027` and
//! `FR-CLI-004`. A segment that names no child is `64` with a suggestion over
//! **that node's children alone**, per `FR-HELP-028`.
//!
//! One resolver serves both representations, which is why `tpl help cfg db add`
//! and `tpl help cfg database add` print the same bytes and why
//! `tpl help cfg database --format json` reduces to the same subtree that
//! `tpl help cfg database` describes.

mod document;
mod render;

use std::io::Write;

use crate::diagnostics::suggest::{self, Population};
use crate::error::{self, Error};
use crate::output::{self, Form};

use super::local::{self, Format};

/// One node of the tree, and the four things `FR-HELP-022` and `FR-HELP-019`
/// require its help to carry beyond what the tree itself declares.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct Entry {
    /// The command path, as the segments below `tpl`. Empty for the root.
    pub(crate) path: &'static [&'static str],

    /// The `DESCRIPTION` section: what the command is and what it does.
    ///
    /// One unwrapped paragraph. The renderer wraps it.
    pub(crate) description: &'static str,

    /// The `EXAMPLES` section. Never empty, per `FR-HELP-012`.
    pub(crate) examples: &'static [Example],

    /// The `EXIT CODES` section, ascending, listing only what this command can
    /// produce, per `FR-HELP-011`.
    pub(crate) exit_codes: &'static [Outcome],

    /// The `SEE ALSO` section: the path of another node of this tree, per
    /// `FR-HELP-014`.
    pub(crate) see_also: &'static [&'static [&'static str]],
}

/// One worked invocation, with the line saying what it does.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct Example {
    /// What the example does, in one unwrapped line.
    pub(crate) caption: &'static str,

    /// The example itself, line by line.
    pub(crate) lines: &'static [Line],
}

/// One line of an example: shell text, a `tpl` invocation, or both.
///
/// The invocation is held as its argument vector rather than as a string, so
/// that the third property of `BR-HELP-003` is asserted over the very tokens
/// the renderer prints. A line that carries shell alone — the head of a loop,
/// its `done`, the second stage of a pipeline — carries an empty invocation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct Line {
    /// Shell written before the invocation on this line, or the empty string.
    pub(crate) prefix: &'static str,

    /// The invocation, token by token, beginning with `tpl`. Empty where the
    /// line carries no invocation.
    pub(crate) invocation: &'static [&'static str],

    /// Shell written after the invocation — a redirection, a pipe, `; do` — or
    /// the empty string.
    pub(crate) suffix: &'static str,
}

/// One row of an `EXIT CODES` section: a code, and what it means for the
/// command whose entry lists it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct Outcome {
    /// The code.
    pub(crate) code: Code,

    /// What produces it in this command, in one unwrapped line.
    pub(crate) meaning: &'static str,
}

/// A code of `FR-ERR-001` that a command's help can list.
///
/// The ten codes of that table. `70` (`EX_SOFTWARE`) is carried by the root
/// entry alone, for the reason this module's own documentation gives. The
/// names are `sysexits.h`'s, as `FR-ERR-001` gives them and as the `exit` line
/// of `FR-ERR-008` writes them.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) enum Code {
    /// `0` — the command succeeded.
    Ok,

    /// `64` — the invocation is not valid.
    Usage,

    /// `65` — a template or a context document is not valid.
    DataError,

    /// `66` — a named object does not exist.
    NoInput,

    /// `69` — the server could not be reached.
    Unavailable,

    /// `70` — a defect in `tpl`. Carried by the root entry alone.
    Software,

    /// `73` — `tpl init` could not create the project.
    CantCreate,

    /// `74` — an I/O failure reading `.tpl` or writing to stdout.
    IoError,

    /// `77` — the server refused the credentials, or the privileges are short.
    NoPermission,

    /// `78` — the project or its configuration cannot be used as it stands.
    Configuration,
}

impl Code {
    /// The number the process exits with.
    pub(crate) const fn code(self) -> u8 {
        match self {
            Self::Ok => 0,
            Self::Usage => 64,
            Self::DataError => 65,
            Self::NoInput => 66,
            Self::Unavailable => 69,
            Self::Software => 70,
            Self::CantCreate => 73,
            Self::IoError => 74,
            Self::NoPermission => 77,
            Self::Configuration => 78,
        }
    }

    /// The `sysexits.h` name of the code, as `FR-ERR-001` gives it.
    pub(crate) const fn name(self) -> &'static str {
        match self {
            Self::Ok => "EX_OK",
            Self::Usage => "EX_USAGE",
            Self::DataError => "EX_DATAERR",
            Self::NoInput => "EX_NOINPUT",
            Self::Unavailable => "EX_UNAVAILABLE",
            Self::Software => "EX_SOFTWARE",
            Self::CantCreate => "EX_CANTCREAT",
            Self::IoError => "EX_IOERR",
            Self::NoPermission => "EX_NOPERM",
            Self::Configuration => "EX_CONFIG",
        }
    }
}

/// The help text of the node `path` names, or [`None`] where it names no node.
///
/// `path` is the segments below `tpl`, and is empty for the root. The text is
/// the seven sections of `FR-HELP-006`, laid out at the fixed width of
/// `FR-HELP-009`, and it ends with a single newline.
///
/// The tree is built here because this is the whole route to the renderer, and
/// the renderer reads the tree as it is declared: below the root, every node
/// carries exactly its own arguments, which is what `FR-GLOB-003` requires of
/// the `OPTIONS` section.
pub(crate) fn text(path: &[&str]) -> Option<String> {
    render::text(&super::tree(), path)
}

/// The whole table, in the order `FR-HELP-019` requires.
pub(crate) const fn entries() -> &'static [Entry] {
    &ENTRIES
}

/// What help states about one flag or one positional argument beyond the facts
/// the declaration itself carries.
///
/// `FR-HELP-030` obliges help to state, in **one sentence**, what supplying an
/// argument does — the object it acts on and the effect it has — beside the six
/// facts of `FR-HELP-013`; and the sixth of those six, mutual exclusion, is a
/// relation between two arguments and not a property of either. Neither can be
/// introspected, so both live here, in the typed table `FR-HELP-022` makes the
/// one source for the text help and the JSON document alike.
///
/// **The text cannot live on the declarations.** `clap`'s derive lifts a field's
/// doc comment verbatim, and those name requirement identifiers in backticks;
/// `FR-HELP-014` makes help self-contained and bars it from referring to any
/// document outside the help system, so a renderer that lifted one would publish
/// citations a caller cannot resolve. `FR-HELP-030` says so in its own words and
/// rejects the alternative.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct Documented {
    /// The argument this row is about: a flag by its long form with both
    /// dashes, or a positional argument by the value name help spells it with.
    pub(crate) name: &'static str,

    /// One sentence saying what supplying it does (`FR-HELP-030`).
    ///
    /// It names a thing the specification fixes rather than restating the
    /// argument's own spelling, and it carries no requirement identifier: help
    /// is self-contained, per `FR-HELP-014`, and a test holds every sentence of
    /// this table to both.
    pub(crate) purpose: &'static str,

    /// The arguments this one may not be given with, at this node
    /// (`FR-HELP-013`).
    ///
    /// Spelled as the reader meets them — a long form with its dashes — and
    /// empty where the argument excludes nothing. The tree carries no
    /// `conflicts_with` to introspect: every exclusion this corpus obliges is
    /// refused away from the parser, so that the caller reads it in the four
    /// labelled lines of `FR-ERR-008` rather than in the parser's words, and
    /// this is the one place the fact is stated.
    pub(crate) excludes: &'static [&'static str],
}

// The rows of the typed table, one per argument the tree declares. A flag that
// more than one node declares is written **once**, here, and named by each of
// those nodes below: `cli::local` declares such a flag once for exactly that
// reason, and a sentence repeated per node would drift between two nodes one
// row of `FR-GLOB-021` names together.

/// `--pattern`, as every node that declares it states it.
const PATTERN: Documented = Documented {
    name: "--pattern",
    purpose: "Keeps only the objects whose name matches the LIKE pattern given, \
              in which % stands for any run of characters and _ for exactly \
              one.",
    excludes: &[],
};
/// `--format`, as every node that declares it states it.
const FORMAT: Documented = Documented {
    name: "--format",
    purpose: "Chooses the representation of the result: aligned columns laid \
              out for a person, or the JSON document anything parsing the \
              output must read.",
    excludes: &[],
};
/// `--pretty`, as every node that declares it states it.
const PRETTY: Documented = Documented {
    name: "--pretty",
    purpose: "Indents the JSON document by two spaces with one key per line, \
              instead of writing it on a single line.",
    excludes: &[],
};
/// `--direct`, as every node that declares it states it.
const DIRECT: Documented = Documented {
    name: "--direct",
    purpose: "Reads the server for this invocation and ignores whatever the \
              project's cache already holds.",
    excludes: &[],
};
/// `--no-cache`, as every node that declares it states it.
const NO_CACHE: Documented = Documented {
    name: "--no-cache",
    purpose: "Leaves the project's cache as it was, storing nothing this \
              invocation read.",
    excludes: &[],
};
/// `--set`, as every node that declares it states it.
const SET: Documented = Documented {
    name: "--set",
    purpose: "Defines one extra variable for the template, written as \
              key=value, and may be given once per key.",
    excludes: &[],
};
/// `--context`, as every node that declares it states it.
const CONTEXT: Documented = Documented {
    name: "--context",
    purpose: "Takes the render context from the JSON document named, or from \
              standard input when the name is -, so that no server is \
              contacted.",
    excludes: &["--database"],
};
/// `--dsn`, as every node that declares it states it.
const DSN: Documented = Documented {
    name: "--dsn",
    purpose: "Supplies the whole connection as one URL, in place of the flags \
              that set its parts one at a time.",
    excludes: &["--host", "--port", "--user", "--schema"],
};
/// `--host`, as every node that declares it states it.
const HOST: Documented = Documented {
    name: "--host",
    purpose: "Sets the host name or address the entry connects to.",
    excludes: &["--dsn"],
};
/// `--port`, as every node that declares it states it.
const PORT: Documented = Documented {
    name: "--port",
    purpose: "Sets the TCP port the entry connects to.",
    excludes: &["--dsn"],
};
/// `--user`, as every node that declares it states it.
const USER: Documented = Documented {
    name: "--user",
    purpose: "Sets the user the entry authenticates to the server as.",
    excludes: &["--dsn"],
};
/// `--schema`, as every node that declares it states it.
const SCHEMA: Documented = Documented {
    name: "--schema",
    purpose: "Sets the server-side database the entry reads the catalogue of.",
    excludes: &["--dsn"],
};
/// `--tls`, as every node that declares it states it.
const TLS: Documented = Documented {
    name: "--tls",
    purpose: "Sets how much the entry's connection encrypts and validates, from \
              no transport security to a validated chain and a matching host \
              name.",
    excludes: &[],
};
/// `--password-command`, as every node that declares it states it.
const PASSWORD_COMMAND: Documented = Documented {
    name: "--password-command",
    purpose: "Sets the command whose standard output supplies the entry's \
              password, so that no password is written into the file.",
    excludes: &[],
};
/// `--ca-file`, as every node that declares it states it.
const CA_FILE: Documented = Documented {
    name: "--ca-file",
    purpose: "Names one file of certificates the entry validates the server's \
              chain against.",
    excludes: &[],
};
/// `--ca-path`, as every node that declares it states it.
const CA_PATH: Documented = Documented {
    name: "--ca-path",
    purpose: "Names a directory of certificate files the entry validates the \
              server's chain against.",
    excludes: &[],
};

// The three object flags of `FR-RND-003` and `FR-CACHE-024`. `FR-RND-005`
// refuses more than one kind in one invocation, which is the exclusion each of
// the three states about the other two.

/// `--table`, the object flag.
const OBJECT_TABLE: Documented = Documented {
    name: "--table",
    purpose: "Narrows the invocation to the one table named, in place of the \
              whole catalogue.",
    excludes: &["--view", "--routine"],
};
/// `--view`, the object flag.
const OBJECT_VIEW: Documented = Documented {
    name: "--view",
    purpose: "Narrows the invocation to the one view named, in place of the \
              whole catalogue.",
    excludes: &["--table", "--routine"],
};
/// `--routine`, the object flag.
const OBJECT_ROUTINE: Documented = Documented {
    name: "--routine",
    purpose: "Narrows the invocation to the one routine named, in place of the \
              whole catalogue, taking a bare name or one qualified as \
              procedure:<name> or function:<name>.",
    excludes: &["--table", "--view"],
};

// The positional arguments, one row per node that declares one: a value name
// says as little about its purpose as a flag name does, and `FR-HELP-030`
// makes the two one population for that reason.

/// `NAME`, as the node that declares it states it.
const SCHEMA_TABLE_NAME: Documented = Documented {
    name: "NAME",
    purpose: "Names the table to describe.",
    excludes: &[],
};
/// `NAME`, as the node that declares it states it.
const SCHEMA_VIEW_NAME: Documented = Documented {
    name: "NAME",
    purpose: "Names the view to describe.",
    excludes: &[],
};
/// `NAME`, as the node that declares it states it.
const SCHEMA_ROUTINE_NAME: Documented = Documented {
    name: "NAME",
    purpose: "Names the routine to describe, bare or qualified as \
              procedure:<name> or function:<name>.",
    excludes: &[],
};
/// `NAME`, as the node that declares it states it.
const TEMPLATE_SHOW_NAME: Documented = Documented {
    name: "NAME",
    purpose: "Names the template whose source to write.",
    excludes: &[],
};
/// `NAME`, as the node that declares it states it.
const TEMPLATE_CHECK_NAME: Documented = Documented {
    name: "NAME",
    purpose: "Names a template to compile; every template the project holds is \
              compiled when none is named.",
    excludes: &[],
};
/// `NAME`, as the node that declares it states it.
const TEMPLATE_PATH_NAME: Documented = Documented {
    name: "NAME",
    purpose: "Names the template whose location on disk to write.",
    excludes: &[],
};
/// `TEMPLATE`, as the node that declares it states it.
const RENDER_TEMPLATE: Documented = Documented {
    name: "TEMPLATE",
    purpose: "Names the template to render.",
    excludes: &[],
};
/// `KEY`, as the node that declares it states it.
const CFG_GET_KEY: Documented = Documented {
    name: "KEY",
    purpose: "Names the configuration key whose value to write.",
    excludes: &[],
};
/// `KEY`, as the node that declares it states it.
const CFG_SET_KEY: Documented = Documented {
    name: "KEY",
    purpose: "Names the configuration key to write into.",
    excludes: &[],
};
/// `VALUE`, as the node that declares it states it.
const CFG_SET_VALUE: Documented = Documented {
    name: "VALUE",
    purpose: "Supplies the value written into that key.",
    excludes: &[],
};
/// `KEY`, as the node that declares it states it.
const CFG_UNSET_KEY: Documented = Documented {
    name: "KEY",
    purpose: "Names the configuration key to remove from the file.",
    excludes: &[],
};
/// `NAME`, as the node that declares it states it.
const ENTRY_ADD_NAME: Documented = Documented {
    name: "NAME",
    purpose: "Names the database entry to create.",
    excludes: &[],
};
/// `NAME`, as the node that declares it states it.
const ENTRY_SHOW_NAME: Documented = Documented {
    name: "NAME",
    purpose: "Names the database entry whose settings to write.",
    excludes: &[],
};
/// `NAME`, as the node that declares it states it.
const ENTRY_UPDATE_NAME: Documented = Documented {
    name: "NAME",
    purpose: "Names the database entry to change.",
    excludes: &[],
};
/// `NAME`, as the node that declares it states it.
const ENTRY_REMOVE_NAME: Documented = Documented {
    name: "NAME",
    purpose: "Names the database entry to delete.",
    excludes: &[],
};
/// `NAME`, as the node that declares it states it.
const ENTRY_TEST_NAME: Documented = Documented {
    name: "NAME",
    purpose: "Names the database entry whose connection to try.",
    excludes: &[],
};
/// `PATH`, as the node that declares it states it.
const INIT_PATH: Documented = Documented {
    name: "PATH",
    purpose: "Names the directory to create the project in; the working \
              directory is used when none is named.",
    excludes: &[],
};
/// `COMMAND_PATH`, as the node that declares it states it.
const HELP_COMMAND_PATH: Documented = Documented {
    name: "COMMAND_PATH",
    purpose: "Names the node whose help to write, one segment of its path per \
              token, at any depth.",
    excludes: &[],
};

/// The seven global flags of `FR-GLOB-001`, indexed by flag name.
///
/// `FR-GLOB-003` lists them **once**, in the `OPTIONS` of `tpl --help`, so they
/// have no command path of their own and `FR-HELP-030` indexes them by name
/// instead. The pair `FR-CLI-015` refuses is stated here, on each of its two
/// members, which is the same place every local pair is stated.
const GLOBAL_ARGUMENTS: [Documented; 7] = [
    Documented {
        name: "--database",
        purpose: "Selects the [database.<name>] entry of .tpl/.cfg this \
                  invocation reads through, in place of the one core.database \
                  names.",
        excludes: &[],
    },
    Documented {
        name: "--tpl-dir",
        purpose: "Names the .tpl folder to work in and suppresses the upward \
                  search for one.",
        excludes: &[],
    },
    Documented {
        name: "--timeout",
        purpose: "Bounds the whole invocation in seconds, measured from process \
                  start, beside the per-phase deadlines the project sets.",
        excludes: &[],
    },
    Documented {
        name: "--verbose",
        purpose: "Raises the diagnostic detail written to standard error by one \
                  level per occurrence, up to three.",
        excludes: &["--quiet"],
    },
    Documented {
        name: "--quiet",
        purpose: "Lowers the diagnostic detail written to standard error to \
                  errors alone.",
        excludes: &["--verbose"],
    },
    Documented {
        name: "--help",
        purpose: "Writes the help of the node it is written at, instead of doing \
                  that node's work.",
        excludes: &[],
    },
    Documented {
        name: "--version",
        purpose: "Writes the program name and its version, instead of doing any \
                  command's work.",
        excludes: &[],
    },
];

/// Every local flag and positional argument, indexed by command path.
///
/// One row per node that declares one, in the order [`ENTRIES`] declares the
/// nodes. The order inside a row is immaterial: the renderer and the JSON
/// document both walk the **tree**, which fixes declaration order per
/// `FR-HELP-023`, and read this table by name.
///
/// A node that declares no argument of its own has no row, which is what a node
/// whose `OPTIONS` and `ARGUMENTS` sections are both omitted means.
const ARGUMENTS: [(&[&str], &[Documented]); 28] = [
    (&["schema", "info"], &[FORMAT, PRETTY, DIRECT, NO_CACHE]),
    (
        &["schema", "tables"],
        &[PATTERN, FORMAT, PRETTY, DIRECT, NO_CACHE],
    ),
    (
        &["schema", "table"],
        &[SCHEMA_TABLE_NAME, FORMAT, PRETTY, DIRECT, NO_CACHE],
    ),
    (
        &["schema", "views"],
        &[PATTERN, FORMAT, PRETTY, DIRECT, NO_CACHE],
    ),
    (
        &["schema", "view"],
        &[SCHEMA_VIEW_NAME, FORMAT, PRETTY, DIRECT, NO_CACHE],
    ),
    (
        &["schema", "routines"],
        &[PATTERN, FORMAT, PRETTY, DIRECT, NO_CACHE],
    ),
    (
        &["schema", "routine"],
        &[SCHEMA_ROUTINE_NAME, FORMAT, PRETTY, DIRECT, NO_CACHE],
    ),
    (&["schema", "dump"], &[PRETTY, DIRECT, NO_CACHE]),
    (&["template", "list"], &[FORMAT, PRETTY]),
    (&["template", "show"], &[TEMPLATE_SHOW_NAME]),
    (&["template", "check"], &[TEMPLATE_CHECK_NAME]),
    (&["template", "path"], &[TEMPLATE_PATH_NAME, FORMAT, PRETTY]),
    (
        &["render"],
        &[
            RENDER_TEMPLATE,
            OBJECT_TABLE,
            OBJECT_VIEW,
            OBJECT_ROUTINE,
            SET,
            CONTEXT,
            DIRECT,
            NO_CACHE,
        ],
    ),
    (
        &["cache", "load"],
        &[OBJECT_TABLE, OBJECT_VIEW, OBJECT_ROUTINE, DIRECT, NO_CACHE],
    ),
    (
        &["cache", "clean"],
        &[OBJECT_TABLE, OBJECT_VIEW, OBJECT_ROUTINE],
    ),
    (&["cache", "status"], &[FORMAT, PRETTY]),
    (&["cfg", "get"], &[CFG_GET_KEY, FORMAT, PRETTY]),
    (&["cfg", "set"], &[CFG_SET_KEY, CFG_SET_VALUE]),
    (&["cfg", "unset"], &[CFG_UNSET_KEY]),
    (&["cfg", "list"], &[FORMAT, PRETTY]),
    (
        &["cfg", "database", "add"],
        &[
            ENTRY_ADD_NAME,
            DSN,
            HOST,
            PORT,
            USER,
            SCHEMA,
            TLS,
            PASSWORD_COMMAND,
            CA_FILE,
            CA_PATH,
        ],
    ),
    (&["cfg", "database", "list"], &[FORMAT, PRETTY]),
    (
        &["cfg", "database", "show"],
        &[ENTRY_SHOW_NAME, FORMAT, PRETTY],
    ),
    (
        &["cfg", "database", "update"],
        &[
            ENTRY_UPDATE_NAME,
            DSN,
            HOST,
            PORT,
            USER,
            SCHEMA,
            TLS,
            PASSWORD_COMMAND,
            CA_FILE,
            CA_PATH,
        ],
    ),
    (&["cfg", "database", "remove"], &[ENTRY_REMOVE_NAME]),
    (
        &["cfg", "database", "test"],
        &[ENTRY_TEST_NAME, FORMAT, PRETTY],
    ),
    (&["init"], &[INIT_PATH]),
    (&["help"], &[HELP_COMMAND_PATH, FORMAT, PRETTY]),
];

/// What help states about the argument `name` at the node `path` names.
///
/// A global flag is looked up by name and has no command path, per
/// `FR-GLOB-003`; every other argument is looked up by the path of the node
/// that declares it. The two spaces do not overlap — no local flag of this tree
/// spells a global one — and the global set is tried first so that the root's
/// own `OPTIONS`, which is the one place the seven appear, resolves them.
///
/// A linear scan, for the reason [`entry`] gives: the table is small and
/// ordered, and no unordered map appears on this path.
pub(crate) fn documented(path: &[&str], name: &str) -> Option<&'static Documented> {
    if let Some(global) = GLOBAL_ARGUMENTS.iter().find(|stated| stated.name == name) {
        return Some(global);
    }

    ARGUMENTS
        .iter()
        .find(|(declared, _)| *declared == path)
        .and_then(|(_, stated)| stated.iter().find(|stated| stated.name == name))
}

/// Runs `tpl help`, in whichever of its two representations was asked for.
///
/// `path` is the sequence of positional segments the caller wrote, of any
/// depth, per `FR-HELP-026`. It is resolved once, by [`resolve`], and the two
/// representations part only after that: `text` is the renderer of
/// `FR-HELP-006` that every other help form reaches, and `json` is the command
/// tree of `FR-HELP-016` reduced to the subtree that path roots, per
/// `FR-HELP-029`. **One resolver serves both**, which is why a subtree and a
/// help text are selected by the same rules and refused by the same message.
///
/// Nothing here discovers a project, reads a configuration file or opens a
/// connection, which is what `FR-PROJ-025` requires of every form of this
/// command: the only inputs are the parser tree and the typed table.
///
/// # Errors
///
/// Returns [`Error::UnknownCommandPathSegment`] for a segment that names no
/// child of the node the preceding segments reached (`FR-HELP-028`),
/// [`Error::StdoutUnwritable`] or [`Error::StdoutClosedMidDocument`] where the
/// stream refused the write, and [`Error::InternalInvariant`] where the tree
/// and the table disagree about which nodes exist.
pub(crate) fn command<W: Write>(
    out: &mut W,
    path: &[String],
    output: &local::Output,
) -> Result<(), Error> {
    let tree = super::tree();
    let (node, canonical) = resolve(&tree, path)?;

    // The vector carries at most one format once `FR-CLI-014` has been applied,
    // and the declaration's own default occupies the place when the flag is
    // absent, so the first entry is the format in force.
    match output.format.first() {
        Some(Format::Json) => document::emit(out, &tree, node, &canonical, form(output)),
        Some(Format::Text) | None => {
            let Some(text) = render::text(&tree, &canonical) else {
                return error::ensure_invariant(
                    false,
                    "every node of the tree carries an entry of the help table",
                );
            };

            output::emit_help(out, &text)
        }
    }
}

/// Which of the two forms of `FR-OUT-007` and `FR-OUT-008` a JSON document is
/// written in.
const fn form(output: &local::Output) -> Form {
    if output.pretty.pretty {
        Form::Indented
    } else {
        Form::Compact
    }
}

/// The canonical path a sequence of segments names, or the refusal of
/// `FR-HELP-028`.
///
/// Each segment is resolved against the children of the node the preceding
/// segments reached, by the rules of the tree: an alias of `FR-CLI-011`
/// resolves to the node it names and no segment is inferred from a prefix, per
/// `FR-HELP-027` and `FR-CLI-004`. `tpl help cfg db add` therefore resolves to
/// the same node as `tpl help cfg database add`, and both consumers of this
/// function are handed the canonical spelling.
///
/// An empty sequence resolves to the root, which is the whole tree for
/// [`document`] and the top-level help for the renderer.
///
/// The node itself is returned beside the path because [`document`] walks the
/// subtree from it, and re-finding it from the path would be a second walk over
/// the same segments.
///
/// # Errors
///
/// Returns [`Error::UnknownCommandPathSegment`] naming the segment that failed,
/// the node it was looked for under, and the nearest matches **among that
/// node's children alone** — which is what makes the suggestion short and right,
/// per `FR-HELP-028`.
fn resolve<'a>(
    tree: &'a clap::Command,
    path: &[String],
) -> Result<(&'a clap::Command, Vec<&'a str>), Error> {
    let mut node = tree;
    let mut canonical: Vec<&str> = Vec::with_capacity(path.len());

    for segment in path {
        let Some(child) = node.find_subcommand(segment.as_str()) else {
            return Err(Error::UnknownCommandPathSegment {
                segment: segment.clone(),
                node: canonical.join(" "),
                nearest: nearest_child(node, segment),
            });
        };

        canonical.push(child.get_name());
        node = child;
    }

    Ok((node, canonical))
}

/// The nearest matches for `segment` among the children of `node`.
///
/// The population is that node's children and nothing else, per `FR-HELP-028`,
/// and it carries each child's canonical name **and** the aliases of
/// `FR-CLI-011`: `BR-CLI-001` routes a mistyped alias through the same
/// nearest-match rule, so `tpl help cfg d` has `db` to propose as well as
/// `database`.
fn nearest_child(node: &clap::Command, segment: &str) -> Vec<String> {
    let candidates: Vec<&str> = node
        .get_subcommands()
        .flat_map(|child| std::iter::once(child.get_name()).chain(child.get_all_aliases()))
        .collect();

    suggest::suggestions(segment, candidates, Population::Commands)
        .names()
        .map(ToOwned::to_owned)
        .collect()
}

/// The entry for one command path, or [`None`] where the path names no node.
///
/// A linear scan over [`ENTRIES`], which is what keeps declaration order the
/// container's property rather than the caller's, per `FR-HELP-023`.
pub(crate) fn entry(path: &[&str]) -> Option<&'static Entry> {
    ENTRIES.iter().find(|entry| entry.path == path)
}

/// A line carrying one `tpl` invocation and no shell.
const fn run(invocation: &'static [&'static str]) -> Line {
    Line {
        prefix: "",
        invocation,
        suffix: "",
    }
}

/// A line carrying one `tpl` invocation inside shell text.
const fn run_in(
    prefix: &'static str,
    invocation: &'static [&'static str],
    suffix: &'static str,
) -> Line {
    Line {
        prefix,
        invocation,
        suffix,
    }
}

/// A line carrying shell text and no invocation.
const fn shell(text: &'static str) -> Line {
    Line {
        prefix: text,
        invocation: &[],
        suffix: "",
    }
}

/// One row of an `EXIT CODES` section.
const fn outcome(code: Code, meaning: &'static str) -> Outcome {
    Outcome { code, meaning }
}

// The shared outcome sets. Two commands share one only where every line of it
// is true of both; where a command adds a condition of its own — an ambiguous
// routine name, a flag it alone refuses — it carries a set of its own rather
// than a shared set qualified in prose.

/// The outcomes of a group node invoked with no child (`FR-HELP-025`).
///
/// The five group nodes below the root. The root carries [`ROOT`] instead, for
/// the one code that is `tpl`'s rather than any command's.
const GROUP: &[Outcome] = &[
    outcome(Code::Ok, "The help of this node was written to stdout."),
    outcome(
        Code::Usage,
        "The invocation is not valid: an unknown command or flag, or a repeated flag value.",
    ),
    outcome(Code::IoError, "The help could not be written to stdout."),
];

/// The outcomes of a bare `tpl`, which are [`GROUP`]'s and one more.
///
/// `70` is stated here and nowhere else, on the precedent `FR-GLOB-003` sets
/// for a fact that belongs to the tool rather than to each command under it.
const ROOT: &[Outcome] = &[
    outcome(Code::Ok, "The help of this node was written to stdout."),
    outcome(
        Code::Usage,
        "The invocation is not valid: an unknown command or flag, or a repeated flag value.",
    ),
    outcome(
        Code::Software,
        "A defect in tpl. It is reachable from any invocation, is not caused by the command line, \
         and cannot be corrected by the caller: report it. It is listed here, once, and under no \
         command of the tree.",
    ),
    outcome(Code::IoError, "The help could not be written to stdout."),
];

/// The outcomes of a `schema` subcommand that names no object.
const SCHEMA_LISTING: &[Outcome] = &[
    outcome(Code::Ok, "The read succeeded."),
    outcome(
        Code::Usage,
        "The invocation is not valid: an unknown or repeated flag, or --pretty without \
         --format json.",
    ),
    outcome(
        Code::NoInput,
        "The entry named by -d/--database is absent from .tpl/.cfg.",
    ),
    outcome(
        Code::Unavailable,
        "The server could not be reached, or a network phase exceeded its deadline.",
    ),
    outcome(
        Code::IoError,
        "Reading .tpl failed, or the result could not be written to stdout.",
    ),
    outcome(
        Code::NoPermission,
        "The server refused the credentials, or this reader cannot read the catalogue.",
    ),
    outcome(
        Code::Configuration,
        "No project was found, .tpl/.cfg cannot be used, no database entry is selected, or the \
         server is not a supported MariaDB series.",
    ),
];

/// The outcomes of `schema table` and `schema view`, which name one object.
const SCHEMA_OBJECT: &[Outcome] = &[
    outcome(Code::Ok, "The read succeeded."),
    outcome(
        Code::Usage,
        "The invocation is not valid: an unknown or repeated flag, a missing NAME, or --pretty \
         without --format json.",
    ),
    outcome(
        Code::NoInput,
        "NAME does not exist in the selected database, or the entry named by -d/--database is \
         absent from .tpl/.cfg.",
    ),
    outcome(
        Code::Unavailable,
        "The server could not be reached, or a network phase exceeded its deadline.",
    ),
    outcome(
        Code::IoError,
        "Reading .tpl failed, or the result could not be written to stdout.",
    ),
    outcome(
        Code::NoPermission,
        "The server refused the credentials, or this reader cannot read the catalogue.",
    ),
    outcome(
        Code::Configuration,
        "No project was found, .tpl/.cfg cannot be used, no database entry is selected, or the \
         server is not a supported MariaDB series.",
    ),
];

/// The outcomes of `schema routine`, which adds the ambiguity of `FR-SCH-010`.
const SCHEMA_ROUTINE: &[Outcome] = &[
    outcome(Code::Ok, "The read succeeded."),
    outcome(
        Code::Usage,
        "The invocation is not valid, or NAME is a bare name matching both a procedure and a \
         function.",
    ),
    outcome(
        Code::NoInput,
        "NAME names no routine of the selected database, or the entry named by -d/--database is \
         absent from .tpl/.cfg.",
    ),
    outcome(
        Code::Unavailable,
        "The server could not be reached, or a network phase exceeded its deadline.",
    ),
    outcome(
        Code::IoError,
        "Reading .tpl failed, or the result could not be written to stdout.",
    ),
    outcome(
        Code::NoPermission,
        "The server refused the credentials, or this reader cannot read the catalogue.",
    ),
    outcome(
        Code::Configuration,
        "No project was found, .tpl/.cfg cannot be used, no database entry is selected, or the \
         server is not a supported MariaDB series.",
    ),
];

/// The outcomes of `schema dump`, which declares neither --format nor --pattern.
const SCHEMA_DUMP: &[Outcome] = &[
    outcome(Code::Ok, "The document was written to stdout."),
    outcome(
        Code::Usage,
        "The invocation is not valid: --format and --pattern are refused here, as is any other \
         undeclared flag.",
    ),
    outcome(
        Code::NoInput,
        "The entry named by -d/--database is absent from .tpl/.cfg.",
    ),
    outcome(
        Code::Unavailable,
        "The server could not be reached, or a network phase exceeded its deadline.",
    ),
    outcome(
        Code::IoError,
        "Reading .tpl failed, or the document could not be written to stdout.",
    ),
    outcome(
        Code::NoPermission,
        "The server refused the credentials, or this reader cannot read the catalogue.",
    ),
    outcome(
        Code::Configuration,
        "No project was found, .tpl/.cfg cannot be used, no database entry is selected, or the \
         server is not a supported MariaDB series.",
    ),
];

/// The outcomes of `tpl render`.
const RENDER: &[Outcome] = &[
    outcome(
        Code::Ok,
        "The render succeeded and its result was written to stdout.",
    ),
    outcome(
        Code::Usage,
        "The invocation is not valid: more than one object flag, a --set without =, an invalid or \
         repeated --set key, or --context together with -d/--database.",
    ),
    outcome(
        Code::DataError,
        "The template has a syntax error, the render failed, the --context document is malformed, \
         or the render deadline expired.",
    ),
    outcome(
        Code::NoInput,
        "TEMPLATE does not exist, the named object is absent from the context source, or the \
         entry named by -d/--database is absent from .tpl/.cfg.",
    ),
    outcome(
        Code::Unavailable,
        "The server could not be reached, or a network phase exceeded its deadline.",
    ),
    outcome(
        Code::IoError,
        "Reading .tpl failed, or the result could not be written to stdout.",
    ),
    outcome(
        Code::NoPermission,
        "The server refused the credentials, or this reader cannot read the catalogue.",
    ),
    outcome(
        Code::Configuration,
        "No project was found, .tpl/.cfg cannot be used, no database entry is selected, or the \
         server is not a supported MariaDB series.",
    ),
];

/// The outcomes of `template list`, which names no template.
const TEMPLATE_LISTING: &[Outcome] = &[
    outcome(
        Code::Ok,
        "The listing was written to stdout. A project with no template lists nothing and still \
         succeeds.",
    ),
    outcome(
        Code::Usage,
        "The invocation is not valid: an unknown or repeated flag, or --pretty without \
         --format json.",
    ),
    outcome(
        Code::IoError,
        "Reading .tpl failed, or the listing could not be written to stdout.",
    ),
    outcome(
        Code::Configuration,
        "No project was found, or .tpl/.cfg cannot be used.",
    ),
];

/// The outcomes of `template show` and `template path`, which resolve a name.
const TEMPLATE_NAMED: &[Outcome] = &[
    outcome(Code::Ok, "The result was written to stdout."),
    outcome(
        Code::Usage,
        "The invocation is not valid: an unknown or repeated flag, or a missing NAME.",
    ),
    outcome(
        Code::DataError,
        "NAME resolves to a path outside .tpl/templates/.",
    ),
    outcome(
        Code::NoInput,
        "NAME names no template of the project; the nearest matches are suggested.",
    ),
    outcome(
        Code::IoError,
        "Reading .tpl failed, or the result could not be written to stdout.",
    ),
    outcome(
        Code::Configuration,
        "No project was found, or .tpl/.cfg cannot be used.",
    ),
];

/// The outcomes of `template check`, which reports through its exit code.
const TEMPLATE_CHECK: &[Outcome] = &[
    outcome(
        Code::Ok,
        "Every template checked compiles. A project with no template checks nothing and still \
         succeeds.",
    ),
    outcome(
        Code::Usage,
        "The invocation is not valid: an unknown or repeated flag.",
    ),
    outcome(
        Code::DataError,
        "A checked template has a syntax error, named with its line and column, or resolves to a \
         path outside .tpl/templates/.",
    ),
    outcome(
        Code::NoInput,
        "A NAME given names no template of the project; the nearest matches are suggested.",
    ),
    outcome(Code::IoError, "Reading .tpl failed."),
    outcome(
        Code::Configuration,
        "No project was found, or .tpl/.cfg cannot be used.",
    ),
];

/// The outcomes of `cache load`, the one cache subcommand that reads a server.
const CACHE_LOAD: &[Outcome] = &[
    outcome(Code::Ok, "The catalogue was read and stored."),
    outcome(
        Code::Usage,
        "The invocation is not valid: --no-cache, more than one object flag, or a bare --routine \
         name matching both a procedure and a function.",
    ),
    outcome(
        Code::NoInput,
        "The named object does not exist in the selected database, or the entry named by \
         -d/--database is absent from .tpl/.cfg.",
    ),
    outcome(
        Code::Unavailable,
        "The server could not be reached, or a network phase exceeded its deadline. Whatever was \
         already stored is left unchanged.",
    ),
    outcome(Code::IoError, "Reading .tpl failed."),
    outcome(
        Code::NoPermission,
        "The server refused the credentials, or this reader cannot read the catalogue.",
    ),
    outcome(
        Code::Configuration,
        "No project was found, .tpl/.cfg cannot be used, no database entry is selected, or the \
         server is not a supported MariaDB series.",
    ),
];

/// The outcomes of `cache clean`, which reads no server.
const CACHE_CLEAN: &[Outcome] = &[
    outcome(Code::Ok, "The cached data named was removed."),
    outcome(
        Code::Usage,
        "The invocation is not valid: --direct and --no-cache are refused here, as is more than \
         one object flag.",
    ),
    outcome(
        Code::NoInput,
        "The entry named by -d/--database is absent from .tpl/.cfg.",
    ),
    outcome(Code::IoError, "Reading .tpl failed."),
    outcome(
        Code::Configuration,
        "No project was found, .tpl/.cfg cannot be used, or no database entry is selected.",
    ),
];

/// The outcomes of `cache status`, which reads no server.
const CACHE_STATUS: &[Outcome] = &[
    outcome(
        Code::Ok,
        "The report was written to stdout. An empty cache is a state, not a failure.",
    ),
    outcome(
        Code::Usage,
        "The invocation is not valid: --direct and --no-cache are refused here, as is --pretty \
         without --format json.",
    ),
    outcome(
        Code::NoInput,
        "The entry named by -d/--database is absent from .tpl/.cfg.",
    ),
    outcome(
        Code::IoError,
        "Reading .tpl failed, or the report could not be written to stdout.",
    ),
    outcome(
        Code::Configuration,
        "No project was found, .tpl/.cfg cannot be used, or no database entry is selected.",
    ),
];

/// The outcomes of `cfg get`.
const CFG_GET: &[Outcome] = &[
    outcome(Code::Ok, "The value was written to stdout."),
    outcome(
        Code::Usage,
        "The invocation is not valid: an unknown or repeated flag, a missing KEY, or --pretty \
         without --format json.",
    ),
    outcome(
        Code::NoInput,
        "KEY is absent from .tpl/.cfg; the nearest matches are suggested.",
    ),
    outcome(
        Code::IoError,
        "Reading .tpl failed, or the value could not be written to stdout.",
    ),
    outcome(
        Code::Configuration,
        "No project was found, or .tpl/.cfg cannot be used.",
    ),
];

/// The outcomes of `cfg set`.
const CFG_SET: &[Outcome] = &[
    outcome(Code::Ok, "The key was written to .tpl/.cfg."),
    outcome(
        Code::Usage,
        "KEY is outside the configuration key space, or VALUE does not conform to the type that \
         key declares.",
    ),
    outcome(Code::IoError, "Reading or rewriting .tpl/.cfg failed."),
    outcome(
        Code::Configuration,
        "No project was found, or .tpl/.cfg cannot be used.",
    ),
];

/// The outcomes of `cfg unset`.
const CFG_UNSET: &[Outcome] = &[
    outcome(Code::Ok, "The key or block was removed from .tpl/.cfg."),
    outcome(
        Code::Usage,
        "The invocation is not valid: an unknown or repeated flag, or a missing KEY.",
    ),
    outcome(Code::NoInput, "KEY is absent from .tpl/.cfg."),
    outcome(Code::IoError, "Reading or rewriting .tpl/.cfg failed."),
    outcome(
        Code::Configuration,
        "No project was found, or .tpl/.cfg cannot be used.",
    ),
];

/// The outcomes of `cfg list` and `cfg database list`.
const CFG_LISTING: &[Outcome] = &[
    outcome(
        Code::Ok,
        "The listing was written to stdout. An empty listing is a state, not a failure.",
    ),
    outcome(
        Code::Usage,
        "The invocation is not valid: an unknown or repeated flag, or --pretty without \
         --format json.",
    ),
    outcome(
        Code::IoError,
        "Reading .tpl failed, or the listing could not be written to stdout.",
    ),
    outcome(
        Code::Configuration,
        "No project was found, or .tpl/.cfg cannot be used.",
    ),
];

/// The outcomes of `cfg database add`.
const CFG_DATABASE_ADD: &[Outcome] = &[
    outcome(Code::Ok, "The entry was written to .tpl/.cfg."),
    outcome(
        Code::Usage,
        "Neither --dsn nor a discrete connection flag was supplied, both groups were, or NAME is \
         already taken.",
    ),
    outcome(Code::IoError, "Reading or rewriting .tpl/.cfg failed."),
    outcome(
        Code::Configuration,
        "No project was found, or .tpl/.cfg cannot be used.",
    ),
];

/// The outcomes of `cfg database show`, `update` and `remove`.
const CFG_DATABASE_ENTRY: &[Outcome] = &[
    outcome(Code::Ok, "The command succeeded."),
    outcome(
        Code::Usage,
        "The invocation is not valid: a missing NAME, an unknown or repeated flag, or --dsn \
         together with a discrete connection flag.",
    ),
    outcome(Code::NoInput, "NAME names no entry of .tpl/.cfg."),
    outcome(
        Code::IoError,
        "Reading .tpl/.cfg failed, rewriting it failed, or the result could not be written to \
         stdout.",
    ),
    outcome(
        Code::Configuration,
        "No project was found, or .tpl/.cfg cannot be used.",
    ),
];

/// The outcomes of `cfg database test`, the one `cfg` subcommand that connects.
const CFG_DATABASE_TEST: &[Outcome] = &[
    outcome(
        Code::Ok,
        "The four steps ran. This does not promise the entry is fully usable: read \
         can_read_catalogue for the fourth answer.",
    ),
    outcome(
        Code::Usage,
        "The invocation is not valid: a missing NAME, an unknown or repeated flag, or --pretty \
         without --format json.",
    ),
    outcome(Code::NoInput, "NAME names no entry of .tpl/.cfg."),
    outcome(
        Code::Unavailable,
        "The server could not be reached, or a network phase exceeded its deadline.",
    ),
    outcome(
        Code::IoError,
        "Reading .tpl failed, or the report could not be written to stdout.",
    ),
    outcome(Code::NoPermission, "The server refused the credentials."),
    outcome(
        Code::Configuration,
        "The connection and the authentication succeeded and the server is not a supported \
         MariaDB series, the read-only session could not be enforced, no project was found, or \
         .tpl/.cfg cannot be used.",
    ),
];

/// The outcomes of `tpl init`, the only command that can produce `73`.
const INIT: &[Outcome] = &[
    outcome(
        Code::Ok,
        "The project was created and nothing was written to stdout. A project created under \
         another one succeeds and warns on stderr that it shadows the one above.",
    ),
    outcome(
        Code::Usage,
        "The invocation is not valid: an unknown or repeated flag, or more than one PATH.",
    ),
    outcome(
        Code::CantCreate,
        "A .tpl folder already exists at the destination, in which case nothing was changed, or \
         the destination could not be created.",
    ),
];

/// The outcomes of `tpl help`.
const HELP: &[Outcome] = &[
    outcome(
        Code::Ok,
        "The help or the command tree was written to stdout.",
    ),
    outcome(
        Code::Usage,
        "A segment of COMMAND_PATH names no child of the node the preceding segments reached, or \
         --pretty was given without --format json.",
    ),
    outcome(
        Code::IoError,
        "The help could not be written to stdout, or stdout closed part-way through the JSON \
         document.",
    ),
];

/// The outcomes of `tpl version`.
const VERSION: &[Outcome] = &[
    outcome(Code::Ok, "The version was written to stdout."),
    outcome(
        Code::Usage,
        "The invocation is not valid: this command declares no argument and no flag of its own.",
    ),
    outcome(Code::IoError, "The version could not be written to stdout."),
];

/// One entry per node of the tree of `FR-CLI-002`, in the order `FR-HELP-019`
/// requires: each node followed by its own children before the next node at
/// its level, preserving the order the tree declares them in.
const ENTRIES: [Entry; 35] = [
    Entry {
        path: &[],
        description: "tpl reads the structure of a MariaDB database and renders MiniJinja \
                      templates against it. It never writes to the database. It works inside a \
                      project — a .tpl folder holding the configuration and the templates — \
                      which tpl init creates. Every command accepts the seven global flags \
                      listed below, in any position.",
        examples: &[
            Example {
                caption: "Print this help.",
                lines: &[run(&["tpl"])],
            },
            Example {
                caption: "Load the whole command surface, as one JSON document.",
                lines: &[run(&["tpl", "help", "--format", "json"])],
            },
            Example {
                caption: "Create a project in the current directory.",
                lines: &[run(&["tpl", "init"])],
            },
        ],
        exit_codes: ROOT,
        see_also: &[&["help"], &["init"], &["version"]],
    },
    Entry {
        path: &["schema"],
        description: "Reads the structure of the selected database and presents it. Every \
                      subcommand reads through the catalogue cache and issues no statement but a \
                      SELECT against INFORMATION_SCHEMA.",
        examples: &[
            Example {
                caption: "Print this help.",
                lines: &[run(&["tpl", "schema"])],
            },
            Example {
                caption: "List the tables of the entry named shop.",
                lines: &[run(&["tpl", "-d", "shop", "schema", "tables"])],
            },
        ],
        exit_codes: GROUP,
        see_also: &[&["render"], &["cache"], &["cfg", "database"]],
    },
    Entry {
        path: &["schema", "info"],
        description: "Reports the metadata of the selected database — its name, character set \
                      and collation — the server it lives on, and the collections it holds.",
        examples: &[
            Example {
                caption: "Report the selected database.",
                lines: &[run(&["tpl", "-d", "shop", "schema", "info"])],
            },
            Example {
                caption: "Read the server version from the JSON document.",
                lines: &[
                    run_in(
                        "",
                        &["tpl", "-d", "shop", "schema", "info", "--format", "json"],
                        " |",
                    ),
                    shell("  jq -r '.data.database.server.version'"),
                ],
            },
        ],
        exit_codes: SCHEMA_LISTING,
        see_also: &[&["schema", "dump"], &["cfg", "database", "test"]],
    },
    Entry {
        path: &["schema", "tables"],
        description: "Lists the tables of the selected database, one row each. Use --format json \
                      to iterate: a table is named by the name column, and tpl render binds one \
                      table per invocation.",
        examples: &[
            Example {
                caption: "List every table.",
                lines: &[run(&["tpl", "-d", "shop", "schema", "tables"])],
            },
            Example {
                caption: "List the tables whose name begins with order.",
                lines: &[run(&[
                    "tpl",
                    "-d",
                    "shop",
                    "schema",
                    "tables",
                    "--pattern",
                    "order%",
                ])],
            },
            Example {
                caption: "Render one file per table, which is how a caller iterates.",
                lines: &[
                    run_in(
                        "",
                        &["tpl", "-d", "shop", "schema", "tables", "--format", "json"],
                        " |",
                    ),
                    shell("  jq -r '.data.tables[].name' |"),
                    shell("  while read -r table; do"),
                    run_in(
                        "    ",
                        &[
                            "tpl",
                            "-d",
                            "shop",
                            "render",
                            "rust/struct",
                            "--table",
                            "\"$table\"",
                        ],
                        " > \"src/models/$table.rs\"",
                    ),
                    shell("  done"),
                ],
            },
        ],
        exit_codes: SCHEMA_LISTING,
        see_also: &[&["schema", "table"], &["render"], &["cache", "load"]],
    },
    Entry {
        path: &["schema", "table"],
        description: "Describes one table: its columns, primary key, indexes, foreign keys, \
                      triggers, CHECK constraints, engine, collation and comment.",
        examples: &[
            Example {
                caption: "Describe the orders table.",
                lines: &[run(&["tpl", "-d", "shop", "schema", "table", "orders"])],
            },
            Example {
                caption: "The same, as indented JSON.",
                lines: &[run(&[
                    "tpl", "-d", "shop", "schema", "table", "orders", "--format", "json",
                    "--pretty",
                ])],
            },
        ],
        exit_codes: SCHEMA_OBJECT,
        see_also: &[&["schema", "tables"], &["render"]],
    },
    Entry {
        path: &["schema", "views"],
        description: "Lists the views of the selected database, one row each.",
        examples: &[
            Example {
                caption: "List every view.",
                lines: &[run(&["tpl", "-d", "shop", "schema", "views"])],
            },
            Example {
                caption: "List the views whose name begins with v.",
                lines: &[run(&[
                    "tpl",
                    "-d",
                    "shop",
                    "schema",
                    "views",
                    "--pattern",
                    "v%",
                ])],
            },
        ],
        exit_codes: SCHEMA_LISTING,
        see_also: &[&["schema", "view"], &["schema", "tables"]],
    },
    Entry {
        path: &["schema", "view"],
        description: "Describes one view, including its SQL definition.",
        examples: &[
            Example {
                caption: "Describe the v_sales view.",
                lines: &[run(&["tpl", "-d", "shop", "schema", "view", "v_sales"])],
            },
            Example {
                caption: "Print the SQL definition alone.",
                lines: &[
                    run_in(
                        "",
                        &[
                            "tpl", "-d", "shop", "schema", "view", "v_sales", "--format", "json",
                        ],
                        " |",
                    ),
                    shell("  jq -r '.data.view.definition'"),
                ],
            },
        ],
        exit_codes: SCHEMA_OBJECT,
        see_also: &[&["schema", "views"], &["render"]],
    },
    Entry {
        path: &["schema", "routines"],
        description: "Lists the stored procedures and the stored functions of the selected \
                      database together, each row stating its kind. There is no flag to select \
                      one kind: filter the JSON document instead.",
        examples: &[
            Example {
                caption: "List every routine.",
                lines: &[run(&["tpl", "-d", "shop", "schema", "routines"])],
            },
            Example {
                caption: "List each routine with its kind, one per line.",
                lines: &[
                    run_in(
                        "",
                        &[
                            "tpl", "-d", "shop", "schema", "routines", "--format", "json",
                        ],
                        " |",
                    ),
                    shell("  jq -r '.data.routines[] | .kind + \" \" + .name'"),
                ],
            },
        ],
        exit_codes: SCHEMA_LISTING,
        see_also: &[&["schema", "routine"]],
    },
    Entry {
        path: &["schema", "routine"],
        description: "Describes one stored procedure or stored function. Procedures and \
                      functions occupy distinct namespaces, so a bare name that matches both is \
                      refused: qualify it as procedure:NAME or function:NAME.",
        examples: &[
            Example {
                caption: "Describe the routine named calc_vat.",
                lines: &[run(&["tpl", "-d", "shop", "schema", "routine", "calc_vat"])],
            },
            Example {
                caption: "Describe the function named calc_vat, where a procedure shares the name.",
                lines: &[run(&[
                    "tpl",
                    "-d",
                    "shop",
                    "schema",
                    "routine",
                    "function:calc_vat",
                ])],
            },
        ],
        exit_codes: SCHEMA_ROUTINE,
        see_also: &[&["schema", "routines"], &["render"]],
    },
    Entry {
        path: &["schema", "dump"],
        description: "Writes the whole selected database as one JSON document. The output is \
                      JSON and nothing else, so this command declares no --format. The document \
                      carries the server-derived context alone — no vars, no tpl, no now — and \
                      is what tpl render --context reads back.",
        examples: &[
            Example {
                caption: "Write the database to a file.",
                lines: &[run_in(
                    "",
                    &["tpl", "-d", "shop", "schema", "dump"],
                    " > shop.json",
                )],
            },
            Example {
                caption: "Render from the dump, without touching a server.",
                lines: &[
                    run_in("", &["tpl", "-d", "shop", "schema", "dump"], " |"),
                    run_in(
                        "  ",
                        &[
                            "tpl",
                            "render",
                            "rust/struct",
                            "--context",
                            "-",
                            "--table",
                            "orders",
                        ],
                        "",
                    ),
                ],
            },
        ],
        exit_codes: SCHEMA_DUMP,
        see_also: &[&["render"], &["schema", "info"]],
    },
    Entry {
        path: &["template"],
        description: "Reads the templates the project carries, under .tpl/templates/. No \
                      subcommand opens a database connection, evaluates a template, or leaves \
                      the template root.",
        examples: &[
            Example {
                caption: "Print this help.",
                lines: &[run(&["tpl", "template"])],
            },
            Example {
                caption: "List the project's templates.",
                lines: &[run(&["tpl", "template", "list"])],
            },
        ],
        exit_codes: GROUP,
        see_also: &[&["render"], &["init"]],
    },
    Entry {
        path: &["template", "list"],
        description: "Lists every template of the project, ordered by name. A listed name omits \
                      the .jinja extension, and so does the positional argument of render, show, \
                      check and path. An {% include %} inside a template does not: it needs the \
                      extension written out.",
        examples: &[
            Example {
                caption: "List the project's templates.",
                lines: &[run(&["tpl", "template", "list"])],
            },
            Example {
                caption: "List them as JSON.",
                lines: &[run(&["tpl", "template", "list", "--format", "json"])],
            },
        ],
        exit_codes: TEMPLATE_LISTING,
        see_also: &[&["template", "show"], &["template", "path"], &["render"]],
    },
    Entry {
        path: &["template", "show"],
        description: "Writes the source of one template to stdout, byte for byte, with nothing \
                      escaped and nothing evaluated.",
        examples: &[
            Example {
                caption: "Print a template's source.",
                lines: &[run(&["tpl", "template", "show", "rust/struct"])],
            },
            Example {
                caption: "The extension is optional, and names the same template.",
                lines: &[run(&["tpl", "template", "show", "rust/struct.jinja"])],
            },
        ],
        exit_codes: TEMPLATE_NAMED,
        see_also: &[&["template", "list"], &["template", "check"]],
    },
    Entry {
        path: &["template", "check"],
        description: "Parses templates without rendering them. It is syntax analysis only: no \
                      expression is evaluated, no filter is called, no database is contacted. It \
                      is therefore safe to run against a template you have not read. Given no \
                      name, it checks every template of the project.",
        examples: &[
            Example {
                caption: "Check every template of the project.",
                lines: &[run(&["tpl", "template", "check"])],
            },
            Example {
                caption: "Check two named templates.",
                lines: &[run(&[
                    "tpl",
                    "template",
                    "check",
                    "rust/struct",
                    "docs/table.md",
                ])],
            },
        ],
        exit_codes: TEMPLATE_CHECK,
        see_also: &[&["template", "list"], &["render"]],
    },
    Entry {
        path: &["template", "path"],
        description: "Writes the absolute path a template name resolves to. Given no name, it \
                      writes the absolute path of the template root.",
        examples: &[
            Example {
                caption: "Print the template root.",
                lines: &[run(&["tpl", "template", "path"])],
            },
            Example {
                caption: "Print where one template lives.",
                lines: &[run(&["tpl", "template", "path", "rust/struct"])],
            },
        ],
        exit_codes: TEMPLATE_NAMED,
        see_also: &[&["template", "list"], &["template", "show"]],
    },
    Entry {
        path: &["render"],
        description: "Renders one template, once, and writes the result to stdout. The context \
                      comes either from the selected database or from a --context document, \
                      never from both. One object at most is bound, named by --table, --view or \
                      --routine; with none, the whole database is in context. Redirect stdout to \
                      write a file: this command has no --output.",
        examples: &[
            Example {
                caption: "Render a template with one table bound.",
                lines: &[run(&[
                    "tpl",
                    "-d",
                    "shop",
                    "render",
                    "rust/struct",
                    "--table",
                    "orders",
                ])],
            },
            Example {
                caption: "Write the result to a file.",
                lines: &[run_in(
                    "",
                    &[
                        "tpl",
                        "-d",
                        "shop",
                        "render",
                        "rust/struct",
                        "--table",
                        "orders",
                    ],
                    " > src/models/orders.rs",
                )],
            },
            Example {
                caption: "Pass two template variables, read as vars.title and vars.author.",
                lines: &[run(&[
                    "tpl",
                    "-d",
                    "shop",
                    "render",
                    "docs/table.md",
                    "--table",
                    "orders",
                    "--set",
                    "title=Orders",
                    "--set",
                    "author=data-team",
                ])],
            },
            Example {
                caption: "Render one file per table, which is how a caller iterates.",
                lines: &[
                    run_in(
                        "",
                        &["tpl", "-d", "shop", "schema", "tables", "--format", "json"],
                        " |",
                    ),
                    shell("  jq -r '.data.tables[].name' |"),
                    shell("  while read -r table; do"),
                    run_in(
                        "    ",
                        &[
                            "tpl",
                            "-d",
                            "shop",
                            "render",
                            "rust/struct",
                            "--table",
                            "\"$table\"",
                        ],
                        " > \"src/models/$table.rs\"",
                    ),
                    shell("  done"),
                ],
            },
            Example {
                caption: "Render from a dump, reading the context from stdin.",
                lines: &[
                    run_in("", &["tpl", "-d", "shop", "schema", "dump"], " |"),
                    run_in(
                        "  ",
                        &[
                            "tpl",
                            "render",
                            "rust/struct",
                            "--context",
                            "-",
                            "--table",
                            "orders",
                        ],
                        "",
                    ),
                ],
            },
        ],
        exit_codes: RENDER,
        see_also: &[
            &["template", "list"],
            &["template", "check"],
            &["schema", "tables"],
            &["schema", "dump"],
        ],
    },
    Entry {
        path: &["cache"],
        description: "Manages the catalogue cache under .tpl/.cache/, one folder per database \
                      entry. The cache is read through on every catalogue read, written on a \
                      miss, and never expires on its own: only load and clean change what it \
                      holds.",
        examples: &[
            Example {
                caption: "Print this help.",
                lines: &[run(&["tpl", "cache"])],
            },
            Example {
                caption: "Report what the cache holds for the entry named shop.",
                lines: &[run(&["tpl", "-d", "shop", "cache", "status"])],
            },
        ],
        exit_codes: GROUP,
        see_also: &[&["schema"], &["render"]],
    },
    Entry {
        path: &["cache", "load"],
        description: "Reads the catalogue from the server and stores it. With no object flag it \
                      loads the whole catalogue of the selected entry. --direct is accepted and \
                      ignored, because reading the server is what this command does; --no-cache \
                      is refused, because loading without storing is a contradiction.",
        examples: &[
            Example {
                caption: "Load the whole catalogue of the entry named shop.",
                lines: &[run(&["tpl", "-d", "shop", "cache", "load"])],
            },
            Example {
                caption: "Load one table.",
                lines: &[run(&[
                    "tpl", "-d", "shop", "cache", "load", "--table", "orders",
                ])],
            },
        ],
        exit_codes: CACHE_LOAD,
        see_also: &[&["cache", "clean"], &["cache", "status"], &["schema"]],
    },
    Entry {
        path: &["cache", "clean"],
        description: "Removes cached data for the selected entry. With no object flag it removes \
                      all of it. Nothing else invalidates the cache: repointing an entry at \
                      another server leaves what was stored in place, so clean after repointing.",
        examples: &[
            Example {
                caption: "Remove everything cached for the entry named shop.",
                lines: &[run(&["tpl", "-d", "shop", "cache", "clean"])],
            },
            Example {
                caption: "Remove one view.",
                lines: &[run(&[
                    "tpl", "-d", "shop", "cache", "clean", "--view", "v_sales",
                ])],
            },
        ],
        exit_codes: CACHE_CLEAN,
        see_also: &[
            &["cache", "load"],
            &["cache", "status"],
            &["cfg", "database", "update"],
        ],
    },
    Entry {
        path: &["cache", "status"],
        description: "Reports the database entry, when its cache was loaded, and the object \
                      counts it holds. This is the supported way to learn the state of the \
                      cache: the on-disk layout is not a contract.",
        examples: &[
            Example {
                caption: "Report the cache of the entry named shop.",
                lines: &[run(&["tpl", "-d", "shop", "cache", "status"])],
            },
            Example {
                caption: "Read the load time from the JSON document.",
                lines: &[
                    run_in(
                        "",
                        &["tpl", "-d", "shop", "cache", "status", "--format", "json"],
                        " |",
                    ),
                    shell("  jq -r '.data.loaded_at'"),
                ],
            },
        ],
        exit_codes: CACHE_STATUS,
        see_also: &[&["cache", "load"], &["cache", "clean"]],
    },
    Entry {
        path: &["cfg"],
        description: "Manages .tpl/.cfg, the only file these commands write. It has two arms: \
                      dotted keys, for any single value, and the database subgroup, so that \
                      registering a connection is one invocation rather than five. No subcommand \
                      contacts a server except cfg database test.",
        examples: &[
            Example {
                caption: "Print this help.",
                lines: &[run(&["tpl", "cfg"])],
            },
            Example {
                caption: "Print the configuration, with passwords redacted.",
                lines: &[run(&["tpl", "cfg", "list"])],
            },
        ],
        exit_codes: GROUP,
        see_also: &[&["cfg", "database"], &["init"]],
    },
    Entry {
        path: &["cfg", "get"],
        description: "Writes the value stored under one key, exactly as the file holds it: \
                      ${VAR} is not expanded and nothing is redacted. This is the one deliberate \
                      exception to redaction, so that a password can be fed to another command.",
        examples: &[
            Example {
                caption: "Read the entry the project uses by default.",
                lines: &[run(&["tpl", "cfg", "get", "core.database"])],
            },
            Example {
                caption: "Read one field of an entry, as JSON.",
                lines: &[run(&[
                    "tpl",
                    "cfg",
                    "get",
                    "database.shop.host",
                    "--format",
                    "json",
                ])],
            },
        ],
        exit_codes: CFG_GET,
        see_also: &[&["cfg", "set"], &["cfg", "list"]],
    },
    Entry {
        path: &["cfg", "set"],
        description: "Writes one value under one key. The key must be in the configuration key \
                      space and the value must conform to the type that key declares. A value \
                      given here is visible in the process table for the life of the invocation; \
                      where the key admits it, write ${VAR} instead and keep the secret in the \
                      environment.",
        examples: &[
            Example {
                caption: "Make shop the entry used when -d is absent.",
                lines: &[run(&["tpl", "cfg", "set", "core.database", "shop"])],
            },
            Example {
                caption: "Take a password from the environment rather than from the file.",
                lines: &[run(&[
                    "tpl",
                    "cfg",
                    "set",
                    "database.shop.password",
                    "'${SHOP_PASSWORD}'",
                ])],
            },
        ],
        exit_codes: CFG_SET,
        see_also: &[
            &["cfg", "get"],
            &["cfg", "unset"],
            &["cfg", "database", "add"],
        ],
    },
    Entry {
        path: &["cfg", "unset"],
        description: "Removes one key, or a whole block. database.shop.host removes that field; \
                      database.shop removes the entry it belongs to.",
        examples: &[
            Example {
                caption: "Remove one field of an entry.",
                lines: &[run(&["tpl", "cfg", "unset", "database.shop.port"])],
            },
            Example {
                caption: "Remove a whole block.",
                lines: &[run(&["tpl", "cfg", "unset", "database.shop"])],
            },
        ],
        exit_codes: CFG_UNSET,
        see_also: &[&["cfg", "set"], &["cfg", "database", "remove"]],
    },
    Entry {
        path: &["cfg", "list"],
        description: "Writes the contents of .tpl/.cfg, with passwords redacted and ${VAR} left \
                      exactly as written. It does not resolve the configuration: no expansion, \
                      no password_command, no defaults.",
        examples: &[
            Example {
                caption: "Print the configuration.",
                lines: &[run(&["tpl", "cfg", "list"])],
            },
            Example {
                caption: "Print it as JSON.",
                lines: &[run(&["tpl", "cfg", "list", "--format", "json"])],
            },
        ],
        exit_codes: CFG_LISTING,
        see_also: &[&["cfg", "get"], &["cfg", "database", "list"]],
    },
    Entry {
        path: &["cfg", "database"],
        description: "Manages the database entries of .tpl/.cfg — the named connections that \
                      -d/--database selects. The alias db is accepted wherever database is.",
        examples: &[
            Example {
                caption: "Print this help.",
                lines: &[run(&["tpl", "cfg", "database"])],
            },
            Example {
                caption: "List the entries, written through the alias.",
                lines: &[run(&["tpl", "cfg", "db", "list"])],
            },
        ],
        exit_codes: GROUP,
        see_also: &[&["cfg"], &["cfg", "database", "add"]],
    },
    Entry {
        path: &["cfg", "database", "add"],
        description: "Creates a database entry from the flags supplied. Either --dsn or at least \
                      one discrete connection flag is required, and the two groups are mutually \
                      exclusive. A name that is already taken is refused: use update to change \
                      an entry. A value given on the command line is visible in the process \
                      table for the life of the invocation.",
        examples: &[
            Example {
                caption: "Register a connection from discrete fields.",
                lines: &[run(&[
                    "tpl",
                    "cfg",
                    "database",
                    "add",
                    "shop",
                    "--host",
                    "db.example.com",
                    "--user",
                    "reader",
                    "--schema",
                    "shop",
                ])],
            },
            Example {
                caption: "Register a connection as one URL.",
                lines: &[run(&[
                    "tpl",
                    "cfg",
                    "database",
                    "add",
                    "reporting",
                    "--dsn",
                    "mysql://reader@db.example.com:3306/reporting",
                ])],
            },
            Example {
                caption: "Register one that takes its password from a keychain command.",
                lines: &[run(&[
                    "tpl",
                    "cfg",
                    "database",
                    "add",
                    "staging",
                    "--host",
                    "db-staging.example.com",
                    "--user",
                    "reader",
                    "--schema",
                    "shop",
                    "--tls",
                    "verify-identity",
                    "--password-command",
                    "\"pass db/staging\"",
                ])],
            },
        ],
        exit_codes: CFG_DATABASE_ADD,
        see_also: &[
            &["cfg", "database", "update"],
            &["cfg", "database", "test"],
            &["cfg", "database", "list"],
        ],
    },
    Entry {
        path: &["cfg", "database", "list"],
        description: "Writes the names of the database entries the file defines. A fresh project \
                      defines none.",
        examples: &[
            Example {
                caption: "List the entries.",
                lines: &[run(&["tpl", "cfg", "database", "list"])],
            },
            Example {
                caption: "List them as JSON.",
                lines: &[run(&["tpl", "cfg", "database", "list", "--format", "json"])],
            },
        ],
        exit_codes: CFG_LISTING,
        see_also: &[&["cfg", "database", "show"], &["cfg", "database", "add"]],
    },
    Entry {
        path: &["cfg", "database", "show"],
        description: "Writes one database entry, with passwords redacted and ${VAR} left exactly \
                      as written.",
        examples: &[
            Example {
                caption: "Show one entry.",
                lines: &[run(&["tpl", "cfg", "database", "show", "shop"])],
            },
            Example {
                caption: "Show it as JSON.",
                lines: &[run(&[
                    "tpl", "cfg", "database", "show", "shop", "--format", "json",
                ])],
            },
        ],
        exit_codes: CFG_DATABASE_ENTRY,
        see_also: &[&["cfg", "database", "list"], &["cfg", "get"]],
    },
    Entry {
        path: &["cfg", "database", "update"],
        description: "Changes the fields of one database entry named by the flags supplied, \
                      leaving the rest of the entry untouched. Repointing an entry does not \
                      invalidate its cache: run cache clean afterwards.",
        examples: &[
            Example {
                caption: "Point an entry at another host.",
                lines: &[
                    run(&[
                        "tpl",
                        "cfg",
                        "database",
                        "update",
                        "shop",
                        "--host",
                        "db-staging.example.com",
                    ]),
                    run(&["tpl", "-d", "shop", "cache", "clean"]),
                ],
            },
            Example {
                caption: "Require a verified certificate chain.",
                lines: &[run(&[
                    "tpl",
                    "cfg",
                    "database",
                    "update",
                    "shop",
                    "--tls",
                    "verify-ca",
                    "--ca-file",
                    "/etc/ssl/certs/db-ca.pem",
                ])],
            },
        ],
        exit_codes: CFG_DATABASE_ENTRY,
        see_also: &[
            &["cfg", "database", "add"],
            &["cfg", "database", "show"],
            &["cache", "clean"],
        ],
    },
    Entry {
        path: &["cfg", "database", "remove"],
        description: "Deletes one database entry. Removing the entry named by core.database also \
                      clears core.database, leaving the file coherent.",
        examples: &[Example {
            caption: "Delete an entry.",
            lines: &[run(&["tpl", "cfg", "database", "remove", "staging"])],
        }],
        exit_codes: CFG_DATABASE_ENTRY,
        see_also: &[&["cfg", "database", "list"], &["cfg", "unset"]],
    },
    Entry {
        path: &["cfg", "database", "test"],
        description: "Opens a connection with one database entry and reports four things: that \
                      it connected and authenticated, that the read-only session was enforced, \
                      which MariaDB server answered, and whether this reader can read the \
                      catalogue. Exit 0 means the four steps ran, not that the entry is fully \
                      usable: read can_read_catalogue for the fourth answer.",
        examples: &[
            Example {
                caption: "Test an entry.",
                lines: &[run(&["tpl", "cfg", "database", "test", "shop"])],
            },
            Example {
                caption: "Read the fourth answer, which the exit code does not carry.",
                lines: &[
                    run_in(
                        "",
                        &["tpl", "cfg", "database", "test", "shop", "--format", "json"],
                        " |",
                    ),
                    shell("  jq -r '.data.can_read_catalogue'"),
                ],
            },
        ],
        exit_codes: CFG_DATABASE_TEST,
        see_also: &[
            &["cfg", "database", "show"],
            &["cfg", "database", "update"],
            &["schema", "info"],
        ],
    },
    Entry {
        path: &["init"],
        description: "Creates a .tpl project at the destination, and any missing parent \
                      directory with it. It writes five artefacts: .cfg at mode 0600, a \
                      .gitignore, the templates folder, a worked example template, and a macro \
                      file mapping a column to a Rust type. The generated .cfg carries no active \
                      database entry, only a commented example showing the shape one takes.",
        examples: &[
            Example {
                caption: "Create a project in the current directory.",
                lines: &[run(&["tpl", "init"])],
            },
            Example {
                caption: "Create one at a path, parents included.",
                lines: &[run(&["tpl", "init", "projects/reports"])],
            },
        ],
        exit_codes: INIT,
        see_also: &[&["cfg", "database", "add"], &["template", "list"]],
    },
    Entry {
        path: &["help"],
        description: "Writes the help of one node of the command tree, or of tpl itself. \
                      COMMAND_PATH is the full path of the node, at any depth, and an alias in \
                      it resolves to its canonical node. With --format json the whole command \
                      tree is written as one document instead, reduced to the subtree the path \
                      names.",
        examples: &[
            Example {
                caption: "Print the top-level help.",
                lines: &[run(&["tpl", "help"])],
            },
            Example {
                caption: "Print the help of a node three levels down.",
                lines: &[run(&["tpl", "help", "cfg", "database", "add"])],
            },
            Example {
                caption: "Load the whole command surface as one JSON document.",
                lines: &[run(&["tpl", "help", "--format", "json"])],
            },
            Example {
                caption: "Load one subtree of it.",
                lines: &[run(&["tpl", "help", "cfg", "database", "--format", "json"])],
            },
        ],
        exit_codes: HELP,
        see_also: &[&["version"]],
    },
    Entry {
        path: &["version"],
        description: "Writes the version, as tpl followed by the version and a newline, and \
                      nothing else. It declares no argument and no flag of its own.",
        examples: &[Example {
            caption: "Print the version.",
            lines: &[run(&["tpl", "version"])],
        }],
        exit_codes: VERSION,
        see_also: &[&["help"]],
    },
];

#[cfg(test)]
mod tests {
    use super::{
        ARGUMENTS, Code, Documented, ENTRIES, Entry, GLOBAL_ARGUMENTS, documented, entries, entry,
        resolve,
    };
    use crate::cli::{parse, tree};
    use crate::error::Error;

    /// The name the typed table indexes one argument of the tree by: a flag by
    /// its long form with both dashes, a positional argument by its value name.
    fn key(argument: &clap::Arg) -> String {
        if argument.is_positional() {
            super::render::value_name(argument).to_owned()
        } else {
            format!(
                "--{}",
                argument
                    .get_long()
                    .unwrap_or_else(|| argument.get_id().as_str())
            )
        }
    }

    /// Every (node path, argument name) pair the **tree** declares, walked
    /// rather than enumerated, with the seven global flags of the root among
    /// them.
    fn declared() -> Vec<(Vec<String>, String)> {
        let tree = tree();
        let mut pairs = Vec::new();

        visit(&tree, &[], &mut |path, node| {
            for argument in node.get_arguments() {
                pairs.push((owned(path), key(argument)));
            }
        });

        pairs
    }

    /// The segments of a command path, as `tpl help` receives them.
    fn segments(path: &[&str]) -> Vec<String> {
        path.iter().map(|&segment| segment.to_owned()).collect()
    }

    /// Applies `visitor` to every node of `command`, with the path it is
    /// reached by, in declaration order.
    fn visit(
        command: &clap::Command,
        path: &[&str],
        visitor: &mut impl FnMut(&[&str], &clap::Command),
    ) {
        visitor(path, command);

        for child in command.get_subcommands() {
            let mut below: Vec<&str> = path.to_vec();
            below.push(child.get_name());
            visit(child, &below, visitor);
        }
    }

    /// Every node of the built tree, by the path it is reached by, in the order
    /// `FR-HELP-019` requires: each node followed by its own children before
    /// the next node at its level.
    fn node_paths() -> Vec<Vec<String>> {
        let tree = tree();
        let mut paths = Vec::new();
        visit(&tree, &[], &mut |path, _| {
            paths.push(path.iter().map(|&segment| segment.to_owned()).collect());
        });

        paths
    }

    /// The path of `entry`, as the node walk writes it.
    fn owned(path: &[&str]) -> Vec<String> {
        path.iter().map(|&segment| segment.to_owned()).collect()
    }

    #[test]
    fn fr_help_030_every_flag_and_positional_argument_of_the_tree_carries_a_sentence() {
        // FR-HELP-030: one sentence per flag and per positional argument,
        // saying what supplying it does. The population is read from the
        // **tree**, so an argument added to a node is covered here without
        // anything in this test changing — which is the property enumerating
        // the arguments by hand would not have.
        for (path, name) in declared() {
            let segments: Vec<&str> = path.iter().map(String::as_str).collect();
            let stated = documented(&segments, &name)
                .unwrap_or_else(|| panic!("{} states no purpose for {name}", written(&segments)));

            assert!(
                !stated.purpose.trim().is_empty(),
                "{} states an empty purpose for {name}",
                written(&segments)
            );
            assert!(
                stated.purpose.ends_with('.'),
                "{} states no sentence for {name}: {:?}",
                written(&segments),
                stated.purpose
            );
        }
    }

    #[test]
    fn fr_help_014_no_sentence_of_the_table_carries_a_requirement_identifier() {
        // FR-HELP-014 makes help self-contained: it refers to no document
        // outside the help system, and a requirement identifier is a citation
        // of one. FR-HELP-030 says so in its own words, which is why the text
        // lives here and not on the declarations, whose doc comments carry
        // them.
        //
        // FR-HELP-015 bars a decorative character with it, and a backtick is
        // `rustdoc` markup rather than anything a caller reads.
        const PREFIXES: [&str; 8] = ["FR-", "NFR-", "BR-", "UC-", "OD-", "ADR-", "OQ-", "DIV-"];

        let every = GLOBAL_ARGUMENTS
            .iter()
            .chain(ARGUMENTS.iter().flat_map(|(_, stated)| stated.iter()));

        for stated in every {
            for prefix in PREFIXES {
                assert!(
                    !stated.purpose.contains(prefix),
                    "{} cites {prefix} in its sentence: {:?}",
                    stated.name,
                    stated.purpose
                );
            }

            assert!(
                !stated.purpose.contains('`'),
                "{} carries a backtick: {:?}",
                stated.name,
                stated.purpose
            );
        }
    }

    #[test]
    fn fr_help_022_the_table_states_nothing_about_an_argument_the_tree_does_not_declare() {
        // The other direction of the walk above: a row whose node or whose
        // argument left the tree is a row that can no longer be reached, and
        // which would go on stating something of nothing.
        let mut declared = declared();
        declared.sort();

        for (path, stated) in ARGUMENTS {
            let owned: Vec<String> = path.iter().map(|&segment| segment.to_owned()).collect();

            for row in stated {
                assert!(
                    declared
                        .binary_search(&(owned.clone(), (*row.name).to_owned()))
                        .is_ok(),
                    "{} states {} and the tree declares no such argument there",
                    written(path),
                    row.name
                );
            }
        }
    }

    #[test]
    fn fr_help_013_every_mutual_exclusion_the_table_declares_names_an_argument_of_the_same_node() {
        // FR-HELP-013's sixth fact. The tree declares no `conflicts_with` to
        // introspect, so the pairs are stated here; what this holds is that
        // each names an argument that exists where it is stated — a global flag
        // for a global row, and an argument of that node for a local one — so
        // the help cannot send a caller after a flag the node does not declare.
        let tree = tree();

        for stated in GLOBAL_ARGUMENTS {
            for excluded in stated.excludes {
                assert!(
                    GLOBAL_ARGUMENTS.iter().any(|other| other.name == *excluded),
                    "the global flag {} excludes {excluded}, which is not a global flag",
                    stated.name
                );
            }
        }

        for (path, rows) in ARGUMENTS {
            let mut node = &tree;
            for segment in path {
                node = node
                    .find_subcommand(segment)
                    .unwrap_or_else(|| panic!("{} is a node of the tree", written(path)));
            }

            let declared: Vec<String> = node.get_arguments().map(key).collect();

            for row in rows {
                for excluded in row.excludes {
                    assert!(
                        declared.iter().any(|name| name == excluded)
                            || GLOBAL_ARGUMENTS.iter().any(|other| other.name == *excluded),
                        "{} states that {} excludes {excluded}, which it does not declare",
                        written(path),
                        row.name
                    );
                }
            }
        }
    }

    #[test]
    fn fr_cli_015_and_fr_rnd_005_state_their_pairs_symmetrically() {
        // A pair is a relation, and a caller reading either member has to meet
        // it. The two global flags of FR-CLI-015 and the three object flags of
        // FR-RND-005 are the pairs both of whose members the same node
        // declares, and each names the other.
        for stated in GLOBAL_ARGUMENTS {
            for excluded in stated.excludes {
                let other = GLOBAL_ARGUMENTS
                    .iter()
                    .find(|other| other.name == *excluded)
                    .expect("the excluded flag is a global flag");

                assert!(
                    other.excludes.contains(&stated.name),
                    "{} excludes {excluded} and is not excluded back",
                    stated.name
                );
            }
        }

        for (path, rows) in ARGUMENTS {
            for row in rows {
                for excluded in row.excludes {
                    let Some(other) = rows.iter().find(|other| other.name == *excluded) else {
                        // The one asymmetric pair of the tree: FR-RND-018
                        // refuses --context beside a flag the **root**
                        // declares, and FR-GLOB-003 forbids repeating a global
                        // flag at another node, so it is stated on --context
                        // alone.
                        assert!(
                            GLOBAL_ARGUMENTS.iter().any(|flag| flag.name == *excluded),
                            "{} excludes {excluded}, which is neither local nor global",
                            row.name
                        );
                        continue;
                    };

                    assert!(
                        other.excludes.contains(&row.name),
                        "{} excludes {excluded} at {} and is not excluded back",
                        row.name,
                        written(path)
                    );
                }
            }
        }
    }

    #[test]
    fn fr_help_013_the_four_exclusions_this_corpus_obliges_are_all_stated() {
        // A floor under the walks above, which would pass over a table that
        // stated no exclusion at all. The four are FR-CLI-015, FR-RND-005,
        // FR-RND-018 and FR-CFG-029, named by the node that refuses each.
        let stated = |path: &[&str], name: &str| -> &'static [&'static str] {
            documented(path, name)
                .unwrap_or_else(|| panic!("{} states nothing about {name}", written(path)))
                .excludes
        };

        // FR-CLI-015, the one global pair.
        assert_eq!(stated(&[], "--quiet"), ["--verbose"]);
        assert_eq!(stated(&[], "--verbose"), ["--quiet"]);

        // FR-RND-005, on each of the three nodes that declare the object flags.
        for path in [
            &["render"][..],
            &["cache", "load"][..],
            &["cache", "clean"][..],
        ] {
            assert_eq!(stated(path, "--table"), ["--view", "--routine"]);
            assert_eq!(stated(path, "--view"), ["--table", "--routine"]);
            assert_eq!(stated(path, "--routine"), ["--table", "--view"]);
        }

        // FR-RND-018.
        assert_eq!(stated(&["render"], "--context"), ["--database"]);

        // FR-CFG-029, on both nodes that declare the entry flags. The discrete
        // **connection** flags are the four that say where to connect.
        for path in [
            &["cfg", "database", "add"][..],
            &["cfg", "database", "update"][..],
        ] {
            assert_eq!(
                stated(path, "--dsn"),
                ["--host", "--port", "--user", "--schema"]
            );

            for discrete in ["--host", "--port", "--user", "--schema"] {
                assert_eq!(stated(path, discrete), ["--dsn"]);
            }
        }
    }

    #[test]
    fn fr_help_030_a_row_is_written_once_for_a_flag_more_than_one_node_declares() {
        // `cli::local` declares such a flag once, and the sentence follows the
        // declaration: two nodes naming one flag read one row, so the text
        // cannot drift between two nodes that one row of FR-GLOB-021 names
        // together.
        let shared: Vec<&Documented> = ARGUMENTS
            .iter()
            .flat_map(|(_, stated)| stated.iter())
            .filter(|stated| stated.name == "--pretty")
            .collect();

        assert!(shared.len() > 1, "--pretty is declared by one node only");

        for stated in &shared {
            assert_eq!(**stated, *shared[0]);
        }
    }

    /// The path of `entry` as a caller writes it, for a failure message.
    fn written(path: &[&str]) -> String {
        std::iter::once("tpl")
            .chain(path.iter().copied())
            .collect::<Vec<&str>>()
            .join(" ")
    }

    /// Every entry of `entry`'s `exit_codes`, by code.
    fn codes(entry: &Entry) -> Vec<u8> {
        entry
            .exit_codes
            .iter()
            .map(|outcome| outcome.code.code())
            .collect()
    }

    /// The entries whose `exit_codes` carry `code`.
    fn carrying(code: Code) -> Vec<String> {
        ENTRIES
            .iter()
            .filter(|entry| entry.exit_codes.iter().any(|outcome| outcome.code == code))
            .map(|entry| written(entry.path))
            .collect()
    }

    #[test]
    fn fr_help_026_a_path_of_any_depth_resolves_to_the_node_it_names() {
        // FR-HELP-026: the full path of any node, at any depth, as a sequence
        // of positional arguments. Every node of the tree is handed back to
        // the resolver by the path the walk reached it by, which is what
        // FR-HELP-026's rationale means by a caller handing the path back.
        let tree = tree();

        for path in node_paths() {
            let written: Vec<&str> = path.iter().map(String::as_str).collect();
            let (node, canonical) =
                resolve(&tree, &path).unwrap_or_else(|error| panic!("{path:?}: {error}"));

            assert_eq!(canonical, written);
            assert_eq!(
                node.get_name(),
                written.last().copied().unwrap_or("tpl"),
                "{path:?}"
            );
        }
    }

    #[test]
    fn fr_help_027_an_alias_resolves_to_the_canonical_node_and_a_prefix_resolves_to_none() {
        // FR-HELP-027: an alias of FR-CLI-011 reaches the node it names, and
        // no segment is inferred from a prefix, per FR-CLI-004. The seven
        // aliases are read from the tree rather than listed, so an alias added
        // to the tree is covered without anything here changing.
        let tree = tree();

        for (path, alias) in [
            (&["schema", "tables"][..], "tbls"),
            (&["schema", "table"][..], "tbl"),
            (&["schema", "views"][..], "vws"),
            (&["schema", "view"][..], "vw"),
            (&["schema", "routines"][..], "rtns"),
            (&["schema", "routine"][..], "rtn"),
            (&["cfg", "database"][..], "db"),
        ] {
            let mut written = segments(path);
            let last = written.len() - 1;
            written[last] = alias.to_owned();

            let (_, canonical) =
                resolve(&tree, &written).unwrap_or_else(|error| panic!("{alias}: {error}"));

            assert_eq!(canonical, path, "{alias} did not reach its canonical node");
        }

        // A prefix of a node's name names no node, whatever its depth.
        for prefix in [
            &["sch"][..],
            &["cfg", "data"][..],
            &["cfg", "database", "a"][..],
        ] {
            resolve(&tree, &segments(prefix)).expect_err("a prefix is not a segment");
        }
    }

    #[test]
    fn fr_help_028_a_segment_that_names_no_child_names_the_segment_and_the_node_it_was_sought_under()
     {
        // FR-HELP-028: exit 64, a nearest-match suggestion over the children
        // of the node reached, and a cause naming both the segment and that
        // node. The suggestion is asserted to be the short one the requirement
        // asks for — `add` under `tpl cfg database`, not every node named
        // `add` anywhere.
        let tree = tree();
        let refused = resolve(&tree, &segments(&["cfg", "database", "ad"]))
            .expect_err("`ad` names no child of `tpl cfg database`");

        assert_eq!(refused.exit_code(), 64);

        let Error::UnknownCommandPathSegment {
            segment,
            node,
            nearest,
        } = &refused
        else {
            panic!("the refusal is not the one FR-HELP-028 names: {refused}");
        };

        assert_eq!(segment, "ad");
        assert_eq!(node, "cfg database");
        assert_eq!(nearest, &["add".to_owned()]);

        // At the root the node is the empty path, which the diagnostic renders
        // as `tpl` itself.
        let refused =
            resolve(&tree, &segments(&["schemas"])).expect_err("`schemas` names no top-level node");

        let Error::UnknownCommandPathSegment { node, nearest, .. } = &refused else {
            panic!("the refusal is not the one FR-HELP-028 names: {refused}");
        };

        assert!(node.is_empty());
        assert_eq!(nearest, &["schema".to_owned()]);

        // And a segment far from every child leaves the suggestion out
        // altogether, per FR-ERR-020, rather than offering a poor one.
        let refused = resolve(&tree, &segments(&["cfg", "zzzzzzzz"]))
            .expect_err("`zzzzzzzz` names no child of `tpl cfg`");

        let Error::UnknownCommandPathSegment { nearest, .. } = &refused else {
            panic!("the refusal is not the one FR-HELP-028 names: {refused}");
        };

        assert!(nearest.is_empty(), "{nearest:?}");
    }

    #[test]
    fn br_cli_001_a_mistyped_alias_is_suggested_from_the_children_of_the_node_reached() {
        // BR-CLI-001: a mistyped alias is resolved through the nearest-match
        // rule, so an alias is a candidate beside a canonical name. `tpl help
        // cfg d` has `db` one step away and `database` far off.
        let tree = tree();
        let refused =
            resolve(&tree, &segments(&["cfg", "d"])).expect_err("`d` names no child of `tpl cfg`");

        let Error::UnknownCommandPathSegment { nearest, .. } = &refused else {
            panic!("the refusal is not the one FR-HELP-028 names: {refused}");
        };

        assert!(nearest.contains(&"db".to_owned()), "{nearest:?}");
    }

    #[test]
    fn fr_help_022_every_node_of_the_tree_has_an_entry() {
        // FR-HELP-022: the table is indexed by command path and feeds both
        // consumers, so a node without an entry is a node whose help has two
        // sections missing. Comparing the whole vector rather than membership
        // asserts three things at once: every node has an entry, no entry
        // names a node the tree does not have, and the order is the one
        // FR-HELP-019 requires.
        let declared: Vec<Vec<String>> = ENTRIES.iter().map(|entry| owned(entry.path)).collect();

        assert_eq!(declared, node_paths());
    }

    #[test]
    fn fr_help_022_the_table_holds_one_entry_per_node() {
        // The array's length is written into its type, so a node added without
        // an entry fails to compile rather than shipping without help. This
        // asserts the count is the tree's: 8 top-level commands, 6 group nodes
        // — `tpl` among them — and 29 leaves.
        assert_eq!(ENTRIES.len(), 35);
        assert_eq!(entries().len(), 35);
    }

    #[test]
    fn fr_help_022_every_entry_is_reachable_by_its_path() {
        for declared in &ENTRIES {
            let found = entry(declared.path).expect("the table carries this path");

            assert_eq!(found, declared, "{}", written(declared.path));
        }

        assert_eq!(entry(&["schema", "nowhere"]), None);
    }

    #[test]
    fn fr_help_007_every_entry_carries_a_description() {
        // FR-HELP-007 makes DESCRIPTION one of the four sections that always
        // appear, so an empty one is a section the renderer cannot omit and
        // cannot fill.
        for declared in &ENTRIES {
            assert!(
                !declared.description.is_empty(),
                "{} carries no description",
                written(declared.path)
            );
        }
    }

    #[test]
    fn fr_help_012_every_entry_carries_at_least_one_example() {
        // FR-HELP-012, and the second property of BR-HELP-003.
        for declared in &ENTRIES {
            assert!(
                !declared.examples.is_empty(),
                "{} carries no example",
                written(declared.path)
            );

            for example in declared.examples {
                assert!(
                    !example.caption.is_empty(),
                    "{} carries an example with no caption",
                    written(declared.path)
                );
                assert!(
                    example.lines.iter().any(|line| !line.invocation.is_empty()),
                    "{} carries an example with no invocation",
                    written(declared.path)
                );
            }
        }
    }

    #[test]
    fn br_help_003_every_example_parses_through_the_command_parser() {
        // The third property of BR-HELP-003, and the one that makes the
        // examples worth carrying: an example that does not parse is worse
        // than no example. `parse` is the whole of step 1 of FR-ERR-006 — what
        // the parser refuses, the repetition of FR-CLI-014, and the pair of
        // FR-CLI-015 — so an example is asserted against every rule an
        // invocation meets, not merely against the tree.
        for declared in &ENTRIES {
            for example in declared.examples {
                for line in example.lines {
                    if line.invocation.is_empty() {
                        continue;
                    }

                    assert_eq!(
                        line.invocation.first(),
                        Some(&"tpl"),
                        "an example of {} does not begin with tpl",
                        written(declared.path)
                    );

                    assert!(
                        parse(line.invocation.iter().copied()).is_ok(),
                        "the example '{}' of {} does not parse",
                        line.invocation.join(" "),
                        written(declared.path)
                    );
                }
            }
        }
    }

    #[test]
    fn fr_help_014_every_see_also_reference_names_another_node() {
        // FR-HELP-014: help is self-contained, and SEE ALSO references only
        // other `tpl` commands. Checking against the tree rather than against
        // a list is what keeps a renamed node from leaving a dangling
        // reference behind.
        let nodes = node_paths();

        for declared in &ENTRIES {
            for reference in declared.see_also {
                assert!(
                    nodes.contains(&owned(reference)),
                    "{} refers to {}, which is not a node of the tree",
                    written(declared.path),
                    written(reference)
                );
                assert_ne!(
                    owned(reference),
                    owned(declared.path),
                    "{} refers to itself",
                    written(declared.path)
                );
            }
        }
    }

    #[test]
    fn fr_help_011_every_exit_code_section_is_ordered_and_carries_no_repetition() {
        for declared in &ENTRIES {
            let listed = codes(declared);
            let mut ascending = listed.clone();
            ascending.sort_unstable();
            ascending.dedup();

            assert_eq!(
                listed,
                ascending,
                "the exit codes of {} repeat or are out of order",
                written(declared.path)
            );

            for outcome in declared.exit_codes {
                assert!(
                    !outcome.meaning.is_empty(),
                    "{} lists {} with no meaning",
                    written(declared.path),
                    outcome.code.code()
                );
            }
        }
    }

    #[test]
    fn fr_err_006_every_entry_lists_success_and_usage() {
        // Step 1 of FR-ERR-006 runs for every command without exception, so
        // every node can be refused with 64; and every node has a success.
        for declared in &ENTRIES {
            let listed = codes(declared);

            assert!(listed.contains(&0), "{} omits 0", written(declared.path));
            assert!(listed.contains(&64), "{} omits 64", written(declared.path));
        }
    }

    #[test]
    fn fr_err_003_only_init_lists_cant_create() {
        // FR-ERR-003: 73 is produced only by `tpl init`.
        assert_eq!(carrying(Code::CantCreate), vec!["tpl init"]);
    }

    #[test]
    fn fr_proj_025_the_commands_that_perform_no_discovery_omit_configuration() {
        // FR-PROJ-025 exempts init, help and version from discovery and states
        // in its own accepted cost that each omits 78. A group node invoked
        // bare joins them: FR-HELP-025 makes it print its help and exit 0
        // unconditionally, so nothing is discovered that could fail. Every
        // other node performs discovery and fails with 78 when it finds none.
        let exempt = [
            "tpl",
            "tpl schema",
            "tpl template",
            "tpl cache",
            "tpl cfg",
            "tpl cfg database",
            "tpl init",
            "tpl help",
            "tpl version",
        ];

        let listing = carrying(Code::Configuration);

        for path in exempt {
            assert!(
                !listing.contains(&path.to_owned()),
                "{path} lists 78 and performs no discovery"
            );
        }

        assert_eq!(listing.len(), ENTRIES.len() - exempt.len());
    }

    #[test]
    fn fr_cache_009_only_the_commands_that_open_a_connection_list_the_server_codes() {
        // FR-CACHE-009 names the commands that read a catalogue — the eight
        // schema subcommands and render — and FR-CFG-005 adds the one cfg
        // subcommand that contacts a server. `cache load` reads the server by
        // definition. No other node can be refused by a server it never
        // reaches, so no other lists 69 or 77.
        let connecting = [
            "tpl schema info",
            "tpl schema tables",
            "tpl schema table",
            "tpl schema views",
            "tpl schema view",
            "tpl schema routines",
            "tpl schema routine",
            "tpl schema dump",
            "tpl render",
            "tpl cache load",
            "tpl cfg database test",
        ];

        assert_eq!(carrying(Code::Unavailable), connecting);
        assert_eq!(carrying(Code::NoPermission), connecting);
    }

    #[test]
    fn fr_err_001_only_the_commands_that_reach_a_template_list_data_error() {
        // The 65 row of FR-ERR-001 is template work throughout: a syntax
        // error, a render failure, a malformed --context document, the render
        // deadline, and a template path escaping the root. Only the four nodes
        // that resolve or parse a template can produce any of them.
        assert_eq!(
            carrying(Code::DataError),
            [
                "tpl template show",
                "tpl template check",
                "tpl template path",
                "tpl render",
            ]
        );
    }

    #[test]
    fn fr_err_030_only_the_root_lists_internal_error() {
        // 70 belongs to `tpl` and to no command under it, for the reason this
        // module's own documentation gives: FR-ERR-030 gives it two producing
        // conditions and neither is a condition of a command. It is therefore
        // stated once, at the root, as FR-GLOB-003 states the global flags.
        // Both halves are pinned — that the root carries it, and that nothing
        // else does — so that moving it is a deliberate act rather than an
        // accident.
        assert_eq!(carrying(Code::Software), vec!["tpl"]);

        let root = entry(&[]).expect("the table carries the root");

        assert!(codes(root).contains(&70));
    }

    #[test]
    fn fr_err_001_every_code_carries_the_sysexits_name_of_its_number() {
        // FR-ERR-001 names the codes, and the `exit` line of FR-ERR-008 writes
        // the name beside the number. The ten are asserted pair by pair so
        // that a mistyped name is a failure rather than a line of help nobody
        // re-reads.
        let table = [
            (Code::Ok, 0_u8, "EX_OK"),
            (Code::Usage, 64, "EX_USAGE"),
            (Code::DataError, 65, "EX_DATAERR"),
            (Code::NoInput, 66, "EX_NOINPUT"),
            (Code::Unavailable, 69, "EX_UNAVAILABLE"),
            (Code::Software, 70, "EX_SOFTWARE"),
            (Code::CantCreate, 73, "EX_CANTCREAT"),
            (Code::IoError, 74, "EX_IOERR"),
            (Code::NoPermission, 77, "EX_NOPERM"),
            (Code::Configuration, 78, "EX_CONFIG"),
        ];

        for (code, number, name) in table {
            assert_eq!(code.code(), number);
            assert_eq!(code.name(), name);
        }
    }
}
