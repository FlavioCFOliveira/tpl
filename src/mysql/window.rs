//! The supported MySQL series of `FR-MYSQL-003`, as one ordered table.
//!
//! `BR-MYSQL-001` states the set once, in the specification, and
//! `FR-MYSQL-019` admits no flag or key that overrides it, so the only
//! admissible source is a table compiled into the binary, and this is that
//! table. The comparison is the one `FR-MYSQL-002` fixes — major number first,
//! then minor, each an integer — which is the comparison of `FR-SRV-021`, so
//! [`Series`] and its ordering are `crate::mariadb::window`'s.
//!
//! | Requirement | What this module does with it |
//! |---|---|
//! | `FR-MYSQL-003` — the two LTS series, on the verification date | [`WINDOW`], newest first |
//! | `FR-MYSQL-009` — a series not of the table and older than its newest is refused | [`standing`] answers [`None`] |
//! | `FR-MYSQL-010` — a series newer than the newest is read and marked | [`standing`] answers [`Standing::NewerThanSupported`] |
//! | `FR-MYSQL-009` — the `cause` names the series that are supported | [`SUPPORTED`] travels on the error value |
//!
//! *The table is stale on a date, not wrong* (`BR-MYSQL-001`, `FR-MYSQL-005`):
//! re-deriving it is an edit to `FR-MYSQL-003` first and to this array second.

use crate::mariadb::window::{Series, against};
use crate::model::server::Standing;

/// One row of the table: the series as `FR-MYSQL-003` spells it, and the two
/// numbers `FR-MYSQL-002` compares.
#[derive(Debug, Clone, Copy)]
struct Row {
    /// The spelling the message of `FR-MYSQL-009` lists.
    name: &'static str,
    /// The value the comparison is made over.
    series: Series,
}

impl Row {
    /// One row, with its spelling and its numbers stated together.
    const fn new(name: &'static str, major: u16, minor: u16) -> Self {
        Self {
            name,
            series: Series::new(major, minor),
        }
    }
}

/// The series of `FR-MYSQL-003`, newest first, as verified on 2026-11-04.
///
/// `FR-MYSQL-004` forbids reading its size as a quantity to preserve.
const WINDOW: [Row; 2] = [Row::new("9.7", 9, 7), Row::new("8.4", 8, 4)];

/// The spellings of [`WINDOW`], in the same order, which the `cause` of
/// `FR-MYSQL-009` lists.
pub(crate) const SUPPORTED: [&str; WINDOW.len()] = [WINDOW[0].name, WINDOW[1].name];

/// The newest series of the table.
fn newest() -> Series {
    // WINDOW is strictly descending, which a test asserts; the fallback is
    // unreachable from a non-empty array and keeps the module free of a panic.
    WINDOW.first().map_or(Series::new(0, 0), |row| row.series)
}

/// The standing of `series` against the table, or [`None`] where
/// `FR-MYSQL-009` refuses it.
///
/// A member of the table is `supported`; a series newer than its newest is
/// `newer_than_supported`, per `FR-MYSQL-010`, which reaches every Innovation
/// release numbered by the calendar; everything else — `8.0`, `9.0` to `9.6`,
/// and an LTS series that has left Extended Support — is refused by the one
/// rule.
pub(crate) fn standing(series: Series) -> Option<Standing> {
    // FR-MYSQL-043 applies FR-SRV-035 to a `mysql` read, through the seam it
    // authorises: the window `crate::mariadb::window::narrow_to` narrows.
    #[cfg(test)]
    if let Some(narrowed) = crate::mariadb::window::narrowed() {
        return against(series, narrowed, |member| member == narrowed);
    }

    against(series, newest(), |member| {
        WINDOW.iter().any(|row| row.series == member)
    })
}

#[cfg(test)]
mod tests {
    use super::{SUPPORTED, Series, WINDOW, newest, standing};
    use crate::model::server::Standing;

    /// The series a test names, parsed.
    fn series(identifier: &str) -> Series {
        Series::parse(identifier).expect("the test writes a well-formed identifier")
    }

    #[test]
    fn fr_mysql_003_the_table_is_the_two_lts_series_newest_first() {
        assert_eq!(SUPPORTED, ["9.7", "8.4"]);

        for row in WINDOW {
            assert_eq!(Series::parse(row.name), Some(row.series), "{}", row.name);
        }
        for pair in WINDOW.windows(2) {
            assert!(pair[0].series > pair[1].series);
        }
        assert_eq!(newest(), series("9.7"));
    }

    #[test]
    fn fr_mysql_002_series_compare_by_major_then_minor_as_integers() {
        // FR-MYSQL-002's own examples.
        assert!(series("8.4") < series("9.7"));
        assert!(series("9.7") < series("26.7"));
        assert!(series("26.7") < series("26.10"));
    }

    #[test]
    fn fr_mysql_009_a_series_outside_the_table_and_older_than_its_newest_is_refused() {
        // 8.0 and 5.7 left support; 9.0 to 9.6 are Innovation releases older
        // than the newest supported series.
        for identifier in ["8.0", "5.7", "9.0", "9.3", "9.6", "8.3"] {
            assert_eq!(standing(series(identifier)), None, "{identifier}");
        }
    }

    #[test]
    fn fr_mysql_010_the_table_is_supported_and_a_newer_series_is_read_and_marked() {
        for identifier in SUPPORTED {
            assert_eq!(standing(series(identifier)), Some(Standing::Supported));
        }

        // Every Innovation release numbered by the calendar is newer than the
        // newest series of the table, and is read and marked.
        for identifier in ["9.8", "26.7", "26.10", "27.4"] {
            assert_eq!(
                standing(series(identifier)),
                Some(Standing::NewerThanSupported),
                "{identifier}"
            );
        }
    }

    #[test]
    fn fr_mysql_043_the_seam_of_fr_srv_035_narrows_the_mysql_window_too() {
        // FR-SRV-035, for a `mysql` read: narrowed to 8.4, the newest real
        // series reads as newer than the window, and is marked.
        let _narrowed = crate::mariadb::window::narrow_to(series("8.4"));

        assert_eq!(standing(series("9.7")), Some(Standing::NewerThanSupported));
        assert_eq!(standing(series("8.4")), Some(Standing::Supported));
    }
}
