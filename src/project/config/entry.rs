//! A `[database.<name>]` block: the five modes of `FR-CONF-013`, the argument
//! array of `FR-CONF-023`, and the values a key of an entry may hold.
//!
//! The block is a **typed** value and not a bag of strings. Each key of
//! `FR-CONF-002` that has an invariant carries it in its type — the TLS mode is
//! a closed enum, the `password_command` a non-empty array, the port a number
//! the file already proved to be a port — so an entry that exists is an entry
//! whose values are already the shapes the rest of the crate expects.
//!
//! Two of the keys carry **where they were written** as well as what they say,
//! and they are exactly the two whose value can still fail once the environment
//! has been read: `port`, which `FR-CONF-015` expands, and `dsn`, whose own
//! port sub-field it expands. The `78` row of `FR-ERR-034` obliges the `cause`
//! line of such a failure to name the position, and the value is the only thing
//! that knows it.
//!
//! What the type deliberately does **not** decide is whether the entry can
//! reach a server. `FR-CONF-002` gives neither `host` nor `database` a default
//! and no requirement in force refuses an entry that omits one, so both are
//! optional here and the condition belongs to the phase that opens the
//! connection.

use std::borrow::Cow;
use std::fmt;
use std::path::PathBuf;

use clap::ValueEnum;
use serde::Serialize;

use super::keys::EntryKey;
use crate::error::Position;

/// The five TLS modes of `FR-CONF-013`, defaulting to `verify-identity`.
///
/// The set is closed and `FR-CONF-036` makes it **normative over the driver**,
/// so it is one type used by the file, by the flag and by the connection
/// alike: a driver that cannot express all five is disqualified, and the way to
/// keep that a property of the code is to give the five one home. The
/// [`ValueEnum`] derive is what `tpl cfg database add --tls` and the help of
/// `FR-HELP-013` read the permitted values from.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, ValueEnum)]
pub(crate) enum TlsMode {
    /// No TLS.
    Disabled,

    /// Encrypt if the server allows it.
    Preferred,

    /// Always encrypt, without validating.
    Required,

    /// Also validate the certificate chain.
    VerifyCa,

    /// Also validate the hostname.
    ///
    /// The default of `FR-CONF-013`, and the reason `FR-SEC-021` gives for it:
    /// defaulting to a preferring mode would let an active intermediary answer
    /// "no TLS", after which the handshake carries the user and the password in
    /// clear.
    #[default]
    VerifyIdentity,
}

impl TlsMode {
    /// The five, in the order `FR-CONF-013` states them.
    pub(crate) const ALL: [Self; 5] = [
        Self::Disabled,
        Self::Preferred,
        Self::Required,
        Self::VerifyCa,
        Self::VerifyIdentity,
    ];

    /// The mode as the file and the command line spell it.
    pub(crate) const fn name(self) -> &'static str {
        match self {
            Self::Disabled => "disabled",
            Self::Preferred => "preferred",
            Self::Required => "required",
            Self::VerifyCa => "verify-ca",
            Self::VerifyIdentity => "verify-identity",
        }
    }

    /// The mode a spelling names, or [`None`] where it names none of the five.
    pub(crate) fn from_name(name: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|mode| mode.name() == name)
    }
}

impl fmt::Display for TlsMode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.name())
    }
}

/// The argument array of `FR-CONF-023`, known to carry at least a program.
///
/// `FR-CONF-024` executes it directly and without a shell, so the first element
/// is the program and there is no fallback to a shell that would run an empty
/// array as a no-op. The non-emptiness is the invariant the newtype holds.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct PasswordCommand(Vec<String>);

impl PasswordCommand {
    /// Takes an argument array, or [`None`] where it is empty.
    pub(crate) fn new(arguments: Vec<String>) -> Option<Self> {
        (!arguments.is_empty()).then_some(Self(arguments))
    }

    /// The array as stored.
    ///
    /// `FR-CONF-017` admits no `${VAR}` expansion in this key, so what is
    /// stored is what is executed and what a `cause` line may name.
    pub(crate) fn arguments(&self) -> &[String] {
        &self.0
    }

