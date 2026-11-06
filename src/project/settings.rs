//! The settings a connection will need, resolved from the file and the command
//! line.
//!
//! This is the **resolution** step the reader of [`super::config`] deliberately
//! is not: it reads the environment through `${VAR}`, it runs
//! `password_command`, and it applies the built-in defaults of `FR-CONF-002`.
//! `FR-CFG-014` forbids `tpl cfg list` to do any of the three, which is why the
//! two are separate modules rather than two methods of one.
//!
//! **Two layers and no third.** `FR-CONF-029` resolves every setting through
//! the flag, then `.tpl/.cfg`, then the built-in default, and `FR-CONF-030`
//! states that there is no environment layer: `${VAR}` supplies the value of a
//! key that is already in the file, and `FR-CLI-021` admits no per-flag
//! variable. The only flag that reaches this module is `-d/--database`, which
//! selects the entry; `--timeout` is not a layer either, per `FR-CONF-004`, and
//! arrives inside the [`Clock`] that composes it with each phase deadline.
//!
//! **A DSN resolves to the same fields a discrete entry does.** `FR-CONF-018`
//! fixes the order — parse the URL, expand inside the already-delimited field,
//! then percent-encode — and this module performs the first two and hands the
//! third to whoever composes a URL for the driver, per [`super::config::dsn`].
//! What reaches a connection is therefore a host, a port, a user, a password
//! and a database, whichever way the entry was written, and an expanded value
//! has no delimiter to move because there is no URL for it to move within.
//!
//! **What this module does not decide.** `FR-CONF-002` gives neither `host` nor
//! `database` a default and no requirement in force refuses an entry that omits
//! one, so both are carried as absent rather than invented here; the condition
//! belongs to the phase that opens the connection.

use std::fmt;
use std::path::{Path, PathBuf};

use super::config::entry::{Combination, Entry, PortSetting, TlsMode};
use super::config::keys::EntryKey;
use super::config::{Configuration, dsn, expand};
use super::password;
use super::secret::Secret;
use crate::deadline::{Clock, Deadlines, Phase, Seconds};
use crate::error::{Error, KeyAbsence, NulField, NulPlace};
use crate::model::database::Provider;
use crate::render::RenderBounds;

/// The default of `database.<name>.port` (`FR-CONF-002`).
pub(crate) const DEFAULT_PORT: u16 = 3306;

/// How the entry for this invocation was chosen (`FR-GLOB-008`).
///
/// The two are distinguishable because `FR-RND-018` and `FR-RND-019` depend on
/// the distinction: `--context` is refused beside an entry named on the command
/// line and admitted beside one resolved from the file.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Selection {
    /// Named by `-d/--database` on the command line (`FR-GLOB-004`).
    CommandLine,
    /// Resolved from `core.database` in the file (`FR-GLOB-005`).
    File,
}

impl fmt::Display for Selection {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let named = match self {
            Self::CommandLine => "-d/--database",
            Self::File => "core.database",
        };
        f.write_str(named)
    }
}

/// The entry this invocation uses, and how it was chosen.
///
/// `requested` is the value of `-d/--database`, or [`None`] where the flag was
/// absent.
///
/// # Errors
///
/// Returns [`Error::NoDatabaseEntrySelected`] where neither the flag nor
/// `core.database` names one (`FR-GLOB-006`), and
/// [`Error::DatabaseEntryNotFound`] where the name reaches no entry of the file
/// (`FR-GLOB-007`).
pub(crate) fn select<'a>(
    configuration: &'a Configuration,
    requested: Option<&str>,
) -> Result<(&'a str, &'a Entry, Selection), Error> {
    let (name, selection) = match requested {
        Some(named) => (named, Selection::CommandLine),
        None => match configuration.core().database.as_deref() {
            Some(named) => (named, Selection::File),
            None => {
                return Err(Error::NoDatabaseEntrySelected {
                    file: configuration.file().to_owned(),
                    has_entries: configuration.names().len() > 0,
                });
            }
        },
    };

    // The entry name is looked up by value and returned borrowed from the
    // document, so the caller holds the file's own spelling rather than the
    // caller's.
    let (found, entry) = configuration
        .entries()
        .find(|(defined, _)| *defined == name)
        .ok_or_else(|| configuration.entry_not_found(name, selection == Selection::File))?;

    Ok((found, entry, selection))
}

/// Everything a connection needs, with every value already resolved.
#[derive(Debug)]
pub(crate) struct Settings {
    /// The entry these settings came from.
    entry: String,
    /// How that entry was chosen (`FR-GLOB-008`).
    ///
    /// Nothing reads it, and nothing is expected to. `FR-GLOB-008` obliges the
    /// distinction to exist because `FR-RND-018` and `FR-RND-019` depend on it,
    /// and `tpl render` decides both **before** this value exists: `FR-ERR-006`
    /// puts an argument-pair refusal at step 1 and entry resolution at step 5,
    /// so the refusal is read off the argument vector — see
    /// `cli::render::document_flag` — and resolving an entry first would let a
    /// `78` preempt the `64` the pair earns.
    #[allow(
        dead_code,
        reason = "FR-GLOB-008 obliges the distinction and FR-ERR-006 puts its two readers at \
                  step 1, before this value exists; the field carries the requirement and a test \
                  asserts both arms of it"
    )]
    selection: Selection,
    /// Whether the entry is defined by `dsn` rather than by the discrete
    /// fields (`FR-CONF-006`); a `hint` that repoints or completes the entry
    /// names the flag of that form, per `FR-ERR-045`.
    by_dsn: bool,
    /// The host, where the entry names one.
    host: Option<String>,
    /// The port, defaulted to `3306` per `FR-CONF-002`.
    port: u16,
    /// The user, where the entry names one.
    user: Option<String>,
    /// The password, from the `password` key or from `password_command`.
    password: Option<Secret>,
    /// The server-side database, where the entry names one.
    database: Option<String>,
    /// The TLS mode, defaulted to `verify-identity` per `FR-CONF-013`.
    tls: TlsMode,
    /// `ca_file` (`FR-CONF-014`).
    ca_file: Option<PathBuf>,
    /// `ca_path` (`FR-CONF-014`).
    ca_path: Option<PathBuf>,
    /// The four phase deadlines of `FR-CONF-004`.
    deadlines: Deadlines,
    /// The `.tpl/.cfg` the entry was read from, which the `cause` of a refused
    /// database name names (`FR-ERR-048`, error `1102`).
    file: PathBuf,
}

