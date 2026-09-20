//! The one connection an invocation opens, and everything the driver is told
//! before it is opened.
//!
//! `NFR-PERF-004` allows one connection per invocation and `ADR-003` uses the
//! driver without a pool, so this module opens exactly one socket and hands it
//! back. Two things are settled here and nowhere else: what the driver is
//! configured with, and how the two phases before the session exists are
//! bounded and attributed.
//!
//! | Obligation | What this module does with it |
//! |---|---|
//! | `FR-CONF-037` — the TLS mode is set explicitly on every connection | [`ssl_mode`] maps all five, and the driver's own default is never reached |
//! | `FR-CONF-013`, `ADR-002` — five modes, five variants, none collapsed | The mapping is one-to-one and total, and a test reads it back |
//! | `FR-CONF-014` — `ca_file` and `ca_path` supply the trust material | [`trust`] assembles both into one bundle, under the two modes that validate |
//! | `FR-CONF-039` — supplied material is **additional** to the bundled roots | Nothing here substitutes: the driver adds what it is given, per `ADR-002` |
//! | `FR-SRV-006`, `FR-SRV-007` — a closed statement list | Every option that would make the driver issue a statement of its own is turned off; see below |
//! | `FR-CONF-005`, `OD-12` — DNS resolution and the connect are separate phases | Both run under one shared budget, each reporting its own phase |
//!
//! **The driver issues a statement of its own unless it is told not to, and
//! that statement is outside the closed list.** `sqlx` 0.9.0 follows the
//! handshake with a single `SET` composed from four of its options — two
//! `sql_mode` flags, a session time zone, and `NAMES` — each of which defaults
//! to on. `FR-SRV-007` admits no such statement and `FR-SRV-006` closes the
//! list at four kinds, so all four options are set to the values that leave
//! that `SET` unissued. The character set is unaffected: the driver sends
//! `utf8mb4` in the handshake packet itself, which is what it falls back to
//! when `NAMES` is not issued.
//!
//! **The name is resolved here, and the driver is still given the name.**
//! `FR-CONF-005` and `FR-ERR-034` oblige the `cause` line to say that a name
//! did not resolve rather than that a host refused a connection, and that
//! distinction cannot be recovered from a driver call that resolves
//! internally — so `tpl` resolves first, and a resolution that yields no
//! address is `FR-ERR-001`'s `69` naming DNS. What the driver receives
//! afterwards is the **configured host**, not the address the resolution
//! produced. `OD-12` records the opposite, and the reason it cannot stand is
//! `FR-CONF-038`: the driver takes the TLS server name from the host it is
//! given, so a driver handed an address would validate the address under
//! `verify-identity` — the default mode of `FR-CONF-013` — and a certificate
//! naming a host would fail against every server reached by name. The
//! duplicated lookup is a second resolution and not a second connection, so
//! `NFR-PERF-004` is untouched.

use std::path::Path;

use sqlx::Connection as _;
use sqlx::mysql::{MySqlConnectOptions, MySqlConnection, MySqlSslMode};
use tokio::net::lookup_host;
use tokio::runtime::Runtime;
use tokio::time::timeout;

use super::fault;
use crate::deadline::{Clock, Deadlines, Phase};
use crate::error::{Error, NetworkPhase};
use crate::project::config::entry::TlsMode;
use crate::project::secret::Secret;
use crate::project::settings::Settings;

/// The brackets an IPv6 literal carries in the authority of a DSN.
const BRACKETS: [char; 2] = ['[', ']'];

