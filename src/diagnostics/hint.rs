//! The `hint` line: the generic hint for each variant, and the character set
//! that decides whether a name may enter a runnable command.
//!
//! `FR-ERR-009` asks for a concrete, runnable command wherever one exists, and
//! `FR-ERR-012` forbids vague advice — the key, the file and the expected value
//! are named rather than the caller being told to check their configuration.
//! `FR-ERR-032` obliges the hint of a `70` to say that the condition is a
//! defect in `tpl` and is not correctable by the caller.
//!
//! `FR-ERR-022` builds a runnable command "only from literals and from names
//! matching `[A-Za-z0-9_]{1,64}`", and the twentieth edition says which of the
//! two a value is. A spelling this specification enumerates is a **literal**
//! and the character set does not govern it: a command or an alias of the
//! command tree, a flag a node declares, a key of `FR-CONF-002`. Every other
//! value is governed by the set, whatever its source — a table, a view, a
//! routine, a template, a database entry, the `<name>` segment of a
//! `database.<name>` key, the name of an environment variable — and is tested
//! on its own, the separators that join names into a command or into a key
//! being literals themselves.
//!
//! [`admits`] is that test, applied here to the two kinds of value this module
//! interpolates that the set governs: a database entry name and an
//! environment-variable name. A value it refuses is replaced by a placeholder,
//! and the command stays runnable without the caller's shell ever interpreting
//! a catalogue byte. `FR-ERR-023` carries the rule the other way, over a
//! nearest-match candidate, and [`super::suggest`] is where it is applied.
//!
//! The nearest-match half of a hint — "did you mean 'orders'?" — is not built
//! here. `FR-ERR-019` through `FR-ERR-021` select candidates from a population
//! each component owns, [`super::suggest`] owns that selection and the line it
//! composes, and this module emits the **generic** hint that `FR-ERR-023` says
//! stands alone when no candidate qualifies.

use std::borrow::Cow;
use std::path::Path;

use super::restate::{self, Edit, Replacement};
use super::suggest;
use crate::error::{
    CatalogueObjectKind, ContextFault, DeadlineBound, DsnFault, EntryNameGiven, EntryRepair, Error,
    LookupKind, Missing, NetworkPhase, ReadOnlyFault, RenderReason, TlsFault, TplDirFault,
    Unresolved,
};
use crate::project::config::keys::ValueType;

/// The longest a name the character set of `FR-ERR-022` governs may be.
const MAX_NAME: usize = 64;

/// The longest a value the character set of `FR-ERR-041` governs may be.
///
/// It is `PATH_MAX` on macOS, the smaller of the two limits of the supported
/// systems, and the number is taken rather than chosen, as `FR-ERR-041` says.
const MAX_PATH: usize = 1024;

/// The `hint` line of a `70`, whose content `FR-ERR-032` fixes.
///
/// `FR-ERR-030` gives `70` two producing conditions and `ADR-004` reaches the
/// second from the panic site, where no [`Error`] exists to match on. The text
/// is a constant rather than a literal inside the match arm so that the two
/// conditions cannot drift apart: [`hint`] serves the invariant violation and
/// [`super::render`] serves the panic, and both read this.
pub(super) const SOFTWARE_DEFECT: &str = "this is a defect in tpl and is not correctable by the \
                                          caller; report it with the command you ran and the \
                                          output of: tpl version";

/// The content of the `hint` line for `error`, without its label, with the
/// caller's `--tpl-dir` and `-d/--database` carried into every runnable `tpl`
/// command it writes (`FR-ERR-043`).
pub(super) fn hint(error: &Error) -> Cow<'static, str> {
    // FR-CLI-026: the hint is the invocation as given, with the placeholder
    // after the flag, and carrying rewrites it: a node on which -d has no
    // effect would lose the `-d <entry>` the hint exists to show (finding Y-04
    // of the eighth re-audit of rmp `#263`).
    if matches!(error, Error::FlagTookCommand { .. }) {
        return bare(error);
    }
    restate::carried(bare(error), carried(error))
}

/// Which of the two flags of `FR-ERR-043` the hint of `error` may carry.
///
/// Both, except where the hint exists to change the flag: a `hint` answering
/// `FR-GLOB-007` names an entry in place of the one `-d` gave, and one
/// answering `FR-PROJ-006` or `FR-PROJ-008` names a project in place of the one
/// `--tpl-dir` gave or failed to find. A variant added to [`Error`] carries
/// both, which is what the requirement asks of every other hint.
fn carried(error: &Error) -> restate::Carry {
    restate::Carry {
        tpl_dir: !matches!(
            error,
            Error::ProjectNotFound { .. }
                | Error::ProjectDirUnusable { .. }
                | Error::ProjectFolderNotOwned { .. }
                | Error::InitDestinationIsTplFolder { .. }
                | Error::FlagTookCommand { .. }
        ),
        // FR-PROJ-029: the hint of `tpl init` carries no flag. FR-CLI-026:
        // the hint is the invocation as given, flags included, and the value
        // the parser read for -d or --tpl-dir is the command it swallowed.
        database: !matches!(
            error,
            Error::DatabaseEntryNotFound { .. }
                | Error::InitDestinationIsTplFolder { .. }
                | Error::FlagTookCommand { .. }
        ),
    }
}

