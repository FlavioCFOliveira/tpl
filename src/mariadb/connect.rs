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
//! | `FR-GLOB-017` — `INFO` reports which phases ran and how long each took | Each of the two is timed where it runs and reported through [`emit::phase_ran`] |
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

use std::fmt;
use std::path::Path;
use std::time::Instant;

use sqlx::Connection as _;
use sqlx::mysql::{MySqlConnectOptions, MySqlConnection, MySqlSslMode};
use tokio::net::lookup_host;
use tokio::runtime::Runtime;
use tokio::time::timeout;

use super::fault;
use crate::deadline::{Clock, Deadlines, Phase};
use crate::diagnostics::emit;
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
/// **Every block is terminated before the next begins**, which `ADR-002`
/// obliges and which the seam between the two keys did not have: a `ca_file`
/// whose last byte is not a newline ran its final PEM block into the first line
/// of the first directory file, and a bundle that will not parse is the one
/// outcome a concatenation has to be built to avoid.
///
/// **A `ca_path` entry is resolved through a symbolic link**, per `FR-CONF-014`
/// as the thirty-second edition amended it: the convention the key exists to
/// serve is a directory of hash-named links beside the certificates they point
/// at, and a reader that skipped them would take nothing from such a directory
/// and say nothing about having taken nothing. An entry whose **target** is a
/// regular file is read at that target; anything else — a directory, a socket,
/// a device — is skipped, a directory being skipped rather than descended into.
/// An entry that cannot be resolved, or cannot be read at its target, is
/// reported against **its own path in the directory** and never passed over,
/// because a skipped dangling link is a trust anchor the operator believes is
/// loaded and is not.
///
/// The names are sorted **before** any of them is resolved, over the directory's
/// own entries and never over the targets they resolve to, so `NFR-DET-001`
/// holds exactly as before. Two links resolving to one certificate contribute
/// it twice, which no requirement forbids.
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
/// Returns [`Error::ProjectFileUnreadable`] — `74` — naming the path the
/// configuration declared, or the directory entry's own path, where a file, a
/// directory or an entry cannot be read; and
/// [`Error::TrustDirectoryEmpty`] — `78` — where `ca_path` is declared and no
/// entry of the directory it names resolves to a regular file, per
/// `FR-CONF-044`. The second is decided here, while the material is assembled,
/// so it costs no round trip and reaches the caller before the server is
/// contacted.
fn trust(target: &Target<'_>) -> Result<Option<Vec<u8>>, Error> {
    if !matches!(target.tls, TlsMode::VerifyCa | TlsMode::VerifyIdentity) {
        return Ok(None);
    }

    let mut bundle = Vec::new();

    if let Some(file) = target.ca_file {
        bundle.extend_from_slice(&read(file)?);
        // ADR-002: each block ends before the next begins. The terminator is
        // written here and not only after a directory entry, because the seam
        // between `ca_file` and the first entry of `ca_path` is a seam like any
        // other.
        bundle.push(b'\n');
    }

    if let Some(directory) = target.ca_path {
        let mut paths = Vec::new();

        for entry in unreadable(directory, std::fs::read_dir(directory))? {
            paths.push(unreadable(directory, entry)?.path());
        }

        // NFR-DET-001: the order is the directory's own names, ascending, fixed
        // before anything is resolved or read.
        paths.sort();

        let mut contributed = false;

        for path in paths {
            // `metadata` follows the link and `symlink_metadata` would not,
            // which is the whole of the amendment: what decides is the kind of
            // the **target**. A failure to resolve is reported against the
            // entry's own path — the link, not what it points at.
            if !unreadable(&path, std::fs::metadata(&path))?.is_file() {
                continue;
            }

            bundle.extend_from_slice(&read(&path)?);
            // A bundle is a concatenation of PEM blocks, and a file whose last
            // line carries no terminator would otherwise run into the next
            // file's first line.
            bundle.push(b'\n');
            contributed = true;
        }

        // FR-CONF-044. The condition is *no entry resolves to a regular file*,
        // which is decidable before a byte is read; a regular file that is
        // empty, or that holds no PEM block, is not this condition, because
        // `tpl` does not parse the bytes it assembles. It is per key, so it
        // fires whether or not `ca_file` is declared beside it.
        if !contributed {
            return Err(Error::TrustDirectoryEmpty {
                entry: target.entry.to_owned(),
                path: directory.to_owned(),
            });
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

/// Everything the driver is told, in a type that cannot print it.
///
/// `MySqlConnectOptions` derives [`Debug`] and its derivation writes
/// `password: Some("…")` **in clear**, which is verified rather than assumed.
/// `FR-ERR-013`, `FR-GLOB-018` and `BR-ERR-003` bar a credential from every
/// message and every stream at every verbosity, and the way to hold a
/// prohibition on printing is to deny the value a printing implementation —
/// which is what [`Secret`] does for the credential itself and what this type
/// does for the composed options that carry it.
///
/// It is a newtype and not a wrapper with an accessor **on purpose**: the two
/// operations [`open`] needs are carried here, so the inner value never leaves
/// and no caller can reach a `{:?}` of it. What it deliberately does **not**
/// implement is as much of the type as what it does: no derived [`Debug`], no
/// [`Clone`], no [`Display`](std::fmt::Display), no
/// [`Serialize`](serde::Serialize), no [`PartialEq`] and no [`Default`].
/// Omitting [`Clone`] is load-bearing — a clone is a second value to keep track
/// of, and one of the two would eventually be held by something that prints.
pub(crate) struct DriverOptions(MySqlConnectOptions);

impl fmt::Debug for DriverOptions {
    /// Writes a placeholder, whatever the options carry.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(REDACTED_OPTIONS)
    }
}

impl DriverOptions {
    /// The user the driver will authenticate as.
    ///
    /// It is the one field of the options a condition names: the `77` row of
    /// `FR-ERR-034` obliges the `cause` line of a refused authentication to
    /// name the user it was refused for.
    fn username(&self) -> &str {
        self.0.get_username()
    }

    /// Opens the connection these options describe.
    async fn connect(&self) -> Result<MySqlConnection, sqlx::Error> {
        MySqlConnection::connect_with(&self.0).await
    }

    /// The TLS mode the options carry.
    #[cfg(test)]
    fn ssl_mode(&self) -> MySqlSslMode {
        self.0.get_ssl_mode()
    }

    /// The server-side database the options carry.
    #[cfg(test)]
    fn database(&self) -> Option<&str> {
        self.0.get_database()
    }
}

/// What [`DriverOptions`]'s [`Debug`] writes in place of the options.
const REDACTED_OPTIONS: &str = "DriverOptions(***)";

/// Everything the driver is told, composed once.
///
/// # Errors
///
/// Returns what [`trust`] returns for trust material that cannot be read.
fn options(target: &Target<'_>) -> Result<DriverOptions, Error> {
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

    Ok(DriverOptions(options))
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
/// **Each of the two is timed and reported**, per `FR-GLOB-017`, whether it
/// ended in an answer or in a refusal: a phase that failed is a phase that ran,
/// and how long it took before it failed is what a caller diagnosing a slow
/// invocation came for. The TLS handshake carries no line of its own for the
/// reason above — its duration is inside the connect's, and a line naming it
/// would be a number this module does not have.
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

        let started = Instant::now();
        let resolved = timeout(bound.remaining(), lookup_host((host, port))).await;
        emit::phase_ran(Phase::DnsResolution, started.elapsed());

        let resolved = match resolved {
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

        let started = Instant::now();
        let opened = timeout(bound.remaining(), options.connect()).await;
        // FR-GLOB-017. The TLS handshake ran inside this call where the mode
        // negotiates one, so its time is inside this number and it has no line
        // of its own.
        emit::phase_ran(Phase::TcpConnect, started.elapsed());

        match opened {
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
                options.username(),
                target.database,
            )),
        }
    })
}

