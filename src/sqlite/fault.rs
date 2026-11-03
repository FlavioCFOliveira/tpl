//! Where the library's result stops and a condition of `FR-ERR-001` begins
//! (`FR-LITE-040`, `FR-LITE-007`, `FR-LITE-012`, `FR-GLOB-018`, `OD-38`).
//!
//! One pure function, [`classify`], over the stage or statement that failed
//! and the integer the library returned, applies the three steps of
//! `FR-LITE-040` in their order; [`condition`] turns its answer into the
//! [`Error`] the caller reads. The classification is keyed on the extended
//! code the driver's `ffi::Error` carries — the open sets
//! `SQLITE_OPEN_EXRESCODE` — and its low eight bits are the primary code; the
//! driver's own `ErrorCode` is not consulted, because it sends five primary
//! codes to one variant and drops the extended code the rows of
//! `SQLITE_READONLY` turn on (`OD-38`).
//!
//! **What a `cause` carries of the library** is the two facts `FR-GLOB-018`
//! admits: the code, by the name SQLite gives it — taken from [`name`], which
//! holds every primary and extended code SQLite's result-code page lists,
//! because neither the library nor the driver supplies a symbolic name — and
//! the library's message. The driver's error value is dropped here, in every
//! rendering, so no `Display` of it reaches a stream.

use std::path::Path;

use rusqlite::ffi;

use crate::deadline::Bound;
use crate::error::{Error, FileValue, LibraryReport, SqliteAt, SqliteEnd};

/// Where on a connection a failure arose, which `FR-LITE-040` step 1
/// classifies by.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Stage {
    /// The read-only open (stage 2).
    Open,
    /// `PRAGMA compile_options` (stage 4).
    Build,
    /// A control or a limit, set or read back (stage 5).
    Control,
    /// The version probe (stage 6).
    Probe,
    /// A catalogue statement, described as the `cause` names it.
    Catalogue(&'static str),
}

impl Stage {
    /// The open or the statement, as a `cause` names it.
    pub(crate) const fn at(self) -> SqliteAt {
        match self {
            Self::Open => SqliteAt::Open,
            Self::Build => SqliteAt::Statement("PRAGMA compile_options"),
            Self::Control => SqliteAt::Statement("a hardening control of the connection"),
            Self::Probe => SqliteAt::Statement("the version probe SELECT sqlite_version()"),
            Self::Catalogue(statement) => SqliteAt::Statement(statement),
        }
    }

    /// The statement, as the `69` of a `sqlite` entry names it.
    const fn statement(self) -> &'static str {
        match self.at() {
            SqliteAt::Open => "the read-only open",
            SqliteAt::Statement(statement) => statement,
        }
    }
}

/// What `FR-LITE-040` makes of a failure.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Class {
    /// Step 1: a control or a limit did not take effect — `70`
    /// (`FR-LITE-012`).
    Control,
    /// The rows of `FR-LITE-007` and of the table that give `74`.
    Unreadable,
    /// The rows of `FR-LITE-007` that give `78`.
    Refused,
    /// `SQLITE_BUSY` — `69`.
    Locked,
    /// `SQLITE_PROTOCOL` or `SQLITE_SCHEMA` — `69`.
    Contended,
    /// `SQLITE_INTERRUPT`: the catalogue-query deadline ended the statement —
    /// `69`.
    Interrupted,
    /// A use of the library by `tpl` that it refuses, a malfunction of the
    /// library, an outcome only an operation `tpl` never performs produces, or
    /// a code SQLite had not published — `70`, with the invariant it breaks.
    Defect(&'static str),
}

/// The invariant a misuse of the library breaks.
const MISUSE: &str = "tpl uses the SQLite library only in ways the library supports";

/// The invariant `SQLITE_ERROR` breaks, on any statement: no statement that
/// remains reads a view's columns or compiles a view (`FR-LITE-015`,
/// `FR-LITE-033`), so none can fail on a view that does not compile.
const STATEMENT_ERROR: &str =
    "every statement of FR-LITE-015 runs without error over a file whose schema has loaded";

/// The invariant `SQLITE_INTERNAL` breaks.
const INTERNAL: &str = "the SQLite library the binary carries does not malfunction";

/// The invariant an outcome of an operation `tpl` never performs breaks.
const NEVER_PERFORMED: &str =
    "a read-only read on one connection produces no write, abort, constraint or lock conflict";

