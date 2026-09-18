//! The supported version window of `FR-SRV-015`, as one ordered table.
//!
//! `BR-SRV-005` states the supported set exactly once, in the specification,
//! and `FR-SRV-020` rejects a flag or a configuration key that overrides it —
//! so the only admissible source is a table compiled into the binary, and this
//! is that table. It is **data**: the four series are four rows, the comparison
//! of `FR-SRV-021` is the derived ordering of two numbers, and no requirement
//! of this module is written as a literal scattered through a condition.
//!
//! | Requirement | What this module does with it |
//! |---|---|
//! | `FR-SRV-015` — the four series, on the verification date | [`WINDOW`], newest first |
//! | `FR-SRV-021` — ordered by major family, then series number | [`Series`] derives [`Ord`] over those two fields in that order |
//! | `FR-SRV-020` — a series below the window is refused | [`standing`] answers [`None`], and the caller raises the condition |
//! | `FR-SRV-031` — a series above the window is read and marked | [`standing`] answers [`Standing::NewerThanSupported`] |
//! | `FR-SRV-030` — the `cause` lists the series that are supported | [`SUPPORTED`] travels on the error value, which is where `BR-SRV-005` puts it |
//!
//! *The spellings and the numbers are one row, not two tables.* A row carries
//! the series as the specification writes it and the two numbers the
//! comparison needs, and [`SUPPORTED`] projects the first out of the same
//! array. A test reads every row's spelling back into its numbers, so a row
//! that disagrees with itself fails rather than silently ordering a series by
//! numbers the message does not name.
//!
//! *The table is stale on a date, not wrong.* `BR-SRV-004` says a maintainer
//! who finds it wrong has found it stale: the criterion is `FR-SRV-001` and
//! the table is its instance on the date `FR-SRV-015` records. Re-deriving it
//! is an edit to that requirement first and to this array second.

use crate::model::server::Standing;

/// The separator between the components of a series identifier.
const COMPONENT: char = '.';

/// A release series, ordered as `FR-SRV-021` orders one.
///
/// The derived ordering is the requirement: the fields are declared major
/// first and minor second, so `12.1` is newer than `11.8` and older than
/// `12.3`, and `10.11.14` and `10.11.2` are the same value because a patch
/// number is not a field of this type.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) struct Series {
    /// The major family.
    major: u16,
    /// The series number inside that family.
    minor: u16,
}

impl Series {
    /// A series, written as its two numbers.
    const fn new(major: u16, minor: u16) -> Self {
        Self { major, minor }
    }

    /// The series a `<major>.<minor>` identifier names.
    ///
    /// The argument is what [`crate::model::server::series_of`] read from the
    /// probe's answer, so it is already two components of ASCII digits; this
    /// answers [`None`] for anything else rather than assuming it.
    pub(crate) fn parse(identifier: &str) -> Option<Self> {
        let (major, minor) = identifier.split_once(COMPONENT)?;

        Some(Self {
            major: major.parse().ok()?,
            minor: minor.parse().ok()?,
        })
    }
}

