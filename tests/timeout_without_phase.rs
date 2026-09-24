//! `--timeout` where the invocation runs no blocking phase: accepted, of no
//! effect, and reported nowhere (`FR-GLOB-026`, rmp `#273`).
//!
//! Every body runs the same command twice, in two sandboxes built alike, once
//! with `--timeout 1` and once without, and compares stdout, stderr, the exit
//! code and the project left behind, byte for byte. No body measures time: a
//! command that runs no phase spends none of the budget, so the outcome does
//! not depend on how long the command takes.

#[path = "support/sandbox.rs"]
mod sandbox;

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::process::Output;

use sandbox::Sandbox;

/// One entry, selected by default, whose server nothing listens on: a command
/// that reached for it would exit `69`, not `0`.
const CONFIGURATION: &str = "[core]\ndatabase = \"shop\"\n\n[database.shop]\nhost = \"127.0.0.1\"\n\
                             port = 9\nuser = \"reader\"\ndatabase = \"shop\"\n";

/// A `meta.json` recording all three collections whole.
const META: &str = r#"{"cache_format":1,"schema_version":1,"loaded_at":"2026-09-01T00:00:00Z","collections":[{"name":"tables","whole":true},{"name":"views","whole":true},{"name":"routines","whole":true}]}
"#;

/// The database's own metadata, as the store holds it.
const DATABASE: &str = r#"{"name":"shop","charset":"utf8mb4","collation":"utf8mb4_general_ci","server":{"version":"11.4.13-MariaDB","series":"11.4","standing":"supported"}}
"#;

/// How a sandbox is prepared before the command runs.
#[derive(Clone, Copy)]
enum Setup {
    /// Nothing: `tpl init` creates the project.
    Empty,
    /// A project with a template and a whole, empty cache for `shop`.
    Project,
}

/// A sandbox prepared as `setup` says.
fn prepared(setup: Setup) -> Sandbox {
    let sandbox = Sandbox::new();

    if let Setup::Project = setup {
        sandbox.project(CONFIGURATION);
        sandbox.write(".tpl/templates/plain.jinja", "{{ database.name }}\n");
        let store = sandbox.path(".tpl/.cache/shop");
        for collection in ["tables", "views", "routines"] {
            std::fs::create_dir_all(store.join(collection)).expect("the sandbox is writable");
        }
        std::fs::write(store.join("meta.json"), META).expect("the sandbox is writable");
        std::fs::write(store.join("database.json"), DATABASE).expect("the sandbox is writable");
    }

    sandbox
}

/// Every file beneath `root`, by path relative to it, with its bytes.
fn tree(root: &Path) -> BTreeMap<PathBuf, Vec<u8>> {
    let mut found = BTreeMap::new();
    let mut pending = vec![root.to_owned()];

    while let Some(current) = pending.pop() {
        let Ok(entries) = std::fs::read_dir(&current) else {
            continue;
        };
        for entry in entries {
            let path = entry.expect("the sandbox is ours").path();
            if path.is_dir() {
                pending.push(path);
            } else {
                let relative = path
                    .strip_prefix(root)
                    .expect("beneath the root")
                    .to_owned();
                found.insert(relative, std::fs::read(&path).expect("the file is ours"));
            }
        }
    }

    found
}

/// Runs `arguments` in a sandbox prepared as `setup` says, with `--timeout 1`
/// in front where `timed`, and returns the output and the tree left behind.
fn outcome(setup: Setup, arguments: &[&str], timed: bool) -> (Output, BTreeMap<PathBuf, Vec<u8>>) {
    let sandbox = prepared(setup);
    let mut vector: Vec<&str> = Vec::new();
    if timed {
        vector.extend(["--timeout", "1"]);
    }
    vector.extend_from_slice(arguments);

    let printed = sandbox.run(&vector);

    (printed, tree(sandbox.root()))
}

#[test]
fn fr_glob_026_timeout_has_no_effect_where_no_blocking_phase_runs() {
    for (setup, arguments) in [
        (Setup::Project, &["cfg", "list"][..]),
        (
            Setup::Project,
            &["cfg", "set", "core.query_timeout", "60"][..],
        ),
        (Setup::Project, &["template", "list"][..]),
        (Setup::Project, &["template", "check", "plain"][..]),
        (Setup::Empty, &["init"][..]),
        (Setup::Project, &["help"][..]),
        (Setup::Project, &["help", "cfg", "set"][..]),
        (Setup::Project, &["version"][..]),
        (Setup::Project, &["cache", "status"][..]),
        (Setup::Project, &["cache", "status", "--format", "json"][..]),
        (Setup::Project, &["cache", "clean"][..]),
        // A read served wholly from the cache runs no phase either: the store
        // above holds a whole, empty catalogue, and a miss would exit 69.
        (Setup::Project, &["schema", "tables"][..]),
        (Setup::Project, &["schema", "dump"][..]),
    ] {
        let (without, left_without) = outcome(setup, arguments, false);
        let (with, left_with) = outcome(setup, arguments, true);

        assert_eq!(
            without.status.code(),
            Some(0),
            "{arguments:?}: {}",
            String::from_utf8_lossy(&without.stderr)
        );
        assert_eq!(with.status.code(), without.status.code(), "{arguments:?}");
        assert_eq!(with.stdout, without.stdout, "{arguments:?}: stdout differs");
        assert_eq!(
            String::from_utf8_lossy(&with.stderr),
            String::from_utf8_lossy(&without.stderr),
            "{arguments:?}: --timeout was reported"
        );
        assert_eq!(
            left_with, left_without,
            "{arguments:?}: the project differs"
        );
    }
}
