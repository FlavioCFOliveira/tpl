//! The tests of a `mysql` read that must run in process: the two seams
//! `FR-MYSQL-043` authorises through `FR-SRV-013` and `FR-SRV-035`, the
//! decoding of every field of the twelve tables, the classification a
//! `mariadb` entry keeps for the refusal of `ADR-014`, the count of a read as
//! the value it produced, and the fold over rows returned in an order no
//! statement states.
//!
//! Each body runs against the fixture of `scripts/mysql/`, through its harness,
//! on every series of `FR-MYSQL-003`, and skips with its reason where the
//! fixture is not up.

use sqlx_core::column::Column as _;
use sqlx_core::row::Row as _;
use sqlx_mysql::MySqlRow;

use super::fixture::{self, Role, Server};
use super::statements::Read;
use super::{Catalogue, arrived};
use crate::error::{Error, ReadOnlyFault};
use crate::mariadb::connect::Target;
use crate::model::database::Database;
use crate::model::routine::RoutineKind;
use crate::model::server::Standing;
use crate::mysql::Session;
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
fn fr_err_047_a_mariadb_entry_meets_the_refusal_of_adr_014_as_its_third_outcome_at_69() {
    // FR-ERR-047, third outcome, with the note on FR-SEC-021 and ADR-014
    // point 4: a `mariadb` entry whose login, over a session TLS does not
    // protect, needs the server's public key is refused before the request is
    // written, and ends at 69 with the facts and the command FR-MYSQL-045
    // states. A MySQL server with its cache emptied is the one server of either
    // fixture that asks for that key, and a `mariadb` entry pointed at it
    // reaches the refusal before its read-back. Amended from the test of rmp
    // #329, which pinned the refused-TCP-connect message the seventy-seventh
    // edition replaced.
    let Some(plain) = fixture::servers(
        "fr_err_047_a_mariadb_entry_meets_the_refusal_of_adr_014_as_its_third_outcome_at_69",
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

        let expected = Error::PublicKeyRequestRefused {
            provider: crate::model::database::Provider::Mariadb,
            entry: ENTRY.to_owned(),
            user: names.reader.0.clone(),
            host: server.host().to_owned(),
            port: server.port().parse().expect("the harness exports a port"),
            tls: "disabled",
            no_tls_offered: false,
            method: "caching_sha2_password",
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
            "{}",
            server.name()
        );
    }
}

#[test]
fn fr_err_048_a_second_login_beyond_the_account_limit_is_69_on_every_series() {
    // FR-ERR-048, observed: the fixture's tpl_limited holds
    // MAX_USER_CONNECTIONS 1, so a second login while the first is open is
    // refused with a limit on connections: 69, not the 77 of a credential.
    let Some(series) =
        fixture::series("fr_err_048_a_second_login_beyond_the_account_limit_is_69_on_every_series")
    else {
        return;
    };
    let _exclusive = fixture::exclusive();
    let names = fixture::names();

    for server in &series {
        let scratch = Scratch::new();
        let file = scratch.file(
            ".cfg",
            &fixture::configuration(
                server,
                ENTRY,
                &names.schema,
                ("tpl_limited", "tpl-limited-pw"),
                "verify-identity",
            ),
        );
        let configuration = config::load(&file).expect("the document is valid");
        let resolved = settings::resolve(
            &configuration,
            Some(ENTRY),
            &settings::clock(None),
            &nothing,
        )
        .expect("the entry resolves")
        .into_mysql()
        .expect("the entry is a mysql entry");
        let target = Target::of(&resolved).expect("the entry names a host");
        let clock = settings::clock(None);

        let first = crate::mysql::open(&target, &clock).expect("the first login opens");
        let second = crate::mysql::open(&target, &clock).map(crate::mysql::Session::close);
        first.close();

        let condition = second.expect_err("the second login is beyond the limit");
        assert_eq!(
            condition.exit_code(),
            69,
            "{}: {condition:?}",
            server.name()
        );
        match condition {
            Error::LoginRefused { number, reason, .. } => {
                fixture::notice(&format!(
                    "{}: the second login of tpl_limited was refused with server error {number}, \
                     classified {reason:?}",
                    server.name()
                ));
                assert_eq!(number, 1226, "{}", server.name());
                assert_eq!(
                    reason,
                    crate::error::LoginRefusal::ResourceLimit,
                    "{}",
                    server.name()
                );
            }
            other => panic!("{}: {other:?}", server.name()),
        }
    }
}

