//! The first arm and the store beneath it, observed from outside the process.
//!
//! Two tasks meet here: the eight `schema` subcommands of `FR-SCH-001` through
//! `FR-SCH-036`, and the catalogue cache of `FR-CACHE-001` through
//! `FR-CACHE-037` with the documents `FR-CDOC-001` through `FR-CDOC-016` fix.
//! Every body runs the **distributed binary** against the fixture of
//! `scripts/mariadb/`, so what is asserted is what a caller receives: an exit
//! code, the bytes of stdout, and the files left on disk.
//!
//! | Requirement | The property this file establishes |
//! |---|---|
//! | `FR-SCH-002` | All eight subcommands answer, on every series of `FR-SRV-015`, in both formats where both are declared |
//! | `FR-SCH-010` | A named object that does not exist is `66`, with the nearest-match suggestion of `FR-ERR-019` and the wording of `FR-ERR-037` |
//! | `FR-SCH-008` | A qualified routine prefix in the wrong case is `64`, with the same invocation corrected |
//! | `FR-SCH-013` | No pattern reaches the server, observed in the server's own statement record |
//! | `FR-SCH-019`, `FR-SCH-021` | `tpl schema dump` declares no `--format` and refuses `--pattern` |
//! | `FR-SCH-035` | `source` is `server` or `cache` according to what served the read |
//! | `FR-CACHE-006`, `NFR-PERF-003` | A read served from the cache opens no connection, observed in the server's own connection record |
//! | `FR-CACHE-007` | A miss writes before it answers |
//! | `FR-CACHE-016` | `--direct --no-cache` touches no file of the store |
//! | `FR-CACHE-033`, `FR-CDOC-004` | An unreadable file and an unknown version are each a miss and not an error |
//! | `FR-CACHE-025`, `FR-CACHE-035` | `tpl cache status` reports the entry, the load time and the counts, and exits `0` on an empty store |
//! | `FR-CACHE-037` | An object marked `restricted` is never written, and its collection is not recorded whole |
//! | `FR-CDOC-007`, `FR-CDOC-008` | A listing is served only from a whole collection; an individual object is served whenever it is present |
//! | `FR-CDOC-012` | `loaded_at` appears in no read output |
//! | `FR-CAT-054` | A table carries its engine and its collation, and a view carries neither field |
//! | `BR-SCH-001` | A pattern selects the same set from a live read and from a cached one |
//!
//! # Every negative assertion carries a control
//!
//! On the terms `outside_the_process` states: an absence is worth nothing until
//! the same instrument is shown to observe the presence. The connection record
//! is shown to count a connection that was made before it is asked to report
//! none; the statement record is shown to hold the observer's own rows before
//! its silence about a pattern means anything.
//!
//! # The fixture, and what happens without one
//!
//! Every server-dependent body is gated on `scripts/mariadb/status.sh`, whose
//! exit code is three-valued: `0` runs the body, `1` skips it with a printed
//! reason, and `2` — half a fixture — fails the run rather than skipping over
//! it. The bodies that need a server are driven **once per series of
//! `FR-SRV-015`**, which `FR-SRV-029` requires of a test of cross-series
//! behaviour.

#[path = "support/fixture.rs"]
mod fixture;
#[path = "support/sandbox.rs"]
mod sandbox;

use fixture::Server;
use sandbox::Sandbox;

/// The schema every fixture server carries.
///
/// It is written here for the reason `src/mariadb/catalogue.rs` writes it: the
/// harness publishes an address per server and does not publish the schema, and
/// `scripts/mariadb/README.md` documents this name beside the credentials.
const SCHEMA: &str = "freight";

/// The database entry every project below defines.
const ENTRY: &str = "fixture";

/// The privileged account of the fixture, which reads every property.
const ROOT: (&str, &str) = ("root", "tpl-root");

/// The reduced-grant reader of `FR-PRIV-018`, which holds
/// `SELECT, EXECUTE ON freight.*` and nothing else.
const REDUCED: (&str, &str) = ("tpl_reader", "tpl-reader-pw");

/// A table the fixture carries, for the bodies that name one.
const TABLE: &str = "charge";

/// A view the fixture carries.
const VIEW: &str = "v_port_directory";

/// A function the fixture carries.
const FUNCTION: &str = "fn_locode_country";

/// A procedure the fixture carries.
const PROCEDURE: &str = "sp_book_consignment";

/// The eleven statements a full catalogue read issues (`NFR-PERF-001`).
const FULL_READ: i64 = 11;

/// The leading token of the one diagnostic line per catalogue query.
///
/// `FR-GLOB-017` constrains the existence of the line and its
/// distinguishability, not its wording, and `NFR-DET-001` puts stderr outside
/// the contract — so this reads the token the emitter writes and nothing after
/// it.
const QUERY_TOKEN: &str = "query:";

/// The `.tpl/.cfg` a project below carries, reaching `address` as `account`.
///
/// The transport is `disabled` because it is not this file's subject: the five
/// modes of `FR-CONF-013` are exercised where the connection is made, and an
/// entry here asks for the one that adds nothing to what is under test.
fn configuration(server: &Server, account: (&str, &str)) -> String {
    fixture::configuration(server, ENTRY, SCHEMA, account)
}

/// A sandbox holding a project that reaches `server` as `account`.
fn project(server: &Server, account: (&str, &str)) -> Sandbox {
    let sandbox = Sandbox::new();
    sandbox.project(&configuration(server, account));

    sandbox
}

/// What one invocation produced: its code, its stdout and its stderr.
struct Outcome {
    /// The exit status, or `None` where the process was signalled.
    code: Option<i32>,
    /// Standard output, as text.
    out: String,
    /// Standard error, as text.
    err: String,
}

/// Runs `tpl` in `sandbox` and reads all three channels.
fn run(sandbox: &Sandbox, arguments: &[&str]) -> Outcome {
    let printed = sandbox.run(arguments);

    Outcome {
        code: printed.status.code(),
        out: String::from_utf8_lossy(&printed.stdout).into_owned(),
        err: String::from_utf8_lossy(&printed.stderr).into_owned(),
    }
}

/// Runs `tpl` and refuses anything but exit `0`.
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

/// The document `arguments` produced, parsed.
fn document(sandbox: &Sandbox, arguments: &[&str]) -> serde_json::Value {
    let out = succeeds(sandbox, arguments);

    serde_json::from_str(&out).unwrap_or_else(|failure| {
        panic!("tpl {} did not emit JSON: {failure}", arguments.join(" "))
    })
}

/// One labelled line of a diagnostic, without its label.
fn line(rendered: &str, label: &str) -> String {
    rendered
        .lines()
        .find(|written| written.starts_with(label))
        .map(|written| written[label.len()..].trim_start().to_owned())
        .unwrap_or_else(|| panic!("{rendered:?} carries no {label} line"))
}

/// The store of the one entry every project here defines.
fn store(sandbox: &Sandbox) -> std::path::PathBuf {
    sandbox.path(&format!(".tpl/.cache/{ENTRY}"))
}

// ---------------------------------------------------- the eight subcommands ---

