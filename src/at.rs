//! Directory-relative file operations below the canonical `.tpl`
//! (`FR-SEC-026`, `FR-SEC-027`, `FR-CDOC-017`, `FR-PROJ-030`, `FR-TMPL-033`;
//! `OD-24`, amended 2026-09-24).
//!
//! The cache, `.tpl/.cfg` and the templates are all read through here.
//!
//! Every operation here starts from a directory descriptor, never from a path
//! resolved by name at the moment of use:
//!
//! | Operation | How |
//! |---|---|
//! | Reach a directory | `open` of the anchor, then `openat` of each component with `O_DIRECTORY \| O_NOFOLLOW` |
//! | Read a file | `openat` with `O_NOFOLLOW \| O_NONBLOCK`, then `fstat` of the descriptor must say regular file |
//! | Write a file | `openat` of the temporary with `O_CREAT \| O_EXCL \| O_NOFOLLOW`, then `renameat` over the target |
//! | Remove | `unlinkat`, recursively for a directory, each child reached by `openat` with `O_NOFOLLOW` |
//! | Create a directory | `mkdirat` on the parent's descriptor |
//!
//! No component is followed, so a component swapped for a symbolic link
//! between one operation and the next cannot redirect the next: the operation
//! fails instead, with `ELOOP` or `ENOTDIR`, and [`Fault::Linked`] names the
//! link. `O_NONBLOCK` keeps a FIFO in a file's place from blocking the open,
//! and the type test on the descriptor refuses what was opened.
//!
//! **One step still resolves a path by name: listing a directory.** The
//! `alloc` feature of `rustix` that would supply `Dir` is not part of the
//! Stack, and `/dev/fd/N` does not list a directory on macOS, so the names are
//! enumerated with [`std::fs::read_dir`] and each is then re-examined relative
//! to the descriptor already opened ([`list`]). A directory swapped between
//! the open and the listing can make the listing name another directory's
//! members, but every name is then tested, read, written or removed inside
//! the directory the descriptor holds: nothing outside it is read, written
//! or removed.
//!
//! The anchor is the path the caller hands over — the canonical `.tpl` of
//! `FR-PROJ-009` in production. It is opened by path, with `O_NOFOLLOW` on its
//! last component; the components above it are the project's own location,
//! which `FR-PROJ-009` has already resolved.

use std::ffi::{OsStr, OsString};
use std::io;
use std::os::unix::ffi::OsStrExt as _;
use std::path::{Component, Path, PathBuf};

use rustix::fd::OwnedFd;
use rustix::fs::{self as rfs, AtFlags, FileType, Mode, OFlags, Stat};
use rustix::io::Errno;

/// How a directory is opened: for reading, never through a link, never as a
/// file.
const DIRECTORY: OFlags = OFlags::RDONLY
    .union(OFlags::DIRECTORY)
    .union(OFlags::NOFOLLOW)
    .union(OFlags::CLOEXEC);

/// How a file is opened for reading: never through a link, and never waiting
/// for a writer — a FIFO opened so returns at once and is refused by its type.
const READ: OFlags = OFlags::RDONLY
    .union(OFlags::NOFOLLOW)
    .union(OFlags::NONBLOCK)
    .union(OFlags::CLOEXEC);

/// How a temporary is created: a new file, never an existing name, and never
/// through a link. `O_EXCL` fails on any existing name, a dangling link
/// included.
const CREATE: OFlags = OFlags::WRONLY
    .union(OFlags::CREATE)
    .union(OFlags::EXCL)
    .union(OFlags::NOFOLLOW)
    .union(OFlags::CLOEXEC);

/// The mode a directory is created with, before the umask: `0o777`, what
/// `std::fs::create_dir_all` gives one.
const FOLDER_MODE: Mode = Mode::RWXU.union(Mode::RWXG).union(Mode::RWXO);

/// Why a path below the anchor could not be reached.
#[derive(Debug)]
pub(crate) enum Fault {
    /// A component is a symbolic link; the path is the link's.
    Linked(PathBuf),
    /// A component does not exist.
    Absent,
    /// A component is neither a directory nor a link.
    NotDirectory,
    /// Any other refusal, or a path that does not lie below the anchor.
    Io(io::Error),
}