/// The `hint` line of `error` before `FR-ERR-043` carries the caller's global
/// flags into the commands it writes.
///
/// The match is exhaustive and carries no wildcard arm, so a variant added to
/// [`Error`] does not compile until it is given a hint.
fn bare(error: &Error) -> Cow<'static, str> {
    match error {
        // ------------------------------------------------------------ 64 ---
        // FR-CLI-003 obliges the nearest-match half of these two lines and
        // BR-CLI-001 routes a mistyped alias through it. The selection is
        // `cli`'s, because FR-ERR-021's populations belong to the components
        // that own them; the line is composed here, from what the variant
        // carries, and falls back to the generic hint alone where nothing
        // qualified, per FR-ERR-020. The generic half lists the children of
        // the node the token was written under, which is the population the
        // suggestion was drawn from.
        // W-05 of the sixth re-audit of rmp `#263`: the command path the
        // words form, each word its own argument. Every word is a literal of
        // the tree.
        // X-05 of the seventh re-audit: the words may end in the command's
        // own arguments, as in 'show x'.
        // Y-03 of the eighth re-audit: every other token of the invocation
        // is kept, each under its set, and where the vector cannot be
        // restated the line says what to give again.
        Error::UnknownCommand { token, node, .. } if split_invocation(node, token).is_some() => {
            match restate::split(token) {
                Some(restated) => Cow::Owned(format!(
                    "give each word as its own argument: {}{}",
                    restated.command,
                    restated.replacing()
                )),
                None if restate::follows(token) => Cow::Owned(format!(
                    "give each word as its own argument: tpl {}, then give the remaining \
                     arguments and flags again",
                    split_invocation(node, token).unwrap_or_default()
                )),
                None => Cow::Owned(format!(
                    "give each word as its own argument: tpl {}",
                    split_invocation(node, token).unwrap_or_default()
                )),
            }
        }
        Error::UnknownCommandPathSegment { segment, node, .. }
            if split_command(node, segment).is_some() =>
        {
            Cow::Owned(format!(
                "give each word as its own argument: tpl help {}",
                split_command(node, segment).unwrap_or_default()
            ))
        }
        Error::UnknownCommand { node, nearest, .. }
        | Error::UnknownCommandPathSegment { node, nearest, .. } => {
            let admitted = admitted(nearest, admits_path);
            let generic = children_of(node);

            Cow::Owned(suggest::hint_line(admitted.iter().copied(), &generic).into_owned())
        }
        // BR-ERR-004: the command whose flags are listed is the one the token
        // was given to, which the variant carries. FR-CLI-017 accepts a value
        // beginning with `-` after `--`, and a node that takes a positional
        // argument is where a caller can have meant one.
        // W-07 of the sixth re-audit of rmp `#263`: the one command that
        // writes JSON and nothing else is given no --format.
        Error::UnknownFlag { token, command, .. } if is_format_on_dump(command, token) => {
            Cow::Borrowed("remove --format and its value: tpl schema dump")
        }
        // Y-02 of the eighth re-audit of rmp `#263`: the caller's command with
        // the flag after the command that declares it.
        Error::UnknownFlag {
            token,
            belongs_to: Some(path),
            ..
        } => match restate::relocated() {
            Some(restated) => Cow::Owned(format!(
                "write the flag after the command: {}{}",
                restated.command,
                restated.replacing()
            )),
            None if admits_flag(token) && admits_path(path) => Cow::Owned(format!(
                "write {token}, with its value if it takes one, after '{path}' and not before \
                 it; list the flags of that command with: tpl help {path}"
            )),
            None => Cow::Borrowed(
                "write the flag after the command it belongs to and not before it; list the \
                 flags of a command with: tpl help <command>",
            ),
        },
        Error::UnknownFlag {
            token,
            command,
            positional,
            nearest,
            belongs_to: None,
        } => {
            let admitted = admitted(nearest, admits_flag);
            let generic = format!("list the flags of this command with: {}", help_of(command));
            let line = suggest::hint_line(admitted.iter().copied(), &generic).into_owned();

            if *positional && admits_flag(token) {
                Cow::Owned(format!(
                    "{line}; if '{token}' is a value and not a flag, write -- before it"
                ))
            } else {
                Cow::Owned(line)
            }
        }
        Error::UnexpectedArgument { command, .. } => {
            Cow::Owned(format!("show what it takes with: {}", help_of(command)))
        }
        // The flag is a spelling this corpus enumerates and is therefore a
        // literal, per FR-ERR-022; the test beside it is the defensive
        // assertion this module applies to every such value.
        // FR-CLI-026: the invocation as given, with a placeholder after the
        // flag; the command path alone where a token is refused by its set.
        Error::FlagTookCommand {
            flag,
            rebuilt,
            path,
            ..
        } => {
            let (flag, what, placeholder) = if flag == "--tpl-dir" {
                ("--tpl-dir", "the path of the .tpl folder", "<path>")
            } else if flag == "--database" {
                ("--database", "the entry name", "<entry>")
            } else {
                ("-d", "the entry name", "<entry>")
            };
            match rebuilt {
                Some(rebuilt)
                    if rebuilt.split(' ').all(|token| {
                        token == placeholder || admits_flag(token) || admits_template(token)
                    }) =>
                {
                    Cow::Owned(format!("give {what} after {flag}: tpl {rebuilt}"))
                }
                _ if admits_path(path) => Cow::Owned(format!(
                    "give {what} after {flag}: tpl {flag} {placeholder} {path}, then give the \
                     remaining arguments and flags again"
                )),
                _ => Cow::Owned(format!(
                    "give {what} after {flag}: tpl {flag} {placeholder} <command>, then give the \
                     remaining arguments and flags again"
                )),
            }
        }
        Error::RepeatedValueFlag { flag, .. } => {
            if admits_flag(flag) {
                Cow::Owned(format!("give '{flag}' once, with the value you intend"))
            } else {
                Cow::Borrowed("give the flag named above once, with the value you intend")
            }
        }
        Error::RepeatedFlag { flag } => {
            if admits_flag(flag) {
                Cow::Owned(format!("give '{flag}' once"))
            } else {
                Cow::Borrowed("give the flag named above once")
            }
        }
        // S-14: a flag with a fixed set of values names them, so the caller
        // need not ask the help for the one it must write.
        Error::FlagValueMissing { flag, permitted }
            if admits_flag(flag)
                && !permitted.is_empty()
                && permitted.iter().all(|value| admits_flag(value)) =>
        {
            Cow::Owned(format!(
                "give {} after the flag, e.g. {flag} {}",
                super::cause::alternatives(permitted),
                permitted[0]
            ))
        }
        Error::FlagValueMissing { flag, .. } => {
            if admits_flag(flag) {
                Cow::Owned(format!(
                    "give the value after the flag, or in one token: {flag}=<value>"
                ))
            } else {
                Cow::Borrowed("give the flag named above a value")
            }
        }
        // FR-CLI-018 fixes this line: the hint shows the corrected
        // `--flag=value` form. The value is the caller's own and is tested
        // before it is written into the line — with `admits_flag`, because a
        // value that reaches this condition begins with `-` and is therefore
        // shaped like one. A value the set refuses leaves the form standing
        // with its placeholder, which is what FR-ERR-023 requires and what
        // FR-CLI-018 asks for either way: the form is the correction.
        Error::SeparateTokenValue { flag, value } => {
            match (admits_flag(flag), admits_flag(value)) {
                (true, true) => Cow::Owned(format!("write the value in one token: {flag}={value}")),
                (true, false) => {
                    Cow::Owned(format!("write the value in one token: {flag}=<value>"))
                }
                (false, _) => {
                    Cow::Borrowed("write the value in one token, joined to its flag by '='")
                }
            }
        }
        // The permitted values are spellings the tree enumerates, so each is a
        // literal; the first one is written after the flag as the worked
        // correction, and the help of the node states them all.
        Error::ValueOutsideEnumeration {
            flag,
            command,
            permitted,
            ..
        } => match permitted.first() {
            Some(first) if admits_flag(flag) && admits_flag(first) => Cow::Owned(format!(
                "give one of the values named above, e.g. {flag} {first}; they are listed by: {}",
                help_of(command)
            )),
            _ => Cow::Owned(format!(
                "show the values it accepts with: {}",
                help_of(command)
            )),
        },
        Error::InvocationRejected { command, .. } => {
            Cow::Owned(format!("show what it accepts with: {}", help_of(command)))
        }
        // V-08: `-d` selects an entry for a command that reads a catalogue,
        // and a `cfg database` command takes the entry as NAME instead.
        Error::MissingArgument { command, argument } => {
            let named = (argument == "<NAME>" && command.starts_with("cfg database "))
                .then(restate::database)
                .flatten();
            let restated = named.as_deref().and_then(|entry| {
                restate::restated(&[Edit::Remove(&["database"]), Edit::Append(&[entry])])
            });
            match restated {
                Some(restated) => Cow::Owned(format!(
                    "-d is not used here; give the entry as NAME: {}{}",
                    restated.command,
                    restated.replacing()
                )),
                None => Cow::Owned(format!("show the usage with: {}", help_of(command))),
            }
        }
        // Neither flag enters a runnable command, so FR-ERR-022 is not engaged
        // and both are named as prose, escaped on the way out. The pair is
        // nonetheless tested, as a defensive assertion: a flag is a spelling
        // this corpus enumerates, so a token that fails the test came from
        // somewhere other than the flag table and is not reproduced.
        // FR-RND-041: the flag to remove for each outcome, and never the path.
        Error::DirectWithContext => Cow::Borrowed(
            "remove --direct to render from the document, or remove --context to read the server",
        ),
        Error::MutuallyExclusiveFlags { first, second } => {
            if admits_flag(first) && admits_flag(second) {
                Cow::Owned(format!("give '{first}' or '{second}', and not both"))
            } else {
                Cow::Borrowed("give one of the two flags named above, and not both")
            }
        }
        // FR-OUT-009: the correction is to add the format, or to drop the
        // flag. The command is written out only where it needs nothing else
        // to run: a node that requires an operand would be written without it,
        // and BR-ERR-004 bars a hint that cannot succeed.
        Error::PrettyWithoutJson { command, complete } => {
            // T-01: the caller's own command with the format added, every
            // other flag and operand kept.
            if let Some(restated) = restate::restated(&[Edit::FormatJson]) {
                Cow::Owned(format!(
                    "add --format json: {}{}; or drop --pretty",
                    restated.command,
                    restated.replacing()
                ))
            } else if *complete && admits_path(command) {
                Cow::Owned(format!(
                    "add --format json, e.g.: tpl {command} --format json --pretty; or drop \
                     --pretty"
                ))
            } else {
                Cow::Borrowed("add --format json to the same command, or drop --pretty")
            }
        }
        // FR-CFG-016: the four flags that say where to connect, and the one
        // that says it all at once. The host, the user and the database are
        // values only the caller knows, so they stay placeholders.
        // T-01: the caller's own command with the three flags added, every
        // other flag kept.
        Error::ConnectionDetailsMissing { entry, .. } => {
            const WHERE: &[&str] = &[
                "--host",
                "<host>",
                "--user",
                "<user>",
                "--schema",
                "<database>",
            ];
            match restate::restated(&[Edit::Append(WHERE)]) {
                Some(restated) => Cow::Owned(format!(
                    "say where to connect, e.g.: {}{}",
                    restated.command,
                    restated.replacing()
                )),
                None => Cow::Owned(format!(
                    "say where to connect, e.g.: tpl cfg database add {} --host <host> --user \
                     <user> --schema <database>",
                    entry_or_placeholder(entry)
                )),
            }
        }
        // FR-CFG-020 fixes this line: every flag of FR-CFG-027.
        Error::NothingToUpdate { .. } => Cow::Borrowed(
            "give at least one of --dsn, --host, --port, --user, --schema, --tls, \
             --password-command, --ca-file, --ca-path",
        ),
        // FR-CFG-007 fixes both halves: the entry's own `show` where the key
        // names an entry the file defines, and the listing otherwise.
        Error::BlockKeyGiven { entry, .. } => match entry {
            Some(entry) if admits(entry) => Cow::Owned(format!(
                "show the entry with: tpl cfg database show {entry}"
            )),
            _ => Cow::Borrowed("show every key and its value with: tpl cfg list"),
        },
        // FR-SCH-008 fixes this line: the same invocation, with the prefix in
        // lower case. The invocation below `tpl` is a spelling this corpus
        // enumerates and the routine name is not, so the name is tested under
        // FR-ERR-022 and the whole command is dropped under FR-ERR-023 where
        // it falls outside the set.
        Error::RoutinePrefixNotLowerCase {
            prefix,
            name,
            invocation,
            template,
            ..
        } => {
            if admits(name) {
                let corrected = format!("{prefix}:{name}");
                match restate::restated(&[routine_as(&corrected)]) {
                    // T-01: the whole invocation, with the one token changed.
                    Some(restated) => Cow::Owned(format!(
                        "write it as: {}{}",
                        restated.command,
                        restated.replacing()
                    )),
                    None => {
                        let invocation = with_template(invocation, template.as_deref());
                        Cow::Owned(format!("write it as: tpl {invocation} {corrected}"))
                    }
                }
            } else {
                Cow::Borrowed(
                    "write the qualifying prefix in lower case: 'procedure:' or 'function:'",
                )
            }
        }
        // FR-SCH-010 refuses the bare name and names both candidates in the
        // `cause`; the hint is the runnable command FR-ERR-009 asks for, which
        // is the same invocation qualified. The second arm is the same
        // correction over the other context source.
        Error::AmbiguousRoutineName {
            name,
            invocation,
            template,
            ..
        }
        | Error::AmbiguousRoutineInContext {
            name,
            invocation,
            template,
            ..
        } => {
            if admits(name) {
                let procedure = format!("procedure:{name}");
                let function = format!("function:{name}");
                match (
                    restate::restated(&[routine_as(&procedure)]),
                    restate::restated(&[routine_as(&function)]),
                ) {
                    // T-01: the whole invocation twice, each with the one
                    // token qualified; the placeholders are the same in both.
                    (Some(first), Some(second)) => Cow::Owned(format!(
                        "name the kind you mean: {}, or {}{}",
                        first.command,
                        second.command,
                        second.replacing()
                    )),
                    _ => {
                        let invocation = with_template(invocation, template.as_deref());
                        Cow::Owned(format!(
                            "name the kind you mean: tpl {invocation} {procedure}, or tpl \
                             {invocation} {function}"
                        ))
                    }
                }
            } else {
                Cow::Borrowed("name the kind you mean, with the prefix 'procedure:' or 'function:'")
            }
        }
        // FR-RND-012 confines a `--set` key to the character set FR-ERR-022
        // admits, so the key reaches the runnable command; the test beside it
        // is the defensive assertion this module applies to every such value.
        Error::RepeatedSetKey { key, .. } => {
            if admits(key) {
                Cow::Owned(format!(
                    "give '--set {key}=<value>' once, with the value you intend"
                ))
            } else {
                Cow::Borrowed("define the key named above once, with the value you intend")
            }
        }
        // FR-CACHE-018 accepts `--direct` on this command and ignores it, so
        // the line names the invocation that does what the caller asked for.
        // W-07 of the sixth re-audit of rmp `#263`: with an object named, the
        // read that matches it is that object's own `schema` subcommand.
        Error::LoadWithoutStoring { object: None } => Cow::Borrowed(
            "remove --no-cache from the command; to read from the server without storing, use \
             tpl schema dump --direct --no-cache instead",
        ),
        Error::LoadWithoutStoring {
            object: Some((kind, name)),
        } => {
            let name = if admits(name) {
                name.as_str()
            } else {
                "<name>"
            };
            Cow::Owned(format!(
                "remove --no-cache from the command; to read from the server without storing, \
                 use tpl schema {kind} {name} --direct --no-cache instead"
            ))
        }
        // FR-CFG-031: the `cause` carries the form expected, and the hint the
        // worked value. The example carries no user, so that no `@` reaches a
        // hint line: the assertion over every hint looks for that character
        // to prove no value reached one ungated.
        Error::MalformedValue {
            parameter,
            command,
            value,
            ..
        } => match (parameter.as_str(), command.as_str()) {
            // FR-CONF-047: the caller's own command with the path in place of
            // the reference.
            (parameter, _) if super::cause::is_path_reference(parameter, value) => {
                path_itself(parameter)
            }
            // V-06: an example whose reference stands for one part only.
            (parameter, _) if super::cause::is_whole_dsn_reference(parameter, value) => {
                const URL: &str = "'mysql://db.example.com/${SHOP_DB}'";
                if parameter == "--dsn" {
                    Cow::Owned(format!(
                        "write the URL itself, with a ${{VAR}} for one part at most, e.g.: \
                         --dsn {URL}"
                    ))
                } else if admits_key(parameter) {
                    Cow::Owned(format!(
                        "write the URL itself, with a ${{VAR}} for one part at most, e.g.: tpl \
                         cfg set {parameter} {URL}"
                    ))
                } else {
                    Cow::Owned(format!(
                        "write the URL itself, with a ${{VAR}} for one part at most, e.g.: {URL}"
                    ))
                }
            }
            (parameter, _) if super::cause::is_port_reference(parameter, value) => Cow::Borrowed(
                "give the port as a number, e.g. 3306; to take it from the environment, \
                     edit .tpl/.cfg and write port = \"${VAR}\" in the entry's block",
            ),
            ("--dsn", _) => Cow::Borrowed(
                "write the URL as scheme://host/database, e.g.: --dsn \
                 mysql://db.example.com:3306/shop",
            ),
            // T-03, FR-CONF-046: the caller's own command with an example
            // command line in place of the value, quoted as the one string the
            // value is, which says the form by showing it. U-07: the example
            // is the whole of it; a placeholder explained in the same words
            // said the form twice.
            (parameter, _) if super::cause::is_password_command(parameter) => {
                const LINE: &str = "'pass db/shop'";
                let ids: &[&str] = if parameter == "--password-command" {
                    &["password_command"]
                } else {
                    &["value"]
                };
                let example = Edit::Value {
                    ids,
                    to: Replacement::Literal(LINE),
                };
                match restate::restated(&[example]) {
                    Some(restated) => Cow::Owned(format!(
                        "write the command as one command line, e.g.: {}{}",
                        restated.command,
                        restated.replacing()
                    )),
                    None if admits_key(parameter) => Cow::Owned(format!(
                        "write the command as one command line, e.g.: tpl cfg set {parameter} \
                         {LINE}"
                    )),
                    None => Cow::Owned(format!(
                        "write the command as one command line, e.g.: --password-command {LINE}"
                    )),
                }
            }
            (_, "cfg set") => {
                Cow::Borrowed("show every key and the value it takes with: tpl help cfg set")
            }
            _ => Cow::Owned(format!("show what it takes with: {}", help_of(command))),
        },
        // FR-CFG-009 obliges the nearest-match half over the enumerated key
        // space of FR-CONF-002. Every key of that space is a spelling this
        // corpus fixes, so FR-ERR-022 makes it a literal and the test beside it
        // is the defensive assertion this module applies to every such value.
        Error::UnknownConfigurationKey { nearest, .. } => {
            let admitted = admitted(nearest, admits_key);
            suggest::hint_line(
                admitted.iter().copied(),
                "list every key, its type and its default with: tpl help cfg set",
            )
        }
        // FR-CFG-017 obliges this hint to point at `tpl cfg database update`.
        // U-07: the caller's own command, whole, with add made update, so that
        // the values it gave are not replaced by placeholders.
        Error::DatabaseEntryAlreadyExists { name, .. } => {
            let edit = Edit::Command {
                from: "add",
                to: "update",
            };
            match restate::restated(&[edit]) {
                Some(restated) => Cow::Owned(format!(
                    "change it instead with: {}{}",
                    restated.command,
                    restated.replacing()
                )),
                None => Cow::Owned(format!("change it instead with: {}", update_entry(name))),
            }
        }
        // FR-CFG-048 obliges a runnable command that makes the change the
        // invocation asked for and deletes nothing it did not name
        // (BR-ERR-005), and its table fixes that command for each refused
        // pair. Which row applies is decided where the refusal is raised,
        // because only the writer knows where each member came from.
        Error::IncoherentEntryWrite {
            entry,
            conflicting,
            repair,
            ..
        } => {
            let named = if admits(entry) {
                entry.as_str()
            } else {
                "<entry>"
            };
            match repair {
                EntryRepair::Unset => Cow::Owned(format!(
                    "remove the key it conflicts with: {}, then run the command again",
                    unset_key(conflicting)
                )),
                // FR-ERR-045: the field is changed inside the dsn.
                EntryRepair::InsideDsn { fields } => Cow::Owned(format!(
                    "write the whole connection as a new dsn: tpl cfg database update {named} \
                     --dsn <url>, where <url> is the connection URL with the new {}",
                    super::cause::conjoined(fields)
                )),
                EntryRepair::DsnWithoutPassword => Cow::Owned(format!(
                    "write the dsn again without its password: tpl cfg database update {named} \
                     --dsn <url>, where <url> is the connection URL with no password in it, \
                     then run the command again"
                )),
                EntryRepair::Discrete { changed, .. } => Cow::Owned(discrete_entry(named, changed)),
                EntryRepair::Restate(command) => Cow::Owned(format!(
                    "write a DSN that carries no password: {}",
                    restate_entry(command, entry)
                )),
            }
        }

        // FR-PROJ-029: `tpl init` with the parent directory, and no flag.
        Error::InitDestinationIsTplFolder { parent, .. } => match parent {
            None => Cow::Borrowed("create the project in the parent directory: tpl init"),
            Some(parent) if admits_file(parent) => Cow::Owned(format!(
                "create the project in the parent directory: tpl init {}",
                parent.display()
            )),
            Some(_) => Cow::Borrowed(
                "create the project in the parent directory: tpl init <path>; replace <path> \
                 with the directory that holds the .tpl folder named above",
            ),
        },
        // FR-CONF-048: the form, with a placeholder for the name.
        Error::InvalidEntryName { given, .. } => invalid_entry_name(*given),
        // FR-CONF-049: the reference in the form ${NAME}, with a placeholder
        // for the name.
        Error::InvalidReference { parameter, .. } => invalid_reference(parameter),
        // FR-CONF-050: the flag or the key with a placeholder.
        Error::EmptyValue { parameter, .. } => empty_value(parameter),
        // ------------------------------------------------------------ 65 ---
        // The template is the one the parser stopped in, and `tpl template
        // check` is syntax analysis alone, so it is the command that confirms
        // the correction — named for this template rather than for all of
        // them, which is the invocation the caller may have just run.
        Error::TemplateSyntax {
            template, position, ..
        } => {
            if admits_template(template) {
                Cow::Owned(format!(
                    "correct line {} of template '{template}', then check it with: tpl template \
                     check {template}",
                    position.line
                ))
            } else {
                Cow::Borrowed(
                    "correct the line named above, then check the templates with: tpl template \
                     check",
                )
            }
        }
        // The most common failure a template author meets: a variable that
        // exists only when a flag binds it. The first segment of the
        // undefined expression says which flag, and BR-ERR-004 asks for that
        // flag rather than for the source. `tpl template check` performs
        // syntax analysis alone, per FR-TMPL-017, so it would not reproduce an
        // evaluation failure; the source is what locates anything else.
        Error::RenderFailed {
            template,
            undefined,
            reason,
            ..
        } => {
            // AB-03: an attribute read under a key of `vars` that no `--set`
            // gave; a `--set` value is a string, so no `--set` can supply it.
            let from_set = match reason.as_deref() {
                None => true,
                Some(RenderReason::Missing(Missing::Attribute { owner, .. })) => owner == "vars",
                Some(_) => false,
            };
            if from_set && let Some(key) = undefined.as_deref().and_then(past_a_vars_key) {
                return beyond_set(key, template);
            }

            match reason.as_deref() {
                Some(RenderReason::Unresolved(unresolved)) => return listing_of(unresolved),
                Some(RenderReason::IncludeNotFound { name, nearest, .. }) => {
                    return include_of(name, nearest);
                }
                // Y-01: the object flag was given, so the fault is the
                // template's and the hint never asks for the flag.
                Some(RenderReason::Missing(missing)) => return missing_step(missing, template),
                Some(RenderReason::Failed(_)) | None => {}
            }

            if let Some(line) = undefined.as_deref().and_then(defining_flag) {
                return line;
            }

            if admits_template(template) {
                Cow::Owned(format!(
                    "print the template's source with: tpl template show {template}"
                ))
            } else {
                Cow::Borrowed("print the template's source with: tpl template show <template>")
            }
        }
        Error::TemplateOutsideRoot { .. } => {
            Cow::Borrowed("print the template folder with: tpl template path")
        }
        // The document came from the caller, and `--context` excludes `-d`, so
        // the entry that would produce a valid one is a value only the caller
        // knows, and a placeholder stands for it. U-09: the dump goes to a new
        // file, never to the one the caller gave, which a redirect would empty
        // and which may hold the caller's own edits.
        Error::ContextDocumentMalformed {
            path,
            fault,
            default_entry,
        } => {
            let target = if path.file_name() == Some(std::ffi::OsStr::new(NEW_CONTEXT)) {
                OTHER_CONTEXT
            } else {
                NEW_CONTEXT
            };
            let lead = match fault {
                ContextFault::NotJson(_) | ContextFault::Empty => "write a well-formed document",
                ContextFault::Structure { .. } | ContextFault::DanglingReference { .. } => {
                    "write a document that matches"
                }
            };
            // With core.database set, the dump reads that entry and needs
            // no -d; without it, the entry is a value only the caller knows.
            let selection = if *default_entry { "" } else { "-d <entry> " };
            Cow::Owned(format!(
                "{lead} to a new file with: tpl {selection}schema dump > {target}, then render \
                 with --context {target}"
            ))
        }
        Error::RenderFuelExhausted { .. } => Cow::Borrowed(
            "look for a loop that never ends, or raise the limit with: tpl cfg set \
             core.render_fuel <evaluation steps>",
        ),
        Error::RenderMemoryLimitExceeded { .. } => Cow::Borrowed(
            "raise the render memory limit with: tpl cfg set core.render_memory_limit <bytes>",
        ),
        Error::RenderOutputLimitExceeded { .. } => Cow::Borrowed(
            "raise the render output limit with: tpl cfg set core.render_output_limit <bytes>",
        ),
        Error::RenderDeadlineExceeded { bound, .. } => match bound {
            DeadlineBound::Phase => Cow::Borrowed(
                "raise the render deadline with: tpl cfg set core.render_timeout <seconds>",
            ),
            DeadlineBound::Overall => Cow::Borrowed(OVERALL_BUDGET),
        },

        // ------------------------------------------------------------ 66 ---
        // The kind is always one of the three FR-SCH-010 names, so `listing`
        // always answers here; the fourth kind reaches `77` under FR-PRIV-021
        // and never this arm. The fall-through is the generic advice rather
        // than an invariant, because a hint is not the place to raise one.
        //
        // FR-SCH-010 obliges the nearest-match half over the objects of that
        // kind that do exist. The population is the reader's, because
        // FR-ERR-021's populations belong to the components that own them; the
        // line is composed here from what the variant carries, and an object
        // name is a value this corpus does not fix, so FR-ERR-022 governs it
        // by the character set and FR-ERR-023 drops a candidate outside it.
        // FR-CACHE-040: the nearest cached names, and no clean command — not
        // of a candidate, which the invocation did not name, and not of the
        // whole cache (BR-ERR-005).
        Error::NothingCachedNamed {
            name,
            held_as: Some(other),
            ..
        } if admits(name) => Cow::Owned(format!(
            "did you mean '{other}:{name}'? the cache holds a {other} named '{name}'; nothing was \
             removed"
        )),
        Error::NothingCachedNamed {
            held_as: Some(other),
            ..
        } => Cow::Owned(format!(
            "the cache holds a {other} of that name; nothing was removed"
        )),
        Error::NothingCachedNamed { nearest, .. } => {
            let admitted = admitted(nearest, admits);
            if admitted.is_empty() {
                Cow::Borrowed(
                    "nothing of that name is cached, so nothing cached is stale for it; nothing \
                     was removed",
                )
            } else {
                Cow::Owned(
                    suggest::hint_line(admitted.iter().copied(), "nothing was removed")
                        .into_owned(),
                )
            }
        }
        Error::CatalogueObjectNotFound { kind, nearest, .. } => {
            let generic = match listing(*kind) {
                // FR-ERR-043: the entry is written only where the caller gave
                // it with -d, and then it is carried in; one taken from
                // core.database or TPL_DATABASE resolves again by itself.
                Some(listing) => Cow::Owned(format!(
                    "list the available {listing} with: tpl schema {listing}"
                )),
                None => Cow::Borrowed("name an object that exists, then run the command again"),
            };
            let admitted = admitted(nearest, admits);

            Cow::Owned(suggest::hint_line(admitted.iter().copied(), &generic).into_owned())
        }
        // FR-RND-032 obliges the nearest-match half over the objects the
        // document does carry. `tpl` has no subcommand that lists a file the
        // caller supplied, so the generic half names the member of the
        // document that holds the names.
        Error::ContextObjectNotFound {
            kind,
            nearest,
            path,
            ..
        } => {
            let generic = match listing(*kind) {
                // No tpl command lists a document the caller supplied; jq, an
                // external tool, does, and the line says it is one.
                Some(listing) if admits_file(path) => Cow::Owned(format!(
                    "name one the --context document lists under data.database.{listing}; list \
                     them with jq, if it is installed: jq -r '.data.database.{listing}[].name' {}",
                    path.display()
                )),
                Some(listing) => Cow::Owned(format!(
                    "name one the --context document lists under data.database.{listing}, then \
                     run the command again"
                )),
                None => Cow::Borrowed(
                    "name an object the --context document carries, then run the command again",
                ),
            };

            Cow::Owned(
                suggest::hint_line(admitted(nearest, admits).iter().copied(), &generic)
                    .into_owned(),
            )
        }
        // FR-TMPL-027 obliges the nearest-match half over the template names
        // that do exist. A template name is a path below the template root,
        // so FR-ERR-041 governs it rather than the set of FR-ERR-022, and a
        // nested name is suggested like any other.
        Error::TemplateNotFound { nearest, .. } => suggested(
            nearest,
            admits_template,
            "list the project's templates with: tpl template list",
        ),
        // FR-GLOB-007 obliges the nearest-match half over the entry names the
        // file defines. An entry name is a value this corpus does not fix, so
        // FR-ERR-022 governs it by the character set and FR-ERR-023 drops a
        // candidate outside it in every form.
        // FR-CFG-054: the nearest declared entry, written into the command
        // that sets it; else the listing; else, with no entry at all, the
        // command that adds one, with its placeholder.
        Error::DefaultEntryUndeclared {
            nearest, declared, ..
        } => {
            let admitted = admitted(nearest, admits);
            match admitted.first() {
                Some(entry) => {
                    let verb = if admitted.len() == 1 {
                        "set it"
                    } else {
                        "set the nearest"
                    };
                    Cow::Owned(
                        suggest::hint_line(
                            admitted.iter().copied(),
                            &format!("{verb} with: tpl cfg set core.database {entry}"),
                        )
                        .into_owned(),
                    )
                }
                None if *declared => Cow::Borrowed("list the entries with: tpl cfg database list"),
                None => Cow::Borrowed("add the entry first with: tpl cfg database add <name>"),
            }
        }
        Error::DatabaseEntryNotFound {
            nearest,
            by_default,
            ..
        } => suggested(
            nearest,
            admits,
            if *by_default {
                "list the entries with: tpl cfg database list, or change the default with: tpl \
                 cfg set core.database <entry>"
            } else {
                "list the entries with: tpl cfg database list"
            },
        ),
        // FR-CFG-007 obliges the nearest-match half over the whole key space,
        // and BR-ERR-004 the statement that the file does not set a
        // candidate: `tpl cfg get` or `tpl cfg unset` of it is a 66 again.
        // FR-CFG-012: a section the file lacks; nothing is misspelt.
        Error::ConfigurationKeyNotFound { key, .. }
            if crate::error::section_named(key).is_some() =>
        {
            Cow::Borrowed("show every key and its value with: tpl cfg list")
        }
        // FR-CFG-012, fifty-ninth edition: an entry's block the file does not
        // declare. The candidates are the blocks it does declare, and the
        // hint shows the nearest rather than deleting it, per BR-ERR-005.
        // With none admitted, the entries are listed.
        Error::ConfigurationKeyNotFound {
            key,
            nearest,
            entry_missing: true,
            ..
        } if crate::project::config::keys::Key::parse(key).is_none() => {
            let admitted: Vec<&str> = nearest
                .iter()
                .map(|(candidate, _)| candidate.as_str())
                .filter(|candidate| candidate.strip_prefix("database.").is_some_and(admits))
                .collect();
            match admitted
                .first()
                .and_then(|block| block.strip_prefix("database."))
            {
                Some(entry) => {
                    let verb = if admitted.len() == 1 {
                        "show it"
                    } else {
                        "show the nearest"
                    };
                    Cow::Owned(
                        suggest::hint_line(
                            admitted.iter().copied(),
                            &format!("{verb} with: tpl cfg database show {entry}"),
                        )
                        .into_owned(),
                    )
                }
                None => Cow::Borrowed("list the entries with: tpl cfg database list"),
            }
        }
        Error::ConfigurationKeyNotFound {
            nearest,
            known,
            entry_missing,
            ..
        } => {
            let kept: Vec<&(String, bool)> = nearest
                .iter()
                .filter(|(candidate, _)| admits_key(candidate))
                .collect();
            let admitted: Vec<&str> = kept.iter().map(|(key, _)| key.as_str()).collect();
            let unset: Vec<&str> = kept
                .iter()
                .filter(|(_, set)| !set)
                .map(|(key, _)| key.as_str())
                .collect();
            // U-01: a key of the space the file does not set is offered no
            // candidate, so the next step is the listing of what is set.
            // U-05: a missing entry is recovered from the list of entries.
            let generic = if *entry_missing {
                "list the entries with: tpl cfg database list"
            } else if *known && unset.is_empty() {
                "list the keys that are set with: tpl cfg list"
            } else {
                "list every key, its type and its default with: tpl help cfg set"
            };
            let statement = match (unset.as_slice(), admitted.len()) {
                ([], _) => String::new(),
                ([only], 1) => match crate::project::config::keys::Key::parse(only)
                    .and_then(|key| key.default_value())
                {
                    Some(_) => ".tpl/.cfg does not set it, so its default applies; ".to_owned(),
                    None => ".tpl/.cfg does not set it; ".to_owned(),
                },
                (all, count) if all.len() == count => ".tpl/.cfg sets none of them; ".to_owned(),
                (some, _) => {
                    let quoted: Vec<String> = some.iter().map(|key| format!("'{key}'")).collect();
                    format!(
                        ".tpl/.cfg does not set {}; ",
                        super::cause::alternatives(&quoted)
                    )
                }
            };
            let generic = format!("{statement}{generic}");

            Cow::Owned(suggest::hint_line(admitted.iter().copied(), &generic).into_owned())
        }

        // ------------------------------------------------------------ 69 ---
        // BR-ERR-004: the entry is the one the connection was opened for, and
        // it is written into the command rather than left as a placeholder.
        // The new host and port are values only the caller knows.
        // FR-ERR-045: an entry defined by dsn is repointed through its dsn.
        Error::NameNotResolved {
            entry,
            by_dsn: true,
            ..
        } => Cow::Owned(format!(
            "check the host name, or change it inside the dsn: {}",
            inside_dsn(entry, "host")
        )),
        Error::NameNotResolved { entry, .. } => Cow::Owned(format!(
            "check the host name, or change it with: tpl cfg database update {} --host <host>",
            entry_or_placeholder(entry)
        )),
        Error::ConnectionRefused {
            entry,
            port,
            by_dsn: true,
            ..
        } => Cow::Owned(format!(
            "check that the server is running and listening on port {port}, or change the \
             address inside the dsn: {}",
            inside_dsn(entry, "host or port")
        )),
        Error::ConnectionRefused { entry, port, .. } => Cow::Owned(format!(
            "check that the server is running and listening on port {port}, or change the \
             address with: tpl cfg database update {} --host <host> --port <port>",
            entry_or_placeholder(entry)
        )),
        // The two remedies follow what the handshake returned. A server that
        // offers no TLS is reached in the clear only by lowering the mode, and
        // a certificate that is not trusted is either vouched for or not
        // checked; `tpl cfg set` reaches the key because the file is valid by
        // the time a connection is attempted.
        Error::TlsHandshakeFailed { entry, fault, .. } => {
            let entry = entry_or_placeholder(entry);

            match fault {
                TlsFault::Refused => Cow::Owned(format!(
                    "check that the server offers TLS; for a server without it, on a network you \
                     trust, allow a plain connection with: tpl cfg set database.{entry}.tls \
                     preferred"
                )),
                TlsFault::CertificateRejected => Cow::Owned(format!(
                    "trust the server's certificate authority with: tpl cfg set \
                     database.{entry}.ca_file <pem-file>, or encrypt without checking the \
                     certificate with: tpl cfg set database.{entry}.tls required"
                )),
            }
        }
        // V-04: a TCP connect that does not finish is most often a wrong
        // address or a firewall, so the address comes before the deadline.
        Error::NetworkDeadlineExceeded {
            entry,
            phase: NetworkPhase::TcpConnect,
            bound,
            by_dsn,
            ..
        } => Cow::Owned(format!(
            "check that the host and port named above are the server's address and that it is \
             reachable from here, or change them {}; to wait longer, {}",
            if *by_dsn {
                format!("inside the dsn: {}", inside_dsn(entry, "host or port"))
            } else {
                format!(
                    "with: tpl cfg database update {} --host <host> --port <port>",
                    entry_or_placeholder(entry)
                )
            },
            match bound {
                DeadlineBound::Phase =>
                    "raise the deadline with: tpl cfg set core.connect_timeout \
                                         <seconds>",
                DeadlineBound::Overall =>
                    "run the same command again with a larger --timeout \
                                           <seconds>",
            }
        )),
        Error::NetworkDeadlineExceeded { phase, bound, .. } => match bound {
            DeadlineBound::Phase => Cow::Owned(format!(
                "raise the deadline with: tpl cfg set {} <seconds>",
                deadline_key(*phase)
            )),
            DeadlineBound::Overall => Cow::Borrowed(OVERALL_BUDGET),
        },

        // ------------------------------------------------------------ 70 ---
        // FR-ERR-032: the hint of a 70 says the condition is a defect in tpl
        // and is not correctable by the caller. The panic path reads the same
        // constant, per ADR-004.
        Error::InternalInvariant { .. } => Cow::Borrowed(SOFTWARE_DEFECT),

        // ------------------------------------------------------------ 73 ---
        Error::ProjectAlreadyExists { .. } => Cow::Borrowed(
            "use the project that is already here, or create one elsewhere with: tpl init <path>",
        ),
        Error::ProjectNotCreated { .. } => Cow::Borrowed(
            "create the project where the invoking user may write, with: tpl init <path>",
        ),

        // ------------------------------------------------------------ 74 ---
        Error::ProjectFileUnreadable { .. } => Cow::Borrowed(
            "make the path named above readable by the invoking user, then run the command again",
        ),
        Error::ContextDocumentUnreadable { .. } => Cow::Borrowed(
            "check the path given to --context, or give --context - to read the document from \
             standard input",
        ),
        Error::TrustMaterialUnreadable { entry, key, .. } => Cow::Owned(format!(
            "make the path readable by the invoking user, or point the setting elsewhere with: \
             tpl cfg set database.{}.{key} <path>",
            entry_or_placeholder(entry)
        )),
        Error::ProjectFileUnwritable { .. } => Cow::Borrowed(
            "free space on the filesystem, or make .tpl writable by the invoking user, then run \
             the command again",
        ),
        Error::StdoutUnwritable { .. } => Cow::Borrowed(
            "free space on the destination, or send the output elsewhere, then run the command \
             again",
        ),
        Error::StdoutClosedMidDocument => Cow::Borrowed(
            "write the document to a file rather than into a consumer that may close early, then \
             read it from there",
        ),

        // ------------------------------------------------------------ 77 ---
        // The usual cause is the password, and the entry states it in one of
        // three places; `tpl cfg database test` is the command that confirms a
        // correction without reading the catalogue.
        Error::AuthenticationRefused { entry, .. } => Cow::Owned(format!(
            "check the user and the password of the entry (password, password_command, or the \
             password inside dsn), then test it with: tpl cfg database test {}",
            entry_or_placeholder(entry)
        )),
        Error::PropertyNotReadable { kind, property, .. } => match (kind, *property) {
            (CatalogueObjectKind::Database, _) => Cow::Borrowed(
                "check the database the entry names, or grant the connecting user access to it, \
                 then run the command again",
            ),
            (CatalogueObjectKind::View, "definition") => Cow::Borrowed(
                "grant the connecting user SHOW VIEW on the view, then run the command again",
            ),
            _ => Cow::Owned(format!(
                "grant the connecting user the privilege to read the {property} of the {kind}, \
                 then run the command again"
            )),
        },

        // ------------------------------------------------------------ 78 ---
        // FR-PROJ-006 obliges this hint to suggest `tpl init`.
        Error::ProjectNotFound { .. } => Cow::Borrowed(
            "create a project here with: tpl init, or name an existing one with: tpl --tpl-dir \
             <path>/.tpl <command>",
        ),
        // FR-PROJ-008 fixes this hint: correct the flag, and create a project
        // only at the path named — its parent, where the last segment is
        // `.tpl`, built under FR-ERR-041.
        // FR-PROJ-027: the corrected value, and the command path or, where the
        // invocation carried an operand, the words to run it again.
        Error::ProjectDirUnusable {
            path,
            fault: TplDirFault::HoldsTplFolder,
        } => holds_tpl_folder(path),
        Error::ProjectDirUnusable {
            fault: TplDirFault::NotTplFolder,
            ..
        } => Cow::Borrowed(
            "correct --tpl-dir to name a project's .tpl folder, as in --tpl-dir <project>/.tpl",
        ),
        Error::ProjectDirUnusable { path, .. } => {
            let named = path.file_name() == Some(std::ffi::OsStr::new(".tpl"));

            match path.parent() {
                Some(parent) if named && admits_file(parent) => Cow::Owned(format!(
                    "correct --tpl-dir, or create the project with: tpl init {}",
                    parent_or_here(parent)
                )),
                _ if named => Cow::Borrowed(
                    "correct --tpl-dir, or create the project with: tpl init <parent>",
                ),
                _ => Cow::Borrowed("correct --tpl-dir so that it names an existing .tpl folder"),
            }
        }
        // FR-PROJ-010 and FR-PROJ-011 fix these two: the absolute path, built
        // under FR-ERR-041, so that the command succeeds from any directory.
        Error::ConfigurationNotOwned { path, .. } => {
            Cow::Owned(format!("chown \"$(id -un)\" {}", configuration_file(path)))
        }
        // FR-PROJ-028: `--tpl-dir` is the way to name the caller's own project.
        Error::ProjectFolderNotOwned { .. } => Cow::Borrowed(
            "name your own project's .tpl folder with: tpl --tpl-dir <project>/.tpl <command>",
        ),
        // FR-CONF-047: the key set to the path itself, with a placeholder. No
        // `tpl cfg` command runs until the file is valid, so the edit is the
        // file's.
        Error::ConfigurationPathReference {
            key,
            file,
            position,
            ..
        } => {
            // The leaf is one of two literals of FR-CONF-002 wherever the
            // key is well formed; anything else is not reproduced.
            let leaf = key
                .rsplit('.')
                .next()
                .filter(|leaf| matches!(*leaf, "ca_file" | "ca_path"))
                .unwrap_or("the key");
            Cow::Owned(format!(
                "edit {} at line {} and give the path itself, as in {leaf} = \"<path>\"; \
                 {NO_CFG_COMMAND}",
                configuration_file(file),
                position.line
            ))
        }
        Error::ConfigurationUnsafeMode { path, .. } => {
            Cow::Owned(format!("chmod 600 {}", configuration_file(path)))
        }
        // FR-CACHE-042 item 1 and FR-CACHE-044: the command that removes the
        // link alone, since `rm` given a link removes the link and not its
        // target. The path is absolute, built under FR-ERR-041, or the
        // placeholder where that set refuses it.
        Error::CachePathLinked { path, within, .. } => {
            if path.is_absolute() && admits_file(path) {
                Cow::Owned(format!("rm {}", path.display()))
            } else {
                Cow::Owned(format!("rm <project>/.tpl/{}", within.display()))
            }
        }
        // BR-ERR-004: no `tpl cfg` command runs while `.tpl/.cfg` fails step 3
        // of FR-ERR-006, so every hint of a fault in the file names the file,
        // the position where the variant carries one, and the edit.
        Error::ConfigurationMalformed { path, position, .. } => Cow::Owned(format!(
            "edit {} at line {}, column {} so that it is valid TOML; {NO_CFG_COMMAND}",
            configuration_file(path),
            position.line,
            position.column
        )),
        // FR-CONF-034 obliges the nearest-match half over the known keys.
        Error::ConfigurationKeyOutsideSpace {
            key,
            file,
            position,
            nearest,
        } => {
            let admitted = admitted(nearest, admits_key);
            let named = if admits_key(key) {
                Cow::Owned(format!("'{key}'"))
            } else {
                Cow::Borrowed("the key named above")
            };
            let generic = format!(
                "or delete {named} from {} at line {}; tpl help cfg set lists every key the \
                 file may hold",
                configuration_file(file),
                position.line
            );

            if admitted.is_empty() {
                Cow::Owned(generic.trim_start_matches("or ").to_owned())
            } else {
                Cow::Owned(suggest::hint_line(admitted.iter().copied(), &generic).into_owned())
            }
        }
        // FR-CONF-044: the remedy FR-ERR-001 gives a `78` is to fix
        // `.tpl/.cfg`, and the key to fix is this entry's own `ca_path`. The
        // file is valid by the time trust material is assembled, so `tpl cfg`
        // reaches the key.
        Error::TrustDirectoryEmpty { entry, .. } => {
            let key = format!("database.{entry}.ca_path");

            // BR-ERR-005: the invocation did not name ca_path, so the hint
            // does not offer to delete it.
            if admits_key(&key) {
                Cow::Owned(format!(
                    "point it at a directory holding certificate files with: tpl cfg set {key} \
                     <path>"
                ))
            } else {
                Cow::Borrowed(
                    "point the key at a directory holding certificate files with: tpl cfg set \
                     database.<entry>.ca_path <path>",
                )
            }
        }
        // T-04: the file is valid and `tpl cfg` reads it; what is at fault is
        // the variable, and the correction is to the environment.
        Error::ConfigurationValueMalformed {
            key,
            file,
            position,
            expected,
            expanded_from: Some(written),
            ..
        } => {
            let variable = written
                .strip_prefix("${")
                .and_then(|rest| rest.strip_suffix('}'))
                .filter(|name| admits(name));
            match variable {
                Some(name) => {
                    let example = if *expected == ValueType::Port.expected() {
                        "3306"
                    } else {
                        "<value>"
                    };
                    Cow::Owned(format!(
                        "set {name} to {expected} in the environment tpl runs in, e.g.: {}",
                        in_environment(name, example)
                    ))
                }
                None => Cow::Owned(format!(
                    "set the variables {} names so that it expands to {expected}, or edit {} at \
                     line {}",
                    key_or_placeholder(key),
                    configuration_file(file),
                    position.line
                )),
            }
        }
        Error::ConfigurationValueMalformed {
            key,
            file,
            position,
            expected,
            expanded_from: None,
            ..
        } => Cow::Owned(format!(
            "edit {} at line {}: give {} {expected}; {NO_CFG_COMMAND}",
            configuration_file(file),
            position.line,
            key_or_placeholder(key)
        )),
        // Neither line carries a user in its example URL. FR-ERR-022 admits
        // only `[A-Za-z0-9_]` in a value, and FR-SEC-019 builds a runnable hint
        // from literals alone; a worked DSN with a user would put an `@` on the
        // line, which is the character the assertion over every hint looks for
        // to prove that no value reached one ungated.
        Error::DsnMalformed { key, file, fault } => {
            let named = key_or_placeholder(key);

            match fault {
                DsnFault::Scheme => Cow::Owned(format!(
                    "edit {}: begin {named} with mysql:// or mariadb://; {NO_CFG_COMMAND}",
                    configuration_file(file)
                )),
                // T-04: a `${VAR}` is expanded inside a part the URL already
                // delimits, per FR-CONF-018, so it never stands for the URL.
                DsnFault::Form => Cow::Owned(format!(
                    "edit {}: write {named} as scheme://host/database, e.g. \
                     mysql://db.example.com:3306/shop; a ${{VAR}} may stand for one part of the \
                     URL, never for the whole of it; {NO_CFG_COMMAND}",
                    configuration_file(file)
                )),
            }
        }
        // FR-CONF-020 makes `$$` the literal dollar, so a value that meant one
        // is corrected by doubling it rather than by closing a brace.
        //
        // U-06: the file is read and the expansion is attempted only on the
        // way to a connection, so every tpl cfg command still runs, and
        // `NO_CFG_COMMAND` would be false here. A key tpl cfg set accepts a
        // whole reference for is given that command; a dsn or a port, for
        // which it accepts none, is edited in the file.
        Error::UnclosedExpansion { key, file } => {
            let named = key_or_placeholder(key);
            if admits_key(key) && matches!(leaf(key), "host" | "user" | "password" | "database") {
                Cow::Owned(format!(
                    "close the expansion as ${{VAR}}, or write $$ for a literal dollar sign, e.g.: \
                     tpl cfg set {named} '${{VAR}}', where VAR is the variable you meant"
                ))
            } else {
                Cow::Owned(format!(
                    "edit {}: close the expansion in {named} as ${{VAR}}, or write $$ for a \
                     literal dollar sign",
                    configuration_file(file)
                ))
            }
        }
        // FR-CONF-049: the reference with a valid name. The file is valid, so
        // `tpl cfg set` rewrites a field it carries.
        Error::InvalidReferenceName { key, file, .. } => {
            if admits_key(key) && matches!(leaf(key), "host" | "user" | "password" | "database") {
                Cow::Owned(format!(
                    "write the reference with a valid name, e.g.: tpl cfg set {key} '${{<NAME>}}', \
                     where <NAME> starts with a letter or an underscore and holds only letters, \
                     digits and underscores"
                ))
            } else {
                Cow::Owned(format!(
                    "edit {}: write each reference in {} as ${{NAME}}, where NAME starts with a \
                     letter or an underscore and holds only letters, digits and underscores",
                    configuration_file(file),
                    key_or_placeholder(key)
                ))
            }
        }
        // FR-CONF-048: no `tpl cfg` command runs while the file fails step 3.
        Error::ConfigurationEntryName {
            file,
            core,
            position,
            ..
        } => {
            if *core {
                Cow::Owned(format!(
                    "edit {} at line {} and set core.database to the name of an entry, as in \
                     database = \"shop\"; {NO_CFG_COMMAND}",
                    configuration_file(file),
                    position.line
                ))
            } else {
                Cow::Owned(format!(
                    "edit {} at line {} and rename the entry with letters, digits and \
                     underscores only, as in [database.shop]; {NO_CFG_COMMAND}",
                    configuration_file(file),
                    position.line
                ))
            }
        }
        // FR-CONF-035 obliges the hint to show the array form, and states this
        // example itself.
        Error::PasswordCommandNotAnArray {
            file,
            position,
            found,
            element,
            ..
        } => Cow::Owned(format!(
            "edit {} at line {} and write it as {}: password_command = [\"security\", \
             \"find-generic-password\", \"-s\", \"tpl-shop\", \"-w\"]",
            configuration_file(file),
            position.line,
            if element.is_some() {
                "an array of quoted strings"
            } else if *found == crate::project::config::EMPTY_ARRAY {
                "an array whose first string is the program"
            } else {
                "an array"
            }
        )),
        // BR-ERR-005 and the spirit of FR-CFG-048: the edit says which key to
        // keep, and what deleting dsn would lose, never "delete either".
        Error::ConflictingEntryKeys {
            entry,
            file,
            first,
            second,
        } => {
            let (first, second) = (leaf(first), leaf(second));
            let block = if admits(entry) {
                Cow::Owned(format!("under [database.{entry}]"))
            } else {
                Cow::Borrowed("in the entry named above")
            };
            let edit: String = match (first, second) {
                ("dsn", "password_command") | ("password_command", "dsn") => format!(
                    "{block}, keep password_command and remove the password from inside dsn: \
                     delete it and the colon before it from the user part, and keep the rest of \
                     dsn as it is"
                ),
                ("password", "password_command") | ("password_command", "password") => {
                    format!(
                        "{block}, keep password_command and delete password, so that no password \
                         is stored in the file"
                    )
                }
                ("dsn", _) | (_, "dsn") => format!(
                    "{block}, keep dsn and delete every separate connection key the block also \
                     carries (host, port, user, password, database); deleting dsn instead would \
                     also remove the host, port, user, password and database it carries"
                ),
                _ => format!(
                    "{block}, keep the first of the two keys named above and delete the other"
                ),
            };

            Cow::Owned(format!(
                "edit {}: {edit}; {NO_CFG_COMMAND}",
                configuration_file(file)
            ))
        }
        // FR-CONF-011 fixes this hint: the file, the entry's dsn key, and the
        // edit that removes the query. The character is named in words, so
        // that no `?` reaches a hint line, for the reason given above.
        Error::DsnQueryParameter { key, file } => Cow::Owned(format!(
            "edit {}: in {}, delete the question mark and everything after it, and state each \
             option as a key of its own; {NO_CFG_COMMAND}",
            configuration_file(file),
            key_or_placeholder(key)
        )),
        Error::UndefinedVariable { name, .. } => {
            if admits(name) {
                Cow::Owned(format!(
                    "define it in the environment tpl runs in, e.g.: {}",
                    in_environment(name, "<value>")
                ))
            } else {
                Cow::Borrowed(
                    "define the environment variable named above, then run the command again",
                )
            }
        }
        Error::PasswordCommandDeadlineExceeded { bound, .. } => match bound {
            DeadlineBound::Phase => Cow::Borrowed(
                "raise the deadline with: tpl cfg set core.password_timeout <seconds>",
            ),
            DeadlineBound::Overall => Cow::Borrowed(OVERALL_BUDGET),
        },
        Error::PasswordCommandOutputCapExceeded { .. } => {
            Cow::Borrowed("make password_command write the password and nothing else")
        }
        Error::PasswordCommandNotExecutable { entry, .. } if admits(entry) => Cow::Owned(format!(
            "make the first word of database.{entry}.password_command an executable program on \
             PATH or its full path, e.g.: tpl cfg set database.{entry}.password_command \
             '<program> <argument>'"
        )),
        Error::PasswordCommandNotExecutable { .. } => Cow::Borrowed(
            "check that the first element of password_command is the path of an executable \
             program, then run the command again",
        ),
        // FR-CONF-032 records the remedy: the child's own standard error is
        // visible when the command is run directly, and nowhere else.
        // The command is written out where every word of it is admitted by
        // FR-ERR-022; a word outside that set leaves it to the file.
        Error::PasswordCommandFailed { command, .. }
            if !command.is_empty() && command.iter().all(|word| admits(word)) =>
        {
            Cow::Owned(format!(
                "tpl discards the command's standard error, so run it directly to see why it \
                 failed: {}",
                command.join(" ")
            ))
        }
        Error::PasswordCommandFailed { .. } => Cow::Borrowed(
            "tpl discards the command's standard error, so run the command directly to see why \
             it failed",
        ),
        Error::ReadOnlySessionNotEnforced {
            entry,
            fault,
            by_dsn,
        } => {
            let remedy = match fault {
                ReadOnlyFault::NotApplied => "accepts a read-only session",
                ReadOnlyFault::ReadBackDisagreed => "reports the read-only session it accepted",
            };
            Cow::Owned(format!(
                "repoint the entry at a server that {remedy}: {}",
                repoint(entry, *by_dsn)
            ))
        }
        // FR-CONF-040 and FR-CONF-041 each fix this line: the flag of
        // `FR-CFG-027` that writes the key the entry does not carry, on the
        // command that updates the entry. The entry name is a value this
        // corpus does not fix, so it is filled in only where `FR-ERR-022`
        // admits it and the placeholder stands otherwise, per `FR-ERR-023`.
        // FR-ERR-045: an entry defined by dsn is completed inside its dsn.
        Error::EntryKeyMissing {
            entry, key, flag, ..
        } if *flag == "--dsn" => Cow::Owned(format!(
            "set it inside the dsn: {}",
            inside_dsn(
                entry,
                if key.ends_with(".host") {
                    "host"
                } else {
                    "database"
                }
            )
        )),
        Error::EntryKeyMissing {
            entry,
            flag,
            placeholder,
            ..
        } => Cow::Owned(format!(
            "set it with: tpl cfg database update {} {flag} {placeholder}",
            entry_or_placeholder(entry)
        )),
        // S-15: with no entry in the file, listing them would print nothing.
        Error::NoDatabaseEntrySelected {
            has_entries: false, ..
        } => Cow::Borrowed(
            "this project has no database entry; add one with: tpl cfg database add <name> --host \
             <host> --user <user> --schema <database>",
        ),
        Error::NoDatabaseEntrySelected { .. } => Cow::Borrowed(
            "select an entry with -d <entry>, or set a default with: tpl cfg set core.database \
             <entry>; list the entries with: tpl cfg database list",
        ),
        Error::ServerNotMariaDb { entry, by_dsn, .. } => Cow::Owned(format!(
            "repoint the entry at a MariaDB server: {}",
            repoint(entry, *by_dsn)
        )),
        // FR-SRV-030 obliges this hint to carry `tpl cfg database update` with
        // the entry name filled in, and FR-ERR-045 its --dsn for a dsn entry.
        Error::SeriesNotSupported { entry, by_dsn, .. } => Cow::Owned(format!(
            "repoint the entry at a supported server: {}",
            repoint(entry, *by_dsn)
        )),
    }
}