// ------------------------------------------------ rows in any order (#351) ---

/// The seeds each order-independence body permutes the rows with.
const SEEDS: [u64; 5] = [1, 2, 3, 0x5eed, 0xdead_beef];

/// The clause each statement carried before rmp `#351`, under which the server
/// returned its rows in the orders the fold now makes: the reference the fold
/// is compared with. The two statements that keep theirs add nothing.
const fn ordered_as_before(read: Read) -> &'static str {
    match read {
        Read::Tables | Read::Views => " ORDER BY TABLE_NAME",
        Read::Columns => " ORDER BY TABLE_NAME, ORDINAL_POSITION",
        Read::Indexes => " ORDER BY TABLE_NAME, INDEX_NAME, SEQ_IN_INDEX",
        Read::ForeignKeyRules | Read::CheckOwners => " ORDER BY TABLE_NAME, CONSTRAINT_NAME",
        Read::KeyColumns => " ORDER BY TABLE_NAME, CONSTRAINT_NAME, ORDINAL_POSITION",
        Read::Checks => " ORDER BY CONSTRAINT_NAME",
        Read::Triggers => " ORDER BY EVENT_OBJECT_TABLE, TRIGGER_NAME",
        Read::Schema | Read::Routines | Read::Parameters => "",
    }
}

/// Whether the server still states the order of this statement's rows, which
/// the fold then keeps rather than makes.
const fn ordered_by_the_server(read: Read) -> bool {
    matches!(read, Read::Routines | Read::Parameters)
}

/// A permutation of `0..len`, drawn from `seed` by xorshift64* and the
/// Fisher–Yates shuffle, as the `sqlite` reader's tests draw theirs, so that a
/// failure reproduces.
fn permutation(len: usize, seed: u64) -> Vec<usize> {
    let mut state = seed.max(1);
    let mut next = || {
        state ^= state >> 12;
        state ^= state << 25;
        state ^= state >> 27;
        state.wrapping_mul(0x2545_f491_4f6c_dd1d)
    };
    let mut order: Vec<usize> = (0..len).collect();
    for last in (1..len).rev() {
        let bound = u64::try_from(last + 1).expect("a length fits in u64");
        let pick = usize::try_from(next() % bound).expect("below a length");
        order.swap(last, pick);
    }
    order
}

/// `rows` in the order `order` draws.
fn reordered(rows: Vec<MySqlRow>, order: &[usize]) -> Vec<MySqlRow> {
    let mut slots: Vec<Option<MySqlRow>> = rows.into_iter().map(Some).collect();
    order
        .iter()
        .filter_map(|&at| slots.get_mut(at).and_then(Option::take))
        .collect()
}

/// `catalogue` as a read would hold it had every statement whose order the
/// server no longer states returned its rows in the order `seed` draws: each
/// such statement's rows permuted, then kept as [`arrived`] keeps them.
fn permute(catalogue: &mut Catalogue, seed: u64) {
    for read in Read::ALL {
        if ordered_by_the_server(read) {
            continue;
        }
        let slot = read.slot();
        let rows = std::mem::take(&mut catalogue.rows[slot]);
        let drawn = seed ^ u64::try_from(slot).expect("a slot fits in u64");
        let order = permutation(rows.len(), drawn);
        catalogue.rows[slot] =
            arrived(read, reordered(rows, &order)).expect("the permuted rows decode");
    }
}

/// The rows `sql` returns with `schema` bound to each of its `binds`
/// parameters, on `session`'s connection.
fn fetched(session: &mut Session, sql: String, schema: &str, binds: usize) -> Vec<MySqlRow> {
    let (runtime, connection) = session.parts();
    let connection = connection.expect("the session is open");
    let mut query = sqlx_core::query::query::<sqlx_mysql::MySql>(
        sqlx_core::sql_str::AssertSqlSafe(sql.clone()),
    );
    for _ in 0..binds {
        query = query.bind(schema);
    }
    runtime
        .block_on(query.fetch_all(connection))
        .unwrap_or_else(|failure| panic!("{sql}: {failure}"))
}

