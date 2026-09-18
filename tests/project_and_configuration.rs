//! The project and its configuration, exercised as a process.
//!
//! This sprint gives `tpl` its notion of a project — how one is created, how
//! one is found, why one is trusted, and how `.tpl/.cfg` is read, written and
//! refused. Every rule it delivers is a rule about a **filesystem** and an
//! **exit code**, and neither is observable from inside the crate:
//!
//! | Property | Requirement |
//! |---|---|
//! | What `tpl init` leaves on disk, and at which mode | `FR-PROJ-017`, `FR-PROJ-019`, `FR-PROJ-020` |
//! | Which directory the walk starts from, and which project it finds | `FR-PROJ-004`, `FR-GLOB-009` |
//! | Which commands may run with no project at all | `FR-PROJ-025` |
//! | The code and the four labelled lines each refusal carries | `FR-ERR-001`, `FR-ERR-008` |
//! | That a refused write left the file byte for byte as it was | `FR-CFG-041`, `FR-CFG-048` |
//!
//! A unit test can show that a function returns the right error; only a process
//! shows that the file on disk was not touched on the way, that stdout stayed
//! empty, and that the hint the caller was given is a command that works.
//!
//! **Every run is made in a temporary directory of its own**, with no `.tpl`
//! above it, under a cleared environment — on the same terms as
//! [`invocation_surface`](../invocation_surface/index.html). The directory is
//! removed when the test's sandbox goes out of scope, and nothing outside it is
//! read or written.
//!
//! **What this file deliberately does not reach.** Three areas of the sprint
//! are unobservable from a process at this commit, because the resolution step
//! of `FR-CONF-029` has no caller until a command opens a connection: `${VAR}`
//! expansion (`FR-CONF-021`, `FR-CONF-022`), `password_command` execution
//! (`FR-CONF-023` … `FR-CONF-033`), and the composition of `--timeout` with a
//! phase deadline (`FR-GLOB-012`). `tpl cfg` reads the file and never resolves
//! it, per `FR-CFG-014`, so no invocation of any command this sprint delivers
//! expands a reference, runs a child, or enters a blocking phase. Those three
//! are exercised by the unit tests of `project::config::expand`,
//! `project::password` and `deadline`, and belong here on the day a command
//! reaches them.

#[path = "support/sandbox.rs"]
mod sandbox;

use std::path::Path;
use std::process::Output;

use sandbox::Sandbox;

/// The mode `FR-PROJ-019` creates `.tpl/.cfg` with and `FR-CFG-034` keeps.
const MODE: u32 = 0o600;

/// The four labels of `FR-ERR-008`, in the order it fixes.
const LABELS: [&str; 4] = ["error: ", "cause: ", "hint:  ", "exit:  "];

/// Sets the mode of a path.
fn chmod(path: &Path, mode: u32) {
    use std::os::unix::fs::PermissionsExt as _;

    std::fs::set_permissions(path, std::fs::Permissions::from_mode(mode))
        .expect("the sandbox is writable");
}

/// The mode of a path, as the twelve bits `FR-PROJ-011` reads.
fn mode_of(path: &Path) -> u32 {
    use std::os::unix::fs::PermissionsExt as _;

    std::fs::metadata(path)
        .expect("the path is there")
        .permissions()
        .mode()
        & 0o7777
}

/// The exit code of a run.
fn code(printed: &Output) -> i32 {
    printed
        .status
        .code()
        .expect("the process was not signalled")
}

/// What the run wrote to stdout, as text.
fn stdout(printed: &Output) -> String {
    String::from_utf8(printed.stdout.clone()).expect("stdout is valid UTF-8")
}

/// What the run wrote to stderr, as text.
fn stderr(printed: &Output) -> String {
    String::from_utf8(printed.stderr.clone()).expect("a diagnostic is valid UTF-8")
}

/// Asserts that `printed` is a refusal carrying `expected`: that code, an empty
/// stdout per `FR-ERR-033`, and the four labelled lines of `FR-ERR-008` in
/// their order.
fn assert_refused(printed: &Output, expected: i32, spelled: &str) -> String {
    let written = stderr(printed);

    assert_eq!(
        code(printed),
        expected,
        "{spelled} did not exit {expected}: {written}"
    );
    assert!(
        printed.stdout.is_empty(),
        "{spelled} wrote {} bytes to stdout",
        printed.stdout.len()
    );

    let lines: Vec<&str> = written.lines().collect();

    assert_eq!(lines.len(), 4, "{spelled} wrote {written:?}");
    for (line, label) in lines.iter().zip(LABELS) {
        assert!(line.starts_with(label), "{spelled} wrote {line:?}");
    }
    assert!(
        lines[3].starts_with(&format!("exit:  {expected} ")),
        "{spelled} wrote {:?}",
        lines[3]
    );

    written
}