/// The file a malformed `--context` document is replaced by (finding U-09 of
/// the fourth re-audit of rmp `#263`), and the one used where the caller's own
/// file already carries that name.
const NEW_CONTEXT: &str = "context.json";

/// See [`NEW_CONTEXT`].
const OTHER_CONTEXT: &str = "context.new.json";

/// The clause every hint of a fault in `.tpl/.cfg` ends with.
///
/// `BR-ERR-004` names the case: no `tpl cfg` subcommand is excused from
/// validating the file, per `FR-ERR-035`, so a caller told to "run tpl cfg set"
/// meets the same `78` again. The clause says so, so that the edit is not
/// mistaken for one route among several.
const NO_CFG_COMMAND: &str = "no tpl cfg command runs until the file is valid";

/// The hint of every condition the overall budget of `FR-GLOB-011` ended.
///
/// The command is the caller's own, so it is not rewritten here: the flag is
/// global and goes anywhere on the line, per `FR-GLOB-002`.
const OVERALL_BUDGET: &str =
    "raise the overall budget: run the same command again with a larger --timeout <seconds>";

/// The `tpl help` command that shows one node, which at the root is `tpl help`
/// alone.
///
/// `command` is the command path below `tpl`, a spelling this corpus
/// enumerates; the test beside it is the defensive assertion this module
/// applies to every such value.
fn help_of(command: &str) -> Cow<'static, str> {
    if command.is_empty() {
        Cow::Borrowed("tpl help")
    } else if admits_path(command) {
        Cow::Owned(format!("tpl help {command}"))
    } else {
        Cow::Borrowed("tpl help <command>")
    }
}