/// The invariant a code the library documents as unused breaks.
const UNUSED: &str = "the SQLite library returns no code it documents as unused";

/// The invariant an unpublished primary code breaks.
const UNPUBLISHED: &str = "the SQLite library returns only result codes SQLite has published";

/// Classifies a failure by the stage it arose at and the code the library
/// returned (`FR-LITE-040`), applying the three steps in their order.
///
/// `code` is the extended code; its low eight bits are the primary one.
pub(crate) const fn classify(stage: Stage, code: i32) -> Class {
    let primary = code & 0xff;

    // Step 1: FR-LITE-012, whatever the library returned there.
    if matches!(stage, Stage::Build | Stage::Control) {
        return Class::Control;
    }

    // Step 2: the rows of FR-LITE-007, by result code, wherever returned.
    if code == ffi::SQLITE_READONLY_DIRECTORY {
        return Class::Unreadable;
    }
    match primary {
        ffi::SQLITE_CANTOPEN | ffi::SQLITE_IOERR => return Class::Unreadable,
        ffi::SQLITE_NOTADB | ffi::SQLITE_CORRUPT | ffi::SQLITE_TOOBIG => return Class::Refused,
        ffi::SQLITE_BUSY => return Class::Locked,
        _ => {}
    }

    // Step 3: the table of FR-LITE-040, by the primary code.
    match primary {
        ffi::SQLITE_MISUSE | ffi::SQLITE_RANGE | ffi::SQLITE_NOTFOUND => Class::Defect(MISUSE),
        ffi::SQLITE_ERROR => Class::Defect(STATEMENT_ERROR),
        ffi::SQLITE_INTERNAL => Class::Defect(INTERNAL),
        // SQLITE_READONLY under an extended code of its own is the host's.
        ffi::SQLITE_READONLY if code != ffi::SQLITE_READONLY => Class::Unreadable,
        ffi::SQLITE_ABORT
        | ffi::SQLITE_AUTH
        | ffi::SQLITE_CONSTRAINT
        | ffi::SQLITE_LOCKED
        | ffi::SQLITE_MISMATCH
        | ffi::SQLITE_READONLY => Class::Defect(NEVER_PERFORMED),
        ffi::SQLITE_EMPTY | ffi::SQLITE_FORMAT | ffi::SQLITE_NOTICE | ffi::SQLITE_WARNING => {
            Class::Defect(UNUSED)
        }
        ffi::SQLITE_NOMEM | ffi::SQLITE_FULL | ffi::SQLITE_NOLFS | ffi::SQLITE_PERM => {
            Class::Unreadable
        }
        ffi::SQLITE_PROTOCOL | ffi::SQLITE_SCHEMA => Class::Contended,
        ffi::SQLITE_INTERRUPT => Class::Interrupted,
        _ => Class::Defect(UNPUBLISHED),
    }
}

/// What the driver reported, reduced to the two facts `FR-GLOB-018` admits,
/// or [`None`] where it carries no result code.
///
/// `SqliteFailure` and `SqlInputError` are the two variants that carry one;
/// the second is what the driver returns in place of the first when the
/// library fails to prepare a statement (`OD-38`, point 2). Every other
/// variant — the driver marks the enumeration non-exhaustive — carries none.
pub(crate) fn report(error: &rusqlite::Error) -> Option<LibraryReport> {
    let (code, message) = match error {
        rusqlite::Error::SqliteFailure(failure, message) => {
            (failure.extended_code, message.clone())
        }
        rusqlite::Error::SqlInputError { error, msg, .. } => {
            (error.extended_code, Some(msg.clone()))
        }
        _ => return None,
    };

    Some(LibraryReport {
        code,
        message: message.map(FileValue::new),
    })
}

/// Where a failure arose, and what the condition a caller reads must name.
#[derive(Debug, Clone, Copy)]
pub(crate) struct Place<'a> {
    /// The entry the read is made through.
    pub(crate) entry: &'a str,
    /// The database file, as resolved.
    pub(crate) path: &'a Path,
}

/// A statement's bound, and whether it was spent when the statement returned
/// (`FR-LITE-009`, `OD-36`).
#[derive(Debug, Clone, Copy)]
pub(crate) struct Timing {
    /// The bound the statement ran under.
    pub(crate) bound: Bound,
    /// Whether the bound was spent when the library returned.
    pub(crate) spent: bool,
}