/// One labelled line of a diagnostic, without its label.
fn line(written: &str, label: &str) -> String {
    written
        .lines()
        .find(|line| line.starts_with(label))
        .unwrap_or_else(|| panic!("no {label:?} line in {written:?}"))
        .trim_start_matches(label)
        .trim_start()
        .to_owned()
}

// ------------------------------------------------------------ FR-PROJ-017 ---

#[test]
fn fr_proj_017_init_creates_exactly_the_five_artefacts_and_nothing_else() {
    // FR-PROJ-017, FR-PROJ-019, FR-PROJ-020 and FR-PROJ-022: five artefacts,
    // `.cfg` at 0600, no `.cache/`, nothing on stdout, exit 0.
    let sandbox = Sandbox::new();

    let printed = sandbox.run(&["init"]);

    assert_eq!(code(&printed), 0, "{}", stderr(&printed));
    assert_eq!(stdout(&printed), "");

    for artefact in [
        ".tpl/.cfg",
        ".tpl/.gitignore",
        ".tpl/templates",
        ".tpl/templates/example.jinja",
        ".tpl/templates/rust/_types.jinja",
    ] {
        assert!(
            sandbox.path(artefact).exists(),
            "tpl init did not create {artefact}"
        );
    }

    assert_eq!(mode_of(&sandbox.path(".tpl/.cfg")), MODE);
    assert!(
        !sandbox.path(".tpl/.cache").exists(),
        "tpl init created .tpl/.cache/, which FR-PROJ-020 forbids"
    );

    // "Exactly five" is a statement about what is absent as well: the folder
    // holds the three entries of the table's first level and no other.
    let mut entries: Vec<String> = std::fs::read_dir(sandbox.path(".tpl"))
        .expect("the folder is there")
        .map(|entry| {
            entry
                .expect("the entry is readable")
                .file_name()
                .to_string_lossy()
                .into_owned()
        })
        .collect();
    entries.sort();

    assert_eq!(entries, [".cfg", ".gitignore", "templates"]);
    assert_eq!(
        std::fs::read_to_string(sandbox.path(".tpl/.gitignore")).expect("it is there"),
        ".cfg\n.cache/\n"
    );
}

#[test]
fn fr_proj_018_the_generated_configuration_carries_no_active_database_entry() {
    // FR-PROJ-018: a fresh project knows about no database, and the shape of a
    // real entry is in front of the reader as a comment.
    let sandbox = Sandbox::new();

    assert_eq!(code(&sandbox.run(&["init"])), 0);

    let printed = sandbox.run(&["cfg", "database", "list", "--format", "json"]);

    assert_eq!(code(&printed), 0, "{}", stderr(&printed));
    assert_eq!(
        stdout(&printed),
        "{\"schema_version\":1,\"source\":\"project\",\"data\":{\"entries\":[]}}\n"
    );
}

// ------------------------------------------------------------ FR-PROJ-011 ---

#[test]
fn fr_proj_011_a_configuration_that_grants_group_or_other_access_is_refused() {
    // FR-PROJ-011: no group and no other access bits, and the refusal is the
    // 78 of its row. Ownership by another user, which FR-PROJ-010 refuses,
    // cannot be staged without privilege and is left to the unit test that
    // supplies the metadata.
    for granted in [0o640, 0o604, 0o666] {
        let sandbox = Sandbox::new();
        sandbox.project("[core]\n");
        chmod(&sandbox.path(".tpl/.cfg"), granted);

        let printed = sandbox.run(&["cfg", "list"]);
        let written = assert_refused(&printed, 78, &format!("mode {granted:04o}"));

        assert!(
            line(&written, "cause: ").contains(&format!("mode {granted:04o}")),
            "{written}"
        );
    }
}