/// The entry name, where `FR-ERR-022` admits it into a runnable command, and
/// the placeholder otherwise, per `FR-ERR-023`.
fn entry_or_placeholder(entry: &str) -> &str {
    if admits(entry) { entry } else { "<entry>" }
}

/// A configuration key, where every segment is admitted, and a phrase that
/// points at the `error:` line otherwise.
fn key_or_placeholder(key: &str) -> &str {
    if admits_key(key) {
        key
    } else {
        "the key named above"
    }
}

/// The last segment of a dotted key: `dsn` of `database.shop.dsn`.
fn leaf(key: &str) -> &str {
    key.rsplit('.').next().unwrap_or(key)
}

/// The path of `.tpl/.cfg` as a hint writes it: absolute where `FR-ERR-041`
/// admits it, and the placeholder `FR-ERR-041` puts in its position otherwise.
fn configuration_file(path: &Path) -> Cow<'static, str> {
    if path.is_absolute() && admits_file(path) {
        Cow::Owned(path.display().to_string())
    } else {
        Cow::Borrowed("<project>/.tpl/.cfg")
    }
}

/// The hint of a `--tpl-dir` that names the directory holding a `.tpl`
/// folder (`FR-PROJ-027`).
///
/// The corrected value is the path as written followed by `/.tpl`, built
/// under `FR-ERR-041`, or the placeholder of `FR-ERR-043` where that set
/// refuses it. With no operand the command path is written back, and `-d` is
/// carried onto it by [`hint`]; with one, the words say to run the same
/// invocation again, and the operand is not reproduced.
fn holds_tpl_folder(path: &Path) -> Cow<'static, str> {
    let written = path.to_str().map(|text| text.trim_end_matches('/'));
    let (corrected, replacing) = match written {
        Some(text) if admits_template(&format!("{text}/.tpl")) => {
            (format!("{text}/.tpl"), String::new())
        }
        _ => (
            "<tpl-dir>/.tpl".to_owned(),
            "; replace <tpl-dir> with the value you gave --tpl-dir".to_owned(),
        ),
    };

    match restate::shape() {
        Some(shape) if !shape.operand && !shape.path.is_empty() => Cow::Owned(format!(
            "name the .tpl folder itself: tpl --tpl-dir {corrected} {}{replacing}",
            shape.path
        )),
        _ => Cow::Owned(format!(
            "name the .tpl folder itself: run the same command again with --tpl-dir \
             {corrected}{replacing}"
        )),
    }
}

