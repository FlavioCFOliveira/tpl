//! `.tpl/.cfg`: the only configuration `tpl` reads, as a typed document.
//!
//! `FR-CONF-001` makes the file TOML and `BR-CONF-004` makes it **strict in
//! both directions**: it is untrusted input that decides which host is
//! contacted, which credential is used and which child process is executed, so
//! a reader that accepted what it did not understand, or repaired what was
//! written incorrectly, would be guessing at those three. [`load`] therefore
//! refuses rather than guesses, and what it returns is a document in which
//! every value is already the shape the rest of the crate expects.
//!
//! | Step | What it refuses | Requirement |
//! |---|---|---|
//! | Parse | TOML the parser rejects | The `78` row of `FR-ERR-001` |
//! | Key space | a key the table of `FR-CONF-002` does not list, anywhere in the file | `FR-CONF-034` |
//! | Types | a value that is not of the declared type, and a `password_command` that is not an array | `FR-CONF-002`, `FR-CONF-035` |
//! | Coherence | an entry that describes its connection two ways, or its password two ways | `FR-CONF-006`, `FR-CONF-007` |
//! | Grammar | a DSN outside the form, the two schemes, or the no-parameter rule | `FR-CONF-009` … `FR-CONF-012` |
//!
//! The steps run in that order over the whole file, so a file carrying both a
//! misspelled key and a malformed value reports the misspelling — which is the
//! likelier cause of the other.
//!
//! **Nothing here reads the environment and nothing here runs a child.**
//! `FR-CFG-014` forbids `tpl cfg list` to resolve the configuration, and the
//! way to hold that is for the reader not to be able to: `${VAR}` expansion
//! lives in [`expand`] and `password_command` execution in
//! [`super::password`], and both are reached by [`super::settings`], which is
//! the resolution step and not this one.
//!
//! The file's **text** is kept beside the document, because `FR-CFG-013` prints
//! it literally and `FR-CFG-021` redacts it in place; [`redact`] owns that, and
//! the spans this reader records are what let it splice a value without
//! reformatting the file around it.

pub(crate) mod dsn;
pub(crate) mod entry;
pub(crate) mod expand;
pub(crate) mod keys;
pub(crate) mod redact;

use std::collections::BTreeMap;
use std::fmt;
use std::io;
use std::num::NonZeroU64;
use std::ops::Range;
use std::path::{Path, PathBuf};

use toml::Spanned;
use toml::de::{DeTable, DeValue};

use crate::deadline::Seconds;
use crate::diagnostics::suggest::{self, Population};
use crate::error::{Error, Position};
use crate::project::secret::Redacted;
use crate::render::{RenderFuel, RenderMemoryLimit, RenderOutputLimit};

use entry::{Combination, Entry, Located, PasswordCommand, PortSetting, TlsMode};
use keys::{CoreKey, EntryKey, Key, ValueType};

/// The section a `[core]` key sits under, as the file spells it.
const CORE: &str = "core";

/// The section a `[database.<name>]` block sits under, as the file spells it.
const DATABASE: &str = "database";

/// What a section is expected to be, where the file wrote something else.
const A_TABLE: &str = "a table";

/// The `[core]` section, with every value already typed.
///
/// A key the file does not carry is [`None`] rather than its built-in default:
/// `FR-CFG-014` forbids applying a default when printing, and `FR-CONF-004`
/// applies them where the deadlines are resolved. The two would disagree if the
/// document carried them.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub(crate) struct Core {
    /// `core.database`, the entry an invocation uses when `-d/--database` is
    /// absent (`FR-GLOB-005`).
    pub(crate) database: Option<String>,
    /// `core.connect_timeout`.
    pub(crate) connect_timeout: Option<Seconds>,
    /// `core.query_timeout`.
    pub(crate) query_timeout: Option<Seconds>,
    /// `core.password_timeout`.
    pub(crate) password_timeout: Option<Seconds>,
    /// `core.render_timeout`.
    pub(crate) render_timeout: Option<Seconds>,
    /// `core.render_fuel` (`FR-CONF-045`).
    pub(crate) render_fuel: Option<RenderFuel>,
    /// `core.render_output_limit` (`FR-CONF-045`).
    pub(crate) render_output_limit: Option<RenderOutputLimit>,
    /// `core.render_memory_limit` (`FR-CONF-045`).
    pub(crate) render_memory_limit: Option<RenderMemoryLimit>,
}

impl Core {
    /// The value of `key` as the file wrote it.
    pub(crate) fn written(&self, key: CoreKey) -> Option<entry::Written<'_>> {
        let seconds =
            |value: Option<Seconds>| value.map(|value| entry::Written::Number(value.get()));

        match key {
            CoreKey::Database => self.database.as_deref().map(entry::Written::Text),
            CoreKey::ConnectTimeout => seconds(self.connect_timeout),
            CoreKey::PasswordTimeout => seconds(self.password_timeout),
            CoreKey::QueryTimeout => seconds(self.query_timeout),
            CoreKey::RenderTimeout => seconds(self.render_timeout),
            CoreKey::RenderFuel => self
                .render_fuel
                .map(|value| entry::Written::Number(value.get())),
            CoreKey::RenderOutputLimit => self
                .render_output_limit
                .map(|value| entry::Written::Number(value.get())),
            CoreKey::RenderMemoryLimit => self
                .render_memory_limit
                .map(|value| entry::Written::Number(value.get())),
        }
    }
}

/// One `.tpl/.cfg`, read, validated, and not resolved.
///
/// Its [`Debug`](fmt::Debug) is hand-written and its
/// [`Configuration::text`] field is the reason. That field is the file's own
/// bytes, credential included, so a derived implementation printed the password
/// **twice** — once raw and once through the entry — beside a
/// [`Configuration::redactions`] list that describes the redaction without
/// applying it. `FR-ERR-013` and `BR-ERR-003` bar a credential from every
/// message, and the way to hold a prohibition on printing is to deny the value
/// a printing implementation, exactly as [`Secret`](crate::project::secret)
/// does for the credential it owns.
#[derive(Clone)]
pub(crate) struct Configuration {
    /// The file it was read from, which every condition it raises names.
    file: PathBuf,
    /// The file's own bytes, kept for `FR-CFG-013`.
    text: String,
    /// The `[core]` section.
    core: Core,
    /// The `[database.<name>]` blocks, by entry name.
    ///
    /// A [`BTreeMap`] rather than a hash map: `NFR-DET-002` orders every
    /// collection by name, ascending, byte by byte, and this is the collection
    /// `tpl cfg database list` presents.
    entries: BTreeMap<String, Entry>,
    /// The spans of [`Configuration::text`] that carry a credential, and what
    /// `FR-CFG-021` prints in their place.
    redactions: Vec<redact::Redaction>,
}

