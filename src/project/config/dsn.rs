//! The DSN grammar of `FR-CONF-009`, the two schemes of `FR-CONF-010`, and the
//! refusal of `FR-CONF-011`.
//!
//! ```text
//! scheme://[user[:password]@]host[:port]/database
//! ```
//!
//! A DSN is **parsed into its fields and kept that way**. `FR-CONF-018` fixes
//! the order in which a `${VAR}` inside one is handled — parse the URL, expand
//! within the already-delimited field, then percent-encode — and this module
//! delivers the first and third of those steps: [`parse`] splits the raw text
//! while every reference is still literal, and [`Field::encode`] percent-encodes
//! an expanded value for the one consumer that has to put it back into a URL.
//! Nothing here re-composes a URL of its own, so an expanded value has no
//! delimiter to move: the threat `FR-SEC-009` closes is closed by the shape of
//! the type rather than by the order of two calls.
//!
//! **The refusal of a query parameter comes first.** `FR-CONF-011` refuses a
//! `?` "whatever follows it" and `FR-CONF-012` makes a TLS parameter a special
//! case of the same rule, because `BR-CONF-001` makes the `tls` key the sole
//! authority on encryption. A DSN carrying both a `?` and an unaccepted scheme
//! is therefore reported as the parameter, which is the condition with the
//! security consequence.

use std::borrow::Cow;
use std::path::Path;

use crate::error::{DsnFault, Error};

/// The separator between the scheme and the authority.
const SCHEME_SEPARATOR: &str = "://";

/// The two schemes `FR-CONF-010` accepts, treated as equivalent.
const SCHEMES: [&str; 2] = ["mysql", "mariadb"];

/// The characters a percent-encoded field leaves unescaped.
///
/// The unreserved set of RFC 3986, and nothing else: every delimiter of the
/// grammar above — `:`, `@`, `/`, `?`, `#`, `[`, `]` — is therefore escaped,
/// which is what makes `FR-CONF-018`'s third step the guarantee `FR-SEC-009`
/// names.
const fn unreserved(byte: u8) -> bool {
    byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'.' | b'_' | b'~')
}

/// One field of a parsed DSN, still carrying whatever `${VAR}` the file wrote.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct Field<'a>(&'a str);

impl<'a> Field<'a> {
    /// The field as the file wrote it, references and all.
    pub(crate) const fn raw(self) -> &'a str {
        self.0
    }

    /// `value` percent-encoded for the position this field occupies in a URL.
    ///
    /// This is the third step of `FR-CONF-018`, and it is applied to the
    /// **expanded** value rather than to the raw one: an expansion that
    /// produced `x@attacker.example.com/shop?#` comes back as
    /// `x%40attacker.example.com%2Fshop%3F%23`, which cannot move the host, the
    /// port or the database.
    ///
    /// A value with nothing to escape is returned borrowed.
    #[allow(
        dead_code,
        reason = "the connection that composes a URL for the driver is a later sprint; \
                  FR-CONF-018 fixes the encoding as the third step of parsing a DSN, so it is \
                  stated beside the first, and a test exercises the threat it closes"
    )]
    pub(crate) fn encode(value: &str) -> Cow<'_, str> {
        if value.bytes().all(unreserved) {
            return Cow::Borrowed(value);
        }

        let mut encoded = String::with_capacity(value.len() * 3);
        for byte in value.bytes() {
            if unreserved(byte) {
                encoded.push(char::from(byte));
            } else {
                encoded.push('%');
                encoded.push_str(&format!("{byte:02X}"));
            }
        }

        Cow::Owned(encoded)
    }
}

/// A DSN split into the fields of `FR-CONF-009`, before any expansion.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct Dsn<'a> {
    /// The value as the file wrote it, which every field borrows from.
    raw: &'a str,
    /// The user, where the authority carries one.
    user: Option<Field<'a>>,
    /// The password, where the user info carries one.
    password: Option<Field<'a>>,
    /// The host, which the grammar requires.
    host: Field<'a>,
    /// The port, where the authority carries one.
    port: Option<Field<'a>>,
    /// The server-side database, which the grammar requires.
    database: Field<'a>,
}