    /// The program, which is the first argument.
    pub(crate) fn program(&self) -> &str {
        // `new` is the only constructor and refuses an empty array, so the
        // fallback is unreachable; it is written as one rather than as an
        // `expect` so the module carries no panic.
        self.0.first().map_or("", String::as_str)
    }

    /// Splits a single string into the stored array by the POSIX quoting rules
    /// of `FR-CONF-025`.
    ///
    /// Single and double quotes are honoured and removed; a backslash escapes
    /// the next character outside a single-quoted run. Unquoted runs of
    /// whitespace separate words. `FR-CFG-046` is this rule's caller on the
    /// command line, and `FR-CONF-035` is why the rule is **not** applied to a
    /// string found in the file.
    ///
    /// Returns [`None`] where the string yields no word at all, which is the
    /// one outcome that cannot be executed.
    pub(crate) fn split(supplied: &str) -> Option<Self> {
        let mut words: Vec<String> = Vec::new();
        let mut word = String::new();
        let mut started = false;
        let mut characters = supplied.chars();

        while let Some(character) = characters.next() {
            match character {
                '\'' => {
                    started = true;
                    for quoted in characters.by_ref() {
                        if quoted == '\'' {
                            break;
                        }
                        word.push(quoted);
                    }
                }
                '"' => {
                    started = true;
                    while let Some(quoted) = characters.next() {
                        match quoted {
                            '"' => break,
                            '\\' => {
                                if let Some(escaped) = characters.next() {
                                    word.push(escaped);
                                }
                            }
                            other => word.push(other),
                        }
                    }
                }
                '\\' => {
                    started = true;
                    if let Some(escaped) = characters.next() {
                        word.push(escaped);
                    }
                }
                whitespace if whitespace.is_whitespace() => {
                    if started {
                        words.push(std::mem::take(&mut word));
                        started = false;
                    }
                }
                other => {
                    started = true;
                    word.push(other);
                }
            }
        }

        if started {
            words.push(word);
        }

        Self::new(words)
    }
}

/// A value together with where the file wrote it.
///
/// Only the two keys whose value can still fail after `${VAR}` expansion carry
/// one, for the reason this module's own documentation gives.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct Located<T> {
    /// The value.
    pub(crate) value: T,
    /// Where in `.tpl/.cfg` it was written.
    pub(crate) position: Position,
}

/// What the `port` key holds, as the file wrote it.
///
/// `FR-CONF-002` declares the key a TCP port and `FR-CONF-015` expands `${VAR}`
/// in it, so TOML admits it as an integer — validated where it is read — and as
/// a string, which is validated once the environment has supplied the value.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum PortSetting {
    /// An integer the file already proved to be a port.
    Fixed(u16),
    /// A string, which may carry a `${VAR}`.
    Written(String),
}

/// A value as the file wrote it, in the one of three JSON shapes it takes.
///
/// `FR-CFG-006` and `FR-CFG-036` require `tpl cfg get` to answer with the value
/// **as written**, and `FR-CFG-037` requires the same of `tpl cfg list`, so a
/// port written as an integer is a JSON number and a `password_command` is a
/// JSON array. The enum is untagged, so it serialises as the value itself and
/// not as a wrapper; it carries no map, per `FR-OUT-013`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(untagged)]
pub(crate) enum Written<'a> {
    /// A TOML string.
    Text(&'a str),
    /// A TOML integer.
    Number(u64),
    /// A TOML array of strings.
    List(&'a [String]),
}

impl<'a> Written<'a> {
    /// The value as one line of `text` output.
    ///
    /// An array is written in the TOML form `FR-CONF-035`'s own hint shows, so
    /// that what `tpl cfg get` prints for a `password_command` is what the file
    /// would carry.
    ///
    /// It takes the value rather than a reference, so that what it borrows is
    /// the document rather than this one: a caller printing a value holds it
    /// for as long as the document lives.
    pub(crate) fn line(self) -> Cow<'a, str> {
        match self {
            Self::Text(text) => Cow::Borrowed(text),
            Self::Number(number) => Cow::Owned(number.to_string()),
            Self::List(members) => {
                let mut line = String::with_capacity(members.len() * 16 + 2);
                line.push('[');
                for (index, member) in members.iter().enumerate() {
                    if index > 0 {
                        line.push_str(", ");
                    }
                    line.push('"');
                    line.push_str(member);
                    line.push('"');
                }
                line.push(']');
                Cow::Owned(line)
            }
        }
    }
}

