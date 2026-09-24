//! The write path: `toml_edit`, a temporary file at mode `0600`, and a rename
//! over the target.
//!
//! Four requirements meet here, and each is a property of how the file is
//! written rather than of what is written into it:
//!
//! | Rule | Requirement |
//! |---|---|
//! | Only `.tpl/.cfg` is written, by any `cfg` subcommand | `FR-CFG-004` |
//! | The rewritten file retains mode `0600` | `FR-CFG-034` |
//! | A temporary file in `.tpl/` at `0600`, renamed over the target; a failure part-way leaves the previous file in place | `FR-CFG-041` |
//! | No lock is taken over the file | `FR-CFG-042` |
//!
//! **Comments and key order survive**, which is why the write path is
//! `toml_edit` and the read path is not: the document is parsed into a tree
//! that remembers its own formatting, one value is replaced, and everything
//! around it is re-emitted byte for byte. A rewrite through `toml` and `serde`
//! would emit a file the serialiser composed, which is a file that has lost
//! every comment its author wrote.
//!
//! **The temporary file is created with the mode rather than chmod'd into it.**
//! `OpenOptions::mode` sets the permissions at creation, so there is no instant
//! at which the new file exists at the umask's mode holding the credentials the
//! old one held. `FR-CFG-042` then makes the rename the whole of the
//! concurrency story: two processes yield one whole file or the other, and a
//! killed process leaves nothing locked and nothing half-written.

use std::fs::OpenOptions;
use std::io::Write as _;
use std::os::unix::fs::OpenOptionsExt as _;
use std::path::{Path, PathBuf};

use toml_edit::{Array, DocumentMut, Item, Table, Value, value};

use super::config::entry::{PasswordCommand, TlsMode};
use super::config::keys::{Key, Target, ValueType};
use crate::error::{Error, Position};
use crate::render::{RenderFuel, RenderMemoryLimit, RenderOutputLimit};

/// The mode `FR-PROJ-019` creates `.tpl/.cfg` with and `FR-CFG-034` keeps.
pub(crate) const MODE: u32 = 0o600;

/// The name of the configuration file inside `.tpl`.
pub(crate) const CONFIGURATION: &str = ".cfg";

/// The section a `[core]` key sits under.
const CORE: &str = "core";

/// The section a `[database.<name>]` block sits under.
const DATABASE: &str = "database";

/// One `.tpl/.cfg` open for writing.
#[derive(Debug)]
pub(crate) struct Editor {
    /// The `.tpl` folder the temporary file is written inside, per
    /// `FR-CFG-041`.
    directory: PathBuf,
    /// The file the temporary one is renamed over.
    file: PathBuf,
    /// The document, with its comments and its key order intact.
    document: DocumentMut,
}

impl Editor {
    /// Opens the `.cfg` of the `.tpl` folder `directory`.
    ///
    /// A file that does not exist opens as an empty document, which is what
    /// lets `tpl cfg set` write the first key of a project whose file was
    /// removed by hand.
    ///
    /// # Errors
    ///
    /// Returns [`Error::ProjectFileUnreadable`] where the file exists and
    /// cannot be read, and [`Error::ConfigurationMalformed`] where it is not
    /// TOML — which the reader of [`super::config`] has already refused for
    /// every command that reaches this one, and which is refused again here
    /// rather than assumed.
    pub(crate) fn open(directory: &Path) -> Result<Self, Error> {
        let file = directory.join(CONFIGURATION);

        let text = match std::fs::read_to_string(&file) {
            Ok(text) => text,
            Err(returned) if returned.kind() == std::io::ErrorKind::NotFound => String::new(),
            Err(returned) => {
                return Err(Error::ProjectFileUnreadable {
                    path: file,
                    returned,
                });
            }
        };

        let document =
            text.parse::<DocumentMut>()
                .map_err(|refused| Error::ConfigurationMalformed {
                    path: file.clone(),
                    position: Position {
                        line: refused.span().map_or(1, |span| {
                            text.get(..span.start)
                                .unwrap_or_default()
                                .bytes()
                                .filter(|byte| *byte == b'\n')
                                .count()
                                + 1
                        }),
                        column: 1,
                    },
                    reason: crate::project::config::parser_reason(refused.message()),
                })?;

        Ok(Self {
            directory: directory.to_owned(),
            file,
            document,
        })
    }