/// Everything the driver is told, resolved and already complete.
///
/// The host is **not** optional, and that is the one thing this type decides.
/// `FR-CONF-002` gives `database.<name>.host` no default and `FR-CFG-016`
/// admits an entry that supplies any one key, so an entry that names no host
/// is legal and describes no connection. The refusal cannot be composed here:
/// `FR-ERR-034` obliges the `cause` line of a `78` naming a key to name the
/// file and the position that declared it, and neither is a thing this module
/// holds. [`Target::of`] therefore answers [`None`], the condition stays with
/// the layer that has the file open, and the gap is reported rather than
/// filled with a message this module would have to invent.
#[derive(Debug)]
pub(crate) struct Target<'a> {
    /// The database entry these settings came from, which every condition of
    /// `FR-SRV-010`, `FR-SRV-003` and `FR-SRV-030` names.
    entry: &'a str,
    /// The host, with an IPv6 literal's brackets removed.
    host: &'a str,
    /// The port, defaulted to `3306` by `FR-CONF-002` before it reaches here.
    port: u16,
    /// The user, where the entry names one.
    user: Option<&'a str>,
    /// The password, where the entry supplies one.
    password: Option<&'a Secret>,
    /// The server-side database, where the entry names one.
    database: Option<&'a str>,
    /// The TLS mode of `FR-CONF-013`, already defaulted.
    tls: TlsMode,
    /// `ca_file` (`FR-CONF-014`).
    ca_file: Option<&'a Path>,
    /// `ca_path` (`FR-CONF-014`).
    ca_path: Option<&'a Path>,
    /// The four phase deadlines of `FR-CONF-004`.
    deadlines: Deadlines,
}

impl<'a> Target<'a> {
    /// The target `settings` describes, or [`None`] where it names no host.
    pub(crate) fn of(settings: &'a Settings) -> Option<Self> {
        Some(Self {
            entry: settings.entry(),
            // The driver's tokio path connects through `(&str, u16)`, which
            // parses an address literal and otherwise resolves a name; a
            // bracketed IPv6 literal is neither, and would be looked up as a
            // host name and fail. The brackets are the DSN grammar's, per
            // `FR-CONF-009`, and they are removed once, here.
            host: settings.host()?.trim_matches(BRACKETS),
            port: settings.port(),
            user: settings.user(),
            password: settings.password(),
            database: settings.database(),
            tls: settings.tls(),
            ca_file: settings.ca_file(),
            ca_path: settings.ca_path(),
            deadlines: settings.deadlines(),
        })
    }

    /// The database entry these settings came from.
    pub(crate) const fn entry(&self) -> &'a str {
        self.entry
    }

    /// The host the connection is made to.
    pub(crate) const fn host(&self) -> &'a str {
        self.host
    }

    /// The port the connection is made to.
    pub(crate) const fn port(&self) -> u16 {
        self.port
    }

    /// The four phase deadlines this connection's statements are bounded by.
    pub(crate) const fn deadlines(&self) -> Deadlines {
        self.deadlines
    }
}

/// The driver variant each mode of `FR-CONF-013` is expressed as.
///
/// The mapping is `ADR-002`'s, and it is one-to-one and total in both
/// directions: five modes, five variants, no mode collapsed onto another and
/// no variant left unnamed. That is what `FR-CONF-036` requires of the driver,
/// and stating it as a total match is what keeps it so — a mode added to
/// `FR-CONF-013` is a compile error here rather than a mode silently reading
/// as another.
const fn ssl_mode(mode: TlsMode) -> MySqlSslMode {
    match mode {
        TlsMode::Disabled => MySqlSslMode::Disabled,
        TlsMode::Preferred => MySqlSslMode::Preferred,
        TlsMode::Required => MySqlSslMode::Required,
        TlsMode::VerifyCa => MySqlSslMode::VerifyCa,
        TlsMode::VerifyIdentity => MySqlSslMode::VerifyIdentity,
    }
}