impl Fault {
    /// The same fault as an [`io::Error`], for a caller that reports one.
    pub(crate) fn into_io(self) -> io::Error {
        match self {
            Self::Linked(_) => io::Error::from_raw_os_error(Errno::LOOP.raw_os_error()),
            Self::Absent => io::Error::from(io::ErrorKind::NotFound),
            Self::NotDirectory => io::Error::from_raw_os_error(Errno::NOTDIR.raw_os_error()),
            Self::Io(error) => error,
        }
    }
}

/// An [`Errno`] as the [`io::Error`] the standard library would carry.
fn io_error(errno: Errno) -> io::Error {
    io::Error::from_raw_os_error(errno.raw_os_error())
}

/// Opens the directory `directory`, which is `anchor` or lies below it, one
/// component at a time and following none.
pub(crate) fn directory(anchor: &Path, directory: &Path) -> Result<OwnedFd, Fault> {
    let below = directory
        .strip_prefix(anchor)
        .map_err(|_| Fault::Io(io::Error::from(io::ErrorKind::InvalidInput)))?;
    let mut fd =
        rfs::open(anchor.as_os_str().as_bytes(), DIRECTORY, Mode::empty()).map_err(|errno| {
            if errno == Errno::NOENT {
                Fault::Absent
            } else {
                Fault::Io(io_error(errno))
            }
        })?;
    let mut walked = anchor.to_path_buf();

    for component in below.components() {
        let Component::Normal(name) = component else {
            return Err(Fault::Io(io::Error::from(io::ErrorKind::InvalidInput)));
        };
        walked.push(name);
        fd = match rfs::openat(&fd, name.as_bytes(), DIRECTORY, Mode::empty()) {
            Ok(opened) => opened,
            Err(errno) => return Err(classify(&fd, name, errno, walked)),
        };
    }

    Ok(fd)
}

/// What a failed `openat` of the directory `name` under `parent` means.
///
/// `O_NOFOLLOW` reports a link as `ELOOP`, and `O_DIRECTORY` beside it may
/// report it as `ENOTDIR` on some targets, so a `ENOTDIR` is told apart by
/// examining the name itself, without following it.
fn classify(parent: &OwnedFd, name: &OsStr, errno: Errno, path: PathBuf) -> Fault {
    if errno == Errno::NOENT {
        return Fault::Absent;
    }
    if errno == Errno::LOOP {
        return Fault::Linked(path);
    }
    if errno == Errno::NOTDIR {
        return match rfs::statat(parent, name.as_bytes(), AtFlags::SYMLINK_NOFOLLOW) {
            Ok(found) if kind(&found).is_symlink() => Fault::Linked(path),
            _ => Fault::NotDirectory,
        };
    }

    Fault::Io(io_error(errno))
}

/// The directory `file` sits in, opened, and `file`'s own name.
pub(crate) fn parent<'f>(anchor: &Path, file: &'f Path) -> Result<(OwnedFd, &'f OsStr), Fault> {
    let (Some(folder), Some(name)) = (file.parent(), file.file_name()) else {
        return Err(Fault::Io(io::Error::from(io::ErrorKind::InvalidInput)));
    };

    Ok((directory(anchor, folder)?, name))
}

/// The type a [`Stat`] records.
pub(crate) fn kind(found: &Stat) -> FileType {
    FileType::from_raw_mode(found.st_mode)
}

/// What `name` under `folder` is, examined without following it.
pub(crate) fn examine(folder: &OwnedFd, name: &OsStr) -> Option<Stat> {
    rfs::statat(folder, name.as_bytes(), AtFlags::SYMLINK_NOFOLLOW).ok()
}

/// What `file` is, examined without following any component of its path.
pub(crate) fn examine_path(anchor: &Path, file: &Path) -> Option<Stat> {
    let (folder, name) = parent(anchor, file).ok()?;

    examine(&folder, name)
}

/// The names `folder` — opened as `descriptor` — holds, each examined relative
/// to the descriptor, or [`None`] where the folder cannot be listed.
///
/// The enumeration is by path, the one step of this module that is (see the
/// module's documentation); a name the descriptor's directory does not hold is
/// dropped, so every name answered is one of that directory.
pub(crate) fn list(folder: &Path, descriptor: &OwnedFd) -> Option<Vec<(OsString, Stat)>> {
    let mut held = Vec::new();

    for entry in std::fs::read_dir(folder).ok()? {
        let name = entry.ok()?.file_name();
        if let Some(found) = examine(descriptor, &name) {
            held.push((name, found));
        }
    }

    Some(held)
}

