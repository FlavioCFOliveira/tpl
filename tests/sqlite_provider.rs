//! The `sqlite` provider, exercised as a process (rmp `#317`).
//!
//! Every body here runs the distributed binary against a project whose entry
//! names a SQLite database file, built at test time from `scripts/sqlite/` by
//! `tests/support/sqlite.rs` (`FR-LITE-038`, `OD-37`). Nothing here needs the
//! MariaDB fixture, so no body is skipped for its absence; the one body that
//! needs a syscall trace skips on a host without one and says so.
//!
//! The statement record of `FR-LITE-018`, and every rule of the catalogue fold,
//! are verified in process, in the library's own tests under `src/sqlite/`:
//! `FR-LITE-018` takes the record through the seam of `FR-ERR-031` and from no
//! invocation of the binary. What this file establishes is what a caller of
//! the binary meets: the exit code, the four labelled lines of `FR-ERR-008`,
//! stdout, `.tpl/.cfg`, and the files beside the database.
//!
//! | Requirement | The property this file establishes |
//! |---|---|
//! | `FR-CONF-051` | A `sqlite` entry renders a template from the fixture; a provider outside the two values is `78` in the file and `64` on the command line |
//! | `FR-CONF-052` | A key the provider refuses is `78` at entry resolution, naming the provider and how it was found; an entry the invocation does not select refuses nothing |
//! | `FR-CONF-053` | A `sqlite` entry with no `path`, or an empty one, is `78`; an empty value on the command line is `64` |
//! | `FR-CONF-054` | `path` resolves against the project directory, from any working directory, through `--tpl-dir` and through a linked `.tpl`; it is never expanded and never resolved when written |
//! | `FR-CFG-048` | A write that would leave an entry carrying a key its provider refuses is `64`, and the file is unchanged |
//! | `FR-CFG-051` | The fifth form of the warning, for an invocation that uses or targets a `sqlite` entry |
//! | `FR-CFG-053` | A write of `path` or `provider`, or of any field of a `sqlite` entry, warns that the entry may now read another database; a connection field of a `mariadb` entry still warns of another server |
//! | `FR-CFG-055` | The four items, each a refusal at the step it names or a value stored as written |
//! | `FR-CFG-056` | `tpl cfg database test` of a `sqlite` entry reports the four steps, and exits with the code of the file's fault — the check's `78` over each database it refuses |
//! | `FR-ERR-024`, `FR-OUT-018` | A C1 control and a bidirectional formatting character read from the file are escaped in `text` output, in `json` output as JSON's own escape, and in a diagnostic; a value carrying neither is written unchanged |
//! | `FR-ERR-046` | A value of the file in a `cause` is written whole at 256 characters and cut past them, never inside an escape; a name from the argument vector is written whole |
//! | `FR-LITE-005`, `FR-SEC-028` | A directory, a FIFO, a socket and a character device are `78` and never opened; a link is followed |
//! | `FR-LITE-006`, `FR-LITE-007` | A path naming nothing stays naming nothing; each file that cannot be read is refused with its code, and an empty file is an empty database |
//! | `FR-LITE-008`, `FR-LITE-018`, `BR-LITE-001` | A read leaves a rollback-journal database and every file outside `.tpl` as found; a write-ahead-log read adds at most the two files of the mode |
//! | `FR-LITE-014` | No configuration key names a control of `FR-LITE-010` or a limit of `FR-LITE-011` |
//! | `FR-LITE-003`, `FR-LITE-023`, `FR-LITE-024` | The `database` and `table` objects a caller reads carry the values those requirements fix |
//! | `FR-LITE-033`, `FR-LITE-036` | Naming a view that does not compile exits `0` with its nine properties, and a dump marks nothing |
//! | `FR-LITE-043` | Two tables, or two columns of one table, whose names differ only in a byte that is not UTF-8 are `78`, with no model and no cache write; a key whose parent differs from a table's name in such a byte is excluded as dangling, and the read succeeds |
//! | `NFR-PERF-003` | A cache hit on a `sqlite` entry reaches no file: the database removed, the hit answers the same bytes |
//! | `FR-CACHE-038`, `FR-CACHE-039` | A cached render decodes a table's `foreign_keys` and `referenced_by` when it reaches them: a malformed part reached is a miss answered with the direct result, one never reached is no miss; `schema dump` finds the same part a miss before it writes |
//! | `NFR-PERF-004` | A `sqlite` invocation opens the database file once, observed by the syscall trace on Linux |

#[path = "support/fixture.rs"]
mod fixture;
#[path = "support/sandbox.rs"]
mod sandbox;
#[path = "support/sqlite.rs"]
mod sqlite;

use std::collections::BTreeMap;
use std::os::unix::fs::{PermissionsExt as _, symlink};
use std::path::{Path, PathBuf};
use std::process::Output;
use std::time::SystemTime;

use sandbox::Sandbox;

/// The entry every project here defines.
const ENTRY: &str = "ledger";

/// The four labelled lines of `FR-ERR-008`, in their order.
const LABELS: [&str; 4] = ["error:", "cause:", "hint:", "exit:"];

/// The template the acceptance renders: the provider, the database, and the
/// name of every table, one per line.
const LISTING: &str = "{{ database.provider }} {{ database.name }}\n\
                       {% for table in database.tables %}{{ table.name }}\n{% endfor %}";

/// What [`LISTING`] renders from the main fixture database: the twenty-four
/// tables `FR-LITE-020` covers — the five shadow tables of the fts5 table of
/// item 12 among them, since the sixty-ninth edition, and the tables of items
/// 8 and 14, since the seventy-first — in the order of `FR-CTX-013`.
const LISTED: &str = "sqlite main\n\
                      berth\n\
                      cargo_note_config\n\
                      cargo_note_content\n\
                      cargo_note_data\n\
                      cargo_note_docsize\n\
                      cargo_note_idx\n\
                      carrier\n\
                      consignment\n\
                      container_size\n\
                      country\n\
                      customs entry\n\
                      hazard_class\n\
                      inspection_zone\n\
                      manifest_line\n\
                      port\n\
                      port_call\n\
                      stowage_plan\n\
                      tariff_band\n\
                      vessel\n\
                      voyage\n\
                      voyage_audit\n\
                      voyage_leg\n\
                      \u{c9}mbarque\n\
                      \u{e9}mbarque\n";

/// What [`LISTING`] renders from a database with no object.
const LISTED_EMPTY: &str = "sqlite main\n";

/// The hint `FR-LITE-005` and `FR-LITE-007` require for `74` and `78`.
const REPOINT: &str = "tpl cfg database update ledger --path <path>";

// ------------------------------------------------------------------ helpers ---

/// The exit code of a run that exited.
fn code(printed: &Output) -> i32 {
    printed
        .status
        .code()
        .expect("the binary exits rather than being signalled")
}

/// What the run wrote to stdout.
fn stdout(printed: &Output) -> String {
    String::from_utf8_lossy(&printed.stdout).into_owned()
}

/// What the run wrote to stderr.
fn stderr(printed: &Output) -> String {
    String::from_utf8_lossy(&printed.stderr).into_owned()
}

/// The labelled line of a diagnostic, without its label.
fn line(written: &str, label: &str) -> String {
    written
        .lines()
        .find(|line| line.starts_with(label))
        .unwrap_or_else(|| panic!("no {label:?} line in {written:?}"))
        .trim_start_matches(label)
        .trim_start()
        .to_owned()
}

/// Asserts that `printed` is a refusal with `expected`, stdout untouched and
/// the four labelled lines of `FR-ERR-008` written in their order, and returns
/// what it wrote to stderr.
fn refused(printed: &Output, expected: i32, spelled: &str) -> String {
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
        "{spelled} wrote to stdout on a refusal"
    );

    let labelled: Vec<&str> = written
        .lines()
        .filter(|line| LABELS.iter().any(|label| line.starts_with(label)))
        .collect();
    assert_eq!(labelled.len(), 4, "{spelled} wrote {written:?}");
    for (line, label) in labelled.iter().zip(LABELS) {
        assert!(line.starts_with(label), "{spelled} wrote {written:?}");
    }

    written
}

/// Asserts that `printed` exited `0`, and returns its stdout.
fn succeeded(printed: &Output, spelled: &str) -> String {
    assert_eq!(
        code(printed),
        0,
        "{spelled} exited {}: {}",
        code(printed),
        stderr(printed)
    );

    stdout(printed)
}

/// A `.tpl/.cfg` whose selected entry is a `sqlite` entry naming `path`.
fn configuration(path: &str) -> String {
    format!(
        "[core]\ndatabase = \"{ENTRY}\"\n\n[database.{ENTRY}]\nprovider = \"sqlite\"\npath = \
         \"{path}\"\n"
    )
}

/// A sandbox holding a project whose entry names `path`, the main fixture
/// database at `data/freight.sqlite`, and the template [`LISTING`].
fn project(path: &str) -> Sandbox {
    let sandbox = Sandbox::new();
    sandbox.project(&configuration(path));
    sandbox.write(".tpl/templates/listing.jinja", LISTING);
    sandbox.directory("data");
    sqlite::main_database(&sandbox.path("data/freight.sqlite"));

    sandbox
}

/// Every artefact of `FR-LITE-038`, built under `data/` of a project whose
/// entry is repointed at each in turn with [`repoint`].
fn artefacts() -> (Sandbox, sqlite::Artefacts) {
    let sandbox = Sandbox::new();
    sandbox.project(&configuration("data/freight.sqlite"));
    sandbox.write(".tpl/templates/listing.jinja", LISTING);
    let built = sqlite::build(&sandbox.directory("data"));

    (sandbox, built)
}

