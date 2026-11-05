//! The `mysql` provider, observed from outside the process
//! (`specification/mysql-contract.md`).
//!
//! Every body drives the binary the project distributes against the fixture of
//! `scripts/mysql/`, through its harness, on every series of `FR-MYSQL-003`
//! where it touches a server, and skips with its reason where the fixture is
//! not up. Three bodies need no server, and one needs the MariaDB fixture of
//! `scripts/mariadb/` as well, for the product refusal `FR-MYSQL-043` assigns
//! to it.
//!
//! | Requirement | What is observed |
//! |---|---|
//! | `FR-CONF-051`, `FR-CONF-052`, `FR-CFG-055` | `provider = "mysql"` is read, written and refused as a `mariadb` entry's keys are |
//! | `FR-MYSQL-023` … `FR-MYSQL-039` | Every object kind of the fixture, read into the document |
//! | `FR-MYSQL-013`, `FR-SRV-026` | Identical DDL yields identical documents on both series |
//! | `FR-MYSQL-046` | The fixture's expressions carried with one layer of escaping removed |
//! | `FR-MYSQL-027`, `FR-MYSQL-033` | The generated invisible primary key and the `SET DEFAULT` key |
//! | `FR-MYSQL-041` | The reduced-grant reader: marked views and routines, no trigger, `77` for a named one |
//! | `FR-MYSQL-009` | The `8.0` server refused with `78` |
//! | `FR-MYSQL-015`, `FR-MYSQL-017`, `FR-MYSQL-018` | The closed list in the server's record, in order, and the read-back |
//! | `FR-MYSQL-021` | The five modes against a server offering TLS and one offering none |
//! | `FR-MYSQL-045` | The uncached login over an unencrypted session ends with `78` |
//! | `FR-MYSQL-042` | Twelve statements over `freight` and `freight_empty`, one connection, none on a cache hit |
//! | `FR-PROJ-021` | The starter template renders against every table, on both series |
//! | `FR-MYSQL-008`, `FR-MYSQL-012` | A `mysql` entry on MariaDB, and a `mariadb` entry on MySQL, both `78` |
//! | `FR-CFG-024`, `FR-CTX-043`, `FR-ENV-049` | `cfg database test`, a `--context` render, and the `textual` family of `json` |
//! | `FR-MYSQL-048`, `FR-SRV-043`, `FR-PRIV-021`, `ADR-015` | The target of item 17 read whole and no decoy with it, by both accounts; a missing name of the same characters `77`; each statement received as its constant with the literal in its placeholder |

#[path = "support/fixture.rs"]
mod mariadb_fixture;
#[path = "support/mysql.rs"]
mod mysql;
#[path = "support/sandbox.rs"]
mod sandbox;

use std::collections::BTreeSet;

use mysql::{Role, Server};
use sandbox::Sandbox;
use serde_json::Value;

/// The entry every body names.
const ENTRY: &str = "shop";

/// The statements a full read issues: one per table of `FR-MYSQL-015`.
const FULL_READ: i64 = 12;

/// The connection-start statements of `FR-MYSQL-015`, in the order of
/// `FR-MYSQL-017`.
const READ_ONLY: &str = "SET SESSION TRANSACTION READ ONLY";
const READ_BACK: &str = "SELECT @@session.transaction_read_only";
const PROBE: &str = "SELECT VERSION()";

/// The leading token of the one diagnostic line per catalogue query
/// (`FR-GLOB-017`, `NFR-PERF-008`).
const QUERY_TOKEN: &str = "query:";

/// What one run produced.
struct Outcome {
    /// The exit code.
    code: Option<i32>,
    /// Standard output.
    out: String,
    /// Standard error.
    err: String,
}

/// Runs `tpl` in `sandbox`.
fn run(sandbox: &Sandbox, arguments: &[&str]) -> Outcome {
    let printed = sandbox.run(arguments);

    Outcome {
        code: printed.status.code(),
        out: String::from_utf8_lossy(&printed.stdout).into_owned(),
        err: String::from_utf8_lossy(&printed.stderr).into_owned(),
    }
}

/// Runs `tpl` in `sandbox` and refuses anything but exit `0`.
fn succeeds(sandbox: &Sandbox, arguments: &[&str]) -> String {
    let outcome = run(sandbox, arguments);

    assert_eq!(
        outcome.code,
        Some(0),
        "tpl {} exited {:?}: {}",
        arguments.join(" "),
        outcome.code,
        outcome.err
    );

    outcome.out
}

/// The labelled line `label` of a diagnostic.
fn line<'a>(err: &'a str, label: &str) -> &'a str {
    err.lines()
        .find(|written| written.starts_with(label))
        .unwrap_or_else(|| panic!("no {label} line in {err:?}"))
}

/// The root account of the fixture.
fn root() -> (&'static str, &'static str) {
    let names = mysql::names();
    (names.root.0.as_str(), names.root.1.as_str())
}

/// The reduced-grant reader of the fixture.
fn reader() -> (&'static str, &'static str) {
    let names = mysql::names();
    (names.reader.0.as_str(), names.reader.1.as_str())
}

/// The fixture's main schema.
fn schema() -> &'static str {
    &mysql::names().schema
}

/// A sandbox holding a project with one `mysql` entry reaching `server`.
fn project(server: &Server, account: (&str, &str), database: &str, tls: &str) -> Sandbox {
    let sandbox = Sandbox::new();
    sandbox.project(&mysql::configuration(server, ENTRY, database, account, tls));

    sandbox
}

/// The `database` object of the document a pure read of `sandbox` writes.
fn dump(sandbox: &Sandbox) -> Value {
    let printed = succeeds(sandbox, &["schema", "dump", "--direct", "--no-cache"]);
    let document: Value = serde_json::from_str(&printed).expect("the dump is a JSON document");

    document["data"]["database"].clone()
}

/// The member of `collection` named `name`.
fn named<'a>(collection: &'a Value, name: &str) -> &'a Value {
    collection
        .as_array()
        .expect("a collection is an array")
        .iter()
        .find(|member| member["name"] == name)
        .unwrap_or_else(|| panic!("no member named {name} in {collection}"))
}

/// The names of the members of `collection`.
fn names_of(collection: &Value) -> BTreeSet<String> {
    collection
        .as_array()
        .expect("a collection is an array")
        .iter()
        .filter_map(|member| member["name"].as_str().map(str::to_owned))
        .collect()
}

// ------------------------------------------------- the provider, no server ---