impl Settings {
    /// The entry these settings came from.
    pub(crate) fn entry(&self) -> &str {
        &self.entry
    }

    /// How that entry was chosen (`FR-GLOB-008`).
    #[allow(
        dead_code,
        reason = "FR-GLOB-008 obliges the distinction and FR-ERR-006 puts its two readers at \
                  step 1, before these settings exist; the accessor is what makes the field \
                  readable at all, and a test asserts both arms of it"
    )]
    pub(crate) const fn selection(&self) -> Selection {
        self.selection
    }

    /// Whether the entry is defined by `dsn` (`FR-ERR-045`).
    pub(crate) const fn by_dsn(&self) -> bool {
        self.by_dsn
    }

    /// The host, where the entry names one.
    pub(crate) fn host(&self) -> Option<&str> {
        self.host.as_deref()
    }

    /// The port.
    pub(crate) const fn port(&self) -> u16 {
        self.port
    }

    /// The user, where the entry names one.
    pub(crate) fn user(&self) -> Option<&str> {
        self.user.as_deref()
    }

    /// The password, where the entry supplies one.
    pub(crate) const fn password(&self) -> Option<&Secret> {
        self.password.as_ref()
    }

    /// The server-side database, where the entry names one.
    pub(crate) fn database(&self) -> Option<&str> {
        self.database.as_deref()
    }

    /// The TLS mode this connection is opened with (`FR-CONF-037`).
    pub(crate) const fn tls(&self) -> TlsMode {
        self.tls
    }

    /// The certificate file trusted by `verify-ca` and `verify-identity`.
    pub(crate) fn ca_file(&self) -> Option<&std::path::Path> {
        self.ca_file.as_deref()
    }

    /// The certificate directory trusted by `verify-ca` and `verify-identity`.
    pub(crate) fn ca_path(&self) -> Option<&std::path::Path> {
        self.ca_path.as_deref()
    }

    /// The four phase deadlines.
    pub(crate) const fn deadlines(&self) -> Deadlines {
        self.deadlines
    }

    /// The `.tpl/.cfg` the entry was read from.
    pub(crate) fn file(&self) -> &std::path::Path {
        &self.file
    }
}

/// What a `sqlite` entry resolves to: its database file, as an absolute path
/// (`FR-CONF-053`, `FR-CONF-054`).
#[derive(Debug, Clone)]
pub(crate) struct FileSettings {
    /// The entry these settings came from.
    entry: String,
    /// The database file, resolved against the project directory and
    /// absolute (`FR-CONF-054`).
    path: PathBuf,
    /// The four phase deadlines; a `sqlite` read uses the catalogue-query one
    /// alone (`FR-LITE-009`).
    deadlines: Deadlines,
}

impl FileSettings {
    /// Settings over `path` directly, for the tests of the reader.
    #[cfg(test)]
    pub(crate) fn new(entry: &str, path: &Path, deadlines: Deadlines) -> Self {
        Self {
            entry: entry.to_owned(),
            path: path.to_owned(),
            deadlines,
        }
    }

    /// The entry these settings came from.
    pub(crate) fn entry(&self) -> &str {
        &self.entry
    }

    /// The database file, absolute.
    pub(crate) fn path(&self) -> &Path {
        &self.path
    }

    /// The four phase deadlines.
    pub(crate) const fn deadlines(&self) -> Deadlines {
        self.deadlines
    }
}

/// The settings an entry resolves to, by its provider (`FR-CONF-051`,
/// `OD-35`).
///
/// It is a closed enumeration whose `mariadb` and `mysql` arms hold the server
/// fields, resolved alike (`FR-MYSQL-020`), and whose `sqlite` arm holds the
/// absolute path; the entry name and the deadlines are common to all three.
#[derive(Debug)]
pub(crate) enum Resolved {
    /// A `mariadb` entry.
    Mariadb(Settings),
    /// A `mysql` entry (`FR-MYSQL-001`).
    Mysql(Settings),
    /// A `sqlite` entry.
    Sqlite(FileSettings),
}

impl Resolved {
    /// The entry these settings came from.
    pub(crate) fn entry(&self) -> &str {
        match self {
            Self::Mariadb(settings) | Self::Mysql(settings) => settings.entry(),
            Self::Sqlite(settings) => settings.entry(),
        }
    }

    /// The server settings, where the entry is a `mariadb` entry.
    #[cfg(test)]
    pub(crate) fn into_mariadb(self) -> Option<Settings> {
        match self {
            Self::Mariadb(settings) => Some(settings),
            Self::Mysql(_) | Self::Sqlite(_) => None,
        }
    }

    /// The server settings, where the entry is a `mysql` entry.
    #[cfg(test)]
    pub(crate) fn into_mysql(self) -> Option<Settings> {
        match self {
            Self::Mysql(settings) => Some(settings),
            Self::Mariadb(_) | Self::Sqlite(_) => None,
        }
    }
}

