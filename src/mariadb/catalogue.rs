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
//! where every other one is, by the binary, and no command that names an object
//! exists yet to ask for it.

mod completeness;
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
#[cfg(test)]
#[path = "../../tests/support/fixture.rs"]
mod fixture;

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

    /// The statements that were issued, in order.
    ///
    /// It is what an assertion of `NFR-PERF-001` and `NFR-PERF-002` reads: the
    /// count of a read is observable from the value the read produced, so a
    /// test names what the process did rather than what the plan said it would
    /// do. The cost is one vector of at most eleven `&'static str`.
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
    /// shape the model cannot represent; [`fold`] records which four those are
    /// and why each is a `70` rather than a condition the caller can act on.
    pub(crate) fn model(&self) -> Result<Database<'_>, Error> {
        fold::database(self)
    }

    /// The statements this read issued, in the order they were issued.
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
    scope: Scope<'_>,
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
        match timeout(bound.remaining(), query.fetch_all(&mut *connection)).await {
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
    use crate::mariadb::connect::Target;
    use crate::model::check_constraint::ConstraintLevel;
    use crate::model::column::GeneratedStorage;
    use crate::model::column_default::DefaultKind;
    use crate::model::foreign_key::ReferentialAction;
    use crate::model::index::PRIMARY_KEY_NAME;
    use crate::model::routine::RoutineKind;
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
        let catalogue = read(&mut session, &target, &clock, SCHEMA, scope)
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
                    assert_eq!(key.referenced_key, PRIMARY_KEY_NAME);
                    assert!(!key.referenced_table.is_empty());
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
                        assert_eq!(entry.key.referenced_table, table.name());
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
                    assert!(!trigger.statement.is_empty());
                    assert!(!trigger.definer.is_empty());
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
                    assert!(!routine.body.is_empty(), "{}", routine.name);
                    assert_eq!(routine.body_kind, "SQL");
                    assert_eq!(routine.parameter_style, "SQL");
                    assert_eq!(routine.security_type, "DEFINER");
                    assert!(routine.restricted.is_none());

                    match routine.kind {
                        RoutineKind::Procedure => assert!(routine.return_type.is_none()),
                        RoutineKind::Function => {
                            let returned = routine
                                .return_type
                                .as_ref()
                                .expect("a function states its return type");

                            assert!(!returned.raw().is_empty(), "{}", routine.name);
                            assert!(returned.data_type().is_some(), "{}", routine.name);
                        }
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

                let table = read_reduced(server, Scope::Table("consignment"));
                let table = table.model().expect("the rows fold");
                let refused = completeness::of_table(&table.tables[0])
                    .expect_err("the referential rules did not come back");

                assert_eq!(refused.exit_code(), 77);

                // FR-PRIV-007 from the caller's side: a table the shortfall
                // did not reach answers the same read with no verdict at all.
                let whole = read_reduced(server, Scope::Table("audit_event"));
                let whole = whole.model().expect("the rows fold");

                assert!(completeness::of_table(&whole.tables[0]).is_ok());
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

    /// The statements a full read of a schema that holds nothing issues.
    ///
    /// The name is one no fixture server carries, so every statement of the
    /// plan returns no row — which is the *one table against many* comparison
    /// `NFR-PERF-001` fixes, taken to its floor.
    fn read_empty_schema(server: &fixture::Server) -> Vec<&'static str> {
        let scratch = Scratch::new();
        let resolved = settings(&scratch, server.address(), ROOT);
        let target = Target::of(&resolved).expect("the entry names a host");
        let clock = settings::clock(None);

        let mut session = crate::mariadb::open(&target, &clock).expect("the server answered");
        let catalogue = read(
            &mut session,
            &target,
            &clock,
            "a_schema_no_server_of_the_fixture_carries",
            Scope::Everything,
        )
        .expect("a schema that holds nothing is read like any other");

        session.close();

        assert!(catalogue.rows(Read::Tables).is_empty());

        catalogue.statements().to_vec()
    }
}
