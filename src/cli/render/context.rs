//! The render context: the five top-level variables of `FR-RND-023`, and the
//! three of them that never come from a server.
//!
//! `FR-RND-023` names five variables and fixes where each comes from;
//! `FR-CTX-026` through `FR-CTX-030` fix what the three non-server ones hold.
//! Both are here, because a context assembled in one place is a context whose
//! shape cannot differ between the two sources of `FR-RND-016` and
//! `FR-RND-026`:
//!
//! | Variable | Source | Fixed by |
//! |---|---|---|
//! | `database` | the context source — a catalogue read or the `--context` document | `FR-RND-023` |
//! | `table` / `view` / `routine` | the context source, selected by the object flag | `FR-RND-003`, `FR-RND-006` |
//! | `vars` | the `--set` flags of this invocation | `FR-CTX-026` |
//! | `tpl` | the binary | `FR-CTX-027` |
//! | `now` | the clock, once | `FR-CTX-028` … `FR-CTX-030` |
//!
//! # The three are injected, and a document cannot supply them
//!
//! `FR-RND-024` requires `vars`, `tpl` and `now` always to be injected and a
//! value a `--context` document supplies for any of the three to be ignored.
//! Both hold **structurally** here rather than by a check: [`assemble`] writes
//! the three itself, and the only thing a document contributes is the
//! `database` object — `FR-SCH-018` keeps the three out of the document and
//! `crate::model::document` reads back exactly the keys the contract names, so
//! a `now` written at the top level of a supplied document reaches nothing
//! that could read it.
//!
//! # `now` is one instant
//!
//! `FR-CTX-029` makes every reference in one render yield the same value and
//! `FR-CTX-030` makes it the single documented source of non-reproducibility.
//! [`now`] is therefore called **once**, by the one function that assembles a
//! context, and the string it produced is what every `{{ now }}` of that render
//! interpolates. `FR-RND-002` gives one render per invocation, so one instant
//! is all there is to record.
//!
//! The formatting is this module's own, and it is eleven lines of arithmetic
//! rather than a dependency: `FR-CTX-028` fixes one format — RFC 3339, UTC, a
//! `Z` offset and second precision — and a crate that formats every other
//! format besides is a crate this project's dependency budget refuses for one
//! call site.

mod lazy;

pub(super) use lazy::Store;

use std::collections::BTreeMap;
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};

use minijinja::Value;

use crate::cli::source::Served;
use crate::error::Error;

/// The context variable the model is bound to (`FR-RND-023`).
const DATABASE: &str = "database";

/// The context variable the `--set` entries are bound to (`FR-CTX-026`).
const VARS: &str = "vars";

/// The context variable the binary describes itself through (`FR-CTX-027`).
const TPL: &str = "tpl";

/// The one key `FR-CTX-027` gives that variable.
const VERSION: &str = "version";

/// The context variable the render time is bound to (`FR-CTX-028`).
const NOW: &str = "now";

/// The version `FR-HELP-005` prints, which `FR-CTX-027` makes `tpl.version`.
///
/// It is read from the package rather than from a second record, on the same
/// terms the version line of `crate::cli` is: the two cannot disagree with the
/// binary they came from, and a test asserts they do not disagree with each
/// other.
const TPL_VERSION: &str = env!("CARGO_PKG_VERSION");

/// The flag whose value `FR-RND-011` and `FR-RND-012` refuse.
const SET: &str = "--set";

/// What `FR-RND-009` splits a `--set` argument on, and only the first of them.
const SEPARATOR: char = '=';

/// The one byte of a `--set` key that is neither a letter nor a digit
/// (`FR-RND-012`).
const UNDERSCORE: u8 = b'_';

/// What a `--set` argument with no `=` at all was expected to be
/// (`FR-RND-011`).
const PAIR: &str = "a <key>=<value> pair";

/// What a `--set` argument whose key is not an identifier was expected to be
/// (`FR-RND-012`, `FR-RND-013`).
const IDENTIFIER: &str = "a key matching [A-Za-z_][A-Za-z0-9_]*, then '=', then the value; a dotted key is refused \
     rather than split";

