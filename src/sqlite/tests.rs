//! The tests of the SQLite reader over the fixture of `FR-LITE-038`.
//!
//! They are unit tests because several of them read the statement record of
//! `FR-LITE-018`, or make another connection write the file between the check
//! and the plan (`FR-LITE-042`), through test seams of `FR-ERR-031` compiled
//! into no other build; the rest share the fixture with them. Each test builds
//! the fixture into a scratch directory of its own.

use std::borrow::Cow;
use std::cell::RefCell;
use std::collections::{BTreeMap, BTreeSet};
use std::rc::Rc;

use rusqlite::config::DbConfig;
use rusqlite::{Connection, OpenFlags};

use super::catalogue::statements;
use super::fixture::{self, Artefacts};
use super::record;
use crate::deadline::{Deadlines, Seconds};
use crate::error::{Error, RefusedKind};
use crate::model::column::GeneratedStorage;
use crate::model::column_default::ColumnDefault;
use crate::model::database::{Database, Provider};
use crate::model::foreign_key::ReferentialAction;
use crate::model::index::SortDirection;
use crate::model::server::Standing;
use crate::model::table::{Table, TableType};
use crate::model::trigger::{TriggerEvent, TriggerTiming};
use crate::project::scratch::Scratch;
use crate::project::settings::{self, FileSettings};

/// The entry every read below is made through.
const ENTRY: &str = "ledger";

/// The fixture, built under a scratch directory the caller keeps alive.
fn built() -> (Scratch, Artefacts) {
    let scratch = Scratch::new();
    let root = scratch.directory("fixture");
    let artefacts = fixture::build(&root);

    (scratch, artefacts)
}

/// Settings over `path`, under the default deadlines.
fn over(path: &std::path::Path) -> FileSettings {
    FileSettings::new(ENTRY, path, Deadlines::default())
}

/// The catalogue of the file at `path`, read through the whole connection
/// start.
fn read(path: &std::path::Path) -> Result<super::Catalogue, Error> {
    let clock = settings::clock(None);
    let session = super::open(&over(path), &clock)?;
    let read = super::read(&session);
    session.close();
    read
}

/// The model of the main fixture database.
fn model(artefacts: &Artefacts) -> Database<'static> {
    read(&artefacts.main)
        .expect("the main fixture database reads")
        .model()
        .expect("the main fixture database folds")
}

/// The table `name` of `database`.
fn table<'d>(database: &'d Database<'static>, name: &str) -> &'d Table<'static> {
    database
        .tables
        .iter()
        .find(|table| table.name() == name)
        .unwrap_or_else(|| panic!("the model carries {name}"))
}

/// The five views of item 10 of `FR-LITE-038`, ordered by name.
const VIEWS: [&str; 5] = [
    "v_document_paths",
    "v_port_country",
    "v_quay_schedule",
    "v_ranked_notes",
    "v_vessel_fleet",
];

/// The three views of item 10 whose definition does not compile: one selects
/// from a missing table, one calls a function the carried library does not
/// hold, and one calls a function `trusted_schema` off refuses.
const UNCOMPILABLE: [&str; 3] = ["v_document_paths", "v_quay_schedule", "v_ranked_notes"];

/// The nine keys `FR-CAT-047` gives a view, and no other.
const VIEW_KEYS: [&str; 9] = [
    "algorithm",
    "character_set_client",
    "check_option",
    "collation_connection",
    "definer",
    "definition",
    "is_updatable",
    "name",
    "security_type",
];

/// The column `name` of `table`.
fn column<'t>(table: &'t Table<'static>, name: &str) -> &'t crate::model::column::Column<'static> {
    table
        .columns()
        .iter()
        .find(|column| column.name == name)
        .unwrap_or_else(|| panic!("{} carries {name}", table.name()))
}

#[test]
fn fr_lite_002_the_server_object_describes_the_carried_library() {
    // FR-LITE-002, FR-LITE-003: the version the probe returns, unaltered; its
    // <major>.<minor>; and `supported`, with no window consulted.
    let (_scratch, artefacts) = built();
    let clock = settings::clock(None);
    let session = super::open(&over(&artefacts.main), &clock).expect("the file opens");
    let server = session.server().clone();
    session.close();

    let bundled = rusqlite::ffi::SQLITE_VERSION
        .to_str()
        .expect("the bundled version is ASCII");
    assert_eq!(server.version(), bundled);
    let mut parts = bundled.split('.');
    let series = format!(
        "{}.{}",
        parts.next().expect("a major"),
        parts.next().expect("a minor")
    );
    assert_eq!(server.series(), series);
    assert_eq!(server.standing(), Standing::Supported);
}

#[test]
fn fr_lite_010_every_control_and_limit_reads_back_as_set_on_the_connection() {
    // FR-LITE-010 and FR-LITE-011: read back on the connection the reader
    // opened, after the connection start.
    let (_scratch, artefacts) = built();
    let clock = settings::clock(None);
    let session = super::open(&over(&artefacts.main), &clock).expect("the file opens");
    let connection = session.connection().expect("the connection is open");

    assert!(
        connection
            .is_readonly("main")
            .expect("read-only reads back")
    );
    assert!(
        connection
            .db_config(DbConfig::SQLITE_DBCONFIG_DEFENSIVE)
            .expect("defensive reads back")
    );
    assert!(
        !connection
            .db_config(DbConfig::SQLITE_DBCONFIG_TRUSTED_SCHEMA)
            .expect("trusted_schema reads back")
    );
    // The ninth control, since the sixty-ninth edition: views disabled.
    assert!(
        !connection
            .db_config(DbConfig::SQLITE_DBCONFIG_ENABLE_VIEW)
            .expect("views read back")
    );
    assert!(
        connection
            .prepare("SELECT * FROM main.v_vessel_fleet")
            .is_err(),
        "a view that compiles is refused on the hardened connection"
    );
    for (pragma, expected) in [
        ("query_only", 1),
        ("cell_size_check", 1),
        ("mmap_size", 0),
        ("temp_store", 2),
    ] {
        let read_back: i64 = connection
            .pragma_query_value(None, pragma, |row| row.get(0))
            .expect("the pragma reads back");
        assert_eq!(read_back, expected, "{pragma}");
    }
    for (limit, expected) in [
        (rusqlite::limits::Limit::SQLITE_LIMIT_LENGTH, 1_000_000),
        (rusqlite::limits::Limit::SQLITE_LIMIT_SQL_LENGTH, 1_000_000),
        (rusqlite::limits::Limit::SQLITE_LIMIT_ATTACHED, 0),
    ] {
        assert_eq!(connection.limit(limit).expect("the limit reads"), expected);
    }

    let options: Vec<String> = connection
        .prepare("PRAGMA compile_options")
        .expect("the pragma prepares")
        .query_map([], |row| row.get(0))
        .expect("the pragma runs")
        .collect::<Result<_, _>>()
        .expect("every row is text");
    assert_eq!(super::harden::build(&options), Ok(()));

    session.close();
}

