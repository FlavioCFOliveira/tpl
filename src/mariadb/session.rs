//! The three statements a connection starts with, and the three verdicts they
//! settle, before a catalogue row is read.
//!
//! `FR-SRV-006` closes the statement list at four kinds and this module issues
//! three of them — the fourth is the catalogue read, which is not this
//! module's. Each of the three is issued **once**, and the order is the one
//! `FR-SRV-042` fixes: the read-only session statement, its read-back
//! immediately after, then the version probe. That requirement states the
//! order and no other passage does; it derives it from the conditions the
//! three settle, whose own order is `FR-ERR-006`'s — the read-only session of
//! `FR-SRV-010`, then the product of `FR-SRV-003`, then the series of
//! `FR-SRV-020`, "so the strongest guarantee is confirmed before the server is
//! characterised".
//!
//! | # | Statement | What it settles | Forced by |
//! |---|---|---|---|
//! | 1 | The read-only session statement | The session may issue no write | `FR-SRV-008` |
//! | 2 | One read of `@@session.tx_read_only` | That the setting took effect | `FR-SRV-009` |
//! | 3 | The version probe | The product, the series, and the standing | `FR-SRV-002`, `FR-SRV-040`, `FR-SRV-041` |
//!
//! **The read-back is the whole of the guarantee, and inferring it is not
//! allowed.** `FR-SRV-006` rejects, in terms, deciding that the setting took
//! effect from the absence of an error when it was applied: a server that
//! accepts the statement and does not apply it is exactly the case
//! `FR-SRV-009` exists to catch. The variable is named by the requirement and
//! is not a free choice — `tx_read_only` is the only spelling present on every
//! series of `FR-SRV-015`, and `10.11` answers `ERROR 1193` to the other,
//! which is difference 12 of `FR-SRV-038`. A statement naming
//! `transaction_read_only` would fail on a supported series and would not be
//! caught by testing against the other three.
//!
//! **Neither statement is behind a condition.** `FR-SRV-011` admits no flag,
//! configuration key or environment condition that disables `FR-SRV-006`
//! through `FR-SRV-010`, so the three are issued by the one path that opens a
//! connection and there is no branch around them to find.
//!
//! **A failure of either half of the read-only pair is `78`, whatever the
//! server said.** `FR-SRV-010` gives both conditions one code and the `cause`
//! line separates them, per `FR-ERR-034`: the setting could not be applied, or
//! the read-back did not confirm it. The one outcome that is not `78` is a
//! deadline, which is `FR-ERR-027`'s `69` and leaves the catalogue unread just
//! the same.

use std::borrow::Cow;
use std::future::Future;
use std::time::Instant;

use sqlx::mysql::MySqlConnection;
use sqlx::{Row as _, mysql::MySqlRow};
use tokio::runtime::Runtime;
use tokio::time::timeout;

use super::connect::Target;
use super::fault;
use super::window::{self, Series};
use crate::deadline::{Bound, Clock, Phase};
use crate::diagnostics::emit;
use crate::error::{Error, NetworkPhase, ReadOnlyFault};
use crate::model::server::{Server, series_of};

/// The read-only session statement of `FR-SRV-008`.
const READ_ONLY: &str = "SET SESSION TRANSACTION READ ONLY";

/// The read-back of `FR-SRV-009`, naming the one spelling every series of
/// `FR-SRV-015` carries.
const READ_BACK: &str = "SELECT @@session.tx_read_only";

/// The server version probe of `FR-SRV-002`.
const PROBE: &str = "SELECT VERSION()";

/// The value `@@session.tx_read_only` reports for a session that is read only.
const ENFORCED: i64 = 1;

/// The value `@@session.tx_read_only` reports for a session that is not read
/// only, which is the answer the seam of [`misreport_read_back`] presents.
///
/// It is the answer a server would give if it had accepted the statement of
/// `FR-SRV-008` and not applied it — the condition `FR-SRV-009` exists to
/// catch, and the one no supported MariaDB produces.
#[cfg(test)]
const NOT_ENFORCED: i64 = 0;

