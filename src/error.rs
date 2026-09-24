//! The error type and the exit status derived from it.
//!
//! `FR-ERR-001` fixes ten exit codes and `FR-ERR-002` forbids two conditions
//! whose remedies differ from sharing one, so every condition `tpl` can reach
//! is a variant of [`enum@Error`] and the assignment of a code is made once, in
//! [`Error::exit_code`], by a match the compiler checks for exhaustiveness. A
//! condition added without a code is therefore a compile error rather than a
//! wrong exit status. `OD-06` settles the shape: one public non-exhaustive
//! enum here, an inherent method for the status, and a renderer in
//! `diagnostics/` for the four labelled lines of `FR-ERR-008`.
//!
//! [`Display`](std::fmt::Display) carries the content of the `error:` line
//! alone, unescaped. The `cause`, `hint` and `exit` lines, the escaping of
//! `FR-ERR-024` and the nearest-match suggestions of `FR-ERR-019` are not this
//! module's business.
//!
//! Every variant names the **instance** that failed rather than the category it
//! belongs to, because `FR-ERR-034` obliges the `cause` line to do so and the
//! renderer can only name what the value carries.
//!
//! No variant carries the database driver's error, and no conversion from one
//! exists. `FR-GLOB-018` keeps a driver message off every stream, and `OD-06`
//! enforces that structurally by denying it a home: a driver failure is
//! classified at the `mariadb/` boundary into a phase, a host, a port and a
//! classification, and the original value is dropped. The template engine is
//! the deliberate exception, because `FR-ERR-011` requires the chain of
//! underlying engine errors to reach the caller.

use std::fmt;
use std::io;
use std::panic::Location;
use std::path::{Path, PathBuf};
use std::time::Duration;

use thiserror::Error;

/// A position inside a text document, counted from one.
///
/// It locates a template error (`FR-ERR-011`), a malformed `--context`
/// document (`FR-ERR-034`) and a `.tpl/.cfg` the TOML parser rejected.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Position {
    /// The line, counted from one.
    pub line: usize,
    /// The column, counted from one.
    pub column: usize,
}

impl fmt::Display for Position {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "line {}, column {}", self.line, self.column)
    }
}

/// The kind of catalogue object an identifier was sought as.
///
/// `FR-ERR-034` obliges the `cause` line of a `66` to name it, and the `cause`
/// line of a `77` to name the object whose property could not be read.
///
/// **The two obligations range over different populations, and the fourth
/// variant is why.** A `66` is reached for the three object kinds
/// `FR-SCH-010` names, and for those alone. A `77` under `FR-PRIV-021` is
/// reached for the **database** itself, whose own metadata the reader could
/// not read, and that requirement rejects `66` for it in as many words: the
/// nearest-match suggestion a `66` carries is drawn from a population, and the
/// population of databases is one this system never reads. [`Self::Database`]
/// therefore has no listing subcommand, which is the one place the difference
/// shows — see `listing` in the crate's own `diagnostics` module.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum CatalogueObjectKind {
    /// A base table.
    Table,
    /// A view.
    View,
    /// A stored procedure or function.
    Routine,
    /// The database a read covers, per `FR-CONF-041` (`FR-PRIV-021`).
    ///
    /// It reaches a `77` and never a `66`, for the reason this type's own
    /// documentation gives.
    Database,
}

impl fmt::Display for CatalogueObjectKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let name = match self {
            Self::Table => "table",
            Self::View => "view",
            Self::Routine => "routine",
            Self::Database => "database",
        };
        f.write_str(name)
    }
}

/// The phase of a connection or a catalogue read that failed.
///
/// The four are the ones `FR-ERR-034` enumerates for the `cause` line of a
/// `69`, and the set is closed by that row.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NetworkPhase {
    /// Resolving the host name.
    DnsResolution,
    /// Opening the TCP connection.
    TcpConnect,
    /// Negotiating TLS, per the mode of `FR-CONF-013`.
    TlsHandshake,
    /// Reading `INFORMATION_SCHEMA`.
    CatalogueQuery,
}

impl fmt::Display for NetworkPhase {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let name = match self {
            Self::DnsResolution => "DNS resolution",
            Self::TcpConnect => "the TCP connect",
            Self::TlsHandshake => "the TLS handshake",
            Self::CatalogueQuery => "a query reading the database structure",
        };
        f.write_str(name)
    }
}

/// Which of the two bounds of `FR-GLOB-012` expired.
///
/// A phase ends when the first of the two expires, so the `cause` line has to
/// say which one did: the accepted cost recorded against `FR-GLOB-012` is that
/// `--timeout` can end a phase whose own deadline is larger.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DeadlineBound {
    /// The phase's own deadline, resolved by `FR-CONF-004`.
    Phase,
    /// The overall wall-clock budget of `FR-GLOB-011`, measured from process
    /// start.
    Overall,
}

impl fmt::Display for DeadlineBound {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let name = match self {
            Self::Phase => "its own deadline",
            Self::Overall => "the overall budget",
        };
        f.write_str(name)
    }
}

/// The global lookup function a template called (`FR-ENV-020`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum LookupKind {
    /// `table(name)`.
    Table,
    /// `view(name)`.
    View,
    /// `routine(name)`.
    Routine,
    /// `column(table, name)`.
    Column,
}

impl fmt::Display for LookupKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::Table => "table",
            Self::View => "view",
            Self::Routine => "routine",
            Self::Column => "column",
        })
    }
}

/// What is known of why a render failed, beyond the engine's chain.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum RenderReason {
    /// The template called `fail(message)` with this message (`FR-SEM-014`,
    /// `FR-SEM-015`).
    Failed(String),
    /// The undefined expression begins with a lookup call that found nothing.
    Unresolved(Unresolved),
    /// The undefined expression begins with `table`, `view` or `routine`, and
    /// the render bound that variable: the flag that binds it was given, and
    /// what is undefined is a step of the expression after it (finding Y-01
    /// of the eighth re-audit of rmp `#263`).
    Missing(Missing),
    /// An `{% include %}` named a template the loader does not hold
    /// (`FR-TMPL-009`).
    IncludeNotFound {
        /// The name the include wrote.
        name: String,
        /// Whether the name with `.jinja` appended is a template of the
        /// project: the include then only lacks the extension `FR-TMPL-009`
        /// requires it to write.
        lacks_extension: bool,
    },
}

/// A lookup call, with arguments the template wrote as literals, that found
/// nothing — the reason an expression built on it is undefined.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Unresolved {
    /// The call as the template wrote it: `table("orders")`.
    pub call: String,
    /// What was sought: the function's kind, or [`LookupKind::Table`] for a
    /// `column` call whose table does not exist.
    pub kind: LookupKind,
    /// The name that was not found.
    pub name: String,
    /// For a column that was not found in a table that exists, that table.
    pub table: Option<String>,
    /// The `--context` document the render read, WHERE it read one: the
    /// names a lookup could have found are then that document's, and no `tpl`
    /// command lists them.
    pub document: Option<std::path::PathBuf>,
}

/// The step of an expression rooted at a bound object variable that found
/// nothing.
///
/// `owner` is the part of the expression before that step, as the template
/// wrote it. It is built only from names and decimal indexes, so every
/// character of it is in `[A-Za-z0-9_.\[\]-]`.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum Missing {
    /// `owner` holds no attribute `name`.
    Attribute {
        /// The part of the expression before the attribute.
        owner: String,
        /// The attribute the template read.
        name: String,
        /// What `owner` is, in words: `an object`, `a list`, `a string`.
        kind: &'static str,
        /// The attributes `owner` holds, in the order it holds them, WHERE it
        /// is an object.
        attributes: Vec<String>,
        /// The attributes nearest to `name`, at most three, in the order of
        /// `FR-ERR-019`.
        nearest: Vec<String>,
    },
    /// `owner` is a list with no item at `index`.
    Index {
        /// The part of the expression before the index.
        owner: String,
        /// The index the template read.
        index: i64,
        /// The number of items `owner` holds.
        length: usize,
    },
    /// `owner` is not a list, and holds no item at `index`.
    NotList {
        /// The part of the expression before the index.
        owner: String,
        /// The index the template read.
        index: i64,
        /// What `owner` is, in words.
        kind: &'static str,
    },
    /// The variable is bound, and the step that found nothing could not be
    /// located: the expression calls a function, a filter or a method.
    Elsewhere {
        /// The bound variable: `table`, `view` or `routine`.
        root: &'static str,
    },
}

/// How a `--context` document failed the contract of `FR-RND-020`.
///
/// `FR-ERR-034` obliges the `cause` line of such a `65` to name the path and
/// *either* the position of the malformed JSON *or* the structural rule the
/// document failed; this is that either.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ContextFault {
    /// The bytes are not well-formed JSON, at this position.
    NotJson(Position),
    /// The bytes hold no JSON text at all: nothing, or only whitespace.
    Empty,
    /// The document is JSON and does not match the contract of
    /// `context-document.md`.
    ///
    /// The forty-third edition of the `65` row of `FR-ERR-034` obliges the
    /// `cause` line to name the key path of the first member, in document
    /// order, that fails the contract, and what the contract expects there —
    /// and to cite no file of the specification. The two fields are those two
    /// facts.
    Structure {
        /// The key path of the member that fails, written as a caller reads it
        /// in the document — `data.database.tables[0].name` — or the empty
        /// string where the failure is the document as a whole.
        at: String,
        /// What the contract expects there: the type, that the key is
        /// required, or the rule the member breaks.
        expected: String,
    },
    /// A foreign key of a member of `tables` names a table that `tables` does
    /// not carry (`FR-CTX-042`).
    ///
    /// `FR-CTX-042` obliges the `cause` line to name the table that carries
    /// the key, the key, and the table it names, beside the path the enclosing
    /// [`Error::ContextDocumentMalformed`] carries.
    DanglingReference {
        /// The member of `tables` that carries the key.
        table: String,
        /// The collection of that member the key is listed under:
        /// `foreign_keys` or `referenced_by`.
        collection: &'static str,
        /// The key's name.
        key: String,
        /// The table the key names and `tables` does not carry — the
        /// referenced table under `foreign_keys`, the referencing table under
        /// `referenced_by`.
        names: String,
    },
}

/// Which half of the DSN grammar a value failed.
///
/// `FR-CONF-009` fixes the form and `FR-CONF-010` the two accepted schemes, and
/// `FR-ERR-034` obliges the `cause` line to separate them: a caller who wrote
/// `postgres://` and a caller who wrote a host with no database have different
/// next steps.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DsnFault {
    /// The scheme is neither `mysql://` nor `mariadb://` (`FR-CONF-010`).
    Scheme,
    /// The value does not have the form
    /// `scheme://[user[:password]@]host[:port]/database` (`FR-CONF-009`).
    Form,
}

/// How a write `FR-CFG-048` refuses is made legal.
///
/// That requirement obliges the `hint` to carry a runnable command that makes
/// the change the invocation asked for and deletes nothing it did not name
/// (`BR-ERR-005`), and its table fixes that command for each refused pair.
/// Which row applies depends on where each member of the pair came from, and
/// only the writer knows that, so the distinction is made where the refusal is
/// raised rather than where the line is composed.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EntryRepair {
    /// The entry carries one source of the password and the invocation writes
    /// the other: one `tpl cfg unset` of the source the entry carries, then the
    /// same write again (rows three and four of the `FR-CFG-048` table).
    ///
    /// The password is information the invocation's own write supplies anew,
    /// so removing the source it replaces deletes nothing the caller did not
    /// name, per `BR-ERR-005`.
    Unset,

    /// The entry is defined by `dsn` and the invocation writes discrete
    /// connection fields (row one of the table, `FR-ERR-045`): the field is
    /// changed inside the dsn, with `tpl cfg database update <entry> --dsn
    /// <url>`.
    InsideDsn {
        /// The leaf names of the discrete connection fields the invocation
        /// writes, in the order `FR-CONF-002` states them.
        fields: Box<[&'static str]>,
    },

    /// The entry's dsn carries a password and the invocation writes
    /// `password_command` (row five of the table): the dsn is written again
    /// without its password, then the same write again.
    DsnWithoutPassword,

    /// The entry is described by discrete connection fields and the
    /// invocation writes a `dsn` (row two of the table): the fields the new
    /// value changes are written with their own flags instead.
    Discrete {
        /// The leaf names of the discrete connection fields the entry carries,
        /// which the `cause` names.
        carried: Box<[&'static str]>,
        /// The leaf names of the fields the dsn written states, whose flags
        /// the `hint` carries.
        changed: Box<[&'static str]>,
    },

    /// The invocation itself supplies both members, so nothing in the file has
    /// to change and the command is written again without one of the two.
    ///
    /// The value is the command path that writes the entry, below `tpl`.
    Restate(String),
}

/// How the read-only session of `FR-SRV-008` failed to take effect.
///
/// `FR-SRV-010` names two conditions and gives both `78`; the `cause` line has
/// to separate them, per `FR-ERR-034`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ReadOnlyFault {
    /// The setting could not be applied.
    NotApplied,
    /// The read-back of `@@session.tx_read_only` did not confirm it.
    ReadBackDisagreed,
}

/// Which of the two conditions of `FR-CONF-042` left `tpl` with no exit status
/// for a `password_command`.
///
/// That requirement gives both `78` and obliges the `cause` line to say which
/// occurred, per `FR-ERR-034`: a program that never ran is corrected in
/// `.tpl/.cfg`, and a child whose outcome could not be read leaves the machine
/// as what to look at.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PasswordCommandFault {
    /// The child could not be started at all.
    NotStarted,

    /// The child had started and the system could not read the status it ended
    /// with.
    StatusUnreadable,
}

/// How a `password_command` child ended, where it did not exit zero.
///
/// `FR-CONF-033` owns the exit and `FR-CONF-043` the signal, and each obliges
/// the `cause` line to name its own instance. The two are one type because they
/// are one condition to a caller — the configured way of obtaining a password
/// produced none — and share the one code those requirements give them.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ChildEnd {
    /// It exited with this non-zero status (`FR-CONF-033`).
    Exited(i32),

    /// It was ended by this signal, which `tpl` did not send (`FR-CONF-043`).
    ///
    /// The two signals `tpl` does send — the deadline of `FR-CONF-028` and the
    /// output cap of `FR-CONF-031` — never reach here, because each owns its
    /// outcome and returns its own condition before the status is read.
    Signalled(i32),

    /// It reported neither an exit status nor a signal.
    ///
    /// No target of `NFR-PERF-018` produces this: a Unix reports one or the
    /// other. It is a variant rather than a panic because the standard library
    /// promises neither, and `FR-ERR-034` bans naming a category where an
    /// instance is available — here none is.
    Unreported,
}

/// What the TLS handshake of `FR-CONF-013` returned, as the driver's
/// discriminants classify it.
///
/// The `69` row of `FR-ERR-034` obliges the `cause` line to name what the
/// phase returned, and `FR-GLOB-018` bars the driver's own message from every
/// stream, so what it returned is stated as this classification and the
/// message is dropped where it is made.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TlsFault {
    /// The TLS layer refused the connection before any certificate was judged:
    /// the server offers no TLS, or the negotiation itself failed.
    Refused,
    /// The server presented a certificate and it was not accepted: its chain
    /// is not trusted by the trust material, or it does not name the host.
    CertificateRejected,
}

/// Where an entry name the rule of `FR-CONF-048` refuses was given on the
/// command line.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EntryNameGiven {
    /// The operand of `tpl cfg database add`.
    Add,
    /// The `<name>` segment of a `database.<name>.<field>` key given to
    /// `tpl cfg set`, with the leaf of that key.
    Key(&'static str),
    /// The value given to `tpl cfg set core.database`.
    CoreDatabase,
}