#[test]
fn fr_sch_002_every_subcommand_answers_on_every_series_in_both_formats() {
    // FR-SCH-002 with FR-SRV-029: the eight subcommands, once per series of
    // FR-SRV-015. FR-SCH-023 gives seven of them `--format text|json` and
    // FR-SCH-019 withholds `--format` from `dump`, which emits JSON and
    // nothing else — so the eighth is driven once and the other seven twice.
    let _guard = fixture::exclusive();
    let Some(series) =
        fixture::series("fr_sch_002_every_subcommand_answers_on_every_series_in_both_formats")
    else {
        return;
    };

    for server in series {
        let sandbox = project(server, ROOT);

        for arguments in [
            &["schema", "info"][..],
            &["schema", "tables"][..],
            &["schema", "views"][..],
            &["schema", "routines"][..],
            &["schema", "table", TABLE][..],
            &["schema", "view", VIEW][..],
            &["schema", "routine", FUNCTION][..],
        ] {
            let text = succeeds(&sandbox, arguments);
            assert!(
                !text.is_empty() && text.ends_with('\n'),
                "{}: tpl {} wrote {text:?}",
                server.name(),
                arguments.join(" ")
            );

            let mut json = arguments.to_vec();
            json.extend_from_slice(&["--format", "json"]);
            let enveloped = document(&sandbox, &json);

            // FR-OUT-024 and FR-SCH-030: the same three keys on every one of
            // them, in one order, whichever subcommand answered.
            assert_eq!(enveloped["schema_version"], 1, "{}", arguments.join(" "));
            assert!(
                enveloped["source"] == "server" || enveloped["source"] == "cache",
                "FR-SCH-035: {} answered source {:?}",
                arguments.join(" "),
                enveloped["source"]
            );
            assert!(
                enveloped["data"].is_object(),
                "{} carried no data object",
                arguments.join(" ")
            );
        }

        // FR-SCH-016 and FR-SCH-017: the whole database as one JSON document,
        // in the envelope, carrying a `data` of one key.
        let dumped = document(&sandbox, &["schema", "dump"]);
        assert!(dumped["data"]["database"]["tables"].is_array());
        assert_eq!(
            dumped["data"].as_object().map(serde_json::Map::len),
            Some(1),
            "FR-SCH-034 gives the dump one data key"
        );

        // FR-SCH-018: the dump carries the server-derived part alone.
        for absent in ["vars", "tpl", "now"] {
            assert!(
                dumped["data"][absent].is_null(),
                "FR-SCH-018 keeps {absent} out of the dump"
            );
        }
    }
}

#[test]
fn fr_sch_032_a_listing_carries_one_key_named_for_its_collection_in_the_plural() {
    // FR-SCH-032 and FR-OUT-030, and FR-SCH-033 and FR-OUT-031 beside them:
    // the plural for a collection, the singular for one named object.
    let _guard = fixture::exclusive();
    let Some(series) = fixture::series(
        "fr_sch_032_a_listing_carries_one_key_named_for_its_collection_in_the_plural",
    ) else {
        return;
    };
    let Some(server) = series.first() else {
        return;
    };
    let sandbox = project(server, ROOT);

    for (arguments, key, plural) in [
        (&["schema", "tables"][..], "tables", true),
        (&["schema", "views"][..], "views", true),
        (&["schema", "routines"][..], "routines", true),
        (&["schema", "table", TABLE][..], "table", false),
        (&["schema", "view", VIEW][..], "view", false),
        (&["schema", "routine", FUNCTION][..], "routine", false),
        (&["schema", "info"][..], "database", false),
    ] {
        let mut json = arguments.to_vec();
        json.extend_from_slice(&["--format", "json"]);
        let enveloped = document(&sandbox, &json);
        let data = &enveloped["data"];

        assert_eq!(
            data.as_object().map(serde_json::Map::len),
            Some(1),
            "{} carries more than one data key",
            arguments.join(" ")
        );
        assert_eq!(data[key].is_array(), plural, "{}", arguments.join(" "));
    }
}

#[test]
fn fr_cat_054_a_table_carries_its_engine_and_collation_and_a_view_carries_neither() {
    // FR-CAT-054 and FR-SCH-009: the two fields are read from the table
    // catalogue and carried as the server returns them. A view takes neither —
    // both are SQL NULL on the row a view has in that catalogue — and
    // FR-CAT-003 keeps a view out of the tables collection, so a view object
    // carries no such field at all.
    let _guard = fixture::exclusive();
    let Some(series) = fixture::series(
        "fr_cat_054_a_table_carries_its_engine_and_collation_and_a_view_carries_neither",
    ) else {
        return;
    };

    for server in series {
        let sandbox = project(server, ROOT);

        let table = document(&sandbox, &["schema", "table", TABLE, "--format", "json"]);
        assert_eq!(
            table["data"]["table"]["engine"],
            "InnoDB",
            "{}",
            server.name()
        );
        assert!(
            table["data"]["table"]["collation"].is_string(),
            "{}: the collation is absent",
            server.name()
        );

        let view = document(&sandbox, &["schema", "view", VIEW, "--format", "json"]);
        let fields = view["data"]["view"]
            .as_object()
            .expect("a view is an object");
        assert!(
            !fields.contains_key("engine") && !fields.contains_key("collation"),
            "{}: a view carries {fields:?}",
            server.name()
        );

        // And the `text` form prints both, which is what FR-SCH-009 obliges.
        let printed = succeeds(&sandbox, &["schema", "table", TABLE]);
        assert!(printed.contains("engine"), "{printed}");
        assert!(printed.contains("collation"), "{printed}");
    }
}

#[test]
fn nfr_perf_001_a_full_read_issues_eleven_catalogue_statements() {
    // NFR-PERF-001, observed in the server's own statement record rather than
    // read from the source: every read of this arm is a whole read, and a whole
    // read is eleven statements against INFORMATION_SCHEMA — the count
    // `src/mariadb/catalogue/statements.rs` fixes in the plan, arriving at the
    // server.
    let _guard = fixture::exclusive();
    let Some(series) =
        fixture::series("nfr_perf_001_a_full_read_issues_eleven_catalogue_statements")
    else {
        return;
    };

    for server in series {
        let sandbox = project(server, ROOT);

        fixture::statements_on(server);
        succeeds(
            &sandbox,
            &["schema", "table", TABLE, "--direct", "--no-cache"],
        );
        fixture::statements_off(server);

        // The driver prepares each statement and executes it, so a catalogue
        // statement reaches the record as an `Execute` row rather than as the
        // `Query` row `observe.sh --catalogue` looks for. The count is taken
        // over the rows themselves, which the instrument prints.
        let executed = fixture::statements_text(server, &["--user", ROOT.0, "--kind", "Execute"]);
        let issued = executed
            .lines()
            .filter(|row| row.contains("INFORMATION_SCHEMA"))
            .count();

        assert_eq!(
            issued as i64,
            FULL_READ,
            "{} received {issued} catalogue statement(s) for one named table:\n{executed}",
            server.name()
        );
    }
}

// ------------------------------------------------------------- the pattern ---

#[test]
fn fr_sch_013_no_pattern_reaches_the_server() {
    // FR-SCH-013: the pattern is evaluated in memory, against the names
    // already read, and is never sent. The assertion is made where the
    // requirement is stated — at the server — and it carries its control: the
    // same dump is shown to hold the statements the read did issue.
    let _guard = fixture::exclusive();
    let Some(series) = fixture::series("fr_sch_013_no_pattern_reaches_the_server") else {
        return;
    };

    for server in series {
        let sandbox = project(server, ROOT);

        fixture::statements_on(server);
        succeeds(
            &sandbox,
            &[
                "schema",
                "tables",
                "--pattern",
                "cont%",
                "--direct",
                "--no-cache",
            ],
        );
        fixture::statements_off(server);

        let received = fixture::statements_text(server, &["--user", ROOT.0]);

        // The control: the window held what the read issued, so its silence
        // about the pattern is an observation rather than an empty log.
        assert!(
            received.contains("INFORMATION_SCHEMA"),
            "{}: the statement record held nothing the read issued",
            server.name()
        );
        assert!(
            !received.contains("cont%"),
            "{}: the pattern reached the server: {received}",
            server.name()
        );
        assert!(
            !received.to_uppercase().contains(" LIKE "),
            "{}: a LIKE predicate reached the server: {received}",
            server.name()
        );
    }
}

