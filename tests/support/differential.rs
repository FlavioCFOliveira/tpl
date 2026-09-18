//! The fourth instrument of `NFR-PERF-007`: the differential run.
//!
//! Three of that requirement's four instruments are the fixture's, and live in
//! [`fixture`](../fixture/index.html). This one is the suite's, because it
//! needs no container and no privilege: it is available on all four targets of
//! `NFR-PERF-018`, which is why `NFR-PERF-005` makes it the whole of the
//! evidence for two of its clauses on the two Darwin targets, where no syscall
//! tracer exists.
//!
//! # What the instrument reads
//!
//! `NFR-PERF-007` defines it as *"the observable outcome of the invocation —
//! its exit code, the bytes on stdout, and the artefacts it leaves on disk —
//! under an arrangement in which the operation, had it been performed, would
//! have changed that outcome"*. [`Outcome`] records exactly those three
//! channels.
//!
//! **stderr is not one of them**, and its absence is the requirement's, not an
//! omission here. It matters for one invocation this suite makes:
//! `FR-PROJ-016` obliges `tpl init` to warn on stderr when the project it
//! creates shadows one in an ancestor directory, and that warning names the
//! ancestor. A differential run therefore says nothing about it, in either
//! direction.
//!
//! # How a differential run is arranged
//!
//! Two invocations of the same command, in two directories that differ in one
//! thing: the **arranged** one holds something the operation under test would
//! not have survived, and the **control** one holds nothing for it to find.
//! Their outcomes are compared, and the run establishes its clause when they
//! are equal.
//!
//! An equality between two outcomes is worth nothing until the arrangement is
//! shown to be potent, so each run in this suite is accompanied by its
//! **inversion**: the same comparison over a command that *does* perform the
//! operation, whose two outcomes are shown to differ.
//!
//! # Why this file is under `tests/support/`
//!
//! For the reason [`fixture`](../fixture/index.html) states, and by the same
//! mechanism: Cargo makes a target of `tests/*.rs` and `tests/*/main.rs` and of
//! nothing else, so a file here is compiled into the binaries that name it with
//! `#[path]` and is never a test target of its own.

use std::path::{Path, PathBuf};
use std::process::Command;

/// What one invocation of the binary under test was observed to do.
///
/// The three fields are the three channels `NFR-PERF-007` names, and there is
/// no fourth: stderr is outside the instrument, and so is anything read from
/// inside the process.
#[derive(PartialEq, Eq)]
pub struct Outcome {
    /// The exit code, or `None` when the process was signalled.
    code: Option<i32>,
    /// The bytes written to stdout.
    stdout: Vec<u8>,
    /// Every path under the observed root, relative to it and sorted, so that
    /// two runs made in two directories are comparable.
    artefacts: Vec<PathBuf>,
}

impl Outcome {
    /// Runs the binary under test on `arguments`, from `directory`, under a
    /// cleared environment, and records what it did.
    ///
    /// `observed` is the root the artefacts are collected under, and is named
    /// separately because the tree a command writes into is not always the
    /// directory it is invoked from.
    ///
    /// The environment is cleared for the reason
    /// `tests/invocation_surface.rs` clears it: an invocation that inherited
    /// one would be describable by something the comparison does not hold
    /// equal between the two runs.
    pub fn record(directory: &Path, observed: &Path, arguments: &[&str]) -> Self {
        let printed = Command::new(env!("CARGO_BIN_EXE_tpl"))
            .env_clear()
            .current_dir(directory)
            .args(arguments)
            .output()
            .expect("the binary under test runs");

        Self {
            code: printed.status.code(),
            stdout: printed.stdout,
            artefacts: artefacts(observed),
        }
    }

    /// The exit code, or `None` when the process was signalled.
    pub fn code(&self) -> Option<i32> {
        self.code
    }

    /// The bytes written to stdout.
    pub fn stdout(&self) -> &[u8] {
        &self.stdout
    }

    /// The paths left under the observed root.
    pub fn artefacts(&self) -> &[PathBuf] {
        &self.artefacts
    }
}

impl std::fmt::Debug for Outcome {
    /// Renders the outcome as a failure message can carry it: the stdout bytes
    /// as text, because every stream this suite compares is text and a failure
    /// that printed a byte vector would have to be decoded by hand.
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("Outcome")
            .field("code", &self.code)
            .field("stdout", &String::from_utf8_lossy(&self.stdout))
            .field("artefacts", &self.artefacts)
            .finish()
    }
}

/// Every path under `root`, relative to it and sorted.
///
/// A missing root yields nothing rather than failing: a command that was
/// expected to create a tree and did not is a difference the comparison should
/// report, not a panic inside the instrument.
fn artefacts(root: &Path) -> Vec<PathBuf> {
    let mut found = Vec::new();

    collect(root, root, &mut found);
    found.sort();

    found
}

/// Walks `directory`, pushing every entry relative to `root`.
fn collect(root: &Path, directory: &Path, found: &mut Vec<PathBuf>) {
    let Ok(entries) = std::fs::read_dir(directory) else {
        return;
    };

    for entry in entries.flatten() {
        let path = entry.path();

        if let Ok(relative) = path.strip_prefix(root) {
            found.push(relative.to_path_buf());
        }
        if path.is_dir() {
            collect(root, &path, found);
        }
    }
}