/// The product marker `FR-SRV-041` tests the version string for.
const MARKER: &str = "MariaDB";

/// Why a statement did not answer.
///
/// The two are kept apart because the caller decides what a driver failure
/// means and the deadline decides itself: a `SET` the server refused is
/// `FR-SRV-010`'s `78`, and a `SET` that outlived its bound is `FR-ERR-027`'s
/// `69`, and one statement can end either way.
enum Attempt {
    /// The bound of `FR-GLOB-012` expired, already classified.
    Expired(Error),
    /// The driver reported a failure, not yet classified.
    Driver(sqlx::Error),
}

/// Runs one statement under the bound of `FR-GLOB-012`.
///
/// The three statements of this module are statements on an open session, so
/// they take the statement deadline of `FR-CONF-005` — `core.query_timeout` —
/// and report under the phase `FR-ERR-034` gives a statement. None of them
/// reads a schema, so none of them is a catalogue query for the counts of
/// `NFR-PERF-001` and `NFR-PERF-002`, which `FR-SRV-006` states of the
/// read-back in its own words.
///
/// **Each run is timed and reported**, per `FR-GLOB-017`: this is where the
/// session start spends its time, and the three statements are three runs of
/// the phase `FR-CONF-005` bounds them by. The line carries that phase and a
/// duration and nothing else — no statement text, which `NFR-DET-001` would
/// make a caller mistake for contract, and which `FR-GLOB-018` keeps off this
/// stream with everything else it bars. The count of these lines is not the
/// catalogue-query count of `NFR-PERF-008`: that count is the lines
/// [`emit::catalogue_query`] writes, under a token of its own.
fn run<T>(
    runtime: &Runtime,
    bound: Bound,
    target: &Target<'_>,
    work: impl Future<Output = Result<T, sqlx::Error>>,
) -> Result<T, Attempt> {
    let expired = || {
        Attempt::Expired(fault::expired(
            NetworkPhase::CatalogueQuery,
            target.host(),
            target.port(),
            bound,
        ))
    };

    if bound.expired() {
        return Err(expired());
    }

    runtime.block_on(async {
        let started = Instant::now();
        let outcome = timeout(bound.remaining(), work).await;
        emit::phase_ran(Phase::CatalogueQuery, started.elapsed());

        match outcome {
            Err(_) => Err(expired()),
            Ok(Ok(answer)) => Ok(answer),
            Ok(Err(driver)) => Err(Attempt::Driver(driver)),
        }
    })
}

/// The condition a failure of the read-only pair produces.
///
/// A deadline keeps its own classification, per `FR-ERR-027`; everything else
/// is `FR-SRV-010`, in the half of it the caller names. The driver's error is
/// dropped here rather than carried, which is `OD-06` applied to a value this
/// module holds for the length of one match.
fn unenforced(attempt: Attempt, entry: &str, fault: ReadOnlyFault) -> Error {
    match attempt {
        Attempt::Expired(expired) => expired,
        Attempt::Driver(_) => Error::ReadOnlySessionNotEnforced {
            entry: entry.to_owned(),
            fault,
        },
    }
}

/// The condition a failure of the version probe produces.
fn unprobed(attempt: Attempt, target: &Target<'_>) -> Error {
    match attempt {
        Attempt::Expired(expired) => expired,
        Attempt::Driver(driver) => fault::speaking(&driver, target.host(), target.port()),
    }
}

