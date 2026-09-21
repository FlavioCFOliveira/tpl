//! The observations `NFR-PERF-007` makes from outside the process.
//!
//! That requirement forbids verifying a requirement of form by reading the
//! source, and `BR-SRV-003` says why: a promise about what a process sends that
//! can only be checked by reading that process's own source is not a promise a
//! caller can rely on. It fixes **four** instruments, and this file is where
//! the suite reaches them.
//!
//! | Instrument | What it observes | Targets it is credited on | Where it lives |
//! |---|---|---|---|
//! | The server's statement record | The statements the server receives | all four | `observe.sh statements`, wrapped by [`fixture`] |
//! | The server's connection record | The connections the server accepts | all four | `observe.sh connections`, wrapped by [`fixture`] |
//! | A syscall trace of the process | The files the process opens | **the two Linux targets** | `observe.sh opens`, wrapped by [`fixture`] |
//! | A differential run | The invocation's exit code, its stdout bytes, and the artefacts it leaves on disk | all four | [`differential`], which is where it was built |
//!
//! # What is observed here
//!
//! Eight requirements are observed **here**. The first two are about commands
//! that must reach no server at all; the rest are about what an invocation
//! that does reach one sends it, and about what a failed render leaves on the
//! stream a caller reads:
//!
//! | Requirement | The property |
//! |---|---|
//! | `NFR-PERF-005` | The commands of `FR-PROJ-025` open no connection, perform no project discovery, and read no configuration file |
//! | `NFR-PERF-006` | A command that requires no catalogue data opens no connection |
//! | `FR-RND-022` | `tpl render --context` opens no connection, from a file and from standard input alike |
//! | `FR-SRV-012` | The server receives the four kinds of the closed list of `FR-SRV-006` and no fifth, the three connection-start statements in the order `FR-SRV-042` fixes |
//! | `FR-SRV-013` | The read-back of `FR-SRV-009` is issued exactly once, in the one spelling every series of the window carries |
//! | `FR-SRV-014`, `NFR-PERF-004` | One invocation opens at most one connection |
//! | `NFR-PERF-002` | The catalogue statement count of a read that names one object does not grow with the database |
//! | `FR-RND-034`, `FR-SEM-020` | A render that fails leaves at most one incomplete result on the process's own stdout |
//!
//! `NFR-PERF-001` is observed here, beside `NFR-PERF-002` and over the same two
//! databases, and again in
//! [`schema_and_cache`](../schema_and_cache/index.html); `NFR-PERF-003` is
//! observed there alone, because the read it is about is the one the cache
//! serves.
//!
//! # Every negative assertion carries a control
//!
//! On the same terms as
//! [`invocation_surface`](../invocation_surface/index.html): an absence is
//! worth nothing until the same instrument is shown to observe the presence.
//!
//! | Negative assertion | Its control |
//! |---|---|
//! | No command opened a connection | One TCP connection is opened, and the same connection record counts it |
//! | The server received no statement from anything under test | The same dump, with `--all`, is shown to hold the observer's own |
//! | The discovery clause's differential run | The same comparison over a command that **does** discover, whose two outcomes differ |
//! | The configuration clause's differential run | The same comparison over a command that **does** read `.tpl/.cfg`, which the arrangement trips |
//!
//! # The fixture, and what happens without one
//!
//! Every server-dependent body here is gated on `scripts/mariadb/status.sh`,
//! whose exit code is three-valued: `0` runs the body, `1` skips it with a
//! printed reason, and `2` — half a fixture — fails the run rather than
//! skipping over it. The bodies that need a server are driven **once per series
//! of `FR-SRV-015`**, which `FR-SRV-029` requires of a test of cross-series
//! behaviour and which costs nothing to hold to here.

#[path = "support/differential.rs"]
mod differential;
#[path = "support/fixture.rs"]
mod fixture;
#[path = "support/sandbox.rs"]
mod sandbox;

use differential::Outcome;
use fixture::Gate;
use sandbox::Sandbox;

/// A `.tpl/.cfg` the reader accepts.
const ACCEPTED: &str = "[core]\n";

/// A `.tpl/.cfg` the reader refuses: `databse` is outside the enumerated key
/// space, which `FR-CONF-034` makes a `78` wherever it appears in the file.
const REFUSED: &str = "[core]\ndatabse = \"shop\"\n";

/// The `78` of `FR-PROJ-006` and of `FR-CONF-034`.
const CONFIG: i32 = 78;

/// The template `FR-PROJ-021` obliges `tpl init` to write, named as
/// `FR-TMPL-006` resolves it — without the extension.
const EXAMPLE: &str = "example";

/// The `--context` document the `tpl render` entry reads, relative to the
/// sandbox the population runs in (`FR-RND-016`).
const CONTEXT: &str = "context.json";

/// The table that entry binds (`FR-RND-003`).
///
/// It is a table of the fixture's `freight`, because the document is the one
/// `tpl schema dump` produced against a fixture server.
const BOUND: &str = "charge";

