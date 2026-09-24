//! The third arm, exercised as a process.
//!
//! `tpl render` joins the other two: it takes a model from one of **two**
//! sources, a template from the project, and produces exactly one result on
//! stdout, per `FR-RND-002`. What this file establishes above everything else
//! is that the result **does not betray which source the model came from**, and
//! beside it the failure surface of the eight steps `FR-ERR-006` orders, as a
//! caller meets it: an exit code, the bytes of stdout, and the four labelled
//! lines of `FR-ERR-008`.
//!
//! | Requirement | The property this file establishes |
//! |---|---|
//! | `FR-RND-016`, `FR-RND-017`, `FR-RND-026` | One template and one object render the same bytes from a server, from the cache, from a document in a file, and from a document on standard input |
//! | `FR-RND-021` | The two forms of one document are one contract, which the pipeline of `FR-RND-017` is what exercises |
//! | `FR-RND-005`, `FR-RND-011` … `FR-RND-014`, `FR-RND-018`, `FR-RND-027` | Step 1 of `FR-ERR-006` refuses six invocations with `64`, before anything is discovered or opened |
//! | `FR-RND-020` | A document that is not well-formed JSON, and one that is JSON and not the contract, are each `65` |
//! | `FR-RND-029`, `FR-RND-032` | A template name and an object name that reach nothing are each `66`, with the nearest-match suggestion of `FR-ERR-019` |
//! | `FR-ERR-006`, `FR-ERR-007` | Template resolution is the fourth step, so it precedes the entry, the document and the catalogue, and the three costs that follow from the position are each what the requirement records |
//! | `FR-RND-030`, `FR-RND-031`, `FR-SEM-005`, `FR-SEM-008`, `FR-SEM-014` | Every way a render fails is `65`, naming the template, the line and the column, per `FR-SEM-019` |
//! | `FR-CTX-029`, `FR-CTX-030` | `now` is one instant per invocation, and a template that does not reference it repeats byte for byte |
//! | `FR-CTX-026`, `FR-RND-024` | `vars` is this invocation's `--set` flags and nothing else, and a document that carries `vars`, `tpl` or `now` has those values ignored |
//! | `FR-CACHE-016`, `FR-RND-025` | `--direct --no-cache` renders without touching a file of the store |
//! | `FR-CACHE-038`, `FR-CACHE-039`, `FR-RND-034` | A render from the store that reaches a damaged file is abandoned, and the invocation is one server read, one write and one result on stdout, exactly as a miss found before the render |
//!
//! # Why most of this file needs the fixture
//!
//! Every body that renders from a `--context` document needs a document, and
//! the contract a document satisfies is stated in
//! `specification/context-document.md` and implemented by `tpl schema dump`.
//! A document written out in Rust here would be a **second statement of that
//! contract**, in a file that would go on compiling after the contract moved —
//! which is the failure mode this project refuses everywhere else. So every
//! document below is one the product produced, against a fixture server, in the
//! body that uses it.
//!
//! Three bodies need no document and therefore no fixture, and they are the
//! three about what happens before one is read: the six refusals of step 1,
//! made in a directory with no project at all; the two documents that are
//! refused before anything is bound; and the fourth position of `FR-ERR-006`,
//! whose two no-server costs are each a refusal reached before a document could
//! matter.
//!
//! # What this file does not establish
//!
//! Two properties, and neither is reachable from an exit code or from the bytes
//! of a run that succeeded:
//!
//! | Property | Requirement |
//! |---|---|
//! | A render from a `--context` document opens no connection | `FR-RND-022` |
//! | A render that fails leaves at most one incomplete result on stdout | `FR-RND-034`, `FR-SEM-020` |
//!
//! Both are observed in
//! [`outside_the_process`](../outside_the_process/index.html), with the
//! instruments `NFR-PERF-007` fixes: the server's own connection record for the
//! first, and the process's own file descriptor 1 for the second. The `65` a
//! failing render exits is asserted **here**; the bytes it did not write are
//! asserted **there**.
//!
//! # The fixture, and what happens without one
//!
//! Every server-dependent body here is gated on `scripts/mariadb/status.sh`,
//! whose exit code is three-valued: `0` runs the body, `1` skips it with a
//! printed reason, and `2` — half a fixture — fails the run rather than
//! skipping over it. Those bodies are driven **once per series of
//! `FR-SRV-015`**, which `FR-SRV-029` requires of a test of cross-series
//! behaviour: a render assembled from a catalogue is one, because the model it
//! renders is what a series returned.

#[path = "support/fixture.rs"]
mod fixture;
#[path = "support/sandbox.rs"]
mod sandbox;

use std::process::Output;

use fixture::Server;
use sandbox::Sandbox;

/// The schema every fixture server carries.
const SCHEMA: &str = "freight";

/// The database entry every project below defines.
const ENTRY: &str = "fixture";

/// The privileged account of the fixture, which reads every property.
const ROOT: (&str, &str) = ("root", "tpl-root");

/// The table every render below binds (`FR-RND-003`).
const TABLE: &str = "charge";

/// The `--context` document, relative to the sandbox it is written in.
const CONTEXT: &str = "context.json";

/// The same document with `vars`, `tpl` and `now` planted in it
/// (`FR-RND-024`).
const FORGED: &str = "forged.json";

/// The four labels of `FR-ERR-008`, in the order it fixes.
const LABELS: [&str; 4] = ["error: ", "cause: ", "hint:  ", "exit:  "];

/// A template name that resolves to nothing, for the bodies about step 4 of
/// `FR-ERR-006`.
///
/// It is one edit away from [`RESOLVES`], so the nearest-match suggestion of
/// `FR-TMPL-027` has something to find and a refusal that carried none would be
/// caught.
const ABSENT: &str = "resolve";

/// The template those same bodies write, which does resolve.
///
/// It is the control for [`ABSENT`] and it renders without a model, so a body
/// may reach step 8 with it in a project that names no database entry.
const RESOLVES: &str = "resolves";

/// The template the byte-for-byte comparison renders, named as `FR-TMPL-006`
/// resolves it.
const WHOLE: &str = "whole";

/// Its source.
///
/// It is deliberately wide: every collection of `FR-CTX-036`, both reference
/// directions of `FR-CTX-009`, the decomposed type parts of `FR-CTX-015`, three
/// of the naming filters and all seven tests of `FR-ENV-014`, including the two
/// that resolve a column's table against the render context. A comparison over
/// a narrow template would hold just as well between two sources that agreed
/// about a name and about nothing else.
///
/// It references neither `now` nor anything that varies between two reads of
/// one server, which is what makes it usable for `FR-CTX-030` as well.
const WHOLE_SOURCE: &str = "{{ database.name }} {{ database.charset }} {{ database.collation }}
{{ database.server.version }} {{ database.server.series }} {{ database.server.standing }}
{%- for t in database.tables %}
T {{ t.name }} {{ t.table_type }} {{ t.engine }} {{ t.collation }} {{ t.comment }} {{ t.columns | length }} {{ t.indexes | length }} {{ t.foreign_keys | length }} {{ t.referenced_by | length }} {{ t.triggers | length }} {{ t.check_constraints | length }}
{%- if t.primary_key %} PK {{ t.primary_key.name }} {{ t.primary_key.columns | length }}{% endif %}
{%- for c in t.columns %}
C {{ c.position }} {{ c.name | pascal }} {{ c.column_type }} {{ c.data_type }} {{ c.precision }} {{ c.scale }} {{ c.length }} {{ c.unsigned }} {{ c.charset }} {{ c.collation }} {{ c.nullable }} {{ c.auto_increment }} {{ c.invisible }} {{ c is nullable }} {{ c is primary_key }} {{ c is unique }} {{ c is numeric }} {{ c is temporal }} {{ c is textual }}
{%- endfor %}
{%- for k in t.foreign_keys %}
F {{ k.name }} {{ k.match_option }} {{ k.on_update }} {{ k.on_delete }}{% if k.referenced_table %} -> {{ k.referenced_table.name }} {{ k.referenced_table.columns | length }}{% endif %}
{%- endfor %}
{%- for r in t.referenced_by %}
R {{ r.table.name }} {{ r.key.name }}
{%- endfor %}
{%- endfor %}
{%- for v in database.views %}
V {{ v.name | kebab }} {{ v.is_updatable }} {{ v.check_option }} {{ v.security_type }} {{ v.algorithm }}
{%- endfor %}
{%- for r in database.routines %}
P {{ r.name | snake }} {{ r.kind }} {{ r.parameters | length }} {{ r.is_deterministic }} {{ r.sql_data_access }}
{%- endfor %}
BOUND {{ table.name }} {{ table.columns | length }} {{ tpl.version }} {{ vars.title }}
";

/// The `--set` entry [`WHOLE_SOURCE`] reads, and the value it must carry.
const TITLE: (&str, &str) = ("title=Charges", "Charges");

/// The exit code of a run.
fn code(printed: &Output) -> Option<i32> {
    printed.status.code()
}

/// What the run wrote to stdout, as text.
fn stdout(printed: &Output) -> String {
    String::from_utf8(printed.stdout.clone()).expect("a render of this file writes UTF-8")
}

/// What the run wrote to stderr, as text.
fn stderr(printed: &Output) -> String {
    String::from_utf8(printed.stderr.clone()).expect("a diagnostic is valid UTF-8")
}

/// Runs `tpl` in `sandbox` and refuses anything but exit `0`, returning stdout.
fn succeeds(sandbox: &Sandbox, arguments: &[&str]) -> String {
    let printed = sandbox.run(arguments);

    assert_eq!(
        code(&printed),
        Some(0),
        "tpl {} exited {:?}: {}",
        arguments.join(" "),
        code(&printed),
        stderr(&printed)
    );

    stdout(&printed)
}

