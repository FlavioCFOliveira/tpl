//! The entry arm: `tpl cfg database add`, `list`, `show`, `update` and
//! `remove`.
//!
//! The arm exists so that registering a database is one invocation rather than
//! five, and `BR-CFG-001` fixes the division between its two writing verbs:
//! `add` creates and `update` changes, and neither silently does the other's
//! job. There is no `--force` that replaces wholesale and no idempotent `add`
//! that would make the two synonyms, so `add` against a name that exists is
//! `64` and `update` against one that does not is `66`.
//!
//! **Two rules bind the nine flags of `FR-CFG-027` to one another**, and
//! neither is a property of one flag, which is why both are applied here rather
//! than declared on the arguments: `FR-CFG-016` requires `add` to be given
//! either `--dsn` or at least one discrete connection flag, and `FR-CFG-029`
//! makes the two groups mutually exclusive in one invocation.
//!
//! **`show` redacts and `FR-CFG-019` forbids it to expand.** It is the second
//! of the two printers `FR-SEC-003` names, and the redaction is
//! [`redact`](crate::project::config::redact)'s rather than this module's.
//!
//! `tpl cfg database test` is not here. It is the one `cfg` subcommand that
//! contacts a server, per `FR-CFG-005`, and the sprint that opens a connection
//! owns it.

use std::borrow::Cow;
use std::collections::BTreeMap;
use std::io::Write;

use serde::Serialize;
use toml_edit::{Item, value};

use super::super::local::Format;
use super::{Supplied, coherence, form, project};
use crate::error::Error;
use crate::output::{self, Collection, Document, Order, Source, Table};
use crate::project::config::entry::{Entry as Block, PasswordCommand};
use crate::project::config::keys::{EntryKey, Key};
use crate::project::config::redact;
use crate::project::edit::{self, Editor};

/// The command path `FR-CFG-016` refuses an invocation of.
const ADD: &str = "cfg database add";

/// The command path `FR-CFG-048` refuses an invocation of, where the entry it
/// changes cannot hold the fields the flags name beside the fields they leave
/// alone.
const UPDATE: &str = "cfg database update";

/// The flag `FR-CFG-029` makes exclusive of the discrete connection flags.
const DSN: &str = "--dsn";

/// The header of the `text` listing of `tpl cfg database list`.
const NAMES: [&str; 1] = ["NAME"];

/// The headers of the `text` layout of `tpl cfg database show`.
const ENTRY: [&str; 2] = ["KEY", "VALUE"];

/// One member of the `entries` array of `FR-CFG-038`.
#[derive(Debug, Serialize)]
struct Named<'a> {
    /// The entry's name.
    name: &'a str,
}

/// The `data` of `tpl cfg database show` (`FR-CFG-038`, `FR-OUT-031`).
///
/// One key, named for the kind in the singular, whose value is that entry with
/// the redaction of `FR-CFG-021` applied.
#[derive(Debug, Serialize)]
struct Shown<'a> {
    /// The entry.
    entry: BTreeMap<&'a str, Cow<'a, str>>,
}

/// `tpl cfg database add <name>` (`FR-CFG-015` … `FR-CFG-017`).
///
/// It writes no result: `FR-OUT-023` names this command as one whose stdout
/// stays empty.
///
/// # Errors
///
/// Returns what opening the project returns, [`Error::MissingArgument`] where
/// neither `--dsn` nor a discrete connection flag was supplied
/// (`FR-CFG-016`), [`Error::MutuallyExclusiveFlags`] where both groups were
/// (`FR-CFG-029`), [`Error::DatabaseEntryAlreadyExists`] where the name is
/// taken (`FR-CFG-017`), [`Error::IncoherentEntryWrite`] where the entry the
/// flags describe is a combination `FR-CONF-007` refuses (`FR-CFG-048`), and
/// [`Error::ProjectFileUnwritable`] where the rewrite failed.
pub(crate) fn add(supplied: &Supplied<'_>, name: &str, flags: &Flags<'_>) -> Result<(), Error> {
    flags.exclusive()?;

    if !flags.connects() {
        return Err(Error::MissingArgument {
            command: ADD.to_owned(),
            argument: "--dsn, or one of --host, --port, --user and --schema".to_owned(),
        });
    }

    let project = project(supplied)?;
    let configuration = project.configuration()?;
    let mut editor = project.editor()?;

    if editor.defines(name) {
        return Err(Error::DatabaseEntryAlreadyExists {
            name: name.to_owned(),
            file: editor.file().to_owned(),
        });
    }

    // Every value is validated, and the entry the flags would leave behind is
    // then measured against `FR-CONF-007` — both before the editor is asked to
    // hold anything, so a refusal of either leaves the file as it was.
    let written = flags.items()?;
    coherence::refuse(&configuration, ADD, name, &named(&written), flags.dsn)?;

    apply(&mut editor, name, written);

    editor.save()
}