/// Every command of `FR-PROJ-025`, every `cfg` subcommand but `database test`,
/// the four `template` subcommands and `tpl render --context`, in an order
/// that leaves each of them a project, a template and an entry to work on.
///
/// `cfg database test` is excluded because it is the one `cfg` leaf still
/// unwritten, and `NFR-PERF-006` excludes it in its own text.
///
/// **The five entries that follow `tpl init` follow it because they need what
/// it creates.** `FR-PROJ-017` gives a fresh project
/// `.tpl/templates/example.jinja`, which is the template `show`, `path` and the
/// render are given and one of the two `check` walks. The render needs a second
/// thing the sandbox does not start with — the `--context` document of
/// `FR-RND-016` — and the body below writes it there before the instruments are
/// armed.
///
/// This is what gives the no-connection claim of `FR-TMPL-003` the **server's
/// own connection record**, in place of the exit-code proxy
/// `tests/template_commands.rs` uses for it where no fixture is required.
const POPULATION: [&[&str]; 23] = [
    &["init"],
    &["template", "list"],
    &["template", "show", EXAMPLE],
    &["template", "check"],
    &["template", "path", EXAMPLE],
    &["render", EXAMPLE, "--context", CONTEXT, "--table", BOUND],
    &["help"],
    &["help", "cfg"],
    &["help", "--format", "json"],
    &["--help"],
    &["cfg", "database", "add", "-h"],
    &["version"],
    &["-V"],
    &["--version"],
    &[
        "cfg",
        "database",
        "add",
        "shop",
        "--host",
        "db.example.com",
        "--user",
        "reader",
        "--database",
        "shop",
    ],
    &["cfg", "database", "list"],
    &["cfg", "database", "show", "shop"],
    &["cfg", "database", "update", "shop", "--port", "3307"],
    &["cfg", "set", "core.database", "shop"],
    &["cfg", "get", "core.database"],
    &["cfg", "list"],
    &["cfg", "unset", "core.database"],
    &["cfg", "database", "remove", "shop"],
];

/// The forms of `help` and of `version` that `FR-PROJ-025` names, each of which
/// exits `0` and writes to stdout wherever it is invoked.
const ANSWERED_ANYWHERE: [&[&str]; 7] = [
    &["help"],
    &["help", "cfg"],
    &["help", "--format", "json"],
    &["--help"],
    &["cfg", "database", "add", "-h"],
    &["version"],
    &["-V"],
];

// --------------------------------------------------------------- the gate ---

#[test]
fn the_gate_maps_its_three_exit_codes_and_needs_no_fixture() {
    // `scripts/mariadb/status.sh` documents three values, and the third is the
    // one that earns its keep: a runner that treated "no fixture" and "half a
    // fixture" alike would either skip silently over a real failure or fail the
    // build of every contributor who has no Docker.
    //
    // The mapping is a free function over an exit code precisely so that it can
    // be fed all three here, with no fixture in reach and nothing launched.
    assert_eq!(fixture::gate_of(Some(0)), Gate::Ready);
    assert_eq!(fixture::gate_of(Some(1)), Gate::Absent);
    assert_eq!(fixture::gate_of(Some(2)), Gate::Broken(Some(2)));

    // A code the header does not define, and a gate that was signalled rather
    // than exited. Neither is an absent fixture, so neither is skipped over.
    assert_eq!(fixture::gate_of(Some(127)), Gate::Broken(Some(127)));
    assert_eq!(fixture::gate_of(None), Gate::Broken(None));
}

#[test]
fn fr_srv_029_one_body_is_driven_once_per_series_of_fr_srv_015() {
    // FR-SRV-029 requires a test body to run against every series of
    // FR-SRV-015, so the suite drives one body once per series rather than once
    // per server: series.env states that the --skip-ssl record FR-CONF-038
    // obliges is not a fifth series.
    //
    // The body below is the smallest one that establishes the drive happened:
    // each series answered at the address the inventory gave for it.
    let _guard = fixture::exclusive();
    let Some(series) =
        fixture::series("fr_srv_029_one_body_is_driven_once_per_series_of_fr_srv_015")
    else {
        return;
    };

    let mut driven: Vec<&str> = Vec::new();

    for server in series {
        fixture::connect_once(server);
        driven.push(server.name());
    }

    assert!(!driven.is_empty(), "the driver drove no series at all");

    let mut distinct = driven.clone();
    distinct.sort_unstable();
    distinct.dedup();
    assert_eq!(
        distinct.len(),
        driven.len(),
        "the driver repeated a series: {driven:?}"
    );

    // The count is not asserted against a number, because FR-SRV-017 forbids
    // fixing the size of the set. What is asserted is the property that number
    // would have stood for: the fixture holds more servers than it holds
    // series, and the driver drove the series.
    assert!(
        driven.len() < fixture::servers_that_answered(),
        "the driver drove {} of the {} servers that answered, so it drove the \
         fixture rather than the series of FR-SRV-015",
        driven.len(),
        fixture::servers_that_answered()
    );
}

// ------------------------------------------ NFR-PERF-005 and NFR-PERF-006 ---

#[test]
fn nfr_perf_007_the_commands_that_need_no_catalogue_open_no_connection() {
    // NFR-PERF-005 requires every command of FR-PROJ-025 to open no connection,
    // and NFR-PERF-006 requires the same of every command that needs no
    // catalogue data — which, among the commands that exist today, is every cfg
    // subcommand but `database test`. One observation of the connections a
    // server accepts discharges both, the way one observation discharges rows 4
    // and 9 of the register.
    //
    // The connection clause is verified on every target of NFR-PERF-018, from
    // the server side, and this is that observation.
    let _guard = fixture::exclusive();
    let Some(series) =
        fixture::series("nfr_perf_007_the_commands_that_need_no_catalogue_open_no_connection")
    else {
        return;
    };

    for server in series {
        // The control, and it comes first: an assertion that no connection was
        // opened is worth nothing until this same record is shown to count one
        // that was. It is made before the statement window opens, because an
        // unauthenticated connection raises the counter and writes no row, and
        // the two instruments are kept independent.
        let control =
            fixture::connections_attributable_to(server, || fixture::connect_once(server));
        assert_eq!(
            control,
            1,
            "the connection record of {} did not count a connection that was made",
            server.name()
        );

        // The `--context` document the render entry of POPULATION is given.
        // It is what `tpl schema dump` produced against this server, so the
        // document contract is stated once — in the product — rather than a
        // second time here, where a later edition of
        // `specification/context-document.md` would leave it silently wrong.
        //
        // It is produced **at this point**: after the control, which is why the
        // one connection it opens is outside the bracket below, and before the
        // statement record is armed, which truncates the log it would otherwise
        // have left rows in.
        let source = project(server);
        let dumped = succeeds(&source, &["schema", "dump"]);

        let sandbox = Sandbox::new();
        sandbox.write(CONTEXT, &dumped);

        fixture::statements_on(server);
        let attributable = fixture::connections_attributable_to(server, || {
            for arguments in POPULATION {
                let printed = std::process::Command::new(env!("CARGO_BIN_EXE_tpl"))
                    .env_clear()
                    .current_dir(sandbox.root())
                    .args(arguments)
                    .output()
                    .expect("the binary under test runs");

                assert_eq!(
                    printed.status.code(),
                    Some(0),
                    "tpl {} did not exit 0: {}",
                    arguments.join(" "),
                    String::from_utf8_lossy(&printed.stderr)
                );
            }
        });
        fixture::statements_off(server);

        // The statement record corroborates the connection record over the same
        // window, and gives the count a second and independent form: a
        // connection that authenticated would have left a Connect row here.
        let from_anything_else = fixture::statements_count(server, &[]);
        let from_anything_at_all = fixture::statements_count(server, &["--all"]);

        assert_eq!(
            attributable,
            0,
            "{} accepted {attributable} connection(s) while tpl ran {} commands \
             that must open none: {POPULATION:?}",
            server.name(),
            POPULATION.len()
        );
        assert_eq!(
            from_anything_else,
            0,
            "{} received {from_anything_else} statement(s) from something that was not the observer",
            server.name()
        );

        // The control for the statement record: the same dump, with the filter
        // that drops the observer's own connections removed, is shown to hold
        // them. Without it, a log that had never been switched on would read
        // exactly like a server that received nothing.
        assert!(
            from_anything_at_all > 0,
            "the statement record of {} held nothing at all, not even the \
             observer's own readings, so its emptiness above establishes nothing",
            server.name()
        );
    }
}