/// Why [`read_in`] read nothing.
#[derive(Debug)]
pub(crate) enum Unread {
    /// The name holds a file of another kind: a FIFO, a socket, a device, a
    /// directory, or — refused by the open itself — a symbolic link.
    NotRegular(FileType),
    /// The file is longer than the cap the caller gave.
    Oversized,
    /// The file could not be opened or read, or is not UTF-8.
    Io(io::Error),
}

/// The words a diagnostic names a kind of file with, as `FR-PROJ-030` and
/// `FR-TMPL-033` require the `cause` to name the kind found.
pub(crate) fn kind_name(kind: FileType) -> &'static str {
    if kind.is_fifo() {
        "a FIFO"
    } else if kind.is_socket() {
        "a socket"
    } else if kind.is_char_device() {
        "a character device"
    } else if kind.is_block_device() {
        "a block device"
    } else if kind.is_dir() {
        "a directory"
    } else if kind.is_symlink() {
        "a symbolic link"
    } else {
        "not a regular file"
    }
}

/// A regular file's contents, opened relative to `folder` without following a
/// link and without waiting for a writer; at most `cap` bytes where one is
/// given.
///
/// The type and the size are the descriptor's own, so they are those of the
/// file read, whatever the name pointed at before the open. A symbolic link
/// fails the open (`O_NOFOLLOW`) and is answered as the link it is.
///
/// # Errors
///
/// Returns [`Unread::NotRegular`] for a file of another kind,
/// [`Unread::Oversized`] for one longer than `cap`, and [`Unread::Io`] for a
/// file that cannot be opened or read or is not UTF-8.
pub(crate) fn read_checked(
    folder: &OwnedFd,
    name: &OsStr,
    cap: Option<u64>,
) -> Result<String, Unread> {
    let (handle, found) = open_regular(folder, name)?;

    read_open(&handle, &found, cap)
}

/// Opens `name` under `folder` for reading, without following a link and
/// without waiting for a writer, and answers the descriptor with its own
/// metadata where it is a regular file.
///
/// Nothing is read: a caller that must judge the file before reading it — the
/// trust checks of `FR-PROJ-010` and `FR-PROJ-011` — judges this metadata,
/// which is the file's that is then read through [`read_open`].
///
/// # Errors
///
/// Returns [`Unread::NotRegular`] for a file of another kind, a symbolic link
/// included, and [`Unread::Io`] for one that cannot be opened; an absent name
/// is an [`Unread::Io`] of kind [`io::ErrorKind::NotFound`].
pub(crate) fn open_regular(folder: &OwnedFd, name: &OsStr) -> Result<(OwnedFd, Stat), Unread> {
    let handle = match rfs::openat(folder, name.as_bytes(), READ, Mode::empty()) {
        Ok(handle) => handle,
        Err(Errno::LOOP) => return Err(Unread::NotRegular(FileType::Symlink)),
        Err(Errno::NOENT) => return Err(Unread::Io(io_error(Errno::NOENT))),
        // Some kinds refuse the open itself before any `fstat` can type them:
        // a socket is `EOPNOTSUPP` on Linux and `ENXIO` on others. The name is
        // then examined without following it, and a file of another kind is
        // answered as that kind rather than as a failure to read.
        Err(errno) => {
            return Err(match examine(folder, name) {
                Some(found) if !kind(&found).is_file() => Unread::NotRegular(kind(&found)),
                _ => Unread::Io(io_error(errno)),
            });
        }
    };
    let found = rfs::fstat(&handle).map_err(|errno| Unread::Io(io_error(errno)))?;
    if !kind(&found).is_file() {
        return Err(Unread::NotRegular(kind(&found)));
    }

    Ok((handle, found))
}

/// The contents of `handle`, a regular file whose metadata is `found`, at most
/// `cap` bytes where one is given.
///
/// # Errors
///
/// Returns [`Unread::Oversized`] for a file longer than `cap`, and
/// [`Unread::Io`] for one that cannot be read or is not UTF-8.
pub(crate) fn read_open(
    handle: &OwnedFd,
    found: &Stat,
    cap: Option<u64>,
) -> Result<String, Unread> {
    let length = u64::try_from(found.st_size).unwrap_or(0);
    if cap.is_some_and(|cap| length > cap) {
        return Err(Unread::Oversized);
    }

    // One byte past the cap is asked for, so a file that grew after the
    // `fstat` is refused rather than read short.
    let limit = cap.map_or(u64::MAX, |cap| cap.saturating_add(1));
    let bytes = read_all(handle, usize::try_from(length).unwrap_or(0), limit)
        .ok_or_else(|| Unread::Io(io::Error::from(io::ErrorKind::Other)))?;
    if cap.is_some_and(|cap| u64::try_from(bytes.len()).unwrap_or(u64::MAX) > cap) {
        return Err(Unread::Oversized);
    }

    String::from_utf8(bytes).map_err(|_| Unread::Io(io::Error::from(io::ErrorKind::InvalidData)))
}