/// Sets the session read only and confirms that it took effect
/// (`FR-SRV-008`, `FR-SRV-009`, `FR-SRV-010`).
///
/// # Errors
///
/// Returns [`Error::ReadOnlySessionNotEnforced`] in either of the two halves
/// `FR-SRV-010` names, and [`Error::NetworkDeadlineExceeded`] where a bound
/// expired. On every one of those paths the catalogue is not read, because the
/// caller has no session to read it with.
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
        sqlx::raw_sql(READ_ONLY).execute(&mut *connection),
    )
    .map_err(|attempt| unenforced(attempt, entry, ReadOnlyFault::NotApplied))?;

    let disagreed = || Error::ReadOnlySessionNotEnforced {
        entry: entry.to_owned(),
        fault: ReadOnlyFault::ReadBackDisagreed,
    };

    let answer: Option<MySqlRow> = run(
        runtime,
        bound(),
        target,
        sqlx::raw_sql(READ_BACK).fetch_optional(&mut *connection),
    )
    .map_err(|attempt| unenforced(attempt, entry, ReadOnlyFault::ReadBackDisagreed))?;

    // A read that returns no row, a row that does not decode, and a row that
    // decodes to anything but the enforced value are one condition: the
    // session did not report the setting back, so nothing confirms it.
    let reported = answer
        .ok_or_else(disagreed)?
        .try_get::<i64, _>(0)
        .map_err(|_| disagreed())?;

    // The seam `FR-SRV-013` authorises, which exists in this system's own test
    // configuration alone. It replaces the answer the session gave with one
    // that does not confirm the setting, so what the condition below decides is
    // the decision under test rather than a branch a test stepped around.
    #[cfg(test)]
    let reported = if misreporting() {
        NOT_ENFORCED
    } else {
        reported
    };

    if reported == ENFORCED {
        Ok(())
    } else {
        Err(disagreed())
    }
}

/// Probes the product and the version, and resolves the server against the
/// window (`FR-SRV-002`, `FR-SRV-040`, `FR-SRV-041`, `FR-SRV-020`,
/// `FR-SRV-031`).
///
/// The answer is the `server` object of `FR-CTX-031`, which `FR-SRV-028`
/// carries into the model: the version exactly as the probe returned it, the
/// series `FR-SRV-040` derives from its first two components **and from
/// nothing else**, and the standing of `FR-CTX-034`.
///
/// # Errors
///
/// Returns [`Error::ServerNotMariaDb`] where the version string does not carry
/// the product marker or is not a version string this system can read a series
/// from, and [`Error::SeriesNotSupported`] where the series is below the
/// window of `FR-SRV-015`.
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
        sqlx::raw_sql(PROBE).fetch_optional(&mut *connection),
    )
    .map_err(|attempt| unprobed(attempt, target))?;

    let reported = answer
        .map(|row| row.try_get::<String, _>(0))
        .transpose()
        .ok()
        .flatten();

    let Some(version) = reported else {
        // A probe that answers nothing has characterised nothing, and
        // `FR-SRV-002` bars every other statement until it has. The server is
        // not established as MariaDB, which is the condition `FR-SRV-003`
        // refuses.
        return Err(not_mariadb(target, String::new()));
    };

    resolve(target, version)
}