/// One `[database.<name>]` block, with every value already typed.
///
/// Every field is optional because `FR-CONF-002` gives every key of an entry a
/// default of "none" and `FR-CFG-016` requires only that **something** was
/// supplied. What is not optional is the coherence of the set, which
/// `FR-CONF-007` fixes and the reader of the file applies.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub(crate) struct Entry {
    /// `dsn`, as written, with every `${VAR}` still literal.
    pub(crate) dsn: Option<Located<String>>,
    /// `host`, as written.
    pub(crate) host: Option<String>,
    /// `port`, as written.
    pub(crate) port: Option<Located<PortSetting>>,
    /// `user`, as written.
    pub(crate) user: Option<String>,
    /// `password`, as written.
    pub(crate) password: Option<String>,
    /// `password_command`, the array of `FR-CONF-023`.
    pub(crate) password_command: Option<PasswordCommand>,
    /// `database`, the server-side database name, as written.
    pub(crate) database: Option<String>,
    /// `tls`, which `FR-CONF-016` never expands.
    pub(crate) tls: Option<TlsMode>,
    /// `ca_file` (`FR-CONF-014`).
    pub(crate) ca_file: Option<PathBuf>,
    /// `ca_path` (`FR-CONF-014`).
    pub(crate) ca_path: Option<PathBuf>,
}

impl Entry {
    /// Whether the entry declares `field`.
    pub(crate) const fn declares(&self, field: EntryKey) -> bool {
        match field {
            EntryKey::CaFile => self.ca_file.is_some(),
            EntryKey::CaPath => self.ca_path.is_some(),
            EntryKey::Database => self.database.is_some(),
            EntryKey::Dsn => self.dsn.is_some(),
            EntryKey::Host => self.host.is_some(),
            EntryKey::Password => self.password.is_some(),
            EntryKey::PasswordCommand => self.password_command.is_some(),
            EntryKey::Port => self.port.is_some(),
            EntryKey::Tls => self.tls.is_some(),
            EntryKey::User => self.user.is_some(),
        }
    }

    /// The value of `field` as the file wrote it, or [`None`] where the entry
    /// does not declare it.
    pub(crate) fn written(&self, field: EntryKey) -> Option<Written<'_>> {
        fn text(value: Option<&String>) -> Option<Written<'_>> {
            value.map(|value| Written::Text(value.as_str()))
        }

        match field {
            EntryKey::CaFile => path(self.ca_file.as_deref()),
            EntryKey::CaPath => path(self.ca_path.as_deref()),
            EntryKey::Database => text(self.database.as_ref()),
            EntryKey::Dsn => self.dsn.as_ref().map(|dsn| Written::Text(&dsn.value)),
            EntryKey::Host => text(self.host.as_ref()),
            EntryKey::Password => text(self.password.as_ref()),
            EntryKey::PasswordCommand => self
                .password_command
                .as_ref()
                .map(|command| Written::List(command.arguments())),
            EntryKey::Port => self.port.as_ref().map(|port| match &port.value {
                PortSetting::Fixed(number) => Written::Number(u64::from(*number)),
                PortSetting::Written(text) => Written::Text(text),
            }),
            EntryKey::Tls => self.tls.map(|mode| Written::Text(mode.name())),
            EntryKey::User => text(self.user.as_ref()),
        }
    }
}