/// The permission bits of `found`, as the integer `chmod` writes, from the
/// flags [`Mode`] names.
///
/// The raw `st_mode` is 16 bits wide on macOS and 32 on Linux, so the bits are
/// assembled from the named flags rather than cast from the raw value.
pub(crate) fn permissions(found: &Stat) -> u32 {
    let mode = Mode::from_raw_mode(found.st_mode);

    [
        (Mode::SUID, 0o4000),
        (Mode::SGID, 0o2000),
        (Mode::SVTX, 0o1000),
        (Mode::RUSR, 0o400),
        (Mode::WUSR, 0o200),
        (Mode::XUSR, 0o100),
        (Mode::RGRP, 0o040),
        (Mode::WGRP, 0o020),
        (Mode::XGRP, 0o010),
        (Mode::ROTH, 0o004),
        (Mode::WOTH, 0o002),
        (Mode::XOTH, 0o001),
    ]
    .iter()
    .filter(|(flag, _)| mode.contains(*flag))
    .map(|(_, bit)| *bit)
    .sum()
}

/// [`read_checked`], with every failure the same [`None`]: the answer a
/// cache read gives, where each is a miss.
pub(crate) fn read_in(folder: &OwnedFd, name: &OsStr, cap: Option<u64>) -> Option<String> {
    read_checked(folder, name, cap).ok()
}

/// A regular file's contents at `file`, on the terms of [`read_in`].
pub(crate) fn read(anchor: &Path, file: &Path, cap: Option<u64>) -> Option<String> {
    let (folder, name) = parent(anchor, file).ok()?;

    read_in(&folder, name, cap)
}

/// Every byte `handle` yields, up to `limit`, into a buffer sized for
/// `expected`.
fn read_all(handle: &OwnedFd, expected: usize, limit: u64) -> Option<Vec<u8>> {
    let mut bytes = Vec::with_capacity(expected);
    let mut chunk = [0_u8; 8192];

    loop {
        let read = match rustix::io::read(handle, &mut chunk) {
            Ok(read) => read,
            Err(Errno::INTR) => continue,
            Err(_) => return None,
        };
        if read == 0 {
            return Some(bytes);
        }
        bytes.extend_from_slice(chunk.get(..read)?);
        if u64::try_from(bytes.len()).unwrap_or(u64::MAX) >= limit {
            return Some(bytes);
        }
    }
}

/// Whether `name` under `folder` is a regular file of mode `mode` holding
/// exactly `encoded`, read into `held`.
///
/// The type, the mode and the length are the descriptor's own, so the file
/// compared is the file examined.
pub(crate) fn holds_bytes(
    folder: &OwnedFd,
    name: &OsStr,
    mode: Mode,
    encoded: &[u8],
    held: &mut Vec<u8>,
) -> bool {
    let Ok(handle) = rfs::openat(folder, name.as_bytes(), READ, Mode::empty()) else {
        return false;
    };
    let Ok(found) = rfs::fstat(&handle) else {
        return false;
    };
    let (Ok(length), Ok(expected)) = (u64::try_from(found.st_size), u64::try_from(encoded.len()))
    else {
        return false;
    };
    if !kind(&found).is_file() || Mode::from_raw_mode(found.st_mode) != mode || length != expected {
        return false;
    }

    let Some(bytes) = read_all(&handle, encoded.len(), expected.saturating_add(1)) else {
        return false;
    };
    held.clear();
    held.extend_from_slice(&bytes);

    held.as_slice() == encoded
}