// ------------------------------------------- NFR-PERF-005, discovery clause ---

#[test]
fn nfr_perf_007_the_discovery_clause_of_nfr_perf_005_is_established_by_a_differential_run() {
    // NFR-PERF-007 fixes the arrangement: `tpl init` invoked inside a
    // subdirectory of an existing project, which per FR-PROJ-012 and
    // FR-PROJ-013 creates a project in that subdirectory. Had the ancestor
    // decided where the project went, the outcome would have differed — there
    // would be no `.tpl` in the subdirectory, and FR-PROJ-014 would have made
    // the invocation a `73` at an ancestor that already has one.
    //
    // **What this run does not establish**, stated because the instrument is
    // weaker than a syscall trace and NFR-PERF-005 requires the difference to be
    // written down. It reads the three channels NFR-PERF-007 names — the exit
    // code, the bytes on stdout, and the artefacts left on disk — and stderr is
    // not among them. FR-PROJ-016 obliges `tpl init` to warn on stderr that the
    // project it created shadows the one above, and that warning names the
    // ancestor, so this run says nothing about whether an ancestor was looked
    // at. On the two Linux targets the syscall trace of NFR-PERF-007 is what
    // establishes the clause as a syscall and this run corroborates it; on the
    // two Darwin targets no such trace exists and this run is the whole of the
    // evidence.
    let arranged = Sandbox::new();
    arranged.project(ACCEPTED);
    let arranged_from = arranged.directory("sub/deep");

    let control = Sandbox::new();
    let control_from = control.directory("sub/deep");

    // The inversion, and it comes first, while neither subdirectory holds a
    // project of its own: a command that does perform discovery finds the
    // ancestor from exactly this directory, and its outcome differs from the
    // same command run where there is nothing to find. Without this the
    // equality below would hold just as well over an arrangement that could
    // never have been tripped over.
    let discovering_arranged = Outcome::record(&arranged_from, &arranged_from, &["cfg", "list"]);
    let discovering_control = Outcome::record(&control_from, &control_from, &["cfg", "list"]);

    assert_eq!(
        discovering_arranged.code(),
        Some(0),
        "a command that discovers did not find the ancestor project: {discovering_arranged:?}"
    );
    assert_eq!(
        discovering_control.code(),
        Some(CONFIG),
        "a command that discovers found a project where none was arranged: {discovering_control:?}"
    );
    assert_ne!(
        discovering_arranged, discovering_control,
        "the arrangement is inert: a command that does perform discovery had the \
         same outcome with an ancestor project and without one"
    );

    let before = arranged.configuration();

    // The differential run itself.
    let performed = Outcome::record(&arranged_from, &arranged_from, &["init"]);
    let absent = Outcome::record(&control_from, &control_from, &["init"]);

    assert_eq!(
        performed.code(),
        Some(0),
        "tpl init inside a subdirectory of a project did not exit 0: {performed:?}"
    );
    assert!(
        !performed.artefacts().is_empty(),
        "tpl init inside a subdirectory of a project created nothing there"
    );
    assert_eq!(
        performed, absent,
        "tpl init inside a subdirectory of a project had a different outcome \
         from tpl init with nothing above it"
    );

    // The ancestor is left exactly as it was, which is the disk half of the
    // same instrument.
    assert_eq!(
        arranged.configuration(),
        before,
        "tpl init rewrote the configuration of the project above it"
    );

    // The ancestor's `.tpl` holds what it was given and nothing the invocation
    // added: the five artefacts of `FR-PROJ-017` landed in the subdirectory, and
    // an invocation that had resolved the ancestor as its destination would have
    // put them here instead.
    let mut above: Vec<String> = std::fs::read_dir(arranged.path(".tpl"))
        .expect("the ancestor project is there")
        .map(|entry| {
            entry
                .expect("the entry is readable")
                .file_name()
                .to_string_lossy()
                .into_owned()
        })
        .collect();
    above.sort();

    assert_eq!(
        above,
        [".cfg"],
        "tpl init wrote into the project above the one it created"
    );
}

// --------------------------------------- NFR-PERF-005, configuration clause ---