#[test]
fn fr_lite_018_the_statements_issued_are_the_closed_list_in_the_order_of_fr_lite_017() {
    // FR-LITE-018 items 1 and 2, over the main fixture database in both
    // journal modes and over an empty file: every statement the trace reports
    // without the prefix is a statement of FR-LITE-015, in the order of
    // FR-LITE-017 — the connection start, BEGIN DEFERRED, the check, the six
    // statements of the plan and COMMIT, which is how FR-LITE-042 is verified;
    // every one with it is nested and neither checked nor counted.
    let (_scratch, artefacts) = built();

    for path in [&artefacts.main, &artefacts.wal, &artefacts.empty] {
        let (answered, texts) = record::recorded(|| read(path).map(drop));
        answered.expect("the file reads");
        let issued = record::issued(&texts);

        for text in &issued {
            assert!(
                statements::CLOSED_LIST.contains(text),
                "{text} is not a statement of FR-LITE-015"
            );
        }

        // The connection start first, in its order, then the read
        // transaction around the check and the plan, and nothing after it.
        let start = statements::CONNECTION_START.len();
        assert_eq!(&issued[..start], statements::CONNECTION_START);
        assert_eq!(&issued[start..], statements::READ, "{}", path.display());

        for text in texts.iter().filter(|text| text.starts_with(record::NESTED)) {
            assert!(!issued.contains(&text.as_str()));
        }
    }
}

#[test]
fn fr_lite_037_the_plan_issues_as_many_statements_over_the_fixture_as_over_an_empty_file() {
    // FR-LITE-037 and NFR-PERF-001, counted under FR-LITE-018: the probe and
    // seven catalogue statements — the check and the six of the plan — over
    // the fixture and over an empty file alike, without exception; the
    // fixture's views that do not compile add no statement. BEGIN DEFERRED
    // and COMMIT are issued once each and are not catalogue statements, so
    // neither is counted (the note of the seventieth edition).
    let (_scratch, artefacts) = built();
    let counted = |path: &std::path::Path| {
        let (answered, texts) = record::recorded(|| read(path).map(drop));
        answered.expect("the file reads");
        let issued = record::issued(&texts);
        let count = |wanted: &[&str]| issued.iter().filter(|text| wanted.contains(text)).count();
        (
            count(&[statements::PROBE]),
            count(&statements::CATALOGUE),
            count(&[statements::BEGIN]),
            count(&[statements::COMMIT]),
            issued.len(),
        )
    };

    let over_fixture = counted(&artefacts.main);
    let over_empty = counted(&artefacts.empty);

    assert_eq!(over_fixture, over_empty);
    assert_eq!(over_fixture.0, 1);
    assert_eq!(over_fixture.1, 7);
    assert_eq!((over_fixture.2, over_fixture.3), (1, 1));
}

#[test]
fn nfr_perf_008_the_query_lines_equal_the_catalogue_statements_the_record_shows() {
    // NFR-PERF-008 for a `sqlite` entry: one invocation at -v, driven in
    // process on a thread of its own, and its lines of FR-GLOB-017 against
    // the catalogue statements the record of FR-LITE-018 reports.
    let _held = crate::diagnostics::verbosity::LEVEL_LOCK
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    let (scratch, artefacts) = built();
    let tpl = scratch.directory("project/.tpl");
    let cfg = scratch.file(
        "project/.tpl/.cfg",
        &format!(
            "[database.{ENTRY}]\nprovider = \"sqlite\"\npath = \"{}\"\n",
            artefacts.main.display()
        ),
    );
    scratch.chmod(&cfg, 0o600);
    let tpl = tpl.to_string_lossy().into_owned();
    let restore = crate::diagnostics::verbosity::level();

    let (lines, texts, answered) = std::thread::spawn(move || {
        let ((answered, lines), texts) = record::recorded(|| {
            crate::diagnostics::emit::capture::captured(|| {
                crate::cli::run_captured(&[
                    "tpl",
                    "--tpl-dir",
                    &tpl,
                    "-d",
                    ENTRY,
                    "-v",
                    "schema",
                    "dump",
                    "--direct",
                    "--no-cache",
                ])
                .0
            })
        });
        (lines, texts, answered)
    })
    .join()
    .expect("the invocation's thread ends");
    crate::diagnostics::verbosity::set_level(restore);

    answered.expect("the dump succeeds");
    let counted = lines
        .iter()
        .filter(|line| crate::diagnostics::emit::capture::is_catalogue_query(line))
        .count();
    let catalogue = record::issued(&texts)
        .into_iter()
        .filter(|text| statements::CATALOGUE.contains(text))
        .count();

    // The check and the six statements of the plan carry the line; neither
    // BEGIN DEFERRED nor COMMIT does.
    assert_eq!(counted, catalogue);
    assert_eq!(catalogue, statements::CATALOGUE.len());
}

#[test]
fn fr_lite_019_the_read_covers_main_and_its_tables_and_views_and_no_routine() {
    // FR-LITE-019 … FR-LITE-022, as amended in the sixty-ninth edition: the
    // database is `main`; tables carry `BASE TABLE`; internal and virtual
    // tables are excluded; the five shadow tables of the fts5 table of item 12
    // are covered as `BASE TABLE`, with their columns and keys; and there is
    // no routine.
    let (_scratch, artefacts) = built();
    let database = model(&artefacts);

    assert_eq!(database.name, "main");
    assert_eq!(database.provider, Provider::Sqlite);
    assert_eq!(database.charset, "UTF-8");
    assert_eq!(database.collation, None);
    assert!(database.routines.is_empty());

    let names: Vec<&str> = database.tables.iter().map(Table::name).collect();
    for covered in [
        "berth",
        "cargo_note_config",
        "cargo_note_content",
        "cargo_note_data",
        "cargo_note_docsize",
        "cargo_note_idx",
        "carrier",
        "consignment",
        "container_size",
        "country",
        "customs entry",
        "hazard_class",
        "inspection_zone",
        "manifest_line",
        "port",
        "tariff_band",
        "vessel",
        "voyage",
        "voyage_audit",
        "voyage_leg",
    ] {
        assert!(names.contains(&covered), "{covered} is covered");
    }
    for excluded in [
        "sqlite_schema",
        "sqlite_sequence",
        "sqlite_stat1",
        "sqlite_stat4",
        "cargo_note",
    ] {
        assert!(!names.contains(&excluded), "{excluded} is excluded");
    }
    assert_eq!(names.len(), 20, "{names:?}");
    // A shadow table carries its columns and its key, like any other table.
    let config = table(&database, "cargo_note_config");
    assert_eq!(config.columns().len(), 2);
    assert!(config.primary_key().is_some());
    assert!(
        database
            .tables
            .iter()
            .all(|table| table.table_type() == TableType::Base)
    );
    let views: Vec<&str> = database.views.iter().map(|view| &*view.name).collect();
    assert_eq!(views, VIEWS);
}

#[test]
fn fr_lite_021_a_database_of_internal_tables_alone_covers_nothing() {
    let (_scratch, artefacts) = built();
    let database = read(&artefacts.internal)
        .expect("the database reads")
        .model()
        .expect("it folds");

    assert!(database.tables.is_empty());
    assert!(database.views.is_empty());
}