/// The combination of connection and password keys one entry declares, which
/// is the whole of what `FR-CONF-007` decides over.
///
/// It carries **whether** each key is declared and, for `dsn` alone, whether
/// that DSN carries a password — the one property of a value the rule reads.
/// The grammar that answers the second question is [`dsn`](super::dsn)'s and
/// the caller supplies the answer, so that the DSN is parsed by the code that
/// already has to parse it and this type reaches for nothing.
///
/// It exists so that **one** piece of code applies the rule to two entries: the
/// entry a file carries, which the reader validates, and the entry an
/// invocation would leave behind, which `FR-CFG-048` refuses before writing.
/// The `tls`, `ca_file` and `ca_path` keys take no part in the rule and are
/// therefore not carried.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub(crate) struct Combination {
    /// Whether the entry declares `dsn`.
    dsn: bool,
    /// Whether the DSN it declares carries a password.
    dsn_password: bool,
    /// Whether it declares `host`.
    host: bool,
    /// Whether it declares `port`.
    port: bool,
    /// Whether it declares `user`.
    user: bool,
    /// Whether it declares `password`.
    password: bool,
    /// Whether it declares `database`.
    database: bool,
    /// Whether it declares `password_command`.
    password_command: bool,
}

impl Combination {
    /// What `entry` declares, with its DSN taken to carry no password.
    ///
    /// The password inside a DSN is known only once the value has been parsed,
    /// which is [`with_dsn_password`](Self::with_dsn_password).
    pub(crate) fn of(entry: &Entry) -> Self {
        let mut combination = Self::default();

        for field in EntryKey::ALL {
            combination.declare(field, entry.declares(field));
        }

        combination
    }

    /// The same combination, with the DSN known to carry a password or not.
    pub(crate) const fn with_dsn_password(mut self, carries: bool) -> Self {
        self.dsn_password = carries;
        self
    }

    /// Records that the entry does or does not declare `field`.
    ///
    /// A field the rule does not read — `tls`, `ca_file`, `ca_path` — is
    /// accepted and recorded nowhere, so that a caller applying an invocation
    /// to a combination may hand it every key it writes without sorting them
    /// first.
    pub(crate) const fn declare(&mut self, field: EntryKey, declared: bool) {
        match field {
            EntryKey::Dsn => self.dsn = declared,
            EntryKey::Host => self.host = declared,
            EntryKey::Port => self.port = declared,
            EntryKey::User => self.user = declared,
            EntryKey::Password => self.password = declared,
            EntryKey::Database => self.database = declared,
            EntryKey::PasswordCommand => self.password_command = declared,
            EntryKey::Tls | EntryKey::CaFile | EntryKey::CaPath => {}
        }
    }

    /// The pair of keys `FR-CONF-007` refuses, or [`None`] where the
    /// combination is one of the two it admits.
    ///
    /// The three refusing rows are tested in the order the table states them,
    /// and the discrete connection field named by the first row is the first of
    /// `EntryKey::DISCRETE` the entry declares.
    pub(crate) fn refused(self) -> Option<(EntryKey, EntryKey)> {
        if self.password && self.password_command {
            return Some((EntryKey::Password, EntryKey::PasswordCommand));
        }

        if !self.dsn {
            return None;
        }

        if let Some(field) = EntryKey::DISCRETE
            .into_iter()
            .find(|field| self.declares(*field))
        {
            return Some((EntryKey::Dsn, field));
        }

        (self.dsn_password && self.password_command)
            .then_some((EntryKey::Dsn, EntryKey::PasswordCommand))
    }

    /// Whether the combination declares `field`.
    ///
    /// A field the rule does not read is never declared, per [`declare`](Self::declare).
    const fn declares(self, field: EntryKey) -> bool {
        match field {
            EntryKey::Dsn => self.dsn,
            EntryKey::Host => self.host,
            EntryKey::Port => self.port,
            EntryKey::User => self.user,
            EntryKey::Password => self.password,
            EntryKey::Database => self.database,
            EntryKey::PasswordCommand => self.password_command,
            EntryKey::Tls | EntryKey::CaFile | EntryKey::CaPath => false,
        }
    }
}