#[test]
fn fr_glob_010_a_folder_named_by_the_flag_is_subject_to_the_same_checks() {
    // FR-GLOB-010 and FR-PROJ-008: --tpl-dir suppresses the walk and is
    // exempted from nothing.
    let sandbox = Sandbox::new();
    let tpl = sandbox.project_at("elsewhere", "[core]\n");
    chmod(&tpl.join(".cfg"), 0o644);

    let printed = sandbox.run(&["cfg", "list", "--tpl-dir", &tpl.display().to_string()]);

    assert_refused(&printed, 78, "--tpl-dir at mode 0644");
}

// ------------------------------------------------------------ FR-PROJ-004 ---

#[test]
fn fr_proj_004_the_walk_finds_the_project_from_a_nested_working_directory() {
    // FR-PROJ-004: the walk starts at the working directory and climbs.
    let sandbox = Sandbox::new();
    sandbox.project("[core]\ndatabase = \"shop\"\n");
    let nested = sandbox.directory("one/two/three");

    let printed = sandbox.run_from(&nested, &[], &["cfg", "get", "core.database"]);

    assert_eq!(code(&printed), 0, "{}", stderr(&printed));
    assert_eq!(stdout(&printed), "shop\n");
}

#[test]
fn fr_proj_004_the_first_project_the_walk_meets_is_the_one_it_stops_at() {
    // FR-PROJ-004: the first `.tpl` found is the project root, and the walk
    // stops there — so a nested project shadows the one above it. The other
    // boundary of the walk, the mount point of FR-PROJ-005, is not staged here:
    // it needs a mounted filesystem, which a temporary directory is not.
    let sandbox = Sandbox::new();
    sandbox.project("[core]\ndatabase = \"outer\"\n");
    sandbox.project_at("inner", "[core]\ndatabase = \"inner\"\n");
    let nested = sandbox.directory("inner/below");

    let printed = sandbox.run_from(&nested, &[], &["cfg", "get", "core.database"]);

    assert_eq!(code(&printed), 0, "{}", stderr(&printed));
    assert_eq!(stdout(&printed), "inner\n");
}

#[test]
fn fr_glob_009_the_flag_names_the_folder_and_suppresses_the_walk() {
    // FR-GLOB-009: the folder named is the one used, whatever the walk from
    // the working directory would have found.
    let sandbox = Sandbox::new();
    sandbox.project("[core]\ndatabase = \"walked\"\n");
    let named = sandbox.project_at("elsewhere", "[core]\ndatabase = \"named\"\n");

    let printed = sandbox.run(&[
        "cfg",
        "get",
        "core.database",
        "--tpl-dir",
        &named.display().to_string(),
    ]);

    assert_eq!(code(&printed), 0, "{}", stderr(&printed));
    assert_eq!(stdout(&printed), "named\n");
}

#[test]
fn fr_proj_006_a_walk_that_meets_no_project_is_refused_and_points_at_tpl_init() {
    // FR-PROJ-006: 78, and the hint suggests `tpl init`.
    let sandbox = Sandbox::new();

    let printed = sandbox.run(&["cfg", "list"]);
    let written = assert_refused(&printed, 78, "cfg list with no project");

    assert!(line(&written, "hint:  ").contains("tpl init"), "{written}");
}

// ------------------------------------------------------------ FR-PROJ-025 ---

#[test]
fn fr_proj_025_the_commands_that_require_no_project_run_where_there_is_none() {
    // FR-PROJ-025 names four and no others. `tpl init` is exercised by its own
    // tests; the other three, in the six forms FR-HELP-001 gives them, are
    // here.
    let sandbox = Sandbox::new();

    for invocation in [
        vec!["help"],
        vec!["help", "cfg"],
        vec!["help", "--format", "json"],
        vec!["--help"],
        vec!["cfg", "database", "add", "-h"],
        vec!["version"],
        vec!["-V"],
    ] {
        let printed = sandbox.run(&invocation);

        assert_eq!(
            code(&printed),
            0,
            "{invocation:?} was refused: {}",
            stderr(&printed)
        );
        assert!(
            !printed.stdout.is_empty(),
            "{invocation:?} answered with nothing"
        );
    }
}