/// The server a probe's answer describes, or the refusal it earns.
///
/// This is the whole of what the probe's answer decides, split out so that the
/// four outcomes of `FR-SRV-041`, `FR-SRV-003`, `FR-SRV-020` and `FR-SRV-031`
/// are testable without a server.
fn resolve(target: &Target<'_>, version: String) -> Result<Server<'static>, Error> {
    // FR-SRV-041: the marker is a **necessary** condition and not a sufficient
    // one. A server determined to pass as MariaDB will pass, and no reading
    // `tpl` can take over the wire separates the two — the limit is the
    // requirement's own, stated there rather than discovered here.
    if !version.contains(MARKER) {
        return Err(not_mariadb(target, version));
    }

    // FR-SRV-040 fixes one form, `<major>.<minor>.<patch>-MariaDB`. A string
    // carrying the marker and outside that form yields no series, so nothing
    // can be compared against the window and `FR-SRV-020` has nothing to name;
    // it is refused as a product this system has not established rather than
    // as a series it has read.
    let Some(identifier) = series_of(&version).map(str::to_owned) else {
        return Err(not_mariadb(target, version));
    };

    let Some(series) = Series::parse(&identifier) else {
        return Err(not_mariadb(target, version));
    };

    let Some(standing) = window::standing(series) else {
        return Err(Error::SeriesNotSupported {
            entry: target.entry().to_owned(),
            series: identifier,
            supported: &window::SUPPORTED,
        });
    };

    Server::probed(Cow::Owned(version), standing).ok_or_else(|| {
        // Unreachable: `series_of` has already read a series from this string.
        // It is written as a condition rather than as an `expect` so that the
        // module carries no panic.
        Error::ServerNotMariaDb {
            entry: target.entry().to_owned(),
            product: String::new(),
        }
    })
}

/// The refusal of `FR-SRV-003`, naming the entry that reached the server and
/// the product it reported.
fn not_mariadb(target: &Target<'_>, product: String) -> Error {
    Error::ServerNotMariaDb {
        entry: target.entry().to_owned(),
        product,
    }
}

/// Runs the connection start of `FR-SRV-006`: the three statements, once each,
/// in the order `FR-SRV-042` fixes.
///
/// # Errors
///
/// Returns what [`enforce`] returns for the read-only session and what
/// [`probe`] returns for the product and the series. Every one of them leaves
/// the catalogue unread, which is what `FR-SRV-010`, `FR-SRV-003` and
/// `FR-SRV-020` each require in their own words.
pub(super) fn start(
    runtime: &Runtime,
    connection: &mut MySqlConnection,
    target: &Target<'_>,
    clock: &Clock,
) -> Result<Server<'static>, Error> {
    enforce(runtime, connection, target, clock)?;

    probe(runtime, connection, target, clock)
}

/// Whether this thread's read-back is presented with an answer that does not
/// confirm the setting.
#[cfg(test)]
fn misreporting() -> bool {
    MISREPORTED.with(std::cell::Cell::get)
}

#[cfg(test)]
thread_local! {
    /// Whether [`misreport_read_back`] is installed on this thread.
    ///
    /// It is **thread-local** because `libtest` runs the tests of one binary on
    /// parallel threads, and a process-wide value would decide the read-only
    /// verdict of a connection another test was opening. The value is read on
    /// the thread that calls [`enforce`], which is the thread that opened the
    /// connection: [`run`] blocks on the runtime from the caller's thread and
    /// the comparison against [`ENFORCED`] is made outside it.
    static MISREPORTED: std::cell::Cell<bool> = const { std::cell::Cell::new(false) };
}

/// Presents this thread's read-back with an answer that does not confirm the
/// setting, until the answer is dropped (`FR-SRV-013`).
///
/// `FR-SRV-013` requires the read-back of `FR-SRV-009` to be verified in both
/// of its outcomes, each by the test form that can reach it, and the failing
/// outcome is reachable by no other means. **No server produces it**: the
/// requirement records three fixture conditions that were tried, and under each
/// of them the session still reported the setting back, because a server that
/// accepts the statement of `FR-SRV-008` and does not apply it is defective
/// rather than configured. **No arrangement outside the process reaches it
/// either**: a seam on `FR-ERR-031`'s terms is reachable only from within this
/// system's own test configuration, and an integration test drives the binary
/// the project distributes, which carries no such seam. The requirement
/// therefore authorises this one, in its own text, and `BR-SRV-003` states the
/// exception it takes against the rule it excepts from.
///
/// `#[cfg(test)]` is the whole of that reachability rule, per `OD-21`, on the
/// same terms as [`super::window::narrow_to`]. The item is not compiled into
/// the artefact `cargo build` produces; an integration test links the library
/// compiled without that configuration and cannot see it either; and it appears
/// in no help text, in no JSON command tree of `FR-HELP-016` and in no command
/// tree of `FR-CLI-002`, because it is not a node of any tree. `FR-ERR-031`
/// rejects by name every mechanism that would reach it from outside the process
/// — a command or a flag, an environment variable, a build selected by a
/// feature — and each of those is rejected here for the same reason.
/// `FR-SRV-011` is untouched by it: the three statements of `FR-SRV-006` are
/// still issued, once each, in the order `FR-SRV-042` fixes, and what this
/// changes is the answer the second of them is taken to have given.
///
/// The answer restores the previous state when it is dropped, including on the
/// unwind of a failing assertion, so a misreported read-back cannot outlive the
/// body that asked for it.
#[cfg(test)]
#[must_use]
fn misreport_read_back() -> Misreport {
    Misreport {
        restore: MISREPORTED.with(|misreported| misreported.replace(true)),
    }
}