#[test]
fn fr_conf_051_a_mysql_entry_is_read_written_and_refused_as_a_server_entry() {
    // FR-CONF-051: `mysql` is the third value; FR-CONF-052: a `mysql` entry
    // takes and refuses the keys a `mariadb` entry does; FR-CFG-055 item 2:
    // `--provider mysql` takes the flags `--provider mariadb` takes.
    let sandbox = Sandbox::new();
    sandbox.project("[core]\n");

    succeeds(
        &sandbox,
        &[
            "cfg",
            "database",
            "add",
            "orders",
            "--provider",
            "mysql",
            "--host",
            "db.example.com",
            "--schema",
            "orders",
        ],
    );
    let written = String::from_utf8(sandbox.configuration()).expect("the file is text");
    assert!(written.contains("provider = \"mysql\""), "{written}");

    let shown = succeeds(
        &sandbox,
        &["cfg", "database", "show", "orders", "--format", "json"],
    );
    assert!(shown.contains("\"mysql\""), "{shown}");

    // --path belongs to a sqlite entry: refused beside --provider mysql.
    let refused = run(
        &sandbox,
        &[
            "cfg",
            "database",
            "add",
            "ledger",
            "--provider",
            "mysql",
            "--path",
            "ledger.sqlite",
        ],
    );
    assert_eq!(refused.code, Some(64), "{}", refused.err);
    assert!(
        line(&refused.err, "cause:").contains("mysql"),
        "{}",
        refused.err
    );

    // A value outside the three names the three.
    let outside = run(
        &sandbox,
        &[
            "cfg",
            "database",
            "add",
            "x",
            "--provider",
            "postgres",
            "--host",
            "db",
            "--schema",
            "x",
        ],
    );
    assert_eq!(outside.code, Some(64), "{}", outside.err);
    let cause = line(&outside.err, "cause:");
    for value in ["mariadb", "mysql", "sqlite"] {
        assert!(cause.contains(value), "{}", outside.err);
    }

    // Moving an entry between the two server providers leaves no key refused.
    succeeds(
        &sandbox,
        &["cfg", "set", "database.orders.provider", "mariadb"],
    );
    succeeds(
        &sandbox,
        &["cfg", "set", "database.orders.provider", "mysql"],
    );

    // The help names the three values.
    let help = succeeds(&sandbox, &["cfg", "database", "add", "-h"]);
    let flowed = help.split_whitespace().collect::<Vec<_>>().join(" ");
    assert!(flowed.contains("mysql, a MySQL server"), "{help}");
    assert!(flowed.contains("one of mariadb, mysql, sqlite"), "{help}");
}

#[test]
fn fr_conf_052_a_mysql_entry_carrying_path_is_refused_at_entry_resolution() {
    let sandbox = Sandbox::new();
    sandbox.project(&format!(
        "[core]\ndatabase = \"{ENTRY}\"\n\n[database.{ENTRY}]\nprovider = \"mysql\"\nhost = \
         \"db.example.com\"\ndatabase = \"orders\"\npath = \"orders.sqlite\"\n"
    ));

    let refused = run(&sandbox, &["schema", "info"]);
    assert_eq!(refused.code, Some(78), "{}", refused.err);
    let cause = line(&refused.err, "cause:");
    assert!(
        cause.contains("mysql") && cause.contains("path"),
        "{}",
        refused.err
    );
}

// ------------------------------------------------------ the catalogue read ---

