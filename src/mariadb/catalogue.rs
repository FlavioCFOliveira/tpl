//! The catalogue read: the fourth entry of `FR-SRV-006`, and the model it
//! produces.
//!
//! This module is what task #127's session was opened for. `FR-SRV-042` puts
//! the three connection-start statements before every `SELECT` against
//! `INFORMATION_SCHEMA`, and [`super::open`] has already issued them: the
//! session is read only and confirmed, the product is MariaDB, and the series
//! is resolved. What follows is a **fixed repertoire** of catalogue queries
//! and the fold that turns their rows into [`crate::model`].
//!
//! | Submodule | Subject | Forced by |
//! |---|---|---|
//! | [`statements`] | The repertoire, and the plan one read issues | `FR-SRV-006`, `FR-SRV-037`, `NFR-PERF-001`, `NFR-PERF-002` |
//! | [`row`] | How one field of one row is decoded | The catalogue field lists of `FR-CAT-042`, `FR-CAT-045` … `FR-CAT-051` |
//! | [`fold`] | How the rows become the model | `FR-CAT-009` … `FR-CAT-015`, `FR-CAT-052`, `FR-CTX-036` |
//! | [`completeness`] | What the reader's privileges did not reach | `FR-PRIV-001` … `FR-PRIV-007`, `FR-PRIV-011` … `FR-PRIV-019` |
//!
//! # The count is fixed before the first statement is sent
//!
//! [`statements::plan`] is a function of the schema name and the [`Scope`] and
//! of nothing else, and [`read`] issues exactly what it returns — one
//! statement per entry, in order, and nothing besides. `NFR-PERF-001` and
//! `NFR-PERF-002` are therefore properties of the plan rather than of the
//! execution, and [`Catalogue::statements`] carries what was issued so that an
//! assertion reads the execution rather than the source.
//!
//! # The model borrows the rows, and the rows are kept for it
//!
//! Every string of [`crate::model`] is a [`Cow<'a, str>`](std::borrow::Cow),
//! and a live read is the path that always borrows. [`Catalogue`] therefore
//! **owns the fetched rows** and [`Catalogue::model`] hands back a
//! [`Database`] that borrows from them: no catalogue string is copied on the
//! way into the model, and the lifetime that makes that sound is the
//! caller's hold on the [`Catalogue`] rather than a convention.
//!
//! *Rejected: folding as the rows arrive and dropping each one.* It removes
//! the intermediate buffer and obliges every string in the model to be owned,
//! which is a copy of the whole catalogue — the cost `crate::model`'s own
//! rejection of `String` everywhere records. *Also rejected: returning the
//! model and the rows as one self-referential value.* It cannot be written in
//! safe Rust, and `unsafe` is barred outright.
//!
//! # A short read says so, object by object
//!
//! A reader whose privileges do not reach the whole model is not refused: it
//! receives rows, and the shortfall is in them. [`completeness`] holds the
//! three checks that find it — the empty string, SQL `NULL` and zero rows, per
//! `FR-PRIV-018` — and [`fold`] makes each of them as it maps the rows, so an
//! object that is short carries the `restricted` marking of `FR-PRIV-016` and
//! an object that is whole carries none.
//!
//! **What this module still does not do is decide what to do about it.**
//! `FR-PRIV-003` answers a caller that named one object with `77`, and
//! [`completeness::of_table`], [`completeness::of_view`] and
//! [`completeness::of_routine`] produce that verdict; the exit code is emitted
//! where every other one is, by the binary. The callers exist: the three
//! subcommands of `tpl schema` that name an object — `table`, `view` and
//! `routine` — take that verdict in `crate::cli::schema::named` once the object
//! they were given has been found, so an object that came back short is the
//! `77` of `FR-PRIV-003` rather than a document with a hole in it. The other
//! five of the eight read the same model and name no object, so `FR-PRIV-016`'s
//! marking is the whole of what a shortfall does to them.

pub(crate) mod completeness;
mod fold;
mod row;
mod statements;

// The harness is asked rather than restated: the gate, the inventory and the
// address of each server come from `scripts/mariadb/`, through the one module
// that speaks to it. It is reached by `#[path]` because
// `tests/support/fixture.rs` is not a test target of its own, which its own
// header records, and it is declared here rather than inside the test module
// because a `#[path]` inside an inline module resolves through a directory
// that does not exist.
// It is `pub(super)` because `super::fault` reaches the same harness and must
// not declare it a second time: two `#[path]` items over one file are two
// modules over one file.
#[cfg(test)]
#[path = "../../tests/support/fixture.rs"]
pub(super) mod fixture;

use std::panic::Location;

use sqlx::mysql::MySqlRow;
use tokio::time::timeout;

use super::Session;
use super::connect::Target;
use super::fault;
use crate::deadline::{Clock, Phase};
use crate::error::{Error, NetworkPhase};
use crate::model::database::Database;
use crate::model::server::Server;
use statements::{Read, Statement};

pub(crate) use statements::Scope;

/// The invariant a read on a closed session violates.
const CONNECTION_OPEN: &str = "the session a catalogue read is issued on is still open";

/// The rows a read returned, and the model they describe.
///
/// The value owns every row the plan fetched, so that [`Catalogue::model`] can
/// borrow the catalogue's own strings rather than copy them. It is inert: no
/// connection, no runtime, and nothing that has to be closed.
#[derive(Debug)]
pub(crate) struct Catalogue {
    /// The server the read was made against, as `FR-SRV-028` carries it into
    /// the model.
    server: Server<'static>,

    /// The database the read covered — the one the selected entry names, per
    /// `FR-CONF-041`.
    ///
    /// It is owned rather than borrowed so that a [`Catalogue`] outlives the
    /// settings the read was made from, as it already outlives the session.
    /// `FR-PRIV-021` is the one caller: a schema catalogue that returned no row
    /// leaves the fold with no row to read the name from, and that condition's
    /// `cause` must name the database all the same.
    schema: String,

    /// The statements that were issued, in order.
    ///
    /// It is what an assertion of `NFR-PERF-001` and `NFR-PERF-002` reads: the
    /// count of a read is observable from the value the read produced, so a
    /// test names what the process did rather than what the plan said it would
    /// do. The cost is one vector of at most eleven `&'static str`.
    ///
    /// It is read by the assertions of `NFR-PERF-001` and `NFR-PERF-002` and
    /// by nothing the process does, which is what it is for: the count is a
    /// property of the read, observed from the value the read produced.
    #[allow(
        dead_code,
        reason = "it exists to be observed by the assertions of NFR-PERF-001 and NFR-PERF-002, \
                  and a process that acted on it would be acting on its own plan"
    )]
    issued: Vec<&'static str>,

    /// The rows of each read, in the slot [`Read::slot`] gives it. A read the
    /// plan did not carry has none.
    rows: [Vec<MySqlRow>; Read::COUNT],
}

impl Catalogue {
    /// The model these rows describe (`FR-CTX-035`, `FR-CTX-036`).
    ///
    /// The answer borrows from `self`, which is what keeps the catalogue's
    /// strings uncopied.
    ///
    /// # Errors
    ///
    /// Returns [`Error::InternalInvariant`] where the catalogue returned a
    /// shape the model cannot represent, and [`Error::PropertyNotReadable`] —
    /// the `77` of `FR-PRIV-021` — where the schema catalogue returned no row
    /// for the database the read covered. [`fold`] records which shapes those
    /// are and why the two codes differ.
    pub(crate) fn model(&self) -> Result<Database<'_>, Error> {
        fold::database(self)
    }

    /// The database this read covered.
    fn schema(&self) -> &str {
        &self.schema
    }

    /// The statements this read issued, in the order they were issued.
    #[allow(
        dead_code,
        reason = "it exists to be observed by the assertions of NFR-PERF-001 and NFR-PERF-002, \
                  and a process that acted on it would be acting on its own plan"
    )]
    pub(crate) fn statements(&self) -> &[&'static str] {
        &self.issued
    }

    /// The rows of one read, or nothing where the plan did not carry it.
    fn rows(&self, read: Read) -> &[MySqlRow] {
        self.rows.get(read.slot()).map_or(&[], Vec::as_slice)
    }

    /// The server the read was made against.
    const fn server(&self) -> &Server<'static> {
        &self.server
    }
}

