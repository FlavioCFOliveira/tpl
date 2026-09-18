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
//! # What is observable today, and what is not
//!
//! No command of `tpl` opens a connection yet: everything under `schema`,
//! `template`, `render`, `cache` and `cfg database test` exits `70`. Two
//! requirements are therefore reachable now, and observing them is what proves
//! the instruments work:
//!
//! | Requirement | The property |
//! |---|---|
//! | `NFR-PERF-005` | The commands of `FR-PROJ-025` open no connection, perform no project discovery, and read no configuration file |
//! | `NFR-PERF-006` | A command that requires no catalogue data opens no connection |
//!
//! The seven other requirements the register holds to this standard —
//! `NFR-PERF-001` through `-004`, `FR-SRV-012`, `-013` and `-014` — need a
//! catalogue reader, and none exists.
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

use differential::{Outcome, Sandbox};
use fixture::Gate;

/// A `.tpl/.cfg` the reader accepts.
const ACCEPTED: &str = "[core]\n";

/// A `.tpl/.cfg` the reader refuses: `databse` is outside the enumerated key
/// space, which `FR-CONF-034` makes a `78` wherever it appears in the file.
const REFUSED: &str = "[core]\ndatabse = \"shop\"\n";

/// The `78` of `FR-PROJ-006` and of `FR-CONF-034`.
const CONFIG: i32 = 78;

/// Every command of `FR-PROJ-025`, and every `cfg` subcommand but
/// `database test`, in an order that leaves each of them a project and an entry
/// to work on.
///
/// `cfg database test` is excluded because it is the one `cfg` leaf still
/// unwritten, and `NFR-PERF-006` excludes it in its own text. The `template`
/// subcommands and `tpl render --context`, which that requirement also covers,
/// exit `70` today and assert nothing.
const POPULATION: [&[&str]; 18] = [
    &["init"],
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

        let sandbox = Sandbox::new();

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