impl<'a> Dsn<'a> {
    /// The user the DSN names.
    pub(crate) const fn user(&self) -> Option<Field<'a>> {
        self.user
    }

    /// The password the DSN carries.
    ///
    /// `FR-CONF-007` refuses a DSN carrying one beside a `password_command`,
    /// which is the one question this accessor is asked at load time.
    pub(crate) const fn password(&self) -> Option<Field<'a>> {
        self.password
    }

    /// The host the DSN names.
    pub(crate) const fn host(&self) -> Field<'a> {
        self.host
    }

    /// The port the DSN names.
    pub(crate) const fn port(&self) -> Option<Field<'a>> {
        self.port
    }

    /// The server-side database the DSN names.
    pub(crate) const fn database(&self) -> Field<'a> {
        self.database
    }

    /// The DSN with its password replaced by `replacement`, or [`None`] where
    /// it carries none.
    ///
    /// `FR-CFG-021` prints the password inside a DSN as `***` and leaves the
    /// user, the host, the port and the database visible, which is exactly a
    /// substitution over the one field. It is composed here, where the grammar
    /// is, rather than by a second splitter in the printing path.
    pub(crate) fn with_password(&self, replacement: &str) -> Option<String> {
        let password = self.password?;
        let at = offset(self.raw, password.0);

        let mut composed = String::with_capacity(self.raw.len() + replacement.len());
        composed.push_str(self.raw.get(..at).unwrap_or_default());
        composed.push_str(replacement);
        composed.push_str(self.raw.get(at + password.0.len()..).unwrap_or_default());

        Some(composed)
    }
}

/// Where a borrowed subslice begins inside the value it was taken from.
///
/// Both are slices of the same allocation, so the difference of their addresses
/// is the byte offset. It is arithmetic over two `usize` and needs no `unsafe`;
/// the saturation is for a caller that passed an unrelated slice, which would
/// otherwise underflow rather than be caught.
fn offset(whole: &str, part: &str) -> usize {
    (part.as_ptr() as usize).saturating_sub(whole.as_ptr() as usize)
}

/// Splits a DSN into its fields, refusing what `FR-CONF-009` through
/// `FR-CONF-012` refuse.
///
/// `key` is the fully qualified key whose value this is and `file` the file
/// that declares it. Neither the DSN nor any part of it enters the reported
/// condition: `BR-ERR-003` bars the resolved DSN from every message, and the
/// key locates the fault without it.
///
/// # Errors
///
/// Returns [`Error::DsnQueryParameter`] where the value carries a `?`
/// (`FR-CONF-011`, `FR-CONF-012`), and [`Error::DsnMalformed`] where the scheme
/// is not one of the two of `FR-CONF-010` or the remainder does not have the
/// form of `FR-CONF-009`.
pub(crate) fn parse<'a>(raw: &'a str, key: &str, file: &Path) -> Result<Dsn<'a>, Error> {
    if raw.contains('?') {
        return Err(Error::DsnQueryParameter {
            key: key.to_owned(),
            file: file.to_owned(),
        });
    }

    let malformed = |fault| Error::DsnMalformed {
        key: key.to_owned(),
        file: file.to_owned(),
        fault,
    };

    let Some((scheme, rest)) = raw.split_once(SCHEME_SEPARATOR) else {
        return Err(malformed(DsnFault::Form));
    };

    if !SCHEMES.contains(&scheme) {
        return Err(malformed(DsnFault::Scheme));
    }

    // The database is everything after the first `/` of the remainder, so a
    // `/` inside it — which the grammar does not admit — is refused with the
    // rest of the form rather than silently kept.
    let Some((authority, database)) = rest.split_once('/') else {
        return Err(malformed(DsnFault::Form));
    };

    if database.is_empty() || database.contains('/') {
        return Err(malformed(DsnFault::Form));
    }

    // The last `@` separates the user info from the host: `FR-CFG-031` stores
    // a DSN verbatim, so a literal password carrying an `@` reaches here
    // unencoded and the first `@` would cut it in half.
    let (userinfo, hostport) = match authority.rsplit_once('@') {
        Some((userinfo, hostport)) => (Some(userinfo), hostport),
        None => (None, authority),
    };

    // FR-CONF-009 puts the user before the optional password, so an empty user
    // info and an empty user are both outside the grammar: a password with
    // nobody to authenticate as is not a form the server can be given.
    let (user, password) = match userinfo {
        None => (None, None),
        Some("") => return Err(malformed(DsnFault::Form)),
        Some(userinfo) => match userinfo.split_once(':') {
            Some(("", _)) => return Err(malformed(DsnFault::Form)),
            Some((user, password)) => (Some(Field(user)), Some(Field(password))),
            None => (Some(Field(userinfo)), None),
        },
    };

    let (host, port) = split_host(hostport).ok_or_else(|| malformed(DsnFault::Form))?;

    Ok(Dsn {
        raw,
        user,
        password,
        host,
        port,
        database: Field(database),
    })
}