/// The condition a failure of the library is, under [`classify`]
/// (`FR-LITE-040`).
///
/// `timing` is the statement's bound, which a `69` names where it was spent.
#[track_caller]
pub(crate) fn condition(
    place: Place<'_>,
    stage: Stage,
    error: &rusqlite::Error,
    timing: Option<Timing>,
) -> Error {
    let Some(report) = report(error) else {
        // OD-38 point 4: a driver error carrying no result code is a violated
        // internal invariant.
        return defect(
            place,
            stage,
            "every failure of the SQLite driver carries a result code",
            None,
        );
    };

    match classify(stage, report.code) {
        Class::Control => Error::SqliteControlNotInEffect {
            entry: place.entry.to_owned(),
            path: place.path.to_owned(),
            control: control_of(stage),
            set: "the value FR-LITE-010 and FR-LITE-011 fix",
            read_back: FileValue::new(report.to_string()),
        },
        Class::Unreadable => Error::SqliteFileUnreadable {
            entry: place.entry.to_owned(),
            path: place.path.to_owned(),
            at: stage.at(),
            report,
        },
        Class::Refused => Error::SqliteFileRefused {
            entry: place.entry.to_owned(),
            path: place.path.to_owned(),
            at: stage.at(),
            report,
        },
        Class::Locked => Error::SqliteUnavailable {
            entry: place.entry.to_owned(),
            path: place.path.to_owned(),
            statement: stage.statement(),
            ended: SqliteEnd::Locked {
                report: Box::new(report),
                waited: timing
                    .filter(|timing| timing.spent)
                    .map(|timing| (timing.bound.bound(), timing.bound.limit())),
            },
        },
        Class::Contended => Error::SqliteUnavailable {
            entry: place.entry.to_owned(),
            path: place.path.to_owned(),
            statement: stage.statement(),
            ended: SqliteEnd::Contended {
                report: Box::new(report),
            },
        },
        Class::Interrupted => match timing {
            Some(timing) => expired(place, stage, timing.bound),
            None => defect(
                place,
                stage,
                "the library is interrupted only at a statement's deadline",
                Some(report),
            ),
        },
        Class::Defect(invariant) => defect(place, stage, invariant, Some(report)),
    }
}

/// The `69` of a statement whose bound expired, before it began or at the
/// watchdog's interrupt (`FR-LITE-009`, `FR-ERR-027`, `OD-36`).
pub(crate) fn expired(place: Place<'_>, stage: Stage, bound: Bound) -> Error {
    Error::SqliteUnavailable {
        entry: place.entry.to_owned(),
        path: place.path.to_owned(),
        statement: stage.statement(),
        ended: SqliteEnd::Deadline {
            bound: bound.bound(),
            limit: bound.limit(),
        },
    }
}

/// The `70` of a violated invariant on a `sqlite` read (`FR-ERR-030`).
#[track_caller]
pub(crate) fn defect(
    place: Place<'_>,
    stage: Stage,
    invariant: &'static str,
    report: Option<LibraryReport>,
) -> Error {
    Error::SqliteDefect {
        entry: place.entry.to_owned(),
        path: place.path.to_owned(),
        at: stage.at(),
        invariant,
        report: report.map(Box::new),
        location: std::panic::Location::caller(),
    }
}

/// The control a failure at `stage` concerns, where it is one of
/// `FR-LITE-012`'s stages.
const fn control_of(stage: Stage) -> &'static str {
    match stage {
        Stage::Build => "the build of the library (PRAGMA compile_options)",
        _ => "a hardening control or limit",
    }
}

/// SQLite's name for `code`, from the result-code page
/// (`https://sqlite.org/rescode.html`, last updated 2026-02-18, consulted
/// 2026-11-03).
///
/// An extended code missing from the table is written as its primary code's
/// name and the number (`OD-38`, point 5).
pub(crate) fn name(code: i32) -> String {
    match known(code) {
        Some(name) => name.to_owned(),
        None => match known(code & 0xff) {
            Some(primary) => format!("{primary} (extended code {code})"),
            None => format!("an unpublished result code {code}"),
        },
    }
}

