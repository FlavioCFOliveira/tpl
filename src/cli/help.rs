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
//! An entry carries the node's `DESCRIPTION` — a first paragraph, the
//! [`Block`]s after it, and on a leaf the four statements of `FR-HELP-031` —
//! its `EXAMPLES`, its `EXIT CODES` and its `SEE ALSO` references, and nothing
//! else. The template surface `FR-HELP-033` places in the `DESCRIPTION` of
//! `tpl render` is a block that names its source rather than a copy of it: the
//! item values of `FR-ENV-047` and the variables of `FR-HELP-032` live in
//! [`surface`], the part of this table indexed by name. Four things are
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
mod surface;

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

    /// The first paragraph of the `DESCRIPTION` section: what the command is
    /// and what it does.
    ///
    /// One unwrapped paragraph. The renderer wraps it.
    pub(crate) description: &'static str,

    /// What the `DESCRIPTION` section states after its first paragraph, in
    /// order: further paragraphs, tables of rows, and, on `tpl render` alone,
    /// the template surface of `FR-HELP-033`. Empty where the first paragraph
    /// is the whole of it.
    pub(crate) blocks: &'static [Block],

    /// The four statements of `FR-HELP-031` that end the `DESCRIPTION` of a
    /// leaf, or [`None`] on a node that has children.
    pub(crate) touches: Option<Touches>,

    /// The `EXAMPLES` section. Never empty, per `FR-HELP-012`.
    pub(crate) examples: &'static [Example],

    /// The `EXIT CODES` section, ascending, listing only what this command can
    /// produce, per `FR-HELP-011`.
    pub(crate) exit_codes: &'static [Outcome],

    /// The `SEE ALSO` section: the path of another node of this tree, per
    /// `FR-HELP-014`.
    pub(crate) see_also: &'static [&'static [&'static str]],
}

impl Entry {
    /// The `DESCRIPTION` as the JSON document carries it: the paragraphs of
    /// the text help, unwrapped, separated by a blank line.
    ///
    /// A table of rows is one paragraph, its heading and each row on a line of
    /// its own. The template surface of `FR-HELP-033` is left out: the
    /// document carries it as data, in `data.template_surface` and
    /// `data.context_variables`, which every form of the document holds
    /// whatever path reduces it, per `FR-HELP-017`. The four statements of
    /// `FR-HELP-031` end it, as they end the text.
    pub(crate) fn written_description(&self) -> String {
        let mut paragraphs: Vec<String> = Vec::with_capacity(self.blocks.len() + 2);
        paragraphs.push(self.description.to_owned());

        for block in self.blocks {
            match block {
                Block::Prose(text) => paragraphs.push((*text).to_owned()),
                Block::Rows { heading, rows } => {
                    let mut written = (*heading).to_owned();

                    for row in *rows {
                        written.push('\n');
                        written.push_str(row.label);
                        written.push_str(": ");
                        written.push_str(row.text);
                    }

                    paragraphs.push(written);
                }
                Block::Surface => {}
            }
        }

        if let Some(touches) = &self.touches {
            paragraphs.push(touches.sentence());
        }

        paragraphs.join("\n\n")
    }
}

/// One part of a `DESCRIPTION` after its first paragraph.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Block {
    /// A paragraph, unwrapped.
    Prose(&'static str),

    /// A heading line followed by rows of two columns: a label, and what it
    /// means.
    Rows {
        /// The line above the rows.
        heading: &'static str,

        /// The rows, in the order they are printed.
        rows: &'static [Row],
    },

    /// The context variables and the filters, tests and functions of
    /// `FR-HELP-033`, rendered from [`surface`].
    Surface,
}

/// One row of a [`Block::Rows`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct Row {
    /// The left column.
    pub(crate) label: &'static str,

    /// The right column, in one unwrapped line.
    pub(crate) text: &'static str,
}

/// The four statements of `FR-HELP-031`, each one sentence, in its order.
///
/// Four fields rather than one paragraph so that none can be left out: a
/// statement whose answer is no is still a statement, and the type makes an
/// entry that omits one fail to build.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct Touches {
    /// Whether the command connects to a database server.
    pub(crate) server: &'static str,

    /// Whether it needs a `[database.<name>]` entry of `.tpl/.cfg`.
    pub(crate) entry: &'static str,

    /// Which files it writes, if any.
    pub(crate) files: &'static str,

    /// What it writes to stdout.
    pub(crate) stdout: &'static str,
}

impl Touches {
    /// The four statements, in their order, as one paragraph.
    pub(crate) fn sentence(&self) -> String {
        [self.server, self.entry, self.files, self.stdout].join(" ")
    }
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
    purpose: "Keeps only the objects whose name matches this LIKE pattern, in which % matches any \
              run of characters and _ matches exactly one.",
    excludes: &[],
};
/// `--format`, as every node that declares it states it.
const FORMAT: Documented = Documented {
    name: "--format",
    purpose: "Chooses the output: text is aligned columns for people and its layout may change; \
              json is a stable document, so use it whenever a program reads the output.",
    excludes: &[],
};
/// `--format` on `tpl cfg list`, whose text output is the file itself.
const FORMAT_CFG_LIST: Documented = Documented {
    name: "--format",
    purpose: "Chooses the output: text is the file itself, comments included, with passwords \
              redacted; json is a stable document, so use it whenever a program reads the output.",
    excludes: &[],
};
/// `--format` on `tpl cfg get`, whose text output is the value alone.
const FORMAT_CFG_GET: Documented = Documented {
    name: "--format",
    purpose: "Chooses the output: text is the bare value; json is a stable document, so use it \
              whenever a program reads the output.",
    excludes: &[],
};
/// `--format` on `tpl template path`, whose text output is a path alone.
const FORMAT_TEMPLATE_PATH: Documented = Documented {
    name: "--format",
    purpose: "Chooses the output: text is the path alone; json is a stable document, so use it \
              whenever a program reads the output.",
    excludes: &[],
};
/// `--format` on `tpl help`, whose text output is the help itself.
const FORMAT_HELP: Documented = Documented {
    name: "--format",
    purpose: "Chooses the output: text is the help as written here; json is the same help as a \
              stable document, so use it whenever a program reads the output.",
    excludes: &[],
};
/// `--pretty`, as every node that declares it states it.
const PRETTY: Documented = Documented {
    name: "--pretty",
    purpose: "Indents the JSON output by two spaces, one key per line, instead of writing it on \
              one line.",
    excludes: &[],
};
/// `--direct`, as every node that declares it states it.
const DIRECT: Documented = Documented {
    name: "--direct",
    purpose: "Reads from the server even when the cache holds the data, and replaces the cached \
              copy with what it read; add --no-cache to leave the cache untouched.",
    excludes: &[],
};
/// `--no-cache`, as every node that declares it states it.
const NO_CACHE: Documented = Documented {
    name: "--no-cache",
    purpose: "Stores nothing in the cache from this invocation; data the cache already holds is \
              still used.",
    excludes: &[],
};
/// `--direct` on `tpl render`, which `FR-RND-041` refuses with `--context`.
const RENDER_DIRECT: Documented = Documented {
    name: "--direct",
    purpose: "Reads from the server even when the cache holds the data, and replaces the cached \
              copy with what it read; add --no-cache to leave the cache untouched.",
    excludes: &["--context"],
};
/// `--no-cache` on `tpl render`, which `FR-CACHE-018` accepts and ignores
/// with `--context`.
const RENDER_NO_CACHE: Documented = Documented {
    name: "--no-cache",
    purpose: "Stores nothing in the cache from this invocation; data the cache already holds is \
              still used. It has no effect with --context, which uses no cache.",
    excludes: &[],
};
/// `--set`, as every node that declares it states it.
const SET: Documented = Documented {
    name: "--set",
    purpose: "Adds a template variable: --set title=Orders makes vars.title the string \"Orders\"; \
              a key is letters, digits and _, not starting with a digit, and each key may be given \
              once.",
    excludes: &[],
};
/// `--context`, as every node that declares it states it.
const CONTEXT: Documented = Documented {
    name: "--context",
    purpose: "Reads the data from this JSON file, written by tpl schema dump, or from stdin when \
              the path is -, so that no server is contacted and no database entry is used.",
    excludes: &["--database", "--direct"],
};
/// `--dsn`, as every node that declares it states it.
const DSN: Documented = Documented {
    name: "--dsn",
    purpose: "Sets the whole connection as one URL, mysql://user:password@host:port/database, \
              instead of --host, --port, --user and --schema.",
    excludes: &["--host", "--port", "--user", "--schema"],
};
/// `--host`, as every node that declares it states it.
const HOST: Documented = Documented {
    name: "--host",
    purpose: "Sets the host name or IP address of the MariaDB server.",
    excludes: &["--dsn"],
};
/// `--port`, as every node that declares it states it.
const PORT: Documented = Documented {
    name: "--port",
    purpose: "Sets the TCP port of the MariaDB server, from 1 to 65535.",
    excludes: &["--dsn"],
};
/// `--user`, as every node that declares it states it.
const USER: Documented = Documented {
    name: "--user",
    purpose: "Sets the user name tpl logs in to the server with.",
    excludes: &["--dsn"],
};
/// `--schema`, as every node that declares it states it.
const SCHEMA: Documented = Documented {
    name: "--schema",
    purpose: "Sets the name of the database on the MariaDB server, stored as \
              database.<entry>.database; this is not the entry name that -d selects.",
    excludes: &["--dsn"],
};
/// `--tls`, as every node that declares it states it.
const TLS: Documented = Documented {
    name: "--tls",
    purpose: "Sets how the connection is secured: disabled (no TLS), preferred (TLS when the \
              server offers it), required (TLS, certificate not checked), verify-ca (certificate \
              chain checked) or verify-identity (chain and host name checked); a local server \
              without a trusted certificate needs disabled or preferred.",
    excludes: &[],
};
/// `--password-command`, as every node that declares it states it.
const PASSWORD_COMMAND: Documented = Documented {
    name: "--password-command",
    purpose: "Sets a command whose standard output is the password. Write it as one string, \
              a command line such as \"pass db/shop\", never as an array. tpl splits the string \
              into words as a shell would (quotes group words, and every quote must be closed) \
              and stores them in the file as an array, [\"pass\", \"db/shop\"]. A value that \
              starts with an unquoted [, or ends in a backslash outside quotes, is refused. The \
              command runs without a shell, and no password is stored in the file.",
    excludes: &[],
};
/// `--ca-file`, as every node that declares it states it.
const CA_FILE: Documented = Documented {
    name: "--ca-file",
    purpose: "Names a file of CA certificates that verify-ca and verify-identity trust in addition \
              to the certificates the system already trusts. The path is read as written: \
              ${VAR} is not expanded in it, and a value containing ${ is refused.",
    excludes: &[],
};
/// `--ca-path`, as every node that declares it states it.
const CA_PATH: Documented = Documented {
    name: "--ca-path",
    purpose: "Names a directory of CA certificate files that verify-ca and verify-identity trust \
              in addition to the certificates the system already trusts. The path is read as \
              written: ${VAR} is not expanded in it, and a value containing ${ is refused.",
    excludes: &[],
};

