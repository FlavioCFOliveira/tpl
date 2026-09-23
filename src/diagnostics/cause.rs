//! The `cause` line, derived per variant from the row `FR-ERR-034` fixes for
//! that variant's code.
//!
//! `FR-ERR-034` states, code by code, what the `cause` line is obliged to name,
//! and bans a wording that "would read identically for a different failure".
//! The error type carries between one and twenty-one variants per code, so the
//! derivation is made **per variant** and each is checked against the row for
//! its code: the row is the floor, not the text.
//!
//! `FR-ERR-010` forbids the line to restate `error:`, which is why no arm here
//! reproduces the `#[error(...)]` wording of the variant it renders. Where a
//! row obliges a fact the `error:` line already carries — the template, the
//! line and the column of a `65`, per `FR-ERR-011` — the arm carries it beside
//! the fact the row exists to add, which is the engine's own diagnosis.
//!
//! Nothing here reaches for a value the variant does not hold: `FR-GLOB-018`
//! keeps the driver's error out of the process entirely, per `OD-06`, and
//! `FR-ERR-013` and `BR-ERR-003` bar every credential, the resolved DSN and the
//! contents of `.tpl/.cfg` from the stream at any verbosity.

use std::borrow::Cow;

use crate::error::{
    ChildEnd, ContextFault, DeadlineBound, DsnFault, EntryRepair, Error, PasswordCommandFault,
    ReadOnlyFault, TlsFault, TplDirFault,
};

/// The separator between two links of a template-engine error chain.
const CHAIN_SEPARATOR: &str = ": caused by ";

/// What the `cause` line says when the engine supplied no chain at all.
const EMPTY_CHAIN: &str = "the engine reported no further detail";

/// The separator between adjacent members of a list, except before the last.
const LIST_SEPARATOR: &str = ", ";

/// The separator before the last member of a list.
const LIST_CONJUNCTION: &str = " and ";

/// The separator before the last member of a list of alternatives, where the
/// list is a choice rather than an enumeration.
const LIST_ALTERNATIVE: &str = " or ";

/// What the `cause` line says when a flag enumerates no value at all.
const NO_PERMITTED_VALUE: &str = "no value at all";

/// The program name, which is the whole of the command path at the root.
const PROGRAM: &str = "tpl";

/// What the `cause` line says when the supported window arrives empty.
const EMPTY_WINDOW: &str = "no series";