#[test]
fn nfr_perf_007_the_configuration_clause_of_nfr_perf_005_is_established_by_a_differential_run() {
    // NFR-PERF-007 fixes the arrangement: a command of FR-PROJ-025 invoked
    // inside a project whose `.tpl/.cfg` would fail the validation of
    // FR-CONF-034, asserting exit `0` and stdout byte-identical to the same
    // command invoked outside any project. Those are the two channels the
    // requirement names for this clause, and they are the two compared below;
    // the disk channel is asserted separately, as the file being left byte for
    // byte as it was.
    //
    // **What this run does not establish.** As for the discovery clause: it
    // reads no syscall, so a build that opened `.tpl/.cfg`, read it and
    // discarded what it read would satisfy it. On the two Linux targets the
    // syscall trace of NFR-PERF-007 is what would catch that; on the two Darwin
    // targets nothing does, and this run is the whole of the evidence.
    let arranged = Sandbox::new();
    arranged.project(REFUSED);

    let control = Sandbox::new();

    // The inversion: the same arrangement, under a command that does read
    // `.tpl/.cfg`, against a project whose file the reader accepts. The two
    // outcomes differ, so the refused file is potent and the equalities below
    // are worth something.
    let accepted = Sandbox::new();
    accepted.project(ACCEPTED);

    let reads_refused = Outcome::record(arranged.root(), arranged.root(), &["cfg", "list"]);
    let reads_accepted = Outcome::record(accepted.root(), accepted.root(), &["cfg", "list"]);

    assert_eq!(
        reads_refused.code(),
        Some(CONFIG),
        "a command that reads the configuration was not refused by a file \
         FR-CONF-034 rejects: {reads_refused:?}"
    );
    assert_eq!(
        reads_accepted.code(),
        Some(0),
        "a command that reads the configuration was refused by a file the \
         reader accepts: {reads_accepted:?}"
    );
    assert_ne!(
        reads_refused, reads_accepted,
        "the arrangement is inert: a command that does read `.tpl/.cfg` had the \
         same outcome over a file FR-CONF-034 rejects and over one it accepts"
    );

    let before = arranged.configuration();

    // The differential run, over every form of help and of version.
    for arguments in ANSWERED_ANYWHERE {
        let inside = Outcome::record(arranged.root(), arranged.root(), arguments);
        let outside = Outcome::record(control.root(), control.root(), arguments);
        let spelled = format!("tpl {}", arguments.join(" "));

        assert_eq!(
            inside.code(),
            Some(0),
            "{spelled} inside a project whose .tpl/.cfg FR-CONF-034 rejects did \
             not exit 0: {inside:?}"
        );
        // Byte-identical, as NFR-PERF-007 words it, and reported as text: a
        // failure that printed two byte vectors would have to be decoded by
        // hand before it could be read.
        assert!(
            inside.stdout() == outside.stdout(),
            "{spelled} wrote a different stdout inside such a project than \
             outside any project:\n{:?}\n{:?}",
            String::from_utf8_lossy(inside.stdout()),
            String::from_utf8_lossy(outside.stdout())
        );
        assert!(
            !inside.stdout().is_empty(),
            "{spelled} answered with nothing, so the comparison above compared \
             two empty streams"
        );
    }

    // `tpl init` is the fourth command of FR-PROJ-025, and it is run into a
    // subdirectory because FR-PROJ-014 makes a second `init` at a destination
    // that already holds `.tpl` a `73`. The artefacts are compared too, which
    // the forms above leave nothing of.
    let arranged_init = Sandbox::new();
    arranged_init.project(REFUSED);
    let control_init = Sandbox::new();

    let inside = Outcome::record(
        arranged_init.root(),
        &arranged_init.path("sub"),
        &["init", "sub"],
    );
    let outside = Outcome::record(
        control_init.root(),
        &control_init.path("sub"),
        &["init", "sub"],
    );

    assert_eq!(
        inside.code(),
        Some(0),
        "tpl init sub inside a project whose .tpl/.cfg FR-CONF-034 rejects did \
         not exit 0: {inside:?}"
    );
    assert!(
        !inside.artefacts().is_empty(),
        "tpl init sub created nothing"
    );
    assert_eq!(
        inside, outside,
        "tpl init sub had a different outcome inside such a project than \
         outside any project"
    );

    assert_eq!(
        arranged.configuration(),
        before,
        "a command of FR-PROJ-025 rewrote a configuration it must not even read"
    );
}

// ------------------------------- NFR-PERF-005, the clauses as syscalls ---

#[test]
fn nfr_perf_007_the_file_open_trace_is_taken_on_a_linux_target_and_credited_nowhere_else() {
    // NFR-PERF-007 credits the syscall trace to the two Linux targets of
    // NFR-PERF-018 and to no other, and NFR-PERF-005 forbids inferring it on
    // either Darwin target from a Linux build observed in a container: the
    // artefact traced would not be the artefact distributed. `observe.sh opens`
    // falls back to exactly that container when the host has no `strace`, which
    // is why the wrapper names the backend rather than leaving it at `auto`.
    //
    // Where the instrument does not exist, the differential runs above are the
    // whole of the evidence for the discovery and configuration clauses, and
    // each says so in its own doc comment.
    if !cfg!(target_os = "linux") {
        fixture::notice(
            "skipped nfr_perf_007_the_file_open_trace_is_taken_on_a_linux_target_and_credited_nowhere_else: \
             NFR-PERF-005 credits the syscall trace to the two Linux targets only, and this is \
             not one of them. The discovery and configuration clauses are established here by \
             the differential runs of NFR-PERF-007.",
        );
        return;
    }

    let sandbox = Sandbox::new();
    sandbox.project(ACCEPTED);

    let binary = env!("CARGO_BIN_EXE_tpl");
    let root = sandbox.root().display().to_string();

    // The command is wrapped in a shell because `observe.sh opens` traces from
    // its own directory, and the arrangement is that the invocation stands
    // inside a project an upward walk would find.
    let traced = format!("cd '{root}' && '{binary}' help");
    let Some(trace) = fixture::opens(&["sh", "-c", &traced]) else {
        fixture::notice(
            "skipped nfr_perf_007_the_file_open_trace_is_taken_on_a_linux_target_and_credited_nowhere_else: \
             this Linux host has no strace, so the instrument NFR-PERF-007 credits to it does \
             not exist here.",
        );
        return;
    };

    // The control, and it is the same instrument over a command that does open
    // the file: without it, a trace that recorded nothing at all would read
    // exactly like a command that opened nothing.
    let reading = format!("cd '{root}' && '{binary}' cfg list");
    let control =
        fixture::opens(&["sh", "-c", &reading]).expect("strace answered once and is still there");

    assert!(
        control.contains(".tpl/.cfg"),
        "the trace of a command that reads .tpl/.cfg did not record it, so the \
         absence below establishes nothing"
    );

    assert!(
        !trace.contains(".tpl/.cfg"),
        "tpl help opened or stat'ed .tpl/.cfg, which NFR-PERF-005 forbids"
    );
    assert!(
        !trace.contains("socket(AF_"),
        "tpl help opened a socket, which NFR-PERF-005 forbids"
    );
}