/// What is wrong with a `${NAME}` reference (`FR-CONF-021`, `FR-CONF-049`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ReferenceFault {
    /// A `${` with no closing brace.
    Unclosed,
    /// A reference whose name is not `[A-Za-z_][A-Za-z0-9_]*`, carrying the
    /// name as written.
    Name(String),
}

/// Why a selected entry does not carry a key it needs (`FR-CONF-040`,
/// `FR-CONF-041`, `FR-CONF-050`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KeyAbsence {
    /// The entry does not declare the key.
    Absent,
    /// The entry declares the key as the empty string.
    Empty,
    /// The key holds a `${VAR}` reference that expands to the empty string.
    ExpandsToEmpty,
}

/// Why the path `--tpl-dir` named cannot be used as the project
/// (`FR-PROJ-008`, `FR-PROJ-027`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TplDirFault {
    /// Nothing exists at the path.
    Missing,
    /// Something exists at the path, and it is not a directory.
    NotDirectory,
    /// The path is a directory whose last segment is `.tpl` in neither its
    /// written nor its canonical form, and it holds a directory named `.tpl`:
    /// the caller named the project directory instead of its `.tpl` folder
    /// (`FR-PROJ-027`).
    HoldsTplFolder,
    /// The path is a directory whose last segment is `.tpl` in neither form,
    /// and it holds no directory named `.tpl` (`FR-PROJ-027`).
    NotTplFolder,
}

/// Every condition `tpl` reports as a failure.
///
/// One variant per distinct condition the specification names, each carrying
/// the instance its code's row in `FR-ERR-034` obliges the `cause` line to
/// name. The code itself is not a field: it is derived by
/// [`exit_code`](Error::exit_code), so no two variants can disagree with the
/// table of `FR-ERR-001` by construction.
///
/// The enum is `#[non_exhaustive]` because it grows with the modules that
/// report through it. Downstream code must therefore carry a wildcard arm; the
/// match inside this crate is exhaustive and the compiler keeps it so. That
/// asymmetry is why `OD-06` puts the derivation here and not in a table in the
/// binary, which is a downstream crate:
///
/// ```compile_fail
/// // error[E0004]: a wildcard arm is required, because `tpl::Error` is
/// // marked as non-exhaustive outside the crate that defines it.
/// fn code(error: &tpl::Error) -> u8 {
///     match error {
///         tpl::Error::StdoutClosedMidDocument => 74,
///     }
/// }
/// ```
///
/// Constructing a variant is unaffected, which is what lets the trigger of
/// `FR-ERR-031` and the tests of `BR-ERR-001` build one.
#[derive(Debug, Error)]
#[non_exhaustive]
pub enum Error {
    // ---------------------------------------------------------------- 64 ---
    /// A command the tree of `FR-CLI-002` does not declare (`FR-CLI-003`).
    #[error("{}", unknown_command(.token, .node))]
    UnknownCommand {
        /// The token as written, never normalised (`FR-CLI-020`).
        token: String,
        /// The command path of the node the token was written under, without
        /// the program name, and empty at the root.
        ///
        /// It is carried so that both lines name the group a mistyped
        /// subcommand was sought in, and so that the hint lists that group's
        /// children rather than the root's, as `FR-HELP-028` already does for
        /// the same mistake made inside `tpl help`.
        node: String,
        /// The nearest matches among the children of the node the token was
        /// written at, selected by `FR-ERR-019` and ordered as it fixes.
        ///
        /// Empty where nothing qualified, which `FR-ERR-020` requires to leave
        /// the generic hint standing alone. `FR-CLI-003` obliges the
        /// suggestion, and `BR-CLI-001` routes a mistyped alias through it.
        nearest: Vec<String>,
    },

    /// A segment of a `tpl help` command path that names no child of the node
    /// the preceding segments resolved to (`FR-HELP-028`).
    ///
    /// It is distinct from [`UnknownCommand`](Error::UnknownCommand), which is
    /// `FR-CLI-003`'s condition — a token of the invocation that is not a
    /// command — because the two oblige different `cause` lines under
    /// `FR-ERR-034` and draw their suggestions from different populations.
    /// `FR-HELP-028` requires the node the segment was looked for under to be
    /// named, and requires the suggestion to be made over **that node's
    /// children alone** rather than over the whole tree, so that
    /// `tpl help cfg database ad` proposes `add` and not every node named `add`
    /// anywhere. The node is therefore carried on the variant: without it the
    /// message could not satisfy the row, and the two conditions would share a
    /// wording that `FR-ERR-034` forbids.
    ///
    /// The node is named by the rule `crate::diagnostics::invoked` fixes, which
    /// is the rule the `cause` line of this variant reads too: the root is the
    /// program name and not the empty path. `FR-HELP-028` obliges the node to
    /// be named on both lines, and a second spelling of the root here is how
    /// the two came to say `''` and `'tpl'` of one node.
    #[error("unknown command '{segment}' under '{}'", crate::diagnostics::invoked(.node))]
    UnknownCommandPathSegment {
        /// The segment as written, never normalised (`FR-CLI-020`).
        segment: String,
        /// The command path of the node the segment was looked for under,
        /// without the program name, and empty at the root.
        node: String,
        /// The nearest matches among that node's children — their canonical
        /// names and the aliases of `FR-CLI-011` alike, per `BR-CLI-001` —
        /// selected by `FR-ERR-019` and ordered as it fixes.
        ///
        /// Empty where nothing qualified, which `FR-ERR-020` requires to leave
        /// the generic hint standing alone.
        nearest: Vec<String>,
    },

    /// A flag the invoked node does not declare (`FR-CLI-005`, `FR-CLI-019`).
    #[error("unknown flag '{token}'")]
    UnknownFlag {
        /// The token as written.
        token: String,
        /// The command path of the node the token was written at, without the
        /// program name, and empty at the root — the node whose help lists the
        /// flags it does declare.
        command: String,
        /// Whether that node takes a positional argument, so that a value
        /// beginning with `-` can have been meant for it; `FR-CLI-017` accepts
        /// such a value after `--`, and the hint says so.
        positional: bool,
        /// The nearest matches among the flags the invoked node declares,
        /// selected by `FR-ERR-019` and ordered as it fixes. Empty where
        /// nothing qualified, per `FR-ERR-020`.
        nearest: Vec<String>,
        /// The command path of the command that declares the flag, WHERE the
        /// flag was written before that command's name: only a global flag
        /// may come before its command (`FR-CLI-024`), and the hint moves the
        /// flag after it (finding Y-02 of the eighth re-audit of rmp `#263`).
        belongs_to: Option<String>,
    },

    /// A token supplied where the invoked command takes no further argument
    /// (`FR-ERR-001`, the `64` row).
    ///
    /// It is not an unknown flag: the token names no flag, either because it
    /// carries no leading `-` or because `FR-CLI-017` already made it a
    /// positional argument by putting it after `--`.
    #[error("unexpected argument '{token}'")]
    UnexpectedArgument {
        /// The command path the token was supplied to, without the program
        /// name, and empty at the root.
        command: String,
        /// The token as written.
        token: String,
    },

    /// A flag that carries a single value was given more than once
    /// (`FR-CLI-014`).
    ///
    /// Both values are carried because the requirement obliges the message to
    /// name both. `OD-08` is why they are available at all: the flag is
    /// declared with `ArgAction::Append` and the repetition is refused by
    /// `cli/`, rather than by the parser, which would name the argument twice
    /// and neither value.
    #[error("the flag '{flag}' was given more than once")]
    RepeatedValueFlag {
        /// The flag, in the long form the tree declares it under.
        flag: String,
        /// The value of the first occurrence, as written (`FR-CLI-020`).
        first: String,
        /// The value of the second occurrence, as written.
        second: String,
    },

    /// A flag that carries no value was given more than once.
    ///
    /// `FR-CLI-014` governs a flag that carries a single value and this one
    /// carries none, so no value is named; the refusal itself is the parser's,
    /// and this variant is what the caller reads it as.
    #[error("the flag '{flag}' was given more than once")]
    RepeatedFlag {
        /// The flag, in the long form the tree declares it under.
        flag: String,
    },

    /// `-d/--database` or `--tpl-dir` took a command name from a separate
    /// token as its value, and the token after it then named no command
    /// (`FR-CLI-026`).
    #[error("{flag} needs a value")]
    FlagTookCommand {
        /// The flag as written.
        flag: String,
        /// The command name it took as its value.
        value: String,
        /// The token that was then read as the command.
        token: String,
        /// The invocation after `tpl` with the flag's placeholder inserted
        /// after it, where every other token is admitted into a hint
        /// (`FR-ERR-022`, `FR-ERR-040`, `FR-ERR-041`); [`None`] otherwise.
        ///
        /// Boxed, with `path`, so that the variant leaves [`Error`] within
        /// the size `clippy::result_large_err` admits.
        rebuilt: Option<Box<str>>,
        /// The command path the invocation names once the flag has its
        /// value, which the hint carries alone where `rebuilt` is [`None`].
        path: Box<str>,
    },

    /// A flag that carries a value was given without one.
    #[error("the flag '{flag}' was given without a value")]
    FlagValueMissing {
        /// The flag, in the long form the tree declares it under.
        flag: String,
        /// The values the flag accepts, where it accepts a fixed set, in the
        /// order the tree declares them; empty otherwise.
        permitted: Vec<String>,
    },

    /// A flag value beginning with `-` was supplied as a separate token
    /// (`FR-CLI-018`).
    ///
    /// The same value is accepted in the `--flag=value` form, and as a
    /// positional argument after `--`, which is what the hint of this
    /// condition shows.
    #[error("'{value}' was read as a flag rather than as the value of '{flag}'")]
    SeparateTokenValue {
        /// The flag the value was written after, in the long form the tree
        /// declares it under.
        flag: String,
        /// The value as written.
        value: String,
    },

    /// A value outside the set the flag it was given for enumerates
    /// (`FR-ERR-001`, the `64` row).
    ///
    /// It is distinct from [`MalformedValue`](Error::MalformedValue) because
    /// the type expected is a closed set of spellings rather than a type name,
    /// and `FR-ERR-034` obliges the `cause` line to name it.
    #[error("'{value}' is not a value '{flag}' accepts")]
    ValueOutsideEnumeration {
        /// The flag, in the long form the tree declares it under.
        flag: String,
        /// The command path of the node the flag was given to, without the
        /// program name, and empty for a global flag — the node whose help
        /// states the values.
        command: String,
        /// The value as written.
        value: String,
        /// The values the flag enumerates, in the order it declares them.
        permitted: Vec<String>,
    },

    /// The argument parser refused the invocation for a condition this crate
    /// does not classify further (`OD-08`).
    ///
    /// `clap::error::ErrorKind` and `clap::error::ContextKind` are both
    /// `#[non_exhaustive]`, so the mapping carries a wildcard arm by force of
    /// the language. That arm produces this variant, and this variant produces
    /// `64` — never another code, so no unclassified refusal can move a caller
    /// onto a different branch.
    #[error("the invocation was rejected: {reason}")]
    InvocationRejected {
        /// What the parser refused, in plain words, as the kind it reported
        /// classifies it. It is a literal: the parser's own message is not
        /// carried.
        reason: &'static str,
        /// The command path of the node the invocation reached, without the
        /// program name, and empty at the root.
        command: String,
        /// The token the parser named, where it named one. A refusal that
        /// carries no token — a value that is not valid UTF-8 is the one this
        /// tree can reach — leaves it [`None`], which is the one case
        /// `FR-ERR-034`'s "wherever one is available" admits here.
        token: Option<String>,
    },

    /// A required argument was not supplied (`FR-ERR-001`, the `64` row).
    #[error("'{}' needs the argument {argument}", crate::diagnostics::invoked(.command))]
    MissingArgument {
        /// The command path, as written.
        command: String,
        /// The name of the argument that is missing.
        argument: String,
    },

    /// Two flags that exclude one another were given together
    /// (`FR-ERR-001`, the `64` row; `FR-CFG-016` is one instance).
    #[error("'{first}' and '{second}' cannot be given together")]
    MutuallyExclusiveFlags {
        /// The first member of the pair, as written.
        first: String,
        /// The second member of the pair, as written.
        second: String,
    },

    /// `tpl render` was given `--direct` together with `--context`
    /// (`FR-RND-041`): the one demands the server read the other excludes.
    #[error("--direct cannot be used with --context")]
    DirectWithContext,

    /// `--pretty` given where the format in force is not JSON (`FR-OUT-009`).
    ///
    /// It was reported as the pair `--pretty` and `--format text`, which named
    /// a flag the caller had usually not written — `text` is the default — and
    /// sent the caller to remove it. `FR-OUT-009` states the rule as a
    /// requirement of one flag on another, and the message states it so.
    #[error("'--pretty' needs '--format json'")]
    PrettyWithoutJson {
        /// The command path the flag was given to, without the program name.
        command: String,
        /// Whether that command runs with no operand, so that the hint can
        /// write it out whole and it succeeds as written (`BR-ERR-004`).
        complete: bool,
    },

    /// `tpl cfg database add` given neither `--dsn` nor a discrete connection
    /// flag (`FR-CFG-016`).
    #[error("'tpl cfg database add' needs connection details for entry '{entry}'")]
    ConnectionDetailsMissing {
        /// The entry the invocation asked to create.
        entry: String,
    },

    /// `tpl cfg database update` given none of the flags of `FR-CFG-027`
    /// (`FR-CFG-020`).
    #[error("nothing to change: tpl cfg database update needs at least one field flag")]
    NothingToUpdate {
        /// The entry the invocation named.
        entry: String,
    },

    /// A block — `core`, `database` or `database.<name>` — given to
    /// `tpl cfg get`, which reads one value (`FR-CFG-007`).
    #[error("{}", block_key(.key, .entry.is_some()))]
    BlockKeyGiven {
        /// The key as written.
        key: String,
        /// The entry the key names, WHERE it has the form `database.<name>`
        /// and `.tpl/.cfg` defines that entry — the case in which the hint is
        /// `tpl cfg database show <name>`.
        entry: Option<String>,
    },

    /// A token naming one routine whose qualifying prefix is spelled in a case
    /// other than lower (`FR-SCH-008`).
    ///
    /// The requirement fixes `procedure:` and `function:` and "no other
    /// spelling of either", and refuses a token whose prefix folds to one of
    /// the two without being written in lower case. It is decided from the
    /// token alone, so it precedes every catalogue read, at step 1 of
    /// `FR-ERR-006`.
    #[error("'{token}' does not carry its prefix in lower case")]
    RoutinePrefixNotLowerCase {
        /// The token as written, never normalised (`FR-CLI-020`).
        token: String,
        /// The prefix the token folds to — `procedure` or `function`, in the
        /// lower case `FR-SCH-008` fixes.
        prefix: &'static str,
        /// The routine name the token carries after the first colon, as
        /// written. It is a value this corpus does not fix, so `FR-ERR-022`
        /// governs it and `FR-ERR-023` drops it from the hint where it falls
        /// outside the character set.
        name: String,
        /// The invocation the token was given to, below `tpl`, as a literal of
        /// `FR-ERR-022` — `schema routine`, `cache load --routine`, or
        /// `cache clean --routine`.
        invocation: &'static str,
        /// The template of the `tpl render` the token was given to, which the
        /// invocation writes as `<template>`; [`None`] for every other
        /// command.
        template: Option<String>,
    },