#[test]
fn fr_mysql_023_to_039_every_object_kind_of_the_fixture_reads_into_the_document() {
    let _exclusive = mysql::exclusive();
    let Some(series) = mysql::series(
        "fr_mysql_023_to_039_every_object_kind_of_the_fixture_reads_into_the_document",
    ) else {
        return;
    };

    for server in &series {
        let name = server.name();
        let database = dump(&project(server, root(), schema(), "verify-identity"));

        // FR-MYSQL-025 and FR-MYSQL-006.
        assert_eq!(database["provider"], "mysql", "{name}");
        assert_eq!(database["name"], schema(), "{name}");
        assert_eq!(database["collation"], "utf8mb4_unicode_520_ci", "{name}");
        assert_eq!(database["server"]["series"], server.series(), "{name}");
        assert_eq!(database["server"]["standing"], "supported", "{name}");
        assert!(
            database["server"]["version"]
                .as_str()
                .is_some_and(|version| version.starts_with(server.series())),
            "{name}"
        );

        // FR-MYSQL-023: the 22 base tables, the five views and the seven
        // routines; a partitioned table is a table.
        let tables = &database["tables"];
        assert_eq!(tables.as_array().map(Vec::len), Some(22), "{name}");
        assert_eq!(
            database["views"].as_array().map(Vec::len),
            Some(5),
            "{name}"
        );
        assert_eq!(
            database["routines"].as_array().map(Vec::len),
            Some(7),
            "{name}"
        );
        for table in tables.as_array().expect("tables") {
            assert_eq!(table["table_type"], "BASE TABLE", "{name}");
        }
        assert!(names_of(tables).contains("vessel_position"), "{name}");

        // FR-MYSQL-028: the type of a MySQL column.
        let edi = named(tables, "legacy_edi_field");
        let zerofill = named(&edi["columns"], "element_position");
        assert_eq!(
            zerofill["column_type"], "smallint(4) unsigned zerofill",
            "{name}"
        );
        assert_eq!(zerofill["unsigned"], true, "{name}");
        let customer = named(tables, "customer");
        let preferences = named(&customer["columns"], "preferences");
        assert_eq!(preferences["data_type"], "json", "{name}");
        assert!(
            preferences["charset"].is_null() && preferences["length"].is_null(),
            "{name}"
        );
        let facility = named(tables, "port_facility");
        assert_eq!(
            named(&facility["columns"], "notices_to_mariners")["data_type"],
            "geomcollection",
            "{name}"
        );
        let vessel = named(tables, "vessel");
        let society = named(&vessel["columns"], "class_society");
        assert_eq!(society["values"][0], "Lloyd's Register", "{name}");
        assert_eq!(
            named(&vessel["columns"], "in_service")["column_type"],
            "tinyint(1)",
            "{name}"
        );

        // FR-MYSQL-029: the classification, row by row, on the fixture's
        // columns.
        let consignment = named(tables, "consignment");
        let edi_columns = &edi["columns"];
        assert_eq!(
            named(edi_columns, "qualifier")["default"]["kind"],
            "literal",
            "{name}"
        );
        assert_eq!(
            named(edi_columns, "qualifier")["default"]["value"],
            "",
            "{name}"
        );
        assert_eq!(
            named(edi_columns, "null_marker")["default"]["value"],
            "NULL",
            "{name}"
        );
        assert_eq!(
            named(edi_columns, "segment_terminator")["default"]["value"],
            "0x27",
            "{name}"
        );
        assert_eq!(
            named(edi_columns, "una_service_string")["default"]["kind"],
            "expression"
        );
        let port_facility_kind = named(&facility["columns"], "facility_kind");
        assert_eq!(
            port_facility_kind["default"]["value"], "Container terminal",
            "{name}"
        );
        let due = named(&consignment["columns"], "payment_due_on");
        assert_eq!(due["default"]["kind"], "expression", "{name}");
        assert_eq!(
            due["default"]["value"], "curdate() + interval 30 day",
            "{name}"
        );
        let broker = named(&consignment["columns"], "broker_reference");
        assert_eq!(broker["default"]["kind"], "null", "{name}");
        assert!(named(&consignment["columns"], "shipper_reference")["default"].is_null());
        assert_eq!(
            named(&customer["columns"], "updated_at")["default"]["value"],
            "CURRENT_TIMESTAMP",
            "{name}"
        );

        // FR-MYSQL-030: the attribute field, phrase by phrase.
        let updated = named(&consignment["columns"], "updated_at");
        assert_eq!(
            updated["on_update"], "on update CURRENT_TIMESTAMP(3)",
            "{name}"
        );
        let insured = named(&consignment["columns"], "insured_amount");
        assert_eq!(insured["invisible"], true, "{name}");
        assert_eq!(
            insured["generated"]["storage"], "VIRTUAL GENERATED",
            "{name}"
        );
        let token = named(&customer["columns"], "crm_sync_token");
        assert_eq!(token["invisible"], true, "{name}");
        assert_eq!(token["default"]["kind"], "expression", "{name}");
        let audit = named(tables, "audit_event");
        let audit_key = named(&audit["columns"], "audit_event_id");
        assert_eq!(audit_key["auto_increment"], true, "{name}");
        assert_eq!(audit_key["invisible"], true, "{name}");

        // FR-MYSQL-031: the storage kind and the expression of a generated
        // column, and no other column generated.
        let cargo = named(tables, "cargo_item");
        let dangerous = named(&cargo["columns"], "is_dangerous_goods");
        assert_eq!(
            dangerous["generated"]["storage"], "STORED GENERATED",
            "{name}"
        );
        assert!(
            named(&cargo["columns"], "description")["generated"].is_null(),
            "{name}"
        );

        // FR-MYSQL-024 and FR-MYSQL-032: indexes.
        let customer_indexes = names_of(&customer["indexes"]);
        assert!(
            !customer_indexes.contains("uq_customer_vat_normalised"),
            "{name}"
        );
        assert!(
            !customer_indexes.contains("idx_customer_milestone_alerts"),
            "{name}"
        );
        assert!(
            !names_of(&named(tables, "voyage")["indexes"]).contains("idx_voyage_departure_year")
        );
        let spatial = named(&facility["indexes"], "sp_facility_berth_outline");
        assert_eq!(spatial["type"], "SPATIAL", "{name}");
        let invisible = named(&named(tables, "charge")["indexes"], "idx_charge_invoice");
        assert_eq!(invisible["ignored"], true, "{name}");
        let descending = named(&named(tables, "voyage")["indexes"], "idx_voyage_etd_desc");
        assert_eq!(descending["columns"][0]["direction"], "D", "{name}");
        let prefix = named(&vessel["indexes"], "idx_vessel_name_prefix");
        assert_eq!(prefix["columns"][0]["prefix_length"], 20, "{name}");
        assert_eq!(
            named(
                &named(tables, "cargo_item")["indexes"],
                "ft_cargo_item_description"
            )["type"],
            "FULLTEXT",
            "{name}"
        );

        // FR-MYSQL-032: the primary key is the index named PRIMARY, and a
        // table with a NOT NULL unique index and none carries none.
        assert!(named(tables, "currency")["primary_key"].is_null(), "{name}");
        assert!(
            named(tables, "edi_message_staging")["primary_key"].is_null(),
            "{name}"
        );
        assert_eq!(
            named(tables, "voyage_leg")["primary_key"]["columns"]
                .as_array()
                .map(Vec::len),
            Some(3),
            "{name}"
        );

        // FR-MYSQL-033: every key of `freight`, the cross-schema ones excluded,
        // with the rules as reported.
        let keys: usize = tables
            .as_array()
            .expect("tables")
            .iter()
            .map(|table| table["foreign_keys"].as_array().map_or(0, Vec::len))
            .sum();
        assert_eq!(keys, 17, "{name}");
        assert!(
            named(tables, "port")["foreign_keys"]
                .as_array()
                .is_some_and(Vec::is_empty)
        );
        let shipper = named(&consignment["foreign_keys"], "fk_consignment_shipper");
        assert_eq!(shipper["on_delete"], "NO ACTION", "{name}");
        assert_eq!(shipper["on_update"], "NO ACTION", "{name}");
        assert_eq!(shipper["referenced_key"], "PRIMARY", "{name}");

        // FR-MYSQL-034: every constraint, level null, NOT ENFORCED included.
        let checks = names_of(&vessel["check_constraints"]);
        assert!(checks.contains("chk_vessel_dimensions"), "{name}");
        assert!(
            names_of(&cargo["check_constraints"]).contains("cargo_item_chk_1"),
            "{name}"
        );
        for table in tables.as_array().expect("tables") {
            for check in table["check_constraints"].as_array().expect("checks") {
                assert!(check["level"].is_null(), "{name}: {check}");
            }
        }

        // FR-MYSQL-035: views, no algorithm, CASCADED as reported.
        for view in database["views"].as_array().expect("views") {
            assert!(view["algorithm"].is_null(), "{name}");
            assert!(
                !view["definition"].as_str().unwrap_or_default().is_empty(),
                "{name}"
            );
        }
        assert_eq!(
            named(&database["views"], "v_reefer_container_deep_frozen")["check_option"],
            "CASCADED"
        );
        assert_eq!(
            named(&database["views"], "v_consignment_manifest")["is_updatable"],
            false
        );

        // FR-MYSQL-036 and FR-MYSQL-037: routines and parameters.
        let book = named(&database["routines"], "sp_book_consignment");
        assert_eq!(book["kind"], "PROCEDURE", "{name}");
        assert!(book["return_type"].is_null(), "{name}");
        let modes: BTreeSet<&str> =
            named(&database["routines"], "sp_port_call_summary")["parameters"]
                .as_array()
                .expect("parameters")
                .iter()
                .filter_map(|parameter| parameter["mode"].as_str())
                .collect();
        assert_eq!(modes, BTreeSet::from(["IN", "INOUT", "OUT"]), "{name}");
        let function = named(&database["routines"], "fn_booking_reference");
        assert_eq!(function["return_type"]["data_type"], "varchar", "{name}");
        assert_eq!(
            function["parameters"].as_array().map(Vec::len),
            Some(1),
            "{name}"
        );

        // FR-MYSQL-038: the triggers, and the second of one event and timing.
        let triggers = &consignment["triggers"];
        assert_eq!(triggers.as_array().map(Vec::len), Some(7), "{name}");
        assert_eq!(
            named(triggers, "trg_consignment_before_insert_currency")["action_order"],
            2,
            "{name}"
        );
    }
}

#[test]
fn fr_mysql_013_identical_ddl_yields_identical_documents_on_every_series() {
    // FR-MYSQL-013 adopts FR-SRV-026, verified as FR-SRV-029 requires: the
    // record of FR-MYSQL-014 holds no difference in a field the model carries,
    // so the two documents differ in the `server` object alone.
    let _exclusive = mysql::exclusive();
    let Some(series) =
        mysql::series("fr_mysql_013_identical_ddl_yields_identical_documents_on_every_series")
    else {
        return;
    };

    let mut documents = series.iter().map(|server| {
        let mut database = dump(&project(server, root(), schema(), "verify-identity"));
        database["server"] = Value::Null;
        (server.name().to_owned(), database)
    });
    let (first_name, first) = documents.next().expect("at least one series");

    for (name, other) in documents {
        assert_eq!(
            other, first,
            "{name} and {first_name} read the same DDL differently"
        );
    }
}

