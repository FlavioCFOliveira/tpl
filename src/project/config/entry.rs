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
use crate::project::secret::Redacted;

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
    /// Single and double quotes are honoured and removed. Outside any quoted
    /// run a backslash escapes the next character; inside a double-quoted run
    /// it escapes only `$`, `` ` ``, `"`, `\` and a newline — the backslash and
    /// the newline it escapes are both removed, as POSIX line continuation
    /// removes them — and before any other character it is kept, so `"a\b"`
    /// is the word `a\b`. Unquoted runs of whitespace separate words.
    /// `FR-CFG-046` is this rule's caller on the command line, and
    /// `FR-CONF-035` is why the rule is **not** applied to a string found in
    /// the file.
    ///
    /// # Errors
    ///
    /// Returns the first condition of the table of `FR-CONF-046` the string
    /// meets, in the order of that table.
    pub(crate) fn split(supplied: &str) -> Result<Self, SplitFault> {
        if supplied.chars().all(char::is_whitespace) {
            return Err(SplitFault::NoWord);
        }

        let mut words: Vec<String> = Vec::new();
        let mut word = String::new();
        let mut started = false;
        let mut characters = supplied.chars();

        while let Some(character) = characters.next() {
            match character {
                '\'' => {
                    started = true;
                    let mut closed = false;
                    for quoted in characters.by_ref() {
                        if quoted == '\'' {
                            closed = true;
                            break;
                        }
                        word.push(quoted);
                    }
                    if !closed {
                        return Err(SplitFault::UnclosedQuote);
                    }
                }
                '"' => {
                    started = true;
                    let mut closed = false;
                    while let Some(quoted) = characters.next() {
                        match quoted {
                            '"' => {
                                closed = true;
                                break;
                            }
                            '\\' => match characters.next() {
                                // POSIX 2.2.3: the backslash keeps its special
                                // meaning before these five characters only.
                                Some('\n') => {}
                                Some(escaped @ ('$' | '`' | '"' | '\\')) => word.push(escaped),
                                Some(other) => {
                                    word.push('\\');
                                    word.push(other);
                                }
                                // The quote is left open; the loop ends and
                                // reports it.
                                None => word.push('\\'),
                            },
                            other => word.push(other),
                        }
                    }
                    if !closed {
                        return Err(SplitFault::UnclosedQuote);
                    }
                }
                '\\' => match characters.next() {
                    Some(escaped) => {
                        started = true;
                        word.push(escaped);
                    }
                    // Nothing follows: the backslash escapes no character.
                    // It is reported once every quote is known to be closed,
                    // because FR-CONF-046 names an unclosed quote first.
                    None => return Err(SplitFault::TrailingBackslash),
                },
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

        // The first character as written, not the first of the first word, so
        // a quoted or escaped `[` passes.
        if supplied.trim_start().starts_with('[') {
            return Err(SplitFault::LeadingBracket);
        }

        // Every string with a character that is not whitespace yields a word
        // or met a condition above; the fallback is written rather than an
        // `expect`, so the module carries no panic.
        Self::new(words).ok_or(SplitFault::NoWord)
    }
}

/// A condition of the table of `FR-CONF-046`, under which a `password_command`
/// supplied as one string is refused rather than split.
///
/// The variants are declared in the order of that table, which is the order
/// [`PasswordCommand::split`] tests them in.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum SplitFault {
    /// The string is empty or holds only whitespace.
    NoWord,
    /// A single or a double quote is left unclosed.
    UnclosedQuote,
    /// The string ends in a backslash outside any quoted run.
    TrailingBackslash,
    /// The first character that is not whitespace is `[`.
    LeadingBracket,
}

