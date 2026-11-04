//! The kind of the database file, and the read-only open (`FR-LITE-005`,
//! `FR-LITE-006`).
//!
//! The kind is established with [`std::fs::metadata`], which "will traverse
//! symbolic links" and corresponds to `stat` on Unix (Rust standard library
//! documentation, consulted 2026-11-03): it opens nothing, so a FIFO cannot
//! block the check, and a link is followed, as `FR-LITE-005` requires. A lookup
//! that fails — the path names nothing, a link on it dangles, the filesystem
//! refuses it — opens nothing either, and is the first row of `FR-LITE-007`.
//!
//! The open follows, by name, under the flags the architecture fixes: read
//! only and never creating the file, single-threaded, with extended result
//! codes, and with neither `SQLITE_OPEN_URI` nor `SQLITE_OPEN_NOFOLLOW`. The
//! path handed to it is the absolute path of `FR-CONF-054`, which begins with
//! `/`, so SQLite reads it as an ordinary filename whatever its URI setting
//! (`https://sqlite.org/uri.html`, consulted 2026-11-03).
//!
//! *Accepted cost, stated in `FR-LITE-005`.* A file replaced by a FIFO between
//! the check and the open still blocks the open; the library takes a path and
//! not an open file.

use std::os::unix::fs::FileTypeExt as _;
use std::path::Path;

use rusqlite::{Connection, OpenFlags};

use crate::error::Error;

/// The kind check of `FR-LITE-005`.
///
/// # Errors
///
/// Returns [`Error::SqliteNotRegularFile`] — `78` — where the path names
/// anything but a regular file, and [`Error::SqliteFileLookup`] — `74` —
/// where the lookup fails.
pub(crate) fn kind(entry: &str, path: &Path) -> Result<(), Error> {
    let metadata = std::fs::metadata(path).map_err(|returned| Error::SqliteFileLookup {
        entry: entry.to_owned(),
        path: path.to_owned(),
        returned,
    })?;
    let found = metadata.file_type();

    if found.is_file() {
        return Ok(());
    }

    let kind = if found.is_dir() {
        "directory"
    } else if found.is_fifo() {
        "FIFO"
    } else if found.is_socket() {
        "socket"
    } else if found.is_char_device() {
        "character device"
    } else if found.is_block_device() {
        "block device"
    } else {
        "file of a kind that is not a regular file"
    };

    Err(Error::SqliteNotRegularFile {
        entry: entry.to_owned(),
        path: path.to_owned(),
        kind,
    })
}

/// The flags of the read-only open (architecture, *The SQLite reader*).
pub(crate) fn flags() -> OpenFlags {
    OpenFlags::SQLITE_OPEN_READ_ONLY
        | OpenFlags::SQLITE_OPEN_NO_MUTEX
        | OpenFlags::SQLITE_OPEN_EXRESCODE
}

/// Opens `path` read-only, never creating it (`FR-LITE-006`).
///
/// # Errors
///
/// Returns what the library returned, for the caller to classify under
/// `FR-LITE-040`.
pub(crate) fn open(path: &Path) -> rusqlite::Result<Connection> {
    Connection::open_with_flags(path, flags())
}

#[cfg(test)]
mod tests {
    use super::{flags, kind};
    use crate::error::Error;
    use crate::project::scratch::Scratch;
    use rusqlite::OpenFlags;

    #[test]
    fn fr_lite_006_the_open_is_read_only_never_creates_and_follows_links() {
        let flags = flags();

        assert!(flags.contains(OpenFlags::SQLITE_OPEN_READ_ONLY));
        assert!(!flags.contains(OpenFlags::SQLITE_OPEN_READ_WRITE));
        assert!(!flags.contains(OpenFlags::SQLITE_OPEN_CREATE));
        assert!(!flags.contains(OpenFlags::SQLITE_OPEN_URI));
        assert!(!flags.contains(OpenFlags::SQLITE_OPEN_NOFOLLOW));
        assert!(flags.contains(OpenFlags::SQLITE_OPEN_EXRESCODE));
    }

    #[test]
    fn fr_lite_005_a_directory_is_refused_by_its_kind_and_a_path_naming_nothing_by_its_lookup() {
        let scratch = Scratch::new();
        let directory = scratch.directory("a-directory");

        assert!(matches!(
            kind("ledger", &directory),
            Err(Error::SqliteNotRegularFile {
                kind: "directory",
                ..
            })
        ));
        assert!(matches!(
            kind("ledger", &scratch.path("absent.sqlite")),
            Err(Error::SqliteFileLookup { .. })
        ));
    }
}