#[test]
fn fr_mysql_046_the_fixture_expressions_are_carried_with_one_layer_removed() {
    // FR-MYSQL-046 on the fixture's own texts: a generation expression, a check
    // clause and an expression default each holding a string literal, read as
    // the model carries them, the introducers kept.
    let _exclusive = mysql::exclusive();
    let Some(series) =
        mysql::series("fr_mysql_046_the_fixture_expressions_are_carried_with_one_layer_removed")
    else {
        return;
    };

    for server in &series {
        let name = server.name();
        let database = dump(&project(server, root(), schema(), "verify-identity"));
        let tables = &database["tables"];

        assert_eq!(
            named(
                &named(tables, "cargo_item")["columns"],
                "is_dangerous_goods"
            )["generated"]["expression"],
            "(`imdg_class` <> _utf8mb4'Not regulated')",
            "{name}"
        );
        assert_eq!(
            named(
                &named(tables, "voyage")["check_constraints"],
                "chk_voyage_direction"
            )["clause"],
            "(right(`voyage_number`,1) in (_utf8mb4'N',_utf8mb4'S',_utf8mb4'E',_utf8mb4'W'))",
            "{name}"
        );
        assert_eq!(
            named(&named(tables, "customer")["columns"], "preferences")["default"]["value"],
            "json_object(_utf8mb4'language',_utf8mb4'en-GB',_utf8mb4'invoice_format',_utf8mb4'PDF')",
            "{name}"
        );

        // No text the rule reaches keeps the layer: no escaped quote survives.
        for table in tables.as_array().expect("tables") {
            for check in table["check_constraints"].as_array().expect("checks") {
                assert!(
                    !check["clause"].as_str().unwrap_or_default().contains("\\'"),
                    "{name}"
                );
            }
            for column in table["columns"].as_array().expect("columns") {
                let expression = column["generated"]["expression"]
                    .as_str()
                    .unwrap_or_default();
                assert!(!expression.contains("\\'"), "{name}: {expression}");
            }
        }

        // A literal is carried unchanged: the apostrophe of 8'6" stays one.
        assert_eq!(
            named(&named(tables, "container_type")["columns"], "height_ft_in")["default"]["value"],
            "8'6\"",
            "{name}"
        );
    }
}

#[test]
fn fr_mysql_027_and_fr_mysql_033_the_generated_key_and_the_set_default_rules_are_carried() {
    // The two cases FR-MYSQL-044 lists apart from its sixteen items, which the
    // fixture's `equipment_survey` holds.
    let _exclusive = mysql::exclusive();
    let Some(series) = mysql::series(
        "fr_mysql_027_and_fr_mysql_033_the_generated_key_and_the_set_default_rules_are_carried",
    ) else {
        return;
    };

    for server in &series {
        let name = server.name();
        let database = dump(&project(server, root(), schema(), "verify-identity"));
        let survey = named(&database["tables"], "equipment_survey");

        // FR-MYSQL-027: my_row_id at position 1, invisible and
        // auto-incremental, and the key PRIMARY on it.
        let generated = &survey["columns"][0];
        assert_eq!(generated["name"], "my_row_id", "{name}");
        assert_eq!(generated["position"], 1, "{name}");
        assert_eq!(generated["invisible"], true, "{name}");
        assert_eq!(generated["auto_increment"], true, "{name}");
        assert_eq!(generated["column_type"], "bigint unsigned", "{name}");
        assert_eq!(
            survey["primary_key"]["columns"][0]["name"], "my_row_id",
            "{name}"
        );

        // FR-MYSQL-033: SET DEFAULT carried as reported, in both directions.
        let key = named(&survey["foreign_keys"], "fk_survey_container_type");
        assert_eq!(key["on_delete"], "SET DEFAULT", "{name}");
        assert_eq!(key["on_update"], "SET DEFAULT", "{name}");
        let incoming = named(&database["tables"], "container_type")["referenced_by"]
            .as_array()
            .expect("referenced_by")
            .iter()
            .any(|entry| entry["key"]["name"] == "fk_survey_container_type");
        assert!(incoming, "{name}");
    }
}

#[test]
fn fr_mysql_041_a_reduced_grant_reader_reads_every_object_and_the_short_ones_are_marked() {
    let _exclusive = mysql::exclusive();
    let Some(series) = mysql::series(
        "fr_mysql_041_a_reduced_grant_reader_reads_every_object_and_the_short_ones_are_marked",
    ) else {
        return;
    };

    for server in &series {
        let name = server.name();
        // The reader's login is cached by a TLS login, which every mode that
        // encrypts makes; the read below is over TLS.
        let sandbox = project(server, reader(), schema(), "verify-identity");
        let database = dump(&sandbox);
        let full = dump(&project(server, root(), schema(), "verify-identity"));

        // Every table and view is seen.
        assert_eq!(
            names_of(&database["tables"]),
            names_of(&full["tables"]),
            "{name}"
        );
        assert_eq!(
            names_of(&database["views"]),
            names_of(&full["views"]),
            "{name}"
        );

        // FR-PRIV-011: a view's definition, the empty string, marked.
        for view in database["views"].as_array().expect("views") {
            assert_eq!(view["definition"], "", "{name}");
            assert_eq!(
                view["restricted"],
                serde_json::json!(["definition"]),
                "{name}"
            );
        }
        // FR-PRIV-017: a routine's body, SQL NULL, marked.
        for routine in database["routines"].as_array().expect("routines") {
            assert!(routine["body"].is_null(), "{name}");
            assert_eq!(routine["restricted"], serde_json::json!(["body"]), "{name}");
        }
        // FR-PRIV-020: no trigger row, and no marking for it; FR-PRIV-019: the
        // constraint tables are whole, so no table is marked.
        for table in database["tables"].as_array().expect("tables") {
            assert!(
                table["triggers"].as_array().is_some_and(Vec::is_empty),
                "{name}"
            );
            assert!(
                table.get("restricted").is_none(),
                "{name}: {}",
                table["name"]
            );
            // The keys and the constraints are the privileged reader's; the
            // embedded tables differ in their triggers alone, which this
            // reader cannot see.
            let reference = named(&full["tables"], table["name"].as_str().unwrap_or_default());
            assert_eq!(
                names_of(&table["foreign_keys"]),
                names_of(&reference["foreign_keys"]),
                "{name}"
            );
            assert_eq!(
                table["check_constraints"], reference["check_constraints"],
                "{name}"
            );
        }

        // FR-PRIV-003: an object named that comes back incomplete is 77.
        for arguments in [
            [
                "schema",
                "view",
                "v_port_directory",
                "--direct",
                "--no-cache",
            ],
            [
                "schema",
                "routine",
                "sp_book_consignment",
                "--direct",
                "--no-cache",
            ],
        ] {
            let refused = run(&sandbox, &arguments);
            assert_eq!(refused.code, Some(77), "{name}: {}", refused.err);
        }
    }
}

// ------------------------------------------------- the connection start ---