#[cfg(test)]
mod tests {
    use super::{REDACTED_OPTIONS, Target, options, ssl_mode, trust};
    use crate::error::Error;
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

            assert_eq!(named(composed.ssl_mode()), named(ssl_mode(mode)), "{mode}");
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
        assert_eq!(composed.username(), "alice");
        assert_eq!(composed.database(), Some("freight"));
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

    /// The bundle `text` assembles for an entry declaring `keys`.
    fn bundle(scratch: &Scratch, keys: &str) -> String {
        let resolved = settings(scratch, &format!("[database.shop]\nhost = \"db\"\n{keys}"));
        let target = Target::of(&resolved).expect("the entry names a host");
        let assembled = trust(&target)
            .expect("the material is readable")
            .expect("the entry declares material");

        String::from_utf8(assembled).expect("the bundle is text")
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

        assert_eq!(
            bundle(
                &scratch,
                &format!(
                    "ca_file = \"{}\"\nca_path = \"{}\"\n",
                    file.display(),
                    anchors.display()
                )
            ),
            "FILE\n\nA\n\nB\n\n"
        );
    }

    #[test]
    fn adr_002_the_ca_file_block_is_terminated_before_the_first_ca_path_block() {
        // ADR-002: each block ends before the next begins. The fixture above
        // happens to end in a newline, which is why the seam between the two
        // keys could be unterminated without any assertion failing; this one
        // carries a ca_file whose last byte is not a newline, so the defect is
        // the difference between 'FILE' running into 'A' and not.
        let scratch = Scratch::new();
        let file = scratch.file("root.pem", "FILE");
        let anchors = scratch.directory("anchors");
        scratch.file("anchors/a.pem", "A\n");

        let assembled = bundle(
            &scratch,
            &format!(
                "ca_file = \"{}\"\nca_path = \"{}\"\n",
                file.display(),
                anchors.display()
            ),
        );

        assert_eq!(assembled, "FILE\nA\n\n");
        assert!(
            !assembled.contains("FILEA"),
            "the ca_file block ran into the first ca_path block"
        );
    }

