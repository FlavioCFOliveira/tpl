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
use std::path::PathBuf;

use super::config::entry::{Entry, PortSetting, TlsMode};
use super::config::keys::EntryKey;
use super::config::{Configuration, dsn, expand};
use super::password;
use super::secret::Secret;
use crate::deadline::{Clock, Deadlines, Phase, Seconds};
use crate::error::Error;
use crate::render::RenderBounds;

/// The default of `database.<name>.port` (`FR-CONF-002`).
const DEFAULT_PORT: u16 = 3306;

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

/// Resolves the settings of the selected entry.
///
/// `lookup` supplies the environment `${VAR}` reads, which is
/// [`expand::environment`] in the process and a table in a test, for the reason
/// [`expand`] states.
///
/// # Errors
///
/// Returns what [`select`] returns for the selection, what [`expand`] returns
/// for an unclosed or undefined reference, what [`dsn`] returns for a DSN the
/// grammar refuses, [`Error::ConfigurationValueMalformed`] where an expanded
/// port is not one, and what [`password::obtain`] returns for the child.
pub(crate) fn resolve<L>(
    configuration: &Configuration,
    requested: Option<&str>,
    clock: &Clock,
    lookup: &L,
) -> Result<Settings, Error>
where
    L: Fn(&str) -> Option<String>,
{
    let (name, entry, selection) = select(configuration, requested)?;
    let file = configuration.file();
    let deadlines = deadlines(configuration);

    let qualified = |field: EntryKey| format!("database.{name}.{field}");

    let mut settings = Settings {
        entry: name.to_owned(),
        selection,
        host: None,
        port: DEFAULT_PORT,
        user: None,
        password: None,
        database: None,
        tls: entry.tls.unwrap_or_default(),
        ca_file: entry.ca_file.clone(),
        ca_path: entry.ca_path.clone(),
        deadlines,
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
            settings.port = number(&expanded, &key, written.position, file)?;
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
                    number(&expanded, &key, port.position, file)?
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
fn number(
    value: &str,
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
            .expect("the entry resolves");

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
            .expect("the entry resolves");

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
            .expect("the entry resolves");

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
            .expect("the child produced a password");

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
            .expect("the entry resolves");

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
        .expect("the entry resolves");

        assert_eq!(settings.host(), Some("db.example.com"));
        assert_eq!(settings.entry(), "shop");
    }
}