impl fmt::Debug for Configuration {
    /// Writes every member but the file's own bytes, which are replaced by
    /// [`Redacted`].
    ///
    /// The span logic of [`redact`] reads the raw text and is the only thing
    /// that does: it is a free function taking `&str`, called by the reader
    /// with the bytes it has just validated and by
    /// [`Configuration::printed`] with this field, and neither route hands the
    /// value to a caller. There is no accessor for it, which is what keeps the
    /// set of things that can reach it at those two.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Configuration")
            .field("file", &self.file)
            .field("text", &Redacted)
            .field("core", &self.core)
            .field("entries", &self.entries)
            .field("redactions", &self.redactions)
            .finish()
    }
}

impl Configuration {
    /// The file this document was read from.
    pub(crate) fn file(&self) -> &Path {
        &self.file
    }

    /// The file as `FR-CFG-013` prints it: literally, with passwords redacted
    /// per `FR-CFG-021`.
    pub(crate) fn printed(&self) -> String {
        redact::document(&self.text, &self.redactions)
    }

    /// The `[core]` section.
    pub(crate) const fn core(&self) -> &Core {
        &self.core
    }

    /// One entry, by name.
    pub(crate) fn entry(&self, name: &str) -> Option<&Entry> {
        self.entries.get(name)
    }

    /// Every entry, in the order `NFR-DET-002` fixes.
    pub(crate) fn entries(&self) -> impl ExactSizeIterator<Item = (&str, &Entry)> {
        self.entries
            .iter()
            .map(|(name, entry)| (name.as_str(), entry))
    }

    /// Every entry name, in the order `NFR-DET-002` fixes.
    pub(crate) fn names(&self) -> impl ExactSizeIterator<Item = &str> + Clone {
        self.entries.keys().map(String::as_str)
    }

    /// The value of `key`, as the file wrote it.
    pub(crate) fn written(&self, key: &Key) -> Option<entry::Written<'_>> {
        match key {
            Key::Core(core) => self.core.written(*core),
            Key::Entry { entry, field } => self.entry(entry)?.written(*field),
        }
    }

    /// Every key the file **sets**, fully qualified, in a fixed order.
    ///
    /// `FR-CFG-007` suggests over the enumerated space, and this is what says
    /// which of its candidates the file does not set.
    pub(crate) fn keys(&self) -> Vec<String> {
        let mut present: Vec<String> = CoreKey::ALL
            .into_iter()
            .filter(|key| self.core.written(*key).is_some())
            .map(|key| key.to_string())
            .collect();

        for (name, entry) in self.entries() {
            for field in EntryKey::ALL {
                if entry.declares(field) {
                    present.push(format!("{DATABASE}.{name}.{field}"));
                }
            }
        }

        present
    }

    /// The nearest matches to `supplied` among the enumerated key space
    /// (`FR-CFG-009`, `FR-CONF-034`).
    pub(crate) fn nearest_key_in_space(&self, supplied: &str) -> Vec<String> {
        let names: Vec<&str> = self.names().collect();
        let population = keys::candidates(names, supplied);
        let suggested = nearest(supplied, &population, Population::ConfigurationKeys);

        misplaced(supplied, &population, suggested)
    }

    /// The nearest matches to `supplied` among the entry names the file defines
    /// (`FR-GLOB-007`).
    pub(crate) fn nearest_entry(&self, supplied: &str) -> Vec<String> {
        let names: Vec<String> = self.names().map(str::to_owned).collect();
        nearest(supplied, &names, Population::Names)
    }

    /// The condition `FR-CFG-007` and `FR-CFG-012` raise for a key or block
    /// this file does not set.
    ///
    /// The population is the key space of `FR-CONF-002` with the `<name>`
    /// segment bound to every entry the file declares (`FR-CFG-007`). A key of
    /// that population exists, so `FR-ERR-019` offers it no suggestion: the
    /// caller spelt it right and the file does not set it, which the `error`
    /// line says with the key's default (finding U-01 of the fourth re-audit of
    /// rmp `#263`). Only a name outside the population is offered candidates,
    /// and each candidate the file does not set is recorded as such, because
    /// the `hint` must say so: a `tpl cfg get` or a `tpl cfg unset` of that
    /// candidate is itself a `66`.
    ///
    /// A key or block that names an entry the file does not declare records
    /// the entry, so that the diagnostic reports the missing entry rather than
    /// a missing key (finding U-05).
    pub(crate) fn key_not_found(&self, key: &str) -> Error {
        let parsed = keys::Key::parse(key);
        let named = match (&parsed, keys::Target::parse(key)) {
            (Some(keys::Key::Entry { entry, .. }), _) => Some(entry.clone()),
            (_, Some(keys::Target::Entry(entry))) => Some(entry),
            _ => None,
        };
        let missing_entry = named.filter(|entry| self.entry(entry).is_none());

        let set = self.keys();
        let unset_or_not = |candidate: String| {
            let carried = set.contains(&candidate);
            (candidate, carried)
        };
        let section = matches!(
            keys::Target::parse(key),
            Some(keys::Target::Core | keys::Target::Databases)
        );
        let nearest = match (&parsed, &missing_entry) {
            // FR-ERR-019: the name exists in the population; nothing is
            // offered. A section of the space exists too.
            (Some(_), None) => Vec::new(),
            _ if section => Vec::new(),
            // A block of an entry the file does not declare: the blocks it
            // does, each of which the file carries.
            (None, Some(entry)) => self
                .nearest_entry(entry)
                .into_iter()
                .map(|name| (format!("{DATABASE}.{name}"), true))
                .collect(),
            // A key of an undeclared entry: the declared population alone,
            // without the same-leaf fallback, which would offer another
            // entry's key for no likeness of name.
            (Some(_), Some(_)) => {
                let population = keys::space(self.names());
                nearest(key, &population, Population::ConfigurationKeys)
                    .into_iter()
                    .map(unset_or_not)
                    .collect()
            }
            (None, None) => {
                let population = keys::space(self.names());
                let suggested = nearest(key, &population, Population::ConfigurationKeys);
                misplaced(key, &population, suggested)
                    .into_iter()
                    .map(unset_or_not)
                    .collect()
            }
        };
        Error::ConfigurationKeyNotFound {
            key: key.to_owned(),
            known: parsed.is_some(),
            default: parsed.as_ref().and_then(keys::Key::default_value),
            file: self.file.clone(),
            entry_missing: missing_entry.is_some(),
            nearest,
        }
    }

    /// The condition `FR-GLOB-007` raises for an entry this file does not
    /// define; `by_default` where `core.database` is what named it.
    pub(crate) fn entry_not_found(&self, name: &str, by_default: bool) -> Error {
        Error::DatabaseEntryNotFound {
            name: name.to_owned(),
            file: self.file.clone(),
            nearest: self.nearest_entry(name),
            by_default,
        }
    }
}

