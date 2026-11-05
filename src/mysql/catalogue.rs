//! The catalogue read of a `mysql` entry: the fourth kind of the closed list
//! of `FR-MYSQL-015`, and the model it produces.
//!
//! [`super::open`] has issued the three connection-start statements, so every
//! statement here follows them (`FR-MYSQL-017`). What follows is the constant
//! plan of [`statements`] — twelve statements, one per table of the closed
//! list — and the fold that turns their rows into [`crate::model`].
//!
//! | Submodule | Subject | Forced by |
//! |---|---|---|
//! | [`statements`] | The twelve statements | `FR-MYSQL-015`, `FR-MYSQL-016`, `FR-MYSQL-042` |
//! | [`row`] | How one field of one row is decoded | The field lists of `FR-MYSQL-025` … `FR-MYSQL-038` |
//! | [`expression`] | The attribute field, and the layer of escaping `FR-MYSQL-046` removes | `FR-MYSQL-029`, `FR-MYSQL-030`, `FR-MYSQL-046` |
//! | [`fold`] | How the rows become the model | `FR-MYSQL-023` … `FR-MYSQL-041` |
//!
//! **The count is fixed before the first statement is sent**: the plan is a
//! constant, and [`read`] issues it and nothing besides, so `NFR-PERF-001` and
//! `NFR-PERF-002` hold by construction (`FR-MYSQL-042`); in a test build
//! [`Catalogue`] keeps what was issued, so a test reads the execution rather
//! than the source. The record is the tests' alone, and a release build
//! neither keeps nor allocates it (rmp `#351`).
//!
//! **One order is made as the rows are fetched**: a foreign key's columns,
//! which the fold shared with `crate::mariadb::catalogue` appends in the order
//! the rows stand in ([`arrived`]). Every other order is the fold's.
//!
//! **The model borrows the rows**, as `crate::mariadb::catalogue` explains for
//! a `mariadb` read: [`Catalogue`] owns them, and [`Catalogue::model`] hands
//! back a model that borrows from it.

mod expression;
mod fold;
mod row;
mod statements;

// The MySQL fixture's harness, reached as `crate::mariadb::catalogue` reaches
// the MariaDB one, and for the same reasons: the file is not a test target of
// its own, and one file reached by two `#[path]` items would be two modules
// over one file.
#[cfg(test)]
#[path = "../../tests/support/mysql.rs"]
pub(crate) mod fixture;

#[cfg(test)]
mod tests;

use std::panic::Location;

use sqlx_mysql::MySqlRow;
use tokio::time::timeout;

use super::Session;
use crate::deadline::{Clock, Phase};
use crate::error::{Error, NetworkPhase, SessionStatement};
use crate::mariadb::connect::Target;
use crate::mariadb::fault;
use crate::model::database::Database;
use crate::model::server::Server;
use statements::Read;

/// The invariant a read on a closed session violates.
const CONNECTION_OPEN: &str = "the session a catalogue read is issued on is still open";

/// The rows a `mysql` read returned, and the model they describe.
///
/// It owns every row the plan fetched and holds no connection and no runtime.
#[derive(Debug)]
pub(crate) struct Catalogue {
    /// The server the read was made against (`FR-MYSQL-006`).
    server: Server<'static>,

    /// The database the read covered (`FR-MYSQL-023`), which the `77` of
    /// `FR-PRIV-021` names where the schema catalogue returned no row.
    schema: String,

    /// The statements that were issued, in order, which the assertions of
    /// `NFR-PERF-001` and `NFR-PERF-002` read. A process that acted on it would
    /// be acting on its own plan, so only a test build carries it.
    #[cfg(test)]
    issued: Vec<&'static str>,

    /// The rows of each statement, in the slot [`Read::slot`] gives it.
    rows: [Vec<MySqlRow>; Read::COUNT],
}

impl Catalogue {
    /// The model these rows describe.
    ///
    /// # Errors
    ///
    /// Returns what the fold returns: [`Error::PropertyNotReadable`] where the
    /// schema catalogue returned no row, and [`Error::InternalInvariant`]
    /// where a field does not decode or a key names a column its table does
    /// not carry.
    pub(crate) fn model(&self) -> Result<Database<'_>, Error> {
        fold::database(self)
    }

    /// The database this read covered.
    fn schema(&self) -> &str {
        &self.schema
    }

    /// The statements this read issued, in the order they were issued.
    #[cfg(test)]
    pub(crate) fn statements(&self) -> &[&'static str] {
        &self.issued
    }

    /// The rows of one statement.
    fn rows(&self, read: Read) -> &[MySqlRow] {
        self.rows.get(read.slot()).map_or(&[], Vec::as_slice)
    }

    /// The server the read was made against.
    const fn server(&self) -> &Server<'static> {
        &self.server
    }
}

