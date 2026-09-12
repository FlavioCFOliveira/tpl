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
use std::path::PathBuf;
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
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CatalogueObjectKind {
    /// A base table.
    Table,
    /// A view.
    View,
    /// A stored procedure or function.
    Routine,
}

impl fmt::Display for CatalogueObjectKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let name = match self {
            Self::Table => "table",
            Self::View => "view",
            Self::Routine => "routine",
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
            Self::CatalogueQuery => "the catalogue query",
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

/// How a `--context` document failed the contract of `FR-RND-020`.
///
/// `FR-ERR-034` obliges the `cause` line of such a `65` to name the path and
/// *either* the position of the malformed JSON *or* the structural rule the
/// document failed; this is that either.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ContextFault {
    /// The bytes are not well-formed JSON, at this position.
    NotJson(Position),
    /// The document is JSON and does not match the contract of
    /// `context-document.md`; the field names the rule it failed.
    Structure {
        /// The structural rule that was not satisfied.
        rule: &'static str,
    },
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
    #[error("unknown command '{token}'")]
    UnknownCommand {
        /// The token as written, never normalised (`FR-CLI-020`).
        token: String,
    },

    /// A flag the invoked node does not declare (`FR-CLI-005`, `FR-CLI-019`).
    #[error("unknown flag '{token}'")]
    UnknownFlag {
        /// The token as written.
        token: String,
    },

    /// A required argument was not supplied (`FR-ERR-001`, the `64` row).
    #[error("the command '{command}' requires the argument '{argument}'")]
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