/// The misreporting [`misreport_read_back`] installed, which is undone when
/// this is dropped.
#[cfg(test)]
#[derive(Debug)]
struct Misreport {
    /// What the thread's state was before.
    restore: bool,
}

#[cfg(test)]
impl Drop for Misreport {
    fn drop(&mut self) {
        MISREPORTED.with(|misreported| misreported.set(self.restore));
    }
}

#[cfg(test)]
mod tests {
    use super::{ENFORCED, MARKER, PROBE, READ_BACK, READ_ONLY, misreport_read_back, resolve};
    use crate::error::{Error, ReadOnlyFault};
    use crate::mariadb::connect::Target;
    use crate::model::server::Standing;
    use crate::project::config;
    use crate::project::scratch::Scratch;
    use crate::project::settings::{self, Settings};

    // The harness is asked rather than restated, for the reason
    // `super::super::catalogue` gives where it declares the module: one file
    // reached by two `#[path]` items is two modules over one file, which
    // `clippy::duplicate_mod` refuses, so this **uses** the module declared
    // there rather than declaring it again.
    use crate::mariadb::catalogue::fixture;

    /// A lookup that defines nothing.
    fn nothing(_: &str) -> Option<String> {
        None
    }

    /// The settings entry `shop` resolves to, pointing at a host it never
    /// reaches: every test below decides a verdict over a version string.
    fn settings(scratch: &Scratch) -> Settings {
        let file = scratch.file(".cfg", "[database.shop]\nhost = \"db.example.com\"\n");
        let configuration = config::load(&file).expect("the document is valid");

        settings::resolve(
            &configuration,
            Some("shop"),
            &settings::clock(None),
            &nothing,
        )
        .expect("the entry resolves")
    }

    /// The verdict `version` earns.
    fn verdict(scratch: &Scratch, version: &str) -> Result<Standing, Error> {
        let resolved = settings(scratch);
        let target = Target::of(&resolved).expect("the entry names a host");

        resolve(&target, version.to_owned()).map(|server| server.standing())
    }

    #[test]
    fn fr_srv_006_the_three_connection_start_statements_are_the_ones_the_list_admits() {
        // FR-SRV-006: the read-only session statement, one read of
        // @@session.tx_read_only reading nothing else, and the version probe.
        // FR-SRV-009 names the variable, and FR-SRV-038 difference 12 is why:
        // `transaction_read_only` does not exist on 10.11.
        assert_eq!(READ_ONLY, "SET SESSION TRANSACTION READ ONLY");
        assert_eq!(READ_BACK, "SELECT @@session.tx_read_only");
        assert_eq!(PROBE, "SELECT VERSION()");

        assert!(!READ_BACK.contains("transaction_read_only"));
        assert!(!READ_BACK.contains(','));
        assert_eq!(ENFORCED, 1);
    }

    #[test]
    fn fr_srv_007_no_statement_of_this_module_reaches_a_schema() {
        // FR-SRV-007: no DDL, no DML, no SHOW, and no statement against any
        // schema other than INFORMATION_SCHEMA. The three here touch none.
        for statement in [READ_ONLY, READ_BACK, PROBE] {
            let upper = statement.to_uppercase();

            assert!(!upper.contains("SHOW"), "{statement}");
            assert!(!upper.contains("FROM"), "{statement}");
            assert!(!upper.contains("INSERT"), "{statement}");
            assert!(!upper.contains("UPDATE"), "{statement}");
            assert!(!upper.contains("DELETE"), "{statement}");
            assert!(!upper.contains("CREATE"), "{statement}");
        }
    }