// `tpl cache load` declares `--direct` and `--no-cache` because
// `FR-CACHE-017` requires it, and gives both a meaning of its own: it accepts
// and ignores the first, per `FR-CACHE-018`, and refuses the second, per
// `FR-CACHE-019`. The shared sentences would be false of it, so it carries two
// rows of its own.

/// `--direct`, as `tpl cache load` states it.
const CACHE_LOAD_DIRECT: Documented = Documented {
    name: "--direct",
    purpose: "Accepted and has no effect: this command always reads from the server.",
    excludes: &[],
};
/// `--no-cache`, as `tpl cache load` states it.
const CACHE_LOAD_NO_CACHE: Documented = Documented {
    name: "--no-cache",
    purpose: "Refused with exit 64: this command exists to store what it reads.",
    excludes: &[],
};

// The three object flags of `FR-RND-003` and `FR-CACHE-024`. `FR-RND-005`
// refuses more than one kind in one invocation, which is the exclusion each of
// the three states about the other two.

/// `--table`, the object flag.
const OBJECT_TABLE: Documented = Documented {
    name: "--table",
    purpose: "Binds this table as the variable table; database still holds every table.",
    excludes: &["--view", "--routine"],
};
/// `--view`, the object flag.
const OBJECT_VIEW: Documented = Documented {
    name: "--view",
    purpose: "Binds this view as the variable view; database still holds every view.",
    excludes: &["--table", "--routine"],
};
/// `--routine`, the object flag.
const OBJECT_ROUTINE: Documented = Documented {
    name: "--routine",
    purpose: "Binds this routine, named bare or as procedure:<name> or function:<name>, as the \
              variable routine; database still holds every routine.",
    excludes: &["--table", "--view"],
};

// The same three flags on `tpl cache load` and `tpl cache clean`, where no
// template sees anything: their purposes say what each command does with the
// object.

/// `--table` on `tpl cache load`.
const LOAD_TABLE: Documented = Documented {
    name: "--table",
    purpose: "Loads only this table into the cache.",
    excludes: &["--view", "--routine"],
};
/// `--view` on `tpl cache load`.
const LOAD_VIEW: Documented = Documented {
    name: "--view",
    purpose: "Loads only this view into the cache.",
    excludes: &["--table", "--routine"],
};
/// `--routine` on `tpl cache load`.
const LOAD_ROUTINE: Documented = Documented {
    name: "--routine",
    purpose: "Loads only this routine into the cache, named bare or as procedure:<name> or \
              function:<name>.",
    excludes: &["--table", "--view"],
};
/// `--table` on `tpl cache clean`.
const CLEAN_TABLE: Documented = Documented {
    name: "--table",
    purpose: "Deletes only this table's cached copy.",
    excludes: &["--view", "--routine"],
};
/// `--view` on `tpl cache clean`.
const CLEAN_VIEW: Documented = Documented {
    name: "--view",
    purpose: "Deletes only this view's cached copy.",
    excludes: &["--table", "--routine"],
};
/// `--routine` on `tpl cache clean`.
const CLEAN_ROUTINE: Documented = Documented {
    name: "--routine",
    purpose: "Deletes only this routine's cached copy, named bare or as procedure:<name> or \
              function:<name>.",
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
    purpose: "Names the routine to describe, bare or as procedure:<name> or function:<name>.",
    excludes: &[],
};
/// `NAME`, as the node that declares it states it.
const TEMPLATE_SHOW_NAME: Documented = Documented {
    name: "NAME",
    purpose: "Names the template to print, with or without its .jinja extension.",
    excludes: &[],
};
/// `NAME`, as the node that declares it states it.
const TEMPLATE_CHECK_NAME: Documented = Documented {
    name: "NAME",
    purpose: "Names a template to check; with none, every template of the project is checked.",
    excludes: &[],
};
/// `NAME`, as the node that declares it states it.
const TEMPLATE_PATH_NAME: Documented = Documented {
    name: "NAME",
    purpose: "Names the template to locate; with none, the path of .tpl/templates/ is printed.",
    excludes: &[],
};
/// `TEMPLATE`, as the node that declares it states it.
const RENDER_TEMPLATE: Documented = Documented {
    name: "TEMPLATE",
    purpose: "Names the template to render, as tpl template list prints it, such as rust/struct.",
    excludes: &[],
};
/// `KEY`, as the node that declares it states it.
const CFG_GET_KEY: Documented = Documented {
    name: "KEY",
    purpose: "Names the key to print, such as core.database or database.shop.host.",
    excludes: &[],
};
/// `KEY`, as the node that declares it states it.
const CFG_SET_KEY: Documented = Documented {
    name: "KEY",
    purpose: "Names the key to write, one of the keys listed under DESCRIPTION.",
    excludes: &[],
};
/// `VALUE`, as the node that declares it states it.
const CFG_SET_VALUE: Documented = Documented {
    name: "VALUE",
    purpose: "Supplies the value to write, of the type its key takes.",
    excludes: &[],
};
/// `KEY`, as the node that declares it states it.
const CFG_UNSET_KEY: Documented = Documented {
    name: "KEY",
    purpose: "Names the key or the block to remove, such as database.shop.port or database.shop.",
    excludes: &[],
};
/// `NAME`, as the node that declares it states it.
const ENTRY_ADD_NAME: Documented = Documented {
    name: "NAME",
    purpose: "Names the new database entry; -d NAME selects it afterwards.",
    excludes: &[],
};
/// `NAME`, as the node that declares it states it.
const ENTRY_SHOW_NAME: Documented = Documented {
    name: "NAME",
    purpose: "Names the database entry to print.",
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
    purpose: "Names the database entry to connect with.",
    excludes: &[],
};
/// `PATH`, as the node that declares it states it.
const INIT_PATH: Documented = Documented {
    name: "PATH",
    purpose: "Names the directory to create the project in; with none, the current directory is \
              used.",
    excludes: &[],
};
/// `COMMAND_PATH`, as the node that declares it states it.
const HELP_COMMAND_PATH: Documented = Documented {
    name: "COMMAND_PATH",
    purpose: "Names the command to describe, one word per token, such as cfg database add.",
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
        purpose: "Selects the database entry to use, a named connection in .tpl/.cfg that tpl cfg \
                  database list lists; it overrides core.database.",
        excludes: &[],
    },
    Documented {
        name: "--tpl-dir",
        purpose: "Uses this .tpl folder as the project, instead of searching the current directory \
                  and its parents for one.",
        excludes: &[],
    },
    Documented {
        name: "--timeout",
        purpose: "Fails the command if it runs longer than this many seconds in total; the \
                  per-step limits of .tpl/.cfg still apply.",
        excludes: &[],
    },
    Documented {
        name: "--verbose",
        purpose: "Writes more diagnostic detail to stderr, one level per occurrence up to three: \
                  -v, -vv or -vvv. A fourth -v, or more, changes nothing.",
        excludes: &["--quiet"],
    },
    Documented {
        name: "--quiet",
        purpose: "Writes only errors to stderr, suppressing warnings.",
        excludes: &["--verbose"],
    },
    Documented {
        name: "--help",
        purpose: "Prints the help of the command it is given to, and does nothing else.",
        excludes: &[],
    },
    Documented {
        name: "--version",
        purpose: "Prints tpl and its version, and does nothing else.",
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
    (
        &["template", "path"],
        &[TEMPLATE_PATH_NAME, FORMAT_TEMPLATE_PATH, PRETTY],
    ),
    (
        &["render"],
        &[
            RENDER_TEMPLATE,
            OBJECT_TABLE,
            OBJECT_VIEW,
            OBJECT_ROUTINE,
            SET,
            CONTEXT,
            RENDER_DIRECT,
            RENDER_NO_CACHE,
        ],
    ),
    (
        &["cache", "load"],
        &[
            LOAD_TABLE,
            LOAD_VIEW,
            LOAD_ROUTINE,
            CACHE_LOAD_DIRECT,
            CACHE_LOAD_NO_CACHE,
        ],
    ),
    (
        &["cache", "clean"],
        &[CLEAN_TABLE, CLEAN_VIEW, CLEAN_ROUTINE],
    ),
    (&["cache", "status"], &[FORMAT, PRETTY]),
    (&["cfg", "get"], &[CFG_GET_KEY, FORMAT_CFG_GET, PRETTY]),
    (&["cfg", "set"], &[CFG_SET_KEY, CFG_SET_VALUE]),
    (&["cfg", "unset"], &[CFG_UNSET_KEY]),
    (&["cfg", "list"], &[FORMAT_CFG_LIST, PRETTY]),
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
    (&["help"], &[HELP_COMMAND_PATH, FORMAT_HELP, PRETTY]),
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

/// The defaults the configuration applies to a key that a flag leaves
/// unwritten, by command path and flag.
///
/// `tpl cfg database add` writes no key for a flag that is not given, and the
/// configuration then gives the absent key its default: `port` is `3306` and
/// `tls` is `verify-identity`. The parser declares no default for either flag,
/// because it supplies none, so the fact is stated here, for the text help and
/// the JSON document alike. `tpl cfg database update` is absent on purpose:
/// a flag it is not given leaves the field as it was.
const IMPLIED: [(&[&str], &str, &str); 2] = [
    (&["cfg", "database", "add"], "--port", "3306"),
    (&["cfg", "database", "add"], "--tls", "verify-identity"),
];

/// The default the configuration gives the key the flag `name` of the node
/// `path` writes, where the command leaves it unwritten, or [`None`].
pub(crate) fn implied(path: &[&str], name: &str) -> Option<&'static str> {
    IMPLIED
        .iter()
        .find(|(declared, flag, _)| *declared == path && *flag == name)
        .map(|(_, _, value)| *value)
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
    outcome(Code::Ok, "This help was written to stdout."),
    outcome(
        Code::Usage,
        "An unknown command or flag, or a flag given twice. On any command, also -v with -q, or a \
         global flag given a value it does not take, such as --timeout 0.",
    ),
    outcome(Code::IoError, "stdout could not be written."),
];

