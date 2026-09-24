//! The help text is the same text whatever the environment says
//! (`FR-HELP-009`, `FR-HELP-010`, `NFR-DET-004`).
//!
//! These properties cannot be asserted from inside the process: the renderer is
//! a pure function of the command tree and the typed help table, so a unit test
//! that called it twice would prove only that it was called twice. What has to
//! be shown is that **the process** reads nothing from its environment on the
//! way to stdout — so the binary is run, twice, under two environments that
//! disagree about everything a terminal-aware tool would consult, and the bytes
//! are compared.
//!
//! The environment of each run is cleared first, so what reaches the process is
//! exactly what is named here and nothing the developer's shell happens to
//! carry. `COLUMNS` is the variable `FR-HELP-010` names; `TERM`,
//! `CLICOLOR_FORCE` and `FORCE_COLOR` are the ones a tool that emitted colour
//! would read, and `NFR-DET-004` forbids colour "under any circumstances".
//!
//! The six group nodes are the paths used, because a group node invoked with no
//! child prints its own help and exits `0`, per `FR-CLI-007` and `FR-HELP-025`,
//! and they are therefore the shortest vectors that reach the renderer. One
//! renderer composes every help text, per `OD-07` and `FR-HELP-002`, so what
//! holds of these six holds of every node: the equivalences and the reach of
//! the six forms of `FR-HELP-001` are asserted in
//! [`help_surface`](../help_surface/index.html), which walks the tree.

use std::process::{Command, Output};

/// The fixed width of `FR-HELP-009`.
const WIDTH: usize = 80;

/// The escape byte that opens every ANSI sequence.
const ESCAPE: u8 = 0x1b;

/// The six group nodes of `FR-CLI-008`, by the path a caller writes.
const GROUPS: [&[&str]; 6] = [
    &[],
    &["schema"],
    &["template"],
    &["cache"],
    &["cfg"],
    &["cfg", "database"],
];

/// Runs `tpl` on `path`, under an environment holding `columns` and `term` and
/// nothing else but the two colour variables.
fn run(path: &[&str], columns: &str, term: &str) -> Output {
    Command::new(env!("CARGO_BIN_EXE_tpl"))
        .env_clear()
        .env("COLUMNS", columns)
        .env("TERM", term)
        .env("CLICOLOR_FORCE", "1")
        .env("FORCE_COLOR", "3")
        .args(path)
        .output()
        .expect("the binary under test runs")
}

/// The path as a caller writes it, for a failure message.
fn spelled(path: &[&str]) -> String {
    std::iter::once("tpl")
        .chain(path.iter().copied())
        .collect::<Vec<&str>>()
        .join(" ")
}

#[test]
fn fr_help_010_the_help_of_a_group_node_is_the_same_under_every_environment() {
    for path in GROUPS {
        let narrow = run(path, "20", "dumb");
        let wide = run(path, "500", "xterm-256color");

        assert_eq!(
            narrow.stdout,
            wide.stdout,
            "{} laid itself out to the terminal",
            spelled(path)
        );
    }
}

#[test]
fn fr_cli_007_the_help_of_a_group_node_is_written_to_stdout_at_exit_zero() {
    // FR-CLI-007, FR-HELP-025 and BR-CLI-005: help is a legitimate stdout
    // payload, and stderr carries nothing beside it, per BR-CLI-006.
    for path in GROUPS {
        let printed = run(path, "80", "xterm");

        assert_eq!(
            printed.status.code(),
            Some(0),
            "{} did not exit 0",
            spelled(path)
        );
        assert!(
            printed.stderr.is_empty(),
            "{} wrote to stderr",
            spelled(path)
        );

        let text = String::from_utf8(printed.stdout).expect("the renderer writes UTF-8");

        assert!(
            text.starts_with(&format!(
                "{}USAGE\n  {}",
                concat!(
                    "tpl v",
                    env!("CARGO_PKG_VERSION"),
                    " - Code Generation based on database schema\n\n"
                ),
                spelled(path)
            )),
            "{} did not print its own help",
            spelled(path)
        );
    }
}

#[test]
fn fr_help_009_no_line_of_a_rendered_help_exceeds_the_fixed_width() {
    // FR-HELP-009, asserted on the bytes the process emits rather than on the
    // renderer's return value. A column is one Unicode scalar value, which is
    // the count the renderer lays out to.
    for path in GROUPS {
        let printed = run(path, "20", "dumb");
        let text = String::from_utf8(printed.stdout).expect("the renderer writes UTF-8");

        for line in text.lines() {
            assert!(
                line.chars().count() <= WIDTH,
                "{} wrote a line of {} columns: {line:?}",
                spelled(path),
                line.chars().count()
            );
        }
    }
}

#[test]
fn nfr_det_004_no_ansi_escape_sequence_reaches_either_stream() {
    // NFR-DET-004 and FR-HELP-015, under an environment that asks three times
    // over for colour.
    for path in GROUPS {
        let printed = run(path, "500", "xterm-256color");

        assert!(
            !printed.stdout.contains(&ESCAPE),
            "{} wrote an escape sequence to stdout",
            spelled(path)
        );
        assert!(
            !printed.stderr.contains(&ESCAPE),
            "{} wrote an escape sequence to stderr",
            spelled(path)
        );
    }
}