    #[test]
    fn fr_srv_015_a_series_of_the_window_is_supported() {
        // The four strings are the ones the four fixture servers returned
        // verbatim to the observation pass of FR-SRV-040.
        let scratch = Scratch::new();

        for version in [
            "10.11.19-MariaDB-ubu2204",
            "11.4.13-MariaDB-ubu2404",
            "11.8.9-MariaDB-ubu2404",
            "12.3.3-MariaDB-ubu2404",
        ] {
            assert_eq!(verdict(&scratch, version).ok(), Some(Standing::Supported));
        }
    }

    #[test]
    fn fr_srv_031_a_series_above_the_window_is_read_and_marked() {
        // FR-SRV-031 and FR-SRV-032: the catalogue is read, the marking is in
        // the document, and the exit code does not change on account of it.
        let scratch = Scratch::new();

        assert_eq!(
            verdict(&scratch, "12.4.0-MariaDB").ok(),
            Some(Standing::NewerThanSupported)
        );
        assert_eq!(
            verdict(&scratch, "13.0.1-MariaDB-ubu2404").ok(),
            Some(Standing::NewerThanSupported)
        );
    }

    #[test]
    fn fr_srv_030_a_series_below_the_window_names_the_series_and_the_entry() {
        // FR-SRV-030: the message names the series found and the database
        // entry that reached it, and the cause lists the series that are
        // supported — which BR-SRV-005 puts on the value, not in the renderer.
        let scratch = Scratch::new();

        match verdict(&scratch, "10.6.21-MariaDB").expect_err("10.6 is below the window") {
            Error::SeriesNotSupported {
                entry,
                series,
                supported,
            } => {
                assert_eq!(entry, "shop");
                assert_eq!(series, "10.6");
                assert_eq!(supported, ["12.3", "11.8", "11.4", "10.11"]);
            }
            other => panic!("expected an unsupported series, got {other:?}"),
        }
    }

    #[test]
    fn fr_srv_020_the_refusal_below_the_window_is_78_and_reads_no_catalogue() {
        let scratch = Scratch::new();
        let condition = verdict(&scratch, "12.1.0-MariaDB").expect_err("12.1 is a rolling release");

        assert_eq!(condition.exit_code(), 78);
    }

    #[test]
    fn fr_srv_041_a_version_string_without_the_marker_is_not_taken_to_be_mariadb() {
        // FR-SRV-041 and FR-SRV-003: the marker is the necessary condition,
        // and the cause names the product the server reported.
        let scratch = Scratch::new();

        match verdict(&scratch, "8.4.0").expect_err("the marker is absent") {
            Error::ServerNotMariaDb { entry, product } => {
                assert_eq!(entry, "shop");
                assert_eq!(product, "8.4.0");
            }
            other => panic!("expected a refused product, got {other:?}"),
        }
    }

    #[test]
    fn fr_srv_003_the_refusal_of_a_product_is_78() {
        let scratch = Scratch::new();
        let condition = verdict(&scratch, "8.4.0-MySQL").expect_err("the marker is absent");

        assert_eq!(condition.exit_code(), 78);
        assert!(!"8.4.0-MySQL".contains(MARKER));
    }

    #[test]
    fn fr_srv_040_a_string_carrying_the_marker_and_no_series_is_refused_as_a_product() {
        // FR-SRV-040 fixes one form. A string outside it yields no series, so
        // there is nothing for FR-SRV-020 to compare or to name.
        let scratch = Scratch::new();

        for version in ["MariaDB", "11.4-MariaDB", "x.y.z-MariaDB", ""] {
            let condition = verdict(&scratch, version).expect_err("no series can be read");

            assert!(
                matches!(condition, Error::ServerNotMariaDb { .. }),
                "{version}: {condition:?}"
            );
        }
    }