#[test]
fn fr_lite_025_a_column_carries_its_position_nullability_and_generation() {
    let (_scratch, artefacts) = built();
    let database = model(&artefacts);
    let leg = table(&database, "voyage_leg");

    let positions: Vec<u64> = leg.columns().iter().map(|column| column.position).collect();
    assert_eq!(positions, (1..=8).collect::<Vec<u64>>());
    assert!(!column(leg, "vessel_imo").nullable);
    assert!(column(leg, "distance_nm").nullable);

    let eta = column(leg, "eta_hours")
        .generated
        .as_ref()
        .expect("eta_hours is generated");
    assert_eq!(eta.storage, GeneratedStorage::Virtual);
    assert_eq!(eta.expression, None);
    let km = column(leg, "distance_km")
        .generated
        .as_ref()
        .expect("distance_km is generated");
    assert_eq!(km.storage, GeneratedStorage::Stored);
    assert!(column(leg, "distance_nm").generated.is_none());
    assert!(leg.columns().iter().all(|column| !column.invisible));
    assert!(leg.columns().iter().all(|column| column.comment.is_empty()));
}

#[test]
fn fr_lite_026_the_type_is_the_declared_text_and_its_affinity() {
    let (_scratch, artefacts) = built();
    let database = model(&artefacts);
    let carrier = table(&database, "carrier");

    for (name, declared, affinity) in [
        ("carrier_id", "INTEGER", "integer"),
        ("scac_code", "VARCHAR(4)", "text"),
        ("founded", "DATETIME", "numeric"),
        ("rating", "NUMERIC(5,2)", "numeric"),
        ("is_active", "BOOLEAN", "numeric"),
        ("notes", "", "blob"),
        ("logo", "BLOB", "blob"),
    ] {
        let column = column(carrier, name);
        assert_eq!(column.column_type.raw(), declared, "{name}");
        assert_eq!(column.column_type.data_type(), Some(affinity), "{name}");
        assert_eq!(column.column_type.length(), None, "{name}");
        assert_eq!(column.column_type.precision(), None, "{name}");
        assert_eq!(column.column_type.unsigned(), None, "{name}");
        assert_eq!(column.column_type.collation(), None, "{name}");
    }
    let port = table(&database, "port");
    assert_eq!(
        column(port, "latitude").column_type.data_type(),
        Some("real")
    );
    assert_eq!(
        column(port, "longitude").column_type.data_type(),
        Some("real")
    );
    // A STRICT column declared ANY takes the fifth rule's `numeric`, as the
    // sixty-ninth edition accepts: no statement reports that the table is
    // STRICT. `column_type` keeps the declared `ANY`.
    let line = table(&database, "manifest_line");
    assert_eq!(column(line, "payload").column_type.raw(), "ANY");
    assert_eq!(
        column(line, "payload").column_type.data_type(),
        Some("numeric")
    );
}

#[test]
fn fr_lite_027_every_default_row_is_classified_from_the_fixture() {
    let (_scratch, artefacts) = built();
    let database = model(&artefacts);
    let consignment = table(&database, "consignment");
    let carrier = table(&database, "carrier");
    let literal = |value: &str| Some(ColumnDefault::Literal(Cow::Owned(value.to_owned())));
    let expression = |value: &str| Some(ColumnDefault::Expression(Cow::Owned(value.to_owned())));

    // SQL NULL, not nullable and nullable.
    assert_eq!(column(consignment, "seal_code").default, None);
    assert_eq!(
        column(consignment, "free_text").default,
        Some(ColumnDefault::Null)
    );
    // `NULL`, on a nullable and on a NOT NULL column.
    assert_eq!(
        column(consignment, "cleared").default,
        Some(ColumnDefault::Null)
    );
    assert_eq!(
        column(consignment, "required_note").default,
        Some(ColumnDefault::Null)
    );
    // Strings, with a doubled apostrophe, and empty.
    assert_eq!(column(carrier, "legal_name").default, literal("unnamed"));
    assert_eq!(
        column(consignment, "shipper").default,
        literal("Lloyd's \"A1\"")
    );
    assert_eq!(column(consignment, "empty_note").default, literal(""));
    // Numbers, one parenthesised.
    assert_eq!(column(carrier, "rating").default, literal("4.5"));
    assert_eq!(column(consignment, "priority").default, literal("+5"));
    assert_eq!(column(consignment, "ratio").default, literal("1e3"));
    assert_eq!(column(consignment, "hex_flags").default, literal("0x1F"));
    assert_eq!(column(consignment, "negative").default, literal("-1"));
    // BLOBs, and the two truth values.
    assert_eq!(column(carrier, "logo").default, literal("x'00ff'"));
    assert_eq!(column(consignment, "signature").default, literal("X'AB'"));
    assert_eq!(column(carrier, "is_active").default, literal("TRUE"));
    assert_eq!(column(consignment, "insured").default, literal("false"));
    // Everything else is an expression.
    assert_eq!(
        column(consignment, "booked_at").default,
        expression("datetime('now')")
    );
    assert_eq!(column(consignment, "summed").default, expression("1+2"));
    assert_eq!(
        column(consignment, "lowered").default,
        expression("lower( 'X' )")
    );
    assert_eq!(
        column(consignment, "received_at").default,
        expression("current_time")
    );
}

#[test]
fn fr_lite_028_only_the_rowid_alias_is_auto_incremental() {
    let (_scratch, artefacts) = built();
    let database = model(&artefacts);

    // With and without AUTOINCREMENT.
    assert!(column(table(&database, "carrier"), "carrier_id").auto_increment);
    assert!(column(table(&database, "vessel"), "vessel_imo").auto_increment);
    assert!(column(table(&database, "consignment"), "consignment_id").auto_increment);
    // INT, BIGINT, INTEGER DESC, and INTEGER in a WITHOUT ROWID table.
    for (owner, name) in [
        ("berth", "berth_no"),
        ("container_size", "size_code"),
        ("tariff_band", "band_id"),
        ("hazard_class", "class_id"),
    ] {
        assert!(
            !column(table(&database, owner), name).auto_increment,
            "{owner}.{name}"
        );
    }
}

#[test]
fn fr_lite_029_the_primary_key_is_read_from_pk_and_named_after_its_index() {
    let (_scratch, artefacts) = built();
    let database = model(&artefacts);

    // A rowid alias: unnamed, ascending, unique, and no index of its own.
    let vessel = table(&database, "vessel")
        .primary_key()
        .expect("vessel has a key");
    assert_eq!(vessel.name, None);
    assert!(vessel.unique);
    assert_eq!(vessel.index_type, None);
    assert!(vessel.comment.is_empty());
    assert!(!vessel.ignored);
    assert_eq!(vessel.columns.len(), 1);
    assert_eq!(vessel.columns[0].name, "vessel_imo");
    assert_eq!(vessel.columns[0].direction, Some(SortDirection::Ascending));
    assert_eq!(vessel.columns[0].prefix_length, None);

    // A composite key, in the order of `pk`, named after its index.
    let voyage = table(&database, "voyage")
        .primary_key()
        .expect("voyage has a key");
    assert_eq!(voyage.name.as_deref(), Some("sqlite_autoindex_voyage_1"));
    let members: Vec<&str> = voyage.columns.iter().map(|member| &*member.name).collect();
    assert_eq!(members, ["vessel_imo", "voyage_no"]);

    // INTEGER PRIMARY KEY DESC keeps its direction.
    let band = table(&database, "tariff_band")
        .primary_key()
        .expect("tariff_band has a key");
    assert_eq!(band.columns[0].direction, Some(SortDirection::Descending));

    // Every table of the fixture declares a key; the rowid alias of
    // voyage_audit is one.
    assert!(table(&database, "voyage_audit").primary_key().is_some());
}

