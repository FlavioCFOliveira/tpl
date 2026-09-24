//! The argument vector is untrusted input, and this is what the process does
//! with a hostile one.
//!
//! Five properties of this corpus cannot be established from inside the crate,
//! because each is a property of the **process** rather than of a function:
//!
//! | Property | Requirement |
//! |---|---|
//! | A rejected token reaches stderr escaped, whatever it carries | `FR-ERR-024` |
//! | One rejected token reaches the message, and never the vector | `FR-GLOB-018`, and the composition note of `OD-08` |
//! | No `tpl-<token>` executable is looked for on `PATH` | `FR-CLI-006` |
//! | No environment variable decides anything | `FR-CLI-021`, `NFR-DET-004` |
//! | The two cache flags are declared by exactly ten commands and are unknown to every other | `FR-CACHE-017`, `FR-CACHE-020` |
//!
//! The fifth is here and not beside the store it governs, because it is settled
//! from the invocation alone: the ten that declare the flags and the twenty-three
//! that refuse them are decided at step 1 of `FR-ERR-006`, before a project is
//! discovered and with no cache in existence.
//!
//! A unit test over the renderer would show that the renderer escapes; it would
//! not show that the argument vector reaches the renderer, that nothing else
//! writes to the stream, or that the parser did not consult `PATH` on the way.
//! Every test here therefore runs the binary and reads the bytes it emitted.
//!
//! **Every run is made in an empty directory with no `.tpl` above it, under a
//! cleared environment**, on the same terms as
//! [`help_surface`](../help_surface/index.html): a run that happened to stand
//! inside a project would pass for the wrong reason.
//!
//! **Every negative test carries a control.** A test that asserts a thing did
//! not happen is worthless unless the same harness is shown to observe the
//! thing when it does happen, so the decoy executable is run directly and the
//! environment is shown to reach a child process.

use std::path::{Path, PathBuf};
use std::process::{Command, Output};

/// The one exit code every refusal of step 1 of `FR-ERR-006` carries.
const USAGE: i32 = 64;

/// The escape byte that opens the two-character form of an ANSI sequence.
const ESCAPE: u8 = 0x1b;

/// The four labels of `FR-ERR-008`, in the order it fixes.
const LABELS: [&str; 4] = ["error: ", "cause: ", "hint:  ", "exit:  "];

/// An environment that disagrees with the empty one about everything a
/// terminal-aware tool, a colour-aware tool, or a tool with its own variables
/// would consult.
///
/// `TERM` carries an escape sequence of its own, so a run that echoed the
/// variable rather than reading it would be caught by the same assertion that
/// catches a composed one.
const HOSTILE: [(&str, &str); 14] = [
    ("TERM", "\u{1b}[31mxterm-256color"),
    ("COLORTERM", "truecolor"),
    ("CLICOLOR", "1"),
    ("CLICOLOR_FORCE", "1"),
    ("FORCE_COLOR", "3"),
    ("NO_COLOR", ""),
    ("COLUMNS", "20"),
    ("LINES", "5"),
    ("RUST_BACKTRACE", "full"),
    ("TPL_DIR", "/nonexistent/hostile"),
    ("TPL_DATABASE", "hostile"),
    ("TPL_FORMAT", "json"),
    ("HOME", "/nonexistent"),
    ("LC_ALL", "tr_TR.UTF-8"),
];

/// The marker the control of the environment test looks for.
const MARKER: (&str, &str) = ("TPL_AUDIT_MARKER", "the-environment-reached-the-child");

/// An empty directory with no `.tpl` folder anywhere above it.
fn sandbox() -> PathBuf {
    let directory = std::env::temp_dir().join(format!("tpl-invocation-{}", std::process::id()));

    std::fs::create_dir_all(&directory).expect("the sandbox directory is created");

    for ancestor in directory.ancestors() {
        assert!(
            !ancestor.join(".tpl").exists(),
            "the sandbox is inside a project, at {}",
            ancestor.display()
        );
    }

    directory
}

