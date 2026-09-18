//! The enumerated key space of `FR-CONF-002`, as a type.
//!
//! `FR-CONF-034` refuses a key the space does not contain wherever it is found
//! in the file, and `FR-CFG-009` refuses one supplied to `tpl cfg set`. Both
//! are the same question — is this dotted name a key — and the way to ask it
//! once is to make the answer a value: [`Key::parse`] returns [`None`] for
//! everything the table of `FR-CONF-002` does not list, and every key that
//! exists is one of the five variants of [`CoreKey`] or one of the ten of
//! [`EntryKey`].
//!
//! The type also carries the **declared type** of each key, through
//! [`ValueType`], because `FR-CFG-010` validates a supplied value against it
//! and the `78` row of `FR-ERR-034` obliges a `cause` line to name "the value
//! found and the value expected". One table, read by the reader of the file and
//! by the writer of it alike.
//!
//! The `<name>` of a `database.<name>` key is the one segment the space does
//! not enumerate: `FR-CONF-008` makes it a label local to the project, so it is
//! carried as text and governed, where it is presented, by the character set of
//! `FR-ERR-022`.

use std::fmt;

/// The section a `[core]` key sits under.
const CORE: &str = "core";

/// The section a `[database.<name>]` key sits under.
const DATABASE: &str = "database";

/// The type a key's value takes, per the table of `FR-CONF-002`.
///
/// It exists so that the validation of a value is looked up from the key rather
/// than written out per call site: `tpl cfg set` and the reader of the file ask
/// the same key for the same answer.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum ValueType {
    /// The name of a `[database.<name>]` entry (`core.database`).
    EntryName,
    /// A positive integer number of seconds (the four `[core]` deadlines).
    Seconds,
    /// A connection URL, per `FR-CONF-009` and `FR-CONF-010`.
    Dsn,
    /// A hostname or address, a user name, a password, or a server-side
    /// database name — text the file carries as written.
    Text,
    /// A TCP port.
    Port,
    /// One of the five modes of `FR-CONF-013`.
    Tls,
    /// The argument array of `FR-CONF-023`.
    ArgumentArray,
    /// A filesystem path to trust material (`FR-CONF-014`).
    Path,
}

impl ValueType {
    /// What the `cause` line of `FR-ERR-034` names as the value expected.
    pub(crate) const fn expected(self) -> &'static str {
        match self {
            Self::EntryName => "the name of a database entry",
            Self::Seconds => "a positive integer number of seconds",
            Self::Dsn => "a connection URL",
            Self::Text => "a string",
            Self::Port => "a TCP port between 1 and 65535",
            Self::Tls => "one of disabled, preferred, required, verify-ca, verify-identity",
            Self::ArgumentArray => "an array of arguments",
            Self::Path => "a filesystem path",
        }
    }

    /// Whether `FR-CONF-015` expands `${VAR}` in a value of this type.
    ///
    /// The requirement lists the six fields by name; the two exclusions it
    /// states separately, `tls` in `FR-CONF-016` and `password_command` in
    /// `FR-CONF-017`, fall out of the same list, and so do `ca_file` and
    /// `ca_path`, which it never names.
    pub(crate) const fn expands(self) -> bool {
        matches!(self, Self::Dsn | Self::Text | Self::Port)
    }
}

/// A key of the `[core]` section (`FR-CONF-002`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) enum CoreKey {
    /// `core.connect_timeout`.
    ConnectTimeout,
    /// `core.database`.
    Database,
    /// `core.password_timeout`.
    PasswordTimeout,
    /// `core.query_timeout`.
    QueryTimeout,
    /// `core.render_timeout`.
    RenderTimeout,
}

impl CoreKey {
    /// The five, in the order `FR-CONF-002` states them.
    pub(crate) const ALL: [Self; 5] = [
        Self::Database,
        Self::ConnectTimeout,
        Self::QueryTimeout,
        Self::PasswordTimeout,
        Self::RenderTimeout,
    ];

    /// The key's last segment, as the file spells it.
    pub(crate) const fn leaf(self) -> &'static str {
        match self {
            Self::ConnectTimeout => "connect_timeout",
            Self::Database => "database",
            Self::PasswordTimeout => "password_timeout",
            Self::QueryTimeout => "query_timeout",
            Self::RenderTimeout => "render_timeout",
        }
    }

    /// The key a last segment names, or [`None`] where the space has none.
    pub(crate) fn from_leaf(leaf: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|key| key.leaf() == leaf)
    }

    /// The type `FR-CONF-002` declares for the key.
    pub(crate) const fn expects(self) -> ValueType {
        match self {
            Self::Database => ValueType::EntryName,
            Self::ConnectTimeout
            | Self::PasswordTimeout
            | Self::QueryTimeout
            | Self::RenderTimeout => ValueType::Seconds,
        }
    }
}