/// The hint of an entry name `FR-CONF-048` refuses: the caller's command with
/// a placeholder for the name, or the form it takes.
fn invalid_entry_name(given: EntryNameGiven) -> Cow<'static, str> {
    const PREFIX: &str = "give the entry a name of 1 to 64 letters, digits or underscores";
    const MEANING: &str = "a name of 1 to 64 letters, digits or underscores";

    match given {
        EntryNameGiven::CoreDatabase => {
            Cow::Borrowed("give the entry name itself: tpl cfg set core.database <entry>")
        }
        EntryNameGiven::Add => {
            let edit = Edit::Value {
                ids: &["name"],
                to: Replacement::Placeholder {
                    text: "<name>",
                    meaning: MEANING,
                },
            };
            match restate::restated(&[edit]) {
                Some(restated) => Cow::Owned(format!(
                    "rename the entry: {}{}",
                    restated.command,
                    restated.replacing()
                )),
                None => Cow::Owned(format!(
                    "{PREFIX}: tpl cfg database add <name> --host <host> --schema <database>"
                )),
            }
        }
        EntryNameGiven::Key(field) => {
            let key = format!("database.<name>.{field}");
            let edit = Edit::Value {
                ids: &["key"],
                to: Replacement::Placeholder {
                    text: &key,
                    meaning: MEANING,
                },
            };
            match restate::restated(&[edit]) {
                Some(restated) => Cow::Owned(format!(
                    "rename the entry: {}{}",
                    restated.command,
                    restated
                        .replacing()
                        .replacen(&format!("replace {key}"), "replace <name>", 1)
                )),
                None => Cow::Owned(format!("{PREFIX}: tpl cfg set {key} <value>")),
            }
        }
    }
}

