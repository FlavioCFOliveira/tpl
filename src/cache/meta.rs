//! `meta.json`: the two versions, the load time, and the per-collection
//! completeness record (`FR-CDOC-001` … `FR-CDOC-007`, `FR-CDOC-013`).
//!
//! | Field | What it versions or records | Fixed by |
//! |---|---|---|
//! | `cache_format` | The on-disk arrangement: which files exist, where they sit, how they are named | `FR-CDOC-002` |
//! | `schema_version` | The model content, and it is the version the documents carry under `FR-OUT-011` | `FR-CDOC-003` |
//! | `loaded_at` | When the cache was last written | `FR-CDOC-013` |
//! | `collections` | Per collection, whether it was loaded whole | `FR-CDOC-006` |
//!
//! **The two versions are independent** (`FR-CDOC-005`): neither moves on
//! account of a change that affects only the other. `BR-CDOC-001` states why
//! one cannot express both — a change to the model invalidates every cached
//! object without moving a file, and a change to the arrangement leaves the
//! content valid — so they are two numbers here and two constants below.
//!
//! **Either version unknown to the running binary makes the data a miss**, per
//! `FR-CDOC-004` and `FR-CACHE-033`: [`Meta::usable`] is the test, and a miss
//! is silent — no error, no warning, and the read goes to the server.
//!
//! **`FR-CACHE-005` is satisfied here and not per file.** That requirement
//! gives each cached document a format version and defers to `FR-CDOC-001`
//! through `FR-CDOC-005` for where it is written; `FR-CDOC-001` writes both
//! versions in this file, which governs every file of the entry's folder. A
//! version repeated in each object file would be the same number in as many
//! places as the cache holds objects, and `FR-CDOC-004` would still have to
//! refuse the folder as a whole the moment one of them was unknown.
//!
//! **`loaded_at` appears here and in `tpl cache status`, and nowhere else**
//! (`FR-CDOC-012`, `FR-CDOC-013`). `BR-CDOC-003` is the reason: a load time in
//! a read would make two identical invocations against an unchanged project
//! produce different bytes, which `NFR-DET-001` forbids. Nothing in this module
//! is reachable from the path that builds a read's document.

use std::fmt::Write as _;
use std::time::{SystemTime, UNIX_EPOCH};

use serde::{Deserialize, Serialize};

use super::paths::Collection;

/// The version of the on-disk arrangement (`FR-CDOC-002`).
///
/// What moves it: a file added, removed, moved or renamed. It started at `1`
/// with the layout [`super::paths`] draws, and it is independent of
/// [`SCHEMA_VERSION`], per `FR-CDOC-005`.
const CACHE_FORMAT: u32 = 1;

/// The version of the model content (`FR-CDOC-003`).
///
/// It is the version the documents in the cache carry under `FR-OUT-011`, so
/// it is read from the one place that publishes it rather than written a second
/// time here: a cached object **is** the `data` of a document, and a binary
/// that emits version *n* may serve only a cache written at version *n*.
const SCHEMA_VERSION: u32 = crate::output::SCHEMA_VERSION;

/// The number of seconds in a minute, an hour and a day, for the one
/// conversion [`instant`] makes.
const MINUTE: u64 = 60;
/// Seconds in an hour.
const HOUR: u64 = 60 * MINUTE;
/// Seconds in a day.
const DAY: u64 = 24 * HOUR;

/// The completeness of one collection (`FR-CDOC-006`).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub(super) struct Recorded {
    /// The collection's plural name, as `FR-OUT-030` writes it.
    pub(super) name: String,

    /// Whether the collection was loaded whole.
    pub(super) whole: bool,
}

/// What `meta.json` carries.
///
/// Every field is required on the way in: a file missing one is a file this
/// binary cannot interpret, which `FR-CACHE-033` makes a miss rather than an
/// error.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub(super) struct Meta {
    /// The on-disk arrangement's version (`FR-CDOC-002`).
    pub(super) cache_format: u32,

    /// The model content's version (`FR-CDOC-003`).
    pub(super) schema_version: u32,

    /// When the cache was last written (`FR-CDOC-013`).
    pub(super) loaded_at: String,

    /// Per collection, whether it was loaded whole (`FR-CDOC-006`).
    pub(super) collections: Vec<Recorded>,
}

impl Meta {
    /// A record of a cache written now, with `whole` as `write` established it.
    pub(super) fn new(whole: [bool; Collection::ALL.len()]) -> Self {
        Self {
            cache_format: CACHE_FORMAT,
            schema_version: SCHEMA_VERSION,
            loaded_at: now(),
            collections: Collection::ALL
                .iter()
                .zip(whole)
                .map(|(collection, whole)| Recorded {
                    name: collection.name().to_owned(),
                    whole,
                })
                .collect(),
        }
    }