impl SplitFault {
    /// The condition, as the `cause` of the refusal names it: a clause whose
    /// subject is the value.
    pub(crate) const fn condition(self) -> &'static str {
        match self {
            Self::NoWord => "the value holds no word, so it names no program to execute",
            Self::UnclosedQuote => {
                "the value leaves a quote unclosed, which shell quoting does not admit"
            }
            Self::TrailingBackslash => {
                "the value ends in a backslash outside quotes, which escapes no character and \
                 would be dropped"
            }
            Self::LeadingBracket => "the value begins with '[', which is how an array arrives",
        }
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
///
/// Its [`Debug`](fmt::Debug) is hand-written, because two of the ten keys carry
/// a credential as the file wrote it: `password` is one outright, and `dsn`
/// carries one in its user info. `FR-ERR-013` and `BR-ERR-003` bar both from
/// every message, and [`redact`](super::redact) applies only on the printing
/// path of `tpl cfg list` and `tpl cfg database show` — it does nothing for a
/// `{:?}`, which is how a derived implementation put the password of a live
/// invocation into the first structure that formatted this one.
#[derive(Clone, Default, PartialEq, Eq)]
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

impl fmt::Debug for Entry {
    /// Writes every key but the two that carry a credential, which are written
    /// as [`Redacted`] where the entry declares them and as `None` where it
    /// does not.
    ///
    /// Whether a key is **set** is not a secret and is what a reader of this
    /// output is looking for; what it is set to is, so the two are separated
    /// rather than the pair being dropped.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Entry")
            .field("dsn", &self.dsn.as_ref().map(|_| Redacted))
            .field("host", &self.host)
            .field("port", &self.port)
            .field("user", &self.user)
            .field("password", &self.password.as_ref().map(|_| Redacted))
            .field("password_command", &self.password_command)
            .field("database", &self.database)
            .field("tls", &self.tls)
            .field("ca_file", &self.ca_file)
            .field("ca_path", &self.ca_path)
            .finish()
    }
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
    use super::{
        Combination, Entry, Located, PasswordCommand, PortSetting, SplitFault, TlsMode, Written,
    };
    use crate::error::Position;

    fn position() -> Position {
        Position { line: 7, column: 1 }
    }

    #[test]
    fn fr_err_013_the_debug_of_an_entry_carries_neither_its_password_nor_its_dsn() {
        // Two of the ten keys carry a credential as the file wrote it, and
        // `redact` applies on the printing path of `tpl cfg list` and
        // `tpl cfg database show` alone — it does nothing for a `{:?}`, which is
        // how a derived Debug put a live invocation's password into the first
        // structure that formatted this one.
        //
        // This fails the moment either value becomes printable again.
        let entry = Entry {
            dsn: Some(Located {
                value: "mysql://alice:hunter2@db.example.com:3306/shop".to_owned(),
                position: position(),
            }),
            host: Some("db.example.com".to_owned()),
            user: Some("alice".to_owned()),
            password: Some("hunter2".to_owned()),
            ..Entry::default()
        };

        for rendered in [format!("{entry:?}"), format!("{entry:#?}")] {
            assert!(!rendered.contains("hunter2"), "{rendered}");
            assert!(!rendered.contains("mysql://"), "{rendered}");
            // Whether a key is set is not a secret, and is what a reader of
            // this output is looking for.
            assert!(rendered.contains("alice"), "{rendered}");
            assert!(rendered.contains("db.example.com"), "{rendered}");
            assert!(rendered.contains("***"), "{rendered}");
        }

        // An entry that declares neither says so, rather than saying `***` of
        // a value it does not hold.
        let bare = Entry {
            host: Some("db".to_owned()),
            ..Entry::default()
        };
        let rendered = format!("{bare:?}");

        assert!(rendered.contains("password: None"), "{rendered}");
        assert!(rendered.contains("dsn: None"), "{rendered}");
    }