/// Writes `bytes` to `name` under `folder` through the temporary `temporary`
/// in the same folder, renamed over the target (`FR-CACHE-030`).
///
/// Whatever stands at the temporary's name is removed first, as a link where
/// it is one, and the temporary is then created exclusively, so the write
/// never opens through a link. Answers whether the file is now stored; a
/// failure leaves the target untouched and the temporary removed.
pub(crate) fn write_in(
    folder: &OwnedFd,
    name: &OsStr,
    temporary: &OsStr,
    mode: Mode,
    bytes: &[u8],
) -> bool {
    let _ = rfs::unlinkat(folder, temporary.as_bytes(), AtFlags::empty());

    let written = (|| -> Result<(), Errno> {
        let handle = rfs::openat(folder, temporary.as_bytes(), CREATE, mode)?;
        let mut rest = bytes;
        while !rest.is_empty() {
            match rustix::io::write(&handle, rest) {
                Ok(0) => return Err(Errno::IO),
                Ok(count) => rest = rest.get(count..).unwrap_or_default(),
                Err(Errno::INTR) => {}
                Err(errno) => return Err(errno),
            }
        }
        drop(handle);

        rfs::renameat(folder, temporary.as_bytes(), folder, name.as_bytes())
    })();

    if written.is_err() {
        let _ = rfs::unlinkat(folder, temporary.as_bytes(), AtFlags::empty());
        return false;
    }

    true
}

/// Opens `directory` below `anchor`, creating each missing component with
/// `mkdirat` on its parent's descriptor, and following none.
pub(crate) fn make_directories(anchor: &Path, directory: &Path) -> Result<OwnedFd, Fault> {
    let below = directory
        .strip_prefix(anchor)
        .map_err(|_| Fault::Io(io::Error::from(io::ErrorKind::InvalidInput)))?;
    let mut fd = rfs::open(anchor.as_os_str().as_bytes(), DIRECTORY, Mode::empty())
        .map_err(|errno| Fault::Io(io_error(errno)))?;
    let mut walked = anchor.to_path_buf();

    for component in below.components() {
        let Component::Normal(name) = component else {
            return Err(Fault::Io(io::Error::from(io::ErrorKind::InvalidInput)));
        };
        walked.push(name);
        fd = match rfs::openat(&fd, name.as_bytes(), DIRECTORY, Mode::empty()) {
            Ok(opened) => opened,
            Err(Errno::NOENT) => {
                match rfs::mkdirat(&fd, name.as_bytes(), FOLDER_MODE) {
                    Ok(()) | Err(Errno::EXIST) => {}
                    Err(errno) => return Err(Fault::Io(io_error(errno))),
                }
                rfs::openat(&fd, name.as_bytes(), DIRECTORY, Mode::empty())
                    .map_err(|errno| classify(&fd, name, errno, walked.clone()))?
            }
            Err(errno) => return Err(classify(&fd, name, errno, walked)),
        };
    }

    Ok(fd)
}

/// Removes `name` under `folder`: a link or any other file as itself, and a
/// directory with everything beneath it, reached by `openat` without
/// following a link and removed with `unlinkat`.
///
/// `path` is `name`'s path, used only to enumerate a directory's names, per
/// [`list`].
///
/// # Errors
///
/// Returns the first refusal met. A name already gone is not one.
pub(crate) fn remove_in(folder: &OwnedFd, name: &OsStr, path: &Path) -> io::Result<()> {
    let Some(found) = examine(folder, name) else {
        return Ok(());
    };

    if kind(&found).is_dir() {
        let descriptor = match rfs::openat(folder, name.as_bytes(), DIRECTORY, Mode::empty()) {
            Ok(opened) => opened,
            Err(Errno::NOENT) => return Ok(()),
            // Swapped for something else since it was examined: whatever it is
            // now is removed as itself, and nothing it points at is.
            Err(Errno::LOOP | Errno::NOTDIR) => return unlink(folder, name, AtFlags::empty()),
            Err(errno) => return Err(io_error(errno)),
        };
        for (child, _) in list(path, &descriptor).unwrap_or_default() {
            remove_in(&descriptor, &child, &path.join(&child))?;
        }

        return unlink(folder, name, AtFlags::REMOVEDIR);
    }

    unlink(folder, name, AtFlags::empty())
}

/// `unlinkat`, with a name already gone taken as removed.
pub(crate) fn unlink(folder: &OwnedFd, name: &OsStr, flags: AtFlags) -> io::Result<()> {
    match rfs::unlinkat(folder, name.as_bytes(), flags) {
        Ok(()) | Err(Errno::NOENT) => Ok(()),
        Err(errno) => Err(io_error(errno)),
    }
}
