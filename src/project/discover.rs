//! The upward walk of `FR-PROJ-004`, the boundary of `FR-PROJ-005`, and the
//! explicit folder of `FR-PROJ-008`.
//!
//! A project is any directory containing a `.tpl` folder, per `FR-PROJ-001`,
//! and the walk climbs from the working directory until it finds one. What
//! stops it is the **mount point** of the filesystem that contains the
//! directory the walk starts from, and nothing else: `FR-PROJ-005` removed the
//! home boundary because locating it needed `HOME`, which `FR-CLI-021`
//! prohibits and which would make two identical command lines discover two
//! different projects in two different shells.
//!
//! The mount point needs no environment at all. A directory is one when its
//! device number differs from its parent's, which is a comparison between two
//! `stat` results — so the boundary is a property of the filesystem rather than
//! of a value a shell can set, which is what `BR-CLI-002` requires of every
//! input to discovery.
//!
//! The mount point itself **is** considered, because it contains the directory
//! the walk starts from; its parent is not.
//!
//! `--tpl-dir` names the folder and suppresses the walk, per `FR-PROJ-008` and
//! `FR-GLOB-009`, and is subject to every check without exemption — which is
//! why it returns through the same function and the caller cannot tell the two
//! apart afterwards.
//!
//! `--tpl-dir` must also name a `.tpl` folder, per `FR-PROJ-027`: the last
//! segment of the path, as written or once canonical, is `.tpl`. The walk only
//! ever finds such a folder, so the test makes the flag accept exactly what
//! discovery accepts, and a caller that named the directory holding the
//! project — the most likely mistake with the flag — is refused rather than
//! served an empty project.
//!
//! There is **no fallback**, per `FR-PROJ-007`: a walk that reaches the
//! boundary without finding a `.tpl` fails with `78`, and settings are never
//! read from anywhere else.

use std::os::unix::fs::MetadataExt as _;
use std::path::{Path, PathBuf};

use crate::error::{Error, TplDirFault};

/// The name of the folder that marks a project (`FR-PROJ-001`).
pub(crate) const MARKER: &str = ".tpl";

/// The `.tpl` folder for this invocation, canonicalised.
///
/// `explicit` is the value of `--tpl-dir`, which suppresses the walk, and
/// `start` is the directory the walk begins from otherwise.
///
/// The returned path is canonical, per `FR-PROJ-009`, so a symlinked `.tpl` is
/// verified at its real target by every check that follows.
///
/// # Errors
///
/// Returns [`Error::ProjectNotFound`] where the walk reaches the boundary
/// without finding a `.tpl` (`FR-PROJ-006`), [`Error::ProjectDirUnusable`]
/// where `--tpl-dir` names a path that is not a directory (`FR-PROJ-008`) or a
/// directory that is not a `.tpl` folder (`FR-PROJ-027`), and
/// [`Error::ProjectFileUnreadable`] where the directory the walk starts from
/// cannot be resolved.
pub(crate) fn locate(explicit: Option<&Path>, start: &Path) -> Result<PathBuf, Error> {
    match explicit {
        Some(named) => named_folder(named),
        None => walk(start),
    }
}

/// The folder `--tpl-dir` named, canonicalised, once it has passed the checks
/// of `FR-PROJ-008` and the name test of `FR-PROJ-027`.
///
/// No walk is made, so a refusal names the path and why it is not usable
/// rather than a walk that never happened. The name test runs after the path
/// is resolved and before the trust checks of [`super::trust`], which is the
/// order `FR-PROJ-027` fixes, and nothing inside a refused directory is read
/// beyond whether it holds a `.tpl` folder.
///
/// # Errors
///
/// Returns [`Error::ProjectDirUnusable`] with the fault that applies.
fn named_folder(named: &Path) -> Result<PathBuf, Error> {
    let unusable = |fault| Error::ProjectDirUnusable {
        path: named.to_owned(),
        fault,
    };

    let Some(resolved) = canonical(named) else {
        return Err(unusable(if std::fs::symlink_metadata(named).is_ok() {
            TplDirFault::NotDirectory
        } else {
            TplDirFault::Missing
        }));
    };

    // FR-PROJ-027: the written form admits a `.tpl` that is itself a link, and
    // the canonical form a link whose target is a `.tpl` folder.
    if is_marker(named) || is_marker(&resolved) {
        return Ok(resolved);
    }

    Err(unusable(if canonical(&resolved.join(MARKER)).is_some() {
        TplDirFault::HoldsTplFolder
    } else {
        TplDirFault::NotTplFolder
    }))
}

