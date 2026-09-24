//! The six forms of `FR-HELP-001`, their three equivalences, and the JSON
//! command tree — asserted on the bytes **the process** emits.
//!
//! These properties cannot be established from inside the crate. `FR-HELP-002`
//! makes `tpl help <path>` and `tpl <path> --help` byte-identical at every
//! depth, and the two are distinct code paths through the parser: one reaches
//! the renderer as a command with a positional argument, the other as a global
//! flag answered at whatever node the vector reached. A unit test calling the
//! renderer twice would prove only that it was called twice. `BR-HELP-001`
//! therefore asks for the comparison to be made on the output, and this is it.
//!
//! **Every test walks the tree rather than naming nodes.** The list of nodes is
//! read from `tpl help --format json`, which `FR-HELP-016` makes the whole
//! surface as one document and which a unit test of `cli::help::document` holds
//! to the tree the binary parses with, node by node. A node added to the tree
//! is therefore covered here without anything in this file changing.
//!
//! **Every run is made in an empty directory with no `.tpl` above it, under a
//! cleared environment.** `FR-PROJ-025` exempts these forms from discovery and
//! from reading configuration, and `FR-CLI-021` forbids reading an environment
//! variable to decide behaviour at all; the harness asserts the precondition
//! rather than assuming it, because a test that happened to run inside a
//! project would pass for the wrong reason.

use std::path::{Path, PathBuf};
use std::process::{Command, Output};

/// The line `FR-HELP-005` fixes, for the version this binary was built at.
///
/// At version 0.1.0 it is exactly `tpl 0.1.0` and one newline. The number is
/// read from the package rather than written out, so the test states the rule
/// and not one of its instances.
const VERSION_LINE: &str = concat!("tpl ", env!("CARGO_PKG_VERSION"), "\n");

/// The seven sections of `FR-HELP-006`, in the one order it fixes.
const SECTIONS: [&str; 7] = [
    "USAGE",
    "DESCRIPTION",
    "ARGUMENTS",
    "OPTIONS",
    "EXAMPLES",
    "EXIT CODES",
    "SEE ALSO",
];

/// The seven global flags of `FR-GLOB-001`, as the document spells them.
const GLOBALS: [&str; 7] = [
    "--database",
    "--tpl-dir",
    "--timeout",
    "--verbose",
    "--quiet",
    "--help",
    "--version",
];

/// An empty directory with no `.tpl` folder anywhere above it.
///
/// It is created once per test binary and never written to: every run of the
/// binary under test uses it as its working directory, so a project the
/// developer happens to be standing in cannot be discovered. The precondition
/// is checked rather than assumed, per this file's own note.
fn sandbox() -> PathBuf {
    let directory = std::env::temp_dir().join(format!("tpl-help-surface-{}", std::process::id()));

    std::fs::create_dir_all(&directory).expect("the sandbox directory is created");
    assert_no_project_above(&directory);

    directory
}

/// Asserts that no `.tpl` folder exists at `directory` or at any ancestor of
/// it.
fn assert_no_project_above(directory: &Path) {
    for ancestor in directory.ancestors() {
        assert!(
            !ancestor.join(".tpl").exists(),
            "the sandbox is inside a project, at {}",
            ancestor.display()
        );
    }
}

/// Runs the binary under test on `arguments`, in the sandbox, with nothing in
/// its environment.
fn run(arguments: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_tpl"))
        .env_clear()
        .current_dir(sandbox())
        .args(arguments)
        .output()
        .expect("the binary under test runs")
}

/// Runs the binary under test and returns what it wrote to stdout, having
/// asserted that it succeeded silently.
fn succeeds(arguments: &[&str]) -> Vec<u8> {
    let printed = run(arguments);

    assert_eq!(
        printed.status.code(),
        Some(0),
        "tpl {} did not exit 0: {}",
        arguments.join(" "),
        String::from_utf8_lossy(&printed.stderr)
    );
    assert!(
        printed.stderr.is_empty(),
        "tpl {} wrote to stderr: {}",
        arguments.join(" "),
        String::from_utf8_lossy(&printed.stderr)
    );

    printed.stdout
}

/// The command tree, as `tpl help --format json` emits it.
fn document() -> serde_json::Value {
    serde_json::from_slice(&succeeds(&["help", "--format", "json"]))
        .expect("the command tree is a JSON document")
}

/// The path of every node of the tree, the root first, as the document
/// publishes them.
///
/// The root carries no entry of `data.commands`, per `FR-HELP-019`, so it is
/// the empty path added here: it is a node of the tree all the same, and
/// `FR-HELP-002` reaches it.
fn node_paths() -> Vec<Vec<String>> {
    let document = document();
    let commands = document["data"]["commands"]
        .as_array()
        .expect("data.commands is an array");

    let mut paths = Vec::with_capacity(commands.len() + 1);
    paths.push(Vec::new());

    for entry in commands {
        paths.push(
            entry["path"]
                .as_array()
                .expect("an entry carries its path as an array")
                .iter()
                .map(|segment| segment.as_str().expect("a segment is a string").to_owned())
                .collect(),
        );
    }

    paths
}

/// `path` with `trailing` appended, as an argument vector.
fn with(path: &[String], trailing: &[&str]) -> Vec<String> {
    path.iter()
        .cloned()
        .chain(trailing.iter().map(|&token| token.to_owned()))
        .collect()
}

/// `arguments` as the slice [`run`] takes.
fn borrowed(arguments: &[String]) -> Vec<&str> {
    arguments.iter().map(String::as_str).collect()
}

/// The path as a caller writes it, for a failure message.
fn written(path: &[String]) -> String {
    std::iter::once("tpl")
        .chain(path.iter().map(String::as_str))
        .collect::<Vec<&str>>()
        .join(" ")
}

// ---------------------------------------------------------------------------
// The six forms and their equivalences
// ---------------------------------------------------------------------------

