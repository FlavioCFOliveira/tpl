//! The hardening controls and limits of a connection, each set and read back
//! (`FR-LITE-010` … `FR-LITE-012`).
//!
//! Every comparison of a value set with the value read back is a function of
//! its own, separable from any connection, so the refusal of `FR-LITE-012` is
//! testable over every control without a library that misbehaves.
//!
//! | Control | Through | Confirmed by |
//! |---|---|---|
//! | Extension loading | The build (`SQLITE_OMIT_LOAD_EXTENSION`, `ADR-013`) | `PRAGMA compile_options` holds `OMIT_LOAD_EXTENSION` |
//! | Read-only open | The open's flags | `sqlite3_db_readonly` on `main` |
//! | Defensive mode | `SQLITE_DBCONFIG_DEFENSIVE` | Its read-back |
//! | `trusted_schema` off | `SQLITE_DBCONFIG_TRUSTED_SCHEMA` | Its read-back |
//! | `query_only`, `cell_size_check`, `mmap_size`, `temp_store` | Their pragmas | Their pragmas, and for `temp_store` the `TEMP_STORE` row of `PRAGMA compile_options` |
//! | The three limits of `FR-LITE-011` | `sqlite3_limit` | `sqlite3_limit` |
//!
//! No statement here reads the file, per the observation under `FR-LITE-010`,
//! so every control is in force before the version probe parses the schema
//! (`FR-LITE-017`).

use crate::error::{Error, FileValue};

/// A control or a limit whose read-back did not confirm the value set
/// (`FR-LITE-012`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct Mismatch {
    /// The control or the limit, as `FR-LITE-010` and `FR-LITE-011` name it.
    pub(crate) control: &'static str,
    /// The value set, or the row required.
    pub(crate) set: &'static str,
    /// What was read back.
    pub(crate) read_back: String,
}

impl Mismatch {
    /// The `70` of `FR-LITE-012`.
    pub(crate) fn into_error(self, entry: &str, path: &std::path::Path) -> Error {
        Error::SqliteControlNotInEffect {
            entry: entry.to_owned(),
            path: path.to_owned(),
            control: self.control,
            set: self.set,
            read_back: FileValue::new(self.read_back),
        }
    }
}

/// The row `PRAGMA compile_options` must return for extension loading to be
/// absent (`FR-LITE-010`).
const OMIT_LOAD_EXTENSION: &str = "OMIT_LOAD_EXTENSION";

/// The prefix of the row that states how the library keeps temporary data.
const TEMP_STORE: &str = "TEMP_STORE=";

/// The `TEMP_STORE` rows under which the `temp_store` pragma takes effect, or
/// keeps everything in memory (`FR-LITE-010`).
const TEMP_STORE_ADMITTED: [&str; 3] = ["TEMP_STORE=1", "TEMP_STORE=2", "TEMP_STORE=3"];

/// The two confirmations `PRAGMA compile_options` gives: extension loading
/// absent, and a build under which `temp_store` takes effect (`FR-LITE-010`).
///
/// # Errors
///
/// Returns the [`Mismatch`] of the first that the rows do not confirm.
pub(crate) fn build(options: &[String]) -> Result<(), Mismatch> {
    if !options.iter().any(|option| option == OMIT_LOAD_EXTENSION) {
        return Err(Mismatch {
            control: "extension loading",
            set: "absent from the build: the row OMIT_LOAD_EXTENSION",
            read_back: "no OMIT_LOAD_EXTENSION row".to_owned(),
        });
    }

    match options.iter().find(|option| option.starts_with(TEMP_STORE)) {
        Some(row) if TEMP_STORE_ADMITTED.contains(&row.as_str()) => Ok(()),
        Some(row) => Err(Mismatch {
            control: "temp_store",
            set: "a build row TEMP_STORE=1, TEMP_STORE=2 or TEMP_STORE=3",
            read_back: row.clone(),
        }),
        None => Err(Mismatch {
            control: "temp_store",
            set: "a build row TEMP_STORE=1, TEMP_STORE=2 or TEMP_STORE=3",
            read_back: "no TEMP_STORE row".to_owned(),
        }),
    }
}