    /// What `trust` made of an entry declaring `keys`, refused.
    fn refused(scratch: &Scratch, keys: &str) -> Error {
        let resolved = settings(scratch, &format!("[database.shop]\nhost = \"db\"\n{keys}"));
        let target = Target::of(&resolved).expect("the entry names a host");

        trust(&target).expect_err("the material is refused")
    }

    #[test]
    fn fr_conf_014_a_ca_path_entry_is_resolved_through_a_symbolic_link() {
        // FR-CONF-014, as the thirty-second edition amended it: an entry whose
        // **target** is a regular file is read at that target. A hash-named
        // CApath directory is a set of links beside the certificates they point
        // at, and a reader that selected on `DirEntry::file_type` — which does
        // not traverse a link — took nothing from one and said nothing about
        // having taken nothing.
        let scratch = Scratch::new();
        let anchors = scratch.directory("anchors");
        let real = scratch.file("store/root.pem", "ROOT\n");
        scratch.link(&real, &anchors.join("a.0"));

        // Two links to one certificate contribute it twice, which no
        // requirement of this corpus forbids and this one states.
        scratch.link(&real, &anchors.join("b.0"));

        assert_eq!(
            bundle(&scratch, &format!("ca_path = \"{}\"\n", anchors.display())),
            "ROOT\n\nROOT\n\n"
        );
    }

    #[test]
    fn nfr_det_001_the_order_is_the_directory_own_names_and_never_the_targets() {
        // FR-CONF-014: the entries are sorted before any of them is resolved,
        // over the names the directory holds and never over what they resolve
        // to. The link names and their targets sort in opposite orders here, so
        // a sort moved after the resolution would produce the other bundle.
        let scratch = Scratch::new();
        let anchors = scratch.directory("anchors");
        let first = scratch.file("store/z.pem", "FIRST\n");
        let second = scratch.file("store/a.pem", "SECOND\n");
        scratch.link(&first, &anchors.join("a.0"));
        scratch.link(&second, &anchors.join("b.0"));

        assert_eq!(
            bundle(&scratch, &format!("ca_path = \"{}\"\n", anchors.display())),
            "FIRST\n\nSECOND\n\n"
        );
    }

    #[test]
    fn fr_conf_014_an_entry_that_resolves_to_anything_but_a_regular_file_is_skipped() {
        // FR-CONF-014: a directory is skipped rather than descended into, and
        // so is a link that resolves to one. The certificate beside them is
        // still taken, so the skip is of the entry and not of the directory.
        let scratch = Scratch::new();
        let anchors = scratch.directory("anchors");
        scratch.file("anchors/a.pem", "TAKEN\n");
        let nested = scratch.directory("anchors/b.d");
        scratch.file("anchors/b.d/buried.pem", "BURIED\n");
        scratch.link(&nested, &anchors.join("c.0"));

        assert_eq!(
            bundle(&scratch, &format!("ca_path = \"{}\"\n", anchors.display())),
            "TAKEN\n\n"
        );
    }

    #[test]
    fn fr_conf_014_a_dangling_entry_is_reported_against_its_own_path_and_never_passed_over() {
        // FR-CONF-014: an entry the system cannot resolve is reported against
        // **that entry's own path in the directory** — the link, not its target
        // — per the `74` row of FR-ERR-034, and is not skipped. A dangling link
        // silently skipped is a trust anchor the operator believes is loaded
        // and is not.
        let scratch = Scratch::new();
        let anchors = scratch.directory("anchors");
        scratch.file("anchors/a.pem", "TAKEN\n");
        let gone = scratch.path("store/removed.pem");
        let link = anchors.join("b.0");
        scratch.link(&gone, &link);

        let condition = refused(&scratch, &format!("ca_path = \"{}\"\n", anchors.display()));

        assert_eq!(condition.exit_code(), 74);

        match condition {
            Error::ProjectFileUnreadable { ref path, .. } => {
                assert_eq!(path, &link, "the condition names the target, not the link");
                assert_ne!(path, &gone);
            }
            other => panic!("expected an unreadable path, got {other:?}"),
        }
    }