/// The four phase deadlines of `FR-CONF-004`, resolved from `[core]`.
///
/// Each takes the value the file declares for its key, or the built-in default
/// of `FR-CONF-002` where the file declares none. `--timeout` does not
/// participate, per `FR-CONF-004`; it is the [`Clock`]'s.
pub(crate) fn deadlines(configuration: &Configuration) -> Deadlines {
    let core = configuration.core();

    Deadlines::resolve(
        core.connect_timeout,
        core.query_timeout,
        core.password_timeout,
        core.render_timeout,
    )
}

/// The three render bounds of `FR-CONF-045`, resolved from `[core]`.
///
/// Each takes the value the file declares for its key, or the built-in default
/// of `FR-CONF-002` where the file declares none. No flag and no environment
/// variable participates, and `--timeout` does not either.
pub(crate) fn render_bounds(configuration: &Configuration) -> RenderBounds {
    let core = configuration.core();

    RenderBounds::resolve(
        core.render_fuel,
        core.render_output_limit,
        core.render_memory_limit,
    )
}

/// An entry selected and checked as the file writes it, not yet resolved for
/// a connection (step 5 of `FR-ERR-006`, `FR-CONF-056`).
///
/// It is what an invocation holds between entry resolution and the
/// cache-or-read step: the provider's refused keys, a `sqlite` entry's
/// `path`, a DSN the grammar refuses, a value holding a NUL character and a
/// port that is not one have each been refused, and **no** field has been
/// expanded and no `password_command` run. [`Prepared::connect`] does both,
/// and only an invocation that opens a connection calls it, so a read the
/// cache serves never reads the environment and never starts the child.
#[derive(Debug)]
pub(crate) enum Prepared {
    /// A `mariadb` entry.
    Mariadb(Pending),
    /// A `mysql` entry (`FR-MYSQL-001`).
    Mysql(Pending),
    /// A `sqlite` entry, which carries no field `FR-CONF-015` expands.
    Sqlite(FileSettings),
}

/// A server entry as the file writes it: what step 5 of `FR-ERR-006` checks
/// before any expansion.
#[derive(Debug)]
pub(crate) struct Pending {
    /// The entry's name, the file's own spelling.
    entry: String,
    /// How that entry was chosen (`FR-GLOB-008`).
    selection: Selection,
    /// Whether the entry is defined by `dsn` (`FR-ERR-045`).
    by_dsn: bool,
    /// `host` as written, references unexpanded.
    host: Option<String>,
    /// `database` as written, references unexpanded.
    database: Option<String>,
    /// The four phase deadlines of `FR-CONF-004`.
    deadlines: Deadlines,
}

impl Pending {
    /// The entry this value came from.
    pub(crate) fn entry(&self) -> &str {
        &self.entry
    }

    /// Whether the entry is defined by `dsn` (`FR-ERR-045`).
    pub(crate) const fn by_dsn(&self) -> bool {
        self.by_dsn
    }

    /// `host` as written, before any expansion.
    pub(crate) fn host(&self) -> Option<&str> {
        self.host.as_deref()
    }

    /// `database` as written, before any expansion.
    pub(crate) fn database(&self) -> Option<&str> {
        self.database.as_deref()
    }
}

impl Prepared {
    /// The entry this value came from.
    pub(crate) fn entry(&self) -> &str {
        match self {
            Self::Mariadb(pending) | Self::Mysql(pending) => pending.entry(),
            Self::Sqlite(settings) => settings.entry(),
        }
    }

    /// The four phase deadlines.
    pub(crate) const fn deadlines(&self) -> Deadlines {
        match self {
            Self::Mariadb(pending) | Self::Mysql(pending) => pending.deadlines,
            Self::Sqlite(settings) => settings.deadlines(),
        }
    }

    /// The entry resolved for a connection: every field `FR-CONF-015` names
    /// expanded, and the `password_command` of `FR-CONF-024` run
    /// (`FR-CONF-056`).
    ///
    /// It is called by an invocation that is to open a connection, at the
    /// cache-or-read step of `FR-ERR-006` and before the first
    /// connection phase of `FR-CONF-005`, and by nothing else.
    ///
    /// # Errors
    ///
    /// Returns what [`expand`] returns for an unclosed or undefined reference,
    /// [`Error::ConfigurationValueMalformed`] where an expanded port is not
    /// one, [`Error::ValueHoldsNul`] where an expanded value holds a NUL
    /// character, and what [`password::obtain`] returns for the child.
    pub(crate) fn connect<L>(
        &self,
        configuration: &Configuration,
        clock: &Clock,
        lookup: &L,
    ) -> Result<Resolved, Error>
    where
        L: Fn(&str) -> Option<String>,
    {
        let server = |pending: &Pending| {
            let (name, entry, _) = select(configuration, Some(&pending.entry))?;

            resolve_server(configuration, name, entry, pending.selection, clock, lookup)
        };

        match self {
            Self::Mariadb(pending) => server(pending).map(Resolved::Mariadb),
            Self::Mysql(pending) => server(pending).map(Resolved::Mysql),
            Self::Sqlite(settings) => Ok(Resolved::Sqlite(settings.clone())),
        }
    }
}

