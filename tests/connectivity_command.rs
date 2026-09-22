//! `tpl cfg database test`, exercised as a process.
//!
//! It is the one `cfg` subcommand that contacts a server, per `FR-CFG-005`, and
//! the only command of the tree whose whole purpose is to report **which** of
//! several things is wrong with a database entry. `FR-CFG-024` gives it four
//! steps, `FR-CFG-039` one field per step, and `FR-CFG-043` and `FR-CFG-045`
//! between them fix which outcomes are a refusal and which are an answer:
//!
//! | Property | Requirement |
//! |---|---|
//! | Four steps performed in one order, each reported | `FR-CFG-024` |
//! | Five members of `data`, in one order, inside the envelope | `FR-CFG-039`, `FR-OUT-024` |
//! | `source` is `server` and not `project` | `FR-CFG-035` |
//! | A reader that cannot see the database is exit `0` carrying false | `FR-CFG-044`, `FR-CFG-045` |
//! | `--format` and `--pretty` are declared | `FR-CFG-026` |
//! | An unreachable server is `69`, a refused credential `77` | `FR-ERR-034` |
//! | An entry that names no database is `78` and reports no step | `FR-CONF-041` |
//! | No failure of this command is a JSON document | `FR-ERR-033`, `FR-ERR-008` |
//! | Neither the store nor a file of it is created | `FR-CACHE-010` |
//!
//! Only a process shows most of that: the exit code, the bytes of stdout, the
//! four labelled lines on stderr, and the absence of `.tpl/.cache/` afterwards
//! are none of them observable from inside the crate.
//!
//! **The one outcome no body here can reach.** `FR-CFG-043` puts a server whose
//! series is outside the supported window on `78`, and the fixture holds no such
//! server: `series.env` carries the four series of `FR-SRV-015` and all four are
//! inside it. `FR-SRV-035` fixes the only admissible way to reach that
//! condition — narrowing the **reader** through the `#[cfg(test)]` seam of
//! `FR-ERR-031` — and an integration test links the library compiled without
//! that configuration and cannot see it. The body that exercises it is
//! therefore a unit test, in `src/cli/cfg/connectivity.rs`, and it asserts the
//! code, the condition and the `cause` line `FR-CFG-043` obliges.
//!
//! **Every run is made in a temporary directory of its own**, with no `.tpl`
//! above it, under a cleared environment. The bodies that need a server are
//! gated on `scripts/mariadb/status.sh` through
//! [`fixture`](../fixture/index.html): `0` runs them, `1` skips them with a
//! printed reason, and `2` — half a fixture — fails the run.

#[path = "support/fixture.rs"]
mod fixture;
#[path = "support/sandbox.rs"]
mod sandbox;

use std::process::Output;

use fixture::Server;
use sandbox::Sandbox;

/// The database entry every project below defines.
const ENTRY: &str = "fixture";

/// The schema every fixture server carries.
const SCHEMA: &str = "freight";

/// The privileged account of the fixture, which sees every database.
const ROOT: (&str, &str) = ("root", "tpl-root");

/// The reduced-grant reader of the fixture, which holds
/// `SELECT, EXECUTE ON freight.*` and nothing else.
const REDUCED: (&str, &str) = ("tpl_reader", "tpl-reader-pw");

/// A database every fixture server carries and [`REDUCED`] holds no grant on.
const UNGRANTED_SCHEMA: &str = "mysql";

/// The four labels of `FR-ERR-008`, in the order it fixes.
const LABELS: [&str; 4] = ["error: ", "cause: ", "hint:  ", "exit:  "];

/// The exit code of a run.
fn code(printed: &Output) -> i32 {
    printed
        .status
        .code()
        .expect("the binary under test exits rather than being signalled")
}

/// Standard output, as text.
fn stdout(printed: &Output) -> String {
    String::from_utf8_lossy(&printed.stdout).into_owned()
}

/// Standard error, as text.
fn stderr(printed: &Output) -> String {
    String::from_utf8_lossy(&printed.stderr).into_owned()
}

/// A sandbox whose one entry reaches `server` as `account` and selects
/// `schema`.
fn project(server: &Server, account: (&str, &str), schema: &str) -> Sandbox {
    let sandbox = Sandbox::new();
    sandbox.project(&fixture::configuration(server, ENTRY, schema, account));

    sandbox
}