impl fmt::Display for CoreKey {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{CORE}.{}", self.leaf())
    }
}

/// A key of a `[database.<name>]` block (`FR-CONF-002`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) enum EntryKey {
    /// `database.<name>.ca_file`.
    CaFile,
    /// `database.<name>.ca_path`.
    CaPath,
    /// `database.<name>.database`, the server-side database name.
    Database,
    /// `database.<name>.dsn`.
    Dsn,
    /// `database.<name>.host`.
    Host,
    /// `database.<name>.password`.
    Password,
    /// `database.<name>.password_command`.
    PasswordCommand,
    /// `database.<name>.port`.
    Port,
    /// `database.<name>.tls`.
    Tls,
    /// `database.<name>.user`.
    User,
}

impl EntryKey {
    /// The ten, in the order `FR-CONF-002` states them.
    pub(crate) const ALL: [Self; 10] = [
        Self::Dsn,
        Self::Host,
        Self::Port,
        Self::User,
        Self::Password,
        Self::PasswordCommand,
        Self::Database,
        Self::Tls,
        Self::CaFile,
        Self::CaPath,
    ];

    /// The five keys `FR-CONF-006` calls the discrete connection fields.
    ///
    /// `password_command` is deliberately not one of them, which is what makes
    /// the third row of the `FR-CONF-007` table an admission rather than a
    /// refusal.
    pub(crate) const DISCRETE: [Self; 5] = [
        Self::Host,
        Self::Port,
        Self::User,
        Self::Password,
        Self::Database,
    ];

    /// The key's last segment, as the file spells it.
    pub(crate) const fn leaf(self) -> &'static str {
        match self {
            Self::CaFile => "ca_file",
            Self::CaPath => "ca_path",
            Self::Database => "database",
            Self::Dsn => "dsn",
            Self::Host => "host",
            Self::Password => "password",
            Self::PasswordCommand => "password_command",
            Self::Port => "port",
            Self::Tls => "tls",
            Self::User => "user",
        }
    }

    /// The key a last segment names, or [`None`] where the space has none.
    pub(crate) fn from_leaf(leaf: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|key| key.leaf() == leaf)
    }

    /// The type `FR-CONF-002` declares for the key.
    pub(crate) const fn expects(self) -> ValueType {
        match self {
            Self::CaFile | Self::CaPath => ValueType::Path,
            Self::Database | Self::Host | Self::Password | Self::User => ValueType::Text,
            Self::Dsn => ValueType::Dsn,
            Self::PasswordCommand => ValueType::ArgumentArray,
            Self::Port => ValueType::Port,
            Self::Tls => ValueType::Tls,
        }
    }

    /// Whether the value of this key may itself be a credential.
    ///
    /// `FR-ERR-013` bars a credential from every message, so a `cause` line
    /// that would otherwise name the value found names the TOML type of the
    /// value instead for these two keys.
    pub(crate) const fn may_be_a_credential(self) -> bool {
        matches!(self, Self::Password | Self::Dsn)
    }
}

impl fmt::Display for EntryKey {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.leaf())
    }
}

/// One key of the space of `FR-CONF-002`, fully qualified.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum Key {
    /// A key of the `[core]` section.
    Core(CoreKey),
    /// A key of one `[database.<name>]` block.
    Entry {
        /// The entry name — the one segment the space does not enumerate,
        /// per `FR-CONF-008`.
        entry: String,
        /// Which key of the block.
        field: EntryKey,
    },
}

impl Key {
    /// The key a dotted name spells, or [`None`] where the space has none.
    ///
    /// A `database.<name>.<field>` key is split at its **last** dot, so an
    /// entry whose name carries one — which `FR-CONF-008` does not forbid —
    /// still round-trips between the file and the command line.
    pub(crate) fn parse(text: &str) -> Option<Self> {
        if let Some(leaf) = text
            .strip_prefix(CORE)
            .and_then(|rest| rest.strip_prefix('.'))
        {
            return CoreKey::from_leaf(leaf).map(Self::Core);
        }

        let rest = text
            .strip_prefix(DATABASE)
            .and_then(|rest| rest.strip_prefix('.'))?;
        let (entry, leaf) = rest.rsplit_once('.')?;

        if entry.is_empty() {
            return None;
        }

        EntryKey::from_leaf(leaf).map(|field| Self::Entry {
            entry: entry.to_owned(),
            field,
        })
    }