/// A flag set to `set` and read back as `read_back`.
///
/// # Errors
///
/// Returns the [`Mismatch`] where the two differ.
pub(crate) fn flag(
    control: &'static str,
    set: &'static str,
    expected: bool,
    read_back: bool,
) -> Result<(), Mismatch> {
    if read_back == expected {
        Ok(())
    } else {
        Err(Mismatch {
            control,
            set,
            read_back: if read_back { "on" } else { "off" }.to_owned(),
        })
    }
}

/// A number set to `expected` and read back as `read_back`.
///
/// # Errors
///
/// Returns the [`Mismatch`] where the two differ.
pub(crate) fn number(
    control: &'static str,
    set: &'static str,
    expected: i64,
    read_back: i64,
) -> Result<(), Mismatch> {
    if read_back == expected {
        Ok(())
    } else {
        Err(Mismatch {
            control,
            set,
            read_back: read_back.to_string(),
        })
    }
}

/// The length of a string, a BLOB or a row, and of one statement
/// (`FR-LITE-011`).
pub(crate) const LENGTH_LIMIT: i32 = 1_000_000;

/// The number of attached databases (`FR-LITE-011`).
pub(crate) const ATTACHED_LIMIT: i32 = 0;

#[cfg(test)]
mod tests {
    use super::{Mismatch, build, flag, number};

    fn rows(rows: &[&str]) -> Vec<String> {
        rows.iter().map(|row| (*row).to_owned()).collect()
    }

    #[test]
    fn fr_lite_010_the_build_is_confirmed_by_the_omission_and_an_admitted_temp_store_row() {
        for admitted in ["TEMP_STORE=1", "TEMP_STORE=2", "TEMP_STORE=3"] {
            assert_eq!(build(&rows(&["OMIT_LOAD_EXTENSION", admitted])), Ok(()));
        }
    }

    #[test]
    fn fr_lite_012_a_build_without_the_omission_names_the_control_the_row_and_what_was_read() {
        let refused = build(&rows(&["TEMP_STORE=1", "THREADSAFE=1"]))
            .expect_err("extension loading is not omitted");

        assert_eq!(refused.control, "extension loading");
        assert!(refused.set.contains("OMIT_LOAD_EXTENSION"));
        assert_eq!(refused.read_back, "no OMIT_LOAD_EXTENSION row");
    }

    #[test]
    fn fr_lite_012_a_temp_store_row_of_0_or_none_is_a_control_not_in_effect() {
        assert_eq!(
            build(&rows(&["OMIT_LOAD_EXTENSION", "TEMP_STORE=0"])),
            Err(Mismatch {
                control: "temp_store",
                set: "a build row TEMP_STORE=1, TEMP_STORE=2 or TEMP_STORE=3",
                read_back: "TEMP_STORE=0".to_owned(),
            })
        );
        assert_eq!(
            build(&rows(&["OMIT_LOAD_EXTENSION"]))
                .expect_err("no row")
                .read_back,
            "no TEMP_STORE row"
        );
    }

    #[test]
    fn fr_lite_012_a_read_back_that_differs_names_the_value_set_and_the_value_read() {
        assert_eq!(flag("query_only", "on", true, true), Ok(()));
        let refused = flag("defensive mode", "on", true, false).expect_err("differs");
        assert_eq!(refused.control, "defensive mode");
        assert_eq!(refused.set, "on");
        assert_eq!(refused.read_back, "off");

        assert_eq!(number("mmap_size", "0", 0, 0), Ok(()));
        let refused = number("temp_store", "2 (MEMORY)", 2, 1).expect_err("differs");
        assert_eq!(refused.read_back, "1");
    }
}