/// The outcomes of a bare `tpl`, which are [`GROUP`]'s and one more.
///
/// `70` is stated here and nowhere else, on the precedent `FR-GLOB-003` sets
/// for a fact that belongs to the tool rather than to each command under it.
const ROOT: &[Outcome] = &[
    outcome(Code::Ok, "This help was written to stdout."),
    outcome(
        Code::Usage,
        "An unknown command or flag, or a flag given twice. On any command, also -v with -q, or a \
         global flag given a value it does not take, such as --timeout 0.",
    ),
    outcome(
        Code::Software,
        "A bug in tpl, which any command can hit. Report it with the command and the output of tpl \
         version.",
    ),
    outcome(Code::IoError, "stdout could not be written."),
];

/// The outcomes of a `schema` subcommand that names no object.
const SCHEMA_LISTING: &[Outcome] = &[
    outcome(Code::Ok, "The result was written to stdout."),
    outcome(
        Code::Usage,
        "An unknown flag, a flag given twice, or --pretty without --format json. On any command, \
         also -v with -q, or a global flag given a value it does not take, such as --timeout 0.",
    ),
    outcome(
        Code::NoInput,
        "The entry that -d/--database names is not in .tpl/.cfg.",
    ),
    outcome(
        Code::Unavailable,
        "The server could not be reached, or a connection step took longer than its time limit.",
    ),
    outcome(
        Code::IoError,
        "A file under .tpl, or the entry's ca_file or ca_path, could not be read or written, or \
         stdout could not be written.",
    ),
    outcome(
        Code::NoPermission,
        "The server refused the login, or the database user is not allowed to read the database \
         structure.",
    ),
    outcome(
        Code::Configuration,
        "No project was found, .tpl/.cfg cannot be used, no database entry is selected (give -d or \
         set core.database), the entry is incomplete (no host, or no server database name) or its \
         password cannot be obtained (a ${VAR} is undefined, or password_command failed), \
         ca_path holds no certificate, or the server is not a supported MariaDB version.",
    ),
];

/// The outcomes of `schema table` and `schema view`, which name one object.
const SCHEMA_OBJECT: &[Outcome] = &[
    outcome(Code::Ok, "The description was written to stdout."),
    outcome(
        Code::Usage,
        "An unknown flag, a flag given twice, a missing NAME, or --pretty without --format json. \
         On any command, also -v with -q, or a global flag given a value it does not take, such \
         as --timeout 0.",
    ),
    outcome(
        Code::NoInput,
        "NAME does not exist in the selected database, or the entry that -d/--database names is \
         not in .tpl/.cfg.",
    ),
    outcome(
        Code::Unavailable,
        "The server could not be reached, or a connection step took longer than its time limit.",
    ),
    outcome(
        Code::IoError,
        "A file under .tpl, or the entry's ca_file or ca_path, could not be read or written, or \
         stdout could not be written.",
    ),
    outcome(
        Code::NoPermission,
        "The server refused the login, or the database user is not allowed to read the database \
         structure.",
    ),
    outcome(
        Code::Configuration,
        "No project was found, .tpl/.cfg cannot be used, no database entry is selected (give -d or \
         set core.database), the entry is incomplete (no host, or no server database name) or its \
         password cannot be obtained (a ${VAR} is undefined, or password_command failed), \
         ca_path holds no certificate, or the server is not a supported MariaDB version.",
    ),
];

/// The outcomes of `schema routine`, which adds the ambiguity of `FR-SCH-010`.
const SCHEMA_ROUTINE: &[Outcome] = &[
    outcome(Code::Ok, "The description was written to stdout."),
    outcome(
        Code::Usage,
        "An unknown flag, a flag given twice, a missing NAME, --pretty without --format json, or \
         a bare NAME that matches both a procedure and a function. On any command, also -v with \
         -q, or a global flag given a value it does not take, such as --timeout 0.",
    ),
    outcome(
        Code::NoInput,
        "NAME names no routine of the selected database, or the entry that -d/--database names is \
         not in .tpl/.cfg.",
    ),
    outcome(
        Code::Unavailable,
        "The server could not be reached, or a connection step took longer than its time limit.",
    ),
    outcome(
        Code::IoError,
        "A file under .tpl, or the entry's ca_file or ca_path, could not be read or written, or \
         stdout could not be written.",
    ),
    outcome(
        Code::NoPermission,
        "The server refused the login, or the database user is not allowed to read the database \
         structure.",
    ),
    outcome(
        Code::Configuration,
        "No project was found, .tpl/.cfg cannot be used, no database entry is selected (give -d or \
         set core.database), the entry is incomplete (no host, or no server database name) or its \
         password cannot be obtained (a ${VAR} is undefined, or password_command failed), \
         ca_path holds no certificate, or the server is not a supported MariaDB version.",
    ),
];

/// The outcomes of `schema dump`, which declares neither --format nor --pattern.
const SCHEMA_DUMP: &[Outcome] = &[
    outcome(Code::Ok, "The document was written to stdout."),
    outcome(
        Code::Usage,
        "An unknown flag or a flag given twice; --format and --pattern are not flags of this \
         command. On any command, also -v with -q, or a global flag given a value it does not \
         take, such as --timeout 0.",
    ),
    outcome(
        Code::NoInput,
        "The entry that -d/--database names is not in .tpl/.cfg.",
    ),
    outcome(
        Code::Unavailable,
        "The server could not be reached, or a connection step took longer than its time limit.",
    ),
    outcome(
        Code::IoError,
        "A file under .tpl, or the entry's ca_file or ca_path, could not be read or written, or \
         stdout could not be written.",
    ),
    outcome(
        Code::NoPermission,
        "The server refused the login, or the database user is not allowed to read the database \
         structure.",
    ),
    outcome(
        Code::Configuration,
        "No project was found, .tpl/.cfg cannot be used, no database entry is selected (give -d or \
         set core.database), the entry is incomplete (no host, or no server database name) or its \
         password cannot be obtained (a ${VAR} is undefined, or password_command failed), \
         ca_path holds no certificate, or the server is not a supported MariaDB version.",
    ),
];