// ------------------------------- FR-SRV-012, FR-SRV-013, FR-SRV-014 ---

/// The database entry every project of this section defines.
const ENTRY: &str = "fixture";

/// The schema every fixture server carries.
const SCHEMA: &str = "freight";

/// The privileged account of the fixture.
const ROOT: (&str, &str) = ("root", "tpl-root");

/// The eleven catalogue statements a full read issues (`NFR-PERF-001`).
const FULL_READ: i64 = 11;

/// The read-only session statement of `FR-SRV-008`, which `FR-SRV-042` issues
/// first.
const READ_ONLY: &str = "SET SESSION TRANSACTION READ ONLY";

/// The read-back of `FR-SRV-009`, issued immediately after it.
///
/// `10.11` does not carry `transaction_read_only` — difference 12 of
/// `FR-SRV-038` — so this is the one spelling a read-back can use across the
/// window, and the test asserts the spelling as well as the position.
const READ_BACK: &str = "SELECT @@session.tx_read_only";

/// The version probe of `FR-SRV-002`, which follows the pair.
const PROBE: &str = "SELECT VERSION()";

/// A sandbox holding a project that reaches `server` as the privileged account.
fn project(server: &fixture::Server) -> Sandbox {
    let sandbox = Sandbox::new();
    sandbox.project(&fixture::configuration(server, ENTRY, SCHEMA, ROOT));

    sandbox
}

/// Runs `tpl` in `sandbox` and refuses anything but exit `0`.
fn succeeds(sandbox: &Sandbox, arguments: &[&str]) -> String {
    let printed = sandbox.run(arguments);

    assert_eq!(
        printed.status.code(),
        Some(0),
        "tpl {} exited {:?}: {}",
        arguments.join(" "),
        printed.status.code(),
        String::from_utf8_lossy(&printed.stderr)
    );

    String::from_utf8_lossy(&printed.stdout).into_owned()
}

/// The statement text of each row the record holds for the account under test,
/// in the order the server received them.
///
/// `observe.sh statements dump` prints a thread, a command type and the
/// statement; this keeps the last two, because what `FR-SRV-012` asserts is
/// which statements arrived and in which order.
fn received(server: &fixture::Server) -> Vec<(String, String)> {
    fixture::statements_text(server, &["--user", ROOT.0])
        .lines()
        .skip(1)
        .filter_map(|row| {
            let mut fields = row.splitn(3, '\t');
            let _thread = fields.next()?;
            let kind = fields.next()?.to_owned();
            let statement = fields.next().unwrap_or_default().trim().to_owned();

            Some((kind, statement))
        })
        .collect()
}

#[test]
fn fr_srv_012_the_server_receives_the_four_kinds_of_the_closed_list_and_no_fifth() {
    // FR-SRV-012 with BR-SRV-003: the closed list of FR-SRV-006 is verified by
    // observing the statements the server **actually receives**, because a
    // promise about what a process sends that can only be checked by reading
    // that process's own source is not a promise a caller can rely on.
    //
    // The test expects the four kinds and no fifth, and asserts that the three
    // connection-start statements are issued exactly once each, in the order
    // FR-SRV-042 fixes — which is the read-only session statement, its
    // read-back immediately after, then the version probe. That order is cited
    // and not restated from the table of FR-SRV-006, which enumerates the four
    // kinds without ordering them: its first row is the statement issued last.
    let _guard = fixture::exclusive();
    let Some(series) = fixture::series(
        "fr_srv_012_the_server_receives_the_four_kinds_of_the_closed_list_and_no_fifth",
    ) else {
        return;
    };

    for server in series {
        let sandbox = project(server);
        let name = server.name();

        fixture::statements_on(server);
        succeeds(&sandbox, &["schema", "dump", "--direct", "--no-cache"]);
        fixture::statements_off(server);

        let rows = received(server);

        // The control: the window held what the invocation issued, so an
        // assertion about its contents is an observation rather than an empty
        // log.
        assert!(
            !rows.is_empty(),
            "{name}: the statement record held nothing the invocation issued"
        );

        // Every row that is a statement is one of the four kinds. `Connect`
        // and `Quit` are connection events and carry no statement, so they are
        // named here rather than silently skipped.
        let mut connection_start: Vec<&str> = Vec::new();
        let mut catalogue = 0;

        for (kind, statement) in &rows {
            match kind.as_str() {
                "Connect" | "Quit" => continue,
                // A prepared statement reaches the record twice — once
                // registered and once issued — and both rows carry the same
                // text, so both are the same kind of the closed list.
                "Query" | "Prepare" | "Execute" => {}
                other => panic!("{name}: the server received a {other} row: {statement}"),
            }

            if statement.contains("INFORMATION_SCHEMA") {
                if kind == "Execute" || kind == "Query" {
                    catalogue += 1;
                }

                continue;
            }

            assert!(
                [READ_ONLY, READ_BACK, PROBE].contains(&statement.as_str()),
                "{name}: the server received a fifth kind of statement: {statement:?}"
            );

            connection_start.push(match statement.as_str() {
                READ_ONLY => READ_ONLY,
                READ_BACK => READ_BACK,
                _ => PROBE,
            });
        }

        // FR-SRV-042: the three, once each, in that order and in no other.
        assert_eq!(
            connection_start,
            [READ_ONLY, READ_BACK, PROBE],
            "{name}: the connection start was not the order FR-SRV-042 fixes"
        );

        // The fourth kind of the closed list, counted through the instrument
        // that exists for it.
        assert_eq!(
            i64::from(catalogue),
            FULL_READ,
            "{name}: the catalogue read was {catalogue} statements"
        );
        assert_eq!(
            fixture::statements_count(server, &["--user", ROOT.0, "--catalogue"]),
            FULL_READ,
            "{name}"
        );
    }
}