/// `tpl cfg database update <name>` (`FR-CFG-020`).
///
/// Only the fields the flags name are changed; the rest of the entry is left
/// untouched. An invocation that names no flag changes nothing and writes
/// nothing.
///
/// # Errors
///
/// Returns what opening the project returns,
/// [`Error::MutuallyExclusiveFlags`] where both flag groups were supplied
/// (`FR-CFG-029`), [`Error::DatabaseEntryNotFound`] where the entry does not
/// exist, [`Error::IncoherentEntryWrite`] where the fields the flags name
/// cannot stand beside the fields they leave alone (`FR-CFG-048`), and
/// [`Error::ProjectFileUnwritable`] where the rewrite failed.
pub(crate) fn update(supplied: &Supplied<'_>, name: &str, flags: &Flags<'_>) -> Result<(), Error> {
    flags.exclusive()?;

    let project = project(supplied)?;
    let configuration = project.configuration()?;

    if configuration.entry(name).is_none() {
        return Err(configuration.entry_not_found(name));
    }

    let written = flags.items()?;
    if written.is_empty() {
        return Ok(());
    }

    // `FR-CFG-020` leaves the rest of the entry untouched, so the combination
    // measured is the entry as it stands with these fields changed — and
    // `FR-CFG-048` refuses the write rather than removing what it was not asked
    // to remove.
    coherence::refuse(&configuration, UPDATE, name, &named(&written), flags.dsn)?;

    let mut editor = project.editor()?;
    apply(&mut editor, name, written);

    editor.save()
}

/// `tpl cfg database remove <name>` (`FR-CFG-022`, `FR-CFG-023`).
///
/// # Errors
///
/// Returns what opening the project returns,
/// [`Error::DatabaseEntryNotFound`] where the entry does not exist, and
/// [`Error::ProjectFileUnwritable`] where the rewrite failed.
pub(crate) fn remove(supplied: &Supplied<'_>, name: &str) -> Result<(), Error> {
    let project = project(supplied)?;
    let configuration = project.configuration()?;

    if configuration.entry(name).is_none() {
        return Err(configuration.entry_not_found(name));
    }

    let mut editor = project.editor()?;
    editor.remove(&crate::project::config::keys::Target::Entry(
        name.to_owned(),
    ));

    // FR-CFG-023: the reference is cleared silently, so the file stays
    // coherent and the next invocation without -d fails with the correct
    // message rather than with a missing entry.
    if configuration.core().database.as_deref() == Some(name) {
        editor.remove(&crate::project::config::keys::Target::Key(Key::Core(
            crate::project::config::keys::CoreKey::Database,
        )));
    }

    editor.save()
}

/// `tpl cfg database list` (`FR-CFG-018`, `FR-CFG-038`, `FR-CFG-040`).
///
/// # Errors
///
/// Returns what opening the project returns, and the write conditions of
/// [`output`].
pub(crate) fn list<W: Write>(out: &mut W, supplied: &Supplied<'_>) -> Result<(), Error> {
    let configuration = project(supplied)?.configuration()?;
    let names: Vec<&str> = configuration.names().collect();

    match supplied.format() {
        Format::Json => {
            let members: Vec<Named<'_>> = names.iter().map(|name| Named { name }).collect();

            output::emit_to(
                out,
                &Document::new(Source::Project, Collection::new("entries", &members)),
                form(supplied),
            )
        }
        Format::Text => {
            let rows: Vec<[&str; 1]> = names.iter().map(|name| [*name]).collect();

            output::emit_table_to(out, &Table::new(NAMES, &rows, Order::ByName))
        }
    }
}