    /// The file this editor writes.
    pub(crate) fn file(&self) -> &Path {
        &self.file
    }

    /// Whether the document defines `name` as a database entry.
    pub(crate) fn defines(&self, name: &str) -> bool {
        self.document
            .get(DATABASE)
            .and_then(Item::as_table)
            .is_some_and(|table| table.contains_key(name))
    }

    /// Writes `item` under `key`, creating the sections it sits in.
    ///
    /// A value that replaces one the file already carried keeps that value's
    /// **decor** — the whitespace before it and the comment after it — so
    /// `query_timeout = 30   # generous` becomes `query_timeout = 60   #
    /// generous` rather than losing the comment its author wrote. That is what
    /// `FR-CFG-041` asks of a rewrite and what the write path is `toml_edit`
    /// for.
    pub(crate) fn set(&mut self, key: &Key, item: Item) {
        let (table, leaf) = match key {
            Key::Core(core) => (
                section(self.document.as_table_mut(), CORE, false),
                core.leaf(),
            ),
            Key::Entry { entry, field } => {
                let databases = section(self.document.as_table_mut(), DATABASE, true);

                (section(databases, entry, false), field.leaf())
            }
        };

        let kept = table
            .get(leaf)
            .and_then(Item::as_value)
            .map(|value| value.decor().clone());

        table[leaf] = item;

        if let Some(decor) = kept
            && let Some(written) = table.get_mut(leaf).and_then(Item::as_value_mut)
        {
            *written.decor_mut() = decor;
        }
    }

    /// Removes what `target` names, and says whether it was there.
    pub(crate) fn remove(&mut self, target: &Target) -> bool {
        let root = self.document.as_table_mut();

        match target {
            Target::Core => root.remove(CORE).is_some(),
            Target::Databases => root.remove(DATABASE).is_some(),
            Target::Entry(name) => root
                .get_mut(DATABASE)
                .and_then(Item::as_table_mut)
                .and_then(|table| table.remove(name))
                .is_some(),
            Target::Key(Key::Core(core)) => root
                .get_mut(CORE)
                .and_then(Item::as_table_mut)
                .and_then(|table| table.remove(core.leaf()))
                .is_some(),
            Target::Key(Key::Entry { entry, field }) => root
                .get_mut(DATABASE)
                .and_then(Item::as_table_mut)
                .and_then(|table| table.get_mut(entry.as_str()))
                .and_then(Item::as_table_mut)
                .and_then(|table| table.remove(field.leaf()))
                .is_some(),
        }
    }

    /// The document as it would be written.
    ///
    /// It exists for the tests that compare a rewrite against the file it was
    /// made from, byte for byte, without a filesystem between the two.
    #[cfg(test)]
    pub(crate) fn rendered(&self) -> String {
        self.document.to_string()
    }