    #[test]
    fn fr_conf_013_the_five_modes_are_the_five_the_requirement_names_and_the_default_is_the_strictest()
     {
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
    fn fr_conf_023_an_argument_array_is_never_empty() {
        // FR-CONF-023, FR-CONF-024: the first element is the program, and a
        // command with no program cannot be executed.
        assert_eq!(PasswordCommand::new(Vec::new()), None);

        let command = PasswordCommand::new(vec!["pass".to_owned(), "db/shop".to_owned()])
            .expect("the array carries a program");
        assert_eq!(command.program(), "pass");
        assert_eq!(command.arguments(), ["pass", "db/shop"]);
    }

    #[test]
    fn fr_conf_025_a_string_supplied_to_a_command_is_split_by_posix_quoting_rules() {
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
    fn fr_conf_025_inside_double_quotes_a_backslash_escapes_five_characters_only() {
        // FR-CONF-025, POSIX 2.2.3: before any other character the backslash
        // is kept.
        let split = |supplied: &str| {
            PasswordCommand::split(supplied)
                .expect("the string is split")
                .arguments()
                .to_vec()
        };

        assert_eq!(split(r#"get "a\b""#), ["get", r"a\b"]);
        assert_eq!(split(r#"get "C:\dir\n""#), ["get", r"C:\dir\n"]);
        assert_eq!(split(r#"get "\$ \` \" \\""#), ["get", r#"$ ` " \"#]);
        assert_eq!(split("get \"a\\\nb\""), ["get", "ab"]);
        // Outside quotes the backslash still escapes whatever follows it, and
        // inside single quotes it is an ordinary character.
        assert_eq!(split(r"get a\b"), ["get", "ab"]);
        assert_eq!(split(r"get 'a\b'"), ["get", r"a\b"]);
    }

    #[test]
    fn fr_conf_046_a_string_that_yields_no_word_is_refused() {
        assert_eq!(PasswordCommand::split(""), Err(SplitFault::NoWord));
        assert_eq!(PasswordCommand::split(" \t "), Err(SplitFault::NoWord));
    }

    #[test]
    fn fr_conf_046_a_string_that_leaves_a_quote_open_is_refused() {
        // A shell refuses an unclosed quote, and so do the POSIX quoting rules
        // FR-CONF-025 splits by.
        for supplied in ["pass 'a b", "pass \"a b", "pass \"a\\\"", "pass \"a\\"] {
            assert_eq!(
                PasswordCommand::split(supplied),
                Err(SplitFault::UnclosedQuote),
                "{supplied:?}"
            );
        }
    }

    #[test]
    fn fr_conf_046_a_trailing_backslash_outside_quotes_is_refused() {
        for supplied in [r"pass db/shop\", "\\", r"pass a\\\"] {
            assert_eq!(
                PasswordCommand::split(supplied),
                Err(SplitFault::TrailingBackslash),
                "{supplied:?}"
            );
        }
        // An escaped backslash is a character, and a quoted one is kept.
        assert!(PasswordCommand::split(r"pass a\\").is_ok());
        assert!(PasswordCommand::split(r"pass 'a\'").is_ok());
    }

    #[test]
    fn fr_conf_046_a_leading_unquoted_bracket_is_refused() {
        for supplied in [r#"["pass","db/shop"]"#, "  [x] y", "["] {
            assert_eq!(
                PasswordCommand::split(supplied),
                Err(SplitFault::LeadingBracket),
                "{supplied:?}"
            );
        }
        // Quoted, escaped, or not first: it names a program.
        for supplied in ["'[x]/get' db", r"\[x]/get db", "/bin/[ x", "get [x]"] {
            assert!(PasswordCommand::split(supplied).is_ok(), "{supplied:?}");
        }
    }

    #[test]
    fn fr_conf_046_the_first_condition_in_table_order_is_the_one_named() {
        // Unclosed quote before trailing backslash and leading bracket;
        // trailing backslash before leading bracket.
        assert_eq!(
            PasswordCommand::split("['a \\"),
            Err(SplitFault::UnclosedQuote)
        );
        assert_eq!(
            PasswordCommand::split(r"[a\"),
            Err(SplitFault::TrailingBackslash)
        );
    }

    #[test]
    fn fr_cfg_036_an_entry_reports_which_keys_it_declares_and_what_they_say() {
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
    fn fr_conf_015_a_port_written_as_a_reference_stays_a_string() {
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
    fn fr_cfg_006_a_value_renders_as_one_line_of_text_in_the_shape_the_file_would_carry() {
        assert_eq!(Written::Text("db.example.com").line(), "db.example.com");
        assert_eq!(Written::Number(3306).line(), "3306");

        let arguments = ["pass".to_owned(), "db/shop".to_owned()];
        assert_eq!(Written::List(&arguments).line(), r#"["pass", "db/shop"]"#);
    }

    #[test]
    fn fr_cfg_036_a_written_value_serialises_as_itself_and_not_as_a_wrapper() {
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
    fn fr_conf_007_the_combination_decides_the_five_rows_of_the_table() {
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
    fn fr_conf_007_the_three_keys_outside_the_table_take_no_part_in_it() {
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
    fn fr_conf_002_the_first_discrete_field_of_the_order_is_the_one_named() {
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