    /// A bare routine name that names both a procedure and a function
    /// (`FR-SCH-010`).
    ///
    /// `FR-SCH-010` refuses it rather than resolving it: a first-wins rule
    /// would make one of the two objects permanently unreachable through a
    /// bare name, and which one it was would depend on the order the catalogue
    /// returned them.
    #[error("routine '{name}' names both a procedure and a function")]
    AmbiguousRoutineName {
        /// The bare name as written.
        name: String,
        /// The database entry of `.tpl/.cfg` the read was made through.
        entry: String,
        /// The server-side database the two objects were found in.
        database: String,
        /// The invocation the token was given to, below `tpl`, as a literal of
        /// `FR-ERR-022`.
        invocation: &'static str,
        /// The template of the `tpl render` the token was given to, which the
        /// invocation writes as `<template>`; [`None`] for every other
        /// command.
        template: Option<String>,
    },

    /// A bare `--routine` name that names both a procedure and a function in a
    /// `--context` document (`FR-RND-032`, with `FR-SCH-010`).
    ///
    /// It is the condition
    /// [`AmbiguousRoutineName`](Error::AmbiguousRoutineName) states over a
    /// catalogue read, met over the other context source of `FR-RND-023`. The
    /// two are separate variants because `FR-ERR-034` forbids a `cause` whose
    /// wording would be equally true of a different failure, and that one's
    /// names a database entry the reader was opened through — which
    /// `FR-RND-019` makes absent here, and `FR-RND-022` makes meaningless: no
    /// connection was opened and no entry was resolved.
    #[error("routine '{name}' names both a procedure and a function in the --context document")]
    AmbiguousRoutineInContext {
        /// The bare name as written.
        name: String,
        /// The document the two objects were found in.
        path: PathBuf,
        /// The database the document describes.
        database: String,
        /// The invocation the token was given to, below `tpl`, as a literal of
        /// `FR-ERR-022`.
        invocation: &'static str,
        /// The template of the `tpl render` the token was given to, which the
        /// invocation writes as `<template>`; [`None`] for every other
        /// command.
        template: Option<String>,
    },

    /// The same `--set` key supplied more than once (`FR-RND-014`).
    ///
    /// It is not the repetition [`RepeatedValueFlag`](Error::RepeatedValueFlag)
    /// refuses: `FR-RND-008` makes `--set` repeatable, so the flag occurring
    /// twice is correct and the **key** occurring twice is not. Both values
    /// are carried because the fault is that two were supplied for one key,
    /// and `FR-ERR-034` obliges the `cause` to say which they were.
    #[error("the --set key '{key}' was given more than once")]
    RepeatedSetKey {
        /// The key, as written (`FR-CLI-020`).
        key: String,
        /// The value of the first occurrence, as written.
        first: String,
        /// The value of the second occurrence, as written.
        second: String,
    },