    /// Whether this binary may serve data recorded under these versions
    /// (`FR-CDOC-004`).
    ///
    /// Both must match exactly. A version this binary has not seen is unknown
    /// to it whichever direction it moved in, and `FR-CDOC-004` makes the
    /// affected data a miss without distinguishing the two — so there is no
    /// forward compatibility to express and no range to compare against.
    pub(super) const fn usable(&self) -> bool {
        self.cache_format == CACHE_FORMAT && self.schema_version == SCHEMA_VERSION
    }

    /// Whether `collection` was loaded whole (`FR-CDOC-006`, `FR-CDOC-007`).
    ///
    /// A collection this record does not name was never written whole, so a
    /// listing of it is a miss — which is the answer `BR-CDOC-002` requires
    /// when `tpl schema tables` follows `tpl cache load --table orders`.
    pub(super) fn whole(&self, collection: Collection) -> bool {
        self.collections
            .iter()
            .any(|recorded| recorded.name == collection.name() && recorded.whole)
    }

    /// The record `self` becomes when a clean removes one object of
    /// `collection`, or [`None`] where the record already says that collection
    /// is not whole (`FR-CACHE-043`).
    ///
    /// Exactly one thing changes: the collection's flag. `loaded_at`, both
    /// versions and every other collection are carried over as they are,
    /// because no load happened and `FR-CACHE-025` reports `loaded_at` as the
    /// time the cache was loaded.
    pub(super) fn without_whole(&self, collection: Collection) -> Option<Self> {
        if !self.whole(collection) {
            return None;
        }

        let mut record = self.clone();
        for recorded in &mut record.collections {
            if recorded.name == collection.name() {
                recorded.whole = false;
            }
        }

        Some(record)
    }

    /// The record `self` becomes when a named read refreshes one object.
    ///
    /// The completeness flags are carried over unchanged and only the load
    /// time moves. `FR-CACHE-028` admits no automatic invalidation, and
    /// refreshing one member of a collection that was whole leaves it whole:
    /// every object it held is still held, and one of them is newer.
    pub(super) fn refreshed(&self) -> Self {
        Self {
            cache_format: CACHE_FORMAT,
            schema_version: SCHEMA_VERSION,
            loaded_at: now(),
            collections: self.collections.clone(),
        }
    }
}

/// The present instant, in the form the worked example of `FR-CACHE-034`
/// carries.
///
/// `2026-09-10T08:14:22Z`: a calendar date and a time of day in UTC, to the
/// second, with the `Z` designator. It is composed here rather than taken from
/// a date library because it is the only date this project formats and the
/// dependency budget refuses a crate used for one function — and because the
/// conversion is a closed arithmetic over the seconds since the epoch, with no
/// locale, no zone table and nothing to keep up to date.
///
/// A clock before the epoch yields the epoch itself. It is not a condition:
/// `loaded_at` is read by an operator through `tpl cache status`, per
/// `BR-CDOC-005`, and a host whose clock is set before 1970 has a fault this
/// cache is not the place to report.
fn now() -> String {
    let seconds = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |since| since.as_secs());

    instant(seconds)
}

/// `seconds` since the Unix epoch, as an instant of `FR-CACHE-034`'s form.
///
/// The civil date is derived by the standard days-from-epoch algorithm, shifted
/// so that the year begins in March and the leap day falls at its end — which
/// removes every special case for February from the arithmetic.
fn instant(seconds: u64) -> String {
    let days = seconds / DAY;
    let (hour, minute, second) = (
        (seconds % DAY) / HOUR,
        (seconds % HOUR) / MINUTE,
        seconds % MINUTE,
    );

    // 719_468 is the number of days from 0000-03-01 to 1970-01-01, which is
    // what shifts the epoch onto the March-first era the algorithm counts in.
    let z = days + 719_468;
    let era = z / 146_097;
    let day_of_era = z % 146_097;
    let year_of_era =
        (day_of_era - day_of_era / 1460 + day_of_era / 36_524 - day_of_era / 146_096) / 365;
    let year = year_of_era + era * 400;
    let day_of_year = day_of_era - (365 * year_of_era + year_of_era / 4 - year_of_era / 100);
    let shifted_month = (5 * day_of_year + 2) / 153;
    let day = day_of_year - (153 * shifted_month + 2) / 5 + 1;
    let month = if shifted_month < 10 {
        shifted_month + 3
    } else {
        shifted_month - 9
    };
    let year = if month <= 2 { year + 1 } else { year };

    let mut written = String::with_capacity(20);
    // Writing into a `String` cannot fail, and the value is discarded rather
    // than unwrapped so that no `expect` stands on a path that has no failure.
    let _ = write!(
        written,
        "{year:04}-{month:02}-{day:02}T{hour:02}:{minute:02}:{second:02}Z"
    );

    written
}

#[cfg(test)]
mod tests {
    use super::super::paths::Collection;
    use super::{CACHE_FORMAT, Meta, SCHEMA_VERSION, instant};