/// Selects the entry and refuses what the file holds as written — step 5 of
/// `FR-ERR-006` — without expanding any field or running `password_command`
/// (`FR-CONF-056`).
///
/// For every provider it begins with the two checks `FR-CONF-052` and
/// `FR-CONF-053` place there: a key the provider refuses would otherwise be put
/// in force for as long as the child runs (`OD-35`). For a server entry it
/// then refuses a DSN the grammar refuses, a value holding a NUL character
/// (`FR-CONF-057`, item 2), and a port written as a value that is not one and
/// carries no reference.
///
/// # Errors
///
/// Returns what [`select`] returns for the selection;
/// [`Error::ProviderRefusesKeys`] where the entry carries a key its provider
/// refuses; [`Error::EntryKeyMissing`] where a `sqlite` entry carries no
/// `path`, or an empty one; what [`dsn`] returns for a DSN the grammar
/// refuses; [`Error::ValueHoldsNul`] for a value holding a NUL character; and
/// [`Error::ConfigurationValueMalformed`] for a port that is not one.
pub(crate) fn prepare(
    configuration: &Configuration,
    requested: Option<&str>,
) -> Result<Prepared, Error> {
    let (name, entry, selection) = select(configuration, requested)?;
    let file = configuration.file();

    // FR-CONF-052: the keys the provider refuses, at entry resolution.
    let combination = Combination::of(entry);
    let refused = combination.refused_by_provider();
    if !refused.is_empty() {
        return Err(Error::ProviderRefusesKeys {
            entry: name.to_owned(),
            file: file.to_owned(),
            provider: combination.provider(),
            written: combination.provider_written(),
            keys: refused.iter().map(|key| key.leaf()).collect(),
        });
    }

    match entry.provider() {
        Provider::Mariadb => {
            written_server(configuration, name, entry, selection).map(Prepared::Mariadb)
        }
        // FR-MYSQL-020: a `mysql` entry is resolved exactly as a `mariadb` one.
        Provider::Mysql => {
            written_server(configuration, name, entry, selection).map(Prepared::Mysql)
        }
        Provider::Sqlite => resolve_file(configuration, name, entry).map(Prepared::Sqlite),
    }
}

/// Resolves the settings of the selected entry, by its provider, for a
/// connection: [`prepare`] followed by [`Prepared::connect`].
///
/// `lookup` supplies the environment `${VAR}` reads, which is
/// [`expand::environment`] in the process and a table in a test, for the reason
/// [`expand`] states.
///
/// It is for an invocation that always opens a connection —
/// `tpl cfg database test` — and for the tests of the readers; an invocation
/// that may be served from the cache calls the two halves apart, so that a hit
/// performs neither the expansion nor the child (`FR-CONF-056`).
///
/// # Errors
///
/// Returns what [`prepare`] and [`Prepared::connect`] return.
pub(crate) fn resolve<L>(
    configuration: &Configuration,
    requested: Option<&str>,
    clock: &Clock,
    lookup: &L,
) -> Result<Resolved, Error>
where
    L: Fn(&str) -> Option<String>,
{
    prepare(configuration, requested)?.connect(configuration, clock, lookup)
}

/// The checks step 5 of `FR-ERR-006` makes of a server entry as the file
/// writes it, before any expansion (`FR-CONF-056`).
///
/// # Errors
///
/// Returns what [`dsn`] returns for a DSN the grammar refuses,
/// [`Error::ValueHoldsNul`] for a value holding a NUL character, and
/// [`Error::ConfigurationValueMalformed`] for a port written without a
/// reference that is not one.
fn written_server(
    configuration: &Configuration,
    name: &str,
    entry: &Entry,
    selection: Selection,
) -> Result<Pending, Error> {
    let file = configuration.file();
    let qualified = |field: EntryKey| format!("database.{name}.{field}");
    let by_dsn = entry.dsn.is_some();

    let (host, user, password, database) = if let Some(written) = entry.dsn.as_ref() {
        let key = qualified(EntryKey::Dsn);
        let parsed = dsn::parse(&written.value, &key, file)?;

        // A port with no reference is a fault of the file as written, and is
        // refused here; one with a reference waits for its expansion.
        if let Some(port) = parsed.port().map(dsn::Field::raw)
            && !port.contains("${")
        {
            number(port, port, &key, written.position, file)?;
        }

        (
            Some(parsed.host().raw().to_owned()),
            parsed.user().map(|user| user.raw().to_owned()),
            parsed.password().map(|password| password.raw().to_owned()),
            Some(parsed.database().raw().to_owned()),
        )
    } else {
        if let Some(port) = entry.port.as_ref()
            && let PortSetting::Written(written) = &port.value
            && !written.contains("${")
        {
            number(
                written,
                written,
                &qualified(EntryKey::Port),
                port.position,
                file,
            )?;
        }

        (
            entry.host.clone(),
            entry.user.clone(),
            entry.password.clone(),
            entry.database.clone(),
        )
    };

    // FR-CONF-057, item 2: a value the file holds with a NUL character is
    // refused at entry resolution, before any connection is opened.
    nul_free(
        name,
        file,
        by_dsn,
        [
            (NulField::Host, host.as_deref(), EntryKey::Host),
            (NulField::User, user.as_deref(), EntryKey::User),
            (NulField::Password, password.as_deref(), EntryKey::Password),
            (NulField::Database, database.as_deref(), EntryKey::Database),
        ],
    )?;

    Ok(Pending {
        entry: name.to_owned(),
        selection,
        by_dsn,
        host,
        database,
        deadlines: deadlines(configuration),
    })
}

/// Refuses the first of `values` that holds a NUL character (`FR-CONF-057`,
/// item 2).
///
/// The parts of a dsn are read as `tpl` carries them to the driver, and the
/// `cause` names the dsn rather than a discrete key.
///
/// # Errors
///
/// Returns [`Error::ValueHoldsNul`] for the first value holding a NUL.
fn nul_free(
    name: &str,
    file: &Path,
    by_dsn: bool,
    values: [(NulField, Option<&str>, EntryKey); 4],
) -> Result<(), Error> {
    for (field, value, key) in values {
        if let Some(before) = value.and_then(before_nul) {
            let key = if by_dsn { EntryKey::Dsn } else { key };
            let key = format!("database.{name}.{key}");
            let file = file.to_owned();
            let place = if by_dsn {
                NulPlace::DsnPart { key, file }
            } else {
                NulPlace::Key { key, file }
            };
            return Err(nul_refused(name, field, place, before));
        }
    }

    Ok(())
}

