//! The two trust checks of `FR-PROJ-010` and `FR-PROJ-011`, applied before
//! `.tpl/.cfg` is read.
//!
//! `.cfg` decides which host is contacted, which credential is used and which
//! child process is executed, so a file another user can write is a file
//! another user can point at their own `password_command` and have it run with
//! the caller's privileges. The two checks are what `FR-SEC-014` closes that
//! with, and they apply to every project without exemption — including one
//! named by `--tpl-dir`, per `FR-PROJ-008` and `FR-GLOB-010`.
//!
//! The file is followed to its target first, per `FR-PROJ-009`, so a `.cfg`
//! that is a symbolic link is checked at its real target rather than at the
//! link, whose own mode Unix does not enforce.
//!
//! An **absent** `.cfg` passes: there is nothing to own and nothing to grant.
//! `FR-PROJ-001` makes the project the folder rather than the file, and
//! `FR-CFG-004` lets `tpl cfg set` write one — so a project whose file was
//! removed by hand is a project with an empty configuration, not a project that
//! cannot be used.
//!
//! The judgment is separated from the reading of the metadata, in [`judge`],
//! because the ownership half cannot otherwise be exercised: a test process
//! cannot give a file to another user, and a check that is only ever called
//! with its own uid is a check nothing has watched fire.

use std::io;
use std::os::unix::fs::{MetadataExt as _, PermissionsExt as _};
use std::path::Path;

use crate::error::Error;

/// The permission bits `FR-PROJ-011` requires to be clear: group and other.
const GROUP_AND_OTHER: u32 = 0o077;

/// The bits of a mode that are permissions.
const PERMISSIONS: u32 = 0o7777;

/// Applies the two checks to `file`.
///
/// # Errors
///
/// Returns [`Error::ConfigurationNotOwned`] where the file belongs to another
/// user (`FR-PROJ-010`), [`Error::ConfigurationUnsafeMode`] where it grants
/// group or other any access (`FR-PROJ-011`), and
/// [`Error::ProjectFileUnreadable`] where its metadata cannot be read for any
/// reason other than its absence.
pub(crate) fn check(file: &Path) -> Result<(), Error> {
    // `metadata` follows the link, which is what FR-PROJ-009 asks for: the
    // ownership and the mode that matter are the target's.
    let metadata = match std::fs::metadata(file) {
        Ok(metadata) => metadata,
        Err(returned) if returned.kind() == io::ErrorKind::NotFound => return Ok(()),
        Err(returned) => {
            return Err(Error::ProjectFileUnreadable {
                path: file.to_owned(),
                returned,
            });
        }
    };

    judge(
        file,
        metadata.uid(),
        metadata.permissions().mode() & PERMISSIONS,
        invoking_user(),
    )
}

/// Decides the two checks over an ownership and a mode already read.
///
/// The order is the order of the requirements: ownership first, because a file
/// belonging to someone else is refused whatever its mode says.
///
/// # Errors
///
/// Returns what [`check`] returns for the two conditions it decides.
fn judge(file: &Path, owner: u32, mode: u32, invoking: u32) -> Result<(), Error> {
    if owner != invoking {
        return Err(Error::ConfigurationNotOwned {
            path: file.to_owned(),
            owner,
            expected: invoking,
        });
    }

    if mode & GROUP_AND_OTHER != 0 {
        return Err(Error::ConfigurationUnsafeMode {
            path: file.to_owned(),
            mode,
        });
    }

    Ok(())
}

/// The user id of the invoking process.
///
/// `rustix` rather than `libc`, because reading it through `libc` would need an
/// `unsafe` block and `#![forbid(unsafe_code)]` denies the crate one.
pub(crate) fn invoking_user() -> u32 {
    rustix::process::getuid().as_raw()
}

#[cfg(test)]
mod tests {
    use super::{check, invoking_user, judge};
    use crate::error::Error;
    use crate::project::scratch::Scratch;
    use std::path::Path;

