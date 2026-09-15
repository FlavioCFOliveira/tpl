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
//! [`render`] composes the seven sections of `FR-HELP-006` from this table and
//! from the parser tree, and [`text`] is the whole of the route to it. The JSON
//! command tree of `FR-HELP-016` is the second consumer and is a later task;
//! [`entries`] is the entry point it reads the table through, in the order
//! `FR-HELP-019` requires.

mod render;

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
// The JSON command tree of `FR-HELP-016` is the consumer, and it is a later
// task. The suppression is this one item's: every other item of the module is
// read by the renderer, so a member that stops being read is still reported.
#[allow(
    dead_code,
    reason = "the JSON command tree of FR-HELP-016 is a later task, and it is what reads the \
              table whole; the text renderer reaches one entry at a time"
)]
pub(crate) const fn entries() -> &'static [Entry] {
    &ENTRIES
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
    use super::{Code, ENTRIES, Entry, entries, entry};
    use crate::cli::{parse, tree};

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
    fn every_node_of_the_tree_has_an_entry() {
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
    fn the_table_holds_one_entry_per_node() {
        // The array's length is written into its type, so a node added without
        // an entry fails to compile rather than shipping without help. This
        // asserts the count is the tree's: 8 top-level commands, 6 group nodes
        // — `tpl` among them — and 29 leaves.
        assert_eq!(ENTRIES.len(), 35);
        assert_eq!(entries().len(), 35);
    }

    #[test]
    fn every_entry_is_reachable_by_its_path() {
        for declared in &ENTRIES {
            let found = entry(declared.path).expect("the table carries this path");

            assert_eq!(found, declared, "{}", written(declared.path));
        }

        assert_eq!(entry(&["schema", "nowhere"]), None);
    }

    #[test]
    fn every_entry_carries_a_description() {
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
    fn every_entry_carries_at_least_one_example() {
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
    fn every_example_parses_through_the_command_parser() {
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
    fn every_see_also_reference_names_another_node() {
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
    fn every_exit_code_section_is_ordered_and_carries_no_repetition() {
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
    fn every_entry_lists_success_and_usage() {
        // Step 1 of FR-ERR-006 runs for every command without exception, so
        // every node can be refused with 64; and every node has a success.
        for declared in &ENTRIES {
            let listed = codes(declared);

            assert!(listed.contains(&0), "{} omits 0", written(declared.path));
            assert!(listed.contains(&64), "{} omits 64", written(declared.path));
        }
    }

    #[test]
    fn only_init_lists_cant_create() {
        // FR-ERR-003: 73 is produced only by `tpl init`.
        assert_eq!(carrying(Code::CantCreate), vec!["tpl init"]);
    }

    #[test]
    fn the_commands_that_perform_no_discovery_omit_configuration() {
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
    fn only_the_commands_that_open_a_connection_list_the_server_codes() {
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
    fn only_the_commands_that_reach_a_template_list_data_error() {
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
    fn only_the_root_lists_internal_error() {
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
    fn every_code_carries_the_sysexits_name_of_its_number() {
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