/// The name of every primary and extended code the page lists.
const fn known(code: i32) -> Option<&'static str> {
    Some(match code {
        0 => "SQLITE_OK",
        1 => "SQLITE_ERROR",
        2 => "SQLITE_INTERNAL",
        3 => "SQLITE_PERM",
        4 => "SQLITE_ABORT",
        5 => "SQLITE_BUSY",
        6 => "SQLITE_LOCKED",
        7 => "SQLITE_NOMEM",
        8 => "SQLITE_READONLY",
        9 => "SQLITE_INTERRUPT",
        10 => "SQLITE_IOERR",
        11 => "SQLITE_CORRUPT",
        12 => "SQLITE_NOTFOUND",
        13 => "SQLITE_FULL",
        14 => "SQLITE_CANTOPEN",
        15 => "SQLITE_PROTOCOL",
        16 => "SQLITE_EMPTY",
        17 => "SQLITE_SCHEMA",
        18 => "SQLITE_TOOBIG",
        19 => "SQLITE_CONSTRAINT",
        20 => "SQLITE_MISMATCH",
        21 => "SQLITE_MISUSE",
        22 => "SQLITE_NOLFS",
        23 => "SQLITE_AUTH",
        24 => "SQLITE_FORMAT",
        25 => "SQLITE_RANGE",
        26 => "SQLITE_NOTADB",
        27 => "SQLITE_NOTICE",
        28 => "SQLITE_WARNING",
        100 => "SQLITE_ROW",
        101 => "SQLITE_DONE",
        256 => "SQLITE_OK_LOAD_PERMANENTLY",
        257 => "SQLITE_ERROR_MISSING_COLLSEQ",
        261 => "SQLITE_BUSY_RECOVERY",
        262 => "SQLITE_LOCKED_SHAREDCACHE",
        264 => "SQLITE_READONLY_RECOVERY",
        266 => "SQLITE_IOERR_READ",
        267 => "SQLITE_CORRUPT_VTAB",
        270 => "SQLITE_CANTOPEN_NOTEMPDIR",
        275 => "SQLITE_CONSTRAINT_CHECK",
        279 => "SQLITE_AUTH_USER",
        283 => "SQLITE_NOTICE_RECOVER_WAL",
        284 => "SQLITE_WARNING_AUTOINDEX",
        512 => "SQLITE_OK_SYMLINK",
        513 => "SQLITE_ERROR_RETRY",
        516 => "SQLITE_ABORT_ROLLBACK",
        517 => "SQLITE_BUSY_SNAPSHOT",
        518 => "SQLITE_LOCKED_VTAB",
        520 => "SQLITE_READONLY_CANTLOCK",
        522 => "SQLITE_IOERR_SHORT_READ",
        523 => "SQLITE_CORRUPT_SEQUENCE",
        526 => "SQLITE_CANTOPEN_ISDIR",
        531 => "SQLITE_CONSTRAINT_COMMITHOOK",
        539 => "SQLITE_NOTICE_RECOVER_ROLLBACK",
        769 => "SQLITE_ERROR_SNAPSHOT",
        773 => "SQLITE_BUSY_TIMEOUT",
        776 => "SQLITE_READONLY_ROLLBACK",
        778 => "SQLITE_IOERR_WRITE",
        779 => "SQLITE_CORRUPT_INDEX",
        782 => "SQLITE_CANTOPEN_FULLPATH",
        787 => "SQLITE_CONSTRAINT_FOREIGNKEY",
        795 => "SQLITE_NOTICE_RBU",
        1025 => "SQLITE_ERROR_RESERVESIZE",
        1032 => "SQLITE_READONLY_DBMOVED",
        1034 => "SQLITE_IOERR_FSYNC",
        1038 => "SQLITE_CANTOPEN_CONVPATH",
        1043 => "SQLITE_CONSTRAINT_FUNCTION",
        1281 => "SQLITE_ERROR_KEY",
        1288 => "SQLITE_READONLY_CANTINIT",
        1290 => "SQLITE_IOERR_DIR_FSYNC",
        1294 => "SQLITE_CANTOPEN_DIRTYWAL",
        1299 => "SQLITE_CONSTRAINT_NOTNULL",
        1537 => "SQLITE_ERROR_UNABLE",
        1544 => "SQLITE_READONLY_DIRECTORY",
        1546 => "SQLITE_IOERR_TRUNCATE",
        1550 => "SQLITE_CANTOPEN_SYMLINK",
        1555 => "SQLITE_CONSTRAINT_PRIMARYKEY",
        1802 => "SQLITE_IOERR_FSTAT",
        1811 => "SQLITE_CONSTRAINT_TRIGGER",
        2058 => "SQLITE_IOERR_UNLOCK",
        2067 => "SQLITE_CONSTRAINT_UNIQUE",
        2314 => "SQLITE_IOERR_RDLOCK",
        2323 => "SQLITE_CONSTRAINT_VTAB",
        2570 => "SQLITE_IOERR_DELETE",
        2579 => "SQLITE_CONSTRAINT_ROWID",
        2826 => "SQLITE_IOERR_BLOCKED",
        2835 => "SQLITE_CONSTRAINT_PINNED",
        3082 => "SQLITE_IOERR_NOMEM",
        3091 => "SQLITE_CONSTRAINT_DATATYPE",
        3338 => "SQLITE_IOERR_ACCESS",
        3594 => "SQLITE_IOERR_CHECKRESERVEDLOCK",
        3850 => "SQLITE_IOERR_LOCK",
        4106 => "SQLITE_IOERR_CLOSE",
        4362 => "SQLITE_IOERR_DIR_CLOSE",
        4618 => "SQLITE_IOERR_SHMOPEN",
        4874 => "SQLITE_IOERR_SHMSIZE",
        5130 => "SQLITE_IOERR_SHMLOCK",
        5386 => "SQLITE_IOERR_SHMMAP",
        5642 => "SQLITE_IOERR_SEEK",
        5898 => "SQLITE_IOERR_DELETE_NOENT",
        6154 => "SQLITE_IOERR_MMAP",
        6410 => "SQLITE_IOERR_GETTEMPPATH",
        6666 => "SQLITE_IOERR_CONVPATH",
        6922 => "SQLITE_IOERR_VNODE",
        7178 => "SQLITE_IOERR_AUTH",
        7434 => "SQLITE_IOERR_BEGIN_ATOMIC",
        7690 => "SQLITE_IOERR_COMMIT_ATOMIC",
        7946 => "SQLITE_IOERR_ROLLBACK_ATOMIC",
        8202 => "SQLITE_IOERR_DATA",
        8458 => "SQLITE_IOERR_CORRUPTFS",
        8714 => "SQLITE_IOERR_IN_PAGE",
        8970 => "SQLITE_IOERR_BADKEY",
        9226 => "SQLITE_IOERR_CODEC",
        _ => return None,
    })
}