    /// `--no-cache` given to `tpl cache load` (`FR-CACHE-019`).
    ///
    /// The flag is declared by the command, per `FR-CACHE-017`, and refused by
    /// it: loading without storing is a contradiction between a flag the
    /// command declares and what the command does. Not declaring it would
    /// report the contradiction as an unknown flag, which says the wrong
    /// thing.
    #[error("'--no-cache' cannot be given to 'tpl cache load'")]
    LoadWithoutStoring {
        /// The object the invocation named, as the `schema` subcommand that
        /// reads it (`table`, `view` or `routine`) and the name given, where
        /// it named one.
        object: Option<(&'static str, String)>,
    },

    /// A value that does not conform to the type its parameter declares: a
    /// flag value (`FR-ERR-001`, the `64` row) or a `tpl cfg set` value
    /// (`FR-CFG-010`).
    #[error("'{value}' is not a valid value for '{parameter}'")]
    MalformedValue {
        /// The flag or the configuration key the value was given for.
        parameter: String,
        /// The command path whose help states what the parameter takes,
        /// without the program name: the node that declares the flag, empty
        /// for a global flag, and `cfg set` for a configuration key.
        command: String,
        /// The value as written.
        value: String,
        /// The type that was expected, per `FR-CONF-002` where the parameter
        /// is a configuration key. For a `password_command` supplied as one
        /// string it is instead the condition of `FR-CONF-046` the string met.
        expected: &'static str,
    },

    /// A key supplied to `tpl cfg set` that is outside the enumerated space of
    /// `FR-CONF-002` (`FR-CFG-009`).
    ///
    /// The same key found *in the file* is
    /// [`ConfigurationKeyOutsideSpace`](Error::ConfigurationKeyOutsideSpace)
    /// and `78`: one is a fault in the invocation, the other in `.tpl/.cfg`.
    #[error("unknown configuration key '{key}'")]
    UnknownConfigurationKey {
        /// The key as written.
        key: String,
        /// The nearest matches among the enumerated key space of
        /// `FR-CONF-002`, selected by `FR-ERR-019` and ordered as it fixes.
        ///
        /// Empty where nothing qualified, which `FR-ERR-020` requires to leave
        /// the generic hint standing alone. `FR-CFG-009` obliges the
        /// suggestion.
        nearest: Vec<String>,
    },

    /// `tpl cfg database add` named an entry that already exists
    /// (`FR-CFG-017`).
    ///
    /// `BR-CFG-001` is why this is a refusal rather than a replacement: `add`
    /// creates and `update` changes, and neither silently does the other's job.
    #[error("database entry '{name}' already exists")]
    DatabaseEntryAlreadyExists {
        /// The entry name the invocation asked to create.
        name: String,
        /// The file that already defines it.
        file: PathBuf,
    },

    /// A `cfg` write that would leave a database entry in a combination
    /// `FR-CONF-007` refuses (`FR-CFG-048`).
    ///
    /// The same combination found **in the file** is
    /// [`ConflictingEntryKeys`](Error::ConflictingEntryKeys) and `78`. The two
    /// are the same rule over two different faults: there, `.tpl/.cfg` is what
    /// the caller must repair; here, `.tpl/.cfg` is valid and stays untouched,
    /// and what is refused is the invocation, which the caller wrote and can
    /// rewrite.
    #[error("{}", incoherent_entry_write(entry, written, conflicting))]
    IncoherentEntryWrite {
        /// The entry the write would be applied to.
        entry: String,
        /// The fully qualified key the invocation writes.
        written: String,
        /// The fully qualified key it cannot stand beside — the one the entry
        /// already carries, or the second the same invocation supplies.
        conflicting: String,
        /// Which runnable command makes the write legal.
        repair: EntryRepair,
    },

    /// The destination of `tpl init` names a `.tpl` folder (`FR-PROJ-029`).
    /// Nothing is created.
    #[error("tpl init takes the directory that will hold .tpl, not the .tpl folder")]
    InitDestinationIsTplFolder {
        /// The path as written, or `.` where no operand was given.
        written: PathBuf,
        /// The canonical path, where it is the canonical form whose last
        /// segment is `.tpl` rather than the path as written.
        canonical: Option<PathBuf>,
        /// The directory that holds the folder named, or [`None`] where it is
        /// the current directory.
        parent: Option<PathBuf>,
    },

    /// An entry name given on the command line is outside
    /// `[A-Za-z0-9_]{1,64}` (`FR-CONF-048`). Nothing is written.
    ///
    /// The same name in the file is
    /// [`ConfigurationEntryName`](Error::ConfigurationEntryName) and `78`.
    #[error("{}", invalid_entry_name(*given))]
    InvalidEntryName {
        /// Where the name was given.
        given: EntryNameGiven,
        /// The name as written.
        name: String,
    },

    /// A value given on the command line for a field `FR-CONF-015` expands
    /// holds a reference left unclosed or whose name is not a variable name
    /// (`FR-CONF-049`). Nothing is written.
    #[error("invalid value for {parameter}")]
    InvalidReference {
        /// The flag or the fully qualified key the value was given for.
        parameter: String,
        /// The command path the value was given to, without the program name.
        command: String,
        /// What is wrong with the reference.
        fault: ReferenceFault,
    },

    /// An empty value given on the command line for a host or a database
    /// (`FR-CONF-050`). Nothing is written.
    #[error("invalid value for {parameter}")]
    EmptyValue {
        /// The flag or the fully qualified key the value was given for.
        parameter: String,
        /// The command path the value was given to, without the program name.
        command: String,
    },

    // ---------------------------------------------------------------- 65 ---
    /// A template the engine could not parse (`FR-TMPL-020`, `FR-RND-030`).
    #[error("template '{template}' has a syntax error at {position}")]
    TemplateSyntax {
        /// The template name, as the caller named it.
        template: String,
        /// Where the engine stopped.
        position: Position,
        /// The chain of underlying template-engine errors, outermost first.
        /// `FR-ERR-011` requires it, and it is the one error this type carries
        /// from a dependency, per `OD-06`.
        chain: Vec<String>,
    },

    /// A render that failed at evaluation time — an undefined variable, a
    /// failing filter, an escape from the template root (`FR-RND-031`).
    #[error("{}", render_failed(template, position, reason.as_deref()))]
    RenderFailed {
        /// The template being rendered when the failure arose — the included
        /// one, for a failure inside an `{% include %}`.
        template: String,
        /// The template the render was asked for, as the caller named it. It
        /// differs from `template` only for a failure inside an include, and
        /// it is the one the hint's command renders.
        invoked: String,
        /// The source text of the expression the engine found undefined,
        /// WHERE the failure is an undefined value and the engine located it.
        ///
        /// It is what lets the `cause` name `table` or `vars.title` rather
        /// than "undefined value", and the `hint` name the flag that defines
        /// it — the most common failure a template author meets.
        undefined: Option<String>,
        /// Why the render failed, WHERE more is known than the engine's
        /// chain says: the template ended it with `fail`, or a lookup found
        /// nothing. Boxed, because it is rare and the variant is the size of
        /// every `Result` this crate returns.
        reason: Option<Box<RenderReason>>,
        /// Where evaluation stopped.
        position: Position,
        /// The chain of underlying template-engine errors, outermost first
        /// (`FR-ERR-011`).
        chain: Vec<String>,
    },

    /// A resolved template path that lies outside the template root
    /// (`FR-TMPL-026`), reached without rendering — `tpl template show` on a
    /// symbolic link is the case `FR-TMPL-024` describes.
    #[error("template '{name}' resolves outside the template folder .tpl/templates/")]
    TemplateOutsideRoot {
        /// The template name, as the caller named it.
        name: String,
        /// The root the resolved path had to stay within (`FR-TMPL-023`).
        root: PathBuf,
    },

    /// A `--context` document that is not well-formed JSON or does not match
    /// the document contract (`FR-RND-020`, `FR-ERR-029`).
    #[error("the --context document {} is malformed", context_origin(.path))]
    ContextDocumentMalformed {
        /// The path the document was read from. Boxed, so that the flag
        /// below leaves every `Result` this crate returns no larger.
        path: Box<Path>,
        /// Which half of `FR-RND-020` it failed.
        fault: ContextFault,
        /// Whether `core.database` names an entry, so that the dump the hint
        /// suggests needs no `-d`.
        default_entry: bool,
    },

    /// The render did not finish within its deadline (`FR-RND-033`,
    /// `FR-ERR-027`, `FR-GLOB-013`).
    #[error("the render exceeded {bound} of {limit:?}")]
    RenderDeadlineExceeded {
        /// Which of the two bounds of `FR-GLOB-012` expired.
        bound: DeadlineBound,
        /// The resolved value of that bound.
        limit: Duration,
    },

    /// The render exhausted its render fuel (`FR-RND-036`, `FR-SEC-025`).
    #[error("the render exhausted its render fuel of {fuel} evaluation steps")]
    RenderFuelExhausted {
        /// The resolved value of `core.render_fuel`.
        fuel: u64,
    },

    /// The render would have produced more bytes than its render output limit
    /// (`FR-RND-037`, `FR-SEC-025`).
    #[error("the render exceeded its render output limit of {limit} bytes")]
    RenderOutputLimitExceeded {
        /// The resolved value of `core.render_output_limit`.
        limit: u64,
    },

    /// The process was observed holding more heap than the render memory
    /// limit while the render ran (`FR-RND-039`, `FR-SEC-025`).
    #[error("the render exceeded its render memory limit of {limit} bytes")]
    RenderMemoryLimitExceeded {
        /// The resolved value of `core.render_memory_limit`.
        limit: u64,
    },

    // ---------------------------------------------------------------- 66 ---
    /// A table, view or routine that does not exist in the selected database
    /// (`FR-SCH-013`, `FR-RND-032`).
    #[error("{kind} '{name}' does not exist in database '{database}'")]
    CatalogueObjectNotFound {
        /// The kind the identifier was sought as.
        kind: CatalogueObjectKind,
        /// The identifier that was not found.
        name: String,
        /// The database entry of `.tpl/.cfg` the read was made through — half
        /// of the population `FR-ERR-034` obliges.
        entry: String,
        /// The server-side database it was sought in — the other half.
        database: String,
        /// The nearest matches among the objects of that kind the database
        /// does hold, selected by `FR-ERR-019` and ordered as it fixes. Empty
        /// where nothing qualified, per `FR-ERR-020`. `FR-SCH-010` obliges the
        /// suggestion.
        nearest: Vec<String>,
    },

    /// `tpl cache clean` named an object the cache of the entry does not hold
    /// (`FR-CACHE-040`). Nothing was deleted and no connection was opened.
    #[error("nothing cached for {kind} '{name}' in database entry '{entry}'")]
    NothingCachedNamed {
        /// The kind the object flag named.
        kind: CatalogueObjectKind,
        /// The name it gave.
        name: String,
        /// The database entry whose cache was consulted.
        entry: String,
        /// The nearest matches among the names of that kind the cache holds,
        /// selected by `FR-ERR-019` and `FR-ERR-044`; empty where nothing
        /// qualified.
        nearest: Vec<String>,
        /// The routine kind the name was qualified with — `procedure` or
        /// `function` — WHERE `--routine` wrote one.
        qualified: Option<&'static str>,
        /// The other routine kind, WHERE the cache holds a routine of that
        /// kind under the same name (finding Y-05 of the eighth re-audit of
        /// rmp `#263`).
        held_as: Option<&'static str>,
    },

    /// A table, view or routine that the `--context` document does not carry
    /// (`FR-RND-032`).
    ///
    /// It is the condition
    /// [`CatalogueObjectNotFound`](Error::CatalogueObjectNotFound) states over
    /// a catalogue read, met over the other context source of `FR-RND-023`,
    /// and it is a variant of its own for the reason
    /// [`AmbiguousRoutineInContext`](Error::AmbiguousRoutineInContext) gives:
    /// that one's `cause` names `INFORMATION_SCHEMA` and a database entry, and
    /// `FR-RND-022` opens no connection and `FR-RND-019` resolves no entry, so
    /// neither exists here to be named.
    #[error("{kind} '{name}' does not exist in the --context document")]
    ContextObjectNotFound {
        /// The kind the identifier was sought as.
        kind: CatalogueObjectKind,
        /// The identifier that was not found.
        name: String,
        /// The document it was sought in — the population `FR-ERR-034`
        /// obliges, this path being what stands where a database entry stands
        /// on the catalogue path.
        path: PathBuf,
        /// The database the document describes — the other half.
        database: String,
        /// The nearest matches among the objects of that kind the document
        /// does carry, selected by `FR-ERR-019` and ordered as it fixes. Empty
        /// where nothing qualified, per `FR-ERR-020`. `FR-RND-032` obliges the
        /// suggestion.
        nearest: Vec<String>,
    },

    /// A named template that does not exist under the template root
    /// (`FR-TMPL-027`, `FR-RND-029`).
    #[error("template '{name}' does not exist")]
    TemplateNotFound {
        /// The identifier that was not found.
        name: String,
        /// The population it was sought in (`FR-TMPL-023`).
        root: PathBuf,
        /// The nearest matches among the displayed names of the templates the
        /// project does carry, selected by `FR-ERR-019` and ordered as it
        /// fixes. Empty where nothing qualified, per `FR-ERR-020`, and empty
        /// where the name was written inside a template rather than on the
        /// command line — `FR-TMPL-027` obliges the suggestion for the second
        /// of those and `FR-TMPL-009` answers the first with the engine's own
        /// position instead.
        nearest: Vec<String>,
    },

    /// A `-d/--database` entry that is absent from `.tpl/.cfg`
    /// (`FR-GLOB-007`, `FR-ERR-005`).
    ///
    /// Nothing *selected* is [`NoDatabaseEntrySelected`](Error::NoDatabaseEntrySelected)
    /// and `78`; the rationale of `FR-ERR-005` keeps the two apart.
    #[error("database entry '{name}' does not exist")]
    DatabaseEntryNotFound {
        /// The identifier that was not found.
        name: String,
        /// The file whose entries it was sought among.
        file: PathBuf,
        /// The nearest matches among the entry names the file defines,
        /// selected by `FR-ERR-019` and ordered as it fixes. Empty where
        /// nothing qualified, per `FR-ERR-020`. `FR-GLOB-007` obliges the
        /// suggestion.
        nearest: Vec<String>,
        /// Whether `core.database` named the entry, rather than `-d` or an
        /// argument of the command: the diagnostic then says so, and names the
        /// command that changes the default.
        by_default: bool,
    },

    /// A key that is absent from `.tpl/.cfg` (`FR-CFG-007`, `FR-CFG-012`).
    ///
    /// A spelling outside the key space of `FR-CONF-002` reaches it too, and
    /// the line says which of the two it is (finding T-05 of the third
    /// re-audit of rmp `#263`): "not set" of a name that is no key reads as
    /// though setting it would help.
    #[error("{}", key_not_found(.key, .default.as_deref(), *.known, *.entry_missing))]
    ConfigurationKeyNotFound {
        /// The key that was not found.
        key: String,
        /// Whether the key is one of the space of `FR-CONF-002`.
        known: bool,
        /// Whether the key is a `database.<name>.<field>` key, or the block a
        /// `database.<name>` form names, of an entry the file does not
        /// declare. The line then reports the missing entry, which the
        /// renderers read out of the key, not a missing key (finding
        /// U-05 of the fourth re-audit of rmp `#263`). A flag rather than the
        /// name keeps [`Error`] within the size `clippy::result_large_err`
        /// admits.
        entry_missing: bool,
        /// The value the configuration gives the key where the file sets
        /// none; [`None`] where it has no default or is not a key.
        default: Option<String>,
        /// The file it was sought in.
        file: PathBuf,
        /// The nearest matches over the whole key space of `FR-CONF-002`, the
        /// `<name>` segment bound to every entry the file declares, selected
        /// by `FR-ERR-019` and ordered as it fixes. Empty where nothing
        /// qualified, per `FR-ERR-020`. `FR-CFG-007` obliges the suggestion.
        ///
        /// Each candidate is paired with whether the file sets it: the `hint`
        /// says of one the file does not set that it does not set it
        /// (`FR-CFG-007`, `BR-ERR-004`).
        nearest: Vec<(String, bool)>,
    },

    // ---------------------------------------------------------------- 69 ---
    /// The host name could not be resolved.
    #[error("host '{host}' could not be resolved, for database entry '{entry}'")]
    NameNotResolved {
        /// The database entry the connection was opened for.
        entry: String,
        /// Whether that entry is defined by `dsn`, so that a `hint` repointing
        /// it names `--dsn` (`FR-ERR-045`). Set by [`Error::of_dsn_entry`].
        by_dsn: bool,
        /// The host attempted.
        host: String,
        /// The port attempted.
        port: u16,
    },

    /// The server refused the TCP connection.
    #[error("the server at {host}:{port} refused the connection, for database entry '{entry}'")]
    ConnectionRefused {
        /// The database entry the connection was opened for.
        entry: String,
        /// Whether that entry is defined by `dsn`, so that a `hint` repointing
        /// it names `--dsn` (`FR-ERR-045`). Set by [`Error::of_dsn_entry`].
        by_dsn: bool,
        /// The host attempted.
        host: String,
        /// The port attempted.
        port: u16,
    },

    /// The TLS handshake did not complete under the mode of `FR-CONF-013`.
    ///
    /// The right-hand column of `FR-CONF-038` is the ordinary case: a server
    /// offering no TLS cannot satisfy `required`, `verify-ca` or
    /// `verify-identity`.
    #[error("the TLS handshake with {host}:{port} failed, for database entry '{entry}'")]
    TlsHandshakeFailed {
        /// The database entry the connection was opened for.
        entry: String,
        /// The host attempted.
        host: String,
        /// The port attempted.
        port: u16,
        /// What the handshake returned, as classified.
        fault: TlsFault,
    },

    /// A network phase did not finish within its deadline (`FR-ERR-027`,
    /// `FR-GLOB-013`).
    #[error(
        "{phase} for {host}:{port} exceeded {bound} of {limit:?}, for database entry '{entry}'"
    )]
    NetworkDeadlineExceeded {
        /// The database entry the connection was opened for.
        entry: String,
        /// Whether that entry is defined by `dsn`, so that a `hint` repointing
        /// it names `--dsn` (`FR-ERR-045`). Set by [`Error::of_dsn_entry`].
        by_dsn: bool,
        /// The phase that was in progress.
        phase: NetworkPhase,
        /// The host attempted.
        host: String,
        /// The port attempted.
        port: u16,
        /// Which of the two bounds of `FR-GLOB-012` expired.
        bound: DeadlineBound,
        /// The resolved value of that bound.
        limit: Duration,
    },

    // ---------------------------------------------------------------- 70 ---
    /// An internal invariant was violated and the system declines to continue
    /// past it (`FR-ERR-030`).
    ///
    /// This is one of the two producing conditions of `70` and the only one
    /// that is an error *value*: a panic reaches the same outcome from the
    /// panic site, per `ADR-004`, and never becomes an [`enum@Error`].
    #[error("the invariant '{invariant}' does not hold")]
    InternalInvariant {
        /// The invariant that was violated, as a literal.
        invariant: &'static str,
        /// Where it was detected — the "where" of the `70` row of
        /// `FR-ERR-034`, which is the location the condition arose at.
        location: &'static Location<'static>,
    },

    // ---------------------------------------------------------------- 73 ---
    /// `tpl init` found a `.tpl` folder already at the destination
    /// (`FR-PROJ-014`). Nothing is changed.
    #[error("a .tpl folder already exists at {}", .path.display())]
    ProjectAlreadyExists {
        /// The path `tpl init` could not create.
        path: PathBuf,
    },

    /// `tpl init` could not create the destination (`FR-PROJ-015`).
    #[error("the project at {} could not be created", .path.display())]
    ProjectNotCreated {
        /// The path `tpl init` could not create.
        path: PathBuf,
        /// What the filesystem reported.
        #[source]
        returned: io::Error,
    },

    // ---------------------------------------------------------------- 74 ---
    /// A file this invocation had to read could not be read (`FR-ERR-001`, the
    /// `74` row).
    ///
    /// It is the project's own file in every case but one: `FR-RND-016` reads
    /// a `--context` document the caller named, which may sit anywhere, and a
    /// stream that refused the read is the same condition wherever the path
    /// pointed.
    #[error("{} could not be read", .path.display())]
    ProjectFileUnreadable {
        /// The path that failed.
        path: PathBuf,
        /// What the filesystem returned.
        #[source]
        returned: io::Error,
    },

    /// The document `--context` named could not be read (`FR-RND-016`, the
    /// `74` row of `FR-ERR-001`).
    ///
    /// It is not [`ProjectFileUnreadable`](Error::ProjectFileUnreadable): the
    /// path is the caller's and may sit anywhere, so the hint points at the
    /// flag rather than at the project's permissions.
    #[error("the --context file {} could not be read", .path.display())]
    ContextDocumentUnreadable {
        /// The path `--context` named, or `-` for standard input.
        path: PathBuf,
        /// What the filesystem or the stream returned.
        #[source]
        returned: io::Error,
    },

    /// A file or directory of trust material that `ca_file` or `ca_path`
    /// declares could not be read (`FR-CONF-014`, the `74` row of
    /// `FR-ERR-001`).
    ///
    /// It is not [`ProjectFileUnreadable`](Error::ProjectFileUnreadable): the
    /// path is outside `.tpl`, and the setting that named it is what the caller
    /// corrects, so the message names the setting.
    #[error("the {key} of database entry '{entry}' could not be read: {}", .path.display())]
    TrustMaterialUnreadable {
        /// The entry whose setting it is.
        entry: String,
        /// The setting that named the path: `ca_file` or `ca_path`.
        key: &'static str,
        /// The path that failed — the one declared, or an entry of the
        /// directory `ca_path` declares.
        path: PathBuf,
        /// What the filesystem returned.
        #[source]
        returned: io::Error,
    },

    /// A file of the project could not be written (`FR-ERR-001`, the `74`
    /// row).
    ///
    /// The one file a command of this crate writes outside `tpl init` is
    /// `.tpl/.cfg`, per `FR-CFG-004`, and `FR-CFG-041` writes it by a
    /// temporary file renamed over the target — so this condition can arise on
    /// the temporary file, on the write into it, or on the rename, and in
    /// every case the previous `.cfg` is still in place, unchanged.
    #[error("{} could not be written", .path.display())]
    ProjectFileUnwritable {
        /// The path that failed. It is the target rather than the temporary
        /// file: `FR-CFG-041` makes the temporary an implementation of the
        /// write, and naming it would send the caller to a path that no longer
        /// exists.
        path: PathBuf,
        /// What the filesystem returned.
        #[source]
        returned: io::Error,
    },

    /// Standard output could not be written (`FR-ERR-001`, the `74` row).
    #[error("standard output could not be written")]
    StdoutUnwritable {
        /// What the stream returned.
        #[source]
        returned: io::Error,
    },

    /// Standard output was closed part-way through a JSON document
    /// (`FR-ERR-026`).
    ///
    /// A close with no document in flight is not an error at all: `FR-ERR-025`
    /// makes it a silent success, so it has no variant here.
    #[error("standard output closed part-way through a JSON document")]
    StdoutClosedMidDocument,

    // ---------------------------------------------------------------- 77 ---
    /// The server refused the credentials (`FR-ERR-001`, the `77` row).
    ///
    /// The credential itself never enters the value: `FR-ERR-013` and
    /// `BR-ERR-003` bar it from every message at every verbosity.
    #[error(
        "the server at '{host}' refused authentication for user '{user}', for database entry '{entry}'"
    )]
    AuthenticationRefused {
        /// The database entry the connection was opened for.
        entry: String,
        /// The user the server refused.
        user: String,
        /// The host that refused it.
        host: String,
    },

    /// A property of an object requested by name could not be read, so the
    /// object is incomplete and is not returned in part (`FR-PRIV-003`,
    /// `FR-PRIV-004`, `FR-PRIV-013`).
    ///
    /// **It carries `FR-PRIV-021` as well**, with `kind` set to
    /// [`CatalogueObjectKind::Database`], `object` the database the selected
    /// entry names per `FR-CONF-041`, and `property` the literal `metadata`.
    /// That requirement obliges exactly what this variant already states — the
    /// database named, and that its metadata could not be read — and its
    /// `77` is this one, per the `77` row of `FR-ERR-034`, which puts *which
    /// property of which object* on the privilege side of the row.
    ///
    /// *A variant of its own was rejected*: it would say the same three things
    /// through a fourth field, and `FR-PRIV-021` records that the condition
    /// admits two explanations it does not separate — a reader who may not see
    /// the database, and a database that is not there — so there is nothing a
    /// second variant could carry that this one cannot.
    #[error("the {property} of {kind} '{object}' could not be read")]
    PropertyNotReadable {
        /// The kind of the object.
        kind: CatalogueObjectKind,
        /// The object that was requested by name.
        object: String,
        /// Which property could not be read, as a literal — `FR-PRIV-013`
        /// obliges the `cause` line to state it.
        property: &'static str,
    },

    // ---------------------------------------------------------------- 78 ---
    /// The walk reached the boundary of `FR-PROJ-005` without finding a `.tpl`
    /// folder (`FR-PROJ-006`).
    #[error("no .tpl project found")]
    ProjectNotFound {
        /// The directory the walk ended at — what the `78` row of
        /// `FR-ERR-034` obliges for this condition.
        walk_ended_at: PathBuf,
    },

    /// `--tpl-dir` named a path that does not exist or is not a directory
    /// (`FR-PROJ-008`).
    ///
    /// No walk was made — the flag suppresses it — so the condition is not
    /// [`ProjectNotFound`](Error::ProjectNotFound), whose `cause` describes a
    /// walk and whose hint creates a project in the working directory.
    #[error("{}", tpl_dir_unusable(.path, *.fault))]
    ProjectDirUnusable {
        /// The path `--tpl-dir` named, as the caller wrote it.
        path: PathBuf,
        /// Why it is not usable.
        fault: TplDirFault,
    },

    /// The project's `.tpl` folder holds no `.cfg`, and the folder is owned by
    /// another user (`FR-PROJ-028`, `FR-SEC-014`).
    ///
    /// With no file, [`ConfigurationNotOwned`](Error::ConfigurationNotOwned)
    /// has nothing to check, and a planted folder would otherwise supply
    /// templates and receive the `.cfg` the caller's next `tpl cfg` writes.
    #[error("the .tpl folder at {} cannot be used as a project", .path.display())]
    ProjectFolderNotOwned {
        /// The `.tpl` folder, canonical per `FR-PROJ-009`.
        path: PathBuf,
        /// The owner found.
        owner: u32,
        /// The owner expected — the invoking user.
        expected: u32,
    },

    /// `.tpl/.cfg` declares `ca_file` or `ca_path` with a value that contains
    /// `${` (`FR-CONF-047`).
    ///
    /// Neither key is expanded, so the value would be read as a file named
    /// after the reference. The same value supplied to a command is
    /// [`MalformedValue`](Error::MalformedValue) and `64`.
    #[error("{key} holds a ${{VAR}} reference, which tpl does not expand in this key")]
    ConfigurationPathReference {
        /// The fully qualified key, `database.<name>.ca_file` or
        /// `database.<name>.ca_path`.
        key: String,
        /// The file that declares it.
        file: PathBuf,
        /// Where the value begins.
        position: Position,
        /// The value as written.
        value: String,
    },

    /// `.tpl/.cfg` is not owned by the current user (`FR-PROJ-010`).
    #[error("{} is not owned by the current user", .path.display())]
    ConfigurationNotOwned {
        /// The file that failed the check.
        path: PathBuf,
        /// The owner found.
        owner: u32,
        /// The owner expected — the invoking user.
        expected: u32,
    },

    /// `.tpl/.cfg` grants access to group or other (`FR-PROJ-011`).
    #[error("{} has unsafe permissions", .path.display())]
    ConfigurationUnsafeMode {
        /// The file that failed the check.
        path: PathBuf,
        /// The permission bits found.
        mode: u32,
    },

    /// `.tpl/.cfg` is not valid TOML (`FR-ERR-001`, the `78` row).
    #[error("{} is not valid TOML", .path.display())]
    ConfigurationMalformed {
        /// The file that could not be parsed.
        path: PathBuf,
        /// Where the parser stopped.
        position: Position,
        /// What the parser expected there, in its own words, with the source
        /// excerpt it would quote removed so that no byte of the file is
        /// written back (`BR-ERR-003`).
        reason: String,
    },

    /// `.tpl/.cfg` carries a key outside the enumerated space of
    /// `FR-CONF-002` (`FR-CONF-034`).
    #[error("{} declares the unknown key '{key}'", .file.display())]
    ConfigurationKeyOutsideSpace {
        /// The key found.
        key: String,
        /// The file that declares it.
        file: PathBuf,
        /// Where the file writes the key, so that the `hint` names the line to
        /// delete, as the TOML errors do (finding U-08 of the fourth re-audit
        /// of rmp `#263`).
        position: Position,
        /// The nearest matches among the enumerated key space of
        /// `FR-CONF-002`, selected by `FR-ERR-019` and ordered as it fixes.
        /// Empty where nothing qualified, per `FR-ERR-020`. `FR-CONF-034`
        /// obliges the suggestion.
        nearest: Vec<String>,
    },

    /// A value in `.tpl/.cfg` that does not conform to the type `FR-CONF-002`
    /// declares for its key (`FR-ERR-001`, the `78` row).
    ///
    /// The `78` row of `FR-ERR-034` obliges the `cause` line to name "the key
    /// and the file, with the value found and the value expected", and
    /// the `found` field below states how that is reconciled with
    /// `FR-ERR-013`.
    #[error("{key} is not {expected}")]
    ConfigurationValueMalformed {
        /// The fully qualified key.
        key: String,
        /// The file that declares it.
        file: PathBuf,
        /// Where in that file the declaration is.
        position: Position,
        /// What was found: the value as written, or — for a key whose value
        /// may be a credential — the TOML type of that value alone.
        ///
        /// `FR-ERR-013` bars a credential from every message at every
        /// verbosity, and a key's secrecy is a property of the key space, so
        /// the choice is made where the condition is raised rather than here.
        found: String,
        /// The type `FR-CONF-002` declares for the key.
        expected: &'static str,
        /// The value as the file wrote it, where `found` is what a `${VAR}`
        /// expanded it to rather than what the file holds; [`None`] where the
        /// file holds `found` itself (finding T-04 of the third re-audit of
        /// rmp `#263`). Boxed behind a thin pointer, so that the variant
        /// leaves [`Error`] no larger: a `Box<str>` is two words wide.
        expanded_from: Option<Box<String>>,
    },

    /// A DSN that is not of the form `FR-CONF-009` fixes, or whose scheme is
    /// not one of the two `FR-CONF-010` accepts.
    ///
    /// The DSN itself is not carried: `BR-ERR-003` bars the resolved DSN from
    /// every message, and the key locates the fault without it. A DSN carrying
    /// a query parameter is [`DsnQueryParameter`](Error::DsnQueryParameter)
    /// instead, because `FR-CONF-011` and `FR-CONF-012` state that condition
    /// separately.
    #[error("{key} is not a valid connection URL")]
    DsnMalformed {
        /// The fully qualified key whose value is at fault.
        key: String,
        /// The file that declares it.
        file: PathBuf,
        /// Which half of the grammar it failed.
        fault: DsnFault,
    },

    /// A `${` with no closing brace (`FR-CONF-021`).
    #[error("{key} carries an unclosed ${{ expansion")]
    UnclosedExpansion {
        /// The fully qualified key whose value carries it.
        key: String,
        /// The file that declares it.
        file: PathBuf,
    },

    /// A `${NAME}` whose name is not `[A-Za-z_][A-Za-z0-9_]*`, met where the
    /// field is expanded (`FR-CONF-049`).
    #[error("{key} carries a ${{VAR}} reference whose name is not a variable name")]
    InvalidReferenceName {
        /// The fully qualified key whose value carries it.
        key: String,
        /// The file that declares it.
        file: PathBuf,
        /// The name as written between the braces.
        name: String,
    },

    /// `.tpl/.cfg` declares a `[database.<name>]` block, or a `core.database`
    /// value, outside `[A-Za-z0-9_]{1,64}` (`FR-CONF-048`).
    #[error("{}", configuration_entry_name(.file, *core))]
    ConfigurationEntryName {
        /// The file that declares it.
        file: PathBuf,
        /// The name as written.
        name: String,
        /// Whether it is the value of `core.database` rather than the name of
        /// a block.
        core: bool,
        /// Where the file writes it.
        position: Position,
    },

    /// `password_command` is stored as something other than an array of
    /// strings (`FR-CONF-035`).
    #[error("{}", password_command_not_an_array(key, found, *element))]
    PasswordCommandNotAnArray {
        /// The fully qualified key.
        key: String,
        /// The file that declares it.
        file: PathBuf,
        /// Where in that file the declaration is. `FR-CONF-035` names a line
        /// in its own `cause`, and one file may declare this key for several
        /// entries, so the position is what locates the declaration at fault.
        position: Position,
        /// The TOML type found, against the array of `FR-CONF-023` expected.
        found: &'static str,
        /// The zero-based index of the element that is not a string, WHERE
        /// the value is an array and one of its elements is the fault.
        element: Option<usize>,
    },

    /// One entry declares two keys that exclude one another (`FR-CONF-006`,
    /// `FR-CONF-007`).
    #[error("{}", conflicting_entry_keys(entry, first, second))]
    ConflictingEntryKeys {
        /// The entry name.
        entry: String,
        /// The file that declares it.
        file: PathBuf,
        /// The first of the two keys.
        first: String,
        /// The second of the two keys.
        second: String,
    },

    /// A DSN carries a query parameter (`FR-CONF-011`, `FR-CONF-012`).
    ///
    /// The DSN itself is not carried: `BR-ERR-003` bars the resolved DSN from
    /// every message, and the key locates the fault without it.
    #[error("{key} carries a query parameter")]
    DsnQueryParameter {
        /// The fully qualified key whose value carries the `?`.
        key: String,
        /// The file that declares it.
        file: PathBuf,
    },

    /// A `${VAR}` reference names an environment variable that is not defined
    /// (`FR-CONF-022`, `FR-SEC-010`).
    #[error("the environment variable '{name}' referenced by {key} is not defined")]
    UndefinedVariable {
        /// The name of the undefined variable — what the `78` row of
        /// `FR-ERR-034` obliges for this condition.
        name: String,
        /// The fully qualified key that references it.
        key: String,
        /// The file that declares it.
        file: PathBuf,
    },

    /// `password_command` did not finish within its deadline — it had not
    /// both exited and reached end of file on its standard output — and its
    /// process group was terminated (`FR-CONF-028`, `FR-ERR-027`).
    #[error("the password_command of database entry '{entry}' exceeded {bound} of {limit:?}")]
    PasswordCommandDeadlineExceeded {
        /// The database entry that declares the command.
        entry: String,
        /// The command as stored, which `FR-CONF-017` guarantees carries no
        /// expanded value and therefore no secret.
        command: Vec<String>,
        /// Which of the two bounds of `FR-GLOB-012` expired.
        bound: DeadlineBound,
        /// The resolved value of that bound.
        limit: Duration,
    },

    /// `password_command` wrote more than the cap of `FR-CONF-031` to standard
    /// output, and its process group was terminated.
    #[error("the password_command of database entry '{entry}' wrote more than {cap} bytes")]
    PasswordCommandOutputCapExceeded {
        /// The database entry that declares the command.
        entry: String,
        /// The command as stored (`FR-CONF-017`).
        command: Vec<String>,
        /// The cap, in bytes — `FR-CONF-031` obliges the `cause` line to name
        /// it beside the command, and never the bytes read.
        cap: usize,
    },

    /// `password_command` yielded no exit status (`FR-CONF-042`).
    ///
    /// `FR-CONF-033` routes a child that **exits** non-zero to `78`, on the
    /// ground that the configured way of obtaining a password failed to produce
    /// one; a child that never starts, and a child whose status the system
    /// cannot read, fail in the same way and for the same reason, and are a
    /// condition of their own because the next step differs — a program that is
    /// absent or not executable is corrected in the file, not by running the
    /// command to see what it printed.
    ///
    /// The two are reached from two places and [`PasswordCommandFault`] says
    /// which, because `FR-CONF-042` obliges the `cause` line to: one wording
    /// for both is what that requirement was written over, and `FR-ERR-002`
    /// forbids it.
    #[error("the password_command of database entry '{entry}' {}", password_command_unusable(*.fault))]
    PasswordCommandNotExecutable {
        /// The database entry that declares the command.
        entry: String,
        /// The command as stored (`FR-CONF-017`).
        command: Vec<String>,
        /// Which of the two conditions of `FR-CONF-042` arose.
        fault: PasswordCommandFault,
        /// What the operating system returned.
        #[source]
        returned: io::Error,
    },

    /// `password_command` exited non-zero (`FR-CONF-033`), or was ended by a
    /// signal `tpl` did not send (`FR-CONF-043`).
    ///
    /// Its standard error is not carried because it was never captured:
    /// `FR-CONF-032` sends it to the null device.
    #[error("the password_command of database entry '{entry}' did not exit successfully")]
    PasswordCommandFailed {
        /// The database entry that declares the command.
        entry: String,
        /// The command as stored (`FR-CONF-017`).
        command: Vec<String>,
        /// How the child ended, which decides which of the two requirements
        /// owns the condition and what its `cause` line names.
        end: ChildEnd,
    },

    /// `ca_path` names a directory that yields no certificate file
    /// (`FR-CONF-044`).
    ///
    /// The condition is decided while the trust material is assembled and
    /// before any connection is opened, so it costs no round trip; it is
    /// *no entry resolves to a regular file*, and a regular file that is empty
    /// or holds no PEM block is not it. It is per key, and fires whether or not
    /// `ca_file` is declared beside it.
    ///
    /// `78` and not `69` — nothing was contacted — and not `74` — nothing
    /// failed to be read; the directory was read and is empty of what the key
    /// promises.
    #[error("the ca_path of database entry '{entry}' supplies no certificate file")]
    TrustDirectoryEmpty {
        /// The entry whose `ca_path` it is.
        entry: String,
        /// The directory as the entry declared it (`FR-CONF-044`).
        path: PathBuf,
    },

    /// The read-only session could not be established or confirmed
    /// (`FR-SRV-010`). The catalogue is not read.
    #[error("the read-only session could not be enforced for database entry '{entry}'")]
    ReadOnlySessionNotEnforced {
        /// The entry whose connection it was.
        entry: String,
        /// Whether that entry is defined by `dsn`, so that a `hint` repointing
        /// it names `--dsn` (`FR-ERR-045`). Set by [`Error::of_dsn_entry`].
        by_dsn: bool,
        /// Which of the two conditions of `FR-SRV-010` arose.
        fault: ReadOnlyFault,
    },

    /// The selected entry does not carry a key the invocation needs
    /// (`FR-CONF-040`, `FR-CONF-041`).
    ///
    /// Two requirements produce it and both are decided where `.tpl/.cfg` is
    /// open, which is entry resolution: `FR-CONF-040` for an entry that names
    /// no host, and `FR-CONF-041` for one that names no database while the
    /// invocation reads the catalogue. `FR-CONF-040` rejects composing either
    /// refusal further down, where neither the file nor the position that
    /// declared the entry is in hand.
    #[error("{}", entry_key_missing(.entry, .key))]
    EntryKeyMissing {
        /// The entry that carries neither the key nor the alternative.
        entry: String,
        /// The key the entry does not carry, fully qualified —
        /// `database.<name>.host` or `database.<name>.database`.
        key: String,
        /// The file the entry is declared in; the `78` row of `FR-ERR-034`
        /// obliges the `cause` to name it.
        file: PathBuf,
        /// The flag of `FR-CFG-027` that writes the key, as a literal of
        /// `FR-ERR-022` — `--host` or `--schema`.
        flag: &'static str,
        /// The placeholder the hint writes after that flag, as a literal.
        placeholder: &'static str,
        /// Whether the key is absent, empty, or expands to the empty string
        /// (`FR-CONF-050`).
        absence: KeyAbsence,
    },

    /// The command requires a database entry and none is selected — neither
    /// `-d/--database` nor `core.database` (`FR-ERR-004`, `FR-GLOB-006`).
    #[error("no database entry is selected")]
    NoDatabaseEntrySelected {
        /// The file the selection would have come from; `FR-GLOB-006` obliges
        /// the message to name it.
        file: PathBuf,
        /// Whether that file declares any entry at all; where it declares
        /// none, there is nothing to select and the hint says how to add one.
        has_entries: bool,
    },

    /// The server is reachable and authenticated and is not MariaDB
    /// (`FR-SRV-003`). The catalogue is not read.
    #[error("the server reached by database entry '{entry}' is not MariaDB")]
    ServerNotMariaDb {
        /// The entry that reached it.
        entry: String,
        /// Whether that entry is defined by `dsn`, so that a `hint` repointing
        /// it names `--dsn` (`FR-ERR-045`). Set by [`Error::of_dsn_entry`].
        by_dsn: bool,
        /// The product the server reported.
        product: String,
    },

    /// The server is MariaDB of a series below the window of `FR-SRV-015`
    /// (`FR-SRV-020`, `FR-SRV-030`). The catalogue is not read.
    #[error("server series '{series}' is not supported, for database entry '{entry}'")]
    SeriesNotSupported {
        /// The entry that reached it.
        entry: String,
        /// Whether that entry is defined by `dsn`, so that a `hint` repointing
        /// it names `--dsn` (`FR-ERR-045`). Set by [`Error::of_dsn_entry`].
        by_dsn: bool,
        /// The series found, as the server reported it.
        series: String,
        /// The series that are supported, as the caller that raised this
        /// condition holds them.
        ///
        /// `FR-SRV-030` obliges the `cause` line to list them and
        /// `BR-SRV-005` states that set exactly once, elsewhere, so neither
        /// this module nor the renderer may keep a copy: the window travels on
        /// the value and the renderer prints what it is handed. The `'static`
        /// bound is deliberate. `FR-SRV-020` rejects a flag or a configuration
        /// key that overrides the window, so the only admissible source is a
        /// table compiled into the binary, and the type admits no other.
        supported: &'static [&'static str],
    },
}