#[test]
fn fr_lite_030_the_indexes_are_those_the_library_reports_less_expression_indexes() {
    let (_scratch, artefacts) = built();
    let database = model(&artefacts);

    let port = table(&database, "port");
    let names: Vec<&str> = port.indexes().iter().map(|index| &*index.name).collect();
    assert_eq!(names, ["sqlite_autoindex_port_1"]);

    let voyage = table(&database, "voyage");
    let departs = voyage
        .indexes()
        .iter()
        .find(|index| index.name == "ix_voyage_departs")
        .expect("the descending index is carried");
    assert!(!departs.unique);
    assert_eq!(
        departs.columns[0].direction,
        Some(SortDirection::Descending)
    );
    assert_eq!(departs.columns[1].direction, Some(SortDirection::Ascending));
    assert_eq!(departs.index_type, None);

    let vessel = table(&database, "vessel");
    let unique = vessel
        .indexes()
        .iter()
        .find(|index| index.name == "ux_vessel_name")
        .expect("the collated unique index is carried");
    assert!(unique.unique);

    let consignment = table(&database, "consignment");
    assert!(
        consignment
            .indexes()
            .iter()
            .any(|index| index.name == "ix_consignment_uninsured"),
        "a partial index is carried"
    );
    let leg = table(&database, "voyage_leg");
    assert!(
        leg.indexes().iter().any(|index| index.name == "ix_leg_eta"),
        "an index on a generated column is carried"
    );
    let carrier = table(&database, "carrier");
    assert!(
        carrier
            .indexes()
            .iter()
            .any(|index| index.name == "sqlite_autoindex_carrier_1" && index.unique),
        "a column-level UNIQUE is carried"
    );
}

#[test]
fn fr_lite_031_the_foreign_keys_carry_their_pairs_rules_and_no_name() {
    let (_scratch, artefacts) = built();
    let database = model(&artefacts);

    // A key naming no parent column takes the parent's primary key.
    let port = table(&database, "port");
    let to_country = &port.foreign_keys()[0];
    assert_eq!(to_country.name, None);
    assert_eq!(to_country.referenced_table.as_deref(), Some("country"));
    assert_eq!(to_country.columns[0].column, "country_code");
    assert_eq!(to_country.columns[0].referenced_column, "country_code");
    assert_eq!(to_country.on_update, ReferentialAction::Cascade);
    assert_eq!(to_country.on_delete, ReferentialAction::Restrict);
    assert_eq!(to_country.referenced_key, None);

    // SET DEFAULT is carried as SQLite reports it.
    let voyage = table(&database, "voyage");
    assert!(voyage.foreign_keys().iter().any(|key| {
        key.on_update.name() == "SET DEFAULT" && key.on_delete.name() == "SET DEFAULT"
    }));
    // The rowid-alias parent, through a key naming no parent column.
    assert!(voyage.foreign_keys().iter().any(|key| {
        key.referenced_table.as_deref() == Some("vessel")
            && key.columns[0].referenced_column == "vessel_imo"
    }));

    // A composite key, paired by `seq`.
    let leg = table(&database, "voyage_leg");
    let composite = leg
        .foreign_keys()
        .iter()
        .find(|key| key.referenced_table.as_deref() == Some("voyage"))
        .expect("the composite key is carried");
    let pairs: Vec<(&str, &str)> = composite
        .columns
        .iter()
        .map(|pair| (&*pair.column, &*pair.referenced_column))
        .collect();
    assert_eq!(
        pairs,
        [("vessel_imo", "vessel_imo"), ("voyage_no", "voyage_no")]
    );
    // Ordered by content: the referencing columns, then the referenced table.
    let referencing: Vec<&str> = leg
        .foreign_keys()
        .iter()
        .map(|key| &*key.columns[0].column)
        .collect();
    assert_eq!(referencing, ["from_port", "to_port", "vessel_imo"]);

    // MATCH FULL reads NONE and is carried as read.
    let vessel = table(&database, "vessel");
    assert!(
        vessel
            .foreign_keys()
            .iter()
            .all(|key| key.match_option == "NONE")
    );

    // A key naming a table the database does not hold is excluded; the
    // self-reference is carried in both directions.
    let consignment = table(&database, "consignment");
    assert!(
        consignment
            .foreign_keys()
            .iter()
            .all(|key| key.referenced_table.as_deref() != Some("customs_office"))
    );
    assert!(
        consignment
            .referenced_by()
            .iter()
            .any(|incoming| incoming.table == "consignment")
    );
    // The incoming direction of the composite key.
    assert!(
        voyage
            .referenced_by()
            .iter()
            .any(|incoming| incoming.table == "voyage_leg")
    );
}

#[test]
fn fr_lite_032_the_triggers_on_tables_carry_their_timing_event_and_whole_statement() {
    let (_scratch, artefacts) = built();
    let database = model(&artefacts);
    let voyage = table(&database, "voyage");
    let found = |name: &str| {
        voyage
            .triggers()
            .iter()
            .find(|trigger| trigger.name == name)
            .unwrap_or_else(|| panic!("voyage carries {name}"))
    };

    for (name, timing, event) in [
        (
            "trg_voyage_before_insert",
            TriggerTiming::Before,
            TriggerEvent::Insert,
        ),
        (
            "trg_voyage_after_insert",
            TriggerTiming::After,
            TriggerEvent::Insert,
        ),
        (
            "trg_voyage_before_update",
            TriggerTiming::Before,
            TriggerEvent::Update,
        ),
        (
            "trg_voyage_after_update",
            TriggerTiming::After,
            TriggerEvent::Update,
        ),
        (
            "trg_voyage_before_delete",
            TriggerTiming::Before,
            TriggerEvent::Delete,
        ),
        (
            "trg voyage \"after\" delete",
            TriggerTiming::After,
            TriggerEvent::Delete,
        ),
    ] {
        let trigger = found(name);
        assert_eq!(trigger.timing, timing, "{name}");
        assert_eq!(trigger.event, event, "{name}");
        assert_eq!(trigger.orientation, "ROW");
        assert_eq!(trigger.old_row_alias, "OLD");
        assert_eq!(trigger.new_row_alias, "NEW");
        assert_eq!(trigger.action_order, None);
        assert_eq!(trigger.sql_mode, None);
        assert_eq!(trigger.definer, None);
        assert!(
            trigger
                .statement
                .as_deref()
                .is_some_and(|statement| statement.starts_with("CREATE TRIGGER"))
        );
    }

    // No timing written is BEFORE.
    let port = table(&database, "port");
    assert_eq!(port.triggers()[0].timing, TriggerTiming::Before);
    assert_eq!(port.triggers()[0].event, TriggerEvent::Insert);

    // The INSTEAD OF trigger on a view is not carried.
    assert!(
        database
            .tables
            .iter()
            .flat_map(|table| table.triggers())
            .all(|trigger| trigger.name != "trg_fleet_rename")
    );
}

