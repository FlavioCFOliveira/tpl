//! The second arm, exercised as a process.
//!
//! `tpl template` reads the templates under `.tpl/templates/` and nothing else.
//! Every rule the four subcommands carry is a rule about a **filesystem**, a
//! **stream** and an **exit code**, and the first and the last are not
//! observable from inside the crate:
//!
//! | Property | Requirement |
//! |---|---|
//! | All four answer, in both formats where both are declared | `FR-TMPL-002`, `FR-GLOB-021` |
//! | The listing's order, and the names it prints | `FR-TMPL-011`, `FR-TMPL-013`, `FR-TMPL-014` |
//! | A listed name is usable verbatim by the other three | `FR-TMPL-012` |
//! | The source reaches stdout unaltered | `FR-TMPL-015`, `FR-OUT-019` |
//! | `check` is syntax analysis and evaluates nothing | `FR-TMPL-017`, `BR-TMPL-001` |
//! | A syntax error is `65`, naming the template, the line and the column | `FR-TMPL-020` |
//! | A symbolic link and a name that leaves the root are refused | `FR-TMPL-024`, `FR-TMPL-026` |
//! | A name that reaches nothing is `66` with a nearest-match suggestion | `FR-TMPL-027` |
//! | An empty project lists nothing, checks nothing, and succeeds | `FR-TMPL-031` |
//! | The `json` documents, and their `source` | `FR-TMPL-028` … `FR-TMPL-030` |
//! | No subcommand opens a database connection | `FR-TMPL-003` |
//!
//! **Every run is made in a temporary directory of its own**, with no `.tpl`
//! above it, under a cleared environment — on the same terms as
//! [`project_and_configuration`](../project_and_configuration/index.html). The
//! directory is removed when the test's sandbox goes out of scope, and nothing
//! outside it is read or written.
//!
//! # No fixture is needed, and the connection clause says why
//!
//! `FR-TMPL-003` is a statement about what these commands do **not** do, and an
//! absence is worth nothing until the same instrument is shown to observe the
//! presence. The instrument here is the exit code over a database entry
//! repointed at a port nothing answers on: the control run — `tpl schema
//! tables`, which does read a catalogue — answers `69`, and the four answer
//! `0` against the same entry. That is weaker than the server-side connection
//! record `tests/outside_the_process.rs` uses for `NFR-PERF-005` and
//! `NFR-PERF-006`, and it is stronger than nothing and needs no fixture: a
//! command that opened the connection could not have exited `0`.

#[path = "support/sandbox.rs"]
mod sandbox;

use std::process::Output;

use sandbox::Sandbox;

/// A `.tpl/.cfg` with no database entry at all, which is what `FR-PROJ-018`
/// leaves a fresh project with and what every body below works against unless
/// it is about the connection clause.
const NO_ENTRY: &str = "[core]\n";

/// A `.tpl/.cfg` whose selected entry describes a read the server would refuse
/// at once: nothing listens on port 1.
const UNREACHABLE: &str = "[core]\ndatabase = \"nowhere\"\n\n[database.nowhere]\n\
                           host = \"127.0.0.1\"\nport = 1\nuser = \"reader\"\n\
                           database = \"freight\"\n";

/// The four labels of `FR-ERR-008`, in the order it fixes.
const LABELS: [&str; 4] = ["error: ", "cause: ", "hint:  ", "exit:  "];

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

/// Runs `tpl` and refuses anything but exit `0`, returning stdout.
fn succeeds(sandbox: &Sandbox, arguments: &[&str]) -> String {
    let printed = sandbox.run(arguments);

    assert_eq!(
        code(&printed),
        0,
        "tpl {} did not exit 0: {}",
        arguments.join(" "),
        stderr(&printed)
    );

    stdout(&printed)
}

/// The document `arguments` produced, parsed.
fn document(sandbox: &Sandbox, arguments: &[&str]) -> serde_json::Value {
    let out = succeeds(sandbox, arguments);

    serde_json::from_str(&out).unwrap_or_else(|failure| {
        panic!("tpl {} did not emit JSON: {failure}", arguments.join(" "))
    })
}

