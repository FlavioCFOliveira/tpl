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
//!
//! **The declared type chooses the decode, and the driver is asked once.** The
//! accessor used to ask for an unsigned value first and for a signed one where
//! that failed, so every non-null signed field made the driver build and drop
//! a mismatched-type error: 76 697 allocations per read of 2 000 tables, 29% of
//! the read's (`BENCHMARKS.md`, 2026-11-05). The field's raw
//! value is now taken once and decoded under the same two tests the driver's
//! own typed accessor applies, in the same order, so every outcome — SQL
//! `NULL`, either signedness, a negative value, a field of another type, a
//! field the row does not carry — is the one the two typed asks gave.

use std::panic::Location;

use sqlx_core::decode::Decode;
use sqlx_core::row::Row as _;
use sqlx_core::type_info::TypeInfo as _;
use sqlx_core::types::Type;
use sqlx_core::value::ValueRef as _;
use sqlx_mysql::{MySql, MySqlRow};

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
///
/// It answers what `try_get::<Option<u64>>` and, where that failed,
/// `try_get::<Option<i64>>` answered, without building the error the first
/// ask returned for a signed field. Each of the driver's typed asks takes the
/// raw value, accepts SQL `NULL` whatever the declared type, refuses a declared
/// type its integer is not compatible with unless the type is the untyped
/// `NULL` type, and then decodes; this applies the same three steps, to one raw
/// value, unsigned first.
#[track_caller]
pub(super) fn maybe_integer(row: &MySqlRow, field: &str) -> Result<Option<u64>, Error> {
    let location = Location::caller();

    // A field the row does not carry failed both asks.
    let value = row.try_get_raw(field).map_err(|_| undecodable(location))?;

    if value.is_null() {
        return Ok(None);
    }

    // PERF: the two compatibility tests of the typed asks, read off the
    // declared type once, so a signed field is never offered to the unsigned
    // decode only for its refusal to be built and dropped.
    let (unsigned, signed) = {
        let declared = value.type_info();
        let untyped = declared.is_null();

        (
            untyped || <u64 as Type<MySql>>::compatible(&declared),
            untyped || <i64 as Type<MySql>>::compatible(&declared),
        )
    };

    if unsigned && let Ok(decoded) = <u64 as Decode<MySql>>::decode(value.clone()) {
        return Ok(Some(decoded));
    }

    if !signed {
        return Err(undecodable(location));
    }

    let decoded = <i64 as Decode<MySql>>::decode(value).map_err(|_| undecodable(location))?;

    u64::try_from(decoded)
        .map(Some)
        .map_err(|_| undecodable(location))
}

/// An integer field the catalogue declares `NOT NULL`, of either signedness.
#[track_caller]
pub(super) fn integer(row: &MySqlRow, field: &str) -> Result<u64, Error> {
    let location = Location::caller();

    maybe_integer(row, field)?.ok_or_else(|| undecodable(location))
}
