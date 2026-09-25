//! Cache containment: no command resolves a cache path through a symbolic
//! link (`FR-CACHE-042`, `FR-CACHE-043`, `FR-CACHE-044`, `FR-SEC-026`).
//!
//! **No body needs a server.** The store is written here by hand, in the
//! arrangement `FR-CACHE-001` draws, with every collection recorded whole and
//! empty, so a listing is a real cache hit. The entry points at a port nothing
//! listens on, so a read that reached for the server would exit `69`: a `78`
//! from a body below therefore also shows the refusal came before any
//! connection, and a hit refused shows it came before the cache was consulted.
//!
//! **Every link points outside the project**, into a second sandbox, and every
//! refusal asserts that the link's target holds exactly what it held before —
//! the sentinel file included — so a command that read, wrote or removed
//! through the link would be caught.

#[path = "support/sandbox.rs"]
mod sandbox;

use std::collections::BTreeMap;
use std::os::unix::fs::symlink;
use std::path::{Path, PathBuf};
use std::process::Output;

use sandbox::Sandbox;

/// The entry every body reads through.
const ENTRY: &str = "shop";

/// The project's configuration: one entry, selected by default, whose server
/// cannot be reached.
const CONFIGURATION: &str = "[core]\ndatabase = \"shop\"\n\n[database.shop]\nhost = \"127.0.0.1\"\n\
                             port = 9\nuser = \"reader\"\ndatabase = \"shop\"\n";

/// A `meta.json` recording all three collections whole, at a fixed load time.
const META: &str = r#"{"cache_format":1,"schema_version":1,"loaded_at":"2026-09-01T00:00:00Z","collections":[{"name":"tables","whole":true},{"name":"views","whole":true},{"name":"routines","whole":true}]}
"#;

/// The database's own metadata, as the store holds it.
const DATABASE: &str = r#"{"name":"shop","charset":"utf8mb4","collation":"utf8mb4_general_ci","server":{"version":"11.4.13-MariaDB","series":"11.4","standing":"supported"}}
"#;

/// The file planted in every link target, which no command may touch.
const SENTINEL: &str = "sentinel.txt";

/// The eight `schema` subcommands, `tpl render` without `--context`,
/// `tpl cache load` and `tpl cache status`: every command `FR-CACHE-044`
/// governs.
const READERS: [&[&str]; 11] = [
    &["schema", "info"],
    &["schema", "tables"],
    &["schema", "table", "orders"],
    &["schema", "views"],
    &["schema", "view", "v_orders"],
    &["schema", "routines"],
    &["schema", "routine", "procedure:p_orders"],
    &["schema", "dump"],
    &["render", "plain"],
    &["cache", "load"],
    &["cache", "status"],
];

/// A project whose store for [`ENTRY`] holds a whole, empty catalogue, and a
/// template that reads it.
fn project() -> Sandbox {
    let sandbox = Sandbox::new();
    sandbox.project(CONFIGURATION);
    sandbox.write(".tpl/templates/plain.jinja", "{{ database.name }}\n");
    seed(&sandbox.path(&format!(".tpl/.cache/{ENTRY}")));

    sandbox
}

/// Writes a whole, empty store for one entry at `folder`.
fn seed(folder: &Path) {
    for collection in ["tables", "views", "routines"] {
        std::fs::create_dir_all(folder.join(collection)).expect("the sandbox is writable");
    }
    std::fs::write(folder.join("meta.json"), META).expect("the sandbox is writable");
    std::fs::write(folder.join("database.json"), DATABASE).expect("the sandbox is writable");
}

/// A folder outside every project, holding the sentinel, for a link to point
/// at.
fn outside(sandbox: &Sandbox, relative: &str) -> PathBuf {
    let folder = sandbox.directory(relative);
    std::fs::write(folder.join(SENTINEL), "keep me\n").expect("the sandbox is writable");

    folder
}