/// The settings of a `sqlite` entry: its database file, resolved against the
/// project directory (`FR-CONF-053`, `FR-CONF-054`).
///
/// # Errors
///
/// Returns [`Error::EntryKeyMissing`] where the entry carries no `path`, or one
/// that is empty under the definition of `FR-CONF-050`.
fn resolve_file(
    configuration: &Configuration,
    name: &str,
    entry: &Entry,
) -> Result<FileSettings, Error> {
    let file = configuration.file();
    let missing = |absence| Error::EntryKeyMissing {
        entry: name.to_owned(),
        key: format!("database.{name}.{}", EntryKey::Path),
        file: file.to_owned(),
        flag: PATH_FLAG,
        placeholder: PATH_PLACEHOLDER,
        absence,
    };

    let Some(written) = entry.path.as_deref() else {
        return Err(missing(KeyAbsence::Absent));
    };
    if written.to_str().is_none_or(super::config::keys::is_blank) {
        return Err(missing(KeyAbsence::Empty));
    }

    // FR-CONF-057, item 2: before the file the path names is opened.
    if let Some(before) = written.to_str().and_then(before_nul) {
        return Err(nul_refused(
            name,
            NulField::Path,
            NulPlace::Key {
                key: format!("database.{name}.{}", EntryKey::Path),
                file: file.to_owned(),
            },
            before,
        ));
    }

    Ok(FileSettings {
        entry: name.to_owned(),
        path: project_directory(file).join(written),
        deadlines: deadlines(configuration),
    })
}

/// The flag of `FR-CFG-027` that writes `database.<name>.path`.
const PATH_FLAG: &str = "--path";

/// The placeholder the hint of `FR-CONF-053` writes after it.
const PATH_PLACEHOLDER: &str = "<path>";

/// The project directory of `FR-CONF-054`: the parent of the canonical `.tpl`
/// folder that holds `file`.
///
/// `file` is `.tpl/.cfg` below the folder [`crate::project::Project`]
/// canonicalised per `FR-PROJ-009`, so where the folder was reached through a
/// symbolic link this is the parent of its target, not of the link. A path
/// joined to it that is already absolute stays as written, which is how
/// [`Path::join`] treats an absolute operand.
fn project_directory(file: &Path) -> &Path {
    file.parent()
        .and_then(Path::parent)
        .unwrap_or_else(|| Path::new("/"))
}

/// The settings of a server entry, resolved for a connection: every field
/// `FR-CONF-015` names expanded and `password_command` run (`FR-CONF-056`).
fn resolve_server<L>(
    configuration: &Configuration,
    name: &str,
    entry: &Entry,
    selection: Selection,
    clock: &Clock,
    lookup: &L,
) -> Result<Settings, Error>
where
    L: Fn(&str) -> Option<String>,
{
    let file = configuration.file();
    let deadlines = deadlines(configuration);

    let qualified = |field: EntryKey| format!("database.{name}.{field}");

    let mut settings = Settings {
        entry: name.to_owned(),
        selection,
        by_dsn: entry.dsn.is_some(),
        host: None,
        port: DEFAULT_PORT,
        user: None,
        password: None,
        database: None,
        tls: entry.tls.unwrap_or_default(),
        ca_file: entry.ca_file.clone(),
        ca_path: entry.ca_path.clone(),
        deadlines,
        file: file.to_owned(),
    };

    let mut from_dsn: Option<String> = None;

    if let Some(written) = entry.dsn.as_ref() {
        let key = qualified(EntryKey::Dsn);
        let parsed = dsn::parse(&written.value, &key, file)?;

        settings.host = Some(expand::expand(parsed.host().raw(), lookup, &key, file)?.into_owned());
        settings.database =
            Some(expand::expand(parsed.database().raw(), lookup, &key, file)?.into_owned());

        if let Some(user) = parsed.user() {
            settings.user = Some(expand::expand(user.raw(), lookup, &key, file)?.into_owned());
        }
        if let Some(password) = parsed.password() {
            from_dsn = Some(expand::expand(password.raw(), lookup, &key, file)?.into_owned());
        }
        if let Some(port) = parsed.port() {
            let expanded = expand::expand(port.raw(), lookup, &key, file)?;
            settings.port = number(&expanded, port.raw(), &key, written.position, file)?;
        }
    } else {
        settings.host = expanded(
            EntryKey::Host,
            entry.host.as_deref(),
            lookup,
            &qualified(EntryKey::Host),
            file,
        )?;
        settings.user = expanded(
            EntryKey::User,
            entry.user.as_deref(),
            lookup,
            &qualified(EntryKey::User),
            file,
        )?;
        settings.database = expanded(
            EntryKey::Database,
            entry.database.as_deref(),
            lookup,
            &qualified(EntryKey::Database),
            file,
        )?;

        if let Some(port) = entry.port.as_ref() {
            let key = qualified(EntryKey::Port);
            settings.port = match &port.value {
                PortSetting::Fixed(fixed) => *fixed,
                PortSetting::Written(written) => {
                    let expanded = expand::expand(written, lookup, &key, file)?;
                    number(&expanded, written, &key, port.position, file)?
                }
            };
        }

        from_dsn = expanded(
            EntryKey::Password,
            entry.password.as_deref(),
            lookup,
            &qualified(EntryKey::Password),
            file,
        )?;
    }

    // FR-CONF-057, item 2: [`written_server`] has refused a value the file
    // holds with a NUL character; an expansion is checked again here, still
    // before any connection is opened.
    nul_free(
        name,
        file,
        settings.by_dsn,
        [
            (NulField::Host, settings.host.as_deref(), EntryKey::Host),
            (NulField::User, settings.user.as_deref(), EntryKey::User),
            (NulField::Password, from_dsn.as_deref(), EntryKey::Password),
            (
                NulField::Database,
                settings.database.as_deref(),
                EntryKey::Database,
            ),
        ],
    )?;

    settings.password = match entry.password_command.as_ref() {
        // FR-CONF-007 has already refused an entry carrying two password
        // sources, so the child is reached only where it is the one source.
        Some(command) => Some(password::obtain(
            name,
            command,
            clock.bound(deadlines.of(Phase::PasswordCommand)),
        )?),
        None => from_dsn.map(Secret::new),
    };

    Ok(settings)
}