#[test]
fn fr_help_002_the_three_help_forms_are_byte_identical_at_every_node_of_the_tree() {
    // FR-HELP-002 and BR-HELP-001, over the whole tree rather than a sample:
    // `tpl help <path>`, `tpl <path> --help` and `tpl <path> -h` are three
    // distinct routes through the parser and must arrive at the same bytes at
    // every depth. The empty path is the root, so `tpl help`, `tpl --help` and
    // `tpl -h` are the first case rather than a case of their own.
    for path in node_paths() {
        let through_help = succeeds(&borrowed(&with(&["help".to_owned()], &borrowed(&path))));
        let through_long = succeeds(&borrowed(&with(&path, &["--help"])));
        let through_short = succeeds(&borrowed(&with(&path, &["-h"])));

        assert_eq!(
            through_help,
            through_long,
            "tpl help {} and {} --help differ",
            path.join(" "),
            written(&path)
        );
        assert_eq!(
            through_long,
            through_short,
            "{} --help and {} -h differ",
            written(&path),
            written(&path)
        );
    }
}

#[test]
fn fr_help_003_the_short_help_flag_is_not_a_summarised_long_one() {
    // FR-HELP-003: `-h` prints everything `--help` prints, EXAMPLES, EXIT
    // CODES, types and defaults included. The equality above already fixes it;
    // this asserts the content of what both print, so that the two agreeing on
    // a summary would still fail. A leaf with a required operand and a flag
    // that carries a default is the case that exercises every clause.
    let printed = String::from_utf8(succeeds(&["schema", "table", "-h"])).expect("UTF-8");

    for section in SECTIONS {
        assert!(
            printed.contains(section),
            "the short form omits {section}: {printed}"
        );
    }

    assert!(printed.contains("Required."), "{printed}");
    assert!(printed.contains("Default: \"text\"."), "{printed}");
    assert!(printed.contains("Type: one of text, json."), "{printed}");
    assert!(printed.contains("Not repeatable."), "{printed}");
}

#[test]
fn fr_glob_019_a_leaf_that_requires_an_operand_still_answers_both_flag_forms() {
    // The defect this effort resolves. `OD-07` turns the parser's own help
    // flag off, so `--help` is an ordinary global argument and the parser
    // validates required arguments first; without the waiver, every node that
    // declares an operand answered `tpl <node> --help` with a missing-argument
    // 64. The nodes are read from the document rather than listed, so the
    // assertion reaches all fourteen required arguments of the tree and any
    // added later.
    let document = document();
    let mut asserted = 0_usize;

    for entry in document["data"]["commands"]
        .as_array()
        .expect("data.commands is an array")
    {
        let required = entry["arguments"]
            .as_array()
            .expect("an entry carries its arguments as an array")
            .iter()
            .filter(|argument| argument["required"] == serde_json::Value::Bool(true))
            .count();

        if required == 0 {
            continue;
        }

        asserted += required;

        let path: Vec<String> = entry["path"]
            .as_array()
            .expect("an entry carries its path as an array")
            .iter()
            .map(|segment| segment.as_str().expect("a segment is a string").to_owned())
            .collect();

        for flag in [
            &["--help"][..],
            &["-h"][..],
            &["--version"][..],
            &["-V"][..],
        ] {
            succeeds(&borrowed(&with(&path, flag)));
        }

        // And the operand is still required: the waiver reaches the two flags
        // and nothing else.
        let refused = run(&borrowed(&path));

        assert_eq!(
            refused.status.code(),
            Some(64),
            "{} accepted a missing operand",
            written(&path)
        );
        assert!(
            String::from_utf8_lossy(&refused.stderr).contains("needs the argument"),
            "{} did not report the missing operand: {}",
            written(&path),
            String::from_utf8_lossy(&refused.stderr)
        );
    }

    assert_eq!(
        asserted, 14,
        "the tree declares a different number of required arguments than this effort waived"
    );
}

#[test]
fn fr_help_005_the_three_version_forms_write_exactly_the_line_of_the_requirement() {
    // FR-HELP-005 and the third equivalence of FR-HELP-002: exactly
    // `tpl <version>` and a single newline, and nothing else, from all three.
    for form in [&["version"][..], &["--version"][..], &["-V"][..]] {
        let printed = succeeds(form);

        assert_eq!(
            String::from_utf8(printed).expect("UTF-8"),
            VERSION_LINE,
            "tpl {} did not write the version line",
            form.join(" ")
        );
    }
}

#[test]
fn fr_help_027_an_alias_prints_the_same_bytes_as_the_node_it_names() {
    // FR-HELP-027 with FR-CLI-011: each of the seven aliases resolves to its
    // canonical node, in the text form and in the JSON one alike, at every
    // depth the alias appears at.
    for (canonical, alias) in [
        (&["schema", "tables"][..], &["schema", "tbls"][..]),
        (&["schema", "table"][..], &["schema", "tbl"][..]),
        (&["schema", "views"][..], &["schema", "vws"][..]),
        (&["schema", "view"][..], &["schema", "vw"][..]),
        (&["schema", "routines"][..], &["schema", "rtns"][..]),
        (&["schema", "routine"][..], &["schema", "rtn"][..]),
        (&["cfg", "database"][..], &["cfg", "db"][..]),
        (&["cfg", "database", "add"][..], &["cfg", "db", "add"][..]),
    ] {
        let mut through_canonical = vec!["help"];
        through_canonical.extend_from_slice(canonical);

        let mut through_alias = vec!["help"];
        through_alias.extend_from_slice(alias);

        assert_eq!(
            succeeds(&through_canonical),
            succeeds(&through_alias),
            "tpl help {} and tpl help {} differ",
            canonical.join(" "),
            alias.join(" ")
        );

        through_canonical.extend_from_slice(&["--format", "json"]);
        through_alias.extend_from_slice(&["--format", "json"]);

        assert_eq!(
            succeeds(&through_canonical),
            succeeds(&through_alias),
            "the documents of {} and {} differ",
            canonical.join(" "),
            alias.join(" ")
        );
    }
}