/// The trust material of `FR-CONF-014`, as one PEM bundle.
///
/// `ca_file` is one file and `ca_path` is a directory of them, and the driver
/// affords a file and a byte slice and no directory — so both keys are read
/// here and concatenated, and the bundle is handed over as bytes. The order is
/// the file first and then the directory's entries by name ascending, so that
/// one configuration always produces one bundle; `NFR-DET-001` promises the
/// same invocation the same result, and a bundle assembled in directory order
/// would not be the same twice.
///
/// Only the two modes `FR-CONF-014` names read it. Under the other three the
/// driver ignores what it is given — `FR-CONF-038` records `required` with
/// trust material ignoring it as one of the three controls that separate the
/// modes — and reading a file whose contents cannot affect the connection
/// would turn an unreadable path into a failure the mode does not have.
///
/// `FR-CONF-039` is unaffected and is the driver's: what is supplied is added
/// to the roots the TLS implementation already trusts, never substituted for
/// them.
///
/// # Errors
///
/// Returns [`Error::ProjectFileUnreadable`] naming the path the configuration
/// declared, where the file or the directory cannot be read.
fn trust(target: &Target<'_>) -> Result<Option<Vec<u8>>, Error> {
    if !matches!(target.tls, TlsMode::VerifyCa | TlsMode::VerifyIdentity) {
        return Ok(None);
    }

    let mut bundle = Vec::new();

    if let Some(file) = target.ca_file {
        bundle.extend_from_slice(&read(file)?);
    }

    if let Some(directory) = target.ca_path {
        let mut paths = Vec::new();

        for entry in unreadable(directory, std::fs::read_dir(directory))? {
            let entry = unreadable(directory, entry)?;

            if unreadable(&entry.path(), entry.file_type())?.is_file() {
                paths.push(entry.path());
            }
        }

        paths.sort();

        for path in paths {
            bundle.extend_from_slice(&read(&path)?);
            // A bundle is a concatenation of PEM blocks, and a file whose last
            // line carries no terminator would otherwise run into the next
            // file's first line.
            bundle.push(b'\n');
        }
    }

    Ok((!bundle.is_empty()).then_some(bundle))
}

/// The bytes of `path`, or the condition `FR-ERR-001` puts an unreadable file
/// on.
fn read(path: &Path) -> Result<Vec<u8>, Error> {
    unreadable(path, std::fs::read(path))
}

/// `outcome`, or the condition an unreadable path produces, naming `path`.
fn unreadable<T>(path: &Path, outcome: std::io::Result<T>) -> Result<T, Error> {
    outcome.map_err(|returned| Error::ProjectFileUnreadable {
        path: path.to_owned(),
        returned,
    })
}

/// Everything the driver is told, composed once.
///
/// # Errors
///
/// Returns what [`trust`] returns for trust material that cannot be read.
fn options(target: &Target<'_>) -> Result<MySqlConnectOptions, Error> {
    let mut options = MySqlConnectOptions::new()
        .host(target.host)
        .port(target.port)
        .ssl_mode(ssl_mode(target.tls))
        // The four options that would otherwise make the driver issue a `SET`
        // of its own after the handshake, which `FR-SRV-007` admits no more
        // than any other statement outside the closed list of `FR-SRV-006`.
        .pipes_as_concat(false)
        .no_engine_substitution(false)
        .timezone(None::<String>)
        .set_names(false);

    if let Some(user) = target.user {
        options = options.username(user);
    }

    if let Some(password) = target.password {
        options = options.password(password.expose());
    }

    if let Some(database) = target.database {
        options = options.database(database);
    }

    if let Some(bundle) = trust(target)? {
        options = options.ssl_ca_from_pem(bundle);
    }

    Ok(options)
}

/// Opens the one connection of `NFR-PERF-004`.
///
/// The two phases before a session exists are bounded by the one budget
/// `FR-CONF-005` gives the three connection phases: the instant is taken once,
/// and DNS resolution and the connect consume it in the order they run, so the
/// second of them may find nothing left. The TLS handshake is inside the
/// second call and is not separable from it — `MySqlConnectOptions` has no
/// method that accepts an already-connected socket — and is separated in the
/// report instead, by the driver's own discriminant, per `OD-12`.
///
/// # Errors
///
/// Returns [`Error::NameNotResolved`] where the host resolves to no address,
/// [`Error::NetworkDeadlineExceeded`] where either phase outlives the budget,
/// and what [`fault::connecting`] classifies for a driver failure.
pub(super) fn open(
    runtime: &Runtime,
    target: &Target<'_>,
    clock: &Clock,
) -> Result<MySqlConnection, Error> {
    let options = options(target)?;
    let budget = clock.connection_budget(target.deadlines.of(Phase::TcpConnect));
    let host = target.host;
    let port = target.port;

    runtime.block_on(async {
        let bound = budget.remaining();

        if bound.expired() {
            return Err(fault::expired(
                NetworkPhase::DnsResolution,
                host,
                port,
                bound,
            ));
        }

        let resolved = match timeout(bound.remaining(), lookup_host((host, port))).await {
            Err(_) => {
                return Err(fault::expired(
                    NetworkPhase::DnsResolution,
                    host,
                    port,
                    bound,
                ));
            }
            Ok(resolved) => resolved,
        };

        // A name that resolves to nothing and a name the resolver refuses are
        // one condition: no address was produced, so the connection was never
        // attempted, which is what the `cause` of `FR-ERR-034`'s `69` row says
        // of this phase.
        if !resolved.is_ok_and(|mut addresses| addresses.next().is_some()) {
            return Err(Error::NameNotResolved {
                host: host.to_owned(),
                port,
            });
        }

        let bound = budget.remaining();

        if bound.expired() {
            return Err(fault::expired(NetworkPhase::TcpConnect, host, port, bound));
        }

        match timeout(bound.remaining(), MySqlConnection::connect_with(&options)).await {
            Err(_) => Err(fault::expired(NetworkPhase::TcpConnect, host, port, bound)),
            Ok(Ok(connection)) => Ok(connection),
            // The database is the one the entry names, per `FR-CONF-041`, and
            // it is passed because the handshake is where a packet about it
            // arrives: the connection carries the database, so a server that
            // will not show it refuses the connection rather than a statement.
            Ok(Err(driver)) => Err(fault::connecting(
                &driver,
                host,
                port,
                options.get_username(),
                target.database,
            )),
        }
    })
}