/// `tpl cfg database show <name>` (`FR-CFG-019`, `FR-CFG-021`, `FR-CFG-038`).
///
/// # Errors
///
/// Returns what opening the project returns,
/// [`Error::DatabaseEntryNotFound`] where the entry does not exist, and the
/// write conditions of [`output`].
pub(crate) fn show<W: Write>(
    out: &mut W,
    supplied: &Supplied<'_>,
    name: &str,
) -> Result<(), Error> {
    let configuration = project(supplied)?.configuration()?;

    let Some(block) = configuration.entry(name) else {
        return Err(configuration.entry_not_found(name));
    };

    let fields = redacted(block);

    match supplied.format() {
        Format::Json => output::emit_to(
            out,
            &Document::new(Source::Project, Shown { entry: fields }),
            form(supplied),
        ),
        Format::Text => {
            let rows: Vec<[Cow<'_, str>; 2]> = fields
                .into_iter()
                .map(|(key, value)| [Cow::Borrowed(key), value])
                .collect();

            output::emit_table_to(out, &Table::new(ENTRY, &rows, Order::ByName))
        }
    }
}

/// The entry's keys and values, with the redaction of `FR-CFG-021` applied and
/// `${VAR}` left exactly as written, per `FR-CFG-019`.
fn redacted(block: &Block) -> BTreeMap<&'static str, Cow<'_, str>> {
    let mut fields = BTreeMap::new();

    for field in EntryKey::ALL {
        let Some(value) = block.written(field) else {
            continue;
        };
        fields.insert(field.leaf(), redact::value(field, value));
    }

    fields
}

/// Writes the flags of `FR-CFG-027` into the entry `name`.
///
/// Every value has already been validated by [`Flags::items`], so nothing here
/// can fail: what reaches the editor is a document the reader will accept.
fn apply(editor: &mut Editor, name: &str, written: Vec<(EntryKey, Item)>) {
    for (field, item) in written {
        editor.set(
            &Key::Entry {
                entry: name.to_owned(),
                field,
            },
            item,
        );
    }
}

/// The keys of what one invocation writes, for the rule of `FR-CFG-048`.
fn named(written: &[(EntryKey, Item)]) -> Vec<EntryKey> {
    written.iter().map(|(field, _)| *field).collect()
}

/// The nine flags of `FR-CFG-027`, reduced to the values one invocation
/// supplied.
///
/// It is the clap struct read once, so that the two rules between the flags —
/// `FR-CFG-016` and `FR-CFG-029` — are asked of one value rather than of nine
/// vectors, and so that the tests of this module can state an invocation
/// without building a parser.
#[derive(Debug, Default)]
pub(crate) struct Flags<'a> {
    /// `--dsn`.
    pub(crate) dsn: Option<&'a str>,
    /// `--host`.
    pub(crate) host: Option<&'a str>,
    /// `--port`.
    pub(crate) port: Option<u16>,
    /// `--user`.
    pub(crate) user: Option<&'a str>,
    /// `--schema`, which writes `database.<name>.database` (`FR-CFG-028`).
    pub(crate) schema: Option<&'a str>,
    /// `--tls`.
    pub(crate) tls: Option<crate::project::config::entry::TlsMode>,
    /// `--password-command`, one string split by `FR-CONF-025`.
    pub(crate) password_command: Option<&'a str>,
    /// `--ca-file`.
    pub(crate) ca_file: Option<&'a std::path::Path>,
    /// `--ca-path`.
    pub(crate) ca_path: Option<&'a std::path::Path>,
}

