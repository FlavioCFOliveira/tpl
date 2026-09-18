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

use super::suggest;
use crate::error::{
    CatalogueObjectKind, ContextFault, DeadlineBound, DsnFault, EntryRepair, Error, NetworkPhase,
    ReadOnlyFault,
};

/// The longest a name the character set of `FR-ERR-022` governs may be.
const MAX_NAME: usize = 64;

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
        // qualified, per FR-ERR-020.
        Error::UnknownCommand { nearest, .. } => {
            let admitted = admitted(nearest, admits_path);
            suggest::hint_line(admitted.iter().copied(), "list the commands with: tpl help")
        }
        // FR-HELP-028 draws the candidates from the children of the node the
        // path reached, which `cli/help.rs` selects over; the generic half
        // names that node, so the runnable command lists exactly the children
        // the segment was measured against rather than the whole tree.
        Error::UnknownCommandPathSegment { node, nearest, .. } => {
            let admitted = admitted(nearest, admits_path);
            let generic = children_of(node);

            Cow::Owned(suggest::hint_line(admitted.iter().copied(), &generic).into_owned())
        }
        Error::UnknownFlag { nearest, .. } => {
            let admitted = admitted(nearest, admits_flag);
            suggest::hint_line(
                admitted.iter().copied(),
                "list the flags a command declares with: tpl help <command>",
            )
        }
        Error::UnexpectedArgument { command, .. } => {
            if admits_path(command) {
                Cow::Owned(format!("show what it takes with: tpl help {command}"))
            } else {
                Cow::Borrowed("show what the command takes with: tpl help <command>")
            }
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
        Error::ValueOutsideEnumeration { .. } => {
            Cow::Borrowed("show the values the command accepts with: tpl help <command>")
        }
        Error::InvocationRejected { .. } => {
            Cow::Borrowed("show what the command accepts with: tpl help <command>")
        }
        Error::MissingArgument { command, .. } => {
            if admits_path(command) {
                Cow::Owned(format!("show the usage with: tpl help {command}"))
            } else {
                Cow::Borrowed("show the usage with: tpl help <command>")
            }
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
        Error::MalformedValue { .. } => {
            Cow::Borrowed("show what the command accepts with: tpl help <command>")
        }
        // FR-CFG-009 obliges the nearest-match half over the enumerated key
        // space of FR-CONF-002. Every key of that space is a spelling this
        // corpus fixes, so FR-ERR-022 makes it a literal and the test beside it
        // is the defensive assertion this module applies to every such value.
        Error::UnknownConfigurationKey { nearest, .. } => {
            let admitted = admitted(nearest, admits_key);
            suggest::hint_line(
                admitted.iter().copied(),
                "show what tpl cfg set accepts with: tpl help cfg set",
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
        Error::TemplateSyntax { .. } => {
            Cow::Borrowed("parse the project's templates with: tpl template check")
        }
        // `tpl template check` performs syntax analysis alone, per FR-TMPL-017,
        // so it would not reproduce an evaluation failure; the source is what
        // locates the line and column named above.
        Error::RenderFailed { .. } => {
            Cow::Borrowed("print the template's source with: tpl template show <template>")
        }
        Error::TemplateOutsideRoot { .. } => {
            Cow::Borrowed("print the template root with: tpl template path")
        }
        Error::ContextDocumentMalformed { fault, .. } => match fault {
            ContextFault::NotJson(_) => {
                Cow::Borrowed("produce a well-formed context document with: tpl schema dump")
            }
            ContextFault::Structure { .. } => {
                Cow::Borrowed("produce a document that matches the contract with: tpl schema dump")
            }
        },
        Error::RenderDeadlineExceeded { bound, .. } => match bound {
            DeadlineBound::Phase => Cow::Borrowed(
                "raise the render deadline with: tpl cfg set core.render_timeout <seconds>",
            ),
            DeadlineBound::Overall => Cow::Borrowed(
                "raise the overall budget with: tpl --timeout <seconds> render <template>",
            ),
        },

        // ------------------------------------------------------------ 66 ---
        Error::CatalogueObjectNotFound { kind, entry, .. } => {
            let listing = listing(*kind);
            if admits(entry) {
                Cow::Owned(format!(
                    "list the available {listing} with: tpl -d {entry} schema {listing}"
                ))
            } else {
                Cow::Owned(format!(
                    "list the available {listing} with: tpl schema {listing}"
                ))
            }
        }
        Error::TemplateNotFound { .. } => {
            Cow::Borrowed("list the project's templates with: tpl template list")
        }
        // FR-GLOB-007 obliges the nearest-match half over the entry names the
        // file defines. An entry name is a value this corpus does not fix, so
        // FR-ERR-022 governs it by the character set and FR-ERR-023 drops a
        // candidate outside it in every form.
        Error::DatabaseEntryNotFound { nearest, .. } => {
            let admitted = admitted(nearest, admits);
            suggest::hint_line(
                admitted.iter().copied(),
                "list the entries with: tpl cfg database list",
            )
        }
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
        Error::NameNotResolved { .. } => Cow::Borrowed(
            "check the name, or repoint the entry with: tpl cfg database update <entry> \
             --host <host>",
        ),
        Error::ConnectionRefused { .. } => Cow::Borrowed(
            "check that the server is listening, or repoint the entry with: tpl cfg database \
             update <entry> --port <port>",
        ),
        Error::TlsHandshakeFailed { .. } => Cow::Borrowed(
            "check the server's TLS, or state the mode with: tpl cfg set database.<entry>.tls \
             <mode>",
        ),
        Error::NetworkDeadlineExceeded { phase, bound, .. } => match bound {
            DeadlineBound::Phase => Cow::Owned(format!(
                "raise the deadline with: tpl cfg set {} <seconds>",
                deadline_key(*phase)
            )),
            DeadlineBound::Overall => {
                Cow::Borrowed("raise the overall budget with: tpl --timeout <seconds> <command>")
            }
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
            "make .tpl and its contents readable by the invoking user, then run the command again",
        ),
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
        Error::AuthenticationRefused { .. } => Cow::Borrowed(
            "correct the credentials with: tpl cfg database update <entry> --user <user>",
        ),
        Error::PropertyNotReadable { .. } => Cow::Borrowed(
            "request read access to the catalogue for this user, then run the command again",
        ),

        // ------------------------------------------------------------ 78 ---
        // FR-PROJ-006 obliges this hint to suggest `tpl init`.
        Error::ProjectNotFound { .. } => Cow::Borrowed("create a project here with: tpl init"),
        Error::ConfigurationNotOwned { .. } => Cow::Borrowed("chown \"$(id -un)\" .tpl/.cfg"),
        // FR-PROJ-011 states this hint itself.
        Error::ConfigurationUnsafeMode { .. } => Cow::Borrowed("chmod 600 .tpl/.cfg"),
        Error::ConfigurationMalformed { .. } => Cow::Borrowed(
            "correct the TOML at the line and column named above, then run the command again",
        ),
        // FR-CONF-034 obliges the nearest-match half over the known keys.
        Error::ConfigurationKeyOutsideSpace { nearest, .. } => {
            let admitted = admitted(nearest, admits_key);
            suggest::hint_line(
                admitted.iter().copied(),
                "remove the key from .tpl/.cfg, or show what tpl cfg set accepts with: \
                 tpl help cfg set",
            )
        }
        Error::ConfigurationValueMalformed { key, .. } => {
            if admits_key(key) {
                Cow::Owned(format!(
                    "write a conforming value with: tpl cfg set {key} <value>"
                ))
            } else {
                Cow::Borrowed(
                    "correct the value at the line and column named above, then run the command \
                     again",
                )
            }
        }
        // Neither line carries an example URL. FR-ERR-022 admits only
        // `[A-Za-z0-9_]` in a value, and FR-SEC-019 builds a runnable hint from
        // literals alone; a worked DSN would put an `@` on the line, which is
        // the character the assertion over every hint looks for to prove that
        // no value reached one ungated. The `cause` carries the grammar, per
        // FR-ERR-034, and the help carries the example.
        Error::DsnMalformed { fault, .. } => match fault {
            DsnFault::Scheme => Cow::Borrowed(
                "write the URL with one of the two schemes tpl accepts, mysql or mariadb",
            ),
            DsnFault::Form => Cow::Borrowed(
                "write the URL with a scheme, a host and a database, and show the form with: \
                 tpl help cfg database add",
            ),
        },
        // FR-CONF-020 makes `$$` the literal dollar, so a value that meant one
        // is corrected by doubling it rather than by closing a brace.
        Error::UnclosedExpansion { .. } => {
            Cow::Borrowed("close the expansion as ${VAR}, or write $$ for a literal dollar sign")
        }
        // FR-CONF-035 obliges the hint to show the array form, and states this
        // example itself.
        Error::PasswordCommandNotAnArray { .. } => Cow::Borrowed(
            "write it as an array: password_command = [\"security\", \"find-generic-password\", \
             \"-s\", \"tpl-shop\", \"-w\"]",
        ),
        Error::ConflictingEntryKeys { .. } => {
            Cow::Borrowed("keep one of the two and remove the other with: tpl cfg unset <key>")
        }
        // FR-CONF-011 points the hint at `tpl cfg database add` with the
        // discrete flags. The DSN itself is barred by BR-ERR-003 and is not
        // carried by the error value at all.
        Error::DsnQueryParameter { .. } => Cow::Borrowed(
            "state each option as its own key: tpl cfg database add <entry> --host <host> \
             --port <port> --user <user>",
        ),
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
            DeadlineBound::Overall => {
                Cow::Borrowed("raise the overall budget with: tpl --timeout <seconds> <command>")
            }
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
        Error::NoDatabaseEntrySelected { .. } => Cow::Borrowed(
            "select an entry with -d, or set a default with: tpl cfg set core.database <entry>",
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
/// of the fifteen forms matching the character set as a whole.
///
/// It is the second of the four spelling tests and lives beside the other
/// three for the reason [`admits_flag`] states.
pub(super) fn admits_key(key: &str) -> bool {
    !key.is_empty() && key.split('.').all(admits)
}

/// Whether every segment of a flag is admitted.
///
/// The `-` or `--` that introduces the flag is a literal, per `FR-ERR-022`, and
/// so is the `-` inside the five flags of this corpus that carry one —
/// `--tpl-dir`, `--no-cache`, `--ca-file`, `--ca-path` and `--password-command`
/// — because a flag is a spelling this specification enumerates.
///
/// This is the third of the three spelling tests, and it lives beside the other
/// two because one rule stated in two places is a rule that drifts: [`hint`]
/// applies it to the pair of a mutually exclusive refusal and
/// [`super::suggest`] applies it to a nearest-match candidate, and both are the
/// same defensive assertion over the same enumerated population.
pub(super) fn admits_flag(flag: &str) -> bool {
    let name = flag
        .strip_prefix("--")
        .or_else(|| flag.strip_prefix('-'))
        .unwrap_or(flag);

    !name.is_empty() && name.split('-').all(admits)
}

/// The subcommand that lists the population a catalogue object was sought in.
const fn listing(kind: CatalogueObjectKind) -> &'static str {
    match kind {
        CatalogueObjectKind::Table => "tables",
        CatalogueObjectKind::View => "views",
        CatalogueObjectKind::Routine => "routines",
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
