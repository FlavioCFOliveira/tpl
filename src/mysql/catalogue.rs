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
//! `NFR-PERF-002` hold by construction (`FR-MYSQL-042`); [`Catalogue`] keeps
//! what was issued, so a test reads the execution rather than the source.
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
    /// `NFR-PERF-001` and `NFR-PERF-002` read.
    #[cfg_attr(
        not(test),
        allow(
            dead_code,
            reason = "it exists to be observed by the assertions of NFR-PERF-001 and \
                      NFR-PERF-002, and a process that acted on it would be acting on its own plan"
        )
    )]
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

        issued.push(statement.sql());

        if let Some(slot) = rows.get_mut(statement.slot()) {
            *slot = fetched;
        }
    }

    Ok(Catalogue {
        server: session.server().clone(),
        schema: schema.to_owned(),
        issued,
        rows,
    })
}

/// Runs the privilege probe of `FR-CFG-044` over a settled session.
///
/// One `SELECT` against `SCHEMATA`, restricted to `schema` by a bind
/// parameter; the answer is true where it was answered without an error
/// packet and returned at least one row. No row is read into the model, and a
/// privilege refusal is an answer rather than a condition (`FR-CFG-045`), as
/// `crate::mariadb::catalogue::probe` records.
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
        Err(driver) if driver.as_database_error().is_some() => Ok(false),
        Err(driver) => Err(fault::speaking(
            target.entry(),
            &driver,
            target.host(),
            target.port(),
            SessionStatement::CatalogueQuery,
        )),
    }
}

/// Sends one statement, `schema` bound to its one parameter, and hands back
/// what the driver answered.
///
/// It is `crate::mariadb::catalogue`'s statement runner for a `mysql`
/// session: the bound of `FR-CONF-005` composed with `FR-GLOB-011`, the two
/// lines of `FR-GLOB-017`, and the bind that keeps the schema out of the text.
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

    let query = sqlx_core::query::query::<sqlx_mysql::MySql>(statement.sql()).bind(schema);

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