/// Runs the binary under test on `arguments`, in the sandbox, with nothing in
/// its environment.
fn run(arguments: &[&str]) -> Output {
    run_with(&[], arguments)
}

/// Runs the binary under test on `arguments`, in the sandbox, with `environment`
/// and nothing else.
fn run_with(environment: &[(&str, &str)], arguments: &[&str]) -> Output {
    let mut command = Command::new(env!("CARGO_BIN_EXE_tpl"));

    command.env_clear().current_dir(sandbox()).args(arguments);
    for (name, value) in environment {
        command.env(name, value);
    }

    command.output().expect("the binary under test runs")
}

/// What the run wrote to stderr, as text.
fn stderr(printed: &Output) -> String {
    String::from_utf8(printed.stderr.clone()).expect("a diagnostic is valid UTF-8")
}

/// Asserts that `printed` is a refusal of step 1: exit `64`, an empty stdout,
/// and the four labelled lines of `FR-ERR-008` in their order.
fn assert_refused(printed: &Output, spelled: &str) {
    assert_eq!(
        printed.status.code(),
        Some(USAGE),
        "{spelled} did not exit {USAGE}: {}",
        stderr(printed)
    );
    assert!(
        printed.stdout.is_empty(),
        "{spelled} wrote {} bytes to stdout, where FR-ERR-033 leaves it empty",
        printed.stdout.len()
    );

    let written = stderr(printed);
    let lines: Vec<&str> = written.lines().collect();

    assert_eq!(lines.len(), 4, "{spelled} wrote {written:?}");
    for (line, label) in lines.iter().zip(LABELS) {
        assert!(line.starts_with(label), "{spelled} wrote {line:?}");
    }
}

/// Asserts that nothing `printed` carries is a control character, on either
/// stream, other than the newline that ends each line.
///
/// This is `FR-ERR-024` read as its rationale states it — a caller reading the
/// stream line by line sees exactly the lines the renderer emitted — and
/// `NFR-DET-004` for the escape byte, which the C0 range covers.
fn assert_no_control_byte(printed: &Output, spelled: &str) {
    for (stream, bytes) in [("stdout", &printed.stdout), ("stderr", &printed.stderr)] {
        let found: Vec<String> = bytes
            .iter()
            .copied()
            .filter(|byte| *byte < 0x20 && *byte != b'\n')
            .map(|byte| format!("{byte:#04x}"))
            .collect();

        assert!(
            found.is_empty(),
            "{spelled} wrote the control bytes {found:?} to {stream}"
        );
        assert!(
            !bytes.contains(&ESCAPE),
            "{spelled} wrote an ANSI escape to {stream}"
        );
    }
}

/// A token carrying every C0 control character the operating system will carry
/// in an argument vector.
///
/// `U+0000` is absent and cannot be tested from here: an argument vector is a
/// list of NUL-terminated strings, so the kernel admits no member that contains
/// one, and `Command` refuses to spawn with one. The other thirty-one are all
/// present, `ESC` among them.
fn every_c0_control() -> String {
    (0x01u32..0x20)
        .map(|value| char::from_u32(value).expect("every C0 code point is a character"))
        .collect()
}

/// [`every_c0_control`] as `FR-ERR-024` requires it to be written.
fn every_c0_control_escaped() -> String {
    (0x01u32..0x20)
        .map(|value| match value {
            0x09 => "\\t".to_owned(),
            0x0a => "\\n".to_owned(),
            0x0d => "\\r".to_owned(),
            other => format!("\\u{{{other:02x}}}"),
        })
        .collect()
}

// ------------------------------------------------------------ FR-ERR-024 ---

