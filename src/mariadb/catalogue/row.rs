//! Reading one field of one catalogue row.
//!
//! Seven accessors, one per shape the catalogue field lists carry, so that the
//! decode of a field is decided **once** rather than at each of the hundred
//! places the fold reads one.
//!
//! | Accessor | The field it reads | The shape it produces |
//! |---|---|---|
//! | [`text`] | A field the catalogue declares `NOT NULL` | `&str`, borrowed from the row |
//! | [`maybe_text`] | A field that admits SQL `NULL` | `Option<&str>`, [`None`] for `NULL` |
//! | [`text_or_empty`] | One of the two fields that admit SQL `NULL` and reach a decomposed type rather than a field of their own | `&str`, empty for `NULL` |
//! | [`count`] | An integer field declared **unsigned** and `NOT NULL` | `u64` |
//! | [`maybe_count`] | An integer field declared **unsigned** that admits SQL `NULL` | `Option<u64>` |
//! | [`signed`] | An integer field declared **signed** and `NOT NULL` | `u64` |
//! | [`maybe_signed`] | An integer field declared **signed** that admits SQL `NULL` | `Option<u64>` |
//!
//! **Every string is borrowed from the row and never copied.** The model holds
//! [`Cow<'a, str>`](std::borrow::Cow) throughout and a live read is the path
//! that always borrows, so a field reaches [`crate::model`] as a
//! `Cow::Borrowed` of the row buffer the driver already allocated. The rows
//! outlive the model by construction: [`super::Catalogue`] owns them and hands
//! out a model that borrows from itself.
//!
//! # Why an integer has two accessors
//!
//! `INFORMATION_SCHEMA` declares its integer fields with **both signednesses**,
//! and the driver refuses to decode across the difference: it compares the
//! column's declared type against the Rust type asked for, and an unsigned
//! column read as `i64` is a `ColumnDecode` failure rather than a conversion.
//! The two accessors are therefore the declaration, and the declaration is an
//! observation rather than a guess.
//!
//! *Observed, 2026-09-18, against the fixture of `scripts/mariadb/`. Every
//! declaration below is **byte-identical on all four series** of
//! `FR-SRV-015`:*
//!
//! | Field | Declared | Read by |
//! |---|---|---|
//! | `COLUMNS.ORDINAL_POSITION` | `bigint(21) unsigned` | [`count`] |
//! | `COLUMNS.CHARACTER_MAXIMUM_LENGTH`, `NUMERIC_PRECISION`, `NUMERIC_SCALE`, `DATETIME_PRECISION` | `bigint(21) unsigned` | [`maybe_count`] |
//! | `ROUTINES.DATETIME_PRECISION`, `PARAMETERS.DATETIME_PRECISION` | `bigint(21) unsigned` | [`maybe_count`] |
//! | `ROUTINES.CHARACTER_MAXIMUM_LENGTH`, `NUMERIC_PRECISION`, `NUMERIC_SCALE` | `int(21)` | [`maybe_signed`] |
//! | `PARAMETERS.CHARACTER_MAXIMUM_LENGTH`, `NUMERIC_PRECISION`, `NUMERIC_SCALE` | `int(21)` | [`maybe_signed`] |
//! | `STATISTICS.NON_UNIQUE` | `bigint(1)` | [`signed`] |
//! | `STATISTICS.SUB_PART` | `bigint(3)` | [`maybe_signed`] |
//! | `TRIGGERS.ACTION_ORDER` | `bigint(4)` | [`signed`] |
//!
//! Note the two rows that sit inside one catalogue table: a routine's and a
//! parameter's datetime precision is unsigned while the three size fields
//! beside it are not. A reader that chose one accessor per table would be
//! wrong for those two fields on every series.
//!
//! **Both accessors answer `u64`**, because the model carries lengths,
//! precisions and ordinals as one type. A signed field is narrowed, and a
//! negative — a shape no read of a supported series produces — is reported as
//! the invariant violation it is rather than wrapped into a large `u64`.
//!
//! *Rejected: decoding every integer without the driver's type check.* It
//! reads whatever bytes the column carries as an integer, so a field a later
//! server declared as text would decode to a number rather than fail. *Also
//! rejected: casting in SQL, so that one accessor serves.* It puts an
//! expression in every column list `FR-SRV-037` governs, which is the one
//! place this reader most needs to be read against the requirement.
//!
//! # Why a decode failure is `70`
//!
//! A field that does not decode is not a condition the caller can act on. The
//! field lists of `FR-CAT-042` and `FR-CAT-045` … `FR-CAT-051` were recorded
//! from an observation of all four series of `FR-SRV-015`, and this reader
//! names only the fields those lists hold, so a failure here means `tpl`'s
//! model of the catalogue is wrong rather than that the server or the
//! invocation is. That is `FR-ERR-030`'s own description of `70` — an internal
//! invariant violated, past which the system declines to continue — and
//! `#[track_caller]` puts the *where* of `FR-ERR-034`'s `70` row at the line
//! of the fold that read the field rather than inside this module.
//!
//! *Rejected: reporting a decode failure as the `69` of a catalogue query.*
//! The statement succeeded, the server answered, and the connection is intact;
//! sending a caller to check whether the server is reachable would be a
//! diagnosis of something that is not wrong. *Also rejected: substituting an
//! empty string or a zero.* It produces a document that is wrong at exit `0`,
//! which is the class of failure this corpus works hardest to prevent.