/// The nearest matches to `supplied` among `population`, per `FR-ERR-019`.
fn nearest(supplied: &str, population: &[String], class: Population) -> Vec<String> {
    suggest::suggestions(supplied, population.iter().map(String::as_str), class)
        .names()
        .map(str::to_owned)
        .collect()
}

/// Reads and validates `file`.
///
/// A file that does not exist is an **empty document** rather than a failure.
/// `FR-PROJ-001` makes the project the `.tpl` folder and `FR-PROJ-017` has
/// `tpl init` create the `.cfg` inside it, so a project without one is a
/// project whose file was removed by hand; reading it as empty is what keeps
/// `tpl cfg set` able to write it again, and there is nothing in an absent file
/// to trust, to misread, or to disagree with.
///
/// # Errors
///
/// Returns [`Error::ProjectFileUnreadable`] where the file exists and cannot be
/// read, and the `78` condition of the step that refuses: the parse, the key
/// space of `FR-CONF-034`, the declared types of `FR-CONF-002`, the coherence
/// of `FR-CONF-007`, or the DSN grammar of `FR-CONF-009` through
/// `FR-CONF-012`.
pub(crate) fn load(file: &Path) -> Result<Configuration, Error> {
    let text = match std::fs::read_to_string(file) {
        Ok(text) => text,
        Err(returned) if returned.kind() == io::ErrorKind::NotFound => String::new(),
        Err(returned) => {
            return Err(Error::ProjectFileUnreadable {
                path: file.to_owned(),
                returned,
            });
        }
    };

    let document = read(&text, file)?;

    Ok(Configuration {
        file: file.to_owned(),
        text,
        core: document.core,
        entries: document.entries,
        redactions: document.redactions,
    })
}

/// What one validated `.tpl/.cfg` yields, before it is joined to its path.
#[derive(Debug, Clone)]
struct Document {
    /// The `[core]` section.
    core: Core,
    /// The `[database.<name>]` blocks.
    entries: BTreeMap<String, Entry>,
    /// The spans `FR-CFG-021` redacts.
    redactions: Vec<redact::Redaction>,
}

/// Reads and validates the document `text` spells.
///
/// It is [`load`] without the filesystem, which is what lets every rule above
/// be exercised against a document written in a test rather than against a file
/// written to disk.
///
/// # Errors
///
/// Returns what [`load`] returns for everything but the read itself.
fn read(text: &str, file: &Path) -> Result<Document, Error> {
    let parsed = DeTable::parse(text).map_err(|refused| Error::ConfigurationMalformed {
        path: file.to_owned(),
        position: position(text, refused.span().map_or(0, |span| span.start)),
        reason: parser_reason(refused.message()),
    })?;
    let root = parsed.get_ref();

    let known: Vec<&str> = root
        .get(DATABASE)
        .and_then(|section| section.get_ref().as_table())
        .map(|table| table.keys().map(|name| name.get_ref().as_ref()).collect())
        .unwrap_or_default();

    check_key_space(root, &known, text, file)?;

    let core = read_core(root, text, file)?;
    let entries = read_entries(root, text, file)?;

    for (name, entry) in &entries {
        check_coherence(name, entry, file)?;
    }

    Ok(Document {
        core,
        entries,
        redactions: redact::spans(root, text),
    })
}

/// Refuses every key of `root` that the space of `FR-CONF-002` does not
/// contain (`FR-CONF-034`).
///
/// The whole file is walked before any value is read, so the key a typo
/// produced is reported rather than the type error it caused.
fn check_key_space(
    root: &DeTable<'_>,
    known: &[&str],
    text: &str,
    file: &Path,
) -> Result<(), Error> {
    for (name, value) in root.iter() {
        let section = name.get_ref().as_ref();

        match section {
            CORE => {
                let table = expect_table(value, CORE, text, file)?;
                for spelled in table.keys() {
                    let leaf = spelled.get_ref().as_ref();
                    if CoreKey::from_leaf(leaf).is_none() {
                        let at = position(text, spelled.span().start);
                        return Err(outside_space(&format!("{CORE}.{leaf}"), known, file, at));
                    }
                }
            }
            DATABASE => {
                let table = expect_table(value, DATABASE, text, file)?;
                for (entry, block) in table.iter() {
                    let entry = entry.get_ref().as_ref();
                    let qualified = format!("{DATABASE}.{entry}");
                    let block = expect_table(block, &qualified, text, file)?;

                    for spelled in block.keys() {
                        let leaf = spelled.get_ref().as_ref();
                        if EntryKey::from_leaf(leaf).is_none() {
                            let at = position(text, spelled.span().start);
                            let key = format!("{qualified}.{leaf}");
                            return Err(outside_space(&key, known, file, at));
                        }
                    }
                }
            }
            other => {
                let at = position(text, name.span().start);
                return Err(outside_space(other, known, file, at));
            }
        }
    }

    Ok(())
}

/// The refusal of `FR-CONF-034`, with the nearest-match suggestion it obliges.
fn outside_space(key: &str, known: &[&str], file: &Path, position: Position) -> Error {
    let population = keys::candidates(known.iter().copied(), key);
    let suggested = nearest(key, &population, Population::ConfigurationKeys);
    let suggested = misplaced(key, &population, suggested);

    Error::ConfigurationKeyOutsideSpace {
        key: key.to_owned(),
        file: file.to_owned(),
        position,
        nearest: suggested,
    }
}

/// The suggestions for `key`, or, where the distance of `FR-ERR-019` found
/// none, the key of the same name in the table that holds it.
///
/// A known key written in the wrong table — `password_timeout` above `[core]`,
/// or under `[database.x]` — is a whole segment away from its own spelling, so
/// the distance never reaches it. The key is offered under every table that
/// does hold a key of that name.
fn misplaced(key: &str, population: &[String], suggested: Vec<String>) -> Vec<String> {
    if !suggested.is_empty() {
        return suggested;
    }

    let leaf = key.rsplit('.').next().unwrap_or(key);
    population
        .iter()
        .filter(|candidate| candidate.as_str() != key && candidate.rsplit('.').next() == Some(leaf))
        .take(3)
        .cloned()
        .collect()
}