/// The catalogue the plan's statements return under the clauses each carried
/// before rmp `#351`: the rows in the orders the server used to give.
fn as_before(session: &mut Session, schema: &str) -> Catalogue {
    let rows = std::array::from_fn(|slot| {
        let read = Read::ALL[slot];
        // A statement that still states its order is its own reference.
        let sql = if read.sql().contains(" ORDER BY ") {
            read.sql().to_owned()
        } else {
            format!("{}{}", read.sql(), ordered_as_before(read))
        };
        fetched(session, sql, schema, 1)
    });

    Catalogue {
        server: session.server().clone(),
        schema: schema.to_owned(),
        issued: Vec::new(),
        rows,
    }
}

/// How many runs of one owner the rows of `read` form: more than the owners
/// themselves where a permutation split them.
fn runs(catalogue: &Catalogue, read: Read, fields: &[&str]) -> (usize, usize) {
    let keys: Vec<Vec<&str>> = catalogue
        .rows(read)
        .iter()
        .map(|row| {
            fields
                .iter()
                .map(|field| row.try_get::<&str, _>(*field).expect("a name decodes"))
                .collect()
        })
        .collect();
    let runs = keys.windows(2).filter(|pair| pair[0] != pair[1]).count() + 1;
    let mut owners = keys.clone();
    owners.sort_unstable();
    owners.dedup();
    (runs, owners.len())
}

/// Asserts the orders the model owes that the fold now makes, each over the
/// values the model carries (`FR-MYSQL-027`, `NFR-DET-002`).
fn assert_ordered(database: &Database<'_>, context: &str) {
    let ascending = |names: Vec<&str>| names.windows(2).all(|pair| pair[0] < pair[1]);

    assert!(
        ascending(database.tables.iter().map(|table| table.name()).collect()),
        "{context}: tables"
    );
    assert!(
        ascending(database.views.iter().map(|view| &*view.name).collect()),
        "{context}: views"
    );
    for table in &database.tables {
        let name = table.name();
        assert!(
            table
                .columns()
                .windows(2)
                .all(|pair| pair[0].position < pair[1].position),
            "{context}: the columns of {name}"
        );
        assert!(
            ascending(table.indexes().iter().map(|index| &*index.name).collect()),
            "{context}: the indexes of {name}"
        );
        assert!(
            ascending(
                table
                    .triggers()
                    .iter()
                    .map(|trigger| &*trigger.name)
                    .collect()
            ),
            "{context}: the triggers of {name}"
        );
        assert!(
            ascending(
                table
                    .check_constraints()
                    .iter()
                    .map(|check| &*check.name)
                    .collect()
            ),
            "{context}: the checks of {name}"
        );
    }
}

#[test]
fn fr_mysql_027_032_033_the_fold_orders_rows_returned_in_any_order() {
    // rmp #351: only the routine and parameter statements state an order, so
    // every other order the model carries must come from the fold, whatever
    // order the rows arrive in. The model folded from the rows as returned,
    // and from the rows of every such statement in five drawn orders — a
    // table's columns, an index's members and a key's columns coming apart
    // into many runs — is the model the rows fold to under the clauses the
    // statements carried before, and every order it owes holds.
    let Some(series) =
        fixture::series("fr_mysql_027_032_033_the_fold_orders_rows_returned_in_any_order")
    else {
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
        let before = as_before(&mut session, &schema);
        let mut read = super::read(&mut session, &target, &clock, &schema);
        session.close();

        let expected = before.model().expect("the ordered rows fold");
        let name = server.name();
        assert_ordered(&expected, name);
        // The orders are not vacuous: an index and a key of several columns.
        assert!(
            expected
                .tables
                .iter()
                .any(|table| table.indexes().iter().any(|index| index.columns.len() > 1)),
            "{name}"
        );
        assert!(
            expected
                .tables
                .iter()
                .any(|table| table.foreign_keys().iter().any(|key| key.columns.len() > 1)),
            "{name}"
        );

        let read = read.as_mut().expect("the catalogue reads");
        assert_eq!(
            read.model().expect("the rows as returned fold"),
            expected,
            "{name}: as returned"
        );

        for seed in SEEDS {
            permute(read, seed);
            for (statement, fields) in [
                (Read::Columns, &["TABLE_NAME"][..]),
                (Read::Indexes, &["TABLE_NAME", "INDEX_NAME"][..]),
            ] {
                let (runs, owners) = runs(read, statement, fields);
                assert!(runs > owners, "{name}: seed {seed}: {statement:?}");
            }

            let database = read.model().expect("the permuted rows fold");
            assert_ordered(&database, &format!("{name}: seed {seed}"));
            assert_eq!(database, expected, "{name}: seed {seed}");
        }
    }
}

