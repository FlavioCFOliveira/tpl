//! The project: where `tpl` finds one, why it trusts it, and what it reads
//! from it.
//!
//! A project is any directory containing a `.tpl` folder, per `FR-PROJ-001`,
//! and that folder is the only source of configuration and templates. This
//! module is the whole of the path from a working directory to a validated
//! configuration, and it runs in the order `FR-ERR-006` fixes:
//!
//! | Step | Submodule | Requirement |
//! |---|---|---|
//! | 2 — discovery | [`discover`] | `FR-PROJ-004` … `FR-PROJ-009`, `FR-PROJ-027` |
//! | 2 — trust | [`trust`] | `FR-PROJ-010`, `FR-PROJ-011`, `FR-PROJ-028` |
//! | 3 — read and validate | [`config`] | `FR-CONF-001` … `FR-CONF-022`, `FR-CONF-034` … `FR-CONF-036` |
//! | 4 — entry resolution | [`settings`] | `FR-CONF-004`, `FR-CONF-029`, `FR-GLOB-004` … `FR-GLOB-008` |
//!
//! Steps 2 and 3 are skipped for the four commands `FR-PROJ-025` names — `tpl
//! init`, the three help forms, and `tpl version` — which is why nothing here
//! is reached from the parser and everything here is reached from a command.
//!
//! Two modules sit beside the four because they are reached from the
//! resolution rather than from the file: [`password`], which runs the child of
//! `FR-CONF-023` through `FR-CONF-033`, and [`secret`], which is the one type a
//! credential is carried in. [`edit`] is the write path, and it is separate
//! from [`config`] for the reason `FR-CFG-041` makes it a different problem:
//! reading validates, writing preserves.
//!
//! [`init`] is the one writer of anything but `.tpl/.cfg`, per `FR-PROJ-023`,
//! and the one command that can reach `73`, per `FR-ERR-003`.

pub(crate) mod config;
pub(crate) mod discover;
pub(crate) mod edit;
pub(crate) mod init;
pub(crate) mod password;
pub(crate) mod secret;
pub(crate) mod trust;

// `tpl cfg …` maintains the file and never resolves it — `FR-CFG-014` forbids
// it to. The commands that resolve it are the ones that read a catalogue: the
// eight `schema` subcommands and the three of `tpl cache`, which reach this
// module through `cli::source`.
pub(crate) mod settings;

#[cfg(test)]
pub(crate) mod scratch;

use std::path::{Path, PathBuf};

use crate::error::Error;

use config::Configuration;

/// One project: the `.tpl` folder this invocation works against.
///
/// Holding it as a type rather than as a path is what keeps the order of
/// `FR-ERR-006` a property of the code: a [`Project`] exists only after
/// discovery and the two trust checks have both passed, so a command that has
/// one has already satisfied step 2, and [`Project::configuration`] is the
/// whole of step 3.
#[derive(Debug, Clone)]
pub(crate) struct Project {
    /// The `.tpl` folder, canonical per `FR-PROJ-009`.
    root: PathBuf,
}

impl Project {
    /// Discovers the project and applies the trust checks.
    ///
    /// `explicit` is the value of `--tpl-dir`, which suppresses the walk and is
    /// subject to every check without exemption, per `FR-PROJ-008` and
    /// `FR-GLOB-010`. `start` is the directory the walk begins from.
    ///
    /// # Errors
    ///
    /// Returns what [`discover::locate`] and [`trust::check`] return: the `78`
    /// of a project that is not found, of a `--tpl-dir` that names no `.tpl`
    /// folder, of a `.cfg` owned by another user, of a `.cfg` that grants
    /// group or other any access, and of a `.tpl` folder without `.cfg` owned
    /// by another user.
    pub(crate) fn open(explicit: Option<&Path>, start: &Path) -> Result<Self, Error> {
        let root = discover::locate(explicit, start)?;

        trust::check(&root.join(edit::CONFIGURATION), &root)?;

        Ok(Self { root })
    }

    /// Discovers the project from the process's own working directory.
    ///
    /// This is [`Project::open`] with the one argument the process supplies,
    /// and it is the only reader of the working directory in the crate: every
    /// other path takes the directory to walk from, so a test never has to move
    /// the process.
    ///
    /// # Errors
    ///
    /// Returns what [`Project::open`] returns, and
    /// [`Error::ProjectFileUnreadable`] where the working directory cannot be
    /// read.
    pub(crate) fn current(explicit: Option<&Path>) -> Result<Self, Error> {
        let start = std::env::current_dir().map_err(|returned| Error::ProjectFileUnreadable {
            path: PathBuf::from("."),
            returned,
        })?;

        Self::open(explicit, &start)
    }

