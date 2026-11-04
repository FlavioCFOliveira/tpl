//! The three statements a `mysql` connection starts with, and the verdicts they
//! settle (`FR-MYSQL-006` … `FR-MYSQL-019`).
//!
//! | # | Statement | What it settles | Forced by |
//! |---|---|---|---|
//! | 1 | `SET SESSION TRANSACTION READ ONLY` | The session may issue no write | `FR-MYSQL-018` |
//! | 2 | One read of `@@session.transaction_read_only` | That the setting took effect: the integer `1` and nothing else | `FR-MYSQL-018`, `FR-MYSQL-019` |
//! | 3 | `SELECT VERSION()` | The product, the series and the standing | `FR-MYSQL-006` … `FR-MYSQL-010` |
//!
//! The order is the one `FR-MYSQL-017` adopts from `FR-SRV-042`, and each is
//! issued once, behind no condition (`FR-MYSQL-019`). The read-back names the
//! one spelling a supported MySQL series carries: `tx_read_only` was removed in
//! MySQL 8.0 (`FR-MYSQL-012`), so the statement `crate::mariadb::session`
//! issues would fail on every MySQL server.
//!
//! **The read-back is read as an integer of either signedness.** Its wire type
//! was observed to be an integer and not text (`FR-MYSQL-018`), and the
//! requirement confirms the setting where the answer is the integer `1` and
//! by no other value: a text `1`, a `0`, no row, or a row that decodes to no
//! integer all leave it unconfirmed, which is the `78` of `FR-MYSQL-019`.
//!
//! **The product check is necessary and not sufficient** (`FR-MYSQL-007`): a
//! version string carrying `MariaDB`, or one that does not begin with three
//! decimal integers separated by dots, is not MySQL; one that passes is read
//! as MySQL, whoever built the server, which is the limit that requirement
//! adopts from `FR-SRV-041`.

use std::borrow::Cow;

use sqlx_core::row::Row as _;
use sqlx_mysql::{MySqlConnection, MySqlRow};
use tokio::runtime::Runtime;

use super::window;
use crate::deadline::{Clock, Phase};
use crate::error::{Error, ReadOnlyFault, SessionStatement};
use crate::mariadb::connect::Target;
use crate::mariadb::fault;
use crate::mariadb::session::{Attempt, run};
use crate::mariadb::window::Series;
use crate::model::server::{Server, series_of};

/// The read-only session statement of `FR-MYSQL-018`.
pub(crate) const READ_ONLY: &str = "SET SESSION TRANSACTION READ ONLY";

/// The read-back of `FR-MYSQL-018`, naming the one spelling it admits.
pub(crate) const READ_BACK: &str = "SELECT @@session.transaction_read_only";

/// The variable [`READ_BACK`] reads, as the `cause` of `FR-MYSQL-019` names it.
const READ_BACK_VARIABLE: &str = "@@session.transaction_read_only";

/// The version probe of `FR-MYSQL-006`.
pub(crate) const PROBE: &str = "SELECT VERSION()";

/// The value the read-back confirms the setting by, and by no other
/// (`FR-MYSQL-018`).
const ENFORCED: u64 = 1;

/// The product marker of `FR-MYSQL-007`: a version string carrying it is not
/// MySQL.
pub(crate) const MARIADB_MARKER: &str = "MariaDB";

/// The condition a failure of the read-only pair produces (`FR-MYSQL-019`).
fn unenforced(attempt: Attempt, entry: &str, fault: ReadOnlyFault) -> Error {
    match attempt {
        Attempt::Expired(expired) => expired,
        Attempt::Driver(_) => not_enforced(entry, fault),
    }
}

/// The `78` of `FR-MYSQL-019`, in the half `fault` names.
fn not_enforced(entry: &str, fault: ReadOnlyFault) -> Error {
    Error::ReadOnlySessionNotEnforced {
        by_dsn: false,
        entry: entry.to_owned(),
        fault,
        read_back: READ_BACK_VARIABLE,
    }
}

/// An integer answer of either signedness, or [`None`] where the field holds
/// no integer.
///
/// The driver refuses to decode across a difference of signedness, so both
/// are asked; a negative value is no value the read-back confirms by, and is
/// answered as [`None`].
fn integer(row: &MySqlRow) -> Option<u64> {
    row.try_get::<u64, _>(0).ok().or_else(|| {
        row.try_get::<i64, _>(0)
            .ok()
            .and_then(|value| u64::try_from(value).ok())
    })
}