    /// The type `FR-CONF-002` declares for this key.
    pub(crate) const fn expects(&self) -> ValueType {
        match self {
            Self::Core(key) => key.expects(),
            Self::Entry { field, .. } => field.expects(),
        }
    }
}

impl fmt::Display for Key {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Core(key) => write!(f, "{key}"),
            Self::Entry { entry, field } => write!(f, "{DATABASE}.{entry}.{field}"),
        }
    }
}

/// What `tpl cfg unset` was given, per `FR-CFG-011`: a leaf key, or a whole
/// block.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum Target {
    /// One key.
    Key(Key),
    /// The whole `[core]` section.
    Core,
    /// Every `[database.<name>]` block.
    Databases,
    /// One `[database.<name>]` block.
    Entry(String),
}

impl Target {
    /// What a dotted name given to `tpl cfg unset` names, or [`None`] where it
    /// names neither a key of the space nor a block of it.
    pub(crate) fn parse(text: &str) -> Option<Self> {
        if text == CORE {
            return Some(Self::Core);
        }
        if text == DATABASE {
            return Some(Self::Databases);
        }

        if let Some(key) = Key::parse(text) {
            return Some(Self::Key(key));
        }

        let entry = text
            .strip_prefix(DATABASE)
            .and_then(|rest| rest.strip_prefix('.'))?;

        (!entry.is_empty() && !entry.contains('.')).then(|| Self::Entry(entry.to_owned()))
    }
}

impl fmt::Display for Target {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Key(key) => write!(f, "{key}"),
            Self::Core => f.write_str(CORE),
            Self::Databases => f.write_str(DATABASE),
            Self::Entry(entry) => write!(f, "{DATABASE}.{entry}"),
        }
    }
}

/// The candidate population `FR-ERR-021` names for a configuration key: the
/// enumerated space of `FR-CONF-002`, written out.
///
/// The five `[core]` keys are the whole of the section. The ten entry keys are
/// written once per entry name the file defines and once for the name the
/// supplied key itself carries, so that `database.shop.hst` is corrected in a
/// project whose file does not yet define `shop`.
///
/// The names are collected rather than borrowed because the entry names come
/// from two populations with different lifetimes and the selection of
/// `FR-ERR-019` measures at most a few dozen strings.
pub(crate) fn candidates<'a, N>(defined: N, supplied: &str) -> Vec<String>
where
    N: IntoIterator<Item = &'a str>,
{
    let mut entries: Vec<String> = defined.into_iter().map(str::to_owned).collect();

    if let Some(named) = entry_named_by(supplied)
        && !entries.iter().any(|known| known == named)
    {
        entries.push(named.to_owned());
    }

    let mut population: Vec<String> = CoreKey::ALL
        .iter()
        .map(|key| key.to_string())
        .collect::<Vec<_>>();

    population.reserve(entries.len() * EntryKey::ALL.len());
    for entry in &entries {
        for field in EntryKey::ALL {
            population.push(format!("{DATABASE}.{entry}.{field}"));
        }
    }

    population
}

/// The entry name a `database.<name>.<anything>` spelling carries, or [`None`].
///
/// It reads the name out of a key that does **not** parse, which is the whole
/// point: a misspelled leaf still names the entry the caller meant.
fn entry_named_by(supplied: &str) -> Option<&str> {
    let rest = supplied
        .strip_prefix(DATABASE)
        .and_then(|rest| rest.strip_prefix('.'))?;
    let (entry, _) = rest.rsplit_once('.')?;

    (!entry.is_empty()).then_some(entry)
}

#[cfg(test)]
mod tests {
    use super::{CoreKey, EntryKey, Key, Target, ValueType, candidates};

    #[test]
    fn fr_conf_002_the_space_is_exactly_the_fifteen_forms_the_table_declares() {
        // FR-CONF-002: five [core] keys and ten keys of an entry, and no
        // others.
        assert_eq!(CoreKey::ALL.len(), 5);
        assert_eq!(EntryKey::ALL.len(), 10);

        let core: Vec<String> = CoreKey::ALL.iter().map(ToString::to_string).collect();
        assert_eq!(
            core,
            [
                "core.database",
                "core.connect_timeout",
                "core.query_timeout",
                "core.password_timeout",
                "core.render_timeout",
            ]
        );

        let entry: Vec<&str> = EntryKey::ALL.iter().map(|key| key.leaf()).collect();
        assert_eq!(
            entry,
            [
                "dsn",
                "host",
                "port",
                "user",
                "password",
                "password_command",
                "database",
                "tls",
                "ca_file",
                "ca_path",
            ]
        );
    }

