//! Reading one field of one row of a `mysql` read.
//!
//! The three text accessors are `crate::mariadb::catalogue::row`'s, because a
//! text field decodes the same way from either server. The integer accessors
//! are this module's own, and they differ in one respect: they accept either
//! signedness.
//!
//! **Why an integer is read whatever its signedness.** The driver refuses to
//! decode across a difference of signedness, and MySQL declares the integer
//! fields of its catalogue with both, table by table and field by field, and
//! unlike MariaDB. *Observed on 2026-11-04 through `scripts/mysql/`, identical
//! on `8.4` and `9.7`:*
//!
//! | Field | Declared |
//! |---|---|
//! | `COLUMNS.ORDINAL_POSITION`, `DATETIME_PRECISION` | `int unsigned` |
//! | `COLUMNS.CHARACTER_MAXIMUM_LENGTH` | `bigint` |
//! | `COLUMNS.NUMERIC_PRECISION`, `NUMERIC_SCALE` | `bigint unsigned` |
//! | `STATISTICS.NON_UNIQUE` | `int` |
//! | `STATISTICS.SUB_PART` | `bigint` |
//! | `ROUTINES.NUMERIC_PRECISION`, `NUMERIC_SCALE`, `DATETIME_PRECISION` | `int unsigned` |
//! | `ROUTINES.CHARACTER_MAXIMUM_LENGTH` | `bigint` |
//! | `PARAMETERS.NUMERIC_PRECISION`, `DATETIME_PRECISION` | `int unsigned` |
//! | `PARAMETERS.CHARACTER_MAXIMUM_LENGTH`, `NUMERIC_SCALE` | `bigint` |
//! | `TRIGGERS.ACTION_ORDER` | `int unsigned` |
//!
//! A field is still type-checked as an integer: text in an integer field is
//! the `70` of a field that does not decode, for the reason
//! `crate::mariadb::catalogue::row` records. What is not fixed is the
//! signedness, so that `FR-MYSQL-010`'s newer series, which may declare a field
//! with the other one, is read rather than refused. A negative value is no
//! length, precision or position the model can carry, and is the same `70`.

use std::panic::Location;

use sqlx_core::row::Row as _;
use sqlx_mysql::MySqlRow;

use crate::error::Error;

pub(super) use crate::mariadb::catalogue::row::{maybe_text, text, text_or_empty};

/// The invariant both integer accessors enforce.
const DECODES: &str = "every integer field of a mysql catalogue field list decodes as a \
     non-negative integer of either signedness";

/// The condition a field that does not decode produces, at the fold's line.
const fn undecodable(location: &'static Location<'static>) -> Error {
    Error::InternalInvariant {
        invariant: DECODES,
        location,
    }
}

/// An integer field that admits SQL `NULL`, of either signedness.
#[track_caller]
pub(super) fn maybe_integer(row: &MySqlRow, field: &str) -> Result<Option<u64>, Error> {
    let location = Location::caller();

    if let Ok(unsigned) = row.try_get::<Option<u64>, _>(field) {
        return Ok(unsigned);
    }

    let signed = row
        .try_get::<Option<i64>, _>(field)
        .map_err(|_| undecodable(location))?;

    signed
        .map(|value| u64::try_from(value).map_err(|_| undecodable(location)))
        .transpose()
}

/// An integer field the catalogue declares `NOT NULL`, of either signedness.
#[track_caller]
pub(super) fn integer(row: &MySqlRow, field: &str) -> Result<u64, Error> {
    let location = Location::caller();

    maybe_integer(row, field)?.ok_or_else(|| undecodable(location))
}