#[test]
fn fr_help_028_a_segment_that_names_no_child_is_refused_with_the_node_it_was_sought_under() {
    // FR-HELP-028: exit 64, a nearest-match suggestion over the children of
    // the node reached and over nothing else, and a cause naming both the
    // segment and that node. Asserted on the process because FR-ERR-033 makes
    // the four labelled lines on stderr the whole of what a caller receives.
    let refused = run(&["help", "cfg", "database", "ad"]);

    assert_eq!(refused.status.code(), Some(64));
    assert!(
        refused.stdout.is_empty(),
        "a refusal wrote to stdout: {}",
        String::from_utf8_lossy(&refused.stdout)
    );

    let reported = String::from_utf8(refused.stderr).expect("UTF-8");

    assert!(reported.contains("did you mean 'add'?"), "{reported}");
    assert!(reported.contains("exit:  64 (EX_USAGE)"), "{reported}");

    let cause = reported
        .lines()
        .find(|line| line.starts_with("cause: "))
        .expect("the four labelled lines carry a cause");

    assert!(cause.contains("'ad'"), "{cause}");
    assert!(cause.contains("'tpl cfg database'"), "{cause}");

    // The same in the JSON form: the path is refused before any document is
    // emitted, per FR-HELP-029.
    let refused = run(&["help", "cfg", "database", "ad", "--format", "json"]);

    assert_eq!(refused.status.code(), Some(64));
    assert!(refused.stdout.is_empty());
}

#[test]
fn fr_proj_025_every_form_succeeds_where_no_project_exists_and_nothing_is_in_the_environment() {
    // FR-PROJ-025 and NFR-PERF-005: none of these forms performs discovery,
    // reads configuration or opens a connection, so each succeeds in a
    // directory with no `.tpl` above it and with an empty environment — which
    // is the state a calling agent's first invocation is made in.
    for form in [
        &["help"][..],
        &["--help"][..],
        &["-h"][..],
        &["help", "cfg", "database", "add"][..],
        &["cfg", "database", "add", "--help"][..],
        &["cfg", "database", "add", "-h"][..],
        &["version"][..],
        &["--version"][..],
        &["-V"][..],
        &["help", "--format", "json"][..],
        &["help", "cfg", "--format", "json"][..],
    ] {
        let printed = succeeds(form);

        assert!(!printed.is_empty(), "tpl {} wrote nothing", form.join(" "));
    }
}

#[test]
fn fr_help_025_a_group_node_with_no_child_prints_what_its_help_form_prints() {
    // FR-HELP-025 with FR-CLI-007: exactly the text `tpl help <node>` would
    // print, and exit 0. The six group nodes are read from the document — a
    // node with at least one child whose own path is a prefix of that child's.
    let paths = node_paths();

    for path in &paths {
        let is_group = paths
            .iter()
            .any(|other| other.len() == path.len() + 1 && other.starts_with(path));

        if !is_group {
            continue;
        }

        assert_eq!(
            succeeds(&borrowed(path)),
            succeeds(&borrowed(&with(&["help".to_owned()], &borrowed(path)))),
            "{} bare and tpl help {} differ",
            written(path),
            path.join(" ")
        );
    }
}

// ---------------------------------------------------------------------------
// The JSON command tree
// ---------------------------------------------------------------------------