    /// Writes the document, per `FR-CFG-041` and `FR-CFG-034`.
    ///
    /// # Errors
    ///
    /// Returns [`Error::ProjectFileUnwritable`] naming the **target** where the
    /// temporary file could not be created or written, or the rename failed.
    /// In every one of those cases the previous `.cfg` is still in place,
    /// unchanged.
    pub(crate) fn save(&self) -> Result<(), Error> {
        let rendered = self.document.to_string();
        let temporary = self
            .directory
            .join(format!("{CONFIGURATION}.{}.tmp", std::process::id()));

        let unwritable = |returned: std::io::Error| Error::ProjectFileUnwritable {
            path: self.file.clone(),
            returned,
        };

        let written = OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(true)
            .mode(MODE)
            .open(&temporary)
            .and_then(|mut handle| {
                handle.write_all(rendered.as_bytes())?;
                handle.flush()
            });

        if let Err(returned) = written {
            let _ = std::fs::remove_file(&temporary);
            return Err(unwritable(returned));
        }

        // A file that already existed keeps whatever mode it had, because
        // `create` does not apply `mode` to it; FR-CFG-034 requires 0600
        // whatever the previous run left behind.
        if let Err(returned) = std::fs::set_permissions(
            &temporary,
            std::os::unix::fs::PermissionsExt::from_mode(MODE),
        ) {
            let _ = std::fs::remove_file(&temporary);
            return Err(unwritable(returned));
        }

        if let Err(returned) = std::fs::rename(&temporary, &self.file) {
            let _ = std::fs::remove_file(&temporary);
            return Err(unwritable(returned));
        }

        Ok(())
    }
}

/// The table `name` names inside `table`, created where it is absent.
///
/// `implicit` is set on a table that exists only to hold others — `[database]`,
/// whose header a file never writes — so that adding `[database.shop]` does not
/// also add an empty `[database]` above it.
fn section<'a>(table: &'a mut Table, name: &str, implicit: bool) -> &'a mut Table {
    let item = table.entry(name).or_insert_with(|| {
        let mut created = Table::new();
        created.set_implicit(implicit);
        Item::Table(created)
    });

    // A key the file wrote as something other than a table is replaced by one,
    // so that the section a value is written into always exists. The reader of
    // `config` has already refused such a file for every command that reaches
    // this one; the replacement is what keeps the writer total.
    if !item.is_table() {
        *item = Item::Table(Table::new());
    }

    item.as_table_mut()
        .expect("the item at this key was replaced by a table above where it was not one")
}

/// The TOML item `supplied` spells for `key`, validated against its declared
/// type (`FR-CFG-010`).
///
/// # Errors
///
/// Returns [`Error::MalformedValue`] naming the key, the value and the type
/// expected, which `FR-CFG-010` makes a `64`.
pub(crate) fn assign(key: &Key, supplied: &str) -> Result<Item, Error> {
    let expects = key.expects();
    let refused = || Error::MalformedValue {
        parameter: key.to_string(),
        command: "cfg set".to_owned(),
        value: supplied.to_owned(),
        expected: expects.expected(),
    };

    let item = match expects {
        ValueType::EntryName => {
            if supplied.is_empty() {
                return Err(refused());
            }
            value(supplied)
        }
        ValueType::Seconds => {
            let seconds: u32 = supplied.parse().map_err(|_| refused())?;
            if seconds == 0 {
                return Err(refused());
            }
            value(i64::from(seconds))
        }
        // FR-CONF-045, FR-CFG-010: the range the file is held to, refused
        // here with `64` rather than the `78` the same value in the file is.
        ValueType::RenderFuel => {
            let fuel = supplied
                .parse()
                .ok()
                .and_then(RenderFuel::new)
                .ok_or_else(refused)?;
            value(i64::try_from(fuel.get()).map_err(|_| refused())?)
        }
        ValueType::RenderOutputLimit => {
            let limit = supplied
                .parse()
                .ok()
                .and_then(RenderOutputLimit::new)
                .ok_or_else(refused)?;
            value(i64::try_from(limit.get()).map_err(|_| refused())?)
        }
        ValueType::RenderMemoryLimit => {
            let limit = supplied
                .parse()
                .ok()
                .and_then(RenderMemoryLimit::new)
                .ok_or_else(refused)?;
            value(i64::try_from(limit.get()).map_err(|_| refused())?)
        }
        ValueType::Port => {
            let port: u16 = supplied.parse().map_err(|_| refused())?;
            if port == 0 {
                return Err(refused());
            }
            value(i64::from(port))
        }
        ValueType::Tls => {
            let mode = TlsMode::from_name(supplied).ok_or_else(refused)?;
            value(mode.name())
        }
        ValueType::Dsn => {
            // The key is supplied to a command, so a value the grammar refuses
            // is the `64` of FR-CFG-010 rather than the `78` the same text in
            // the file would be, per FR-ERR-035.
            super::config::dsn::parse(supplied, &key.to_string(), Path::new(""))
                .map_err(|_| refused())?;
            value(supplied)
        }
        ValueType::ArgumentArray => {
            // T-03: the cause names what a caller writes — one string — and
            // not the array the file stores, which is what `expected` says.
            let command =
                PasswordCommand::split(supplied).map_err(|fault| Error::MalformedValue {
                    parameter: key.to_string(),
                    command: "cfg set".to_owned(),
                    value: supplied.to_owned(),
                    expected: fault.condition(),
                })?;
            value(array(command.arguments()))
        }
        ValueType::Path | ValueType::Text => {
            if supplied.is_empty() && expects == ValueType::Path {
                return Err(refused());
            }
            value(supplied)
        }
    };

    Ok(item)
}