/// The part of `value` before its first NUL character, where it holds one
/// (`FR-CONF-057`).
pub(crate) fn before_nul(value: &str) -> Option<&str> {
    value.split_once('\0').map(|(before, _)| before)
}

/// The refusal of `FR-CONF-057`, items 2 and 3, for `field` of entry `name`.
///
/// The part before the NUL is carried for every field but the password, which
/// no message reproduces any part of (`FR-ERR-013`).
pub(crate) fn nul_refused(name: &str, field: NulField, place: NulPlace, before: &str) -> Error {
    Error::ValueHoldsNul {
        entry: name.to_owned(),
        field,
        place,
        before: (field != NulField::Password).then(|| before.to_owned()),
    }
}

/// `value` expanded, where `FR-CONF-015` expands the field it belongs to.
///
/// The table decides, not the caller: a field `FR-CONF-016` or `FR-CONF-017`
/// keeps out of the rule is carried as written even if this function is asked
/// to expand it, so the prohibition holds wherever the resolution grows rather
/// than wherever it is remembered.
fn expanded<L>(
    field: EntryKey,
    value: Option<&str>,
    lookup: &L,
    key: &str,
    file: &std::path::Path,
) -> Result<Option<String>, Error>
where
    L: Fn(&str) -> Option<String>,
{
    if !field.expects().expands() {
        return Ok(value.map(str::to_owned));
    }

    value
        .map(|raw| expand::expand(raw, lookup, key, file).map(std::borrow::Cow::into_owned))
        .transpose()
}

/// `value` as a TCP port, or the refusal of a value that is not one.
///
/// `written` is the value as the file wrote it. Where it differs from `value`
/// a `${VAR}` produced the fault, and the refusal says so: the file holds the
/// reference, not the number, and `tpl cfg` reads it without expanding it.
fn number(
    value: &str,
    written: &str,
    key: &str,
    position: crate::error::Position,
    file: &std::path::Path,
) -> Result<u16, Error> {
    value
        .parse::<u16>()
        .ok()
        .filter(|port| *port > 0)
        .ok_or_else(|| Error::ConfigurationValueMalformed {
            key: key.to_owned(),
            file: file.to_owned(),
            position,
            found: value.to_owned(),
            expected: super::config::keys::ValueType::Port.expected(),
            expanded_from: (written != value).then(|| Box::new(written.to_owned())),
        })
}

/// The seconds `--timeout` was given, as the clock's budget.
pub(crate) fn clock(budget: Option<Seconds>) -> Clock {
    Clock::new(budget)
}

#[cfg(test)]
mod tests {
    use super::{Selection, clock, deadlines, resolve, select};
    use crate::deadline::Phase;
    use crate::error::Error;
    use crate::project::config;
    use crate::project::scratch::Scratch;
    use std::path::PathBuf;

    /// A lookup over a fixed table, standing in for the process environment.
    fn table(name: &str) -> Option<String> {
        match name {
            "SHOP_DB_PASSWORD" => Some("hunter2".to_owned()),
            "SHOP_DB_PORT" => Some("3307".to_owned()),
            "SHOP_DB_HOST" => Some("db.example.com".to_owned()),
            "HOSTILE" => Some("x@attacker.example.com/other".to_owned()),
            _ => None,
        }
    }

    /// A document loaded from a real file, which is the only shape `load`
    /// accepts and the shape every command meets.
    fn document(scratch: &Scratch, text: &str) -> config::Configuration {
        let file = scratch.file(".cfg", text);

        config::load(&file).expect("the document is valid")
    }

    #[test]
    fn fr_glob_004_the_flag_selects_the_entry_and_the_file_supplies_it_otherwise() {
        // FR-GLOB-004, FR-GLOB-005, FR-GLOB-008.
        let scratch = Scratch::new();
        let configuration = document(
            &scratch,
            "[core]\ndatabase = \"shop\"\n\n[database.shop]\nhost = \"a\"\n\n[database.reporting]\nhost = \"b\"\n",
        );

        let (name, _, selection) = select(&configuration, None).expect("core.database applies");
        assert_eq!(name, "shop");
        assert_eq!(selection, Selection::File);

        let (name, _, selection) =
            select(&configuration, Some("reporting")).expect("the flag applies");
        assert_eq!(name, "reporting");
        assert_eq!(selection, Selection::CommandLine);
    }

    #[test]
    fn fr_glob_006_nothing_selected_is_a_configuration_fault_naming_the_file() {
        // FR-GLOB-006: 78, with a message naming the file.
        let scratch = Scratch::new();
        let configuration = document(&scratch, "[database.shop]\nhost = \"a\"\n");

        let condition = select(&configuration, None).expect_err("nothing is selected");

        assert!(matches!(condition, Error::NoDatabaseEntrySelected { .. }));
        assert_eq!(condition.exit_code(), 78);
    }