/// The table `value` holds, or the refusal of a section that is not one.
fn expect_table<'a>(
    value: &'a Spanned<DeValue<'a>>,
    key: &str,
    text: &str,
    file: &Path,
) -> Result<&'a DeTable<'a>, Error> {
    value
        .get_ref()
        .as_table()
        .ok_or_else(|| malformed_section(text, value, key, file))
}

/// Reads the `[core]` section, applying the declared types of `FR-CONF-002`.
fn read_core(root: &DeTable<'_>, text: &str, file: &Path) -> Result<Core, Error> {
    let mut core = Core::default();

    let Some(section) = root.get(CORE) else {
        return Ok(core);
    };
    let Some(table) = section.get_ref().as_table() else {
        return Err(malformed_section(text, section, CORE, file));
    };

    for key in CoreKey::ALL {
        let Some(value) = table.get(key.leaf()) else {
            continue;
        };
        let qualified = key.to_string();

        match key {
            CoreKey::Database => {
                core.database = Some(string(value, text, &qualified, key.expects(), file)?);
            }
            CoreKey::ConnectTimeout => {
                core.connect_timeout = Some(seconds(value, text, &qualified, file)?);
            }
            CoreKey::PasswordTimeout => {
                core.password_timeout = Some(seconds(value, text, &qualified, file)?);
            }
            CoreKey::QueryTimeout => {
                core.query_timeout = Some(seconds(value, text, &qualified, file)?);
            }
            CoreKey::RenderTimeout => {
                core.render_timeout = Some(seconds(value, text, &qualified, file)?);
            }
            CoreKey::RenderFuel => {
                core.render_fuel = Some(bounded(
                    value,
                    text,
                    &qualified,
                    ValueType::RenderFuel,
                    RenderFuel::new,
                    file,
                )?);
            }
            CoreKey::RenderOutputLimit => {
                core.render_output_limit = Some(bounded(
                    value,
                    text,
                    &qualified,
                    ValueType::RenderOutputLimit,
                    RenderOutputLimit::new,
                    file,
                )?);
            }
            CoreKey::RenderMemoryLimit => {
                core.render_memory_limit = Some(bounded(
                    value,
                    text,
                    &qualified,
                    ValueType::RenderMemoryLimit,
                    RenderMemoryLimit::new,
                    file,
                )?);
            }
        }
    }

    Ok(core)
}

/// Reads the `[database.<name>]` blocks, applying the declared types of
/// `FR-CONF-002` and the array rule of `FR-CONF-035`.
fn read_entries(
    root: &DeTable<'_>,
    text: &str,
    file: &Path,
) -> Result<BTreeMap<String, Entry>, Error> {
    let mut entries = BTreeMap::new();

    let Some(section) = root.get(DATABASE) else {
        return Ok(entries);
    };
    let Some(table) = section.get_ref().as_table() else {
        return Err(malformed_section(text, section, DATABASE, file));
    };

    for (name, block) in table.iter() {
        let name = name.get_ref().as_ref();
        let qualified = format!("{DATABASE}.{name}");
        let Some(block) = block.get_ref().as_table() else {
            return Err(malformed_section(text, block, &qualified, file));
        };

        entries.insert(name.to_owned(), read_entry(block, text, &qualified, file)?);
    }

    Ok(entries)
}

/// Reads one `[database.<name>]` block.
fn read_entry(block: &DeTable<'_>, text: &str, entry: &str, file: &Path) -> Result<Entry, Error> {
    let mut read = Entry::default();

    for field in EntryKey::ALL {
        let Some(value) = block.get(field.leaf()) else {
            continue;
        };
        let key = format!("{entry}.{field}");
        let expects = field.expects();

        match field {
            EntryKey::Dsn => {
                read.dsn = Some(Located {
                    value: string(value, text, &key, expects, file)?,
                    position: at(text, value),
                });
            }
            EntryKey::Host => read.host = Some(string(value, text, &key, expects, file)?),
            EntryKey::Port => {
                read.port = Some(Located {
                    value: port(value, text, &key, file)?,
                    position: at(text, value),
                });
            }
            EntryKey::User => read.user = Some(string(value, text, &key, expects, file)?),
            EntryKey::Password => read.password = Some(string(value, text, &key, expects, file)?),
            EntryKey::PasswordCommand => {
                read.password_command = Some(arguments(value, text, &key, file)?);
            }
            EntryKey::Database => read.database = Some(string(value, text, &key, expects, file)?),
            EntryKey::Tls => read.tls = Some(tls(value, text, &key, file)?),
            EntryKey::CaFile => {
                read.ca_file = Some(PathBuf::from(literal_path(value, text, &key, file)?));
            }
            EntryKey::CaPath => {
                read.ca_path = Some(PathBuf::from(literal_path(value, text, &key, file)?));
            }
        }
    }

    Ok(read)
}

/// Applies `FR-CONF-006` and `FR-CONF-007` to one entry, and the DSN grammar of
/// `FR-CONF-009` through `FR-CONF-012` to the DSN it may carry.
fn check_coherence(name: &str, entry: &Entry, file: &Path) -> Result<(), Error> {
    let conflict = |(first, second): (EntryKey, EntryKey)| Error::ConflictingEntryKeys {
        entry: name.to_owned(),
        file: file.to_owned(),
        first: first.to_string(),
        second: second.to_string(),
    };

    // The rule is asked twice of the same combination, because the third row of
    // `FR-CONF-007` reads the DSN's own password and the first two do not: a
    // DSN the grammar refuses, beside a discrete field, is reported as the pair
    // it forms rather than as the DSN, which is the order this check has always
    // applied.
    let combination = Combination::of(entry);
    if let Some(pair) = combination.refused() {
        return Err(conflict(pair));
    }

    let Some(written) = entry.dsn.as_ref() else {
        return Ok(());
    };

    let key = format!("{DATABASE}.{name}.{}", EntryKey::Dsn);
    let parsed = dsn::parse(&written.value, &key, file)?;

    match combination
        .with_dsn_password(parsed.password().is_some())
        .refused()
    {
        Some(pair) => Err(conflict(pair)),
        None => Ok(()),
    }
}