/// Runs `tpl cfg database test <ENTRY>` and refuses anything but exit `0`.
fn tested(sandbox: &Sandbox, flags: &[&str]) -> String {
    let mut arguments = vec!["cfg", "database", "test", ENTRY];
    arguments.extend_from_slice(flags);

    let printed = sandbox.run(&arguments);

    assert_eq!(
        code(&printed),
        0,
        "tpl {} exited {}: {}",
        arguments.join(" "),
        code(&printed),
        stderr(&printed)
    );

    stdout(&printed)
}

/// Asserts that `printed` is the refusal `expected` names, in the four labelled
/// lines of `FR-ERR-008`, with stdout untouched — which is `FR-ERR-033` for
/// every one of them, because this command declares `--format`.
fn assert_refused(printed: &Output, expected: i32, spelled: &str) -> String {
    let written = stderr(printed);

    assert_eq!(
        code(printed),
        expected,
        "{spelled} exited {}: {written}",
        code(printed)
    );
    assert_eq!(
        stdout(printed),
        "",
        "{spelled}: FR-ERR-033 leaves stdout empty on a refusal"
    );

    for label in LABELS {
        assert!(
            written.contains(label),
            "{spelled}: the diagnostic carries no {label:?} line: {written}"
        );
    }

    written
}

/// The series of `FR-SRV-015`, or [`None`] where the fixture is not up.
fn series(test: &str) -> Option<&'static [Server]> {
    fixture::series(test)
}

// ------------------------------------------------------------ FR-CFG-039 ---

#[test]
fn fr_cfg_039_the_document_is_the_envelope_and_the_five_members_in_order() {
    // FR-CFG-039 writes the document out in full and FR-CFG-035 fixes its
    // `source`: `server`, and not the `project` every other cfg subcommand
    // carries, because contacting the server is what this command does. The key
    // order is part of the requirement, so the assertion is over the bytes: a
    // parsed value is exactly what loses it.
    //
    // The prefix below is FR-CFG-039's own example with the entry name of this
    // file's projects substituted, and the suffix is its last member. What sits
    // between them is the `server` object of FR-CTX-031, whose three values
    // differ from server to server and whose three keys do not.
    let Some(series) =
        series("fr_cfg_039_the_document_is_the_envelope_and_the_five_members_in_order")
    else {
        return;
    };
    let _exclusive = fixture::exclusive();

    for server in series {
        let sandbox = project(server, ROOT, SCHEMA);
        let written = tested(&sandbox, &["--format", "json"]);
        let name = server.name();

        assert!(
            written.starts_with(
                "{\"schema_version\":1,\"source\":\"server\",\"data\":\
                 {\"entry\":\"fixture\",\"connected\":true,\"read_only_session\":true,\
                 \"server\":{\"version\":\""
            ),
            "{name}: {written}"
        );
        assert!(
            written.ends_with("\"standing\":\"supported\"},\"can_read_catalogue\":true}}\n"),
            "{name}: {written}"
        );
        assert!(
            written.contains(&format!("\"series\":\"{name}\"")),
            "{name}: the document does not report the series that answered: {written}"
        );
    }
}

#[test]
fn fr_cfg_026_the_command_declares_format_and_pretty_and_nothing_else_changes() {
    // FR-CFG-026 gives this command both flags, and FR-OUT-007 and FR-OUT-008
    // fix what the second does: it changes the whitespace and nothing else.
    // FR-OUT-009 is the other half — `--pretty` without `--format json` is the
    // `64` of a refusal between two arguments — and it is asserted here because
    // this leaf declares the pair for the first time.
    let Some(series) =
        series("fr_cfg_026_the_command_declares_format_and_pretty_and_nothing_else_changes")
    else {
        return;
    };
    let _exclusive = fixture::exclusive();
    let Some(server) = series.first() else {
        return;
    };

    let sandbox = project(server, ROOT, SCHEMA);
    let compact = tested(&sandbox, &["--format", "json"]);
    let indented = tested(&sandbox, &["--format", "json", "--pretty"]);

    assert_eq!(compact.matches('\n').count(), 1, "{compact}");
    assert!(
        indented.contains("\n  \"schema_version\": 1,\n"),
        "{indented}"
    );
    assert!(
        indented.contains("\n    \"can_read_catalogue\": true\n"),
        "{indented}"
    );

    assert_refused(
        &sandbox.run(&["cfg", "database", "test", ENTRY, "--pretty"]),
        64,
        "--pretty without --format json",
    );
}