/// `arguments` as a TOML array.
pub(crate) fn array(arguments: &[String]) -> Value {
    let mut composed = Array::new();
    for argument in arguments {
        composed.push(argument.as_str());
    }

    Value::Array(composed)
}

#[cfg(test)]
mod tests {
    use super::{Editor, MODE, array, assign};
    use crate::error::Error;
    use crate::project::config::keys::{Key, Target};
    use crate::project::scratch::Scratch;

    /// A `.tpl` folder carrying `text` as its `.cfg`.
    fn project(scratch: &Scratch, text: &str) -> std::path::PathBuf {
        let directory = scratch.directory(".tpl");
        let file = scratch.file(".tpl/.cfg", text);
        scratch.chmod(&file, MODE);

        directory
    }

    fn key(spelling: &str) -> Key {
        Key::parse(spelling).expect("the key is in the space")
    }

    #[test]
    fn fr_cfg_041_a_rewrite_preserves_the_comments_and_the_key_order_of_the_file() {
        // FR-CFG-041, and the reason the write path is toml_edit: a file that
        // carried both comes back carrying both, byte for byte outside the one
        // value that changed.
        let scratch = Scratch::new();
        let original = concat!(
            "# the project's own note, which must survive a rewrite\n",
            "[core]\n",
            "# the entry every invocation uses\n",
            "database   = \"shop\"\n",
            "query_timeout = 45   # generous, this catalogue is large\n",
            "\n",
            "[database.shop]\n",
            "host = \"db.example.com\"\n",
            "user = \"alice\"\n",
        );
        let directory = project(&scratch, original);

        let mut editor = Editor::open(&directory).expect("the file parses");
        editor.set(
            &key("core.query_timeout"),
            assign(&key("core.query_timeout"), "60").expect("60 is a positive integer"),
        );
        editor.save().expect("the file is written");

        let rewritten = std::fs::read_to_string(directory.join(".cfg")).expect("the file is there");

        assert_eq!(
            rewritten,
            concat!(
                "# the project's own note, which must survive a rewrite\n",
                "[core]\n",
                "# the entry every invocation uses\n",
                "database   = \"shop\"\n",
                "query_timeout = 60   # generous, this catalogue is large\n",
                "\n",
                "[database.shop]\n",
                "host = \"db.example.com\"\n",
                "user = \"alice\"\n",
            )
        );
    }

    #[test]
    fn fr_cfg_034_a_rewrite_keeps_the_file_at_six_hundred() {
        // FR-CFG-034: FR-PROJ-019 creates it at 0600 and FR-PROJ-011 refuses to
        // read it at any looser mode, so a command that loosened it would break
        // the next invocation.
        let scratch = Scratch::new();
        let directory = project(&scratch, "[core]\n");

        let mut editor = Editor::open(&directory).expect("the file parses");
        editor.set(
            &key("core.database"),
            assign(&key("core.database"), "shop").expect("an entry name is a string"),
        );
        editor.save().expect("the file is written");

        assert_eq!(scratch.mode(&directory.join(".cfg")), MODE);
    }