/// The string `value` holds, or the refusal of a value that is not one.
fn string(
    value: &Spanned<DeValue<'_>>,
    text: &str,
    key: &str,
    expects: ValueType,
    file: &Path,
) -> Result<String, Error> {
    value
        .get_ref()
        .as_str()
        .map(str::to_owned)
        .ok_or_else(|| malformed(text, value, key, expects.expected(), file))
}

/// The path `value` holds for `ca_file` or `ca_path`, which `FR-CONF-047`
/// reads literally and refuses where it holds `${`.
fn literal_path(
    value: &Spanned<DeValue<'_>>,
    text: &str,
    key: &str,
    file: &Path,
) -> Result<String, Error> {
    let path = string(value, text, key, ValueType::Path, file)?;

    if path.contains("${") {
        return Err(Error::ConfigurationPathReference {
            key: key.to_owned(),
            file: file.to_owned(),
            position: at(text, value),
            value: path,
        });
    }

    Ok(path)
}

/// The positive number of seconds `value` holds (`FR-CONF-002`).
fn seconds(
    value: &Spanned<DeValue<'_>>,
    text: &str,
    key: &str,
    file: &Path,
) -> Result<Seconds, Error> {
    let expected = ValueType::Seconds.expected();

    let DeValue::Integer(integer) = value.get_ref() else {
        return Err(malformed(text, value, key, expected, file));
    };

    u64::from_str_radix(integer.as_str(), integer.radix())
        .ok()
        .and_then(NonZeroU64::new)
        .map(Seconds::new)
        .ok_or_else(|| malformed(text, value, key, expected, file))
}

/// The render bound `value` holds, where it is an integer within the range
/// `FR-CONF-002` declares for `key` (`FR-CONF-045`).
///
/// `admit` is the bound's own constructor, which is where the range lives, so
/// the file and `tpl cfg set` refuse exactly the same values. A value outside
/// the range — `0`, a negative integer, one past the maximum — and a value
/// that is not an integer at all are the same `78`, naming the key, the file,
/// the value found and the range expected.
fn bounded<T>(
    value: &Spanned<DeValue<'_>>,
    text: &str,
    key: &str,
    expects: ValueType,
    admit: fn(u64) -> Option<T>,
    file: &Path,
) -> Result<T, Error> {
    let DeValue::Integer(integer) = value.get_ref() else {
        return Err(malformed(text, value, key, expects.expected(), file));
    };

    u64::from_str_radix(integer.as_str(), integer.radix())
        .ok()
        .and_then(admit)
        .ok_or_else(|| malformed(text, value, key, expects.expected(), file))
}

/// The port `value` holds, as the file wrote it (`FR-CONF-002`,
/// `FR-CONF-015`).
fn port(
    value: &Spanned<DeValue<'_>>,
    text: &str,
    key: &str,
    file: &Path,
) -> Result<PortSetting, Error> {
    let expected = ValueType::Port.expected();

    match value.get_ref() {
        DeValue::Integer(integer) => u64::from_str_radix(integer.as_str(), integer.radix())
            .ok()
            .and_then(|number| u16::try_from(number).ok())
            .filter(|number| *number > 0)
            .map(PortSetting::Fixed)
            .ok_or_else(|| malformed(text, value, key, expected, file)),
        DeValue::String(written) => Ok(PortSetting::Written(written.as_ref().to_owned())),
        _ => Err(malformed(text, value, key, expected, file)),
    }
}

/// The TLS mode `value` holds (`FR-CONF-013`).
fn tls(value: &Spanned<DeValue<'_>>, text: &str, key: &str, file: &Path) -> Result<TlsMode, Error> {
    let expected = ValueType::Tls.expected();

    value
        .get_ref()
        .as_str()
        .and_then(TlsMode::from_name)
        .ok_or_else(|| malformed(text, value, key, expected, file))
}

/// The argument array `value` holds (`FR-CONF-023`, `FR-CONF-035`).
///
/// A value that is not an array of strings is the condition `FR-CONF-035`
/// states, whose `hint` shows the array form. An array with no program is the
/// same fault in a different shape: `FR-CONF-024` executes the array directly,
/// and there is nothing to execute.
fn arguments(
    value: &Spanned<DeValue<'_>>,
    text: &str,
    key: &str,
    file: &Path,
) -> Result<PasswordCommand, Error> {
    let not_an_array = |found: &'static str, element| Error::PasswordCommandNotAnArray {
        key: key.to_owned(),
        file: file.to_owned(),
        position: at(text, value),
        found,
        element,
    };

    let Some(members) = value.get_ref().as_array() else {
        return Err(not_an_array(value.get_ref().type_str(), None));
    };

    let mut collected = Vec::with_capacity(members.len());
    for (index, member) in members.iter().enumerate() {
        let Some(argument) = member.get_ref().as_str() else {
            return Err(not_an_array(member.get_ref().type_str(), Some(index)));
        };
        collected.push(argument.to_owned());
    }

    PasswordCommand::new(collected).ok_or_else(|| not_an_array(EMPTY_ARRAY, None))
}

/// The `found` of [`Error::PasswordCommandNotAnArray`] for `password_command =
/// []`: an array, but one with no program to run.
pub(crate) const EMPTY_ARRAY: &str = "empty array";

/// The refusal of a value that is not of its declared type.
///
/// `FR-ERR-013` bars a credential from every message, so the value found is
/// named only where the key cannot hold one; for the two keys that can, the
/// TOML type of the value is named instead, which is what the reader needs and
/// carries nothing of the value.
fn malformed(
    text: &str,
    value: &Spanned<DeValue<'_>>,
    key: &str,
    expected: &'static str,
    file: &Path,
) -> Error {
    let secret = Key::parse(key).is_some_and(|parsed| match parsed {
        Key::Core(_) => false,
        Key::Entry { field, .. } => field.may_be_a_credential(),
    });

    let found = if secret {
        value.get_ref().type_str().to_owned()
    } else {
        rendered(text, value.span())
    };

    Error::ConfigurationValueMalformed {
        key: key.to_owned(),
        file: file.to_owned(),
        position: at(text, value),
        found,
        expected,
        expanded_from: None,
    }
}

/// The refusal of a section that is not a table.
fn malformed_section(text: &str, value: &Spanned<DeValue<'_>>, key: &str, file: &Path) -> Error {
    Error::ConfigurationValueMalformed {
        key: key.to_owned(),
        file: file.to_owned(),
        position: at(text, value),
        found: value.get_ref().type_str().to_owned(),
        expected: A_TABLE,
        expanded_from: None,
    }
}