#[test]
fn fr_out_006_the_text_form_reports_the_same_four_answers_for_a_person() {
    // FR-OUT-004 makes the `text` form no contract at all, and FR-OUT-006 makes
    // it a layout carrying the useful information rather than only a name. What
    // is asserted is that each of the four answers is present and readable, and
    // that `text` is the default when `--format` is absent.
    let Some(series) =
        series("fr_out_006_the_text_form_reports_the_same_four_answers_for_a_person")
    else {
        return;
    };
    let _exclusive = fixture::exclusive();
    let Some(server) = series.first() else {
        return;
    };

    let sandbox = project(server, ROOT, SCHEMA);
    let written = tested(&sandbox, &[]);

    assert!(written.starts_with("PROPERTY"), "{written}");

    for row in [
        "entry               fixture",
        "connected           yes",
        "read_only_session   yes",
        "standing            supported",
        "can_read_catalogue  yes",
    ] {
        assert!(written.contains(row), "{row:?} is not in {written}");
    }
}

// ----------------------------------------------- FR-CFG-044 and FR-CFG-045 ---

#[test]
fn fr_cfg_045_a_reader_that_cannot_see_the_database_is_exit_zero_carrying_false() {
    // FR-CFG-044 and FR-CFG-045 together, in the arrangement the fixture was
    // built to make reproducible: `tpl_reader` holds `SELECT, EXECUTE ON
    // freight.*` and nothing else, so the server accepts its credentials and
    // shows it no row of the schema catalogue for `mysql`.
    //
    // **`0`, not `77`.** That is the whole of FR-CFG-045: "A can_read_catalogue
    // of false SHALL NOT change the exit code", because "a false that exited 77
    // could never be observed in the success document of FR-CFG-039, which
    // would make the field dead surface". A caller that branches on the exit
    // code alone and needs the fourth answer reads the field, which is what the
    // help of this command says and what its second EXAMPLES entry shows.
    //
    // The control is the same account on the same server against the database it
    // *can* see: without it, a probe that always answered false would pass.
    let Some(series) =
        series("fr_cfg_045_a_reader_that_cannot_see_the_database_is_exit_zero_carrying_false")
    else {
        return;
    };
    let _exclusive = fixture::exclusive();

    for server in series {
        let name = server.name();
        let hidden = project(server, REDUCED, UNGRANTED_SCHEMA);
        let written = tested(&hidden, &["--format", "json"]);

        assert!(
            written.contains("\"connected\":true")
                && written.contains("\"read_only_session\":true"),
            "{name}: the first two steps succeeded and the document says otherwise: {written}"
        );
        assert!(
            written.contains("\"can_read_catalogue\":false"),
            "{name}: {written}"
        );

        let granted = project(server, REDUCED, SCHEMA);

        assert!(
            tested(&granted, &["--format", "json"]).contains("\"can_read_catalogue\":true"),
            "{name}: the probe answered false for a database this reader can see"
        );
    }
}

// ------------------------------------------------------------ FR-ERR-034 ---

#[test]
fn fr_err_034_a_refused_credential_is_seventy_seven_and_writes_no_document() {
    // The `77` row of FR-ERR-034 and the help of this command: the server
    // refused the credentials. The entry is right in every other respect — the
    // host answers and the database exists — so the code separates this from
    // the `69` below and from the `0` above.
    let Some(series) =
        series("fr_err_034_a_refused_credential_is_seventy_seven_and_writes_no_document")
    else {
        return;
    };
    let _exclusive = fixture::exclusive();
    let Some(server) = series.first() else {
        return;
    };

    let sandbox = project(server, ("root", "not-the-password"), SCHEMA);
    let written = assert_refused(
        &sandbox.run(&["cfg", "database", "test", ENTRY, "--format", "json"]),
        77,
        "an entry whose password the server refuses",
    );

    assert!(
        written.contains("root"),
        "the 77 row of FR-ERR-034 obliges the cause to name the user presented: {written}"
    );
}