#[test]
fn fr_mysql_009_a_series_outside_the_set_and_older_than_its_newest_is_refused_with_78() {
    let _exclusive = mysql::exclusive();
    let Some(refused) = mysql::servers(
        "fr_mysql_009_a_series_outside_the_set_and_older_than_its_newest_is_refused_with_78",
        Role::Refused,
    ) else {
        return;
    };

    for server in &refused {
        let sandbox = project(server, root(), schema(), "verify-identity");

        mysql::statements_on(server);
        let outcome = run(&sandbox, &["schema", "info", "--direct", "--no-cache"]);
        mysql::statements_off(server);

        assert_eq!(outcome.code, Some(78), "{}", outcome.err);
        assert!(
            line(&outcome.err, "error:").contains(&format!("server series '{}'", server.series())),
            "{}",
            outcome.err
        );
        let cause = line(&outcome.err, "cause:");
        assert!(
            cause.contains("connected and authenticated"),
            "{}",
            outcome.err
        );
        assert!(cause.contains("9.7 and 8.4"), "{}", outcome.err);
        assert!(cause.contains("mysql entry"), "{}", outcome.err);
        assert!(
            line(&outcome.err, "hint:")
                .contains(&format!("tpl cfg database update {ENTRY} --host")),
            "{}",
            outcome.err
        );

        // The catalogue was not read.
        assert_eq!(
            mysql::statements_count(server, &["--user", root().0, "--catalogue"]),
            0,
            "{}",
            server.name()
        );

        // `tpl cfg database test` refuses it the same way (FR-MYSQL-011).
        let tested = run(&sandbox, &["cfg", "database", "test", ENTRY]);
        assert_eq!(tested.code, Some(78), "{}", tested.err);
    }
}

#[test]
fn fr_mysql_043_the_server_receives_the_closed_list_in_the_order_of_fr_mysql_017() {
    // FR-MYSQL-043 applies FR-SRV-012 and the confirming half of FR-SRV-013:
    // the statements the server actually receives are the four kinds of
    // FR-MYSQL-015 and no fifth, the three connection-start statements once
    // each in the order of FR-MYSQL-017, then the twelve catalogue statements.
    let _exclusive = mysql::exclusive();
    let Some(series) = mysql::series(
        "fr_mysql_043_the_server_receives_the_closed_list_in_the_order_of_fr_mysql_017",
    ) else {
        return;
    };

    for server in &series {
        let name = server.name();
        let sandbox = project(server, root(), schema(), "verify-identity");

        mysql::statements_on(server);
        succeeds(&sandbox, &["schema", "dump", "--direct", "--no-cache"]);
        mysql::statements_off(server);

        let rows = mysql::received(server, root().0);
        assert!(!rows.is_empty(), "{name}: the record held nothing");

        let mut start: Vec<&str> = Vec::new();
        let mut catalogue = 0;
        let mut catalogue_before_start = false;

        for (kind, statement) in &rows {
            match kind.as_str() {
                "Connect" | "Quit" => continue,
                "Query" | "Prepare" | "Execute" => {}
                other => panic!("{name}: the server received a {other} row: {statement}"),
            }

            if statement.contains("INFORMATION_SCHEMA") {
                if kind == "Execute" || kind == "Query" {
                    catalogue += 1;
                    catalogue_before_start |= start.len() < 3;
                }
                continue;
            }

            assert!(
                [READ_ONLY, READ_BACK, PROBE].contains(&statement.as_str()),
                "{name}: a fifth kind of statement: {statement:?}"
            );
            start.push(match statement.as_str() {
                READ_ONLY => READ_ONLY,
                READ_BACK => READ_BACK,
                _ => PROBE,
            });
        }

        assert_eq!(start, [READ_ONLY, READ_BACK, PROBE], "{name}");
        assert!(
            !catalogue_before_start,
            "{name}: a catalogue statement preceded the start"
        );
        assert_eq!(i64::from(catalogue), FULL_READ, "{name}");
        // The read-back names transaction_read_only and no other spelling.
        assert!(
            !rows
                .iter()
                .any(|(_, statement)| statement.contains("tx_read_only")),
            "{name}"
        );
    }
}

#[test]
fn fr_mysql_021_the_five_modes_against_a_server_offering_tls_and_one_offering_none() {
    // FR-MYSQL-021: each mode's behaviour against both kinds of server, the
    // session's encryption read from the server's own record of the
    // connection — `using SSL/TLS` or `using TCP/IP`.
    let _exclusive = mysql::exclusive();
    let test = "fr_mysql_021_the_five_modes_against_a_server_offering_tls_and_one_offering_none";
    let (Some(tls), Some(plain)) = (mysql::series(test), mysql::servers(test, Role::NoTls)) else {
        return;
    };

    // The transport of the one connection the record holds for root.
    let transport = |server: &Server, tls_mode: &str| -> Outcome {
        let sandbox = project(server, root(), schema(), tls_mode);
        mysql::statements_on(server);
        let outcome = run(&sandbox, &["schema", "info", "--direct", "--no-cache"]);
        mysql::statements_off(server);
        let connect = mysql::received(server, root().0)
            .into_iter()
            .find(|(kind, _)| kind == "Connect")
            .map(|(_, statement)| statement)
            .unwrap_or_default();

        Outcome {
            code: outcome.code,
            out: connect,
            err: outcome.err,
        }
    };

    for server in &tls {
        let name = server.name();
        // The encrypting modes first, which also fill the authentication cache
        // the plaintext login of `disabled` needs (FR-MYSQL-045).
        for mode in ["verify-identity", "verify-ca", "required", "preferred"] {
            let outcome = transport(server, mode);
            assert_eq!(outcome.code, Some(0), "{name} {mode}: {}", outcome.err);
            assert!(
                outcome.out.contains("using SSL/TLS"),
                "{name} {mode}: {}",
                outcome.out
            );
        }
        let outcome = transport(server, "disabled");
        assert_eq!(outcome.code, Some(0), "{name} disabled: {}", outcome.err);
        assert!(
            outcome.out.contains("using TCP/IP"),
            "{name} disabled: {}",
            outcome.out
        );
    }

    for server in &plain {
        let name = server.name();
        // Only a cached login authenticates over plaintext (FR-MYSQL-045), so
        // the harness fills the cache first.
        mysql::prime_authentication_cache(server, root().0);

        for mode in ["disabled", "preferred"] {
            let outcome = transport(server, mode);
            assert_eq!(outcome.code, Some(0), "{name} {mode}: {}", outcome.err);
            assert!(
                outcome.out.contains("using TCP/IP"),
                "{name} {mode}: {}",
                outcome.out
            );
        }
        for mode in ["required", "verify-ca", "verify-identity"] {
            let sandbox = project(server, root(), schema(), mode);
            let outcome = run(&sandbox, &["schema", "info", "--direct", "--no-cache"]);
            assert_eq!(outcome.code, Some(69), "{name} {mode}: {}", outcome.err);
            assert!(
                line(&outcome.err, "cause:").contains("TLS handshake"),
                "{name} {mode}: {}",
                outcome.err
            );
            // The hint of a `mysql` entry states that the server must offer
            // TLS, and carries no command setting `preferred` or `disabled`.
            let hint = line(&outcome.err, "hint:");
            assert_eq!(
                hint,
                "hint:  check that the server offers TLS, and enable TLS on the server where it does not",
                "{name} {mode}: {}",
                outcome.err
            );
            for word in ["preferred", "disabled", "tpl "] {
                assert!(!hint.contains(word), "{name} {mode}: {}", outcome.err);
            }
        }
    }
}