#[test]
fn fr_srv_013_the_read_back_confirms_the_setting_on_every_series_of_the_window() {
    // FR-SRV-013: the read-back of FR-SRV-009 is verified by an integration
    // test executed against every series of FR-SRV-015. The binding to the
    // series is the whole of what is at risk here — `transaction_read_only`
    // does not exist on 10.11, difference 12 of FR-SRV-038, so a test that ran
    // anywhere else would pass under either spelling.
    //
    // **This body carries the confirming outcome of FR-SRV-013**, which that
    // requirement assigns to it by name: the outcome is verified by an
    // integration test that observes, on the server, that the read-back is
    // issued and that the value the session reports confirms the setting, and
    // it is an observation made from outside the process, per BR-SRV-003. It is
    // the half that carries the whole of what FR-SRV-013 promises about the
    // statement the process **sends**. The failing outcome is the other half,
    // and the requirement assigns it to the only form that can reach it — a
    // unit test in `src/mariadb/session.rs`, driven through the seam FR-SRV-013
    // authorises on FR-ERR-031's terms, because no server produces a read-back
    // that does not confirm and no arrangement outside the process presents
    // one.
    let _guard = fixture::exclusive();
    let Some(series) = fixture::series(
        "fr_srv_013_the_read_back_confirms_the_setting_on_every_series_of_the_window",
    ) else {
        return;
    };

    for server in series {
        let sandbox = project(server);
        let name = server.name();

        fixture::statements_on(server);
        succeeds(&sandbox, &["schema", "info", "--direct", "--no-cache"]);
        fixture::statements_off(server);

        let statements: Vec<String> = received(server)
            .into_iter()
            .map(|(_, statement)| statement)
            .collect();

        // The statement the server received, in the one spelling every series
        // of the window carries.
        assert_eq!(
            statements
                .iter()
                .filter(|statement| *statement == READ_BACK)
                .count(),
            1,
            "{name}: the read-back was not issued exactly once"
        );
        assert!(
            !statements
                .iter()
                .any(|statement| statement.contains("transaction_read_only")),
            "{name}: the read-back used a spelling 10.11 does not carry"
        );

        // The outcome the read-back confirmed: the invocation completed, and
        // FR-SRV-010 would have made it a 78 in either half of the pair had it
        // not. The value the session reports is the other half of the
        // observation, and it is read here from the server rather than from the
        // process.
        assert_eq!(
            statements
                .iter()
                .filter(|statement| *statement == READ_ONLY)
                .count(),
            1,
            "{name}: the read-only session statement was not issued exactly once"
        );
    }
}

#[test]
fn fr_srv_014_and_nfr_perf_004_an_invocation_opens_at_most_one_connection() {
    // FR-SRV-014: the connection count of an invocation is as NFR-PERF-004
    // fixes it — at most one — and is verifiable from the server side. This is
    // that verification, made in the server's own connection record.
    let _guard = fixture::exclusive();
    let Some(series) =
        fixture::series("fr_srv_014_and_nfr_perf_004_an_invocation_opens_at_most_one_connection")
    else {
        return;
    };

    for server in series {
        let name = server.name();

        // The control, and it comes first: a count of one is worth nothing
        // until this same record is shown to count a connection that was made.
        let control =
            fixture::connections_attributable_to(server, || fixture::connect_once(server));
        assert_eq!(control, 1, "{name}: the connection record counted nothing");

        let sandbox = project(server);

        // One invocation that reads the whole catalogue: eleven statements,
        // and one connection to carry them.
        let opened = fixture::connections_attributable_to(server, || {
            succeeds(&sandbox, &["schema", "dump", "--direct", "--no-cache"]);
        });

        assert_eq!(
            opened, 1,
            "{name}: one invocation opened {opened} connection(s)"
        );

        // And an invocation that names one object, which reads the same
        // catalogue through the same one connection.
        let named = fixture::connections_attributable_to(server, || {
            succeeds(
                &sandbox,
                &["schema", "table", "charge", "--direct", "--no-cache"],
            );
        });

        assert_eq!(
            named, 1,
            "{name}: a named read opened {named} connection(s)"
        );
    }
}

#[test]
fn nfr_perf_001_and_nfr_perf_002_the_catalogue_query_count_does_not_grow_with_the_database() {
    // NFR-PERF-001 forbids a query count that grows with the number of
    // objects, and NFR-PERF-002 says the same of a read that presents one
    // named object — counting **statements**, because the rows such a read
    // returns MAY be the whole catalogue and a read that returns them does not
    // violate it.
    //
    // The comparison is made from the server side, over two databases of
    // different size on the same server, and over a whole read against a named
    // one. `freight` carries 23 catalogue objects; `mysql` carries more, and
    // is the pair `scripts/mariadb/README.md` already uses for the shape of
    // this comparison.
    let _guard = fixture::exclusive();
    let Some(series) = fixture::series(
        "nfr_perf_001_and_nfr_perf_002_the_catalogue_query_count_does_not_grow_with_the_database",
    ) else {
        return;
    };

    for server in series {
        let name = server.name();

        for (schema, invocation) in [
            (SCHEMA, vec!["schema", "dump", "--direct", "--no-cache"]),
            (
                SCHEMA,
                vec!["schema", "table", "charge", "--direct", "--no-cache"],
            ),
            ("mysql", vec!["schema", "dump", "--direct", "--no-cache"]),
            ("mysql", vec!["schema", "tables", "--direct", "--no-cache"]),
        ] {
            let sandbox = Sandbox::new();
            sandbox.project(&fixture::configuration(server, ENTRY, schema, ROOT));

            fixture::statements_on(server);
            succeeds(&sandbox, &invocation);
            fixture::statements_off(server);

            let issued = fixture::statements_count(server, &["--user", ROOT.0, "--catalogue"]);

            assert_eq!(
                issued,
                FULL_READ,
                "{name}: tpl {} against {schema} issued {issued} catalogue statement(s)",
                invocation.join(" ")
            );
        }
    }
}

