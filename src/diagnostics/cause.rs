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

use crate::error::{ContextFault, DeadlineBound, Error, ReadOnlyFault};

/// The separator between two links of a template-engine error chain.
const CHAIN_SEPARATOR: &str = ": caused by ";

/// What the `cause` line says when the engine supplied no chain at all.
const EMPTY_CHAIN: &str = "the engine reported no further detail";

/// The content of the `cause` line for `error`, without its label.
///
/// The match is exhaustive and carries no wildcard arm, so a variant added to
/// [`Error`] does not compile until its `cause` is derived from the row of
/// `FR-ERR-034` for its code.
pub(super) fn cause(error: &Error) -> Cow<'static, str> {
    match error {
        // ------------------------------------------------------------ 64 ---
        // The row obliges the token as written, and why it was rejected.
        Error::UnknownCommand { token } => Cow::Owned(format!(
            "'{token}' is not a name in the command tree, which is closed; tpl matches a command \
             exactly and never by a prefix of one"
        )),
        Error::UnknownFlag { token } => Cow::Owned(format!(
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
        Error::UnknownConfigurationKey { key } => Cow::Owned(format!(
            "'{key}' is outside the key space tpl cfg set writes into; the space is closed and a \
             key is never created"
        )),

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
        Error::DatabaseEntryNotFound { name, file } => Cow::Owned(format!(
            "{} declares no database entry named '{name}'",
            file.display()
        )),
        // FR-ERR-035 separates this from the 64 and the 78 that also name a
        // key: this key is one tpl recognises, and the file does not set it.
        Error::ConfigurationKeyNotFound { key, file } => Cow::Owned(format!(
            "'{key}' is a key tpl recognises, and {} sets no value for it",
            file.display()
        )),

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
        Error::ConfigurationKeyOutsideSpace { key, file } => Cow::Owned(format!(
            "{} declares '{key}', which is outside the key space tpl recognises; an unrecognised \
             key is refused and never ignored",
            file.display()
        )),
        Error::PasswordCommandNotAnArray { key, file, found } => Cow::Owned(format!(
            "{} declares {key} as a {found}; this key takes an array of strings",
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
        Error::SeriesNotSupported { entry, series } => Cow::Owned(format!(
            "database entry '{entry}' connected and authenticated, and the server reports series \
             '{series}', which is outside the window of series tpl supports"
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
    use super::{CHAIN_SEPARATOR, EMPTY_CHAIN, joined};

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
}