/// The outcomes of `tpl render`.
const RENDER: &[Outcome] = &[
    outcome(Code::Ok, "The rendered text was written to stdout."),
    outcome(
        Code::Usage,
        "An unknown flag, a missing TEMPLATE, or a flag given twice; more than one of --table, \
         --view and --routine; a bare --routine NAME that names both a procedure and a function; \
         a --set without =, with an invalid key, or with a key given twice; or --context together \
         with -d/--database or --direct. On any command, also -v with -q, or a global flag given \
         a value it does not take, such as --timeout 0.",
    ),
    outcome(
        Code::DataError,
        "The template has a syntax error; the render failed, for example on an undefined variable, \
         a filter given the wrong type, or fail(); the --context document is malformed; or a \
         render limit was reached: core.render_timeout, core.render_fuel, core.render_output_limit \
         or core.render_memory_limit; or TEMPLATE resolves to a path outside .tpl/templates/.",
    ),
    outcome(
        Code::NoInput,
        "TEMPLATE does not exist, the object that --table, --view or --routine names is not in the \
         database or the --context document, or the entry that -d/--database names is not in \
         .tpl/.cfg.",
    ),
    outcome(
        Code::Unavailable,
        "The server could not be reached, or a connection step took longer than its time limit.",
    ),
    outcome(
        Code::IoError,
        "A file under .tpl, the --context file, or the entry's ca_file or ca_path could not be \
         read or written, or stdout could not be written.",
    ),
    outcome(
        Code::NoPermission,
        "The server refused the login, or the database user is not allowed to read the database \
         structure.",
    ),
    outcome(
        Code::Configuration,
        "No project was found, .tpl/.cfg cannot be used, no database entry is selected (give -d or \
         set core.database), the entry is incomplete (no host, or no server database name) or its \
         password cannot be obtained (a ${VAR} is undefined, or password_command failed), \
         ca_path holds no certificate, or the server is not a supported MariaDB version.",
    ),
];

/// The outcomes of `template list`, which names no template.
const TEMPLATE_LISTING: &[Outcome] = &[
    outcome(
        Code::Ok,
        "The list was written to stdout. A project with no template lists nothing and still \
         succeeds.",
    ),
    outcome(
        Code::Usage,
        "An unknown flag, a flag given twice, or --pretty without --format json. On any command, \
         also -v with -q, or a global flag given a value it does not take, such as --timeout 0.",
    ),
    outcome(
        Code::IoError,
        "Reading .tpl failed, or stdout could not be written.",
    ),
    outcome(
        Code::Configuration,
        "No project was found, or .tpl/.cfg cannot be used.",
    ),
];

/// The outcomes of `template show`, which resolves a name.
const TEMPLATE_NAMED: &[Outcome] = &[
    outcome(Code::Ok, "The source was written to stdout."),
    outcome(
        Code::Usage,
        "An unknown flag, a flag given twice, a missing NAME, or more than one NAME. On any \
         command, also -v with -q, or a global flag given a value it does not take, such as \
         --timeout 0.",
    ),
    outcome(
        Code::DataError,
        "NAME resolves to a path outside .tpl/templates/.",
    ),
    outcome(
        Code::NoInput,
        "NAME names no template of the project; the closest template name is suggested when one is \
         close.",
    ),
    outcome(
        Code::IoError,
        "Reading .tpl failed, or stdout could not be written.",
    ),
    outcome(
        Code::Configuration,
        "No project was found, or .tpl/.cfg cannot be used.",
    ),
];

/// The outcomes of `template path`, whose NAME is optional.
const TEMPLATE_PATH: &[Outcome] = &[
    outcome(Code::Ok, "The path was written to stdout."),
    outcome(
        Code::Usage,
        "An unknown flag, a flag given twice, more than one NAME, or --pretty without --format \
         json. On any command, also -v with -q, or a global flag given a value it does not take, \
         such as --timeout 0.",
    ),
    outcome(
        Code::DataError,
        "NAME resolves to a path outside .tpl/templates/.",
    ),
    outcome(
        Code::NoInput,
        "NAME names no template of the project; the closest template name is suggested when one is \
         close.",
    ),
    outcome(
        Code::IoError,
        "Reading .tpl failed, or stdout could not be written.",
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
        "Every checked template parses. A project with no template checks nothing and still \
         succeeds.",
    ),
    outcome(
        Code::Usage,
        "An unknown flag or a flag given twice. On any command, also -v with -q, or a global flag \
         given a value it does not take, such as --timeout 0.",
    ),
    outcome(
        Code::DataError,
        "A checked template has a syntax error; every failing template is reported, one \
         message each with its line and column. Or a NAME resolves to a path outside \
         .tpl/templates/.",
    ),
    outcome(
        Code::NoInput,
        "A NAME names no template of the project; the closest template name is suggested when one \
         is close.",
    ),
    outcome(Code::IoError, "Reading .tpl failed."),
    outcome(
        Code::Configuration,
        "No project was found, or .tpl/.cfg cannot be used.",
    ),
];

/// The outcomes of `cache load`, the one cache subcommand that reads a server.
const CACHE_LOAD: &[Outcome] = &[
    outcome(
        Code::Ok,
        "The data was read from the server and stored in .tpl/.cache/.",
    ),
    outcome(
        Code::Usage,
        "An unknown flag, or a flag given twice; --no-cache; more than one of --table, --view and \
         --routine; or a bare --routine name that matches both a procedure and a function. On any \
         command, also -v with -q, or a global flag given a value it does not take, such as \
         --timeout 0.",
    ),
    outcome(
        Code::NoInput,
        "The object named does not exist in the selected database, or the entry that -d/--database \
         names is not in .tpl/.cfg.",
    ),
    outcome(
        Code::Unavailable,
        "The server could not be reached, or a connection step took longer than its time limit. \
         What was already stored is left unchanged.",
    ),
    outcome(
        Code::IoError,
        "A file under .tpl, or the entry's ca_file or ca_path, could not be read or written.",
    ),
    outcome(
        Code::NoPermission,
        "The server refused the login, or the database user is not allowed to read the database \
         structure.",
    ),
    outcome(
        Code::Configuration,
        "No project was found, .tpl/.cfg cannot be used, no database entry is selected (give -d or \
         set core.database), the entry is incomplete (no host, or no server database name) or its \
         password cannot be obtained (a ${VAR} is undefined, or password_command failed), \
         ca_path holds no certificate, or the server is not a supported MariaDB version.",
    ),
];

/// The outcomes of `cache clean`, which reads no server.
const CACHE_CLEAN: &[Outcome] = &[
    outcome(Code::Ok, "The cached data was deleted from .tpl/.cache/."),
    outcome(
        Code::Usage,
        "An unknown flag (--direct and --no-cache included), a flag given twice, or more than one \
         of --table, --view and --routine. On any command, also -v with -q, or a global flag \
         given a value it does not take, such as --timeout 0.",
    ),
    outcome(
        Code::NoInput,
        "The entry that -d/--database names is not in .tpl/.cfg.",
    ),
    outcome(
        Code::IoError,
        "Reading .tpl failed, or a file under .tpl/.cache/ could not be deleted.",
    ),
    outcome(
        Code::Configuration,
        "No project was found, .tpl/.cfg cannot be used, or no database entry is selected (give -d \
         or set core.database).",
    ),
];

/// The outcomes of `cache status`, which reads no server.
const CACHE_STATUS: &[Outcome] = &[
    outcome(
        Code::Ok,
        "The report was written to stdout. An empty cache is reported, not refused.",
    ),
    outcome(
        Code::Usage,
        "An unknown flag (--direct and --no-cache included), a flag given twice, or --pretty \
         without --format json. On any command, also -v with -q, or a global flag given a value \
         it does not take, such as --timeout 0.",
    ),
    outcome(
        Code::NoInput,
        "The entry that -d/--database names is not in .tpl/.cfg.",
    ),
    outcome(
        Code::IoError,
        "Reading .tpl failed, or stdout could not be written.",
    ),
    outcome(
        Code::Configuration,
        "No project was found, .tpl/.cfg cannot be used, or no database entry is selected (give -d \
         or set core.database).",
    ),
];