#[test]
fn fr_mysql_045_an_uncached_login_over_an_unencrypted_session_ends_with_78() {
    // FR-MYSQL-045: tpl never requests the server's public key over a session
    // TLS does not protect, and a login that needs it ends with 78 before the
    // read-only session is set. The cache is emptied by the harness first.
    let _exclusive = mysql::exclusive();
    let test = "fr_mysql_045_an_uncached_login_over_an_unencrypted_session_ends_with_78";
    let (Some(tls), Some(plain)) = (mysql::series(test), mysql::servers(test, Role::NoTls)) else {
        return;
    };

    let cases = tls
        .iter()
        .map(|server| (server, "disabled", false))
        .chain(plain.iter().map(|server| (server, "disabled", false)))
        .chain(plain.iter().map(|server| (server, "preferred", true)));

    for (server, mode, offered_none) in cases {
        let name = server.name();
        let (user, _) = reader();
        let sandbox = project(server, reader(), schema(), mode);

        mysql::flush_authentication_cache(server);
        mysql::statements_on(server);
        let outcome = run(&sandbox, &["schema", "info", "--direct", "--no-cache"]);
        mysql::statements_off(server);

        assert_eq!(outcome.code, Some(78), "{name} {mode}: {}", outcome.err);
        assert!(
            line(&outcome.err, "error:").contains(&format!("cannot authenticate user '{user}'")),
            "{name} {mode}: {}",
            outcome.err
        );
        let cause = line(&outcome.err, "cause:");
        for fact in [
            format!("{}:{}", server.host(), server.port()),
            format!("user '{user}'"),
            format!("under tls '{mode}'"),
            "caching_sha2_password".to_owned(),
            "not completed".to_owned(),
            "does not request that key".to_owned(),
        ] {
            assert!(
                cause.contains(&fact),
                "{name} {mode}: {fact} not in {}",
                outcome.err
            );
        }
        assert_eq!(
            cause.contains("offered no TLS"),
            offered_none,
            "{name} {mode}"
        );
        let hint = line(&outcome.err, "hint:");
        assert!(
            hint.contains(&format!(
                "tpl cfg database update {ENTRY} --tls verify-identity"
            )),
            "{name} {mode}: {}",
            outcome.err
        );
        assert_eq!(
            hint.contains("must offer it"),
            offered_none,
            "{name} {mode}"
        );

        // Raised at the authentication: no statement reached the server.
        let statements: Vec<String> = mysql::received(server, user)
            .into_iter()
            .filter(|(kind, _)| kind != "Connect" && kind != "Quit")
            .map(|(_, statement)| statement)
            .collect();
        assert!(statements.is_empty(), "{name} {mode}: {statements:?}");

        // The same condition ends `tpl cfg database test` at its first step.
        mysql::flush_authentication_cache(server);
        let tested = run(&sandbox, &["cfg", "database", "test", ENTRY]);
        assert_eq!(tested.code, Some(78), "{name} {mode}: {}", tested.err);
    }

    // The accepted cost: once the server holds a cached login, the same entry
    // under `disabled` authenticates, with nothing in `.tpl/.cfg` changed.
    for server in &tls {
        mysql::flush_authentication_cache(server);
        let sandbox = project(server, reader(), schema(), "disabled");
        assert_eq!(
            run(&sandbox, &["schema", "info", "--direct", "--no-cache"]).code,
            Some(78)
        );
        succeeds(
            &project(server, reader(), schema(), "verify-identity"),
            &["schema", "info", "--direct", "--no-cache"],
        );
        succeeds(&sandbox, &["schema", "info", "--direct", "--no-cache"]);
    }
}

// ------------------------------------------------------ requirements of form ---

#[test]
fn fr_mysql_042_the_statement_count_is_the_same_over_freight_freight_empty_and_a_named_read() {
    // FR-MYSQL-042 with NFR-PERF-001, NFR-PERF-002 and NFR-PERF-008: the count
    // is taken from the server's record and from the process's own diagnostic
    // lines, which agree.
    let _exclusive = mysql::exclusive();
    let Some(series) = mysql::series(
        "fr_mysql_042_the_statement_count_is_the_same_over_freight_freight_empty_and_a_named_read",
    ) else {
        return;
    };
    let empty = mysql::names().empty_schema.as_str();

    for server in &series {
        for (database, invocation) in [
            (
                schema(),
                vec!["schema", "dump", "--direct", "--no-cache", "-v"],
            ),
            (
                empty,
                vec!["schema", "dump", "--direct", "--no-cache", "-v"],
            ),
            (
                schema(),
                vec!["schema", "table", "charge", "--direct", "--no-cache", "-v"],
            ),
        ] {
            let sandbox = project(server, root(), database, "verify-identity");

            mysql::statements_on(server);
            let outcome = run(&sandbox, &invocation);
            mysql::statements_off(server);
            assert_eq!(outcome.code, Some(0), "{}", outcome.err);

            let received = mysql::statements_count(server, &["--user", root().0, "--catalogue"]);
            let reported = outcome
                .err
                .lines()
                .filter(|written| written.starts_with(QUERY_TOKEN))
                .count();

            assert_eq!(
                received,
                FULL_READ,
                "{} {database} {invocation:?}",
                server.name()
            );
            assert_eq!(reported as i64, received, "{} {database}", server.name());
        }
    }
}

#[test]
fn fr_mysql_042_an_invocation_opens_one_connection_and_a_cache_hit_opens_none() {
    // FR-MYSQL-042 with NFR-PERF-003, NFR-PERF-004 and NFR-PERF-006, from the
    // server's own connection record; the control comes first.
    let _exclusive = mysql::exclusive();
    let Some(series) =
        mysql::series("fr_mysql_042_an_invocation_opens_one_connection_and_a_cache_hit_opens_none")
    else {
        return;
    };

    for server in &series {
        let name = server.name();
        let control = mysql::connections_attributable_to(server, || mysql::connect_once(server));
        assert_eq!(control, 1, "{name}: the record counted nothing");

        let sandbox = project(server, root(), schema(), "verify-identity");

        for invocation in [
            vec!["schema", "dump", "--direct", "--no-cache"],
            vec!["schema", "table", "charge", "--direct", "--no-cache"],
            vec!["cfg", "database", "test", ENTRY],
        ] {
            let opened = mysql::connections_attributable_to(server, || {
                succeeds(&sandbox, &invocation);
            });
            assert_eq!(opened, 1, "{name}: {invocation:?} opened {opened}");
        }

        // NFR-PERF-003: the miss warms the store, and the hit opens nothing.
        succeeds(&sandbox, &["schema", "tables", "--format", "json"]);
        let hit = mysql::connections_attributable_to(server, || {
            let served = succeeds(&sandbox, &["schema", "tables", "--format", "json"]);
            assert!(served.contains("\"source\":\"cache\""), "{name}: {served}");
        });
        assert_eq!(hit, 0, "{name}: a cache hit opened {hit}");

        // NFR-PERF-006: a command that needs no catalogue opens none.
        let none = mysql::connections_attributable_to(server, || {
            succeeds(&sandbox, &["cfg", "database", "list"]);
            succeeds(&sandbox, &["cfg", "database", "show", ENTRY]);
        });
        assert_eq!(none, 0, "{name}: {none}");
    }
}