#[test]
fn fr_err_034_a_server_that_does_not_answer_is_sixty_nine_and_needs_no_fixture() {
    // The `69` row of FR-ERR-034. Nothing listens on port 1, and this body is
    // not gated: a server that is not there is exactly what it is about.
    let sandbox = Sandbox::new();
    sandbox.project(
        "[core]\ndatabase = \"fixture\"\n\n\
         [database.fixture]\nhost = \"127.0.0.1\"\nport = 1\n\
         user = \"root\"\npassword = \"irrelevant\"\n\
         database = \"freight\"\ntls = \"disabled\"\n",
    );

    let written = assert_refused(
        &sandbox.run(&["cfg", "database", "test", ENTRY, "--format", "json"]),
        69,
        "an entry pointing at an address nothing answers on",
    );

    assert!(
        written.contains("127.0.0.1:1"),
        "the 69 row of FR-ERR-034 obliges the cause to name the address: {written}"
    );
}

#[test]
fn fr_conf_041_an_entry_that_names_no_database_is_seventy_eight_and_reports_no_step() {
    // FR-CONF-041 names this command: the probe of FR-CFG-044 "has nothing to
    // restrict itself to" without a database, so `tpl cfg database test` exits
    // `78` on such an entry "and it reports none of the four steps of
    // FR-CFG-024". Needs no fixture, because no connection is opened — which is
    // the half a body reaching a server could not show.
    //
    // FR-GLOB-007's `66` is asserted beside it, for the same reason: entry
    // resolution precedes the four steps, so a name that reaches no entry
    // reports none of them either.
    let sandbox = Sandbox::new();
    sandbox.project(
        "[core]\ndatabase = \"fixture\"\n\n\
         [database.fixture]\nhost = \"db.example.com\"\nuser = \"reader\"\n",
    );

    let written = assert_refused(
        &sandbox.run(&["cfg", "database", "test", ENTRY, "--format", "json"]),
        78,
        "an entry that names no server-side database",
    );

    assert!(
        written.contains("database.fixture.database"),
        "the 78 row of FR-ERR-034 obliges the cause to name the key: {written}"
    );
    assert!(
        written.contains("tpl cfg database update fixture --schema"),
        "FR-CONF-041 obliges the hint to carry the command that repairs it: {written}"
    );

    assert_refused(
        &sandbox.run(&["cfg", "database", "test", "reporting"]),
        66,
        "a NAME that reaches no entry of .tpl/.cfg",
    );
}

// ------------------------------------------------------------ FR-CACHE-010 ---

#[test]
fn fr_cache_010_the_command_creates_no_store_and_reads_nothing_into_the_model() {
    // FR-CACHE-010's middle clause, in the state `FR-PROJ-018` makes the
    // ordinary first one: a project that has never read a catalogue has no
    // `.tpl/.cache/`, and a successful `tpl cfg database test` leaves it with
    // none. `tests/schema_and_cache.rs` asserts the same clause over a **warm**
    // store, which is the other half; this is the half where a store would have
    // to be created for the clause to be broken.
    //
    // The third clause — that it reads nothing into the model — is read from
    // the document: `data` carries the five members of FR-CFG-039 and no
    // `database` object, so nothing of the catalogue reached it. The one
    // statement the command issues against the catalogue is the probe, whose
    // result is a boolean.
    let Some(series) =
        series("fr_cache_010_the_command_creates_no_store_and_reads_nothing_into_the_model")
    else {
        return;
    };
    let _exclusive = fixture::exclusive();
    let Some(server) = series.first() else {
        return;
    };

    let sandbox = project(server, ROOT, SCHEMA);
    let written = tested(&sandbox, &["--format", "json"]);

    assert!(
        !sandbox.path(".tpl/.cache").exists(),
        "FR-CACHE-010: tpl cfg database test created a store"
    );
    assert!(
        !written.contains("\"tables\"")
            && !written.contains("\"views\"")
            && !written.contains("\"routines\"")
            && !written.contains("\"collation\""),
        "FR-CACHE-010: the document carries catalogue content: {written}"
    );
}