impl Flags<'_> {
    /// Whether the invocation described a connection at all (`FR-CFG-016`).
    ///
    /// The discrete **connection** flags are the four that map onto the fields
    /// `FR-CONF-006` enumerates and that the command declares; `--tls`,
    /// `--password-command`, `--ca-file` and `--ca-path` are not among them,
    /// because none of them says where to connect.
    const fn connects(&self) -> bool {
        self.dsn.is_some()
            || self.host.is_some()
            || self.port.is_some()
            || self.user.is_some()
            || self.schema.is_some()
    }

    /// Refuses `--dsn` beside a discrete connection flag (`FR-CFG-029`).
    ///
    /// # Errors
    ///
    /// Returns [`Error::MutuallyExclusiveFlags`] naming `--dsn` and the first
    /// discrete flag the invocation also supplied.
    fn exclusive(&self) -> Result<(), Error> {
        if self.dsn.is_none() {
            return Ok(());
        }

        let discrete = [
            ("--host", self.host.is_some()),
            ("--port", self.port.is_some()),
            ("--user", self.user.is_some()),
            ("--schema", self.schema.is_some()),
        ];

        match discrete.into_iter().find(|(_, given)| *given) {
            Some((second, _)) => Err(Error::MutuallyExclusiveFlags {
                first: DSN.to_owned(),
                second: second.to_owned(),
            }),
            None => Ok(()),
        }
    }

    /// The keys this invocation writes, with their values validated.
    ///
    /// Three of the nine can fail — a `--dsn` the grammar refuses, per
    /// `FR-CFG-031`, a `--password-command` that splits into nothing, and a
    /// path that is not valid UTF-8 — and every one of them is a fault in the
    /// invocation. They are resolved together, and before anything is written,
    /// so that a refused value leaves `.tpl/.cfg` exactly as it was.
    ///
    /// # Errors
    ///
    /// Returns [`Error::MalformedValue`] naming the flag and the value.
    fn items(&self) -> Result<Vec<(EntryKey, Item)>, Error> {
        self.assignments()
            .into_iter()
            .map(|(field, item)| item.map(|item| (field, item)))
            .collect()
    }

    /// The keys this invocation writes, in the order of `FR-CONF-002`.
    ///
    /// The item of each is produced lazily, because three of the nine can fail;
    /// [`Flags::items`] is what resolves them.
    fn assignments(&self) -> Vec<(EntryKey, Result<Item, Error>)> {
        let mut written: Vec<(EntryKey, Result<Item, Error>)> = Vec::with_capacity(9);

        if let Some(dsn) = self.dsn {
            written.push((EntryKey::Dsn, dsn_item(dsn)));
        }
        if let Some(host) = self.host {
            written.push((EntryKey::Host, Ok(value(host))));
        }
        if let Some(port) = self.port {
            written.push((EntryKey::Port, Ok(value(i64::from(port)))));
        }
        if let Some(user) = self.user {
            written.push((EntryKey::User, Ok(value(user))));
        }
        if let Some(schema) = self.schema {
            written.push((EntryKey::Database, Ok(value(schema))));
        }
        if let Some(command) = self.password_command {
            written.push((EntryKey::PasswordCommand, command_item(command)));
        }
        if let Some(tls) = self.tls {
            written.push((EntryKey::Tls, Ok(value(tls.name()))));
        }
        if let Some(path) = self.ca_file {
            written.push((EntryKey::CaFile, path_item(EntryKey::CaFile, path)));
        }
        if let Some(path) = self.ca_path {
            written.push((EntryKey::CaPath, path_item(EntryKey::CaPath, path)));
        }

        written
    }
}

/// The `dsn` item a `--dsn` writes, verbatim, once the grammar has accepted it.
///
/// `FR-CFG-031` stores the value **verbatim**, and nothing here rewrites it:
/// the grammar of `FR-CONF-009` through `FR-CONF-012` is applied and the string
/// is then written as it was typed, a literal password included. The check is
/// applied because `FR-CONF-034` and `BR-CONF-004` make the file strict, so a
/// value the file cannot be read with would make every later invocation — the
/// `tpl cfg database remove` that would undo it included — a `78`.
fn dsn_item(written: &str) -> Result<Item, Error> {
    crate::project::config::dsn::parse(written, DSN, std::path::Path::new("")).map_err(|_| {
        Error::MalformedValue {
            parameter: DSN.to_owned(),
            value: written.to_owned(),
            expected: "a connection URL",
        }
    })?;

    Ok(value(written))
}

/// The `password_command` item a `--password-command` writes (`FR-CFG-046`).
fn command_item(written: &str) -> Result<Item, Error> {
    let command = PasswordCommand::split(written).ok_or_else(|| Error::MalformedValue {
        parameter: "--password-command".to_owned(),
        value: written.to_owned(),
        expected: "a command to run",
    })?;

    Ok(value(edit::array(command.arguments())))
}

/// The item a path flag writes.
fn path_item(field: EntryKey, path: &std::path::Path) -> Result<Item, Error> {
    path.to_str()
        .map(value)
        .ok_or_else(|| Error::MalformedValue {
            parameter: format!("--{}", field.leaf().replace('_', "-")),
            value: path.to_string_lossy().into_owned(),
            expected: "a filesystem path",
        })
}