/// The content of the `cause` line for `error`, without its label.
///
/// The match is exhaustive and carries no wildcard arm, so a variant added to
/// [`Error`] does not compile until its `cause` is derived from the row of
/// `FR-ERR-034` for its code.
pub(super) fn cause(error: &Error) -> Cow<'static, str> {
    match error {
        // ------------------------------------------------------------ 64 ---
        // The row obliges the token as written, and why it was rejected.
        Error::UnknownCommand { token, node, .. } => Cow::Owned(format!(
            "'{token}' is not a subcommand of '{}' (commands are matched in full, never by a \
             prefix)",
            invoked(node)
        )),
        // FR-HELP-028 raises the row's floor for this one condition: the
        // `cause` names the segment that failed **and** the node it was looked
        // for under, because a segment names no command anywhere on its own —
        // `add` is a child of `tpl cfg database` and of nothing else — so a
        // line naming only the segment would read identically for a segment
        // mistyped at any depth.
        Error::UnknownCommandPathSegment { segment, node, .. } => Cow::Owned(format!(
            "'{segment}' is not a child of '{}' (segments are matched in full, never by a prefix)",
            invoked(node)
        )),
        Error::UnknownFlag { token, command, .. } => Cow::Owned(format!(
            "'{token}' is not a flag of '{}' or a global flag (flags are matched in full, never \
             by a prefix)",
            invoked(command)
        )),
        Error::MissingArgument { command, argument } => Cow::Owned(format!(
            "the invocation of '{}' gave no value for {argument}, which is required",
            invoked(command)
        )),
        Error::PrettyWithoutJson { command, .. } => Cow::Owned(format!(
            "--pretty indents a JSON document, and '{}' is writing text, which is the default \
             format",
            invoked(command)
        )),
        Error::ConnectionDetailsMissing { entry } => Cow::Owned(format!(
            "no --dsn, and none of --host, --port, --user or --schema, was given for entry \
             '{entry}'; --tls, --password-command, --ca-file and --ca-path do not say where to \
             connect"
        )),
        Error::NothingToUpdate { entry } => {
            Cow::Owned(format!("no field flag was given for entry '{entry}'"))
        }
        Error::BlockKeyGiven { key, entry } => match entry {
            Some(entry) => Cow::Owned(format!(
                "tpl cfg get reads one key; {key} is the block of entry '{entry}'"
            )),
            None => Cow::Owned(format!(
                "tpl cfg get reads one key; {key} is a block of keys, not a key"
            )),
        },
        Error::MutuallyExclusiveFlags { first, second } => Cow::Owned(format!(
            "the invocation supplies both '{first}' and '{second}'; exactly one of the two may be \
             given"
        )),
        // FR-SCH-008: the cause names the token as written and the spelling
        // expected. The token is reproduced byte for byte, per FR-CLI-020, and
        // is escaped on the way out by the emitter, per FR-ERR-024.
        Error::RoutinePrefixNotLowerCase { token, prefix, .. } => Cow::Owned(format!(
            "'{token}' carries the qualifying prefix '{prefix}' in a spelling other than lower \
             case; tpl accepts 'procedure:' and 'function:' and no other spelling of either"
        )),
        // FR-SCH-010: both candidates, named in the qualified form of
        // FR-SCH-008.
        Error::AmbiguousRoutineName {
            name,
            entry,
            database,
            ..
        } => Cow::Owned(format!(
            "database '{database}', read through database entry '{entry}', holds both a \
             procedure and a function named '{name}': 'procedure:{name}' and 'function:{name}'"
        )),
        // FR-RND-032, over the other context source of FR-RND-023. The line
        // names the document rather than a database entry, because
        // FR-RND-022 opened no connection and FR-RND-019 resolved no entry —
        // which is the whole of why this is not the arm above.
        Error::AmbiguousRoutineInContext {
            name,
            path,
            database,
            ..
        } => Cow::Owned(format!(
            "the --context document {} describes database '{database}', which holds both a \
             procedure and a function named '{name}': 'procedure:{name}' and 'function:{name}'",
            path.display()
        )),
        // FR-RND-014: the key is what was given twice, and both values are
        // named because the `64` row obliges the line to say why the token was
        // rejected. FR-RND-008 makes the flag itself repeatable, so a line
        // naming the flag would report the wrong fault.
        Error::RepeatedSetKey { key, first, second } => Cow::Owned(format!(
            "'--set' defines each key once and '{key}' was defined twice, as '{first}' and then \
             '{second}'"
        )),
        // FR-CACHE-019: the flag is declared by the command and contradicts
        // what the command does.
        Error::LoadWithoutStoring => Cow::Borrowed(
            "'tpl cache load' reads the server in order to store what it read, so an invocation \
             that forbids the store asks the command to do nothing",
        ),
        Error::MalformedValue {
            parameter,
            value,
            expected,
            ..
        } => Cow::Owned(format!(
            "'{value}' was supplied for '{parameter}', which takes {expected}"
        )),
        Error::UnknownConfigurationKey { key, .. } => Cow::Owned(format!(
            "'{key}' is not a configuration key; tpl cfg set writes only the keys tpl knows"
        )),
        Error::DatabaseEntryAlreadyExists { name, file } => Cow::Owned(format!(
            "{} already defines the database entry '{name}'; add creates an entry and never \
             replaces one",
            file.display()
        )),
        // FR-CFG-048 obliges this line to name both members of the pair — the
        // key the invocation writes and the key the entry already carries —
        // which is the `64` row's "both members of the mutually exclusive
        // pair". The third repair is the invocation supplying both itself, and
        // it is said so rather than attributed to the file.
        Error::IncoherentEntryWrite {
            entry,
            written,
            conflicting,
            repair,
        } => Cow::Owned(match repair {
            EntryRepair::Restate(_) => format!(
                "the invocation writes both {written} and {conflicting} to database entry \
                 '{entry}'; an entry gives its connection either as dsn or as \
                 host/port/user/database, and its password in one place only; nothing was written"
            ),
            EntryRepair::Unset | EntryRepair::Rewrite => format!(
                "the invocation writes {written} and database entry '{entry}' already declares \
                 {conflicting}; an entry gives its connection either as dsn or as \
                 host/port/user/database, and its password in one place only; nothing was written"
            ),
        }),
        Error::UnexpectedArgument { command, token } => Cow::Owned(format!(
            "'{token}' was supplied to '{}', which takes no argument in that position",
            invoked(command)
        )),
        // FR-CLI-014 obliges the message to name both values, and this is the
        // line that names them: the `error:` line names the flag, which
        // FR-ERR-010 then forbids this one to restate.
        Error::RepeatedValueFlag {
            flag,
            first,
            second,
        } => Cow::Owned(format!(
            "'{flag}' takes one value and was given two, '{first}' and then '{second}'"
        )),
        Error::RepeatedFlag { flag } => Cow::Owned(format!(
            "'{flag}' carries no value, so a second occurrence of it states nothing the first did \
             not; tpl accepts each flag once"
        )),
        Error::FlagValueMissing { flag } => Cow::Owned(format!(
            "'{flag}' carries one value and the invocation supplied none for it"
        )),
        // FR-CLI-018: the token was written as a token of its own, which is
        // the fact that decides it, and the hint carries the form that works.
        Error::SeparateTokenValue { flag, value } => Cow::Owned(format!(
            "'{value}' begins with '-' and was written as a token of its own, so tpl read it as a \
             flag rather than as the value of '{flag}'"
        )),
        Error::ValueOutsideEnumeration {
            flag,
            value,
            permitted,
            ..
        } => Cow::Owned(format!(
            "'{value}' was supplied for '{flag}', which takes {}",
            alternatives(permitted)
        )),
        // OD-08's wildcard arm. It names the token where the parser named one,
        // and says what happened where it did not — the one wording
        // FR-ERR-034's ban on naming a category cannot reach, because no
        // instance existed to name.
        Error::InvocationRejected {
            reason,
            command,
            token,
        } => match token {
            Some(token) => Cow::Owned(format!(
                "'{token}', given to '{}', was refused: {reason}",
                invoked(command)
            )),
            None => Cow::Owned(format!(
                "the invocation of '{}' was refused before any token could be named: {reason}",
                invoked(command)
            )),
        },

        // ------------------------------------------------------------ 65 ---
        // The row obliges, for a template, the name, the line, the column and
        // the chain of underlying engine errors, per FR-ERR-011.
        Error::TemplateSyntax {
            template,
            position,
            chain,
        } => Cow::Owned(format!(
            "'{template}' is not valid template syntax at {position}: {}",
            joined(chain)
        )),
        Error::RenderFailed {
            template,
            position,
            chain,
            undefined,
            ..
        } => match undefined {
            Some(expression) => Cow::Owned(format!(
                "'{template}' at {position} reads '{expression}', which is not defined in this \
                 render: {}",
                joined(chain)
            )),
            None => Cow::Owned(format!(
                "'{template}' failed while being evaluated, at {position}: {}",
                joined(chain)
            )),
        },
        Error::TemplateOutsideRoot { name, root } => Cow::Owned(format!(
            "'{name}' resolves to a path outside the template root {}, and tpl reads no template \
             from outside it",
            root.display()
        )),
        // The row obliges the path and either the position of the malformed
        // JSON or the structural rule the document failed.
        Error::ContextDocumentMalformed { path, fault } => match fault {
            ContextFault::NotJson(position) => Cow::Owned(format!(
                "'{}' is not well-formed JSON; the parser stopped at {position}",
                path.display()
            )),
            ContextFault::Structure { at, expected } if at.is_empty() => Cow::Owned(format!(
                "'{}' is well-formed JSON and is not a context document: {expected}",
                path.display()
            )),
            ContextFault::Structure { at, expected } => Cow::Owned(format!(
                "'{}' is well-formed JSON, and at {at} {expected}",
                path.display()
            )),
            // FR-CTX-042: the path, the table carrying the key, the key, and
            // the table it names that `tables` does not carry.
            ContextFault::DanglingReference {
                table,
                collection,
                key,
                names,
            } => Cow::Owned(format!(
                "'{}' is well-formed JSON, and table '{table}' lists key '{key}' under \
                 {collection}, naming table '{names}', which data.database.tables does not carry",
                path.display()
            )),
        },
        // The row obliges, for a render bound, which bound was exceeded, its
        // resolved value, and the key that raises it (FR-RND-036, FR-RND-037).
        Error::RenderFuelExhausted { fuel } => Cow::Owned(format!(
            "the render used its whole render fuel of {fuel} evaluation steps without finishing, \
             which usually means a loop that never ends; the limit is set by core.render_fuel"
        )),
        Error::RenderMemoryLimitExceeded { limit } => Cow::Owned(format!(
            "the process was observed holding more heap than the render memory limit of {limit} \
             bytes while the render ran; the bound is set by core.render_memory_limit"
        )),
        Error::RenderOutputLimitExceeded { limit } => Cow::Owned(format!(
            "the render would have produced more than its render output limit of {limit} bytes, \
             and stdout received none of it; the bound is set by core.render_output_limit"
        )),
        // The row obliges which deadline expired and its resolved value.
        Error::RenderDeadlineExceeded { bound, limit } => match bound {
            DeadlineBound::Phase => Cow::Owned(format!(
                "the render was still running when its own deadline of {limit:?} expired; that \
                 deadline bounds the render phase alone"
            )),
            DeadlineBound::Overall => Cow::Owned(format!(
                "the render was still running when the overall budget of {limit:?} expired; that \
                 budget is measured from process start and bounds every phase together"
            )),
        },

        // ------------------------------------------------------------ 66 ---
        // The row obliges the identifier, the kind it was sought as, and the
        // population it was sought in.
        Error::CatalogueObjectNotFound {
            kind,
            name,
            entry,
            database,
            ..
        } => Cow::Owned(format!(
            "no row of INFORMATION_SCHEMA matches {kind} '{name}' in database '{database}', read \
             through database entry '{entry}'"
        )),
        // The population is the document rather than a catalogue, so the line
        // names the document and the database it describes. Nothing here says
        // INFORMATION_SCHEMA, because on this path nothing read it.
        Error::ContextObjectNotFound {
            kind,
            name,
            path,
            database,
            ..
        } => Cow::Owned(format!(
            "the --context document {} describes database '{database}' and carries no {kind} \
             named '{name}'",
            path.display()
        )),
        Error::TemplateNotFound { name, root, .. } => Cow::Owned(format!(
            "no template named '{name}' exists under the template root {}",
            root.display()
        )),
        Error::DatabaseEntryNotFound { name, file, .. } => Cow::Owned(format!(
            "{} declares no database entry named '{name}'",
            file.display()
        )),
        // FR-ERR-035 separates this from the 64 and the 78 that also name a
        // key, and the exit code carries the separation: this is the key the
        // file does not set. The line does not also claim the key is one the
        // space admits, because FR-CFG-007 reaches a key outside it too — a
        // spelling `tpl cfg get` was given and `.tpl/.cfg` does not carry.
        Error::ConfigurationKeyNotFound { key, file, .. } => {
            Cow::Owned(format!("{} sets no value for '{key}'", file.display()))
        }

        // ------------------------------------------------------------ 69 ---
        // The row obliges the phase, the host and port attempted, and what
        // that phase returned. What the phase returned is the classification
        // `mariadb/` made of it: OD-06 drops the driver's own value at that
        // boundary, so the variant is the classification.
        Error::NameNotResolved { host, port, .. } => Cow::Owned(format!(
            "DNS resolution returned no address for '{host}', so the connection to port {port} was \
             never attempted"
        )),
        Error::ConnectionRefused { host, port, .. } => Cow::Owned(format!(
            "the TCP connect to {host}:{port} did not open a session: the host refused it, or \
             nothing is listening on that port"
        )),
        Error::TlsHandshakeFailed {
            host, port, fault, ..
        } => match fault {
            TlsFault::Refused => Cow::Owned(format!(
                "the TLS handshake with {host}:{port} was refused before any certificate was \
                 checked: the server offers no TLS, or the negotiation failed"
            )),
            TlsFault::CertificateRejected => Cow::Owned(format!(
                "the TLS handshake with {host}:{port} returned a certificate that was not \
                 accepted: its chain is not trusted, or it does not name the host"
            )),
        },
        Error::NetworkDeadlineExceeded {
            phase,
            host,
            port,
            bound,
            limit,
            ..
        } => match bound {
            DeadlineBound::Phase => Cow::Owned(format!(
                "{phase} for {host}:{port} had not finished when its own deadline of {limit:?} \
                 expired; that deadline bounds this phase alone"
            )),
            DeadlineBound::Overall => Cow::Owned(format!(
                "{phase} for {host}:{port} had not finished when the overall budget of {limit:?} \
                 expired; that budget is measured from process start and bounds every phase \
                 together"
            )),
        },

        // ------------------------------------------------------------ 70 ---
        // The row obliges the invariant that was violated and where. "Where"
        // is the location the condition arose at, per the ninth edition.
        Error::InternalInvariant {
            invariant,
            location,
        } => Cow::Owned(format!(
            "the invariant '{invariant}' was found violated at {}:{}:{}",
            location.file(),
            location.line(),
            location.column()
        )),

        // ------------------------------------------------------------ 73 ---
        // The row obliges the path, and whether the obstacle was an existing
        // .tpl or a failure the filesystem reported.
        Error::ProjectAlreadyExists { path } => Cow::Owned(format!(
            "{} already exists; tpl init creates a project only where there is none, and has \
             changed nothing here",
            path.display()
        )),
        Error::ProjectNotCreated { path, returned } => Cow::Owned(format!(
            "the filesystem refused to create {}: {returned}",
            path.display()
        )),

        // ------------------------------------------------------------ 74 ---
        // The row obliges the path or stream, the operation attempted on it,
        // and what the filesystem or the stream returned.
        Error::ProjectFileUnreadable { path, returned } => Cow::Owned(format!(
            "the read of {} returned: {returned}",
            path.display()
        )),
        Error::ContextDocumentUnreadable { path, returned } => Cow::Owned(format!(
            "the read of the file --context named, {}, returned: {returned}",
            path.display()
        )),
        Error::TrustMaterialUnreadable { path, returned, .. } => Cow::Owned(format!(
            "the read of {} returned: {returned}; the connection was not attempted",
            path.display()
        )),
        Error::ProjectFileUnwritable { path, returned } => Cow::Owned(format!(
            "the write of {} returned: {returned}; the previous file is still in place, unchanged",
            path.display()
        )),
        Error::StdoutUnwritable { returned } => {
            Cow::Owned(format!("the write to standard output returned: {returned}"))
        }
        Error::StdoutClosedMidDocument => Cow::Borrowed(
            "the consumer closed standard output while a JSON document was still being written, so \
             the write returned a broken pipe and the document that arrived is truncated",
        ),

        // ------------------------------------------------------------ 77 ---
        // The row obliges, for authentication, the user and the host the
        // server refused and that the refusal came from the server; and for
        // privileges, which property of which object, per FR-PRIV-013.
        Error::AuthenticationRefused { user, host, .. } => Cow::Owned(format!(
            "the server at '{host}' rejected the user '{user}' or its password; the refusal came \
             from the server"
        )),
        Error::PropertyNotReadable {
            kind,
            object,
            property,
        } => Cow::Owned(format!(
            "the catalogue did not return the {property} of {kind} '{object}' to the connecting \
             user, so the object would be incomplete and tpl does not return it in part"
        )),

        // ------------------------------------------------------------ 78 ---
        // The row obliges the key and the file, with the value found and the
        // value expected; or, where the fault is not a key, the specific
        // condition.
        Error::ProjectNotFound { walk_ended_at } => Cow::Owned(format!(
            "no .tpl folder in the working directory or in any parent of it, up to {}",
            walk_ended_at.display()
        )),
        Error::ProjectDirUnusable { path, fault } => match fault {
            TplDirFault::Missing => Cow::Owned(format!(
                "--tpl-dir disabled the upward search; nothing exists at {}",
                path.display()
            )),
            TplDirFault::NotDirectory => Cow::Owned(format!(
                "--tpl-dir disabled the upward search; {} exists and is not a folder",
                path.display()
            )),
        },
        Error::ConfigurationNotOwned {
            path,
            owner,
            expected,
        } => Cow::Owned(format!(
            "{} is owned by uid {owner}; tpl reads it only when it is owned by the invoking user, \
             uid {expected}",
            path.display()
        )),
        Error::ConfigurationUnsafeMode { path, mode } => Cow::Owned(format!(
            "mode {mode:04o} grants access to group or other; tpl reads {} only at mode 0600",
            path.display()
        )),
        Error::ConfigurationMalformed {
            path,
            position,
            reason,
        } => Cow::Owned(format!(
            "the TOML parser stopped at {position} of {}: {reason}",
            path.display()
        )),
        Error::ConfigurationKeyOutsideSpace { key, file, .. } => Cow::Owned(format!(
            "{} declares '{key}', which is not a configuration key; tpl refuses a key it does \
             not know rather than ignore it",
            file.display()
        )),
        Error::PasswordCommandNotAnArray {
            key,
            file,
            position,
            found,
        } => Cow::Owned(format!(
            "{} at {position} declares {key} as a {found}; this key takes an array of strings",
            file.display()
        )),
        Error::ConflictingEntryKeys {
            entry,
            file,
            first,
            second,
        } => Cow::Owned(format!(
            "{} declares both {first} and {second} for database entry '{entry}'; an entry gives \
             its connection either as dsn or as host/port/user/database, and its password in one \
             place only",
            file.display()
        )),
        Error::ConfigurationValueMalformed {
            key,
            file,
            position,
            found,
            expected,
        } => Cow::Owned(format!(
            "{} at {position} declares {key} as {found}; this key takes {expected}",
            file.display()
        )),
        Error::DsnMalformed { key, file, fault } => match fault {
            DsnFault::Scheme => Cow::Owned(format!(
                "{key} in {} names a scheme tpl does not accept; a DSN is written with mysql:// \
                 or mariadb://, which are equivalent",
                file.display()
            )),
            DsnFault::Form => Cow::Owned(format!(
                "{key} in {} is not of the form scheme://[user[:password]@]host[:port]/database; \
                 the host and the database are both required",
                file.display()
            )),
        },
        Error::UnclosedExpansion { key, file } => Cow::Owned(format!(
            "{key} in {} opens a ${{ expansion that is never closed; an expansion is written \
             ${{VAR}} and ends at its brace",
            file.display()
        )),
        Error::DsnQueryParameter { key, file } => Cow::Owned(format!(
            "{key} in {} carries a '?' query parameter; a DSN takes none, and every connection \
             option is a key of its own",
            file.display()
        )),
        Error::UndefinedVariable { name, key, file } => Cow::Owned(format!(
            "{key} in {} references the environment variable '{name}', and no variable of that \
             name is defined in the environment tpl was invoked with",
            file.display()
        )),
        // FR-CONF-028: the phase ends at the exit **and** the end of the
        // standard output, so a child that exited while a descendant held its
        // output is still "not finished", and the whole group is ended.
        Error::PasswordCommandDeadlineExceeded {
            command,
            bound,
            limit,
        } => match bound {
            DeadlineBound::Phase => Cow::Owned(format!(
                "password_command {command:?} had not both exited and closed its standard output \
                 when its own deadline of {limit:?} expired, and its process group was terminated"
            )),
            DeadlineBound::Overall => Cow::Owned(format!(
                "password_command {command:?} had not both exited and closed its standard output \
                 when the overall budget of {limit:?}, measured from process start, expired, and \
                 its process group was terminated"
            )),
        },
        Error::PasswordCommandOutputCapExceeded { command, cap } => Cow::Owned(format!(
            "password_command {command:?} wrote more than {cap} bytes to its standard output, and \
             its process group was terminated"
        )),
        // FR-CONF-042: the line names the command as stored, says which of the
        // two conditions occurred, and names what the operating system
        // returned. One wording for both is what that requirement was written
        // over — `could not be started` is false of a child that had started.
        Error::PasswordCommandNotExecutable {
            command,
            fault,
            returned,
        } => match fault {
            PasswordCommandFault::NotStarted => Cow::Owned(format!(
                "password_command {command:?} could not be started: {returned}"
            )),
            PasswordCommandFault::StatusUnreadable => Cow::Owned(format!(
                "password_command {command:?} was started and tpl could not read the status it \
                 ended with: {returned}"
            )),
        },
        // FR-CONF-033 for the exit and FR-CONF-043 for the signal. The signal
        // number is named because FR-ERR-034 bans a category where an instance
        // is available, and a Unix reports the signal that ended a child: a
        // line saying only that the child was signalled would read identically
        // for a supervisor, a memory limit and an interactive interrupt.
        Error::PasswordCommandFailed { command, end } => match end {
            ChildEnd::Exited(status) => Cow::Owned(format!(
                "password_command {command:?} exited with status {status}; its standard error went \
                 to the null device and tpl never saw it"
            )),
            ChildEnd::Signalled(signal) => Cow::Owned(format!(
                "password_command {command:?} was ended by signal {signal}, which tpl did not \
                 send, and returned no exit status; its standard error went to the null device and \
                 tpl never saw it"
            )),
            ChildEnd::Unreported => Cow::Owned(format!(
                "password_command {command:?} returned neither an exit status nor a signal, and \
                 the operating system reported no number for either; its standard error went to \
                 the null device and tpl never saw it"
            )),
        },
        // FR-CONF-044: the line names the directory as the entry declared it
        // and states that it yielded no certificate file. The key is named
        // beside it because the `78` row of FR-ERR-034 asks for the key where
        // the fault is one, and this fault is `ca_path`'s.
        Error::TrustDirectoryEmpty { entry, path } => Cow::Owned(format!(
            "no entry of the directory {} resolves to a regular file, so \
             'database.{entry}.ca_path' supplies no certificate to the trust material the \
             connection was to validate against",
            path.display()
        )),
        Error::ReadOnlySessionNotEnforced { entry, fault } => match fault {
            ReadOnlyFault::NotApplied => Cow::Owned(format!(
                "the session opened for database entry '{entry}' did not accept the read-only \
                 setting, so tpl cannot guarantee that the session issues no write"
            )),
            ReadOnlyFault::ReadBackDisagreed => Cow::Owned(format!(
                "the session opened for database entry '{entry}' accepted the read-only setting \
                 and the read-back of @@session.tx_read_only did not confirm it"
            )),
        },
        // FR-CONF-040 and FR-CONF-041: the cause names the file, the entry and
        // the key the entry does not carry.
        Error::EntryKeyMissing {
            entry, key, file, ..
        } => Cow::Owned(format!(
            "{} declares database entry '{entry}' without '{key}', and this command cannot be run \
             without it",
            file.display()
        )),
        Error::NoDatabaseEntrySelected { file } => Cow::Owned(format!(
            "neither -d/--database nor core.database in {} names an entry, and this command needs \
             one to know which database to use",
            file.display()
        )),
        Error::ServerNotMariaDb { entry, product } => Cow::Owned(format!(
            "database entry '{entry}' reached a server that connected and authenticated and \
             reports '{product}'; tpl reads the catalogue of MariaDB alone"
        )),
        Error::SeriesNotSupported {
            entry,
            series,
            supported,
        } => Cow::Owned(format!(
            "database entry '{entry}' connected and authenticated, and the server reports series \
             '{series}', which is outside the supported window; tpl supports {}",
            listed(supported)
        )),
    }
}