    #[test]
    fn fr_srv_038_the_greeting_would_be_refused_where_the_probes_answer_is_not() {
        // Difference 13 of FR-SRV-038: on 10.11 the connection greeting
        // carries a `5.5.5-` prefix the probe's answer does not, and a series
        // read from it would be `5.5`. The trap is pinned here as well as in
        // `model::server`, because this is the module that chooses which
        // string to feed the derivation.
        let scratch = Scratch::new();
        let greeting = "5.5.5-10.11.19-MariaDB-ubu2204";

        assert!(verdict(&scratch, greeting).is_err());
        assert_eq!(
            verdict(&scratch, "10.11.19-MariaDB-ubu2204").ok(),
            Some(Standing::Supported)
        );
    }

    #[test]
    fn fr_ctx_031_the_version_reaches_the_model_exactly_as_the_probe_returned_it() {
        // FR-SRV-028 carries the probe's answer into the model, and
        // FR-SRV-040 bars the build suffix from reaching any field but
        // `version`.
        let scratch = Scratch::new();
        let resolved = settings(&scratch);
        let target = Target::of(&resolved).expect("the entry names a host");
        let server = resolve(&target, "11.8.9-MariaDB-ubu2404".to_owned())
            .expect("11.8 is a series of the window");

        assert_eq!(server.version(), "11.8.9-MariaDB-ubu2404");
        assert_eq!(server.series(), "11.8");
        assert_eq!(server.standing(), Standing::Supported);
    }

    // ------------------------------------------- against the fixture ---

    /// The privileged account of the fixture.
    const ROOT: (&str, &str) = ("root", "tpl-root");

    /// The database entry the body below reaches a fixture server through.
    const FIXTURE_ENTRY: &str = "fixture";

    /// The schema every fixture server carries.
    const FIXTURE_SCHEMA: &str = "freight";

    /// The settings that reach `server` as the privileged account.
    ///
    /// The document is composed by the harness's own helper, so no port and no
    /// address is written in Rust: the inventory is asked of
    /// `scripts/mariadb/`, which is the only way a test reaches the fixture.
    fn reaching(scratch: &Scratch, server: &fixture::Server) -> Settings {
        let file = scratch.file(
            ".cfg",
            &fixture::configuration(server, FIXTURE_ENTRY, FIXTURE_SCHEMA, ROOT),
        );
        let configuration = config::load(&file).expect("the document is valid");

        settings::resolve(
            &configuration,
            Some(FIXTURE_ENTRY),
            &settings::clock(None),
            &nothing,
        )
        .expect("the entry resolves")
    }

    /// The statement text of each row the server's record holds for the account
    /// under test, in the order the server received them.
    ///
    /// `observe.sh statements dump` prints a thread, a command type and the
    /// statement; this keeps the last, because what is asserted below is which
    /// statements arrived and which did not.
    fn received(server: &fixture::Server) -> Vec<String> {
        fixture::statements_text(server, &["--user", ROOT.0])
            .lines()
            .skip(1)
            .filter_map(|row| row.splitn(3, '\t').nth(2))
            .map(|statement| statement.trim().to_owned())
            .collect()
    }