/// The bytes of `text` a span covers, as the file wrote them.
fn rendered(text: &str, span: Range<usize>) -> String {
    text.get(span).unwrap_or_default().to_owned()
}

/// Where in `text` a spanned value was written.
pub(crate) fn at(text: &str, value: &Spanned<DeValue<'_>>) -> Position {
    position(text, value.span().start)
}

/// The line and column, counted from one, of a byte offset into `text`.
/// What the TOML parser said it expected, on one line.
///
/// The parser's message is its own diagnosis — "invalid table header",
/// "expected `.`, `]`" — and carries no excerpt of the file; the excerpt is
/// what its `Display` adds, and `BR-ERR-003` bars the contents of `.tpl/.cfg`
/// from every message, so the message is taken alone. Its lines are joined,
/// because `FR-ERR-008` gives the `cause` one line.
pub(crate) fn parser_reason(message: &str) -> String {
    let joined = message
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty())
        .collect::<Vec<_>>()
        .join("; ");

    if joined.is_empty() {
        "the parser gave no reason".to_owned()
    } else {
        joined
    }
}

fn position(text: &str, offset: usize) -> Position {
    let upto = text.get(..offset).unwrap_or(text);
    let line = upto.bytes().filter(|byte| *byte == b'\n').count() + 1;
    let column = upto
        .rsplit_once('\n')
        .map_or(upto, |(_, last)| last)
        .chars()
        .count()
        + 1;

    Position { line, column }
}

#[cfg(test)]
mod tests {
    use super::entry::{PortSetting, TlsMode, Written};
    use super::keys::{CoreKey, EntryKey, Key};
    use super::{Configuration, Core, Document, read};
    use crate::error::{DsnFault, Error};
    use crate::render::{RenderFuel, RenderMemoryLimit, RenderOutputLimit};
    use std::path::{Path, PathBuf};

    fn file() -> PathBuf {
        PathBuf::from("/work/.tpl/.cfg")
    }

    fn accepted(text: &str) -> Document {
        read(text, &file()).expect("the document is valid")
    }

    fn refused(text: &str) -> Error {
        read(text, &file()).expect_err("the document is refused")
    }

    /// A `.tpl/.cfg` carrying `text`, loaded from a scratch directory.
    fn loaded(scratch: &crate::project::scratch::Scratch, text: &str) -> Configuration {
        super::load(&scratch.file(".cfg", text)).expect("the document is valid")
    }

    #[test]
    fn fr_err_013_the_debug_of_a_configuration_carries_none_of_the_file_it_read() {
        // FR-ERR-013 and BR-ERR-003 bar a credential from every message. The
        // `text` field is the file's own bytes, so a derived Debug printed the
        // password twice — once raw and once through the entry — beside a
        // `redactions` list that describes the redaction without applying it,
        // and `cli::source::Opened` derives Debug on a struct holding this one.
        //
        // This fails the moment the raw text becomes printable again, by a
        // derive restored here or by an accessor that hands it out.
        let scratch = crate::project::scratch::Scratch::new();
        let document = loaded(
            &scratch,
            "[database.shop]\nhost = \"db\"\nuser = \"alice\"\npassword = \"hunter2\"\n",
        );

        for rendered in [format!("{document:?}"), format!("{document:#?}")] {
            assert!(!rendered.contains("hunter2"), "{rendered}");
            // The shape survives: what a reader of this output wants is which
            // keys the file set, and that is not a secret.
            assert!(rendered.contains("shop"), "{rendered}");
            assert!(rendered.contains("***"), "{rendered}");
        }

        // FR-CFG-013 still prints the file literally with the password
        // redacted, which is the one printer the raw text has.
        let printed = document.printed();
        assert!(!printed.contains("hunter2"), "{printed}");
        assert!(printed.contains("***"), "{printed}");
        assert!(printed.contains("user = \"alice\""), "{printed}");
    }

    #[test]
    fn fr_conf_002_an_absent_section_yields_an_empty_document() {
        let document = accepted("");

        assert_eq!(document.core, Core::default());
        assert!(document.entries.is_empty());
    }

    #[test]
    fn fr_conf_002_the_five_core_keys_read_as_the_types_the_table_declares() {
        // FR-CONF-002.
        let core = accepted(
            r#"
[core]
database = "shop"
connect_timeout = 3
query_timeout = 45
password_timeout = 7
render_timeout = 90
"#,
        )
        .core;

        assert_eq!(core.database.as_deref(), Some("shop"));
        assert_eq!(core.connect_timeout.map(|value| value.get()), Some(3));
        assert_eq!(core.query_timeout.map(|value| value.get()), Some(45));
        assert_eq!(core.password_timeout.map(|value| value.get()), Some(7));
        assert_eq!(core.render_timeout.map(|value| value.get()), Some(90));
    }

    #[test]
    fn fr_conf_034_a_key_outside_the_space_is_refused_with_a_nearest_match() {
        // FR-CONF-034: anywhere in the file, with a nearest-match suggestion
        // over the known keys.
        let condition = refused("[core]\ndatabse = \"shop\"\n");

        match condition {
            Error::ConfigurationKeyOutsideSpace {
                ref key,
                ref nearest,
                ..
            } => {
                assert_eq!(key, "core.databse");
                assert!(
                    nearest.iter().any(|name| name == "core.database"),
                    "{nearest:?}"
                );
            }
            other => panic!("expected a key outside the space, got {other:?}"),
        }
        assert_eq!(condition.exit_code(), 78);
    }

    #[test]
    fn fr_conf_034_a_misspelled_entry_key_is_refused_and_the_suggestion_names_the_entry() {
        // FR-CONF-034: the rationale's own example — a misspelled
        // passwrod_command must not be a silent no-op.
        let condition =
            refused("[database.shop]\nhost = \"db.example.com\"\npasswrod_command = [\"pass\"]\n");

        match condition {
            Error::ConfigurationKeyOutsideSpace { key, nearest, .. } => {
                assert_eq!(key, "database.shop.passwrod_command");
                assert!(
                    nearest
                        .iter()
                        .any(|name| name == "database.shop.password_command"),
                    "{nearest:?}"
                );
            }
            other => panic!("expected a key outside the space, got {other:?}"),
        }
    }

    #[test]
    fn fr_conf_034_a_section_the_space_does_not_name_is_refused_too() {
        // FR-CONF-034: the space has exactly two sections.
        assert!(matches!(
            refused("[cache]\nsize = 1\n"),
            Error::ConfigurationKeyOutsideSpace { .. }
        ));
    }