/// Splits `host[:port]`, honouring the bracketed form an IPv6 literal takes.
///
/// Returns [`None`] where the host is empty or the brackets do not close, which
/// the caller reports as the form of `FR-CONF-009`.
fn split_host(hostport: &str) -> Option<(Field<'_>, Option<Field<'_>>)> {
    if let Some(rest) = hostport.strip_prefix('[') {
        let end = rest.find(']')?;
        let host = &rest[..end];
        let after = &rest[end + 1..];

        if host.is_empty() {
            return None;
        }

        return match after {
            "" => Some((Field(hostport), None)),
            _ => {
                let port = after.strip_prefix(':')?;
                (!port.is_empty()).then(|| (Field(&hostport[..end + 2]), Some(Field(port))))
            }
        };
    }

    match hostport.rsplit_once(':') {
        Some((host, port)) => {
            (!host.is_empty() && !port.is_empty()).then_some((Field(host), Some(Field(port))))
        }
        None => (!hostport.is_empty()).then_some((Field(hostport), None)),
    }
}

#[cfg(test)]
mod tests {
    use super::{Dsn, Field, parse};
    use crate::error::{DsnFault, Error};
    use std::path::{Path, PathBuf};

    /// The key every condition below names.
    const KEY: &str = "database.shop.dsn";

    fn file() -> PathBuf {
        PathBuf::from("/work/.tpl/.cfg")
    }