/// Whether the last segment of `path` is `.tpl` (`FR-PROJ-027`).
fn is_marker(path: &Path) -> bool {
    path.file_name() == Some(std::ffi::OsStr::new(MARKER))
}

/// Climbs from `start` to the boundary, returning the first `.tpl` it finds.
///
/// # Errors
///
/// Returns what [`locate`] returns for the walking form.
fn walk(start: &Path) -> Result<PathBuf, Error> {
    let from = std::fs::canonicalize(start).map_err(|returned| Error::ProjectFileUnreadable {
        path: start.to_owned(),
        returned,
    })?;

    let mut directory = from.as_path();

    loop {
        if let Some(found) = canonical(&directory.join(MARKER)) {
            return Ok(found);
        }

        // FR-PROJ-005: the mount point is the last directory considered, and a
        // `.tpl` above it is not considered at all.
        if is_mount_point(directory) {
            break;
        }

        match directory.parent() {
            Some(parent) => directory = parent,
            None => break,
        }
    }

    Err(Error::ProjectNotFound {
        walk_ended_at: directory.to_owned(),
    })
}

/// `path` canonicalised, where it names a directory.
///
/// `FR-PROJ-009` canonicalises before any check applies, so that a symlinked
/// `.tpl` is verified at its real target rather than at the link.
fn canonical(path: &Path) -> Option<PathBuf> {
    let resolved = std::fs::canonicalize(path).ok()?;

    resolved.is_dir().then_some(resolved)
}

/// Whether `directory` is the mount point of the filesystem it sits on.
///
/// A directory whose device number differs from its parent's is where one
/// filesystem is mounted on another. A directory with no parent is the root,
/// which is a boundary in any case, and a directory whose metadata cannot be
/// read is treated as one too: a walk that cannot see where it is does not
/// climb past it.
fn is_mount_point(directory: &Path) -> bool {
    let Some(parent) = directory.parent() else {
        return true;
    };

    let (Ok(here), Ok(above)) = (std::fs::metadata(directory), std::fs::metadata(parent)) else {
        return true;
    };

    here.dev() != above.dev()
}

#[cfg(test)]
mod tests {
    use super::{MARKER, locate};
    use crate::error::{Error, TplDirFault};
    use crate::project::scratch::Scratch;

    #[test]
    fn fr_proj_001_the_walk_finds_the_marker_in_the_directory_it_starts_from() {
        // FR-PROJ-001, FR-PROJ-004.
        let scratch = Scratch::new();
        let root = scratch.directory("project");
        scratch.directory("project/.tpl");

        let found = locate(None, &root).expect("the project is found");

        assert_eq!(found, scratch.canonical("project").join(MARKER));
    }

    #[test]
    fn fr_proj_004_the_walk_climbs_until_it_finds_the_first_marker_and_stops_there() {
        // FR-PROJ-004: the first one found is the project root, and the walk
        // stops there.
        let scratch = Scratch::new();
        scratch.directory("outer/.tpl");
        scratch.directory("outer/inner/.tpl");
        let deep = scratch.directory("outer/inner/src/models");

        let found = locate(None, &deep).expect("the project is found");

        assert_eq!(found, scratch.canonical("outer/inner").join(MARKER));
    }

    #[test]
    fn fr_proj_006_a_walk_that_finds_nothing_is_the_condition_the_requirement_names() {
        // FR-PROJ-006, FR-PROJ-007: no fallback, and the walk's end is named.
        let scratch = Scratch::new();
        let start = scratch.directory("bare/deeper");

        let condition = locate(None, &start).expect_err("there is no project");

        assert!(matches!(condition, Error::ProjectNotFound { .. }));
        assert_eq!(condition.exit_code(), 78);
    }

    #[test]
    fn fr_proj_009_the_resolved_path_is_canonical() {
        // FR-PROJ-009: a symlinked .tpl is verified at its real target.
        let scratch = Scratch::new();
        let real = scratch.directory("elsewhere/tpl-data");
        let project = scratch.directory("project");
        scratch.link(&real, &project.join(MARKER));

        let found = locate(None, &project).expect("the link resolves to a directory");

        assert_eq!(found, real);
        assert!(!found.ends_with(MARKER));
    }