#[test]
fn br_sch_001_a_pattern_selects_the_same_set_from_a_live_read_and_from_a_cached_one() {
    // BR-SCH-001: a pattern selects the same set whatever the source of the
    // catalogue. The two reads below differ in exactly that, which FR-SCH-035
    // makes observable in the document itself.
    let _guard = fixture::exclusive();
    let Some(series) = fixture::series(
        "br_sch_001_a_pattern_selects_the_same_set_from_a_live_read_and_from_a_cached_one",
    ) else {
        return;
    };
    let Some(server) = series.first() else {
        return;
    };
    let sandbox = project(server, ROOT);

    let live = document(
        &sandbox,
        &[
            "schema",
            "tables",
            "--pattern",
            "c%",
            "--direct",
            "--format",
            "json",
        ],
    );
    let cached = document(
        &sandbox,
        &["schema", "tables", "--pattern", "c%", "--format", "json"],
    );

    assert_eq!(live["source"], "server");
    assert_eq!(cached["source"], "cache");
    assert_eq!(live["data"], cached["data"]);

    // And the set is a proper subset of the whole listing, so the filter did
    // something rather than nothing.
    let whole = document(&sandbox, &["schema", "tables", "--format", "json"]);
    let selected = live["data"]["tables"].as_array().expect("an array").len();
    let held = whole["data"]["tables"].as_array().expect("an array").len();

    assert!(selected > 0 && selected < held, "{selected} of {held}");
}

#[test]
fn fr_sch_014_the_pattern_folds_case_over_ascii_alone() {
    // FR-SCH-014, with the accepted cost the requirement records: `order%`
    // also matches a name that begins with an upper-case letter.
    let _guard = fixture::exclusive();
    let Some(series) = fixture::series("fr_sch_014_the_pattern_folds_case_over_ascii_alone") else {
        return;
    };
    let Some(server) = series.first() else {
        return;
    };
    let sandbox = project(server, ROOT);

    let lower = document(
        &sandbox,
        &["schema", "tables", "--pattern", "cont%", "--format", "json"],
    );
    let upper = document(
        &sandbox,
        &["schema", "tables", "--pattern", "CONT%", "--format", "json"],
    );

    assert_eq!(lower["data"], upper["data"]);
    assert!(
        !lower["data"]["tables"]
            .as_array()
            .expect("an array")
            .is_empty()
    );
}

// ------------------------------------------------------ the named refusals ---

#[test]
fn fr_sch_010_a_named_object_that_does_not_exist_exits_sixty_six_with_a_suggestion() {
    // FR-SCH-010: `66`, with a nearest-match suggestion over the objects of
    // that kind that do exist. FR-ERR-037 fixes the wording where more than one
    // candidate qualifies, and the fixture carries a pair one name is within
    // the distance of both.
    let _guard = fixture::exclusive();
    let Some(series) = fixture::series(
        "fr_sch_010_a_named_object_that_does_not_exist_exits_sixty_six_with_a_suggestion",
    ) else {
        return;
    };
    let Some(server) = series.first() else {
        return;
    };
    let sandbox = project(server, ROOT);

    // One candidate: `charg` is one deletion from `charge`.
    let one = run(&sandbox, &["schema", "table", "charg"]);
    assert_eq!(one.code, Some(66), "{}", one.err);
    assert_eq!(
        line(&one.err, "hint:"),
        format!(
            "did you mean '{TABLE}'? list the available tables with: tpl -d {ENTRY} schema tables"
        )
    );
    assert!(one.out.is_empty(), "FR-ERR-033 leaves stdout empty");

    // FR-ERR-037's wording for two and three candidates is exercised where a
    // population that holds two within the distance of one name exists: no two
    // objects of the reference database are within the two edits FR-ERR-019
    // admits, so the case is a unit test of `cli::schema::named` and of
    // `diagnostics::suggest` rather than a body here.

    // And the other two kinds reach the same code over their own population.
    for arguments in [
        &["schema", "view", "v_port_directoy"][..],
        &["schema", "routine", "fn_locode_countr"][..],
    ] {
        let outcome = run(&sandbox, arguments);

        assert_eq!(outcome.code, Some(66), "{}", outcome.err);
        assert!(
            line(&outcome.err, "hint:").starts_with("did you mean "),
            "{}",
            outcome.err
        );
    }
}

#[test]
fn fr_sch_008_a_qualified_prefix_in_the_wrong_case_exits_sixty_four_with_the_invocation_corrected()
{
    // FR-SCH-008 as the twenty-fifth edition amended it. The token's shape is
    // decidable without a server, so the refusal precedes every catalogue read
    // — which is why this body needs neither a fixture nor an entry.
    let sandbox = Sandbox::new();
    sandbox.project("[core]\n");

    for (written, corrected) in [
        (
            format!("PROCEDURE:{PROCEDURE}"),
            format!("procedure:{PROCEDURE}"),
        ),
        (
            format!("Function:{FUNCTION}"),
            format!("function:{FUNCTION}"),
        ),
    ] {
        let outcome = run(&sandbox, &["schema", "routine", &written]);

        assert_eq!(outcome.code, Some(64), "{}", outcome.err);
        assert!(outcome.out.is_empty(), "FR-ERR-033 leaves stdout empty");
        assert_eq!(
            line(&outcome.err, "hint:"),
            format!("write it as: tpl schema routine {corrected}")
        );
        assert!(
            line(&outcome.err, "cause:").contains(&written),
            "the cause names the token as written: {}",
            outcome.err
        );
    }
}

#[test]
fn fr_sch_019_and_fr_sch_021_the_dump_declares_no_format_and_refuses_a_pattern() {
    // FR-SCH-019 withholds `--format`, FR-SCH-021 withholds `--pattern`, and
    // FR-SCH-020 gives `--pretty` on its own. The first two are the ordinary
    // unknown-flag `64` of FR-CLI-019, which needs no project at all.
    let sandbox = Sandbox::new();
    sandbox.project("[core]\n");

    for refused in [
        &["schema", "dump", "--format", "json"][..],
        &["schema", "dump", "--format", "text"][..],
        &["schema", "dump", "--pattern", "x"][..],
    ] {
        let outcome = run(&sandbox, refused);

        assert_eq!(outcome.code, Some(64), "{}", outcome.err);
    }

    // `--pretty` stands alone on this command, per FR-SCH-020 and FR-OUT-010,
    // so it is not the `64` of FR-OUT-009 that it would be on a command
    // declaring `--format`. Without an entry it reaches the `78` of
    // FR-GLOB-006, which is past the parsing step and is what proves the flag
    // itself was accepted.
    let accepted = run(&sandbox, &["schema", "dump", "--pretty"]);
    assert_eq!(accepted.code, Some(78), "{}", accepted.err);
}