use std::panic::Location;

use sqlx::Row as _;
use sqlx::mysql::MySqlRow;

use crate::error::Error;

/// The invariant every accessor of this module enforces.
const DECODES: &str =
    "every field of a catalogue field list decodes as the shape the requirement records";

/// The condition a field that does not decode produces.
///
/// `location` is the fold's line, captured by the accessor's own
/// `#[track_caller]` before the closure that reports it — a `Location::caller`
/// evaluated inside the closure would name the closure instead.
const fn undecodable(location: &'static Location<'static>) -> Error {
    Error::InternalInvariant {
        invariant: DECODES,
        location,
    }
}

/// A field the catalogue declares `NOT NULL`, borrowed from the row.
#[track_caller]
pub(super) fn text<'r>(row: &'r MySqlRow, field: &str) -> Result<&'r str, Error> {
    let location = Location::caller();

    row.try_get::<&str, _>(field)
        .map_err(|_| undecodable(location))
}

/// A field that admits SQL `NULL`, [`None`] where the catalogue returned one.
#[track_caller]
pub(super) fn maybe_text<'r>(row: &'r MySqlRow, field: &str) -> Result<Option<&'r str>, Error> {
    let location = Location::caller();

    row.try_get::<Option<&str>, _>(field)
        .map_err(|_| undecodable(location))
}

/// A field that admits SQL `NULL` and reaches a model field that does not.
///
/// Two fields are read through this accessor and no others, and both reach a
/// **decomposed type** rather than a field of their own:
///
/// | Field | Where it goes | Why the empty string is the value and not a substitution |
/// |---|---|---|
/// | `COLUMNS.GENERATION_EXPRESSION` | [`Generated::expression`](crate::model::column::Generated::expression) | It is read only where the column is generated, per `FR-CAT-051`, and the catalogue populates it on exactly those rows |
/// | `DTD_IDENTIFIER`, in the routine and parameter catalogues | [`ColumnType::raw`](crate::model::column_type::ColumnType::raw) | `FR-CAT-048` records that a procedure's is SQL `NULL` beside a data-type field that is the empty string, and the model answers the pair by emitting the whole return type as `null` from the **kind**, so this value is never the one presented |
///
/// **The four fields this accessor used to serve have left it**, because
/// `FR-CAT-056` forbids the substitution for them by name: a trigger's definer
/// and action statement, and a foreign key's unique-constraint name and
/// referenced table name, are declared nullable on all four series and now
/// reach the model as [`Option`], read with [`maybe_text`]. A routine's
/// definition left earlier, for the reason `FR-PRIV-017` gives, and is now an
/// [`Option`] too.
///
/// The requirement's ground is the one this accessor's own documentation used
/// to concede: the empty string is a recorded limit rather than a value the
/// catalogue produced, and a model field that cannot tell the two apart makes
/// an absent value indistinguishable from an empty one.
#[track_caller]
pub(super) fn text_or_empty<'r>(row: &'r MySqlRow, field: &str) -> Result<&'r str, Error> {
    Ok(maybe_text(row, field)?.unwrap_or_default())
}

/// An integer field the catalogue declares unsigned and `NOT NULL`.
#[track_caller]
pub(super) fn count(row: &MySqlRow, field: &str) -> Result<u64, Error> {
    let location = Location::caller();

    row.try_get::<u64, _>(field)
        .map_err(|_| undecodable(location))
}

/// An integer field declared unsigned that admits SQL `NULL`.
#[track_caller]
pub(super) fn maybe_count(row: &MySqlRow, field: &str) -> Result<Option<u64>, Error> {
    let location = Location::caller();

    row.try_get::<Option<u64>, _>(field)
        .map_err(|_| undecodable(location))
}

/// An integer field the catalogue declares signed and `NOT NULL`, narrowed.
#[track_caller]
pub(super) fn signed(row: &MySqlRow, field: &str) -> Result<u64, Error> {
    let location = Location::caller();
    let declared = row
        .try_get::<i64, _>(field)
        .map_err(|_| undecodable(location))?;

    u64::try_from(declared).map_err(|_| undecodable(location))
}

/// An integer field declared signed that admits SQL `NULL`, narrowed.
#[track_caller]
pub(super) fn maybe_signed(row: &MySqlRow, field: &str) -> Result<Option<u64>, Error> {
    let location = Location::caller();
    let declared = row
        .try_get::<Option<i64>, _>(field)
        .map_err(|_| undecodable(location))?;

    declared
        .map(|value| u64::try_from(value).map_err(|_| undecodable(location)))
        .transpose()
}