#[test]
fn fr_proj_025_every_other_command_performs_discovery_and_fails_without_a_project() {
    // FR-PROJ-025: every command outside the four requires a project, and no
    // cfg subcommand is among the four.
    let sandbox = Sandbox::new();

    for invocation in [
        vec!["cfg", "get", "core.database"],
        vec!["cfg", "set", "core.database", "shop"],
        vec!["cfg", "unset", "core.database"],
        vec!["cfg", "list"],
        vec!["cfg", "database", "list"],
        vec!["cfg", "database", "show", "shop"],
        vec!["cfg", "database", "add", "shop", "--host", "db"],
        vec!["cfg", "database", "update", "shop", "--host", "db"],
        vec!["cfg", "database", "remove", "shop"],
    ] {
        assert_refused(&sandbox.run(&invocation), 78, &format!("{invocation:?}"));
    }
}

// ------------------------------------------------------------ FR-CONF-034 ---

#[test]
fn fr_err_006_a_file_the_reader_refuses_is_refused_before_the_command_is_reached() {
    // Step 3 of FR-ERR-006, over the three faults a `.cfg` written by hand
    // reaches without resolving anything: a key outside the space
    // (FR-CONF-034), a DSN outside the grammar (FR-CONF-009, FR-CONF-010) and a
    // DSN carrying a query parameter (FR-CONF-011).
    for (spelled, configuration, expected) in [
        (
            "an unknown key",
            "[core]\ndatabse = \"shop\"\n",
            "outside the key space",
        ),
        (
            "a malformed DSN",
            "[database.shop]\ndsn = \"postgres://db.example.com/shop\"\n",
            "scheme",
        ),
        (
            "a DSN carrying a query parameter",
            "[database.shop]\ndsn = \"mysql://db.example.com/shop?charset=utf8\"\n",
            "parameter",
        ),
    ] {
        let sandbox = Sandbox::new();
        sandbox.project(configuration);

        let printed = sandbox.run(&["cfg", "list"]);
        let written = assert_refused(&printed, 78, spelled);

        assert!(
            written.to_lowercase().contains(expected),
            "{spelled} did not name it: {written}"
        );
    }
}

// ------------------------------------------------------------ FR-CFG-041 ---

#[test]
fn fr_cfg_041_a_rewrite_keeps_every_byte_it_was_not_asked_to_change() {
    // FR-CFG-041 and FR-CFG-034: the comments, the key order and the spacing
    // are the file's own, and the mode survives the rename. The comparison is
    // byte for byte, against the file composed from the original with one value
    // replaced — which is the only difference a rewrite may make.
    let sandbox = Sandbox::new();
    let before = concat!(
        "# the project's own note\n",
        "[core]\n",
        "query_timeout = 30   # generous\n",
        "database = \"shop\"\n",
        "\n",
        "# the entry the team shares\n",
        "[database.shop]\n",
        "user = \"alice\"\n",
        "host = \"db.example.com\"\n",
    );
    sandbox.project(before);

    let printed = sandbox.run(&["cfg", "set", "core.query_timeout", "60"]);

    assert_eq!(code(&printed), 0, "{}", stderr(&printed));
    assert_eq!(stdout(&printed), "");
    assert_eq!(
        String::from_utf8(sandbox.configuration()).expect("the file is UTF-8"),
        before.replace("query_timeout = 30", "query_timeout = 60")
    );
    assert_eq!(mode_of(&sandbox.path(".tpl/.cfg")), MODE);
    assert_eq!(
        std::fs::read_dir(sandbox.path(".tpl"))
            .expect("the folder is there")
            .count(),
        1,
        "the rewrite left its temporary file behind"
    );
}

// ------------------------------------------------------------ FR-CFG-021 ---

#[test]
fn fr_cfg_021_the_two_printers_redact_and_the_directed_read_does_not() {
    // FR-CFG-021 redacts in `cfg list` and `cfg database show`; BR-CFG-002
    // makes `cfg get` the one deliberate exception, so that a password can be
    // fed to another command.
    let sandbox = Sandbox::new();
    sandbox.project(concat!(
        "[database.shop]\n",
        "host = \"db.example.com\"\n",
        "password = \"hunter2\"\n",
    ));

    for invocation in [
        vec!["cfg", "list"],
        vec!["cfg", "list", "--format", "json"],
        vec!["cfg", "database", "show", "shop"],
        vec!["cfg", "database", "show", "shop", "--format", "json"],
    ] {
        let printed = sandbox.run(&invocation);

        assert_eq!(code(&printed), 0, "{}", stderr(&printed));
        assert!(
            !stdout(&printed).contains("hunter2"),
            "{invocation:?} printed the password: {}",
            stdout(&printed)
        );
        assert!(
            stdout(&printed).contains("***"),
            "{invocation:?} printed no redaction: {}",
            stdout(&printed)
        );
    }

    let printed = sandbox.run(&["cfg", "get", "database.shop.password"]);

    assert_eq!(code(&printed), 0, "{}", stderr(&printed));
    assert_eq!(stdout(&printed), "hunter2\n");
}