/// Reads the catalogue over a settled session (`FR-MYSQL-015`,
/// `FR-MYSQL-017`, `FR-MYSQL-042`).
///
/// # Errors
///
/// Returns [`Error::NetworkDeadlineExceeded`] where a statement outlived its
/// bound, what a driver failure is classified as, and
/// [`Error::InternalInvariant`] where the session has been closed.
pub(crate) fn read(
    session: &mut Session,
    target: &Target<'_>,
    clock: &Clock,
    schema: &str,
) -> Result<Catalogue, Error> {
    let plan = statements::plan();
    #[cfg(test)]
    let mut issued = Vec::with_capacity(plan.len());
    let mut rows: [Vec<MySqlRow>; Read::COUNT] = std::array::from_fn(|_| Vec::new());

    for statement in plan {
        let fetched = issue(session, target, clock, statement, schema)?.map_err(|driver| {
            fault::speaking(
                target.entry(),
                &driver,
                target.host(),
                target.port(),
                SessionStatement::CatalogueQuery,
            )
        })?;

        #[cfg(test)]
        issued.push(statement.sql());

        if let Some(slot) = rows.get_mut(statement.slot()) {
            *slot = arrived(statement, fetched)?;
        }
    }

    Ok(Catalogue {
        server: session.server().clone(),
        schema: schema.to_owned(),
        #[cfg(test)]
        issued,
        rows,
    })
}

/// The rows one statement returned, in the order the read keeps them.
///
/// The key-column rows are put in the order of each key's positions
/// (`FR-MYSQL-033`), because the fold they go through appends a key's columns
/// in the order the rows stand in and the model borrows from them; every other
/// read is kept as it arrived, and its order is the fold's.
///
/// # Errors
///
/// Returns [`Error::InternalInvariant`] where a field the order is read from
/// does not decode.
fn arrived(statement: Read, rows: Vec<MySqlRow>) -> Result<Vec<MySqlRow>, Error> {
    match statement {
        Read::KeyColumns => fold::key_columns(rows),
        Read::Schema
        | Read::Tables
        | Read::Columns
        | Read::Indexes
        | Read::ForeignKeyRules
        | Read::CheckOwners
        | Read::Checks
        | Read::Triggers
        | Read::Views
        | Read::Routines
        | Read::Parameters => Ok(rows),
    }
}

/// Runs the privilege probe of `FR-CFG-044` over a settled session.
///
/// One `SELECT` against `SCHEMATA`, restricted to `schema` by the literal of
/// `ADR-015`; the answer is true where it was answered without a privilege
/// error and returned at least one row. No row is read into the model, and a
/// privilege refusal — one of the four numbers of `FR-CFG-044` — is an answer
/// rather than a condition (`FR-CFG-045`), as
/// `crate::mariadb::catalogue::probe` records; any other error packet is the
/// third way of `FR-ERR-036`.
///
/// # Errors
///
/// Returns [`Error::NetworkDeadlineExceeded`] where the statement outlived its
/// bound, what a driver failure that is not the server's own answer is
/// classified as, and [`Error::InternalInvariant`] where the session has been
/// closed.
pub(crate) fn probe(
    session: &mut Session,
    target: &Target<'_>,
    clock: &Clock,
    schema: &str,
) -> Result<bool, Error> {
    match issue(session, target, clock, statements::probe(), schema)? {
        Ok(rows) => Ok(!rows.is_empty()),
        // FR-CFG-044: only the four privilege errors are its `false`; any
        // other error packet is the third way of FR-ERR-036.
        Err(driver) if fault::is_privilege_error(&driver) => Ok(false),
        Err(driver) => Err(fault::speaking(
            target.entry(),
            &driver,
            target.host(),
            target.port(),
            SessionStatement::CatalogueQuery,
        )),
    }
}

/// The prefix of [`literal`]'s output: the introducer that fixes the
/// literal's character set, and the opening of a hexadecimal literal.
const LITERAL_OPEN: &str = "_utf8mb4 X'";

/// The close of [`literal`]'s output.
const LITERAL_CLOSE: char = '\'';

/// The uppercase hexadecimal digits, indexed by value.
const HEX_UPPER: &[u8; 16] = b"0123456789ABCDEF";

/// What a statement's one placeholder must follow: the comparison of an
/// `INFORMATION_SCHEMA` column with the name (`ADR-015`, point 2).
const COMPARED: &str = " = ";

/// The one placeholder of each statement, which [`compose`] replaces.
const PLACEHOLDER: char = '?';

/// The invariant [`compose`] enforces on the twelve constant texts.
const ONE_COMPARED_PLACEHOLDER: &str = "every mysql catalogue statement holds one `?`, \
     and it is the right-hand side of `<INFORMATION_SCHEMA column> =`";

