//! A temporary directory for the tests of this module, and nothing else.
//!
//! Every rule this module holds is a rule about the **filesystem** — a walk
//! over real directories, an ownership, a mode, a rename over a target — so the
//! tests need a real tree. They do not need a dependency for one: a directory
//! under the system's temporary location, named for the process and a counter,
//! is unique without coordination and is removed when the value that made it
//! goes out of scope.
//!
//! The working directory is deliberately never changed. `std::env::set_current_dir`
//! is process-wide state, and `cargo test` runs these tests on several threads
//! of one process, so a test that moved the process would decide where another
//! test was standing. Every function that walks therefore takes the directory
//! to start from, and the process supplies its own only at the one call site
//! that is the process.

use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU32, Ordering};

/// Distinguishes two scratch directories made by one process.
static COUNTER: AtomicU32 = AtomicU32::new(0);

/// A temporary directory, removed when it goes out of scope.
#[derive(Debug)]
pub(crate) struct Scratch {
    /// The directory itself, canonicalised so that a test comparing a
    /// discovered path against it is comparing like with like.
    root: PathBuf,
}

impl Scratch {
    /// Makes a temporary directory.
    pub(crate) fn new() -> Self {
        let name = format!(
            "tpl-{}-{}",
            std::process::id(),
            COUNTER.fetch_add(1, Ordering::Relaxed)
        );
        let root = std::env::temp_dir().join(name);

        std::fs::create_dir_all(&root).expect("the system's temporary directory is writable");

        Self {
            root: std::fs::canonicalize(&root).expect("the directory was just created"),
        }
    }

    /// The directory itself.
    pub(crate) fn root(&self) -> PathBuf {
        self.root.clone()
    }

    /// A path inside the directory, which need not exist.
    pub(crate) fn path(&self, relative: &str) -> PathBuf {
        self.root.join(relative)
    }

    /// Creates a directory inside it, with its missing parents, and returns it.
    pub(crate) fn directory(&self, relative: &str) -> PathBuf {
        let path = self.path(relative);
        std::fs::create_dir_all(&path).expect("the scratch directory is writable");

        path
    }

    /// The canonical form of a path inside it.
    pub(crate) fn canonical(&self, relative: &str) -> PathBuf {
        std::fs::canonicalize(self.path(relative)).expect("the path exists")
    }

    /// Creates a file inside it, with its missing parents, and returns it.
    pub(crate) fn file(&self, relative: &str, contents: &str) -> PathBuf {
        let path = self.path(relative);
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent).expect("the scratch directory is writable");
        }
        std::fs::write(&path, contents).expect("the scratch directory is writable");

        path
    }

    /// Sets the permission bits of a path inside it.
    pub(crate) fn chmod(&self, path: &Path, mode: u32) {
        use std::os::unix::fs::PermissionsExt as _;

        std::fs::set_permissions(path, std::fs::Permissions::from_mode(mode))
            .expect("the scratch directory is writable");
    }

    /// Makes `link` a symbolic link to `target`.
    pub(crate) fn link(&self, target: &Path, link: &Path) {
        std::os::unix::fs::symlink(target, link).expect("the scratch directory is writable");
    }

    /// The permission bits of a path inside it.
    pub(crate) fn mode(&self, path: &Path) -> u32 {
        use std::os::unix::fs::PermissionsExt as _;

        std::fs::metadata(path)
            .expect("the path exists")
            .permissions()
            .mode()
            & 0o7777
    }
}

impl Drop for Scratch {
    /// Removes the directory and everything under it.
    ///
    /// A failure is ignored: a test that has already reported its result must
    /// not be turned into a panic by the cleanup behind it.
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.root);
    }
}