// ------------------------------------------------------------ FR-RND-022 ---

/// The template every render of this file is given, named as `FR-TMPL-006`
/// resolves it.
const REPORT: &str = "report";

/// Its source, which reads only what a context source carries and never
/// references `now`.
const REPORT_SOURCE: &str = "{{ database.name }}/{{ table.name }}\n";

#[test]
fn fr_rnd_022_a_render_from_a_document_opens_no_connection_and_touches_no_cache() {
    // FR-RND-022 with BR-SRV-003: a render from a `--context` document opens no
    // connection and reads and writes no file of the cache. The first clause is
    // the one a process cannot establish about itself, and this is the server's
    // own connection record answering it — over the file form of FR-RND-016 and
    // over the standard-input form of FR-RND-017 alike, because the two reach
    // the same step of FR-ERR-006 by two different reads.
    //
    // **The arrangement is one that could have been tripped.** The project
    // names an entry that reaches this very server and `core.database` selects
    // it, which FR-RND-019 makes no conflict at all — so a build that let the
    // selected entry win over `--context` would have read the catalogue, would
    // have succeeded, and would have been counted here.
    let _guard = fixture::exclusive();
    let Some(series) = fixture::series(
        "fr_rnd_022_a_render_from_a_document_opens_no_connection_and_touches_no_cache",
    ) else {
        return;
    };

    for server in series {
        let name = server.name();

        // The control, and it comes first: an assertion that no connection was
        // opened is worth nothing until this same record is shown to count one
        // that was.
        let control =
            fixture::connections_attributable_to(server, || fixture::connect_once(server));
        assert_eq!(control, 1, "{name}: the connection record counted nothing");

        let sandbox = project(server);
        sandbox.write(&format!(".tpl/templates/{REPORT}.jinja"), REPORT_SOURCE);

        // The second control, and the document the renders below are given. It
        // is `tpl schema dump` against this server, so the contract of
        // `specification/context-document.md` is stated once — in the product —
        // rather than a second time here; and it is a read of the catalogue, so
        // the same record is shown counting the one connection it opens.
        // FR-CACHE-016 makes `--direct --no-cache` the pure read, which is what
        // leaves the store untouched for the assertion at the end.
        let mut dumped = String::new();
        let producing = fixture::connections_attributable_to(server, || {
            dumped = succeeds(&sandbox, &["schema", "dump", "--direct", "--no-cache"]);
        });

        assert_eq!(
            producing, 1,
            "{name}: producing the document opened {producing} connection(s), so the \
             record below is not the instrument this test takes it for"
        );
        sandbox.write(CONTEXT, &dumped);

        let mut from_file = None;
        let mut from_stdin = None;

        fixture::statements_on(server);
        let attributable = fixture::connections_attributable_to(server, || {
            from_file =
                Some(sandbox.run(&["render", REPORT, "--context", CONTEXT, "--table", BOUND]));
            from_stdin = Some(sandbox.run_with_stdin(
                &["render", REPORT, "--context", "-", "--table", BOUND],
                dumped.as_bytes(),
            ));
        });
        fixture::statements_off(server);

        let from_file = from_file.expect("the bracket ran the file form");
        let from_stdin = from_stdin.expect("the bracket ran the standard-input form");

        // Both renders succeeded, so the count of zero below is the count of a
        // window in which two renders happened rather than of one in which two
        // invocations failed before they could reach a server.
        for (form, printed) in [
            ("--context <path>", &from_file),
            ("--context -", &from_stdin),
        ] {
            assert_eq!(
                printed.status.code(),
                Some(0),
                "{name}: tpl render {form} exited {:?}: {}",
                printed.status.code(),
                String::from_utf8_lossy(&printed.stderr)
            );
            assert!(
                !printed.stdout.is_empty(),
                "{name}: tpl render {form} produced nothing"
            );
        }

        // FR-RND-016 and FR-RND-017 are one contract: the two forms read the
        // same bytes and the result must not betray which carried them.
        assert_eq!(
            from_file.stdout, from_stdin.stdout,
            "{name}: the file form and the standard-input form of one document \
             rendered different bytes"
        );

        assert_eq!(
            attributable, 0,
            "{name}: two renders from a --context document opened {attributable} \
             connection(s), which FR-RND-022 forbids"
        );
        assert_eq!(
            fixture::statements_count(server, &[]),
            0,
            "{name}: the server received a statement from something that was not the observer"
        );
        assert!(
            fixture::statements_count(server, &["--all"]) > 0,
            "{name}: the statement record held nothing at all, not even the \
             observer's own readings, so its emptiness above establishes nothing"
        );

        // The other clause of FR-RND-022, read from the disk rather than from
        // the server: no file of the store was read, and none was written.
        assert!(
            !sandbox.path(".tpl/.cache").exists(),
            "{name}: a render from a --context document created the cache store"
        );
    }
}

// ------------------------------------------- FR-RND-034 and FR-SEM-020 ---

/// A project whose render phase is bounded at one second and which names no
/// database entry at all.
///
/// A render from a `--context` document needs none, per `FR-RND-022`, and the
/// bound is what puts the deadline of `FR-RND-033` within reach of a test.
const BOUNDED: &str = "[core]\nrender_timeout = 1\n";