    /// The `.tpl` folder itself, canonical per `FR-PROJ-009`.
    ///
    /// It is what the catalogue cache is laid out under: `FR-CACHE-001` puts
    /// the store at `.tpl/.cache/`, so [`crate::cache`] is given this path and
    /// composes the rest.
    pub(crate) fn root(&self) -> &Path {
        &self.root
    }

    /// The path of `.tpl/.cfg`.
    pub(crate) fn configuration_path(&self) -> PathBuf {
        self.root.join(edit::CONFIGURATION)
    }

    /// Reads and validates `.tpl/.cfg` — step 3 of `FR-ERR-006`.
    ///
    /// # Errors
    ///
    /// Returns what [`config::load`] returns.
    pub(crate) fn configuration(&self) -> Result<Configuration, Error> {
        config::load(&self.configuration_path())
    }

    /// Opens `.tpl/.cfg` for writing — the path of `FR-CFG-041`.
    ///
    /// # Errors
    ///
    /// Returns what [`edit::Editor::open`] returns.
    pub(crate) fn editor(&self) -> Result<edit::Editor, Error> {
        edit::Editor::open(&self.root)
    }
}

#[cfg(test)]
mod tests {
    use super::{Project, scratch::Scratch};
    use crate::error::Error;

    #[test]
    fn fr_err_006_a_project_exists_only_once_discovery_and_the_two_checks_have_passed() {
        // FR-ERR-006, step 2: discovery and the trust checks, before the file
        // is read at step 3.
        let scratch = Scratch::new();
        let root = scratch.directory("project");
        scratch.directory("project/.tpl");
        let file = scratch.file("project/.tpl/.cfg", "[core]\ndatabase = \"shop\"\n");
        scratch.chmod(&file, 0o600);

        let project = Project::open(None, &root).expect("the project is trusted");

        assert_eq!(
            project.configuration_path(),
            scratch.canonical("project/.tpl").join(".cfg")
        );
        assert_eq!(
            project
                .configuration()
                .expect("the file is valid")
                .core()
                .database
                .as_deref(),
            Some("shop")
        );
    }

    #[test]
    fn fr_proj_011_an_unsafe_configuration_is_refused_before_it_is_read() {
        // FR-PROJ-011, and step 2 before step 3: the file is never read.
        let scratch = Scratch::new();
        let root = scratch.directory("project");
        scratch.directory("project/.tpl");
        let file = scratch.file("project/.tpl/.cfg", "this is not TOML at all [[[\n");
        scratch.chmod(&file, 0o644);

        let condition = Project::open(None, &root).expect_err("the mode is unsafe");

        // The mode is reported rather than the TOML, which proves the order.
        assert!(matches!(condition, Error::ConfigurationUnsafeMode { .. }));
    }

    #[test]
    fn fr_proj_008_a_project_named_by_the_flag_is_subject_to_the_same_checks() {
        // FR-PROJ-008, FR-GLOB-010: without exemption.
        let scratch = Scratch::new();
        let named = scratch.directory("elsewhere/.tpl");
        let file = scratch.file("elsewhere/.tpl/.cfg", "[core]\n");
        scratch.chmod(&file, 0o640);

        let condition =
            Project::open(Some(&named), &scratch.root()).expect_err("the mode is unsafe");

        assert!(matches!(condition, Error::ConfigurationUnsafeMode { .. }));
        assert_eq!(condition.exit_code(), 78);
    }

    #[test]
    fn fr_proj_027_a_directory_that_is_not_a_tpl_folder_is_refused_before_the_trust_checks() {
        // FR-PROJ-027: evaluated before FR-PROJ-010 and FR-PROJ-011, so an
        // unsafe `.cfg` inside the refused directory is never judged, and
        // nothing is written into it.
        let scratch = Scratch::new();
        let project = scratch.directory("shop");
        scratch.directory("shop/.tpl");
        let stray = scratch.file("shop/.cfg", "[core]\n");
        scratch.chmod(&stray, 0o644);

        let condition = Project::open(Some(&project), &scratch.root()).expect_err("not .tpl");

        assert!(matches!(condition, Error::ProjectDirUnusable { .. }));
        assert_eq!(condition.exit_code(), 78);
    }

    #[test]
    fn fr_proj_001_a_project_with_no_configuration_file_is_a_project_with_an_empty_one() {
        // FR-PROJ-001 makes the project the folder, and FR-CFG-004 lets
        // `tpl cfg set` write the file again.
        let scratch = Scratch::new();
        let root = scratch.directory("project");
        scratch.directory("project/.tpl");

        let project = Project::open(None, &root).expect("the folder is the project");
        let configuration = project.configuration().expect("an absent file is empty");

        assert!(configuration.keys().is_empty());
        assert_eq!(configuration.names().len(), 0);
    }
}
