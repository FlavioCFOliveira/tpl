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
//! | That a second `tpl init` is refused `73` and changes nothing | `FR-PROJ-014`, `BR-ERR-001` |
//! | That a destination that cannot be created is refused `73` and changes nothing | `FR-PROJ-015`, `BR-ERR-001` |
//! | That the password reaches neither stream, however the entry delivers it | `BR-SEC-003` |
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
//! **The one body that needs a server.** The sentinel sweep of `BR-SEC-003`
//! runs every command of the tree, and the commands that reach a server have to
//! reach one for the sweep to say anything about what they write on the way. It
//! is therefore gated on `scripts/mariadb/status.sh` through
//! [`fixture`](../fixture/index.html), on the terms
//! [`outside_the_process`](../outside_the_process/index.html) states: `0` runs
//! the half that needs a server, `1` skips it with a printed reason, and `2` —
//! half a fixture — fails the run. The half that needs none runs always, which
//! is what `BR-SEC-003` says of it. Every other body here needs no server and
//! is not gated at all.
//!
//! # The sentinel travels all three credential paths
//!
//! `BR-SEC-003` names a literal password, and a literal password is one of the
//! three ways an entry can supply one. The other two are exercised here beside
//! it, each as a sweep of its own over the whole tree:
//!
//! | Arrangement | What it puts on the credential path | Requirement |
//! |---|---|---|
//! | `password = "<sentinel>"` | The value the file itself carries | `FR-CONF-002` |
//! | `password = "${VAR}"` | The value the **environment** carries, reaching the process through the expansion | `FR-CONF-015`, `FR-CONF-021` |
//! | `password_command = [...]` | The trimmed standard output of a **child process** | `FR-CONF-023` … `FR-CONF-033` |
//!
//! Each carries its own control, because a sweep over a credential that never
//! reached the process proves nothing: the reference is shown to be `78` while
//! undefined and to reach the connect phase once defined, and the child is
//! shown to be `78` while absent and to reach the connect phase once present.
//!
//! **What this file deliberately does not reach.** One area of the
//! configuration is exercised by unit tests rather than here, because no body
//! here arranges it: the composition of `--timeout` with a phase deadline
//! (`FR-GLOB-012`), which the unit tests of `deadline` cover.

#[path = "support/fixture.rs"]
mod fixture;
#[path = "support/sandbox.rs"]
mod sandbox;

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
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