#[test]
fn fr_lite_033_every_view_carries_exactly_the_nine_properties_and_no_columns() {
    // FR-LITE-033 as amended in the sixty-eighth edition: every view of the
    // main database, the three of item 10 that do not compile among them,
    // carries the nine properties of FR-CAT-047 and no other key — its name,
    // its whole stored statement unchanged, and null for the seven server
    // fields. No view carries its columns.
    let (_scratch, artefacts) = built();
    let database = model(&artefacts);

    // The stored text, read apart from the reader.
    let stored: BTreeMap<String, String> = {
        let connection =
            Connection::open_with_flags(&artefacts.main, OpenFlags::SQLITE_OPEN_READ_ONLY)
                .expect("the fixture opens");
        let mut statement = connection
            .prepare("SELECT name, sql FROM main.sqlite_schema WHERE type = 'view'")
            .expect("the schema table is read");
        statement
            .query_map([], |row| Ok((row.get(0)?, row.get(1)?)))
            .expect("the schema table is read")
            .collect::<Result<_, _>>()
            .expect("every view has a name and a statement")
    };
    assert_eq!(stored.keys().map(String::as_str).collect::<Vec<_>>(), VIEWS);
    let nine: BTreeSet<&str> = VIEW_KEYS.into_iter().collect();

    for view in &database.views {
        let name = &*view.name;
        assert_eq!(
            Some(&*view.definition),
            stored.get(name).map(String::as_str)
        );
        assert!(view.definition.starts_with("CREATE VIEW "), "{name}");
        assert_eq!(view.check_option, None, "{name}");
        assert_eq!(view.is_updatable, None, "{name}");
        assert_eq!(view.definer, None, "{name}");
        assert_eq!(view.security_type, None, "{name}");
        assert_eq!(view.character_set_client, None, "{name}");
        assert_eq!(view.collation_connection, None, "{name}");
        assert_eq!(view.algorithm, None, "{name}");
        assert_eq!(view.restricted, None, "{name}");

        let serialised = serde_json::to_value(view).expect("a view serialises");
        let keys: BTreeSet<&str> = serialised
            .as_object()
            .expect("a view serialises as an object")
            .keys()
            .map(String::as_str)
            .collect();
        assert_eq!(keys, nine, "{name}");
    }
}

#[test]
fn fr_lite_034_no_check_constraint_is_read_and_the_table_holding_one_is_read() {
    // FR-LITE-034, and item 9 of FR-LITE-038: the CHECK calling a function
    // not marked innocuous does not stop the read.
    let (_scratch, artefacts) = built();
    let database = model(&artefacts);

    assert!(
        database
            .tables
            .iter()
            .all(|table| table.check_constraints().is_empty())
    );
    assert_eq!(table(&database, "inspection_zone").columns().len(), 3);
    assert!(
        database
            .tables
            .iter()
            .all(|table| table.engine().is_none() && table.collation().is_none())
    );
}

#[test]
fn fr_lite_036_a_read_of_the_main_database_is_complete_and_marks_nothing() {
    // FR-LITE-036 as amended in the sixty-eighth edition: a `sqlite` read is
    // complete or fails, and no object it presents carries a `restricted`
    // marking. The three views of item 10 that do not compile are presented,
    // whole and unmarked (FR-LITE-038).
    let (_scratch, artefacts) = built();
    let database = model(&artefacts);

    assert!(
        database
            .tables
            .iter()
            .all(|table| table.restricted().is_none())
    );
    assert!(database.views.iter().all(|view| view.restricted.is_none()));
    for name in UNCOMPILABLE {
        assert!(
            database.views.iter().any(|view| view.name == name),
            "{name} is presented"
        );
    }

    // The document a dump writes carries the marking nowhere.
    let dumped = crate::model::document::dump(&database, crate::output::Source::Server)
        .expect("the document builds");
    let bytes = serde_json::to_string(&dumped).expect("it serialises");
    assert!(!bytes.contains("\"restricted\""), "{bytes}");
}

#[test]
fn fr_lite_018_no_statement_names_a_view_and_the_read_succeeds_over_the_views_that_do_not_compile()
{
    // FR-LITE-018 (sixty-eighth edition, as amended in the sixty-ninth) and
    // FR-LITE-038: the read of the main database succeeds over the three views
    // of item 10 that do not compile, and the record holds the read
    // transaction around the check and the six statements of the plan and
    // nothing else, none of which names a view. The outcome of the read is the
    // instrument: the control below shows that a statement naming any of the
    // three on the reader's own hardened connection fails, so a read that
    // named one would have failed with it.
    let (_scratch, artefacts) = built();
    let clock = settings::clock(None);
    let session = super::open(&over(&artefacts.main), &clock).expect("the file opens");

    let (answered, texts) = record::recorded(|| super::read(&session).map(drop));
    answered.expect("the read succeeds over the views that do not compile");

    let issued = record::issued(&texts);
    assert_eq!(issued, statements::READ);
    for text in &issued {
        for view in VIEWS {
            assert!(!text.contains(view), "{text} names the view {view}");
        }
    }

    // The control: each of the three, compiled on the same connection, fails.
    let connection = session.connection().expect("the connection is open");
    for view in UNCOMPILABLE {
        let compiled = connection.prepare(&format!("SELECT * FROM main.\"{view}\""));
        assert!(compiled.is_err(), "{view} compiles");
    }
    session.close();
}

#[test]
fn fr_lite_031_the_document_orders_unnamed_keys_by_content_and_round_trips() {
    // FR-LITE-031 and NFR-DET-002 through the document, and the read-back of
    // FR-CTX-043 and OD-35: a `sqlite` document's key is read from
    // `primary_key`.
    let (_scratch, artefacts) = built();
    let database = model(&artefacts);
    let dumped = crate::model::document::dump(&database, crate::output::Source::Server)
        .expect("the document builds");
    let bytes = serde_json::to_string(&dumped).expect("it serialises");

    assert!(bytes.contains("\"provider\":\"sqlite\""));
    let read_back =
        crate::model::document::read(&bytes, std::path::Path::new("dump.json")).expect("65 free");
    assert_eq!(read_back.provider, Provider::Sqlite);
    let vessel = read_back
        .tables
        .iter()
        .find(|table| table.name() == "vessel")
        .expect("vessel round-trips");
    assert_eq!(
        vessel.primary_key().map(|key| key.name.clone()),
        Some(None),
        "the rowid alias is read back from primary_key"
    );
}