/// Seconds in one day, for the civil conversion of [`stamp`].
const SECONDS_PER_DAY: u64 = 86_400;

/// Seconds in one hour.
const SECONDS_PER_HOUR: u64 = 3_600;

/// Seconds in one minute.
const SECONDS_PER_MINUTE: u64 = 60;

/// Days from the civil epoch `0000-03-01` to `1970-01-01`, which is the shift
/// Howard Hinnant's `civil_from_days` applies before its era arithmetic.
const EPOCH_SHIFT: u64 = 719_468;

/// Days in one four-hundred-year era of the proleptic Gregorian calendar.
const DAYS_PER_ERA: u64 = 146_097;

/// The `vars` object of this invocation (`FR-RND-008` … `FR-RND-015`,
/// `FR-CTX-026`).
///
/// Each entry is split on its **first** `=`, per `FR-RND-009`, so
/// `--set "msg=a=b"` sets `vars.msg` to `a=b`; an empty value is valid, per
/// `FR-RND-010`; and the key must be an identifier, per `FR-RND-012`, which is
/// what refuses a dotted key rather than splitting it, per `FR-RND-013`. The
/// value is carried as written and is never read as a number or a boolean, per
/// `FR-RND-015` — the map is of strings, so there is no representation in which
/// it could be anything else.
///
/// The map is ordered by key and borrows the argument vector, so `vars` has a
/// determinate shape without a copy of anything the caller wrote.
///
/// # Errors
///
/// Returns [`Error::MalformedValue`] — `64` — for an argument with no `=` at
/// all (`FR-RND-011`) and for one whose key is not an identifier
/// (`FR-RND-012`), and [`Error::RepeatedSetKey`] — `64` — for a key supplied
/// more than once (`FR-RND-014`).
pub(super) fn vars(set: &[String]) -> Result<BTreeMap<&str, &str>, Error> {
    let mut defined: BTreeMap<&str, &str> = BTreeMap::new();

    for written in set {
        // FR-RND-009: the first `=`, so everything after it is the value.
        let Some((key, value)) = written.split_once(SEPARATOR) else {
            return Err(malformed(written, PAIR));
        };

        // FR-RND-012 and FR-RND-013.
        if !identifier(key) {
            return Err(malformed(written, IDENTIFIER));
        }

        // FR-RND-014: last-wins would let a script emitting the same key twice
        // pass unnoticed, so the second definition is refused and both values
        // travel on the condition.
        if let Some(first) = defined.insert(key, value) {
            return Err(Error::RepeatedSetKey {
                key: key.to_owned(),
                first: first.to_owned(),
                second: value.to_owned(),
            });
        }
    }

    Ok(defined)
}