#[test]
fn fr_mysql_027_positions_that_start_at_2_keep_the_order_of_their_values() {
    // FR-MYSQL-027: with the generated invisible primary key hidden, the
    // fixture's equipment_survey reports its columns from position 2. The
    // order is the values', in any order the rows arrive in, and the model is
    // the one the rows fold to in the order the statement used to state. The
    // setting is the test's own session's, made before the read; no statement
    // of the read names it (FR-MYSQL-016).
    let Some(series) =
        fixture::series("fr_mysql_027_positions_that_start_at_2_keep_the_order_of_their_values")
    else {
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
        fetched(
            &mut session,
            "SET SESSION show_gipk_in_create_table_and_information_schema = OFF".to_owned(),
            &schema,
            0,
        );
        let before = as_before(&mut session, &schema);
        let mut read = super::read(&mut session, &target, &clock, &schema);
        session.close();

        let name = server.name();
        let expected = before.model().expect("the ordered rows fold");
        let survey = expected
            .tables
            .iter()
            .find(|table| table.name() == "equipment_survey")
            .expect("the fixture carries a table with a generated invisible primary key");
        assert_eq!(
            survey.columns().first().map(|column| column.position),
            Some(2),
            "{name}"
        );
        assert!(
            survey
                .columns()
                .iter()
                .all(|column| column.name != "my_row_id"),
            "{name}"
        );
        assert!(survey.primary_key().is_none(), "{name}");
        assert_ordered(&expected, name);

        let read = read.as_mut().expect("the catalogue reads");
        assert_eq!(read.model().expect("the rows fold"), expected, "{name}");
        for seed in SEEDS {
            permute(read, seed);
            assert_eq!(
                read.model().expect("the permuted rows fold"),
                expected,
                "{name}: seed {seed}"
            );
        }
    }
}

#[test]
fn fr_mysql_036_a_function_and_a_procedure_of_one_name_keep_the_order_the_statement_states() {
    // FR-CAT-018: a function and a procedure may share a name, and the fold
    // orders the routines by name with a stable sort, so between two of one
    // name the order is the routine statement's — which is why it keeps its
    // ORDER BY (rmp #351). The fixture holds no such pair and the reader
    // writes nothing, so the pair is the statement's own rows: every function
    // of the fixture, and each again with its type read as PROCEDURE.
    let Some(series) = fixture::series(
        "fr_mysql_036_a_function_and_a_procedure_of_one_name_keep_the_order_the_statement_states",
    ) else {
        return;
    };
    let _exclusive = fixture::exclusive();
    let schema = fixture::names().schema.clone();
    let (body, order) = Read::Routines
        .sql()
        .split_once(" ORDER BY ")
        .expect("the routine statement orders its rows");
    let twin = body.replacen("ROUTINE_TYPE,", "'PROCEDURE' AS ROUTINE_TYPE,", 1);
    assert_ne!(twin, body);
    let paired = format!(
        "({body} AND ROUTINE_TYPE = 'FUNCTION') UNION ALL \
         ({twin} AND ROUTINE_TYPE = 'FUNCTION') ORDER BY {order}"
    );

    for server in &series {
        let scratch = Scratch::new();
        let resolved = reaching(&scratch, server, &schema, "verify-identity");
        let target = Target::of(&resolved).expect("the entry names a host");
        let clock = settings::clock(None);
        let mut session = crate::mysql::open(&target, &clock).expect("the session opens");
        let rows = fetched(&mut session, paired.clone(), &schema, 2);
        let mut read = super::read(&mut session, &target, &clock, &schema);
        session.close();

        let name = server.name();
        let read = read.as_mut().expect("the catalogue reads");
        let count = rows.len();
        read.rows[Read::Routines.slot()] = rows;

        let kinds = |database: &Database<'_>| -> Vec<(String, bool)> {
            database
                .routines
                .iter()
                .map(|routine| {
                    (
                        routine.name.to_string(),
                        matches!(routine.kind, RoutineKind::Function),
                    )
                })
                .collect()
        };

        // As the statement orders them: the function first, as FUNCTION
        // precedes PROCEDURE, each name twice in a row.
        let stated = kinds(&read.model().expect("the paired rows fold"));
        assert_eq!(stated.len(), count, "{name}");
        assert!(count >= 2, "{name}: the fixture holds a function");
        for pair in stated.chunks(2) {
            assert_eq!(pair[0].0, pair[1].0, "{name}: {stated:?}");
            assert!(pair[0].1 && !pair[1].1, "{name}: {stated:?}");
        }

        // Returned the other way round, the pair keeps the order it arrived
        // in: the order between them is the statement's to state.
        let routines = std::mem::take(&mut read.rows[Read::Routines.slot()]);
        let reversed: Vec<usize> = (0..routines.len()).rev().collect();
        read.rows[Read::Routines.slot()] = reordered(routines, &reversed);
        let arrived = kinds(&read.model().expect("the reversed rows fold"));
        for pair in arrived.chunks(2) {
            assert_eq!(pair[0].0, pair[1].0, "{name}: {arrived:?}");
            assert!(!pair[0].1 && pair[1].1, "{name}: {arrived:?}");
        }
    }
}