    #[test]
    fn fr_cdoc_001_the_record_carries_both_versions_and_the_load_time() {
        // FR-CDOC-001, FR-CDOC-013: `cache_format` and `schema_version` in
        // meta.json, with the load time beside them.
        let meta = Meta::new([true, true, true]);
        let written = serde_json::to_string(&meta).expect("the record serialises");

        assert!(written.contains(r#""cache_format":1"#), "{written}");
        assert!(written.contains(r#""schema_version":1"#), "{written}");
        assert!(written.contains(r#""loaded_at":"#), "{written}");
        assert!(written.contains(r#""collections":"#), "{written}");
    }

    #[test]
    fn fr_cdoc_003_the_content_version_is_the_one_the_documents_carry() {
        // FR-CDOC-003: `schema_version` is the same version the documents in
        // the cache carry under FR-OUT-011, read from the one place that
        // publishes it.
        assert_eq!(SCHEMA_VERSION, crate::output::SCHEMA_VERSION);
    }

    #[test]
    fn fr_cdoc_005_the_two_versions_are_independent_and_neither_is_the_other() {
        // FR-CDOC-005: neither is incremented on account of a change that
        // affects only the other. They are two constants, and the test pins
        // that they are read separately rather than aliased.
        let meta = Meta::new([false, false, false]);

        assert_eq!(meta.cache_format, CACHE_FORMAT);
        assert_eq!(meta.schema_version, SCHEMA_VERSION);
    }

    #[test]
    fn fr_cdoc_004_an_unknown_version_in_either_field_makes_the_data_unusable() {
        // FR-CDOC-004 and FR-CACHE-033: either version unknown is a miss, and
        // the direction it moved in does not matter.
        let mut meta = Meta::new([true, true, true]);
        assert!(meta.usable());

        for unknown in [0, CACHE_FORMAT + 1] {
            let mut moved = meta.clone();
            moved.cache_format = unknown;
            assert!(!moved.usable(), "cache_format {unknown}");
        }

        meta.schema_version = SCHEMA_VERSION + 1;
        assert!(!meta.usable());
    }

    #[test]
    fn fr_cdoc_006_the_record_answers_per_collection() {
        // FR-CDOC-006 and FR-CDOC-007: a listing is served from the cache only
        // where its own collection is recorded whole.
        let meta = Meta::new([true, false, true]);

        assert!(meta.whole(Collection::Tables));
        assert!(!meta.whole(Collection::Views));
        assert!(meta.whole(Collection::Routines));
    }

    #[test]
    fn fr_cdoc_007_a_collection_the_record_does_not_name_is_not_whole() {
        // A record written by a binary that knew fewer collections, or one
        // truncated by hand: the absent collection is a miss rather than a
        // silent success.
        let meta = Meta {
            cache_format: CACHE_FORMAT,
            schema_version: SCHEMA_VERSION,
            loaded_at: "2026-09-18T00:00:00Z".to_owned(),
            collections: Vec::new(),
        };

        for collection in Collection::ALL {
            assert!(!meta.whole(collection), "{}", collection.name());
        }
    }

    #[test]
    fn fr_cache_043_a_partial_clean_changes_the_one_flag_and_keeps_the_load_time() {
        // FR-CACHE-043: exactly one thing changes, and a collection already
        // recorded as not whole has nothing to change.
        let mut whole = Meta::new([true, false, true]);
        whole.loaded_at = "2026-09-01T00:00:00Z".to_owned();

        let cleaned = whole
            .without_whole(Collection::Tables)
            .expect("the tables were whole");
        let mut expected = whole.clone();
        expected.collections[0].whole = false;

        assert_eq!(cleaned, expected);
        assert_eq!(whole.without_whole(Collection::Views), None);
    }

    #[test]
    fn fr_cache_028_refreshing_one_object_keeps_the_completeness_record() {
        // FR-CACHE-028: nothing invalidates automatically. Loading one table
        // into a cache whose tables are whole leaves them whole — every object
        // is still held and one of them is newer.
        let whole = Meta::new([true, false, true]);
        let refreshed = whole.refreshed();

        assert!(refreshed.whole(Collection::Tables));
        assert!(!refreshed.whole(Collection::Views));
        assert!(refreshed.whole(Collection::Routines));
    }

    #[test]
    fn fr_cache_034_the_load_time_is_written_in_the_form_the_worked_example_carries() {
        // FR-CACHE-034 shows `2026-09-10T08:14:22Z`. The three readings below
        // are the epoch, the worked example's own instant, and a leap day.
        assert_eq!(instant(0), "1970-01-01T00:00:00Z");
        // 1_789_028_062 is the instant that example carries; the constant
        // this test first named was six days earlier.
        assert_eq!(instant(1_789_028_062), "2026-09-10T08:14:22Z");
        assert_eq!(instant(1_709_164_800), "2024-02-29T00:00:00Z");
        assert_eq!(instant(951_782_400), "2000-02-29T00:00:00Z");
    }
}