    #[test]
    fn fr_conf_002_a_value_of_the_wrong_type_is_refused_with_its_position() {
        // FR-CONF-002, and the `78` row of FR-ERR-034: the key and the file,
        // with the value found and the value expected.
        let condition = refused("[core]\nconnect_timeout = \"soon\"\n");

        match condition {
            Error::ConfigurationValueMalformed {
                ref key,
                position,
                ref found,
                expected,
                ..
            } => {
                assert_eq!(key, "core.connect_timeout");
                assert_eq!(position.line, 2);
                assert_eq!(found, "\"soon\"");
                assert_eq!(expected, "a positive integer number of seconds");
            }
            other => panic!("expected a malformed value, got {other:?}"),
        }
        assert_eq!(condition.exit_code(), 78);
    }

    #[test]
    fn fr_conf_002_a_deadline_of_zero_seconds_is_not_a_positive_integer() {
        // FR-CONF-002 declares the four keys positive integers.
        assert!(matches!(
            refused("[core]\nquery_timeout = 0\n"),
            Error::ConfigurationValueMalformed { .. }
        ));
        assert!(matches!(
            refused("[core]\nquery_timeout = -5\n"),
            Error::ConfigurationValueMalformed { .. }
        ));
    }

    #[test]
    fn fr_conf_045_the_three_render_bounds_are_read_at_their_extremes() {
        let scratch = crate::project::scratch::Scratch::new();
        let core = loaded(
            &scratch,
            "[core]\nrender_fuel = 1000000000000\nrender_output_limit = 1\nrender_memory_limit = 8388608\n",
        )
        .core;

        assert_eq!(
            core.render_fuel.map(RenderFuel::get),
            Some(1_000_000_000_000)
        );
        assert_eq!(
            core.render_output_limit.map(RenderOutputLimit::get),
            Some(1)
        );
        assert_eq!(
            core.render_memory_limit.map(RenderMemoryLimit::get),
            Some(8_388_608)
        );
        assert_eq!(
            core.written(CoreKey::RenderFuel),
            Some(Written::Number(1_000_000_000_000))
        );
    }

    #[test]
    fn fr_conf_045_a_render_bound_outside_its_range_is_78_naming_the_key_the_value_and_the_range() {
        // FR-CONF-045: neither key admits 0, a value past its maximum, or
        // anything that is not an integer; the cause names the key, the file,
        // the value found and the range expected, per the 78 row of FR-ERR-034.
        for (text, key, found, expected) in [
            (
                "[core]\nrender_fuel = 0\n",
                "core.render_fuel",
                "0",
                "an integer number of evaluation steps from 1 to 1000000000000",
            ),
            (
                "[core]\nrender_fuel = 1000000000001\n",
                "core.render_fuel",
                "1000000000001",
                "an integer number of evaluation steps from 1 to 1000000000000",
            ),
            (
                "[core]\nrender_fuel = -1\n",
                "core.render_fuel",
                "-1",
                "an integer number of evaluation steps from 1 to 1000000000000",
            ),
            (
                "[core]\nrender_output_limit = 0\n",
                "core.render_output_limit",
                "0",
                "an integer number of bytes from 1 to 1099511627776",
            ),
            (
                "[core]\nrender_output_limit = 1099511627777\n",
                "core.render_output_limit",
                "1099511627777",
                "an integer number of bytes from 1 to 1099511627776",
            ),
            (
                "[core]\nrender_memory_limit = 8388607\n",
                "core.render_memory_limit",
                "8388607",
                "an integer number of bytes from 8388608 to 1099511627776",
            ),
            (
                "[core]\nrender_memory_limit = 1099511627777\n",
                "core.render_memory_limit",
                "1099511627777",
                "an integer number of bytes from 8388608 to 1099511627776",
            ),
            (
                "[core]\nrender_output_limit = \"lots\"\n",
                "core.render_output_limit",
                "\"lots\"",
                "an integer number of bytes from 1 to 1099511627776",
            ),
        ] {
            let condition = refused(text);

            match &condition {
                Error::ConfigurationValueMalformed {
                    key: named,
                    found: shown,
                    expected: range,
                    ..
                } => {
                    assert_eq!(named, key, "{text}");
                    assert_eq!(shown, found, "{text}");
                    assert_eq!(*range, expected, "{text}");
                }
                other => panic!("{text}: expected a malformed value, got {other:?}"),
            }
            assert_eq!(condition.exit_code(), 78, "{text}");
        }
    }

    #[test]
    fn fr_err_013_a_malformed_value_of_a_credential_key_names_its_type_and_never_its_bytes() {
        // FR-ERR-013: no credential in any message, at any verbosity.
        let condition = refused("[database.shop]\npassword = 1234\n");

        match condition {
            Error::ConfigurationValueMalformed { ref found, .. } => {
                assert_eq!(found, "integer");
                assert!(!format!("{condition:?}").contains("1234"));
            }
            other => panic!("expected a malformed value, got {other:?}"),
        }
    }

    #[test]
    fn fr_conf_035_a_password_command_that_is_not_an_array_of_strings_is_refused() {
        // FR-CONF-035, and the hint shows the array form.
        for text in [
            "[database.shop]\npassword_command = \"pass db/shop\"\n",
            "[database.shop]\npassword_command = [1, 2]\n",
            "[database.shop]\npassword_command = []\n",
        ] {
            let condition = read(text, &file()).expect_err("the value is refused");
            assert!(
                matches!(condition, Error::PasswordCommandNotAnArray { .. }),
                "{text}: {condition:?}"
            );
            assert_eq!(condition.exit_code(), 78);
        }
    }

    #[test]
    fn fr_conf_002_an_entry_reads_every_key_of_the_space_as_its_declared_type() {
        let entries = accepted(
            r#"
[database.shop]
host = "db.example.com"
port = 3307
user = "alice"
password = "hunter2"
database = "shop"
tls = "required"
ca_file = "/etc/ssl/private.pem"
ca_path = "/etc/ssl/certs"
"#,
        )
        .entries;

        let entry = entries.get("shop").expect("the entry was read");
        assert_eq!(entry.host.as_deref(), Some("db.example.com"));
        assert_eq!(
            entry.port.as_ref().map(|port| port.value.clone()),
            Some(PortSetting::Fixed(3307))
        );
        assert_eq!(entry.user.as_deref(), Some("alice"));
        assert_eq!(entry.tls, Some(TlsMode::Required));
        assert_eq!(
            entry.ca_file.as_deref(),
            Some(Path::new("/etc/ssl/private.pem"))
        );
    }