#[test]
fn fr_cfg_021_the_password_inside_a_url_is_redacted_and_the_rest_of_it_is_not() {
    // FR-CFG-021: the substitution is over the one field, so the user, the
    // host, the port and the database stay visible.
    let sandbox = Sandbox::new();
    sandbox.project("[database.shop]\ndsn = \"mysql://alice:hunter2@db.example.com:3306/shop\"\n");

    let printed = sandbox.run(&["cfg", "database", "show", "shop"]);
    let written = stdout(&printed);

    assert_eq!(code(&printed), 0, "{}", stderr(&printed));
    assert!(
        written.contains("mysql://alice:***@db.example.com:3306/shop"),
        "{written}"
    );
    assert!(!written.contains("hunter2"), "{written}");
}

// ------------------------------------------------------------ FR-ERR-035 ---

#[test]
fn fr_cfg_007_a_named_thing_that_is_not_there_and_a_name_that_is_already_taken() {
    // FR-CFG-007 and FR-CFG-012 make an absent key 66; FR-CFG-017 makes a name
    // `add` cannot create 64; the entry `update`, `show` and `remove` cannot
    // find is 66, per BR-CFG-001 and FR-GLOB-007.
    let sandbox = Sandbox::new();
    sandbox.project("[database.shop]\nhost = \"db.example.com\"\n");

    assert_refused(
        &sandbox.run(&["cfg", "get", "core.database"]),
        66,
        "a key the file does not set",
    );
    assert_refused(
        &sandbox.run(&["cfg", "unset", "core.database"]),
        66,
        "unset of a key the file does not set",
    );
    assert_refused(
        &sandbox.run(&["cfg", "database", "add", "shop", "--host", "other"]),
        64,
        "add against a name that is taken",
    );
    assert_refused(
        &sandbox.run(&["cfg", "database", "show", "reporting"]),
        66,
        "show of an entry that is not there",
    );
    assert_refused(
        &sandbox.run(&["cfg", "database", "update", "reporting", "--host", "db"]),
        66,
        "update of an entry that is not there",
    );
    assert_refused(
        &sandbox.run(&["cfg", "database", "remove", "reporting"]),
        66,
        "remove of an entry that is not there",
    );

    // Nothing above wrote: the file is as it was.
    assert_eq!(
        String::from_utf8(sandbox.configuration()).expect("the file is UTF-8"),
        "[database.shop]\nhost = \"db.example.com\"\n"
    );
}

// ------------------------------------------------------------ FR-CFG-035 ---

#[test]
fn fr_cfg_035_every_subcommand_that_declares_the_flag_answers_in_the_envelope() {
    // FR-CFG-035 with FR-OUT-024 and FR-OUT-026: the envelope, and `source`
    // set to `project`. `tpl cfg database test` is the one exception, and the
    // sprint that opens a connection owns it.
    let sandbox = Sandbox::new();
    sandbox.project("[core]\ndatabase = \"shop\"\n\n[database.shop]\nhost = \"db.example.com\"\n");

    for invocation in [
        vec!["cfg", "get", "core.database", "--format", "json"],
        vec!["cfg", "list", "--format", "json"],
        vec!["cfg", "database", "list", "--format", "json"],
        vec!["cfg", "database", "show", "shop", "--format", "json"],
    ] {
        let printed = sandbox.run(&invocation);
        let written = stdout(&printed);

        assert_eq!(code(&printed), 0, "{invocation:?}: {}", stderr(&printed));
        assert!(
            written.starts_with("{\"schema_version\":1,\"source\":\"project\",\"data\":"),
            "{invocation:?} answered {written}"
        );
        assert!(
            written.ends_with("}\n"),
            "{invocation:?} answered {written}"
        );
    }

    // The five that change the file or name a key for removal declare neither
    // flag, so `--format` is the ordinary unknown-flag 64 of FR-CLI-019.
    assert_refused(
        &sandbox.run(&["cfg", "set", "core.database", "shop", "--format", "json"]),
        64,
        "cfg set --format json",
    );
}