    #[test]
    fn fr_cfg_034_a_rewrite_of_a_file_left_at_a_looser_mode_tightens_it() {
        // The rename replaces the inode, so the mode that survives is the
        // temporary file's, which FR-CFG-034 fixes at 0600.
        let scratch = Scratch::new();
        let directory = project(&scratch, "[core]\n");
        scratch.chmod(&directory.join(".cfg"), 0o644);

        let mut editor = Editor::open(&directory).expect("the file parses");
        editor.set(
            &key("core.database"),
            assign(&key("core.database"), "shop").expect("an entry name is a string"),
        );
        editor.save().expect("the file is written");

        assert_eq!(scratch.mode(&directory.join(".cfg")), MODE);
    }

    #[test]
    fn fr_cfg_041_no_temporary_file_is_left_behind() {
        // FR-CFG-041: the temporary is renamed over the target, so `.tpl` holds
        // the two artefacts it held before and nothing else.
        let scratch = Scratch::new();
        let directory = project(&scratch, "[core]\n");

        let mut editor = Editor::open(&directory).expect("the file parses");
        editor.set(
            &key("core.database"),
            assign(&key("core.database"), "shop").expect("an entry name is a string"),
        );
        editor.save().expect("the file is written");

        let left: Vec<String> = std::fs::read_dir(&directory)
            .expect("the folder is there")
            .filter_map(|found| Some(found.ok()?.file_name().to_string_lossy().into_owned()))
            .collect();

        assert_eq!(left, [".cfg"]);
    }

    #[test]
    fn fr_cfg_008_a_new_entry_is_written_as_its_own_block() {
        let scratch = Scratch::new();
        let directory = project(&scratch, "[core]\ndatabase = \"shop\"\n");

        let mut editor = Editor::open(&directory).expect("the file parses");
        for (spelling, supplied) in [
            ("database.reporting.host", "10.0.1.5"),
            ("database.reporting.port", "3306"),
            ("database.reporting.tls", "verify-ca"),
        ] {
            let key = key(spelling);
            let item = assign(&key, supplied).expect("the value conforms");
            editor.set(&key, item);
        }

        assert_eq!(
            editor.rendered(),
            concat!(
                "[core]\n",
                "database = \"shop\"\n",
                "\n",
                "[database.reporting]\n",
                "host = \"10.0.1.5\"\n",
                "port = 3306\n",
                "tls = \"verify-ca\"\n",
            )
        );
    }

    #[test]
    fn fr_cfg_011_removing_a_leaf_and_removing_a_block_each_delete_what_they_are_given() {
        // FR-CFG-011.
        let scratch = Scratch::new();
        let directory = project(
            &scratch,
            "[core]\ndatabase = \"shop\"\n\n[database.shop]\nhost = \"a\"\nuser = \"b\"\n\n[database.other]\nhost = \"c\"\n",
        );

        let mut editor = Editor::open(&directory).expect("the file parses");

        assert!(editor.remove(&Target::Key(key("database.shop.user"))));
        assert!(editor.remove(&Target::Entry("other".to_owned())));
        assert!(!editor.remove(&Target::Entry("absent".to_owned())));

        assert_eq!(
            editor.rendered(),
            concat!(
                "[core]\n",
                "database = \"shop\"\n",
                "\n",
                "[database.shop]\n",
                "host = \"a\"\n",
            )
        );
    }

    #[test]
    fn fr_cfg_017_the_editor_reports_which_entries_the_document_defines() {
        // FR-CFG-017 turns on this question.
        let scratch = Scratch::new();
        let directory = project(&scratch, "[database.shop]\nhost = \"a\"\n");
        let editor = Editor::open(&directory).expect("the file parses");

        assert!(editor.defines("shop"));
        assert!(!editor.defines("reporting"));
    }