/// Punctuates a list of series in the form `FR-SRV-030` shows: the members
/// separated by commas, with `and` before the last.
///
/// Nothing here names a series. `BR-SRV-005` states the supported set exactly
/// once and elsewhere, so the window arrives on the value and this function
/// only punctuates what it is handed. The empty arm is unreachable from a
/// window that satisfies `FR-SRV-015`; it is written because the slice type
/// admits one and the match carries no wildcard.
fn listed(series: &[&'static str]) -> Cow<'static, str> {
    match series {
        [] => Cow::Borrowed(EMPTY_WINDOW),
        [only] => Cow::Borrowed(only),
        [rest @ .., last] => Cow::Owned(format!(
            "{}{LIST_CONJUNCTION}{last}",
            rest.join(LIST_SEPARATOR)
        )),
    }
}

/// The command path as a caller writes it, which at the root is the program
/// name alone.
///
/// It is `pub(crate)` rather than private because the `error:` line of
/// [`Error::UnknownCommandPathSegment`] names the **same node** this line
/// names, and `FR-HELP-028` obliges both to name it. Spelling the root twice
/// is how the two came to disagree — this line wrote `'tpl'` and that one
/// wrote `''` — so both now read one rule, and neither can be corrected
/// without the other following.
pub(crate) fn invoked(command: &str) -> Cow<'_, str> {
    if command.is_empty() {
        Cow::Borrowed(PROGRAM)
    } else {
        Cow::Owned(format!("{PROGRAM} {command}"))
    }
}

/// Joins the values a flag enumerates into the choice they are.
fn alternatives(permitted: &[String]) -> Cow<'_, str> {
    match permitted {
        [] => Cow::Borrowed(NO_PERMITTED_VALUE),
        [only] => Cow::Borrowed(only.as_str()),
        [rest @ .., last] => Cow::Owned(format!(
            "{}{LIST_ALTERNATIVE}{last}",
            rest.join(LIST_SEPARATOR)
        )),
    }
}