    #[test]
    fn a_file_owned_by_the_invoking_user_at_six_hundred_passes() {
        // FR-PROJ-010, FR-PROJ-011, and the mode FR-PROJ-019 creates.
        let scratch = Scratch::new();
        let file = scratch.file(".cfg", "[core]\n");
        scratch.chmod(&file, 0o600);

        assert!(check(&file).is_ok());
    }

    #[test]
    fn an_absent_file_passes_because_there_is_nothing_to_trust() {
        let scratch = Scratch::new();

        assert!(check(&scratch.path("absent.cfg")).is_ok());
    }

    #[test]
    fn a_file_owned_by_another_user_is_refused() {
        // FR-PROJ-010: the ownership is judged over values already read,
        // because a test process cannot give a file away.
        let invoking = invoking_user();
        let condition = judge(
            Path::new("/work/.tpl/.cfg"),
            invoking.wrapping_add(1),
            0o600,
            invoking,
        )
        .expect_err("the file belongs to another user");

        match condition {
            Error::ConfigurationNotOwned {
                owner, expected, ..
            } => {
                assert_eq!(owner, invoking.wrapping_add(1));
                assert_eq!(expected, invoking);
            }
            other => panic!("expected an ownership refusal, got {other:?}"),
        }
        assert_eq!(condition.exit_code(), 78);
    }

    #[test]
    fn ownership_is_judged_before_the_mode() {
        // FR-PROJ-010 before FR-PROJ-011: a file belonging to someone else is
        // refused whatever its mode says, and the caller's next step is to
        // stop using it rather than to chmod it.
        let invoking = invoking_user();
        let condition = judge(
            Path::new("/work/.tpl/.cfg"),
            invoking.wrapping_add(1),
            0o644,
            invoking,
        )
        .expect_err("the file belongs to another user");

        assert!(matches!(condition, Error::ConfigurationNotOwned { .. }));
    }

    #[test]
    fn a_file_granting_group_or_other_any_access_is_refused() {
        // FR-PROJ-011: no group and no other access bits, and the message
        // names the mode found.
        let scratch = Scratch::new();

        for mode in [0o640, 0o604, 0o644, 0o660, 0o606, 0o601, 0o610, 0o777] {
            let file = scratch.file(&format!("cfg-{mode:o}"), "[core]\n");
            scratch.chmod(&file, mode);

            let condition = check(&file).expect_err("the mode is unsafe");
            match condition {
                Error::ConfigurationUnsafeMode { mode: found, .. } => {
                    assert_eq!(found, mode);
                }
                other => panic!("expected an unsafe mode for {mode:o}, got {other:?}"),
            }
            assert_eq!(condition.exit_code(), 78);
        }
    }

    #[test]
    fn a_mode_that_grants_the_owner_alone_passes_whatever_the_owner_may_do() {
        // The requirement is about group and other; the owner's own bits are
        // not constrained by it.
        let scratch = Scratch::new();

        for mode in [0o400, 0o600, 0o700] {
            let file = scratch.file(&format!("owner-{mode:o}"), "[core]\n");
            scratch.chmod(&file, mode);

            assert!(check(&file).is_ok(), "mode {mode:o} is the owner's alone");
        }
    }

    #[test]
    fn a_symbolic_link_is_checked_at_its_target() {
        // FR-PROJ-009: the path is followed before any check, so the mode that
        // decides is the target's and not the link's.
        let scratch = Scratch::new();
        let target = scratch.file("target.cfg", "[core]\n");
        scratch.chmod(&target, 0o644);
        let link = scratch.path("link.cfg");
        scratch.link(&target, &link);

        let condition = check(&link).expect_err("the target's mode is unsafe");

        assert!(matches!(
            condition,
            Error::ConfigurationUnsafeMode { mode: 0o644, .. }
        ));
    }

    #[test]
    fn the_invoking_user_owns_a_file_this_process_creates() {
        // FR-PROJ-010 compares the owner against this value.
        let scratch = Scratch::new();
        let file = scratch.file("owned.cfg", "");

        let metadata = std::fs::metadata(&file).expect("the file was just created");
        assert_eq!(
            std::os::unix::fs::MetadataExt::uid(&metadata),
            invoking_user()
        );
    }
}