/// The `error:` line of [`Error::UnknownCommand`]: the token, and the group it
/// was sought under where that is not the root.
fn unknown_command(token: &str, node: &str) -> String {
    // W-05 of the sixth re-audit of rmp `#263`: a token holding a space is a
    // command path given as one argument, and the line reads as the one
    // `tpl help` writes for the same token, which names the node.
    if node.is_empty() && !token.contains(char::is_whitespace) {
        format!("unknown command '{token}'")
    } else {
        format!(
            "unknown command '{token}' under '{}'",
            crate::diagnostics::invoked(node)
        )
    }
}

/// The `error:` line of [`Error::ConfigurationKeyNotFound`]: a key of the
/// space the file does not set, with its default where it has one, or a name
/// that is no key at all.
///
/// A key of the space is said to be one, so that a caller who spelt it right
/// is not left wondering whether it did (finding U-01 of the fourth re-audit of
/// rmp `#263`); a name that reaches an entry the file does not declare says
/// that the entry is what is missing (finding U-05).
///
/// `core` and `database` reach it only from `tpl cfg unset`, where the file
/// has no such section: the line says there is nothing to remove, rather than
/// that a block of the space is no key (`FR-CFG-012`).
fn key_not_found(key: &str, default: Option<&str>, known: bool, entry_missing: bool) -> String {
    if let Some(section) = section_named(key) {
        return format!("{section}, and .tpl/.cfg has none, so there is nothing to remove");
    }
    match (entry_missing, known, default) {
        (true, _, _) => format!(
            "database entry '{}' does not exist, so .tpl/.cfg holds no '{key}'",
            named_entry(key)
        ),
        (false, false, _) => format!("'{key}' is not a configuration key"),
        (false, true, Some(default)) => format!(
            "'{key}' is a configuration key that .tpl/.cfg does not set; tpl uses its default, \
             {default}"
        ),
        (false, true, None) => format!(
            "'{key}' is a configuration key that .tpl/.cfg does not set, and it has no default"
        ),
    }
}