/// Rewrites `.tpl/.cfg` so that the entry names `path`.
fn repoint(sandbox: &Sandbox, path: &Path) {
    let written = path.to_str().expect("the sandbox path is UTF-8");
    std::fs::write(sandbox.path(".tpl/.cfg"), configuration(written))
        .expect("the configuration is writable");
}

/// The state of one file: its kind, and for a regular file its length,
/// modification time and bytes.
#[derive(Debug, PartialEq, Eq)]
enum State {
    /// A regular file.
    File(u64, SystemTime, Vec<u8>),
    /// A directory.
    Directory,
    /// Anything else, which is never read: reading a FIFO would block.
    Other,
}

/// Every path under `root`, the `.tpl` folder of `FR-LITE-008` apart, with its
/// [`State`].
fn snapshot(root: &Path) -> BTreeMap<PathBuf, State> {
    fn walk(directory: &Path, root: &Path, into: &mut BTreeMap<PathBuf, State>) {
        for entry in std::fs::read_dir(directory).expect("the directory lists") {
            let path = entry.expect("the entry reads").path();
            if path == root.join(".tpl") {
                continue;
            }
            let metadata = std::fs::symlink_metadata(&path).expect("the entry stats");
            let state = if metadata.is_dir() {
                walk(&path, root, into);
                State::Directory
            } else if metadata.is_file() {
                State::File(
                    metadata.len(),
                    metadata.modified().expect("the platform records mtime"),
                    std::fs::read(&path).expect("the file reads"),
                )
            } else {
                State::Other
            };
            into.insert(path, state);
        }
    }

    let mut into = BTreeMap::new();
    walk(root, root, &mut into);
    into
}

/// The names in `directory`, sorted.
fn listing(directory: &Path) -> Vec<String> {
    let mut names: Vec<String> = std::fs::read_dir(directory)
        .expect("the directory lists")
        .map(|entry| {
            entry
                .expect("the entry reads")
                .file_name()
                .to_string_lossy()
                .into_owned()
        })
        .collect();
    names.sort();
    names
}

// ------------------------------------------- FR-CONF-051, the acceptance ---

#[test]
fn fr_conf_051_a_sqlite_entry_renders_a_template_from_the_fixture() {
    let sandbox = project("data/freight.sqlite");

    // The read through the cache, and the pure read of FR-CACHE-016, render
    // the same bytes from the file.
    let cached = sandbox.run(&["render", "listing"]);
    assert_eq!(succeeded(&cached, "tpl render listing"), LISTED);
    assert_eq!(stderr(&cached), "", "a render that succeeds writes no line");

    let direct = sandbox.run(&["render", "listing", "--direct", "--no-cache"]);
    assert_eq!(
        succeeded(&direct, "tpl render listing --direct --no-cache"),
        LISTED
    );
}

#[test]
fn fr_lite_023_the_database_object_of_a_sqlite_read_carries_the_values_it_fixes() {
    // FR-LITE-023 and FR-LITE-003: main, the encoding, no collation, the
    // provider, and a server object whose standing is supported.
    let sandbox = project("data/freight.sqlite");

    let printed = sandbox.run(&[
        "schema",
        "info",
        "--format",
        "json",
        "--direct",
        "--no-cache",
    ]);
    let document: serde_json::Value =
        serde_json::from_str(&succeeded(&printed, "tpl schema info")).expect("the output is JSON");
    let database = &document["data"]["database"];

    assert_eq!(database["name"], "main");
    assert_eq!(database["charset"], "UTF-8");
    assert_eq!(database["collation"], serde_json::Value::Null);
    assert_eq!(database["provider"], "sqlite");
    assert_eq!(database["server"]["standing"], "supported");

    let version = database["server"]["version"]
        .as_str()
        .expect("the version is a string");
    let series = database["server"]["series"]
        .as_str()
        .expect("the series is a string");
    assert!(
        version.starts_with(&format!("{series}.")),
        "the series {series} is not the major and minor of {version}"
    );
}

#[test]
fn fr_lite_024_a_table_of_a_sqlite_read_carries_the_values_it_fixes() {
    let sandbox = project("data/freight.sqlite");

    let printed = sandbox.run(&[
        "schema",
        "table",
        "carrier",
        "--format",
        "json",
        "--direct",
        "--no-cache",
    ]);
    let document: serde_json::Value =
        serde_json::from_str(&succeeded(&printed, "tpl schema table carrier"))
            .expect("the output is JSON");
    let table = &document["data"]["table"];

    assert_eq!(table["name"], "carrier");
    assert_eq!(table["table_type"], "BASE TABLE");
    assert_eq!(table["engine"], serde_json::Value::Null);
    assert_eq!(table["collation"], serde_json::Value::Null);
    assert_eq!(table["comment"], "");
    assert_eq!(table["check_constraints"], serde_json::json!([]));
}