/// Reads the catalogue over a settled session (`FR-SRV-006`, `FR-SRV-042`).
///
/// `schema` is the database the read covers, and `scope` is how much of it.
/// Every statement is a `SELECT` against `INFORMATION_SCHEMA`, bounded by the
/// statement deadline of `FR-CONF-005` and composed with the overall budget as
/// `FR-GLOB-012` composes them.
///
/// The statements are issued **in the order the plan states**, and the plan is
/// complete before the first of them is sent: nothing a row says can add a
/// statement, which is `NFR-PERF-001` and `NFR-PERF-002` by construction. This
/// is also where `FR-SRV-023` is satisfied without a mechanism — no statement
/// is attempted in order to discover whether it works, because
/// [`statements`] names only columns every series of `FR-SRV-015` carries.
///
/// # Errors
///
/// Returns [`Error::NetworkDeadlineExceeded`] where a statement outlived its
/// bound, and what [`fault::speaking`] classifies for a driver failure — which
/// for an open session is the `69` of a server that stopped answering.
/// Returns [`Error::InternalInvariant`] where the session has already been
/// closed, which no caller of this crate can arrange: [`Session::close`]
/// consumes the session.
pub(crate) fn read(
    session: &mut Session,
    target: &Target<'_>,
    clock: &Clock,
    schema: &str,
    scope: &Scope<'_>,
) -> Result<Catalogue, Error> {
    let plan = statements::plan(schema, scope);
    let mut issued = Vec::with_capacity(plan.len());
    let mut rows: [Vec<MySqlRow>; Read::COUNT] = std::array::from_fn(|_| Vec::new());

    for statement in &plan {
        let fetched = fetch(session, target, clock, statement)?;

        issued.push(statement.sql());

        if let Some(slot) = rows.get_mut(statement.read().slot()) {
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

/// Issues one statement of the plan and takes its rows.
///
/// The bound is the statement deadline of `FR-CONF-005` — `core.query_timeout`
/// — composed with the overall budget of `FR-GLOB-011`, and the phase it
/// reports under is the catalogue query of `FR-ERR-034`.
///
/// Every value the statement varies by travels as a bind parameter, so the
/// text is the `&'static str` the repertoire holds and nothing a caller
/// supplies is composed into SQL.
fn fetch(
    session: &mut Session,
    target: &Target<'_>,
    clock: &Clock,
    statement: &Statement<'_>,
) -> Result<Vec<MySqlRow>, Error> {
    let bound = clock.bound(target.deadlines().of(Phase::CatalogueQuery));
    let expired = || {
        fault::expired(
            NetworkPhase::CatalogueQuery,
            target.host(),
            target.port(),
            bound,
        )
    };

    if bound.expired() {
        return Err(expired());
    }

    // FR-GLOB-017: exactly one line per catalogue query issued, at `INFO`, in a
    // form distinguishable from every other diagnostic line — which is what
    // makes the count observable from outside the process under `NFR-PERF-008`,
    // and therefore what makes `NFR-PERF-001` and `NFR-PERF-002` checkable at
    // all. The gate is inside the emission function, so the cost of a run that
    // does not emit is one load of an atomic per statement.
    //
    // It is written **here**, after the deadline check and before the statement
    // is sent, rather than after the statement returns: a statement that
    // reached the server and then failed was issued, and the server's own
    // record counts it, so a line written only on success would undercount
    // exactly the reads a caller most wants to see. A statement the bound
    // refused was never sent and is never counted, which is why the check comes
    // first.
    crate::diagnostics::emit::catalogue_query();

    let mut query = sqlx::query::<sqlx::MySql>(statement.sql());

    for bind in statement.binds() {
        query = query.bind(bind);
    }

    let runtime = &session.runtime;
    let connection = session
        .connection
        .as_mut()
        .ok_or_else(|| Error::InternalInvariant {
            invariant: CONNECTION_OPEN,
            location: Location::caller(),
        })?;

    runtime.block_on(async {
        let started = std::time::Instant::now();
        let answered = timeout(bound.remaining(), query.fetch_all(&mut *connection)).await;
        // FR-GLOB-017's other half: how long the phase took, beside the line
        // above that says it happened. The two are separate obligations of one
        // requirement and carry separate tokens, so the count of catalogue
        // queries `NFR-PERF-008` reads is untouched by this line.
        crate::diagnostics::emit::phase_ran(Phase::CatalogueQuery, started.elapsed());

        match answered {
            Err(_) => Err(expired()),
            Ok(Ok(rows)) => Ok(rows),
            Ok(Err(driver)) => Err(fault::speaking(&driver, target.host(), target.port())),
        }
    })
}

#[cfg(test)]
mod tests {
    use super::statements::Read;
    use super::{Catalogue, Scope, completeness, read};
    use crate::error::{CatalogueObjectKind, Error};
    use crate::mariadb::connect::Target;
    use crate::model::check_constraint::ConstraintLevel;
    use crate::model::column::GeneratedStorage;
    use crate::model::column_default::DefaultKind;
    use crate::model::foreign_key::ReferentialAction;
    use crate::model::index::PRIMARY_KEY_NAME;
    use crate::model::routine::RoutineKind;
    use crate::model::server::Standing;
    use crate::model::table::TableType;
    use crate::model::trigger::{TriggerEvent, TriggerTiming};
    use crate::project::config;
    use crate::project::scratch::Scratch;
    use crate::project::settings::{self, Settings};

    use super::fixture;

    /// The schema every fixture server carries.
    const SCHEMA: &str = "freight";

    /// The database entry every test below resolves.
    const ENTRY: &str = "fixture";

    /// The privileged account of the fixture, which reads every property.
    const ROOT: (&str, &str) = ("root", "tpl-root");

    /// The reduced-grant reader of `FR-PRIV-018`, which holds
    /// `SELECT, EXECUTE ON freight.*` and nothing else.
    ///
    /// It is the account the fixture carries so that an incomplete read is
    /// reproducible without breaking anything, and it is the only way the three
    /// shapes of `FR-PRIV-018` can be observed rather than simulated.
    const REDUCED: (&str, &str) = ("tpl_reader", "tpl-reader-pw");

    /// A database name no fixture server carries.
    ///
    /// A read issued with it returns no row from the schema catalogue, which
    /// is the condition `FR-PRIV-021` governs.
    const ABSENT_SCHEMA: &str = "a_schema_no_server_of_the_fixture_carries";

    /// The settings that reach `address` as `account`, over a plain connection.
    ///
    /// The transport is not this module's subject — `FR-CONF-013`'s five modes
    /// are `connect`'s, and the fixture's certificate is exercised where that
    /// belongs — so the entry asks for the one mode that adds nothing to what
    /// is under test here.
    fn settings(scratch: &Scratch, address: &str, account: (&str, &str)) -> Settings {
        let (host, port) = address
            .rsplit_once(':')
            .expect("status.sh --export prints host:port");
        let (user, password) = account;

        let file = scratch.file(
            ".cfg",
            &format!(
                "[database.{ENTRY}]\nhost = \"{host}\"\nport = {port}\n\
                 user = \"{user}\"\npassword = \"{password}\"\n\
                 database = \"{SCHEMA}\"\ntls = \"disabled\"\n"
            ),
        );
        let configuration = config::load(&file).expect("the document is valid");

        settings::resolve(
            &configuration,
            Some(ENTRY),
            &settings::clock(None),
            &|_: &str| None,
        )
        .expect("the entry resolves")
    }

    /// Reads `scope` from one fixture server as `account`, and closes the
    /// session.
    ///
    /// The answer outlives the session on purpose: a [`Catalogue`] owns its
    /// rows, so the one connection of `NFR-PERF-004` is closed as soon as the
    /// read ends.
    fn read_as(server: &fixture::Server, account: (&str, &str), scope: Scope<'_>) -> Catalogue {
        let scratch = Scratch::new();
        let resolved = settings(&scratch, server.address(), account);
        let target = Target::of(&resolved).expect("the entry names a host");
        let clock = settings::clock(None);

        let mut session = crate::mariadb::open(&target, &clock)
            .unwrap_or_else(|failure| panic!("{} did not answer: {failure}", server.name()));
        let catalogue = read(&mut session, &target, &clock, SCHEMA, &scope)
            .unwrap_or_else(|failure| panic!("{} refused the read: {failure}", server.name()));

        session.close();

        catalogue
    }

    /// Reads `scope` from one fixture server as the privileged account.
    fn read_from(server: &fixture::Server, scope: Scope<'_>) -> Catalogue {
        read_as(server, ROOT, scope)
    }

    /// Reads `scope` from one fixture server as the reduced-grant reader.
    fn read_reduced(server: &fixture::Server, scope: Scope<'_>) -> Catalogue {
        read_as(server, REDUCED, scope)
    }

    /// The property names one object's marking carries, or [`None`] where the
    /// object carries no marking at all (`FR-PRIV-007`).
    fn marked<'a>(
        marking: Option<&'a crate::model::restricted::Restricted<'a>>,
    ) -> Option<Vec<&'a str>> {
        marking.map(|marking| {
            marking
                .properties()
                .iter()
                .map(std::convert::AsRef::as_ref)
                .collect()
        })
    }

    /// Runs `body` against every series of `FR-SRV-015`, or reports the skip.
    ///
    /// `FR-SRV-029` requires a test of cross-series behaviour to run on every
    /// series, and every test below is one: a catalogue read is the thing the
    /// four series were observed to differ about.
    fn on_every_series(test: &str, body: impl Fn(&fixture::Server)) {
        let Some(series) = fixture::series(test) else {
            return;
        };
        let _exclusive = fixture::exclusive();

        for server in series {
            body(server);
        }
    }

    #[test]
    fn fr_srv_037_the_repertoire_runs_unmodified_on_every_series_of_the_window() {
        // FR-SRV-037: no statement names a column absent from a series, so the
        // same eleven statements run on all four. A column list that named one
        // would be ERROR 1054 (42S22) here and nowhere else — which is the
        // failure FR-SRV-023 bars the reader from discovering by attempting.
        on_every_series(
            "fr_srv_037_the_repertoire_runs_unmodified_on_every_series_of_the_window",
            |server| {
                let catalogue = read_from(server, Scope::Everything);
                let model = catalogue.model().expect("the rows fold");

                assert_eq!(catalogue.statements().len(), 11, "{}", server.name());
                assert_eq!(model.name, SCHEMA);
                assert_eq!(model.charset, "utf8mb4");
                assert_eq!(model.tables.len(), 17, "{}", server.name());
                assert_eq!(model.views.len(), 5, "{}", server.name());
                assert_eq!(model.routines.len(), 7, "{}", server.name());
            },
        );
    }

    #[test]
    fn nfr_perf_001_a_full_read_issues_eleven_statements_whatever_the_database_holds() {
        // NFR-PERF-001: the count over a database of many objects equals the
        // count over a database of none. `freight` carries 23 catalogue
        // objects and the schema below carries nothing at all — it does not
        // exist — and the two reads issue byte-identical statements.
        on_every_series(
            "nfr_perf_001_a_full_read_issues_eleven_statements_whatever_the_database_holds",
            |server| {
                let populated = read_from(server, Scope::Everything);

                assert_eq!(populated.statements().len(), 11);

                let empty = read_empty_schema(server);

                assert_eq!(empty, populated.statements());
            },
        );
    }

    #[test]
    fn nfr_perf_002_a_named_read_issues_a_count_that_does_not_depend_on_the_database() {
        // NFR-PERF-002: reading one object issues the statements that object
        // needs and no more, and the narrowing is in SQL — a named table read
        // returns one table from a schema of seventeen.
        on_every_series(
            "nfr_perf_002_a_named_read_issues_a_count_that_does_not_depend_on_the_database",
            |server| {
                let table = read_from(server, Scope::Table("consignment"));
                let view = read_from(server, Scope::View("v_consignment_manifest"));

                assert_eq!(table.statements().len(), 8);
                assert_eq!(view.statements().len(), 2);

                let table = table.model().expect("the rows fold");
                let view = view.model().expect("the rows fold");

                assert_eq!(table.tables.len(), 1);
                assert_eq!(table.tables[0].name(), "consignment");
                assert!(table.views.is_empty());
                assert_eq!(view.views.len(), 1);
                assert!(view.tables.is_empty());
            },
        );
    }

    #[test]
    fn fr_sch_008_a_routine_is_read_by_a_bare_name_and_by_a_qualified_one() {
        // FR-SCH-008: the qualified forms name one of the two namespaces a
        // routine can live in, and a bare name names neither. Both narrow in
        // SQL and both issue three statements, so FR-SCH-010's ambiguity is a
        // count the caller reads rather than a second read it has to make.
        on_every_series(
            "fr_sch_008_a_routine_is_read_by_a_bare_name_and_by_a_qualified_one",
            |server| {
                let bare = read_from(
                    server,
                    Scope::Routine {
                        name: "sp_book_consignment",
                        kind: None,
                    },
                );
                let qualified = read_from(
                    server,
                    Scope::Routine {
                        name: "sp_book_consignment",
                        kind: Some(RoutineKind::Procedure),
                    },
                );
                let absent = read_from(
                    server,
                    Scope::Routine {
                        name: "sp_book_consignment",
                        kind: Some(RoutineKind::Function),
                    },
                );

                assert_eq!(bare.statements().len(), 3);
                assert_eq!(qualified.statements().len(), 3);
                assert_eq!(absent.statements().len(), 3);

                for read in [&bare, &qualified] {
                    let model = read.model().expect("the rows fold");

                    assert_eq!(model.routines.len(), 1);
                    assert_eq!(model.routines[0].kind, RoutineKind::Procedure);
                    assert_eq!(model.routines[0].parameters.len(), 9);
                    assert!(model.routines[0].return_type.is_none());
                }

                // The kind narrows in SQL: no function of that name exists, so
                // the read returns nothing rather than the procedure.
                assert!(absent.model().expect("the rows fold").routines.is_empty());
            },
        );
    }

    #[test]
    fn fr_cat_004_a_sequence_a_system_view_and_a_temporary_table_are_excluded_by_the_type_filter() {
        // FR-CAT-004, FR-CAT-005 and FR-CAT-006, through the filter
        // FR-CAT-032 requires: the fixture's 23 catalogue objects are 16 base
        // tables, one system-versioned table, five views and one sequence, and
        // the model carries the first two kinds as tables and the third as
        // views. FR-CAT-003 keeps a view out of `tables`.
        on_every_series(
            "fr_cat_004_a_sequence_a_system_view_and_a_temporary_table_are_excluded_by_the_type_filter",
            |server| {
                let catalogue = read_from(server, Scope::Everything);
                let model = catalogue.model().expect("the rows fold");
                let names: Vec<&str> = model.tables.iter().map(|table| table.name()).collect();

                assert_eq!(model.tables.len(), 17);
                assert!(!names.contains(&"reference_number_seq"), "{names:?}");

                let versioned = model
                    .tables
                    .iter()
                    .filter(|table| table.table_type() == TableType::SystemVersioned)
                    .count();

                assert_eq!(versioned, 1);

                for view in &model.views {
                    assert!(!names.contains(&view.name.as_ref()), "{}", view.name);
                }
            },
        );
    }

    #[test]
    fn fr_cat_052_no_column_of_an_object_the_model_does_not_cover_is_presented() {
        // FR-CAT-052: the column catalogue is not restricted to the objects
        // the model covers. It carries eight rows for the fixture's sequence
        // and a full set for every view — 52 of its 301 rows — and none of
        // them reaches a table, because the object each hangs from is not one.
        on_every_series(
            "fr_cat_052_no_column_of_an_object_the_model_does_not_cover_is_presented",
            |server| {
                let catalogue = read_from(server, Scope::Everything);
                let model = catalogue.model().expect("the rows fold");
                let columns: usize = model.tables.iter().map(|table| table.columns().len()).sum();

                // 301 catalogue rows less the sequence's eight and the views'
                // 44, observed identically on all four series.
                assert_eq!(columns, 249, "{}", server.name());

                let sequence = ["next_not_cached_value", "minimum_value", "cycle_count"];

                for table in &model.tables {
                    for column in table.columns() {
                        assert!(!sequence.contains(&column.name.as_ref()), "{}", column.name);
                        assert_eq!(column.table_name, table.name());
                    }
                }
            },
        );
    }

    #[test]
    fn fr_cat_009_a_table_carries_its_columns_in_ordinal_position_order() {
        // FR-CAT-009 with NFR-DET-002: ordinal position, and generated and
        // invisible columns among them.
        on_every_series(
            "fr_cat_009_a_table_carries_its_columns_in_ordinal_position_order",
            |server| {
                let catalogue = read_from(server, Scope::Everything);
                let model = catalogue.model().expect("the rows fold");

                for table in &model.tables {
                    let positions: Vec<u64> = table
                        .columns()
                        .iter()
                        .map(|column| column.position)
                        .collect();
                    let mut ordered = positions.clone();
                    ordered.sort_unstable();

                    assert_eq!(positions, ordered, "{}", table.name());
                    assert_eq!(positions.first(), Some(&1), "{}", table.name());
                }

                let invisible: usize = model
                    .tables
                    .iter()
                    .flat_map(crate::model::table::Table::columns)
                    .filter(|column| column.invisible)
                    .count();

                assert_eq!(invisible, 1, "{}", server.name());
            },
        );
    }

    #[test]
    fn fr_cat_041_every_static_attribute_is_read_from_the_one_attribute_field() {
        // FR-CAT-041 records what the column attribute field holds over the
        // fixture's 301 columns: `auto_increment` on 12, `INVISIBLE` on 1,
        // `STORED GENERATED` on 6, `VIRTUAL GENERATED` on 3, and the two
        // `on update` forms on one column each. Nine of those columns belong
        // to covered tables, and the counts below are what survives the
        // coverage filter.
        on_every_series(
            "fr_cat_041_every_static_attribute_is_read_from_the_one_attribute_field",
            |server| {
                let catalogue = read_from(server, Scope::Everything);
                let model = catalogue.model().expect("the rows fold");
                let columns: Vec<_> = model
                    .tables
                    .iter()
                    .flat_map(crate::model::table::Table::columns)
                    .collect();

                let auto = columns
                    .iter()
                    .filter(|column| column.auto_increment)
                    .count();
                let generated = columns
                    .iter()
                    .filter(|column| column.generated.is_some())
                    .count();
                let on_update = columns
                    .iter()
                    .filter(|column| column.on_update.is_some())
                    .count();

                assert_eq!(auto, 12, "{}", server.name());
                assert_eq!(generated, 9, "{}", server.name());
                assert_eq!(on_update, 2, "{}", server.name());

                for column in &columns {
                    if let Some(on_update) = &column.on_update {
                        assert!(on_update.starts_with("on update "), "{on_update}");
                    }
                }
            },
        );
    }

    #[test]
    fn fr_cat_051_a_generated_column_carries_its_expression_and_its_storage_kind() {
        // FR-CAT-051: the storage kind comes from the attribute field and
        // never from the is-generated field, and the expression arrives
        // rewritten — identifiers backtick-quoted, function names lower-cased.
        on_every_series(
            "fr_cat_051_a_generated_column_carries_its_expression_and_its_storage_kind",
            |server| {
                let catalogue = read_from(server, Scope::Everything);
                let model = catalogue.model().expect("the rows fold");
                let generated: Vec<_> = model
                    .tables
                    .iter()
                    .flat_map(crate::model::table::Table::columns)
                    .filter_map(|column| column.generated.as_ref())
                    .collect();

                assert_eq!(generated.len(), 9, "{}", server.name());

                let stored = generated
                    .iter()
                    .filter(|generated| generated.storage == GeneratedStorage::Stored)
                    .count();

                assert_eq!(stored, 6, "{}", server.name());

                for generated in &generated {
                    assert!(!generated.expression.is_empty());
                    assert!(!generated.expression.contains(" AS "));
                    assert!(!generated.expression.contains("VIRTUAL"));
                    assert!(!generated.expression.contains("STORED"));
                }
            },
        );
    }

    #[test]
    fn fr_cat_010_an_index_is_one_object_carrying_its_columns_in_the_catalogue_order() {
        // FR-CAT-010: 77 index rows fold into the indexes they belong to, and
        // a multi-column index keeps the order the sequence field states —
        // `voyage_leg`'s primary key is (vessel_imo, voyage_number,
        // leg_sequence), which sorted by name would be a different key.
        on_every_series(
            "fr_cat_010_an_index_is_one_object_carrying_its_columns_in_the_catalogue_order",
            |server| {
                let catalogue = read_from(server, Scope::Everything);
                let model = catalogue.model().expect("the rows fold");
                let members: usize = model
                    .tables
                    .iter()
                    .flat_map(crate::model::table::Table::indexes)
                    .map(|index| index.columns.len())
                    .sum();

                assert_eq!(members, 77, "{}", server.name());

                let leg = model
                    .tables
                    .iter()
                    .find(|table| table.name() == "voyage_leg")
                    .expect("the fixture carries voyage_leg");
                let primary = leg.primary_key().expect("voyage_leg has a primary key");
                let columns: Vec<&str> = primary
                    .columns
                    .iter()
                    .map(|column| column.name.as_ref())
                    .collect();

                assert_eq!(columns, ["vessel_imo", "voyage_number", "leg_sequence"]);
                assert!(primary.unique);
            },
        );
    }

    #[test]
    fn fr_cat_043_the_primary_key_is_the_index_named_primary_and_names_no_period_column() {
        // FR-CAT-043 and FR-CAT-044: key column usage reports the primary key
        // of the system-versioned `tariff` as (tariff_id, row_end), and the
        // column catalogue carries no row for `row_end` on any table. The
        // index catalogue reports (tariff_id), which is what the model
        // carries — and Table::assemble would have refused the other.
        on_every_series(
            "fr_cat_043_the_primary_key_is_the_index_named_primary_and_names_no_period_column",
            |server| {
                let catalogue = read_from(server, Scope::Everything);
                let model = catalogue.model().expect("the rows fold");
                let tariff = model
                    .tables
                    .iter()
                    .find(|table| table.name() == "tariff")
                    .expect("the fixture carries tariff");

                assert_eq!(tariff.table_type(), TableType::SystemVersioned);

                let primary = tariff.primary_key().expect("tariff has a primary key");

                assert_eq!(primary.name, PRIMARY_KEY_NAME);
                assert_eq!(primary.columns.len(), 1);
                assert_eq!(primary.columns[0].name, "tariff_id");

                for table in &model.tables {
                    let carried: Vec<&str> = table
                        .columns()
                        .iter()
                        .map(|column| column.name.as_ref())
                        .collect();

                    assert!(!carried.contains(&"row_end"), "{}", table.name());
                    assert!(!carried.contains(&"row_start"), "{}", table.name());
                }
            },
        );
    }

    #[test]
    fn fr_cat_045_a_foreign_key_carries_its_rules_and_its_columns_paired_by_position() {
        // FR-CAT-045: the fixture declares fifteen foreign keys over 17
        // columns, and the catalogue returns fourteen distinct rule pairs
        // because the one declared SET DEFAULT comes back RESTRICT/RESTRICT,
        // per FR-CAT-033.
        on_every_series(
            "fr_cat_045_a_foreign_key_carries_its_rules_and_its_columns_paired_by_position",
            |server| {
                let catalogue = read_from(server, Scope::Everything);
                let model = catalogue.model().expect("the rows fold");
                let keys: Vec<_> = model
                    .tables
                    .iter()
                    .flat_map(crate::model::table::Table::foreign_keys)
                    .collect();
                let members: usize = keys.iter().map(|key| key.columns.len()).sum();

                assert_eq!(keys.len(), 15, "{}", server.name());
                assert_eq!(members, 17, "{}", server.name());

                for key in &keys {
                    assert_eq!(key.match_option, "NONE");
                    // FR-CAT-056: both are declared nullable and both
                    // were observed populated on all 15 rules of the fixture.
                    assert_eq!(key.referenced_key.as_deref(), Some(PRIMARY_KEY_NAME));
                    assert!(
                        key.referenced_table
                            .as_deref()
                            .is_some_and(|named| !named.is_empty())
                    );
                    assert!(!key.columns.is_empty());
                }

                let actions = [
                    ReferentialAction::Cascade,
                    ReferentialAction::NoAction,
                    ReferentialAction::Restrict,
                    ReferentialAction::SetNull,
                ];

                for action in actions {
                    assert!(
                        keys.iter().any(|key| key.on_delete == action),
                        "{action:?} was not returned"
                    );
                }
            },
        );
    }

    #[test]
    fn fr_cat_013_the_incoming_direction_is_carried_under_referenced_by() {
        // FR-CAT-013 and FR-CAT-045: the incoming direction is the outgoing
        // rows of every other table, read from the other end, so the two
        // collections hold the same fifteen keys.
        on_every_series(
            "fr_cat_013_the_incoming_direction_is_carried_under_referenced_by",
            |server| {
                let catalogue = read_from(server, Scope::Everything);
                let model = catalogue.model().expect("the rows fold");
                let incoming: usize = model
                    .tables
                    .iter()
                    .map(|table| table.referenced_by().len())
                    .sum();

                assert_eq!(incoming, 15, "{}", server.name());

                for table in &model.tables {
                    for entry in table.referenced_by() {
                        assert_eq!(entry.key.referenced_table.as_deref(), Some(table.name()));
                        assert_ne!(entry.table, "");
                    }
                }
            },
        );
    }

    #[test]
    fn fr_cat_046_a_check_constraint_carries_its_name_its_level_and_its_clause() {
        // FR-CAT-046: 24 constraints, 22 at table level and two at column
        // level — and FR-CAT-038's implicit json_valid constraints are among
        // them, carried because FR-CAT-015 admits no exception.
        on_every_series(
            "fr_cat_046_a_check_constraint_carries_its_name_its_level_and_its_clause",
            |server| {
                let catalogue = read_from(server, Scope::Everything);
                let model = catalogue.model().expect("the rows fold");
                let constraints: Vec<_> = model
                    .tables
                    .iter()
                    .flat_map(crate::model::table::Table::check_constraints)
                    .collect();

                assert_eq!(constraints.len(), 24, "{}", server.name());

                let column_level = constraints
                    .iter()
                    .filter(|constraint| constraint.level == ConstraintLevel::Column)
                    .count();

                assert_eq!(column_level, 2, "{}", server.name());

                for constraint in &constraints {
                    assert!(!constraint.name.is_empty());
                    assert!(!constraint.clause.is_empty());
                }
            },
        );
    }

    #[test]
    fn fr_cat_050_a_trigger_carries_the_fields_its_catalogue_field_list_names() {
        // FR-CAT-050: six triggers, all six combinations of the three events
        // and the two timings, and the two row-alias fields reading OLD and
        // NEW on every row whatever the event.
        on_every_series(
            "fr_cat_050_a_trigger_carries_the_fields_its_catalogue_field_list_names",
            |server| {
                let catalogue = read_from(server, Scope::Everything);
                let model = catalogue.model().expect("the rows fold");
                let triggers: Vec<_> = model
                    .tables
                    .iter()
                    .flat_map(crate::model::table::Table::triggers)
                    .collect();

                assert_eq!(triggers.len(), 6, "{}", server.name());

                for event in [
                    TriggerEvent::Insert,
                    TriggerEvent::Update,
                    TriggerEvent::Delete,
                ] {
                    for timing in [TriggerTiming::Before, TriggerTiming::After] {
                        assert!(
                            triggers
                                .iter()
                                .any(|trigger| trigger.event == event && trigger.timing == timing),
                            "{event:?} {timing:?} is absent"
                        );
                    }
                }

                for trigger in &triggers {
                    assert_eq!(trigger.action_order, 1);
                    assert_eq!(trigger.orientation, "ROW");
                    assert_eq!(trigger.old_row_alias, "OLD");
                    assert_eq!(trigger.new_row_alias, "NEW");
                    // FR-CAT-056: both are declared nullable and both were
                    // observed populated on all 6 triggers of the fixture.
                    assert!(
                        trigger
                            .statement
                            .as_deref()
                            .is_some_and(|body| !body.is_empty())
                    );
                    assert!(
                        trigger
                            .definer
                            .as_deref()
                            .is_some_and(|definer| !definer.is_empty())
                    );
                }
            },
        );
    }

    #[test]
    fn fr_cat_047_a_view_carries_nine_fields_and_its_definition() {
        // FR-CAT-047: the definition is the server's rewritten form, and all
        // three check options and both updatability values appear over the
        // fixture's five views.
        on_every_series(
            "fr_cat_047_a_view_carries_nine_fields_and_its_definition",
            |server| {
                let catalogue = read_from(server, Scope::Everything);
                let model = catalogue.model().expect("the rows fold");

                assert_eq!(model.views.len(), 5, "{}", server.name());

                for view in &model.views {
                    assert!(!view.definition.is_empty());
                    assert!(!view.definer.is_empty());
                    assert_eq!(view.algorithm, "UNDEFINED");
                    assert_eq!(view.character_set_client, "utf8mb4");
                    assert!(view.restricted.is_none());
                }

                for option in ["NONE", "LOCAL", "CASCADED"] {
                    assert!(
                        model.views.iter().any(|view| view.check_option == option),
                        "{option} is absent"
                    );
                }

                assert!(model.views.iter().any(|view| view.is_updatable));
                assert!(model.views.iter().any(|view| !view.is_updatable));
            },
        );
    }

    #[test]
    fn fr_cat_048_a_routine_carries_its_kind_its_body_and_a_return_type_only_where_it_has_one() {
        // FR-CAT-048 with FR-CAT-016: four functions and three procedures, the
        // kind carried as the catalogue's own string, and the return type
        // `null` in full for a procedure.
        on_every_series(
            "fr_cat_048_a_routine_carries_its_kind_its_body_and_a_return_type_only_where_it_has_one",
            |server| {
                let catalogue = read_from(server, Scope::Everything);
                let model = catalogue.model().expect("the rows fold");

                assert_eq!(model.routines.len(), 7, "{}", server.name());

                let functions = model
                    .routines
                    .iter()
                    .filter(|routine| routine.kind == RoutineKind::Function)
                    .count();

                assert_eq!(functions, 4, "{}", server.name());

                for routine in &model.routines {
                    // FR-CAT-056: declared nullable, populated on all 7
                    // routines for a privileged reader.
                    assert!(
                        routine.body.as_deref().is_some_and(|body| !body.is_empty()),
                        "{}",
                        routine.name
                    );
                    assert_eq!(routine.body_kind, "SQL");
                    assert_eq!(routine.parameter_style, "SQL");
                    assert_eq!(routine.security_type, "DEFINER");
                    assert!(routine.restricted.is_none());

                    match &routine.kind {
                        RoutineKind::Procedure => assert!(routine.return_type.is_none()),
                        RoutineKind::Function => {
                            let returned = routine
                                .return_type
                                .as_ref()
                                .expect("a function states its return type");

                            assert!(!returned.raw().is_empty(), "{}", routine.name);
                            assert!(returned.data_type().is_some(), "{}", routine.name);
                        }
                        // FR-CAT-016: the two strings are what a server whose
                        // `standing` is `supported` returns, and every server
                        // of the fixture is one. A third kind here would
                        // falsify that requirement, which is `FR-CAT-055`'s own
                        // *Consequence* arriving as a test failure rather than
                        // as a wrong document.
                        unrecorded => panic!(
                            "{} reported routine kind {:?}, which FR-CAT-016 does not admit \
                             from a supported series",
                            routine.name,
                            unrecorded.name()
                        ),
                    }
                }
            },
        );
    }

    #[test]
    fn fr_cat_049_a_functions_return_row_is_not_presented_as_a_parameter() {
        // FR-CAT-049: the parameter catalogue holds 22 rows for seven
        // routines — 18 declared parameters and four function returns — and
        // the return row is excluded, so no parameter arrives without a name
        // or without a mode.
        on_every_series(
            "fr_cat_049_a_functions_return_row_is_not_presented_as_a_parameter",
            |server| {
                let catalogue = read_from(server, Scope::Everything);
                let model = catalogue.model().expect("the rows fold");
                let parameters: usize = model
                    .routines
                    .iter()
                    .map(|routine| routine.parameters.len())
                    .sum();

                assert_eq!(parameters, 18, "{}", server.name());

                for routine in &model.routines {
                    for parameter in &routine.parameters {
                        assert!(!parameter.name.is_empty(), "{}", routine.name);
                        assert!(
                            ["IN", "OUT", "INOUT"].contains(&parameter.mode.as_ref()),
                            "{}",
                            parameter.mode
                        );
                        assert!(!parameter.parameter_type.raw().is_empty());
                    }
                }
            },
        );
    }

    #[test]
    fn fr_ctx_036_the_database_carries_the_three_metadata_fields_and_the_server() {
        // FR-CTX-036 and FR-CTX-031: the three fields come from the schema
        // catalogue and the server from the probe of FR-SRV-002, which
        // BR-CTX-006 records as not coming from the catalogue at all.
        on_every_series(
            "fr_ctx_036_the_database_carries_the_three_metadata_fields_and_the_server",
            |server| {
                let catalogue = read_from(server, Scope::Everything);
                let model = catalogue.model().expect("the rows fold");

                assert_eq!(model.name, SCHEMA);
                assert_eq!(model.charset, "utf8mb4");
                assert_eq!(model.collation, "utf8mb4_unicode_520_ci");
                assert!(model.server.version().contains("MariaDB"));
                assert_eq!(model.server.series(), server.name());
            },
        );
    }

    #[test]
    fn nfr_det_001_two_reads_of_one_unchanged_server_produce_the_same_model() {
        // NFR-DET-001, and the ordering of NFR-DET-002 beneath it: nothing in
        // the model depends on the order the server returned rows in, so two
        // reads made in one run compare equal.
        on_every_series(
            "nfr_det_001_two_reads_of_one_unchanged_server_produce_the_same_model",
            |server| {
                let first = read_from(server, Scope::Everything);
                let second = read_from(server, Scope::Everything);

                assert_eq!(
                    format!("{:?}", first.model().expect("the rows fold")),
                    format!("{:?}", second.model().expect("the rows fold"))
                );
            },
        );
    }

    #[test]
    fn nfr_det_002_every_collection_the_default_rule_governs_is_ordered_byte_wise() {
        // NFR-DET-002: tables, views, routines, indexes, triggers and check
        // constraints are ordered by name ascending, byte-wise. A table's
        // columns are not among them — ordinal position is one of the six
        // exceptions — and neither is an index's column list.
        on_every_series(
            "nfr_det_002_every_collection_the_default_rule_governs_is_ordered_byte_wise",
            |server| {
                let catalogue = read_from(server, Scope::Everything);
                let model = catalogue.model().expect("the rows fold");

                assert_ordered(
                    &model
                        .tables
                        .iter()
                        .map(|table| table.name())
                        .collect::<Vec<_>>(),
                );
                assert_ordered(
                    &model
                        .views
                        .iter()
                        .map(|view| view.name.as_ref())
                        .collect::<Vec<_>>(),
                );
                assert_ordered(
                    &model
                        .routines
                        .iter()
                        .map(|routine| routine.name.as_ref())
                        .collect::<Vec<_>>(),
                );

                for table in &model.tables {
                    assert_ordered(
                        &table
                            .indexes()
                            .iter()
                            .map(|index| index.name.as_ref())
                            .collect::<Vec<_>>(),
                    );
                    assert_ordered(
                        &table
                            .triggers()
                            .iter()
                            .map(|trigger| trigger.name.as_ref())
                            .collect::<Vec<_>>(),
                    );
                    assert_ordered(
                        &table
                            .check_constraints()
                            .iter()
                            .map(|constraint| constraint.name.as_ref())
                            .collect::<Vec<_>>(),
                    );
                    assert_ordered(
                        &table
                            .foreign_keys()
                            .iter()
                            .map(|key| key.name.as_ref())
                            .collect::<Vec<_>>(),
                    );
                }
            },
        );
    }

    #[test]
    fn fr_ctx_037_a_column_default_reaches_the_model_classified_rather_than_raw() {
        // FR-CTX-037 and FR-CTX-011: the catalogue answers with one string and
        // the model carries the three-way discriminant, so no template has to
        // classify it. All three kinds appear over the fixture's columns.
        on_every_series(
            "fr_ctx_037_a_column_default_reaches_the_model_classified_rather_than_raw",
            |server| {
                let catalogue = read_from(server, Scope::Everything);
                let model = catalogue.model().expect("the rows fold");
                let defaults: Vec<_> = model
                    .tables
                    .iter()
                    .flat_map(crate::model::table::Table::columns)
                    .filter_map(|column| column.default.as_ref())
                    .collect();

                for kind in [
                    DefaultKind::Literal,
                    DefaultKind::Expression,
                    DefaultKind::Null,
                ] {
                    assert!(
                        defaults.iter().any(|default| default.kind() == kind),
                        "{} returned no {} default",
                        server.name(),
                        kind.name()
                    );
                }
            },
        );
    }

    #[test]
    fn fr_priv_011_a_view_whose_definition_is_the_empty_string_is_reported_incomplete() {
        // FR-PRIV-011, the first shape of FR-PRIV-018, observed rather than
        // simulated: the reduced-grant reader holds no SHOW VIEW, so all five
        // views arrive with every attribute and an empty definition. The row
        // is present, which is why the check is on the value and not on a
        // count of rows.
        on_every_series(
            "fr_priv_011_a_view_whose_definition_is_the_empty_string_is_reported_incomplete",
            |server| {
                let catalogue = read_reduced(server, Scope::Everything);
                let model = catalogue.model().expect("the rows fold");

                assert_eq!(model.views.len(), 5, "{}", server.name());

                for view in &model.views {
                    assert!(view.definition.is_empty(), "{}", view.name);
                    assert_eq!(
                        marked(view.restricted.as_ref()),
                        Some(vec!["definition"]),
                        "{}",
                        view.name
                    );
                }
            },
        );
    }

    #[test]
    fn fr_priv_017_a_routine_whose_body_is_null_is_reported_incomplete() {
        // FR-PRIV-017, the second shape of FR-PRIV-018. The catalogue answers
        // SQL NULL and not the empty string, and the model has no shape for a
        // NULL, so the verdict is taken on the row: the same reader sees all
        // seven routines and every one of their bodies is gone.
        on_every_series(
            "fr_priv_017_a_routine_whose_body_is_null_is_reported_incomplete",
            |server| {
                let catalogue = read_reduced(server, Scope::Everything);
                let model = catalogue.model().expect("the rows fold");

                assert_eq!(model.routines.len(), 7, "{}", server.name());

                for routine in &model.routines {
                    assert_eq!(
                        marked(routine.restricted.as_ref()),
                        Some(vec!["body"]),
                        "{}",
                        routine.name
                    );
                }
            },
        );
    }

    #[test]
    fn fr_priv_019_a_key_column_under_no_referential_constraint_row_marks_both_ends() {
        // FR-PRIV-019, the third shape of FR-PRIV-018. The reduced reader
        // receives zero rows from REFERENTIAL_CONSTRAINTS and every row from
        // KEY_COLUMN_USAGE, so the fixture's 15 keys are seen as 17 columns
        // under no rule at all. Nine tables lose their outgoing rules and nine
        // lose their incoming ones; four tables are in both lists.
        on_every_series(
            "fr_priv_019_a_key_column_under_no_referential_constraint_row_marks_both_ends",
            |server| {
                let catalogue = read_reduced(server, Scope::Everything);
                let model = catalogue.model().expect("the rows fold");

                assert!(catalogue.rows(Read::ForeignKeyRules).is_empty());
                assert_eq!(catalogue.rows(Read::KeyColumns).len(), 17);

                let referencing = ["cargo_item", "document", "voyage_leg"];
                let referenced = ["charge_code", "customer", "vessel"];
                let both = ["consignment", "container", "customs_declaration", "voyage"];

                for table in &model.tables {
                    let expected = if both.contains(&table.name()) {
                        Some(vec!["foreign_keys", "referenced_by"])
                    } else if referencing.contains(&table.name()) {
                        Some(vec!["foreign_keys"])
                    } else if referenced.contains(&table.name()) {
                        Some(vec!["referenced_by"])
                    } else {
                        continue;
                    };

                    assert_eq!(marked(table.restricted()), expected, "{}", table.name());
                    assert!(table.foreign_keys().is_empty(), "{}", table.name());
                    assert!(table.referenced_by().is_empty(), "{}", table.name());
                }
            },
        );
    }

    #[test]
    fn fr_priv_007_a_complete_object_in_a_marked_document_carries_no_marking() {
        // FR-PRIV-007: the marking is per object, per FR-PRIV-006, so the
        // three tables the fixture's foreign keys reach from neither end are
        // unmarked in the same document that marks fourteen others. The whole
        // of a privileged read is unmarked, which is the same requirement from
        // the other side.
        on_every_series(
            "fr_priv_007_a_complete_object_in_a_marked_document_carries_no_marking",
            |server| {
                let reduced = read_reduced(server, Scope::Everything);
                let reduced = reduced.model().expect("the rows fold");
                let unmarked: Vec<&str> = reduced
                    .tables
                    .iter()
                    .filter(|table| table.restricted().is_none())
                    .map(crate::model::table::Table::name)
                    .collect();

                assert_eq!(unmarked, ["audit_event", "legacy_edi_field", "tariff"]);

                let whole = read_from(server, Scope::Everything);
                let whole = whole.model().expect("the rows fold");

                for table in &whole.tables {
                    assert_eq!(marked(table.restricted()), None, "{}", table.name());
                }
                for view in &whole.views {
                    assert_eq!(marked(view.restricted.as_ref()), None, "{}", view.name);
                }
                for routine in &whole.routines {
                    assert_eq!(
                        marked(routine.restricted.as_ref()),
                        None,
                        "{}",
                        routine.name
                    );
                }
            },
        );
    }

    #[test]
    fn fr_priv_015_no_object_kind_but_a_view_and_a_foreign_key_is_cross_checked() {
        // FR-PRIV-015 with FR-PRIV-020. The reduced reader also receives zero
        // rows from the trigger catalogue, and nothing anywhere says so: the
        // six triggers of the fixture are simply absent, the tables that
        // carried them are marked only for what the foreign-key check found,
        // and no marking in the document names a property outside the four.
        on_every_series(
            "fr_priv_015_no_object_kind_but_a_view_and_a_foreign_key_is_cross_checked",
            |server| {
                let catalogue = read_reduced(server, Scope::Everything);
                let model = catalogue.model().expect("the rows fold");
                let triggers: usize = model.tables.iter().map(|t| t.triggers().len()).sum();

                assert!(catalogue.rows(Read::Triggers).is_empty());
                assert_eq!(triggers, 0);

                let named: Vec<&str> = model
                    .tables
                    .iter()
                    .filter_map(|table| marked(table.restricted()))
                    .chain(
                        model
                            .views
                            .iter()
                            .filter_map(|v| marked(v.restricted.as_ref())),
                    )
                    .chain(
                        model
                            .routines
                            .iter()
                            .filter_map(|r| marked(r.restricted.as_ref())),
                    )
                    .flatten()
                    .collect();

                assert!(!named.is_empty());
                for property in &named {
                    assert!(
                        ["body", "definition", "foreign_keys", "referenced_by"].contains(property),
                        "{property}"
                    );
                }

                // The table `audit_event` carries no key at either end and
                // three of the fixture's six triggers; it is complete, and its
                // hidden triggers are the limit FR-PRIV-020 states.
                let audit = model
                    .tables
                    .iter()
                    .find(|table| table.name() == "audit_event")
                    .expect("the fixture carries it");

                assert_eq!(marked(audit.restricted()), None);
            },
        );
    }

    #[test]
    fn fr_priv_012_a_read_that_names_one_object_performs_the_same_checks() {
        // FR-PRIV-012: every read that presents the property performs the
        // check, so the narrowed forms of FR-SCH-008 and the dump agree with
        // the schema-wide read rather than each carrying a check of its own.
        on_every_series(
            "fr_priv_012_a_read_that_names_one_object_performs_the_same_checks",
            |server| {
                let view = read_reduced(server, Scope::View("v_consignment_manifest"));
                let view = view.model().expect("the rows fold");

                assert_eq!(
                    marked(view.views[0].restricted.as_ref()),
                    Some(vec!["definition"])
                );

                let routine = read_reduced(
                    server,
                    Scope::Routine {
                        name: "sp_book_consignment",
                        kind: None,
                    },
                );
                let routine = routine.model().expect("the rows fold");

                assert_eq!(
                    marked(routine.routines[0].restricted.as_ref()),
                    Some(vec!["body"])
                );

                let table = read_reduced(server, Scope::Table("consignment"));
                let table = table.model().expect("the rows fold");

                assert_eq!(
                    marked(table.tables[0].restricted()),
                    Some(vec!["foreign_keys", "referenced_by"])
                );
            },
        );
    }

    #[test]
    fn fr_priv_003_a_named_object_that_came_back_short_yields_the_verdict_that_owes_77() {
        // FR-PRIV-003 and FR-PRIV-004: the caller that named the object
        // receives a verdict rather than half of it, and FR-PRIV-013 puts the
        // property in the message. The code is read from the verdict; nothing
        // here emits it, because no command that names an object exists yet.
        on_every_series(
            "fr_priv_003_a_named_object_that_came_back_short_yields_the_verdict_that_owes_77",
            |server| {
                let view = read_reduced(server, Scope::View("v_consignment_manifest"));
                let view = view.model().expect("the rows fold");
                let refused = completeness::of_view(&view.views[0])
                    .expect_err("the definition did not come back");

                assert_eq!(refused.exit_code(), 77);

                let rendered = crate::diagnostics::rendered(&refused);

                assert!(
                    rendered.contains("the definition of view 'v_consignment_manifest'"),
                    "{rendered}"
                );
                for credential in [REDUCED.1, ROOT.1, "password"] {
                    assert!(!rendered.contains(credential), "{rendered}");
                }

                let routine = read_reduced(
                    server,
                    Scope::Routine {
                        name: "sp_book_consignment",
                        kind: None,
                    },
                );
                let routine = routine.model().expect("the rows fold");
                let refused = completeness::of_routine(&routine.routines[0])
                    .expect_err("the body did not come back");

                assert_eq!(refused.exit_code(), 77);

                // The table verdict is taken over the document's own table,
                // which is the shape every caller of it holds; the read is a
                // whole one because `FR-CTX-006` embeds the table at each end
                // of every foreign key and a narrowed plan returns the keys
                // without those tables.
                let read = read_reduced(server, Scope::Everything);
                let model = read.model().expect("the rows fold");
                let document = crate::model::document::context(&model)
                    .expect("a whole read carries every table its keys name");
                let named = |wanted: &str| {
                    document
                        .tables
                        .iter()
                        .find(|table| table.name == wanted)
                        .unwrap_or_else(|| panic!("the fixture carries {wanted}"))
                };

                let refused = completeness::of_table(named("consignment"))
                    .expect_err("the referential rules did not come back");

                assert_eq!(refused.exit_code(), 77);

                // FR-PRIV-007 from the caller's side: a table the shortfall
                // did not reach answers the same read with no verdict at all.
                assert!(completeness::of_table(named("audit_event")).is_ok());
            },
        );
    }

    /// Asserts that `names` is ordered by the comparison `NFR-DET-002` fixes.
    fn assert_ordered(names: &[&str]) {
        for pair in names.windows(2) {
            assert!(
                pair[0].as_bytes() <= pair[1].as_bytes(),
                "{:?} precedes {:?}",
                pair[0],
                pair[1]
            );
        }
    }

    /// A full read of a schema no fixture server carries, kept whole.
    ///
    /// The name is one no server holds, so the schema catalogue returns no row
    /// and every other statement of the plan returns none either. The
    /// [`Catalogue`] is returned rather than folded, because the two callers
    /// want different halves of it: one the statements it issued, and one the
    /// condition the fold raises over a schema row that is not there.
    fn read_absent_schema(server: &fixture::Server) -> Catalogue {
        let scratch = Scratch::new();
        let resolved = settings(&scratch, server.address(), ROOT);
        let target = Target::of(&resolved).expect("the entry names a host");
        let clock = settings::clock(None);

        let mut session = crate::mariadb::open(&target, &clock).expect("the server answered");
        let catalogue = read(
            &mut session,
            &target,
            &clock,
            ABSENT_SCHEMA,
            &Scope::Everything,
        )
        .expect("a schema that holds nothing is read like any other");

        session.close();

        catalogue
    }

    /// Opens a session and reads the whole catalogue as the privileged
    /// account, returning whatever the attempt produced.
    ///
    /// Unlike [`read_as`] it refuses nothing: the two bodies below are about
    /// the verdict the connection start reaches, so the condition is the
    /// answer rather than a failure.
    fn attempt_read(server: &fixture::Server) -> Result<Catalogue, Error> {
        let scratch = Scratch::new();
        let resolved = settings(&scratch, server.address(), ROOT);
        let target = Target::of(&resolved).expect("the entry names a host");
        let clock = settings::clock(None);

        let mut session = crate::mariadb::open(&target, &clock)?;
        let catalogue = read(&mut session, &target, &clock, SCHEMA, &Scope::Everything);

        session.close();

        catalogue
    }

    #[test]
    fn fr_srv_035_a_series_above_the_window_is_read_and_marked_newer_than_supported() {
        // FR-SRV-035: present the reader with a series above its own window,
        // and assert that the read completes without error and that `standing`
        // is `newer_than_supported`.
        //
        // No such server exists to point the test at — by construction the
        // window contains the newest MariaDB there is — so the requirement
        // obliges the test to narrow the **reader** rather than widen the
        // server, through the seam of FR-ERR-031. The server below is a real
        // one of FR-SRV-015, answering a real connection and a real catalogue
        // read; what is narrowed is the table the reader compares against.
        //
        // The body is a unit test rather than an integration test because that
        // seam is `#[cfg(test)]`: an integration test links the library
        // compiled without that configuration and cannot see it. FR-SRV-035's
        // own eighth-edition amendment is what admits that — the assertion on
        // the exit code is the half that yielded, and the step from a read that
        // completes to exit `0` is BR-CLI-004, observed by every successful
        // command of the integration suite.
        on_every_series(
            "fr_srv_035_a_series_above_the_window_is_read_and_marked_newer_than_supported",
            |server| {
                let name = server.name();

                // Every series of FR-SRV-015 is 10.11 or newer, so a window
                // holding 10.6 alone puts all four above it.
                let _narrowed = crate::mariadb::window::narrow_to(
                    crate::mariadb::window::Series::parse("10.6").expect("two components"),
                );

                let catalogue = attempt_read(server).unwrap_or_else(|failure| {
                    panic!("{name}: the read did not complete: {failure}")
                });
                let model = catalogue
                    .model()
                    .unwrap_or_else(|failure| panic!("{name}: the rows did not fold: {failure}"));

                // FR-SRV-031: the catalogue is read in full, not refused.
                assert_eq!(catalogue.statements().len(), 11, "{name}");
                assert_eq!(model.tables.len(), 17, "{name}");
                assert_eq!(model.views.len(), 5, "{name}");
                assert_eq!(model.routines.len(), 7, "{name}");

                // FR-SRV-032 with FR-CTX-034: the marking is in the document,
                // and it is the field that says the read is unverified.
                assert_eq!(
                    model.server.standing(),
                    Standing::NewerThanSupported,
                    "{name}"
                );
                assert_eq!(model.server.standing().name(), "newer_than_supported");

                // The control: with the window restored, the same server is
                // `supported`. Without it, a reader that answered
                // `newer_than_supported` to everything would pass the
                // assertion above.
                drop(_narrowed);

                let restored =
                    attempt_read(server).expect("the same server is read with the window restored");
                let restored = restored.model().expect("the rows fold");

                assert_eq!(restored.server.standing(), Standing::Supported, "{name}");
            },
        );
    }

    #[test]
    fn fr_srv_029_a_series_below_the_window_is_refused_with_seventy_eight_and_reads_no_catalogue() {
        // FR-SRV-029's second half asks for the refusal of FR-SRV-020 against
        // at least one series outside the window. **The fixture holds no such
        // server**: `series.env` carries 10.11, 11.4, 11.8 and 12.3, and all
        // four are inside it. The condition is reached here the only way this
        // project can reach it against a real server — by narrowing the
        // reader's window through the seam FR-SRV-035 mandates, so that a real
        // server of a real series falls below it.
        //
        // What that establishes is what FR-SRV-020 promises: the series is
        // refused, the code is 78, and the catalogue is not read. What it does
        // not establish is the integration test FR-SRV-029 words, which needs a
        // server this fixture does not have.
        on_every_series(
            "fr_srv_029_a_series_below_the_window_is_refused_with_seventy_eight_and_reads_no_catalogue",
            |server| {
                let name = server.name();

                // Every series of FR-SRV-015 is older than 13.0, so a window
                // holding it alone puts all four below.
                let _narrowed = crate::mariadb::window::narrow_to(
                    crate::mariadb::window::Series::parse("13.0").expect("two components"),
                );

                let refused = attempt_read(server)
                    .err()
                    .unwrap_or_else(|| panic!("{name}: a series below the window was not refused"));

                assert_eq!(
                    refused.exit_code(),
                    78,
                    "{name}: {refused:?} — FR-SRV-020 with FR-ERR-006 fixes 78 (EX_CONFIG)"
                );

                // FR-SRV-030: the message names the series found and the entry
                // that reached it, and the `cause` lists the series that are
                // supported — which BR-SRV-005 puts on the value rather than in
                // the renderer.
                let Error::SeriesNotSupported {
                    entry, supported, ..
                } = &refused
                else {
                    panic!("{name}: expected an unsupported series, got {refused:?}")
                };

                assert_eq!(entry, ENTRY);
                assert_eq!(*supported, crate::mariadb::window::SUPPORTED);
            },
        );
    }

    #[test]
    fn fr_priv_021_a_schema_catalogue_that_returns_no_row_is_seventy_seven_and_never_seventy() {
        // FR-PRIV-021 rejects the reading this fold shipped with, in as many
        // words: "The condition was reported as a violated internal invariant,
        // exit 70. It is not one." FR-ERR-030 closes 70 to a panic and to an
        // invariant the system detects in itself, and nothing about tpl is
        // defective when a server declines to show a schema — the caller can
        // fix it with a grant or by correcting `database.<name>.database`.
        //
        // The condition is not reachable through the distributed binary:
        // FR-CONF-041 puts the database on the connection, so the server
        // refuses the handshake with 1049 or 1044 and `mariadb::fault` routes
        // that pair to this same condition long before a statement is sent.
        // What is exercised here is this system's own handling of a schema read
        // that returned no row, in process, which is the whole of what
        // FR-PRIV-021's own *Consequence* says can be exercised.
        on_every_series(
            "fr_priv_021_a_schema_catalogue_that_returns_no_row_is_seventy_seven_and_never_seventy",
            |server| {
                let catalogue = read_absent_schema(server);

                assert!(
                    catalogue.rows(Read::Schema).is_empty(),
                    "{}: the schema catalogue returned a row for a database it does not hold",
                    server.name()
                );

                let refused = catalogue
                    .model()
                    .expect_err("a database whose own metadata is unreadable is not presented");

                assert_eq!(
                    refused.exit_code(),
                    77,
                    "{}: {refused:?} — FR-PRIV-021 fixes 77 (EX_NOPERM) here",
                    server.name()
                );
                assert_ne!(
                    refused.exit_code(),
                    70,
                    "{}: the 70 FR-PRIV-021 rejected has come back",
                    server.name()
                );

                // FR-PRIV-013 and the 77 row of FR-ERR-034 oblige the `cause`
                // to name the database and to state which property could not be
                // read. FR-CONF-041 is what guarantees there is a name to
                // give: it is the one the read was issued with.
                let Error::PropertyNotReadable {
                    kind,
                    object,
                    property,
                } = &refused
                else {
                    panic!(
                        "{}: expected a property that could not be read, got {refused:?}",
                        server.name()
                    )
                };

                assert_eq!(*kind, CatalogueObjectKind::Database);
                assert_eq!(object, ABSENT_SCHEMA);
                assert_eq!(*property, "metadata");

                let rendered = crate::diagnostics::rendered(&refused);

                assert!(rendered.contains(ABSENT_SCHEMA), "{rendered}");
                assert!(rendered.contains("metadata"), "{rendered}");
            },
        );
    }

    #[test]
    fn fr_cat_056_the_six_nullable_fields_are_populated_on_every_series_and_carry_no_empty_string()
    {
        // FR-CAT-056: six fields the model carries are declared to admit SQL
        // `NULL`, and each was observed populated on every row the read
        // presents. The requirement fixes what happens when one is not — the
        // model carries `null`, never the empty string — and this asserts the
        // population the requirement records, which is what makes the
        // substitution it forbids unreachable here.
        on_every_series(
            "fr_cat_056_the_six_nullable_fields_are_populated_on_every_series_and_carry_no_empty_string",
            |server| {
                let catalogue = read_from(server, Scope::Everything);
                let model = catalogue.model().expect("the rows fold");
                let name = server.name();

                let triggers: Vec<_> = model
                    .tables
                    .iter()
                    .flat_map(crate::model::table::Table::triggers)
                    .collect();

                assert_eq!(triggers.len(), 6, "{name}");

                for trigger in &triggers {
                    assert!(trigger.statement.is_some(), "{name}: {}", trigger.name);
                    assert!(trigger.definer.is_some(), "{name}: {}", trigger.name);
                }

                assert_eq!(model.routines.len(), 7, "{name}");

                for routine in &model.routines {
                    assert!(routine.body.is_some(), "{name}: {}", routine.name);
                }

                let keys: Vec<_> = model
                    .tables
                    .iter()
                    .flat_map(crate::model::table::Table::foreign_keys)
                    .collect();
                let pairs: usize = keys.iter().map(|key| key.columns.len()).sum();

                assert_eq!(keys.len(), 15, "{name}");
                assert_eq!(pairs, 17, "{name}");

                for key in &keys {
                    assert!(key.referenced_key.is_some(), "{name}: {}", key.name);
                    assert!(key.referenced_table.is_some(), "{name}: {}", key.name);

                    // The sixth field is settled by the read's population
                    // rather than by a value: FR-CAT-045 restricts the read to
                    // the rows that name a referenced table, which are exactly
                    // the rows this field is populated on, so the model carries
                    // a string there and not an absent scalar.
                    for pair in &key.columns {
                        assert!(
                            !pair.referenced_column.is_empty(),
                            "{name}: {} names an empty referenced column",
                            key.name
                        );
                    }
                }
            },
        );
    }

    #[test]
    fn fr_cat_057_every_foreign_key_the_fixture_holds_names_one_schema_at_both_ends() {
        // FR-CAT-057 excludes a cross-schema foreign key in both directions,
        // and states that the behaviour is excluded rather than recorded
        // **because it could not be observed**: the fixture declares one user
        // schema and carries no such key. This is that observation, taken
        // through the reader rather than by hand — every key the model carries
        // names a table the model also carries, which is the condition
        // FR-CTX-023 needs and the exclusion exists to preserve.
        on_every_series(
            "fr_cat_057_every_foreign_key_the_fixture_holds_names_one_schema_at_both_ends",
            |server| {
                let catalogue = read_from(server, Scope::Everything);
                let model = catalogue.model().expect("the rows fold");
                let name = server.name();

                let carried: Vec<&str> = model
                    .tables
                    .iter()
                    .map(crate::model::table::Table::name)
                    .collect();
                let mut keys = 0;

                for table in &model.tables {
                    for key in table.foreign_keys() {
                        keys += 1;

                        let referenced = key
                            .referenced_table
                            .as_deref()
                            .expect("every key of the fixture names a table");

                        assert!(
                            carried.contains(&referenced),
                            "{name}: {} names {referenced}, which this read does not carry —                              a key FR-CAT-057 should have excluded reached the model",
                            key.name
                        );
                    }

                    for incoming in table.referenced_by() {
                        assert!(
                            carried.contains(&incoming.table.as_ref()),
                            "{name}: {} is referenced by {}, which this read does not carry",
                            table.name(),
                            incoming.table
                        );
                    }
                }

                // Nothing was excluded: the 15 rules the fixture declares all
                // reached the model, so the exclusion did not fire on a key it
                // should have carried.
                assert_eq!(keys, 15, "{name}");
            },
        );
    }

    /// The statements a full read of a schema that holds nothing issues.
    ///
    /// The name is one no fixture server carries, so every statement of the
    /// plan returns no row — which is the *one table against many* comparison
    /// `NFR-PERF-001` fixes, taken to its floor.
    fn read_empty_schema(server: &fixture::Server) -> Vec<&'static str> {
        let catalogue = read_absent_schema(server);

        assert!(catalogue.rows(Read::Tables).is_empty());

        catalogue.statements().to_vec()
    }
}