#[cfg(test)]
mod tests {
    use super::{Target, options, ssl_mode, trust};
    use crate::project::config;
    use crate::project::config::entry::TlsMode;
    use crate::project::scratch::Scratch;
    use crate::project::settings::{self, Settings};
    use sqlx::mysql::MySqlSslMode;

    /// A lookup that defines nothing, which every document below is written
    /// to need nothing from.
    fn nothing(_: &str) -> Option<String> {
        None
    }

    /// The settings a `.tpl/.cfg` of `text` resolves to for entry `shop`.
    fn settings(scratch: &Scratch, text: &str) -> Settings {
        let file = scratch.file(".cfg", text);
        let configuration = config::load(&file).expect("the document is valid");

        settings::resolve(
            &configuration,
            Some("shop"),
            &settings::clock(None),
            &nothing,
        )
        .expect("the entry resolves")
    }

    /// A driver variant as a value two of them can be compared by.
    ///
    /// `MySqlSslMode` derives `Debug` and no equality, so the variant's own
    /// name is what a test has to compare — which is enough here, because the
    /// five names are distinct and a mapping that collapsed two modes would
    /// produce one name twice.
    fn named(mode: MySqlSslMode) -> String {
        format!("{mode:?}")
    }

    #[test]
    fn fr_conf_013_the_five_modes_map_onto_five_distinct_driver_variants() {
        // FR-CONF-036 and ADR-002: the mapping is one-to-one and total, and no
        // mode is collapsed onto another.
        let mapped = TlsMode::ALL.map(ssl_mode).map(named);

        assert_eq!(
            mapped,
            [
                "Disabled".to_owned(),
                "Preferred".to_owned(),
                "Required".to_owned(),
                "VerifyCa".to_owned(),
                "VerifyIdentity".to_owned(),
            ]
        );
    }

    #[test]
    fn fr_conf_037_the_mode_is_set_explicitly_including_the_one_the_driver_defaults_to() {
        // FR-CONF-037: the driver's own default is MySqlSslMode::Preferred,
        // and it is never relied on — not even for `disabled`, and not even
        // where the entry asks for the mode the driver would have chosen.
        let scratch = Scratch::new();

        for mode in TlsMode::ALL {
            let resolved = settings(
                &scratch,
                &format!(
                    "[database.shop]\nhost = \"db\"\ntls = \"{}\"\n",
                    mode.name()
                ),
            );
            let target = Target::of(&resolved).expect("the entry names a host");
            let composed = options(&target).expect("no trust material is declared");

            assert_eq!(
                named(composed.get_ssl_mode()),
                named(ssl_mode(mode)),
                "{mode}"
            );
        }
    }

    #[test]
    fn fr_conf_002_the_target_carries_the_entry_and_the_defaulted_port() {
        let scratch = Scratch::new();
        let resolved = settings(
            &scratch,
            "[database.shop]\nhost = \"db.example.com\"\nuser = \"alice\"\ndatabase = \"freight\"\n",
        );
        let target = Target::of(&resolved).expect("the entry names a host");
        let composed = options(&target).expect("no trust material is declared");

        assert_eq!(target.entry(), "shop");
        assert_eq!(target.host(), "db.example.com");
        assert_eq!(target.port(), 3306);
        assert_eq!(composed.get_username(), "alice");
        assert_eq!(composed.get_database(), Some("freight"));
    }