// ----------------------------------------------------------- FR-PROJ-021 ---

#[test]
fn fr_proj_021_the_starter_template_renders_against_every_table_of_the_fixture() {
    let _exclusive = mysql::exclusive();
    let Some(series) = mysql::series(
        "fr_proj_021_the_starter_template_renders_against_every_table_of_the_fixture",
    ) else {
        return;
    };

    for server in &series {
        let sandbox = Sandbox::new();
        succeeds(&sandbox, &["init"]);
        sandbox.write(
            ".tpl/.cfg",
            &mysql::configuration(server, ENTRY, schema(), root(), "verify-identity"),
        );

        let tables: Vec<String> = names_of(&dump(&sandbox)["tables"]).into_iter().collect();
        assert_eq!(tables.len(), 22, "{}", server.name());

        for table in &tables {
            let rendered = succeeds(&sandbox, &["render", "example", "--table", table]);
            assert!(
                rendered.contains("pub struct"),
                "{} {table}: {rendered}",
                server.name()
            );
        }
    }
}

// ----------------------------------------------- the wrong product, both ways ---

#[test]
fn fr_mysql_008_a_mysql_entry_on_a_mariadb_server_is_refused_with_78() {
    // FR-MYSQL-043 assigns this refusal to a MariaDB server of
    // `scripts/mariadb/`, of a series whose read-back a `mysql` entry passes;
    // FR-MYSQL-012 records that 10.11 is refused at the read-back instead.
    let _exclusive = mariadb_fixture::exclusive();
    let Some(mariadb) = mariadb_fixture::series(
        "fr_mysql_008_a_mysql_entry_on_a_mariadb_server_is_refused_with_78",
    ) else {
        return;
    };

    for server in mariadb {
        let sandbox = Sandbox::new();
        let configuration =
            mariadb_fixture::configuration(server, ENTRY, "freight", ("root", "tpl-root")).replace(
                &format!("[database.{ENTRY}]\n"),
                &format!("[database.{ENTRY}]\nprovider = \"mysql\"\n"),
            );
        sandbox.project(&configuration);

        let outcome = run(&sandbox, &["schema", "info", "--direct", "--no-cache"]);
        assert_eq!(outcome.code, Some(78), "{}: {}", server.name(), outcome.err);

        let cause = line(&outcome.err, "cause:");
        if server.name().starts_with("10.") {
            assert!(cause.contains("transaction_read_only"), "{}", outcome.err);
        } else {
            assert!(
                line(&outcome.err, "error:").contains("is not MySQL"),
                "{}",
                outcome.err
            );
            assert!(
                cause.contains("MariaDB") && cause.contains("mysql entry"),
                "{}",
                outcome.err
            );
            assert!(
                line(&outcome.err, "hint:").contains("--host"),
                "{}",
                outcome.err
            );
        }
    }
}

#[test]
fn fr_mysql_012_a_mariadb_entry_on_a_mysql_server_is_refused_at_its_read_back() {
    // FR-MYSQL-012, first row: server-contract.md unchanged. MySQL has no
    // `tx_read_only`, so a `mariadb` entry is refused at its read-back.
    let _exclusive = mysql::exclusive();
    let Some(series) =
        mysql::series("fr_mysql_012_a_mariadb_entry_on_a_mysql_server_is_refused_at_its_read_back")
    else {
        return;
    };

    for server in &series {
        let sandbox = Sandbox::new();
        sandbox.project(
            &mysql::configuration(server, ENTRY, schema(), root(), "verify-identity")
                .replace("provider = \"mysql\"\n", ""),
        );

        let outcome = run(&sandbox, &["schema", "info", "--direct", "--no-cache"]);
        assert_eq!(outcome.code, Some(78), "{}: {}", server.name(), outcome.err);
        assert!(
            line(&outcome.err, "cause:").contains("@@session.tx_read_only"),
            "{}",
            outcome.err
        );
    }
}

// --------------------------------- cfg database test, --context, the families ---

#[test]
fn fr_cfg_024_cfg_database_test_reports_a_mysql_server_and_its_probe() {
    let _exclusive = mysql::exclusive();
    let Some(series) =
        mysql::series("fr_cfg_024_cfg_database_test_reports_a_mysql_server_and_its_probe")
    else {
        return;
    };

    for server in &series {
        for (account, database, readable) in [
            (root(), schema(), true),
            (reader(), schema(), true),
            (root(), "no_such_database", false),
        ] {
            let sandbox = project(server, account, database, "verify-identity");
            let printed = succeeds(
                &sandbox,
                &["cfg", "database", "test", ENTRY, "--format", "json"],
            );
            let report: Value = serde_json::from_str(&printed).expect("a JSON document");
            let data = &report["data"];

            assert_eq!(data["connected"], true, "{printed}");
            assert_eq!(data["read_only_session"], true, "{printed}");
            assert_eq!(data["server"]["series"], server.series(), "{printed}");
            assert_eq!(data["can_read_catalogue"], readable, "{printed}");
        }
    }
}

#[test]
fn fr_ctx_043_and_fr_env_049_a_mysql_dump_renders_through_context_and_json_is_textual() {
    let _exclusive = mysql::exclusive();
    let Some(series) = mysql::series(
        "fr_ctx_043_and_fr_env_049_a_mysql_dump_renders_through_context_and_json_is_textual",
    ) else {
        return;
    };
    let Some(server) = series.first() else {
        return;
    };

    let sandbox = project(server, root(), schema(), "verify-identity");
    sandbox.write(
        ".tpl/templates/textual.jinja",
        "{{ database.provider }}:{% for c in table.columns %}{% if c is textual %} {{ c.name }}{% endif %}{% endfor %}\n",
    );

    let live = succeeds(&sandbox, &["render", "textual", "--table", "customer"]);
    assert!(live.starts_with("mysql:"), "{live}");
    assert!(
        live.split_whitespace().any(|word| word == "preferences"),
        "{live}"
    );

    let document = succeeds(&sandbox, &["schema", "dump", "--direct", "--no-cache"]);
    sandbox.write("context.json", &document);
    let from_document = succeeds(
        &sandbox,
        &[
            "render",
            "textual",
            "--context",
            "context.json",
            "--table",
            "customer",
        ],
    );
    assert_eq!(from_document, live);
}

// ------------------------------------------- FR-MYSQL-048 and ADR-015 ---