#[test]
fn fr_env_041_a_rowid_alias_a_desc_key_and_a_without_rowid_key_are_primary_and_unique() {
    // FR-ENV-041 as amended: the tests read the table's key, so a rowid alias
    // that has no index answers true to both.
    let (_scratch, artefacts) = built();
    let database = model(&artefacts);
    let document = crate::model::document::context(&database).expect("the document builds");
    let context = minijinja::context! { database => minijinja::Value::from_serialize(&document) };
    let mut engine = minijinja::Environment::new();
    engine.set_undefined_behavior(minijinja::UndefinedBehavior::Strict);
    crate::render::register_surface(&mut engine);

    for (owner, name, expected) in [
        ("vessel", "vessel_imo", (true, true)),
        ("tariff_band", "band_id", (true, true)),
        ("hazard_class", "class_id", (true, true)),
        ("voyage", "voyage_no", (true, true)),
        ("vessel", "gross_tonnage", (false, false)),
        ("carrier", "scac_code", (false, true)),
    ] {
        let source = format!(
            "{{% for t in database.tables %}}{{% if t.name == '{owner}' %}}{{% for c in t.columns \
             %}}{{% if c.name == '{name}' %}}{{{{ c is primary_key }}}}/{{{{ c is unique \
             }}}}{{% endif %}}{{% endfor %}}{{% endif %}}{{% endfor %}}"
        );
        let answered = engine
            .render_str(&source, &context)
            .expect("the template renders");
        let expected = format!(
            "{}/{}",
            if expected.0 { "true" } else { "false" },
            if expected.1 { "true" } else { "false" }
        );
        assert_eq!(answered.to_lowercase(), expected, "{owner}.{name}");
    }
}

#[test]
fn fr_lite_007_a_file_that_cannot_be_read_is_refused_by_its_result_code() {
    let (_scratch, artefacts) = built();

    for (path, code) in [
        (&artefacts.text, 78),
        (&artefacts.truncated, 78),
        (&artefacts.oversized, 78),
        (&artefacts.directory, 78),
        (&artefacts.dangling, 74),
    ] {
        let refused = read(path).expect_err("the file is refused");
        assert_eq!(refused.exit_code(), code, "{}: {refused:?}", path.display());
    }

    let oversized = read(&artefacts.oversized).expect_err("refused");
    let Error::SqliteFileRefused { report, .. } = &oversized else {
        panic!("{oversized:?}");
    };
    assert_eq!(report.name(), "SQLITE_TOOBIG");

    // An empty file is a database with no object.
    let empty = read(&artefacts.empty)
        .expect("an empty file reads")
        .model()
        .expect("it folds");
    assert!(empty.tables.is_empty() && empty.views.is_empty());
}

#[test]
fn fr_lite_009_a_bound_already_spent_starts_no_statement_and_is_69() {
    let (_scratch, artefacts) = built();
    let spent = crate::deadline::Clock::spent();
    let refused = super::open(&over(&artefacts.main), &spent).expect_err("69");

    assert_eq!(refused.exit_code(), 69);
    assert!(matches!(
        refused,
        Error::SqliteUnavailable {
            ended: crate::error::SqliteEnd::Deadline { .. },
            ..
        }
    ));
    assert!(super::quiescent());
}

#[test]
fn fr_lite_009_a_lock_held_past_the_deadline_is_69_naming_the_lock() {
    let (_scratch, artefacts) = built();
    let holder = Connection::open(&artefacts.main).expect("the holder opens");
    holder
        .execute_batch("BEGIN EXCLUSIVE")
        .expect("the holder locks the file");

    let deadlines = Deadlines::resolve(
        None,
        Some(Seconds::new(std::num::NonZeroU64::MIN)),
        None,
        None,
    );
    let settings = FileSettings::new(ENTRY, &artefacts.main, deadlines);
    let refused =
        super::open(&settings, &settings::clock(None)).expect_err("the lock outlasts the bound");

    assert_eq!(refused.exit_code(), 69);
    let Error::SqliteUnavailable {
        ended: crate::error::SqliteEnd::Locked { report, waited },
        ..
    } = &refused
    else {
        panic!("{refused:?}");
    };
    assert_eq!(report.code & 0xff, rusqlite::ffi::SQLITE_BUSY);
    assert!(waited.is_some(), "the deadline expired while it waited");
    let rendered = crate::diagnostics::rendered(&refused);
    assert!(rendered.contains("locked"), "{rendered}");
    assert!(super::quiescent());

    holder
        .execute_batch("ROLLBACK")
        .expect("the holder releases");
}

#[test]
fn fr_rnd_040_nothing_of_a_sqlite_read_is_alive_once_it_is_closed() {
    let (_scratch, artefacts) = built();
    assert!(super::quiescent());

    let clock = settings::clock(None);
    let session = super::open(&over(&artefacts.main), &clock).expect("the file opens");
    assert!(!super::quiescent());
    assert!(!crate::provider::quiescent());
    session.close();

    assert!(super::quiescent());
    assert!(crate::provider::quiescent());
}

// ------------------------------------------- FR-LITE-018, row 32 of the register ---

#[test]
fn fr_lite_018_over_the_fts5_database_no_statement_compiles_a_view_or_connects_a_virtual_table() {
    // FR-LITE-018 (sixty-ninth edition) and FR-LITE-038: over the database
    // holding an fts5 table and a view over it that compiles, the record of
    // the read holds no nested statement other than one whose text begins
    // `-- PRAGMA `. Connecting the table, whether a statement names it or
    // compiles the view, makes the module run a statement of its own.
    let (_scratch, artefacts) = built();

    let (answered, texts) = record::recorded(|| read(&artefacts.fts5).map(drop));
    answered.expect("the fts5 database reads");
    let issued = record::issued(&texts);
    assert_eq!(
        &issued[statements::CONNECTION_START.len()..],
        statements::READ
    );

    let nested: Vec<&String> = texts
        .iter()
        .filter(|text| text.starts_with(record::NESTED))
        .collect();
    assert!(
        !nested.is_empty(),
        "the record holds the pragma functions' own statements: {texts:?}"
    );
    for text in &nested {
        assert!(
            text.starts_with("-- PRAGMA "),
            "a module ran a statement of its own: {text}"
        );
    }

    // The control: the same instrument sees the module run when the view is
    // compiled and run on a plain connection, so the silence above means
    // something.
    let mut plain = Connection::open_with_flags(&artefacts.fts5, OpenFlags::SQLITE_OPEN_READ_ONLY)
        .expect("the fixture opens");
    record::install(&mut plain);
    let (counted, control) = record::recorded(|| {
        plain.query_row("SELECT count(*) FROM main.v_cargo_notes", [], |row| {
            row.get::<_, i64>(0)
        })
    });
    assert_eq!(counted.expect("the view runs on a plain connection"), 2);
    assert!(
        control
            .iter()
            .any(|text| text.starts_with(record::NESTED) && !text.starts_with("-- PRAGMA ")),
        "the record does not see the module run: {control:?}"
    );
}

// --------------------------------------- FR-LITE-041, row 33 of the register ---