/// Sets the session read only and confirms that it took effect
/// (`FR-MYSQL-018`, `FR-MYSQL-019`).
///
/// # Errors
///
/// Returns [`Error::ReadOnlySessionNotEnforced`] in either half
/// `FR-MYSQL-019` names, and [`Error::NetworkDeadlineExceeded`] where a bound
/// expired.
fn enforce(
    runtime: &Runtime,
    connection: &mut MySqlConnection,
    target: &Target<'_>,
    clock: &Clock,
) -> Result<(), Error> {
    let entry = target.entry();
    let bound = || clock.bound(target.deadlines().of(Phase::CatalogueQuery));

    run(
        runtime,
        bound(),
        target,
        sqlx_core::raw_sql::raw_sql(READ_ONLY).execute(&mut *connection),
    )
    .map_err(|attempt| unenforced(attempt, entry, ReadOnlyFault::NotApplied))?;

    let answer: Option<MySqlRow> = run(
        runtime,
        bound(),
        target,
        sqlx_core::raw_sql::raw_sql(READ_BACK).fetch_optional(&mut *connection),
    )
    .map_err(|attempt| unenforced(attempt, entry, ReadOnlyFault::ReadBackDisagreed))?;

    let reported = answer.as_ref().and_then(integer);

    // The seam FR-MYSQL-043 authorises through FR-SRV-013: the answer the
    // session gave is replaced by one that does not confirm the setting.
    #[cfg(test)]
    let reported = if crate::mariadb::session::misreporting() {
        u64::try_from(crate::mariadb::session::NOT_ENFORCED).ok()
    } else {
        reported
    };

    if reported == Some(ENFORCED) {
        Ok(())
    } else {
        Err(not_enforced(entry, ReadOnlyFault::ReadBackDisagreed))
    }
}

/// Probes the version and resolves the server (`FR-MYSQL-006` …
/// `FR-MYSQL-010`).
///
/// # Errors
///
/// Returns [`Error::ServerNotMysql`], [`Error::MysqlSeriesNotSupported`], or
/// what a driver failure of the probe is classified as.
fn probe(
    runtime: &Runtime,
    connection: &mut MySqlConnection,
    target: &Target<'_>,
    clock: &Clock,
) -> Result<Server<'static>, Error> {
    let bound = clock.bound(target.deadlines().of(Phase::CatalogueQuery));

    let answer: Option<MySqlRow> = run(
        runtime,
        bound,
        target,
        sqlx_core::raw_sql::raw_sql(PROBE).fetch_optional(&mut *connection),
    )
    .map_err(|attempt| match attempt {
        Attempt::Expired(expired) => expired,
        Attempt::Driver(driver) => fault::speaking(
            target.entry(),
            &driver,
            target.host(),
            target.port(),
            SessionStatement::VersionProbe,
        ),
    })?;

    let reported = answer
        .map(|row| row.try_get::<String, _>(0))
        .transpose()
        .ok()
        .flatten();

    // A probe that answers nothing has characterised nothing, and the server is
    // not established as MySQL.
    resolve(target, reported.unwrap_or_default())
}

/// Whether `version` begins with three decimal integers separated by dots
/// (`FR-MYSQL-007`).
///
/// What follows them is not read: the condition is a prefix, and the suffix of
/// `FR-MYSQL-006` derives nothing.
fn three_integers(version: &str) -> bool {
    let mut rest = version;

    for component in 0..3 {
        let digits = rest.bytes().take_while(u8::is_ascii_digit).count();
        if digits == 0 {
            return false;
        }
        rest = &rest[digits..];

        if component < 2 {
            let Some(after) = rest.strip_prefix('.') else {
                return false;
            };
            rest = after;
        }
    }

    true
}

/// The server a probe's answer describes, or the refusal it earns.
///
/// Split out so that the outcomes of `FR-MYSQL-007` … `FR-MYSQL-010` are
/// testable without a server.
fn resolve(target: &Target<'_>, version: String) -> Result<Server<'static>, Error> {
    let not_mysql = |reported: String| Error::ServerNotMysql {
        by_dsn: false,
        entry: target.entry().to_owned(),
        reported,
    };

    // FR-MYSQL-007: the product check, a necessary condition and not a
    // sufficient one.
    if version.contains(MARIADB_MARKER) || !three_integers(&version) {
        return Err(not_mysql(version));
    }

    // FR-MYSQL-006: the series is the first two components, and nothing else.
    let Some(identifier) = series_of(&version).map(str::to_owned) else {
        return Err(not_mysql(version));
    };
    let Some(series) = Series::parse(&identifier) else {
        return Err(not_mysql(version));
    };

    let Some(standing) = window::standing(series) else {
        return Err(Error::MysqlSeriesNotSupported {
            by_dsn: false,
            entry: target.entry().to_owned(),
            version,
            series: identifier,
            supported: &window::SUPPORTED,
        });
    };

    // `series_of` has already read a series from this string, so the
    // constructor answers; the refusal keeps the module free of a panic.
    let reported = version.clone();
    Server::probed(Cow::Owned(version), standing).ok_or_else(|| not_mysql(reported))
}

/// Runs the connection start of `FR-MYSQL-015`: the three statements, once
/// each, in the order `FR-MYSQL-017` fixes.
///
/// # Errors
///
/// Returns what [`enforce`] and [`probe`] return.
pub(crate) fn start(
    runtime: &Runtime,
    connection: &mut MySqlConnection,
    target: &Target<'_>,
    clock: &Clock,
) -> Result<Server<'static>, Error> {
    enforce(runtime, connection, target, clock)?;

    probe(runtime, connection, target, clock)
}