/// Everything beneath `directory`, examined without following a link: each
/// file's bytes, each directory as a marker, each link as its target.
fn contents(directory: &Path) -> BTreeMap<PathBuf, String> {
    let mut found = BTreeMap::new();
    let mut pending = vec![directory.to_owned()];

    while let Some(current) = pending.pop() {
        for entry in std::fs::read_dir(&current).expect("the folder is ours") {
            let path = entry.expect("the folder is ours").path();
            let relative = path
                .strip_prefix(directory)
                .expect("beneath the walked folder")
                .to_owned();
            let kind = std::fs::symlink_metadata(&path).expect("the entry is ours");

            if kind.file_type().is_symlink() {
                let target = std::fs::read_link(&path).expect("a link has a target");
                found.insert(relative, format!("link -> {}", target.display()));
            } else if kind.is_dir() {
                found.insert(relative, "dir".to_owned());
                pending.push(path);
            } else {
                found.insert(
                    relative,
                    std::fs::read_to_string(&path).expect("every file here is text"),
                );
            }
        }
    }

    found
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

/// Asserts that `printed` is the `78` of `FR-CACHE-042` item 1 or of
/// `FR-CACHE-044` naming `link`: nothing on stdout, the four labelled lines,
/// the link named by its path in the `error` and `cause` lines, and a `hint`
/// that removes the link alone.
fn refused(printed: &Output, spelled: &str, tpl: &Path, within: &str, removal: bool) {
    let written = stderr(printed);
    let link = tpl.join(within);

    assert_eq!(printed.status.code(), Some(78), "{spelled}: {written}");
    assert!(
        printed.stdout.is_empty(),
        "{spelled} wrote to stdout: {}",
        String::from_utf8_lossy(&printed.stdout)
    );

    let labels: Vec<&str> = written
        .lines()
        .map(|line| line.split_once(':').map_or(line, |(label, _)| label))
        .collect();
    assert_eq!(
        labels,
        ["error", "cause", "hint", "exit"],
        "{spelled}: {written}"
    );

    let done = if removal {
        "nothing was removed"
    } else {
        "nothing was read or written"
    };
    assert_eq!(
        line(&written, "error: "),
        format!(".tpl/{within} is a symbolic link; {done}"),
        "{spelled}"
    );
    assert!(
        line(&written, "cause: ").starts_with(&format!("{} is a symbolic link", link.display())),
        "{spelled}: {written}"
    );
    assert_eq!(
        line(&written, "hint:  "),
        format!("rm {}", link.display()),
        "{spelled}"
    );
    assert!(line(&written, "exit:  ").starts_with("78 "), "{spelled}");
}

/// The canonical `.tpl` of `sandbox`, which the diagnostics name.
fn tpl(sandbox: &Sandbox) -> PathBuf {
    std::fs::canonicalize(sandbox.path(".tpl")).expect("the project exists")
}

// ------------------------------------------------------------ controls ---

#[test]
fn the_seeded_store_is_a_hit_for_every_listing_while_no_link_stands() {
    // The control of every refusal below: without a link, the store written
    // by hand serves each listing, the dump, the summary, a render and the
    // status report, from the cache and with no server.
    let sandbox = project();

    for arguments in [
        &["schema", "info"][..],
        &["schema", "tables"][..],
        &["schema", "views"][..],
        &["schema", "routines"][..],
        &["schema", "dump"][..],
        &["render", "plain"][..],
        &["cache", "status"][..],
    ] {
        let printed = sandbox.run(arguments);
        assert_eq!(
            printed.status.code(),
            Some(0),
            "{arguments:?}: {}",
            stderr(&printed)
        );
    }

    let dumped = sandbox.run(&["schema", "dump"]);
    assert!(
        String::from_utf8_lossy(&dumped.stdout).contains(r#""source":"cache""#),
        "the dump was not served from the cache"
    );
}

// ---------------------------------------------------- FR-CACHE-044, #305 ---

#[test]
fn fr_cache_044_a_linked_cache_root_refuses_every_command_that_reads_or_writes_the_cache() {
    // The link's target holds a whole store for the entry, so every listing
    // through it would be a hit: the hit is refused as the miss is, and
    // `cache load`, which would otherwise connect, is refused before it does.
    for arguments in READERS {
        let sandbox = project();
        let away = Sandbox::new();
        let target = outside(&away, "cache");
        seed(&target.join(ENTRY));
        std::fs::remove_dir_all(sandbox.path(".tpl/.cache")).expect("the store is ours");
        symlink(&target, sandbox.path(".tpl/.cache")).expect("the sandbox is writable");
        let before = contents(&target);

        let printed = sandbox.run(arguments);

        refused(
            &printed,
            &arguments.join(" "),
            &tpl(&sandbox),
            ".cache",
            false,
        );
        assert_eq!(
            contents(&target),
            before,
            "{arguments:?} touched the target"
        );
    }
}

#[test]
fn fr_cache_044_a_linked_entry_folder_refuses_every_command_that_reads_or_writes_the_cache() {
    for arguments in READERS {
        let sandbox = project();
        let away = Sandbox::new();
        let target = outside(&away, ENTRY);
        seed(&target);
        let folder = sandbox.path(&format!(".tpl/.cache/{ENTRY}"));
        std::fs::remove_dir_all(&folder).expect("the store is ours");
        symlink(&target, &folder).expect("the sandbox is writable");
        let before = contents(&target);

        let printed = sandbox.run(arguments);

        refused(
            &printed,
            &arguments.join(" "),
            &tpl(&sandbox),
            &format!(".cache/{ENTRY}"),
            false,
        );
        assert_eq!(
            contents(&target),
            before,
            "{arguments:?} touched the target"
        );
    }
}

#[test]
fn fr_cache_044_a_linked_collection_folder_refuses_every_command_that_would_reach_it() {
    // Each of these reads the tables folder on a hit or writes it on a miss.
    for arguments in [
        &["schema", "info"][..],
        &["schema", "tables"][..],
        &["schema", "table", "orders"][..],
        &["schema", "views"][..],
        &["schema", "dump"][..],
        &["render", "plain"][..],
        &["cache", "load"][..],
        &["cache", "load", "--table", "orders"][..],
        &["cache", "status"][..],
    ] {
        let sandbox = project();
        let away = Sandbox::new();
        let target = outside(&away, "tables");
        let folder = sandbox.path(&format!(".tpl/.cache/{ENTRY}/tables"));
        std::fs::remove_dir_all(&folder).expect("the store is ours");
        symlink(&target, &folder).expect("the sandbox is writable");
        let before = contents(&target);

        let printed = sandbox.run(arguments);

        refused(
            &printed,
            &arguments.join(" "),
            &tpl(&sandbox),
            &format!(".cache/{ENTRY}/tables"),
            false,
        );
        assert_eq!(
            contents(&target),
            before,
            "{arguments:?} touched the target"
        );
    }
}

#[test]
fn fr_cache_044_a_read_that_writes_nothing_is_refused_only_for_the_collections_it_reads() {
    // Under `--no-cache` a listing of views reads the views folder and writes
    // nothing, so a linked tables folder is not on its path: it is served from
    // the cache. A listing of tables under `--no-cache` still reads the link,
    // and is refused.
    let sandbox = project();
    let away = Sandbox::new();
    let target = outside(&away, "tables");
    let folder = sandbox.path(&format!(".tpl/.cache/{ENTRY}/tables"));
    std::fs::remove_dir_all(&folder).expect("the store is ours");
    symlink(&target, &folder).expect("the sandbox is writable");
    let before = contents(&target);

    let views = sandbox.run(&["schema", "views", "--no-cache"]);
    assert_eq!(views.status.code(), Some(0), "{}", stderr(&views));

    let tables = sandbox.run(&["schema", "tables", "--no-cache"]);
    refused(
        &tables,
        "schema tables --no-cache",
        &tpl(&sandbox),
        &format!(".cache/{ENTRY}/tables"),
        false,
    );
    assert_eq!(contents(&target), before);
}

#[test]
fn fr_cache_044_direct_with_no_cache_touches_no_cache_and_is_not_refused() {
    // FR-CACHE-016: the pure read neither looks up nor writes, so the link is
    // not on its path. It goes to the server — which this project cannot
    // reach — and the link's target is untouched either way.
    let sandbox = project();
    let away = Sandbox::new();
    let target = outside(&away, "cache");
    seed(&target.join(ENTRY));
    std::fs::remove_dir_all(sandbox.path(".tpl/.cache")).expect("the store is ours");
    symlink(&target, sandbox.path(".tpl/.cache")).expect("the sandbox is writable");
    let before = contents(&target);

    for arguments in [
        &["schema", "tables", "--direct", "--no-cache"][..],
        &["render", "plain", "--direct", "--no-cache"][..],
    ] {
        let printed = sandbox.run(arguments);
        let written = stderr(&printed);

        assert_ne!(printed.status.code(), Some(78), "{arguments:?}: {written}");
        assert!(
            !written.contains("symbolic link"),
            "{arguments:?} was refused for the link: {written}"
        );
    }
    assert_eq!(contents(&target), before);
}

// ---------------------------------------------------- FR-CACHE-042, #288 ---

#[test]
fn fr_cache_042_a_linked_cache_root_refuses_every_form_of_clean_and_removes_nothing() {
    for arguments in [
        &["cache", "clean"][..],
        &["cache", "clean", "--table", "orders"][..],
        &["cache", "clean", "--view", "v_orders"][..],
        &["cache", "clean", "--routine", "procedure:p_orders"][..],
        // A name no entry declares, per FR-CACHE-041.
        &["-d", "ghost", "cache", "clean"][..],
    ] {
        let sandbox = project();
        let away = Sandbox::new();
        let target = outside(&away, "cache");
        seed(&target.join(ENTRY));
        seed(&target.join("ghost"));
        std::fs::write(target.join(ENTRY).join("tables/orders.json"), "{}\n")
            .expect("the sandbox is writable");
        std::fs::remove_dir_all(sandbox.path(".tpl/.cache")).expect("the store is ours");
        symlink(&target, sandbox.path(".tpl/.cache")).expect("the sandbox is writable");
        let before = contents(&target);

        let printed = sandbox.run(arguments);

        refused(
            &printed,
            &arguments.join(" "),
            &tpl(&sandbox),
            ".cache",
            true,
        );
        assert_eq!(
            contents(&target),
            before,
            "{arguments:?} touched the target"
        );
        assert!(
            std::fs::symlink_metadata(sandbox.path(".tpl/.cache"))
                .expect("the link stands")
                .file_type()
                .is_symlink(),
            "{arguments:?} removed the link"
        );
    }
}

#[test]
fn fr_cache_042_a_clean_given_an_object_refuses_a_linked_entry_or_collection_folder() {
    for (linked, within) in [
        (ENTRY.to_owned(), format!(".cache/{ENTRY}")),
        (format!("{ENTRY}/tables"), format!(".cache/{ENTRY}/tables")),
    ] {
        let sandbox = project();
        let away = Sandbox::new();
        let target = outside(&away, "target");
        seed(&target);
        std::fs::write(target.join("orders.json"), "{}\n").expect("the sandbox is writable");
        std::fs::write(target.join("tables/orders.json"), "{}\n").expect("the sandbox is writable");
        let folder = sandbox.path(&format!(".tpl/.cache/{linked}"));
        std::fs::remove_dir_all(&folder).expect("the store is ours");
        symlink(&target, &folder).expect("the sandbox is writable");
        let before = contents(&target);

        let printed = sandbox.run(&["cache", "clean", "--table", "orders"]);

        refused(
            &printed,
            "cache clean --table orders",
            &tpl(&sandbox),
            &within,
            true,
        );
        assert_eq!(
            contents(&target),
            before,
            "{linked}: the target was touched"
        );
    }
}

#[test]
fn fr_cache_042_a_linked_entry_folder_is_removed_as_a_link_by_a_whole_clean() {
    // Item 2: the entry folder is the thing a clean with no object flag
    // removes, so a link there is removed and what it points at is kept.
    for arguments in [
        &["cache", "clean"][..],
        &["-d", "ghost", "cache", "clean"][..],
    ] {
        let sandbox = project();
        let name = if arguments[0] == "-d" { "ghost" } else { ENTRY };
        let away = Sandbox::new();
        let target = outside(&away, "target");
        seed(&target);
        let folder = sandbox.path(&format!(".tpl/.cache/{name}"));
        let _ = std::fs::remove_dir_all(&folder);
        symlink(&target, &folder).expect("the sandbox is writable");
        let before = contents(&target);

        let printed = sandbox.run(arguments);

        assert_eq!(
            printed.status.code(),
            Some(0),
            "{arguments:?}: {}",
            stderr(&printed)
        );
        assert!(
            std::fs::symlink_metadata(&folder).is_err(),
            "{arguments:?}: the link is still there"
        );
        assert_eq!(
            contents(&target),
            before,
            "{arguments:?} touched the target"
        );
    }
}

#[test]
fn fr_cache_042_a_linked_object_file_is_removed_as_a_link() {
    let sandbox = project();
    let away = Sandbox::new();
    let target = outside(&away, "target");
    let planted = target.join("orders.json");
    std::fs::write(&planted, "{}\n").expect("the sandbox is writable");
    let file = sandbox.path(&format!(".tpl/.cache/{ENTRY}/tables/orders.json"));
    symlink(&planted, &file).expect("the sandbox is writable");
    let before = contents(&target);

    let printed = sandbox.run(&["cache", "clean", "--table", "orders"]);

    assert_eq!(printed.status.code(), Some(0), "{}", stderr(&printed));
    assert!(
        std::fs::symlink_metadata(&file).is_err(),
        "the link is still there"
    );
    assert_eq!(contents(&target), before, "the target was touched");
}

#[test]
fn fr_cache_042_a_link_beneath_a_removed_folder_is_removed_and_not_followed() {
    // Item 3.
    let sandbox = project();
    let away = Sandbox::new();
    let target = outside(&away, "target");
    std::fs::write(target.join("orders.json"), "{}\n").expect("the sandbox is writable");
    symlink(
        &target,
        sandbox.path(&format!(".tpl/.cache/{ENTRY}/tables/away")),
    )
    .expect("the sandbox is writable");
    symlink(
        target.join("orders.json"),
        sandbox.path(&format!(".tpl/.cache/{ENTRY}/views/orders.json")),
    )
    .expect("the sandbox is writable");
    let before = contents(&target);

    let printed = sandbox.run(&["cache", "clean"]);

    assert_eq!(printed.status.code(), Some(0), "{}", stderr(&printed));
    assert!(
        std::fs::symlink_metadata(sandbox.path(&format!(".tpl/.cache/{ENTRY}"))).is_err(),
        "the entry folder is still there"
    );
    assert_eq!(contents(&target), before, "the target was touched");
}

// ---------------------------------------------------- FR-CACHE-043, #302 ---

/// `meta.json` of `sandbox`'s store, parsed.
fn record(sandbox: &Sandbox) -> serde_json::Value {
    let bytes = std::fs::read_to_string(sandbox.path(&format!(".tpl/.cache/{ENTRY}/meta.json")))
        .expect("the record is there");
    serde_json::from_str(&bytes).expect("the record is JSON")
}

#[test]
fn fr_cache_043_a_partial_clean_changes_only_the_collection_flag_and_keeps_the_load_time() {
    for (flag, name, file, collection) in [
        ("--table", "orders", "tables/orders.json", 0),
        ("--view", "v_orders", "views/v_orders.json", 1),
        (
            "--routine",
            "procedure:p_orders",
            "routines/procedure.p_orders.json",
            2,
        ),
    ] {
        let sandbox = project();
        sandbox.write(&format!(".tpl/.cache/{ENTRY}/{file}"), "{}\n");
        let mut expected = record(&sandbox);
        expected["collections"][collection]["whole"] = serde_json::Value::Bool(false);

        let printed = sandbox.run(&["cache", "clean", flag, name]);

        assert_eq!(
            printed.status.code(),
            Some(0),
            "{flag}: {}",
            stderr(&printed)
        );
        assert_eq!(record(&sandbox), expected, "{flag}");
        assert_eq!(
            record(&sandbox)["loaded_at"],
            "2026-09-01T00:00:00Z",
            "{flag}"
        );
    }
}

#[test]
fn fr_cache_043_an_absent_or_unusable_record_is_not_written() {
    // Absent: the clean creates none.
    let sandbox = project();
    let meta = sandbox.path(&format!(".tpl/.cache/{ENTRY}/meta.json"));
    std::fs::remove_file(&meta).expect("the record is ours");
    sandbox.write(&format!(".tpl/.cache/{ENTRY}/tables/orders.json"), "{}\n");

    let printed = sandbox.run(&["cache", "clean", "--table", "orders"]);
    assert_eq!(printed.status.code(), Some(0), "{}", stderr(&printed));
    assert!(!meta.exists(), "an absent record was written");

    // Unusable under FR-CDOC-004: the bytes are kept as they are.
    let sandbox = project();
    let meta = sandbox.path(&format!(".tpl/.cache/{ENTRY}/meta.json"));
    let unknown = META.replace(r#""cache_format":1"#, r#""cache_format":99"#);
    std::fs::write(&meta, &unknown).expect("the record is ours");
    sandbox.write(&format!(".tpl/.cache/{ENTRY}/tables/orders.json"), "{}\n");

    let printed = sandbox.run(&["cache", "clean", "--table", "orders"]);
    assert_eq!(printed.status.code(), Some(0), "{}", stderr(&printed));
    assert_eq!(
        std::fs::read_to_string(&meta).expect("the record is kept"),
        unknown
    );
}

#[test]
fn fr_sec_026_a_link_at_the_record_is_an_unusable_record_and_is_neither_read_nor_written_through() {
    // Finding F2: `meta.json` is read through the guard of an object file, so
    // a link there — here to a valid, whole record outside the project — is
    // an unusable record. A partial clean therefore writes no record, per
    // FR-CACHE-043, and leaves the link as it found it; a listing is not
    // served from the record it points at; and the target is untouched.
    let sandbox = project();
    let away = Sandbox::new();
    let target = outside(&away, "target");
    std::fs::write(target.join("meta.json"), META).expect("the sandbox is writable");
    let meta = sandbox.path(&format!(".tpl/.cache/{ENTRY}/meta.json"));
    std::fs::remove_file(&meta).expect("the record is ours");
    symlink(target.join("meta.json"), &meta).expect("the sandbox is writable");
    let object = sandbox.write(&format!(".tpl/.cache/{ENTRY}/tables/orders.json"), "{}\n");
    let before = contents(&target);

    let printed = sandbox.run(&["cache", "clean", "--table", "orders"]);
    assert_eq!(printed.status.code(), Some(0), "{}", stderr(&printed));
    assert!(!object.exists(), "the object was not removed");
    assert!(
        std::fs::symlink_metadata(&meta)
            .expect("the link is there")
            .file_type()
            .is_symlink(),
        "an unusable record was written"
    );

    let status = sandbox.run(&["cache", "status", "--format", "json"]);
    assert_eq!(status.status.code(), Some(0), "{}", stderr(&status));
    assert!(
        String::from_utf8_lossy(&status.stdout).contains(r#""loaded_at":null"#),
        "the record was read through the link: {}",
        String::from_utf8_lossy(&status.stdout)
    );

    let listed = sandbox.run(&["schema", "tables", "--format", "json"]);
    assert!(
        !String::from_utf8_lossy(&listed.stdout).contains(r#""source":"cache""#),
        "a listing was served from the record the link points at"
    );

    assert_eq!(contents(&target), before, "the target was touched");
}

// ----------------------------------------- FR-CACHE-034 and FR-CACHE-035 ---

/// The `data` of `tpl cache status --format json` in `sandbox`, with its text
/// form, having asserted both exited `0`.
fn status(sandbox: &Sandbox) -> (serde_json::Value, String) {
    let json = sandbox.run(&["cache", "status", "--format", "json"]);
    assert_eq!(json.status.code(), Some(0), "{}", stderr(&json));
    let text = sandbox.run(&["cache", "status"]);
    assert_eq!(text.status.code(), Some(0), "{}", stderr(&text));

    let document: serde_json::Value =
        serde_json::from_slice(&json.stdout).expect("the report is JSON");

    (
        document["data"].clone(),
        String::from_utf8(text.stdout).expect("the report is UTF-8"),
    )
}

/// What `FR-CACHE-034` reports with no usable record beside one table and one
/// view file.
fn unrecorded() -> serde_json::Value {
    serde_json::json!({
        "entry": ENTRY,
        "loaded_at": null,
        "collections": [
            {"name": "tables", "count": 1, "whole": false},
            {"name": "views", "count": 1, "whole": false},
            {"name": "routines", "count": 0, "whole": false},
        ],
    })
}

/// Asserts the text form reports no load time without calling the cache
/// empty, and shows the counts.
fn assert_unrecorded_text(text: &str) {
    let loaded_at = line(text, "loaded_at");
    assert!(loaded_at.starts_with("not recorded"), "{text}");
    assert!(!text.contains("empty"), "{text}");
    assert!(text.contains("tables"), "{text}");
    assert!(text.contains("views"), "{text}");
}

#[test]
fn fr_cache_034_an_absent_record_beside_object_files_reports_the_counts_and_no_load_time() {
    let sandbox = project();
    std::fs::remove_file(sandbox.path(&format!(".tpl/.cache/{ENTRY}/meta.json")))
        .expect("the record is ours");
    sandbox.write(&format!(".tpl/.cache/{ENTRY}/tables/orders.json"), "{}\n");
    sandbox.write(&format!(".tpl/.cache/{ENTRY}/views/v_orders.json"), "{}\n");

    let (data, text) = status(&sandbox);

    assert_eq!(data, unrecorded());
    assert_unrecorded_text(&text);
}

#[test]
fn fr_cache_034_a_linked_record_beside_object_files_is_not_read_through() {
    // FR-CDOC-017: the link is an unusable record. The valid record it points
    // at, with its load time and its flags, is not reported.
    let sandbox = project();
    let away = Sandbox::new();
    let target = outside(&away, "target");
    std::fs::write(target.join("meta.json"), META).expect("the sandbox is writable");
    let meta = sandbox.path(&format!(".tpl/.cache/{ENTRY}/meta.json"));
    std::fs::remove_file(&meta).expect("the record is ours");
    symlink(target.join("meta.json"), &meta).expect("the sandbox is writable");
    sandbox.write(&format!(".tpl/.cache/{ENTRY}/tables/orders.json"), "{}\n");
    sandbox.write(&format!(".tpl/.cache/{ENTRY}/views/v_orders.json"), "{}\n");
    let before = contents(&target);

    let (data, text) = status(&sandbox);

    assert_eq!(data, unrecorded());
    assert_unrecorded_text(&text);
    assert_eq!(contents(&target), before, "the target was touched");
}

#[test]
fn fr_cache_035_an_empty_cache_still_reports_no_collections() {
    // No store at all, and a store folder holding no record and no object
    // file: both are the empty cache, with its text unchanged.
    let absent = Sandbox::new();
    absent.project(CONFIGURATION);

    let bare = project();
    std::fs::remove_file(bare.path(&format!(".tpl/.cache/{ENTRY}/meta.json")))
        .expect("the record is ours");

    for sandbox in [&absent, &bare] {
        let (data, text) = status(sandbox);

        assert_eq!(
            data,
            serde_json::json!({"entry": ENTRY, "loaded_at": null, "collections": []})
        );
        assert!(
            text.contains("loaded_at  never (the cache is empty; fill it with tpl cache load)\n"),
            "{text}"
        );
    }
}
