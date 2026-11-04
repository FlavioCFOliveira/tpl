//! The tests of a `mysql` read that must run in process: the two seams
//! `FR-MYSQL-043` authorises through `FR-SRV-013` and `FR-SRV-035`, the
//! decoding of every field of the twelve tables, the classification a
//! `mariadb` entry keeps for the refusal of `ADR-014`, and the count of a read
//! as the value it produced.
//!
//! Each body runs against the fixture of `scripts/mysql/`, through its harness,
//! on every series of `FR-MYSQL-003`, and skips with its reason where the
//! fixture is not up.

use sqlx_core::row::Row as _;
use sqlx_mysql::MySqlRow;

use super::fixture::{self, Role, Server};
use super::statements::Read;
use crate::error::{Error, ReadOnlyFault};
use crate::mariadb::connect::Target;
use crate::model::server::Standing;
use crate::mysql::session::{PROBE, READ_BACK, READ_ONLY};
use crate::project::config;
use crate::project::scratch::Scratch;
use crate::project::settings::{self, Settings};

/// The entry every body reaches a server through.
const ENTRY: &str = "fixture";

/// A lookup that defines nothing.
fn nothing(_: &str) -> Option<String> {
    None
}

/// The settings of a `mysql` entry reaching `server` as root, over `tls`.
fn reaching(scratch: &Scratch, server: &Server, schema: &str, tls: &str) -> Settings {
    let names = fixture::names();
    let root = (names.root.0.as_str(), names.root.1.as_str());
    let file = scratch.file(
        ".cfg",
        &fixture::configuration(server, ENTRY, schema, root, tls),
    );
    let configuration = config::load(&file).expect("the document is valid");

    settings::resolve(
        &configuration,
        Some(ENTRY),
        &settings::clock(None),
        &nothing,
    )
    .expect("the entry resolves")
    .into_mysql()
    .expect("the entry is a mysql entry")
}

/// The field each of the twelve tables names its schema by.
fn schema_field(read: Read) -> &'static str {
    match read {
        Read::Schema => "SCHEMA_NAME",
        Read::ForeignKeyRules | Read::Checks => "CONSTRAINT_SCHEMA",
        Read::Routines => "ROUTINE_SCHEMA",
        Read::Parameters => "SPECIFIC_SCHEMA",
        Read::Triggers => "TRIGGER_SCHEMA",
        Read::Tables
        | Read::Columns
        | Read::Indexes
        | Read::KeyColumns
        | Read::CheckOwners
        | Read::Views => "TABLE_SCHEMA",
    }
}

#[test]
fn adr_014_every_field_of_the_twelve_tables_decodes_through_the_driver_on_every_series() {
    // ADR-014: the decoder refuses a result column whose wire type it does not
    // list, and that every field of the twelve tables of FR-MYSQL-015 arrives
    // on a listed one is for the provider's tests to observe. Every field is
    // asked for — `SELECT *` — restricted to the fixture's schema, and every
    // value of every row is taken as its raw value.
    let Some(series) = fixture::series(
        "adr_014_every_field_of_the_twelve_tables_decodes_through_the_driver_on_every_series",
    ) else {
        return;
    };
    let _exclusive = fixture::exclusive();
    let schema = fixture::names().schema.clone();

    for server in &series {
        let scratch = Scratch::new();
        let resolved = reaching(&scratch, server, &schema, "verify-identity");
        let target = Target::of(&resolved).expect("the entry names a host");
        let clock = settings::clock(None);
        let mut session = crate::mysql::open(&target, &clock).expect("the session opens");

        for read in Read::ALL {
            let sql = format!(
                "SELECT * FROM INFORMATION_SCHEMA.{} WHERE {} = ?",
                read.table(),
                schema_field(read)
            );
            let (runtime, connection) = session.parts();
            let connection = connection.expect("the session is open");
            let rows: Vec<MySqlRow> = runtime
                .block_on(
                    sqlx_core::query::query::<sqlx_mysql::MySql>(
                        sqlx_core::sql_str::AssertSqlSafe(sql),
                    )
                    .bind(schema.as_str())
                    .fetch_all(connection),
                )
                .unwrap_or_else(|failure| {
                    panic!(
                        "{}: {} did not decode: {failure}",
                        server.name(),
                        read.table()
                    )
                });

            assert!(
                !rows.is_empty(),
                "{}: {} returned no row",
                server.name(),
                read.table()
            );

            for row in &rows {
                for index in 0..row.len() {
                    row.try_get_raw(index).unwrap_or_else(|failure| {
                        panic!(
                            "{}: {} field {index}: {failure}",
                            server.name(),
                            read.table()
                        )
                    });
                }
            }
        }

        session.close();
    }
}