#[cfg(test)]
mod tests {
    use super::super::tests::Harness;
    use super::Flags;
    use crate::error::{EntryRepair, Error};
    use crate::project::config::entry::TlsMode;

    #[test]
    fn fr_cfg_015_add_creates_the_block_from_the_flags_supplied() {
        // FR-CFG-015, FR-CFG-027, FR-CFG-028.
        let harness = Harness::new("[core]\ndatabase = \"shop\"\n");

        harness
            .add(
                "reporting",
                Flags {
                    host: Some("10.0.1.5"),
                    port: Some(3306),
                    user: Some("reader"),
                    schema: Some("reporting"),
                    tls: Some(TlsMode::VerifyIdentity),
                    password_command: Some("security find-generic-password -s tpl -w"),
                    ..Flags::default()
                },
            )
            .expect("the entry is created");

        assert_eq!(
            harness.written(),
            concat!(
                "[core]\n",
                "database = \"shop\"\n",
                "\n",
                "[database.reporting]\n",
                "host = \"10.0.1.5\"\n",
                "port = 3306\n",
                "user = \"reader\"\n",
                "database = \"reporting\"\n",
                "password_command = [\"security\", \"find-generic-password\", \"-s\", \"tpl\", \"-w\"]\n",
                "tls = \"verify-identity\"\n",
            )
        );
    }

    #[test]
    fn fr_out_023_add_writes_nothing_to_stdout() {
        // FR-OUT-023 names this command.
        let harness = Harness::new("");

        assert_eq!(
            harness.add_output(
                "shop",
                Flags {
                    host: Some("db"),
                    ..Flags::default()
                }
            ),
            ""
        );
    }

    #[test]
    fn fr_cfg_016_add_requires_a_dsn_or_a_discrete_connection_flag() {
        // FR-CFG-016: 64 where neither is supplied.
        let harness = Harness::new("");

        for flags in [
            Flags::default(),
            Flags {
                tls: Some(TlsMode::Required),
                ..Flags::default()
            },
            Flags {
                password_command: Some("pass db/shop"),
                ..Flags::default()
            },
        ] {
            let condition = harness
                .add("shop", flags)
                .expect_err("no connection was described");

            assert!(matches!(condition, Error::MissingArgument { .. }));
            assert_eq!(condition.exit_code(), 64);
        }
    }

    #[test]
    fn fr_cfg_029_a_dsn_and_a_discrete_connection_flag_cannot_be_given_together() {
        // FR-CFG-029.
        let harness = Harness::new("");

        let condition = harness
            .add(
                "shop",
                Flags {
                    dsn: Some("mysql://db.example.com/shop"),
                    host: Some("other"),
                    ..Flags::default()
                },
            )
            .expect_err("the two groups are exclusive");

        match condition {
            Error::MutuallyExclusiveFlags {
                ref first,
                ref second,
            } => {
                assert_eq!(first, "--dsn");
                assert_eq!(second, "--host");
            }
            other => panic!("expected exclusive flags, got {other:?}"),
        }
        assert_eq!(condition.exit_code(), 64);
    }

    #[test]
    fn fr_conf_007_a_dsn_composes_with_a_password_command_and_with_the_trust_material() {
        // FR-CONF-007, second row, and FR-CFG-027's last three rows: they are
        // not connection flags, so they are not exclusive of --dsn.
        let harness = Harness::new("");

        harness
            .add(
                "shop",
                Flags {
                    dsn: Some("mysql://alice@db.example.com/shop"),
                    password_command: Some("pass db/shop"),
                    ca_file: Some(std::path::Path::new("/etc/ssl/private.pem")),
                    ..Flags::default()
                },
            )
            .expect("the combination is admitted");

        assert!(harness.written().contains("ca_file"));
    }

    #[test]
    fn fr_cfg_017_add_refuses_a_name_that_is_already_taken() {
        // FR-CFG-017, BR-CFG-001: add creates and update changes.
        let harness = Harness::new("[database.shop]\nhost = \"db\"\n");

        let condition = harness
            .add(
                "shop",
                Flags {
                    host: Some("other"),
                    ..Flags::default()
                },
            )
            .expect_err("the name is taken");

        assert!(matches!(
            condition,
            Error::DatabaseEntryAlreadyExists { .. }
        ));
        assert_eq!(condition.exit_code(), 64);
        assert!(harness.written().contains("host = \"db\""));
    }