/// The entry a `database.<name>.<field>` key or a `database.<name>` block
/// names: the segment between `database.` and the field, or everything after
/// `database.` for a block.
pub(crate) fn named_entry(key: &str) -> &str {
    let rest = key.strip_prefix("database.").unwrap_or(key);
    if crate::project::config::keys::Key::parse(key).is_some() {
        rest.rsplit_once('.').map_or(rest, |(entry, _)| entry)
    } else {
        rest
    }
}

/// What `core` or `database` names, as the lines of a block say it, or
/// [`None`] for any other key.
pub(crate) fn section_named(key: &str) -> Option<&'static str> {
    match key {
        "core" => Some("'core' names the [core] section"),
        "database" => Some("'database' names every [database.<name>] block"),
        _ => None,
    }
}

/// The `error:` line of [`Error::BlockKeyGiven`], in the words `FR-CFG-007`
/// shows: a `database.<name>` form is a whole entry, said not to exist where
/// the file does not declare it, and `core` and `database` are sections.
fn block_key(key: &str, entry: bool) -> String {
    if entry {
        return format!("'{key}' names a whole entry, not one value");
    }
    match key.strip_prefix("database.") {
        Some(name) => format!(
            "'{key}' names a whole entry, not one value, and database entry '{name}' does not \
             exist"
        ),
        None => format!("'{key}' names a whole section, not one value"),
    }
}

/// The `error:` line of [`Error::ProjectDirUnusable`], in the words
/// `FR-PROJ-008` shows.
fn tpl_dir_unusable(path: &std::path::Path, fault: TplDirFault) -> String {
    match fault {
        TplDirFault::Missing => format!(
            "the folder named by --tpl-dir does not exist: {}",
            path.display()
        ),
        TplDirFault::NotDirectory => format!(
            "the path named by --tpl-dir is not a folder: {}",
            path.display()
        ),
        TplDirFault::HoldsTplFolder | TplDirFault::NotTplFolder => format!(
            "--tpl-dir names {}, which is not a .tpl folder",
            path.display()
        ),
    }
}

/// The `error:` line of [`Error::PasswordCommandNotExecutable`], which states
/// which of the two conditions of `FR-CONF-042` arose: "yielded no exit
/// status" is true of both and was the only thing the line said, while the
/// `cause` beneath it said the command had never started.
const fn password_command_unusable(fault: PasswordCommandFault) -> &'static str {
    match fault {
        PasswordCommandFault::NotStarted => "could not be started",
        PasswordCommandFault::StatusUnreadable => "ended with a status tpl could not read",
    }
}

/// Whether a pair of entry keys `FR-CONF-007` refuses is two sources of the
/// password — `password_command` beside a DSN that carries one, or beside
/// `password` — rather than two forms of the connection.
pub(crate) fn password_pair(first: &str, second: &str) -> bool {
    let leaf = |key: &'_ str| key.rsplit('.').next().unwrap_or_default().to_owned();
    let (first, second) = (leaf(first), leaf(second));
    let other = match (first.as_str(), second.as_str()) {
        ("password_command", other) | (other, "password_command") => other.to_owned(),
        _ => return false,
    };

    matches!(other.as_str(), "dsn" | "password")
}

/// The `error:` line of [`Error::ConflictingEntryKeys`]: two sources of the
/// password are the password given twice, and two forms of the connection
/// are two keys declared together.
fn conflicting_entry_keys(entry: &str, first: &str, second: &str) -> String {
    if password_pair(first, second) {
        let (source, command) = if first.ends_with(".password_command") {
            (second, first)
        } else {
            (first, second)
        };
        let gives = if source.ends_with(".dsn") {
            "carries a password"
        } else {
            "gives one"
        };
        format!(
            "database entry '{entry}' gets its password twice: {source} {gives}, and {command} \
             gives another"
        )
    } else {
        format!("database entry '{entry}' declares both {first} and {second}")
    }
}

/// The `error:` line of [`Error::IncoherentEntryWrite`]: a DSN beside
/// `password_command` is refused only because the DSN carries a password, and
/// the line says so rather than state a rule that is false of a DSN without
/// one.
fn incoherent_entry_write(entry: &str, written: &str, conflicting: &str) -> String {
    if password_pair(written, conflicting) {
        let (source, command) = if written.ends_with(".password_command") {
            (conflicting, written)
        } else {
            (written, conflicting)
        };
        let gives = if source.ends_with(".dsn") {
            "carries a password"
        } else {
            "gives one"
        };
        format!(
            "database entry '{entry}' would get its password twice: {source} {gives}, and \
             {command} gives another"
        )
    } else {
        format!("database entry '{entry}' cannot declare both {written} and {conflicting}")
    }
}

/// The `error:` line of [`Error::RenderFailed`]: the author's own message
/// where the template ended the render with `fail` (`FR-SEM-015`), and the
/// template and the position otherwise.
fn render_failed(template: &str, position: &Position, reason: Option<&RenderReason>) -> String {
    match reason {
        Some(RenderReason::Failed(message)) => {
            format!("template '{template}' called fail() at {position}: {message}")
        }
        Some(
            RenderReason::Unresolved(_)
            | RenderReason::Missing(_)
            | RenderReason::IncludeNotFound { .. },
        )
        | None => {
            format!("rendering template '{template}' failed at {position}")
        }
    }
}

/// Where the `--context` document was read from, as the `error:` line says
/// it: `on standard input` for `-`, and the quoted path otherwise.
fn context_origin(path: &std::path::Path) -> String {
    if path == std::path::Path::new("-") {
        "on standard input".to_owned()
    } else {
        format!("'{}'", path.display())
    }
}

/// How a diagnostic names the `--context` document read from `path`: `-` is
/// standard input, which a reader would not recognise in the quoted `'-'`.
pub(crate) fn context_name(path: &std::path::Path) -> String {
    if path == std::path::Path::new("-") {
        "standard input".to_owned()
    } else {
        format!("'{}'", path.display())
    }
}

/// The `error:` line of [`Error::PasswordCommandNotAnArray`]: an empty array is
/// an array, so the line says what is wrong with it instead.
fn password_command_not_an_array(key: &str, found: &str, element: Option<usize>) -> String {
    if let Some(index) = element {
        format!("{key} holds a non-string element at index {index}")
    } else if found == crate::project::config::EMPTY_ARRAY {
        format!("{key} is an empty array")
    } else {
        format!("{key} is not an array")
    }
}

/// The `error:` line of [`Error::InvalidEntryName`].
fn invalid_entry_name(given: EntryNameGiven) -> &'static str {
    match given {
        EntryNameGiven::Add => "invalid entry name for tpl cfg database add",
        EntryNameGiven::Key(_) => "invalid entry name in the key given to tpl cfg set",
        EntryNameGiven::CoreDatabase => "invalid value for core.database",
    }
}

/// The `error:` line of [`Error::ConfigurationEntryName`].
fn configuration_entry_name(file: &std::path::Path, core: bool) -> String {
    if core {
        format!(
            "{} sets core.database to a value that is not an entry name",
            file.display()
        )
    } else {
        format!(
            "{} declares a database entry with an invalid name",
            file.display()
        )
    }
}

/// The `error:` line of [`Error::EntryKeyMissing`].
///
/// `database.<name>.database` names the word three times over, which is how the
/// entry, the key and the server-side database came to read as one thing; the
/// line says which of the two keys it is in words, and gives the key after.
fn entry_key_missing(entry: &str, key: &str) -> String {
    let what = if key.ends_with(".database") {
        "no server database name"
    } else if key.ends_with(".host") {
        "no host"
    } else {
        "a required key missing"
    };

    format!("database entry '{entry}' has {what} (key {key})")
}

/// Checks an internal invariant, and reports `FR-ERR-030` where it does not
/// hold.
///
/// This is the guard `FR-ERR-031` speaks of: the one place that decides a
/// violated invariant is a `70` and the one that names where it was detected.
/// `#[track_caller]` is what makes the location the **call site's** rather than
/// this function's, which is the "where" the `70` row of `FR-ERR-034` obliges.
///
/// The guard is not an assertion: it returns the condition rather than raising
/// one, so a caller declines to continue past it through the same `?` every
/// other condition of `FR-ERR-006` travels, and the status is derived once, by
/// [`Error::exit_code`].
///
/// # Errors
///
/// Returns [`Error::InternalInvariant`] naming `invariant` and the call site
/// when `holds` is `false`.
#[track_caller]
pub(crate) fn ensure_invariant(holds: bool, invariant: &'static str) -> Result<(), Error> {
    if holds {
        Ok(())
    } else {
        Err(Error::InternalInvariant {
            invariant,
            location: Location::caller(),
        })
    }
}

/// The deliberate trigger for `70` that `FR-ERR-031` requires.
///
/// `70` cannot be reached from a correct invocation by definition, so without a
/// trigger it is a code nobody has confirmed the binary can return. The
/// requirement rejects by name every mechanism that would reach it from outside
/// the process — a command or a flag, an environment variable, a build selected
/// by a feature — so the trigger is inside the process and reachable from
/// nothing a caller can write.
///
/// `#[cfg(test)]` is that reachability rule, per `OD-21`: the item is not
/// compiled into the artefact `cargo build` produces, and an integration test
/// links the library compiled without that configuration and cannot see it
/// either. Its test is therefore a unit test in this crate, which is the
/// composition `BR-ERR-001` admits for `70` alone.
///
/// The invariant it violates is its own and guards nothing: a trigger that
/// broke a real one would report a defect the caller might actually have.
///
/// # Errors
///
/// Always. That is what it is for.
#[cfg(test)]
pub(crate) fn trigger_internal_invariant() -> Result<(), Error> {
    // Written as `false` rather than as an expression a reader must evaluate:
    // the invariant of the trigger is that the trigger never fires, and the
    // trigger exists to falsify it.
    ensure_invariant(false, "the deliberate trigger of FR-ERR-031 never fires")
}

impl Error {
    /// The same condition, marked as raised for an entry defined by `dsn`
    /// where it is one whose `hint` repoints the entry (`FR-ERR-045`).
    ///
    /// The conditions are raised below the layer that knows the entry's form,
    /// so the one function that opens a connection marks what it returns. Any
    /// other condition is returned unchanged.
    #[must_use]
    pub(crate) fn of_dsn_entry(mut self) -> Self {
        match &mut self {
            Self::NameNotResolved { by_dsn, .. }
            | Self::ConnectionRefused { by_dsn, .. }
            | Self::NetworkDeadlineExceeded { by_dsn, .. }
            | Self::ReadOnlySessionNotEnforced { by_dsn, .. }
            | Self::ServerNotMariaDb { by_dsn, .. }
            | Self::SeriesNotSupported { by_dsn, .. } => *by_dsn = true,
            _ => {}
        }
        self
    }