/// The hint of a malformed reference given on the command line
/// (`FR-CONF-049`): the caller's command with the reference written as
/// `${<NAME>}`. A DSN is shown as a URL with a reference for one part, since
/// a reference never stands for the whole of it.
fn invalid_reference(parameter: &str) -> Cow<'static, str> {
    const RULE: &str =
        "write each reference as ${NAME}, where NAME starts with a letter or an underscore";
    let dsn = parameter == "--dsn" || parameter.ends_with(".dsn");
    let example = if dsn {
        "'mysql://db.example.com/${<NAME>}'"
    } else {
        "'${<NAME>}'"
    };

    if admits_key(parameter) {
        return Cow::Owned(format!("{RULE}, e.g.: tpl cfg set {parameter} {example}"));
    }
    if !admits_flag(parameter) {
        return Cow::Borrowed(RULE);
    }

    let ids = [parameter.trim_start_matches('-')];
    let edit = Edit::Value {
        ids: &ids,
        to: Replacement::Literal(example),
    };
    match restate::restated(&[edit]) {
        Some(restated) => Cow::Owned(format!(
            "{RULE}, e.g.: {}{}",
            restated.command,
            restated.replacing()
        )),
        None => Cow::Owned(format!("{RULE}, e.g.: {parameter} {example}")),
    }
}

/// The hint of an empty host or database given on the command line
/// (`FR-CONF-050`): the flag or the key with a placeholder.
fn empty_value(parameter: &str) -> Cow<'static, str> {
    let (what, placeholder, meaning, ids): (&str, &str, &str, &[&str]) =
        if parameter.ends_with("host") {
            ("host", "<host>", "the host of the server", &["host"])
        } else {
            (
                "database",
                "<database>",
                "the database on the server",
                &["schema"],
            )
        };

    if admits_key(parameter) {
        return Cow::Owned(format!(
            "give the {what}: tpl cfg set {parameter} {placeholder}"
        ));
    }

    let edit = Edit::Value {
        ids,
        to: Replacement::Placeholder {
            text: placeholder,
            meaning,
        },
    };
    match restate::restated(&[edit]) {
        Some(restated) => Cow::Owned(format!(
            "give the {what}: {}{}",
            restated.command,
            restated.replacing()
        )),
        None if admits_flag(parameter) => {
            Cow::Owned(format!("give the {what}: {parameter} {placeholder}"))
        }
        None => Cow::Owned(format!("give the {what} in place of the empty value")),
    }
}

/// The hint of a `${VAR}` given for `ca_file` or `ca_path`, or for their
/// flags (`FR-CONF-047`): the caller's command with `<path>` in its place.
fn path_itself(parameter: &str) -> Cow<'static, str> {
    let ids: &[&str] = match parameter {
        "--ca-file" => &["ca_file"],
        "--ca-path" => &["ca_path"],
        _ => &["value"],
    };
    let edit = Edit::Value {
        ids,
        to: Replacement::Placeholder {
            text: "<path>",
            meaning: "the path of the trust material itself",
        },
    };

    match restate::restated(&[edit]) {
        Some(restated) => Cow::Owned(format!(
            "give the path itself: {}{}",
            restated.command,
            restated.replacing()
        )),
        None if admits_key(parameter) => Cow::Owned(format!(
            "give the path itself: tpl cfg set {parameter} <path>"
        )),
        None if admits_flag(parameter) => {
            Cow::Owned(format!("give the path itself: {parameter} <path>"))
        }
        None => Cow::Borrowed("give the path itself, in place of the value named above"),
    }
}

/// A parent directory as `tpl init` takes it: `.` where the path had no parent
/// segment of its own.
fn parent_or_here(parent: &Path) -> Cow<'_, str> {
    if parent.as_os_str().is_empty() {
        Cow::Borrowed(".")
    } else {
        parent.to_string_lossy()
    }
}

/// An invocation of `FR-ERR-022` with the `<template>` it writes replaced by
/// the template the render was given, where `FR-ERR-041` admits it.
/// The edit that writes `corrected` in place of the routine the caller named:
/// the value of `--routine`, or the operand of `tpl schema routine`.
fn routine_as(corrected: &str) -> Edit<'_> {
    Edit::Value {
        ids: &["routine", "name"],
        to: Replacement::Literal(corrected),
    }
}

fn with_template(invocation: &'static str, template: Option<&str>) -> Cow<'static, str> {
    match template {
        Some(template) if admits_template(template) => {
            Cow::Owned(invocation.replace("<template>", template))
        }
        _ => Cow::Borrowed(invocation),
    }
}

/// The hint for a lookup that found nothing: the command that lists the names
/// it could have found. A render reads the selected database entry, and so
/// does the listing; a render from a `--context` document reads that document,
/// which no `tpl` command lists, so the line names jq over it instead.
fn listing_of(unresolved: &Unresolved) -> Cow<'static, str> {
    if let Some(document) = &unresolved.document {
        return listing_in_document(unresolved, document);
    }

    match (unresolved.kind, &unresolved.table) {
        (LookupKind::Column, Some(table)) if admits(table) => Cow::Owned(format!(
            "list the columns of table '{table}' with: tpl schema table {table}"
        )),
        (LookupKind::Column, _) => {
            Cow::Borrowed("list the columns of a table with: tpl schema table <table>")
        }
        (LookupKind::Table, _) => Cow::Borrowed("list the tables with: tpl schema tables"),
        (LookupKind::View, _) => Cow::Borrowed("list the views with: tpl schema views"),
        (LookupKind::Routine, _) => Cow::Borrowed("list the routines with: tpl schema routines"),
    }
}

/// The hint for an `{% include %}` that named no template (`FR-TMPL-009`).
///
/// The rule that an include writes the extension is stated in the help of
/// `tpl template list` alone, so the line states it again where the include
/// did not write it, and drops it where the include did (finding Z-03 of the
/// ninth re-audit of rmp `#263`). Either way the line suggests the nearest
/// templates, with the extension, that `FR-ERR-041` admits.
fn include_of(name: &str, nearest: &[String]) -> Cow<'static, str> {
    const RULE: &str = "an {% include %} names the template file with its extension, as in {% \
                        include \"example.jinja\" %}; list the templates with: tpl template list";
    const LIST: &str = "list the templates with: tpl template list";

    let generic = if name.ends_with(".jinja") { LIST } else { RULE };
    let admitted = admitted(nearest, admits_template);
    Cow::Owned(suggest::hint_line(admitted.into_iter(), generic).into_owned())
}

/// [`listing_of`] for a render from the `--context` document `document`.
///
/// jq is an external tool, and the line says so. The path and a table name are
/// written into the command only where `FR-ERR-022` admits them; standard input
/// cannot be read twice, so `-` leaves the member of the document to name.
fn listing_in_document(unresolved: &Unresolved, document: &Path) -> Cow<'static, str> {
    let member = match unresolved.kind {
        LookupKind::Table | LookupKind::Column => "tables",
        LookupKind::View => "views",
        LookupKind::Routine => "routines",
    };
    let file = (document != Path::new("-") && admits_file(document)).then(|| document.display());

    match (unresolved.kind, &unresolved.table, file) {
        (LookupKind::Column, Some(table), Some(file)) if admits(table) => Cow::Owned(format!(
            "list the columns of table '{table}' in the --context document with jq, if it is \
             installed: jq -r '.data.database.tables[] | select(.name==\"{table}\") | \
             .columns[].name' {file}"
        )),
        (LookupKind::Column, _, _) => Cow::Borrowed(
            "name a column the --context document lists under data.database.tables[].columns",
        ),
        (_, _, Some(file)) => Cow::Owned(format!(
            "list the {member} of the --context document with jq, if it is installed: jq -r \
             '.data.database.{member}[].name' {file}"
        )),
        (_, _, None) => Cow::Owned(format!(
            "name one the --context document lists under data.database.{member}"
        )),
    }
}

/// The most attributes a hint lists by name.
const MAX_ATTRIBUTES: usize = 24;

/// The hint for an undefined expression rooted at a bound context variable
/// (finding Y-01 of the eighth re-audit of rmp `#263`, and Z-01 of the ninth),
/// or at a name near one (finding Z-04).
///
/// The variable is bound, so the correction is in the template: the hint says
/// what the step that found nothing reads, what the value before it holds, and
/// prints the template's source. A key of `vars` is the exception, because
/// `--set` may be the side that is wrong, and [`set_key`] answers it. An
/// attribute name reaches the line only where the set of `FR-ERR-022` admits
/// it.
fn missing_step(missing: &Missing, template: &str) -> Cow<'static, str> {
    let show = if admits_template(template) {
        format!("print the template's source with: tpl template show {template}")
    } else {
        "print the template's source with: tpl template show <template>".to_owned()
    };

    let line = match missing {
        Missing::Attribute {
            owner,
            name,
            attributes,
            nearest,
            ..
        } if owner == "vars" => return set_key(name, attributes, nearest),
        Missing::Attribute {
            owner,
            name,
            kind,
            attributes,
            nearest,
        } => {
            let named: Vec<&str> = attributes
                .iter()
                .map(String::as_str)
                .filter(|attribute| admits(attribute))
                .collect();
            let generic = if named.is_empty() {
                format!(
                    "'{owner}' is {kind} with no attribute '{name}'; correct the template, then {show}"
                )
            } else {
                let listed = named
                    .iter()
                    .take(MAX_ATTRIBUTES)
                    .copied()
                    .collect::<Vec<_>>()
                    .join(", ");
                let more = named.len().saturating_sub(MAX_ATTRIBUTES);
                let tail = if more > 0 {
                    format!(" and {more} more")
                } else {
                    String::new()
                };
                format!(
                    "'{owner}' has no attribute '{name}'; its attributes are {listed}{tail}; \
                     correct the template, then {show}"
                )
            };
            suggest::hint_line(admitted(nearest, admits).into_iter(), &generic).into_owned()
        }
        Missing::Index {
            owner,
            index,
            length,
        } => match length {
            0 => format!(
                "'{owner}' is an empty list, so [{index}] reads nothing; correct the template, \
                 then {show}"
            ),
            1 => format!(
                "'{owner}' has 1 item, at index 0, so [{index}] reads nothing; correct the \
                 template, then {show}"
            ),
            _ => format!(
                "'{owner}' has {length} items, at indexes 0 to {}, so [{index}] reads nothing; \
                 correct the template, then {show}",
                length - 1
            ),
        },
        Missing::NotList { owner, index, kind } => format!(
            "'{owner}' is {kind}, which has no item [{index}]; correct the template, then {show}"
        ),
        Missing::Elsewhere { root } => format!(
            "'{root}' is defined, {}, so the fault is in the expression; correct the template, \
             then {show}",
            super::cause::bound_because(root)
        ),
        // Z-04: the nearest are context variables, literals of FR-ERR-022,
        // and the name is written only where its set admits it.
        // AA-03: a candidate the render did not bind needs its flag as well
        // as the corrected name, and the line says so; the flag is the
        // variable's own name, a literal.
        // AC-04: where the render bound another of the three objects, the
        // flag is replaced rather than added; both names are literals.
        Missing::Variable {
            nearest,
            unbound,
            replacing: Some(bound),
            ..
        } if !unbound.is_empty() => {
            let generic = match (nearest.len(), unbound.as_slice()) {
                (1, [variable]) => format!(
                    "this render names a {bound} with --{bound}, so '{variable}' is not bound: \
                     correct the template to read '{bound}', or correct it and replace --{bound} \
                     with --{variable} <name> in the tpl render command"
                ),
                (_, [variable]) => format!(
                    "this render names a {bound} with --{bound}, so '{variable}' is not bound; \
                     correct the template, and if you mean '{variable}', replace --{bound} with \
                     --{variable} <name> in the tpl render command"
                ),
                _ => format!(
                    "this render names a {bound} with --{bound}, so {} are not bound; correct \
                     the template, and if you mean one of them, replace --{bound} with its flag \
                     in the tpl render command",
                    unbound
                        .iter()
                        .map(|variable| format!("'{variable}'"))
                        .collect::<Vec<_>>()
                        .join(" and ")
                ),
            };
            suggest::hint_line(nearest.iter().copied(), &generic).into_owned()
        }
        Missing::Variable {
            nearest, unbound, ..
        } if !unbound.is_empty() => {
            let flags = unbound
                .iter()
                .map(|variable| {
                    format!(
                        "'{variable}' exists only when the render names one with --{variable} \
                         <name>"
                    )
                })
                .collect::<Vec<_>>()
                .join("; ");
            let generic = match (nearest.len(), unbound.as_slice()) {
                (1, [variable]) => format!(
                    "'{variable}' exists only when the render names one: correct the template and \
                     add --{variable} <name> to the tpl render command"
                ),
                (_, [variable]) => format!(
                    "{flags}; correct the template, and if you mean '{variable}', add \
                     --{variable} <name> to the tpl render command"
                ),
                _ => format!(
                    "{flags}; correct the template, and add the flag of the one you mean to the \
                     tpl render command"
                ),
            };
            suggest::hint_line(nearest.iter().copied(), &generic).into_owned()
        }
        // AB-04: the three object flags exclude one another, so the fix is
        // to read the bound one or to give the other flag in its place, never
        // to add a second. Both names are literals of the tree.
        Missing::OtherObject { root, bound } => format!(
            "this render names a {bound} with --{bound}, so '{root}' is not bound: read \
             '{bound}' in the template, or replace --{bound} with --{root} <name> in the tpl \
             render command"
        ),
        Missing::Variable { name, nearest, .. } => {
            let generic = if admits(name) {
                format!(
                    "'{name}' is no variable tpl binds in this render; correct the template, \
                     then {show}"
                )
            } else {
                format!(
                    "the name is no variable tpl binds in this render; correct the template, \
                     then {show}"
                )
            };
            suggest::hint_line(nearest.iter().copied(), &generic).into_owned()
        }
        // The template binds the name, so no flag and no --set is the fix,
        // and the render's value of the same name is not described. The name
        // is a context variable's, a literal of FR-ERR-022.
        Missing::Bound {
            name,
            loop_variable,
            step,
            read,
        } => {
            let kind = if *loop_variable {
                "a loop variable of the template"
            } else {
                "a variable the template binds"
            };
            let lead = format!("'{name}' is {kind}, not the render's '{name}': ");
            match (step.as_deref(), read) {
                // A step of `vars` would be read as a `--set` key.
                (Some(step), _) if name != "vars" => {
                    let inner = missing_step(step, template);
                    match inner
                        .strip_prefix("did you mean ")
                        .and_then(|rest| rest.split_once("? "))
                    {
                        Some((names, generic)) => {
                            format!("did you mean {names}? {lead}{generic}")
                        }
                        None => format!("{lead}{inner}"),
                    }
                }
                (_, Some(read)) if admits(read) => format!(
                    "{lead}reading '{read}' of it found nothing; correct the template, then {show}"
                ),
                _ => format!("{lead}correct the expression in the template, then {show}"),
            }
        }
    };
    Cow::Owned(line)
}