// ------------------------------------------------------------ FR-CFG-031 ---

#[test]
fn fr_cfg_031_a_dsn_outside_what_the_file_admits_is_refused_by_both_write_paths() {
    // FR-CFG-031 with FR-CFG-010: `--dsn` and `tpl cfg set database.<name>.dsn`
    // admit exactly what FR-CONF-009, FR-CONF-010 and FR-CONF-011 admit, the
    // refusal is 64 because the invocation is at fault and the file is not, and
    // nothing is written — no `.cfg`, and no temporary file beside it.
    let before = "[core]\ndatabase = \"shop\"\n";

    for value in [
        "postgres://db.example.com/shop",
        "mysql://db.example.com/shop?charset=utf8",
        "mysql://db.example.com",
        "db.example.com/shop",
    ] {
        let sandbox = Sandbox::new();
        sandbox.project(before);

        for invocation in [
            vec!["cfg", "database", "add", "shop", "--dsn", value],
            vec!["cfg", "set", "database.shop.dsn", value],
        ] {
            let printed = sandbox.run(&invocation);
            let written = assert_refused(&printed, 64, &format!("{invocation:?}"));

            assert!(line(&written, "cause: ").contains(value), "{written}");
            assert_eq!(
                String::from_utf8(sandbox.configuration()).expect("the file is UTF-8"),
                before,
                "{invocation:?} changed the file"
            );
            assert_eq!(
                std::fs::read_dir(sandbox.path(".tpl"))
                    .expect("the folder is there")
                    .count(),
                1,
                "{invocation:?} left a temporary file behind"
            );
        }
    }
}

#[test]
fn fr_cfg_031_a_dsn_carrying_a_reference_is_admitted_and_stored_as_the_caller_wrote_it() {
    // FR-CFG-031: the value is validated as written, with ${VAR} left
    // unexpanded and treated as opaque text within the field it occupies. The
    // variable is defined in the child's environment, so a run that expanded it
    // would be caught here rather than pass for want of a value: no cfg command
    // reads the environment.
    let sandbox = Sandbox::new();
    sandbox.project("[core]\n");
    let value = "mysql://alice:${SHOP_DB_PASSWORD}@db.example.com:3306/shop";

    let printed = sandbox.run_from(
        &sandbox.path("."),
        &[("SHOP_DB_PASSWORD", "hunter2")],
        &["cfg", "database", "add", "shop", "--dsn", value],
    );

    assert_eq!(code(&printed), 0, "{}", stderr(&printed));
    assert_eq!(
        String::from_utf8(sandbox.configuration()).expect("the file is UTF-8"),
        format!("[core]\n\n[database.shop]\ndsn = \"{value}\"\n")
    );

    let read = sandbox.run_from(
        &sandbox.path("."),
        &[("SHOP_DB_PASSWORD", "hunter2")],
        &["cfg", "get", "database.shop.dsn"],
    );

    assert_eq!(stdout(&read), format!("{value}\n"));
    assert!(!stdout(&read).contains("hunter2"), "{}", stdout(&read));
}

// ------------------------------------------------------------ FR-CFG-048 ---

#[test]
fn fr_cfg_048_set_refuses_a_write_the_entry_cannot_hold_and_the_hint_repairs_it() {
    // FR-CFG-048: 64, the file byte for byte as it was, a cause naming both
    // keys, and a hint whose command is run here to prove it makes the write
    // legal.
    let sandbox = Sandbox::new();
    let before = "[database.shop]\ndsn = \"mysql://alice@db.example.com/shop\"\n";
    sandbox.project(before);

    let printed = sandbox.run(&["cfg", "set", "database.shop.host", "10.0.1.5"]);
    let written = assert_refused(&printed, 64, "cfg set beside a dsn");
    let cause = line(&written, "cause: ");

    assert!(cause.contains("database.shop.host"), "{written}");
    assert!(cause.contains("database.shop.dsn"), "{written}");
    assert_eq!(sandbox.configuration(), before.as_bytes());

    let hint = line(&written, "hint:  ");
    let repair = hint
        .rsplit_once(": ")
        .expect("the hint carries a command")
        .1;
    assert_eq!(repair, "tpl cfg unset database.shop.dsn");

    let arguments: Vec<&str> = repair.split(' ').skip(1).collect();
    assert_eq!(code(&sandbox.run(&arguments)), 0, "the hint did not run");
    assert_eq!(
        code(&sandbox.run(&["cfg", "set", "database.shop.host", "10.0.1.5"])),
        0,
        "the hint did not make the write legal"
    );
}