/// The renders that fail, and the requirement each is written for.
///
/// Every source begins with the five bytes `kept\n`, so a build that streamed
/// its result would have put them on stdout before the construct that fails was
/// reached — which is exactly the incomplete result `FR-RND-034` bounds.
///
/// [`render_command`](../render_command/index.html) drives the same five for a
/// different subject — the `65` each exits and the position `FR-SEM-019`
/// obliges it to carry — and the list is written twice because the two are
/// separate test binaries and `tests/support/` holds what the fixture and the
/// sandbox are, not what any one subject is made of. Neither copy can make the
/// other wrong: each is the population of the body beside it.
const FAILING: [(&str, &str, &str); 5] = [
    ("broken", "kept\n{% if %}\n", "FR-RND-030, a syntax error"),
    (
        "absent",
        "kept\n{{ database.missing }}\n",
        "FR-SEM-012 through FR-RND-031, a field that does not exist",
    ),
    (
        "stopped",
        "kept\n{{ fail('no mapping') }}\n",
        "FR-SEM-014, the author's own failure",
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

/// The name of the template that outlasts the bound of [`BOUNDED`].
const SLOW_TEMPLATE: &str = "slow";

/// The template that outlasts the bound of [`BOUNDED`].
///
/// The nested ranges are 64 million iterations, which the binary under test
/// takes tens of seconds to walk: the body has to outlast one second on every
/// machine this suite runs on, and no machine is fast enough to finish it
/// inside the bound. What the test costs is the bound and not the walk — the
/// timer of `FR-RND-033` ends the process one second in.
const SLOW: &str = "kept\n{% for a in range(8000) %}{% for b in range(8000) %}\
                    {% endfor %}{% endfor %}done\n";

#[test]
fn fr_rnd_034_and_fr_sem_020_a_render_that_fails_leaves_nothing_on_the_process_s_own_stdout() {
    // FR-RND-034 admits at most one incomplete result on stdout when a render
    // fails, and FR-SEM-020 repeats it for every failure render-semantics.md
    // defines. What `tpl` leaves is none, and **this is the only place that can
    // be seen**: a unit test hands `render` a buffer of its own and observes
    // what was written to that buffer, which says nothing about the stream a
    // caller reads — not about the writer, not about its buffering, and least
    // of all about the deadline of FR-RND-033, which ends the process through
    // `std::process::exit` and therefore runs no destructor and flushes
    // nothing.
    //
    // The instrument is the process's own file descriptor 1, read by the
    // parent. Its control is the first assertion made: the same instrument,
    // over the same project, observing a render that succeeds putting bytes
    // there. Without it an empty stdout would mean only that the test had
    // stopped looking.
    let _guard = fixture::exclusive();
    let Some(series) = fixture::series(
        "fr_rnd_034_and_fr_sem_020_a_render_that_fails_leaves_nothing_on_the_process_s_own_stdout",
    ) else {
        return;
    };

    for server in series {
        let name = server.name();

        // The document, produced by the product against this server, in a
        // project of its own: the project the renders run in names no entry,
        // because FR-RND-022 gives a render from a document no use for one.
        let source = project(server);
        let dumped = succeeds(&source, &["schema", "dump", "--direct", "--no-cache"]);

        let sandbox = Sandbox::new();
        sandbox.project(BOUNDED);
        sandbox.write(CONTEXT, &dumped);
        sandbox.write(&format!(".tpl/templates/{REPORT}.jinja"), REPORT_SOURCE);
        sandbox.write(&format!(".tpl/templates/{SLOW_TEMPLATE}.jinja"), SLOW);
        for (template, body, _) in FAILING {
            sandbox.write(&format!(".tpl/templates/{template}.jinja"), body);
        }

        let render = |template: &str| {
            sandbox.run(&["render", template, "--context", CONTEXT, "--table", BOUND])
        };

        // The control.
        let produced = render(REPORT);

        assert_eq!(
            produced.status.code(),
            Some(0),
            "{name}: the control render exited {:?}: {}",
            produced.status.code(),
            String::from_utf8_lossy(&produced.stderr)
        );
        assert!(
            !produced.stdout.is_empty(),
            "{name}: the control render wrote nothing to stdout, so an empty \
             stdout below establishes nothing"
        );

        // The five failures of FR-RND-030, FR-RND-031 and render-semantics.md,
        // and the deadline of FR-RND-033 beside them.
        for (template, _, requirement) in FAILING {
            let printed = render(template);

            assert_eq!(
                printed.status.code(),
                Some(65),
                "{name}: {template} ({requirement}) exited {:?}: {}",
                printed.status.code(),
                String::from_utf8_lossy(&printed.stderr)
            );
            assert!(
                printed.stdout.is_empty(),
                "{name}: {template} ({requirement}) left {} byte(s) on stdout: {:?}",
                printed.stdout.len(),
                String::from_utf8_lossy(&printed.stdout)
            );
        }

        let expired = render(SLOW_TEMPLATE);
        let reported = String::from_utf8_lossy(&expired.stderr).into_owned();

        assert_eq!(
            expired.status.code(),
            Some(65),
            "{name}: the render that outlasts core.render_timeout exited {:?}: {reported}",
            expired.status.code()
        );

        // The `65` this body is about is the deadline's and not another one the
        // template happened to reach: the hint of FR-ERR-008 names the key that
        // bounds the phase, which no other `65` of this file carries.
        assert!(
            reported.contains("core.render_timeout"),
            "{name}: the render ended on something that was not its deadline, so \
             the empty stdout below says nothing about the path that runs no \
             destructor: {reported}"
        );
        assert!(
            expired.stdout.is_empty(),
            "{name}: a render ended by its own deadline left {} byte(s) on stdout, \
             so the process flushed a buffer FR-ERR-033 requires it to discard",
            expired.stdout.len()
        );
    }
}