/// A path as the file wrote it, or [`None`] where it is not valid UTF-8.
///
/// A path this crate wrote is UTF-8 by construction, and one a hand-written
/// `.cfg` carries came through TOML, which is UTF-8 by definition — so the
/// fallback is unreachable and is written as one rather than as a panic.
fn path(value: Option<&std::path::Path>) -> Option<Written<'_>> {
    value.and_then(std::path::Path::to_str).map(Written::Text)
}

#[cfg(test)]
mod tests {
    use super::super::keys::EntryKey;
    use super::{Combination, Entry, Located, PasswordCommand, PortSetting, TlsMode, Written};
    use crate::error::Position;

    fn position() -> Position {
        Position { line: 7, column: 1 }
    }

    #[test]
    fn the_five_modes_are_the_five_the_requirement_names_and_the_default_is_the_strictest() {
        // FR-CONF-013, FR-SEC-021.
        let spellings: Vec<&str> = TlsMode::ALL.iter().map(|mode| mode.name()).collect();

        assert_eq!(
            spellings,
            [
                "disabled",
                "preferred",
                "required",
                "verify-ca",
                "verify-identity"
            ]
        );
        assert_eq!(TlsMode::default(), TlsMode::VerifyIdentity);
        assert_eq!(TlsMode::from_name("verify-ca"), Some(TlsMode::VerifyCa));
        assert_eq!(TlsMode::from_name("VERIFY-CA"), None);
        assert_eq!(TlsMode::from_name("off"), None);
    }

    #[test]
    fn an_argument_array_is_never_empty() {
        // FR-CONF-023, FR-CONF-024: the first element is the program, and a
        // command with no program cannot be executed.
        assert_eq!(PasswordCommand::new(Vec::new()), None);

        let command = PasswordCommand::new(vec!["pass".to_owned(), "db/shop".to_owned()])
            .expect("the array carries a program");
        assert_eq!(command.program(), "pass");
        assert_eq!(command.arguments(), ["pass", "db/shop"]);
    }

