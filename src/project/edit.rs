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
//! **The temporary file is created with the mode, and then given it.** The
//! `openat` that creates it carries `0600`, so there is no instant at which the
//! new file exists at a looser mode holding the credentials the old one held;
//! the umask can only narrow that mode, and the `fchmod` on the same
//! descriptor sets it exactly. `FR-CFG-042` then makes the rename the whole of
//! the concurrency story: two processes yield one whole file or the other, and
//! a killed process leaves nothing locked and nothing half-written.
//!
//! **Every step acts in the `.tpl` the trust checks judged** (`FR-CFG-041`,
//! `FR-SEC-027`). The checks of [`super::trust`] open that folder once, judge
//! `.cfg` on a descriptor relative to it and read its text there, and the
//! project keeps the folder's descriptor; [`Editor::new`] is handed both, and
//! the creation of the temporary, its write, its mode, its `fsync`, the rename
//! and the folder's `fsync` are each made relative to that descriptor by
//! [`crate::at`]. Nothing on the rewrite path resolves the folder's path again,
//! so a link or a folder put in place of a component of it after the checks
//! redirects none of those steps.

use std::ffi::OsStr;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use rustix::fd::OwnedFd;
use rustix::fs::Mode;

use toml_edit::{Array, DocumentMut, Item, Table, Value, value};

use super::config::entry::{PasswordCommand, TlsMode};
use super::config::keys::{EntryKey, Key, Target, ValueType};
use crate::error::{Error, Position};
use crate::model::database::Provider;
use crate::render::{RenderFuel, RenderMemoryLimit, RenderOutputLimit};

/// The mode `FR-PROJ-019` creates `.tpl/.cfg` with and `FR-CFG-034` keeps.
pub(crate) const MODE: u32 = 0o600;

/// [`MODE`] as the mode `rustix` takes, whose width differs by platform.
const FILE_MODE: Mode = Mode::RUSR.union(Mode::WUSR);

/// The name of the configuration file inside `.tpl`.
pub(crate) const CONFIGURATION: &str = ".cfg";

/// The section a `[core]` key sits under.
const CORE: &str = "core";

/// The section a `[database.<name>]` block sits under.
const DATABASE: &str = "database";

/// One `.tpl/.cfg` open for writing.
#[derive(Debug)]
pub(crate) struct Editor {
    /// The `.tpl` folder the trust checks judged, the file was read from and
    /// the temporary file is written inside, held open, per `FR-CFG-041`.
    folder: Arc<OwnedFd>,
    /// The file the temporary one is renamed over.
    file: PathBuf,
    /// The document, with its comments and its key order intact.
    document: DocumentMut,
}