    #[test]
    fn fr_glob_007_a_named_entry_that_does_not_exist_is_a_named_object_that_does_not_exist() {
        // FR-GLOB-007: 66, with a nearest-match suggestion over the entry
        // names that exist.
        let scratch = Scratch::new();
        let configuration = document(&scratch, "[database.shop]\nhost = \"a\"\n");

        let condition = select(&configuration, Some("shup")).expect_err("the entry is absent");

        match condition {
            Error::DatabaseEntryNotFound { ref nearest, .. } => {
                assert_eq!(nearest, &["shop".to_owned()]);
            }
            other => panic!("expected a missing entry, got {other:?}"),
        }
        assert_eq!(condition.exit_code(), 66);
    }

    #[test]
    fn fr_conf_004_the_four_deadlines_take_the_file_and_then_the_built_in_default() {
        // FR-CONF-004, FR-CONF-005.
        let scratch = Scratch::new();
        let configuration = document(&scratch, "[core]\nconnect_timeout = 3\n");
        let resolved = deadlines(&configuration);

        assert_eq!(resolved.of(Phase::TcpConnect).get(), 3);
        assert_eq!(resolved.of(Phase::CatalogueQuery).get(), 30);
        assert_eq!(resolved.of(Phase::PasswordCommand).get(), 5);
        assert_eq!(resolved.of(Phase::Render).get(), 30);
    }

    #[test]
    fn fr_conf_002_a_discrete_entry_resolves_its_fields_and_defaults_the_rest() {
        // FR-CONF-002: port defaults to 3306; FR-CONF-013: tls defaults to
        // verify-identity.
        let scratch = Scratch::new();
        let configuration = document(
            &scratch,
            "[database.shop]\nhost = \"db.example.com\"\nuser = \"alice\"\ndatabase = \"shop\"\n",
        );

        let settings = resolve(&configuration, Some("shop"), &clock(None), &table)
            .expect("the entry resolves")
            .into_mariadb()
            .expect("the entry is a mariadb entry");

        assert_eq!(settings.host(), Some("db.example.com"));
        assert_eq!(settings.port(), 3306);
        assert_eq!(settings.user(), Some("alice"));
        assert_eq!(settings.database(), Some("shop"));
        assert_eq!(
            settings.tls(),
            crate::project::config::entry::TlsMode::VerifyIdentity
        );
        assert!(settings.password().is_none());
    }

    #[test]
    fn fr_conf_018_a_dsn_resolves_to_the_same_fields_a_discrete_entry_does() {
        // FR-CONF-018: the URL is parsed, and each field expanded inside it.
        let scratch = Scratch::new();
        let configuration = document(
            &scratch,
            "[database.shop]\ndsn = \"mysql://alice:${SHOP_DB_PASSWORD}@${SHOP_DB_HOST}:${SHOP_DB_PORT}/shop\"\n",
        );

        let settings = resolve(&configuration, Some("shop"), &clock(None), &table)
            .expect("the entry resolves")
            .into_mariadb()
            .expect("the entry is a mariadb entry");

        assert_eq!(settings.host(), Some("db.example.com"));
        assert_eq!(settings.port(), 3307);
        assert_eq!(settings.user(), Some("alice"));
        assert_eq!(settings.database(), Some("shop"));
        assert_eq!(
            settings
                .password()
                .map(crate::project::secret::Secret::expose),
            Some("hunter2")
        );
    }

    #[test]
    fn fr_conf_018_an_expanded_value_cannot_move_the_host_the_port_or_the_database() {
        // FR-CONF-018, FR-SEC-009: the threat is
        // SHOP_PW=x@attacker.example.com/shop redirecting the connection.
        let scratch = Scratch::new();
        let configuration = document(
            &scratch,
            "[database.shop]\ndsn = \"mysql://alice:${HOSTILE}@db.example.com/shop\"\n",
        );

        let settings = resolve(&configuration, Some("shop"), &clock(None), &table)
            .expect("the entry resolves")
            .into_mariadb()
            .expect("the entry is a mariadb entry");

        assert_eq!(settings.host(), Some("db.example.com"));
        assert_eq!(settings.database(), Some("shop"));
        assert_eq!(
            settings
                .password()
                .map(crate::project::secret::Secret::expose),
            Some("x@attacker.example.com/other")
        );
    }

    #[test]
    fn fr_conf_022_an_undefined_variable_stops_the_resolution() {
        // FR-CONF-022, BR-CONF-002.
        let scratch = Scratch::new();
        let configuration = document(&scratch, "[database.shop]\nhost = \"${ABSENT}\"\n");

        let condition = resolve(&configuration, Some("shop"), &clock(None), &table)
            .expect_err("the variable is not defined");

        assert!(matches!(condition, Error::UndefinedVariable { .. }));
        assert_eq!(condition.exit_code(), 78);
    }

    #[test]
    fn fr_conf_015_an_expanded_port_that_is_not_a_port_is_refused() {
        let scratch = Scratch::new();
        let configuration = document(
            &scratch,
            "[database.shop]\nhost = \"a\"\nport = \"${SHOP_DB_HOST}\"\n",
        );

        let condition = resolve(&configuration, Some("shop"), &clock(None), &table)
            .expect_err("the expanded value is not a port");

        assert!(matches!(
            condition,
            Error::ConfigurationValueMalformed { .. }
        ));
        assert_eq!(condition.exit_code(), 78);
    }

    #[test]
    fn fr_conf_007_a_password_command_supplies_the_password() {
        // FR-CONF-007, fourth row, and FR-CONF-027.
        let echo = ["/bin/echo", "/usr/bin/echo"]
            .into_iter()
            .find(|path| PathBuf::from(path).exists())
            .expect("the system has echo");
        let scratch = Scratch::new();
        let configuration = document(
            &scratch,
            &format!(
                "[database.shop]\nhost = \"a\"\npassword_command = [\"{echo}\", \"hunter2\"]\n"
            ),
        );

        let settings = resolve(&configuration, Some("shop"), &clock(None), &table)
            .expect("the child produced a password")
            .into_mariadb()
            .expect("the entry is a mariadb entry");

        assert_eq!(
            settings
                .password()
                .map(crate::project::secret::Secret::expose),
            Some("hunter2")
        );
    }