/// The same, with `supplied` on standard input (`FR-RND-017`).
fn succeeds_with_stdin(sandbox: &Sandbox, arguments: &[&str], supplied: &str) -> String {
    let printed = sandbox.run_with_stdin(arguments, supplied.as_bytes());

    assert_eq!(
        code(&printed),
        Some(0),
        "tpl {} exited {:?}: {}",
        arguments.join(" "),
        code(&printed),
        stderr(&printed)
    );

    stdout(&printed)
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
        Some(expected),
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

/// A sandbox holding a project that reaches `server` and carries `templates`,
/// each written under `.tpl/templates/` with the `.jinja` extension
/// `FR-TMPL-006` resolves without.
fn project(server: &Server, templates: &[(&str, &str)]) -> Sandbox {
    let sandbox = Sandbox::new();
    sandbox.project(&fixture::configuration(server, ENTRY, SCHEMA, ROOT));

    for (name, source) in templates {
        sandbox.write(&format!(".tpl/templates/{name}.jinja"), source);
    }

    sandbox
}

/// Runs `tpl schema dump | tpl <arguments>` in `sandbox` and returns what each
/// stage did (`FR-RND-017`).
///
/// The two are wired the way a shell wires them: the producer's standard output
/// **is** the consumer's standard input, one pipe with one writer and one
/// reader, so neither stage is buffered through this process.
///
/// `intercept` reads a single byte from the pipe before the consumer is given
/// it. It is what makes an observation of `FR-ERR-026` an observation rather
/// than a race — the read returns only once the producer has emitted a byte,
/// which is the condition that requirement is written about. With `false`
/// nothing is read and the pipeline is exactly the one `FR-RND-017` documents.
///
/// # Panics
///
/// Panics when either stage does not run, and when the interception finds the
/// producer's stream already at end of file.
fn piped(sandbox: &Sandbox, arguments: &[&str], intercept: bool) -> (Output, Output) {
    use std::io::Read as _;
    use std::process::{Command, Stdio};

    let mut producer = Command::new(env!("CARGO_BIN_EXE_tpl"))
        .env_clear()
        .current_dir(sandbox.root())
        .args(["schema", "dump", "--direct", "--no-cache"])
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("the binary under test runs");

    let mut document = producer.stdout.take().expect("standard output was piped");

    if intercept {
        let mut first = [0_u8; 1];

        document
            .read_exact(&mut first)
            .expect("the producer emits a document");
    }

    let consumer = Command::new(env!("CARGO_BIN_EXE_tpl"))
        .env_clear()
        .current_dir(sandbox.root())
        .args(arguments)
        .stdin(Stdio::from(document))
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()
        .expect("the binary under test runs");

    let producer = producer
        .wait_with_output()
        .expect("the binary under test terminates");

    (producer, consumer)
}

/// The same, with a `--context` document produced by the product against
/// `server` already written at [`CONTEXT`].
///
/// The document is read with `--direct --no-cache`, which `FR-CACHE-016` makes
/// the pure read: the store is left untouched, so a body that asserts something
/// about the cache is not looking at a store this helper wrote.
///
/// The dump is returned as well as written, because the standard-input form of
/// `FR-RND-017` is handed the bytes rather than the path.
fn documented(server: &Server, templates: &[(&str, &str)]) -> (Sandbox, String) {
    let sandbox = project(server, templates);
    let dumped = succeeds(&sandbox, &["schema", "dump", "--direct", "--no-cache"]);

    sandbox.write(CONTEXT, &dumped);

    (sandbox, dumped)
}

// ------------------------------------------------ step 1 of FR-ERR-006 ---

#[test]
fn fr_err_006_the_six_refusals_of_step_one_are_64_where_there_is_no_project_at_all() {
    // Step 1 of FR-ERR-006 is decided from the invocation alone, so it
    // precedes project discovery, the trust checks and `.tpl/.cfg`. The
    // sandbox below therefore holds **nothing**: no project, no template, no
    // configuration and no document — and a build that read any of them first
    // would answer `78` for a project it could not find rather than the `64`
    // each of these six owes.
    //
    // The `64` row of FR-ERR-034 obliges the refusal to name what did not
    // conform, so each case asserts the tokens at fault appear in the
    // diagnostic. The wording around them is not asserted: NFR-DET-001 puts
    // stderr outside the contract.
    let sandbox = Sandbox::new();

    for (arguments, named) in [
        // FR-RND-005: two kinds of object flag name no object.
        (
            &["render", WHOLE, "--table", "orders", "--view", "v_sales"][..],
            &["--table", "--view"][..],
        ),
        // FR-RND-014: the same key twice, refused rather than last-wins.
        (
            &["render", WHOLE, "--set", "title=one", "--set", "title=two"][..],
            &["title"][..],
        ),
        // FR-RND-012 and FR-RND-013: a dotted key is refused, never split.
        (
            &["render", WHOLE, "--set", "db.host=1"][..],
            &["db.host=1"][..],
        ),
        // FR-RND-011: an argument that is not a pair at all.
        (&["render", WHOLE, "--set", "novalue"][..], &["novalue"][..]),
        // FR-RND-018: two conflicting context sources.
        (
            &["-d", "shop", "render", WHOLE, "--context", CONTEXT][..],
            &["--context", "--database"][..],
        ),
        // FR-RND-027: `--format` is not declared, so it is the ordinary
        // unknown-flag `64` of FR-CLI-019 and not a case of its own.
        (
            &["render", WHOLE, "--format", "json"][..],
            &["--format"][..],
        ),
    ] {
        let written = refused(&sandbox, arguments, 64);

        for token in named {
            assert!(
                written.contains(token),
                "tpl {} did not name {token:?}: {written}",
                arguments.join(" ")
            );
        }
    }
}

// ------------------------------------------------------------ FR-RND-020 ---

#[test]
fn fr_rnd_020_a_document_that_is_not_the_contract_is_65_once_the_template_resolves() {
    // FR-RND-020 in both halves: bytes that are not well-formed JSON, and JSON
    // that is well formed and is not the document contract. Neither needs a
    // server.
    //
    // **The template has to exist**, and that is the second thing this body
    // establishes. FR-ERR-006 resolves the template name at step 4 and reads
    // the document at step 6, so a name that resolves to nothing is reported
    // first and the document is never read — which is the `66` the body below
    // this one asserts. Here the name resolves, so the condition reported is
    // the document's.
    //
    // A project **is** needed: FR-TMPL-023 makes the template root a property
    // of the resolved project and FR-CONF-004 resolves the render deadline from
    // `[core]`, so steps 2 and 3 run on the document path exactly as they run
    // on the other one. The project below names no database entry, so a build
    // that fell through to step 5 would answer `78` rather than `65`.
    let sandbox = Sandbox::new();
    sandbox.project("[core]\n");
    sandbox.write(&format!(".tpl/templates/{RESOLVES}.jinja"), "{{ 1 }}\n");

    for (file, bytes) in [("broken.json", "{ this is not json"), ("bare.json", "{}")] {
        sandbox.write(file, bytes);

        let written = refused(&sandbox, &["render", RESOLVES, "--context", file], 65);
        let cause = line(&written, LABELS[1]);

        // The `65` row of FR-ERR-034 obliges the path the document was read
        // from to travel on the condition.
        assert!(
            line(&written, LABELS[0]).contains(file),
            "the refusal did not name the document: {written}"
        );
        assert!(
            cause.contains(file),
            "the cause did not name the document: {written}"
        );
        assert!(
            !written.contains(RESOLVES),
            "the refusal named the template, so the condition reported is not \
             the document's: {written}"
        );
    }
}

// --------------------------- FR-ERR-006, the fourth position and its costs ---

#[test]
fn fr_err_006_a_template_that_does_not_resolve_is_66_before_the_entry_and_before_the_document() {
    // FR-ERR-006 puts template resolution at the fourth position, immediately
    // after `.tpl/.cfg`, and FR-ERR-007 reports the first condition that fails.
    // The requirement records the two consequences as accepted costs, and this
    // body is both of them, each beside the control that shows the condition it
    // now precedes is still there and still reported once the template
    // resolves.
    //
    // Neither needs a server: the project below names no database entry at all,
    // which is what makes the first control a `78`.
    let sandbox = Sandbox::new();
    sandbox.project("[core]\n");
    sandbox.write(&format!(".tpl/templates/{RESOLVES}.jinja"), "{{ 1 }}\n");
    sandbox.write("bad.json", "{}");

    // The first cost: `tpl render nosuch` in a project that selects no entry is
    // `66`, where it was `78`.
    let absent = refused(&sandbox, &["render", ABSENT], 66);

    assert!(
        line(&absent, LABELS[0]).contains(ABSENT),
        "the refusal did not name the template: {absent}"
    );
    assert!(
        line(&absent, LABELS[2]).contains(RESOLVES),
        "no nearest match among the project's templates: {absent}"
    );

    // Its control: the same project and a template that does resolve reaches
    // step 5 and is refused there, so the `66` above is the order working and
    // not a `78` that stopped being raised.
    refused(&sandbox, &["render", RESOLVES], 78);

    // The second cost: `tpl render nosuch --context bad.json` is `66`, where
    // FR-RND-020 gave it `65`.
    let document = refused(&sandbox, &["render", ABSENT, "--context", "bad.json"], 66);

    assert!(
        line(&document, LABELS[0]).contains(ABSENT),
        "the refusal did not name the template: {document}"
    );
    assert!(
        !document.contains("bad.json"),
        "the refusal named the document, so it was read before the template \
         name was judged: {document}"
    );

    // Its control is the body above: the same document, under a template that
    // resolves, is the `65` of FR-RND-020. It is asserted here too, over this
    // very file, so that the pair is one observation.
    refused(&sandbox, &["render", RESOLVES, "--context", "bad.json"], 65);
}

#[test]
fn fr_err_006_under_context_dash_the_producer_is_cut_off_and_the_pair_is_still_66() {
    // The third cost the fourth position of FR-ERR-006 records, and the only
    // one visible outside the process: under `--context -` the document is no
    // longer read before the template name is judged, so the producer of the
    // pipeline FR-RND-017 documents is **cut off rather than drained**.
    //
    // Two things are asserted, and the arrangement differs between them for a
    // reason stated below.
    //
    // 1. What a caller branches on is unchanged. The plain pipeline exits `66`:
    //    the refusal is the consumer's, it is the rightmost stage, and it is
    //    therefore what a shell reports with `pipefail` and without it.
    //
    // 2. The producer is cut off. FR-ERR-026 makes a stdout closed **after** a
    //    byte of a JSON document was emitted a `74`, where before this move the
    //    consumer read the document to end of file and the producer exited `0`.
    //
    // **Why the second assertion reads one byte first.** The two halves of the
    // pair are separate processes and nothing orders them: the consumer refuses
    // after four cheap steps while the producer is still opening a connection
    // and reading a catalogue, so in the plain pipeline the producer's first
    // write finds the reader already gone and no byte of the document was ever
    // emitted — which is the silent `0` of FR-ERR-025 and not this cost at all.
    // Reading one byte from the producer's stdout before the consumer is handed
    // the rest establishes the condition FR-ERR-026 names, as an observation
    // rather than as a race: the read returns only once a byte has been
    // emitted. A dump of the fixture is hundreds of kilobytes and nothing
    // drains the pipe, so the producer is still writing when the consumer
    // exits.
    //
    // The interception is neutral between the two behaviours under test: a
    // consumer that drained the pipe would let the producer finish and exit
    // `0`, with one byte fewer to read, and that is what this arrangement
    // produced before the move.
    let _guard = fixture::exclusive();
    let Some(series) = fixture::series(
        "fr_err_006_under_context_dash_the_producer_is_cut_off_and_the_pair_is_still_66",
    ) else {
        return;
    };

    for server in series {
        let name = server.name();
        let sandbox = project(server, &[(RESOLVES, "{{ database.name }}\n")]);

        // The plain pipeline, which is what FR-RND-017 writes.
        let (producer, consumer) = piped(&sandbox, &["render", ABSENT, "--context", "-"], false);

        assert_eq!(
            code(&consumer),
            Some(66),
            "{name}: the consumer of the pipeline did not refuse the template: {}",
            stderr(&consumer)
        );
        assert!(
            consumer.stdout.is_empty(),
            "{name}: the refused render wrote {} byte(s) to stdout",
            consumer.stdout.len()
        );
        assert!(
            producer.status.code().is_some(),
            "{name}: the producer was signalled rather than exited, so neither \
             FR-ERR-025 nor FR-ERR-026 decided its status"
        );

        // The same pipeline with one byte of the document taken first, which is
        // the condition FR-ERR-026 names.
        let (producer, consumer) = piped(&sandbox, &["render", ABSENT, "--context", "-"], true);

        assert_eq!(
            code(&consumer),
            Some(66),
            "{name}: the consumer did not refuse the template: {}",
            stderr(&consumer)
        );
        assert_eq!(
            code(&producer),
            Some(74),
            "{name}: a producer cut off part-way through its document did not \
             exit 74: {}",
            stderr(&producer)
        );

        // The control, and it is what the move changed: a consumer that does
        // drain the same producer leaves it exiting `0`. The template resolves
        // here, so the document is read to end of file exactly as it was before
        // template resolution moved.
        let (producer, consumer) = piped(&sandbox, &["render", RESOLVES, "--context", "-"], false);

        assert_eq!(
            code(&consumer),
            Some(0),
            "{name}: the draining consumer did not render: {}",
            stderr(&consumer)
        );
        assert_eq!(
            code(&producer),
            Some(0),
            "{name}: a drained producer did not exit 0: {}",
            stderr(&producer)
        );
    }
}

// ------------------------------------- the two sources, and the one result ---

#[test]
fn fr_rnd_016_and_fr_rnd_026_one_template_and_one_object_render_the_same_bytes_from_every_source() {
    // **The claim of this sprint.** One template and one bound object produce
    // the same bytes whichever source produced the model, and there are four
    // ways a model can arrive:
    //
    // | Source | How it is reached | Requirement |
    // |---|---|---|
    // | The server, with the store untouched | `--direct --no-cache` | `FR-CACHE-016` |
    // | The server, through a store this read fills | the read-through of `FR-CACHE-006` | `FR-RND-026` |
    // | The store, now warm | the same invocation again | `FR-CACHE-006` |
    // | A document, from a file and from standard input | `--context <path>` and `--context -` | `FR-RND-016`, `FR-RND-017` |
    //
    // The comparison is over the **bytes**, which is the whole of what a caller
    // receives, and the template is wide enough that two sources agreeing about
    // a name and nothing else would not pass it.
    //
    // The project names an entry and `core.database` selects it, and the
    // `--context` runs are made against that same project without changing it:
    // FR-RND-019 makes a selected entry no conflict at all when `--context` is
    // supplied and `-d/--database` is not.
    let _guard = fixture::exclusive();
    let Some(series) = fixture::series(
        "fr_rnd_016_and_fr_rnd_026_one_template_and_one_object_render_the_same_bytes_from_every_source",
    ) else {
        return;
    };

    for server in series {
        let name = server.name();
        let sandbox = project(server, &[(WHOLE, WHOLE_SOURCE)]);
        let store = sandbox.path(&format!(".tpl/.cache/{ENTRY}"));
        let render: Vec<&str> = vec!["render", WHOLE, "--table", TABLE, "--set", TITLE.0];

        let mut pure = render.clone();
        pure.extend_from_slice(&["--direct", "--no-cache"]);

        // The server, with nothing of the store read and nothing written.
        let from_server = succeeds(&sandbox, &pure);

        assert!(
            !store.exists(),
            "{name}: FR-CACHE-016 makes --direct --no-cache the pure read, and it \
             wrote {}",
            store.display()
        );

        // The server again, this time through the read-through of FR-RND-026,
        // which fills the store on its way past.
        let filling = succeeds(&sandbox, &render);

        assert!(
            store.exists(),
            "{name}: the read-through render wrote no store, so the run below is \
             a second server read rather than a cache read"
        );

        // The store, now warm.
        let from_cache = succeeds(&sandbox, &render);

        // The document, in both of the forms FR-RND-021 makes one contract.
        let dumped = succeeds(&sandbox, &["schema", "dump", "--direct", "--no-cache"]);
        sandbox.write(CONTEXT, &dumped);

        let mut from_file = render.clone();
        from_file.extend_from_slice(&["--context", CONTEXT]);
        let from_file = succeeds(&sandbox, &from_file);

        let mut from_stdin = render.clone();
        from_stdin.extend_from_slice(&["--context", "-"]);
        let from_stdin = succeeds_with_stdin(&sandbox, &from_stdin, &dumped);

        // The control: the comparison is over a result that carries the model,
        // and not over two empty strings or two error-free nothings.
        assert!(
            from_server.contains(&format!("BOUND {TABLE} ")) && from_server.lines().count() > 100,
            "{name}: the render produced {} line(s), which is not the model",
            from_server.lines().count()
        );
        assert!(
            from_server.contains(TITLE.1),
            "{name}: the render did not carry the --set entry it was given"
        );

        for (source, produced) in [
            ("the read-through server read", &filling),
            ("the cache", &from_cache),
            ("a --context document in a file", &from_file),
            ("a --context document on stdin", &from_stdin),
        ] {
            assert!(
                &from_server == produced,
                "{name}: the result betrays its source — a model from {source} \
                 rendered different bytes from one read straight from the server"
            );
        }
    }
}

// ------------------------------------------ FR-CACHE-038 and FR-CACHE-039 ---

#[test]
fn fr_cache_039_a_miss_reached_during_a_render_is_one_server_read_and_one_result() {
    // FR-CACHE-039: a render served from the store reads each object file when
    // the template first reaches it, and a file that is then a miss abandons the
    // render. The invocation is a miss like any other — one connection, per
    // NFR-PERF-004, the write of FR-CACHE-007, and a render from the server's
    // document — and nothing of the abandoned render reaches stdout, per
    // FR-RND-034: the template below writes the database before it reaches a
    // table, so a byte that escaped would stand before the one result.
    //
    // The control is the miss the fortieth edition did not change, found
    // before the render starts: the bound table's own file damaged. Both are
    // compared with the render a warm store served, and with each other.
    let _guard = fixture::exclusive();
    let Some(series) = fixture::series(
        "fr_cache_039_a_miss_reached_during_a_render_is_one_server_read_and_one_result",
    ) else {
        return;
    };

    for server in series {
        let name = server.name();
        let control =
            fixture::connections_attributable_to(server, || fixture::connect_once(server));
        assert_eq!(
            control, 1,
            "{name}: the connection record did not count a connection that was made"
        );

        let sandbox = project(server, &[(WHOLE, WHOLE_SOURCE)]);
        let tables = sandbox.path(&format!(".tpl/.cache/{ENTRY}/tables"));
        let render = ["render", WHOLE, "--table", TABLE, "--set", TITLE.0];
        let mut unstored = render.to_vec();
        unstored.push("--no-cache");

        succeeds(&sandbox, &["cache", "load"]);
        let from_cache = succeeds(&sandbox, &render);
        assert!(
            from_cache.contains(&format!("BOUND {TABLE} ")),
            "{name}: the control render carries no model"
        );

        let bound = tables.join(format!("{TABLE}.json"));
        let reached = std::fs::read_dir(&tables)
            .expect("the store holds its tables")
            .flatten()
            .map(|entry| entry.path())
            .filter(|path| path != &bound)
            .max()
            .expect("the fixture carries a second table");
        let held = std::fs::read(&reached).expect("the store is ours");

        for (case, damaged, arguments, rewritten) in [
            ("reached during the render", &reached, &render[..], true),
            ("found before the render", &bound, &render[..], true),
            ("reached, under --no-cache", &reached, &unstored[..], false),
        ] {
            let original = std::fs::read(damaged).expect("the store is ours");
            std::fs::write(damaged, "{ torn").expect("the store is ours");

            let mut printed = None;
            let opened = fixture::connections_attributable_to(server, || {
                printed = Some(sandbox.run(arguments));
            });
            let printed = printed.expect("the body ran");

            assert_eq!(
                code(&printed),
                Some(0),
                "{name}, {case}: {}",
                stderr(&printed)
            );
            assert!(
                printed.stderr.is_empty(),
                "{name}, {case}: FR-CACHE-033 reports nothing: {}",
                stderr(&printed)
            );
            assert!(
                stdout(&printed) == from_cache,
                "{name}, {case}: stdout is not the one result the warm store rendered"
            );
            assert_eq!(
                opened, 1,
                "{name}, {case}: NFR-PERF-004 allows one connection"
            );

            let now = std::fs::read(damaged).expect("the store is ours");
            if rewritten {
                assert_eq!(
                    now, original,
                    "{name}, {case}: FR-CACHE-007 rewrote the file"
                );
            } else {
                assert_eq!(now, b"{ torn", "{name}, {case}: FR-CACHE-014 wrote nothing");
                std::fs::write(damaged, &original).expect("the store is ours");
            }
        }

        assert_eq!(
            std::fs::read(&reached).expect("the store is ours"),
            held,
            "{name}: the store ends as it began"
        );
    }
}

// ------------------------------------------- FR-RND-029 and FR-RND-032 ---

#[test]
fn fr_rnd_029_and_fr_rnd_032_a_name_that_reaches_nothing_is_66_with_a_nearest_match() {
    // Two names, two steps of FR-ERR-006, one exit code. FR-RND-029 is step 4
    // and FR-RND-032 is step 7, and both take the nearest-match suggestion of
    // FR-ERR-019 — the `66` row of FR-ERR-034 obliges the population the name
    // was sought in to be named beside it, which for the object on this path is
    // the document and the database it describes rather than an entry and a
    // server.
    let _guard = fixture::exclusive();
    let Some(series) = fixture::series(
        "fr_rnd_029_and_fr_rnd_032_a_name_that_reaches_nothing_is_66_with_a_nearest_match",
    ) else {
        return;
    };

    for server in series {
        let name = server.name();
        let (sandbox, _) = documented(server, &[(WHOLE, WHOLE_SOURCE)]);

        // Step 4 first, because it runs first: the object named is one the
        // document does carry, so nothing but the template can be the
        // condition.
        let absent_template = refused(
            &sandbox,
            &["render", "whol", "--context", CONTEXT, "--table", TABLE],
            66,
        );

        assert!(
            line(&absent_template, LABELS[0]).contains("whol"),
            "{name}: {absent_template}"
        );
        assert!(
            line(&absent_template, LABELS[2]).contains(WHOLE),
            "{name}: no nearest match among the project's templates: \
             {absent_template}"
        );

        // Step 7, reached because step 4 passed: the template resolves and the
        // object is the thing the document does not carry, misspelled by one
        // byte.
        let absent_object = refused(
            &sandbox,
            &["render", WHOLE, "--context", CONTEXT, "--table", "charg"],
            66,
        );

        assert!(
            line(&absent_object, LABELS[0]).contains("charg"),
            "{name}: {absent_object}"
        );
        assert!(
            line(&absent_object, LABELS[1]).contains(CONTEXT)
                && line(&absent_object, LABELS[1]).contains(SCHEMA),
            "{name}: the cause did not name the population the object was sought \
             in: {absent_object}"
        );
        assert!(
            line(&absent_object, LABELS[2]).contains(TABLE),
            "{name}: no nearest match for a name one byte from a real one: \
             {absent_object}"
        );
    }
}

// ------------------------------- FR-RND-030, FR-RND-031 and FR-SEM-019 ---

/// Every way a render fails, and the requirement each is written for.
///
/// `FR-SEM-019` makes all of them one kind of failure — a render failure under
/// `FR-RND-031`, carrying the template, the line and the column — so they are
/// driven as one list rather than as five bodies.
///
/// [`outside_the_process`](../outside_the_process/index.html) drives the same
/// five for the other half of what they owe: the bytes they did not write to
/// the process's own stdout, per `FR-RND-034` and `FR-SEM-020`. The list is
/// written twice because the two are separate test binaries, and neither copy
/// can make the other wrong: each is the population of the body beside it.
const FAILING: [(&str, &str, &str); 5] = [
    ("broken", "kept\n{% if %}\n", "FR-RND-030, a syntax error"),
    (
        "undefined",
        "kept\n{{ database.missing }}\n",
        "FR-SEM-012 through FR-RND-031, a field that does not exist",
    ),
    (
        "stopped",
        "kept\n{{ fail('no mapping') }}\n",
        "FR-SEM-014, the author ending the render",
    ),
    (
        "operand",
        "kept\n{{ 42 | snake }}\n",
        "FR-SEM-008 and FR-SEM-009, a filter given an operand it does not accept",
    ),
    (
        "predicate",
        "kept\n{% if database is nullable %}x{% endif %}\n",
        "FR-SEM-005 and FR-SEM-007, a test given an operand it does not accept",
    ),
];

#[test]
fn fr_sem_019_every_way_a_render_fails_is_65_and_says_where_it_failed() {
    // FR-RND-030 for the syntax error, FR-RND-031 for the four that fail while
    // being evaluated, and FR-SEM-019 for what every one of them must carry:
    // the template name, the line and the column. FR-SEM-007 and the second
    // sentence of FR-SEM-008 are what the last two are for — a test that
    // answered `false` and a filter that returned the empty string would both
    // have exited `0` here, and would have put a plausible wrong answer in a
    // generated file.
    //
    // The bytes these runs did not write to stdout are asserted in
    // `outside_the_process`, on the process's own stream; what is asserted here
    // is the code and the diagnostic.
    let _guard = fixture::exclusive();
    let Some(series) =
        fixture::series("fr_sem_019_every_way_a_render_fails_is_65_and_says_where_it_failed")
    else {
        return;
    };

    let templates: Vec<(&str, &str)> = FAILING
        .iter()
        .map(|(template, source, _)| (*template, *source))
        .collect();

    for server in series {
        let name = server.name();
        let (sandbox, _) = documented(server, &templates);

        for (template, _, requirement) in FAILING {
            let written = refused(&sandbox, &["render", template, "--context", CONTEXT], 65);
            let reported = line(&written, LABELS[0]);

            assert!(
                reported.contains(template),
                "{name}: {requirement} did not name the template: {written}"
            );
            assert!(
                reported.contains("line 2") && reported.contains("column"),
                "{name}: {requirement} did not report the line and the column \
                 FR-SEM-019 obliges: {written}"
            );
        }
    }
}

// ------------------------------------------- FR-CTX-029 and FR-CTX-030 ---

/// A template that reads `now` twice, and nothing else.
const TWICE: (&str, &str) = ("twice", "{{ now }}|{{ now }}");

#[test]
fn fr_ctx_029_and_fr_ctx_030_now_is_one_instant_and_a_template_without_it_repeats_byte_for_byte() {
    // FR-CTX-029: `now` is evaluated once per invocation and every reference in
    // one render yields the same value — so the two halves below are one string
    // read twice and not two readings of a clock.
    //
    // FR-CTX-030: `now` is the **single** documented source of
    // non-reproducibility, which is a claim about every other variable. A
    // template that does not reference it produces byte-identical output
    // between runs against unchanged inputs, and [`WHOLE_SOURCE`] is that
    // template: the two runs below are made against one document, so the inputs
    // are unchanged in the strongest sense available.
    let _guard = fixture::exclusive();
    let Some(series) = fixture::series(
        "fr_ctx_029_and_fr_ctx_030_now_is_one_instant_and_a_template_without_it_repeats_byte_for_byte",
    ) else {
        return;
    };

    for server in series {
        let name = server.name();
        let (sandbox, _) = documented(server, &[(WHOLE, WHOLE_SOURCE), TWICE]);

        let written = succeeds(&sandbox, &["render", TWICE.0, "--context", CONTEXT]);
        let (first, second) = written
            .split_once('|')
            .unwrap_or_else(|| panic!("{name}: the template wrote {written:?}"));

        assert_eq!(
            first, second,
            "{name}: two references to `now` in one render yielded two values"
        );

        // FR-CTX-028 fixes the one format: RFC 3339, UTC, a `Z` offset and
        // second precision, which is twenty characters and no more.
        assert_eq!(first.len(), 20, "{name}: `now` was {first:?}");
        assert!(first.ends_with('Z'), "{name}: `now` was {first:?}");
        assert_eq!(&first[4..5], "-", "{name}: `now` was {first:?}");
        assert_eq!(&first[10..11], "T", "{name}: `now` was {first:?}");

        let once = succeeds(
            &sandbox,
            &[
                "render",
                WHOLE,
                "--context",
                CONTEXT,
                "--table",
                TABLE,
                "--set",
                TITLE.0,
            ],
        );
        let again = succeeds(
            &sandbox,
            &[
                "render",
                WHOLE,
                "--context",
                CONTEXT,
                "--table",
                TABLE,
                "--set",
                TITLE.0,
            ],
        );

        assert!(
            !once.is_empty(),
            "{name}: the template produced nothing, so the equality below \
             compares two empty strings"
        );
        assert_eq!(
            once, again,
            "{name}: a template that never references `now` produced different \
             bytes on two runs against one document"
        );
    }
}

// ------------------------------------------- FR-CTX-026 and FR-RND-024 ---

/// A template that reads `vars` whole, and the three injected variables.
const THREE: (&str, &str) = ("three", "{{ vars }}|{{ tpl.version }}|{{ now }}");

/// A template that reads the two `--set` entries `FR-RND-009` and `FR-RND-010`
/// are about.
const PAIR: (&str, &str) = ("pair", "[{{ vars.empty }}][{{ vars.msg }}]");

/// The instant the forged document below states, which must reach nothing.
const PLANTED_NOW: &str = "1999-01-01T00:00:00Z";

#[test]
fn fr_ctx_026_and_fr_rnd_024_vars_is_this_invocations_and_a_document_supplies_none_of_the_three() {
    // FR-CTX-026: `vars` is the `--set` keys of **this** invocation, and `{}`
    // where there are none. FR-RND-009 and FR-RND-010 are the two splitting
    // rules that a caller generating a command line meets first — the first `=`
    // and nothing after it, and an empty value that is valid.
    //
    // FR-RND-024: `vars`, `tpl` and `now` are always injected, and a value a
    // `--context` document supplies for any of them is ignored. The forged
    // document below plants all three at every level a producer could plausibly
    // have put them — beside `schema_version`, inside `data`, and inside
    // `database` — and none of them reaches the template.
    let _guard = fixture::exclusive();
    let Some(series) = fixture::series(
        "fr_ctx_026_and_fr_rnd_024_vars_is_this_invocations_and_a_document_supplies_none_of_the_three",
    ) else {
        return;
    };

    for server in series {
        let name = server.name();
        let (sandbox, dumped) = documented(server, &[THREE, PAIR]);

        // FR-CTX-026: no --set at all.
        let empty = succeeds(&sandbox, &["render", THREE.0, "--context", CONTEXT]);
        let fields: Vec<&str> = empty.split('|').collect();

        assert_eq!(fields[0], "{}", "{name}: `vars` was {empty:?}");
        assert_eq!(
            fields[1],
            env!("CARGO_PKG_VERSION"),
            "{name}: FR-CTX-027 makes `tpl.version` the version the binary is"
        );

        // FR-RND-010 and FR-RND-009: an empty value, and a value carrying the
        // separator it was split on.
        let written = succeeds(
            &sandbox,
            &[
                "render",
                PAIR.0,
                "--context",
                CONTEXT,
                "--set",
                "empty=",
                "--set",
                "msg=a=b",
            ],
        );

        assert_eq!(written, "[][a=b]", "{name}");

        // FR-RND-024, over a document that states all three itself.
        let mut forged: serde_json::Value =
            serde_json::from_str(&dumped).expect("tpl schema dump emits JSON");
        let planted = serde_json::json!({"title": "forged"});

        forged["vars"] = planted.clone();
        forged["tpl"] = serde_json::json!({"version": "9.9.9"});
        forged["now"] = serde_json::json!(PLANTED_NOW);
        forged["data"]["vars"] = planted.clone();
        forged["data"]["tpl"] = serde_json::json!({"version": "9.9.9"});
        forged["data"]["now"] = serde_json::json!(PLANTED_NOW);
        forged["data"]["database"]["vars"] = planted;
        forged["data"]["database"]["tpl"] = serde_json::json!({"version": "9.9.9"});
        forged["data"]["database"]["now"] = serde_json::json!(PLANTED_NOW);

        sandbox.write(FORGED, &forged.to_string());

        let ignored = succeeds(&sandbox, &["render", THREE.0, "--context", FORGED]);
        let fields: Vec<&str> = ignored.split('|').collect();

        assert_eq!(
            fields[0], "{}",
            "{name}: a document supplied `vars`: {ignored:?}"
        );
        assert_eq!(
            fields[1],
            env!("CARGO_PKG_VERSION"),
            "{name}: a document supplied `tpl`: {ignored:?}"
        );
        assert_ne!(
            fields[2], PLANTED_NOW,
            "{name}: a document supplied `now`: {ignored:?}"
        );
    }
}

// ------------------------------------------------- FR-RND-038 and FR-RND-039 ---

/// A `--context` document carrying a database with no member, which is all the
/// bodies below need: what they measure is the render, not the model.
const EMPTY_CONTEXT: &str = r#"{"schema_version":1,"source":"server","data":{"database":{"name":"shop","charset":"utf8mb4","collation":"utf8mb4_general_ci","server":{"version":"11.4.13-MariaDB","series":"11.4","standing":"supported"},"tables":[],"views":[],"routines":[]}}}"#;

/// A template that doubles a string through a `namespace` until it would hold
/// 2^40 bytes — the shape of hardening observation H-1 — within a few dozen
/// evaluation steps, far inside the fuel budget.
const DOUBLING: &str = "{% set ns = namespace(s='x') %}\
                        {% for i in range(40) %}{% set ns.s = ns.s ~ ns.s %}{% endfor %}\
                        {{ ns.s | length }}";

/// A template that legitimately holds a 32 MiB string and prints its length.
const LARGE: &str = "{% set ns = namespace(s='x') %}\
                     {% for i in range(25) %}{% set ns.s = ns.s ~ ns.s %}{% endfor %}\
                     {{ ns.s | length }}\n";

/// A sandbox holding a project whose `.tpl/.cfg` is `configuration`, the
/// templates given, and [`EMPTY_CONTEXT`] at [`CONTEXT`].
fn bounded_project(configuration: &str, templates: &[(&str, &str)]) -> Sandbox {
    let sandbox = Sandbox::new();
    sandbox.project(configuration);
    sandbox.write(CONTEXT, EMPTY_CONTEXT);

    for (name, source) in templates {
        sandbox.write(&format!(".tpl/templates/{name}.jinja"), source);
    }

    sandbox
}

#[test]
fn fr_rnd_039_a_render_that_grows_past_its_memory_limit_is_65_naming_the_bound_and_writes_nothing()
{
    // FR-RND-039, FR-SEC-025, ADR-011: the counting allocator the binary
    // installs is read by the render's watchdog, and a count above the limit
    // ends the render with 65 and a cause naming the bound, its resolved value
    // and the key. Nothing reaches stdout, per FR-RND-034.
    let sandbox = bounded_project(
        "[core]\nrender_memory_limit = 16777216\n",
        &[("grow", DOUBLING)],
    );

    let written = refused(&sandbox, &["render", "grow", "--context", CONTEXT], 65);
    let cause = line(&written, "cause: ");

    assert!(cause.contains("render memory limit"), "{cause}");
    assert!(cause.contains("16777216"), "{cause}");
    assert!(cause.contains("core.render_memory_limit"), "{cause}");
    assert!(
        line(&written, "hint:  ").contains("tpl cfg set core.render_memory_limit"),
        "{written}"
    );
}

#[test]
fn fr_rnd_039_the_default_memory_limit_stops_the_doubling_render() {
    // FR-CONF-002: the default, 128 MiB, applies where the key is absent.
    let sandbox = bounded_project("[core]\n", &[("grow", DOUBLING)]);

    let written = refused(&sandbox, &["render", "grow", "--context", CONTEXT], 65);

    assert!(line(&written, "cause: ").contains("134217728"), "{written}");
}

#[test]
fn fr_conf_045_raising_the_memory_limit_lets_a_legitimately_large_render_pass() {
    // The same 32 MiB render is refused under a 16 MiB limit and produced
    // under a raised one.
    let low = bounded_project(
        "[core]\nrender_memory_limit = 16777216\n",
        &[("large", LARGE)],
    );
    let written = refused(&low, &["render", "large", "--context", CONTEXT], 65);
    assert!(
        line(&written, "cause: ").contains("render memory limit"),
        "{written}"
    );

    let raised = bounded_project(
        "[core]\nrender_memory_limit = 536870912\n",
        &[("large", LARGE)],
    );
    assert_eq!(
        succeeds(&raised, &["render", "large", "--context", CONTEXT]),
        "33554432\n"
    );
}

#[test]
fn fr_rnd_038_under_the_defaults_endless_output_reaches_the_output_limit_before_the_memory_limit() {
    // FR-RND-038 and FR-CONF-045: the output is held until the render ends, so
    // it counts toward the memory limit; at 64 MiB the output default sits at
    // half the memory default, and the output limit is the one reported.
    let sandbox = bounded_project(
        "[core]\n",
        &[(
            "flood",
            "{% set s = 'x' * 1048576 %}{% for i in range(100) %}{{ s }}{% endfor %}",
        )],
    );

    let written = refused(&sandbox, &["render", "flood", "--context", CONTEXT], 65);
    let cause = line(&written, "cause: ");

    assert!(cause.contains("render output limit"), "{cause}");
    assert!(cause.contains("67108864"), "{cause}");
}

#[test]
fn fr_conf_045_a_memory_limit_below_its_floor_is_78_in_the_file_and_64_on_the_command_line() {
    let sandbox = bounded_project(
        "[core]\nrender_memory_limit = 8388607\n",
        &[("large", LARGE)],
    );

    let written = refused(&sandbox, &["render", "large", "--context", CONTEXT], 78);
    let cause = line(&written, "cause: ");
    assert!(cause.contains("core.render_memory_limit"), "{cause}");
    assert!(cause.contains("8388607"), "{cause}");
    assert!(cause.contains("8388608"), "{cause}");

    let valid = bounded_project("[core]\n", &[]);
    refused(
        &valid,
        &["cfg", "set", "core.render_memory_limit", "8388607"],
        64,
    );
}

// ----------------------------------------------------------------- FR-RND-040 ---

/// A template that reads the whole database and then runs for a few seconds
/// before it produces its only byte.
///
/// The render holds its output until it ends, so its first byte reaches stdout
/// only after the loop: a session that is recorded as ended while the process
/// is still running is ended before that byte.
const SLOW_AFTER_READ: &str = "{% for t in database.tables %}{{ t.name | length }}{% endfor %}\
                               {% for i in range(10000) %}{% for j in range(1000) %}{% endfor %}{% endfor %}\
                               done\n";

#[test]
fn fr_rnd_040_every_server_read_of_a_render_is_closed_before_the_template_evaluates() {
    // FR-RND-040, in process: the render refuses to start, as the 70 of a
    // violated invariant, while its thread holds a connection or a driver
    // runtime. Each of the three paths that read the server must therefore
    // exit 0 with the render's bytes: `--direct`, a miss on an empty store,
    // and the re-read of FR-CACHE-039 after an abandoned render.
    let _guard = fixture::exclusive();
    let Some(series) = fixture::series(
        "fr_rnd_040_every_server_read_of_a_render_is_closed_before_the_template_evaluates",
    ) else {
        return;
    };

    for server in series {
        let name = server.name();
        let render = ["render", WHOLE, "--table", TABLE, "--set", TITLE.0];

        let direct = project(server, &[(WHOLE, WHOLE_SOURCE)]);
        let mut unstored = render.to_vec();
        unstored.push("--direct");
        let expected = succeeds(&direct, &unstored);
        assert!(
            expected.contains(&format!("BOUND {TABLE} ")),
            "{name}: --direct rendered no model"
        );

        let missed = project(server, &[(WHOLE, WHOLE_SOURCE)]);
        assert_eq!(
            succeeds(&missed, &render),
            expected,
            "{name}: a miss on an empty store"
        );

        let abandoned = project(server, &[(WHOLE, WHOLE_SOURCE)]);
        succeeds(&abandoned, &["cache", "load"]);
        let tables = abandoned.path(&format!(".tpl/.cache/{ENTRY}/tables"));
        let bound = tables.join(format!("{TABLE}.json"));
        let reached = std::fs::read_dir(&tables)
            .expect("the store holds its tables")
            .flatten()
            .map(|entry| entry.path())
            .filter(|path| path != &bound)
            .max()
            .expect("the fixture carries a second table");
        std::fs::write(&reached, "{ torn").expect("the store is ours");
        assert_eq!(
            succeeds(&abandoned, &render),
            expected,
            "{name}: the re-read of FR-CACHE-039"
        );
    }
}

#[test]
fn fr_rnd_040_the_server_records_the_session_ended_before_the_first_byte_of_the_render() {
    // FR-RND-040, from the server side: the general log records the Quit of
    // the render's connection while the render is still running, which is
    // before its first byte reaches stdout.
    use std::io::Read as _;
    use std::process::{Command, Stdio};
    use std::time::{Duration, Instant};

    let _guard = fixture::exclusive();
    let Some(series) = fixture::series(
        "fr_rnd_040_the_server_records_the_session_ended_before_the_first_byte_of_the_render",
    ) else {
        return;
    };

    for server in series {
        let name = server.name();
        let sandbox = Sandbox::new();
        sandbox.project(
            &fixture::configuration(server, ENTRY, SCHEMA, ROOT).replacen(
                "[core]\n",
                "[core]\nrender_fuel = 1000000000000\n",
                1,
            ),
        );
        sandbox.write(".tpl/templates/slow.jinja", SLOW_AFTER_READ);

        fixture::statements_on(server);

        let mut child = Command::new(env!("CARGO_BIN_EXE_tpl"))
            .env_clear()
            .current_dir(sandbox.root())
            .args(["render", "slow", "--direct"])
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .expect("the binary under test runs");

        let until = Instant::now() + Duration::from_secs(60);
        let observed = loop {
            let log = fixture::statements_text(server, &[]);
            if log
                .lines()
                .any(|row| row.split('\t').nth(1) == Some("Quit"))
            {
                break child.try_wait().expect("the child can be polled");
            }
            assert!(
                child.try_wait().expect("the child can be polled").is_none(),
                "{name}: the render ended before the session's end was recorded: {log}"
            );
            assert!(
                Instant::now() < until,
                "{name}: no Quit was recorded: {log}"
            );
            std::thread::sleep(Duration::from_millis(50));
        };

        fixture::statements_off(server);

        assert!(
            observed.is_none(),
            "{name}: the session was recorded as ended only after the render had ended"
        );

        let mut produced = String::new();
        child
            .stdout
            .take()
            .expect("stdout was piped")
            .read_to_string(&mut produced)
            .expect("the render writes UTF-8");
        let status = child.wait().expect("the binary under test terminates");

        assert_eq!(status.code(), Some(0), "{name}");
        assert!(produced.ends_with("done\n"), "{name}: {produced:?}");
    }
}

// ------------------------------------ FR-CACHE-039 with FR-RND-038: bounds kept ---

#[test]
fn fr_cache_039_an_abandoned_render_that_crosses_any_bound_ends_with_65_and_reads_no_server() {
    // FR-CACHE-039, second paragraph, FR-RND-038 and the note on FR-ERR-006:
    // an abandoned render keeps every render bound until it has returned. A
    // render that reaches the miss through `table(...)` survives it and keeps
    // evaluating; crossing the deadline, the memory limit, render fuel or the
    // output limit after the miss ends the invocation with 65 and that bound's
    // cause, opens no connection and writes nothing to stdout. The control —
    // the same miss within every bound — reads the server and renders.
    let _guard = fixture::exclusive();
    let Some(series) = fixture::series(
        "fr_cache_039_an_abandoned_render_that_crosses_any_bound_ends_with_65_and_reads_no_server",
    ) else {
        return;
    };

    let reach = "{% set t = table('vessel') %}";
    let cases = [
        (
            "the deadline",
            "render_timeout = 1\nrender_fuel = 1000000000000\n",
            "{% for i in range(100000) %}{% for j in range(10000) %}{% endfor %}{% endfor %}",
            "deadline of 1s",
        ),
        (
            "the memory limit",
            "render_memory_limit = 16777216\n",
            "{% set ns = namespace(s='x') %}{% for i in range(40) %}{% set ns.s = ns.s ~ ns.s %}{% endfor %}{{ ns.s | length }}",
            "render memory limit of 16777216",
        ),
        (
            "render fuel",
            "render_fuel = 5000\n",
            "{% for i in range(10000) %}{% for j in range(10000) %}{% endfor %}{% endfor %}",
            "render fuel",
        ),
        (
            "the output limit",
            "render_output_limit = 64\n",
            "{% for i in range(100) %}0123456789{% endfor %}",
            "render output limit of 64",
        ),
    ];

    for server in series {
        let name = server.name();

        for (bound, keys, after, named) in cases {
            let sandbox = Sandbox::new();
            sandbox.project(
                &fixture::configuration(server, ENTRY, SCHEMA, ROOT).replacen(
                    "[core]\n",
                    &format!("[core]\n{keys}"),
                    1,
                ),
            );
            sandbox.write(".tpl/templates/abandon.jinja", &format!("{reach}{after}"));
            sandbox.write(".tpl/templates/calm.jinja", &format!("{reach}calm\n"));
            succeeds(&sandbox, &["cache", "load"]);
            let vessel = sandbox.path(&format!(".tpl/.cache/{ENTRY}/tables/vessel.json"));
            let intact = std::fs::read(&vessel).expect("the store holds vessel");

            std::fs::write(&vessel, "{ torn").expect("the store is ours");
            let mut printed = None;
            let opened = fixture::connections_attributable_to(server, || {
                printed = Some(sandbox.run(&["render", "abandon"]));
            });
            let printed = printed.expect("the body ran");

            assert_eq!(
                code(&printed),
                Some(65),
                "{name}, {bound}: {}",
                stderr(&printed)
            );
            assert!(
                printed.stdout.is_empty(),
                "{name}, {bound}: wrote {} bytes",
                printed.stdout.len()
            );
            assert!(
                line(&stderr(&printed), "cause: ").contains(named),
                "{name}, {bound}: {}",
                stderr(&printed)
            );
            assert_eq!(opened, 0, "{name}, {bound}: a connection was opened");

            // The control: the same miss, within every bound, reads the server.
            std::fs::write(&vessel, "{ torn").expect("the store is ours");
            let mut calm = None;
            let reread = fixture::connections_attributable_to(server, || {
                calm = Some(sandbox.run(&["render", "calm"]));
            });
            let calm = calm.expect("the body ran");
            assert_eq!(code(&calm), Some(0), "{name}, {bound}: {}", stderr(&calm));
            assert_eq!(stdout(&calm), "calm\n", "{name}, {bound}");
            assert_eq!(
                reread, 1,
                "{name}, {bound}: the control reads the server once"
            );
            assert_eq!(
                std::fs::read(&vessel).expect("the store holds vessel"),
                intact,
                "{name}, {bound}: FR-CACHE-007 rewrote the file"
            );
        }
    }
}

#[test]
fn fr_err_034_a_context_file_that_cannot_be_read_names_the_flag_and_not_the_project() {
    // Finding E-04: the path is the caller's, outside `.tpl`, so the hint
    // points at `--context` rather than at the project's permissions.
    let sandbox = Sandbox::new();
    sandbox.project("[core]\n");
    sandbox.write(&format!(".tpl/templates/{RESOLVES}.jinja"), "{{ 1 }}\n");

    let written = refused(
        &sandbox,
        &["render", RESOLVES, "--context", "nofile.json"],
        74,
    );

    assert_eq!(
        line(&written, LABELS[0]),
        "the --context file nofile.json could not be read"
    );
    assert!(line(&written, LABELS[2]).contains("--context"), "{written}");
    assert!(!line(&written, LABELS[2]).contains(".tpl"), "{written}");
}

#[test]
fn fr_err_034_a_context_document_that_breaks_the_contract_names_the_key_path() {
    // The `65` row as the forty-third edition amends it: the key path and
    // what the contract expects there, and no file of the specification.
    let sandbox = Sandbox::new();
    sandbox.project("[core]\n");
    sandbox.write(&format!(".tpl/templates/{RESOLVES}.jinja"), "{{ 1 }}\n");
    sandbox.write(
        "shape.json",
        r#"{"schema_version":1,"source":"server","data":{}}"#,
    );

    let written = refused(
        &sandbox,
        &["render", RESOLVES, "--context", "shape.json"],
        65,
    );
    let cause = line(&written, LABELS[1]);

    assert!(
        cause.ends_with("and data.database is missing; a context document requires it"),
        "{cause}"
    );
    assert!(!cause.contains(".md"), "{cause}");
    assert_eq!(
        line(&written, LABELS[2]),
        "write a document that matches to a new file with: tpl -d <entry> schema dump > \
         context.json, then render with --context context.json"
    );
}

#[test]
fn r_01_a_map_where_an_array_is_expected_names_that_member_and_not_the_one_before() {
    // Finding R-01 of the re-audit for rmp #269: the decoder refuses an object
    // at its opening bracket, and the path named was the member before it.
    let sandbox = Sandbox::new();
    sandbox.project("[core]\n");
    sandbox.write(&format!(".tpl/templates/{RESOLVES}.jinja"), "{{ 1 }}\n");

    for collection in ["tables", "views", "routines"] {
        sandbox.write(
            "shape.json",
            &EMPTY_CONTEXT.replace(
                &format!("\"{collection}\":[]"),
                &format!("\"{collection}\": {{}}"),
            ),
        );

        let written = refused(
            &sandbox,
            &["render", RESOLVES, "--context", "shape.json"],
            65,
        );

        assert_eq!(
            line(&written, LABELS[1]),
            format!(
                "'shape.json' is well-formed JSON, and data.database.{collection} must be an \
                 array, but it is an object"
            )
        );
    }
}

#[test]
fn r_06_a_structural_fault_is_said_in_the_words_of_json_and_not_of_the_decoder() {
    let sandbox = Sandbox::new();
    sandbox.project("[core]\n");
    sandbox.write(&format!(".tpl/templates/{RESOLVES}.jinja"), "{{ 1 }}\n");

    let cases = [
        (
            "[]".to_owned(),
            "'shape.json' is well-formed JSON and is not a context document: the top level must \
             be an object with the keys schema_version, source and data",
        ),
        (
            EMPTY_CONTEXT.replace(r#""source":"server""#, r#""source":"bogus""#),
            "'shape.json' is well-formed JSON, and source must be one of server, cache, project, \
             binary, but it is 'bogus'",
        ),
    ];

    for (document, expected) in cases {
        sandbox.write("shape.json", &document);
        let written = refused(
            &sandbox,
            &["render", RESOLVES, "--context", "shape.json"],
            65,
        );

        assert_eq!(line(&written, LABELS[1]), expected);
    }

    // Empty standard input is named as what it is, and the hint keeps the
    // placeholder: standard input is no file a dump can be written to.
    let printed = sandbox.run_with_stdin(&["render", RESOLVES, "--context", "-"], b"");
    let written = stderr(&printed);

    assert_eq!(code(&printed), Some(65), "{written}");
    assert_eq!(
        line(&written, LABELS[0]),
        "the --context document on standard input is malformed"
    );
    assert_eq!(
        line(&written, LABELS[1]),
        "standard input is empty, and a context document is one JSON object"
    );
    assert!(
        line(&written, LABELS[2])
            .ends_with("> context.json, then render with --context context.json"),
        "{written}"
    );
}

#[test]
fn r_02_an_undefined_value_quotes_the_whole_expression_and_a_lookup_that_found_nothing_says_so() {
    let sandbox = Sandbox::new();
    sandbox.project("[core]\n");
    sandbox.write(CONTEXT, EMPTY_CONTEXT);
    sandbox.write(".tpl/templates/deep.jinja", "{{ database.nosuch.x }}");
    sandbox.write(
        ".tpl/templates/lookup.jinja",
        "{{ table(\"orders\").name }}",
    );

    let written = refused(&sandbox, &["render", "deep", "--context", CONTEXT], 65);
    assert!(
        // Z-01: the walk now names the step that found nothing.
        line(&written, LABELS[1])
            .contains("reads 'database.nosuch.x', and 'database' has no attribute 'nosuch'"),
        "{written}"
    );

    let written = refused(&sandbox, &["render", "lookup", "--context", CONTEXT], 65);
    assert!(
        line(&written, LABELS[1]).contains(
            "reads 'table(\"orders\").name', and table(\"orders\") found no table named 'orders'"
        ),
        "{written}"
    );
    // S-06 of #274: the names are the document's, which no tpl command lists.
    assert_eq!(
        line(&written, LABELS[2]),
        "list the tables of the --context document with jq, if it is installed: jq -r \
         '.data.database.tables[].name' context.json"
    );
}

#[test]
fn r_09_and_e_21_fail_puts_the_message_on_the_error_line_and_no_location_is_said_three_times() {
    let sandbox = Sandbox::new();
    sandbox.project("[core]\n");
    sandbox.write(CONTEXT, EMPTY_CONTEXT);
    sandbox.write(".tpl/templates/stop.jinja", "{{ fail(\"boom\") }}");
    sandbox.write(".tpl/templates/broken.jinja", "{% if %}");

    let written = refused(&sandbox, &["render", "stop", "--context", CONTEXT], 65);
    assert_eq!(
        line(&written, LABELS[0]),
        "template 'stop.jinja' called fail() at line 1, column 4: boom"
    );
    assert!(!line(&written, LABELS[1]).contains("(in "), "{written}");

    let written = refused(&sandbox, &["render", "broken", "--context", CONTEXT], 65);
    assert!(!line(&written, LABELS[1]).contains("(in "), "{written}");
}

// ------------------------------------------------------------ #274 ---------

#[test]
fn fr_rnd_041_direct_with_context_is_64_before_any_project_is_discovered() {
    // FR-RND-041: refused at step 1, so the empty sandbox is never searched
    // for a project, and neither the document nor a server is read.
    let sandbox = Sandbox::new();

    let written = refused(
        &sandbox,
        &["render", WHOLE, "--context", "missing.json", "--direct"],
        64,
    );
    assert_eq!(
        line(&written, LABELS[0]),
        "--direct cannot be used with --context"
    );
    assert_eq!(
        line(&written, LABELS[1]),
        "--context reads the context from a document and contacts no server; --direct demands \
         a read from the server"
    );
    assert_eq!(
        line(&written, LABELS[2]),
        "remove --direct to render from the document, or remove --context to read the server"
    );
    assert!(!written.contains("missing.json"), "{written}");

    // With an explicit -d as well, FR-RND-018 is the conflict reported.
    let written = refused(
        &sandbox,
        &[
            "-d",
            "shop",
            "render",
            WHOLE,
            "--context",
            CONTEXT,
            "--direct",
        ],
        64,
    );
    assert!(
        line(&written, LABELS[0]).contains("'--database'"),
        "{written}"
    );
    // S-08: neither flag is required, so the cause does not say one is.
    assert!(
        line(&written, LABELS[1]).contains("at most one"),
        "{written}"
    );
}

#[test]
fn s_07_an_include_without_its_extension_is_told_the_rule_and_the_name_it_meant() {
    let sandbox = Sandbox::new();
    sandbox.project("[core]\n");
    sandbox.write(CONTEXT, EMPTY_CONTEXT);
    sandbox.write(".tpl/templates/resolves.jinja", "fine\n");
    sandbox.write(".tpl/templates/inc.jinja", "{% include \"resolves\" %}");
    sandbox.write(
        ".tpl/templates/gone.jinja",
        "{% include \"nothing.jinja\" %}",
    );

    let written = refused(&sandbox, &["render", "inc", "--context", CONTEXT], 65);
    assert_eq!(
        line(&written, LABELS[2]),
        "did you mean 'resolves.jinja'? an {% include %} names the template file with its \
         extension, as in {% include \"example.jinja\" %}; list the templates with: tpl \
         template list"
    );

    // Z-03: the include wrote the extension, so the rule is not repeated.
    let written = refused(&sandbox, &["render", "gone", "--context", CONTEXT], 65);
    assert_eq!(
        line(&written, LABELS[2]),
        "list the templates with: tpl template list",
        "{written}"
    );
}

#[test]
fn s_12_and_s_13_fail_drops_the_engine_label_and_a_default_entry_needs_no_d_in_the_dump_hint() {
    let sandbox = Sandbox::new();
    sandbox.project("[core]\ndatabase = \"shop\"\n");
    sandbox.write(CONTEXT, EMPTY_CONTEXT);
    sandbox.write("bad.json", "{}");
    sandbox.write(".tpl/templates/stop.jinja", "{{ fail(\"boom\") }}");
    sandbox.write(".tpl/templates/resolves.jinja", "fine\n");

    let written = refused(&sandbox, &["render", "stop", "--context", CONTEXT], 65);
    assert!(line(&written, LABELS[1]).ends_with(": boom"), "{written}");
    assert!(!written.contains("invalid operation"), "{written}");

    // core.database names the entry, so the dump needs no -d.
    let written = refused(
        &sandbox,
        &["render", "resolves", "--context", "bad.json"],
        65,
    );
    assert!(
        line(&written, LABELS[2]).contains("with: tpl schema dump > context.json"),
        "{written}"
    );

    // Without it, the entry is the caller's to name.
    sandbox.project("[core]\n");
    let written = refused(
        &sandbox,
        &["render", "resolves", "--context", "bad.json"],
        65,
    );
    assert!(
        line(&written, LABELS[2]).contains("with: tpl -d <entry> schema dump > context.json"),
        "{written}"
    );
}

/// A procedure and a function both named `r`, for the ambiguity of
/// `FR-SCH-010` over a `--context` document.
const TWO_ROUTINES: &str = r#"[{"name":"r","kind":"procedure","parameters":[],"body_kind":"SQL","parameter_style":"SQL","is_deterministic":false,"sql_data_access":"SQL","security_type":"SQL","sql_mode":"SQL","comment":"SQL","definer":"SQL","character_set_client":"SQL","collation_connection":"SQL","database_collation":"SQL"},{"name":"r","kind":"function","parameters":[],"body_kind":"SQL","parameter_style":"SQL","is_deterministic":false,"sql_data_access":"SQL","security_type":"SQL","sql_mode":"SQL","comment":"SQL","definer":"SQL","character_set_client":"SQL","collation_connection":"SQL","database_collation":"SQL"}]"#;

#[test]
fn t_01_a_hint_that_rewrites_the_render_keeps_every_flag_it_was_given() {
    // Finding T-01 of the third re-audit of rmp #263: the rewritten command
    // dropped --context, -d and --set, and copied verbatim it read another
    // source. The whole invocation comes back, with the one token changed.
    let sandbox = Sandbox::new();
    sandbox.project("[core]\n");
    sandbox.write(&format!(".tpl/templates/{RESOLVES}.jinja"), "{{ 1 }}\n");
    sandbox.write(
        "rt.json",
        &EMPTY_CONTEXT.replace(r#""routines":[]"#, &format!(r#""routines":{TWO_ROUTINES}"#)),
    );

    let written = refused(
        &sandbox,
        &[
            "render",
            RESOLVES,
            "--context",
            "rt.json",
            "--routine",
            "r",
            "--set",
            "a=b",
        ],
        64,
    );
    assert_eq!(
        line(&written, LABELS[2]),
        format!(
            "name the kind you mean: tpl render {RESOLVES} --context rt.json --routine \
             procedure:r --set a=b, or tpl render {RESOLVES} --context rt.json --routine \
             function:r --set a=b"
        )
    );

    // A value the character set refuses is a placeholder, and the line says
    // what it stands for rather than dropping it.
    let written = refused(
        &sandbox,
        &[
            "render",
            RESOLVES,
            "--context",
            "rt.json",
            "--routine=FUNCTION:r",
            "--set",
            "t=a b",
        ],
        64,
    );
    assert_eq!(
        line(&written, LABELS[2]),
        format!(
            "write it as: tpl render {RESOLVES} --context rt.json --routine=function:r --set \
             t=<t>; replace <t> with the value you gave --set t"
        )
    );
}

#[test]
fn t_02_an_undefined_operand_of_a_filter_names_the_flag_that_defines_it() {
    // Finding T-02: `{{ table.name|pascal }}` without --table named no flag.
    let sandbox = Sandbox::new();
    sandbox.project("[core]\n");
    sandbox.write(CONTEXT, EMPTY_CONTEXT);
    sandbox.write(".tpl/templates/piped.jinja", "{{ table.name|pascal }}\n");
    sandbox.write(".tpl/templates/bare.jinja", "{{ nosuchvar }}\n");
    sandbox.write(".tpl/templates/typed.jinja", "{{ database|pascal }}\n");

    let written = refused(&sandbox, &["render", "piped", "--context", CONTEXT], 65);
    assert!(
        line(&written, LABELS[1]).contains(
            "reads 'table.name', which is not defined in this render; the template engine \
             reports: the filter 'pascal' accepts a string, and was given an undefined value"
        ),
        "{written}"
    );
    assert_eq!(
        line(&written, LABELS[2]),
        "'table' exists only when the render names one: add --table <name> to the tpl render \
         command"
    );

    // T-09: a bare name the render never defines is answered with the names it
    // does define.
    let written = refused(&sandbox, &["render", "bare", "--context", CONTEXT], 65);
    assert_eq!(
        line(&written, LABELS[2]),
        "'nosuchvar' is not a variable of this render: a template sees database, vars, tpl and \
         now, and table, view or routine when --table, --view or --routine names one; list them \
         with: tpl help render"
    );

    // T-07: the type is named in the author's words, not the engine's.
    let written = refused(&sandbox, &["render", "typed", "--context", CONTEXT], 65);
    assert!(
        line(&written, LABELS[1]).contains("and was given an object"),
        "{written}"
    );
}

/// One table, `orders`, with one column, as a `--context` document holds it.
const ORDERS: &str = r#"[{"name":"orders","table_type":"BASE TABLE","engine":"InnoDB","collation":null,"comment":"","columns":[{"name":"id","table_name":"orders","position":1,"column_type":"int(11)","nullable":false,"default":null,"comment":"","auto_increment":true,"invisible":false,"generated":null,"on_update":null}],"indexes":[],"foreign_keys":[],"referenced_by":[],"triggers":[],"check_constraints":[]}]"#;

#[test]
fn y_01_an_undefined_member_of_a_bound_object_points_at_the_template_and_not_the_flag() {
    // Finding Y-01 of the eighth re-audit of rmp #263: with --table given, a
    // misspelt attribute or an index past the end asked for --table.
    let sandbox = Sandbox::new();
    sandbox.project("[core]\n");
    sandbox.write(
        CONTEXT,
        &EMPTY_CONTEXT
            .replace(r#""tables":[]"#, &format!(r#""tables":{ORDERS}"#))
            .replace(
                r#""routines":[]"#,
                &format!(
                    r#""routines":{}"#,
                    TWO_ROUTINES.replace(r#""kind":"procedure""#, r#""kind":"PROCEDURE""#)
                ),
            ),
    );
    sandbox.write(".tpl/templates/nme.jinja", "{{ table.nme }}\n");
    sandbox.write(
        ".tpl/templates/index.jinja",
        "{{ table.columns[5].name }}\n",
    );
    sandbox.write(".tpl/templates/deep.jinja", "{{ table.columns[0].nam }}\n");
    sandbox.write(".tpl/templates/body.jinja", "{{ routine.qqqqqqqq }}\n");

    let table = ["--context", CONTEXT, "--table", "orders"];
    let routine = ["--context", CONTEXT, "--routine", "procedure:r"];
    for (template, flags, cause, hint) in [
        (
            "nme",
            &table,
            "reads 'table.nme', and 'table' has no attribute 'nme'",
            "did you mean 'name'? 'table' has no attribute 'nme'; its attributes are name, \
             table_type, engine, collation, comment, columns, indexes, primary_key, foreign_keys, \
             referenced_by, triggers, check_constraints; correct the template, then print the \
             template's source with: tpl template show nme.jinja",
        ),
        (
            "index",
            &table,
            "reads 'table.columns[5].name', and 'table.columns' has 1 item, so it has no item [5]",
            "'table.columns' has 1 item, at index 0, so [5] reads nothing; correct the template, \
             then print the template's source with: tpl template show index.jinja",
        ),
        (
            "deep",
            &table,
            "reads 'table.columns[0].nam', and 'table.columns[0]' has no attribute 'nam'",
            "did you mean 'name'? 'table.columns[0]' has no attribute 'nam'",
        ),
        (
            "body",
            &routine,
            "reads 'routine.qqqqqqqq', and 'routine' has no attribute 'qqqqqqqq'",
            "'routine' has no attribute 'qqqqqqqq'; its attributes are name, kind",
        ),
    ] {
        let mut arguments = vec!["render", template];
        arguments.extend_from_slice(flags);
        let written = refused(&sandbox, &arguments, 65);
        assert!(line(&written, LABELS[1]).contains(cause), "{written}");
        let given = line(&written, LABELS[2]);
        assert!(given.starts_with(hint), "{written}");
        assert!(!given.contains("add --"), "{written}");

        // The command the hint ends with prints the template.
        let (_, command) = given
            .rsplit_once(": ")
            .expect("the hint ends with a command");
        let words: Vec<&str> = command.split_whitespace().skip(1).collect();
        assert_eq!(words[..2], ["template", "show"], "{given}");
        succeeds(&sandbox, &words);
    }

    // Without --table, the flag is still what the hint asks for.
    let written = refused(&sandbox, &["render", "nme", "--context", CONTEXT], 65);
    assert_eq!(
        line(&written, LABELS[2]),
        "'table' exists only when the render names one: add --table <name> to the tpl render \
         command"
    );
}

#[test]
fn z_01_an_undefined_member_of_any_context_variable_names_the_step_that_found_nothing() {
    // Finding Z-01 of the ninth re-audit of rmp #263: `database`, `tpl` and
    // `now` got none of the Y-01 treatment.
    let sandbox = Sandbox::new();
    sandbox.project("[core]\n");
    sandbox.write(
        CONTEXT,
        &EMPTY_CONTEXT.replace(r#""tables":[]"#, &format!(r#""tables":{ORDERS}"#)),
    );
    sandbox.write(".tpl/templates/db.jinja", "{{ database.nme }}\n");
    sandbox.write(
        ".tpl/templates/past.jinja",
        "{{ database.tables[3].name }}\n",
    );
    sandbox.write(
        ".tpl/templates/server.jinja",
        "{{ database.server.vrsion }}\n",
    );
    sandbox.write(".tpl/templates/tv.jinja", "{{ tpl.vrsion }}\n");
    sandbox.write(".tpl/templates/when.jinja", "{{ now.year }}\n");

    for (template, cause, hint) in [
        (
            "db",
            "reads 'database.nme', and 'database' has no attribute 'nme'",
            "did you mean 'name'? 'database' has no attribute 'nme'; its attributes are name, \
             charset, collation, server, tables, views, routines; correct the template, then \
             print the template's source with: tpl template show db.jinja",
        ),
        (
            "past",
            "reads 'database.tables[3].name', and 'database.tables' has 1 item, so it has no \
             item [3]",
            "'database.tables' has 1 item, at index 0, so [3] reads nothing",
        ),
        (
            "server",
            "and 'database.server' has no attribute 'vrsion'",
            "did you mean 'version'? 'database.server' has no attribute 'vrsion'",
        ),
        (
            "tv",
            "and 'tpl' has no attribute 'vrsion'",
            "did you mean 'version'? 'tpl' has no attribute 'vrsion'; its attributes are \
             version;",
        ),
        (
            "when",
            "and 'now' is a string, which has no attribute 'year'",
            "'now' is a string with no attribute 'year'; correct the template",
        ),
    ] {
        let written = refused(&sandbox, &["render", template, "--context", CONTEXT], 65);
        assert!(line(&written, LABELS[1]).contains(cause), "{written}");
        let given = line(&written, LABELS[2]);
        assert!(given.starts_with(hint), "{written}");
        assert!(!given.contains("add --"), "{written}");
        assert!(
            given.ends_with(&format!("tpl template show {template}.jinja")),
            "{written}"
        );
    }
}

#[test]
fn z_02_a_vars_key_near_a_set_key_is_named_and_never_hidden_behind_a_new_set() {
    // Finding Z-02 of the ninth re-audit of rmp #263: the hint asked for a
    // new --set whether the typo sat in the template or in the flag.
    let sandbox = Sandbox::new();
    sandbox.project("[core]\n");
    sandbox.write(CONTEXT, EMPTY_CONTEXT);
    sandbox.write(".tpl/templates/titl.jinja", "{{ vars.titl }}\n");
    sandbox.write(".tpl/templates/title.jinja", "{{ vars.title }}\n");

    for (template, set, cause, hint) in [
        (
            "titl",
            "title=x",
            "reads 'vars.titl', and no --set gave the key 'titl'",
            "did you mean 'title'? --set gave title; correct the key in the template or in \
             --set so the two match",
        ),
        (
            "title",
            "titel=x",
            "reads 'vars.title', and no --set gave the key 'title'",
            "did you mean 'titel'? --set gave titel; correct the key in the template or in \
             --set so the two match",
        ),
        (
            "title",
            "author=x",
            "no --set gave the key 'title'",
            "--set gave author, and none is near 'title': add --set title=<value> to the tpl \
             render command",
        ),
    ] {
        let written = refused(
            &sandbox,
            &["render", template, "--context", CONTEXT, "--set", set],
            65,
        );
        assert!(line(&written, LABELS[1]).contains(cause), "{written}");
        assert!(line(&written, LABELS[2]).starts_with(hint), "{written}");
    }

    // With no --set at all, the key is the one to add.
    let written = refused(&sandbox, &["render", "title", "--context", CONTEXT], 65);
    assert!(
        line(&written, LABELS[2]).starts_with("'vars.title' is set with --set: add --set title="),
        "{written}"
    );
}

#[test]
fn z_03_an_include_with_its_extension_gets_the_nearest_template_and_not_the_rule() {
    let sandbox = Sandbox::new();
    sandbox.project("[core]\n");
    sandbox.write(CONTEXT, EMPTY_CONTEXT);
    sandbox.write(".tpl/templates/t/ok.jinja", "fine\n");
    sandbox.write(".tpl/templates/near.jinja", "{% include \"t/oj.jinja\" %}");
    sandbox.write(".tpl/templates/bare.jinja", "{% include \"t/oj\" %}");

    let written = refused(&sandbox, &["render", "near", "--context", CONTEXT], 65);
    assert_eq!(
        line(&written, LABELS[2]),
        "did you mean 't/ok.jinja'? list the templates with: tpl template list"
    );

    // Without the extension, the rule stays, and so does the suggestion.
    let written = refused(&sandbox, &["render", "bare", "--context", CONTEXT], 65);
    assert_eq!(
        line(&written, LABELS[2]),
        "did you mean 't/ok.jinja'? an {% include %} names the template file with its \
         extension, as in {% include \"example.jinja\" %}; list the templates with: tpl \
         template list"
    );
}

#[test]
fn z_04_a_misspelt_root_suggests_the_bound_variable_and_a_loop_variable_does_not() {
    let sandbox = Sandbox::new();
    sandbox.project("[core]\n");
    sandbox.write(
        CONTEXT,
        &EMPTY_CONTEXT.replace(r#""tables":[]"#, &format!(r#""tables":{ORDERS}"#)),
    );
    sandbox.write(".tpl/templates/top.jinja", "{{ tabel.name }}\n");
    sandbox.write(
        ".tpl/templates/row.jinja",
        "{% for row in table.columns %}{{ row.nme }}{% endfor %}\n",
    );

    let table = ["--context", CONTEXT, "--table", "orders"];
    let written = refused(&sandbox, &[&["render", "top"][..], &table].concat(), 65);
    assert!(
        line(&written, LABELS[1])
            .contains("reads 'tabel.name', and 'tabel' is no variable tpl binds in this render"),
        "{written}"
    );
    assert_eq!(
        line(&written, LABELS[2]),
        "did you mean 'table'? 'tabel' is no variable tpl binds in this render; correct the \
         template, then print the template's source with: tpl template show top.jinja"
    );

    // `row` is the template's own, one edit from `now`: no suggestion.
    let written = refused(&sandbox, &[&["render", "row"][..], &table].concat(), 65);
    assert!(!written.contains("did you mean"), "{written}");
}

#[test]
fn aa_03_a_misspelt_object_variable_without_its_flag_names_it_and_the_flag() {
    // Finding AA-03 of the tenth re-audit of rmp #263: without --table,
    // `tbl.name` was steered to `tpl`, and `tabel.name` got no suggestion.
    let sandbox = Sandbox::new();
    sandbox.project("[core]\n");
    sandbox.write(
        CONTEXT,
        &EMPTY_CONTEXT.replace(r#""tables":[]"#, &format!(r#""tables":{ORDERS}"#)),
    );
    for (template, source) in [
        ("tbl", "{{ tbl.name }}\n"),
        ("tabel", "{{ tabel.name }}\n"),
        ("viw", "{{ viw.name }}\n"),
        ("rotine", "{{ rotine.name }}\n"),
        ("both", "{{ tbl.version }}\n"),
    ] {
        sandbox.write(&format!(".tpl/templates/{template}.jinja"), source);
    }

    for (template, flag) in [
        ("tbl", "table"),
        ("tabel", "table"),
        ("viw", "view"),
        ("rotine", "routine"),
    ] {
        let written = refused(&sandbox, &["render", template, "--context", CONTEXT], 65);
        assert_eq!(
            line(&written, LABELS[2]),
            format!(
                "did you mean '{flag}'? '{flag}' exists only when the render names one: correct \
                 the template and add --{flag} <name> to the tpl render command"
            ),
            "{written}"
        );
        assert!(!written.contains("'tpl'"), "{written}");
    }

    // `tpl` holds `version`, so both are candidates, and the line says which
    // one needs its flag.
    let written = refused(&sandbox, &["render", "both", "--context", CONTEXT], 65);
    assert_eq!(
        line(&written, LABELS[2]),
        "did you mean 'tpl' or 'table'? 'table' exists only when the render names one with \
         --table <name>; correct the template, and if you mean 'table', add --table <name> to \
         the tpl render command"
    );

    // With --table, `table` is bound, and `tpl`, which has no `name`, is not
    // offered.
    let written = refused(
        &sandbox,
        &["render", "tbl", "--context", CONTEXT, "--table", "orders"],
        65,
    );
    assert_eq!(
        line(&written, LABELS[2]),
        "did you mean 'table'? 'tbl' is no variable tpl binds in this render; correct the \
         template, then print the template's source with: tpl template show tbl.jinja"
    );
}

#[test]
fn aa_04_a_vars_the_template_binds_is_not_blamed_on_set_in_the_cause() {
    // Finding AA-04 of the tenth re-audit of rmp #263.
    let sandbox = Sandbox::new();
    sandbox.project("[core]\n");
    sandbox.write(CONTEXT, EMPTY_CONTEXT);
    sandbox.write(
        ".tpl/templates/shadow.jinja",
        "{% set vars = {\"a\": 1} %}{{ vars.titl }}\n",
    );

    let written = refused(&sandbox, &["render", "shadow", "--context", CONTEXT], 65);
    let cause = line(&written, LABELS[1]);
    assert!(
        cause.contains(
            "'vars' is a variable the template binds, which holds the template's own value, not \
             the render's, and reading 'titl' of it found nothing"
        ),
        "{written}"
    );
    assert!(!cause.contains("--set"), "{written}");
    assert!(!line(&written, LABELS[2]).contains("--set"), "{written}");
}

#[test]
fn a_context_variable_name_the_template_binds_is_described_as_the_templates_own() {
    // A loop variable named `table` is not the render's `table`: the walk
    // describes what the loop yields, and never asks for --table.
    let sandbox = Sandbox::new();
    sandbox.project("[core]\n");
    sandbox.write(
        CONTEXT,
        &EMPTY_CONTEXT.replace(r#""tables":[]"#, &format!(r#""tables":{ORDERS}"#)),
    );
    sandbox.write(
        ".tpl/templates/loop.jinja",
        "{% for table in database.tables %}{{ table.nme }}{% endfor %}\n",
    );
    sandbox.write(
        ".tpl/templates/param.jinja",
        "{% macro m(table) %}{{ table.nme }}{% endmacro %}{{ m(database) }}\n",
    );

    for flags in [
        &["--context", CONTEXT][..],
        &["--context", CONTEXT, "--table", "orders"][..],
    ] {
        let written = refused(&sandbox, &[&["render", "loop"][..], flags].concat(), 65);
        assert!(
            line(&written, LABELS[1]).contains(
                "reads 'table.nme', and 'table' is a loop variable of the template, which holds \
                 the template's own value, not the render's, and 'table' has no attribute 'nme'"
            ),
            "{written}"
        );
        let given = line(&written, LABELS[2]);
        assert!(
            given.starts_with(
                "did you mean 'name'? 'table' is a loop variable of the template, not the \
                 render's 'table': 'table' has no attribute 'nme'; its attributes are name, \
                 table_type, engine,"
            ),
            "{written}"
        );
        assert!(!given.contains("add --"), "{written}");
        assert!(given.ends_with("tpl template show loop.jinja"), "{written}");

        // A macro's parameter holds a value only the call knows: no list.
        let written = refused(&sandbox, &[&["render", "param"][..], flags].concat(), 65);
        assert!(
            line(&written, LABELS[1]).contains(
                "'table' is a variable the template binds, which holds the template's own value, \
                 not the render's, and reading 'nme' of it found nothing"
            ),
            "{written}"
        );
        assert_eq!(
            line(&written, LABELS[2]),
            "'table' is a variable the template binds, not the render's 'table': reading 'nme' \
             of it found nothing; correct the template, then print the template's source with: \
             tpl template show param.jinja"
        );
    }
}

#[test]
fn t_07_a_parse_position_is_one_based_at_the_start_of_a_line() {
    // The decoder reports column 0 for an end of input that begins a line.
    let sandbox = Sandbox::new();
    sandbox.project("[core]\n");
    sandbox.write(&format!(".tpl/templates/{RESOLVES}.jinja"), "{{ 1 }}\n");

    let printed = sandbox.run_with_stdin(&["render", RESOLVES, "--context", "-"], b"{\n");
    let written = stderr(&printed);
    assert_eq!(code(&printed), Some(65), "{written}");
    assert!(
        line(&written, LABELS[1]).ends_with("the parser stopped at line 2, column 1"),
        "{written}"
    );
}

#[test]
fn u_08_and_u_09_render_refusals_speak_plainly_and_never_overwrite_the_callers_file() {
    let sandbox = Sandbox::new();
    sandbox.project("[core]\ndatabase = \"shop\"\n");
    sandbox.write(".tpl/templates/resolves.jinja", "fine\n");

    // U-08: the key set in words, and the dotted clause only for a dot.
    let written = refused(
        &sandbox,
        &["render", "resolves", "--context", "x.json", "--set", "1a=b"],
        64,
    );
    let cause = line(&written, LABELS[1]);
    assert!(
        cause.ends_with(
            "a key of letters, digits and _, not starting with a digit, then '=', then the value"
        ),
        "{cause}"
    );
    let written = refused(
        &sandbox,
        &[
            "render",
            "resolves",
            "--context",
            "x.json",
            "--set",
            "a.b=c",
        ],
        64,
    );
    assert!(
        line(&written, LABELS[1]).ends_with("a dotted key is refused rather than split"),
        "{written}"
    );

    // U-08: what the filesystem returned, without the OS error number.
    let written = refused(
        &sandbox,
        &["render", "resolves", "--context", "absent.json"],
        74,
    );
    assert!(!written.contains("(os error"), "{written}");

    // U-09: the dump goes to a new file, and never to the one given.
    sandbox.write("context.json", "{}");
    let written = refused(
        &sandbox,
        &["render", "resolves", "--context", "context.json"],
        65,
    );
    assert_eq!(
        line(&written, LABELS[2]),
        "write a document that matches to a new file with: tpl schema dump > context.new.json, \
         then render with --context context.new.json"
    );
}

#[test]
fn ab_03_an_attribute_under_a_vars_key_is_never_answered_with_a_set() {
    // Finding AB-03 of the eleventh re-audit of rmp #263: `vars.a.b` was told
    // to add `--set a=<value>`, which gives a string with no attribute `b`.
    let sandbox = Sandbox::new();
    sandbox.project("[core]\n");
    sandbox.write(CONTEXT, EMPTY_CONTEXT);
    sandbox.write(".tpl/templates/deep.jinja", "{{ vars.a.b }}\n");

    for set in [&[][..], &["--set", "b=x"][..], &["--set", "aa=x"][..]] {
        let written = refused(
            &sandbox,
            &[&["render", "deep", "--context", CONTEXT][..], set].concat(),
            65,
        );
        assert_eq!(
            line(&written, LABELS[2]),
            "the template reads an attribute of 'vars.a', and --set gives only top-level keys \
             of vars, each a string with no attributes, so no --set can supply it; correct the \
             template, then print the template's source with: tpl template show deep.jinja",
            "{set:?}: {written}"
        );
        assert!(!written.contains("add --set"), "{set:?}: {written}");
    }

    // The key is given: its value is a string, and the hint was never a --set.
    let written = refused(
        &sandbox,
        &["render", "deep", "--context", CONTEXT, "--set", "a=x"],
        65,
    );
    assert!(!written.contains("add --set"), "{written}");
}

#[test]
fn ab_04_an_object_variable_the_render_did_not_bind_names_the_flag_to_replace() {
    // Finding AB-04 of the eleventh re-audit of rmp #263: `view` in a render
    // given --table was told to add --view, which --table excludes.
    let sandbox = Sandbox::new();
    sandbox.project("[core]\n");
    sandbox.write(
        CONTEXT,
        &EMPTY_CONTEXT.replace(r#""tables":[]"#, &format!(r#""tables":{ORDERS}"#)),
    );
    sandbox.write(".tpl/templates/vw.jinja", "{{ view.nme }}\n");
    sandbox.write(".tpl/templates/rt.jinja", "{{ routine.name }}\n");

    for (template, root) in [("vw", "view"), ("rt", "routine")] {
        let written = refused(
            &sandbox,
            &[
                "render",
                template,
                "--context",
                CONTEXT,
                "--table",
                "orders",
            ],
            65,
        );
        assert!(
            line(&written, LABELS[1]).contains(&format!(
                "and '{root}' is not bound, because this render names a table with --table"
            )),
            "{written}"
        );
        assert_eq!(
            line(&written, LABELS[2]),
            format!(
                "this render names a table with --table, so '{root}' is not bound: read 'table' \
                 in the template, or replace --table with --{root} <name> in the tpl render \
                 command"
            ),
            "{written}"
        );
        assert!(!written.contains(&format!("add --{root}")), "{written}");
    }

    // Without an object flag, the hint still asks for the flag.
    let written = refused(&sandbox, &["render", "vw", "--context", CONTEXT], 65);
    assert_eq!(
        line(&written, LABELS[2]),
        "'view' exists only when the render names one: add --view <name> to the tpl render \
         command"
    );
}

#[test]
fn a_string_subscript_of_a_bound_object_is_answered_as_the_attribute_is() {
    // Recorded by the eleventh re-audit of rmp #263, fixed under #286:
    // `table["nme"]` got "a later step of the expression is not" and no
    // attribute list, where `table.nme` gets both.
    let sandbox = Sandbox::new();
    sandbox.project("[core]\n");
    sandbox.write(
        CONTEXT,
        &EMPTY_CONTEXT.replace(r#""tables":[]"#, &format!(r#""tables":{ORDERS}"#)),
    );
    for (template, source) in [
        ("dot", "{{ table.nme }}\n"),
        ("dq", "{{ table[\"nme\"] }}\n"),
        ("sq", "{{ table['nme'] }}\n"),
        ("deep", "{{ table.columns[0][\"nam\"] }}\n"),
    ] {
        sandbox.write(&format!(".tpl/templates/{template}.jinja"), source);
    }
    let table = ["--context", CONTEXT, "--table", "orders"];

    let dotted = refused(&sandbox, &[&["render", "dot"][..], &table].concat(), 65);
    let expected = line(&dotted, LABELS[2]).replace("dot.jinja", "{}");
    assert!(expected.starts_with("did you mean 'name'?"), "{dotted}");

    for template in ["dq", "sq"] {
        let written = refused(&sandbox, &[&["render", template][..], &table].concat(), 65);
        assert_eq!(
            line(&written, LABELS[2]),
            expected.replace("{}", &format!("{template}.jinja")),
            "{written}"
        );
        assert!(
            line(&written, LABELS[1]).contains("and 'table' has no attribute 'nme'"),
            "{written}"
        );
        assert!(!written.contains("a later step"), "{written}");
    }

    let written = refused(&sandbox, &[&["render", "deep"][..], &table].concat(), 65);
    assert!(
        line(&written, LABELS[2])
            .starts_with("did you mean 'name'? 'table.columns[0]' has no attribute 'nam'"),
        "{written}"
    );
}