/// Every path under `directory`, relative to it, with the bytes of each file
/// and `None` for each folder.
///
/// It is what makes *"SHALL change nothing"* an assertion rather than a claim:
/// two snapshots taken around an invocation are equal only if no file changed,
/// none was added, and none was removed.
///
/// # Panics
///
/// Panics when `directory` cannot be walked or one of its files cannot be read.
fn snapshot(directory: &Path) -> BTreeMap<PathBuf, Option<Vec<u8>>> {
    fn walk(root: &Path, at: &Path, into: &mut BTreeMap<PathBuf, Option<Vec<u8>>>) {
        for entry in std::fs::read_dir(at).expect("the folder is there") {
            let entry = entry.expect("the entry is readable");
            let path = entry.path();
            let relative = path
                .strip_prefix(root)
                .expect("the walk started at the root")
                .to_path_buf();

            if entry.file_type().expect("the kind is readable").is_dir() {
                into.insert(relative, None);
                walk(root, &path, into);
            } else {
                into.insert(
                    relative,
                    Some(std::fs::read(&path).expect("the file is readable")),
                );
            }
        }
    }

    let mut found = BTreeMap::new();

    walk(directory, directory, &mut found);

    found
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

// ------------------------------------------------------------ FR-PROJ-014 ---

#[test]
fn br_err_001_a_second_init_at_a_destination_that_already_carries_tpl_is_73() {
    // BR-ERR-001 makes at least one **integration** test per exit code part of
    // the definition of done, and FR-ERR-003 reserves `73` for `tpl init`, so
    // the only invocation that can be observed carrying it is an `init`. The
    // condition is FR-PROJ-014's: a destination that already holds a `.tpl`.
    //
    // FR-PROJ-014 states two obligations and both are read here — the code, and
    // that the refused run "SHALL change nothing": it does not merge, complete
    // partially, or overwrite. The second is the one a unit test over the
    // mapping cannot reach, and it is asserted over the whole of what the first
    // `init` left, folder by folder and byte by byte.
    //
    // The `cause` is read against the `73` row of FR-ERR-034, which obliges it
    // to name the path that could not be created and to say which of the two
    // obstacles stopped it: an existing `.tpl`, or a failure the filesystem
    // reported. The destination is spelled twice — the working directory, and a
    // path named as an argument — so that the path the message carries is shown
    // to be the destination of that invocation and not a constant.
    let sandbox = Sandbox::new();

    assert_eq!(code(&sandbox.run(&["init"])), 0);
    assert_eq!(code(&sandbox.run(&["init", "nested"])), 0);

    for (arguments, destination) in [
        (&["init"][..], ".tpl"),
        (&["init", "nested"][..], "nested/.tpl"),
    ] {
        let spelled = format!("tpl {}", arguments.join(" "));
        let folder = sandbox.path(destination);
        let before = snapshot(&folder);

        let written = assert_refused(&sandbox.run(arguments), 73, &spelled);
        let cause = line(&written, "cause: ");

        assert!(
            cause.contains(destination),
            "{spelled}: the cause names no destination: {cause:?}"
        );
        assert!(
            cause.contains("exists"),
            "{spelled}: the cause does not say the obstacle was an existing .tpl: {cause:?}"
        );

        assert_eq!(
            snapshot(&folder),
            before,
            "{spelled} changed {destination}, which FR-PROJ-014 forbids"
        );
        assert_eq!(mode_of(&folder.join(".cfg")), MODE);
    }
}

#[test]
fn br_err_001_a_destination_that_cannot_be_created_is_73() {
    // BR-ERR-001 asks for at least one **integration** test per exit code, and
    // `73` has two conditions rather than one: FR-PROJ-014's, a destination
    // that already holds a `.tpl`, which the body above covers; and
    // FR-PROJ-015's, a destination that cannot be created, which is this one.
    // The `73` row of FR-ERR-034 obliges the `cause` to say which of the two
    // obstacles stopped it, so a test of one of them leaves the other half of
    // that obligation unread.
    //
    // The obstacle is a **regular file** standing where a directory would have
    // to be. It is chosen over an unwritable directory because it needs no
    // privilege to arrange, behaves identically on both supported families, and
    // is not defeated by a test run as a user the permission bits do not reach.
    //
    // FR-PROJ-013 is the control beside it: `tpl init` creates the destination
    // and every missing parent, so the failure below is the obstacle and not
    // the depth of the path.
    let sandbox = Sandbox::new();

    assert_eq!(code(&sandbox.run(&["init", "deep/under/here"])), 0);
    assert!(sandbox.path("deep/under/here/.tpl").is_dir());

    sandbox.write("blocking", "not a directory\n");

    let destination = "blocking/under-a-file";
    let written = assert_refused(
        &sandbox.run(&["init", destination]),
        73,
        "tpl init under a regular file",
    );
    let cause = line(&written, "cause: ");

    assert!(
        cause.contains(destination),
        "the cause names no destination: {cause:?}"
    );
    assert!(
        !cause.contains("exists"),
        "the cause reports the obstacle of FR-PROJ-014, and this is FR-PROJ-015's: {cause:?}"
    );

    // FR-PROJ-015 changes nothing either: the obstacle is still the file it
    // was, and nothing was created beside it.
    assert_eq!(
        std::fs::read_to_string(sandbox.path("blocking")).expect("the obstacle is still there"),
        "not a directory\n"
    );
    assert!(!sandbox.path(destination).exists());
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
            "not a configuration key",
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

// ------------------------------------------------------------- BR-SEC-003 ---

/// The sentinel password of `BR-SEC-003`.
///
/// It is distinctive by construction: no help text, no diagnostic, no error
/// message and no document this project emits can carry these bytes by
/// coincidence, so a search for them over a stream answers exactly the question
/// the rule asks and never a broader one.
const SENTINEL: &str = "tpl-sentinel-1f9c4b7e-no-byte-of-any-stream-carries-this";

/// The one name everything in the swept project is given: the database entry,
/// and the template.
///
/// The sweep takes the commands from the tree and fills each required argument
/// with this name, so naming the project's own contents after it is what
/// carries each command as far as it can go — `tpl cfg database show x` prints
/// an entry rather than failing to find one, and `tpl render x` resolves a
/// template and opens a connection rather than stopping at `66`. A per-command
/// table of arguments would reach further still, and would stop reaching a
/// command the day one is added, which is the property `BR-SEC-003` exists for.
const SWEPT: &str = "x";

/// The server-side database the swept entry selects.
///
/// It is the schema `scripts/mariadb/setup.sql` creates, so that the entry
/// describes a read that would succeed if the credential were right.
const SWEPT_SCHEMA: &str = "freight";

/// An address no server answers on: nothing listens on port 1.
///
/// It is not a fixture address and is not read from the harness, because the
/// half of the sweep that uses it is the half `BR-SEC-003` says needs no
/// container at all.
const UNREACHABLE: &str = "127.0.0.1:1";

/// Maximum verbosity: three occurrences of `-v`, which `FR-GLOB-014` makes
/// `TRACE` and which `BR-SEC-003` requires every command of the sweep to be run
/// at.
const LOUDEST: [&str; 3] = ["-v", "-v", "-v"];

/// The environment variable the second arrangement delivers the sentinel
/// through (`FR-CONF-015`, `FR-CONF-021`).
///
/// It is not a `TPL_` name and must not become one: `FR-CONF-030` admits no
/// environment layer in the precedence, so the only thing a variable does here
/// is supply the value of a key the file already carries.
const VARIABLE: &str = "SENTINEL_PASSWORD_OF_BR_SEC_003";

/// The script the third arrangement delivers the sentinel through, relative to
/// the sandbox.
///
/// A `password_command` child inherits `tpl`'s working directory, which is the
/// sandbox root, so the entry names the script by this relative path and the
/// file itself carries the sentinel — which keeps it out of `.tpl/.cfg`, for
/// the reason [`sentinel_project`] states.
const PRODUCER: &str = "password-of-br-sec-003.sh";

/// The first of `candidates` that exists on this system.
///
/// The two supported families put a POSIX shell in one of two places, and a
/// body that silently skipped would report nothing at all. It is the helper
/// `src/project/password.rs` uses for the same reason, written again here
/// because that one is a unit test's private helper.
fn tool(candidates: &[&str]) -> String {
    candidates
        .iter()
        .find(|path| Path::new(path).exists())
        .map(|path| (*path).to_owned())
        .unwrap_or_else(|| panic!("none of {candidates:?} exists on this system"))
}

/// The POSIX shell the producer of [`PRODUCER`] is run by.
fn shell() -> String {
    tool(&["/bin/sh", "/usr/bin/sh"])
}

/// A `.tpl/.cfg` selecting one entry at `address` whose password is the
/// sentinel, written as a literal, which is the arrangement `BR-SEC-003` names.
///
/// # Panics
///
/// Panics when `address` is not `host:port`.
fn sentinel_configuration(address: &str) -> String {
    entry(address, &format!("password = \"{SENTINEL}\"\n"))
}

/// The same entry with the password delivered through `${VAR}`
/// (`FR-CONF-015`, `FR-CONF-021`).
///
/// The file carries the **reference** and the environment carries the value, so
/// the sentinel enters the process only where the expansion runs. `FR-CFG-014`
/// and the third row of `FR-CFG-021` keep the printers showing the reference as
/// written, which is what makes this arrangement a second credential path
/// rather than a second spelling of the first.
///
/// # Panics
///
/// Panics when `address` is not `host:port`.
fn sentinel_by_variable(address: &str) -> String {
    entry(address, &format!("password = \"${{{VARIABLE}}}\"\n"))
}

/// The same entry with the password produced by a child process
/// (`FR-CONF-023` … `FR-CONF-033`).
///
/// The child's **standard output is the password**, per `FR-CONF-027`, so the
/// sentinel lives in [`PRODUCER`] and not in `.tpl/.cfg`. The command is stored
/// as the array `FR-CONF-023` fixes, and it is run directly and without a shell
/// interpreting it, per `FR-CONF-024`: the shell named here is the program
/// being executed, and the script is its argument.
///
/// # Panics
///
/// Panics when `address` is not `host:port`, and when no POSIX shell is found.
fn sentinel_by_command(address: &str) -> String {
    entry(
        address,
        &format!("password_command = [\"{}\", \"{PRODUCER}\"]\n", shell()),
    )
}

/// One entry reaching `address` as `root`, with `credential` supplying the
/// password however the arrangement supplies it.
///
/// # Panics
///
/// Panics when `address` is not `host:port`.
fn entry(address: &str, credential: &str) -> String {
    let (host, port) = address
        .rsplit_once(':')
        .expect("an address is written host:port");

    format!(
        "[core]\ndatabase = \"{SWEPT}\"\n\n\
         [database.{SWEPT}]\nhost = \"{host}\"\nport = {port}\n\
         user = \"root\"\n{credential}\
         database = \"{SWEPT_SCHEMA}\"\ntls = \"disabled\"\n"
    )
}

/// A sandbox holding that configuration and one template, both named [`SWEPT`].
///
/// It also holds the producer of [`sentinel_by_command`], at [`PRODUCER`]. It
/// is written for every arrangement rather than for that one, so that a sweep
/// differs from another sweep in its `.tpl/.cfg` and in the environment it is
/// given and in nothing else — and the two arrangements that do not name the
/// script leave it unread, which is the state a sandbox with an unused file is
/// in.
///
/// **The sentinel lives in the script and never in `.tpl/.cfg`.** `FR-CFG-021`
/// redacts `password` and `dsn` and nothing else, and it is right not to
/// redact `password_command`: the command is not the credential, its **standard
/// output** is, per `FR-CONF-027`. Writing the sentinel as an argument of the
/// command would therefore put it in a file `tpl cfg list` prints literally,
/// per `FR-CFG-013`, and the sweep would be failing a command that leaked
/// nothing it was not told to print.
fn sentinel_project(configuration: &str) -> Sandbox {
    let sandbox = Sandbox::new();

    sandbox.project(configuration);
    sandbox.write(
        &format!(".tpl/templates/{SWEPT}.jinja"),
        "{{ database.name }}\n",
    );
    sandbox.write(PRODUCER, &format!("printf %s {SENTINEL}\n"));

    sandbox
}

/// Every command of the tree of `FR-CLI-002`, as the invocation that reaches
/// it: the node's path, followed by [`SWEPT`] for each argument it requires.
///
/// The tree is the binary's own account of its surface, introspected from the
/// parser rather than maintained beside it, per `FR-HELP-021`. Reading it here
/// rather than writing the commands out is what makes this a sentinel: a
/// command added to the parser enters the sweep on the day it is added, and a
/// command that leaks on a path nobody thought was a credential path is caught
/// without anyone having thought of it.
///
/// # Panics
///
/// Panics when `tpl help --format json` does not answer with the tree.
fn every_invocation(sandbox: &Sandbox) -> Vec<Vec<String>> {
    let printed = sandbox.run(&["help", "--format", "json"]);

    assert_eq!(code(&printed), 0, "{}", stderr(&printed));

    let tree: serde_json::Value =
        serde_json::from_slice(&printed.stdout).expect("the command tree is a JSON document");

    // The root of the tree is `tpl` itself, which `data.commands` does not
    // carry because it is not a subcommand. A bare `tpl` is an invocation of
    // it, and the sweep is over every command of the tree.
    let mut invocations = vec![Vec::new()];

    invocations.extend(
        tree["data"]["commands"]
            .as_array()
            .expect("data.commands is an array")
            .iter()
            .map(|entry| {
                let mut invocation: Vec<String> = entry["path"]
                    .as_array()
                    .expect("every entry carries a path")
                    .iter()
                    .map(|segment| {
                        segment
                            .as_str()
                            .expect("a path segment is a string")
                            .to_owned()
                    })
                    .collect();
                let required = entry["arguments"]
                    .as_array()
                    .expect("every entry carries its arguments")
                    .iter()
                    .filter(|argument| argument["required"] == true)
                    .count();

                invocation.extend(std::iter::repeat_n(SWEPT.to_owned(), required));

                invocation
            }),
    );

    invocations
}

/// Whether `bytes` carries the sentinel anywhere in it.
fn carries_sentinel(bytes: &[u8]) -> bool {
    bytes
        .windows(SENTINEL.len())
        .any(|window| window == SENTINEL.as_bytes())
}

/// Runs every command of the tree at maximum verbosity against a project
/// configured with `configuration`, asserts that neither stream of any of them
/// carries the sentinel, and returns the exit codes observed.
///
/// Each command gets a **fresh** project, because the sweep includes the four
/// commands that rewrite `.tpl/.cfg` — `cfg set`, `cfg unset`,
/// `cfg database add` and `cfg database remove` — and a sweep sharing one
/// project could disarm itself part-way through by removing the entry the
/// sentinel lives in.
///
/// `environment` is what each run is given and is empty for every arrangement
/// but the `${VAR}` one, whose value lives nowhere else. It is applied over a
/// **cleared** environment, as every run in this file is.
///
/// `arrangement` names the case under test, so that a failure says which one
/// saw the leak.
fn sweep(configuration: &str, environment: &[(&str, &str)], arrangement: &str) -> Vec<i32> {
    let invocations = every_invocation(&sentinel_project(configuration));
    let mut codes = Vec::with_capacity(invocations.len());

    for invocation in &invocations {
        let sandbox = sentinel_project(configuration);
        let mut arguments: Vec<&str> = LOUDEST.to_vec();

        arguments.extend(invocation.iter().map(String::as_str));

        let spelled = format!("tpl {}", arguments.join(" "));
        let printed = sandbox.run_from(sandbox.root(), environment, &arguments);

        for (stream, bytes) in [("stdout", &printed.stdout), ("stderr", &printed.stderr)] {
            assert!(
                !carries_sentinel(bytes),
                "{spelled}, {arrangement}, wrote the sentinel password to {stream}:\n{}",
                String::from_utf8_lossy(bytes)
            );
        }

        codes.push(code(&printed));
    }

    codes
}

#[test]
fn br_sec_003_no_command_of_the_tree_writes_the_sentinel_at_maximum_verbosity() {
    // BR-SEC-003, which `specification/security.md` owns outright: "A known
    // sentinel password SHALL never appear in any byte `tpl` writes", tested in
    // the form that rule fixes — a database entry configured with a distinctive
    // sentinel, every command of the tree of FR-CLI-002 run against it at
    // maximum verbosity, and no byte of stdout and no byte of stderr, from any
    // of them, carrying the sentinel.
    //
    // **What it catches.** Each rule in the *Credentials* section of that file
    // closes one path — FR-SEC-003 the two printers, FR-SEC-005 the diagnostic
    // stream, FR-SEC-006 error messages, FR-CONF-032 a child's stderr — and a
    // test of each proves only that the path it names is closed. This proves
    // the property those prohibitions exist to produce, over the whole surface
    // at once, and it keeps proving it when a path is added: the commands come
    // from the tree the parser introspects, so a command, a diagnostic or an
    // error message that begins to echo the resolved entry fails here whether
    // or not anyone thought it was on the credential path. Maximum verbosity is
    // where FR-GLOB-018 and FR-SEC-005 are least likely to be reviewed, which
    // is the rule's own reason for fixing it.
    //
    // **The one invocation the sweep does not make.** FR-SEC-004 and BR-CFG-002
    // make `tpl cfg get` the single deliberate exception to redaction, because
    // it is a directed read of a named key; `tpl cfg get database.x.password`
    // therefore prints the password and is sanctioned in doing so. It is the
    // control below rather than a member of the sweep, and it is what makes the
    // sweep mean anything: the same byte search, over the same stream, is shown
    // to find the sentinel when it is there.
    let control = sentinel_project(&sentinel_configuration(UNREACHABLE));
    let printed = control.run(&["cfg", "get", &format!("database.{SWEPT}.password")]);

    assert_eq!(code(&printed), 0, "{}", stderr(&printed));
    assert!(
        carries_sentinel(&printed.stdout),
        "the directed read of FR-SEC-004 did not print the sentinel, so the search below \
         would not have found it either: {:?}",
        stdout(&printed)
    );

    // The half that needs no server, which is most of the tree: every command
    // that reads `.tpl/.cfg` without connecting, and every command that does
    // connect stopped at the refusal of a TCP connect. BR-SEC-003 says this
    // half needs no container, so it is not gated.
    let codes = sweep(
        &sentinel_configuration(UNREACHABLE),
        &[],
        "a literal password, against an address no server answers on",
    );

    assert!(
        codes.contains(&69),
        "no command of the sweep reached the connect phase, so the sweep says nothing \
         about what the connection path writes: {codes:?}"
    );

    // The half that needs one. The register makes this row an integration test
    // whose server-reaching commands need a server, and this is what makes the
    // credential leave the process: the entry is carried to a real MariaDB,
    // presented, and refused there with `77`. One server is enough — BR-SEC-003
    // is a property of what `tpl` writes and not of what a server answers, and
    // the row does not make it a cross-series test the way rows 10 and 12 do —
    // so the first series of FR-SRV-015 is the one used.
    let Some(series) = fixture::series("the server half of br_sec_003_no_command_of_the_tree")
    else {
        return;
    };
    let server = series.first().expect("a ready fixture has its series");
    let _exclusive = fixture::exclusive();

    let codes = sweep(
        &sentinel_configuration(server.address()),
        &[],
        "a literal password, against a fixture server that refuses the credential",
    );

    assert!(
        codes.contains(&77),
        "no command of the sweep carried the credential to {}, so the sweep says nothing \
         about what the authentication path writes: {codes:?}",
        server.name()
    );
}

#[test]
fn br_sec_003_a_sentinel_delivered_through_a_variable_reference_reaches_no_stream() {
    // BR-SEC-003 over the second credential path, which the literal arrangement
    // above cannot reach: the file carries `${VAR}` and the **environment**
    // carries the value, per FR-CONF-015, so the sentinel enters the process
    // only where the expansion of FR-CONF-021 runs. Nothing about that path is
    // exercised by a literal password — the value never passes through the
    // reader's expansion, and the printers of FR-CFG-021 print a reference
    // rather than a redaction.
    //
    // **The control comes first, and it is the variable itself.** An undefined
    // reference is FR-CONF-022's `78`, refused before a connection is opened;
    // defined, the same entry reaches the connect phase and is the `69` of an
    // address nothing answers on. The pair is what establishes that the value
    // under test travelled from the environment into the resolved entry — with
    // the variable undefined, the sweep below would be sweeping an invocation
    // that never held the credential at all.
    let configuration = sentinel_by_variable(UNREACHABLE);
    let reaching = ["cfg", "database", "test", SWEPT];
    let control = sentinel_project(&configuration);

    assert_eq!(
        code(&control.run(&reaching)),
        78,
        "FR-CONF-022: an undefined reference is refused before a connection: {}",
        stderr(&control.run(&reaching))
    );
    assert_eq!(
        code(&control.run_from(control.root(), &[(VARIABLE, SENTINEL)], &reaching)),
        69,
        "the defined reference did not reach the connect phase, so the sweep below \
         would say nothing about what the expansion path writes"
    );

    let codes = sweep(
        &configuration,
        &[(VARIABLE, SENTINEL)],
        "a password expanded from ${VAR}, against an address no server answers on",
    );

    assert!(
        codes.contains(&69),
        "no command of the sweep reached the connect phase, so the sweep says nothing \
         about what the expansion path writes: {codes:?}"
    );
}

#[test]
fn br_sec_003_a_sentinel_produced_by_password_command_reaches_no_stream() {
    // BR-SEC-003 over the third credential path: the password is the trimmed
    // standard output of a child process, per FR-CONF-027, so the sentinel
    // enters the process through a pipe and through nothing else. It is the
    // path FR-CONF-032 exists for — the child's standard error goes to the null
    // device and reaches no diagnostic and no message — and the only one of the
    // three where a second process holds the credential.
    //
    // The sentinel lives in the script rather than in `.tpl/.cfg`, for the
    // reason `sentinel_project` states: FR-CFG-021 redacts `password` and `dsn`
    // and rightly not `password_command`, because the command is not the
    // credential.
    //
    // **The control is the child.** With the producer removed the command
    // cannot be executed and the invocation is FR-CONF-033's `78`, before a
    // connection; with it in place the same entry reaches the connect phase and
    // is `69`. The pair establishes that the credential the sweep carries is
    // the one the child produced.
    let configuration = sentinel_by_command(UNREACHABLE);
    let reaching = ["cfg", "database", "test", SWEPT];
    let control = sentinel_project(&configuration);

    assert_eq!(
        code(&control.run(&reaching)),
        69,
        "the child did not produce a password, so the sweep below would say nothing \
         about what the password_command path writes: {}",
        stderr(&control.run(&reaching))
    );

    std::fs::remove_file(control.path(PRODUCER)).expect("the sandbox is writable");

    assert_eq!(
        code(&control.run(&reaching)),
        78,
        "FR-CONF-033: a child that cannot be run is a configuration fault, refused \
         before a connection"
    );

    let codes = sweep(
        &configuration,
        &[],
        "a password produced by password_command, against an address no server answers on",
    );

    assert!(
        codes.contains(&69),
        "no command of the sweep reached the connect phase, so the sweep says nothing \
         about what the password_command path writes: {codes:?}"
    );
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
    // removing what the caller never named. The hint unsets each conflicting
    // key and then writes again (S-03 of #274), and every command is run: the
    // write succeeds, and core.database still names the entry.
    let sandbox = Sandbox::new();
    let before = concat!(
        "[core]\n",
        "database = \"shop\"\n",
        "\n",
        "[database.shop]\n",
        "host = \"db.example.com\"\n",
        "user = \"alice\"\n",
        "password = \"hunter2\"\n",
        "tls = \"disabled\"\n",
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

    let hint = line(&written, "hint:  ");
    assert_eq!(
        hint,
        "remove the keys it conflicts with, then write it again: tpl cfg unset \
         database.shop.host; tpl cfg unset database.shop.user; tpl cfg unset \
         database.shop.password; then tpl cfg database update shop --dsn <dsn>; replace <dsn> \
         with the value you gave --dsn",
        "{written}"
    );
    assert!(!hint.contains("remove shop"), "{written}");

    for key in [
        "database.shop.host",
        "database.shop.user",
        "database.shop.password",
    ] {
        assert_eq!(code(&sandbox.run(&["cfg", "unset", key])), 0, "{key}");
    }
    assert_eq!(
        code(&sandbox.run(&[
            "cfg",
            "database",
            "update",
            "shop",
            "--dsn",
            "mysql://alice@db.example.com/shop",
        ])),
        0,
        "the hint did not make the write legal"
    );

    let kept = String::from_utf8(sandbox.configuration()).expect("the file is UTF-8");
    assert!(kept.contains("database = \"shop\""), "{kept}");
    assert!(kept.contains("tls = \"disabled\""), "{kept}");
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

// ------------------------------------------------ the forty-third edition ---

/// A project whose `.cfg` defines the entry `shop`.
const SHOP: &str = "[core]\n\n[database.shop]\nhost = \"db.example.com\"\n";

#[test]
fn fr_cfg_031_a_port_the_reader_refuses_is_refused_at_the_flag_and_writes_nothing() {
    // Finding E-02: `--port 0` was accepted and wrote a file every later
    // command, the repair included, refused with 78. The flag now admits
    // exactly what the reader admits.
    let sandbox = Sandbox::new();
    sandbox.project("[core]\n");
    let before = sandbox.configuration();

    let written = assert_refused(
        &sandbox.run(&["cfg", "database", "add", "s6", "--host", "h", "--port", "0"]),
        64,
        "--port 0",
    );

    assert!(
        line(&written, "cause: ").contains("a whole number from 1 to 65535"),
        "{written}"
    );
    assert_eq!(sandbox.configuration(), before, "the file was written");
}

#[test]
fn fr_cfg_020_an_update_that_names_no_field_is_64_and_lists_every_field_flag() {
    let sandbox = Sandbox::new();
    sandbox.project(SHOP);
    let before = sandbox.configuration();

    let written = assert_refused(
        &sandbox.run(&["cfg", "database", "update", "shop"]),
        64,
        "update with no flag",
    );

    assert_eq!(
        line(&written, "error: "),
        "nothing to change: tpl cfg database update needs at least one field flag"
    );
    assert_eq!(
        line(&written, "cause: "),
        "no field flag was given for entry 'shop'"
    );
    assert_eq!(
        line(&written, "hint:  "),
        "give at least one of --dsn, --host, --port, --user, --schema, --tls, \
         --password-command, --ca-file, --ca-path"
    );
    assert_eq!(sandbox.configuration(), before, "the file was written");

    // Decided at argument parsing, before the entry is resolved: an entry the
    // file does not define is refused for the same reason.
    assert_refused(
        &sandbox.run(&["cfg", "database", "update", "absent"]),
        64,
        "update of an absent entry with no flag",
    );
}

#[test]
fn fr_cfg_016_an_add_that_says_nowhere_to_connect_names_the_flags_that_do() {
    let sandbox = Sandbox::new();
    sandbox.project("[core]\n");

    let written = assert_refused(
        &sandbox.run(&["cfg", "database", "add", "shop", "--tls", "disabled"]),
        64,
        "add with no connection flag",
    );

    assert!(line(&written, "cause: ").contains("--host"), "{written}");
    assert!(
        line(&written, "hint:  ")
            .starts_with("say where to connect, e.g.: tpl cfg database add shop"),
        "{written}"
    );
}

#[test]
fn fr_cfg_007_a_block_given_to_get_is_64_and_points_at_the_command_that_shows_it() {
    let sandbox = Sandbox::new();
    sandbox.project(SHOP);

    let entry = assert_refused(
        &sandbox.run(&["cfg", "get", "database.shop"]),
        64,
        "an entry",
    );
    assert_eq!(
        line(&entry, "error: "),
        "'database.shop' names a whole entry, not one value"
    );
    assert_eq!(
        line(&entry, "cause: "),
        "tpl cfg get reads one key; database.shop is the block of entry 'shop'"
    );
    assert_eq!(
        line(&entry, "hint:  "),
        "show the entry with: tpl cfg database show shop"
    );

    // Whether or not the file carries the block, the form decides: 64, and
    // the listing where no entry of that name exists.
    for block in ["core", "database", "database.absent"] {
        let written = assert_refused(&sandbox.run(&["cfg", "get", block]), 64, block);
        assert_eq!(
            line(&written, "hint:  "),
            "show every key and its value with: tpl cfg list",
            "{block}"
        );
    }
}

#[test]
fn fr_proj_008_a_tpl_dir_that_names_nothing_is_78_and_describes_no_walk() {
    let sandbox = Sandbox::new();
    let named = sandbox.path("absent/.tpl");
    let spelled = named.to_string_lossy().into_owned();

    let written = assert_refused(
        &sandbox.run(&["--tpl-dir", &spelled, "template", "list"]),
        78,
        "--tpl-dir naming nothing",
    );

    assert_eq!(
        line(&written, "error: "),
        format!("the folder named by --tpl-dir does not exist: {spelled}")
    );
    assert_eq!(
        line(&written, "cause: "),
        format!("--tpl-dir disabled the upward search; nothing exists at {spelled}")
    );
    assert_eq!(
        line(&written, "hint:  "),
        format!(
            "correct --tpl-dir, or create the project with: tpl init {}",
            sandbox.path("absent").display()
        )
    );
    assert!(!written.contains("walk"), "{written}");
}

#[test]
fn fr_out_009_pretty_without_json_names_the_flag_it_needs_and_not_a_flag_never_written() {
    let sandbox = Sandbox::new();
    sandbox.project("[core]\n");

    let written = assert_refused(
        &sandbox.run(&["template", "list", "--pretty"]),
        64,
        "--pretty alone",
    );

    assert_eq!(
        line(&written, "error: "),
        "'--pretty' needs '--format json'"
    );
    assert!(!written.contains("--format text"), "{written}");
    assert_eq!(
        line(&written, "hint:  "),
        "add --format json: tpl template list --format json --pretty; or drop --pretty"
    );
}

#[test]
fn br_err_004_a_fault_in_the_file_is_repaired_by_an_edit_and_never_by_tpl_cfg() {
    // Finding E-01: every `tpl cfg` command refuses a file that fails step 3
    // of FR-ERR-006, so a hint naming one could never succeed. The hint names
    // the file, by its absolute path, the position and the edit.
    let sandbox = Sandbox::new();
    sandbox.project("[core]\n\n[database.s6]\nhost = \"h\"\nport = 0\n");
    let file = sandbox.path(".tpl/.cfg");
    let file = std::fs::canonicalize(&file).expect("the file is there");

    let written = assert_refused(&sandbox.run(&["cfg", "list"]), 78, "port 0 in the file");
    let hint = line(&written, "hint:  ");

    assert!(
        hint.starts_with(&format!("edit {} at line 5", file.display())),
        "{hint}"
    );
    assert!(hint.contains("no tpl cfg command runs"), "{hint}");
    assert!(!hint.contains("tpl cfg set"), "{hint}");

    // Two keys that cannot stand together: the edit names both.
    let sandbox = Sandbox::new();
    sandbox.project("[core]\n\n[database.x]\ndsn = \"mysql://h/d\"\nhost = \"h\"\n");
    let written = assert_refused(&sandbox.run(&["cfg", "list"]), 78, "dsn beside host");

    assert!(
        line(&written, "hint:  ").contains("under [database.x], delete either dsn or host"),
        "{written}"
    );
}

#[test]
fn fr_err_019_a_known_key_in_the_wrong_table_is_suggested_under_its_own() {
    // Finding E-17: `password_timeout` written above `[core]`.
    let sandbox = Sandbox::new();
    sandbox.project("password_timeout = 1\n[core]\n");

    let written = assert_refused(&sandbox.run(&["cfg", "list"]), 78, "a key above [core]");

    assert!(
        line(&written, "hint:  ").starts_with("did you mean 'core.password_timeout'?"),
        "{written}"
    );
}

#[test]
fn fr_err_034_a_toml_fault_states_what_the_parser_expected() {
    let sandbox = Sandbox::new();
    sandbox.project("[core\n");

    let written = assert_refused(&sandbox.run(&["cfg", "list"]), 78, "an unclosed header");
    let cause = line(&written, "cause: ");

    assert!(
        cause.contains("the TOML parser stopped at line 1"),
        "{cause}"
    );
    assert!(
        cause
            .rsplit_once(": ")
            .is_some_and(|(_, reason)| !reason.is_empty()),
        "{cause}"
    );
}

#[test]
fn fr_cli_017_a_value_beginning_with_a_dash_is_told_to_follow_the_terminator() {
    // Finding E-11: a one-letter token is not matched against every
    // one-letter flag, and a command that takes a value says how to give it.
    let sandbox = Sandbox::new();
    sandbox.project("[core]\n");

    let written = assert_refused(
        &sandbox.run(&["cfg", "set", "core.query_timeout", "-5"]),
        64,
        "a negative value",
    );
    let hint = line(&written, "hint:  ");

    assert!(!hint.contains("did you mean"), "{hint}");
    assert!(hint.contains("tpl help cfg set"), "{hint}");
    assert!(hint.contains("write -- before it"), "{hint}");
}

#[test]
fn fr_proj_026_init_warns_that_tpl_dir_has_no_effect_and_never_looks_at_its_path() {
    // FR-PROJ-026: the flag is accepted, the destination is FR-PROJ-012's,
    // one warning line precedes everything else on stderr, and the exit code
    // is the one the same invocation without the flag returns.
    const WARNING: &str = "warning: --tpl-dir has no effect on tpl init; it takes its \
                           destination as an operand: tpl init <path>";
    let sandbox = Sandbox::new();
    let named = sandbox.path("elsewhere/.tpl");
    let flag = named.to_str().expect("the sandbox path is UTF-8");

    let printed = sandbox.run(&["init", "--tpl-dir", flag]);
    assert_eq!(code(&printed), 0, "{}", stderr(&printed));
    assert_eq!(stderr(&printed), format!("{WARNING}\n"));
    assert!(sandbox.path(".tpl/.cfg").is_file());
    assert!(
        !sandbox.path("elsewhere").exists(),
        "the flag's path was created"
    );

    // The same invocation again is FR-PROJ-014's 73, with the warning first.
    let printed = sandbox.run(&["init", "--tpl-dir", flag]);
    let written = stderr(&printed);
    assert_eq!(code(&printed), 73, "{written}");
    assert_eq!(written.lines().next(), Some(WARNING));
    assert!(
        !written.contains(flag),
        "the value is written back: {written}"
    );

    // -q suppresses it; the positional still names the destination.
    let printed = sandbox.run(&["-q", "init", "nested", "--tpl-dir", flag]);
    assert_eq!(code(&printed), 0, "{}", stderr(&printed));
    assert!(printed.stderr.is_empty(), "{}", stderr(&printed));
    assert!(sandbox.path("nested/.tpl/.cfg").is_file());
}

// ------------------------------------------------------------ #274 ---------

#[test]
fn s_04_an_empty_password_command_array_is_named_as_empty() {
    let sandbox = Sandbox::new();
    sandbox.project("[database.x]\nhost = \"h\"\ndatabase = \"d\"\npassword_command = []\n");

    let written = assert_refused(
        &sandbox.run(&["-d", "x", "schema", "tables"]),
        78,
        "an empty password_command",
    );

    assert_eq!(
        line(&written, "error: "),
        "database.x.password_command is an empty array"
    );
    let cause = line(&written, "cause: ");
    assert!(
        cause.ends_with(
            "declares database.x.password_command as an empty array; this key takes an array \
             of at least one string, the program first"
        ),
        "{written}"
    );
    assert!(!cause.contains("an an"), "{written}");
}

#[test]
fn s_10_key_suggestions_stay_in_the_named_entry_and_find_a_core_key_under_an_entry() {
    let sandbox = Sandbox::new();
    sandbox.project(concat!(
        "[database.p]\nhost = \"h\"\n\n",
        "[database.b]\nhost = \"h\"\n\n",
        "[database.c]\nhost = \"h\"\n",
    ));

    let written = assert_refused(
        &sandbox.run(&["cfg", "set", "database.p.hots", "x"]),
        64,
        "a misspelled key of entry p",
    );
    assert!(
        line(&written, "hint:  ").starts_with("did you mean 'database.p.host'? list"),
        "{written}"
    );

    let written = assert_refused(
        &sandbox.run(&["cfg", "set", "database.p.password_timeout", "5"]),
        64,
        "a core key under an entry",
    );
    assert!(
        line(&written, "hint:  ").starts_with("did you mean 'core.password_timeout'?"),
        "{written}"
    );

    // The same key in the file itself.
    sandbox.project("[database.x]\npassword_timeout = 5\n");
    let written = assert_refused(
        &sandbox.run(&["cfg", "list"]),
        78,
        "a core key in [database.x]",
    );
    assert!(
        line(&written, "hint:  ").starts_with("did you mean 'core.password_timeout'?"),
        "{written}"
    );
}

#[test]
fn s_15_a_project_with_no_entry_is_told_how_to_add_one() {
    let sandbox = Sandbox::new();
    sandbox.project("[core]\n");

    let written = assert_refused(&sandbox.run(&["schema", "tables"]), 78, "no entry at all");
    assert_eq!(
        line(&written, "hint:  "),
        "this project has no database entry; add one with: tpl cfg database add <name> --host \
         <host> --user <user> --schema <database>"
    );

    // With an entry to select, the hint still says how to select it.
    sandbox.project("[database.shop]\nhost = \"h\"\ndatabase = \"d\"\n");
    let written = assert_refused(&sandbox.run(&["schema", "tables"]), 78, "none selected");
    assert!(
        line(&written, "hint:  ").starts_with("select an entry with -d <entry>"),
        "{written}"
    );
}

#[test]
fn s_13_a_failed_password_command_is_written_out_to_run_directly() {
    let sandbox = Sandbox::new();
    sandbox.project(
        "[database.p]\nhost = \"127.0.0.1\"\ndatabase = \"d\"\npassword_command = [\"false\"]\n",
    );

    let written = assert_refused(
        &sandbox.run_from(
            sandbox.root(),
            &[("PATH", "/usr/bin:/bin")],
            &["-d", "p", "schema", "tables"],
        ),
        78,
        "a password_command that fails",
    );
    assert_eq!(
        line(&written, "hint:  "),
        "tpl discards the command's standard error, so run it directly to see why it failed: false"
    );
}

#[test]
fn rmp_274_a_non_string_element_of_password_command_is_named_by_its_index() {
    let sandbox = Sandbox::new();
    sandbox.project(
        "[database.x]\nhost = \"h\"\ndatabase = \"d\"\npassword_command = [\"pass\", 1]\n",
    );

    let written = assert_refused(
        &sandbox.run(&["-d", "x", "schema", "tables"]),
        78,
        "a password_command with an integer element",
    );

    assert_eq!(
        line(&written, "error: "),
        "database.x.password_command holds a non-string element at index 1"
    );
    assert!(
        line(&written, "cause: ").ends_with(
            "declares database.x.password_command with an integer at index 1; every element of \
             this key is a string"
        ),
        "{written}"
    );
    assert!(!written.contains("is not an array"), "{written}");
}

#[test]
fn t_01_a_hint_that_rewrites_a_cfg_command_keeps_every_flag_it_was_given() {
    // Finding T-01 of the third re-audit of rmp #263.
    let sandbox = Sandbox::new();
    sandbox.project("[core]\n");

    let written = assert_refused(
        &sandbox.run(&["cfg", "database", "add", "zz", "--tls", "required"]),
        64,
        "no connection details",
    );
    assert_eq!(
        line(&written, "hint:  "),
        "say where to connect, e.g.: tpl cfg database add zz --tls required --host <host> --user \
         <user> --schema <database>"
    );

    // The project named with --tpl-dir is named again, and --pattern, whose
    // value the character set refuses, is a placeholder the line explains.
    let written = assert_refused(
        &sandbox.run(&[
            "--tpl-dir",
            ".tpl",
            "-d",
            "shop",
            "schema",
            "tables",
            "--pattern",
            "ord%",
            "--pretty",
        ]),
        64,
        "--pretty alone",
    );
    assert_eq!(
        line(&written, "hint:  "),
        "add --format json: tpl --tpl-dir .tpl -d shop schema tables --pattern <pattern> --format \
         json --pretty; replace <pattern> with the value you gave --pattern; or drop --pretty"
    );
}

#[test]
fn t_03_and_t_08_a_password_command_is_one_string_with_every_quote_closed() {
    let sandbox = Sandbox::new();
    sandbox.project("[core]\n");

    let written = assert_refused(
        &sandbox.run(&["cfg", "set", "database.new.password_command", ""]),
        64,
        "an empty command",
    );
    assert_eq!(
        line(&written, "cause: "),
        "the value holds no word, so it names no program to execute; \
         database.new.password_command takes one command line written as one string, which tpl \
         splits into words"
    );
    assert_eq!(
        line(&written, "hint:  "),
        "write the command as one command line, e.g.: tpl cfg set \
         database.new.password_command 'pass db/shop'"
    );

    // A shell refuses an unclosed quote, and so does the split.
    let written = assert_refused(
        &sandbox.run(&["cfg", "set", "database.new.password_command", "pass 'a b"]),
        64,
        "an unclosed quote",
    );
    assert!(
        line(&written, "cause: ").starts_with("the value leaves a quote unclosed"),
        "{written}"
    );
    assert_eq!(sandbox.configuration(), b"[core]\n");
}

#[test]
fn fr_conf_046_a_trailing_backslash_and_a_leading_bracket_are_refused_and_write_nothing() {
    let sandbox = Sandbox::new();
    let file = "[database.shop]\nhost = \"db.example.com\"\n";
    sandbox.project(file);

    for (arguments, condition) in [
        (
            &[
                "cfg",
                "set",
                "database.shop.password_command",
                r#"["pass","db/shop"]"#,
            ][..],
            "the value begins with '[', which is how an array arrives",
        ),
        (
            &[
                "cfg",
                "set",
                "database.shop.password_command",
                r"pass db/shop\",
            ][..],
            "the value ends in a backslash outside quotes",
        ),
        (
            &[
                "cfg",
                "database",
                "update",
                "shop",
                "--password-command=[x] y",
            ][..],
            "the value begins with '['",
        ),
        (
            &[
                "cfg",
                "database",
                "update",
                "shop",
                r"--password-command=pass \",
            ][..],
            "the value ends in a backslash outside quotes",
        ),
    ] {
        let written = assert_refused(&sandbox.run(arguments), 64, &arguments.join(" "));
        assert!(
            line(&written, "cause: ").starts_with(condition),
            "{arguments:?}: {written}"
        );
        assert!(
            line(&written, "hint:  ")
                .starts_with("write the command as one command line, e.g.: tpl "),
            "{arguments:?}: {written}"
        );
        assert_eq!(sandbox.configuration(), file.as_bytes(), "{arguments:?}");
    }

    // A quoted or escaped bracket, or an absolute path, names a program.
    for supplied in ["'[x]/get' db", r"\[x]/get db", "/bin/[ x"] {
        let printed = sandbox.run(&["cfg", "set", "database.shop.password_command", supplied]);
        assert_eq!(code(&printed), 0, "{supplied}: {}", stderr(&printed));
    }
}

#[test]
fn fr_conf_025_inside_double_quotes_a_backslash_before_another_character_is_kept() {
    let sandbox = Sandbox::new();
    sandbox.project("[core]\n");

    let printed = sandbox.run(&[
        "cfg",
        "set",
        "database.shop.password_command",
        r#"get "a\b" "\$x""#,
    ]);
    assert_eq!(code(&printed), 0, "{}", stderr(&printed));

    let printed = sandbox.run(&[
        "cfg",
        "get",
        "database.shop.password_command",
        "--format",
        "json",
    ]);
    assert_eq!(code(&printed), 0, "{}", stderr(&printed));
    let document = String::from_utf8(printed.stdout).expect("UTF-8");
    assert!(document.contains(r#"["get","a\\b","$x"]"#), "{document}");
}

#[test]
fn fr_cfg_007_a_slip_in_an_unset_key_is_suggested_and_said_to_be_unset() {
    let sandbox = Sandbox::new();
    sandbox.project("[core]\n");

    for command in ["get", "unset"] {
        let written = assert_refused(
            &sandbox.run(&["cfg", command, "core.conect_timeout"]),
            66,
            command,
        );
        assert_eq!(
            line(&written, "hint:  "),
            "did you mean 'core.connect_timeout'? .tpl/.cfg does not set it, so its default \
             applies; list every key, its type and its default with: tpl help cfg set",
            "{command}"
        );
    }
}

#[test]
fn fr_err_043_a_hint_command_carries_the_tpl_dir_and_the_entry_the_caller_gave() {
    let sandbox = Sandbox::new();
    sandbox.project_at("p", "[core]\n");
    sandbox.project_at("my dir", "[core]\n");

    // --tpl-dir is carried; -d is not, since no cfg command reads an entry.
    let written = assert_refused(
        &sandbox.run(&[
            "--tpl-dir",
            "p/.tpl",
            "-d",
            "shop",
            "cfg",
            "get",
            "core.database",
        ]),
        66,
        "an unset key",
    );
    assert_eq!(
        line(&written, "hint:  "),
        "list the keys that are set with: tpl --tpl-dir p/.tpl cfg list"
    );

    // A path outside the set of FR-ERR-041 is a placeholder the line explains.
    let written = assert_refused(
        &sandbox.run(&[
            "-d",
            "shop",
            "--tpl-dir",
            "my dir/.tpl",
            "template",
            "show",
            "nope",
        ]),
        66,
        "a missing template",
    );
    assert_eq!(
        line(&written, "hint:  "),
        "list the project's templates with: tpl --tpl-dir <tpl-dir> template list; replace \
         <tpl-dir> with the value you gave --tpl-dir"
    );

    // Both flags lead a command on which both have effect, in order, wherever
    // the caller wrote them.
    let written = assert_refused(
        &sandbox.run(&[
            "schema",
            "tables",
            "-d",
            "shop",
            "--pretty",
            "--tpl-dir=p/.tpl",
        ]),
        64,
        "--pretty alone",
    );
    assert_eq!(
        line(&written, "hint:  "),
        "add --format json: tpl --tpl-dir=p/.tpl -d shop schema tables --format json --pretty; \
         or drop --pretty"
    );
}

#[test]
fn t_04_a_port_a_variable_expanded_badly_blames_the_variable() {
    let sandbox = Sandbox::new();
    sandbox.project("[database.x]\nhost = \"127.0.0.1\"\nport = \"${P}\"\ndatabase = \"d\"\n");

    let written = assert_refused(
        &sandbox.run_from(
            sandbox.root(),
            &[("PATH", "/usr/bin:/bin"), ("P", "99999")],
            &["-d", "x", "schema", "tables"],
        ),
        78,
        "a port the environment makes invalid",
    );
    assert!(
        line(&written, "cause: ").contains(
            "writes database.x.port as ${P}, which the environment expands to 99999; this key \
             takes a TCP port between 1 and 65535"
        ),
        "{written}"
    );
    assert_eq!(
        line(&written, "hint:  "),
        "set P to a TCP port between 1 and 65535, e.g.: export P=3306"
    );
}

#[test]
fn t_05_cfg_get_tells_an_unknown_name_from_an_unset_key_and_gives_the_default() {
    let sandbox = Sandbox::new();
    sandbox.project("[core]\n");

    let written = assert_refused(&sandbox.run(&["cfg", "get", "nope"]), 66, "no such key");
    assert_eq!(
        line(&written, "error: "),
        "'nope' is not a configuration key"
    );
    assert_eq!(
        line(&written, "hint:  "),
        "list every key, its type and its default with: tpl help cfg set"
    );

    let written = assert_refused(
        &sandbox.run(&["cfg", "get", "core.connect_timeout"]),
        66,
        "an unset key",
    );
    assert_eq!(
        line(&written, "error: "),
        "'core.connect_timeout' is a configuration key that .tpl/.cfg does not set; tpl uses \
         its default, 10"
    );

    let written = assert_refused(
        &sandbox.run(&["cfg", "unset", "core.database"]),
        66,
        "an unset key without a default",
    );
    assert_eq!(
        line(&written, "error: "),
        "'core.database' is a configuration key that .tpl/.cfg does not set, and it has no \
         default"
    );
}

#[test]
fn t_07_and_t_10_the_generated_configuration_says_what_its_commented_values_are() {
    let sandbox = Sandbox::new();
    let printed = sandbox.run(&["init"]);
    assert_eq!(code(&printed), 0, "{}", stderr(&printed));

    let written = String::from_utf8(sandbox.configuration()).expect("UTF-8");
    assert!(
        written.contains("shown with its\n# default, or with an example where it has none."),
        "{written}"
    );
    assert!(
        written.contains("per query reading the database"),
        "{written}"
    );
    assert!(!written.contains("catalogue"), "{written}");
}

#[test]
fn u_01_a_key_of_the_space_the_file_does_not_set_is_offered_no_other_key() {
    // Finding U-01 of the fourth re-audit of rmp #263: the caller spelt the
    // key right, so no "did you mean" names another one (FR-ERR-019).
    let sandbox = Sandbox::new();
    sandbox.project("[database.shop]\nhost = \"h\"\ndatabase = \"shop\"\n");

    for arguments in [
        &["cfg", "get", "core.database"][..],
        &["cfg", "get", "database.shop.port"][..],
        &["cfg", "unset", "database.shop.port"][..],
    ] {
        let written = assert_refused(&sandbox.run(arguments), 66, &arguments.join(" "));
        assert!(!written.contains("did you mean"), "{written}");
        assert!(
            line(&written, "error: ")
                .contains("is a configuration key that .tpl/.cfg does not set"),
            "{written}"
        );
        assert_eq!(
            line(&written, "hint:  "),
            "list the keys that are set with: tpl cfg list"
        );
    }
    let written = assert_refused(
        &sandbox.run(&["cfg", "get", "database.shop.port"]),
        66,
        "an unset port",
    );
    assert!(
        line(&written, "error: ").ends_with("tpl uses its default, 3306"),
        "{written}"
    );

    // A name that is not a key is still offered the key it resembles.
    let written = assert_refused(
        &sandbox.run(&["cfg", "get", "database.shop.hots"]),
        66,
        "a misspelt key",
    );
    assert!(
        line(&written, "hint:  ").starts_with("did you mean 'database.shop.host'?"),
        "{written}"
    );
}

#[test]
fn u_05_a_key_or_block_of_an_entry_the_file_does_not_declare_names_the_missing_entry() {
    let sandbox = Sandbox::new();
    sandbox.project("[database.shop]\nhost = \"h\"\ndatabase = \"shop\"\n");

    let written = assert_refused(
        &sandbox.run(&["cfg", "unset", "database.nope"]),
        66,
        "a missing block",
    );
    assert_eq!(
        line(&written, "error: "),
        "database entry 'nope' does not exist, so .tpl/.cfg holds no 'database.nope'"
    );
    assert_eq!(
        line(&written, "hint:  "),
        "list the entries with: tpl cfg database list"
    );

    let written = assert_refused(
        &sandbox.run(&["cfg", "get", "database.nope.host"]),
        66,
        "a key of a missing entry",
    );
    assert_eq!(
        line(&written, "error: "),
        "database entry 'nope' does not exist, so .tpl/.cfg holds no 'database.nope.host'"
    );
    assert!(!written.contains("database.nope.port"), "{written}");

    // A slip in the entry name is offered the entry the file declares.
    let written = assert_refused(
        &sandbox.run(&["cfg", "get", "database.shp.host"]),
        66,
        "a misspelt entry",
    );
    assert_eq!(
        line(&written, "hint:  "),
        "did you mean 'database.shop.host'? list the entries with: tpl cfg database list"
    );
    let written = assert_refused(
        &sandbox.run(&["cfg", "unset", "database.shp"]),
        66,
        "a misspelt block",
    );
    assert!(
        line(&written, "hint:  ").starts_with("did you mean 'database.shop'?"),
        "{written}"
    );
}

#[test]
fn u_03_a_port_reference_is_refused_by_the_write_paths_with_where_it_is_accepted() {
    let sandbox = Sandbox::new();
    sandbox.project("[database.x]\nhost = \"h\"\n");

    for arguments in [
        &["cfg", "set", "database.x.port", "${P}"][..],
        &[
            "cfg", "database", "add", "q", "--host", "h", "--port", "${P}",
        ][..],
    ] {
        let written = assert_refused(&sandbox.run(arguments), 64, &arguments.join(" "));
        assert!(
            line(&written, "cause: ").ends_with(
                "a ${VAR} reference for a port is accepted only when written in .tpl/.cfg"
            ),
            "{written}"
        );
        assert_eq!(
            line(&written, "hint:  "),
            "give the port as a number, e.g. 3306; to take it from the environment, edit \
             .tpl/.cfg and write port = \"${VAR}\" in the entry's block"
        );
    }
}

#[test]
fn u_06_an_unclosed_expansion_does_not_claim_that_no_cfg_command_runs() {
    let sandbox = Sandbox::new();
    sandbox.project(
        "[database.x]\nhost = \"127.0.0.1\"\ndatabase = \"d\"\ntls = \"disabled\"\npassword = \
         \"${NOPE\"\n\n[database.y]\nhost = \"127.0.0.1\"\nport = \"${P\"\ndatabase = \"d\"\ntls \
         = \"disabled\"\n",
    );

    let written = assert_refused(
        &sandbox.run(&["-d", "x", "schema", "tables", "--direct"]),
        78,
        "an unclosed password reference",
    );
    let hint = line(&written, "hint:  ");
    assert!(!hint.contains("no tpl cfg command runs"), "{written}");
    assert!(
        hint.ends_with(
            "e.g.: tpl cfg set database.x.password '${VAR}', where VAR is the variable you meant"
        ),
        "{written}"
    );

    // tpl cfg set accepts no reference for a port, so the file is edited.
    let written = assert_refused(
        &sandbox.run(&["-d", "y", "schema", "tables", "--direct"]),
        78,
        "an unclosed port reference",
    );
    let hint = line(&written, "hint:  ");
    assert!(hint.starts_with("edit "), "{written}");
    assert!(!hint.contains("no tpl cfg command runs"), "{written}");

    // And the claim would have been false: a cfg command still runs.
    assert_eq!(code(&sandbox.run(&["cfg", "list"])), 0);
}

#[test]
fn u_07_an_add_of_an_existing_entry_is_answered_with_the_callers_own_values() {
    let sandbox = Sandbox::new();
    sandbox.project("[database.x]\nhost = \"a\"\n");

    let written = assert_refused(
        &sandbox.run(&[
            "cfg", "database", "add", "x", "--host", "h", "--schema", "s",
        ]),
        64,
        "an existing entry",
    );
    assert!(
        line(&written, "hint:  ")
            .starts_with("change it instead with: tpl cfg database update x --host h --schema s"),
        "{written}"
    );
}

#[test]
fn u_08_os_wording_is_dropped_and_an_unknown_key_in_the_file_names_its_line() {
    let sandbox = Sandbox::new();
    sandbox.project(
        "[database.p]\nhost = \"127.0.0.1\"\nport = 1\ndatabase = \"d\"\ntls = \
         \"disabled\"\npassword_command = [\"/nonexistent/tpl-no-such-program\"]\n",
    );
    let written = assert_refused(
        &sandbox.run(&["-d", "p", "schema", "tables", "--direct"]),
        78,
        "a password_command that cannot start",
    );
    assert!(!written.contains("(os error"), "{written}");
    assert!(
        line(&written, "cause: ").ends_with("could not be started: No such file or directory"),
        "{written}"
    );
    assert!(
        line(&written, "hint:  ")
            .ends_with("tpl cfg set database.p.password_command '<program> <argument>'"),
        "{written}"
    );

    let written = assert_refused(
        &sandbox.run(&["cfg", "set", "database.new.password_command", "pass 'a b"]),
        64,
        "an unclosed quote",
    );
    assert!(!written.contains("POSIX"), "{written}");
    assert!(
        written.contains("shell quoting does not admit"),
        "{written}"
    );

    sandbox.project("[core]\ndatabase = \"shop\"\nfoo = 1\n");
    let written = assert_refused(&sandbox.run(&["cfg", "list"]), 78, "an unknown key");
    assert!(
        line(&written, "hint:  ").contains(" at line 3; tpl help cfg set lists every key"),
        "{written}"
    );
}

#[test]
fn rmp_279_a_block_that_is_absent_is_reported_as_absent() {
    let sandbox = Sandbox::new();
    sandbox.project("[database.shop]\nhost = \"h\"\n");

    // FR-CFG-007 keeps a block given to cfg get at 64, with tpl cfg list as
    // the hint where the entry does not exist; the line says it does not.
    let written = assert_refused(
        &sandbox.run(&["cfg", "get", "database.nope"]),
        64,
        "a block of an entry that does not exist",
    );
    assert_eq!(
        line(&written, "error: "),
        "'database.nope' names a whole entry, not one value, and database entry 'nope' does \
         not exist"
    );
    assert!(!written.contains("section"), "{written}");
    assert_eq!(
        line(&written, "hint:  "),
        "show every key and its value with: tpl cfg list"
    );

    // FR-CFG-012: a section the file lacks is absent, and nothing is removed.
    let written = assert_refused(&sandbox.run(&["cfg", "unset", "core"]), 66, "no [core]");
    assert_eq!(
        line(&written, "error: "),
        "'core' names the [core] section, and .tpl/.cfg has none, so there is nothing to remove"
    );
    assert!(!written.contains("not a configuration key"), "{written}");
    assert_eq!(
        line(&written, "hint:  "),
        "show every key and its value with: tpl cfg list"
    );

    sandbox.project("[core]\n");
    let written = assert_refused(
        &sandbox.run(&["cfg", "unset", "database"]),
        66,
        "no [database]",
    );
    assert!(
        line(&written, "error: ").ends_with("so there is nothing to remove"),
        "{written}"
    );
}