    #[test]
    fn fr_err_013_the_resolved_settings_carry_the_secret_in_a_type_that_does_not_print_it() {
        // FR-ERR-013, FR-GLOB-018: no credential in any message, and the
        // derived Debug of the settings delegates to Secret's own.
        let scratch = Scratch::new();
        let configuration = document(
            &scratch,
            "[database.shop]\nhost = \"db\"\npassword = \"${SHOP_DB_PASSWORD}\"\n",
        );

        let settings = resolve(&configuration, Some("shop"), &clock(None), &table)
            .expect("the entry resolves")
            .into_mariadb()
            .expect("the entry is a mariadb entry");

        assert!(!format!("{settings:?}").contains("hunter2"));
        assert!(!format!("{settings:#?}").contains("hunter2"));
    }

    #[test]
    fn fr_conf_030_there_is_no_environment_layer_in_the_precedence() {
        // FR-CONF-030: `${VAR}` supplies the value of a key that is already in
        // the file, and a variable named after a key reaches nothing.
        let scratch = Scratch::new();
        let configuration = document(&scratch, "[database.shop]\nhost = \"db.example.com\"\n");

        let settings = resolve(
            &configuration,
            Some("shop"),
            &clock(None),
            &|name: &str| match name {
                "TPL_DATABASE" | "TPL_DIR" => Some("elsewhere".to_owned()),
                other => table(other),
            },
        )
        .expect("the entry resolves")
        .into_mariadb()
        .expect("the entry is a mariadb entry");

        assert_eq!(settings.host(), Some("db.example.com"));
        assert_eq!(settings.entry(), "shop");
    }

    /// What resolving entry `shop` of `text` refused.
    fn refused(scratch: &Scratch, text: &str) -> Error {
        let configuration = document(scratch, text);

        resolve(&configuration, Some("shop"), &clock(None), &table)
            .expect_err("the entry is refused")
    }

    #[test]
    fn fr_conf_057_a_value_in_the_file_holding_a_nul_is_refused_at_entry_resolution() {
        // FR-CONF-057, item 2: host, user, password and database of a server
        // entry, each part of its dsn, and the path of a sqlite entry; 78,
        // naming the key or the part and the file, and the part before the NUL
        // for every field but the password.
        use crate::error::{NulField, NulPlace};

        let scratch = Scratch::new();
        for (text, field, part, before) in [
            (
                "[database.shop]\nhost = \"db\\u0000x\"\n",
                NulField::Host,
                false,
                Some("db"),
            ),
            (
                "[database.shop]\nhost = \"db\"\nuser = \"rea\\u0000der\"\n",
                NulField::User,
                false,
                Some("rea"),
            ),
            (
                "[database.shop]\nhost = \"db\"\npassword = \"hun\\u0000ter2\"\n",
                NulField::Password,
                false,
                None,
            ),
            (
                "[database.shop]\nhost = \"db\"\ndatabase = \"freight\\u0000x\"\n",
                NulField::Database,
                false,
                Some("freight"),
            ),
            (
                "[database.shop]\ndsn = \"mysql://alice:hun\\u0000ter2@db/freight\"\n",
                NulField::Password,
                true,
                None,
            ),
            (
                "[database.shop]\ndsn = \"mysql://alice@db/freight\\u0000x\"\n",
                NulField::Database,
                true,
                Some("freight"),
            ),
            (
                "[database.shop]\nprovider = \"sqlite\"\npath = \"data/led\\u0000ger.sqlite\"\n",
                NulField::Path,
                false,
                Some("data/led"),
            ),
        ] {
            let condition = refused(&scratch, text);

            assert_eq!(condition.exit_code(), 78, "{text}");
            match condition {
                Error::ValueHoldsNul {
                    ref entry,
                    field: found,
                    ref place,
                    before: ref kept,
                } => {
                    assert_eq!(entry, "shop");
                    assert_eq!(found, field, "{text}");
                    assert_eq!(kept.as_deref(), before, "{text}");
                    match place {
                        NulPlace::DsnPart { key, .. } => {
                            assert!(part, "{text}");
                            assert_eq!(key, "database.shop.dsn");
                        }
                        NulPlace::Key { key, file } => {
                            assert!(!part, "{text}");
                            assert_eq!(key, &format!("database.shop.{field}"));
                            assert!(file.ends_with(".cfg"), "{}", file.display());
                        }
                        NulPlace::Helper { .. } => panic!("{text}: {place:?}"),
                    }
                }
                ref other => panic!("{text}: {other:?}"),
            }
        }
    }

    #[test]
    fn fr_conf_057_a_password_command_that_prints_a_nul_is_refused_and_not_reproduced() {
        // FR-CONF-057, item 3: 78, naming the command as stored and nothing of
        // what it printed.
        use crate::error::{NulField, NulPlace};

        let printf = ["/usr/bin/printf", "/bin/printf"]
            .into_iter()
            .find(|path| PathBuf::from(path).exists())
            .expect("the system has printf");
        let scratch = Scratch::new();
        let condition = refused(
            &scratch,
            &format!(
                "[database.shop]\nhost = \"db\"\npassword_command = [\"{printf}\", \"hun\\\\000ter2\"]\n"
            ),
        );

        assert_eq!(condition.exit_code(), 78);
        assert!(
            matches!(
                condition,
                Error::ValueHoldsNul {
                    field: NulField::Password,
                    place: NulPlace::Helper { ref command },
                    before: None,
                    ..
                } if command.first().map(String::as_str) == Some(printf)
            ),
            "{condition:?}"
        );
    }
}