#[test]
fn fr_mysql_042_a_read_issues_the_twelve_statements_of_the_plan_and_nothing_else() {
    // FR-MYSQL-042 with NFR-PERF-001: the count is a property of the read, read
    // off the value it produced, over the fixture's database and over the one
    // that holds no object.
    let Some(series) = fixture::series(
        "fr_mysql_042_a_read_issues_the_twelve_statements_of_the_plan_and_nothing_else",
    ) else {
        return;
    };
    let _exclusive = fixture::exclusive();
    let names = fixture::names();

    for server in &series {
        for schema in [&names.schema, &names.empty_schema] {
            let scratch = Scratch::new();
            let resolved = reaching(&scratch, server, schema, "verify-identity");
            let target = Target::of(&resolved).expect("the entry names a host");
            let clock = settings::clock(None);
            let mut session = crate::mysql::open(&target, &clock).expect("the session opens");
            let read = super::read(&mut session, &target, &clock, schema);
            session.close();

            let read = read.expect("the catalogue reads");
            let expected: Vec<&str> = Read::ALL.iter().map(|read| read.sql()).collect();

            assert_eq!(
                read.statements(),
                expected.as_slice(),
                "{} {schema}",
                server.name()
            );
            read.model().expect("the rows fold into the model");
        }
    }
}

#[test]
fn fr_mysql_043_a_read_back_that_does_not_confirm_is_refused_before_the_catalogue() {
    // FR-MYSQL-043 applies FR-SRV-013: the failing outcome of the read-back of
    // FR-MYSQL-018 is verified in process, through the seam that requirement
    // authorises, against a real server: the real handshake, the real
    // statement and the real read-back, with only the answer replaced.
    let Some(series) = fixture::series(
        "fr_mysql_043_a_read_back_that_does_not_confirm_is_refused_before_the_catalogue",
    ) else {
        return;
    };
    let _exclusive = fixture::exclusive();
    let names = fixture::names();
    let server = series.first().expect("the gate admitted a series");

    let scratch = Scratch::new();
    let resolved = reaching(&scratch, server, &names.schema, "verify-identity");
    let target = Target::of(&resolved).expect("the entry names a host");
    let clock = settings::clock(None);

    fixture::statements_on(server);
    let condition = {
        let _misreported = crate::mariadb::session::misreport_read_back();

        crate::mysql::open(&target, &clock)
            .map(crate::mysql::Session::close)
            .expect_err("the read-back was presented with an answer that does not confirm")
    };
    fixture::statements_off(server);

    match condition {
        Error::ReadOnlySessionNotEnforced {
            ref entry,
            fault,
            read_back,
            ..
        } => {
            assert_eq!(entry, ENTRY);
            assert_eq!(fault, ReadOnlyFault::ReadBackDisagreed);
            assert_eq!(read_back, "@@session.transaction_read_only");
        }
        ref other => panic!("expected the read-back half of FR-MYSQL-019, got {other:?}"),
    }
    assert_eq!(condition.exit_code(), 78);

    let statements: Vec<String> = fixture::received(server, &names.root.0)
        .into_iter()
        .map(|(_, statement)| statement)
        .collect();

    // The control: both statements of the pair were sent.
    assert!(
        statements.iter().any(|statement| statement == READ_ONLY),
        "{statements:?}"
    );
    assert!(
        statements.iter().any(|statement| statement == READ_BACK),
        "{statements:?}"
    );
    // The probe and the catalogue follow a confirmed read-back only.
    assert!(
        !statements.iter().any(|statement| statement == PROBE),
        "{statements:?}"
    );
    assert!(
        !statements
            .iter()
            .any(|statement| statement.contains("INFORMATION_SCHEMA")),
        "{statements:?}"
    );
}