/// The render context of `FR-RND-023`, assembled from its four sources.
///
/// `bound` is the object variable the invocation named, or [`None`] for the
/// whole-database form of `FR-RND-006`, which binds no object variable at all.
/// The other four are written here and cannot be supplied from anywhere else,
/// which is `FR-RND-024` held by construction.
///
/// `database` is reachable whole and converted only where the template reads
/// it: [`lazy::database`] keeps the document — copying it only where it is
/// borrowed ([`Served::borrowed`]) — and answers every read exactly as the whole
/// conversion would have.
pub(super) fn assemble(
    database: Served<'_, '_>,
    bound: Option<(&'static str, Value)>,
    defined: &BTreeMap<&str, &str>,
    at: &str,
) -> Value {
    // PERF: converting the whole document before the template ran was 41.4%
    // of a render of `example` over `WL-001` (`BENCHMARKS.md`, 2026-09-22, row
    // 6 of the waste register); a member is now converted when first read.
    assembled(lazy::database(database), bound, defined, at)
}

/// The render context of `FR-RND-023`, with `database` served from the cache
/// files `store` names (`FR-CACHE-038`).
///
/// It is [`assemble`] with one source changed: each member of a collection is
/// read from its file the first time the template reaches it, and the four
/// other variables are written exactly as [`assemble`] writes them.
pub(super) fn assemble_shelved(
    store: &Arc<Store>,
    bound: Option<(&'static str, Value)>,
    defined: &BTreeMap<&str, &str>,
    at: &str,
) -> Value {
    // PERF: reading and decoding every cache file before a render bound to one
    // table was 9.1 ms of 11.3 ms (`BENCHMARKS.md`, 2026-09-23, `#243` row 1);
    // a member's file is now read when the template first reaches it.
    assembled(lazy::shelved(store), bound, defined, at)
}

/// The context of `FR-RND-023`, from its `database` value and the other four
/// sources.
fn assembled(
    database: Value,
    bound: Option<(&'static str, Value)>,
    defined: &BTreeMap<&str, &str>,
    at: &str,
) -> Value {
    let mut entries: Vec<(&'static str, Value)> = Vec::with_capacity(5);

    entries.push((DATABASE, database));

    // FR-RND-006: absent, no object variable is bound, so a template written
    // for the whole database never has to defend itself against one.
    if let Some((variable, object)) = bound {
        entries.push((variable, object));
    }

    // FR-CTX-026: `{}` where no `--set` was supplied, which is the empty map.
    entries.push((VARS, Value::from_serialize(defined)));
    // FR-CTX-027: an object of exactly one key, so the variable can gain a
    // field without breaking a template, per `FR-OUT-014`.
    entries.push((TPL, Value::from_iter([(VERSION, Value::from(TPL_VERSION))])));
    // FR-CTX-028 and FR-CTX-029: one instant, already formatted.
    entries.push((NOW, Value::from(at)));

    Value::from_iter(entries)
}

/// The render time, as `FR-CTX-028` fixes it.
///
/// A clock that reports an instant before the Unix epoch — which is a machine
/// whose clock is wrong rather than a condition of this invocation — is read as
/// the epoch itself. It is a degradation rather than a failure because
/// `FR-CTX-030` makes `now` the one source of non-reproducibility and nothing
/// else in a render depends on it, and rather than a panic because this crate
/// carries none on a read path.
pub(super) fn now() -> String {
    let since = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |elapsed| elapsed.as_secs());

    stamp(since)
}

/// `seconds` after the Unix epoch, as an RFC 3339 timestamp in UTC
/// (`FR-CTX-028`).
fn stamp(seconds: u64) -> String {
    let days = seconds / SECONDS_PER_DAY;
    let time = seconds % SECONDS_PER_DAY;
    let (year, month, day) = civil(days);

    format!(
        "{year:04}-{month:02}-{day:02}T{:02}:{:02}:{:02}Z",
        time / SECONDS_PER_HOUR,
        (time % SECONDS_PER_HOUR) / SECONDS_PER_MINUTE,
        time % SECONDS_PER_MINUTE,
    )
}

/// The civil date `days` after the Unix epoch, in the proleptic Gregorian
/// calendar.
///
/// It is Howard Hinnant's `civil_from_days`, whose derivation and range are
/// published with `date.h` ("chrono-Compatible Low-Level Date Algorithms").
/// The arithmetic is over a shifted era beginning on `0000-03-01`, which is
/// what puts the leap day at the end of a year and removes every special case
/// from the conversion: there is no branch for February and none for a leap
/// year.
///
/// Every subtraction below is non-negative by construction — `doe` is at most
/// `DAYS_PER_ERA - 1`, `doy` at most `365`, and each subtrahend is a floor
/// division of the value it is taken from — so the unsigned arithmetic cannot
/// wrap.
fn civil(days: u64) -> (u64, u64, u64) {
    let shifted = days + EPOCH_SHIFT;
    let era = shifted / DAYS_PER_ERA;
    let day_of_era = shifted % DAYS_PER_ERA;
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

    // The era begins in March, so January and February belong to the next
    // civil year.
    (if month <= 2 { year + 1 } else { year }, month, day)
}

/// Whether a `--set` key matches `[A-Za-z_][A-Za-z0-9_]*` (`FR-RND-012`).
///
/// The comparison is over bytes because every admitted character is ASCII: a
/// key carrying a multi-byte character fails at its first byte, which is not
/// an ASCII letter, digit or underscore.
fn identifier(key: &str) -> bool {
    let mut bytes = key.bytes();

    bytes
        .next()
        .is_some_and(|first| first.is_ascii_alphabetic() || first == UNDERSCORE)
        && bytes.all(|byte| byte.is_ascii_alphanumeric() || byte == UNDERSCORE)
}

/// The `64` of `FR-RND-011` and `FR-RND-012`, naming the argument as written.
///
/// The `64` row of `FR-ERR-034` obliges the `cause` to name "the value that did
/// not conform together with the type expected", and the two conditions carry
/// different expectations so that neither line would read as truly of the
/// other.
fn malformed(written: &str, expected: &'static str) -> Error {
    Error::MalformedValue {
        parameter: SET.to_owned(),
        command: "render".to_owned(),
        value: written.to_owned(),
        expected,
    }
}

#[cfg(test)]
mod tests {
    use super::{TPL_VERSION, assemble, civil, identifier, now, stamp, vars};
    use crate::cli::source::Served;
    use crate::error::Error;
    use crate::model::document;
    use minijinja::Value;

    /// The `--set` arguments of one invocation.
    fn set(written: &[&str]) -> Vec<String> {
        written.iter().map(|entry| (*entry).to_owned()).collect()
    }

    /// The condition `written` produces.
    fn refused(written: &[&str]) -> Error {
        let supplied = set(written);

        vars(&supplied).expect_err("the invocation is refused")
    }

    #[test]
    fn fr_rnd_009_an_argument_is_split_on_its_first_separator_and_never_on_a_later_one() {
        // FR-RND-009, with the worked case the requirement gives: everything
        // after the first `=` is the value.
        let supplied = set(&["msg=a=b", "title=Orders"]);
        let defined = vars(&supplied).expect("both are pairs");

        assert_eq!(defined.get("msg"), Some(&"a=b"));
        assert_eq!(defined.get("title"), Some(&"Orders"));
    }

    #[test]
    fn fr_rnd_010_an_empty_value_is_valid() {
        // FR-RND-010: `--set empty=` sets `vars.empty` to the empty string,
        // which is a value and not an absence.
        let supplied = set(&["empty="]);
        let defined = vars(&supplied).expect("an empty value is a value");

        assert_eq!(defined.get("empty"), Some(&""));
    }

    #[test]
    fn fr_rnd_011_an_argument_with_no_separator_at_all_is_64() {
        // FR-RND-011, and the `64` row of FR-ERR-034: the argument as written
        // reaches the condition.
        let condition = refused(&["title"]);

        assert_eq!(condition.exit_code(), 64);
        assert!(
            matches!(&condition, Error::MalformedValue { value, .. } if value == "title"),
            "{condition}"
        );
    }

    #[test]
    fn fr_rnd_012_a_key_outside_the_identifier_pattern_is_64() {
        // FR-RND-012 over the two spellings its rationale names, and over the
        // empty key and a leading digit, which the pattern also refuses.
        for written in ["my key=x", "db.host=x", "=x", "1st=x", "na-me=x", "ké=x"] {
            let condition = refused(&[written]);

            assert_eq!(condition.exit_code(), 64, "{written}");
            assert!(
                matches!(&condition, Error::MalformedValue { value, .. } if value == written),
                "{written}: {condition}"
            );
        }

        // And the spellings it admits, including the underscore at both ends.
        for admitted in ["_", "_x", "A", "a_1", "TITLE_2"] {
            let supplied = set(&[&format!("{admitted}=x")]);

            assert!(vars(&supplied).is_ok(), "{admitted}");
        }
    }

    #[test]
    fn fr_rnd_013_a_dotted_key_is_refused_and_never_split_into_nested_structure() {
        // FR-RND-013: the refusal is FR-RND-012's, and nothing anywhere builds
        // a nested value from the key — which is why this asserts the
        // condition rather than the absence of a structure.
        let condition = refused(&["db.host=localhost"]);

        assert_eq!(condition.exit_code(), 64);
        assert!(
            matches!(&condition, Error::MalformedValue { expected, .. } if expected.contains("dotted")),
            "{condition}"
        );
    }

    #[test]
    fn fr_rnd_014_a_repeated_key_is_64_and_carries_both_values() {
        // FR-RND-014: last-wins would let a script emitting the same key twice
        // pass unnoticed, with the result depending on ordering.
        let condition = refused(&["title=Orders", "author=data", "title=Consignments"]);

        assert_eq!(condition.exit_code(), 64);
        assert!(
            matches!(
                &condition,
                Error::RepeatedSetKey { key, first, second }
                    if key == "title" && first == "Orders" && second == "Consignments"
            ),
            "{condition}"
        );
    }

    #[test]
    fn fr_rnd_015_a_value_is_always_a_string_and_is_never_read_as_a_number_or_a_boolean() {
        // FR-RND-015 over the two traps its rationale names. The assertion is
        // on the assembled context rather than on the map, because the trap is
        // what a template sees.
        let model = document::fixture::database();
        let built = document::context(&model).expect("the fixture model is coherent");
        let supplied = set(&["version=1.0", "name=true", "count=7"]);
        let defined = vars(&supplied).expect("all three are pairs");
        let context = assemble(
            Served::borrowed(&built),
            None,
            &defined,
            "1970-01-01T00:00:00Z",
        );
        let carried = context
            .get_attr("vars")
            .expect("FR-RND-024 always injects vars");

        for (key, value) in [("version", "1.0"), ("name", "true"), ("count", "7")] {
            let held = carried.get_attr(key).expect("the key was supplied");

            assert_eq!(held.as_str(), Some(value), "{key}");
            assert_eq!(held.kind(), minijinja::value::ValueKind::String, "{key}");
        }
    }

    #[test]
    fn fr_ctx_026_vars_is_an_empty_object_where_no_set_was_supplied() {
        // FR-CTX-026: `{}`, so `{{ vars }}` and a loop over it both work
        // without the template testing for the variable's existence.
        let model = document::fixture::database();
        let built = document::context(&model).expect("the fixture model is coherent");
        let defined = vars(&[]).expect("no argument is no condition");
        let context = assemble(
            Served::borrowed(&built),
            None,
            &defined,
            "1970-01-01T00:00:00Z",
        );
        let carried = context.get_attr("vars").expect("vars is always injected");

        assert_eq!(carried.kind(), minijinja::value::ValueKind::Map);
        assert_eq!(carried.len(), Some(0));
    }

    #[test]
    fn fr_rnd_023_the_context_carries_the_five_variables_and_binds_the_object_it_was_given() {
        // FR-RND-023 and FR-RND-006: four variables always, and the fifth only
        // where an object flag named one.
        let model = document::fixture::database();
        let built = document::context(&model).expect("the fixture model is coherent");
        let defined = vars(&[]).expect("no argument is no condition");
        let whole = assemble(
            Served::borrowed(&built),
            None,
            &defined,
            "1970-01-01T00:00:00Z",
        );

        for variable in ["database", "vars", "tpl", "now"] {
            assert!(
                !whole
                    .get_attr(variable)
                    .expect("the attribute is readable")
                    .is_undefined(),
                "{variable}"
            );
        }
        for unbound in ["table", "view", "routine"] {
            assert!(
                whole
                    .get_attr(unbound)
                    .expect("the attribute is readable")
                    .is_undefined(),
                "{unbound} is bound by no flag"
            );
        }

        let bound = assemble(
            Served::borrowed(&built),
            Some(("table", Value::from("bound"))),
            &defined,
            "1970-01-01T00:00:00Z",
        );

        assert_eq!(
            bound
                .get_attr("table")
                .expect("the attribute is readable")
                .as_str(),
            Some("bound")
        );
    }

    #[test]
    fn fr_ctx_027_tpl_is_an_object_of_one_key_carrying_the_version_the_version_line_prints() {
        // FR-CTX-027 ties the value to FR-HELP-005, and this is the tie: the
        // line `tpl version` writes carries exactly this string.
        let model = document::fixture::database();
        let built = document::context(&model).expect("the fixture model is coherent");
        let defined = vars(&[]).expect("no argument is no condition");
        let context = assemble(
            Served::borrowed(&built),
            None,
            &defined,
            "1970-01-01T00:00:00Z",
        );
        let carried = context.get_attr("tpl").expect("tpl is always injected");

        assert_eq!(carried.len(), Some(1));
        assert_eq!(
            carried
                .get_attr("version")
                .expect("the one key of FR-CTX-027")
                .as_str(),
            Some(TPL_VERSION)
        );
        assert_eq!(format!("tpl {TPL_VERSION}\n"), {
            let mut written = Vec::new();
            crate::cli::version(&mut written).expect("a buffer accepts every write");
            String::from_utf8(written).expect("the line is UTF-8")
        });
    }

    #[test]
    fn fr_ctx_028_a_timestamp_is_rfc_3339_in_utc_with_a_z_offset_and_second_precision() {
        // FR-CTX-028 over the epoch, the example the requirement writes, and
        // the two leap days a naive conversion gets wrong.
        for (seconds, expected) in [
            (0, "1970-01-01T00:00:00Z"),
            (1_757_491_200, "2025-09-10T08:00:00Z"),
            (951_782_400, "2000-02-29T00:00:00Z"),
            (1_709_164_800, "2024-02-29T00:00:00Z"),
            (1_709_251_199, "2024-02-29T23:59:59Z"),
            (4_102_444_800, "2100-01-01T00:00:00Z"),
        ] {
            assert_eq!(stamp(seconds), expected, "{seconds}");
        }

        // The shape of what the clock actually produces: twenty characters,
        // the two separators, and the `Z` FR-CTX-028 fixes.
        let produced = now();

        assert_eq!(produced.len(), 20, "{produced}");
        assert!(produced.ends_with('Z'), "{produced}");
        assert_eq!(produced.as_bytes()[10], b'T', "{produced}");
    }

    #[test]
    fn fr_ctx_029_one_instant_is_recorded_and_every_reference_yields_it() {
        // FR-CTX-029: a template that interpolates `now` twice must not
        // produce two timestamps. The assembled context holds the string, so
        // the property is the one value being read many times.
        let model = document::fixture::database();
        let built = document::context(&model).expect("the fixture model is coherent");
        let defined = vars(&[]).expect("no argument is no condition");
        let at = now();
        let context = assemble(Served::borrowed(&built), None, &defined, &at);
        let carried = context.get_attr("now").expect("now is always injected");

        assert_eq!(carried.as_str(), Some(at.as_str()));
        assert_eq!(
            context
                .get_attr("now")
                .expect("now is always injected")
                .as_str(),
            carried.as_str()
        );
    }

    #[test]
    fn the_civil_conversion_agrees_with_itself_across_every_day_of_four_centuries() {
        // The derivation this module borrows is total over a range far wider
        // than the one a clock reaches, and the property that catches an
        // off-by-one in it is monotonicity: consecutive days produce
        // consecutive dates, and no day produces a month or a day outside its
        // bounds.
        let mut previous = civil(0);

        for days in 1..=146_097_u64 {
            let (year, month, day) = civil(days);

            assert!((1..=12).contains(&month), "{days}");
            assert!((1..=31).contains(&day), "{days}");
            assert!(
                (year, month, day) > previous,
                "{days}: {previous:?} then {:?}",
                (year, month, day)
            );

            previous = (year, month, day);
        }
    }

    #[test]
    fn the_identifier_test_reads_the_first_byte_and_the_rest_by_different_rules() {
        // The pattern of FR-RND-012 is two classes, not one: a digit is
        // admitted after the first position and refused at it.
        assert!(identifier("a1"));
        assert!(!identifier("1a"));
        assert!(identifier("_"));
        assert!(!identifier(""));
    }
}