#[test]
fn fr_sch_020_the_dump_indents_when_it_is_asked_to() {
    // FR-SCH-020 and FR-OUT-008: a two-space indent with one key per line.
    let _guard = fixture::exclusive();
    let Some(series) = fixture::series("fr_sch_020_the_dump_indents_when_it_is_asked_to") else {
        return;
    };
    let Some(server) = series.first() else {
        return;
    };
    let sandbox = project(server, ROOT);

    // Both forms are taken from the same source, so that what differs between
    // them is the whitespace and nothing else: FR-SCH-035 would otherwise make
    // the first a `server` document and the second a `cache` one.
    succeeds(&sandbox, &["cache", "load"]);
    let compact = succeeds(&sandbox, &["schema", "dump"]);
    let indented = succeeds(&sandbox, &["schema", "dump", "--pretty"]);

    assert_eq!(compact.matches('\n').count(), 1, "FR-OUT-007");
    assert!(
        indented.starts_with("{\n  \"schema_version\": 1,"),
        "{indented}"
    );

    let one: serde_json::Value = serde_json::from_str(&compact).expect("compact is JSON");
    let other: serde_json::Value = serde_json::from_str(&indented).expect("indented is JSON");
    assert_eq!(one, other, "FR-OUT-016: the two forms are one document");
}

#[test]
fn fr_priv_003_a_named_object_that_came_back_short_exits_seventy_seven() {
    // FR-PRIV-003 and FR-PRIV-004: a named object that came back incomplete is
    // `77`, and is not returned in part. The reduced reader of FR-PRIV-018 is
    // how the shortfall is reproduced rather than simulated.
    let _guard = fixture::exclusive();
    let Some(series) =
        fixture::series("fr_priv_003_a_named_object_that_came_back_short_exits_seventy_seven")
    else {
        return;
    };

    for server in series {
        let sandbox = project(server, REDUCED);
        let outcome = run(&sandbox, &["schema", "view", VIEW]);

        assert_eq!(outcome.code, Some(77), "{}", outcome.err);
        assert!(outcome.out.is_empty(), "FR-PRIV-004: {}", outcome.out);
        assert!(
            line(&outcome.err, "cause:").contains("definition"),
            "FR-PRIV-013 names the property: {}",
            outcome.err
        );

        // FR-PRIV-005: the same object inside a **listing** is answered, and
        // marked.
        let listed = document(&sandbox, &["schema", "views", "--format", "json"]);
        let marked = listed["data"]["views"]
            .as_array()
            .expect("an array")
            .iter()
            .find(|view| view["name"] == VIEW)
            .expect("the view is listed");

        assert_eq!(marked["restricted"][0], "definition");
    }
}

// ----------------------------------------------------------- the read-through ---

#[test]
fn fr_cache_006_a_read_served_from_the_cache_opens_no_connection() {
    // FR-CACHE-006 and NFR-PERF-003, observed in the server's own connection
    // record. The control comes first: an assertion that no connection was
    // opened is worth nothing until this same record is shown to count one that
    // was.
    let _guard = fixture::exclusive();
    let Some(series) =
        fixture::series("fr_cache_006_a_read_served_from_the_cache_opens_no_connection")
    else {
        return;
    };

    for server in series {
        let control =
            fixture::connections_attributable_to(server, || fixture::connect_once(server));
        assert_eq!(
            control,
            1,
            "the connection record of {} did not count a connection that was made",
            server.name()
        );

        let sandbox = project(server, ROOT);

        // The miss, which warms the store. FR-CACHE-007 writes before it
        // answers, and the files are there once the process has exited.
        let warmed = document(&sandbox, &["schema", "tables", "--format", "json"]);
        assert_eq!(warmed["source"], "server");
        assert!(store(&sandbox).join("meta.json").is_file());

        let attributable = fixture::connections_attributable_to(server, || {
            let served = document(&sandbox, &["schema", "tables", "--format", "json"]);

            assert_eq!(served["source"], "cache", "{}", server.name());
            assert_eq!(served["data"], warmed["data"], "{}", server.name());
        });

        assert_eq!(
            attributable,
            0,
            "{} accepted {attributable} connection(s) for a read served from the cache",
            server.name()
        );
    }
}

#[test]
fn fr_cache_016_the_pure_read_touches_no_file_of_the_store() {
    // FR-CACHE-016: `--direct --no-cache` is the pure read, and is the form
    // that works when `.tpl/` is not writable. FR-CACHE-015's other three rows
    // are asserted beside it, because the four together are the table.
    let _guard = fixture::exclusive();
    let Some(series) = fixture::series("fr_cache_016_the_pure_read_touches_no_file_of_the_store")
    else {
        return;
    };
    let Some(server) = series.first() else {
        return;
    };
    let sandbox = project(server, ROOT);

    // FR-CACHE-003: the store appears on the first read that populates it, and
    // `tpl init` never creates it.
    assert!(!store(&sandbox).exists());

    let pure = document(
        &sandbox,
        &[
            "schema",
            "tables",
            "--direct",
            "--no-cache",
            "--format",
            "json",
        ],
    );
    assert_eq!(pure["source"], "server");
    assert!(
        !sandbox.path(".tpl/.cache").exists(),
        "FR-CACHE-016: the pure read wrote {:?}",
        sandbox.path(".tpl/.cache")
    );

    // `--no-cache` alone still reads through the store, and still stores
    // nothing.
    let unstored = document(
        &sandbox,
        &["schema", "tables", "--no-cache", "--format", "json"],
    );
    assert_eq!(unstored["source"], "server");
    assert!(!sandbox.path(".tpl/.cache").exists());

    // Neither flag: the miss writes, and the next read is served from it.
    assert_eq!(
        document(&sandbox, &["schema", "tables", "--format", "json"])["source"],
        "server"
    );
    assert_eq!(
        document(&sandbox, &["schema", "tables", "--format", "json"])["source"],
        "cache"
    );

    // `--direct` alone ignores what is stored and stores what it read.
    assert_eq!(
        document(
            &sandbox,
            &["schema", "tables", "--direct", "--format", "json"]
        )["source"],
        "server"
    );
    assert!(store(&sandbox).join("meta.json").is_file());
}