#[test]
fn fr_help_017_the_document_is_the_envelope_of_the_contract_carrying_the_five_keys_of_data() {
    // FR-HELP-017 with FR-OUT-024 and FR-OUT-026: the three envelope keys in
    // order, `source` set to `binary`, and a `data` carrying `tpl_version`,
    // `global_flags`, `commands`, `template_surface` and `context_variables`
    // in that order. The
    // bytes are compared rather than the parsed value, because the order is
    // the property under test and parsing discards it.
    let printed = String::from_utf8(succeeds(&["help", "--format", "json"])).expect("UTF-8");

    assert!(
        printed.starts_with(concat!(
            r#"{"schema_version":1,"source":"binary","data":{"tpl_version":""#,
            env!("CARGO_PKG_VERSION"),
            r#"","global_flags":["#
        )),
        "{}",
        &printed[..printed.len().min(200)]
    );

    for (first, second) in [
        (r#""tpl_version""#, r#""global_flags""#),
        (r#""global_flags""#, r#""commands""#),
        (r#""commands""#, r#""template_surface""#),
        (r#""template_surface""#, r#""context_variables""#),
    ] {
        assert!(
            printed.find(first) < printed.find(second),
            "{first} does not precede {second}"
        );
    }
}

#[test]
fn fr_help_024_the_document_is_compact_by_default_and_indented_under_pretty() {
    // FR-HELP-024 with FR-OUT-007 and FR-OUT-008: one line and one terminating
    // newline by default, a two-space indent with one key per line under
    // --pretty, and the same document either way.
    let compact = String::from_utf8(succeeds(&["help", "--format", "json"])).expect("UTF-8");

    assert_eq!(
        compact.matches('\n').count(),
        1,
        "the compact document is not one line"
    );
    assert!(
        compact.ends_with("}\n"),
        "the terminator is not one newline"
    );
    // No superfluous whitespace: the three envelope keys are written without a
    // space after the colon or the comma, which the prefix asserted by the test
    // above fixes for the whole document — the encoder does not vary its
    // spacing from one key to the next.
    assert!(
        compact.starts_with(r#"{"schema_version":1,"source":"binary","data":{"#),
        "{}",
        &compact[..compact.len().min(80)]
    );

    let indented =
        String::from_utf8(succeeds(&["help", "--format", "json", "--pretty"])).expect("UTF-8");

    assert!(
        indented.starts_with("{\n  \"schema_version\": 1,\n"),
        "{}",
        &indented[..indented.len().min(80)]
    );
    assert!(indented.ends_with("}\n"));
    assert!(!indented.ends_with("}\n\n"));
    assert!(
        indented.len() > compact.len(),
        "the indented form is not the larger one"
    );

    let one: serde_json::Value = serde_json::from_str(&compact).expect("compact is JSON");
    let other: serde_json::Value = serde_json::from_str(&indented).expect("indented is JSON");

    assert_eq!(one, other, "--pretty changed more than the whitespace");
}

#[test]
fn fr_help_029_a_path_reduces_the_document_to_the_subtree_it_roots() {
    // FR-HELP-029: `data.commands` reduced to the entry whose path is the path
    // given, together with every entry whose path extends it segment by
    // segment, in the order they hold unreduced. Every other key of `data` is
    // emitted unreduced.
    let whole = document();
    let reduced: serde_json::Value =
        serde_json::from_slice(&succeeds(&["help", "cfg", "database", "--format", "json"]))
            .expect("the subtree is a JSON document");

    let selected: Vec<&serde_json::Value> = whole["data"]["commands"]
        .as_array()
        .expect("data.commands is an array")
        .iter()
        .filter(|entry| {
            let path = entry["path"].as_array().expect("a path is an array");

            path.len() >= 2 && path[0] == "cfg" && path[1] == "database"
        })
        .collect();

    assert_eq!(
        reduced["data"]["commands"]
            .as_array()
            .expect("data.commands is an array")
            .iter()
            .collect::<Vec<&serde_json::Value>>(),
        selected
    );
    assert_eq!(
        selected.len(),
        7,
        "the subtree is the node and its six children"
    );

    assert_eq!(reduced["schema_version"], whole["schema_version"]);
    assert_eq!(reduced["source"], whole["source"]);
    assert_eq!(reduced["data"]["tpl_version"], whole["data"]["tpl_version"]);
    assert_eq!(
        reduced["data"]["global_flags"],
        whole["data"]["global_flags"]
    );
    assert_eq!(
        reduced["data"]["template_surface"],
        whole["data"]["template_surface"]
    );
}

#[test]
fn fr_help_018_the_global_flags_are_carried_once_and_no_command_repeats_them() {
    // FR-HELP-018 with FR-GLOB-003, asserted on what a caller receives: the
    // seven appear in `data.global_flags`, each command says
    // `inherits_globals` instead, and no command's `options` names one of them.
    let document = document();

    assert_eq!(
        document["data"]["global_flags"]
            .as_array()
            .expect("data.global_flags is an array")
            .iter()
            .map(|flag| flag["long"].as_str().expect("a long form"))
            .collect::<Vec<&str>>(),
        GLOBALS
    );

    for entry in document["data"]["commands"]
        .as_array()
        .expect("data.commands is an array")
    {
        assert_eq!(
            entry["inherits_globals"],
            serde_json::Value::Bool(true),
            "{} does not inherit the global flags",
            entry["path"]
        );

        for flag in entry["options"]
            .as_array()
            .expect("an entry carries its options as an array")
        {
            let long = flag["long"].as_str().expect("a long form");

            assert!(
                !GLOBALS.contains(&long),
                "{} repeats the global flag {long}",
                entry["path"]
            );
        }
    }
}

#[test]
fn nfr_det_001_the_document_is_byte_identical_between_runs() {
    // NFR-DET-001 over this document, and the observable consequence of
    // FR-HELP-023 and FR-OUT-013: an unordered map anywhere on the emitting
    // path would reorder some run's keys, and the order of the whole document
    // is a property a caller parses against.
    assert_eq!(
        succeeds(&["help", "--format", "json"]),
        succeeds(&["help", "--format", "json"])
    );
    assert_eq!(
        succeeds(&["help", "cfg", "--format", "json"]),
        succeeds(&["help", "cfg", "--format", "json"])
    );
}

// ---------------------------------------------------------------------------
// The statement of purpose, and the sixth fact
// ---------------------------------------------------------------------------

/// The prefixes a requirement identifier of this corpus is written with.
///
/// `FR-HELP-014` makes help self-contained — it refers the reader to no
/// document outside the help system — and a requirement identifier is a
/// citation of one, which a caller reading help cannot resolve.
const IDENTIFIERS: [&str; 8] = ["FR-", "NFR-", "BR-", "UC-", "OD-", "ADR-", "OQ-", "DIV-"];

/// The `purpose` of one entry of `options`, `arguments` or `global_flags`, with
/// the name it is stated for.
fn purpose(argument: &serde_json::Value) -> (String, String) {
    let name = argument
        .get("long")
        .or_else(|| argument.get("name"))
        .and_then(serde_json::Value::as_str)
        .expect("an argument is named")
        .to_owned();
    let stated = argument["purpose"]
        .as_str()
        .expect("an argument states its purpose")
        .to_owned();

    (name, stated)
}

/// Every (node path, argument name, sentence) the document publishes, walked
/// from the document itself rather than enumerated.
fn every_purpose() -> Vec<(String, String, String)> {
    let document = document();
    let mut stated = Vec::new();

    for flag in document["data"]["global_flags"]
        .as_array()
        .expect("data.global_flags is an array")
    {
        let (name, sentence) = purpose(flag);
        stated.push((String::from("tpl"), name, sentence));
    }

    for entry in document["data"]["commands"]
        .as_array()
        .expect("data.commands is an array")
    {
        let path = entry["path"]
            .as_array()
            .expect("an entry carries its path")
            .iter()
            .map(|segment| segment.as_str().expect("a segment is a string").to_owned())
            .collect::<Vec<String>>();

        for argument in entry["options"]
            .as_array()
            .expect("an entry carries its options")
            .iter()
            .chain(
                entry["arguments"]
                    .as_array()
                    .expect("an entry carries its arguments"),
            )
        {
            let (name, sentence) = purpose(argument);
            stated.push((written(&path), name, sentence));
        }
    }

    stated
}

#[test]
fn fr_help_030_every_flag_and_argument_of_the_document_states_what_supplying_it_does() {
    // FR-HELP-030: one sentence per flag and per positional argument, in the
    // `options` and `arguments` arrays of every command and in
    // `data.global_flags`. The population is read from the document, so an
    // argument added to the tree is covered here without this test changing —
    // which is what enumerating the arguments by hand would not give.
    let stated = every_purpose();

    assert!(
        stated.len() > 100,
        "only {} arguments were walked",
        stated.len()
    );

    for (at, name, sentence) in stated {
        assert!(
            !sentence.trim().is_empty(),
            "{at} states an empty purpose for {name}"
        );
        assert!(
            sentence.ends_with('.'),
            "{at} states no sentence for {name}: {sentence:?}"
        );

        for prefix in IDENTIFIERS {
            assert!(
                !sentence.contains(prefix),
                "{at} cites {prefix} in the purpose of {name}: {sentence:?}"
            );
        }
    }
}

#[test]
fn fr_help_030_the_sentence_of_the_document_is_the_sentence_of_the_text_help() {
    // FR-HELP-022 makes the typed table the one source of the sentence for both
    // channels and forbids deriving either from the other. What a caller can
    // check from outside is that the two agree, node by node.
    for (path, help) in node_paths()
        .into_iter()
        .map(|path| {
            let text = String::from_utf8(succeeds(&borrowed(&with(&path, &["--help"]))))
                .expect("help is text");
            (path, text)
        })
        .collect::<Vec<(Vec<String>, String)>>()
    {
        let at = written(&path);
        let collapsed = help.split_whitespace().collect::<Vec<&str>>().join(" ");

        for (stated_at, name, sentence) in every_purpose() {
            // A global flag is stated once, at the root, per FR-GLOB-003; a
            // local one at the node that declares it.
            let expected = if stated_at == "tpl" {
                path.is_empty()
            } else {
                stated_at == at
            };

            if !expected {
                continue;
            }

            assert!(
                collapsed.contains(&sentence),
                "{at} does not carry the sentence of {name}: {sentence:?}"
            );
        }
    }
}

#[test]
fn fr_help_013_the_mutual_exclusion_of_the_two_verbosity_flags_is_named_in_help() {
    // FR-HELP-013's sixth fact, for the one pair that is global. It appeared in
    // no help text of the tree: the root's `64` line carries an unknown
    // command, an unknown flag and a repeated flag value, and never this pair.
    let root = String::from_utf8(succeeds(&["--help"])).expect("help is text");
    let collapsed = root.split_whitespace().collect::<Vec<&str>>().join(" ");

    assert!(
        collapsed.contains("-q, --quiet"),
        "the root does not list --quiet"
    );
    assert!(
        collapsed.contains("Not to be given with --verbose."),
        "the root does not name the exclusion on --quiet: {root}"
    );
    assert!(
        collapsed.contains("Not to be given with --quiet."),
        "the root does not name the exclusion on --verbose: {root}"
    );
}

#[test]
fn fr_help_013_every_exclusion_the_document_declares_is_named_in_the_text_help_of_its_node() {
    // The same walk over the other channel: whatever the document states about
    // an argument's exclusions reaches the reader of the text help of the node
    // that declares it. A pair stated in one channel and not the other is the
    // inconsistency FR-HELP-022 exists to prevent.
    let document = document();
    let mut met = 0usize;

    let root = String::from_utf8(succeeds(&["--help"])).expect("help is text");
    let root = root.split_whitespace().collect::<Vec<&str>>().join(" ");

    for flag in document["data"]["global_flags"]
        .as_array()
        .expect("data.global_flags is an array")
    {
        for excluded in flag["excludes"]
            .as_array()
            .expect("a flag carries its exclusions")
        {
            met += 1;
            let excluded = excluded.as_str().expect("an exclusion is a string");

            assert!(
                root.contains(&format!("Not to be given with {excluded}")),
                "tpl --help does not name the exclusion of {excluded}"
            );
        }
    }

    for entry in document["data"]["commands"]
        .as_array()
        .expect("data.commands is an array")
    {
        let path: Vec<String> = entry["path"]
            .as_array()
            .expect("an entry carries its path")
            .iter()
            .map(|segment| segment.as_str().expect("a segment is a string").to_owned())
            .collect();

        let mut declared: Vec<&str> = Vec::new();

        for argument in entry["options"]
            .as_array()
            .expect("an entry carries its options")
            .iter()
            .chain(
                entry["arguments"]
                    .as_array()
                    .expect("an entry carries its arguments"),
            )
        {
            for excluded in argument["excludes"]
                .as_array()
                .expect("an argument carries its exclusions")
            {
                declared.push(excluded.as_str().expect("an exclusion is a string"));
            }
        }

        if declared.is_empty() {
            continue;
        }

        let help = String::from_utf8(succeeds(&borrowed(&with(&path, &["--help"]))))
            .expect("help is text");
        let collapsed = help.split_whitespace().collect::<Vec<&str>>().join(" ");

        let named = excluded_in(&collapsed);

        for excluded in declared {
            met += 1;

            assert!(
                named.iter().any(|stated| stated == excluded),
                "{} does not name the exclusion of {excluded}",
                written(&path)
            );
        }
    }

    assert!(met >= 20, "only {met} exclusions were walked");
}

/// Every argument named by an exclusion sentence of a collapsed help text.
///
/// The sentence is `Not to be given with A, B or C.`, so the names are read
/// from it rather than searched for one at a time: a search would pass for a
/// name that appears anywhere in the help, which is most of them.
fn excluded_in(collapsed: &str) -> Vec<String> {
    const OPENING: &str = "Not to be given with ";

    let mut named = Vec::new();

    for sentence in collapsed.split(OPENING).skip(1) {
        let listed = sentence
            .split_once(". ")
            .map_or(sentence, |(head, _)| head)
            .trim_end_matches('.');

        named.extend(
            listed
                .split([','])
                .flat_map(|part| part.split(" or "))
                .map(|part| part.trim().to_owned())
                .filter(|part| !part.is_empty()),
        );
    }

    named
}

// ---------------------------------------------------------------------------
// What a leaf touches, and what a template sees
// ---------------------------------------------------------------------------

/// The `DESCRIPTION` section of `help`, its lines joined by one space.
fn description(help: &str) -> String {
    help.split_once("DESCRIPTION\n")
        .and_then(|(_, rest)| rest.split("\nARGUMENTS\n").next())
        .and_then(|rest| rest.split("\nOPTIONS\n").next())
        .and_then(|rest| rest.split("\nEXAMPLES\n").next())
        .map(|section| section.split_whitespace().collect::<Vec<&str>>().join(" "))
        .unwrap_or_default()
}

#[test]
fn fr_help_031_every_leaf_ends_its_description_with_the_four_statements() {
    // FR-HELP-031: the first statement answers the server question, the
    // second the entry question, and both channels end with the same four.
    // Each of the four is recognised by the words the first two open with;
    // the text and the JSON are compared against each other rather than
    // against a copy of the sentences.
    let document = document();
    let commands = document["data"]["commands"]
        .as_array()
        .expect("commands is an array");

    for path in node_paths() {
        let is_leaf = !commands.iter().any(|other| {
            let segments = other["path"].as_array().expect("path is an array");
            segments.len() == path.len() + 1
                && segments
                    .iter()
                    .zip(&path)
                    .all(|(segment, own)| segment.as_str() == Some(own.as_str()))
        });

        if !is_leaf {
            continue;
        }

        let text = description(
            &String::from_utf8(succeeds(&borrowed(&with(&path, &["--help"])))).expect("UTF-8"),
        );
        let entry = commands
            .iter()
            .find(|entry| {
                entry["path"].as_array().is_some_and(|segments| {
                    segments
                        .iter()
                        .map(|s| s.as_str())
                        .eq(path.iter().map(|s| Some(s.as_str())))
                })
            })
            .expect("the node has an entry");
        let json = entry["description"].as_str().expect("a description");
        let closing = json.rsplit("\n\n").next().expect("a last paragraph");

        assert!(
            closing.starts_with("Connects")
                || closing.starts_with("Always connects")
                || closing.starts_with("Does not contact the server."),
            "{} does not open its last paragraph with the server statement: {closing}",
            written(&path)
        );
        assert!(
            closing.contains(" Needs "),
            "{} states no entry requirement: {closing}",
            written(&path)
        );
        assert!(
            closing.contains(" Writes ")
                || closing.contains(" Stores ")
                || closing.contains(" Deletes "),
            "{} states no file statement: {closing}",
            written(&path)
        );
        assert!(
            closing.contains(" Prints "),
            "{} states nothing about stdout: {closing}",
            written(&path)
        );
        assert!(
            text.ends_with(closing),
            "{} ends its text DESCRIPTION differently from its JSON description",
            written(&path)
        );
    }
}

#[test]
fn fr_help_033_the_help_of_render_lists_every_variable_and_every_guaranteed_name() {
    // FR-HELP-033: every context variable, and every filter, test and
    // function of groups 1 and 2 with its signature and purpose, read from
    // the JSON document so that both channels are held to one statement.
    let document = document();
    let render = description(&String::from_utf8(succeeds(&["render", "--help"])).expect("UTF-8"));

    for variable in document["data"]["context_variables"]
        .as_array()
        .expect("an array")
    {
        let line = format!(
            "{} {}",
            variable["name"].as_str().expect("a name"),
            variable["purpose"].as_str().expect("a purpose")
        );
        assert!(render.contains(&line), "render help omits: {line}");
    }

    let surface = &document["data"]["template_surface"];
    let mut listed = 0;

    for group in ["registered", "inherited"] {
        for family in ["filters", "tests", "functions"] {
            for item in surface[group][family].as_array().expect("an array") {
                let line = format!(
                    "{} {}",
                    item["signature"].as_str().expect("a signature"),
                    item["purpose"].as_str().expect("a purpose")
                );
                assert!(render.contains(&line), "render help omits: {line}");
                listed += 1;
            }
        }
    }

    assert_eq!(
        listed, 37,
        "groups 1 and 2 hold 23 registered and 14 inherited names"
    );
    assert!(
        render.contains(
            "Everything else the template engine offers also works, but carries no guarantee"
        ),
        "render help does not state that group 3 is unguaranteed"
    );

    // The list appears in no other node's help, per BR-HELP-002.
    for path in node_paths() {
        if path == ["render"] {
            continue;
        }

        let help =
            String::from_utf8(succeeds(&borrowed(&with(&path, &["--help"])))).expect("UTF-8");
        assert!(
            !help.contains("A template sees these variables"),
            "{} repeats the template surface",
            written(&path)
        );
    }
}

#[test]
fn fr_help_022_an_example_invocation_is_the_vector_the_shell_would_pass() {
    // BR-HELP-003 asks that an example parse; a vector that carries the
    // shell's own quotes parses and still is not what the shell passes. A
    // line whose token the shell would expand has no vector to publish.
    let document = document();

    for entry in document["data"]["commands"].as_array().expect("an array") {
        for example in entry["examples"].as_array().expect("an array") {
            for line in example["lines"].as_array().expect("an array") {
                let text = line["text"].as_str().expect("a text");
                let Some(invocation) = line["invocation"].as_array() else {
                    continue;
                };

                assert!(
                    !text.contains("\"$"),
                    "{text} expands a variable and still publishes a vector"
                );

                for token in invocation {
                    let token = token.as_str().expect("a token");
                    assert!(
                        !token.starts_with(['"', '\'']) && !token.ends_with(['"', '\'']),
                        "{text} carries the shell's quotes: {token:?}"
                    );
                }
            }
        }
    }
}

#[test]
fn fr_help_013_a_default_the_configuration_applies_is_stated_where_the_flag_is() {
    // `tpl cfg database add` writes no key for a flag it is not given, and the
    // configuration then gives `port` 3306 and `tls` verify-identity. Help says
    // so on the flag, in both channels; `update` leaves an absent field as it
    // was, so it states no default.
    let add = String::from_utf8(succeeds(&["cfg", "database", "add", "--help"])).expect("UTF-8");
    let collapsed = add.split_whitespace().collect::<Vec<&str>>().join(" ");

    assert!(
        collapsed.contains("Type: integer. Default: \"3306\"."),
        "{add}"
    );
    assert!(collapsed.contains("Default: \"verify-identity\"."), "{add}");

    let update =
        String::from_utf8(succeeds(&["cfg", "database", "update", "--help"])).expect("UTF-8");
    assert!(!update.contains("Default: \"3306\""), "{update}");

    let document = document();
    let options = document["data"]["commands"]
        .as_array()
        .expect("an array")
        .iter()
        .find(|entry| entry["path"] == serde_json::json!(["cfg", "database", "add"]))
        .expect("the entry")["options"]
        .clone();
    let port = options
        .as_array()
        .expect("an array")
        .iter()
        .find(|flag| flag["long"] == "--port")
        .expect("--port");

    assert_eq!(port["default"], serde_json::json!("3306"));
}

/// Every string the JSON command tree carries, in document order.
fn strings_of(value: &serde_json::Value, into: &mut Vec<String>) {
    match value {
        serde_json::Value::String(text) => into.push(text.clone()),
        serde_json::Value::Array(items) => items.iter().for_each(|item| strings_of(item, into)),
        serde_json::Value::Object(members) => {
            members.values().for_each(|member| strings_of(member, into));
        }
        _ => {}
    }
}

/// The text help of `path`, with its line breaks and indentation folded into
/// single spaces so that a sentence reads the same wherever it wrapped.
fn folded_help(path: &[&str]) -> String {
    let mut arguments = vec!["help"];
    arguments.extend_from_slice(path);
    let text = String::from_utf8(succeeds(&arguments)).expect("help is UTF-8");

    text.split_whitespace().collect::<Vec<_>>().join(" ")
}

#[test]
fn r_269_the_re_audit_texts_are_stated_in_the_text_help_and_in_the_json_tree() {
    // The findings of the re-audit for rmp #269 whose fix is a sentence of
    // help: each is read in the text help of the node that states it and
    // among the strings of the JSON tree, which FR-HELP-019 makes one content.
    let cases: [(&[&str], &str); 16] = [
        // R-04: the one-string form, its splitting, and the stored array.
        (
            &["cfg", "set"],
            "Given to cfg set as one string, a command line such as \"pass db/shop\", never as \
             an array; split into words as a shell would (quotes group words, and every quote \
             must be closed); stored in the file as an array, [\"pass\", \"db/shop\"]. A value \
             that starts with an unquoted [, or ends in a backslash outside quotes, is refused.",
        ),
        (
            &["cfg", "database", "add"],
            "tpl splits the string into words as a shell would (quotes group words, and every \
             quote must be closed) and stores them in the file as an array, [\"pass\", \
             \"db/shop\"]. A value that starts with an unquoted [, or ends in a backslash \
             outside quotes, is refused.",
        ),
        // R-05: column() returns the column object.
        (
            &["render"],
            "Returns the column called name in the table called table, as the same object \
             table.columns holds;",
        ),
        // R-07: the missing clauses of the 64 and 65 rows.
        (
            &["render"],
            "An unknown flag, a missing TEMPLATE, or a flag given twice;",
        ),
        (
            &["render"],
            "or TEMPLATE resolves to a path outside .tpl/templates/.",
        ),
        (
            &["cache", "load"],
            "An unknown flag, or a flag given twice; --no-cache;",
        ),
        // R-08: what text output is, per command, and the header line.
        (
            &["cfg", "list"],
            "text is the file itself, comments included, with passwords redacted;",
        ),
        (
            &["template", "list"],
            "Prints a NAME header line, then one template name per line.",
        ),
        // R-10: the object flags say what the command does with the object.
        (
            &["cache", "clean"],
            "Deletes only this table's cached copy.",
        ),
        // FR-HELP-034.
        (
            &["init"],
            "--tpl-dir has no effect here: the project is created at PATH, or in the current \
             directory when PATH is absent.",
        ),
        // The walkthrough: a local server, and a password from the environment.
        (&[], "--tls disabled"),
        (&[], "tpl cfg set database.shop.password '${SHOP_PASSWORD}'"),
        // T-06 of the third re-audit: the 64 conditions every command shares,
        // the ambiguous bare --routine, and what a fourth -v does.
        (
            &["version"],
            "On any command, also -v with -q, or a global flag given a value it does not take, \
             such as --timeout 0.",
        ),
        (
            &["help"],
            "On any command, also -v with -q, or a global flag given a value it does not take, \
             such as --timeout 0.",
        ),
        (
            &["render"],
            "a bare --routine NAME that names both a procedure and a function;",
        ),
        (&[], "A fourth -v, or more, changes nothing."),
    ];

    let mut strings = Vec::new();
    strings_of(&document(), &mut strings);
    let folded: Vec<String> = strings
        .iter()
        .map(|text| text.split_whitespace().collect::<Vec<_>>().join(" "))
        .collect();

    for (path, sentence) in cases {
        assert!(
            folded_help(path).contains(sentence),
            "tpl help {} does not state: {sentence}",
            path.join(" ")
        );
        assert!(
            folded.iter().any(|text| text.contains(sentence)),
            "the JSON tree does not state: {sentence}"
        );
    }

    // The every-table loops say they need jq.
    for path in [&["render"][..], &["schema", "tables"][..]] {
        assert!(
            folded_help(path).contains("this needs jq, an external JSON tool."),
            "tpl help {}",
            path.join(" ")
        );
    }
}

#[test]
fn r_269_every_example_that_pipes_into_jq_says_it_needs_jq() {
    // An example that runs jq, an external tool, says so in its caption, in
    // the words the every-table loops use; the root is not a command entry,
    // and its examples run no jq.
    let tree = document();
    let commands = tree["data"]["commands"]
        .as_array()
        .expect("the tree lists its commands");
    let mut seen = 0;

    for command in commands {
        for example in command["examples"].as_array().into_iter().flatten() {
            let runs_jq = example["lines"]
                .as_array()
                .into_iter()
                .flatten()
                .any(|line| {
                    line["text"]
                        .as_str()
                        .is_some_and(|text| text.contains("jq "))
                });
            if runs_jq {
                seen += 1;
                let caption = example["caption"].as_str().unwrap_or_default();
                assert!(
                    caption.ends_with("; this needs jq, an external JSON tool."),
                    "{}: {caption}",
                    command["path"]
                );
            }
        }
    }

    assert!(seen >= 7, "only {seen} examples run jq");
}

#[test]
fn r_269_the_password_rows_of_add_and_update_state_the_password_given_twice() {
    for (path, sentence) in [
        (
            &["cfg", "database", "add"][..],
            "the password given twice, as a password inside --dsn and as --password-command;",
        ),
        (
            &["cfg", "database", "update"][..],
            "the password given twice, as a password inside the dsn or password and as \
             --password-command;",
        ),
    ] {
        assert!(
            folded_help(path).contains(sentence),
            "tpl help {} does not state: {sentence}",
            path.join(" ")
        );
    }
}

/// The help of `path`, as text with its lines rejoined, so that a sentence
/// the layout wrapped can be found whole.
fn prose(path: &[&str]) -> String {
    let mut arguments = vec!["help"];
    arguments.extend_from_slice(path);
    let written = String::from_utf8(succeeds(&arguments)).expect("the help is UTF-8");

    written.split_whitespace().collect::<Vec<_>>().join(" ")
}

#[test]
fn rmp_274_the_help_states_what_the_second_re_audit_found_it_left_out() {
    let render = prose(&["render"]);
    for stated in [
        // S-01: the object flags bind one object; database stays whole.
        "The template always sees the whole database as database; --table, --view or \
         --routine also binds that one object as table, view or routine.",
        "Binds this table as the variable table; database still holds every table.",
        "Binds this view as the variable view; database still holds every view.",
        // S-11 and FR-HELP-013: the exclusion is stated on both flags.
        "Not to be given with --database or --direct.",
        "Not to be given with --context.",
        "or --context together with -d/--database or --direct.",
        // --no-cache is accepted and ignored with --context.
        "It has no effect with --context, which uses no cache.",
    ] {
        assert!(render.contains(stated), "tpl help render lacks {stated:?}");
    }
    assert!(!render.contains("Acts on this one"), "{render}");

    // S-02: what an entry needs before it can connect.
    let add = prose(&["cfg", "database", "add"]);
    assert!(
        add.contains(
            "To connect, the entry needs --host and --schema, or a --dsn that names both; --port \
             defaults to 3306 and --user is optional. An entry without them is stored, but every \
             later command that connects with it exits 78."
        ),
        "{add}"
    );

    // S-05 and FR-TMPL-032.
    assert!(
        prose(&["template", "check"]).contains("every failing template is reported"),
        "tpl help template check"
    );

    // S-09: the conditions the 78 and 64 rows left out.
    for path in [
        &["schema", "tables"][..],
        &["render"][..],
        &["cfg", "database", "test"][..],
    ] {
        let text = prose(path);
        for stated in ["password_command failed", "ca_path holds no certificate"] {
            assert!(text.contains(stated), "tpl help {path:?} lacks {stated:?}");
        }
    }
    assert!(prose(&["help"]).contains("An unknown flag, a flag given twice"));
    assert!(prose(&["version"]).contains("a global flag that takes a value given twice"));

    // The walkthrough: --schema is glossed, and listing points at JSON.
    let root = prose(&[]);
    assert!(
        root.contains("Its --schema is the name of the database on that server"),
        "{root}"
    );
    assert!(root.contains("tpl schema tables --format json"), "{root}");
}

#[test]
fn u_02_every_help_that_reads_the_cache_says_it_never_expires_in_the_same_words() {
    // Finding U-02 of the fourth re-audit of rmp #263.
    const RULE: &str = "Nothing in the cache expires: after the database structure changes, run \
                        tpl cache load or add --direct.";
    let mut paths: Vec<Vec<&str>> = vec![vec![], vec!["render"]];
    for leaf in [
        "tables", "table", "views", "view", "routines", "routine", "info", "dump",
    ] {
        paths.push(vec!["schema", leaf]);
    }

    for path in &paths {
        let text = prose(path);
        assert_eq!(text.matches(RULE).count(), 1, "tpl help {path:?}: {text}");
    }
}

#[test]
fn u_04_no_example_names_a_missing_template_unannounced_or_truncates_a_file_it_renders_into() {
    // Finding U-04 of the fourth re-audit of rmp #263: the template commands
    // name the templates tpl init creates, and a render into a file goes
    // through a temporary one.
    let tree = document();
    let commands = tree["data"]["commands"]
        .as_array()
        .expect("the tree lists its commands");

    for command in commands {
        let path: Vec<&str> = command["path"]
            .as_array()
            .into_iter()
            .flatten()
            .filter_map(serde_json::Value::as_str)
            .collect();
        let path = path.join(" ");
        for example in command["examples"].as_array().into_iter().flatten() {
            let lines: Vec<&str> = example["lines"]
                .as_array()
                .into_iter()
                .flatten()
                .filter_map(|line| line["text"].as_str())
                .collect();
            let joined = lines.join("\n");
            if path.starts_with("template ") {
                assert!(!joined.contains("rust/struct"), "{path}: {joined}");
                assert!(!joined.contains("docs/table.md"), "{path}: {joined}");
            }
            if joined.contains(" render ") && joined.contains(" > ") {
                for line in lines.iter().filter(|line| line.contains(" > ")) {
                    assert!(line.contains(".tmp"), "{path}: {line}");
                }
                assert!(joined.contains("mv "), "{path}: {joined}");
            }
        }
    }
    assert!(
        prose(&["render"]).contains(
            "The examples below use rust/struct and docs/table.md as stand-ins for templates of \
             your own, which must exist before they render."
        ),
        "tpl help render"
    );
}

#[test]
fn fr_help_035_every_flag_and_argument_default_is_null_or_one_string() {
    let document = document();
    let mut flags: Vec<serde_json::Value> = document["data"]["global_flags"]
        .as_array()
        .expect("an array")
        .clone();
    for entry in document["data"]["commands"].as_array().expect("an array") {
        flags.extend(
            entry["options"]
                .as_array()
                .expect("an array")
                .iter()
                .cloned(),
        );
    }

    assert!(!flags.is_empty());
    for flag in &flags {
        let default = &flag["default"];
        assert!(
            default.is_null() || default.is_string(),
            "{}: {default}",
            if flag["long"].is_null() {
                &flag["name"]
            } else {
                &flag["long"]
            }
        );
    }
}