#[cfg(test)]
mod tests {
    use super::{Class, Stage, classify, name};
    use rusqlite::ffi;

    /// Every stage outside step 1: neither the build nor a control.
    const ORDINARY: [Stage; 3] = [Stage::Open, Stage::Probe, Stage::Catalogue("statement 3")];

    #[test]
    fn fr_lite_012_any_code_at_the_build_or_a_control_is_a_control_not_in_effect() {
        // FR-LITE-040 step 1: whatever the library returned there.
        for code in [
            ffi::SQLITE_ERROR,
            ffi::SQLITE_NOTADB,
            ffi::SQLITE_BUSY,
            ffi::SQLITE_READONLY_DIRECTORY,
            9999,
        ] {
            assert_eq!(classify(Stage::Build, code), Class::Control, "{code}");
            assert_eq!(classify(Stage::Control, code), Class::Control, "{code}");
        }
    }

    #[test]
    fn fr_lite_040_sqlite_error_on_any_statement_is_a_defect_of_tpl() {
        // FR-LITE-040 as amended in the sixty-eighth edition: the step that
        // gave SQLITE_ERROR on a view statement to FR-LITE-035 is removed, and
        // the SQLITE_ERROR row takes every one with 70, under any extended
        // code, wherever it is returned outside step 1.
        for stage in ORDINARY {
            for code in [
                ffi::SQLITE_ERROR,
                ffi::SQLITE_ERROR_MISSING_COLLSEQ,
                ffi::SQLITE_ERROR_RETRY,
                ffi::SQLITE_ERROR_SNAPSHOT,
            ] {
                assert_eq!(
                    classify(stage, code),
                    Class::Defect(super::STATEMENT_ERROR),
                    "{code} at {stage:?}"
                );
            }
        }
    }