    /// The process exit status this condition produces, per `FR-ERR-001`.
    ///
    /// The match is exhaustive and carries no wildcard arm, which is what makes
    /// a new variant without a code a compile error rather than a wrong status
    /// — the property `OD-06` chose an inherent method in the defining crate
    /// to obtain. `main.rs` returns what this yields and classifies nothing of
    /// its own.
    ///
    /// The value is one of the nine non-zero codes of `FR-ERR-001`. The tenth,
    /// `0`, is success and has no variant: it is not a condition this type
    /// reports, and the silent close of `FR-ERR-025` is a success too.
    ///
    /// # Examples
    ///
    /// ```
    /// use tpl::Error;
    ///
    /// let error = Error::UnknownCommand {
    ///     token: "sch".to_owned(),
    ///     node: String::new(),
    ///     nearest: Vec::new(),
    /// };
    /// assert_eq!(error.exit_code(), 64);
    /// assert_eq!(error.to_string(), "unknown command 'sch'");
    /// ```
    #[must_use]
    pub const fn exit_code(&self) -> u8 {
        match self {
            // 64 EX_USAGE
            Self::UnknownCommand { .. }
            | Self::UnknownCommandPathSegment { .. }
            | Self::UnknownFlag { .. }
            | Self::UnexpectedArgument { .. }
            | Self::RepeatedValueFlag { .. }
            | Self::RepeatedFlag { .. }
            | Self::FlagValueMissing { .. }
            | Self::FlagTookCommand { .. }
            | Self::SeparateTokenValue { .. }
            | Self::ValueOutsideEnumeration { .. }
            | Self::InvocationRejected { .. }
            | Self::MissingArgument { .. }
            | Self::MutuallyExclusiveFlags { .. }
            | Self::DirectWithContext
            | Self::PrettyWithoutJson { .. }
            | Self::ConnectionDetailsMissing { .. }
            | Self::NothingToUpdate { .. }
            | Self::BlockKeyGiven { .. }
            | Self::RoutinePrefixNotLowerCase { .. }
            | Self::AmbiguousRoutineName { .. }
            | Self::AmbiguousRoutineInContext { .. }
            | Self::RepeatedSetKey { .. }
            | Self::LoadWithoutStoring { .. }
            | Self::MalformedValue { .. }
            | Self::UnknownConfigurationKey { .. }
            | Self::DatabaseEntryAlreadyExists { .. }
            | Self::IncoherentEntryWrite { .. }
            | Self::InitDestinationIsTplFolder { .. }
            | Self::InvalidEntryName { .. }
            | Self::InvalidReference { .. }
            | Self::EmptyValue { .. } => 64,

            // 65 EX_DATAERR
            Self::TemplateSyntax { .. }
            | Self::RenderFailed { .. }
            | Self::TemplateOutsideRoot { .. }
            | Self::ContextDocumentMalformed { .. }
            | Self::RenderDeadlineExceeded { .. }
            | Self::RenderFuelExhausted { .. }
            | Self::RenderOutputLimitExceeded { .. }
            | Self::RenderMemoryLimitExceeded { .. } => 65,

            // 66 EX_NOINPUT
            Self::NothingCachedNamed { .. }
            | Self::CatalogueObjectNotFound { .. }
            | Self::ContextObjectNotFound { .. }
            | Self::TemplateNotFound { .. }
            | Self::DatabaseEntryNotFound { .. }
            | Self::ConfigurationKeyNotFound { .. } => 66,

            // 69 EX_UNAVAILABLE
            Self::NameNotResolved { .. }
            | Self::ConnectionRefused { .. }
            | Self::TlsHandshakeFailed { .. }
            | Self::NetworkDeadlineExceeded { .. } => 69,

            // 70 EX_SOFTWARE
            Self::InternalInvariant { .. } => 70,

            // 73 EX_CANTCREAT
            Self::ProjectAlreadyExists { .. } | Self::ProjectNotCreated { .. } => 73,

            // 74 EX_IOERR
            Self::ProjectFileUnreadable { .. }
            | Self::ContextDocumentUnreadable { .. }
            | Self::TrustMaterialUnreadable { .. }
            | Self::ProjectFileUnwritable { .. }
            | Self::StdoutUnwritable { .. }
            | Self::StdoutClosedMidDocument => 74,

            // 77 EX_NOPERM
            Self::AuthenticationRefused { .. } | Self::PropertyNotReadable { .. } => 77,

            // 78 EX_CONFIG
            Self::ProjectNotFound { .. }
            | Self::ProjectDirUnusable { .. }
            | Self::ProjectFolderNotOwned { .. }
            | Self::ConfigurationPathReference { .. }
            | Self::ConfigurationNotOwned { .. }
            | Self::ConfigurationUnsafeMode { .. }
            | Self::ConfigurationMalformed { .. }
            | Self::ConfigurationKeyOutsideSpace { .. }
            | Self::ConfigurationValueMalformed { .. }
            | Self::DsnMalformed { .. }
            | Self::UnclosedExpansion { .. }
            | Self::InvalidReferenceName { .. }
            | Self::ConfigurationEntryName { .. }
            | Self::PasswordCommandNotAnArray { .. }
            | Self::ConflictingEntryKeys { .. }
            | Self::DsnQueryParameter { .. }
            | Self::UndefinedVariable { .. }
            | Self::PasswordCommandDeadlineExceeded { .. }
            | Self::PasswordCommandOutputCapExceeded { .. }
            | Self::PasswordCommandNotExecutable { .. }
            | Self::PasswordCommandFailed { .. }
            | Self::TrustDirectoryEmpty { .. }
            | Self::ReadOnlySessionNotEnforced { .. }
            | Self::EntryKeyMissing { .. }
            | Self::NoDatabaseEntrySelected { .. }
            | Self::ServerNotMariaDb { .. }
            | Self::SeriesNotSupported { .. } => 78,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{
        CatalogueObjectKind, ChildEnd, ContextFault, DeadlineBound, DsnFault, EntryNameGiven,
        EntryRepair, Error, KeyAbsence, NetworkPhase, PasswordCommandFault, Position,
        ReadOnlyFault, ReferenceFault, ensure_invariant, trigger_internal_invariant,
    };
    use std::collections::BTreeSet;
    use std::io;
    use std::panic::Location;
    use std::path::PathBuf;
    use std::time::Duration;

    /// The number of variants of [`Error`]. Adding one without adding a sample
    /// below fails `the_sample_set_covers_every_variant`.
    const VARIANT_COUNT: usize = 87;

    fn path() -> PathBuf {
        PathBuf::from(".tpl/.cfg")
    }

    fn position() -> Position {
        Position { line: 7, column: 3 }
    }

    /// A supported window for a sample.
    ///
    /// It is deliberately not the table of `FR-SRV-015`. `BR-SRV-005` states
    /// that set once and forbids a second copy, and the point of carrying the
    /// window on the variant is that this module never holds one: a sample
    /// needs a window of the right *shape*, not the right contents.
    const WINDOW: &[&str] = &["Z.9", "Z.8"];

    /// One value of every variant, beside the code its row of `FR-ERR-001`
    /// fixes.
    fn samples() -> Vec<(Error, u8)> {
        vec![
            // 64 EX_USAGE
            (
                Error::UnknownCommand {
                    node: String::new(),
                    token: "sch".to_owned(),
                    nearest: vec!["schema".to_owned()],
                },
                64,
            ),
            (
                Error::UnknownCommandPathSegment {
                    segment: "ad".to_owned(),
                    node: "cfg database".to_owned(),
                    nearest: vec!["add".to_owned()],
                },
                64,
            ),
            (
                Error::UnknownFlag {
                    positional: false,
                    command: String::new(),
                    token: "--data".to_owned(),
                    nearest: vec!["--database".to_owned()],
                    belongs_to: None,
                },
                64,
            ),
            (
                Error::UnexpectedArgument {
                    command: "version".to_owned(),
                    token: "json".to_owned(),
                },
                64,
            ),
            (
                Error::RepeatedValueFlag {
                    flag: "--database".to_owned(),
                    first: "a".to_owned(),
                    second: "b".to_owned(),
                },
                64,
            ),
            (
                Error::FlagTookCommand {
                    flag: "-d".to_owned(),
                    value: "schema".to_owned(),
                    token: "tables".to_owned(),
                    rebuilt: Some("-d <entry> schema tables".into()),
                    path: "schema tables".into(),
                },
                64,
            ),
            (
                Error::RepeatedFlag {
                    flag: "--quiet".to_owned(),
                },
                64,
            ),
            (
                Error::FlagValueMissing {
                    flag: "--timeout".to_owned(),
                    permitted: Vec::new(),
                },
                64,
            ),
            (
                Error::SeparateTokenValue {
                    flag: "--database".to_owned(),
                    value: "-x".to_owned(),
                },
                64,
            ),
            (
                Error::ValueOutsideEnumeration {
                    command: String::new(),
                    flag: "--format".to_owned(),
                    value: "xml".to_owned(),
                    permitted: vec!["text".to_owned(), "json".to_owned()],
                },
                64,
            ),
            (
                Error::InvocationRejected {
                    reason: "the value is not one the argument accepts",
                    command: String::new(),
                    token: Some("-x".to_owned()),
                },
                64,
            ),
            (
                Error::MissingArgument {
                    command: "schema table".to_owned(),
                    argument: "<name>".to_owned(),
                },
                64,
            ),
            (
                Error::MutuallyExclusiveFlags {
                    first: "--dsn".to_owned(),
                    second: "--host".to_owned(),
                },
                64,
            ),
            (Error::DirectWithContext, 64),
            (
                Error::RoutinePrefixNotLowerCase {
                    token: "PROCEDURE:calc_vat".to_owned(),
                    prefix: "procedure",
                    name: "calc_vat".to_owned(),
                    invocation: "schema routine",
                    template: None,
                },
                64,
            ),
            (
                Error::AmbiguousRoutineName {
                    name: "calc_vat".to_owned(),
                    entry: "shop".to_owned(),
                    database: "shop".to_owned(),
                    invocation: "schema routine",
                    template: None,
                },
                64,
            ),
            (
                Error::AmbiguousRoutineInContext {
                    name: "calc_vat".to_owned(),
                    path: PathBuf::from("context.json"),
                    database: "freight".to_owned(),
                    invocation: "render --routine",
                    template: None,
                },
                64,
            ),
            (
                Error::RepeatedSetKey {
                    key: "title".to_owned(),
                    first: "Orders".to_owned(),
                    second: "Consignments".to_owned(),
                },
                64,
            ),
            (Error::LoadWithoutStoring { object: None }, 64),
            (
                Error::PrettyWithoutJson {
                    command: "template list".to_owned(),
                    complete: true,
                },
                64,
            ),
            (
                Error::ConnectionDetailsMissing {
                    entry: "shop".to_owned(),
                },
                64,
            ),
            (
                Error::NothingToUpdate {
                    entry: "shop".to_owned(),
                },
                64,
            ),
            (
                Error::BlockKeyGiven {
                    key: "database.shop".to_owned(),
                    entry: Some("shop".to_owned()),
                },
                64,
            ),
            (
                Error::ContextDocumentUnreadable {
                    path: PathBuf::from("context.json"),
                    returned: io::Error::from(io::ErrorKind::NotFound),
                },
                74,
            ),
            (
                Error::TrustMaterialUnreadable {
                    entry: "shop".to_owned(),
                    key: "ca_file",
                    path: PathBuf::from("/etc/ssl/ca.pem"),
                    returned: io::Error::from(io::ErrorKind::NotFound),
                },
                74,
            ),
            (
                Error::ProjectDirUnusable {
                    path: PathBuf::from("/srv/shop/.tpl"),
                    fault: super::TplDirFault::Missing,
                },
                78,
            ),
            (
                Error::MalformedValue {
                    command: String::new(),
                    parameter: "--timeout".to_owned(),
                    value: "soon".to_owned(),
                    expected: "integer",
                },
                64,
            ),
            (
                Error::UnknownConfigurationKey {
                    key: "core.databse".to_owned(),
                    nearest: vec!["core.database".to_owned()],
                },
                64,
            ),
            (
                Error::DatabaseEntryAlreadyExists {
                    name: "shop".to_owned(),
                    file: path(),
                },
                64,
            ),
            (
                Error::IncoherentEntryWrite {
                    entry: "shop".to_owned(),
                    written: "database.shop.dsn".to_owned(),
                    conflicting: "database.shop.host".to_owned(),
                    repair: EntryRepair::Unset,
                },
                64,
            ),
            (
                Error::InitDestinationIsTplFolder {
                    written: PathBuf::from("proj/.tpl"),
                    canonical: None,
                    parent: Some(PathBuf::from("proj")),
                },
                64,
            ),
            (
                Error::InvalidEntryName {
                    given: EntryNameGiven::Add,
                    name: "a b".to_owned(),
                },
                64,
            ),
            (
                Error::InvalidReference {
                    parameter: "database.shop.user".to_owned(),
                    command: "cfg set".to_owned(),
                    fault: ReferenceFault::Name("1X".to_owned()),
                },
                64,
            ),
            (
                Error::EmptyValue {
                    parameter: "--host".to_owned(),
                    command: "cfg database add".to_owned(),
                },
                64,
            ),
            // 65 EX_DATAERR
            (
                Error::TemplateSyntax {
                    template: "example.jinja".to_owned(),
                    position: position(),
                    chain: vec!["unexpected end of input".to_owned()],
                },
                65,
            ),
            (
                Error::RenderFailed {
                    undefined: None,
                    reason: None,
                    invoked: String::from("example"),
                    template: "example.jinja".to_owned(),
                    position: position(),
                    chain: vec!["undefined value".to_owned()],
                },
                65,
            ),
            (
                Error::TemplateOutsideRoot {
                    name: "leak.jinja".to_owned(),
                    root: PathBuf::from(".tpl/templates"),
                },
                65,
            ),
            (
                Error::ContextDocumentMalformed {
                    path: std::path::Path::new("context.json").into(),
                    fault: ContextFault::NotJson(position()),
                    default_entry: false,
                },
                65,
            ),
            (
                Error::RenderDeadlineExceeded {
                    bound: DeadlineBound::Phase,
                    limit: Duration::from_secs(30),
                },
                65,
            ),
            (Error::RenderFuelExhausted { fuel: 100_000_000 }, 65),
            (Error::RenderOutputLimitExceeded { limit: 67_108_864 }, 65),
            (Error::RenderMemoryLimitExceeded { limit: 134_217_728 }, 65),
            // 66 EX_NOINPUT
            (
                Error::NothingCachedNamed {
                    kind: CatalogueObjectKind::Table,
                    name: "ordrs".to_owned(),
                    entry: "shop".to_owned(),
                    nearest: vec!["orders".to_owned()],
                    qualified: None,
                    held_as: None,
                },
                66,
            ),
            (
                Error::CatalogueObjectNotFound {
                    kind: CatalogueObjectKind::Table,
                    name: "ordrs".to_owned(),
                    entry: "shop".to_owned(),
                    database: "shop".to_owned(),
                    nearest: Vec::new(),
                },
                66,
            ),
            (
                Error::ContextObjectNotFound {
                    kind: CatalogueObjectKind::Table,
                    name: "ordrs".to_owned(),
                    path: PathBuf::from("context.json"),
                    database: "freight".to_owned(),
                    nearest: vec!["orders".to_owned()],
                },
                66,
            ),
            (
                Error::TemplateNotFound {
                    name: "missing.jinja".to_owned(),
                    root: PathBuf::from(".tpl/templates"),
                    nearest: Vec::new(),
                },
                66,
            ),
            (
                Error::DatabaseEntryNotFound {
                    name: "shup".to_owned(),
                    file: path(),
                    nearest: vec!["shop".to_owned()],
                    by_default: false,
                },
                66,
            ),
            (
                Error::ConfigurationKeyNotFound {
                    key: "core.database".to_owned(),
                    known: true,
                    default: None,
                    entry_missing: false,
                    file: path(),
                    nearest: Vec::new(),
                },
                66,
            ),
            // 69 EX_UNAVAILABLE
            (
                Error::NameNotResolved {
                    by_dsn: false,
                    entry: String::from("shop"),
                    host: "db.example.com".to_owned(),
                    port: 3306,
                },
                69,
            ),
            (
                Error::ConnectionRefused {
                    by_dsn: false,
                    entry: String::from("shop"),
                    host: "127.0.0.1".to_owned(),
                    port: 3306,
                },
                69,
            ),
            (
                Error::TlsHandshakeFailed {
                    fault: crate::error::TlsFault::Refused,
                    entry: String::from("shop"),
                    host: "db.example.com".to_owned(),
                    port: 3306,
                },
                69,
            ),
            (
                Error::NetworkDeadlineExceeded {
                    by_dsn: false,
                    entry: String::from("shop"),
                    phase: NetworkPhase::CatalogueQuery,
                    host: "db.example.com".to_owned(),
                    port: 3306,
                    bound: DeadlineBound::Overall,
                    limit: Duration::from_secs(10),
                },
                69,
            ),
            // 70 EX_SOFTWARE
            (
                Error::InternalInvariant {
                    invariant: "the key column has no column",
                    location: Location::caller(),
                },
                70,
            ),
            // 73 EX_CANTCREAT
            (
                Error::ProjectAlreadyExists {
                    path: PathBuf::from("/work/.tpl"),
                },
                73,
            ),
            (
                Error::ProjectNotCreated {
                    path: PathBuf::from("/work/.tpl"),
                    returned: io::Error::from(io::ErrorKind::PermissionDenied),
                },
                73,
            ),
            // 74 EX_IOERR
            (
                Error::ProjectFileUnreadable {
                    path: path(),
                    returned: io::Error::from(io::ErrorKind::PermissionDenied),
                },
                74,
            ),
            (
                Error::ProjectFileUnwritable {
                    path: path(),
                    returned: io::Error::from(io::ErrorKind::StorageFull),
                },
                74,
            ),
            (
                Error::StdoutUnwritable {
                    returned: io::Error::from(io::ErrorKind::StorageFull),
                },
                74,
            ),
            (Error::StdoutClosedMidDocument, 74),
            // 77 EX_NOPERM
            (
                Error::AuthenticationRefused {
                    entry: String::from("shop"),
                    user: "reader".to_owned(),
                    host: "db.example.com".to_owned(),
                },
                77,
            ),
            (
                Error::PropertyNotReadable {
                    kind: CatalogueObjectKind::Routine,
                    object: "shop.recalculate_totals".to_owned(),
                    property: "body",
                },
                77,
            ),
            // 78 EX_CONFIG
            (
                Error::ProjectNotFound {
                    walk_ended_at: PathBuf::from("/"),
                },
                78,
            ),
            (
                Error::ConfigurationNotOwned {
                    path: path(),
                    owner: 0,
                    expected: 501,
                },
                78,
            ),
            (
                Error::ProjectFolderNotOwned {
                    path: PathBuf::from("/tmp/.tpl"),
                    owner: 0,
                    expected: 501,
                },
                78,
            ),
            (
                Error::ConfigurationPathReference {
                    key: "database.shop.ca_file".to_owned(),
                    file: path(),
                    position: position(),
                    value: "${SHOP_CA}".to_owned(),
                },
                78,
            ),
            (
                Error::ConfigurationUnsafeMode {
                    path: path(),
                    mode: 0o644,
                },
                78,
            ),
            (
                Error::ConfigurationMalformed {
                    reason: "expected `]`".to_owned(),
                    path: path(),
                    position: position(),
                },
                78,
            ),
            (
                Error::ConfigurationKeyOutsideSpace {
                    key: "core.databse".to_owned(),
                    file: path(),
                    position: position(),
                    nearest: vec!["core.database".to_owned()],
                },
                78,
            ),
            (
                Error::ConfigurationValueMalformed {
                    key: "core.connect_timeout".to_owned(),
                    file: path(),
                    position: position(),
                    found: "0".to_owned(),
                    expected: "a positive integer number of seconds",
                    expanded_from: None,
                },
                78,
            ),
            (
                Error::DsnMalformed {
                    key: "database.shop.dsn".to_owned(),
                    file: path(),
                    fault: DsnFault::Scheme,
                },
                78,
            ),
            (
                Error::UnclosedExpansion {
                    key: "database.shop.password".to_owned(),
                    file: path(),
                },
                78,
            ),
            (
                Error::InvalidReferenceName {
                    key: "database.shop.user".to_owned(),
                    file: path(),
                    name: "1X".to_owned(),
                },
                78,
            ),
            (
                Error::ConfigurationEntryName {
                    file: path(),
                    name: "x.y".to_owned(),
                    core: false,
                    position: position(),
                },
                78,
            ),
            (
                Error::PasswordCommandNotAnArray {
                    key: "database.shop.password_command".to_owned(),
                    file: path(),
                    position: position(),
                    found: "string",
                    element: None,
                },
                78,
            ),
            (
                Error::ConflictingEntryKeys {
                    entry: "shop".to_owned(),
                    file: path(),
                    first: "dsn".to_owned(),
                    second: "host".to_owned(),
                },
                78,
            ),
            (
                Error::DsnQueryParameter {
                    key: "database.shop.dsn".to_owned(),
                    file: path(),
                },
                78,
            ),
            (
                Error::UndefinedVariable {
                    name: "SHOP_PW".to_owned(),
                    key: "database.shop.password".to_owned(),
                    file: path(),
                },
                78,
            ),
            (
                Error::PasswordCommandDeadlineExceeded {
                    entry: "shop".to_owned(),
                    command: vec!["security".to_owned(), "find-generic-password".to_owned()],
                    bound: DeadlineBound::Phase,
                    limit: Duration::from_secs(5),
                },
                78,
            ),
            (
                Error::PasswordCommandOutputCapExceeded {
                    entry: "shop".to_owned(),
                    command: vec!["cat".to_owned(), "/dev/urandom".to_owned()],
                    cap: 4096,
                },
                78,
            ),
            (
                Error::PasswordCommandNotExecutable {
                    entry: "shop".to_owned(),
                    command: vec!["pass".to_owned()],
                    fault: PasswordCommandFault::NotStarted,
                    returned: io::Error::from(io::ErrorKind::NotFound),
                },
                78,
            ),
            (
                Error::PasswordCommandFailed {
                    entry: "shop".to_owned(),
                    command: vec!["op".to_owned(), "read".to_owned()],
                    end: ChildEnd::Exited(1),
                },
                78,
            ),
            (
                Error::TrustDirectoryEmpty {
                    entry: "shop".to_owned(),
                    path: path(),
                },
                78,
            ),
            (
                Error::ReadOnlySessionNotEnforced {
                    by_dsn: false,
                    entry: "shop".to_owned(),
                    fault: ReadOnlyFault::ReadBackDisagreed,
                },
                78,
            ),
            (
                Error::EntryKeyMissing {
                    entry: "shop".to_owned(),
                    key: "database.shop.host".to_owned(),
                    file: path(),
                    flag: "--host",
                    placeholder: "<host>",
                    absence: KeyAbsence::Absent,
                },
                78,
            ),
            (
                Error::NoDatabaseEntrySelected {
                    file: path(),
                    has_entries: true,
                },
                78,
            ),
            (
                Error::ServerNotMariaDb {
                    by_dsn: false,
                    entry: "shop".to_owned(),
                    product: "MySQL".to_owned(),
                },
                78,
            ),
            (
                Error::SeriesNotSupported {
                    by_dsn: false,
                    entry: "shop".to_owned(),
                    series: "10.6".to_owned(),
                    supported: WINDOW,
                },
                78,
            ),
        ]
    }

    /// The variant's name, by an exhaustive match with no wildcard arm: a
    /// variant added to [`Error`] does not compile until it is named here too,
    /// which is what keeps `samples` honest.
    fn variant_name(error: &Error) -> &'static str {
        match error {
            Error::UnknownCommand { .. } => "UnknownCommand",
            Error::UnknownCommandPathSegment { .. } => "UnknownCommandPathSegment",
            Error::UnknownFlag { .. } => "UnknownFlag",
            Error::UnexpectedArgument { .. } => "UnexpectedArgument",
            Error::RepeatedValueFlag { .. } => "RepeatedValueFlag",
            Error::RepeatedFlag { .. } => "RepeatedFlag",
            Error::FlagValueMissing { .. } => "FlagValueMissing",
            Error::FlagTookCommand { .. } => "FlagTookCommand",
            Error::SeparateTokenValue { .. } => "SeparateTokenValue",
            Error::ValueOutsideEnumeration { .. } => "ValueOutsideEnumeration",
            Error::InvocationRejected { .. } => "InvocationRejected",
            Error::MissingArgument { .. } => "MissingArgument",
            Error::MutuallyExclusiveFlags { .. } => "MutuallyExclusiveFlags",
            Error::DirectWithContext => "DirectWithContext",
            Error::RoutinePrefixNotLowerCase { .. } => "RoutinePrefixNotLowerCase",
            Error::AmbiguousRoutineName { .. } => "AmbiguousRoutineName",
            Error::AmbiguousRoutineInContext { .. } => "AmbiguousRoutineInContext",
            Error::RepeatedSetKey { .. } => "RepeatedSetKey",
            Error::LoadWithoutStoring { .. } => "LoadWithoutStoring",
            Error::MalformedValue { .. } => "MalformedValue",
            Error::UnknownConfigurationKey { .. } => "UnknownConfigurationKey",
            Error::DatabaseEntryAlreadyExists { .. } => "DatabaseEntryAlreadyExists",
            Error::IncoherentEntryWrite { .. } => "IncoherentEntryWrite",
            Error::InitDestinationIsTplFolder { .. } => "InitDestinationIsTplFolder",
            Error::InvalidEntryName { .. } => "InvalidEntryName",
            Error::InvalidReference { .. } => "InvalidReference",
            Error::EmptyValue { .. } => "EmptyValue",
            Error::TemplateSyntax { .. } => "TemplateSyntax",
            Error::RenderFailed { .. } => "RenderFailed",
            Error::TemplateOutsideRoot { .. } => "TemplateOutsideRoot",
            Error::ContextDocumentMalformed { .. } => "ContextDocumentMalformed",
            Error::RenderDeadlineExceeded { .. } => "RenderDeadlineExceeded",
            Error::RenderFuelExhausted { .. } => "RenderFuelExhausted",
            Error::RenderOutputLimitExceeded { .. } => "RenderOutputLimitExceeded",
            Error::RenderMemoryLimitExceeded { .. } => "RenderMemoryLimitExceeded",
            Error::CatalogueObjectNotFound { .. } => "CatalogueObjectNotFound",
            Error::NothingCachedNamed { .. } => "NothingCachedNamed",
            Error::ContextObjectNotFound { .. } => "ContextObjectNotFound",
            Error::TemplateNotFound { .. } => "TemplateNotFound",
            Error::DatabaseEntryNotFound { .. } => "DatabaseEntryNotFound",
            Error::ConfigurationKeyNotFound { .. } => "ConfigurationKeyNotFound",
            Error::NameNotResolved { .. } => "NameNotResolved",
            Error::ConnectionRefused { .. } => "ConnectionRefused",
            Error::TlsHandshakeFailed { .. } => "TlsHandshakeFailed",
            Error::NetworkDeadlineExceeded { .. } => "NetworkDeadlineExceeded",
            Error::InternalInvariant { .. } => "InternalInvariant",
            Error::ProjectAlreadyExists { .. } => "ProjectAlreadyExists",
            Error::ProjectNotCreated { .. } => "ProjectNotCreated",
            Error::ProjectFileUnreadable { .. } => "ProjectFileUnreadable",
            Error::ProjectFileUnwritable { .. } => "ProjectFileUnwritable",
            Error::StdoutUnwritable { .. } => "StdoutUnwritable",
            Error::StdoutClosedMidDocument => "StdoutClosedMidDocument",
            Error::AuthenticationRefused { .. } => "AuthenticationRefused",
            Error::PropertyNotReadable { .. } => "PropertyNotReadable",
            Error::ProjectNotFound { .. } => "ProjectNotFound",
            Error::ConfigurationNotOwned { .. } => "ConfigurationNotOwned",
            Error::ProjectFolderNotOwned { .. } => "ProjectFolderNotOwned",
            Error::ConfigurationPathReference { .. } => "ConfigurationPathReference",
            Error::ConfigurationUnsafeMode { .. } => "ConfigurationUnsafeMode",
            Error::ConfigurationMalformed { .. } => "ConfigurationMalformed",
            Error::ConfigurationKeyOutsideSpace { .. } => "ConfigurationKeyOutsideSpace",
            Error::ConfigurationValueMalformed { .. } => "ConfigurationValueMalformed",
            Error::DsnMalformed { .. } => "DsnMalformed",
            Error::UnclosedExpansion { .. } => "UnclosedExpansion",
            Error::InvalidReferenceName { .. } => "InvalidReferenceName",
            Error::ConfigurationEntryName { .. } => "ConfigurationEntryName",
            Error::PasswordCommandNotAnArray { .. } => "PasswordCommandNotAnArray",
            Error::ConflictingEntryKeys { .. } => "ConflictingEntryKeys",
            Error::DsnQueryParameter { .. } => "DsnQueryParameter",
            Error::UndefinedVariable { .. } => "UndefinedVariable",
            Error::PasswordCommandDeadlineExceeded { .. } => "PasswordCommandDeadlineExceeded",
            Error::PasswordCommandOutputCapExceeded { .. } => "PasswordCommandOutputCapExceeded",
            Error::PasswordCommandNotExecutable { .. } => "PasswordCommandNotExecutable",
            Error::PasswordCommandFailed { .. } => "PasswordCommandFailed",
            Error::TrustDirectoryEmpty { .. } => "TrustDirectoryEmpty",
            Error::ReadOnlySessionNotEnforced { .. } => "ReadOnlySessionNotEnforced",
            Error::EntryKeyMissing { .. } => "EntryKeyMissing",
            Error::NoDatabaseEntrySelected { .. } => "NoDatabaseEntrySelected",
            Error::PrettyWithoutJson { .. } => "PrettyWithoutJson",
            Error::ConnectionDetailsMissing { .. } => "ConnectionDetailsMissing",
            Error::NothingToUpdate { .. } => "NothingToUpdate",
            Error::BlockKeyGiven { .. } => "BlockKeyGiven",
            Error::ContextDocumentUnreadable { .. } => "ContextDocumentUnreadable",
            Error::TrustMaterialUnreadable { .. } => "TrustMaterialUnreadable",
            Error::ProjectDirUnusable { .. } => "ProjectDirUnusable",
            Error::ServerNotMariaDb { .. } => "ServerNotMariaDb",
            Error::SeriesNotSupported { .. } => "SeriesNotSupported",
        }
    }

    #[test]
    fn fr_err_001_every_variant_carries_the_code_of_its_row() {
        for (error, expected) in samples() {
            assert_eq!(
                error.exit_code(),
                expected,
                "{} does not carry the code its row of FR-ERR-001 fixes",
                variant_name(&error)
            );
        }
    }

    #[test]
    fn the_sample_set_covers_every_variant() {
        let samples = samples();
        let names: BTreeSet<&'static str> = samples.iter().map(|(e, _)| variant_name(e)).collect();

        assert_eq!(names.len(), samples.len(), "a variant is sampled twice");
        assert_eq!(
            names.len(),
            VARIANT_COUNT,
            "every variant of Error must appear in samples()"
        );
    }

    #[test]
    fn fr_err_001_the_codes_are_the_nine_of_the_table_and_no_others() {
        let produced: BTreeSet<u8> = samples().iter().map(|(e, _)| e.exit_code()).collect();
        let table = BTreeSet::from([64, 65, 66, 69, 70, 73, 74, 77, 78]);

        assert_eq!(produced, table);
    }

    #[test]
    fn fr_err_003_code_73_is_confined_to_the_init_path() {
        // FR-ERR-003: 73 is produced only by `tpl init`.
        for (error, _) in samples() {
            if error.exit_code() == 73 {
                let name = variant_name(&error);
                assert!(
                    name == "ProjectAlreadyExists" || name == "ProjectNotCreated",
                    "{name} produces 73 and is not on the `tpl init` path"
                );
            }
        }
    }

    #[test]
    fn fr_err_030_code_70_is_confined_to_the_internal_error_variant() {
        // FR-ERR-030: 70 is a defect in `tpl`, never something a caller
        // provokes. The other producing condition, a panic, never becomes an
        // Error value.
        for (error, _) in samples() {
            if error.exit_code() == 70 {
                assert_eq!(variant_name(&error), "InternalInvariant");
            }
        }
    }

    #[test]
    fn fr_err_030_the_guard_passes_an_invariant_that_holds() {
        assert!(ensure_invariant(true, "a sample invariant").is_ok());
    }

    #[test]
    fn fr_err_034_the_guard_reports_a_violated_invariant_at_its_call_site() {
        // FR-ERR-034, the 70 row: the invariant, and where it was detected.
        // `#[track_caller]` is what makes "where" this line and not the guard.
        let here = Location::caller();
        let error = ensure_invariant(false, "a sample invariant")
            .expect_err("the guard reports an invariant that does not hold");

        let Error::InternalInvariant {
            invariant,
            location,
        } = error
        else {
            panic!("the guard produced {error:?}, not the condition of FR-ERR-030");
        };

        assert_eq!(invariant, "a sample invariant");
        assert_eq!(location.file(), here.file());
        assert_eq!(
            location.line(),
            here.line() + 1,
            "the location is the call site, not a frame inside the guard"
        );
    }

    #[test]
    fn fr_err_031_the_trigger_always_fires() {
        // FR-ERR-031: the trigger exists so that the condition can be exercised
        // in process, and it is reachable from nothing a caller can write.
        let error =
            trigger_internal_invariant().expect_err("the trigger exists to produce the condition");

        assert_eq!(error.exit_code(), 70);
        assert_eq!(variant_name(&error), "InternalInvariant");
    }

    #[test]
    fn fr_err_008_display_carries_the_error_line_of_the_corpus_examples() {
        // FR-ERR-008's own example.
        assert_eq!(
            Error::CatalogueObjectNotFound {
                kind: CatalogueObjectKind::Table,
                name: "ordrs".to_owned(),
                entry: "shop".to_owned(),
                database: "shop".to_owned(),
                nearest: Vec::new(),
            }
            .to_string(),
            "table 'ordrs' does not exist in database 'shop'"
        );

        // FR-PROJ-011's own example.
        assert_eq!(
            Error::ConfigurationUnsafeMode {
                path: path(),
                mode: 0o644,
            }
            .to_string(),
            ".tpl/.cfg has unsafe permissions"
        );

        // FR-CONF-035's own example.
        assert_eq!(
            Error::PasswordCommandNotAnArray {
                key: "database.shop.password_command".to_owned(),
                file: path(),
                position: position(),
                found: "string",
                element: None,
            }
            .to_string(),
            "database.shop.password_command is not an array"
        );

        // FR-SRV-030's own example.
        assert_eq!(
            Error::SeriesNotSupported {
                by_dsn: false,
                entry: "shop".to_owned(),
                series: "10.6".to_owned(),
                supported: WINDOW,
            }
            .to_string(),
            "server series '10.6' is not supported, for database entry 'shop'"
        );
    }

    #[test]
    fn display_is_one_line_and_carries_no_labelled_line() {
        // OD-06: Display is the content of the `error:` line alone. The four
        // labelled lines belong to the renderer of diagnostics/.
        for (error, _) in samples() {
            let line = error.to_string();
            assert!(!line.contains('\n'), "{line:?} spans more than one line");
            for label in ["error:", "cause:", "hint:", "exit:"] {
                assert!(!line.contains(label), "{line:?} carries the {label} label");
            }
        }
    }
}