    #[test]
    fn br_conf_004_a_dsn_the_grammar_refuses_is_not_written_into_the_file() {
        // BR-CONF-004 makes the file strict in both directions, and a value the
        // reader would refuse would make every later invocation a 78.
        let harness = Harness::new("");

        let condition = harness
            .add(
                "shop",
                Flags {
                    dsn: Some("postgres://db.example.com/shop"),
                    ..Flags::default()
                },
            )
            .expect_err("the scheme is not one of the two");

        assert!(matches!(condition, Error::MalformedValue { .. }));
        assert_eq!(condition.exit_code(), 64);
        assert_eq!(harness.written(), "");
    }

    #[test]
    fn fr_cfg_020_update_changes_the_fields_the_flags_name_and_leaves_the_rest() {
        // FR-CFG-020.
        let harness =
            Harness::new("[database.shop]\nhost = \"db\"\nuser = \"alice\"\ntls = \"required\"\n");

        harness
            .update(
                "shop",
                Flags {
                    user: Some("bob"),
                    ..Flags::default()
                },
            )
            .expect("the entry is changed");

        assert_eq!(
            harness.written(),
            "[database.shop]\nhost = \"db\"\nuser = \"bob\"\ntls = \"required\"\n"
        );
    }

    #[test]
    fn br_cfg_001_update_refuses_an_entry_that_does_not_exist() {
        // BR-CFG-001: update changes, and there is nothing to change.
        let harness = Harness::new("[database.shop]\nhost = \"db\"\n");

        let condition = harness
            .update(
                "shup",
                Flags {
                    host: Some("other"),
                    ..Flags::default()
                },
            )
            .expect_err("the entry does not exist");

        assert!(matches!(condition, Error::DatabaseEntryNotFound { .. }));
        assert_eq!(condition.exit_code(), 66);
    }

    #[test]
    fn fr_cfg_022_remove_deletes_the_entry() {
        // FR-CFG-022.
        let harness =
            Harness::new("[database.shop]\nhost = \"a\"\n\n[database.other]\nhost = \"b\"\n");

        harness.remove("shop").expect("the entry is deleted");

        // The blank line that separated the two blocks belonged to the second
        // of them, so removing the first leaves it where the author put it.
        assert_eq!(harness.written(), "\n[database.other]\nhost = \"b\"\n");
    }

    #[test]
    fn fr_cfg_023_remove_clears_the_reference_the_core_section_held_to_it() {
        // FR-CFG-023: silently, leaving the file coherent.
        let harness = Harness::new(
            "[core]\ndatabase = \"shop\"\nquery_timeout = 45\n\n[database.shop]\nhost = \"a\"\n",
        );

        harness.remove("shop").expect("the entry is deleted");

        assert_eq!(harness.written(), "[core]\nquery_timeout = 45\n");
    }

    #[test]
    fn fr_cfg_023_remove_leaves_a_reference_to_another_entry_alone() {
        let harness = Harness::new(
            "[core]\ndatabase = \"other\"\n\n[database.shop]\nhost = \"a\"\n\n[database.other]\nhost = \"b\"\n",
        );

        harness.remove("shop").expect("the entry is deleted");

        assert!(harness.written().contains("database = \"other\""));
    }

    #[test]
    fn br_cfg_001_remove_refuses_an_entry_that_does_not_exist() {
        let harness = Harness::new("[database.shop]\nhost = \"a\"\n");

        assert_eq!(
            harness
                .remove("shup")
                .expect_err("it is not there")
                .exit_code(),
            66
        );
    }

    #[test]
    fn fr_cfg_018_list_prints_the_names_of_the_entries_the_file_defines() {
        // FR-CFG-018, FR-CFG-038, NFR-DET-002.
        let harness =
            Harness::new("[database.shop]\nhost = \"a\"\n\n[database.archive]\nhost = \"b\"\n");

        assert_eq!(harness.database_list(), "NAME\narchive\nshop\n");
        assert_eq!(
            harness.database_list_json(),
            "{\"schema_version\":1,\"source\":\"project\",\"data\":{\"entries\":[{\"name\":\"archive\"},{\"name\":\"shop\"}]}}\n"
        );
    }