#[test]
fn fr_err_024_a_command_token_a_flag_token_and_a_help_path_segment_reach_stderr_escaped() {
    // FR-ERR-024 escapes `\n`, `\r`, `\t` and every C0 control character in
    // every value a message interpolates, and names the argument vector among
    // them. The three populations below are the three ways a caller's own bytes
    // enter a message on this surface: a token read as a command, a token read
    // as a flag, and a positional segment of a help path — the last new with
    // the nested help path of FR-HELP-026.
    let hostile = every_c0_control();
    let escaped = every_c0_control_escaped();

    let flag = format!("--{hostile}");
    let vectors: [(&str, Vec<&str>, String); 3] = [
        ("a command token", vec![hostile.as_str()], escaped.clone()),
        (
            "a flag token",
            vec!["schema", "tables", flag.as_str()],
            format!("--{escaped}"),
        ),
        (
            "a help path segment",
            vec!["help", "schema", hostile.as_str()],
            escaped.clone(),
        ),
    ];

    for (population, arguments, expected) in vectors {
        let printed = run(&arguments);

        assert_refused(&printed, population);
        assert_no_control_byte(&printed, population);

        let written = stderr(&printed);
        assert!(
            written.contains(&expected),
            "{population} was not escaped as FR-ERR-024 requires: {written:?}"
        );
        assert!(
            !written.contains(&hostile),
            "{population} reached the stream unescaped: {written:?}"
        );
    }
}

#[test]
fn fr_err_024_a_token_cannot_forge_a_labelled_line_of_its_own() {
    // The rationale of FR-ERR-024: escaping the newline separately is what
    // protects the line-oriented format of FR-ERR-008 from having a whole
    // diagnostic line forged. The token below is a complete `exit` line telling
    // the caller the invocation succeeded.
    let printed = run(&["orders\nexit:  0 (EX_OK)"]);

    assert_refused(&printed, "a forged exit line");
    assert!(
        !stderr(&printed).contains("\nexit:  0 (EX_OK)"),
        "a token forged a labelled line: {:?}",
        stderr(&printed)
    );
}

// ------------------------------------------------ FR-GLOB-018 and OD-08 ---

#[test]
fn fr_glob_018_one_rejected_token_reaches_the_message_and_never_the_argument_vector() {
    // FR-GLOB-018 bars the argument vector from every diagnostic stream at
    // every verbosity, and the composition note of OD-08 reads that against the
    // `64` row of FR-ERR-034, which obliges the rejected token to be named:
    // "one rejected token is not the vector". Each token a vector does not need
    // spelled exactly is given a distinct marker, so what reached the reader is
    // read off the stream rather than argued.
    let markers: Vec<String> = (0..12).map(|index| format!("Zmarker{index:02}z")).collect();
    let noise: Vec<&str> = markers.iter().map(String::as_str).collect();

    let vectors: [(&str, Vec<&str>, usize); 6] = [
        ("an unknown command", vec![noise[0], noise[1], noise[2]], 1),
        (
            "an unknown flag",
            vec!["schema", "tables", "--nope", noise[3]],
            0,
        ),
        ("an unknown help path segment", vec!["help", noise[4]], 1),
        (
            "a help path segment under a node",
            vec!["help", "cfg", "database", noise[5], noise[6]],
            1,
        ),
        ("an unexpected argument", vec!["version", noise[7]], 1),
        (
            "a token after the terminator",
            vec!["version", "--", noise[8], noise[9]],
            1,
        ),
    ];

    for (condition, arguments, expected) in vectors {
        let printed = run(&arguments);
        assert_refused(&printed, condition);

        let written = stderr(&printed);
        let echoed: Vec<&String> = markers
            .iter()
            .filter(|marker| written.contains(marker.as_str()))
            .collect();

        assert_eq!(
            echoed.len(),
            expected,
            "{condition} put {echoed:?} on the stream: {written:?}"
        );
    }

    // Verbosity cannot widen it: -q and -v reach the same four lines, because
    // FR-GLOB-018 bars the vector "at any verbosity level".
    for level in [vec!["-q"], vec!["-v"], vec!["-v", "-v", "-v"]] {
        let mut arguments = level;
        arguments.extend([noise[10], noise[11]]);

        let printed = run(&arguments);
        assert_refused(&printed, "a refusal under an explicit verbosity");

        let written = stderr(&printed);
        assert!(written.contains(noise[10]), "{written:?}");
        assert!(
            !written.contains(noise[11]),
            "the vector reached the stream: {written:?}"
        );
    }
}