#[test]
fn fr_lite_036_naming_a_view_that_does_not_compile_exits_0_and_nothing_is_marked() {
    // FR-LITE-033 and FR-LITE-036 as amended in the sixty-eighth edition, and
    // FR-PRIV-003 with them: each of the three views of item 10 of FR-LITE-038
    // that do not compile is returned whole at exit 0, carrying the nine keys
    // of FR-CAT-047 and no other, and a dump of the database marks no object.
    const NINE: [&str; 9] = [
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
    let sandbox = project("data/freight.sqlite");

    for name in ["v_document_paths", "v_quay_schedule", "v_ranked_notes"] {
        let printed = sandbox.run(&[
            "schema",
            "view",
            name,
            "--format",
            "json",
            "--direct",
            "--no-cache",
        ]);
        let document: serde_json::Value =
            serde_json::from_str(&succeeded(&printed, &format!("tpl schema view {name}")))
                .expect("the output is JSON");
        let view = document["data"]["view"]
            .as_object()
            .expect("the view is an object");

        let mut keys: Vec<&str> = view.keys().map(String::as_str).collect();
        keys.sort_unstable();
        assert_eq!(keys, NINE, "{name}");
        assert_eq!(view["name"], name);
        assert!(
            view["definition"]
                .as_str()
                .is_some_and(|definition| definition.starts_with("CREATE VIEW ")),
            "{name}: {}",
            view["definition"]
        );
    }

    let printed = sandbox.run(&["schema", "dump", "--direct", "--no-cache"]);
    let document: serde_json::Value =
        serde_json::from_str(&succeeded(&printed, "tpl schema dump")).expect("the output is JSON");
    let database = &document["data"]["database"];
    let views = database["views"].as_array().expect("views is an array");
    assert_eq!(views.len(), 5);
    for object in views
        .iter()
        .chain(database["tables"].as_array().expect("tables is an array"))
    {
        assert!(
            object.get("restricted").is_none(),
            "{} is marked",
            object["name"]
        );
    }
}

// ----------------------------------------------- FR-CONF-054, the path ---

#[test]
fn fr_conf_054_a_relative_path_resolves_against_the_project_directory_from_any_directory() {
    let sandbox = project("data/freight.sqlite");

    // A decoy where a path resolved against the working directory would land:
    // a database with no object, which renders differently.
    let below = sandbox.directory("docs/generated");
    sandbox.directory("docs/generated/data");
    std::fs::write(below.join("data/freight.sqlite"), b"").expect("the sandbox is writable");

    let printed = sandbox.run_from(
        &below,
        &[],
        &["render", "listing", "--direct", "--no-cache"],
    );
    assert_eq!(
        succeeded(&printed, "tpl render listing, from docs/generated"),
        LISTED
    );

    // A project named by --tpl-dir, from a directory outside it holding its
    // own decoy.
    let outside = Sandbox::new();
    outside.directory("data");
    std::fs::write(outside.path("data/freight.sqlite"), b"").expect("the sandbox is writable");
    let named = sandbox.path(".tpl").display().to_string();
    let printed = outside.run(&[
        "--tpl-dir",
        &named,
        "render",
        "listing",
        "--direct",
        "--no-cache",
    ]);
    assert_eq!(succeeded(&printed, "tpl --tpl-dir render listing"), LISTED);
}

#[test]
fn fr_conf_054_a_linked_tpl_folder_resolves_against_the_parent_of_its_canonical_path() {
    let sandbox = Sandbox::new();
    sandbox.project_at("real", &configuration("data/freight.sqlite"));
    sandbox.write("real/.tpl/templates/listing.jinja", LISTING);
    sandbox.directory("real/data");
    sqlite::main_database(&sandbox.path("real/data/freight.sqlite"));

    // The directory holding the link carries a decoy at the same relative
    // path: a database with no object.
    let linked = sandbox.directory("linked");
    sandbox.directory("linked/data");
    std::fs::write(linked.join("data/freight.sqlite"), b"").expect("the sandbox is writable");
    symlink(sandbox.path("real/.tpl"), linked.join(".tpl")).expect("the link is made");

    let printed = sandbox.run_from(
        &linked,
        &[],
        &["render", "listing", "--direct", "--no-cache"],
    );
    assert_eq!(
        succeeded(&printed, "tpl render listing, through a linked .tpl"),
        LISTED
    );
}

#[test]
fn fr_conf_054_the_path_is_literal_never_expanded_by_tilde_or_by_reference() {
    // A leading `~` is a directory of the project, not the home directory.
    let sandbox = project("~/freight.sqlite");
    sandbox.directory("~");
    sqlite::main_database(&sandbox.path("~/freight.sqlite"));
    let home = sandbox.directory("home");
    let home = home.to_str().expect("the sandbox path is UTF-8");

    let printed = sandbox.run_from(
        sandbox.root(),
        &[("HOME", home)],
        &["render", "listing", "--direct", "--no-cache"],
    );
    assert_eq!(succeeded(&printed, "tpl render listing, path ~/…"), LISTED);

    // `${` in the file is 78 at step 3, whatever the variable holds.
    let referenced = project("${DATA}/freight.sqlite");
    let printed = referenced.run_from(
        referenced.root(),
        &[("DATA", "data")],
        &["render", "listing"],
    );
    let written = refused(&printed, 78, "a path holding ${DATA}");
    assert!(
        line(&written, "cause:").contains("database.ledger.path"),
        "the cause does not name the key: {written}"
    );

    // On the command line it is 64, and nothing is written.
    let sandbox = project("data/freight.sqlite");
    let before = sandbox.configuration();
    for arguments in [
        &["cfg", "set", "database.ledger.path", "${DATA}/x.sqlite"][..],
        &[
            "cfg",
            "database",
            "update",
            ENTRY,
            "--path",
            "${DATA}/x.sqlite",
        ][..],
        &[
            "cfg",
            "database",
            "add",
            "archive",
            "--provider",
            "sqlite",
            "--path",
            "${DATA}/x.sqlite",
        ][..],
    ] {
        let spelled = arguments.join(" ");
        refused(&sandbox.run(arguments), 64, &spelled);
        assert_eq!(sandbox.configuration(), before, "{spelled} wrote the file");
    }
}

// ----------------------------------------------- FR-CONF-051, the value ---

#[test]
fn fr_conf_051_a_provider_outside_the_two_values_is_78_in_the_file_and_64_on_the_command_line() {
    // The comparison is exact: `SQLite` is not `sqlite`.
    for provider in ["postgres", "SQLite"] {
        let sandbox = Sandbox::new();
        sandbox.project(&format!(
            "[core]\ndatabase = \"{ENTRY}\"\n\n[database.{ENTRY}]\nprovider = \"{provider}\"\npath \
             = \"data/freight.sqlite\"\n"
        ));

        let written = refused(&sandbox.run(&["schema", "info"]), 78, provider);
        let cause = line(&written, "cause:");
        assert!(cause.contains("database.ledger.provider"), "{written}");
        assert!(
            cause.contains("mariadb") && cause.contains("sqlite"),
            "the cause does not state the two values: {written}"
        );
        assert!(
            line(&written, "hint:").contains(".tpl/.cfg"),
            "the hint does not name the file: {written}"
        );
    }

    let sandbox = project("data/freight.sqlite");
    let before = sandbox.configuration();
    for arguments in [
        &["cfg", "set", "database.ledger.provider", "postgres"][..],
        &["cfg", "set", "database.ledger.provider", "SQLite"][..],
        &[
            "cfg",
            "database",
            "add",
            "archive",
            "--provider",
            "postgres",
            "--path",
            "archive.sqlite",
        ][..],
    ] {
        let spelled = arguments.join(" ");
        let written = refused(&sandbox.run(arguments), 64, &spelled);
        let cause = line(&written, "cause:");
        assert!(
            cause.contains("mariadb") && cause.contains("sqlite"),
            "{spelled}: the cause does not state the two values: {written}"
        );
        assert!(
            line(&written, "hint:").contains("<provider>")
                || line(&written, "hint:").contains("--provider"),
            "{spelled}: the hint does not show the form: {written}"
        );
        assert_eq!(sandbox.configuration(), before, "{spelled} wrote the file");
    }
}

// ------------------------------------------------------------ FR-CONF-052 ---

#[test]
fn fr_conf_052_a_key_the_provider_refuses_is_78_at_entry_resolution_and_opens_nothing() {
    let refused_keys = [
        ("dsn", "\"mariadb://ana@db.example.com/ledger\""),
        ("host", "\"db.example.com\""),
        ("port", "3306"),
        ("user", "\"ana\""),
        ("password", "\"s3cret\""),
        ("password_command", "[\"pass\", \"show\", \"ledger\"]"),
        ("database", "\"ledger\""),
        ("tls", "\"verify-identity\""),
        ("ca_file", "\"/etc/ssl/cert.pem\""),
        ("ca_path", "\"/etc/ssl/certs\""),
    ];

    for (key, value) in refused_keys {
        let sandbox = Sandbox::new();
        sandbox.project(&format!(
            "{}{key} = {value}\n",
            configuration("absent.sqlite")
        ));

        let written = refused(&sandbox.run(&["schema", "info"]), 78, key);
        let cause = line(&written, "cause:");
        assert!(cause.contains(".tpl/.cfg"), "{key}: {written}");
        assert!(cause.contains("'ledger'"), "{key}: {written}");
        assert!(cause.contains("sqlite"), "{key}: {written}");
        assert!(cause.contains(key), "{key}: {written}");
        assert_eq!(
            line(&written, "hint:"),
            "show the entry and the keys it carries with: tpl cfg database show ledger",
            "{key}"
        );
        assert!(
            !sandbox.path("absent.sqlite").exists(),
            "{key}: a refused entry created its file"
        );
    }

    // An entry with `path` and no provider is a mariadb entry, and the cause
    // says the provider was taken from an absent key.
    let sandbox = Sandbox::new();
    sandbox.project(&format!(
        "[core]\ndatabase = \"{ENTRY}\"\n\n[database.{ENTRY}]\npath = \"data/freight.sqlite\"\n"
    ));
    let written = refused(
        &sandbox.run(&["schema", "info"]),
        78,
        "path without provider",
    );
    let cause = line(&written, "cause:");
    assert!(cause.contains("mariadb"), "{written}");
    assert!(cause.contains("absent"), "{written}");
    assert!(cause.contains("path"), "{written}");

    // The repair is a cfg command, which step 3 does not refuse.
    let repaired = sandbox.run(&["cfg", "set", "database.ledger.provider", "sqlite"]);
    succeeded(&repaired, "tpl cfg set database.ledger.provider sqlite");

    // An entry the invocation does not select refuses nothing.
    let sandbox = project("data/freight.sqlite");
    std::fs::write(
        sandbox.path(".tpl/.cfg"),
        format!(
            "{}\n[database.stray]\nprovider = \"sqlite\"\npath = \"x.sqlite\"\nhost = \"h\"\n",
            configuration("data/freight.sqlite")
        ),
    )
    .expect("the configuration is writable");
    let printed = sandbox.run(&["render", "listing", "--direct", "--no-cache"]);
    assert_eq!(
        succeeded(&printed, "a render beside an unselected stray entry"),
        LISTED
    );
}

// ------------------------------------------------------------ FR-CONF-053 ---

#[test]
fn fr_conf_053_a_sqlite_entry_without_a_path_or_with_an_empty_one_is_78() {
    for (body, empty) in [
        (String::new(), false),
        ("path = \"\"\n".to_owned(), true),
        ("path = \"   \"\n".to_owned(), true),
    ] {
        let sandbox = Sandbox::new();
        sandbox.project(&format!(
            "[core]\ndatabase = \"{ENTRY}\"\n\n[database.{ENTRY}]\nprovider = \"sqlite\"\n{body}"
        ));

        let written = refused(&sandbox.run(&["schema", "info"]), 78, &body);
        let cause = line(&written, "cause:");
        assert!(cause.contains(".tpl/.cfg"), "{written}");
        assert!(cause.contains("'ledger'"), "{written}");
        assert!(cause.contains("database.ledger.path"), "{written}");
        if empty {
            assert!(cause.contains("empty"), "{written}");
        }
        assert!(line(&written, "hint:").contains(REPOINT), "{written}");
    }

    // On the command line an empty value is 64, writing nothing.
    let sandbox = project("data/freight.sqlite");
    let before = sandbox.configuration();
    for arguments in [
        &["cfg", "set", "database.ledger.path", ""][..],
        &["cfg", "set", "database.ledger.path", "  "][..],
        &["cfg", "database", "update", ENTRY, "--path", ""][..],
        &[
            "cfg",
            "database",
            "add",
            "archive",
            "--provider",
            "sqlite",
            "--path",
            " ",
        ][..],
    ] {
        let spelled = format!("{arguments:?}");
        let written = refused(&sandbox.run(arguments), 64, &spelled);
        assert!(
            line(&written, "cause:").contains("empty"),
            "{spelled}: {written}"
        );
        assert!(
            line(&written, "hint:").contains("<path>"),
            "{spelled}: {written}"
        );
        assert_eq!(sandbox.configuration(), before, "{spelled} wrote the file");
    }
}

// ------------------------------------------------------------ FR-CFG-055 ---

/// The nine flags of item 1, each with a value its flag admits.
const SERVER_FLAGS: [(&str, &str); 9] = [
    ("--dsn", "mariadb://ana@db.example.com/ledger"),
    ("--host", "db.example.com"),
    ("--port", "3306"),
    ("--user", "ana"),
    ("--schema", "ledger"),
    ("--tls", "verify-identity"),
    ("--password-command", "pass show ledger"),
    ("--ca-file", "/etc/ssl/cert.pem"),
    ("--ca-path", "/etc/ssl/certs"),
];

#[test]
fn fr_cfg_055_item_1_a_sqlite_add_refuses_every_server_flag_and_requires_path() {
    let sandbox = project("data/freight.sqlite");
    let before = sandbox.configuration();

    for (flag, value) in SERVER_FLAGS {
        let arguments = [
            "cfg",
            "database",
            "add",
            "archive",
            "--provider",
            "sqlite",
            "--path",
            "archive.sqlite",
            flag,
            value,
        ];
        let written = refused(&sandbox.run(&arguments), 64, flag);
        assert_eq!(
            line(&written, "error:"),
            format!("{flag} cannot be given with --provider sqlite")
        );
        assert!(line(&written, "cause:").contains("sqlite"), "{written}");
        assert!(
            line(&written, "hint:")
                .contains("tpl cfg database add archive --provider sqlite --path <path>"),
            "{written}"
        );
        assert_eq!(sandbox.configuration(), before, "{flag} wrote the file");
    }

    let written = refused(
        &sandbox.run(&["cfg", "database", "add", "archive", "--provider", "sqlite"]),
        64,
        "add --provider sqlite without --path",
    );
    assert!(
        line(&written, "hint:").contains("--path <path>"),
        "{written}"
    );
    assert_eq!(sandbox.configuration(), before);

    // Step 1: the refusal is made before a project is looked for.
    let nowhere = Sandbox::new();
    let written = refused(
        &nowhere.run(&[
            "cfg",
            "database",
            "add",
            "archive",
            "--provider",
            "sqlite",
            "--host",
            "h",
        ]),
        64,
        "add --provider sqlite --host, outside a project",
    );
    assert_eq!(
        line(&written, "error:"),
        "--host cannot be given with --provider sqlite"
    );
}

#[test]
fn fr_cfg_055_item_2_path_without_sqlite_is_refused_at_argument_parsing() {
    let nowhere = Sandbox::new();
    let cases: [(&[&str], &str); 4] = [
        (
            &["cfg", "database", "add", "archive", "--path", "a.sqlite"],
            "--path cannot be given without --provider sqlite",
        ),
        (
            &[
                "cfg",
                "database",
                "add",
                "archive",
                "--provider",
                "mariadb",
                "--path",
                "a.sqlite",
            ],
            "--path cannot be given with --provider mariadb",
        ),
        (
            &[
                "cfg",
                "database",
                "update",
                ENTRY,
                "--provider",
                "mariadb",
                "--path",
                "a.sqlite",
            ],
            "--path cannot be given with --provider mariadb",
        ),
        (
            &[
                "cfg", "database", "update", ENTRY, "--path", "a.sqlite", "--user", "ana",
            ],
            "--user cannot be given with --path",
        ),
    ];

    // Outside a project, so that a refusal made after discovery would be a
    // different one.
    for (arguments, headline) in cases {
        let spelled = arguments.join(" ");
        let written = refused(&nowhere.run(arguments), 64, &spelled);
        assert_eq!(line(&written, "error:"), headline, "{spelled}");
        assert!(
            line(&written, "hint:").contains("--provider sqlite --path <path>"),
            "{spelled}: {written}"
        );
    }

    // Every flag of item 1 beside --path on an update, in a project, writes
    // nothing.
    let sandbox = project("data/freight.sqlite");
    let before = sandbox.configuration();
    for (flag, value) in SERVER_FLAGS {
        let arguments = [
            "cfg", "database", "update", ENTRY, "--path", "b.sqlite", flag, value,
        ];
        refused(&sandbox.run(&arguments), 64, flag);
        assert_eq!(sandbox.configuration(), before, "{flag} wrote the file");
    }
}

#[test]
fn fr_cfg_055_item_3_an_update_is_decided_against_the_entry_after_the_write() {
    let sandbox = project("data/freight.sqlite");
    std::fs::write(
        sandbox.path(".tpl/.cfg"),
        format!(
            "{}\n[database.shop]\nhost = \"db.example.com\"\nport = 3306\nuser = \"ana\"\ndatabase \
             = \"shop\"\n",
            configuration("data/freight.sqlite")
        ),
    )
    .expect("the configuration is writable");

    // A sqlite entry takes a new path, which is stored as written.
    let printed = sandbox.run(&[
        "cfg",
        "database",
        "update",
        ENTRY,
        "--path",
        "data/b.sqlite",
    ]);
    succeeded(&printed, "update ledger --path");
    let after = String::from_utf8(sandbox.configuration()).expect("the file is UTF-8");
    assert!(after.contains("path = \"data/b.sqlite\""), "{after}");

    let before = sandbox.configuration();
    let cases: [(&[&str], &str); 4] = [
        // A mariadb entry refuses path.
        (
            &["cfg", "database", "update", "shop", "--path", "shop.sqlite"],
            "cannot give entry 'shop' the key database.shop.path, which its provider mariadb \
             does not take",
        ),
        // A sqlite entry refuses host.
        (
            &["cfg", "database", "update", ENTRY, "--host", "h"],
            "cannot give entry 'ledger' the key database.ledger.host, which its provider sqlite \
             does not take",
        ),
        // A mariadb entry cannot become sqlite while it carries server keys.
        (
            &["cfg", "database", "update", "shop", "--provider", "sqlite"],
            "cannot give entry 'shop' the provider sqlite",
        ),
        // A sqlite entry cannot become mariadb while it carries path.
        (
            &["cfg", "database", "update", ENTRY, "--provider", "mariadb"],
            "cannot give entry 'ledger' the provider mariadb",
        ),
    ];
    for (arguments, headline) in cases {
        let spelled = arguments.join(" ");
        let written = refused(&sandbox.run(arguments), 64, &spelled);
        assert_eq!(line(&written, "error:"), headline, "{spelled}");
        assert!(
            line(&written, "cause:").contains("requires removing"),
            "{spelled}: {written}"
        );
        assert!(
            line(&written, "hint:").contains("tpl cfg database add <name>"),
            "{spelled}: {written}"
        );
        assert!(
            !line(&written, "hint:").contains("unset"),
            "{spelled}: BR-ERR-005 forbids a hint that removes a key: {written}"
        );
        assert_eq!(sandbox.configuration(), before, "{spelled} wrote the file");
    }
}

#[test]
fn fr_cfg_055_item_4_the_path_is_stored_as_written_and_never_resolved() {
    let sandbox = project("data/freight.sqlite");
    let below = sandbox.directory("docs");

    for written in [
        "./data/../data/archive.sqlite",
        "data/archive.sqlite",
        "~/x.sqlite",
    ] {
        let name = format!("archive{}", written.len());
        let printed = sandbox.run_from(
            &below,
            &[],
            &[
                "cfg",
                "database",
                "add",
                &name,
                "--provider",
                "sqlite",
                "--path",
                written,
            ],
        );
        succeeded(&printed, written);

        let stored = String::from_utf8(sandbox.configuration()).expect("the file is UTF-8");
        assert!(
            stored.contains(&format!(
                "[database.{name}]\nprovider = \"sqlite\"\npath = \"{written}\""
            )),
            "{written} was not stored as written: {stored}"
        );
    }
}

// ------------------------------------------------------------ FR-CFG-048 ---

#[test]
fn fr_cfg_048_a_provider_written_onto_an_entry_carrying_the_other_providers_keys_is_64() {
    let sandbox = Sandbox::new();
    sandbox.project(
        "[core]\n\n[database.shop]\nhost = \"db.example.com\"\nport = 3306\nuser = \"ana\"\n\
         database = \"shop\"\n",
    );
    let before = sandbox.configuration();

    let written = refused(
        &sandbox.run(&["cfg", "set", "database.shop.provider", "sqlite"]),
        64,
        "cfg set database.shop.provider sqlite",
    );
    assert_eq!(
        line(&written, "error:"),
        "cannot give entry 'shop' the provider sqlite"
    );
    let cause = line(&written, "cause:");
    for key in ["host", "port", "user", "database"] {
        assert!(
            cause.contains(key),
            "the cause does not name {key}: {written}"
        );
    }
    assert!(cause.contains("requires removing"), "{written}");
    assert_eq!(
        line(&written, "hint:"),
        "register the database file as a new entry: tpl cfg database add <name> --provider \
         sqlite --path <path>"
    );
    assert_eq!(sandbox.configuration(), before);

    // A path written onto it is refused the same way.
    refused(
        &sandbox.run(&["cfg", "set", "database.shop.path", "shop.sqlite"]),
        64,
        "cfg set database.shop.path",
    );
    assert_eq!(sandbox.configuration(), before);
}

// --------------------------------------------------- FR-CFG-051, FR-CFG-053 ---

/// The warning of `FR-CFG-053` for the entry `ledger`, a `sqlite` entry: it may
/// now read another database, not point at another server.
const MAY_READ_ANOTHER_DATABASE: &str = "warning: entry 'ledger' may now read another database; \
                                         any data cached for it under .tpl/.cache/ledger/ is kept \
                                         and still served; clear it with: tpl -d ledger cache \
                                         clean";

#[test]
fn fr_cfg_051_the_fifth_form_names_path_for_an_invocation_on_a_sqlite_entry() {
    let sandbox = project("data/freight.sqlite");

    let printed = sandbox.run(&[
        "cfg",
        "database",
        "add",
        "archive",
        "--provider",
        "sqlite",
        "--path",
        "archive.sqlite",
        "-d",
        "other",
    ]);
    succeeded(&printed, "add --provider sqlite -d other");
    assert_eq!(
        stderr(&printed),
        "warning: -d/--database has no effect on tpl cfg database add; it selects the entry for \
         commands that connect; the database file of a sqlite entry is set with --path\n"
    );

    // An update naming an entry the file gives the sqlite provider: the fifth
    // form first, then the line of FR-CFG-053.
    let printed = sandbox.run(&[
        "-d",
        "other",
        "cfg",
        "database",
        "update",
        ENTRY,
        "--path",
        "data/b.sqlite",
    ]);
    succeeded(&printed, "-d other cfg database update ledger --path");
    assert_eq!(
        stderr(&printed),
        format!(
            "warning: -d/--database has no effect on tpl cfg database update; it selects the \
             entry for commands that connect; the database file of a sqlite entry is set with \
             --path\n{MAY_READ_ANOTHER_DATABASE}\n"
        )
    );
}

#[test]
fn fr_cfg_053_a_write_of_path_or_provider_warns_that_the_entry_may_point_elsewhere() {
    let sandbox = project("data/freight.sqlite");

    for arguments in [
        &[
            "cfg",
            "database",
            "update",
            ENTRY,
            "--path",
            "data/b.sqlite",
        ][..],
        &["cfg", "database", "update", ENTRY, "--provider", "sqlite"][..],
        &["cfg", "set", "database.ledger.path", "data/c.sqlite"][..],
        &["cfg", "set", "database.ledger.provider", "sqlite"][..],
        &["cfg", "unset", "database.ledger.path"][..],
    ] {
        let spelled = arguments.join(" ");
        let printed = sandbox.run(arguments);
        assert_eq!(succeeded(&printed, &spelled), "", "{spelled} wrote stdout");
        assert_eq!(
            stderr(&printed),
            format!("{MAY_READ_ANOTHER_DATABASE}\n"),
            "{spelled}"
        );
    }
}

#[test]
fn fr_cfg_053_the_provider_after_the_write_decides_between_a_server_and_a_database() {
    // A `sqlite` entry carrying a key its provider refuses still loads for a
    // cfg command, the repair path of FR-CONF-052. Removing that key leaves a
    // `sqlite` entry, so the line says another database although the key is a
    // connection field.
    let sandbox = Sandbox::new();
    sandbox.project(&format!(
        "{}host = \"db.example.com\"\n",
        configuration("data/freight.sqlite")
    ));
    let printed = sandbox.run(&["cfg", "unset", "database.ledger.host"]);
    assert_eq!(
        succeeded(&printed, "cfg unset database.ledger.host"),
        "",
        "the unset wrote stdout"
    );
    assert_eq!(stderr(&printed), format!("{MAY_READ_ANOTHER_DATABASE}\n"));

    // A `mariadb` entry: a connection field keeps the line of a server, byte
    // for byte, and a write or removal of `provider` reads another database.
    let sandbox = Sandbox::new();
    sandbox.project("[database.shop]\nhost = \"db.example.com\"\n");
    let server = "warning: entry 'shop' may now point at another server; any data cached for it \
                  under .tpl/.cache/shop/ is kept and still served; clear it with: tpl -d shop \
                  cache clean\n";
    let database = server.replace("point at another server", "read another database");

    for (arguments, expected) in [
        (
            &[
                "cfg",
                "database",
                "update",
                "shop",
                "--host",
                "db-staging.example.com",
            ][..],
            server,
        ),
        (&["cfg", "set", "database.shop.port", "3307"][..], server),
        (&["cfg", "unset", "database.shop.port"][..], server),
        (
            &["cfg", "database", "update", "shop", "--provider", "mariadb"][..],
            database.as_str(),
        ),
        (
            &["cfg", "set", "database.shop.provider", "mariadb"][..],
            database.as_str(),
        ),
        (
            &["cfg", "unset", "database.shop.provider"][..],
            database.as_str(),
        ),
    ] {
        let spelled = arguments.join(" ");
        let printed = sandbox.run(arguments);
        assert_eq!(succeeded(&printed, &spelled), "", "{spelled} wrote stdout");
        assert_eq!(stderr(&printed), expected, "{spelled}");
    }
}

// ------------------------------------------------------------ FR-CFG-056 ---

#[test]
fn fr_cfg_056_the_test_of_a_sqlite_entry_reports_the_four_steps() {
    let (sandbox, built) = artefacts();

    for database in [&built.main, &built.empty] {
        repoint(&sandbox, database);
        let printed = sandbox.run(&["cfg", "database", "test", ENTRY, "--format", "json"]);
        let document: serde_json::Value =
            serde_json::from_str(&succeeded(&printed, "tpl cfg database test"))
                .expect("the output is JSON");

        assert_eq!(document["source"], "server", "{}", database.display());
        let data = &document["data"];
        assert_eq!(data["entry"], ENTRY);
        assert_eq!(data["connected"], true);
        assert_eq!(data["read_only_session"], true);
        assert_eq!(data["server"]["standing"], "supported");
        assert!(data["server"]["version"].is_string());
        // An answered probe is enough, the empty database included.
        assert_eq!(data["can_read_catalogue"], true, "{}", database.display());
    }

    // FR-CFG-025: the command reads and writes no cache.
    assert!(
        !sandbox.path(".tpl/.cache").exists(),
        "tpl cfg database test wrote a cache"
    );

    for (database, expected) in [
        (&built.text, 78),
        (&built.directory, 78),
        (&built.root.join("nothing-here.sqlite"), 74),
    ] {
        repoint(&sandbox, database);
        refused(
            &sandbox.run(&["cfg", "database", "test", ENTRY]),
            expected,
            &database.display().to_string(),
        );
    }

    // FR-CFG-056 as amended in the sixty-ninth edition: the catalogue probe is
    // the check of FR-LITE-041, so a file every read refuses is not reported
    // as readable — each of the fourteen databases FR-LITE-038 names for the
    // check, the eight of the seventy-first edition among them, is its 78.
    for (database, kind) in built.refused_by_the_check() {
        repoint(&sandbox, database);
        let written = refused(
            &sandbox.run(&["cfg", "database", "test", ENTRY]),
            78,
            &database.display().to_string(),
        );
        let cause = line(&written, "cause:");
        assert!(cause.contains("main.sqlite_schema"), "{written}");
        assert!(cause.contains(kind), "{kind}: {written}");
        assert!(line(&written, "hint:").contains(REPOINT), "{written}");
    }
}

// ----------------------------------------------- FR-LITE-005, FR-SEC-028 ---

#[test]
fn fr_lite_005_a_path_that_is_not_a_regular_file_is_78_and_never_opened() {
    let (sandbox, built) = artefacts();

    // The socket is bound by this process, in a short path: sockaddr_un is
    // short on every target.
    let socket_path = sandbox.path("s.sock");
    let _listener = std::os::unix::net::UnixListener::bind(&socket_path).expect("the socket binds");

    // A FIFO opened for a read blocks until a writer appears, and no writer
    // ever does here: a body that returns at all shows that it was not opened.
    for (path, kind) in [
        (built.directory.clone(), "a directory"),
        (built.fifo.clone(), "a FIFO"),
        (socket_path.clone(), "a socket"),
        (PathBuf::from("/dev/null"), "a character device"),
    ] {
        repoint(&sandbox, &path);
        let spelled = path.display().to_string();
        let written = refused(
            &sandbox.run(&["render", "listing", "--direct", "--no-cache"]),
            78,
            &spelled,
        );
        let cause = line(&written, "cause:");
        assert!(
            cause.contains(&spelled),
            "the cause does not name the path: {written}"
        );
        assert!(
            cause.contains(kind),
            "the cause does not name {kind}: {written}"
        );
        assert!(cause.contains("regular file"), "{written}");
        assert!(line(&written, "hint:").contains(REPOINT), "{written}");
    }

    // A link is followed to the database it names.
    repoint(&sandbox, &built.link);
    let printed = sandbox.run(&["render", "listing", "--direct", "--no-cache"]);
    assert_eq!(succeeded(&printed, "a link to the database"), LISTED);
}

// ------------------------------------------------ FR-LITE-006, FR-LITE-007 ---

#[test]
fn fr_lite_006_a_path_naming_nothing_is_74_and_stays_naming_nothing() {
    let (sandbox, built) = artefacts();
    let nothing = built.root.join("absent.sqlite");
    let target = built.root.join("nothing-here.sqlite");
    let before = listing(&built.root);

    for path in [&nothing, &built.dangling] {
        repoint(&sandbox, path);
        for arguments in [
            &["render", "listing", "--direct", "--no-cache"][..],
            &["cfg", "database", "test", ENTRY][..],
        ] {
            let spelled = format!("{} over {}", arguments.join(" "), path.display());
            let written = refused(&sandbox.run(arguments), 74, &spelled);
            let cause = line(&written, "cause:");
            assert!(cause.contains(&path.display().to_string()), "{written}");
            assert!(cause.contains("checking the kind"), "{written}");
            assert!(line(&written, "hint:").contains(REPOINT), "{written}");
        }
    }

    assert!(!nothing.exists(), "the read created the file");
    assert!(
        !target.exists(),
        "the read created the target of the dangling link"
    );
    assert_eq!(
        listing(&built.root),
        before,
        "the read changed the directory"
    );
}

#[test]
fn fr_lite_007_a_file_that_cannot_be_read_is_refused_and_never_an_empty_database() {
    let (sandbox, built) = artefacts();

    for (path, returned) in [
        (&built.text, "SQLITE_NOTADB"),
        (&built.truncated, "SQLITE_"),
        (&built.oversized, "SQLITE_TOOBIG"),
    ] {
        repoint(&sandbox, path);
        let spelled = path.display().to_string();
        let written = refused(
            &sandbox.run(&["render", "listing", "--direct", "--no-cache"]),
            78,
            &spelled,
        );
        let cause = line(&written, "cause:");
        assert!(cause.contains(&spelled), "{written}");
        assert!(cause.contains(returned), "{written}");
        assert!(line(&written, "hint:").contains(REPOINT), "{written}");
    }

    // An empty file is SQLite's own empty database.
    repoint(&sandbox, &built.empty);
    let printed = sandbox.run(&["render", "listing", "--direct", "--no-cache"]);
    assert_eq!(succeeded(&printed, "an empty file"), LISTED_EMPTY);
}

#[test]
fn fr_lite_007_a_wal_database_in_a_directory_that_refuses_its_files_is_74() {
    let (sandbox, built) = artefacts();
    let locked = sandbox.directory("locked");
    let copy = locked.join("freight-wal.sqlite");
    std::fs::copy(&built.wal, &copy).expect("the WAL database copies");

    std::fs::set_permissions(&locked, std::fs::Permissions::from_mode(0o555))
        .expect("the directory's mode is set");
    let restore = || {
        std::fs::set_permissions(&locked, std::fs::Permissions::from_mode(0o755))
            .expect("the directory's mode is restored");
    };

    // A process the filesystem lets write anyway — uid 0 — cannot observe the
    // refusal, so the body says so rather than passing in silence.
    let probe = locked.join("probe");
    if std::fs::write(&probe, b"").is_ok() {
        let _ = std::fs::remove_file(&probe);
        restore();
        fixture::notice(
            "skipped fr_lite_007_a_wal_database_in_a_directory_that_refuses_its_files_is_74: \
             this process writes into a directory of mode 0555, so the refusal FR-LITE-008 \
             observes cannot occur here.",
        );
        return;
    }

    repoint(&sandbox, &copy);
    let printed = sandbox.run(&["cfg", "database", "test", ENTRY]);
    let before = listing(&locked);
    restore();

    let written = refused(&printed, 74, "a WAL database in a read-only directory");
    assert!(
        line(&written, "cause:").contains("SQLITE_READONLY_DIRECTORY"),
        "{written}"
    );
    assert!(line(&written, "hint:").contains(REPOINT), "{written}");
    assert_eq!(
        before,
        ["freight-wal.sqlite"],
        "the refused read left a file"
    );
}

// ----------------------------- FR-LITE-008, FR-LITE-018, BR-LITE-001 ---

#[test]
fn fr_lite_018_a_read_leaves_the_database_and_every_file_outside_tpl_as_found() {
    // The read-only promise from outside the process: a rollback-journal
    // database keeps its bytes, its size and its modification time, and no
    // file appears beside it or anywhere outside `.tpl`.
    let sandbox = project("data/freight.sqlite");
    let before = snapshot(sandbox.root());

    for arguments in [
        &["schema", "dump"][..],
        &["render", "listing"][..],
        &["render", "listing", "--direct", "--no-cache"][..],
        &["schema", "table", "voyage", "--direct"][..],
        &["schema", "views", "--direct"][..],
        &["cfg", "database", "test", ENTRY][..],
    ] {
        succeeded(&sandbox.run(arguments), &arguments.join(" "));
        assert_eq!(
            snapshot(sandbox.root()),
            before,
            "tpl {} changed a file outside .tpl",
            arguments.join(" ")
        );
    }
}

#[test]
fn fr_lite_008_a_wal_read_adds_at_most_the_two_files_of_the_mode() {
    let (sandbox, built) = artefacts();
    let directory = built
        .wal
        .parent()
        .expect("the WAL database has a directory");
    let bytes = std::fs::read(&built.wal).expect("the database reads");
    assert_eq!(listing(directory), ["freight-wal.sqlite"]);

    repoint(&sandbox, &built.wal);
    let printed = sandbox.run(&["render", "listing", "--direct", "--no-cache"]);
    assert_eq!(succeeded(&printed, "a WAL database"), LISTED);

    for name in listing(directory) {
        assert!(
            [
                "freight-wal.sqlite",
                "freight-wal.sqlite-wal",
                "freight-wal.sqlite-shm"
            ]
            .contains(&name.as_str()),
            "the read left {name} beside the database"
        );
    }
    assert_eq!(
        std::fs::read(&built.wal).expect("the database reads"),
        bytes,
        "the read wrote the database file"
    );
}

// ----------------------------------------------------------- FR-LITE-014 ---

#[test]
fn fr_lite_014_no_configuration_key_names_a_control_or_a_limit() {
    let sandbox = project("data/freight.sqlite");
    let before = sandbox.configuration();

    for control in [
        "query_only",
        "trusted_schema",
        "cell_size_check",
        "mmap_size",
        "temp_store",
        "defensive",
        "load_extension",
        "immutable",
        "length_limit",
        "read_only",
    ] {
        for key in [
            format!("database.{ENTRY}.{control}"),
            format!("core.{control}"),
        ] {
            refused(&sandbox.run(&["cfg", "set", &key, "0"]), 64, &key);
            assert_eq!(sandbox.configuration(), before, "{key} was written");
        }

        // Written by hand, the key is refused at step 3.
        let hand = Sandbox::new();
        hand.project(&format!("{}{control} = false\n", configuration("x.sqlite")));
        refused(&hand.run(&["schema", "info"]), 78, control);
    }
}

// ----------------------------------------------------------- NFR-PERF-003 ---

#[test]
fn nfr_perf_003_a_cache_hit_on_a_sqlite_entry_reaches_no_file() {
    let sandbox = project("data/freight.sqlite");

    let filled = sandbox.run(&["schema", "table", "carrier"]);
    let answer = succeeded(&filled, "the read that fills the cache");

    // The database is moved away: a hit that opened it would fail with 74.
    let moved = sandbox.path("data/moved.sqlite");
    std::fs::rename(sandbox.path("data/freight.sqlite"), &moved).expect("the database moves");

    let hit = sandbox.run(&["schema", "table", "carrier"]);
    assert_eq!(succeeded(&hit, "the cache hit"), answer);

    // The control: the same read that skips the cache does reach the file.
    refused(
        &sandbox.run(&["schema", "table", "carrier", "--direct"]),
        74,
        "the read that skips the cache",
    );
}

// ----------------------------------------------------------- NFR-PERF-004 ---

#[test]
fn nfr_perf_004_a_sqlite_invocation_opens_the_database_file_once() {
    const NAME: &str = "nfr_perf_004_a_sqlite_invocation_opens_the_database_file_once";

    if !cfg!(target_os = "linux") {
        fixture::notice(&format!(
            "skipped {NAME}: NFR-PERF-004 is observed for a sqlite entry by the syscall trace of \
             NFR-PERF-007, which exists on the two Linux targets only; no instrument of \
             NFR-PERF-007 observes the clause on this target."
        ));
        return;
    }

    let sandbox = project("data/freight.sqlite");
    let binary = env!("CARGO_BIN_EXE_tpl");
    let root = sandbox.root().display().to_string();

    for arguments in [
        "render listing --direct --no-cache",
        "cfg database test ledger",
    ] {
        let traced = format!("cd '{root}' && '{binary}' {arguments}");
        let Some(trace) = fixture::opens(&["sh", "-c", &traced]) else {
            fixture::notice(&format!(
                "skipped {NAME}: this Linux host has no strace, so the instrument NFR-PERF-007 \
                 credits to it does not exist here."
            ));
            return;
        };

        // The database itself, and not its journal: the closing quote
        // follows the name.
        let opened = trace
            .lines()
            .filter(|line| line.contains("open(") || line.contains("openat("))
            .filter(|line| line.contains("freight.sqlite\""))
            .count();

        assert_eq!(
            opened, 1,
            "tpl {arguments} opened the database file {opened} times:\n{trace}"
        );
    }
}

// ------------------------------------------------- FR-ERR-024, FR-OUT-018 ---

/// The name of the table [`hostile`] builds: U+202E, which shows the rest of
/// the name reversed, and U+009B, the one-character control sequence
/// introducer.
const HOSTILE_TABLE: &str = "manifest\u{202e}fdp.exe\u{9b}31m";

/// Builds, in `sandbox`, a database whose table name, column name and column
/// default carry a C1 control and a bidirectional formatting character, and
/// points the entry at it.
fn hostile(sandbox: &Sandbox) {
    let path = sandbox.path("data/hostile.sqlite");
    let connection = rusqlite::Connection::open(&path).expect("the database opens");
    connection
        .execute_batch(&format!(
            "CREATE TABLE \"{HOSTILE_TABLE}\" (\
                 line_id INTEGER PRIMARY KEY, \
                 \"note\u{200f}\u{2066}x\" TEXT DEFAULT 'a\u{9b}b\u{202e}c\u{61c}d', \
                 plain TEXT DEFAULT 'caf\u{e9} \u{2014} \u{6e2f}');"
        ))
        .expect("the database builds");
    connection.close().expect("the database closes");
    repoint(sandbox, &path);
}

/// Whether `written` carries a C1 control or a bidirectional formatting
/// character raw.
fn carries_the_set_raw(written: &str) -> bool {
    written.chars().any(|character| {
        matches!(
            character,
            '\u{80}'..='\u{9f}'
                | '\u{61c}'
                | '\u{200e}'
                | '\u{200f}'
                | '\u{202a}'..='\u{202e}'
                | '\u{2066}'..='\u{2069}'
        )
    })
}

#[test]
fn fr_out_018_a_c1_control_and_a_bidirectional_character_of_the_file_are_escaped_in_text() {
    // FR-OUT-018 as amended in the sixty-ninth edition, `text`: each is
    // written in the form of a C0 escape, and a value carrying neither is
    // written unchanged.
    let sandbox = project("data/freight.sqlite");
    hostile(&sandbox);

    let tables = succeeded(
        &sandbox.run(&["schema", "tables", "--direct", "--no-cache"]),
        "tpl schema tables",
    );
    assert!(
        tables.contains("manifest\\u{202e}fdp.exe\\u{9b}31m"),
        "{tables:?}"
    );
    assert!(!carries_the_set_raw(&tables), "{tables:?}");

    let table = succeeded(
        &sandbox.run(&["schema", "table", HOSTILE_TABLE, "--direct", "--no-cache"]),
        "tpl schema table",
    );
    assert!(table.contains("note\\u{200f}\\u{2066}x"), "{table:?}");
    assert!(table.contains("a\\u{9b}b\\u{202e}c\\u{61c}d"), "{table:?}");
    assert!(table.contains("caf\u{e9} \u{2014} \u{6e2f}"), "{table:?}");
    assert!(!carries_the_set_raw(&table), "{table:?}");
}

#[test]
fn fr_out_018_a_c1_control_and_a_bidirectional_character_of_the_file_take_json_s_escape() {
    // FR-OUT-018 as amended in the sixty-ninth edition, `json`: JSON's own
    // `\u` and four lowercase hexadecimal digits, so the parsed document
    // holds the characters the file holds; a value carrying neither is
    // written unchanged.
    let sandbox = project("data/freight.sqlite");
    hostile(&sandbox);

    for pretty in [false, true] {
        let mut arguments = vec![
            "schema",
            "table",
            HOSTILE_TABLE,
            "--format",
            "json",
            "--direct",
            "--no-cache",
        ];
        if pretty {
            arguments.push("--pretty");
        }
        let written = succeeded(&sandbox.run(&arguments), "tpl schema table --format json");

        assert!(
            written.contains("manifest\\u202efdp.exe\\u009b31m"),
            "{written}"
        );
        assert!(written.contains("note\\u200f\\u2066x"), "{written}");
        assert!(written.contains("a\\u009bb\\u202ec\\u061cd"), "{written}");
        assert!(written.contains("caf\u{e9} \u{2014} \u{6e2f}"), "{written}");
        assert!(!carries_the_set_raw(&written), "{written:?}");

        let document: serde_json::Value =
            serde_json::from_str(&written).expect("the output is JSON");
        let table = &document["data"]["table"];
        assert_eq!(
            table["name"], HOSTILE_TABLE,
            "the parsed name is the file's"
        );
        let columns = table["columns"].as_array().expect("the columns");
        assert!(
            columns
                .iter()
                .any(|column| column["name"] == "note\u{200f}\u{2066}x"),
            "{written}"
        );
    }
}

#[test]
fn fr_err_024_a_c1_control_and_a_bidirectional_character_are_escaped_in_a_diagnostic() {
    // FR-ERR-024 as amended in the sixty-ninth edition: a name carrying both,
    // given on the command line and not found, comes back escaped in the
    // `error` and `cause` lines, in the form of a C0 escape.
    let sandbox = project("data/freight.sqlite");
    let missing = "orders\u{202e}gpj.exe\u{9b}31m";

    let written = refused(
        &sandbox.run(&["schema", "table", missing, "--direct", "--no-cache"]),
        66,
        "a missing table carrying the set",
    );
    for label in ["error:", "cause:"] {
        assert!(
            line(&written, label).contains("orders\\u{202e}gpj.exe\\u{9b}31m"),
            "{label} {written:?}"
        );
    }
    assert!(!carries_the_set_raw(&written), "{written:?}");
}

// ----------------------------------------------------------- FR-ERR-046 ---

#[test]
fn fr_err_046_a_value_of_the_file_is_bounded_in_a_cause_and_never_cut_inside_an_escape() {
    // FR-ERR-046 through a SQLite path: the name of a row the check refuses is
    // a value of the file, interpolated into the `cause`. 256 characters as
    // escaped are written whole; 257 are cut to 128 and 128 around the marker
    // counting what was left out; a cut that would fall inside an escape —
    // of C0, C1 or a bidirectional formatting character — keeps the escape
    // whole outside the kept part.
    let sandbox = project("data/freight.sqlite");
    let build = |name: &str| {
        let path = sandbox.path("data/long.sqlite");
        let _ = std::fs::remove_file(&path);
        let connection = rusqlite::Connection::open(&path).expect("the database opens");
        connection
            .execute_batch(&format!(
                "CREATE TABLE \"{}\" (a)",
                name.replace('"', "\"\"")
            ))
            .expect("the database builds");
        connection.close().expect("the database closes");
        repoint(&sandbox, &path);
        let written = refused(
            &sandbox.run(&["schema", "tables", "--direct", "--no-cache"]),
            78,
            "a table named pragma_",
        );
        line(&written, "cause:")
    };

    // 256 escaped characters: whole.
    let whole = format!("pragma_{}", "w".repeat(249));
    assert!(build(&whole).contains(&format!("named '{whole}'")));

    // 257: both ends kept, one character counted out.
    let cut = format!("pragma_{}", "c".repeat(250));
    let cause = build(&cut);
    let expected = format!(
        "named '{}[... 1 characters omitted ...]{}'",
        &cut[..128],
        &cut[cut.len() - 128..]
    );
    assert!(cause.contains(&expected), "{cause}");

    // An escape where a cut would fall, at each end.
    for (character, form) in [
        ('\n', "\\n"),
        ('\u{1}', "\\u{01}"),
        ('\u{9b}', "\\u{9b}"),
        ('\u{202e}', "\\u{202e}"),
    ] {
        let name: String = "pragma_"
            .chars()
            .chain("x".repeat(120).chars())
            .chain(std::iter::once(character))
            .chain("y".repeat(300).chars())
            .chain(std::iter::once(character))
            .chain("z".repeat(127).chars())
            .collect();
        let cause = build(&name);
        let head = format!("named 'pragma_{}[... ", "x".repeat(120));
        let tail = format!(" characters omitted ...]{}'", "z".repeat(127));
        assert!(cause.contains(&head), "{form}: {cause}");
        assert!(cause.contains(&tail), "{form}: {cause}");
        let omitted = 2 * form.chars().count() + 300;
        assert!(
            cause.contains(&format!("[... {omitted} characters omitted ...]")),
            "{form}: {cause}"
        );
    }
}

#[test]
fn fr_err_046_a_name_from_the_argument_vector_is_written_whole_on_a_sqlite_path() {
    // FR-ERR-046: the bound reaches a value of the file and never the
    // argument vector, which FR-ERR-034 echoes as written — over a `sqlite`
    // entry as over a `mariadb` one.
    let sandbox = project("data/freight.sqlite");
    let long = "a".repeat(300);

    let written = refused(
        &sandbox.run(&["schema", "table", &long, "--direct", "--no-cache"]),
        66,
        "a long missing table",
    );
    assert!(line(&written, "cause:").contains(&long), "{written}");
    assert!(!written.contains("characters omitted"), "{written}");
}

#[test]
fn fr_err_046_a_name_of_the_file_reaches_a_did_you_mean_only_within_the_bound() {
    // FR-ERR-046 through the nearest-match suggestion of FR-ERR-019: the
    // candidates are names of the file. One past the bound never reaches the
    // line — FR-ERR-022 admits no candidate past 64 characters and FR-ERR-023
    // drops it in every form — and one within it is written whole.
    let sandbox = project("data/freight.sqlite");
    let path = sandbox.path("data/long-names.sqlite");
    let long = format!("manifest_{}", "l".repeat(300));
    let within = format!("{}a", "v".repeat(63));
    let connection = rusqlite::Connection::open(&path).expect("the database opens");
    connection
        .execute_batch(&format!(
            "CREATE TABLE \"{long}\" (line_id INTEGER PRIMARY KEY); \
             CREATE TABLE \"{within}\" (vessel_imo INTEGER PRIMARY KEY);"
        ))
        .expect("the database builds");
    connection.close().expect("the database closes");
    repoint(&sandbox, &path);

    // One substitution from the long name, so that it qualifies by distance
    // and the argument vector does not hold it.
    let supplied = format!("manifest_{}k{}", "l".repeat(150), "l".repeat(149));
    let written = refused(
        &sandbox.run(&["schema", "table", &supplied, "--direct", "--no-cache"]),
        66,
        "a missing table one step from a long name of the file",
    );
    assert!(!written.contains(&long), "{written}");
    assert!(
        !line(&written, "hint:").contains("did you mean"),
        "{written}"
    );
    assert!(
        line(&written, "hint:").contains("tpl schema tables"),
        "{written}"
    );
    // The argument vector is written whole, per FR-ERR-034.
    assert!(line(&written, "error:").contains(&supplied), "{written}");

    // One substitution from the name of 64 characters: written whole.
    let supplied = format!("{}b", "v".repeat(63));
    let written = refused(
        &sandbox.run(&["schema", "table", &supplied, "--direct", "--no-cache"]),
        66,
        "a missing table one step from a name of 64 characters",
    );
    assert!(
        line(&written, "hint:").starts_with(&format!("did you mean '{within}'?")),
        "{written}"
    );
    assert!(!written.contains("characters omitted"), "{written}");
}

// ------------------------------------------------------------ FR-LITE-043 ---

/// The text `entrep\xF4t` and `entrep\xD4t` both convert to (`FR-OUT-017`).
const WAREHOUSE: &str = "entrep\u{fffd}t";

#[test]
fn fr_lite_043_two_names_of_one_set_that_convert_alike_are_78_with_no_model_and_no_cache() {
    // FR-LITE-043 over the first two databases FR-LITE-038 adds for it: two
    // tables, then two columns of one table, whose names differ only in a
    // byte that is not UTF-8. Each read is 78, writes nothing to stdout and
    // writes no cache file; the cause names the path, the kind of the two
    // objects, the table that holds two columns, and the text both convert
    // to; the hint repoints the entry.
    let (sandbox, built) = artefacts();

    for (database, kind, table, text) in [
        (&built.alike_tables, "two tables", None, WAREHOUSE),
        (
            &built.alike_columns,
            "two columns",
            Some("customs_line"),
            "d\u{fffd}signation",
        ),
    ] {
        repoint(&sandbox, database);
        for arguments in [&["schema", "dump"][..], &["render", "listing"][..]] {
            let spelled = format!("{} over {}", arguments.join(" "), database.display());
            let written = refused(&sandbox.run(arguments), 78, &spelled);

            assert!(
                line(&written, "error:").contains(&format!("holds {kind} tpl cannot tell apart")),
                "{spelled}: {written}"
            );
            let cause = line(&written, "cause:");
            assert!(
                cause.contains(&database.display().to_string()),
                "{spelled}: {written}"
            );
            assert!(
                cause.contains(&format!("{kind} whose names both read '{text}'")),
                "{spelled}: {written}"
            );
            assert!(
                cause.contains("differ only in bytes that are not valid UTF-8"),
                "{spelled}: {written}"
            );
            if let Some(table) = table {
                assert!(
                    cause.contains(&format!("in table '{table}'")),
                    "{spelled}: {written}"
                );
            }
            assert!(line(&written, "hint:").contains(REPOINT), "{written}");
            assert!(
                !sandbox.path(&format!(".tpl/.cache/{ENTRY}")).exists(),
                "{spelled} wrote to the cache"
            );
        }
    }
}

#[test]
fn fr_lite_043_a_key_whose_parent_differs_only_in_an_invalid_byte_is_excluded_as_dangling() {
    // FR-LITE-043 and FR-LITE-021 over the third database: the key of
    // `stock_lot` names `entrep\xD4t`, which SQLite does not hold, while the
    // table it holds is `entrep\xF4t`; both convert to one text. The read
    // succeeds, carries the table under that text, and excludes the key,
    // which a match on the text would have attached. The cache written is the
    // control that the absence above observes a write.
    let (sandbox, built) = artefacts();
    repoint(&sandbox, &built.alike_dangling_key);

    let dumped = succeeded(&sandbox.run(&["schema", "dump"]), "schema dump");
    let document: serde_json::Value = serde_json::from_str(&dumped).expect("the dump is JSON");
    let tables = document["data"]["database"]["tables"]
        .as_array()
        .expect("the dump carries tables");
    let names: Vec<&str> = tables
        .iter()
        .map(|table| table["name"].as_str().expect("a table has a name"))
        .collect();
    assert_eq!(names, [WAREHOUSE, "stock_lot"]);

    for table in tables {
        assert_eq!(
            table["foreign_keys"].as_array().map(Vec::len),
            Some(0),
            "{}",
            table["name"]
        );
        assert_eq!(
            table["referenced_by"].as_array().map(Vec::len),
            Some(0),
            "{}",
            table["name"]
        );
    }
    assert!(
        sandbox.path(&format!(".tpl/.cache/{ENTRY}")).exists(),
        "the read that succeeded wrote no cache"
    );
}

// ------------------------------------------------------------ FR-CACHE-038 ---

/// A template that reaches both deferred parts of every table.
const REACHES_KEYS: &str = "{% for table in database.tables %}{{ table.name }} \
                            {{ table.foreign_keys | length }} {{ table.referenced_by | length }}\n\
                            {% for key in table.foreign_keys %}  -> {{ key.referenced_table.name }}\n\
                            {% endfor %}{% endfor %}";

/// A template that reaches every table and neither deferred part.
const SKIPS_KEYS: &str = "{% for table in database.tables %}{{ table.name }} \
                          {{ table.columns | length }} {{ table.indexes | length }}\n{% endfor %}";

/// A project over the main fixture database, less `émbarque`, its cache
/// loaded, carrying [`REACHES_KEYS`] and [`SKIPS_KEYS`].
///
/// `Émbarque` and `émbarque` share one cache file on a volume that folds
/// letter case, which records the tables as not whole and sends every cached
/// read to the database; one of the two is dropped so that the render below is
/// served from the cache on every host.
fn loaded() -> Sandbox {
    let sandbox = project("data/freight.sqlite");
    let connection = rusqlite::Connection::open(sandbox.path("data/freight.sqlite"))
        .expect("the database opens");
    connection
        .execute_batch("DROP TABLE \"émbarque\";")
        .expect("the table drops");
    connection.close().expect("the database closes");
    sandbox.write(".tpl/templates/keys.jinja", REACHES_KEYS);
    sandbox.write(".tpl/templates/skips.jinja", SKIPS_KEYS);
    succeeded(&sandbox.run(&["cache", "load"]), "cache load");

    sandbox
}

/// The cache file of a table whose `part` holds at least one member.
fn holding(sandbox: &Sandbox, part: &str) -> PathBuf {
    let tables = sandbox.path(&format!(".tpl/.cache/{ENTRY}/tables"));
    let mut files: Vec<PathBuf> = std::fs::read_dir(&tables)
        .expect("the cache holds its tables")
        .map(|entry| entry.expect("the entry reads").path())
        .collect();
    files.sort();

    files
        .into_iter()
        .find(|file| {
            let table: serde_json::Value =
                serde_json::from_slice(&std::fs::read(file).expect("the file reads"))
                    .expect("a file tpl wrote is JSON");
            table[part]
                .as_array()
                .is_some_and(|members| !members.is_empty())
        })
        .unwrap_or_else(|| panic!("no table of the fixture holds a member of {part}"))
}

/// Replaces the first member of each of `parts` in `file` with an object the
/// member's type refuses, leaving the file valid JSON whose every other part
/// decodes, and answers the bytes it held.
fn malform(file: &Path, parts: &[&str]) -> Vec<u8> {
    let held = std::fs::read(file).expect("the file reads");
    let mut table: serde_json::Value = serde_json::from_slice(&held).expect("the file is JSON");
    for part in parts {
        table[*part][0] = serde_json::json!({ "name": 5 });
    }
    std::fs::write(file, format!("{table}\n")).expect("the cache is ours");

    held
}

#[test]
fn fr_cache_038_a_malformed_deferred_part_a_render_reaches_is_a_miss_answered_from_the_database() {
    // FR-CACHE-038, eighty-second edition: the part is decoded when the
    // template first reaches it, and a fault in it makes the file a miss under
    // FR-CACHE-039 — the render abandoned, nothing of it on stdout, and the
    // invocation rendered again from the database's document and written to
    // the cache.
    for part in ["foreign_keys", "referenced_by"] {
        let sandbox = loaded();
        let direct = succeeded(
            &sandbox.run(&["render", "keys", "--direct", "--no-cache"]),
            "the direct render",
        );
        let file = holding(&sandbox, part);
        let held = malform(&file, &[part]);

        let printed = sandbox.run(&["render", "keys"]);

        assert_eq!(succeeded(&printed, part), direct, "{part}");
        assert_eq!(stderr(&printed), "", "{part}: FR-CACHE-033 reports nothing");
        assert_eq!(
            std::fs::read(&file).expect("the file reads"),
            held,
            "{part}: FR-CACHE-007 rewrote the file the miss was found in"
        );
    }
}

#[test]
fn fr_cache_038_a_malformed_deferred_part_a_render_never_reaches_is_no_miss() {
    // FR-CACHE-038: a deferred part the template never reaches is never
    // decoded, and a fault in it is no miss of that invocation. The database
    // is moved away, so a miss would end in 74 rather than be served.
    let sandbox = loaded();
    let direct = succeeded(
        &sandbox.run(&["render", "skips", "--direct", "--no-cache"]),
        "the direct render",
    );
    let files = [
        holding(&sandbox, "foreign_keys"),
        holding(&sandbox, "referenced_by"),
    ];
    malform(&files[0], &["foreign_keys"]);
    malform(&files[1], &["referenced_by"]);
    let damaged = files
        .each_ref()
        .map(|file| std::fs::read(file).expect("the file reads"));
    std::fs::rename(
        sandbox.path("data/freight.sqlite"),
        sandbox.path("data/moved.sqlite"),
    )
    .expect("the database moves");

    let printed = sandbox.run(&["render", "skips"]);

    assert_eq!(succeeded(&printed, "the cached render"), direct);
    assert_eq!(
        files
            .each_ref()
            .map(|file| std::fs::read(file).expect("the file reads")),
        damaged,
        "a file that was no miss is left as it is"
    );
}

#[test]
fn fr_cache_038_schema_dump_decodes_every_part_before_it_writes_so_the_same_part_is_a_miss() {
    // FR-CACHE-038: the deferral is the render's alone. `tpl schema dump` over
    // the same malformed part finds the file a miss before its first byte:
    // with the database moved away the miss reads it and ends in 74, with
    // nothing on stdout; with the database back it answers the direct
    // document and rewrites the file.
    let sandbox = loaded();
    let direct = succeeded(
        &sandbox.run(&["schema", "dump", "--direct", "--no-cache"]),
        "the direct dump",
    );
    let file = holding(&sandbox, "foreign_keys");
    let held = malform(&file, &["foreign_keys"]);
    let database = sandbox.path("data/freight.sqlite");
    let moved = sandbox.path("data/moved.sqlite");

    std::fs::rename(&database, &moved).expect("the database moves");
    refused(
        &sandbox.run(&["schema", "dump"]),
        74,
        "a dump whose cache file is a miss, with no database",
    );
    std::fs::rename(&moved, &database).expect("the database moves back");

    let printed = sandbox.run(&["schema", "dump"]);

    assert_eq!(succeeded(&printed, "the dump"), direct);
    assert_eq!(
        std::fs::read(&file).expect("the file reads"),
        held,
        "FR-CACHE-007 rewrote the file the miss was found in"
    );
}