/// The hint for a key of `vars` that no `--set` gave (finding Z-02 of the
/// ninth re-audit of rmp `#263`).
///
/// A key `--set` gave that is near the one the template reads is a typo on
/// one side or the other, so the line names it and never asks for a new
/// `--set`; only where no key is near does it ask for one. The keys are
/// `--set` keys, which `FR-RND-012` confines to the set of `FR-ERR-022`; each
/// is tested again, and one outside the set is not written.
fn set_key(name: &str, given: &[String], nearest: &[String]) -> Cow<'static, str> {
    let keys: Vec<&str> = given
        .iter()
        .map(String::as_str)
        .filter(|key| admits(key))
        .collect();
    let listed = {
        let shown = keys
            .iter()
            .take(MAX_ATTRIBUTES)
            .copied()
            .collect::<Vec<_>>()
            .join(", ");
        match keys.len().saturating_sub(MAX_ATTRIBUTES) {
            0 => shown,
            more => format!("{shown} and {more} more"),
        }
    };
    let near = admitted(nearest, admits);

    if !near.is_empty() {
        let generic = format!(
            "--set gave {listed}; correct the key in the template or in --set so the two match"
        );
        return Cow::Owned(suggest::hint_line(near.into_iter(), &generic).into_owned());
    }

    let add = if admits(name) {
        format!("add --set {name}=<value> to the tpl render command")
    } else {
        "add --set <key>=<value> to the tpl render command".to_owned()
    };
    Cow::Owned(match (keys.is_empty(), admits(name)) {
        (true, true) => format!("'vars.{name}' is set with --set: {add}"),
        (true, false) => format!("a key of 'vars' is set with --set: {add}"),
        (false, true) => format!("--set gave {listed}, and none is near '{name}': {add}"),
        (false, false) => {
            format!("--set gave {listed}, and none is near the key the template reads: {add}")
        }
    })
}

/// The key of `vars` an expression reads an attribute of, as in `vars.a.b`,
/// or [`None`] where the expression reads no attribute past a key of `vars`.
///
/// The inner [`Option`] is [`None`] where the set of `FR-ERR-022` refuses the
/// key, which is then not written. An index past the key, as in `vars.a[0]`,
/// is not matched: a string has items, and a `--set` value may supply one.
fn past_a_vars_key(expression: &str) -> Option<Option<&str>> {
    let rest = expression.trim().strip_prefix("vars.")?;
    let (key, after) = rest.split_once('.')?;
    let key = key.trim();
    (!key.contains(['[', ']', '(', ' ']) && !after.trim().is_empty())
        .then(|| admits(key).then_some(key))
}

/// The hint for an attribute read under a key of `vars` (finding AB-03 of the
/// eleventh re-audit of rmp `#263`).
///
/// `FR-RND-013` gives `--set` top-level keys only and `FR-RND-015` makes each
/// value a string, which has no attributes, so the line never asks for a
/// `--set`: the correction is in the template.
fn beyond_set(key: Option<&str>, template: &str) -> Cow<'static, str> {
    let show = if admits_template(template) {
        format!("print the template's source with: tpl template show {template}")
    } else {
        "print the template's source with: tpl template show <template>".to_owned()
    };
    let read = key.map_or_else(
        || "the template reads an attribute of a key of 'vars'".to_owned(),
        |key| format!("the template reads an attribute of 'vars.{key}'"),
    );
    Cow::Owned(format!(
        "{read}, and --set gives only top-level keys of vars, each a string with no attributes, \
         so no --set can supply it; correct the template, then {show}"
    ))
}

/// The hint for an undefined expression whose first segment is a variable a
/// flag binds, or [`None`] where it is not one.
///
/// `table`, `view` and `routine` exist only when the object flag of the same
/// name is given, per `FR-RND-023`, and a key of `vars` exists only when a
/// `--set` defines it, per `FR-CTX-026`. The key is a `--set` key, which
/// `FR-RND-012` confines to the set of `FR-ERR-022`; one outside it leaves the
/// placeholder. A bare name that is none of the variables is answered with the
/// list of them.
fn defining_flag(expression: &str) -> Option<Cow<'static, str>> {
    let mut segments = expression.split(['.', '[']);
    let root = segments.next()?.trim();

    match root {
        "table" | "view" | "routine" => Some(Cow::Owned(format!(
            "'{root}' exists only when the render names one: add --{root} <name> to the tpl \
             render command"
        ))),
        "vars" => {
            let key = segments.next().map(str::trim).filter(|key| admits(key));

            Some(Cow::Owned(match key {
                Some(key) => format!(
                    "'vars.{key}' is set with --set: add --set {key}=<value> to the tpl render \
                     command"
                ),
                None => "a key of 'vars' is set with --set: add --set <key>=<value> to the tpl \
                         render command"
                    .to_owned(),
            }))
        }
        // T-09: a bare name the render never defines is most often a
        // misspelling of one it does, so the hint names them all. A dotted
        // expression may begin with a loop variable or a `set`, which the
        // render does define, so it is left to the generic hint.
        other if other == expression.trim() && !matches!(other, "database" | "tpl" | "now") => {
            const DEFINED: &str = "a template sees database, vars, tpl and now, and table, view \
                                   or routine when --table, --view or --routine names one; list \
                                   them with: tpl help render";
            Some(Cow::Owned(if admits(other) {
                format!("'{other}' is not a variable of this render: {DEFINED}")
            } else {
                format!("the name is not a variable of this render: {DEFINED}")
            }))
        }
        _ => None,
    }
}

/// Whether a template name may be written into a hint (`FR-ERR-041`).
///
/// `[A-Za-z0-9_./-]`, measured over the whole value, at most [`MAX_PATH`]
/// characters, and not beginning with `-`. A nested template name carries a
/// `/`, which the set of `FR-ERR-022` refused, so none could be suggested.
pub(super) fn admits_template(name: &str) -> bool {
    // Every admitted byte is ASCII, so the byte length is the character count.
    !name.is_empty()
        && name.len() <= MAX_PATH
        && !name.starts_with('-')
        && name
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'_' | b'.' | b'/' | b'-'))
}

/// Whether a filesystem path of the project may be written into a hint
/// (`FR-ERR-041`): the same set as a template name, over the whole path.
pub(super) fn admits_file(path: &Path) -> bool {
    path.to_str().is_some_and(admits_template)
}

/// The candidates of a nearest-match hint that `admits` lets through.
///
/// `FR-ERR-023` drops a candidate outside the character set in **every** form,
/// and [`super::suggest`] applies that where the selection is made. This is the
/// same test applied again where the line is composed, and it is the defensive
/// assertion this module applies to every enumerated population: a command, an
/// alias and a flag are spellings this corpus fixes, so a candidate that fails
/// here came from somewhere other than the command tree and no line is built
/// from it.
fn admitted(nearest: &[String], admits: fn(&str) -> bool) -> Vec<&str> {
    nearest
        .iter()
        .map(String::as_str)
        .filter(|candidate| admits(candidate))
        .collect()
}

/// The `hint` line for a variant whose nearest matches are drawn from a
/// population `FR-ERR-022` governs by a character set.
///
/// Three arms above compose the same two steps — admit the candidates the set
/// allows, then compose the line `FR-ERR-008` fixes around them — over three
/// populations that differ only in the generic hint that follows. Writing the
/// pair once is what keeps `FR-ERR-023` applied identically to all three: a
/// fourth arm of this shape reads the same rule rather than restating it, and
/// a correction to either step cannot land on two of the three.
///
/// The composed line is byte-identical to what the three composed separately.
fn suggested(
    nearest: &[String],
    admits: fn(&str) -> bool,
    generic: &'static str,
) -> Cow<'static, str> {
    suggest::hint_line(admitted(nearest, admits).iter().copied(), generic)
}

/// The `tpl help` command that lists the children of one node.
///
/// `node` is the command path below `tpl`, empty at the root, and it is a
/// spelling this corpus enumerates — a literal of `FR-ERR-022`, like every
/// other command path. The test beside it is the defensive assertion this
/// module applies to every such value, and the root's empty path falls to the
/// bare `tpl help` rather than to a placeholder, because that command lists
/// exactly the children the root has.
/// Whether `token`, refused as a flag of `command`, is `--format` given to
/// `tpl schema dump`, which always writes JSON.
pub(super) fn is_format_on_dump(command: &str, token: &str) -> bool {
    command == "schema dump" && (token == "--format" || token.starts_with("--format="))
}

/// The canonical command path, without the program name, that the words of
/// `token` form under the node `node` names, or [`None`] where `token` holds
/// no whitespace or its words do not all name commands.
///
/// It reads the command tree, so every segment it returns is a literal of
/// `FR-ERR-022`.
pub(super) fn split_command(node: &str, token: &str) -> Option<String> {
    if !token.contains(char::is_whitespace) {
        return None;
    }

    let tree = crate::cli::tree();
    let mut reached = &tree;
    let mut canonical: Vec<String> = Vec::new();

    for word in node.split_whitespace().chain(token.split_whitespace()) {
        reached = reached.find_subcommand(word)?;
        canonical.push(reached.get_name().to_owned());
    }

    (!canonical.is_empty()).then(|| canonical.join(" "))
}

/// The invocation, without the program name, that the words of `token` form
/// under the node `node` names: a command path, optionally followed by the
/// arguments of the command it reaches. [`None`] where `token` holds no
/// whitespace, its first word names no child of `node`, the command reached
/// takes no argument or has subcommands, or an argument word is outside the
/// set of `FR-ERR-041`.
///
/// The command path is read from the tree, so every segment is a literal of
/// `FR-ERR-022`; each argument word is tested before it is written.
pub(super) fn split_invocation(node: &str, token: &str) -> Option<String> {
    if let Some(path) = split_command(node, token) {
        return Some(path);
    }
    if !token.contains(char::is_whitespace) {
        return None;
    }

    let tree = crate::cli::tree();
    let mut reached = &tree;
    let mut canonical: Vec<&str> = Vec::new();
    for word in node.split_whitespace() {
        reached = reached.find_subcommand(word)?;
        canonical.push(reached.get_name());
    }

    let mut words = token.split_whitespace();
    reached = reached.find_subcommand(words.next()?)?;
    canonical.push(reached.get_name());

    let mut arguments: Vec<&str> = Vec::new();
    for word in words {
        match reached.find_subcommand(word) {
            Some(child) if arguments.is_empty() => {
                reached = child;
                canonical.push(reached.get_name());
            }
            _ => arguments.push(word),
        }
    }

    let takes_arguments = !reached.has_subcommands() && reached.get_positionals().next().is_some();
    (takes_arguments && arguments.iter().all(|word| admits_template(word)))
        .then(|| format!("{} {}", canonical.join(" "), arguments.join(" ")))
}

fn children_of(node: &str) -> Cow<'static, str> {
    if node.is_empty() {
        Cow::Borrowed("list the commands with: tpl help")
    } else if admits_path(node) {
        Cow::Owned(format!("list what it takes with: tpl help {node}"))
    } else {
        Cow::Borrowed("list what a command takes with: tpl help <command>")
    }
}

/// The `tpl cfg database update` command that repoints an entry.
///
/// The entry name is filled in only where `FR-ERR-022` admits it into a
/// runnable command; otherwise the placeholder stands, per `FR-ERR-023`.
fn update_entry(entry: &str) -> Cow<'static, str> {
    if admits(entry) {
        Cow::Owned(format!("tpl cfg database update {entry} --host <host>"))
    } else {
        Cow::Borrowed("tpl cfg database update <entry> --host <host>")
    }
}

/// The `tpl cfg database update` that repoints an entry in the form it is
/// defined by: `--host` for the discrete fields, `--dsn` for a dsn
/// (`FR-ERR-045`).
fn repoint(entry: &str, by_dsn: bool) -> Cow<'static, str> {
    if by_dsn {
        Cow::Owned(inside_dsn(entry, "host"))
    } else {
        update_entry(entry)
    }
}

/// The command `FR-ERR-045` fixes for a field of an entry defined by `dsn`,
/// with the words that say what `<url>` stands for.
///
/// `field` names the field that changes, as the words say it. The stored dsn
/// is never reproduced, per `BR-ERR-003`: `<url>` is a placeholder.
fn inside_dsn(entry: &str, field: &str) -> String {
    format!(
        "tpl cfg database update {} --dsn <url>, where <url> is the whole connection URL with \
         the new {field}",
        entry_or_placeholder(entry)
    )
}