/// The outcomes of `cfg get`.
const CFG_GET: &[Outcome] = &[
    outcome(Code::Ok, "The value was written to stdout."),
    outcome(
        Code::Usage,
        "An unknown flag, a flag given twice, a missing KEY, --pretty without --format json, or a \
         KEY that names a whole block (core, database or database.<entry>) rather than one value. \
         On any command, also -v with -q, or a global flag given a value it does not take, such \
         as --timeout 0.",
    ),
    outcome(
        Code::NoInput,
        "KEY is not set in .tpl/.cfg, or names a database entry the file does not have. A known \
         key that the file does not set is reported with its default; an unknown key gets close \
         key names when there are any.",
    ),
    outcome(
        Code::IoError,
        "Reading .tpl failed, or stdout could not be written.",
    ),
    outcome(
        Code::Configuration,
        "No project was found, or .tpl/.cfg cannot be used.",
    ),
];

/// The outcomes of `cfg set`.
const CFG_SET: &[Outcome] = &[
    outcome(Code::Ok, "The value was written to .tpl/.cfg."),
    outcome(
        Code::Usage,
        "An unknown flag, a flag given twice, a missing KEY or VALUE, a KEY that is not one of \
         the keys listed under DESCRIPTION, a VALUE of the wrong type for its key, or a VALUE \
         that cannot stand beside a key the entry already holds, such as a dsn beside a host. On \
         any command, also -v with -q, or a global flag given a value it does not take, such as \
         --timeout 0.",
    ),
    outcome(Code::IoError, "Reading or rewriting .tpl/.cfg failed."),
    outcome(
        Code::Configuration,
        "No project was found, or .tpl/.cfg cannot be used.",
    ),
];

/// The outcomes of `cfg unset`.
const CFG_UNSET: &[Outcome] = &[
    outcome(Code::Ok, "The key or the block was removed from .tpl/.cfg."),
    outcome(
        Code::Usage,
        "An unknown flag, a flag given twice, or a missing KEY. On any command, also -v with -q, \
         or a global flag given a value it does not take, such as --timeout 0.",
    ),
    outcome(
        Code::NoInput,
        "KEY is not set in .tpl/.cfg, or names a database entry the file does not have.",
    ),
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
        "The list was written to stdout. An empty list is a result, not a failure.",
    ),
    outcome(
        Code::Usage,
        "An unknown flag, a flag given twice, or --pretty without --format json. On any command, \
         also -v with -q, or a global flag given a value it does not take, such as --timeout 0.",
    ),
    outcome(
        Code::IoError,
        "Reading .tpl failed, or stdout could not be written.",
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
        "An unknown flag, a missing NAME, or a flag given twice; no connection flag was given; \
         --dsn was given with --host, --port, --user or --schema; a --dsn, --port, --tls or \
         --password-command value is malformed, or a --ca-file or --ca-path value contains ${; the \
         password given twice, as a password inside --dsn and as --password-command; or NAME is \
         already taken. On any command, also -v with -q, or a \
         global flag given a value it does not take, such as --timeout 0.",
    ),
    outcome(Code::IoError, "Reading or rewriting .tpl/.cfg failed."),
    outcome(
        Code::Configuration,
        "No project was found, or .tpl/.cfg cannot be used.",
    ),
];

/// The outcomes of `cfg database show`.
const CFG_DATABASE_SHOW: &[Outcome] = &[
    outcome(
        Code::Ok,
        "The entry was written to stdout, passwords redacted.",
    ),
    outcome(
        Code::Usage,
        "An unknown flag, a flag given twice, a missing NAME, or --pretty without --format json. \
         On any command, also -v with -q, or a global flag given a value it does not take, such \
         as --timeout 0.",
    ),
    outcome(Code::NoInput, "NAME names no entry of .tpl/.cfg."),
    outcome(
        Code::IoError,
        "Reading .tpl/.cfg failed, or stdout could not be written.",
    ),
    outcome(
        Code::Configuration,
        "No project was found, or .tpl/.cfg cannot be used.",
    ),
];

/// The outcomes of `cfg database update`.
const CFG_DATABASE_UPDATE: &[Outcome] = &[
    outcome(Code::Ok, "The entry was changed in .tpl/.cfg."),
    outcome(
        Code::Usage,
        "An unknown flag, a missing NAME, or a flag given twice; no field flag; --dsn given with \
         --host, --port, --user or --schema; a malformed flag value; the password given twice, as \
         a password inside the dsn or password and as --password-command; or a change that would \
         leave the entry with both a dsn and host, port, user, password or database. On any \
         command, also -v with -q, or a global flag given a value it does not take, such as \
         --timeout 0.",
    ),
    outcome(Code::NoInput, "NAME names no entry of .tpl/.cfg."),
    outcome(Code::IoError, "Reading or rewriting .tpl/.cfg failed."),
    outcome(
        Code::Configuration,
        "No project was found, or .tpl/.cfg cannot be used.",
    ),
];

/// The outcomes of `cfg database remove`.
const CFG_DATABASE_REMOVE: &[Outcome] = &[
    outcome(
        Code::Ok,
        "The entry was removed from .tpl/.cfg, and core.database was cleared if it named the \
         entry.",
    ),
    outcome(
        Code::Usage,
        "An unknown flag, a flag given twice, or a missing NAME. On any command, also -v with -q, \
         or a global flag given a value it does not take, such as --timeout 0.",
    ),
    outcome(Code::NoInput, "NAME names no entry of .tpl/.cfg."),
    outcome(Code::IoError, "Reading or rewriting .tpl/.cfg failed."),
    outcome(
        Code::Configuration,
        "No project was found, or .tpl/.cfg cannot be used.",
    ),
];

/// The outcomes of `cfg database test`, the one `cfg` subcommand that connects.
const CFG_DATABASE_TEST: &[Outcome] = &[
    outcome(
        Code::Ok,
        "The four checks ran and the report was written to stdout. This does not mean the entry is \
         fully usable: read can_read_catalogue.",
    ),
    outcome(
        Code::Usage,
        "An unknown flag, a flag given twice, a missing NAME, or --pretty without --format json. \
         On any command, also -v with -q, or a global flag given a value it does not take, such \
         as --timeout 0.",
    ),
    outcome(Code::NoInput, "NAME names no entry of .tpl/.cfg."),
    outcome(
        Code::Unavailable,
        "The server could not be reached, or a connection step took longer than its time limit.",
    ),
    outcome(
        Code::IoError,
        "Reading .tpl, or the entry's ca_file or ca_path, failed, or stdout could not be written.",
    ),
    outcome(Code::NoPermission, "The server refused the login."),
    outcome(
        Code::Configuration,
        "The login succeeded but the server is not a supported MariaDB version or the session \
         could not be made read-only; the entry is incomplete (no host, or no server database \
         name) or its password cannot be obtained (a ${VAR} is undefined, or password_command \
         failed); ca_path holds no certificate; or no project was found, or .tpl/.cfg cannot be \
         used.",
    ),
];

/// The outcomes of `tpl init`, the only command that can produce `73`.
const INIT: &[Outcome] = &[
    outcome(
        Code::Ok,
        "The project was created; nothing was written to stdout. Creating a project below another \
         one succeeds, with a warning on stderr that the new one hides the one above.",
    ),
    outcome(
        Code::Usage,
        "An unknown flag, a flag given twice, or more than one PATH. On any command, also -v with \
         -q, or a global flag given a value it does not take, such as --timeout 0.",
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
        "The help or the JSON document was written to stdout.",
    ),
    outcome(
        Code::Usage,
        "An unknown flag, a flag given twice, a word of COMMAND_PATH that names no command under \
         the words before it, or --pretty without --format json. On any command, also -v with -q, \
         or a global flag given a value it does not take, such as --timeout 0.",
    ),
    outcome(
        Code::IoError,
        "stdout could not be written, or it closed part-way through the JSON document.",
    ),
];

/// The outcomes of `tpl version`.
const VERSION: &[Outcome] = &[
    outcome(Code::Ok, "The version line was written to stdout."),
    outcome(
        Code::Usage,
        "An argument, a flag other than the global ones (this command takes none), or a global \
         flag that takes a value given twice. On any command, also -v with -q, or a global flag \
         given a value it does not take, such as --timeout 0.",
    ),
    outcome(Code::IoError, "stdout could not be written."),
];

/// A row of a [`Block::Rows`].
const fn row(label: &'static str, text: &'static str) -> Row {
    Row { label, text }
}

// The sentences of `FR-HELP-031` that more than one leaf states. A sentence is
// written once where it is true of every leaf that uses it, so that two leaves
// cannot drift apart on one fact.

/// A command that never opens a connection.
const NO_SERVER: &str = "Does not contact the server.";

/// A command that works without a `[database.<name>]` entry.
const NO_ENTRY: &str = "Needs no database entry.";

/// A command that needs the entry `-d` or `core.database` selects.
const NEEDS_ENTRY: &str = "Needs a database entry (-d or core.database).";

/// A command that writes nothing to disk.
const NO_FILE: &str = "Writes no file.";

/// A command that writes `.tpl/.cfg` and nothing else.
const WRITES_CFG: &str = "Writes .tpl/.cfg.";

/// A command that prints nothing on success.
const PRINTS_NOTHING: &str = "Prints nothing.";