    #[test]
    fn fr_conf_044_a_ca_path_that_yields_no_regular_file_is_refused_before_anything_is_contacted() {
        // FR-CONF-044: a declared key that contributes nothing is a fault in
        // .tpl/.cfg, and the condition is decided while the material is
        // assembled. 78 and not 69 — nothing was contacted — and not 74 —
        // nothing failed to be read.
        let scratch = Scratch::new();
        let empty = scratch.directory("empty");
        scratch.directory("empty/nested");

        let condition = refused(&scratch, &format!("ca_path = \"{}\"\n", empty.display()));

        assert_eq!(condition.exit_code(), 78);
        assert!(
            matches!(condition, Error::TrustDirectoryEmpty { ref path, .. } if *path == empty),
            "{condition:?}"
        );

        // The check is per key: it fires whether or not `ca_file` is declared
        // beside it, because the operator asked for both and only one was
        // honoured.
        let file = scratch.file("root.pem", "FILE\n");
        let beside = refused(
            &scratch,
            &format!(
                "ca_file = \"{}\"\nca_path = \"{}\"\n",
                file.display(),
                empty.display()
            ),
        );

        assert_eq!(beside.exit_code(), 78);
    }

    #[test]
    fn fr_conf_044_a_regular_file_that_is_empty_or_carries_no_pem_block_is_not_that_condition() {
        // FR-CONF-044 states its condition exactly and no wider: it is *no
        // entry resolves to a regular file*, decidable before a byte is read.
        // tpl does not parse the bytes it assembles, and a requirement that
        // refused on their content would oblige this corpus to fix a
        // certificate format it names nowhere.
        let scratch = Scratch::new();
        let anchors = scratch.directory("anchors");
        scratch.file("anchors/a.pem", "");
        scratch.file("anchors/b.pem", "not a certificate at all\n");

        assert_eq!(
            bundle(&scratch, &format!("ca_path = \"{}\"\n", anchors.display())),
            "\nnot a certificate at all\n\n"
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
        // stream at any verbosity. This type is one of the two this module
        // owns and could reach for, and it holds the secret behind the type
        // that redacts itself.
        //
        // The prohibition on the **driver's** own options is no longer this
        // test's: `MySqlConnectOptions` derives `Debug` and that derivation
        // writes the password in clear, so the composed value is wrapped in
        // `DriverOptions`, which has a hand-written one. The property has moved
        // to the test below, and it is a property of a type rather than of a
        // local nobody happens to print.
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

    #[test]
    fn fr_err_013_the_options_the_driver_is_given_do_not_print_the_password() {
        // FR-ERR-013, FR-GLOB-018 and BR-ERR-003 bar a credential from every
        // message and every stream at every verbosity. `MySqlConnectOptions`
        // derives `Debug` and prints `password: Some("…")` in clear, so the
        // composed value is denied a printing implementation rather than
        // trusted not to reach one.
        //
        // Both composition paths are driven: `FR-CONF-006` supplies the
        // password through the discrete `password` key, and `FR-CONF-009`
        // supplies it inside the user info of a DSN. The two build the options
        // by different routes and either would have printed it.
        let scratch = Scratch::new();

        for document in [
            "[database.shop]\nhost = \"db\"\nuser = \"alice\"\npassword = \"hunter2\"\n",
            "[database.shop]\ndsn = \"mysql://alice:hunter2@db.example.com:3306/freight\"\n",
        ] {
            let resolved = settings(&scratch, document);
            let target = Target::of(&resolved).expect("the entry names a host");
            let composed = options(&target).expect("no trust material is declared");

            assert_eq!(format!("{composed:?}"), REDACTED_OPTIONS);
            assert_eq!(format!("{composed:#?}"), REDACTED_OPTIONS);
            assert!(!format!("{composed:?}").contains("hunter2"), "{document}");
            assert!(!format!("{composed:#?}").contains("hunter2"), "{document}");

            // The value did reach the options: the property under test is that
            // it cannot be printed, not that it was never supplied.
            assert_eq!(composed.username(), "alice");
        }
    }
}
