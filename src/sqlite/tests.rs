//! The tests of the SQLite reader over the fixture of `FR-LITE-038`.
//!
//! They are unit tests because two of them read the statement record of
//! `FR-LITE-018`, which is taken through the test seam of `FR-ERR-031` and is
//! compiled into no other build; the rest share the fixture with them. Each
//! test builds the fixture into a scratch directory of its own.

use std::borrow::Cow;

use rusqlite::Connection;
use rusqlite::config::DbConfig;

use super::catalogue::statements;
use super::fixture::{self, Artefacts};
use super::record;
use crate::deadline::{Deadlines, Seconds};
use crate::error::Error;
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
    // FR-LITE-018 items 1 and 2, over the main fixture database and over an
    // empty file: every statement the trace reports without the prefix is a
    // statement of FR-LITE-015, in the order of FR-LITE-017; every one with
    // it is nested and neither checked nor counted.
    let (_scratch, artefacts) = built();

    for path in [&artefacts.main, &artefacts.empty] {
        let (answered, texts) = record::recorded(|| read(path).map(drop));
        answered.expect("the file reads");
        let issued = record::issued(&texts);

        for text in &issued {
            assert!(
                statements::CLOSED_LIST.contains(text),
                "{text} is not a statement of FR-LITE-015"
            );
        }

        // The connection start first, in its order, then the plan, then the
        // per-view fallback where statement 7 failed.
        let start = statements::CONNECTION_START.len();
        assert_eq!(&issued[..start], statements::CONNECTION_START);
        let plan = statements::PLAN.len();
        assert_eq!(&issued[start..start + plan], statements::PLAN);
        assert!(
            issued[start + plan..]
                .iter()
                .all(|text| *text == statements::ONE_VIEW_COLUMNS)
        );

        for text in texts.iter().filter(|text| text.starts_with(record::NESTED)) {
            assert!(!issued.contains(&text.as_str()));
        }
    }
}

#[test]
fn fr_lite_037_the_plan_issues_as_many_statements_over_the_fixture_as_over_an_empty_file() {
    // FR-LITE-037 and NFR-PERF-001, counted under FR-LITE-018: the seven
    // statements of the plan over the fixture and over an empty file alike.
    // The one exception FR-LITE-037 admits is the fallback of FR-LITE-035
    // after statement 7 failed, which issues one statement per view and is
    // counted apart.
    let (_scratch, artefacts) = built();
    let plan = |path: &std::path::Path| {
        let (answered, texts) = record::recorded(|| read(path).map(drop));
        answered.expect("the file reads");
        let issued = record::issued(&texts)
            .into_iter()
            .map(str::to_owned)
            .collect::<Vec<_>>();
        let fallback = issued
            .iter()
            .filter(|text| *text == statements::ONE_VIEW_COLUMNS)
            .count();
        let catalogue = issued
            .iter()
            .filter(|text| statements::PLAN.contains(&text.as_str()))
            .count();
        (catalogue, fallback)
    };

    let (over_fixture, fallback) = plan(&artefacts.main);
    let (over_empty, none) = plan(&artefacts.empty);

    assert_eq!(over_fixture, over_empty);
    assert_eq!(over_fixture, 7);
    assert_eq!(none, 0);
    // The fixture holds five views, three of which do not compile, so the
    // fallback read each of the five once.
    assert_eq!(fallback, 5);
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
        .filter(|text| statements::PLAN.contains(text) || *text == statements::ONE_VIEW_COLUMNS)
        .count();

    assert_eq!(counted, catalogue);
    assert!(catalogue >= statements::PLAN.len());
}

#[test]
fn fr_lite_019_the_read_covers_main_and_its_tables_and_views_and_no_routine() {
    // FR-LITE-019 … FR-LITE-022: the database is `main`; tables carry
    // `BASE TABLE`; internal, virtual and shadow tables are excluded; and
    // there is no routine.
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
        "cargo_note_config",
        "cargo_note_content",
        "cargo_note_data",
        "cargo_note_docsize",
        "cargo_note_idx",
    ] {
        assert!(!names.contains(&excluded), "{excluded} is excluded");
    }
    assert!(
        database
            .tables
            .iter()
            .all(|table| table.table_type() == TableType::Base)
    );
    let views: Vec<&str> = database.views.iter().map(|view| &*view.name).collect();
    assert_eq!(
        views,
        [
            "v_document_paths",
            "v_port_country",
            "v_quay_schedule",
            "v_ranked_notes",
            "v_vessel_fleet"
        ]
    );
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
    // A STRICT column declared ANY.
    let line = table(&database, "manifest_line");
    assert_eq!(
        column(line, "payload").column_type.data_type(),
        Some("blob")
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
fn fr_lite_033_a_view_carries_its_whole_statement_and_null_for_the_server_fields() {
    let (_scratch, artefacts) = built();
    let database = model(&artefacts);
    let fleet = database
        .views
        .iter()
        .find(|view| view.name == "v_vessel_fleet")
        .expect("the view is carried");

    assert!(
        fleet
            .definition
            .starts_with("CREATE VIEW v_vessel_fleet AS")
    );
    assert_eq!(fleet.check_option, None);
    assert_eq!(fleet.is_updatable, None);
    assert_eq!(fleet.definer, None);
    assert_eq!(fleet.security_type, None);
    assert_eq!(fleet.character_set_client, None);
    assert_eq!(fleet.collation_connection, None);
    assert_eq!(fleet.algorithm, None);
    assert_eq!(fleet.restricted, None);
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
fn fr_lite_035_exactly_the_views_that_do_not_compile_are_marked_with_the_library_reason() {
    let (_scratch, artefacts) = built();
    let catalogue = read(&artefacts.main).expect("the file reads");
    let database = catalogue.model().expect("it folds");

    let marked: Vec<&str> = database
        .views
        .iter()
        .filter(|view| view.restricted.is_some())
        .map(|view| &*view.name)
        .collect();
    assert_eq!(
        marked,
        ["v_document_paths", "v_quay_schedule", "v_ranked_notes"]
    );
    for view in database
        .views
        .iter()
        .filter(|view| view.restricted.is_some())
    {
        assert_eq!(
            view.restricted
                .as_ref()
                .expect("marked")
                .properties()
                .iter()
                .map(|property| &**property)
                .collect::<Vec<_>>(),
            ["columns"]
        );
    }

    let reasons: Vec<(&str, i32)> = catalogue
        .unreadable()
        .iter()
        .map(|unreadable| (unreadable.name.as_str(), unreadable.reason.code & 0xff))
        .collect();
    assert_eq!(
        reasons,
        [
            ("v_document_paths", rusqlite::ffi::SQLITE_ERROR),
            ("v_quay_schedule", rusqlite::ffi::SQLITE_ERROR),
            ("v_ranked_notes", rusqlite::ffi::SQLITE_ERROR),
        ]
    );
    let quay = &catalogue.unreadable()[1];
    assert!(
        quay.reason
            .message
            .as_ref()
            .is_some_and(|message| message.as_str().contains("quay_schedule"))
    );
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