/// The integer accessor `row.rs` replaced in rmp `#351`, kept as the oracle of
/// its replacement: an unsigned ask, and a signed one where it failed.
fn asked_twice(row: &MySqlRow, field: &str) -> Result<Option<u64>, ()> {
    if let Ok(unsigned) = row.try_get::<Option<u64>, _>(field) {
        return Ok(unsigned);
    }
    let signed = row.try_get::<Option<i64>, _>(field).map_err(|_| ())?;
    signed
        .map(|value| u64::try_from(value).map_err(|_| ()))
        .transpose()
}

#[test]
fn adr_014_the_integer_accessor_answers_every_field_as_the_two_typed_asks_did() {
    // rmp #351: `maybe_integer` reads the declared type once instead of
    // asking for an unsigned value and then a signed one. Over every field of
    // every row of the twelve statements, of every type, and over values the
    // catalogue does not hold — a negative, the extremes of both widths, SQL
    // NULL of three types, text, a fraction and a field the row does not
    // carry — both answer alike, on every series.
    let Some(series) = fixture::series(
        "adr_014_the_integer_accessor_answers_every_field_as_the_two_typed_asks_did",
    ) else {
        return;
    };
    let _exclusive = fixture::exclusive();
    let schema = fixture::names().schema.clone();
    let edges = "SELECT CAST(-1 AS SIGNED) AS negative, CAST(7 AS UNSIGNED) AS positive, \
         CAST(9223372036854775807 AS SIGNED) AS largest, 18446744073709551615 AS widest, \
         NULL AS untyped, CAST(NULL AS SIGNED) AS signed_null, \
         CAST(NULL AS UNSIGNED) AS unsigned_null, 'text' AS word, 2.5 AS fraction, \
         CAST(3 AS UNSIGNED) + 0 * LENGTH(?) AS bound";

    for server in &series {
        let scratch = Scratch::new();
        let resolved = reaching(&scratch, server, &schema, "verify-identity");
        let target = Target::of(&resolved).expect("the entry names a host");
        let mut session =
            crate::mysql::open(&target, &settings::clock(None)).expect("the session opens");
        let name = server.name();
        let mut compared = 0;
        let mut integers = 0;

        let mut answered = |rows: &[MySqlRow]| {
            for row in rows {
                let mut fields: Vec<&str> =
                    row.columns().iter().map(|column| column.name()).collect();
                fields.push("absent");
                for field in fields {
                    let read = super::row::maybe_integer(row, field);
                    let oracle = asked_twice(row, field);
                    assert_eq!(read.as_ref().ok(), oracle.as_ref().ok(), "{name}: {field}");
                    assert_eq!(read.is_err(), oracle.is_err(), "{name}: {field}");
                    compared += 1;
                    integers += usize::from(matches!(read, Ok(Some(_))));
                }
            }
        };

        for read in Read::ALL {
            answered(&fetched(&mut session, read.sql().to_owned(), &schema, 1));
        }
        answered(&fetched(&mut session, edges.to_owned(), &schema, 1));
        session.close();

        assert!(
            integers > 0 && compared > integers,
            "{name}: {integers} of {compared}"
        );
    }
}
