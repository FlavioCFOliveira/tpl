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
    ChildEnd, ContextFault, DeadlineBound, DsnFault, EntryNameGiven, EntryRepair, Error,
    KeyAbsence, Missing, PasswordCommandFault, ReadOnlyFault, ReferenceFault, RenderReason,
    TlsFault, TplDirFault, Unresolved, context_name, password_pair,
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

/// The rule of `FR-CONF-048`, in the words every `cause` that applies it uses.
const ENTRY_NAME_RULE: &str = "an entry name is 1 to 64 letters, digits or underscores";

/// The rule of `FR-CONF-049`, in the words every `cause` that applies it uses.
const VARIABLE_NAME_RULE: &str = "a variable name starts with a letter or an underscore and \
                                  holds only letters, digits and underscores";

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
        // W-05 of the sixth re-audit of rmp `#263`: the words of a command
        // given as one argument, where they form a command path.
        Error::UnknownCommand { token, node, .. }
            if super::hint::split_invocation(node, token).is_some() =>
        {
            Cow::Owned(format!(
                "'{token}' was given as one argument; each word of a command is its own argument"
            ))
        }
        Error::UnknownCommandPathSegment {
            segment: token,
            node,
            ..
        } if super::hint::split_command(node, token).is_some() => Cow::Owned(format!(
            "'{token}' was given as one argument; each word of a command is its own argument"
        )),
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
            "'{segment}' is not a subcommand of '{}' (commands are matched in full, never by a \
             prefix)",
            invoked(node)
        )),
        Error::UnknownFlag { token, command, .. }
            if super::hint::is_format_on_dump(command, token) =>
        {
            Cow::Borrowed("schema dump always writes JSON and takes no --format")
        }
        // Y-02 of the eighth re-audit of rmp `#263`: the flag is the
        // command's, written before it, where FR-CLI-024 admits only a global
        // flag.
        Error::UnknownFlag {
            token,
            belongs_to: Some(path),
            ..
        } => Cow::Owned(format!(
            "'{token}' is a flag of '{}' and was written before that command; only the global \
             flags ({}) may come before a command",
            invoked(path),
            global_flags()
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
            None => match key.strip_prefix("database.") {
                Some(name) => Cow::Owned(format!(
                    "tpl cfg get reads one key; {key} is the block of entry '{name}', which \
                     .tpl/.cfg does not declare"
                )),
                None => Cow::Owned(format!(
                    "tpl cfg get reads one key; {key} is a block of keys, not a key"
                )),
            },
        },
        // FR-RND-041 fixes both lines: why the two contradict each other.
        Error::DirectWithContext => Cow::Borrowed(
            "--context reads the context from a document and contacts no server; --direct demands \
             a read from the server",
        ),
        Error::MutuallyExclusiveFlags { first, second } => Cow::Owned(format!(
            "the invocation supplies both '{first}' and '{second}'; at most one of the two may be \
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
        Error::LoadWithoutStoring { .. } => Cow::Borrowed(
            "'tpl cache load' reads the server in order to store what it read, so an invocation \
             that forbids the store asks the command to do nothing",
        ),
        // FR-CONF-046: the condition the string met, then what the key or the
        // flag takes. The condition is what `expected` carries for these two.
        Error::MalformedValue {
            parameter,
            expected,
            ..
        } if is_password_command(parameter) => Cow::Owned(format!(
            "{expected}; {parameter} takes one command line written as one string, which tpl \
             splits into words"
        )),
        // U-03: the file expands a reference in a port, and the write paths
        // do not; the line says where the reference is accepted.
        Error::MalformedValue {
            parameter,
            value,
            expected,
            ..
        } if is_port_reference(parameter, value) => Cow::Owned(format!(
            "'{value}' was supplied for '{parameter}', which takes {expected}; a ${{VAR}} \
             reference for a port is accepted only when written in .tpl/.cfg"
        )),
        // V-03, FR-CONF-047: the key or the flag, and why the reference is
        // refused rather than stored.
        Error::MalformedValue {
            parameter, value, ..
        } if is_path_reference(parameter, value) => Cow::Owned(format!(
            "{} is read as a literal path, and ${{VAR}} is not expanded in it; {}",
            parameter.rsplit('.').next().unwrap_or(parameter),
            reference_held(value)
        )),
        // V-06: the file variant's clause, so that the two say the same.
        Error::MalformedValue {
            parameter,
            value,
            expected,
            ..
        } if is_whole_dsn_reference(parameter, value) => Cow::Owned(format!(
            "'{value}' was supplied for '{parameter}', which takes {expected}; a ${{VAR}} may \
             stand for one part of the URL, never for the whole of it"
        )),
        Error::MalformedValue {
            parameter,
            value,
            expected,
            ..
        } => Cow::Owned(format!(
            "'{value}' was supplied for '{parameter}', which takes {expected}"
        )),
        // FR-PROJ-029: the path as written and why it cannot be a destination.
        Error::InitDestinationIsTplFolder {
            written, canonical, ..
        } => match canonical {
            None => Cow::Owned(format!(
                "the path {} ends in .tpl, so a project there would be nested inside that folder",
                written.display()
            )),
            Some(canonical) => Cow::Owned(format!(
                "the path {} is the folder {}, whose last segment is .tpl, so a project there \
                 would be nested inside that folder",
                written.display(),
                canonical.display()
            )),
        },
        // FR-CONF-048: the rule, and the value described rather than
        // reproduced, since a refused name is outside the set of FR-ERR-022.
        Error::InvalidEntryName { given, name } => {
            let described = entry_name_fault(name);
            match given {
                EntryNameGiven::CoreDatabase if name.contains("${") => Cow::Owned(format!(
                    "core.database names a database entry and does not expand ${{VAR}}; \
                     {ENTRY_NAME_RULE}"
                )),
                EntryNameGiven::CoreDatabase => Cow::Owned(format!(
                    "core.database names a database entry and {described}; {ENTRY_NAME_RULE}"
                )),
                EntryNameGiven::Add | EntryNameGiven::Key(_) => {
                    Cow::Owned(format!("the entry name {described}; {ENTRY_NAME_RULE}"))
                }
            }
        }
        // FR-CONF-049: the key or the flag, and the rule for a variable name.
        Error::InvalidReference {
            parameter, fault, ..
        } => match fault {
            ReferenceFault::Unclosed => Cow::Owned(format!(
                "the value of {parameter} opens a ${{ reference that is never closed; a \
                 reference is written ${{NAME}}"
            )),
            ReferenceFault::Name(name) => Cow::Owned(format!(
                "{} in the value of {parameter} is not a valid reference; {VARIABLE_NAME_RULE}",
                reference_named(name)
            )),
        },
        // FR-CONF-050: the flag or the key, and that the value is empty.
        Error::EmptyValue { parameter, .. } => Cow::Owned(format!(
            "{parameter} is empty or holds only whitespace; an entry must name {}",
            if parameter.ends_with("host") {
                "a host"
            } else {
                "a database"
            }
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
        } => {
            // The rule the pair breaks, and only that one: the connection
            // form, or the source of the password.
            let rule = if password_pair(written, conflicting) {
                PASSWORD_RULE
            } else {
                CONNECTION_RULE
            };
            Cow::Owned(match repair {
                EntryRepair::Restate(_) => format!(
                    "the invocation writes both {written} and {conflicting} to database entry \
                     '{entry}'; {rule}; nothing was written"
                ),
                // FR-CFG-048: what switching the entry to the other form
                // removes, stated and never performed by the hint.
                EntryRepair::InsideDsn { .. } => format!(
                    "the invocation writes {written} and database entry '{entry}' is defined by \
                     {conflicting}; {rule}; nothing was written. Unsetting {conflicting} would \
                     also remove the host, port, user, password and database it carries"
                ),
                EntryRepair::Discrete { carried, .. } => format!(
                    "the invocation writes {written} and database entry '{entry}' already \
                     declares {conflicting}; {rule}; nothing was written. Describing the entry \
                     by dsn requires unsetting {}",
                    conjoined(
                        &carried
                            .iter()
                            .map(|field| format!("database.{entry}.{field}"))
                            .collect::<Vec<_>>()
                    )
                ),
                EntryRepair::Unset | EntryRepair::DsnWithoutPassword => format!(
                    "the invocation writes {written} and database entry '{entry}' already \
                     declares {conflicting}; {rule}; nothing was written"
                ),
            })
        }
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
        // FR-CLI-026: the value the flag took, and the token read as the
        // command because of it.
        Error::FlagTookCommand {
            flag, value, token, ..
        } => Cow::Owned(format!(
            "{flag} took '{value}' as its value, which is a command, so '{token}' was read as the \
             command"
        )),
        Error::RepeatedFlag { flag } => Cow::Owned(format!(
            "'{flag}' carries no value, so a second occurrence of it states nothing the first did \
             not; tpl accepts each flag once"
        )),
        Error::FlagValueMissing { flag, .. } => Cow::Owned(format!(
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
            reason,
            ..
        } => match (undefined, reason.as_deref()) {
            // FR-SEM-015: the message is on the error line; the cause says
            // where the call is and what the engine reported.
            (_, Some(RenderReason::Failed(_))) => Cow::Owned(format!(
                "'{template}' ended the render on purpose by calling fail() at {position}: {}",
                joined(chain)
            )),
            (Some(expression), Some(RenderReason::Unresolved(unresolved))) => Cow::Owned(format!(
                "'{template}' at {position} reads '{expression}', and {} found {}, so there is \
                 nothing to read",
                unresolved.call,
                sought(unresolved)
            )),
            // Y-01: the variable is bound, so the cause names the step of the
            // expression that found nothing.
            (Some(expression), Some(RenderReason::Missing(missing))) => Cow::Owned(format!(
                "'{template}' at {position} reads '{expression}', and {}; the template engine \
                 reports: {}",
                missing_step(missing),
                joined(chain)
            )),
            // T-07: the chain FR-ERR-011 carries is introduced as what the
            // engine reported, so its bare "undefined value" does not read as
            // a dangling label of this sentence.
            (Some(expression), None) => Cow::Owned(format!(
                "'{template}' at {position} reads '{expression}', which is not defined in this \
                 render; the template engine reports: {}",
                joined(chain)
            )),
            (_, Some(RenderReason::IncludeNotFound { .. }))
            | (None, None | Some(RenderReason::Unresolved(_) | RenderReason::Missing(_))) => {
                Cow::Owned(format!(
                    "'{template}' failed while being evaluated, at {position}: {}",
                    joined(chain)
                ))
            }
        },
        Error::TemplateOutsideRoot { name, root } => Cow::Owned(format!(
            "'{name}' resolves to a path outside the template folder {}, and tpl reads no template \
             from outside it",
            root.display()
        )),
        // The row obliges the path and either the position of the malformed
        // JSON or the structural rule the document failed.
        Error::ContextDocumentMalformed { path, fault, .. } => match fault {
            ContextFault::NotJson(position) => Cow::Owned(format!(
                "{} is not well-formed JSON; the parser stopped at {position}",
                context_name(path)
            )),
            ContextFault::Empty => Cow::Owned(format!(
                "{} is empty, and a context document is one JSON object",
                context_name(path)
            )),
            ContextFault::Structure { at, expected } if at.is_empty() => Cow::Owned(format!(
                "{} is well-formed JSON and is not a context document: {expected}",
                context_name(path)
            )),
            ContextFault::Structure { at, expected } => Cow::Owned(format!(
                "{} is well-formed JSON, and {at} {expected}",
                context_name(path)
            )),
            // FR-CTX-042: the path, the table carrying the key, the key, and
            // the table it names that `tables` does not carry.
            ContextFault::DanglingReference {
                table,
                collection,
                key,
                names,
            } => Cow::Owned(format!(
                "{} is well-formed JSON, and table '{table}' lists key '{key}' under \
                 {collection}, naming table '{names}', which data.database.tables does not carry",
                context_name(path)
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
        // FR-CACHE-040: the population is what the cache holds.
        // Y-05 of the eighth re-audit of rmp `#263`: a qualified routine is
        // named by its kind, and a routine of the other kind under that name
        // is named too.
        Error::NothingCachedNamed {
            kind,
            name,
            entry,
            qualified,
            held_as,
            ..
        } => {
            let sought = qualified.map_or_else(|| kind.to_string(), str::to_owned);
            Cow::Owned(match held_as {
                Some(other) => format!(
                    "the cache of entry '{entry}' holds no {sought} named '{name}'; it holds a \
                     {other} of that name"
                ),
                None => format!("the cache of entry '{entry}' holds no {sought} named '{name}'"),
            })
        }
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
            "no template named '{name}' exists under the template folder {}",
            root.display()
        )),
        Error::DatabaseEntryNotFound {
            name,
            file,
            by_default,
            ..
        } => Cow::Owned(format!(
            "{} declares no database entry named '{name}'{}",
            file.display(),
            if *by_default {
                ", and core.database names it as the entry to use when -d is not given"
            } else {
                ""
            }
        )),
        // FR-ERR-035 separates this from the 64 and the 78 that also name a
        // key, and the exit code carries the separation: this is the key the
        // file does not set. The line does not also claim the key is one the
        // space admits, because FR-CFG-007 reaches a key outside it too — a
        // spelling `tpl cfg get` was given and `.tpl/.cfg` does not carry.
        // T-05: a name outside the space is said to be one, because "sets no
        // value" of it reads as though setting it would help.
        // FR-CFG-012: `tpl cfg unset core` or `database` where the file has no
        // such section.
        Error::ConfigurationKeyNotFound { key, file, .. }
            if crate::error::section_named(key).is_some() =>
        {
            Cow::Owned(format!(
                "{} has no [{key}] section, so tpl cfg unset has nothing to delete",
                file.display()
            ))
        }
        // U-05: a key or block of an entry the file does not declare names
        // the missing entry; the key is not what the caller got wrong.
        Error::ConfigurationKeyNotFound {
            key,
            file,
            entry_missing: true,
            ..
        } => Cow::Owned(format!(
            "{} declares no [database.{}] block",
            file.display(),
            crate::error::named_entry(key)
        )),
        Error::ConfigurationKeyNotFound {
            key,
            file,
            known: true,
            ..
        } => Cow::Owned(format!("{} sets no value for '{key}'", file.display())),
        Error::ConfigurationKeyNotFound {
            key, known: false, ..
        } => Cow::Owned(format!(
            "'{key}' is none of the keys tpl reads, so no file sets it"
        )),

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
            "the filesystem refused to create {}: {}",
            path.display(),
            os(returned)
        )),

        // ------------------------------------------------------------ 74 ---
        // The row obliges the path or stream, the operation attempted on it,
        // and what the filesystem or the stream returned.
        Error::ProjectFileUnreadable { path, returned } => Cow::Owned(format!(
            "the read of {} returned: {}",
            path.display(),
            os(returned)
        )),
        Error::ContextDocumentUnreadable { path, returned } => Cow::Owned(format!(
            "the read of the file --context named, {}, returned: {}",
            path.display(),
            os(returned)
        )),
        Error::TrustMaterialUnreadable { path, returned, .. } => Cow::Owned(format!(
            "the read of {} returned: {}; the connection was not attempted",
            path.display(),
            os(returned)
        )),
        Error::ProjectFileUnwritable { path, returned } => Cow::Owned(format!(
            "the write of {} returned: {}; the previous file is still in place, unchanged",
            path.display(),
            os(returned)
        )),
        Error::StdoutUnwritable { returned } => Cow::Owned(format!(
            "the write to standard output returned: {}",
            os(returned)
        )),
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
            "INFORMATION_SCHEMA did not return the {property} of {kind} '{object}' to the connecting \
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
            // FR-PROJ-027 names the one of two facts that applies.
            TplDirFault::HoldsTplFolder => Cow::Owned(format!(
                "--tpl-dir disabled the upward search; {} holds a .tpl folder, and --tpl-dir \
                 must name that folder",
                path.display()
            )),
            TplDirFault::NotTplFolder => Cow::Owned(format!(
                "--tpl-dir disabled the upward search; --tpl-dir names the .tpl folder of a \
                 project, not the directory that holds it, and {} holds none",
                path.display()
            )),
        },
        // FR-PROJ-028: the folder, that it holds no `.cfg`, and that another
        // user owns it.
        Error::ProjectFolderNotOwned {
            path,
            owner,
            expected,
        } => Cow::Owned(format!(
            "{} holds no .cfg, and the folder is owned by another user, uid {owner}; tpl uses a \
             .tpl folder without .cfg only when the invoking user, uid {expected}, owns it",
            path.display()
        )),
        // FR-CONF-047: the key, that it is read as a literal path, and that
        // `${VAR}` is not expanded in it.
        Error::ConfigurationPathReference {
            key,
            file,
            position,
            value,
        } => Cow::Owned(format!(
            "{} at {position} declares {key} with a ${{VAR}} reference; {} is read as a literal \
             path, and ${{VAR}} is not expanded in it; {}",
            file.display(),
            key.rsplit('.').next().unwrap_or(key),
            reference_held(value)
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
            "mode {mode:04o} grants access to group or other; tpl reads {} only when group and \
             other have no access, as at mode 0600",
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
            element,
        } => {
            let article = if found.starts_with(['a', 'e', 'i', 'o', 'u']) {
                "an"
            } else {
                "a"
            };
            Cow::Owned(match element {
                Some(index) => format!(
                    "{} at {position} declares {key} with {article} {found} at index {index}; \
                     every element of this key is a string",
                    file.display()
                ),
                None if *found == crate::project::config::EMPTY_ARRAY => format!(
                    "{} at {position} declares {key} as an empty array; this key takes an array \
                     of at least one string, the program first",
                    file.display()
                ),
                None => format!(
                    "{} at {position} declares {key} as {article} {found}; this key takes an \
                     array of strings",
                    file.display()
                ),
            })
        }
        Error::ConflictingEntryKeys {
            entry,
            file,
            first,
            second,
        } => Cow::Owned(if password_pair(first, second) {
            format!(
                "{} gives database entry '{entry}' a password through both {first} and {second}; \
                 {PASSWORD_RULE}",
                file.display()
            )
        } else {
            format!(
                "{} declares both {first} and {second} for database entry '{entry}'; \
                 {CONNECTION_RULE}",
                file.display()
            )
        }),
        // T-04: a value a `${VAR}` produced is not what the file holds, and
        // the line says which of the two is at fault.
        Error::ConfigurationValueMalformed {
            key,
            file,
            position,
            found,
            expected,
            expanded_from: Some(written),
        } => Cow::Owned(format!(
            "{} at {position} writes {key} as {written}, which the environment expands to \
             {found}; this key takes {expected}",
            file.display()
        )),
        Error::ConfigurationValueMalformed {
            key,
            file,
            position,
            found,
            expected,
            expanded_from: None,
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
        // FR-CONF-049: met where the field is expanded, as FR-CONF-021 is.
        Error::InvalidReferenceName { key, file, name } => Cow::Owned(format!(
            "{key} in {} holds {}, which is not a valid reference; {VARIABLE_NAME_RULE}",
            file.display(),
            reference_named(name)
        )),
        // FR-CONF-048: the rule, and the value described where it is outside
        // the set of FR-ERR-022, which a refused name always is.
        Error::ConfigurationEntryName {
            file,
            name,
            core,
            position,
        } => {
            let what = if *core {
                "the value of core.database"
            } else {
                "the name of a [database.<name>] block"
            };
            let reference = if *core && name.contains("${") {
                "; core.database names an entry and ${VAR} is not expanded in it"
            } else {
                ""
            };
            Cow::Owned(format!(
                "{what} at line {}, column {} of {} {}{reference}; {ENTRY_NAME_RULE}",
                position.line,
                position.column,
                file.display(),
                entry_name_fault(name)
            ))
        }
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
            ..
        } => match bound {
            DeadlineBound::Phase => Cow::Owned(format!(
                "password_command {command:?} was still running, or still held its standard output \
                 open, when its own deadline of {limit:?} expired, and its process group was \
                 terminated"
            )),
            DeadlineBound::Overall => Cow::Owned(format!(
                "password_command {command:?} was still running, or still held its standard output \
                 open, when the overall budget of {limit:?}, measured from process start, expired, \
                 and its process group was terminated"
            )),
        },
        Error::PasswordCommandOutputCapExceeded { command, cap, .. } => Cow::Owned(format!(
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
            ..
        } => match fault {
            PasswordCommandFault::NotStarted => Cow::Owned(format!(
                "password_command {command:?} could not be started: {}",
                os(returned)
            )),
            PasswordCommandFault::StatusUnreadable => Cow::Owned(format!(
                "password_command {command:?} was started and tpl could not read the status it \
                 ended with: {}",
                os(returned)
            )),
        },
        // FR-CONF-033 for the exit and FR-CONF-043 for the signal. The signal
        // number is named because FR-ERR-034 bans a category where an instance
        // is available, and a Unix reports the signal that ended a child: a
        // line saying only that the child was signalled would read identically
        // for a supervisor, a memory limit and an interactive interrupt.
        Error::PasswordCommandFailed { command, end, .. } => match end {
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
             'database.{entry}.ca_path' adds no certificate to the certificates the connection \
             trusts",
            path.display()
        )),
        Error::ReadOnlySessionNotEnforced { entry, fault, .. } => match fault {
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
        // FR-CONF-041 names the key; an entry defined by dsn carries it in its
        // dsn, so the line names the part of the dsn that holds nothing.
        Error::EntryKeyMissing {
            entry,
            key,
            file,
            flag,
            absence,
            ..
        } if *flag == "--dsn" => {
            let part = if key.ends_with(".host") {
                "host part"
            } else {
                "/database part"
            };
            let why = match absence {
                KeyAbsence::ExpandsToEmpty => {
                    "references an environment variable that expands to the empty string or to \
                     whitespace only"
                }
                KeyAbsence::Absent | KeyAbsence::Empty => "is empty or holds only whitespace",
            };
            Cow::Owned(format!(
                "the {part} of database.{entry}.dsn in {} {why}, so database entry '{entry}' \
                 has no '{key}', and this command cannot be run without it",
                file.display()
            ))
        }
        Error::EntryKeyMissing {
            entry,
            key,
            file,
            absence,
            ..
        } => match absence {
            KeyAbsence::Absent => Cow::Owned(format!(
                "{} declares database entry '{entry}' without '{key}', and this command cannot be \
                 run without it",
                file.display()
            )),
            // FR-CONF-050: an empty value is an absent key, and the line says
            // which it is.
            KeyAbsence::Empty => Cow::Owned(format!(
                "{} sets '{key}' of database entry '{entry}' to the empty string or to \
                 whitespace only, which names nothing, and this command cannot be run without it",
                file.display()
            )),
            KeyAbsence::ExpandsToEmpty => Cow::Owned(format!(
                "'{key}' of database entry '{entry}' in {} references an environment variable \
                 that expands to the empty string or to whitespace only, which names nothing, and \
                 this command cannot be run without it",
                file.display()
            )),
        },
        Error::NoDatabaseEntrySelected { file, .. } => Cow::Owned(format!(
            "neither -d/--database nor core.database in {} names an entry, and this command needs \
             one to know which database to use",
            file.display()
        )),
        Error::ServerNotMariaDb { entry, product, .. } => Cow::Owned(format!(
            "database entry '{entry}' reached a server that connected and authenticated and \
             reports '{product}'; tpl reads only MariaDB servers"
        )),
        Error::SeriesNotSupported {
            entry,
            series,
            supported,
            ..
        } => Cow::Owned(format!(
            "database entry '{entry}' connected and authenticated, and the server reports series \
             '{series}', which is outside the supported window; tpl supports {}",
            listed(supported)
        )),
    }
}

/// The rule of `FR-CONF-007` a pair of password sources breaks.
const PASSWORD_RULE: &str = "an entry takes its password from one place only: password_command, \
                             password, or the password inside dsn";

/// The rule of `FR-CONF-006` a pair of connection forms breaks.
const CONNECTION_RULE: &str = "an entry gives its connection either as dsn or as the separate \
                               keys host, port, user, password and database";

/// What a lookup that found nothing had sought, as the cause says it: "no
/// table named 'orders'", or "no column named 'id' in table 'orders'".
fn sought(unresolved: &Unresolved) -> String {
    match &unresolved.table {
        Some(table) => format!(
            "no {} named '{}' in table '{table}'",
            unresolved.kind, unresolved.name
        ),
        None => format!("no {} named '{}'", unresolved.kind, unresolved.name),
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

/// Joins names into one list, the last two with "and".
pub(super) fn conjoined<S: AsRef<str>>(names: &[S]) -> String {
    let mut line = String::new();
    for (index, name) in names.iter().enumerate() {
        if index + 1 == names.len() && index > 0 {
            line.push_str(LIST_CONJUNCTION);
        } else if index > 0 {
            line.push_str(LIST_SEPARATOR);
        }
        line.push_str(name.as_ref());
    }
    line
}

/// Joins the values a flag enumerates into the choice they are.
pub(super) fn alternatives(permitted: &[String]) -> Cow<'_, str> {
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

/// What the operating system returned, without the ` (os error N)` suffix
/// `std` appends to its own message: the number says nothing a reader of the
/// message can act on (finding U-08 of the fourth re-audit of rmp `#263`).
fn os(returned: &std::io::Error) -> String {
    let text = returned.to_string();
    match text.rsplit_once(" (os error ") {
        Some((message, code)) if code.ends_with(')') => message.to_owned(),
        _ => text,
    }
}

/// Whether `value`, given for a port, is a `${VAR}` reference: `FR-CONF-015`
/// expands one in the file, while `tpl cfg set` and `--port` take a number
/// (finding U-03 of the fourth re-audit of rmp `#263`).
pub(super) fn is_port_reference(parameter: &str, value: &str) -> bool {
    (parameter == "--port" || parameter.ends_with(".port")) && value.contains("${")
}

/// Whether `value`, given for `ca_file` or `ca_path` or for their flags,
/// holds `${`, which `FR-CONF-047` refuses in either key.
pub(crate) fn is_path_reference(parameter: &str, value: &str) -> bool {
    (matches!(parameter, "--ca-file" | "--ca-path")
        || parameter.ends_with(".ca_file")
        || parameter.ends_with(".ca_path"))
        && value.contains("${")
}

/// What is wrong with an entry name `FR-CONF-048` refuses, described rather
/// than reproduced: a refused name is outside the set of `FR-ERR-022`.
fn entry_name_fault(name: &str) -> &'static str {
    if name.is_empty() {
        "is empty"
    } else if name.chars().count() > 64 {
        "is longer than 64 characters"
    } else {
        "holds a character other than a letter, a digit or an underscore"
    }
}

/// The reference a `cause` names: `${NAME}` where the set of `FR-ERR-022`
/// admits the name, and a description otherwise (`FR-CONF-049`).
fn reference_named(name: &str) -> String {
    if name.is_empty() {
        "${}".to_owned()
    } else if super::hint::admits(name) {
        format!("${{{name}}}")
    } else {
        "a reference whose name holds a character other than a letter, a digit or an underscore"
            .to_owned()
    }
}

/// The words that say which reference a refused path holds: the first
/// `${NAME}` whose name the set of `FR-ERR-022` admits, or `${` alone
/// (`FR-CONF-047`).
fn reference_held(value: &str) -> String {
    value
        .split("${")
        .skip(1)
        .find_map(|rest| {
            let (name, _) = rest.split_once('}')?;
            super::hint::admits(name).then(|| format!("the value holds ${{{name}}}"))
        })
        .unwrap_or_else(|| "the value holds ${".to_owned())
}

/// Whether `value`, given for a `dsn` or for `--dsn`, is one `${VAR}` and
/// nothing else: `FR-CONF-018` expands a reference inside a part the URL
/// delimits, so it never stands for the whole URL (finding V-06 of the fifth
/// re-audit of rmp `#263`).
pub(super) fn is_whole_dsn_reference(parameter: &str, value: &str) -> bool {
    (parameter == "--dsn" || parameter.ends_with(".dsn"))
        && crate::project::config::expand::is_whole_reference(value)
}

/// Whether `parameter` is one that takes a `password_command` supplied as one
/// string: the flag of `FR-CFG-046`, or the key of `FR-CONF-002`.
pub(super) fn is_password_command(parameter: &str) -> bool {
    parameter == "--password-command" || parameter.ends_with(".password_command")
}

/// What the step of an expression rooted at a bound variable found
/// (finding Y-01 of the eighth re-audit of rmp `#263`).
fn missing_step(missing: &Missing) -> String {
    match missing {
        Missing::Attribute {
            owner, name, kind, ..
        } => {
            if *kind == "an object" {
                format!("'{owner}' has no attribute '{name}'")
            } else {
                format!("'{owner}' is {kind}, which has no attribute '{name}'")
            }
        }
        Missing::Index {
            owner,
            index,
            length,
        } => {
            let items = if *length == 1 { "item" } else { "items" };
            format!("'{owner}' has {length} {items}, so it has no item [{index}]")
        }
        Missing::NotList { owner, index, kind } => {
            format!("'{owner}' is {kind}, which has no item [{index}]")
        }
        Missing::Elsewhere { root } => format!(
            "'{root}' is defined, because --{root} names one, but a later step of the expression \
             is not"
        ),
    }
}

/// The global flags the root declares, as they are written: `-d/--database`.
fn global_flags() -> String {
    let tree = crate::cli::tree();
    let flags: Vec<String> = tree
        .get_arguments()
        .filter(|argument| !argument.is_positional())
        .map(
            |argument| match (argument.get_short(), argument.get_long()) {
                (Some(short), Some(long)) => format!("-{short}/--{long}"),
                (Some(short), None) => format!("-{short}"),
                (None, Some(long)) => format!("--{long}"),
                (None, None) => argument.get_id().as_str().to_owned(),
            },
        )
        .collect();
    flags.join(", ")
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