    #[test]
    fn fr_lite_007_each_row_is_read_by_its_result_code_wherever_it_is_returned() {
        // FR-LITE-040 step 2, over every stage but the two of step 1.
        for stage in ORDINARY {
            assert_eq!(classify(stage, ffi::SQLITE_CANTOPEN), Class::Unreadable);
            assert_eq!(
                classify(stage, ffi::SQLITE_CANTOPEN_ISDIR),
                Class::Unreadable
            );
            assert_eq!(classify(stage, ffi::SQLITE_NOTADB), Class::Refused);
            assert_eq!(classify(stage, ffi::SQLITE_CORRUPT), Class::Refused);
            assert_eq!(classify(stage, ffi::SQLITE_CORRUPT_INDEX), Class::Refused);
            assert_eq!(classify(stage, ffi::SQLITE_TOOBIG), Class::Refused);
            assert_eq!(
                classify(stage, ffi::SQLITE_READONLY_DIRECTORY),
                Class::Unreadable
            );
            assert_eq!(classify(stage, ffi::SQLITE_IOERR), Class::Unreadable);
            assert_eq!(classify(stage, ffi::SQLITE_IOERR_LOCK), Class::Unreadable);
            assert_eq!(classify(stage, ffi::SQLITE_BUSY), Class::Locked);
            assert_eq!(classify(stage, ffi::SQLITE_BUSY_TIMEOUT), Class::Locked);
        }
    }

    #[test]
    fn fr_lite_040_every_other_published_code_takes_the_code_its_row_gives() {
        // FR-LITE-040 step 3.
        for stage in ORDINARY {
            for code in [
                ffi::SQLITE_MISUSE,
                ffi::SQLITE_RANGE,
                ffi::SQLITE_NOTFOUND,
                ffi::SQLITE_INTERNAL,
                ffi::SQLITE_ABORT,
                ffi::SQLITE_ABORT_ROLLBACK,
                ffi::SQLITE_AUTH,
                ffi::SQLITE_CONSTRAINT,
                ffi::SQLITE_CONSTRAINT_UNIQUE,
                ffi::SQLITE_LOCKED,
                ffi::SQLITE_LOCKED_SHAREDCACHE,
                ffi::SQLITE_MISMATCH,
                ffi::SQLITE_READONLY,
                ffi::SQLITE_EMPTY,
                ffi::SQLITE_FORMAT,
                ffi::SQLITE_NOTICE,
                ffi::SQLITE_WARNING,
            ] {
                assert!(
                    matches!(classify(stage, code), Class::Defect(_)),
                    "{code} at {stage:?}"
                );
            }
            for code in [
                ffi::SQLITE_NOMEM,
                ffi::SQLITE_FULL,
                ffi::SQLITE_NOLFS,
                ffi::SQLITE_PERM,
                ffi::SQLITE_READONLY_RECOVERY,
                ffi::SQLITE_READONLY_CANTLOCK,
                ffi::SQLITE_READONLY_ROLLBACK,
                ffi::SQLITE_READONLY_DBMOVED,
                ffi::SQLITE_READONLY_CANTINIT,
            ] {
                assert_eq!(classify(stage, code), Class::Unreadable, "{code}");
            }
            assert_eq!(classify(stage, ffi::SQLITE_PROTOCOL), Class::Contended);
            assert_eq!(classify(stage, ffi::SQLITE_SCHEMA), Class::Contended);
            assert_eq!(classify(stage, ffi::SQLITE_INTERRUPT), Class::Interrupted);
        }
    }

    #[test]
    fn fr_lite_040_a_primary_code_sqlite_has_not_published_is_a_defect() {
        for code in [29, 99, 200, 255, 29 | 0x100] {
            assert!(matches!(
                classify(Stage::Probe, code),
                Class::Defect(super::UNPUBLISHED)
            ));
        }
    }

    #[test]
    fn fr_glob_018_a_code_is_named_as_sqlite_names_it_and_an_unknown_extended_code_by_its_primary()
    {
        assert_eq!(name(ffi::SQLITE_NOTADB), "SQLITE_NOTADB");
        assert_eq!(
            name(ffi::SQLITE_READONLY_DIRECTORY),
            "SQLITE_READONLY_DIRECTORY"
        );
        assert_eq!(
            name(26 | (40 << 8)),
            format!("SQLITE_NOTADB (extended code {})", 26 | (40 << 8))
        );
        assert_eq!(name(77), "an unpublished result code 77");
    }
}