/// The no-expiry rule of the cache, in the same words wherever a command reads
/// it or the help defines it (finding U-02 of the fourth re-audit of rmp
/// `#263`): after an `ALTER TABLE`, a read through the cache exits `0` with the
/// old structure, and the caller must be told how to refresh it.
macro_rules! no_expiry {
    () => {
        "Nothing in the cache expires: after the database structure changes, run tpl cache \
         load or add --direct."
    };
}

/// The four statements of a `schema` leaf, which reads through the cache per
/// `FR-CACHE-015`: only its output differs from one leaf to the next.
const fn reads_catalogue(stdout: &'static str) -> Touches {
    Touches {
        server: concat!(
            "Connects to the server only when .tpl/.cache/ does not already hold the data, and \
             always with --direct. ",
            no_expiry!()
        ),
        entry: NEEDS_ENTRY,
        files: "Stores what it read in the entry's folder under .tpl/.cache/, unless --no-cache \
                is given.",
        stdout,
    }
}

/// The four statements of a command that reads only the project.
const fn local_only(entry: &'static str, files: &'static str, stdout: &'static str) -> Touches {
    Touches {
        server: NO_SERVER,
        entry,
        files,
        stdout,
    }
}

/// One entry per node of the tree of `FR-CLI-002`, in the order `FR-HELP-019`
/// requires: each node followed by its own children before the next node at
/// its level, preserving the order the tree declares them in.
const ENTRIES: [Entry; 35] = [
    Entry {
        path: &[],
        description: "tpl reads the structure of a MariaDB database (its tables, views and \
                      routines) and renders MiniJinja templates with it. It never writes to the \
                      database. It works inside a project: a .tpl folder, created by tpl init, \
                      that holds the configuration file .tpl/.cfg and the templates under \
                      .tpl/templates/.",
        blocks: &[
            Block::Rows {
                heading: "Words this help uses:",
                rows: &[
                    row(
                        "database entry",
                        "A named connection to a MariaDB server, stored in .tpl/.cfg. Its \
                         --schema is the name of the database on that server, which need not \
                         be the entry's name. Select one with -d NAME, or make one the default \
                         with tpl cfg set core.database NAME.",
                    ),
                    row(
                        "cache",
                        concat!(
                            "Copies of what tpl read from the server, kept under .tpl/.cache/ \
                             so that later commands need no connection. ",
                            no_expiry!()
                        ),
                    ),
                    row(
                        "template",
                        "A MiniJinja file under .tpl/templates/, named without its .jinja \
                         extension: .tpl/templates/rust/struct.jinja is rust/struct. tpl init \
                         creates example and rust/_types.",
                    ),
                ],
            },
            Block::Prose(
                "Every command accepts the seven global flags listed below, before or after \
                 the command name.",
            ),
        ],
        touches: None,
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
                caption: "Set up a project, connect it to a local server that has no trusted \
                          certificate (hence --tls disabled), take the password from the \
                          environment variable SHOP_PASSWORD (or write the password itself in \
                          place of the reference), list the tables as JSON, the form a program \
                          reads, and render one table.",
                lines: &[
                    run(&["tpl", "init"]),
                    run(&[
                        "tpl",
                        "cfg",
                        "database",
                        "add",
                        "shop",
                        "--host",
                        "127.0.0.1",
                        "--user",
                        "reader",
                        "--schema",
                        "shop",
                        "--tls",
                        "disabled",
                    ]),
                    run(&[
                        "tpl",
                        "cfg",
                        "set",
                        "database.shop.password",
                        "'${SHOP_PASSWORD}'",
                    ]),
                    run(&["tpl", "cfg", "set", "core.database", "shop"]),
                    run(&["tpl", "schema", "tables", "--format", "json"]),
                    run(&["tpl", "render", "example", "--table", "orders"]),
                ],
            },
        ],
        exit_codes: ROOT,
        see_also: &[&["help"], &["init"], &["version"]],
    },
    Entry {
        path: &["schema"],
        description: "Reads the structure of the selected database: its tables, views, routines \
                      and server. It only reads: the session is made read-only first, and every \
                      query is a SELECT, against INFORMATION_SCHEMA for the structure. What it \
                      reads is kept in .tpl/.cache/, so a second read of the same data needs no \
                      connection.",
        blocks: &[],
        touches: None,
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
        description: "Reports the selected database: its name, character set and collation, the \
                      server it runs on, and how many tables, views and routines it holds.",
        blocks: &[],
        touches: Some(reads_catalogue("Prints that report.")),
        examples: &[
            Example {
                caption: "Report the selected database.",
                lines: &[run(&["tpl", "-d", "shop", "schema", "info"])],
            },
            Example {
                caption: "Read the server version from the JSON document; this needs jq, \
                          an external JSON tool.",
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
        description: "Lists the tables of the selected database, one row each. To act on every \
                      table, read the names from the JSON output, as the last example does.",
        blocks: &[],
        touches: Some(reads_catalogue("Prints the tables, one row each.")),
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
                caption: "Render one file per table, which is how a caller iterates; \
                          rust/struct stands for a template of your own, and each file is \
                          written through a temporary one because a redirect empties its file \
                          before tpl runs; this needs jq, an external JSON tool.",
                lines: &[
                    run_in(
                        "",
                        &["tpl", "-d", "shop", "schema", "tables", "--format", "json"],
                        " |",
                    ),
                    shell("  jq -r '.data.tables[].name' |"),
                    shell("  while read -r table; do"),
                    shell("    f=\"src/models/$table.rs\""),
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
                        " > \"$f.tmp\" &&",
                    ),
                    shell("      mv \"$f.tmp\" \"$f\""),
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
        blocks: &[],
        touches: Some(reads_catalogue("Prints that description.")),
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
        blocks: &[],
        touches: Some(reads_catalogue("Prints the views, one row each.")),
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
        blocks: &[],
        touches: Some(reads_catalogue("Prints that description.")),
        examples: &[
            Example {
                caption: "Describe the v_sales view.",
                lines: &[run(&["tpl", "-d", "shop", "schema", "view", "v_sales"])],
            },
            Example {
                caption: "Print the SQL definition alone; this needs jq, an external JSON tool.",
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
        description: "Lists the stored procedures and stored functions of the selected database \
                      together, each row stating its kind. To keep one kind, filter the JSON \
                      output, as the last example does.",
        blocks: &[],
        touches: Some(reads_catalogue("Prints the routines, one row each.")),
        examples: &[
            Example {
                caption: "List every routine.",
                lines: &[run(&["tpl", "-d", "shop", "schema", "routines"])],
            },
            Example {
                caption: "List each routine with its kind, one per line; this needs jq, \
                          an external JSON tool.",
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
        description: "Describes one stored procedure or stored function. A procedure and a \
                      function may share a name; a bare NAME that matches both is refused, so \
                      write procedure:NAME or function:NAME.",
        blocks: &[],
        touches: Some(reads_catalogue("Prints that description.")),
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
        description: "Writes the whole selected database as one JSON document; the output is \
                      always JSON, and --format is refused. tpl render --context reads this \
                      document back and renders without a server. The document holds what a \
                      template sees as database; tpl render adds vars, tpl and now itself.",
        blocks: &[],
        touches: Some(reads_catalogue("Prints the JSON document.")),
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
                caption: "Render from a dump; the render itself contacts no server. rust/struct \
                          stands for a template of your own.",
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
        description: "Reads the templates of the project, under .tpl/templates/. No subcommand \
                      connects to a server, runs a template, or reads a template outside \
                      .tpl/templates/.",
        blocks: &[],
        touches: None,
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
        description: "Lists every template of the project by name, in name order. A name has no \
                      .jinja extension, and the NAME given to render, template show, template \
                      check and template path needs none. An {% include %} inside a template is \
                      different: it needs the extension, as in {% include \"rust/_types.jinja\" \
                      %}.",
        blocks: &[],
        touches: Some(local_only(
            NO_ENTRY,
            NO_FILE,
            "Prints a NAME header line, then one template name per line.",
        )),
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
        description: "Prints the source of one template exactly as it is stored: nothing is run \
                      and nothing is escaped.",
        blocks: &[],
        touches: Some(local_only(
            NO_ENTRY,
            NO_FILE,
            "Prints the template's source.",
        )),
        examples: &[
            Example {
                caption: "Print a template's source.",
                lines: &[run(&["tpl", "template", "show", "example"])],
            },
            Example {
                caption: "The extension is optional, and names the same template.",
                lines: &[run(&["tpl", "template", "show", "example.jinja"])],
            },
        ],
        exit_codes: TEMPLATE_NAMED,
        see_also: &[&["template", "list"], &["template", "check"]],
    },
    Entry {
        path: &["template", "check"],
        description: "Checks the syntax of templates without running them: no expression is \
                      evaluated, no filter is called and no server is contacted, so it is safe on \
                      a template you have not read. With no NAME, it checks every template of the \
                      project.",
        blocks: &[],
        touches: Some(local_only(
            NO_ENTRY,
            NO_FILE,
            "Prints nothing; a syntax error is reported on stderr, with its line and column.",
        )),
        examples: &[
            Example {
                caption: "Check every template of the project.",
                lines: &[run(&["tpl", "template", "check"])],
            },
            Example {
                caption: "Check two named templates.",
                lines: &[run(&["tpl", "template", "check", "example", "rust/_types"])],
            },
        ],
        exit_codes: TEMPLATE_CHECK,
        see_also: &[&["template", "list"], &["render"]],
    },
    Entry {
        path: &["template", "path"],
        description: "Prints the absolute path a template name resolves to. With no NAME, it \
                      prints the absolute path of .tpl/templates/.",
        blocks: &[],
        touches: Some(local_only(NO_ENTRY, NO_FILE, "Prints the path.")),
        examples: &[
            Example {
                caption: "Print the template folder, .tpl/templates/.",
                lines: &[run(&["tpl", "template", "path"])],
            },
            Example {
                caption: "Print where one template lives.",
                lines: &[run(&["tpl", "template", "path", "example"])],
            },
        ],
        exit_codes: TEMPLATE_PATH,
        see_also: &[&["template", "list"], &["template", "show"]],
    },
    Entry {
        path: &["render"],
        description: "Renders one template once and prints the result to stdout. To write a file, \
                      redirect stdout: there is no --output flag. The data comes from the selected \
                      database entry, or from a --context file written by tpl schema dump, never \
                      from both. The template always sees the whole database as database; \
                      --table, --view or --routine also binds that one object as table, view or \
                      routine.",
        blocks: &[
            Block::Surface,
            Block::Prose(
                "tpl init creates two templates: example, a worked example, and rust/_types; \
                 print one with tpl template show example. The examples below use rust/struct \
                 and docs/table.md as stand-ins for templates of your own, which must exist \
                 before they render.",
            ),
        ],
        touches: Some(Touches {
            server: concat!(
                "Connects to the server only when .tpl/.cache/ does not already hold what the \
                 template reads, always with --direct, and never with --context. ",
                no_expiry!()
            ),
            entry: "Needs a database entry (-d or core.database), except with --context, which \
                     takes none.",
            files: "Stores what it read under .tpl/.cache/, unless --no-cache or --context is \
                     given; the rendered text is never written to a file by tpl.",
            stdout: "Prints the rendered text and nothing else.",
        }),
        examples: &[
            Example {
                caption: "Render rust/struct for the orders table.",
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
                caption: "Write the result to a file, through a temporary one: a redirect empties \
                          its file before tpl runs, and a failed render prints nothing.",
                lines: &[
                    run_in(
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
                        " > src/models/orders.rs.tmp &&",
                    ),
                    shell("  mv src/models/orders.rs.tmp src/models/orders.rs"),
                ],
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
                caption: "Render one file per table, which is how a caller iterates; each file \
                          is written through a temporary one because a redirect empties its file \
                          before tpl runs; this needs jq, an external JSON tool.",
                lines: &[
                    run_in(
                        "",
                        &["tpl", "-d", "shop", "schema", "tables", "--format", "json"],
                        " |",
                    ),
                    shell("  jq -r '.data.tables[].name' |"),
                    shell("  while read -r table; do"),
                    shell("    f=\"src/models/$table.rs\""),
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
                        " > \"$f.tmp\" &&",
                    ),
                    shell("      mv \"$f.tmp\" \"$f\""),
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
        description: "Manages the cache under .tpl/.cache/: copies of what tpl read from the \
                      server, one folder per database entry. The schema commands and render read \
                      the cache first and connect only for what it lacks, storing what they read. \
                      Nothing in the cache expires; only cache load, cache clean and --direct \
                      replace or remove what it holds.",
        blocks: &[],
        touches: None,
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
        description: "Reads from the server and stores the result in .tpl/.cache/, replacing what \
                      was stored before. With no --table, --view or --routine, it loads the whole \
                      database of the selected entry.",
        blocks: &[],
        touches: Some(Touches {
            server: "Always connects to the server.",
            entry: NEEDS_ENTRY,
            files: "Writes what it read to the entry's folder under .tpl/.cache/.",
            stdout: PRINTS_NOTHING,
        }),
        examples: &[
            Example {
                caption: "Load the whole database of the entry named shop.",
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
        description: "Deletes the cached data of the selected entry. With no --table, --view or \
                      --routine, it deletes all of it. Nothing else removes cached data: after \
                      pointing an entry at another server with tpl cfg database update, run tpl \
                      cache clean.",
        blocks: &[],
        touches: Some(local_only(
            NEEDS_ENTRY,
            "Deletes files from the entry's folder under .tpl/.cache/ and writes none.",
            PRINTS_NOTHING,
        )),
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
        description: "Reports what the cache holds for the selected entry: when it was loaded, and \
                      how many tables, views and routines it holds. Use it instead of reading \
                      .tpl/.cache/, whose layout can change.",
        blocks: &[],
        touches: Some(local_only(NEEDS_ENTRY, NO_FILE, "Prints that report.")),
        examples: &[
            Example {
                caption: "Report the cache of the entry named shop.",
                lines: &[run(&["tpl", "-d", "shop", "cache", "status"])],
            },
            Example {
                caption: "Read the load time from the JSON document; this needs jq, \
                          an external JSON tool.",
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
        description: "Reads and writes .tpl/.cfg, the configuration file of the project. Use get, \
                      set, unset and list for single keys, and cfg database for whole database \
                      entries. Only cfg database test connects to a server.",
        blocks: &[],
        touches: None,
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
        description: "Prints the value stored under one key exactly as the file holds it: ${VAR} \
                      is not expanded and a password is not redacted, so the value can be passed \
                      to another command. KEY must name one value, such as core.database or \
                      database.shop.host; for a whole entry, use tpl cfg database show.",
        blocks: &[],
        touches: Some(local_only(NO_ENTRY, NO_FILE, "Prints the value.")),
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
        description: "Writes VALUE under KEY in .tpl/.cfg. KEY must be one of the keys below, and \
                      VALUE must be of the type that key takes.",
        blocks: &[
            Block::Rows {
                heading: "The keys (a time is a whole number of seconds, 1 or more):",
                rows: &[
                    row(
                        "core.database",
                        "The entry used when -d is not given. No default.",
                    ),
                    row(
                        "core.connect_timeout",
                        "Time to reach the server: DNS, TCP and TLS. Default 10.",
                    ),
                    row("core.query_timeout", "Time for each query. Default 30."),
                    row(
                        "core.password_timeout",
                        "Time for password_command to answer. Default 5.",
                    ),
                    row("core.render_timeout", "Time for one render. Default 30."),
                    row(
                        "core.render_fuel",
                        "Evaluation steps one render may take, from 1 to 1000000000000. \
                         Default 100000000.",
                    ),
                    row(
                        "core.render_output_limit",
                        "Bytes one render may print, from 1 to 1099511627776. Default \
                         67108864.",
                    ),
                    row(
                        "core.render_memory_limit",
                        "Bytes of memory one render may use, from 8388608 to 1099511627776. \
                         Default 134217728.",
                    ),
                    row(
                        "database.<name>.dsn",
                        "The whole connection as one URL, \
                         mysql://user:password@host:port/database. Not together with host, \
                         port, user, password or database.",
                    ),
                    row(
                        "database.<name>.host",
                        "The host name or IP address of the server.",
                    ),
                    row(
                        "database.<name>.port",
                        "The TCP port of the server, from 1 to 65535. Default 3306.",
                    ),
                    row("database.<name>.user", "The user name to log in with."),
                    row(
                        "database.<name>.password",
                        "The password. Not together with password_command.",
                    ),
                    row(
                        "database.<name>.password_command",
                        "A command that prints the password. Given to cfg set as one string, \
                         a command line such as \"pass db/shop\", never as an array; split into \
                         words as a shell would (quotes group words, and every quote must be \
                         closed); stored in the file as an array, [\"pass\", \"db/shop\"]. A \
                         value that starts with an unquoted [, or ends in a backslash outside \
                         quotes, is refused.",
                    ),
                    row(
                        "database.<name>.database",
                        "The name of the database on the server.",
                    ),
                    row(
                        "database.<name>.tls",
                        "disabled, preferred, required, verify-ca or verify-identity. \
                         Default verify-identity.",
                    ),
                    row(
                        "database.<name>.ca_file",
                        "A file of extra CA certificates to trust. Read as written; a value \
                         containing ${ is refused.",
                    ),
                    row(
                        "database.<name>.ca_path",
                        "A directory of extra CA certificate files to trust. Read as written; a \
                         value containing ${ is refused.",
                    ),
                ],
            },
            Block::Prose(
                "A value given here is visible to other users in the process list while tpl \
                 runs. For a secret, write a reference such as '${SHOP_PASSWORD}', in single \
                 quotes, and tpl reads that environment variable when it connects; a \
                 reference is expanded in dsn, host, port, user, password and database only. \
                 tpl cfg set takes a reference as the value of host, user, password or \
                 database, and as one part of a dsn, never as the whole dsn; a reference for \
                 port is accepted only when written in .tpl/.cfg by editing the file, as port \
                 = \"${SHOP_PORT}\".",
            ),
        ],
        touches: Some(local_only(NO_ENTRY, WRITES_CFG, PRINTS_NOTHING)),
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
        description: "Removes one key, or a whole block, from .tpl/.cfg. database.shop.host \
                      removes that field; database.shop removes the whole entry, and also clears \
                      core.database when it names that entry.",
        blocks: &[],
        touches: Some(local_only(NO_ENTRY, WRITES_CFG, PRINTS_NOTHING)),
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
        description: "Prints the whole of .tpl/.cfg, with passwords redacted and ${VAR} left as \
                      written. Nothing is resolved: no ${VAR} is expanded, no password_command is \
                      run and no default is filled in.",
        blocks: &[],
        touches: Some(local_only(NO_ENTRY, NO_FILE, "Prints the configuration.")),
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
        description: "Manages the database entries of .tpl/.cfg: the named connections that \
                      -d/--database selects. db is an alias of database: tpl cfg db list is tpl \
                      cfg database list.",
        blocks: &[],
        touches: None,
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
        description: "Creates a database entry named NAME. Give --dsn, or at least one of --host, \
                      --port, --user and --schema, but not both kinds; --tls, --password-command, \
                      --ca-file and --ca-path may be added to either. A NAME already in use is \
                      refused; change an entry with tpl cfg database update. To connect, the \
                      entry needs --host and --schema, or a --dsn that names both; --port defaults \
                      to 3306 and --user is optional. An entry without them is stored, but every \
                      later command that connects with it exits 78.",
        blocks: &[
            Block::Prose(
                "Without --tls, the entry uses verify-identity, which needs a server \
                 certificate the system trusts; for a local server without one, give --tls \
                 disabled or --tls preferred.",
            ),
            Block::Prose(
                "A value given here is visible to other users in the process list while tpl \
                 runs, so give a password through --password-command, or as a reference such \
                 as '${SHOP_PASSWORD}' inside --dsn.",
            ),
        ],
        touches: Some(local_only(
            "Needs no database entry: it creates one.",
            WRITES_CFG,
            PRINTS_NOTHING,
        )),
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
        description: "Prints the names of the database entries in .tpl/.cfg. A new project has \
                      none.",
        blocks: &[],
        touches: Some(local_only(
            NO_ENTRY,
            NO_FILE,
            "Prints a NAME header line, then one entry name per line.",
        )),
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
        description: "Prints one database entry, with passwords redacted and ${VAR} left as \
                      written.",
        blocks: &[],
        touches: Some(local_only(
            "Needs the entry that NAME names; -d and core.database are not used.",
            NO_FILE,
            "Prints the entry's keys and values.",
        )),
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
        exit_codes: CFG_DATABASE_SHOW,
        see_also: &[&["cfg", "database", "list"], &["cfg", "get"]],
    },
    Entry {
        path: &["cfg", "database", "update"],
        description: "Changes the fields of one database entry that the flags given name; every \
                      other field keeps its value, and at least one flag is required. The cache is \
                      not cleared: after pointing an entry at another server, run tpl -d NAME \
                      cache clean.",
        blocks: &[],
        touches: Some(local_only(
            "Needs the entry that NAME names; -d and core.database are not used.",
            "Writes .tpl/.cfg; the cache is left as it was.",
            PRINTS_NOTHING,
        )),
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
        exit_codes: CFG_DATABASE_UPDATE,
        see_also: &[
            &["cfg", "database", "add"],
            &["cfg", "database", "show"],
            &["cache", "clean"],
        ],
    },
    Entry {
        path: &["cfg", "database", "remove"],
        description: "Deletes one database entry from .tpl/.cfg. When core.database names it, \
                      core.database is cleared too.",
        blocks: &[],
        touches: Some(local_only(
            "Needs the entry that NAME names; -d and core.database are not used.",
            WRITES_CFG,
            PRINTS_NOTHING,
        )),
        examples: &[Example {
            caption: "Delete an entry.",
            lines: &[run(&["tpl", "cfg", "database", "remove", "staging"])],
        }],
        exit_codes: CFG_DATABASE_REMOVE,
        see_also: &[&["cfg", "database", "list"], &["cfg", "unset"]],
    },
    Entry {
        path: &["cfg", "database", "test"],
        description: "Connects with one database entry and reports four answers: whether it \
                      connected and logged in, whether the session was made read-only, which \
                      MariaDB server answered, and whether the database user may read the database \
                      structure (can_read_catalogue). Exit 0 means the four checks ran, not that \
                      all of them passed: read can_read_catalogue.",
        blocks: &[],
        touches: Some(Touches {
            server: "Always connects to the server the entry describes.",
            entry: "Needs the entry that NAME names; -d and core.database are not used.",
            files: "Writes no file, and neither reads nor writes the cache.",
            stdout: "Prints the four answers.",
        }),
        examples: &[
            Example {
                caption: "Test an entry.",
                lines: &[run(&["tpl", "cfg", "database", "test", "shop"])],
            },
            Example {
                caption: "Read the fourth answer, which the exit code does not carry; this \
                          needs jq, an external JSON tool.",
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
        description: "Creates a .tpl project in PATH, or in the current directory, creating \
                      missing parent directories. It writes .tpl/.cfg at mode 0600, with commented \
                      examples and no database entry; .tpl/.gitignore; and .tpl/templates/, \
                      holding example.jinja and the Rust type macros rust/_types.jinja. Next, add \
                      a database entry with tpl cfg database add.",
        // FR-HELP-034: the sentence, before the four statements of FR-HELP-031.
        blocks: &[Block::Prose(
            "--tpl-dir has no effect here: the project is created at PATH, or in the current \
             directory when PATH is absent.",
        )],
        touches: Some(local_only(
            NO_ENTRY,
            "Writes the new .tpl folder and what it holds.",
            "Prints nothing; a warning goes to stderr when the new project hides one in a parent \
             directory, and when --tpl-dir is given.",
        )),
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
        description: "Prints the help of any command, given as its path, such as tpl help cfg \
                      database add; an alias works in the path. With --format json it prints the \
                      whole command tree as one JSON document instead, or only the part under the \
                      path given.",
        blocks: &[],
        touches: Some(local_only(
            "Needs no database entry and no project.",
            NO_FILE,
            "Prints the help text, or the JSON document with --format json.",
        )),
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
        description: "Prints tpl, a space and the version, such as tpl 0.1.0, and nothing else.",
        blocks: &[],
        touches: Some(local_only(
            "Needs no database entry and no project.",
            NO_FILE,
            "Prints the version line.",
        )),
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

        // FR-RND-018 and FR-RND-041.
        assert_eq!(stated(&["render"], "--context"), ["--database", "--direct"]);

        // FR-RND-041.
        assert_eq!(stated(&["render"], "--direct"), ["--context"]);

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
    fn fr_help_031_every_leaf_and_only_a_leaf_carries_the_four_statements() {
        // FR-HELP-031 obliges the four statements of every leaf; a group node
        // does nothing but print its help, and states none.
        let tree = tree();

        for entry in &ENTRIES {
            let node = entry.path.iter().fold(&tree, |node, segment| {
                node.find_subcommand(segment)
                    .expect("every entry's path names a node")
            });
            let leaf = node.get_subcommands().next().is_none();

            assert_eq!(
                entry.touches.is_some(),
                leaf,
                "{} disagrees with the tree about being a leaf",
                written(entry.path)
            );

            if let Some(touches) = entry.touches {
                for statement in [touches.server, touches.entry, touches.files, touches.stdout] {
                    assert!(
                        statement.ends_with('.') && !statement.contains("  "),
                        "{} states {statement:?}",
                        written(entry.path)
                    );
                }

                assert!(
                    entry.written_description().ends_with(&touches.sentence()),
                    "{} does not end its JSON description with the statements",
                    written(entry.path)
                );
            }
        }
    }

    #[test]
    fn fr_help_014_no_description_or_exit_code_cites_a_requirement() {
        // FR-HELP-014 over the prose of every entry: its description, every
        // block, the four statements, the captions and the exit-code meanings.
        const PREFIXES: [&str; 8] = ["FR-", "NFR-", "BR-", "UC-", "OD-", "ADR-", "OQ-", "DIV-"];

        for entry in &ENTRIES {
            let mut prose = vec![entry.written_description()];
            prose.extend(
                entry
                    .examples
                    .iter()
                    .map(|example| example.caption.to_owned()),
            );
            prose.extend(
                entry
                    .exit_codes
                    .iter()
                    .map(|outcome| outcome.meaning.to_owned()),
            );

            for text in prose {
                for prefix in PREFIXES {
                    assert!(
                        !text.contains(prefix),
                        "{} cites {prefix}: {text}",
                        written(entry.path)
                    );
                }
            }
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