    /// A value that does not conform to the type its parameter declares: a
    /// flag value (`FR-ERR-001`, the `64` row) or a `tpl cfg set` value
    /// (`FR-CFG-010`).
    #[error("'{value}' is not a valid value for '{parameter}'")]
    MalformedValue {
        /// The flag or the configuration key the value was given for.
        parameter: String,
        /// The value as written.
        value: String,
        /// The type that was expected, per `FR-CONF-002` where the parameter
        /// is a configuration key.
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
    #[error("rendering template '{template}' failed at {position}")]
    RenderFailed {
        /// The template being rendered when the failure arose.
        template: String,
        /// Where evaluation stopped.
        position: Position,
        /// The chain of underlying template-engine errors, outermost first
        /// (`FR-ERR-011`).
        chain: Vec<String>,
    },

    /// A resolved template path that lies outside the template root
    /// (`FR-TMPL-026`), reached without rendering — `tpl template show` on a
    /// symbolic link is the case `FR-TMPL-024` describes.
    #[error("template '{name}' resolves outside the template root")]
    TemplateOutsideRoot {
        /// The template name, as the caller named it.
        name: String,
        /// The root the resolved path had to stay within (`FR-TMPL-023`).
        root: PathBuf,
    },

    /// A `--context` document that is not well-formed JSON or does not match
    /// the document contract (`FR-RND-020`, `FR-ERR-029`).
    #[error("the --context document '{}' is malformed", .path.display())]
    ContextDocumentMalformed {
        /// The path the document was read from.
        path: PathBuf,
        /// Which half of `FR-RND-020` it failed.
        fault: ContextFault,
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
    },

    /// A named template that does not exist under the template root
    /// (`FR-TMPL-027`, `FR-RND-029`).
    #[error("template '{name}' does not exist")]
    TemplateNotFound {
        /// The identifier that was not found.
        name: String,
        /// The population it was sought in (`FR-TMPL-023`).
        root: PathBuf,
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
    },

    /// A key that is absent from `.tpl/.cfg` (`FR-CFG-007`, `FR-CFG-012`).
    #[error("configuration key '{key}' is not set")]
    ConfigurationKeyNotFound {
        /// The key that was not found.
        key: String,
        /// The file it was sought in.
        file: PathBuf,
    },

    // ---------------------------------------------------------------- 69 ---
    /// The host name could not be resolved.
    #[error("host '{host}' could not be resolved")]
    NameNotResolved {
        /// The host attempted.
        host: String,
        /// The port attempted.
        port: u16,
    },

    /// The server refused the TCP connection.
    #[error("the server at {host}:{port} refused the connection")]
    ConnectionRefused {
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
    #[error("the TLS handshake with {host}:{port} failed")]
    TlsHandshakeFailed {
        /// The host attempted.
        host: String,
        /// The port attempted.
        port: u16,
    },

    /// A network phase did not finish within its deadline (`FR-ERR-027`,
    /// `FR-GLOB-013`).
    #[error("{phase} for {host}:{port} exceeded {bound} of {limit:?}")]
    NetworkDeadlineExceeded {
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
    /// A file of the project could not be read (`FR-ERR-001`, the `74` row).
    #[error("{} could not be read", .path.display())]
    ProjectFileUnreadable {
        /// The path that failed.
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
    #[error("the server at '{host}' refused authentication for user '{user}'")]
    AuthenticationRefused {
        /// The user the server refused.
        user: String,
        /// The host that refused it.
        host: String,
    },

    /// A property of an object requested by name could not be read, so the
    /// object is incomplete and is not returned in part (`FR-PRIV-003`,
    /// `FR-PRIV-004`, `FR-PRIV-013`).
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
    },

    /// `.tpl/.cfg` carries a key outside the enumerated space of
    /// `FR-CONF-002` (`FR-CONF-034`).
    #[error("{} declares the unknown key '{key}'", .file.display())]
    ConfigurationKeyOutsideSpace {
        /// The key found.
        key: String,
        /// The file that declares it.
        file: PathBuf,
    },

    /// `password_command` is stored as something other than an array of
    /// strings (`FR-CONF-035`).
    #[error("{key} is not an array")]
    PasswordCommandNotAnArray {
        /// The fully qualified key.
        key: String,
        /// The file that declares it.
        file: PathBuf,
        /// The TOML type found, against the array of `FR-CONF-023` expected.
        found: &'static str,
    },

    /// One entry declares two keys that exclude one another (`FR-CONF-006`,
    /// `FR-CONF-007`).
    #[error("database entry '{entry}' declares both {first} and {second}")]
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

    /// `password_command` did not finish within its deadline
    /// (`FR-CONF-028`, `FR-ERR-027`).
    #[error("password_command exceeded {bound} of {limit:?}")]
    PasswordCommandDeadlineExceeded {
        /// The command as stored, which `FR-CONF-017` guarantees carries no
        /// expanded value and therefore no secret.
        command: Vec<String>,
        /// Which of the two bounds of `FR-GLOB-012` expired.
        bound: DeadlineBound,
        /// The resolved value of that bound.
        limit: Duration,
    },

    /// `password_command` wrote more than the cap of `FR-CONF-031` to standard
    /// output, and the child was terminated.
    #[error("password_command wrote more than {cap} bytes")]
    PasswordCommandOutputCapExceeded {
        /// The command as stored (`FR-CONF-017`).
        command: Vec<String>,
        /// The cap, in bytes — `FR-CONF-031` obliges the `cause` line to name
        /// it beside the command, and never the bytes read.
        cap: usize,
    },

    /// `password_command` exited non-zero (`FR-CONF-033`).
    ///
    /// Its standard error is not carried because it was never captured:
    /// `FR-CONF-032` sends it to the null device.
    #[error("password_command exited with a non-zero status")]
    PasswordCommandFailed {
        /// The command as stored (`FR-CONF-017`).
        command: Vec<String>,
        /// The exit status the child returned, or `None` where a signal ended
        /// it.
        status: Option<i32>,
    },

    /// The read-only session could not be established or confirmed
    /// (`FR-SRV-010`). The catalogue is not read.
    #[error("the read-only session could not be enforced for database entry '{entry}'")]
    ReadOnlySessionNotEnforced {
        /// The entry whose connection it was.
        entry: String,
        /// Which of the two conditions of `FR-SRV-010` arose.
        fault: ReadOnlyFault,
    },

    /// The command requires a database entry and none is selected — neither
    /// `-d/--database` nor `core.database` (`FR-ERR-004`, `FR-GLOB-006`).
    #[error("no database entry is selected")]
    NoDatabaseEntrySelected {
        /// The file the selection would have come from; `FR-GLOB-006` obliges
        /// the message to name it.
        file: PathBuf,
    },

    /// The server is reachable and authenticated and is not MariaDB
    /// (`FR-SRV-003`). The catalogue is not read.
    #[error("the server reached by database entry '{entry}' is not MariaDB")]
    ServerNotMariaDb {
        /// The entry that reached it.
        entry: String,
        /// The product the server reported.
        product: String,
    },

    /// The server is MariaDB of a series below the window of `FR-SRV-015`
    /// (`FR-SRV-020`, `FR-SRV-030`). The catalogue is not read.
    #[error("server series '{series}' is not supported, for database entry '{entry}'")]
    SeriesNotSupported {
        /// The entry that reached it.
        entry: String,
        /// The series found, as the server reported it.
        series: String,
    },
}

impl Error {
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
    /// let error = Error::UnknownCommand { token: "sch".to_owned() };
    /// assert_eq!(error.exit_code(), 64);
    /// assert_eq!(error.to_string(), "unknown command 'sch'");
    /// ```
    #[must_use]
    pub const fn exit_code(&self) -> u8 {
        match self {
            // 64 EX_USAGE
            Self::UnknownCommand { .. }
            | Self::UnknownFlag { .. }
            | Self::MissingArgument { .. }
            | Self::MutuallyExclusiveFlags { .. }
            | Self::MalformedValue { .. }
            | Self::UnknownConfigurationKey { .. } => 64,

            // 65 EX_DATAERR
            Self::TemplateSyntax { .. }
            | Self::RenderFailed { .. }
            | Self::TemplateOutsideRoot { .. }
            | Self::ContextDocumentMalformed { .. }
            | Self::RenderDeadlineExceeded { .. } => 65,

            // 66 EX_NOINPUT
            Self::CatalogueObjectNotFound { .. }
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
            | Self::StdoutUnwritable { .. }
            | Self::StdoutClosedMidDocument => 74,

            // 77 EX_NOPERM
            Self::AuthenticationRefused { .. } | Self::PropertyNotReadable { .. } => 77,

            // 78 EX_CONFIG
            Self::ProjectNotFound { .. }
            | Self::ConfigurationNotOwned { .. }
            | Self::ConfigurationUnsafeMode { .. }
            | Self::ConfigurationMalformed { .. }
            | Self::ConfigurationKeyOutsideSpace { .. }
            | Self::PasswordCommandNotAnArray { .. }
            | Self::ConflictingEntryKeys { .. }
            | Self::DsnQueryParameter { .. }
            | Self::UndefinedVariable { .. }
            | Self::PasswordCommandDeadlineExceeded { .. }
            | Self::PasswordCommandOutputCapExceeded { .. }
            | Self::PasswordCommandFailed { .. }
            | Self::ReadOnlySessionNotEnforced { .. }
            | Self::NoDatabaseEntrySelected { .. }
            | Self::ServerNotMariaDb { .. }
            | Self::SeriesNotSupported { .. } => 78,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{
        CatalogueObjectKind, ContextFault, DeadlineBound, Error, NetworkPhase, Position,
        ReadOnlyFault,
    };
    use std::collections::BTreeSet;
    use std::io;
    use std::panic::Location;
    use std::path::PathBuf;
    use std::time::Duration;

    /// The number of variants of [`Error`]. Adding one without adding a sample
    /// below fails `the_sample_set_covers_every_variant`.
    const VARIANT_COUNT: usize = 43;

    fn path() -> PathBuf {
        PathBuf::from(".tpl/.cfg")
    }

    fn position() -> Position {
        Position { line: 7, column: 3 }
    }

    /// One value of every variant, beside the code its row of `FR-ERR-001`
    /// fixes.
    fn samples() -> Vec<(Error, u8)> {
        vec![
            // 64 EX_USAGE
            (
                Error::UnknownCommand {
                    token: "sch".to_owned(),
                },
                64,
            ),
            (
                Error::UnknownFlag {
                    token: "--data".to_owned(),
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
            (
                Error::MalformedValue {
                    parameter: "--timeout".to_owned(),
                    value: "soon".to_owned(),
                    expected: "integer",
                },
                64,
            ),
            (
                Error::UnknownConfigurationKey {
                    key: "core.databse".to_owned(),
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
                    path: PathBuf::from("context.json"),
                    fault: ContextFault::NotJson(position()),
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
            // 66 EX_NOINPUT
            (
                Error::CatalogueObjectNotFound {
                    kind: CatalogueObjectKind::Table,
                    name: "ordrs".to_owned(),
                    entry: "shop".to_owned(),
                    database: "shop".to_owned(),
                },
                66,
            ),
            (
                Error::TemplateNotFound {
                    name: "missing.jinja".to_owned(),
                    root: PathBuf::from(".tpl/templates"),
                },
                66,
            ),
            (
                Error::DatabaseEntryNotFound {
                    name: "shup".to_owned(),
                    file: path(),
                },
                66,
            ),
            (
                Error::ConfigurationKeyNotFound {
                    key: "core.database".to_owned(),
                    file: path(),
                },
                66,
            ),
            // 69 EX_UNAVAILABLE
            (
                Error::NameNotResolved {
                    host: "db.example.com".to_owned(),
                    port: 3306,
                },
                69,
            ),
            (
                Error::ConnectionRefused {
                    host: "127.0.0.1".to_owned(),
                    port: 3306,
                },
                69,
            ),
            (
                Error::TlsHandshakeFailed {
                    host: "db.example.com".to_owned(),
                    port: 3306,
                },
                69,
            ),
            (
                Error::NetworkDeadlineExceeded {
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
                Error::StdoutUnwritable {
                    returned: io::Error::from(io::ErrorKind::StorageFull),
                },
                74,
            ),
            (Error::StdoutClosedMidDocument, 74),
            // 77 EX_NOPERM
            (
                Error::AuthenticationRefused {
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
                Error::ConfigurationUnsafeMode {
                    path: path(),
                    mode: 0o644,
                },
                78,
            ),
            (
                Error::ConfigurationMalformed {
                    path: path(),
                    position: position(),
                },
                78,
            ),
            (
                Error::ConfigurationKeyOutsideSpace {
                    key: "core.databse".to_owned(),
                    file: path(),
                },
                78,
            ),
            (
                Error::PasswordCommandNotAnArray {
                    key: "database.shop.password_command".to_owned(),
                    file: path(),
                    found: "string",
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
                    command: vec!["security".to_owned(), "find-generic-password".to_owned()],
                    bound: DeadlineBound::Phase,
                    limit: Duration::from_secs(5),
                },
                78,
            ),
            (
                Error::PasswordCommandOutputCapExceeded {
                    command: vec!["cat".to_owned(), "/dev/urandom".to_owned()],
                    cap: 4096,
                },
                78,
            ),
            (
                Error::PasswordCommandFailed {
                    command: vec!["op".to_owned(), "read".to_owned()],
                    status: Some(1),
                },
                78,
            ),
            (
                Error::ReadOnlySessionNotEnforced {
                    entry: "shop".to_owned(),
                    fault: ReadOnlyFault::ReadBackDisagreed,
                },
                78,
            ),
            (Error::NoDatabaseEntrySelected { file: path() }, 78),
            (
                Error::ServerNotMariaDb {
                    entry: "shop".to_owned(),
                    product: "MySQL".to_owned(),
                },
                78,
            ),
            (
                Error::SeriesNotSupported {
                    entry: "shop".to_owned(),
                    series: "10.6".to_owned(),
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
            Error::UnknownFlag { .. } => "UnknownFlag",
            Error::MissingArgument { .. } => "MissingArgument",
            Error::MutuallyExclusiveFlags { .. } => "MutuallyExclusiveFlags",
            Error::MalformedValue { .. } => "MalformedValue",
            Error::UnknownConfigurationKey { .. } => "UnknownConfigurationKey",
            Error::TemplateSyntax { .. } => "TemplateSyntax",
            Error::RenderFailed { .. } => "RenderFailed",
            Error::TemplateOutsideRoot { .. } => "TemplateOutsideRoot",
            Error::ContextDocumentMalformed { .. } => "ContextDocumentMalformed",
            Error::RenderDeadlineExceeded { .. } => "RenderDeadlineExceeded",
            Error::CatalogueObjectNotFound { .. } => "CatalogueObjectNotFound",
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
            Error::StdoutUnwritable { .. } => "StdoutUnwritable",
            Error::StdoutClosedMidDocument => "StdoutClosedMidDocument",
            Error::AuthenticationRefused { .. } => "AuthenticationRefused",
            Error::PropertyNotReadable { .. } => "PropertyNotReadable",
            Error::ProjectNotFound { .. } => "ProjectNotFound",
            Error::ConfigurationNotOwned { .. } => "ConfigurationNotOwned",
            Error::ConfigurationUnsafeMode { .. } => "ConfigurationUnsafeMode",
            Error::ConfigurationMalformed { .. } => "ConfigurationMalformed",
            Error::ConfigurationKeyOutsideSpace { .. } => "ConfigurationKeyOutsideSpace",
            Error::PasswordCommandNotAnArray { .. } => "PasswordCommandNotAnArray",
            Error::ConflictingEntryKeys { .. } => "ConflictingEntryKeys",
            Error::DsnQueryParameter { .. } => "DsnQueryParameter",
            Error::UndefinedVariable { .. } => "UndefinedVariable",
            Error::PasswordCommandDeadlineExceeded { .. } => "PasswordCommandDeadlineExceeded",
            Error::PasswordCommandOutputCapExceeded { .. } => "PasswordCommandOutputCapExceeded",
            Error::PasswordCommandFailed { .. } => "PasswordCommandFailed",
            Error::ReadOnlySessionNotEnforced { .. } => "ReadOnlySessionNotEnforced",
            Error::NoDatabaseEntrySelected { .. } => "NoDatabaseEntrySelected",
            Error::ServerNotMariaDb { .. } => "ServerNotMariaDb",
            Error::SeriesNotSupported { .. } => "SeriesNotSupported",
        }
    }

    #[test]
    fn every_variant_carries_the_code_of_its_row() {
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
    fn the_codes_are_the_nine_of_the_table_and_no_others() {
        let produced: BTreeSet<u8> = samples().iter().map(|(e, _)| e.exit_code()).collect();
        let table = BTreeSet::from([64, 65, 66, 69, 70, 73, 74, 77, 78]);

        assert_eq!(produced, table);
    }

    #[test]
    fn code_73_is_confined_to_the_init_path() {
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
    fn code_70_is_confined_to_the_internal_error_variant() {
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
    fn display_carries_the_error_line_of_the_corpus_examples() {
        // FR-ERR-008's own example.
        assert_eq!(
            Error::CatalogueObjectNotFound {
                kind: CatalogueObjectKind::Table,
                name: "ordrs".to_owned(),
                entry: "shop".to_owned(),
                database: "shop".to_owned(),
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
                found: "string",
            }
            .to_string(),
            "database.shop.password_command is not an array"
        );

        // FR-SRV-030's own example.
        assert_eq!(
            Error::SeriesNotSupported {
                entry: "shop".to_owned(),
                series: "10.6".to_owned(),
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