    #[test]
    fn fr_conf_002_an_entry_that_names_no_host_describes_no_connection() {
        // FR-CONF-002 gives `host` no default and FR-CFG-016 admits an entry
        // that supplies any one key, so this entry is legal and unusable.
        let scratch = Scratch::new();
        let resolved = settings(&scratch, "[database.shop]\nuser = \"alice\"\n");

        assert!(Target::of(&resolved).is_none());
    }

    #[test]
    fn fr_conf_009_an_ipv6_literal_reaches_the_driver_without_its_brackets() {
        // The brackets belong to the DSN grammar of FR-CONF-009. The driver's
        // tokio path parses an address literal and otherwise resolves a name,
        // and `[2001:db8::1]` is neither.
        let scratch = Scratch::new();
        let resolved = settings(
            &scratch,
            "[database.shop]\ndsn = \"mysql://alice@[2001:db8::1]:3307/freight\"\n",
        );
        let target = Target::of(&resolved).expect("the entry names a host");

        assert_eq!(target.host(), "2001:db8::1");
        assert_eq!(target.port(), 3307);
    }

    #[test]
    fn fr_conf_014_both_keys_supply_the_bundle_and_the_order_is_fixed() {
        // FR-CONF-014: ca_file and ca_path together, file first and then the
        // directory by name ascending, so one configuration is one bundle.
        let scratch = Scratch::new();
        let file = scratch.file("root.pem", "FILE\n");
        let anchors = scratch.directory("anchors");
        scratch.file("anchors/b.pem", "B\n");
        scratch.file("anchors/a.pem", "A\n");

        let resolved = settings(
            &scratch,
            &format!(
                "[database.shop]\nhost = \"db\"\nca_file = \"{}\"\nca_path = \"{}\"\n",
                file.display(),
                anchors.display()
            ),
        );
        let target = Target::of(&resolved).expect("the entry names a host");
        let bundle = trust(&target)
            .expect("the material is readable")
            .expect("the entry declares material");

        assert_eq!(
            String::from_utf8(bundle).expect("the bundle is text"),
            "FILE\nA\n\nB\n\n"
        );
    }

    #[test]
    fn fr_conf_014_the_material_is_read_only_by_the_two_modes_that_validate() {
        // FR-CONF-014 names verify-ca and verify-identity. FR-CONF-038 records
        // `required` ignoring supplied material as one of the three controls
        // that separate the modes, so a path that cannot be read is not a
        // failure of a mode that would never have looked at it.
        let scratch = Scratch::new();

        for mode in TlsMode::ALL {
            let resolved = settings(
                &scratch,
                &format!(
                    "[database.shop]\nhost = \"db\"\ntls = \"{}\"\nca_file = \"/no/such/root.pem\"\n",
                    mode.name()
                ),
            );
            let target = Target::of(&resolved).expect("the entry names a host");
            let validates = matches!(mode, TlsMode::VerifyCa | TlsMode::VerifyIdentity);

            assert_eq!(trust(&target).is_err(), validates, "{mode}");
        }
    }

    #[test]
    fn fr_err_013_the_target_carries_the_password_in_a_type_that_does_not_print_it() {
        // FR-ERR-013 and FR-GLOB-018: a credential reaches no message and no
        // stream at any verbosity. This type is the one this crate owns and
        // could reach for, and it holds the secret behind the type that
        // redacts itself.
        //
        // The driver's own options are the reason that matters here: their
        // derived `Debug` prints the password in clear, which is why the value
        // this module composes is a local of `open` and is never held by a
        // type of this crate, logged, or returned.
        let scratch = Scratch::new();
        let resolved = settings(
            &scratch,
            "[database.shop]\nhost = \"db\"\nuser = \"alice\"\npassword = \"hunter2\"\n",
        );
        let target = Target::of(&resolved).expect("the entry names a host");

        assert!(!format!("{target:?}").contains("hunter2"));
        assert!(!format!("{target:#?}").contains("hunter2"));
        assert!(options(&target).is_ok());
    }
}