#[cfg(test)]
mod tests {
    use super::{MARIADB_MARKER, PROBE, READ_BACK, READ_ONLY, resolve, three_integers};
    use crate::error::Error;
    use crate::mariadb::connect::Target;
    use crate::model::server::Standing;
    use crate::project::config;
    use crate::project::scratch::Scratch;
    use crate::project::settings::{self, Settings};

    /// A lookup that defines nothing.
    fn nothing(_: &str) -> Option<String> {
        None
    }

    /// The settings of a `mysql` entry `shop` that is never reached.
    fn settings(scratch: &Scratch) -> Settings {
        let file = scratch.file(
            ".cfg",
            "[database.shop]\nprovider = \"mysql\"\nhost = \"db.example.com\"\n",
        );
        let configuration = config::load(&file).expect("the document is valid");

        settings::resolve(
            &configuration,
            Some("shop"),
            &settings::clock(None),
            &nothing,
        )
        .expect("the entry resolves")
        .into_mysql()
        .expect("the entry is a mysql entry")
    }

    /// The verdict `version` earns.
    fn verdict(scratch: &Scratch, version: &str) -> Result<Standing, Error> {
        let resolved = settings(scratch);
        let target = Target::of(&resolved).expect("the entry names a host");

        resolve(&target, version.to_owned()).map(|server| server.standing())
    }

    #[test]
    fn fr_mysql_015_the_three_connection_start_statements_are_the_ones_the_list_admits() {
        assert_eq!(READ_ONLY, "SET SESSION TRANSACTION READ ONLY");
        // FR-MYSQL-015 names the variable, and no other spelling.
        assert_eq!(READ_BACK, "SELECT @@session.transaction_read_only");
        assert!(!READ_BACK.contains("tx_read_only"));
        assert_eq!(PROBE, "SELECT VERSION()");
    }

    #[test]
    fn fr_mysql_006_the_server_object_carries_the_probe_unaltered() {
        let scratch = Scratch::new();
        let resolved = settings(&scratch);
        let target = Target::of(&resolved).expect("the entry names a host");

        for (version, series) in [("8.4.13", "8.4"), ("9.7.4", "9.7")] {
            let server = resolve(&target, version.to_owned()).expect("a supported series");

            assert_eq!(server.version(), version);
            assert_eq!(server.series(), series);
            assert_eq!(server.standing(), Standing::Supported);
        }

        // The suffix derives nothing.
        let suffixed = resolve(&target, "8.4.13-debug".to_owned()).expect("8.4 is supported");
        assert_eq!(suffixed.series(), "8.4");
        assert_eq!(suffixed.version(), "8.4.13-debug");
    }

    #[test]
    fn fr_mysql_007_a_mariadb_string_or_one_outside_the_form_is_not_mysql() {
        let scratch = Scratch::new();

        for version in [
            "11.8.9-MariaDB-ubu2404",
            "10.11.19-MariaDB-ubu2204",
            "8.4",
            "8.4.x",
            "x.4.13",
            "",
            "8..13",
        ] {
            let condition = verdict(&scratch, version).expect_err("not MySQL");

            assert!(
                matches!(condition, Error::ServerNotMysql { ref reported, .. } if reported == version),
                "{version}: {condition:?}"
            );
            assert_eq!(condition.exit_code(), 78, "{version}");
        }

        assert!(three_integers("8.4.13"));
        assert!(three_integers("8.4.13.0"));
        assert!(!three_integers("8.4."));
        assert!("11.8.9-MariaDB".contains(MARIADB_MARKER));
    }

    #[test]
    fn fr_mysql_009_a_series_below_the_window_is_refused_naming_the_version_and_the_set() {
        let scratch = Scratch::new();

        match verdict(&scratch, "8.0.46").expect_err("8.0 is outside the set") {
            Error::MysqlSeriesNotSupported {
                entry,
                version,
                series,
                supported,
                ..
            } => {
                assert_eq!(entry, "shop");
                assert_eq!(version, "8.0.46");
                assert_eq!(series, "8.0");
                assert_eq!(supported, ["9.7", "8.4"]);
            }
            other => panic!("expected an unsupported series, got {other:?}"),
        }

        for version in ["9.0.1", "9.6.0", "5.7.44"] {
            let condition = verdict(&scratch, version).expect_err("refused");
            assert_eq!(condition.exit_code(), 78, "{version}");
        }
    }

    #[test]
    fn fr_mysql_010_an_innovation_release_newer_than_the_window_is_read_and_marked() {
        let scratch = Scratch::new();

        for version in ["26.10.0", "26.7.0", "9.8.0"] {
            assert_eq!(
                verdict(&scratch, version).ok(),
                Some(Standing::NewerThanSupported),
                "{version}"
            );
        }
    }
}