// ------------------------------------------------------------ FR-CLI-006 ---

#[test]
fn fr_cli_006_no_tpl_prefixed_executable_is_searched_for_on_path() {
    // FR-CLI-006: a token that is not a declared command SHALL NOT cause a
    // lookup of a `tpl-<token>` executable on PATH. The observable form of a
    // lookup is the execution that would follow it, so the decoy writes a file:
    // if it ever runs, the file is there to prove it.
    let directory = sandbox().join("path-decoys");
    std::fs::create_dir_all(&directory).expect("the decoy directory is created");

    let marker = directory.join("decoy-ran");
    let tokens = ["sch", "nope", "help", "schema", "version"];

    for token in tokens {
        write_decoy(&directory.join(format!("tpl-{token}")), &marker);
    }

    // The control: the decoy is an executable that runs and leaves the marker
    // when it is invoked. Without this, the assertions below would pass just as
    // well over a decoy that could never have run at all.
    let _ = std::fs::remove_file(&marker);
    let control = Command::new(directory.join("tpl-sch"))
        .output()
        .expect("the decoy is executable");
    assert!(control.status.success(), "the decoy did not run");
    assert!(marker.exists(), "the decoy ran without leaving its marker");

    let path = directory.to_str().expect("the sandbox path is UTF-8");
    for token in tokens {
        let _ = std::fs::remove_file(&marker);

        let printed = run_with(&[("PATH", path)], &[token]);

        assert!(
            !marker.exists(),
            "tpl {token} executed tpl-{token} from PATH"
        );
        assert!(
            !String::from_utf8_lossy(&printed.stdout).contains("the decoy"),
            "tpl {token} carried the output of tpl-{token}"
        );
    }

    // The two tokens that are not commands are refused as unknown commands
    // rather than handed on, which is the whole of what the requirement leaves
    // as the outcome.
    for token in ["sch", "nope"] {
        let printed = run_with(&[("PATH", path)], &[token]);

        assert_refused(&printed, token);
        assert!(
            stderr(&printed).starts_with(&format!("error: unknown command '{token}'")),
            "{token}: {:?}",
            stderr(&printed)
        );
    }
}

/// Writes an executable shell script at `at` that creates `marker` and prints a
/// line, and makes it executable.
fn write_decoy(at: &Path, marker: &Path) {
    use std::os::unix::fs::PermissionsExt as _;

    let script = format!(
        "#!/bin/sh\n: > '{}'\necho 'the decoy ran'\n",
        marker.display()
    );

    std::fs::write(at, script).expect("the decoy is written");
    std::fs::set_permissions(at, std::fs::Permissions::from_mode(0o755))
        .expect("the decoy is made executable");
}

// ------------------------------------------- FR-CLI-021 and NFR-DET-004 ---