    #[test]
    fn a_string_supplied_to_a_command_is_split_by_posix_quoting_rules() {
        // FR-CONF-025, FR-CFG-046.
        let split = |supplied: &str| {
            PasswordCommand::split(supplied)
                .expect("the string yields a word")
                .arguments()
                .to_vec()
        };

        assert_eq!(
            split("security find-generic-password -s tpl -w"),
            ["security", "find-generic-password", "-s", "tpl", "-w"]
        );
        assert_eq!(
            split("/usr/local/bin/get-secret --profile 'prod eu'"),
            ["/usr/local/bin/get-secret", "--profile", "prod eu"]
        );
        assert_eq!(
            split(r#"helper --note "it's here" --flag"#),
            ["helper", "--note", "it's here", "--flag"]
        );
        assert_eq!(split(r"helper a\ b"), ["helper", "a b"]);
        assert_eq!(split("  spaced   out  "), ["spaced", "out"]);
        assert_eq!(split("empty '' tail"), ["empty", "", "tail"]);
    }

    #[test]
    fn a_string_that_yields_no_word_is_refused() {
        assert_eq!(PasswordCommand::split(""), None);
        assert_eq!(PasswordCommand::split("   "), None);
    }

    #[test]
    fn an_entry_reports_which_keys_it_declares_and_what_they_say() {
        let entry = Entry {
            host: Some("db.example.com".to_owned()),
            tls: Some(TlsMode::Required),
            port: Some(Located {
                value: PortSetting::Fixed(3306),
                position: position(),
            }),
            password_command: PasswordCommand::new(vec!["pass".to_owned()]),
            ..Entry::default()
        };

        assert!(entry.declares(EntryKey::Host));
        assert!(entry.declares(EntryKey::Tls));
        assert!(!entry.declares(EntryKey::Dsn));
        assert_eq!(
            entry.written(EntryKey::Host),
            Some(Written::Text("db.example.com"))
        );
        assert_eq!(
            entry.written(EntryKey::Tls),
            Some(Written::Text("required"))
        );
        assert_eq!(entry.written(EntryKey::Port), Some(Written::Number(3306)));
        assert_eq!(entry.written(EntryKey::Dsn), None);
    }

    #[test]
    fn a_port_written_as_a_reference_stays_a_string() {
        // FR-CONF-015 expands ${VAR} in `port`, so the key admits a string as
        // well as an integer, and `tpl cfg get` answers with what was written.
        let entry = Entry {
            port: Some(Located {
                value: PortSetting::Written("${SHOP_DB_PORT}".to_owned()),
                position: position(),
            }),
            ..Entry::default()
        };

        assert_eq!(
            entry.written(EntryKey::Port),
            Some(Written::Text("${SHOP_DB_PORT}"))
        );
    }

    #[test]
    fn a_value_renders_as_one_line_of_text_in_the_shape_the_file_would_carry() {
        assert_eq!(Written::Text("db.example.com").line(), "db.example.com");
        assert_eq!(Written::Number(3306).line(), "3306");

        let arguments = ["pass".to_owned(), "db/shop".to_owned()];
        assert_eq!(Written::List(&arguments).line(), r#"["pass", "db/shop"]"#);
    }

    #[test]
    fn a_written_value_serialises_as_itself_and_not_as_a_wrapper() {
        // FR-CFG-036: `value` is the value as written, so a port is a JSON
        // number and a password_command a JSON array.
        let arguments = ["pass".to_owned()];

        assert_eq!(
            serde_json::to_string(&Written::Text("shop")).expect("a string serialises"),
            r#""shop""#
        );
        assert_eq!(
            serde_json::to_string(&Written::Number(3306)).expect("a number serialises"),
            "3306"
        );
        assert_eq!(
            serde_json::to_string(&Written::List(&arguments)).expect("an array serialises"),
            r#"["pass"]"#
        );
    }

    #[test]
    fn the_combination_decides_the_five_rows_of_the_table() {
        // FR-CONF-007, row by row, over the predicate both the reader and the
        // writer ask: three refusals, two admissions.
        let combination = |fields: &[EntryKey], dsn_password: bool| {
            let mut combination = Combination::default();
            for field in fields {
                combination.declare(*field, true);
            }
            combination.with_dsn_password(dsn_password).refused()
        };

        assert_eq!(
            combination(&[EntryKey::Dsn, EntryKey::Host], false),
            Some((EntryKey::Dsn, EntryKey::Host)),
            "a DSN and a discrete connection field"
        );
        assert_eq!(
            combination(&[EntryKey::Dsn, EntryKey::PasswordCommand], false),
            None,
            "a DSN carrying no password, and a password command"
        );
        assert_eq!(
            combination(&[EntryKey::Dsn, EntryKey::PasswordCommand], true),
            Some((EntryKey::Dsn, EntryKey::PasswordCommand)),
            "a DSN carrying a password, and a password command"
        );
        assert_eq!(
            combination(
                &[EntryKey::Host, EntryKey::User, EntryKey::PasswordCommand],
                false
            ),
            None,
            "discrete fields and a password command"
        );
        assert_eq!(
            combination(&[EntryKey::Password, EntryKey::PasswordCommand], false),
            Some((EntryKey::Password, EntryKey::PasswordCommand)),
            "a password and a password command"
        );
    }

    #[test]
    fn the_three_keys_outside_the_table_take_no_part_in_it() {
        // FR-CONF-007 names the connection and password keys; tls, ca_file and
        // ca_path are neither, and an entry that carries them beside a DSN is
        // admitted.
        let mut combination = Combination::default();
        for field in [
            EntryKey::Dsn,
            EntryKey::Tls,
            EntryKey::CaFile,
            EntryKey::CaPath,
        ] {
            combination.declare(field, true);
        }

        assert_eq!(combination.refused(), None);
    }

    #[test]
    fn the_first_discrete_field_of_the_order_is_the_one_named() {
        // FR-CONF-002 fixes the order, and the pair reported is the first field
        // of it the entry declares, so the message is the same for one file
        // however the keys were written.
        let mut combination = Combination::default();
        for field in [EntryKey::Dsn, EntryKey::User, EntryKey::Host] {
            combination.declare(field, true);
        }

        assert_eq!(combination.refused(), Some((EntryKey::Dsn, EntryKey::Host)));
    }
}