    #[test]
    fn fr_proj_008_the_explicit_folder_suppresses_the_walk() {
        // FR-PROJ-008, FR-GLOB-009: --tpl-dir names the folder, and the walk
        // does not run — so a project above the named one is not found.
        let scratch = Scratch::new();
        scratch.directory("outer/.tpl");
        let named = scratch.directory("outer/aside/.tpl");
        let start = scratch.directory("outer/inner");

        let found = locate(Some(&named), &start).expect("the named folder is used");

        assert_eq!(found, scratch.canonical("outer/aside/.tpl"));
    }

    #[test]
    fn fr_proj_027_a_directory_holding_a_tpl_folder_is_refused_and_says_so() {
        // FR-PROJ-027: the project directory named instead of its `.tpl`.
        let scratch = Scratch::new();
        let project = scratch.directory("shop");
        scratch.directory("shop/.tpl");

        let condition = locate(Some(&project), &scratch.root()).expect_err("not a .tpl folder");

        match &condition {
            Error::ProjectDirUnusable { path, fault } => {
                assert_eq!(path, &project);
                assert_eq!(*fault, TplDirFault::HoldsTplFolder);
            }
            other => panic!("expected ProjectDirUnusable, got {other:?}"),
        }
        assert_eq!(condition.exit_code(), 78);
    }

    #[test]
    fn fr_proj_027_a_directory_holding_no_tpl_folder_is_refused() {
        // FR-PROJ-027: an empty directory, and a folder inside a project that
        // is not its `.tpl` folder.
        let scratch = Scratch::new();
        let empty = scratch.directory("empty");
        let templates = scratch.directory("shop/.tpl/templates");

        for named in [empty, templates] {
            let condition = locate(Some(&named), &scratch.root()).expect_err("not a .tpl folder");
            assert!(matches!(
                condition,
                Error::ProjectDirUnusable {
                    fault: TplDirFault::NotTplFolder,
                    ..
                }
            ));
        }
    }

    #[test]
    fn fr_proj_027_the_name_is_tested_as_written_and_once_canonical() {
        // FR-PROJ-027: a link named `.tpl` to a folder named otherwise, and a
        // link named otherwise to a folder named `.tpl`, are both accepted; a
        // link whose name and target are both something else is refused.
        let scratch = Scratch::new();
        let data = scratch.directory("data/tpl-data");
        let real = scratch.directory("real/.tpl");
        let written = scratch.directory("written");
        scratch.link(&data, &written.join(MARKER));
        let alias = scratch.path("alias");
        scratch.link(&real, &alias);
        let neither = scratch.path("neither");
        scratch.link(&data, &neither);

        assert_eq!(
            locate(Some(&written.join(MARKER)), &scratch.root()).expect("written .tpl"),
            data
        );
        assert_eq!(
            locate(Some(&alias), &scratch.root()).expect("canonical .tpl"),
            scratch.canonical("real/.tpl")
        );
        assert!(matches!(
            locate(Some(&neither), &scratch.root()),
            Err(Error::ProjectDirUnusable {
                fault: TplDirFault::NotTplFolder,
                ..
            })
        ));
    }

    #[test]
    fn fr_proj_008_an_explicit_folder_that_is_not_a_directory_is_refused() {
        // FR-PROJ-008: it names a `.tpl` folder, and a path that is not one
        // leaves the invocation with no project at all.
        let scratch = Scratch::new();
        let missing = scratch.path("absent");
        let file = scratch.file("a-file", "");

        for named in [missing, file] {
            let condition =
                locate(Some(&named), &scratch.root()).expect_err("the path is not a folder");
            assert!(matches!(condition, Error::ProjectDirUnusable { .. }));
            assert_eq!(condition.exit_code(), 78);
        }
    }

    #[test]
    fn fr_proj_001_a_marker_that_is_a_file_is_not_a_project() {
        // FR-PROJ-001: a project is a directory containing a `.tpl` **folder**.
        let scratch = Scratch::new();
        let project = scratch.directory("project");
        scratch.file("project/.tpl", "not a folder");

        let condition = locate(None, &project).expect_err("a file is not a project");

        assert!(matches!(condition, Error::ProjectNotFound { .. }));
    }
}