#[test]
fn fr_lite_041_each_database_the_check_refuses_is_78_naming_its_row_and_no_statement_follows() {
    // FR-LITE-038 and FR-LITE-041: a read of each of the six databases named
    // like a pragma function or written under writable_schema ends with 78,
    // the `cause` naming the path, the check, the row's type and name, and its
    // kind; no statement follows the check, COMMIT included (FR-LITE-042).
    let (_scratch, artefacts) = built();
    let storage =
        |column: &'static str, class: &'static str| RefusedKind::StorageClass { column, class };

    for (path, row_type, name, kind, said) in [
        (
            &artefacts.pragma_view,
            "view",
            "pragma_table_list",
            RefusedKind::PragmaName,
            "in place of the pragma table-valued function",
        ),
        (
            &artefacts.pragma_table,
            "table",
            "PRAGMA_TABLE_XINFO",
            RefusedKind::PragmaName,
            "in place of the pragma table-valued function",
        ),
        (
            &artefacts.blob_view_sql,
            "view",
            "v_port_names",
            storage("sql", "BLOB"),
            "its sql holds a value of the storage class BLOB",
        ),
        (
            &artefacts.blob_trigger_type,
            "trigger",
            "trg_voyage_after_insert",
            storage("type", "BLOB"),
            "its type holds a value of the storage class BLOB",
        ),
        (
            &artefacts.rootpage_zero,
            "table",
            "berth",
            RefusedKind::RootPage {
                rootpage: Some(0),
                creates_virtual: false,
            },
            "its root page is 0 and its text does not create a virtual table",
        ),
        (
            &artefacts.rootpage_virtual,
            "table",
            "cargo_note",
            RefusedKind::RootPage {
                rootpage: Some(2),
                creates_virtual: true,
            },
            "its root page is 2 and its text creates a virtual table",
        ),
    ] {
        let spelled = path.display().to_string();
        let (answered, texts) = record::recorded(|| read(path).map(drop));
        let refused = answered.expect_err("the check refuses the file");

        assert_eq!(refused.exit_code(), 78, "{spelled}: {refused:?}");
        let Error::SqliteSchemaRefused { row, .. } = &refused else {
            panic!("{spelled}: {refused:?}");
        };
        assert_eq!(row.row_type.as_str(), row_type, "{spelled}");
        assert_eq!(row.name.as_str(), name, "{spelled}");
        assert_eq!(row.kind, kind, "{spelled}");

        let rendered = crate::diagnostics::rendered(&refused);
        let cause = rendered
            .lines()
            .find(|line| line.starts_with("cause:"))
            .unwrap_or_else(|| panic!("{rendered}"));
        assert!(cause.contains(&spelled), "{cause}");
        assert!(
            cause.contains("check of the schema table main.sqlite_schema"),
            "{cause}"
        );
        assert!(
            cause.contains(&format!("'{row_type}' named '{name}'")),
            "{cause}"
        );
        assert!(cause.contains(said), "{cause}");
        assert!(
            rendered.contains("tpl cfg database update ledger --path <path>"),
            "{rendered}"
        );

        let issued = record::issued(&texts);
        assert_eq!(
            issued.last(),
            Some(&statements::CHECK),
            "{spelled}: {issued:?}"
        );
        assert!(!issued.contains(&statements::COMMIT), "{spelled}");
        assert!(super::quiescent(), "{spelled}");
    }
}

#[test]
fn fr_lite_041_a_view_named_like_a_function_called_without_an_argument_is_refused() {
    // The security review's proof of concept: a view named pragma_encoding
    // replaced the encoding at exit 0 with views enabled. And the accepted
    // cost: a table named pragma_settings, which replaces no function the read
    // calls, is refused all the same. An index and a trigger so named replace
    // nothing, and pass.
    let scratch = Scratch::new();
    let root = scratch.directory("shadowing");

    for (name, sql, refused) in [
        (
            "encoding-view",
            "CREATE TABLE port (port_code TEXT PRIMARY KEY); \
             CREATE VIEW pragma_encoding AS SELECT 'forged' AS encoding;",
            Some(("view", "pragma_encoding")),
        ),
        (
            "settings-table",
            "CREATE TABLE pragma_settings (key TEXT PRIMARY KEY, value TEXT);",
            Some(("table", "pragma_settings")),
        ),
        (
            "index-and-trigger",
            "CREATE TABLE port (port_code TEXT PRIMARY KEY, name TEXT); \
             CREATE INDEX pragma_table_list ON port (name); \
             CREATE TRIGGER pragma_index_list AFTER INSERT ON port BEGIN SELECT 1; END;",
            None,
        ),
    ] {
        let path = root.join(format!("{name}.sqlite"));
        let builder = Connection::open(&path).expect("the database opens");
        builder.execute_batch(sql).expect("the database builds");
        builder.close().expect("the database closes");

        match refused {
            Some((row_type, row_name)) => {
                let error = read(&path).expect_err("refused");
                let Error::SqliteSchemaRefused { row, .. } = &error else {
                    panic!("{name}: {error:?}");
                };
                assert_eq!(row.kind, RefusedKind::PragmaName, "{name}");
                assert_eq!(row.row_type.as_str(), row_type);
                assert_eq!(row.name.as_str(), row_name);
            }
            None => {
                let database = read(&path)
                    .expect("an index and a trigger replace nothing")
                    .model()
                    .expect("it folds");
                assert_eq!(database.charset, "UTF-8");
                assert_eq!(database.tables.len(), 1);
            }
        }
    }
}

// ----------------------------------------------------------- FR-LITE-042 ---

#[test]
fn fr_lite_042_tpl_cfg_database_test_issues_the_check_alone_and_no_transaction() {
    // FR-LITE-042 and FR-CFG-056: the test's only catalogue statement is the
    // check, so it issues neither BEGIN DEFERRED nor COMMIT; over a database
    // the check refuses it ends with the 78 of FR-LITE-041.
    let (_scratch, artefacts) = built();
    let clock = settings::clock(None);

    let settings = over(&artefacts.main);
    let (tested, texts) = record::recorded(|| {
        crate::provider::test(crate::provider::Source::File(&settings), &clock)
    });
    let tested = tested.expect("the main database is tested");
    assert!(tested.can_read_catalogue);
    let issued = record::issued(&texts);
    assert_eq!(
        &issued[..statements::CONNECTION_START.len()],
        statements::CONNECTION_START
    );
    assert_eq!(
        &issued[statements::CONNECTION_START.len()..],
        [statements::CHECK]
    );

    let settings = over(&artefacts.pragma_view);
    let (refused, texts) = record::recorded(|| {
        crate::provider::test(crate::provider::Source::File(&settings), &clock)
    });
    assert_eq!(refused.expect_err("refused").exit_code(), 78);
    assert_eq!(record::issued(&texts).last(), Some(&statements::CHECK));
    assert!(super::quiescent());
}

/// The writer's connection, and what its `COMMIT` answered during the read.
type Written = (Connection, rusqlite::Result<()>);

