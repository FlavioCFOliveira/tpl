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

use crate::error::{ContextFault, DeadlineBound, DsnFault, EntryRepair, Error, ReadOnlyFault};

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
        Error::UnknownCommand { token, .. } => Cow::Owned(format!(
            "'{token}' is not a name in the command tree, which is closed; tpl matches a command \
             exactly and never by a prefix of one"
        )),
        // FR-HELP-028 raises the row's floor for this one condition: the
        // `cause` names the segment that failed **and** the node it was looked
        // for under, because a segment names no command anywhere on its own —
        // `add` is a child of `tpl cfg database` and of nothing else — so a
        // line naming only the segment would read identically for a segment
        // mistyped at any depth.
        Error::UnknownCommandPathSegment { segment, node, .. } => Cow::Owned(format!(
            "'{segment}' names no child of '{}', whose children are the whole of what the path may \
             continue with; tpl matches a segment exactly and never by a prefix of one",
            invoked(node)
        )),
        Error::UnknownFlag { token, .. } => Cow::Owned(format!(
            "'{token}' is not a flag the invoked command declares; tpl matches a long flag exactly \
             and never by a prefix of one"
        )),
        Error::MissingArgument { command, argument } => Cow::Owned(format!(
            "'{command}' takes the argument '{argument}', and the invocation supplied no value for \
             it"
        )),
        Error::MutuallyExclusiveFlags { first, second } => Cow::Owned(format!(
            "the invocation supplies both '{first}' and '{second}'; exactly one of the two may be \
             given"
        )),
        Error::MalformedValue {
            parameter,
            value,
            expected,
        } => Cow::Owned(format!(
            "'{value}' was supplied for '{parameter}', which takes {expected}"
        )),
        Error::UnknownConfigurationKey { key, .. } => Cow::Owned(format!(
            "'{key}' is outside the key space tpl cfg set writes into; the space is closed and a \
             key is never created"
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
                 '{entry}', which may state its connection and its password one way or the other \
                 and never both; nothing was written"
            ),
            EntryRepair::Unset | EntryRepair::Rewrite => format!(
                "the invocation writes {written} and database entry '{entry}' already declares \
                 {conflicting}, which may state its connection and its password one way or the \
                 other and never both; nothing was written"
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
            "'{flag}' carries one value and was given two, '{first}' and then '{second}'; tpl \
             refuses the repetition rather than letting one of them silently win"
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
        } => Cow::Owned(format!(
            "'{value}' was supplied for '{flag}', which takes {}",
            alternatives(permitted)
        )),
        // OD-08's wildcard arm. It names the token where the parser named one,
        // and says what happened where it did not — the one wording
        // FR-ERR-034's ban on naming a category cannot reach, because no
        // instance existed to name.
        Error::InvocationRejected { token } => match token {
            Some(token) => Cow::Owned(format!(
                "'{token}' was rejected while the invocation was being parsed, and tpl does not \
                 classify the refusal further"
            )),
            None => Cow::Borrowed(
                "the invocation was rejected while it was being parsed, and the parser named no \
                 token of it",
            ),
        },

        // ------------------------------------------------------------ 65 ---
        // The row obliges, for a template, the name, the line, the column and
        // the chain of underlying engine errors, per FR-ERR-011.
        Error::TemplateSyntax {
            template,
            position,
            chain,
        } => Cow::Owned(format!(
            "'{template}' could not be compiled; the parser stopped at {position} reporting {}",
            joined(chain)
        )),
        Error::RenderFailed {
            template,
            position,
            chain,
        } => Cow::Owned(format!(
            "'{template}' compiled and then failed while being evaluated; evaluation stopped at \
             {position} reporting {}",
            joined(chain)
        )),
        Error::TemplateOutsideRoot { name, root } => Cow::Owned(format!(
            "the path '{name}' resolves to lies outside the template root {}, and tpl reads no \
             template from outside it",
            root.display()
        )),
        // The row obliges the path and either the position of the malformed
        // JSON or the structural rule the document failed.
        Error::ContextDocumentMalformed { path, fault } => match fault {
            ContextFault::NotJson(position) => Cow::Owned(format!(
                "'{}' is not well-formed JSON; the parser stopped at {position}",
                path.display()
            )),
            ContextFault::Structure { rule } => Cow::Owned(format!(
                "'{}' is well-formed JSON and does not satisfy the context-document contract: \
                 {rule}",
                path.display()
            )),
        },
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
        } => Cow::Owned(format!(
            "no row of INFORMATION_SCHEMA matches {kind} '{name}' in database '{database}', read \
             through database entry '{entry}'"
        )),
        Error::TemplateNotFound { name, root } => Cow::Owned(format!(
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
        Error::NameNotResolved { host, port } => Cow::Owned(format!(
            "DNS resolution returned no address for '{host}', so the connection to port {port} was \
             never attempted"
        )),
        Error::ConnectionRefused { host, port } => Cow::Owned(format!(
            "the TCP connect to {host}:{port} returned a refusal from the host, so no session was \
             opened"
        )),
        Error::TlsHandshakeFailed { host, port } => Cow::Owned(format!(
            "the TLS handshake with {host}:{port} did not complete, so no session was opened and \
             no catalogue statement was issued"
        )),
        Error::NetworkDeadlineExceeded {
            phase,
            host,
            port,
            bound,
            limit,
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
        Error::AuthenticationRefused { user, host } => Cow::Owned(format!(
            "the server at '{host}' rejected the credentials presented for user '{user}'; the \
             refusal came from the server and not from tpl"
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
            "the walk upward ended at {} without meeting a .tpl folder",
            walk_ended_at.display()
        )),
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
        Error::ConfigurationMalformed { path, position } => Cow::Owned(format!(
            "the TOML parser stopped at {position} of {}",
            path.display()
        )),
        Error::ConfigurationKeyOutsideSpace { key, file, .. } => Cow::Owned(format!(
            "{} declares '{key}', which is outside the key space tpl recognises; an unrecognised \
             key is refused and never ignored",
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
            "{} declares both {first} and {second} for database entry '{entry}'; an entry states \
             its connection one way or the other and never both",
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
        Error::PasswordCommandDeadlineExceeded {
            command,
            bound,
            limit,
        } => match bound {
            DeadlineBound::Phase => Cow::Owned(format!(
                "password_command {command:?} had not exited when its own deadline of {limit:?} \
                 expired, and the child was terminated"
            )),
            DeadlineBound::Overall => Cow::Owned(format!(
                "password_command {command:?} had not exited when the overall budget of {limit:?}, \
                 measured from process start, expired, and the child was terminated"
            )),
        },
        Error::PasswordCommandOutputCapExceeded { command, cap } => Cow::Owned(format!(
            "password_command {command:?} was terminated after writing more than {cap} bytes to \
             its standard output"
        )),
        Error::PasswordCommandNotExecutable { command, returned } => Cow::Owned(format!(
            "password_command {command:?} could not be started: {returned}"
        )),
        Error::PasswordCommandFailed { command, status } => match status {
            Some(status) => Cow::Owned(format!(
                "password_command {command:?} exited with status {status}; its standard error went \
                 to the null device and tpl never saw it"
            )),
            None => Cow::Owned(format!(
                "password_command {command:?} was ended by a signal and returned no exit status; \
                 its standard error went to the null device and tpl never saw it"
            )),
        },
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
        Error::NoDatabaseEntrySelected { file } => Cow::Owned(format!(
            "neither -d/--database nor core.database in {} names an entry, and this command reads \
             the catalogue through one",
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
fn invoked(command: &str) -> Cow<'_, str> {
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
    fn an_empty_chain_says_so_rather_than_rendering_nothing() {
        assert_eq!(joined(&[]), EMPTY_CHAIN);
    }

    #[test]
    fn a_chain_of_one_is_the_link_itself() {
        let chain = [String::from("unexpected end of input")];
        assert_eq!(joined(&chain), "unexpected end of input");
    }

    #[test]
    fn a_chain_is_joined_outermost_first() {
        let chain = [String::from("outer"), String::from("inner")];
        assert_eq!(joined(&chain), format!("outer{CHAIN_SEPARATOR}inner"));
    }

    #[test]
    fn an_empty_window_says_so_rather_than_rendering_nothing() {
        assert_eq!(listed(&[]), EMPTY_WINDOW);
    }

    #[test]
    fn a_window_of_one_is_the_series_itself() {
        assert_eq!(listed(&["Z.9"]), "Z.9");
    }

    #[test]
    fn a_window_of_two_is_joined_by_the_conjunction_alone() {
        assert_eq!(listed(&["Z.9", "Z.8"]), "Z.9 and Z.8");
    }

    #[test]
    fn a_longer_window_separates_by_comma_and_conjoins_the_last() {
        // The form FR-SRV-030 shows.
        assert_eq!(
            listed(&["Z.9", "Z.8", "Z.7", "Z.6"]),
            "Z.9, Z.8, Z.7 and Z.6"
        );
    }
}