/// One row of the window: the series as `FR-SRV-015` spells it, and the two
/// numbers `FR-SRV-021` compares.
#[derive(Debug, Clone, Copy)]
struct Row {
    /// The spelling the message of `FR-SRV-030` lists.
    name: &'static str,
    /// The value the comparison of `FR-SRV-021` is made over.
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

/// The series of `FR-SRV-015`, newest first.
///
/// The order is the one `FR-SRV-021` fixes and the one the message of
/// `FR-SRV-030` shows; a test asserts that it is strictly descending, so
/// [`newest`] may read the first row rather than scan for a maximum.
///
/// `FR-SRV-017` forbids reading the size of this array as a quantity to be
/// preserved: four series across three families is what `FR-SRV-001` admitted
/// on its verification date, not a count this code may rely on.
const WINDOW: [Row; 4] = [
    Row::new("12.3", 12, 3),
    Row::new("11.8", 11, 8),
    Row::new("11.4", 11, 4),
    Row::new("10.11", 10, 11),
];

/// The spellings of [`WINDOW`], in the same order.
///
/// `FR-SRV-030` obliges the `cause` line of a refused series to list the
/// series that are supported, and `BR-SRV-005` forbids a second copy of the
/// set: this is a projection of the one array above rather than a list written
/// again, and it travels on the error value so that the renderer prints what
/// it is handed.
pub(crate) const SUPPORTED: [&str; WINDOW.len()] = [
    WINDOW[0].name,
    WINDOW[1].name,
    WINDOW[2].name,
    WINDOW[3].name,
];

/// The newest series of the window.
fn newest() -> Series {
    // WINDOW is strictly descending, which a test of this module asserts, so
    // the first row is the maximum. The fallback is unreachable from a
    // non-empty array and is written as one so that the module carries no
    // panic at all.
    WINDOW.first().map_or(Series::new(0, 0), |row| row.series)
}

/// The standing of `series` against the window, or [`None`] where
/// `FR-SRV-020` refuses it.
///
/// Three outcomes and one comparison, which is what `FR-SRV-021` orders the
/// series for: a member of the window is `supported`; anything newer than the
/// newest member is `newer_than_supported`, per `FR-SRV-031`; everything else
/// is below the window and is the condition `FR-SRV-020` refuses — a series
/// whose maintenance has ended, a rolling release, and a series of a family
/// the window has left, all three reached by the one rule.
///
/// The refusal is answered as [`None`] rather than raised here, because the
/// message of `FR-SRV-030` names the database entry that reached the server
/// and this module knows nothing of entries.
pub(crate) fn standing(series: Series) -> Option<Standing> {
    if WINDOW.iter().any(|row| row.series == series) {
        return Some(Standing::Supported);
    }

    (series > newest()).then_some(Standing::NewerThanSupported)
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
    fn fr_srv_015_the_window_is_the_four_series_the_requirement_records() {
        // FR-SRV-015: the table as verified on 2026-09-10, newest first.
        assert_eq!(SUPPORTED, ["12.3", "11.8", "11.4", "10.11"]);
    }

    #[test]
    fn fr_srv_015_each_rows_spelling_and_its_numbers_are_the_same_series() {
        // The spelling is what a message lists and the numbers are what the
        // comparison uses; a row that disagreed with itself would order a
        // series by numbers the caller is never shown.
        for row in WINDOW {
            assert_eq!(Series::parse(row.name), Some(row.series), "{}", row.name);
        }
    }

    #[test]
    fn fr_srv_021_the_window_is_strictly_descending_so_the_first_row_is_the_newest() {
        // FR-SRV-021 orders by major family first and then series number, and
        // `newest` reads the first row on the strength of that order.
        for pair in WINDOW.windows(2) {
            let [older, newer] = [pair[0].series, pair[1].series];
            assert!(older > newer, "{older:?} does not precede {newer:?}");
        }

        assert_eq!(newest(), series("12.3"));
    }

    #[test]
    fn fr_srv_021_the_comparison_is_by_major_family_and_then_series_number() {
        // FR-SRV-021, in its own examples: 12.1 is newer than 11.8 and older
        // than 12.3, and a patch number is not part of a series at all.
        assert!(series("12.1") > series("11.8"));
        assert!(series("12.1") < series("12.3"));
        assert!(series("10.11") > series("10.6"));
        assert_eq!(series("10.11"), series("10.11"));
    }

    #[test]
    fn fr_srv_020_a_series_below_the_window_is_refused_whatever_put_it_there() {
        // FR-SRV-020 covers three cases with one rule: maintenance ended
        // (10.6), a rolling release (12.1, 12.2, 12.0), and a family the
        // window has left (9.x).
        for identifier in ["10.6", "10.5", "12.0", "12.1", "12.2", "9.9"] {
            assert_eq!(standing(series(identifier)), None, "{identifier}");
        }
    }

    #[test]
    fn fr_srv_015_every_series_of_the_window_is_supported() {
        for identifier in SUPPORTED {
            assert_eq!(
                standing(series(identifier)),
                Some(Standing::Supported),
                "{identifier}"
            );
        }
    }

    #[test]
    fn fr_srv_031_a_series_newer_than_the_newest_is_marked_rather_than_refused() {
        // FR-SRV-031: above the window the catalogue is read, and FR-SRV-032
        // carries the marking in the document rather than in the exit code.
        for identifier in ["12.4", "13.0", "99.1"] {
            assert_eq!(
                standing(series(identifier)),
                Some(Standing::NewerThanSupported),
                "{identifier}"
            );
        }
    }

    #[test]
    fn fr_srv_040_a_series_identifier_outside_the_two_component_form_yields_nothing() {
        assert_eq!(Series::parse(""), None);
        assert_eq!(Series::parse("11"), None);
        assert_eq!(Series::parse("11."), None);
        assert_eq!(Series::parse("x.4"), None);
        assert_eq!(Series::parse("11.4.13"), None);
    }
}