#[test]
fn fr_lite_042_a_pragma_table_a_writer_commits_after_the_check_is_not_read_in_either_journal_mode()
{
    // FR-LITE-042: the check and every catalogue statement read one snapshot.
    // A second connection, given the moment between the check and the first
    // statement after it, creates a table named pragma_encoding holding the
    // row `forged` — which, read by the statement reading the encoding, would
    // forge it at exit 0. The read is unaffected: in rollback-journal mode the
    // writer cannot commit until the read ends, and in write-ahead-log mode
    // what it commits stays invisible to the read. Once the writer has
    // committed, a new read is refused by the check, which shows the table
    // was one the check exists for.
    let (_scratch, artefacts) = built();

    for (path, journal) in [(&artefacts.main, "rollback"), (&artefacts.wal, "wal")] {
        let slot: Rc<RefCell<Option<Written>>> = Rc::new(RefCell::new(None));
        let kept = Rc::clone(&slot);
        let target = path.clone();
        super::catalogue::race::after_the_check(move || {
            let writer = Connection::open(&target).expect("the writer opens");
            writer
                .busy_timeout(std::time::Duration::ZERO)
                .expect("the writer waits for nothing");
            writer
                .execute_batch(
                    "BEGIN IMMEDIATE; \
                     CREATE TABLE pragma_encoding (encoding TEXT); \
                     INSERT INTO pragma_encoding (encoding) VALUES ('forged');",
                )
                .expect("the writer writes its own transaction");
            let committed = writer.execute_batch("COMMIT");
            *kept.borrow_mut() = Some((writer, committed));
        });

        let database = read(path)
            .unwrap_or_else(|refused| panic!("{journal}: the read failed: {refused:?}"))
            .model()
            .expect("it folds");
        assert_eq!(
            database.charset, "UTF-8",
            "{journal}: the encoding was forged"
        );
        assert!(
            database
                .tables
                .iter()
                .all(|table| table.name() != "pragma_encoding"),
            "{journal}: the read listed the writer's table"
        );

        let (writer, committed) = slot.borrow_mut().take().expect("the writer ran");
        match journal {
            "rollback" => {
                let code = match &committed {
                    Err(rusqlite::Error::SqliteFailure(failure, _)) => failure.extended_code & 0xff,
                    other => panic!("the writer committed during the read: {other:?}"),
                };
                assert_eq!(code, rusqlite::ffi::SQLITE_BUSY);
                writer
                    .execute_batch("COMMIT")
                    .expect("the writer commits once the read has ended");
            }
            _ => committed.expect("the writer commits during the read"),
        }
        writer.close().expect("the writer closes");

        let refused = read(path).expect_err("the table the writer added is refused now");
        let Error::SqliteSchemaRefused { row, .. } = &refused else {
            panic!("{journal}: {refused:?}");
        };
        assert_eq!(row.name.as_str(), "pragma_encoding");
        assert_eq!(row.kind, RefusedKind::PragmaName);
    }
}

// ------------------------------------- the security review's proofs of concept ---

#[test]
fn fr_lite_015_the_poc_dos_shape_reads_without_compiling_a_view() {
    // The shape of the review's `poc_dos`: thirteen views, each selecting
    // twice from the one before, over a table of 2 000 columns. Listing the
    // objects through pragma_table_list compiled them and ran past every
    // deadline. The statements of FR-LITE-015 compile no view: with views
    // disabled a statement compiling one would fail and end the read with
    // 70, so the read's success shows none did — the deterministic form of
    // what the release build measures. The read presents every view whole.
    let scratch = Scratch::new();
    let path = scratch.directory("poc").join("poc-dos.sqlite");
    fixture::nested_views(&path, 2_000, 13);

    let clock = settings::clock(None);
    let session = super::open(&over(&path), &clock).expect("the file opens");
    let (answered, texts) = record::recorded(|| super::read(&session));
    let database = answered
        .expect("the read succeeds without compiling a view")
        .model()
        .expect("it folds");
    assert_eq!(record::issued(&texts), statements::READ);
    assert_eq!(database.views.len(), 14);
    assert_eq!(database.tables.len(), 1);
    assert_eq!(database.tables[0].columns().len(), 2_000);

    // The control: on the reader's own connection the top view fails to
    // compile at once.
    let connection = session.connection().expect("the connection is open");
    assert!(connection.prepare("SELECT * FROM main.v13").is_err());
    session.close();
}

#[test]
fn fr_lite_037_the_many_views_shape_reads_with_the_constant_plan() {
    // The shape of the review's `many_N`: one table and N views of the form
    // `SELECT 1`. pragma_table_list restarted its scan after each view it
    // compiled, for a time growing with the square of the objects. No
    // statement invokes it now, and the plan is the same constant over ten
    // thousand views as over none (FR-LITE-037).
    let scratch = Scratch::new();
    let path = scratch.directory("poc").join("many-views.sqlite");
    fixture::many_views(&path, 10_000);

    let (answered, texts) = record::recorded(|| read(&path));
    let database = answered
        .expect("the many views read")
        .model()
        .expect("it folds");
    let issued = record::issued(&texts);
    assert_eq!(
        &issued[statements::CONNECTION_START.len()..],
        statements::READ
    );
    assert_eq!(database.views.len(), 10_000);
    assert!(
        database
            .views
            .iter()
            .all(|view| view.definition.ends_with("AS SELECT 1")),
        "every view carries its whole stored statement"
    );
}

// ----------------------------------------------------------- FR-LITE-009 ---

#[test]
fn fr_lite_009_a_bound_past_i32_max_milliseconds_sets_the_busy_timeout_to_that_maximum() {
    // OD-36 as amended for rmp #317: the busy timeout takes the smaller of the
    // remaining bound and i32::MAX milliseconds, where rusqlite would panic
    // with "too big"; the watchdog keeps the whole bound.
    let ceiling = std::time::Duration::from_millis(u64::from(i32::MAX.unsigned_abs()));
    assert_eq!(
        super::busy_timeout(std::time::Duration::from_secs(30)),
        std::time::Duration::from_secs(30)
    );
    assert_eq!(super::busy_timeout(ceiling), ceiling);
    assert_eq!(super::busy_timeout(std::time::Duration::MAX), ceiling);

    // Through the reader: a core.query_timeout of 3 000 000 seconds, past the
    // ceiling, reads the main database and does not panic.
    let (_scratch, artefacts) = built();
    let deadlines = Deadlines::resolve(
        None,
        Some(Seconds::new(
            std::num::NonZeroU64::new(3_000_000).expect("not zero"),
        )),
        None,
        None,
    );
    let settings = FileSettings::new(ENTRY, &artefacts.main, deadlines);
    let clock = settings::clock(None);
    let session = super::open(&settings, &clock).expect("the file opens");
    let read = super::read(&session);
    session.close();
    read.expect("the read succeeds");

    // And the largest value the key admits.
    let deadlines = Deadlines::resolve(
        None,
        Some(Seconds::new(std::num::NonZeroU64::MAX)),
        None,
        None,
    );
    let settings = FileSettings::new(ENTRY, &artefacts.main, deadlines);
    let session = super::open(&settings, &clock).expect("the file opens");
    let read = super::read(&session);
    session.close();
    read.expect("the read succeeds");
}