/// The target database of item 17 of `FR-MYSQL-044`, as the server names it.
const QUOTED_TARGET: &str = "tpl_q'b\\s`k";

/// [`QUOTED_TARGET`] as a TOML basic string holds it, for `.tpl/.cfg`.
const QUOTED_TARGET_TOML: &str = "tpl_q'b\\\\s`k";

/// The literal of `ADR-015` that stands for [`QUOTED_TARGET`]: its bytes, as
/// `scripts/mysql/README.md` lists them.
const QUOTED_TARGET_LITERAL: &str = "_utf8mb4 X'74706C5F7127625C73606B'";

/// The tables and the view of the target, and nothing else.
const QUOTED_TABLES: [&str; 3] = ["consignee", "tariff'line`%", "tariff'line`2026"];
const QUOTED_VIEWS: [&str; 1] = ["v_consignee_by_place"];

/// The one table of each of the four decoys of item 17.
const DECOY_TABLES: [&str; 4] = [
    "decoy_escape_consumed",
    "decoy_backslash_doubled",
    "decoy_like_pattern",
    "decoy_letter_case",
];

/// A set of the names in `names`.
fn set_of(names: &[&str]) -> BTreeSet<String> {
    names.iter().map(|name| (*name).to_owned()).collect()
}

#[test]
fn fr_mysql_048_a_name_holding_a_quote_a_backslash_and_a_backtick_reads_that_database_alone() {
    // FR-MYSQL-048 applies FR-SRV-043, and ADR-015 owes this test: the target of
    // item 17 reads whole, as the privileged account and as the reduced-grant
    // reader, with not one object of the four decoys whose names an escape, a
    // doubled backslash, a pattern or a letter case would have read instead;
    // `cfg database test` reaches it; and the table whose name holds `%` reads
    // as itself, not as the table its name matches as a pattern.
    let _exclusive = mysql::exclusive();
    let Some(series) = mysql::series(
        "fr_mysql_048_a_name_holding_a_quote_a_backslash_and_a_backtick_reads_that_database_alone",
    ) else {
        return;
    };

    for server in &series {
        let name = server.name();
        for account in [root(), reader()] {
            let who = account.0;
            let sandbox = project(server, account, QUOTED_TARGET_TOML, "verify-identity");
            let database = dump(&sandbox);
            let written = database.to_string();

            assert_eq!(database["name"], QUOTED_TARGET, "{name} {who}");
            assert_eq!(
                names_of(&database["tables"]),
                set_of(&QUOTED_TABLES),
                "{name} {who}"
            );
            assert_eq!(
                names_of(&database["views"]),
                set_of(&QUOTED_VIEWS),
                "{name} {who}"
            );
            assert_eq!(
                names_of(&database["routines"]),
                BTreeSet::new(),
                "{name} {who}"
            );
            for decoy in DECOY_TABLES {
                assert!(!written.contains(decoy), "{name} {who}: {decoy} was read");
            }

            let printed = succeeds(
                &sandbox,
                &["cfg", "database", "test", ENTRY, "--format", "json"],
            );
            let report: Value = serde_json::from_str(&printed).expect("a JSON document");
            assert_eq!(
                report["data"]["can_read_catalogue"], true,
                "{name} {who}: {printed}"
            );

            let printed = succeeds(
                &sandbox,
                &[
                    "schema",
                    "table",
                    "tariff'line`%",
                    "--format",
                    "json",
                    "--direct",
                    "--no-cache",
                ],
            );
            let table: Value = serde_json::from_str(&printed).expect("a JSON document");
            let table = &table["data"]["table"];
            assert_eq!(table["name"], "tariff'line`%", "{name} {who}");
            assert!(
                table["comment"]
                    .as_str()
                    .is_some_and(|comment| comment.contains("percent sign")),
                "{name} {who}: {table}"
            );
            assert!(!printed.contains("2026"), "{name} {who}: {printed}");
        }
    }
}

#[test]
fn fr_priv_021_a_missing_name_holding_a_quote_a_backslash_and_a_percent_ends_with_77() {
    // FR-PRIV-021 through FR-SRV-043: a name no database carries returns no
    // schema row, whatever its characters would mean as syntax or as a
    // pattern, and `tpl_q'b\s%` matches the target as a LIKE pattern would.
    let _exclusive = mysql::exclusive();
    let Some(series) = mysql::series(
        "fr_priv_021_a_missing_name_holding_a_quote_a_backslash_and_a_percent_ends_with_77",
    ) else {
        return;
    };

    for server in &series {
        for account in [root(), reader()] {
            let sandbox = project(server, account, "tpl_q'b\\\\s%", "verify-identity");
            let outcome = run(&sandbox, &["schema", "dump", "--direct", "--no-cache"]);

            assert_eq!(
                outcome.code,
                Some(77),
                "{} {}: {}",
                server.name(),
                account.0,
                outcome.err
            );
            assert_eq!(outcome.out, "", "{} {}", server.name(), account.0);
        }
    }
}

#[test]
fn fr_mysql_048_each_statement_reaches_the_server_with_the_literal_in_its_placeholder() {
    // ADR-015, from the server's own record: a full read of the target is the
    // twelve statements, each one `Query` of the text protocol — no Prepare and
    // no Execute — each carrying the literal of the target's bytes once, and
    // each, with the literal put back as `?`, one of twelve distinct texts whose
    // one placeholder is the right-hand side of `=`.
    let _exclusive = mysql::exclusive();
    let Some(series) = mysql::series(
        "fr_mysql_048_each_statement_reaches_the_server_with_the_literal_in_its_placeholder",
    ) else {
        return;
    };

    for server in &series {
        let name = server.name();
        let sandbox = project(server, root(), QUOTED_TARGET_TOML, "verify-identity");

        mysql::statements_on(server);
        succeeds(&sandbox, &["schema", "dump", "--direct", "--no-cache"]);
        mysql::statements_off(server);

        let rows = mysql::received(server, root().0);
        let catalogue: Vec<&(String, String)> = rows
            .iter()
            .filter(|(_, statement)| statement.contains("INFORMATION_SCHEMA"))
            .collect();
        let mut templates = BTreeSet::new();

        for (kind, statement) in &catalogue {
            assert_eq!(kind, "Query", "{name}: {statement}");
            assert_eq!(
                statement.matches(QUOTED_TARGET_LITERAL).count(),
                1,
                "{name}: {statement}"
            );
            let template = statement.replace(QUOTED_TARGET_LITERAL, "?");
            assert_eq!(template.matches('?').count(), 1, "{name}: {template}");
            assert!(template.contains(" = ?"), "{name}: {template}");
            assert!(!statement.contains(';'), "{name}: {statement}");
            templates.insert(template);
        }

        assert_eq!(i64::try_from(catalogue.len()), Ok(FULL_READ), "{name}");
        assert_eq!(i64::try_from(templates.len()), Ok(FULL_READ), "{name}");
        assert_eq!(
            mysql::statements_count(server, &["--user", root().0, "--catalogue"]),
            FULL_READ,
            "{name}"
        );
    }
}