#[test]
fn fr_cli_021_no_environment_variable_decides_a_form_of_help_of_version_or_a_refusal() {
    // FR-CLI-021 reads no environment variable to decide behaviour, defaults or
    // the location of the project, and BR-CLI-002 states the consequence: an
    // invocation is fully described by what is visible of it. The observable
    // form is that the same vector under two environments that disagree about
    // everything produces the same bytes on both streams and the same status.
    let vectors: [&[&str]; 16] = [
        &[],
        &["help"],
        &["-h"],
        &["--help"],
        &["help", "schema"],
        &["schema", "-h"],
        &["schema", "table", "--help"],
        &["help", "--format", "json"],
        &["help", "cfg", "database", "--format", "json", "--pretty"],
        &["version"],
        &["-V"],
        &["--version"],
        &["nope"],
        &["schema", "tables", "--nope"],
        &["help", "nope"],
        &["--timeout", "soon", "version"],
    ];

    // The control: the environment this harness builds does reach a child
    // process, so a run that read one would have something to read. `/usr/bin/env`
    // is present on both supported platforms.
    let delivered = Command::new("/usr/bin/env")
        .env_clear()
        .env(MARKER.0, MARKER.1)
        .output()
        .expect("/usr/bin/env runs");
    assert!(
        String::from_utf8_lossy(&delivered.stdout).contains(MARKER.1),
        "the harness did not deliver an environment to a child process"
    );

    let mut hostile: Vec<(&str, &str)> = HOSTILE.to_vec();
    hostile.push(MARKER);

    for arguments in vectors {
        let bare = run(arguments);
        let loaded = run_with(&hostile, arguments);
        let spelled = format!("tpl {}", arguments.join(" "));

        assert_eq!(
            bare.status.code(),
            loaded.status.code(),
            "{spelled} exited differently under an environment"
        );
        assert_eq!(
            bare.stdout, loaded.stdout,
            "{spelled} wrote a different stdout under an environment"
        );
        assert_eq!(
            bare.stderr, loaded.stderr,
            "{spelled} wrote a different stderr under an environment"
        );

        // NFR-DET-004 forbids colour and every ANSI escape sequence on both
        // streams "under any circumstances", and a TERM that advertises colour
        // together with CLICOLOR_FORCE is the circumstance a tool that emitted
        // any would emit it under.
        assert_no_control_byte(&loaded, &spelled);

        // Nor is a variable echoed, which is the other way one could reach a
        // reader.
        let blob = [loaded.stdout.clone(), loaded.stderr.clone()].concat();
        assert!(
            !String::from_utf8_lossy(&blob).contains(MARKER.1),
            "{spelled} echoed an environment variable"
        );
    }
}

// ----------------------------------------------- FR-CLI-017 and -CLI-019 ---

#[test]
fn fr_cli_017_the_argument_terminator_is_never_read_as_the_value_of_the_flag_before_it() {
    // FR-CLI-017 makes `--` end the flags, so it is never the value of the flag
    // it follows, and the token after it is a positional argument and never a
    // command. A walk that stepped over it resolved that positional as a node,
    // and the `cause` line then named a node the invocation never reached —
    // `tpl -d -- version` reported the token as supplied to `tpl version`.
    for (arguments, expected) in [
        (vec!["-d", "--", "version"], "supplied to 'tpl'"),
        (
            vec!["--database", "--", "schema", "tables"],
            "supplied to 'tpl'",
        ),
        (
            vec!["--timeout", "--", "cfg", "database"],
            "supplied to 'tpl'",
        ),
        // The control: with a value of its own the flag does carry the token
        // after it, and the walk is unchanged.
        (vec!["-d", "shop", "--", "version"], "supplied to 'tpl'"),
        (vec!["version", "--", "x"], "supplied to 'tpl version'"),
    ] {
        let printed = run(&arguments);
        let spelled = format!("tpl {}", arguments.join(" "));

        assert_refused(&printed, &spelled);
        assert!(
            stderr(&printed).contains(expected),
            "{spelled} named the wrong node: {:?}",
            stderr(&printed)
        );
    }
}

#[test]
fn fr_cli_019_a_token_written_on_both_sides_of_the_terminator_is_refused_where_it_stands_first() {
    // The parser reads left to right and refuses the first token it cannot
    // accept, so a token written on both sides of `--` was refused at the
    // occurrence before it, where FR-CLI-019 makes it an unknown flag.
    for token in ["-x", "--nope"] {
        let arguments = ["schema", "tables", token, "--", token];
        let printed = run(&arguments);
        let spelled = format!("tpl {}", arguments.join(" "));

        assert_refused(&printed, &spelled);
        assert!(
            stderr(&printed).starts_with(&format!("error: unknown flag '{token}'")),
            "{spelled}: {:?}",
            stderr(&printed)
        );
    }

    // The control: written only after the terminator, the same token is the
    // positional argument FR-CLI-017 makes of it.
    for token in ["-x", "--nope"] {
        let arguments = ["schema", "tables", "--", token];
        let printed = run(&arguments);
        let spelled = format!("tpl {}", arguments.join(" "));

        assert_refused(&printed, &spelled);
        assert!(
            stderr(&printed).starts_with(&format!("error: unexpected argument '{token}'")),
            "{spelled}: {:?}",
            stderr(&printed)
        );
    }
}