#[test]
fn fr_cfg_048_add_refuses_a_dsn_carrying_a_password_beside_a_password_command() {
    // FR-CFG-048 over the third row of FR-CONF-007, which FR-CFG-029 does not
    // separate: password_command is not one of the discrete connection fields.
    let sandbox = Sandbox::new();
    let before = "[core]\n";
    sandbox.project(before);

    let printed = sandbox.run(&[
        "cfg",
        "database",
        "add",
        "shop",
        "--dsn",
        "mysql://alice:hunter2@db.example.com/shop",
        "--password-command",
        "pass db/shop",
    ]);
    let written = assert_refused(&printed, 64, "add with two password sources");
    let cause = line(&written, "cause: ");

    assert!(cause.contains("database.shop.dsn"), "{written}");
    assert!(
        cause.contains("database.shop.password_command"),
        "{written}"
    );
    assert!(!written.contains("hunter2"), "the secret reached a message");
    assert_eq!(sandbox.configuration(), before.as_bytes());

    // The hint is the same command with a DSN that carries no password, and it
    // is run here with that one change.
    assert!(
        line(&written, "hint:  ").contains("tpl cfg database add shop --dsn <url>"),
        "{written}"
    );
    assert_eq!(
        code(&sandbox.run(&[
            "cfg",
            "database",
            "add",
            "shop",
            "--dsn",
            "mysql://alice@db.example.com/shop",
            "--password-command",
            "pass db/shop",
        ])),
        0,
        "the hint did not make the write legal"
    );
}

#[test]
fn fr_cfg_048_update_refuses_a_field_that_cannot_stand_beside_one_it_leaves_alone() {
    // FR-CFG-048 with FR-CFG-020: the fields the flags do not name stay in
    // place, so the write is refused rather than the entry made coherent by
    // removing what the caller never named. The hint is the pair of commands
    // that writes the entry afresh, and both are run.
    let sandbox = Sandbox::new();
    let before = concat!(
        "[database.shop]\n",
        "host = \"db.example.com\"\n",
        "user = \"alice\"\n",
        "password = \"hunter2\"\n",
    );
    sandbox.project(before);

    let printed = sandbox.run(&[
        "cfg",
        "database",
        "update",
        "shop",
        "--dsn",
        "mysql://alice@db.example.com/shop",
    ]);
    let written = assert_refused(&printed, 64, "update to a url beside discrete fields");
    let cause = line(&written, "cause: ");

    assert!(cause.contains("database.shop.dsn"), "{written}");
    assert!(cause.contains("database.shop.host"), "{written}");
    assert_eq!(sandbox.configuration(), before.as_bytes());

    assert!(
        line(&written, "hint:  ")
            .contains("tpl cfg database remove shop, then tpl cfg database add shop --dsn <url>"),
        "{written}"
    );
    assert_eq!(
        code(&sandbox.run(&["cfg", "database", "remove", "shop"])),
        0
    );
    assert_eq!(
        code(&sandbox.run(&[
            "cfg",
            "database",
            "add",
            "shop",
            "--dsn",
            "mysql://alice@db.example.com/shop",
        ])),
        0,
        "the hint did not make the write legal"
    );
}

#[test]
fn fr_cfg_048_the_combinations_the_table_admits_are_written_by_all_three_paths() {
    // FR-CFG-048 refuses what FR-CONF-007 refuses and nothing else: the two
    // admitted rows of the table go through `set`, `add` and `update` alike.
    let sandbox = Sandbox::new();
    sandbox.project("[core]\n");

    for invocation in [
        vec![
            "cfg",
            "database",
            "add",
            "reporting",
            "--host",
            "10.0.1.5",
            "--password-command",
            "pass db/reporting",
        ],
        vec![
            "cfg",
            "database",
            "add",
            "shop",
            "--dsn",
            "mysql://alice@db.example.com/shop",
            "--password-command",
            "pass db/shop",
        ],
        vec!["cfg", "database", "update", "shop", "--tls", "required"],
        vec!["cfg", "set", "database.reporting.user", "reader"],
        vec!["cfg", "set", "core.database", "shop"],
    ] {
        let printed = sandbox.run(&invocation);

        assert_eq!(
            code(&printed),
            0,
            "{invocation:?} was refused: {}",
            stderr(&printed)
        );
    }

    assert_eq!(code(&sandbox.run(&["cfg", "list"])), 0, "the file is valid");
}