/// The caller's own command with `name` defined for it alone, then the
/// `export` that defines it for the rest of one shell (finding X-02 of the
/// seventh re-audit of rmp `#263`).
///
/// An agent's shell often keeps no variable from one call to the next, so an
/// `export` alone, run in one call, leaves the next call without it. `name` is
/// admitted by the caller; `value` is a literal or a placeholder.
fn in_environment(name: &str, value: &str) -> String {
    match restate::restated(&[]) {
        Some(again) => format!(
            "{name}={value} {}, or export {name}={value} in the same shell before running tpl{}",
            again.command,
            again.replacing()
        ),
        None => format!(
            "{name}={value} tpl <command>, or export {name}={value} in the same shell before \
             running tpl"
        ),
    }
}

/// The `tpl cfg unset` that removes one key of one entry (`FR-CFG-048`).
///
/// The key arrives fully qualified, so every segment but the entry name is a
/// spelling this corpus fixes: [`admits_key`] is the governing test for that
/// one segment and the defensive assertion for the others.
fn unset_key(key: &str) -> Cow<'static, str> {
    if admits_key(key) {
        Cow::Owned(format!("tpl cfg unset {key}"))
    } else {
        Cow::Borrowed("tpl cfg unset <key>")
    }
}

/// The update that writes, with their own flags, the fields a refused dsn
/// would have changed (row two of the `FR-CFG-048` table).
///
/// Each value is a placeholder, never the caller's own: `BR-ERR-003` bars a
/// DSN, and every part of it, from every message. The password has no flag of
/// `FR-CFG-027`, so where the dsn carries one the line says how to set it.
fn discrete_entry(named: &str, changed: &[&'static str]) -> String {
    let flags: Vec<&str> = changed
        .iter()
        .filter_map(|field| match *field {
            "host" => Some("--host <host>"),
            "port" => Some("--port <port>"),
            "user" => Some("--user <user>"),
            "database" => Some("--schema <database>"),
            _ => None,
        })
        .collect();
    let mut line = format!(
        "write each field with its own flag: tpl cfg database update {named} {}",
        flags.join(" ")
    );
    if changed.contains(&"password") {
        line.push_str(&format!(
            "; the password has no flag, so set it with: tpl cfg set database.{named}.password \
             <password>"
        ));
    }
    line
}

/// The same command written with the password in one place only
/// (`FR-CFG-048`).
///
/// `command` is the command path below `tpl`, which is a spelling this corpus
/// enumerates and is tested as the defensive assertion this module applies to
/// every such value. The DSN is a placeholder and never the caller's own:
/// `BR-ERR-003` bars the value from every message, and a DSN carrying a
/// password is precisely what this refusal is about.
fn restate_entry(command: &str, entry: &str) -> Cow<'static, str> {
    const FORM: &str = "--dsn <url> --password-command <command>";

    // T-01: the caller's own command, whole, with the DSN as a placeholder.
    let url = Edit::Value {
        ids: &["dsn"],
        to: Replacement::Placeholder {
            text: "<url>",
            meaning: "the DSN you gave, without its password",
        },
    };
    if let Some(restated) = restate::restated(&[url]) {
        return Cow::Owned(format!("{}{}", restated.command, restated.replacing()));
    }

    if !admits_path(command) {
        return Cow::Owned(format!("tpl cfg database add <entry> {FORM}"));
    }

    let named = if admits(entry) { entry } else { "<entry>" };

    Cow::Owned(format!("tpl {command} {named} {FORM}"))
}

/// Whether a name this corpus does not fix may be interpolated into a runnable
/// command.
///
/// `[A-Za-z0-9_]{1,64}` is the set `FR-ERR-022` applies to a value chosen
/// outside this specification — a table, a view, a routine, a template, a
/// database entry, the `<name>` segment of a `database.<name>` key, an
/// environment variable — and to no other kind of value: a spelling this
/// specification enumerates is a literal, and the twentieth edition says so
/// because no key of `FR-CONF-002` and five flags of this corpus would
/// otherwise be admissible anywhere.
///
/// A catalogue name is free text in MariaDB and can carry a semicolon, a
/// quotation mark or a newline, so formatting one straight into a suggested
/// command would be command injection with the caller as the interpreter.
pub(super) fn admits(name: &str) -> bool {
    // Every admitted byte is ASCII, so the byte length is the character count.
    !name.is_empty()
        && name.len() <= MAX_NAME
        && name
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || byte == b'_')
}

/// Whether every segment of a space-separated command path is admitted.
///
/// A command path is a spelling this specification enumerates, so `FR-ERR-022`
/// makes it a literal in its entirety and does not demand this test; the space
/// between its segments is a literal too. The test is kept as a **defensive
/// assertion**: every command and alias of the tree passes it, so a path that
/// fails came from somewhere other than the command tree and no runnable
/// command is built from it.
pub(super) fn admits_path(path: &str) -> bool {
    !path.is_empty() && path.split(' ').all(admits)
}

/// Whether every segment of a configuration key is admitted.
///
/// The `.` between key segments is a literal, per `FR-ERR-022`, so a key is
/// tested segment by segment rather than refused for carrying a dot — which is
/// what dropped every key of `FR-CONF-002` before the twentieth edition, none
/// of the eighteen forms matching the character set as a whole.
///
/// It is the second of the four spelling tests and lives beside the other
/// three for the reason [`admits_flag`] states.
pub(super) fn admits_key(key: &str) -> bool {
    !key.is_empty() && key.split('.').all(admits)
}

/// Whether a flag, or a flag **value** `FR-CLI-018` writes back, is admitted.
///
/// The `-` or `--` that introduces the flag is a literal, per `FR-ERR-022`, and
/// so is the `-` inside the five flags of this corpus that carry one —
/// `--tpl-dir`, `--no-cache`, `--ca-file`, `--ca-path` and `--password-command`
/// — because a flag is a spelling this specification enumerates.
///
/// **The whole value is bounded at [`MAX_NAME`], and each `-`-separated segment
/// is bounded by the alphabet.** `FR-ERR-040` states the set this test enforces
/// over the one population it governs — a flag value the caller supplied in a
/// separate token — as `[A-Za-z0-9_-]{1,64}` *measured over the whole value,
/// including every leading `-`*, and the per-segment form alone bounds no value
/// at all: `tpl -d -a-a-a-a-a-a-a-a version` reaches the `hint` with every
/// segment one character long, and a value of any length composed the same way
/// reaches it too. What was unbounded was how much of the caller's own token
/// could be written back to the caller's own terminal.
///
/// `FR-CLI-018`'s obligation to show the corrected form stays reachable for
/// every admitted value, because the value the set refuses leaves a placeholder
/// in its position rather than removing the line.
///
/// This is the third of the three spelling tests, and it lives beside the other
/// two because one rule stated in two places is a rule that drifts: [`hint`]
/// applies it to the pair of a mutually exclusive refusal and
/// [`super::suggest`] applies it to a nearest-match candidate, and both are the
/// same defensive assertion over the same enumerated population.
pub(super) fn admits_flag(flag: &str) -> bool {
    // Every admitted byte is ASCII, so the byte length is the character count.
    if flag.is_empty() || flag.len() > MAX_NAME {
        return false;
    }

    let name = flag
        .strip_prefix("--")
        .or_else(|| flag.strip_prefix('-'))
        .unwrap_or(flag);

    !name.is_empty() && name.split('-').all(admits)
}

/// The subcommand that lists the population a catalogue object was sought in,
/// or [`None`] where the kind has no population this system reads.
///
/// Three of the four kinds have one, and they are the three `FR-SCH-010`
/// reaches with a `66`. The fourth does not: `FR-PRIV-021` rejects `66` for a
/// database precisely because "the population of databases is one this system
/// never reads", so there is no listing subcommand to name and the caller is
/// sent to `.tpl/.cfg`, where the database a read covers is fixed by
/// `FR-CONF-041`.
///
/// It returns an [`Option`] rather than naming a command for the fourth kind,
/// because `FR-ERR-012` forbids vague advice and a hint naming a subcommand
/// that does not exist is worse than vague.
const fn listing(kind: CatalogueObjectKind) -> Option<&'static str> {
    match kind {
        CatalogueObjectKind::Table => Some("tables"),
        CatalogueObjectKind::View => Some("views"),
        CatalogueObjectKind::Routine => Some("routines"),
        CatalogueObjectKind::Database => None,
    }
}

/// The `[core]` key that resolves the deadline of a network phase.
///
/// `FR-CONF-005` maps every blocking phase to a key of `FR-CONF-002`, and
/// `FR-ERR-012` obliges the hint to name that key rather than tell the caller
/// to check their configuration.
const fn deadline_key(phase: NetworkPhase) -> &'static str {
    match phase {
        NetworkPhase::DnsResolution | NetworkPhase::TcpConnect | NetworkPhase::TlsHandshake => {
            "core.connect_timeout"
        }
        NetworkPhase::CatalogueQuery => "core.query_timeout",
    }
}

#[cfg(test)]
mod tests {
    use super::{MAX_NAME, admits, admits_flag, admits_path, update_entry};

    #[test]
    fn fr_err_022_the_admitted_set_is_the_one_the_requirement_states() {
        assert!(admits("orders"));
        assert!(admits("order_lines_2026"));
        assert!(admits("A9"));

        assert!(!admits(""), "an empty name is outside {{1,64}}");
        assert!(!admits("order lines"), "a space is outside [A-Za-z0-9_]");
        assert!(
            !admits("orders;drop"),
            "a semicolon is outside [A-Za-z0-9_]"
        );
        assert!(
            !admits("orders\nrm -rf /"),
            "a newline is outside [A-Za-z0-9_]"
        );
        // A dot is outside the set. `core.database` is nonetheless suggestible,
        // because a key is a literal tested segment by segment; that is
        // `suggest`'s business and not this predicate's.
        assert!(!admits("core.database"), "a dot is outside [A-Za-z0-9_]");
        assert!(!admits("encomendas_pag\u{e1}s"), "the set is ASCII");
    }

    #[test]
    fn fr_err_022_a_name_of_more_than_sixty_four_characters_is_refused() {
        assert!(admits(&"a".repeat(MAX_NAME)));
        assert!(!admits(&"a".repeat(MAX_NAME + 1)));
    }

    #[test]
    fn fr_err_040_the_whole_value_is_bounded_and_not_only_its_segments() {
        // FR-ERR-040 measures its set over the **whole value**, every leading
        // `-` included. The per-segment form derived from FR-ERR-022 bounds no
        // value at all: `tpl -d -a-a-a-a-a-a-a-a version` reaches the hint with
        // every segment one character long, and a value of any length composed
        // the same way reaches it too.
        let exactly = format!("-{}", "a".repeat(MAX_NAME - 1));
        let one_more = format!("-{}", "a".repeat(MAX_NAME));

        assert_eq!(exactly.len(), MAX_NAME);
        assert_eq!(one_more.len(), MAX_NAME + 1);

        assert!(admits_flag(&exactly), "a value of exactly 64 is admitted");
        assert!(
            !admits_flag(&one_more),
            "a value of 65 is refused, whatever its segments measure"
        );

        // The shape the requirement reproduces, at both sides of the bound: 32
        // segments of one character is 64 and is admitted; 33 is 66 and is not.
        let admitted = "-a".repeat(MAX_NAME / 2);
        let refused = "-a".repeat(MAX_NAME / 2 + 1);

        assert_eq!(admitted.len(), MAX_NAME);
        assert!(admits_flag(&admitted));
        assert!(!admits_flag(&refused));

        // And the bound is not a refusal of the alphabet: the five flags this
        // corpus enumerates with a hyphen inside the name are all admitted.
        for flag in [
            "--tpl-dir",
            "--no-cache",
            "--ca-file",
            "--ca-path",
            "--password-command",
        ] {
            assert!(admits_flag(flag), "{flag}");
        }
    }

    #[test]
    fn fr_cli_018_a_value_the_set_refuses_leaves_the_corrected_form_reachable() {
        // FR-CLI-018 obliges the hint to show the corrected form for every
        // value, and FR-ERR-040 puts a placeholder in the value's position
        // rather than removing the line.
        use crate::error::Error;

        let refused = super::hint(&Error::SeparateTokenValue {
            flag: "--database".to_owned(),
            value: format!("-{}", "a".repeat(MAX_NAME)),
        });

        assert_eq!(refused, "write the value in one token: --database=<value>");

        let admitted = super::hint(&Error::SeparateTokenValue {
            flag: "--database".to_owned(),
            value: "-shop".to_owned(),
        });

        assert_eq!(admitted, "write the value in one token: --database=-shop");
    }

    #[test]
    fn fr_err_022_a_command_path_is_admitted_segment_by_segment() {
        assert!(admits_path("schema"));
        assert!(admits_path("cfg database add"));
        assert!(!admits_path(""));
        assert!(!admits_path("cfg database add; rm"));
    }

    #[test]
    fn fr_err_022_a_flag_is_admitted_segment_by_segment() {
        // FR-ERR-022: the `-` or `--` that introduces a flag is a literal, and
        // so is the `-` inside the five flags of this corpus that carry one.
        assert!(admits_flag("-d"));
        assert!(admits_flag("--host"));
        assert!(admits_flag("--ca-file"));
        assert!(admits_flag("--password-command"));

        assert!(!admits_flag(""), "a flag with no name");
        assert!(!admits_flag("--"), "a flag that is only its introducer");
        assert!(!admits_flag("--ca-"), "an empty trailing segment");
        assert!(!admits_flag("---rf"), "an empty leading segment");
        assert!(!admits_flag("--host; rm -rf /"), "a semicolon and a space");
        assert!(!admits_flag("--host\nrm"), "a newline");
    }

    #[test]
    fn fr_err_023_a_refused_entry_name_leaves_the_placeholder_standing() {
        assert_eq!(
            update_entry("shop"),
            "tpl cfg database update shop --host <host>"
        );
        assert_eq!(
            update_entry("shop; rm -rf /"),
            "tpl cfg database update <entry> --host <host>"
        );
    }
}