// ----------------------------------------- FR-CACHE-017 and FR-CACHE-020 ---

/// The ten commands `FR-CACHE-017` gives `--direct` and `--no-cache`, in the
/// order the command tree of `FR-HELP-019` lists them.
///
/// It is written out because it **is** the requirement: a surface derived from
/// the tree and then compared against the tree would agree with itself whatever
/// the tree said.
const CACHING: [&[&str]; 10] = [
    &["schema", "info"],
    &["schema", "tables"],
    &["schema", "table"],
    &["schema", "views"],
    &["schema", "view"],
    &["schema", "routines"],
    &["schema", "routine"],
    &["schema", "dump"],
    &["render"],
    &["cache", "load"],
];

/// The two flags of `FR-CACHE-013` and `FR-CACHE-014`.
const CACHE_FLAGS: [&str; 2] = ["--direct", "--no-cache"];

/// The placeholder a required positional argument is given, so that the node
/// under test is reached rather than refused for an argument it is missing.
const PLACEHOLDER: &str = "x";

/// The JSON command tree of `FR-HELP-016`, parsed.
///
/// It is the binary's own account of the surface it parses with, introspected
/// from the parser tree rather than maintained beside it, per `FR-HELP-021`.
fn command_tree() -> serde_json::Value {
    let printed = run(&["help", "--format", "json"]);

    assert_eq!(
        printed.status.code(),
        Some(0),
        "tpl help --format json exited {:?}: {}",
        printed.status.code(),
        stderr(&printed)
    );

    serde_json::from_slice(&printed.stdout).expect("the command tree is a JSON document")
}

/// Every node of `tree` below `tpl`, as its path and the long flags it
/// declares.
fn nodes(tree: &serde_json::Value) -> Vec<(Vec<String>, Vec<String>)> {
    tree["data"]["commands"]
        .as_array()
        .expect("data.commands is an array")
        .iter()
        .map(|entry| {
            let path = entry["path"]
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
            let options = entry["options"]
                .as_array()
                .expect("every entry carries its options")
                .iter()
                .map(|flag| flag["long"].as_str().expect("a flag has a name").to_owned())
                .collect();

            (path, options)
        })
        .collect()
}

/// The invocation that reaches the node at `path`: its segments, followed by a
/// placeholder for each argument the node requires.
fn reaching(path: &[String], tree: &serde_json::Value) -> Vec<String> {
    let entry = tree["data"]["commands"]
        .as_array()
        .expect("data.commands is an array")
        .iter()
        .find(|entry| {
            entry["path"]
                .as_array()
                .expect("every entry carries a path")
                .iter()
                .map(|segment| segment.as_str().expect("a path segment is a string"))
                .eq(path.iter().map(String::as_str))
        })
        .unwrap_or_else(|| panic!("{path:?} names a node of the tree"));

    let required = entry["arguments"]
        .as_array()
        .expect("every entry carries its arguments")
        .iter()
        .filter(|argument| argument["required"] == true)
        .count();
    let mut reaching = path.to_vec();

    reaching.extend(std::iter::repeat_n(PLACEHOLDER.to_owned(), required));

    reaching
}