    fn parsed(raw: &str) -> Dsn<'_> {
        parse(raw, KEY, &file()).expect("the DSN is of the form the grammar fixes")
    }

    fn refused(raw: &str) -> Error {
        parse(raw, KEY, &file()).expect_err("the DSN is refused")
    }

    #[test]
    fn the_full_form_splits_into_its_five_fields() {
        // FR-CONF-009: scheme://[user[:password]@]host[:port]/database.
        let dsn = parsed("mysql://alice:hunter2@db.example.com:3306/shop");

        assert_eq!(dsn.user().map(Field::raw), Some("alice"));
        assert_eq!(dsn.password().map(Field::raw), Some("hunter2"));
        assert_eq!(dsn.host().raw(), "db.example.com");
        assert_eq!(dsn.port().map(Field::raw), Some("3306"));
        assert_eq!(dsn.database().raw(), "shop");
    }

    #[test]
    fn every_optional_part_of_the_grammar_is_optional() {
        let bare = parsed("mysql://db.example.com/shop");
        assert_eq!(bare.user(), None);
        assert_eq!(bare.password(), None);
        assert_eq!(bare.port(), None);

        let user_only = parsed("mysql://alice@db.example.com/shop");
        assert_eq!(user_only.user().map(Field::raw), Some("alice"));
        assert_eq!(user_only.password(), None);
    }

    #[test]
    fn both_schemes_are_accepted_and_are_equivalent() {
        // FR-CONF-010: the two are treated as equivalent, so the fields they
        // yield are the same. The values themselves differ in the raw text each
        // borrows from, which is what the redaction of FR-CFG-021 splices into.
        let mysql = parsed("mysql://alice@db.example.com:3306/shop");
        let mariadb = parsed("mariadb://alice@db.example.com:3306/shop");

        assert_eq!(mysql.user(), mariadb.user());
        assert_eq!(mysql.password(), mariadb.password());
        assert_eq!(mysql.host(), mariadb.host());
        assert_eq!(mysql.port(), mariadb.port());
        assert_eq!(mysql.database(), mariadb.database());
    }

    #[test]
    fn a_scheme_outside_the_two_is_refused_as_the_scheme() {
        // FR-CONF-010: the `cause` line separates the scheme from the form,
        // because the two have different next steps.
        for raw in [
            "postgres://db.example.com/shop",
            "MySQL://db.example.com/shop",
            "://db.example.com/shop",
        ] {
            match refused(raw) {
                Error::DsnMalformed { fault, .. } => {
                    assert_eq!(fault, DsnFault::Scheme, "{raw}");
                }
                other => panic!("expected a scheme fault for {raw}, got {other:?}"),
            }
        }
    }

    #[test]
    fn a_value_that_is_not_of_the_form_is_refused_as_the_form() {
        // FR-CONF-009: the host and the database are both required.
        for raw in [
            "db.example.com/shop",
            "mysql://db.example.com",
            "mysql://db.example.com/",
            "mysql:///shop",
            "mysql://@db.example.com/shop",
            "mysql://:hunter2@db.example.com/shop",
            "mysql://db.example.com:/shop",
            "mysql://db.example.com/shop/extra",
        ] {
            match refused(raw) {
                Error::DsnMalformed { fault, .. } => assert_eq!(fault, DsnFault::Form, "{raw}"),
                other => panic!("expected a form fault for {raw}, got {other:?}"),
            }
        }
    }

    #[test]
    fn a_query_parameter_is_refused_whatever_follows_it() {
        // FR-CONF-011, FR-CONF-012: a `?` is 78, and a TLS parameter is the
        // special case that would otherwise contradict BR-CONF-001.
        for raw in [
            "mysql://db.example.com/shop?charset=utf8",
            "mysql://db.example.com/shop?tls=false",
            "mysql://db.example.com/shop?",
        ] {
            let condition = refused(raw);
            assert!(
                matches!(condition, Error::DsnQueryParameter { ref key, .. } if key == KEY),
                "{raw}: {condition:?}"
            );
            assert_eq!(condition.exit_code(), 78);
        }
    }

    #[test]
    fn the_parameter_refusal_is_reached_before_the_scheme_is_judged() {
        // FR-CONF-011 refuses a `?` "whatever follows it", and the parameter is
        // the condition with the security consequence BR-CONF-001 names.
        assert!(matches!(
            refused("postgres://db.example.com/shop?tls=false"),
            Error::DsnQueryParameter { .. }
        ));
    }

    #[test]
    fn a_literal_password_carrying_an_at_sign_is_not_cut_in_half() {
        // FR-CFG-031 stores a DSN verbatim, so the authority is split at its
        // last `@` rather than its first.
        let dsn = parsed("mysql://alice:p@ss@db.example.com/shop");

        assert_eq!(dsn.user().map(Field::raw), Some("alice"));
        assert_eq!(dsn.password().map(Field::raw), Some("p@ss"));
        assert_eq!(dsn.host().raw(), "db.example.com");
    }

    #[test]
    fn a_reference_survives_the_parse_as_written() {
        // FR-CONF-018: the URL is parsed first and the reference is expanded
        // inside the already-delimited field, so the parser never sees an
        // expanded value.
        let dsn = parsed("mysql://alice:${SHOP_DB_PASSWORD}@db.example.com:${PORT}/shop");

        assert_eq!(dsn.password().map(Field::raw), Some("${SHOP_DB_PASSWORD}"));
        assert_eq!(dsn.port().map(Field::raw), Some("${PORT}"));
        assert_eq!(dsn.host().raw(), "db.example.com");
    }

    #[test]
    fn an_ipv6_literal_keeps_its_brackets_and_its_port() {
        let dsn = parsed("mysql://[2001:db8::1]:3307/shop");

        assert_eq!(dsn.host().raw(), "[2001:db8::1]");
        assert_eq!(dsn.port().map(Field::raw), Some("3307"));

        let no_port = parsed("mysql://[2001:db8::1]/shop");
        assert_eq!(no_port.host().raw(), "[2001:db8::1]");
        assert_eq!(no_port.port(), None);
    }

    #[test]
    fn percent_encoding_denies_an_expanded_value_every_delimiter() {
        // FR-CONF-018, FR-SEC-009: the threat is
        // SHOP_PW=x@attacker.example.com/shop?# redirecting the connection.
        let encoded = Field::encode("x@attacker.example.com/shop?#");

        assert_eq!(encoded, "x%40attacker.example.com%2Fshop%3F%23");
        for delimiter in ['@', '/', '?', '#', ':', '[', ']'] {
            assert!(
                !encoded.contains(delimiter),
                "{encoded} carries {delimiter}"
            );
        }
    }

    #[test]
    fn the_password_inside_a_dsn_is_replaced_and_nothing_else_is() {
        // FR-CFG-021: `***`, with user, host, port and database left visible.
        let raw = "mysql://alice:hunter2@db.example.com:3306/shop";
        let redacted = parsed(raw)
            .with_password("***")
            .expect("the DSN carries a password");

        assert_eq!(redacted, "mysql://alice:***@db.example.com:3306/shop");
        assert!(!redacted.contains("hunter2"));
    }

    #[test]
    fn a_dsn_with_no_password_has_nothing_to_replace() {
        assert_eq!(
            parsed("mysql://alice@db.example.com/shop").with_password("***"),
            None
        );
        assert_eq!(
            parsed("mysql://db.example.com/shop").with_password("***"),
            None
        );
    }

    #[test]
    fn a_value_with_nothing_to_escape_is_returned_borrowed() {
        let encoded = Field::encode("hunter2");

        assert_eq!(encoded, "hunter2");
        assert!(matches!(encoded, std::borrow::Cow::Borrowed(_)));
    }

    #[test]
    fn the_condition_names_the_key_and_the_file_and_never_the_value() {
        // BR-ERR-003: the resolved DSN reaches no message.
        let condition = parse(
            "postgres://alice:hunter2@db.example.com/shop",
            KEY,
            Path::new("/work/.tpl/.cfg"),
        )
        .expect_err("the scheme is refused");

        let rendered = format!("{condition}");
        assert!(!rendered.contains("hunter2"), "{rendered}");
        assert!(rendered.contains(KEY), "{rendered}");
    }
}
