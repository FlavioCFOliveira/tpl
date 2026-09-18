//! The temporary directory an integration test works in.
//!
//! Every test in this suite that runs the binary runs it somewhere, and that
//! somewhere has to satisfy three conditions at once: it is writable, it is
//! removed when the test ends, and it has **no `.tpl` folder anywhere above
//! it** — because the discovery walk of `FR-PROJ-004` climbs to the root, and a
//! sandbox sitting inside a project would hand the run under test a project
//! nobody arranged.
//!
//! It is the harness of `project::scratch` written where an integration test
//! can reach it: that module is `pub(crate)` and a test binary links the
//! crate's public surface alone. Nothing is added — a directory under the
//! system's temporary location, named for the process and a counter, is unique
//! without coordination and needs no dependency.
//!
//! # Why this file is under `tests/support/` and is reached by `#[path]`
//!
//! For the reason [`fixture`](../fixture/index.html) states, and by the same
//! mechanism: Cargo makes an integration-test target of `tests/*.rs` and of
//! `tests/*/main.rs`, and of nothing else, so a file here whose name is not
//! `main.rs` is compiled into the binaries that name it with `#[path]` and is
//! never a target of its own.
//!
//! # Why this module allows dead code
//!
//! It is included by more than one test binary, and each of them uses the part
//! of the sandbox its own subject needs: a differential run wants two roots and
//! the artefacts under them, and a test of the project and its configuration
//! wants the runner and the nested project. An item unused in one of those
//! binaries is not dead code, and without this allowance
//! `cargo clippy --all-targets` would reject the file for being complete.

#![allow(
    dead_code,
    reason = "this module is included by more than one test binary and each uses the part of the \
              sandbox its own subject needs; an item unused in one of them is not dead code"
)]

use std::path::{Path, PathBuf};
use std::process::{Command, Output};
use std::sync::atomic::{AtomicU32, Ordering};

/// The mode `FR-PROJ-019` creates `.tpl/.cfg` with and `FR-CFG-034` keeps.
const MODE: u32 = 0o600;

/// Distinguishes two sandboxes made by one process.
static COUNTER: AtomicU32 = AtomicU32::new(0);

/// A temporary directory with no `.tpl` folder anywhere above it, removed when
/// it goes out of scope.
#[derive(Debug)]
pub struct Sandbox {
    /// The directory itself, canonicalised so that a path the binary reports
    /// can be compared against it.
    root: PathBuf,
}

impl Sandbox {
    /// Makes one.
    ///
    /// # Panics
    ///
    /// Panics when the system's temporary directory is not writable, and when
    /// the directory it made turns out to sit inside a project — which would
    /// give every run made in it a `.tpl` the test did not arrange.
    pub fn new() -> Self {
        let name = format!(
            "tpl-sandbox-{}-{}",
            std::process::id(),
            COUNTER.fetch_add(1, Ordering::Relaxed)
        );
        let root = std::env::temp_dir().join(name);

        std::fs::create_dir_all(&root).expect("the system's temporary directory is writable");

        let root = std::fs::canonicalize(&root).expect("the directory was just created");

        for ancestor in root.ancestors() {
            assert!(
                !ancestor.join(".tpl").exists(),
                "the sandbox is inside a project, at {}",
                ancestor.display()
            );
        }

        Self { root }
    }

    /// The directory itself.
    pub fn root(&self) -> &Path {
        &self.root
    }

    /// A path inside it, which need not exist.
    pub fn path(&self, relative: &str) -> PathBuf {
        self.root.join(relative)
    }

    /// Creates a directory inside it, with its missing parents.
    ///
    /// # Panics
    ///
    /// Panics when the sandbox is not writable.
    pub fn directory(&self, relative: &str) -> PathBuf {
        let path = self.path(relative);

        std::fs::create_dir_all(&path).expect("the sandbox is writable");

        path
    }

    /// Writes a file inside it, creating the directories it sits in.
    ///
    /// # Panics
    ///
    /// Panics when the sandbox is not writable.
    pub fn write(&self, relative: &str, contents: &str) -> PathBuf {
        let path = self.path(relative);

        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent).expect("the sandbox is writable");
        }
        std::fs::write(&path, contents).expect("the sandbox is writable");

        path
    }

    /// A project at the root of the sandbox, whose `.tpl/.cfg` holds
    /// `configuration` at the mode `FR-PROJ-011` requires of it.
    ///
    /// # Panics
    ///
    /// Panics when the sandbox is not writable.
    pub fn project(&self, configuration: &str) -> PathBuf {
        let tpl = self.directory(".tpl");

        self.write(".tpl/.cfg", configuration);
        chmod(&tpl.join(".cfg"), MODE);

        tpl
    }

    /// A project at `relative`, on the same terms as [`Sandbox::project`].
    ///
    /// # Panics
    ///
    /// Panics when the sandbox is not writable.
    pub fn project_at(&self, relative: &str, configuration: &str) -> PathBuf {
        let tpl = self.directory(&format!("{relative}/.tpl"));

        self.write(&format!("{relative}/.tpl/.cfg"), configuration);
        chmod(&tpl.join(".cfg"), MODE);

        tpl
    }

    /// The bytes of `.tpl/.cfg`.
    ///
    /// # Panics
    ///
    /// Panics when there is no configuration to read.
    pub fn configuration(&self) -> Vec<u8> {
        std::fs::read(self.path(".tpl/.cfg")).expect("the configuration is there")
    }

    /// Runs the binary under test from the root of the sandbox.
    ///
    /// # Panics
    ///
    /// Panics when the binary under test does not run.
    pub fn run(&self, arguments: &[&str]) -> Output {
        self.run_from(&self.root, &[], arguments)
    }

    /// Runs it from `directory`, with `environment` and nothing else.
    ///
    /// # Panics
    ///
    /// Panics when the binary under test does not run.
    pub fn run_from(
        &self,
        directory: &Path,
        environment: &[(&str, &str)],
        arguments: &[&str],
    ) -> Output {
        let mut command = Command::new(env!("CARGO_BIN_EXE_tpl"));

        command.env_clear().current_dir(directory).args(arguments);
        for (name, value) in environment {
            command.env(name, value);
        }

        command.output().expect("the binary under test runs")
    }
}

impl Drop for Sandbox {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.root);
    }
}

/// Sets the mode of a path.
fn chmod(path: &Path, mode: u32) {
    use std::os::unix::fs::PermissionsExt as _;

    std::fs::set_permissions(path, std::fs::Permissions::from_mode(mode))
        .expect("the sandbox is writable");
}