#[test]
fn fr_cache_017_the_two_cache_flags_are_declared_by_exactly_ten_commands_and_are_unknown_elsewhere()
{
    // FR-CACHE-017 with FR-CLI-019: `--direct` and `--no-cache` are declared by
    // the eight `schema` subcommands, `tpl render` and `tpl cache load`, and by
    // nothing else — and every other command refuses them as unknown flags,
    // which is step 1 of FR-ERR-006 and is therefore decided in a directory
    // with no project at all.
    //
    // The surface is read twice over, and the two readings are independent: the
    // command tree of FR-HELP-016 says which nodes **declare** the flags, and
    // the process says which nodes **accept** them. A tree that disagreed with
    // the parser would be caught between the two.
    let tree = command_tree();
    let nodes = nodes(&tree);

    let declaring: Vec<Vec<String>> = nodes
        .iter()
        .filter(|(_, options)| {
            CACHE_FLAGS
                .iter()
                .any(|flag| options.iter().any(|option| option == flag))
        })
        .map(|(path, _)| path.clone())
        .collect();
    let expected: Vec<Vec<String>> = CACHING
        .iter()
        .map(|path| path.iter().map(|&segment| segment.to_owned()).collect())
        .collect();

    assert_eq!(
        declaring, expected,
        "FR-CACHE-017 fixes the ten commands that declare either flag"
    );

    // Each of the ten declares **both**: the two are orthogonal, per
    // FR-CACHE-015, so a command declaring one of them and not the other would
    // satisfy the count above and not the requirement.
    for (path, options) in &nodes {
        if !expected.contains(path) {
            continue;
        }

        for flag in CACHE_FLAGS {
            assert!(
                options.iter().any(|option| option == flag),
                "tpl {} declares no {flag}",
                path.join(" ")
            );
        }
    }

    // And every other node refuses each of them where FR-CLI-019 puts it.
    for (path, _) in &nodes {
        if expected.contains(path) {
            continue;
        }

        for flag in CACHE_FLAGS {
            let mut arguments = reaching(path, &tree);
            arguments.push(flag.to_owned());

            let spelled = format!("tpl {}", arguments.join(" "));
            let borrowed: Vec<&str> = arguments.iter().map(String::as_str).collect();
            let printed = run(&borrowed);

            assert_refused(&printed, &spelled);
            assert!(
                stderr(&printed).starts_with(&format!("error: unknown flag '{flag}'")),
                "{spelled}: {:?}",
                stderr(&printed)
            );
        }
    }

    // The control, and it is what makes the refusals above mean something: the
    // ten are not refused as unknown flags. Each is carried past the parser and
    // fails on what an empty directory gives it — no project, per FR-PROJ-005 —
    // or, for `tpl cache load --no-cache`, on the command's own contradiction,
    // per FR-CACHE-019.
    for path in &expected {
        for flag in CACHE_FLAGS {
            let mut arguments = reaching(path, &tree);
            arguments.push(flag.to_owned());

            let spelled = format!("tpl {}", arguments.join(" "));
            let borrowed: Vec<&str> = arguments.iter().map(String::as_str).collect();
            let written = stderr(&run(&borrowed));

            assert!(
                !written.starts_with(&format!("error: unknown flag '{flag}'")),
                "{spelled} was refused as an unknown flag: {written:?}"
            );
        }
    }
}

#[test]
fn fr_cache_020_clean_and_status_declare_neither_cache_flag() {
    // FR-CACHE-020 names the two commands of the `cache` arm that declare
    // neither flag, and makes supplying either the unknown-flag 64 of
    // FR-CLI-019. It is the case of FR-CACHE-017 worth stating on its own: the
    // two stand beside `cache load`, which declares both, so the refusal is a
    // property of the node rather than of the arm.
    let tree = command_tree();
    let nodes = nodes(&tree);
    let options_of = |wanted: &[&str]| -> Vec<String> {
        nodes
            .iter()
            .find(|(path, _)| path.iter().map(String::as_str).eq(wanted.iter().copied()))
            .map(|(_, options)| options.clone())
            .unwrap_or_else(|| panic!("{wanted:?} names a node of the tree"))
    };

    for node in [["cache", "clean"], ["cache", "status"]] {
        let options = options_of(&node);

        for flag in CACHE_FLAGS {
            assert!(
                !options.iter().any(|option| option == flag),
                "tpl {} declares {flag}",
                node.join(" ")
            );

            let arguments = [node[0], node[1], flag];
            let spelled = format!("tpl {}", arguments.join(" "));
            let printed = run(&arguments);

            assert_refused(&printed, &spelled);
            assert!(
                stderr(&printed).starts_with(&format!("error: unknown flag '{flag}'")),
                "{spelled}: {:?}",
                stderr(&printed)
            );
        }
    }

    // The control: their sibling declares both, so the two refusals above are
    // the requirement and not an arm that parses no flags at all.
    let sibling = options_of(&["cache", "load"]);
    for flag in CACHE_FLAGS {
        assert!(
            sibling.iter().any(|option| option == flag),
            "tpl cache load declares no {flag}"
        );
    }
}