/// Asserts that `arguments` is a refusal carrying `expected`: that code, an
/// empty stdout per `FR-ERR-033`, and the four labelled lines of `FR-ERR-008`
/// in their order. Returns what reached stderr.
fn refused(sandbox: &Sandbox, arguments: &[&str], expected: i32) -> String {
    let printed = sandbox.run(arguments);
    let written = stderr(&printed);
    let spelled = arguments.join(" ");

    assert_eq!(
        code(&printed),
        expected,
        "tpl {spelled} did not exit {expected}: {written}"
    );
    assert!(
        printed.stdout.is_empty(),
        "tpl {spelled} wrote {} bytes to stdout",
        printed.stdout.len()
    );

    let lines: Vec<&str> = written.lines().collect();

    assert_eq!(lines.len(), 4, "tpl {spelled} wrote {written:?}");
    for (line, label) in lines.iter().zip(LABELS) {
        assert!(line.starts_with(label), "tpl {spelled} wrote {line:?}");
    }

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

/// The names the `templates` array of `FR-TMPL-028` carries, in order.
fn names(document: &serde_json::Value) -> Vec<String> {
    document["data"]["templates"]
        .as_array()
        .expect("FR-OUT-035 makes the value an array")
        .iter()
        .map(|member| {
            member["name"]
                .as_str()
                .expect("FR-TMPL-028 makes each member an object carrying `name`")
                .to_owned()
        })
        .collect()
}

/// A project carrying `templates`, each written at the relative path given.
fn project(sandbox: &Sandbox, configuration: &str, templates: &[(&str, &str)]) {
    sandbox.project(configuration);
    sandbox.directory(".tpl/templates");

    for (relative, source) in templates {
        sandbox.write(&format!(".tpl/templates/{relative}"), source);
    }
}

// ------------------------------------------------------------ FR-TMPL-002 ---

#[test]
fn fr_tmpl_002_all_four_subcommands_answer_in_both_formats_where_both_are_declared() {
    // FR-TMPL-002 with FR-GLOB-021: four subcommands, of which `list` and
    // `path` declare `--format` and `--pretty` and the other two declare
    // neither, because neither has a second representation to choose between.
    // FR-TMPL-030 fixes `source` on every one of the documents.
    let sandbox = Sandbox::new();
    project(
        &sandbox,
        NO_ENTRY,
        &[
            ("example.jinja", "hello\n"),
            ("rust/struct.jinja", "struct\n"),
        ],
    );

    // `list`, in both representations and in both forms of the JSON one.
    assert_eq!(succeeds(&sandbox, &["template", "list"]).lines().count(), 3);

    let compact = document(&sandbox, &["template", "list", "--format", "json"]);
    assert_eq!(compact["source"], "project");
    assert_eq!(compact["schema_version"], 1);
    assert_eq!(names(&compact), ["example", "rust/struct"]);

    let indented = succeeds(
        &sandbox,
        &["template", "list", "--format", "json", "--pretty"],
    );
    assert!(indented.lines().count() > 1, "{indented:?}");
    assert_eq!(
        serde_json::from_str::<serde_json::Value>(&indented).expect("it is a document"),
        compact,
        "FR-OUT-008 changes the whitespace and nothing else"
    );

    // `show`, which has one representation and adds nothing to it.
    assert_eq!(
        succeeds(&sandbox, &["template", "show", "example"]),
        "hello\n"
    );

    // `check`, whose answer is the exit code and whose stdout stays empty,
    // per BR-CLI-004.
    assert_eq!(succeeds(&sandbox, &["template", "check"]), "");

    // `path`, in both representations.
    let root = succeeds(&sandbox, &["template", "path"]);
    assert_eq!(
        root.trim_end(),
        sandbox.path(".tpl/templates").to_str().unwrap()
    );

    let located = document(
        &sandbox,
        &["template", "path", "rust/struct", "--format", "json"],
    );
    assert_eq!(located["source"], "project");
    assert_eq!(
        located["data"]["path"],
        serde_json::Value::from(
            sandbox
                .path(".tpl/templates/rust/struct.jinja")
                .to_str()
                .expect("the sandbox path is UTF-8")
        )
    );
}

// ------------------------------------------------- FR-TMPL-011 … FR-TMPL-014 ---

#[test]
fn fr_tmpl_013_the_listing_is_ordered_by_the_displayed_name_in_both_representations() {
    // FR-TMPL-011 removes the extension; FR-TMPL-013 orders by the displayed
    // name, byte by byte, and the worked case of the requirement's fifth
    // edition is `rust/_types` before `rust/struct`. FR-TMPL-014 puts the
    // partial in the listing without a flag, and its own accepted cost — a
    // `.md.jinja` listing as a Markdown name — is here too.
    let sandbox = Sandbox::new();
    project(
        &sandbox,
        NO_ENTRY,
        &[
            ("rust/struct.jinja", "struct\n"),
            ("example.jinja", "hello\n"),
            ("rust/_types.jinja", "types\n"),
            ("docs/table.md.jinja", "table\n"),
        ],
    );

    let expected = ["docs/table.md", "example", "rust/_types", "rust/struct"];

    // The `text` representation: the header row of FR-OUT-006, then the names.
    let listed = succeeds(&sandbox, &["template", "list"]);
    let rows: Vec<&str> = listed.lines().collect();

    assert_eq!(rows[0], "NAME");
    assert_eq!(rows[1..], expected);

    // And the `json` one, which FR-TMPL-028's composition note holds to the
    // same order.
    assert_eq!(
        names(&document(
            &sandbox,
            &["template", "list", "--format", "json"]
        )),
        expected
    );
}

#[test]
fn fr_tmpl_005_a_file_that_is_not_a_template_is_neither_listed_nor_reachable() {
    // FR-TMPL-004 and FR-TMPL-005, through the command line: a LICENSE beside
    // the templates is not listed, and naming it is the `66` of a template
    // that does not exist rather than a dump of its contents.
    let sandbox = Sandbox::new();
    project(&sandbox, NO_ENTRY, &[("example.jinja", "hello\n")]);
    sandbox.write(".tpl/templates/LICENSE", "not a template\n");
    sandbox.write(".tpl/templates/notes.md", "not a template\n");

    assert_eq!(
        names(&document(
            &sandbox,
            &["template", "list", "--format", "json"]
        )),
        ["example"]
    );

    for named in ["LICENSE", "notes.md"] {
        refused(&sandbox, &["template", "show", named], 66);
    }
}

#[test]
fn fr_tmpl_012_every_listed_name_is_usable_verbatim_by_the_other_three() {
    // FR-TMPL-012: a listed name is the positional argument of `show`, `check`
    // and `path`, at any depth and with the extension omitted, and FR-TMPL-007
    // makes the spelt-out form name the same template.
    let sandbox = Sandbox::new();
    project(
        &sandbox,
        NO_ENTRY,
        &[
            ("example.jinja", "hello\n"),
            ("rust/_types.jinja", "types\n"),
        ],
    );

    for listed in names(&document(
        &sandbox,
        &["template", "list", "--format", "json"],
    )) {
        let spelt = format!("{listed}.jinja");

        succeeds(&sandbox, &["template", "show", &listed]);
        succeeds(&sandbox, &["template", "check", &listed]);

        assert_eq!(
            succeeds(&sandbox, &["template", "path", &listed]),
            succeeds(&sandbox, &["template", "path", &spelt]),
            "FR-TMPL-007: {listed} and {spelt} name one template"
        );
        assert_eq!(
            succeeds(&sandbox, &["template", "show", &listed]),
            succeeds(&sandbox, &["template", "show", &spelt])
        );
    }
}

// ------------------------------------------------------------ FR-TMPL-031 ---

#[test]
fn fr_tmpl_031_a_project_with_no_template_lists_nothing_and_checks_nothing() {
    // FR-TMPL-031 through FR-OUT-033, FR-OUT-034, FR-OUT-035 and FR-OUT-037:
    // exit 0 in every case, the header row alone in `text`, an empty array in
    // `json`, and nothing at all from `check`. Both states are covered — a
    // template directory that is empty, and one that is not there at all.
    for arrange in [0_u8, 1] {
        let sandbox = Sandbox::new();
        sandbox.project(NO_ENTRY);
        if arrange == 0 {
            sandbox.directory(".tpl/templates");
        }

        assert_eq!(succeeds(&sandbox, &["template", "list"]), "NAME\n");
        assert_eq!(
            succeeds(&sandbox, &["template", "list", "--format", "json"]),
            "{\"schema_version\":1,\"source\":\"project\",\"data\":{\"templates\":[]}}\n"
        );
        assert_eq!(succeeds(&sandbox, &["template", "check"]), "");

        // The root is still reported, because FR-TMPL-021 asks where a
        // template would be read from rather than what is there.
        assert_eq!(
            succeeds(&sandbox, &["template", "path"]).trim_end(),
            sandbox.path(".tpl/templates").to_str().unwrap()
        );
    }
}

// ------------------------------------------------- FR-TMPL-015, FR-OUT-019 ---

#[test]
fn fr_tmpl_015_show_writes_the_source_byte_for_byte_and_adds_nothing_to_it() {
    // FR-TMPL-015 with FR-OUT-019: unaltered, and explicitly not escaped. The
    // source carries a tab, a form feed and an escape sequence — the three the
    // `text` layout would rewrite — and a body that ends without a newline,
    // which nothing here supplies for it.
    let sandbox = Sandbox::new();
    let source = "{% raw %}\tone\x0ctwo\x1b[2mthree{% endraw %}\nno final newline";
    project(&sandbox, NO_ENTRY, &[("odd.jinja", source)]);

    let printed = sandbox.run(&["template", "show", "odd"]);

    assert_eq!(code(&printed), 0, "{}", stderr(&printed));
    assert_eq!(printed.stdout, source.as_bytes());
}

// ------------------------------------------------- FR-TMPL-017 … FR-TMPL-020 ---

#[test]
fn fr_tmpl_020_a_syntax_error_is_sixty_five_naming_the_template_the_line_and_the_column() {
    // FR-TMPL-020 with FR-ERR-011: the template, the line and the column, in
    // the four labelled lines of FR-ERR-008. The template is reached both by
    // FR-TMPL-018's whole-project form and by FR-TMPL-019's named one, and the
    // same condition answers each.
    let sandbox = Sandbox::new();
    project(
        &sandbox,
        NO_ENTRY,
        &[("good.jinja", "fine\n"), ("broken.jinja", "ok\n{% if %}\n")],
    );

    for arguments in [
        &["template", "check"][..],
        &["template", "check", "broken"][..],
        &["template", "check", "good", "broken"][..],
    ] {
        let written = refused(&sandbox, arguments, 65);

        assert!(written.contains("broken.jinja"), "{written}");
        assert!(written.contains("line 2"), "{written}");
        assert!(line(&written, "cause:").contains("column"), "{written}");
    }

    // The template that does parse is not the one reported, and on its own it
    // is a success.
    assert_eq!(succeeds(&sandbox, &["template", "check", "good"]), "");
}

#[test]
fn br_tmpl_001_check_evaluates_nothing_so_a_template_that_would_fail_still_passes() {
    // FR-TMPL-017 and BR-TMPL-001: syntax analysis alone — no expression is
    // evaluated, no function or filter is called. Each of these three fails a
    // render and none of them fails a check: `fail` ends a render with 65 per
    // FR-SEM-014, an undefined variable is the strict behaviour of FR-SEM-012,
    // and an `{% include %}` that does not resolve literally is FR-TMPL-009.
    // A `check` that evaluated anything would have to refuse one of them.
    let sandbox = Sandbox::new();
    project(
        &sandbox,
        NO_ENTRY,
        &[
            ("stops.jinja", "{{ fail('DECIMAL has no mapping') }}\n"),
            ("undefined.jinja", "{{ nothing.at.all }}\n"),
            ("includes.jinja", "{% include \"_absent\" %}\n"),
        ],
    );

    assert_eq!(succeeds(&sandbox, &["template", "check"]), "");

    for named in ["stops", "undefined", "includes"] {
        assert_eq!(succeeds(&sandbox, &["template", "check", named]), "");
    }
}

// ------------------------------------------------- FR-TMPL-024, FR-TMPL-026 ---

#[test]
fn fr_tmpl_024_a_symbolic_link_is_refused_and_never_listed() {
    // FR-TMPL-024 with the exploit FR-TMPL-026's rationale names:
    // `ln -s ../.cfg .tpl/templates/leak.jinja` must not turn
    // `tpl template show` into a credential dump. The link is not listed, and
    // naming it is the 65 of an escape — and the credential the file holds
    // reaches neither stream.
    let sandbox = Sandbox::new();
    project(&sandbox, NO_ENTRY, &[("example.jinja", "hello\n")]);
    sandbox.write(
        ".tpl/.cfg",
        "[core]\n\n[database.shop]\nhost = \"db\"\nuser = \"reader\"\n\
         database = \"shop\"\npassword = \"s3cr3t\"\n",
    );

    std::os::unix::fs::symlink(
        sandbox.path(".tpl/.cfg"),
        sandbox.path(".tpl/templates/leak.jinja"),
    )
    .expect("the sandbox is writable");
    std::os::unix::fs::symlink(
        sandbox.path(".tpl/templates/example.jinja"),
        sandbox.path(".tpl/templates/alias.jinja"),
    )
    .expect("the sandbox is writable");

    assert_eq!(
        names(&document(
            &sandbox,
            &["template", "list", "--format", "json"]
        )),
        ["example"]
    );

    for named in ["leak", "alias"] {
        for verb in ["show", "check", "path"] {
            let written = refused(&sandbox, &["template", verb, named], 65);

            assert!(
                !written.contains("s3cr3t"),
                "the refusal of {verb} {named} carried the file it refused to read: {written}"
            );
        }
    }
}

#[test]
fn fr_tmpl_026_a_name_that_leaves_the_root_is_refused_through_the_command_line() {
    // FR-TMPL-026: a resolved path outside the template root is 65, whichever
    // way the name reaches outside — by climbing with `..`, or by being
    // absolute. The file exists in both cases, so what is refused is the
    // escape and not the absence.
    let sandbox = Sandbox::new();
    project(&sandbox, NO_ENTRY, &[("example.jinja", "hello\n")]);
    let outside = sandbox.write("outside.jinja", "leaked\n");
    sandbox.write(".tpl/outside.jinja", "leaked\n");

    let climbing = "../outside";
    let absolute = outside
        .to_str()
        .expect("the sandbox path is UTF-8")
        .to_owned();
    let spelt = absolute.trim_end_matches(".jinja").to_owned();

    for named in [climbing, &spelt] {
        for verb in ["show", "check", "path"] {
            let written = refused(&sandbox, &["template", verb, named], 65);

            assert!(!written.contains("leaked"), "{written}");
        }
    }
}

// ------------------------------------------------------------ FR-TMPL-027 ---

#[test]
fn fr_tmpl_027_a_name_that_does_not_exist_is_sixty_six_with_a_nearest_match() {
    // FR-TMPL-027 with FR-ERR-019 and FR-ERR-037: 66, and a `did you mean`
    // question over the template names that do exist, inside the one hint line
    // FR-ERR-008 fixes. The population is the displayed names, which
    // FR-TMPL-012 makes usable verbatim — so the suggestion is a name the
    // caller can run.
    let sandbox = Sandbox::new();
    project(
        &sandbox,
        NO_ENTRY,
        &[
            ("example.jinja", "hello\n"),
            ("exemple.jinja", "bonjour\n"),
            ("unrelated.jinja", "other\n"),
        ],
    );

    for verb in ["show", "check", "path"] {
        let written = refused(&sandbox, &["template", verb, "exmple"], 66);
        let hint = line(&written, "hint:");

        assert_eq!(
            hint,
            "did you mean 'example' or 'exemple'? list the project's templates with: \
             tpl template list",
            "tpl template {verb}"
        );
    }

    // The suggested name is one the caller can run, which is the whole point
    // of drawing the population from the displayed names.
    assert_eq!(
        succeeds(&sandbox, &["template", "show", "example"]),
        "hello\n"
    );
}

#[test]
fn fr_err_020_a_name_nothing_is_near_is_refused_with_the_generic_hint_alone() {
    // FR-ERR-020: the suggestion is omitted rather than weakened, and the
    // runnable command FR-ERR-009 obliges stands on its own.
    let sandbox = Sandbox::new();
    project(&sandbox, NO_ENTRY, &[("example.jinja", "hello\n")]);

    let written = refused(&sandbox, &["template", "show", "wholly_unlike_it"], 66);

    assert_eq!(
        line(&written, "hint:"),
        "list the project's templates with: tpl template list"
    );
}

// ------------------------------------------------------------ FR-TMPL-003 ---

#[test]
fn fr_tmpl_003_no_subcommand_of_the_second_arm_opens_a_database_connection() {
    // FR-TMPL-003: no `template` subcommand opens a connection, reads the
    // cache, or requires a database entry to be selected. The project below
    // selects an entry that describes a read no server would answer, so a
    // subcommand that resolved and connected could not exit 0.
    //
    // The control comes first, and it is what makes the four assertions below
    // mean anything: `tpl schema tables` against the same entry does open a
    // connection, and answers 69.
    let sandbox = Sandbox::new();
    project(
        &sandbox,
        UNREACHABLE,
        &[
            ("example.jinja", "hello\n"),
            ("rust/_types.jinja", "types\n"),
        ],
    );

    let control = sandbox.run(&["schema", "tables"]);
    assert_eq!(
        code(&control),
        69,
        "the control did not reach the server at all: {}",
        stderr(&control)
    );

    for arguments in [
        &["template", "list"][..],
        &["template", "show", "example"][..],
        &["template", "check"][..],
        &["template", "path", "rust/_types"][..],
    ] {
        succeeds(&sandbox, arguments);
    }

    // And nothing was written on the way: BR-TMPL-002 makes the arm read-only
    // with respect to the filesystem as well as the database, so the cache
    // FR-TMPL-003 also names was never created.
    assert!(
        !sandbox.path(".tpl/.cache").exists(),
        "a template subcommand created the cache store"
    );
}

// ------------------------------------------------- FR-TMPL-021, FR-TMPL-022 ---

#[test]
fn fr_tmpl_021_path_answers_with_the_root_and_with_where_one_name_resolves() {
    // FR-TMPL-021 and FR-TMPL-022, with the worked example of the second: the
    // root with no argument, and the file's own path with one. Both are
    // absolute, and the `text` form is the path and one newline — nothing else,
    // because the requirement gives it no header row to sit under.
    let sandbox = Sandbox::new();
    project(&sandbox, NO_ENTRY, &[("rust/struct.jinja", "struct\n")]);

    let root = sandbox.path(".tpl/templates");
    let file = sandbox.path(".tpl/templates/rust/struct.jinja");

    assert_eq!(
        succeeds(&sandbox, &["template", "path"]),
        format!("{}\n", root.display())
    );
    assert_eq!(
        succeeds(&sandbox, &["template", "path", "rust/struct"]),
        format!("{}\n", file.display())
    );
    assert!(root.is_absolute() && file.is_absolute());

    // FR-TMPL-029: the `json` form carries the same bytes under `path`.
    assert_eq!(
        document(&sandbox, &["template", "path", "--format", "json"])["data"]["path"],
        serde_json::Value::from(root.to_str().expect("the sandbox path is UTF-8"))
    );
}

#[test]
fn fr_err_041_a_nested_template_name_is_suggested_like_any_other() {
    // Finding H-07: the set of FR-ERR-022 refused the `/` every nested name
    // carries, so `rust/_type` was offered nothing although `rust/_types` is
    // one edit away. FR-ERR-041 governs a template name.
    let sandbox = Sandbox::new();
    project(&sandbox, NO_ENTRY, &[("rust/_types.jinja", "{{ 1 }}\n")]);

    let written = refused(&sandbox, &["template", "show", "rust/_type"], 66);

    assert_eq!(
        line(&written, LABELS[2]),
        "did you mean 'rust/_types'? list the project's templates with: tpl template list"
    );
}

#[test]
fn fr_err_021_a_syntax_error_names_the_template_to_check_again() {
    let sandbox = Sandbox::new();
    project(&sandbox, NO_ENTRY, &[("t/syntax.jinja", "{% if %}\n")]);

    let written = refused(&sandbox, &["template", "check"], 65);

    assert_eq!(
        line(&written, LABELS[2]),
        "correct line 1 of template 't/syntax.jinja', then check it with: tpl template check \
         t/syntax.jinja"
    );
}

// ------------------------------------------------------------ FR-TMPL-032 ---

/// The `error:` lines of a diagnostic stream, without their label.
fn error_lines(written: &str) -> Vec<String> {
    written
        .lines()
        .filter_map(|line| line.strip_prefix("error: "))
        .map(str::to_owned)
        .collect()
}

#[test]
fn fr_tmpl_032_check_reports_every_failing_template_one_message_each_in_checking_order() {
    // FR-TMPL-032, S-05 of #274: every selected template is checked before
    // anything is reported, and each failure is one four-line message.
    let sandbox = Sandbox::new();
    project(
        &sandbox,
        NO_ENTRY,
        &[
            ("a.jinja", "{% if %}\n"),
            ("b.jinja", "fine\n"),
            ("c.jinja", "ok\n{{ x.\n"),
        ],
    );

    let printed = sandbox.run(&["template", "check"]);
    let written = stderr(&printed);

    assert_eq!(code(&printed), 65, "{written}");
    assert!(printed.stdout.is_empty(), "point 4: stdout stays empty");

    // Point 2: no separator, so eight lines, each message in label order and
    // ending with the one exit line.
    let lines: Vec<&str> = written.lines().collect();
    assert_eq!(lines.len(), 8, "{written}");
    for (line, label) in lines.iter().zip(LABELS.iter().cycle()) {
        assert!(line.starts_with(label), "{written}");
    }
    assert_eq!(lines[3], "exit:  65 (EX_DATAERR)");
    assert_eq!(lines[7], "exit:  65 (EX_DATAERR)");

    // Point 3: the order of FR-TMPL-013 with no argument.
    let errors = error_lines(&written);
    assert!(errors[0].contains("'a.jinja'"), "{written}");
    assert!(errors[1].contains("'c.jinja'"), "{written}");

    // Point 3: with names, the order written, and a repetition checked once.
    let written = stderr(&sandbox.run(&["template", "check", "c", "b", "a", "c.jinja"]));
    let errors = error_lines(&written);
    assert_eq!(errors.len(), 2, "{written}");
    assert!(errors[0].contains("'c.jinja'"), "{written}");
    assert!(errors[1].contains("'a.jinja'"), "{written}");

    // Resolution runs first over every name: one that resolves to nothing is
    // reported alone, with its own code.
    let written = refused(&sandbox, &["template", "check", "a", "nosuch"], 66);
    assert!(line(&written, "error:").contains("nosuch"), "{written}");
}