/// `name` as the text of a literal the server reads as exactly those bytes,
/// in `utf8mb4` (`FR-SRV-043`, `FR-MYSQL-048`, `ADR-015`, point 2).
///
/// It is `_utf8mb4 X'`, two uppercase hexadecimal digits for each byte of
/// `name`, and `'`, and nothing else: an empty name is `_utf8mb4 X''`. The
/// output is ASCII and holds no `\`, `"`, `|`, `;` or `--`, so no `sql_mode`
/// and no client character set changes how the server reads it, and no byte
/// of the name reaches the text as itself.
fn literal(name: &str) -> String {
    let mut written =
        String::with_capacity(LITERAL_OPEN.len() + 2 * name.len() + LITERAL_CLOSE.len_utf8());
    written.push_str(LITERAL_OPEN);
    for byte in name.bytes() {
        written.push(char::from(HEX_UPPER[usize::from(byte >> 4)]));
        written.push(char::from(HEX_UPPER[usize::from(byte & 0x0f)]));
    }
    written.push(LITERAL_CLOSE);

    written
}

/// The text of `statement` with [`literal`] of `schema` in place of its one
/// `?` (`ADR-015`, point 2).
///
/// The constant must hold one `?`, as the right-hand side of
/// `<INFORMATION_SCHEMA column> =`; a text that does not is the `70` of a
/// violated invariant and is never sent. None of the twelve is written beside
/// `COLLATE`, `CONVERT`, `LIKE` or `REGEXP`, which a test asserts of every
/// composed text.
///
/// # Errors
///
/// Returns [`Error::InternalInvariant`] where the constant does not hold the
/// one compared placeholder.
#[track_caller]
fn compose(statement: Read, schema: &str) -> Result<String, Error> {
    let sql = statement.sql();
    let violated = || Error::InternalInvariant {
        invariant: ONE_COMPARED_PLACEHOLDER,
        location: Location::caller(),
    };
    let (before, after) = sql.split_once(PLACEHOLDER).ok_or_else(violated)?;
    if !before.ends_with(COMPARED) || after.contains(PLACEHOLDER) {
        return Err(violated());
    }

    let literal = literal(schema);
    let mut composed = String::with_capacity(sql.len() - PLACEHOLDER.len_utf8() + literal.len());
    composed.push_str(before);
    composed.push_str(&literal);
    composed.push_str(after);

    Ok(composed)
}

/// Sends one statement, `schema` written into it as [`literal`] writes it, and
/// hands back what the driver answered.
///
/// It is `crate::mariadb::catalogue`'s statement runner for a `mysql`
/// session: the bound of `FR-CONF-005` composed with `FR-GLOB-011`, and the
/// two lines of `FR-GLOB-017`. The statement goes over the text protocol
/// (`ADR-015`, point 1): handed to the driver with no arguments, it is one
/// `COM_QUERY` and its rows arrive in the text format, where a bound value
/// made it a `COM_STMT_PREPARE` and a `COM_STMT_EXECUTE`.
fn issue(
    session: &mut Session,
    target: &Target<'_>,
    clock: &Clock,
    statement: Read,
    schema: &str,
) -> Result<Result<Vec<MySqlRow>, sqlx_core::Error>, Error> {
    let bound = clock.bound(target.deadlines().of(Phase::CatalogueQuery));
    let expired = || {
        fault::expired(
            target.entry(),
            NetworkPhase::CatalogueQuery,
            target.host(),
            target.port(),
            bound,
        )
    };

    if bound.expired() {
        return Err(expired());
    }

    // FR-GLOB-017 and NFR-PERF-008: one line per catalogue query issued,
    // written before the statement is sent, under the token every catalogue
    // query of a server read carries.
    crate::diagnostics::emit::catalogue_query(
        crate::diagnostics::emit::CatalogueSource::InformationSchema,
    );

    // ADR-015, point 2: the one site that vouches for a composed text. It is
    // a constant of `statements` and one `literal`, and nothing else.
    let query = sqlx_core::raw_sql::raw_sql(sqlx_core::sql_str::AssertSqlSafe(compose(
        statement, schema,
    )?));

    let (runtime, connection) = session.parts();
    let connection = connection.ok_or_else(|| Error::InternalInvariant {
        invariant: CONNECTION_OPEN,
        location: Location::caller(),
    })?;

    runtime.block_on(async {
        let started = std::time::Instant::now();
        let answered = timeout(bound.remaining(), query.fetch_all(connection)).await;
        crate::diagnostics::emit::phase_ran(Phase::CatalogueQuery, started.elapsed());

        match answered {
            Err(_) => Err(expired()),
            Ok(answer) => Ok(answer),
        }
    })
}
