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

use super::suggest;
use crate::error::{
    CatalogueObjectKind, ContextFault, DeadlineBound, DsnFault, EntryRepair, Error, NetworkPhase,
    ReadOnlyFault, TlsFault,
};

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

/// The content of the `hint` line for `error`, without its label.
///
/// The match is exhaustive and carries no wildcard arm, so a variant added to
/// [`Error`] does not compile until it is given a hint.
pub(super) fn hint(error: &Error) -> Cow<'static, str> {
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
        Error::UnknownFlag {
            token,
            command,
            positional,
            nearest,
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
        Error::FlagValueMissing { flag } => {
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
        Error::MissingArgument { command, .. } => {
            Cow::Owned(format!("show the usage with: {}", help_of(command)))
        }
        // Neither flag enters a runnable command, so FR-ERR-022 is not engaged
        // and both are named as prose, escaped on the way out. The pair is
        // nonetheless tested, as a defensive assertion: a flag is a spelling
        // this corpus enumerates, so a token that fails the test came from
        // somewhere other than the flag table and is not reproduced.
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
            if *complete && admits_path(command) {
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
        Error::ConnectionDetailsMissing { entry } => Cow::Owned(format!(
            "say where to connect, e.g.: tpl cfg database add {} --host <host> --user <user> \
             --schema <database>",
            entry_or_placeholder(entry)
        )),
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
            ..
        } => {
            if admits(name) {
                Cow::Owned(format!("write it as: tpl {invocation} {prefix}:{name}"))
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
            name, invocation, ..
        }
        | Error::AmbiguousRoutineInContext {
            name, invocation, ..
        } => {
            if admits(name) {
                Cow::Owned(format!(
                    "name the kind you mean: tpl {invocation} procedure:{name}, or tpl \
                     {invocation} function:{name}"
                ))
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
        Error::LoadWithoutStoring => Cow::Borrowed(
            "remove --no-cache from the command; to read without storing, use tpl schema dump \
             --no-cache instead",
        ),
        // FR-CFG-031: the `cause` carries the form expected, and the hint the
        // worked value. The example carries no user, so that no `@` reaches a
        // hint line: the assertion over every hint looks for that character
        // to prove no value reached one ungated.
        Error::MalformedValue {
            parameter, command, ..
        } => match (parameter.as_str(), command.as_str()) {
            ("--dsn", _) => Cow::Borrowed(
                "write the URL as scheme://host/database, e.g.: --dsn \
                 mysql://db.example.com:3306/shop",
            ),
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
        Error::DatabaseEntryAlreadyExists { name, .. } => {
            Cow::Owned(format!("change it instead with: {}", update_entry(name)))
        }
        // FR-CFG-048 obliges a runnable command that makes the write legal, and
        // the three are the three shapes such a command takes. Which one is
        // decided where the refusal is raised, because only the writer knows
        // whether the conflicting key is one the file carries, one of several,
        // or one the same invocation supplied.
        Error::IncoherentEntryWrite {
            entry,
            conflicting,
            repair,
            ..
        } => match repair {
            EntryRepair::Unset => Cow::Owned(format!(
                "remove the key it conflicts with: {}",
                unset_key(conflicting)
            )),
            EntryRepair::Rewrite => Cow::Owned(format!(
                "write the entry one way only: {}",
                rewrite_entry(entry)
            )),
            EntryRepair::Restate(command) => Cow::Owned(format!(
                "write a DSN that carries no password: {}",
                restate_entry(command, entry)
            )),
        },

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
            ..
        } => {
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
            Cow::Borrowed("print the template root with: tpl template path")
        }
        // The document came from the caller, and `--context` excludes `-d`, so
        // the entry that would produce a valid one is a value only the caller
        // knows; the placeholders stand for it and for the file.
        Error::ContextDocumentMalformed { fault, .. } => match fault {
            ContextFault::NotJson(_) => Cow::Borrowed(
                "write a well-formed document with: tpl -d <entry> schema dump > <file>",
            ),
            ContextFault::Structure { .. } | ContextFault::DanglingReference { .. } => {
                Cow::Borrowed(
                    "write a document that matches, with: tpl -d <entry> schema dump > <file>",
                )
            }
        },
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
        Error::CatalogueObjectNotFound {
            kind,
            entry,
            nearest,
            ..
        } => {
            let generic = match listing(*kind) {
                Some(listing) if admits(entry) => Cow::Owned(format!(
                    "list the available {listing} with: tpl -d {entry} schema {listing}"
                )),
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
        Error::ContextObjectNotFound { kind, nearest, .. } => {
            let generic = match listing(*kind) {
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
        Error::DatabaseEntryNotFound { nearest, .. } => suggested(
            nearest,
            admits,
            "list the entries with: tpl cfg database list",
        ),
        // FR-CFG-007 obliges the nearest-match half over the keys that do
        // exist in the file.
        Error::ConfigurationKeyNotFound { nearest, .. } => {
            let admitted = admitted(nearest, admits_key);
            suggest::hint_line(
                admitted.iter().copied(),
                "list the keys that are set with: tpl cfg list",
            )
        }

        // ------------------------------------------------------------ 69 ---
        // BR-ERR-004: the entry is the one the connection was opened for, and
        // it is written into the command rather than left as a placeholder.
        // The new host and port are values only the caller knows.
        Error::NameNotResolved { entry, .. } => Cow::Owned(format!(
            "check the host name, or change it with: tpl cfg database update {} --host <host>",
            entry_or_placeholder(entry)
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
        Error::ConfigurationUnsafeMode { path, .. } => {
            Cow::Owned(format!("chmod 600 {}", configuration_file(path)))
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
            key, file, nearest, ..
        } => {
            let admitted = admitted(nearest, admits_key);
            let named = if admits_key(key) {
                Cow::Owned(format!("'{key}'"))
            } else {
                Cow::Borrowed("the key named above")
            };
            let generic = format!(
                "or delete {named} from {}; tpl help cfg set lists every key the file may hold",
                configuration_file(file)
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

            if admits_key(&key) {
                Cow::Owned(format!(
                    "point it at a directory holding certificate files with: tpl cfg set {key} \
                     <path>, or remove it with: tpl cfg unset {key}"
                ))
            } else {
                Cow::Borrowed(
                    "point the key at a directory holding certificate files, or remove it with: \
                     tpl cfg unset database.<entry>.ca_path",
                )
            }
        }
        Error::ConfigurationValueMalformed {
            key,
            file,
            position,
            expected,
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
                DsnFault::Form => Cow::Owned(format!(
                    "edit {}: write {named} as scheme://host/database, e.g. \
                     mysql://db.example.com:3306/shop; {NO_CFG_COMMAND}",
                    configuration_file(file)
                )),
            }
        }
        // FR-CONF-020 makes `$$` the literal dollar, so a value that meant one
        // is corrected by doubling it rather than by closing a brace.
        Error::UnclosedExpansion { key, file } => Cow::Owned(format!(
            "edit {}: close the expansion in {} as ${{VAR}}, or write $$ for a literal dollar \
             sign; {NO_CFG_COMMAND}",
            configuration_file(file),
            key_or_placeholder(key)
        )),
        // FR-CONF-035 obliges the hint to show the array form, and states this
        // example itself.
        Error::PasswordCommandNotAnArray { file, position, .. } => Cow::Owned(format!(
            "edit {} at line {} and write it as an array: password_command = [\"security\", \
             \"find-generic-password\", \"-s\", \"tpl-shop\", \"-w\"]",
            configuration_file(file),
            position.line
        )),
        Error::ConflictingEntryKeys {
            entry,
            file,
            first,
            second,
        } => {
            let (first, second) = (leaf(first), leaf(second));

            if admits(entry) && admits(first) && admits(second) {
                Cow::Owned(format!(
                    "edit {}: under [database.{entry}], delete either {first} or {second}; \
                     {NO_CFG_COMMAND}",
                    configuration_file(file)
                ))
            } else {
                Cow::Owned(format!(
                    "edit {}: keep one of the two keys named above and delete the other; \
                     {NO_CFG_COMMAND}",
                    configuration_file(file)
                ))
            }
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
                Cow::Owned(format!("define it with: export {name}=<value>"))
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
        Error::PasswordCommandNotExecutable { .. } => Cow::Borrowed(
            "check that the first element of password_command is the path of an executable \
             program, then run the command again",
        ),
        // FR-CONF-032 records the remedy: the child's own standard error is
        // visible when the command is run directly, and nowhere else.
        Error::PasswordCommandFailed { .. } => Cow::Borrowed(
            "run the command directly to see why it failed; tpl sends its standard error to the \
             null device",
        ),
        Error::ReadOnlySessionNotEnforced { entry, fault } => {
            let remedy = match fault {
                ReadOnlyFault::NotApplied => "accepts a read-only session",
                ReadOnlyFault::ReadBackDisagreed => "reports the read-only session it accepted",
            };
            Cow::Owned(format!(
                "repoint the entry at a server that {remedy}: {}",
                update_entry(entry)
            ))
        }
        // FR-CONF-040 and FR-CONF-041 each fix this line: the flag of
        // `FR-CFG-027` that writes the key the entry does not carry, on the
        // command that updates the entry. The entry name is a value this
        // corpus does not fix, so it is filled in only where `FR-ERR-022`
        // admits it and the placeholder stands otherwise, per `FR-ERR-023`.
        Error::EntryKeyMissing {
            entry,
            flag,
            placeholder,
            ..
        } => Cow::Owned(format!(
            "set it with: tpl cfg database update {} {flag} {placeholder}",
            entry_or_placeholder(entry)
        )),
        Error::NoDatabaseEntrySelected { .. } => Cow::Borrowed(
            "select an entry with -d <entry>, or set a default with: tpl cfg set core.database \
             <entry>; list the entries with: tpl cfg database list",
        ),
        Error::ServerNotMariaDb { entry, .. } => Cow::Owned(format!(
            "repoint the entry at a MariaDB server: {}",
            update_entry(entry)
        )),
        // FR-SRV-030 obliges this hint to carry `tpl cfg database update` with
        // the entry name filled in.
        Error::SeriesNotSupported { entry, .. } => Cow::Owned(format!(
            "repoint the entry at a supported server: {}",
            update_entry(entry)
        )),
    }
}

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

/// A parent directory as `tpl init` takes it: `.` where the path had no parent
/// segment of its own.
fn parent_or_here(parent: &Path) -> Cow<'_, str> {
    if parent.as_os_str().is_empty() {
        Cow::Borrowed(".")
    } else {
        parent.to_string_lossy()
    }
}

/// The hint for an undefined expression whose first segment is a variable a
/// flag binds, or [`None`] where it is not one.
///
/// `table`, `view` and `routine` exist only when the object flag of the same
/// name is given, per `FR-RND-023`, and a key of `vars` exists only when a
/// `--set` defines it, per `FR-CTX-026`. The key is a `--set` key, which
/// `FR-RND-012` confines to the set of `FR-ERR-022`; one outside it leaves the
/// placeholder.
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

/// The two commands that write one entry afresh (`FR-CFG-048`).
///
/// `FR-CFG-048` names this pair as the second of the two repairs, and it is the
/// one that applies where no single `tpl cfg unset` would make the write legal.
fn rewrite_entry(entry: &str) -> Cow<'static, str> {
    if admits(entry) {
        Cow::Owned(format!(
            "tpl cfg database remove {entry}, then tpl cfg database add {entry} --dsn <url>"
        ))
    } else {
        Cow::Borrowed(
            "tpl cfg database remove <entry>, then tpl cfg database add <entry> --dsn <url>",
        )
    }
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