    #[test]
    fn fr_cfg_040_an_empty_listing_is_a_success_carrying_its_header_and_an_empty_array() {
        // FR-CFG-040, FR-OUT-033 … FR-OUT-035, FR-PROJ-018.
        let harness = Harness::new("[core]\n");

        assert_eq!(harness.database_list(), "NAME\n");
        assert_eq!(
            harness.database_list_json(),
            "{\"schema_version\":1,\"source\":\"project\",\"data\":{\"entries\":[]}}\n"
        );
    }

    #[test]
    fn fr_cfg_019_show_redacts_the_secret_and_leaves_a_reference_as_written() {
        // FR-CFG-019, FR-CFG-021, FR-SEC-003.
        let harness = Harness::new("[database.shop]\nhost = \"db\"\npassword = \"hunter2\"\n");

        let printed = harness.show("shop");
        assert!(!printed.contains("hunter2"), "{printed}");
        assert!(printed.contains("***"), "{printed}");

        let document = harness.show_json("shop");
        assert_eq!(
            document,
            "{\"schema_version\":1,\"source\":\"project\",\"data\":{\"entry\":{\"host\":\"db\",\"password\":\"***\"}}}\n"
        );
    }

    #[test]
    fn fr_cfg_019_show_does_not_expand_a_reference() {
        // FR-CFG-019.
        let harness = Harness::new(
            "[database.shop]\nhost = \"${SHOP_DB_HOST}\"\npassword = \"${SHOP_DB_PASSWORD}\"\n",
        );

        let document = harness.show_json("shop");

        assert!(document.contains("${SHOP_DB_HOST}"), "{document}");
        assert!(document.contains("${SHOP_DB_PASSWORD}"), "{document}");
    }

    #[test]
    fn br_cfg_001_show_refuses_an_entry_that_does_not_exist() {
        let harness = Harness::new("[database.shop]\nhost = \"a\"\n");

        assert_eq!(
            harness
                .show_refused("shup")
                .expect_err("it is not there")
                .exit_code(),
            66
        );
    }

    #[test]
    fn fr_out_006_the_text_layout_of_an_entry_is_aligned_columns_under_a_header_row() {
        // FR-OUT-006.
        let harness = Harness::new("[database.shop]\nhost = \"db.example.com\"\nport = 3306\n");

        assert_eq!(
            harness.show("shop"),
            concat!("KEY   VALUE\n", "host  db.example.com\n", "port  3306\n",)
        );
    }

    #[test]
    fn fr_cfg_031_the_dsn_flag_admits_exactly_what_the_file_admits_and_leaves_a_reference_alone() {
        // FR-CFG-031: the flag admits the values FR-CONF-009, FR-CONF-010 and
        // FR-CONF-011 admit, and no others. A ${VAR} is opaque text inside the
        // field it occupies, so the value below is admitted and stored
        // verbatim; no cfg command reads the environment, and this one does not
        // define the variable.
        let harness = Harness::new("");
        let written = "mysql://alice:${SHOP_DB_PASSWORD}@db.example.com:3306/shop";

        harness
            .add(
                "shop",
                Flags {
                    dsn: Some(written),
                    ..Flags::default()
                },
            )
            .expect("the reference is opaque text within the password field");

        assert_eq!(
            harness.written(),
            format!("[database.shop]\ndsn = \"{written}\"\n")
        );
        assert!(
            std::env::var_os("SHOP_DB_PASSWORD").is_none(),
            "the test would prove nothing if the variable were defined"
        );
    }

    #[test]
    fn fr_cfg_031_a_dsn_outside_the_three_requirements_is_sixty_four_and_writes_nothing() {
        // FR-CFG-031: the refusal is 64, the invocation being at fault and not
        // the file, and nothing is written — including no temporary file left
        // behind under FR-CFG-041.
        for value in [
            "postgres://db.example.com/shop",
            "mysql://db.example.com/shop?charset=utf8",
            "mysql://db.example.com",
            "db.example.com/shop",
        ] {
            let harness = Harness::new("[core]\n");

            let condition = harness
                .add(
                    "shop",
                    Flags {
                        dsn: Some(value),
                        ..Flags::default()
                    },
                )
                .expect_err("the value is outside what the file admits");

            assert_eq!(condition.exit_code(), 64, "{value}");
            assert_eq!(harness.written(), "[core]\n", "{value}");
            assert_eq!(
                std::fs::read_dir(harness.tpl())
                    .expect("the folder is there")
                    .count(),
                1,
                "{value} left a second file in .tpl"
            );
        }
    }