    #[test]
    fn fr_conf_002_a_key_of_the_space_parses_and_spells_itself_back() {
        // FR-CONF-002, FR-CFG-009: the dotted form is the key's identity, and
        // parsing it is the whole of the membership test.
        for spelling in [
            "core.database",
            "core.connect_timeout",
            "database.shop.dsn",
            "database.shop.password_command",
            "database.shop.ca_path",
        ] {
            let key = Key::parse(spelling).expect("the spelling is in the space");
            assert_eq!(key.to_string(), spelling);
        }
    }

    #[test]
    fn fr_conf_034_a_key_outside_the_space_does_not_parse() {
        // FR-CONF-034, FR-CFG-009: the space is closed, and a key is never
        // created by writing one.
        for spelling in [
            "",
            "core",
            "core.databse",
            "core.database.extra",
            "database",
            "database.shop",
            "database..host",
            "database.shop.hst",
            "passwrod_command",
            "cache.size",
        ] {
            assert!(
                Key::parse(spelling).is_none(),
                "{spelling:?} is outside the space and must not parse"
            );
        }
    }

    #[test]
    fn fr_conf_008_an_entry_name_carrying_a_dot_still_round_trips() {
        // FR-CONF-008: the entry name is a label local to the project, and
        // nothing in the corpus forbids a dot in it. The key splits at its last
        // dot so the field is never mistaken for part of the name.
        let key = Key::parse("database.a.b.host").expect("the field is the last segment");

        assert_eq!(
            key,
            Key::Entry {
                entry: "a.b".to_owned(),
                field: EntryKey::Host,
            }
        );
        assert_eq!(key.to_string(), "database.a.b.host");
    }

    #[test]
    fn fr_conf_002_each_key_declares_the_type_the_table_gives_it() {
        // FR-CONF-002, FR-CFG-010: the declared type is looked up from the key.
        assert_eq!(CoreKey::Database.expects(), ValueType::EntryName);
        assert_eq!(CoreKey::QueryTimeout.expects(), ValueType::Seconds);
        assert_eq!(EntryKey::Port.expects(), ValueType::Port);
        assert_eq!(EntryKey::Tls.expects(), ValueType::Tls);
        assert_eq!(
            EntryKey::PasswordCommand.expects(),
            ValueType::ArgumentArray
        );
        assert_eq!(EntryKey::CaFile.expects(), ValueType::Path);
    }

    #[test]
    fn fr_conf_015_expansion_reaches_exactly_the_six_fields_the_requirement_names() {
        // FR-CONF-015, FR-CONF-016, FR-CONF-017: dsn, host, port, user,
        // password and database expand; tls, password_command, ca_file and
        // ca_path do not.
        for field in [
            EntryKey::Dsn,
            EntryKey::Host,
            EntryKey::Port,
            EntryKey::User,
            EntryKey::Password,
            EntryKey::Database,
        ] {
            assert!(field.expects().expands(), "{field} expands");
        }

        for field in [
            EntryKey::Tls,
            EntryKey::PasswordCommand,
            EntryKey::CaFile,
            EntryKey::CaPath,
        ] {
            assert!(!field.expects().expands(), "{field} does not expand");
        }
    }

    #[test]
    fn fr_cfg_011_unset_accepts_a_leaf_key_and_a_whole_block() {
        // FR-CFG-011: either a leaf key, such as database.shop.host, or a whole
        // block, such as database.shop.
        assert_eq!(
            Target::parse("database.shop.host"),
            Some(Target::Key(
                Key::parse("database.shop.host").expect("the key is in the space")
            ))
        );
        assert_eq!(
            Target::parse("database.shop"),
            Some(Target::Entry("shop".to_owned()))
        );
        assert_eq!(Target::parse("core"), Some(Target::Core));
        assert_eq!(Target::parse("database"), Some(Target::Databases));
        assert_eq!(Target::parse("core.databse"), None);
        assert_eq!(Target::parse("nonsense"), None);
    }

    #[test]
    fn fr_err_021_the_candidate_population_carries_the_entry_the_supplied_key_names() {
        // FR-ERR-021: the population is the enumerated key space, which for an
        // entry key is written out per entry name — including the one the
        // misspelled key itself carries.
        let population = candidates(["reporting"], "database.shop.hst");

        assert!(population.iter().any(|key| key == "core.database"));
        assert!(population.iter().any(|key| key == "database.shop.host"));
        assert!(
            population
                .iter()
                .any(|key| key == "database.reporting.host")
        );
    }

    #[test]
    fn fr_err_021_the_candidate_population_does_not_repeat_a_defined_entry() {
        let population = candidates(["shop"], "database.shop.hst");
        let hosts = population
            .iter()
            .filter(|key| key.as_str() == "database.shop.host")
            .count();

        assert_eq!(hosts, 1);
    }
}