// ------------------------------------------------------------ FR-CFG-023 ---

#[test]
fn fr_cfg_023_unset_of_the_block_clears_the_reference_and_unset_of_a_leaf_does_not() {
    // FR-CFG-023: the block of the entry `core.database` names takes the
    // reference with it, silently, in the same rewrite and with no change to
    // the exit code. A leaf leaves the entry in place, so the reference still
    // resolves and the rule does not engage.
    let sandbox = Sandbox::new();
    sandbox.project(concat!(
        "# the note survives both\n",
        "[core]\n",
        "database = \"shop\"\n",
        "\n",
        "[database.shop]\n",
        "host = \"db.example.com\"\n",
        "user = \"alice\"\n",
    ));

    let leaf = sandbox.run(&["cfg", "unset", "database.shop.user"]);

    assert_eq!(code(&leaf), 0, "{}", stderr(&leaf));
    assert_eq!(stdout(&leaf), "");
    assert_eq!(stderr(&leaf), "");
    assert_eq!(
        String::from_utf8(sandbox.configuration()).expect("the file is UTF-8"),
        concat!(
            "# the note survives both\n",
            "[core]\n",
            "database = \"shop\"\n",
            "\n",
            "[database.shop]\n",
            "host = \"db.example.com\"\n",
        )
    );

    let block = sandbox.run(&["cfg", "unset", "database.shop"]);

    assert_eq!(code(&block), 0, "{}", stderr(&block));
    assert_eq!(stdout(&block), "");
    assert_eq!(stderr(&block), "", "the reference is cleared silently");
    assert_eq!(
        String::from_utf8(sandbox.configuration()).expect("the file is UTF-8"),
        "# the note survives both\n[core]\n"
    );
}

#[test]
fn fr_cfg_023_remove_clears_the_reference_and_leaves_one_to_another_entry_alone() {
    // FR-CFG-023 over the command it was first written for, beside the entry it
    // does not name.
    let sandbox = Sandbox::new();
    sandbox.project(concat!(
        "[core]\n",
        "database = \"shop\"\n",
        "\n",
        "[database.shop]\n",
        "host = \"db.example.com\"\n",
        "\n",
        "[database.reporting]\n",
        "host = \"10.0.1.5\"\n",
    ));

    assert_eq!(
        code(&sandbox.run(&["cfg", "database", "remove", "reporting"])),
        0
    );
    assert!(
        String::from_utf8(sandbox.configuration())
            .expect("the file is UTF-8")
            .contains("database = \"shop\""),
        "a reference to another entry was cleared"
    );

    assert_eq!(
        code(&sandbox.run(&["cfg", "database", "remove", "shop"])),
        0
    );
    assert_eq!(
        String::from_utf8(sandbox.configuration()).expect("the file is UTF-8"),
        "[core]\n"
    );
}

// ------------------------------------------------------------ FR-CFG-046 ---

#[test]
fn fr_cfg_046_a_password_command_is_stored_as_the_array_the_quoting_rule_splits_it_into() {
    // FR-CFG-046 with FR-CONF-025: one string on the command line, the array of
    // FR-CONF-023 in the file. A shell metacharacter is a character of an
    // argument and never a separator — the `;` below is one element of the
    // array and starts no second command — which is the write end of the
    // no-shell guarantee FR-CONF-024 makes of the execution.
    let sandbox = Sandbox::new();
    sandbox.project("[core]\n");

    let printed = sandbox.run(&[
        "cfg",
        "database",
        "add",
        "shop",
        "--host",
        "db.example.com",
        "--password-command",
        "security find-generic-password -s 'tpl shop; rm -rf .' -w",
    ]);

    assert_eq!(code(&printed), 0, "{}", stderr(&printed));
    assert_eq!(
        String::from_utf8(sandbox.configuration()).expect("the file is UTF-8"),
        concat!(
            "[core]\n",
            "\n",
            "[database.shop]\n",
            "host = \"db.example.com\"\n",
            "password_command = [\"security\", \"find-generic-password\", \"-s\", ",
            "\"tpl shop; rm -rf .\", \"-w\"]\n",
        )
    );
}