    #[test]
    fn fr_cfg_008_an_absent_file_opens_as_an_empty_document() {
        let scratch = Scratch::new();
        let directory = scratch.directory(".tpl");

        let mut editor = Editor::open(&directory).expect("an absent file is empty");
        editor.set(
            &key("core.database"),
            assign(&key("core.database"), "shop").expect("an entry name is a string"),
        );
        editor.save().expect("the file is written");

        assert_eq!(
            std::fs::read_to_string(directory.join(".cfg")).expect("the file was created"),
            "[core]\ndatabase = \"shop\"\n"
        );
        assert_eq!(scratch.mode(&directory.join(".cfg")), MODE);
    }

    #[test]
    fn fr_cfg_010_a_value_is_validated_against_the_type_the_key_declares() {
        // FR-CFG-010: a value that does not conform is 64.
        for (spelling, supplied) in [
            ("core.connect_timeout", "soon"),
            ("core.connect_timeout", "0"),
            ("core.connect_timeout", "-3"),
            ("core.database", ""),
            ("database.shop.port", "70000"),
            ("database.shop.port", "0"),
            ("database.shop.tls", "off"),
            ("database.shop.dsn", "postgres://db/shop"),
            ("database.shop.dsn", "mysql://db/shop?tls=false"),
            ("database.shop.password_command", "   "),
            ("database.shop.ca_file", ""),
            ("core.render_fuel", "0"),
            ("core.render_fuel", "1000000000001"),
            ("core.render_fuel", "-1"),
            ("core.render_fuel", "many"),
            ("core.render_output_limit", "0"),
            ("core.render_output_limit", "1099511627777"),
            ("core.render_output_limit", "1.5"),
            ("core.render_memory_limit", "8388607"),
            ("core.render_memory_limit", "0"),
            ("core.render_memory_limit", "1099511627777"),
        ] {
            let condition = assign(&key(spelling), supplied)
                .expect_err("the value does not conform to the declared type");

            assert!(
                matches!(condition, Error::MalformedValue { .. }),
                "{spelling} = {supplied:?}: {condition:?}"
            );
            assert_eq!(condition.exit_code(), 64);
        }
    }

    #[test]
    fn fr_conf_045_tpl_cfg_set_writes_a_render_bound_within_its_range_as_an_integer() {
        for (spelling, supplied, written) in [
            ("core.render_fuel", "1", 1),
            ("core.render_fuel", "1000000000000", 1_000_000_000_000),
            (
                "core.render_output_limit",
                "1099511627776",
                1_099_511_627_776,
            ),
            ("core.render_memory_limit", "8388608", 8_388_608),
        ] {
            let item = assign(&key(spelling), supplied).expect("the value is in range");

            assert_eq!(item.as_integer(), Some(written), "{spelling} = {supplied}");
        }
    }

    #[test]
    fn fr_conf_025_a_password_command_supplied_as_a_string_is_stored_as_the_array_it_splits_into() {
        // FR-CONF-025, FR-CFG-046.
        let scratch = Scratch::new();
        let directory = project(&scratch, "");

        let mut editor = Editor::open(&directory).expect("the file parses");
        let key = key("database.shop.password_command");
        let item = assign(&key, "security find-generic-password -s 'tpl shop' -w")
            .expect("the string splits");
        editor.set(&key, item);

        assert_eq!(
            editor.rendered(),
            concat!(
                "[database.shop]\n",
                "password_command = [\"security\", \"find-generic-password\", \"-s\", \"tpl shop\", \"-w\"]\n",
            )
        );
    }

    #[test]
    fn fr_conf_023_an_argument_array_composes_into_a_toml_array() {
        let arguments = ["pass".to_owned(), "db/shop".to_owned()];

        assert_eq!(array(&arguments).to_string(), r#"["pass", "db/shop"]"#);
    }
}