#[test]
fn fr_mysql_043_a_series_newer_than_the_window_is_read_and_marked_through_the_seam() {
    // FR-MYSQL-043 applies FR-SRV-035: narrowed to 8.4, the 9.7 server is
    // newer than the window, so its catalogue is read and the document says
    // `newer_than_supported`, as FR-MYSQL-010 requires of a series newer than
    // the newest of FR-MYSQL-003.
    let Some(series) = fixture::series(
        "fr_mysql_043_a_series_newer_than_the_window_is_read_and_marked_through_the_seam",
    ) else {
        return;
    };
    let _exclusive = fixture::exclusive();
    let names = fixture::names();
    let Some(newest) = series.iter().find(|server| server.series() == "9.7") else {
        panic!("the series of FR-MYSQL-003 include 9.7");
    };
    let oldest = crate::mariadb::window::Series::parse("8.4").expect("a series");

    let scratch = Scratch::new();
    let resolved = reaching(&scratch, newest, &names.schema, "verify-identity");
    let target = Target::of(&resolved).expect("the entry names a host");
    let clock = settings::clock(None);

    let read = {
        let _narrowed = crate::mariadb::window::narrow_to(oldest);
        crate::provider::read(
            crate::provider::Source::Mysql {
                target,
                database: &names.schema,
            },
            &clock,
        )
        .expect("a series newer than the window is read")
    };
    let model = read.model().expect("the rows fold");

    assert_eq!(model.server.standing(), Standing::NewerThanSupported);
    assert!(!model.tables.is_empty(), "the catalogue was read");
}

#[test]
fn adr_014_a_mariadb_entry_keeps_its_outcome_for_the_refusal_the_patch_adds() {
    // ADR-014, Consequences: a `mariadb` entry runs through the patched driver
    // and keeps exactly its present outcome — `Error::ConnectionRefused`, exit
    // 69, with the same message — where a login over a session TLS does not
    // protect needs the server's public key. A MySQL server with its cache
    // emptied is the one server of either fixture that asks for that key, and
    // a `mariadb` entry pointed at it reaches the refusal before its product
    // check. The `mysql` entry's outcome for the same login is FR-MYSQL-045's,
    // asserted from outside the process in `tests/mysql_provider.rs`.
    let Some(plain) = fixture::servers(
        "adr_014_a_mariadb_entry_keeps_its_outcome_for_the_refusal_the_patch_adds",
        Role::NoTls,
    ) else {
        return;
    };
    let _exclusive = fixture::exclusive();
    let names = fixture::names();

    for server in &plain {
        let scratch = Scratch::new();
        let reader = (names.reader.0.as_str(), names.reader.1.as_str());
        let text = fixture::configuration(server, ENTRY, &names.schema, reader, "disabled")
            .replace("provider = \"mysql\"\n", "");
        let file = scratch.file(".cfg", &text);
        let configuration = config::load(&file).expect("the document is valid");
        let resolved = settings::resolve(
            &configuration,
            Some(ENTRY),
            &settings::clock(None),
            &nothing,
        )
        .expect("the entry resolves")
        .into_mariadb()
        .expect("with no provider key the entry is a mariadb entry");
        let target = Target::of(&resolved).expect("the entry names a host");

        fixture::flush_authentication_cache(server);

        let condition = crate::mariadb::open(&target, &settings::clock(None))
            .map(crate::mariadb::Session::close)
            .expect_err("the login needs the key, which the driver does not request");

        let expected = Error::ConnectionRefused {
            by_dsn: false,
            entry: ENTRY.to_owned(),
            host: server.host().to_owned(),
            port: server.port().parse().expect("the harness exports a port"),
        };

        assert_eq!(
            condition.exit_code(),
            69,
            "{}: {condition:?}",
            server.name()
        );
        assert_eq!(
            crate::diagnostics::rendered(&condition),
            crate::diagnostics::rendered(&expected),
            "{}: a mariadb entry's message moved",
            server.name()
        );
    }
}