    #[test]
    fn fr_cfg_048_add_refuses_a_dsn_carrying_a_password_beside_a_password_command() {
        // FR-CFG-048, the third row of FR-CONF-007: FR-CFG-029 does not
        // separate this pair, because FR-CONF-006 excludes password_command
        // from the discrete connection fields.
        let harness = Harness::new("[core]\n");

        let condition = harness
            .add(
                "shop",
                Flags {
                    dsn: Some("mysql://alice:hunter2@db.example.com/shop"),
                    password_command: Some("pass db/shop"),
                    ..Flags::default()
                },
            )
            .expect_err("two password sources in one entry");

        assert_eq!(condition.exit_code(), 64);
        assert_eq!(harness.written(), "[core]\n");

        match condition {
            Error::IncoherentEntryWrite {
                ref written,
                ref conflicting,
                ref repair,
                ..
            } => {
                assert_eq!(written, "database.shop.dsn");
                assert_eq!(conflicting, "database.shop.password_command");
                assert_eq!(*repair, EntryRepair::Restate("cfg database add".to_owned()));
            }
            other => panic!("expected an incoherent write, got {other:?}"),
        }
    }

    #[test]
    fn fr_cfg_048_update_refuses_a_field_that_cannot_stand_beside_one_it_leaves_alone() {
        // FR-CFG-048 with FR-CFG-020: the rest of the entry is left in place,
        // so the write that would contradict it is refused rather than made
        // coherent by removing what the invocation did not name.
        let harness = Harness::new("[database.shop]\ndsn = \"mysql://alice@db/shop\"\n");

        let condition = harness
            .update(
                "shop",
                Flags {
                    host: Some("10.0.1.5"),
                    ..Flags::default()
                },
            )
            .expect_err("the entry already states its connection as a URL");

        assert_eq!(condition.exit_code(), 64);
        assert_eq!(
            harness.written(),
            "[database.shop]\ndsn = \"mysql://alice@db/shop\"\n"
        );

        match condition {
            Error::IncoherentEntryWrite {
                ref written,
                ref conflicting,
                ref repair,
                ..
            } => {
                assert_eq!(written, "database.shop.host");
                assert_eq!(conflicting, "database.shop.dsn");
                assert_eq!(*repair, EntryRepair::Unset);
            }
            other => panic!("expected an incoherent write, got {other:?}"),
        }
    }

    #[test]
    fn fr_cfg_048_an_entry_with_more_than_one_conflicting_field_is_repaired_by_a_rewrite() {
        // FR-CFG-048: the hint carries a command that makes the write legal,
        // and no single `tpl cfg unset` does where three discrete fields stand
        // against the DSN.
        let harness = Harness::new(
            "[database.shop]\nhost = \"db\"\nuser = \"alice\"\npassword = \"hunter2\"\n",
        );

        let condition = harness
            .update(
                "shop",
                Flags {
                    dsn: Some("mysql://alice@db/shop"),
                    ..Flags::default()
                },
            )
            .expect_err("three discrete fields stand against the URL");

        match condition {
            Error::IncoherentEntryWrite { ref repair, .. } => {
                assert_eq!(*repair, EntryRepair::Rewrite);
            }
            other => panic!("expected an incoherent write, got {other:?}"),
        }
    }

    #[test]
    fn fr_conf_007_the_two_admitted_rows_of_the_table_are_written_without_complaint() {
        // FR-CONF-007 admits password_command beside either way of describing a
        // connection, and FR-CFG-048 refuses only what that table refuses.
        let discrete = Harness::new("");
        discrete
            .add(
                "reporting",
                Flags {
                    host: Some("10.0.1.5"),
                    password_command: Some("pass db/reporting"),
                    ..Flags::default()
                },
            )
            .expect("discrete fields and a password command are admitted");

        let url = Harness::new("");
        url.add(
            "shop",
            Flags {
                dsn: Some("mysql://alice@db.example.com/shop"),
                password_command: Some("pass db/shop"),
                ..Flags::default()
            },
        )
        .expect("a DSN carrying no password and a password command are admitted");

        assert!(
            url.written().contains("password_command"),
            "{}",
            url.written()
        );
    }
}