// ---------------------------------------------------------------------------
// A repeated valueless flag
// ---------------------------------------------------------------------------

#[test]
fn fr_cli_025_a_repeated_valueless_flag_is_accepted_by_the_process() {
    // FR-CLI-025 makes a repeated flag that carries no value idempotent, and
    // the fact a caller can observe is the exit code. The two invocations the
    // requirement writes out are driven here, beside the other four flags it
    // names; each ends the invocation or reaches a command that needs no
    // project, so `0` is the whole of what is being read.
    for arguments in [
        &["-q", "-q", "version"][..],
        &["--quiet", "--quiet", "version"][..],
        &["-v", "-v", "-v", "-v", "version"][..],
        &["-h", "-h"][..],
        &["--help", "--help"][..],
        &["-V", "-V"][..],
        &["--version", "--version"][..],
        &["help", "--pretty", "--pretty", "--format", "json"][..],
    ] {
        let printed = run(arguments);

        assert_eq!(
            printed.status.code(),
            Some(0),
            "tpl {} did not exit 0: {}",
            arguments.join(" "),
            stderr(&printed)
        );
    }

    // The effect is the effect of one occurrence: `-q` lowers the diagnostic
    // level and the second one changes nothing, which is what the requirement
    // means by the flag having the effect of one occurrence.
    assert_eq!(
        run(&["-q", "version"]).stdout,
        run(&["-q", "-q", "version"]).stdout
    );
}

#[test]
fn fr_cli_014_a_repeated_value_carrying_flag_is_still_refused_by_the_process() {
    // The control for the test above, and the half of the pair FR-CLI-025
    // leaves untouched: a flag that carries a single value is `64` with both
    // values named, because the ground FR-CLI-014 gives is about two values
    // disagreeing.
    for (arguments, first, second) in [
        (&["-d", "a", "-d", "b", "version"][..], "a", "b"),
        (
            &["--timeout", "1", "--timeout", "2", "version"][..],
            "1",
            "2",
        ),
    ] {
        let printed = run(arguments);
        let written = stderr(&printed);

        assert_eq!(
            printed.status.code(),
            Some(USAGE),
            "tpl {} did not exit {USAGE}: {written}",
            arguments.join(" ")
        );
        assert!(written.contains(first), "{written}");
        assert!(written.contains(second), "{written}");
    }
}

/// The `hint` line of a refusal, without its label.
fn hint_of(printed: &Output) -> String {
    stderr(printed)
        .lines()
        .find_map(|line| line.strip_prefix("hint:  "))
        .expect("a refusal carries a hint")
        .to_owned()
}

#[test]
fn fr_err_042_a_prefix_of_a_command_is_suggested_and_never_run() {
    // FR-ERR-042 at the three places a command token is read: the first
    // token, a token after a group node, and a segment of the help path.
    for (arguments, suggested) in [
        (&["sch", "tables"][..], "did you mean 'schema'?"),
        (&["cfg", "datab"][..], "did you mean 'database'?"),
        (&["help", "sch"][..], "did you mean 'schema'?"),
    ] {
        let printed = run(arguments);
        let spelled = arguments.join(" ");

        assert_refused(&printed, &spelled);
        assert!(
            hint_of(&printed).starts_with(suggested),
            "tpl {spelled}: {}",
            stderr(&printed)
        );
    }

    // FR-ERR-038: the comparison is over the characters as written.
    let printed = run(&["SCH"]);
    assert_refused(&printed, "SCH");
    assert!(
        !hint_of(&printed).contains("'schema'"),
        "{}",
        stderr(&printed)
    );
}

#[test]
fn s_14_a_flag_with_a_fixed_set_of_values_names_them_when_it_is_given_none() {
    let printed = run(&["help", "--format"]);

    assert_refused(&printed, "help --format");
    assert_eq!(
        hint_of(&printed),
        "give text or json after the flag, e.g. --format text"
    );
}