    #[test]
    fn fr_srv_013_a_read_back_that_does_not_confirm_is_refused_before_the_catalogue() {
        // FR-SRV-013's **failing** outcome: a read-back that does not confirm
        // the setting. The requirement verifies each outcome by the test form
        // that can reach it, and this one is reachable by no other means —
        // no server produces it, and no arrangement outside the process
        // presents it — so the requirement authorises the seam of FR-ERR-031
        // in its own text and BR-SRV-003 yields for this clause alone.
        //
        // **The body is a unit test and not an integration test** because that
        // seam is `#[cfg(test)]`: an integration test links the library
        // compiled without that configuration and cannot see it.
        //
        // **It is bound to no series**, unlike the confirming outcome, which
        // FR-SRV-013 binds to every series of FR-SRV-015. The binding exists
        // there because the spelling FR-SRV-009 names is discriminated by
        // exactly one series of the window; here the answer is not the
        // server's, so no series can be named for it. One real server is used
        // all the same, so that the connection start under test is the real one
        // — a real handshake, the real statement of FR-SRV-008 and the real
        // read-back of FR-SRV-009 — and only the answer is replaced.
        //
        // **What this does not establish**, per the requirement's own
        // consequence note: no invocation of the distributed binary is observed
        // refusing on a read-back that did not confirm, and no server is
        // observed producing one. What is observed is the decision itself, in
        // process, and separately the step from a condition to the exit status
        // it carries — `78` (EX_CONFIG), which BR-ERR-001 obliges to have an
        // integration test and which other producing conditions of that code
        // reach from an invocation. The composition of the two is reasoned
        // rather than executed.
        let Some(series) = fixture::series(
            "fr_srv_013_a_read_back_that_does_not_confirm_is_refused_before_the_catalogue",
        ) else {
            return;
        };
        let _exclusive = fixture::exclusive();

        let server = series
            .first()
            .expect("the gate exited 0, so the inventory named a server");
        let name = server.name();

        let scratch = Scratch::new();
        let resolved = reaching(&scratch, server);
        let target = Target::of(&resolved).expect("the entry names a host");
        let clock = settings::clock(None);

        fixture::statements_on(server);

        let condition = {
            // The seam is installed for the connection start and for nothing
            // else: the guard is dropped with this block, so a later body on
            // this thread reads the answer the server gave.
            let _misreported = misreport_read_back();

            crate::mariadb::open(&target, &clock)
                .map(crate::mariadb::Session::close)
                .expect_err("the read-back was presented with an answer that does not confirm")
        };

        fixture::statements_off(server);

        let statements = received(server);

        // FR-SRV-010 gives both halves of the read-only pair one code, and
        // FR-ERR-034 obliges the `cause` line to separate them. This is the
        // half **this read-back** decides, and it is distinct from the half a
        // statement the server refuses would produce.
        match condition {
            Error::ReadOnlySessionNotEnforced { ref entry, fault } => {
                assert_eq!(entry, FIXTURE_ENTRY, "{name}");
                assert_eq!(fault, ReadOnlyFault::ReadBackDisagreed, "{name}");
                assert_ne!(fault, ReadOnlyFault::NotApplied, "{name}");
            }
            ref other => panic!("{name}: expected the read-back half of FR-SRV-010, got {other:?}"),
        }

        assert_eq!(
            condition.exit_code(),
            78,
            "{name}: {condition:?} — FR-SRV-010 with FR-ERR-006 fixes 78 (EX_CONFIG)"
        );

        // The control, and it comes first: the record held what this attempt
        // issued, so the two absences below are observations rather than an
        // empty log. Both statements of the read-only pair were really sent —
        // the seam replaces the answer, not the statement.
        assert!(
            statements.iter().any(|statement| statement == READ_ONLY),
            "{name}: the statement record held nothing this attempt issued"
        );
        assert!(
            statements.iter().any(|statement| statement == READ_BACK),
            "{name}: the read-back was not issued"
        );

        // `enforce` fails before `probe`: the version probe of FR-SRV-002 is
        // the third statement of FR-SRV-042's order and is never reached.
        assert!(
            !statements.iter().any(|statement| statement == PROBE),
            "{name}: the version probe followed a read-back that did not confirm"
        );

        // And before the fourth kind of FR-SRV-006's closed list, which is what
        // FR-SRV-010 requires of this condition: the catalogue is not read.
        assert!(
            !statements
                .iter()
                .any(|statement| statement.contains("INFORMATION_SCHEMA")),
            "{name}: a catalogue statement was issued: {statements:?}"
        );
    }

    #[test]
    fn fr_srv_013_the_seam_is_installed_for_one_body_and_restores_what_it_replaced() {
        // The seam of FR-ERR-031 is a test hook, and a test hook that outlived
        // the body that asked for it would decide the read-only verdict of a
        // connection another body opened. The guard restores on drop, so the
        // state is false before, true within, and false after — including on
        // the unwind of a failing assertion, which is what `Drop` gives it.
        assert!(!super::misreporting());

        {
            let _misreported = misreport_read_back();

            assert!(super::misreporting());
        }

        assert!(!super::misreporting());
    }
}