impl Editor {
    /// An editor over `text`, the `.cfg` the trust checks read from `folder`,
    /// the `.tpl` folder they judged and hold open; `file` is the path of that
    /// `.cfg`, for a message.
    ///
    /// Nothing is opened or read here: the rewrite starts from the text the
    /// checks judged, and acts in the folder they judged (`FR-CFG-041`). An
    /// absent file is the empty text, which is what lets `tpl cfg set` write
    /// the first key of a project whose file was removed by hand.
    ///
    /// # Errors
    ///
    /// Returns [`Error::ConfigurationMalformed`] where `text` is not TOML —
    /// which the reader of [`super::config`] has already refused for every
    /// command that reaches this one, and which is refused again here rather
    /// than assumed.
    pub(crate) fn new(folder: Arc<OwnedFd>, file: PathBuf, text: &str) -> Result<Self, Error> {
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
            folder,
            file,
            document,
        })
    }

    /// Opens the `.cfg` of the `.tpl` folder `directory`, for the tests of the
    /// write path that start from a folder rather than from a project.
    ///
    /// The production path never comes here: [`crate::project::Project::editor`]
    /// hands [`Editor::new`] the folder and the text the trust checks judged.
    #[cfg(test)]
    pub(crate) fn open(directory: &Path) -> Result<Self, Error> {
        let file = directory.join(CONFIGURATION);
        let unreadable = |returned: std::io::Error| Error::ProjectFileUnreadable {
            path: file.clone(),
            returned,
        };
        let folder = crate::at::directory(directory, directory)
            .map_err(|fault| unreadable(fault.into_io()))?;
        let text = match crate::at::read_checked(&folder, OsStr::new(CONFIGURATION), None) {
            Ok(text) => text,
            Err(crate::at::Unread::Io(returned))
                if returned.kind() == std::io::ErrorKind::NotFound =>
            {
                String::new()
            }
            Err(crate::at::Unread::NotRegular(kind)) => {
                return Err(Error::ConfigurationNotRegular {
                    path: file,
                    kind: crate::at::kind_name(kind),
                });
            }
            Err(crate::at::Unread::Io(returned)) => return Err(unreadable(returned)),
            Err(crate::at::Unread::Oversized) => {
                return Err(unreadable(std::io::Error::from(
                    std::io::ErrorKind::InvalidData,
                )));
            }
        };

        Self::new(Arc::new(folder), file, &text)
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

    /// The name of every database entry the document defines, in the order
    /// the entries appear in the file.
    ///
    /// `toml_edit` keeps the order the file wrote, which the read path's
    /// `NFR-DET-002` order does not; item 6 of `FR-CFG-052` asks for the
    /// file's.
    pub(crate) fn entry_names(&self) -> Vec<String> {
        self.document
            .get(DATABASE)
            .and_then(Item::as_table_like)
            .map(|table| {
                table
                    .iter()
                    .filter(|(_, item)| item.is_table_like())
                    .map(|(name, _)| name.to_owned())
                    .collect()
            })
            .unwrap_or_default()
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
    /// temporary file could not be created, written, given mode `0600` or made
    /// durable, or the rename failed. In every one of those cases the previous
    /// `.cfg` is still in place, unchanged, and no step has acted outside the
    /// folder the trust checks judged. Returns
    /// [`Error::ProjectFileNotDurable`] where the rename took effect and its
    /// durability could not then be established: the file holds the new
    /// content (`FR-CFG-041`, as amended in the eighty-third edition).
    pub(crate) fn save(&self) -> Result<(), Error> {
        let rendered = self.document.to_string();
        let temporary = format!("{CONFIGURATION}.{}.tmp", std::process::id());

        crate::at::replace_in(
            &self.folder,
            OsStr::new(CONFIGURATION),
            OsStr::new(&temporary),
            FILE_MODE,
            rendered.as_bytes(),
        )
        .map_err(|unreplaced| match unreplaced {
            crate::at::Unreplaced::Failed(returned) => Error::ProjectFileUnwritable {
                path: self.file.clone(),
                returned,
            },
            // FR-CFG-041, the eighty-third edition: the rename took effect.
            crate::at::Unreplaced::NotDurable(returned) => Error::ProjectFileNotDurable {
                path: self.file.clone(),
                returned,
            },
        })
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

    if let Key::Entry { entry, field } = key {
        // FR-CONF-057, item 1: a value the login, the connection or the
        // library would cut at a NUL character is refused where the caller
        // wrote it, and nothing is written.
        if let Some(refused) = nul_given(&key.to_string(), "cfg set", entry, *field, supplied) {
            return Err(refused);
        }

        // FR-CONF-049: a value the next expansion would refuse is refused
        // where the caller wrote it. The port keeps its own refusal, since
        // this command takes it as a number only.
        if matches!(
            field,
            EntryKey::Dsn
                | EntryKey::Host
                | EntryKey::User
                | EntryKey::Password
                | EntryKey::Database
        ) && let Some(fault) = super::config::expand::reference_fault(supplied)
        {
            return Err(Error::InvalidReference {
                parameter: key.to_string(),
                command: "cfg set".to_owned(),
                fault,
            });
        }

        // FR-CONF-050: an empty string names no host and no database; and
        // FR-CONF-053 refuses an empty `path` as item 1 of FR-CONF-050
        // refuses an empty host.
        if matches!(field, EntryKey::Host | EntryKey::Database | EntryKey::Path)
            && super::config::keys::is_blank(supplied)
        {
            return Err(Error::EmptyValue {
                parameter: key.to_string(),
                command: "cfg set".to_owned(),
            });
        }
    }

    let item = match expects {
        // FR-CONF-048: core.database names an entry, and ${VAR} is not
        // expanded in it.
        ValueType::EntryName => {
            if !super::config::keys::is_entry_name(supplied) {
                return Err(Error::InvalidEntryName {
                    given: crate::error::EntryNameGiven::CoreDatabase,
                    name: supplied.to_owned(),
                });
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
            // FR-CONF-047: `ca_file` and `ca_path` are read as literal paths,
            // so a `${` is refused rather than stored as a file name.
            if expects == ValueType::Path && (supplied.is_empty() || supplied.contains("${")) {
                return Err(refused());
            }
            value(supplied)
        }
        // FR-CONF-051 item 1: exactly one of the three values, compared exactly.
        ValueType::Provider => {
            let provider = Provider::from_name(supplied).ok_or_else(refused)?;
            value(provider.name())
        }
        // FR-CONF-054: a literal path, stored as written and never resolved;
        // a `${` is refused rather than stored, as for `ca_file`.
        ValueType::DatabasePath => {
            if supplied.contains("${") {
                return Err(refused());
            }
            value(supplied)
        }
    };

    Ok(item)
}

/// The refusal of `FR-CONF-057`, item 1, where `supplied` — given to
/// `parameter` of `command` for `field` of `entry` — holds a NUL character.
///
/// For `dsn` the value is the whole URL, and the part named is the first of
/// its four parts that carries one, as `tpl` reads them; a NUL elsewhere in it
/// is left to the grammar, which refuses what is not a part.
pub(crate) fn nul_given(
    parameter: &str,
    command: &str,
    entry: &str,
    field: EntryKey,
    supplied: &str,
) -> Option<Error> {
    use crate::error::NulField;

    let refused = |field: NulField, in_dsn: bool, before: &str| Error::NulValueGiven {
        parameter: parameter.to_owned(),
        command: command.to_owned(),
        entry: entry.to_owned(),
        field,
        in_dsn,
        before: (field != NulField::Password).then(|| before.to_owned()),
    };
    let before_nul = super::settings::before_nul;

    let field = match field {
        EntryKey::Host => NulField::Host,
        EntryKey::User => NulField::User,
        EntryKey::Password => NulField::Password,
        EntryKey::Database => NulField::Database,
        EntryKey::Path => NulField::Path,
        EntryKey::Dsn => {
            before_nul(supplied)?;
            let parsed = super::config::dsn::parse(supplied, parameter, Path::new("")).ok()?;
            return [
                (NulField::Host, Some(parsed.host())),
                (NulField::User, parsed.user()),
                (NulField::Password, parsed.password()),
                (NulField::Database, Some(parsed.database())),
            ]
            .into_iter()
            .find_map(|(field, part)| {
                part.and_then(|part| before_nul(part.raw()))
                    .map(|before| refused(field, true, before))
            });
        }
        EntryKey::Port
        | EntryKey::PasswordCommand
        | EntryKey::Tls
        | EntryKey::CaFile
        | EntryKey::CaPath
        | EntryKey::Provider => return None,
    };

    before_nul(supplied).map(|before| refused(field, false, before))
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
    fn fr_cfg_034_the_mode_the_rewrite_passes_is_six_hundred() {
        // FR-CFG-034: the mode handed to `fchmod` is the one the file keeps.
        // `mode_t` is `u32` on Linux and `u16` on macOS; widening both sides to
        // `u64` is a real conversion on every target, so no lint fires anywhere.
        assert_eq!(u64::from(super::FILE_MODE.bits()), u64::from(MODE));
    }

    /// An editor over `directory` that sets `core.database` to `shop`.
    fn edited(directory: &std::path::Path) -> Editor {
        let mut editor = Editor::open(directory).expect("the file parses");
        editor.set(
            &key("core.database"),
            assign(&key("core.database"), "shop").expect("an entry name is a string"),
        );

        editor
    }

    #[test]
    fn fr_cfg_041_a_folder_swapped_for_a_link_after_the_open_redirects_nothing() {
        // FR-CFG-041, the paragraph of the seventy-ninth edition: the `.tpl`
        // the file was read from is replaced by a link to another folder
        // between the open and the rewrite. The rewrite lands in the folder it
        // was read from, and the file the link reaches is untouched.
        let scratch = Scratch::new();
        let directory = project(&scratch, "[core]\n");
        let elsewhere = scratch.directory("elsewhere");
        let decoy = scratch.file("elsewhere/.cfg", "decoy\n");

        let editor = edited(&directory);
        let moved = scratch.path("moved");
        std::fs::rename(&directory, &moved).expect("the folder moves");
        scratch.link(&elsewhere, &directory);

        editor
            .save()
            .expect("the rewrite acts in the folder it holds");

        assert_eq!(
            std::fs::read_to_string(&decoy).expect("the decoy is there"),
            "decoy\n"
        );
        let left: Vec<_> = std::fs::read_dir(&elsewhere)
            .expect("the folder is there")
            .filter_map(|found| Some(found.ok()?.file_name()))
            .collect();
        assert_eq!(left, [".cfg"], "nothing was created through the link");
        assert_eq!(
            std::fs::read_to_string(moved.join(".cfg")).expect("the file is there"),
            "[core]\ndatabase = \"shop\"\n"
        );
        assert_eq!(scratch.mode(&moved.join(".cfg")), MODE);
    }

    #[test]
    fn fr_cfg_041_the_rename_replaces_a_link_at_cfg_and_not_the_file_it_reaches() {
        // FR-CFG-041: `.cfg` replaced by a link after the open. The rename
        // replaces the entry named `.cfg`, the link, and the file it pointed
        // at keeps its bytes.
        let scratch = Scratch::new();
        let directory = project(&scratch, "[core]\n");
        let outside = scratch.file("outside", "outside\n");

        let editor = edited(&directory);
        std::fs::remove_file(directory.join(".cfg")).expect("the file is removed");
        scratch.link(&outside, &directory.join(".cfg"));

        editor.save().expect("the rewrite replaces the link");

        assert_eq!(
            std::fs::read_to_string(&outside).expect("the target is there"),
            "outside\n"
        );
        let written = std::fs::symlink_metadata(directory.join(".cfg")).expect("the file is there");
        assert!(written.file_type().is_file(), "the link was not replaced");
    }

    #[test]
    fn fr_cfg_041_a_link_at_the_temporary_name_is_not_opened() {
        // FR-CFG-041: the temporary is created as a new file, so a link
        // standing at its name is never opened or written through.
        let scratch = Scratch::new();
        let directory = project(&scratch, "[core]\n");
        let outside = scratch.file("outside", "outside\n");
        let temporary = directory.join(format!(".cfg.{}.tmp", std::process::id()));
        scratch.link(&outside, &temporary);

        edited(&directory).save().expect("the rewrite succeeds");

        assert_eq!(
            std::fs::read_to_string(&outside).expect("the target is there"),
            "outside\n"
        );
        assert!(std::fs::symlink_metadata(&temporary).is_err());
        assert_eq!(
            std::fs::read_to_string(directory.join(".cfg")).expect("the file is there"),
            "[core]\ndatabase = \"shop\"\n"
        );
    }

    #[test]
    #[allow(
        clippy::print_stderr,
        reason = "a run as the superuser cannot observe the refusal, and says so rather than \
                  passing in silence"
    )]
    fn fr_cfg_041_a_rewrite_that_cannot_act_in_the_folder_is_74_and_leaves_the_file() {
        // FR-CFG-041: a temporary that cannot be created is 74, and the
        // previous `.cfg` is in place, unchanged. A process that bypasses the
        // folder's mode cannot observe the refusal, and says so.
        if rustix::process::getuid().is_root() {
            eprintln!("skipped: the superuser bypasses the folder's mode");
            return;
        }
        let scratch = Scratch::new();
        let directory = project(&scratch, "[core]\n");
        let editor = edited(&directory);
        scratch.chmod(&directory, 0o500);

        let refused = editor.save().expect_err("the folder refuses the temporary");
        scratch.chmod(&directory, 0o700);

        assert!(matches!(refused, Error::ProjectFileUnwritable { .. }));
        assert_eq!(refused.exit_code(), 74);
        assert_eq!(
            std::fs::read_to_string(directory.join(".cfg")).expect("the file is there"),
            "[core]\n"
        );
    }

    #[test]
    fn fr_cfg_041_a_rename_whose_durability_fails_is_74_and_the_new_content_stands() {
        // FR-CFG-041 as amended in the eighty-third edition: the folder's
        // `fsync` fails after the rename, through the seam of `crate::at`, as
        // a device error would. The rewrite exits 74 with its own condition,
        // and the file holds the new content, which is not undone.
        let scratch = Scratch::new();
        let directory = project(&scratch, "[core]\n");
        let editor = edited(&directory);
        crate::at::steps::fail_folder_sync();

        let refused = editor.save().expect_err("the folder's fsync fails");

        assert!(
            matches!(refused, Error::ProjectFileNotDurable { .. }),
            "{refused:?}"
        );
        assert_eq!(refused.exit_code(), 74);
        assert_eq!(
            std::fs::read_to_string(directory.join(".cfg")).expect("the file is there"),
            "[core]\ndatabase = \"shop\"\n"
        );
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
    fn fr_conf_048_core_database_takes_an_entry_name_and_nothing_else() {
        // FR-CONF-048: the empty name, a character outside the set, a name
        // longer than 64 characters, and a ${VAR}, which is not expanded here.
        let long = "a".repeat(65);
        for supplied in ["", "a b", "x.y", "shop-db", long.as_str(), "${DB}"] {
            let condition = assign(&key("core.database"), supplied)
                .expect_err("the value is not an entry name");

            assert!(
                matches!(condition, Error::InvalidEntryName { .. }),
                "{supplied:?}: {condition:?}"
            );
            assert_eq!(condition.exit_code(), 64);
        }
        assign(&key("core.database"), &"a".repeat(64)).expect("64 characters are admitted");
        assign(&key("core.database"), "shop_2").expect("letters, digits and underscores");
    }

    #[test]
    fn fr_conf_049_a_reference_no_expansion_could_satisfy_is_refused_on_write() {
        // FR-CONF-049, in the five fields the command line can write a
        // reference into.
        for (spelling, supplied) in [
            ("database.shop.user", "${1X}"),
            ("database.shop.host", "db-${X.Y}"),
            ("database.shop.password", "${SHOP"),
            ("database.shop.database", "${}"),
            ("database.shop.dsn", "mysql://${1U}@db/shop"),
        ] {
            let condition =
                assign(&key(spelling), supplied).expect_err("the reference is malformed");

            assert!(
                matches!(condition, Error::InvalidReference { .. }),
                "{spelling} = {supplied:?}: {condition:?}"
            );
            assert_eq!(condition.exit_code(), 64);
        }
        assign(&key("database.shop.user"), "${SHOP_USER}").expect("a valid reference");
        assign(&key("database.shop.password"), "$${1X}").expect("a doubled dollar");
        assign(&key("database.shop.password_command"), "echo ${PW}")
            .expect("FR-CONF-017: password_command passes a reference through");
    }

    #[test]
    fn fr_conf_057_a_value_holding_a_nul_is_refused_on_write() {
        // FR-CONF-057, item 1: the five keys of the rule, and each part of a
        // dsn; a key outside the rule is not this condition.
        use crate::error::NulField;

        for (spelling, supplied, field, in_dsn, before) in [
            (
                "database.shop.host",
                "db\0.example.com",
                NulField::Host,
                false,
                Some("db"),
            ),
            (
                "database.shop.user",
                "\0alice",
                NulField::User,
                false,
                Some(""),
            ),
            (
                "database.shop.password",
                "hun\0ter2",
                NulField::Password,
                false,
                None,
            ),
            (
                "database.shop.database",
                "freight\0x",
                NulField::Database,
                false,
                Some("freight"),
            ),
            (
                "database.ledger.path",
                "a\0b.sqlite",
                NulField::Path,
                false,
                Some("a"),
            ),
            (
                "database.shop.dsn",
                "mysql://ali\0ce@db.example.com/freight",
                NulField::User,
                true,
                Some("ali"),
            ),
            (
                "database.shop.dsn",
                "mysql://alice@db\0x/freight",
                NulField::Host,
                true,
                Some("db"),
            ),
        ] {
            let condition = assign(&key(spelling), supplied).expect_err("the value holds a NUL");

            assert_eq!(condition.exit_code(), 64, "{spelling}");
            match condition {
                Error::NulValueGiven {
                    parameter,
                    command,
                    field: refused,
                    in_dsn: part,
                    before: kept,
                    ..
                } => {
                    assert_eq!(parameter, spelling);
                    assert_eq!(command, "cfg set");
                    assert_eq!(refused, field, "{spelling}");
                    assert_eq!(part, in_dsn, "{spelling}");
                    assert_eq!(kept.as_deref(), before, "{spelling}");
                }
                other => panic!("{spelling}: {other:?}"),
            }
        }
    }

    #[test]
    fn fr_conf_050_an_empty_host_or_database_is_refused_on_write() {
        for spelling in ["database.shop.host", "database.shop.database"] {
            let condition = assign(&key(spelling), "").expect_err("the value is empty");

            assert!(
                matches!(condition, Error::EmptyValue { .. }),
                "{spelling}: {condition:?}"
            );
            assert_eq!(condition.exit_code(), 64);
        }
        assign(&key("database.shop.user"), "").expect("FR-CONF-050 names host and database");
    }

    #[test]
    fn fr_cfg_010_a_value_is_validated_against_the_type_the_key_declares() {
        // FR-CFG-010: a value that does not conform is 64.
        for (spelling, supplied) in [
            ("core.connect_timeout", "soon"),
            ("core.connect_timeout", "0"),
            ("core.connect_timeout", "-3"),
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