/// Joins a template-engine error chain, outermost first, per `FR-ERR-011`.
fn joined(chain: &[String]) -> Cow<'_, str> {
    match chain {
        [] => Cow::Borrowed(EMPTY_CHAIN),
        [only] => Cow::Borrowed(only.as_str()),
        _ => Cow::Owned(chain.join(CHAIN_SEPARATOR)),
    }
}

#[cfg(test)]
mod tests {
    use super::{CHAIN_SEPARATOR, EMPTY_CHAIN, EMPTY_WINDOW, joined, listed};

    #[test]
    fn fr_err_011_an_empty_chain_says_so_rather_than_rendering_nothing() {
        assert_eq!(joined(&[]), EMPTY_CHAIN);
    }

    #[test]
    fn fr_err_011_a_chain_of_one_is_the_link_itself() {
        let chain = [String::from("unexpected end of input")];
        assert_eq!(joined(&chain), "unexpected end of input");
    }

    #[test]
    fn fr_err_011_a_chain_is_joined_outermost_first() {
        let chain = [String::from("outer"), String::from("inner")];
        assert_eq!(joined(&chain), format!("outer{CHAIN_SEPARATOR}inner"));
    }

    #[test]
    fn fr_srv_030_an_empty_window_says_so_rather_than_rendering_nothing() {
        assert_eq!(listed(&[]), EMPTY_WINDOW);
    }

    #[test]
    fn fr_srv_030_a_window_of_one_is_the_series_itself() {
        assert_eq!(listed(&["Z.9"]), "Z.9");
    }

    #[test]
    fn fr_srv_030_a_window_of_two_is_joined_by_the_conjunction_alone() {
        assert_eq!(listed(&["Z.9", "Z.8"]), "Z.9 and Z.8");
    }

    #[test]
    fn fr_srv_030_a_longer_window_separates_by_comma_and_conjoins_the_last() {
        // The form FR-SRV-030 shows.
        assert_eq!(
            listed(&["Z.9", "Z.8", "Z.7", "Z.6"]),
            "Z.9, Z.8, Z.7 and Z.6"
        );
    }
}