#[test]
fn fr_cache_033_an_unreadable_file_and_an_unknown_version_are_each_a_miss() {
    // FR-CACHE-033 and FR-CDOC-004: the read goes to the server, the file is
    // rewritten, and neither an error nor a warning is reported.
    let _guard = fixture::exclusive();
    let Some(series) =
        fixture::series("fr_cache_033_an_unreadable_file_and_an_unknown_version_are_each_a_miss")
    else {
        return;
    };
    let Some(server) = series.first() else {
        return;
    };
    let sandbox = project(server, ROOT);

    let whole = document(&sandbox, &["schema", "tables", "--format", "json"]);
    assert_eq!(whole["source"], "server");

    // A file that is not this contract at all.
    std::fs::write(store(&sandbox).join("database.json"), "{ not json").expect("the store is ours");
    let after_damage = run(&sandbox, &["schema", "tables", "--format", "json"]);
    assert_eq!(after_damage.code, Some(0), "{}", after_damage.err);
    assert!(
        after_damage.err.is_empty(),
        "FR-CACHE-033 reports neither an error nor a warning: {}",
        after_damage.err
    );
    let served: serde_json::Value =
        serde_json::from_str(&after_damage.out).expect("the answer is a document");
    assert_eq!(served["source"], "server");
    assert_eq!(served["data"], whole["data"]);

    // A version this binary does not know. `FR-CDOC-004` makes it the same
    // miss, and the file is rewritten rather than reported.
    let meta = store(&sandbox).join("meta.json");
    let written = std::fs::read_to_string(&meta).expect("the store is ours");
    std::fs::write(
        &meta,
        written.replace(r#""cache_format":1"#, r#""cache_format":99"#),
    )
    .expect("the store is ours");

    let after_version = run(&sandbox, &["schema", "tables", "--format", "json"]);
    assert_eq!(after_version.code, Some(0), "{}", after_version.err);
    assert!(after_version.err.is_empty(), "{}", after_version.err);
    assert_eq!(
        serde_json::from_str::<serde_json::Value>(&after_version.out).expect("a document")["source"],
        "server"
    );
    assert!(
        std::fs::read_to_string(&meta)
            .expect("the store is ours")
            .contains(r#""cache_format":1"#),
        "the file was rewritten"
    );
}

#[test]
fn fr_cdoc_007_a_listing_is_served_only_from_a_whole_collection() {
    // FR-CDOC-007 and FR-CDOC-008, and BR-CDOC-002 between them: after
    // `tpl cache load --table`, a listing is a **miss** and the named object is
    // a **hit**. The failure the record exists to prevent is a listing of one
    // table at exit 0.
    let _guard = fixture::exclusive();
    let Some(series) =
        fixture::series("fr_cdoc_007_a_listing_is_served_only_from_a_whole_collection")
    else {
        return;
    };
    let Some(server) = series.first() else {
        return;
    };
    let sandbox = project(server, ROOT);

    succeeds(&sandbox, &["cache", "load", "--table", TABLE]);

    let held: Vec<String> = std::fs::read_dir(store(&sandbox).join("tables"))
        .expect("the store holds a tables directory")
        .map(|entry| {
            entry
                .expect("the store is ours")
                .file_name()
                .to_string_lossy()
                .into_owned()
        })
        .collect();
    assert_eq!(held, [format!("{TABLE}.json")]);

    // FR-CDOC-008: present, so it is served, whatever its collection's record
    // says.
    assert_eq!(
        document(&sandbox, &["schema", "table", TABLE, "--format", "json"])["source"],
        "cache"
    );

    // FR-CDOC-007: the collection is not whole, so the listing is a miss — and
    // the answer it produces is the whole listing rather than the one table.
    let listed = document(&sandbox, &["schema", "tables", "--format", "json"]);
    assert_eq!(listed["source"], "server");
    assert!(
        listed["data"]["tables"].as_array().expect("an array").len() > 1,
        "BR-CDOC-002: the listing answered from one cached table"
    );
}

#[test]
fn fr_cache_037_an_object_marked_restricted_is_never_written() {
    // FR-CACHE-037: the object is not written, and the collection holding it is
    // not recorded whole — so no listing is ever served short.
    let _guard = fixture::exclusive();
    let Some(series) = fixture::series("fr_cache_037_an_object_marked_restricted_is_never_written")
    else {
        return;
    };
    let Some(server) = series.first() else {
        return;
    };
    let sandbox = project(server, REDUCED);

    let listed = document(&sandbox, &["schema", "views", "--format", "json"]);
    assert_eq!(listed["source"], "server");
    assert!(
        !listed["data"]["views"]
            .as_array()
            .expect("an array")
            .is_empty()
    );

    let views = store(&sandbox).join("views");
    let stored = views
        .read_dir()
        .map(|entries| entries.count())
        .unwrap_or_default();
    assert_eq!(stored, 0, "a marked view reached the store");

    // And the record says so, which is what keeps the next listing honest.
    let status = document(&sandbox, &["cache", "status", "--format", "json"]);
    let collections = status["data"]["collections"]
        .as_array()
        .expect("an array")
        .iter()
        .find(|collection| collection["name"] == "views")
        .expect("the record names every collection")
        .clone();

    assert_eq!(collections["count"], 0);
    assert_eq!(collections["whole"], false);
    assert_eq!(
        document(&sandbox, &["schema", "views", "--format", "json"])["source"],
        "server",
        "the listing is a miss for as long as the object is out of reach"
    );
}

#[test]
fn fr_cdoc_012_the_load_time_appears_in_no_read_output() {
    // FR-CDOC-012 and FR-CDOC-013: `loaded_at` appears in `meta.json` and in
    // `tpl cache status`, and nowhere else. BR-CDOC-003 is why — a load time in
    // a read would make two identical invocations produce different bytes.
    let _guard = fixture::exclusive();
    let Some(series) = fixture::series("fr_cdoc_012_the_load_time_appears_in_no_read_output")
    else {
        return;
    };
    let Some(server) = series.first() else {
        return;
    };
    let sandbox = project(server, ROOT);

    succeeds(&sandbox, &["cache", "load"]);

    // The value itself, rather than the key: `loaded_at` is also the name of a
    // column the reference database carries, and a read that presents that
    // column presents its name. What FR-CDOC-012 keeps out of a read is the
    // **load time**, so that is what is looked for.
    let record =
        std::fs::read_to_string(store(&sandbox).join("meta.json")).expect("the store is ours");
    let stored: serde_json::Value =
        serde_json::from_str(&record).expect("the record is a document");
    let at = stored["loaded_at"]
        .as_str()
        .expect("a load time")
        .to_owned();

    for arguments in [
        &["schema", "info", "--format", "json"][..],
        &["schema", "tables", "--format", "json"][..],
        &["schema", "views", "--format", "json"][..],
        &["schema", "routines", "--format", "json"][..],
        &["schema", "table", TABLE, "--format", "json"][..],
        &["schema", "view", VIEW, "--format", "json"][..],
        &["schema", "routine", FUNCTION, "--format", "json"][..],
        &["schema", "dump"][..],
        &["schema", "tables"][..],
        &["schema", "table", TABLE][..],
    ] {
        let out = succeeds(&sandbox, arguments);

        assert!(
            !out.contains(&at),
            "tpl {} carried the load time",
            arguments.join(" ")
        );
    }

    // The two places it does appear, per FR-CDOC-013.
    assert!(record.contains("loaded_at"));
    assert!(succeeds(&sandbox, &["cache", "status"]).contains(&at));
}

// -------------------------------------------------------------- tpl cache ---

#[test]
fn fr_cache_025_status_reports_the_entry_the_load_time_and_the_counts() {
    // FR-CACHE-025 and FR-CACHE-034, and FR-CACHE-026 and FR-CACHE-035 for the
    // empty store: empty is a state, not a failure.
    let _guard = fixture::exclusive();
    let Some(series) =
        fixture::series("fr_cache_025_status_reports_the_entry_the_load_time_and_the_counts")
    else {
        return;
    };
    let Some(server) = series.first() else {
        return;
    };
    let sandbox = project(server, ROOT);

    // FR-CACHE-026 and FR-CACHE-035, before anything is stored.
    let empty = document(&sandbox, &["cache", "status", "--format", "json"]);
    assert_eq!(empty["source"], "project", "FR-CACHE-034");
    assert_eq!(empty["data"]["entry"], ENTRY);
    assert!(empty["data"]["loaded_at"].is_null());
    assert_eq!(
        empty["data"]["collections"].as_array().map(Vec::len),
        Some(0)
    );

    // FR-OUT-034 in the `text` form: the header row, and nothing beneath it.
    let printed = succeeds(&sandbox, &["cache", "status"]);
    assert!(
        printed.contains("COLLECTIONS\nNAME  COUNT  WHOLE\n"),
        "{printed}"
    );

    succeeds(&sandbox, &["cache", "load"]);

    let loaded = document(&sandbox, &["cache", "status", "--format", "json"]);
    assert_eq!(loaded["data"]["entry"], ENTRY);
    assert!(
        loaded["data"]["loaded_at"]
            .as_str()
            .is_some_and(|at| at.ends_with('Z') && at.len() == 20),
        "FR-CACHE-034 fixes the form: {:?}",
        loaded["data"]["loaded_at"]
    );

    let collections = loaded["data"]["collections"].as_array().expect("an array");
    let named: Vec<&str> = collections
        .iter()
        .map(|collection| collection["name"].as_str().expect("a name"))
        .collect();

    assert_eq!(named, ["tables", "views", "routines"]);
    for collection in collections {
        assert!(collection["count"].as_u64().expect("a count") > 0);
        assert_eq!(collection["whole"], true);
    }
}

#[test]
fn fr_cache_022_load_stores_the_whole_catalogue_or_the_one_object_it_was_given() {
    // FR-CACHE-022 and FR-CACHE-024. The command writes nothing to stdout:
    // FR-CACHE-027 gives `--format` to `status` alone, so this one has no
    // representation to answer in and BR-CLI-004 leaves the stream empty.
    let _guard = fixture::exclusive();
    let Some(series) = fixture::series(
        "fr_cache_022_load_stores_the_whole_catalogue_or_the_one_object_it_was_given",
    ) else {
        return;
    };
    let Some(server) = series.first() else {
        return;
    };
    let sandbox = project(server, ROOT);

    assert_eq!(succeeds(&sandbox, &["cache", "load"]), "");
    let whole = document(&sandbox, &["cache", "status", "--format", "json"]);
    for collection in whole["data"]["collections"].as_array().expect("an array") {
        assert_eq!(collection["whole"], true);
    }

    succeeds(&sandbox, &["cache", "clean"]);
    succeeds(&sandbox, &["cache", "load", "--view", VIEW]);
    succeeds(
        &sandbox,
        &[
            "cache",
            "load",
            "--routine",
            &format!("function:{FUNCTION}"),
        ],
    );

    assert!(
        store(&sandbox)
            .join("views")
            .join(format!("{VIEW}.json"))
            .is_file(),
        "the named view was not stored"
    );
    // FR-CDOC-014: the path carries the kind as well as the name, in lower
    // case.
    assert!(
        store(&sandbox)
            .join("routines")
            .join(format!("function.{FUNCTION}.json"))
            .is_file(),
        "the named routine was not stored under its kind"
    );

    // FR-CACHE-018: `--direct` is accepted and ignored.
    succeeds(&sandbox, &["cache", "load", "--direct", "--table", TABLE]);
    assert!(
        store(&sandbox)
            .join("tables")
            .join(format!("{TABLE}.json"))
            .is_file()
    );

    // FR-CACHE-019: `--no-cache` is refused.
    let refused = run(&sandbox, &["cache", "load", "--no-cache"]);
    assert_eq!(refused.code, Some(64), "{}", refused.err);

    // FR-CACHE-024: a name that reaches no object is the `66` every named read
    // produces.
    let absent = run(&sandbox, &["cache", "load", "--table", "no_such_table"]);
    assert_eq!(absent.code, Some(66), "{}", absent.err);

    // And two object flags in one invocation name no individual object.
    let both = run(
        &sandbox,
        &["cache", "load", "--table", TABLE, "--view", VIEW],
    );
    assert_eq!(both.code, Some(64), "{}", both.err);
}

#[test]
fn fr_cache_023_clean_removes_everything_or_the_one_object_it_was_given() {
    // FR-CACHE-023 and FR-CACHE-024, and FR-CACHE-026's reading of an absent
    // object: nothing stored is the state the caller asked for.
    let _guard = fixture::exclusive();
    let Some(series) =
        fixture::series("fr_cache_023_clean_removes_everything_or_the_one_object_it_was_given")
    else {
        return;
    };
    let Some(server) = series.first() else {
        return;
    };
    let sandbox = project(server, ROOT);

    // An empty store: `clean` is a success with nothing to remove.
    assert_eq!(succeeds(&sandbox, &["cache", "clean"]), "");

    succeeds(&sandbox, &["cache", "load"]);
    succeeds(&sandbox, &["cache", "clean", "--table", TABLE]);

    assert!(
        !store(&sandbox)
            .join("tables")
            .join(format!("{TABLE}.json"))
            .exists()
    );
    let after = document(&sandbox, &["cache", "status", "--format", "json"]);
    let tables = after["data"]["collections"]
        .as_array()
        .expect("an array")
        .iter()
        .find(|collection| collection["name"] == "tables")
        .expect("the record names every collection")
        .clone();
    assert_eq!(
        tables["whole"], false,
        "BR-CDOC-002: a collection one object was removed from is no longer whole"
    );

    // A bare routine name that reaches one stored object removes it.
    succeeds(&sandbox, &["cache", "clean", "--routine", FUNCTION]);
    assert!(
        !store(&sandbox)
            .join("routines")
            .join(format!("function.{FUNCTION}.json"))
            .exists()
    );

    // And the whole store goes.
    succeeds(&sandbox, &["cache", "clean"]);
    assert!(!store(&sandbox).exists());
    assert!(
        document(&sandbox, &["cache", "status", "--format", "json"])["data"]["loaded_at"].is_null()
    );
}

#[test]
fn fr_cache_029_repointing_an_entry_invalidates_nothing() {
    // FR-CACHE-029 and BR-CACHE-003, stated plainly because the consequence is
    // accepted: after repointing an entry, a read serves the previous server's
    // catalogue with exit 0 and nothing in the output says so.
    let _guard = fixture::exclusive();
    let Some(series) = fixture::series("fr_cache_029_repointing_an_entry_invalidates_nothing")
    else {
        return;
    };
    let Some(server) = series.first() else {
        return;
    };
    let sandbox = project(server, ROOT);

    let warmed = document(&sandbox, &["schema", "tables", "--format", "json"]);
    assert_eq!(warmed["source"], "server");

    // The entry is repointed at a port nothing answers on. The read is served
    // from the store all the same, and opens no connection to find out.
    succeeds(
        &sandbox,
        &["cfg", "set", &format!("database.{ENTRY}.port"), "1"],
    );

    let served = document(&sandbox, &["schema", "tables", "--format", "json"]);
    assert_eq!(served["source"], "cache");
    assert_eq!(served["data"], warmed["data"]);

    // The only signal is the load time, per BR-CACHE-003, and it is in
    // `tpl cache status` alone.
    assert!(succeeds(&sandbox, &["cache", "status"]).contains("loaded_at"));

    // `--direct` does go to the server, and finds nothing there.
    let direct = run(&sandbox, &["schema", "tables", "--direct"]);
    assert_eq!(direct.code, Some(69), "{}", direct.err);
}

// ------------------------------------------------- the entry a read needs ---

#[test]
fn fr_conf_040_and_fr_conf_041_an_entry_that_describes_no_read_is_refused_with_seventy_eight() {
    // FR-CONF-040 and FR-CONF-041: the `cause` names the file, the entry and
    // the key the entry does not carry, and the `hint` carries the
    // `tpl cfg database update` that writes it. Neither needs a server: both
    // are decided where `.tpl/.cfg` is open.
    let sandbox = Sandbox::new();
    sandbox.project(&format!(
        "[core]\ndatabase = \"{ENTRY}\"\n\n[database.{ENTRY}]\nuser = \"reader\"\n"
    ));

    let no_host = run(&sandbox, &["schema", "tables"]);
    assert_eq!(no_host.code, Some(78), "{}", no_host.err);
    assert!(
        line(&no_host.err, "cause:").contains(&format!("database.{ENTRY}.host")),
        "{}",
        no_host.err
    );
    assert_eq!(
        line(&no_host.err, "hint:"),
        format!("tpl cfg database update {ENTRY} --host <host>")
    );

    // With a host and no database, the second of the two.
    succeeds(
        &sandbox,
        &[
            "cfg",
            "set",
            &format!("database.{ENTRY}.host"),
            "db.example.com",
        ],
    );

    let no_database = run(&sandbox, &["schema", "tables"]);
    assert_eq!(no_database.code, Some(78), "{}", no_database.err);
    assert!(
        line(&no_database.err, "cause:").contains(&format!("database.{ENTRY}.database")),
        "{}",
        no_database.err
    );
    assert_eq!(
        line(&no_database.err, "hint:"),
        format!("tpl cfg database update {ENTRY} --schema <database>")
    );
}

#[test]
fn fr_cache_024_the_cache_arm_names_a_routine_by_the_same_rules_the_schema_arm_does() {
    // FR-CACHE-024 adopts the qualified forms of `FR-SCH-008` and the `64` of
    // `FR-SCH-010` for a bare name that reaches both namespaces. The hint of
    // each refusal is the **same** invocation corrected, per `FR-ERR-009`, and
    // that invocation names the object by flag here where the first arm names
    // it positionally.
    let _guard = fixture::exclusive();
    let Some(series) = fixture::series(
        "fr_cache_024_the_cache_arm_names_a_routine_by_the_same_rules_the_schema_arm_does",
    ) else {
        return;
    };
    let Some(server) = series.first() else {
        return;
    };
    let sandbox = project(server, ROOT);

    // The prefix case, which is decided from the token alone.
    for (command, written, corrected) in [
        (
            "load",
            format!("FUNCTION:{FUNCTION}"),
            format!("function:{FUNCTION}"),
        ),
        (
            "clean",
            format!("Procedure:{PROCEDURE}"),
            format!("procedure:{PROCEDURE}"),
        ),
    ] {
        let outcome = run(&sandbox, &["cache", command, "--routine", &written]);

        assert_eq!(outcome.code, Some(64), "{}", outcome.err);
        assert_eq!(
            line(&outcome.err, "hint:"),
            format!("write it as: tpl cache {command} --routine {corrected}")
        );
    }

    // The ambiguity, which `tpl cache clean` decides over what the store holds
    // because that is what the command acts on. The fixture carries no name in
    // both namespaces, so the store is given one.
    succeeds(&sandbox, &["cache", "load"]);
    let routines = store(&sandbox).join("routines");
    std::fs::copy(
        routines.join(format!("function.{FUNCTION}.json")),
        routines.join(format!("procedure.{FUNCTION}.json")),
    )
    .expect("the store is ours");

    let ambiguous = run(&sandbox, &["cache", "clean", "--routine", FUNCTION]);
    assert_eq!(ambiguous.code, Some(64), "{}", ambiguous.err);
    let cause = line(&ambiguous.err, "cause:");
    assert!(
        cause.contains(&format!("'procedure:{FUNCTION}'")),
        "{cause}"
    );
    assert!(cause.contains(&format!("'function:{FUNCTION}'")), "{cause}");
    assert_eq!(
        line(&ambiguous.err, "hint:"),
        format!("name the kind you mean: tpl cache clean --routine procedure:{FUNCTION}")
    );

    // Qualified, it removes the one it names and leaves the other.
    succeeds(
        &sandbox,
        &[
            "cache",
            "clean",
            "--routine",
            &format!("procedure:{FUNCTION}"),
        ],
    );
    assert!(!routines.join(format!("procedure.{FUNCTION}.json")).exists());
    assert!(routines.join(format!("function.{FUNCTION}.json")).exists());
}

// ------------------------------------------------- the reduction of info ---

#[test]
fn fr_sch_031_info_carries_four_members_and_does_not_emit_what_the_dump_emits() {
    // FR-SCH-031 as the twenty-seventh edition amended it: `data.database` is
    // a **named subset** — `name`, `charset`, `collation` and the `server`
    // object — and it does not carry the three collections of FR-CTX-035. The
    // requirement states the consequence in its own text: `tpl schema info
    // --format json` and `tpl schema dump` SHALL NOT emit the same bytes.
    //
    // The reversed reading had made them byte-identical, which cost a calling
    // agent the whole model to ask a database's name and left two of the eight
    // subcommands indistinguishable to a caller that reads only the bytes.
    let _guard = fixture::exclusive();
    let Some(series) = fixture::series(
        "fr_sch_031_info_carries_four_members_and_does_not_emit_what_the_dump_emits",
    ) else {
        return;
    };

    for server in series {
        let sandbox = project(server, ROOT);
        let name = server.name();

        let info = document(
            &sandbox,
            &[
                "schema",
                "info",
                "--format",
                "json",
                "--direct",
                "--no-cache",
            ],
        );
        let database = info["data"]["database"]
            .as_object()
            .unwrap_or_else(|| panic!("{name}: data.database is not an object: {info}"));

        // Exactly four members, and the four FR-SCH-031 names. The count is
        // asserted beside the membership because a fifth member is the defect
        // the amendment removed.
        //
        // The membership is read from the parsed document and the **order**
        // from the bytes below: `preserve_order` is off, per FR-OUT-013, so a
        // parsed object carries its keys sorted and says nothing about the
        // order they were emitted in — which is the order `OD-18` fixes.
        let members: Vec<&str> = database.keys().map(String::as_str).collect();

        assert_eq!(
            members,
            ["charset", "collation", "name", "server"],
            "{name}: FR-SCH-031 fixes exactly these four members"
        );

        for absent in ["tables", "views", "routines"] {
            assert!(
                !database.contains_key(absent),
                "{name}: info carried {absent}, which FR-SCH-031 withholds"
            );
        }

        // Every member is the same member, under the same name and with the
        // same value, that the context document carries — which is the other
        // half of what FR-SCH-031 promises a caller.
        let dump = document(&sandbox, &["schema", "dump", "--direct", "--no-cache"]);

        for shared in ["name", "charset", "collation", "server"] {
            assert_eq!(
                info["data"]["database"][shared], dump["data"]["database"][shared],
                "{name}: {shared} differs between the two commands"
            );
        }

        // And the two do not emit the same bytes, which is the sentence the
        // requirement carries in bold.
        let info_bytes = succeeds(
            &sandbox,
            &[
                "schema",
                "info",
                "--format",
                "json",
                "--direct",
                "--no-cache",
            ],
        );
        let dump_bytes = succeeds(&sandbox, &["schema", "dump", "--direct", "--no-cache"]);

        // The emitted key order is FR-CTX-036's, with the collections removed
        // from the end, per OD-18 — read where it is observable, in the bytes.
        let mut at = 0;

        for key in ["\"name\"", "\"charset\"", "\"collation\"", "\"server\""] {
            let found = info_bytes[at..]
                .find(key)
                .unwrap_or_else(|| panic!("{name}: {key} is out of order in {info_bytes}"));

            at += found + key.len();
        }

        assert_ne!(
            info_bytes, dump_bytes,
            "{name}: info and dump emitted the same bytes"
        );
        assert!(
            dump_bytes.len() > info_bytes.len(),
            "{name}: the dump carries the three collections and info does not"
        );
    }
}

// ------------------------------------ the diagnostic line per catalogue query ---

#[test]
fn fr_glob_017_one_diagnostic_line_per_catalogue_query_and_none_at_the_default_level() {
    // FR-GLOB-017: at INFO the system writes exactly one line per catalogue
    // query it issues, in a form distinguishable from every other diagnostic
    // line. NFR-PERF-008 is what that rule is for — it makes the query count
    // observable from outside the process, which is what lets NFR-PERF-001 and
    // NFR-PERF-002 be checked at all.
    //
    // The count is asserted against the eleven statements the server itself
    // receives, so the two instruments are read against each other rather than
    // against a number written twice.
    let _guard = fixture::exclusive();
    let Some(series) = fixture::series(
        "fr_glob_017_one_diagnostic_line_per_catalogue_query_and_none_at_the_default_level",
    ) else {
        return;
    };

    for server in series {
        let sandbox = project(server, ROOT);
        let name = server.name();

        fixture::statements_on(server);
        let verbose = run(
            &sandbox,
            &["schema", "dump", "--direct", "--no-cache", "-v"],
        );
        fixture::statements_off(server);

        assert_eq!(verbose.code, Some(0), "{name}: {}", verbose.err);

        let lines = verbose
            .err
            .lines()
            .filter(|line| line.starts_with(QUERY_TOKEN))
            .count();
        let received = fixture::statements_count(server, &["--user", ROOT.0, "--catalogue"]);

        assert_eq!(
            lines as i64, FULL_READ,
            "{name}: {lines} diagnostic line(s) for a full read:\n{}",
            verbose.err
        );
        assert_eq!(
            lines as i64, received,
            "{name}: the process reported {lines} queries and the server received {received}"
        );

        // FR-ERR-013 and FR-GLOB-018: no credential and no driver message. The
        // whole of stderr is read, not only the query lines, because the
        // prohibition is over the stream.
        for forbidden in [ROOT.1, "password", "tls", "sqlx"] {
            assert!(
                !verbose
                    .err
                    .to_lowercase()
                    .contains(&forbidden.to_lowercase()),
                "{name}: the diagnostic stream carried {forbidden:?}:\n{}",
                verbose.err
            );
        }

        // The default level is below INFO, per FR-GLOB-014, so the same read
        // writes none of them.
        let quiet = run(&sandbox, &["schema", "dump", "--direct", "--no-cache"]);

        assert_eq!(quiet.code, Some(0), "{name}: {}", quiet.err);
        assert_eq!(
            quiet
                .err
                .lines()
                .filter(|line| line.starts_with(QUERY_TOKEN))
                .count(),
            0,
            "{name}: a run at the default level wrote a catalogue-query line:\n{}",
            quiet.err
        );

        // The control: the same invocation at INFO did write them, so the
        // silence above is an observation rather than a stream nobody reads.
        assert!(
            lines > 0,
            "{name}: the INFO run wrote nothing, so the default run's silence establishes nothing"
        );
    }
}

// ------------------------------------------------- FR-SRV-026, FR-SRV-029 ---

/// The field the divergence register of `FR-SRV-036` holds, on every view,
/// routine and trigger.
///
/// `FR-SRV-039` passes it through, so it differs between `10.11` and the other
/// three from identical DDL; `FR-SRV-026` excepts exactly the fields that
/// register names, and a field that differs without a row there is a failure
/// of that requirement rather than an instance of its exception.
const PASSED_THROUGH: &str = "collation_connection";

/// What every value of a registered field is replaced by before the four
/// documents are compared.
const REGISTERED: &str = "<registered under FR-SRV-039>";

/// Replaces every registered value in `document`, in place, wherever it sits.
///
/// It walks the whole document rather than the paths the register names,
/// because `collation_connection` sits on three object kinds and inside two
/// levels of embedding, and a walk cannot miss one the way a list of paths can.
fn normalise(document: &mut serde_json::Value) {
    match document {
        serde_json::Value::Object(members) => {
            for (key, value) in members.iter_mut() {
                if key == PASSED_THROUGH {
                    *value = serde_json::Value::String(REGISTERED.to_owned());
                } else {
                    normalise(value);
                }
            }
        }
        serde_json::Value::Array(members) => {
            for member in members.iter_mut() {
                normalise(member);
            }
        }
        _ => {}
    }
}

#[test]
fn fr_srv_029_the_same_ddl_yields_the_same_document_on_every_series_of_the_window() {
    // FR-SRV-029 with FR-SRV-026: for a database created from the same
    // accepted DDL on each series of FR-SRV-015, the document is
    // byte-identical across the four, except for the fields `null` under
    // FR-SRV-004, the `server` object of FR-SRV-028, and the values passed
    // through under FR-SRV-039.
    //
    // This is what "properly supported" means, stated so that it can be tested
    // rather than reviewed: a normalisation that is merely intended is
    // indistinguishable from one that is absent until four documents are
    // diffed. The fixture's DDL is DDL all four accept, which is the
    // requirement's own consequence for `scripts/mariadb/`.
    let _guard = fixture::exclusive();
    let Some(series) = fixture::series(
        "fr_srv_029_the_same_ddl_yields_the_same_document_on_every_series_of_the_window",
    ) else {
        return;
    };

    let mut documents: Vec<(&str, serde_json::Value, serde_json::Value)> = Vec::new();

    for server in series {
        let sandbox = project(server, ROOT);
        let raw = document(&sandbox, &["schema", "dump", "--direct", "--no-cache"]);

        let mut compared = raw["data"]["database"].clone();

        // FR-SRV-028: the `server` object carries the version, the series and
        // the standing of the server that was read, so it differs by
        // construction and is the requirement's first named exception.
        compared
            .as_object_mut()
            .expect("the database object is an object")
            .remove("server")
            .expect("FR-CTX-031 puts a server object on every document");

        let unnormalised = compared.clone();
        normalise(&mut compared);

        documents.push((server.name(), unnormalised, compared));
    }

    assert!(
        documents.len() > 1,
        "the equivalence needs more than one series to compare"
    );

    // The control: before the registered field is normalised the four are
    // **not** all equal, so the comparison below is a live one rather than one
    // over documents that could not differ. FR-SRV-038's difference in the
    // servers' own default collations is what produces it.
    let differs = documents
        .iter()
        .any(|(_, unnormalised, _)| *unnormalised != documents[0].1);

    assert!(
        differs,
        "no two series differed before normalisation, so the register's exception \
         is not being exercised and this comparison establishes nothing"
    );

    // And after it, every one of the four is the same document.
    let (first, _, expected) = &documents[0];

    for (name, _, compared) in &documents[1..] {
        assert_eq!(
            compared, expected,
            "{name} and {first} produced different documents from the same DDL. \
             FR-SRV-026 excepts only the fields `null` under FR-SRV-004, the server object, \
             and the values the register of FR-SRV-036 names; a field that differs without a \
             row there is a failure of that requirement, not an instance of its exception."
        );
    }

    // The exception is bounded to the fields the register names: the field
    // that was normalised did carry a value, so the normalisation is not
    // silently erasing an absent one.
    for (name, unnormalised, _) in &documents {
        let carried = unnormalised["routines"]
            .as_array()
            .expect("the document carries a routines collection")
            .first()
            .and_then(|routine| routine[PASSED_THROUGH].as_str());

        assert!(
            carried.is_some_and(|value| !value.is_empty()),
            "{name}: {PASSED_THROUGH} carried nothing, so normalising it hid no difference"
        );
    }
}