    #[test]
    fn fr_conf_002_a_port_outside_the_range_of_a_tcp_port_is_refused() {
        assert!(matches!(
            refused("[database.shop]\nport = 70000\n"),
            Error::ConfigurationValueMalformed { .. }
        ));
        assert!(matches!(
            refused("[database.shop]\nport = 0\n"),
            Error::ConfigurationValueMalformed { .. }
        ));
    }

    #[test]
    fn fr_conf_013_a_tls_mode_outside_the_five_is_refused() {
        // FR-CONF-013: the set is closed.
        assert!(matches!(
            refused("[database.shop]\ntls = \"off\"\n"),
            Error::ConfigurationValueMalformed { .. }
        ));
    }

    #[test]
    fn fr_conf_006_a_dsn_beside_a_discrete_connection_field_is_refused() {
        // FR-CONF-006, FR-CONF-007, first row.
        let condition =
            refused("[database.shop]\ndsn = \"mysql://db.example.com/shop\"\nhost = \"other\"\n");

        match condition {
            Error::ConflictingEntryKeys {
                ref entry,
                ref first,
                ref second,
                ..
            } => {
                assert_eq!(entry, "shop");
                assert_eq!(first, "dsn");
                assert_eq!(second, "host");
            }
            other => panic!("expected conflicting keys, got {other:?}"),
        }
        assert_eq!(condition.exit_code(), 78);
    }

    #[test]
    fn fr_conf_007_a_dsn_with_no_password_beside_a_password_command_is_admitted() {
        // FR-CONF-007, second row: the safest configuration there is.
        let entries = accepted(
            "[database.shop]\ndsn = \"mysql://alice@db.example.com/shop\"\npassword_command = [\"pass\", \"db/shop\"]\n",
        )
        .entries;

        let entry = entries.get("shop").expect("the entry was read");
        assert!(entry.dsn.is_some());
        assert!(entry.password_command.is_some());
    }

    #[test]
    fn fr_conf_007_a_dsn_carrying_a_password_beside_a_password_command_is_refused() {
        // FR-CONF-007, third row: two answers to one question.
        assert!(matches!(
            refused(
                "[database.shop]\ndsn = \"mysql://alice:hunter2@db.example.com/shop\"\npassword_command = [\"pass\"]\n"
            ),
            Error::ConflictingEntryKeys { .. }
        ));
    }

    #[test]
    fn fr_conf_007_discrete_fields_beside_a_password_command_are_admitted() {
        // FR-CONF-007, fourth row.
        let entries = accepted(
            "[database.shop]\nhost = \"db.example.com\"\nuser = \"alice\"\npassword_command = [\"pass\"]\n",
        )
        .entries;

        assert!(entries.contains_key("shop"));
    }

    #[test]
    fn fr_conf_007_a_password_beside_a_password_command_is_refused() {
        // FR-CONF-007, fifth row.
        let condition = refused(
            "[database.shop]\nhost = \"db\"\npassword = \"hunter2\"\npassword_command = [\"pass\"]\n",
        );

        match condition {
            Error::ConflictingEntryKeys {
                ref first,
                ref second,
                ..
            } => {
                assert_eq!(first, "password");
                assert_eq!(second, "password_command");
            }
            other => panic!("expected conflicting keys, got {other:?}"),
        }
        assert!(!format!("{condition}").contains("hunter2"));
    }

    #[test]
    fn fr_conf_009_a_malformed_dsn_in_the_file_is_refused_at_the_read() {
        // FR-CONF-009, FR-CONF-010: the file is validated before any command
        // resolves a key of its own, per step 3 of FR-ERR-006.
        let condition = refused("[database.shop]\ndsn = \"postgres://db.example.com/shop\"\n");

        assert!(matches!(
            condition,
            Error::DsnMalformed {
                fault: DsnFault::Scheme,
                ..
            }
        ));
        assert_eq!(condition.exit_code(), 78);
    }

    #[test]
    fn fr_conf_011_a_dsn_query_parameter_in_the_file_is_refused_at_the_read() {
        // FR-CONF-011, FR-CONF-012.
        let condition =
            refused("[database.shop]\ndsn = \"mysql://db.example.com/shop?tls=false\"\n");

        assert!(matches!(condition, Error::DsnQueryParameter { .. }));
        assert_eq!(condition.exit_code(), 78);
    }

    #[test]
    fn fr_err_001_toml_the_parser_rejects_is_refused_with_its_position() {
        let condition = read("[core\n", &file()).expect_err("the TOML is malformed");

        assert!(matches!(condition, Error::ConfigurationMalformed { .. }));
        assert_eq!(condition.exit_code(), 78);
    }

    #[test]
    fn fr_conf_034_the_unknown_key_is_reported_before_the_malformed_value() {
        // The steps run in order over the whole file, so a file carrying both
        // reports the misspelling, which is the likelier cause of the other.
        assert!(matches!(
            refused("[core]\ndatabse = \"shop\"\nconnect_timeout = \"soon\"\n"),
            Error::ConfigurationKeyOutsideSpace { .. }
        ));
    }

    #[test]
    fn fr_cfg_006_the_document_answers_with_the_value_as_written() {
        // FR-CFG-006, FR-CFG-036.
        let document = accepted(
            "[core]\ndatabase = \"shop\"\n\n[database.shop]\nport = 3307\npassword_command = [\"pass\", \"db/shop\"]\n",
        );
        let (core, entries) = (document.core, document.entries);

        assert_eq!(core.written(CoreKey::Database), Some(Written::Text("shop")));

        let entry = entries.get("shop").expect("the entry was read");
        assert_eq!(entry.written(EntryKey::Port), Some(Written::Number(3307)));

        let arguments = ["pass".to_owned(), "db/shop".to_owned()];
        assert_eq!(
            entry.written(EntryKey::PasswordCommand),
            Some(Written::List(&arguments))
        );
    }

    #[test]
    fn fr_cfg_006_the_key_a_document_sets_is_reachable_by_its_dotted_name() {
        let document = accepted("[core]\ndatabase = \"shop\"\n");
        let configuration = Configuration {
            file: file(),
            text: String::new(),
            core: document.core,
            entries: document.entries,
            redactions: document.redactions,
        };

        let key = Key::parse("core.database").expect("the key is in the space");
        assert_eq!(configuration.written(&key), Some(Written::Text("shop")));
        assert_eq!(configuration.keys(), ["core.database"]);
    }
}
