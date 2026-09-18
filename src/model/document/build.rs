//! The outward direction: a model becomes the document that carries it
//! (`FR-CTX-003` … `FR-CTX-010`, `NFR-DET-002`, `ADR-009`).
//!
//! Two things happen here and nowhere else.
//!
//! **The embedding is materialised.** `ADR-009` settles that the object graph
//! *is* the document: each embedding site holds the embedded table, already cut
//! at the first hop, so serialisation is a walk over the structure rather than
//! a reconstruction of it. The alternative it rejected — reproducing the cut as
//! the bytes are written — would put the two directions of `FR-CTX-009` into an
//! emitter, where a defect is a wrong document at exit `0`, with no exit code
//! and nothing for a caller to branch on.
//!
//! **Every collection is ordered where it is built.** [`super::order`] holds
//! the one comparator and the six exceptions; this module calls it once per
//! collection, and an excepted collection says so at its call site.
//!
//! *The members of an embedded table are borrowed from the model, not copied.*
//! What `ADR-009` fixes is the **structure**: the cut is a property of the value
//! rather than of the emitter, which is what a borrowed member leaves intact. A
//! deep copy of every embedded table was rejected because it pays for the
//! embedding a second time — `BR-CTX-001` already carries the cost once and
//! `FR-CTX-010` again — and buys nothing this file needs.

use std::borrow::Cow;

use super::order;
use super::shape::{
    ContextData, DatabaseDocument, EmbeddedTable, ForeignKeyShape, IncomingKey, NamedKey,
    OutgoingKey, TableDocument, TableShape,
};
use crate::error::{Error, ensure_invariant};
use crate::model::database::Database;
use crate::model::foreign_key::ForeignKey;
use crate::model::table::Table;

/// The invariant a dangling reference violates.
///
/// `FR-CTX-023` promises that every object referenced from another object in a
/// document produced by a server read is present in it, which is exactly the
/// condition the embedding needs in order to be materialisable.
const REFERENCE_IS_CARRIED: &str =
    "every table a foreign key names is carried by the model the document is built from";

/// Builds the `data` of a dump from a model.
///
/// # Errors
///
/// Returns [`Error::InternalInvariant`] where a foreign key, in either
/// direction, names a table the model does not carry. `FR-CTX-023` is what
/// makes that unreachable on the path that produces a document, and `ADR-009`'s
/// materialisation is what makes it detectable at all: an emitter that
/// reproduced the embedding at write time would have written the document and
/// left the caller to discover the dangling reference.
pub(super) fn context<'a>(database: &'a Database<'a>) -> Result<ContextData<'a>, Error> {
    Ok(ContextData {
        database: self::database(database)?,
    })
}

/// Builds the `database` object of `FR-CTX-036`.
fn database<'a>(database: &'a Database<'a>) -> Result<DatabaseDocument<'a>, Error> {
    // The pointers are ordered once and serve twice: they are the order
    // `tables` is emitted in, and the population every reference is resolved
    // against, by binary search rather than by a scan per key.
    let carried = order::pointers_by_name(&database.tables);

    let mut tables = Vec::with_capacity(carried.len());
    for table in &carried {
        tables.push(document(table, &carried)?);
    }

    Ok(DatabaseDocument {
        name: database.name.clone(),
        charset: database.charset.clone(),
        collation: database.collation.clone(),
        server: database.server.clone(),
        tables,
        views: order::by_name(&database.views),
        routines: order::by_name(&database.routines),
    })
}

/// Builds a table at the first hop, embedding both directions of every
/// reference (`FR-CTX-006`, `FR-CTX-010`).
fn document<'a>(
    table: &'a Table<'a>,
    carried: &[&'a Table<'a>],
) -> Result<TableDocument<'a>, Error> {
    let mut foreign_keys: Vec<OutgoingKey<'a>> = Vec::with_capacity(table.foreign_keys().len());
    for outgoing in table.foreign_keys() {
        let referenced = find(carried, &outgoing.referenced_table)?;
        foreign_keys.push(key(outgoing, embedded(referenced)));
    }
    order::sort_by_name(&mut foreign_keys);

    let mut referenced_by: Vec<IncomingKey<'a>> = Vec::with_capacity(table.referenced_by().len());
    for incoming in table.referenced_by() {
        referenced_by.push(IncomingKey {
            table: embedded(find(carried, &incoming.table)?),
            key: named(&incoming.key),
        });
    }
    // An entry of this collection is a pair and has no single name, so the
    // default rule of `NFR-DET-002` is read over the two names it does have:
    // the referencing table first, because that is the entry's subject, and the
    // constraint second, because a table may reference this one more than once.
    referenced_by.sort_by(|left, right| {
        order::compare(&left.table.name, &right.table.name)
            .then_with(|| order::compare(&left.key.name, &right.key.name))
    });

    Ok(TableDocument {
        foreign_keys,
        referenced_by,
        ..shape(table)
    })
}

/// Builds a table one hop in, cut to names in both directions (`FR-CTX-008`,
/// `FR-CTX-009`).
///
/// The cut is the whole difference from [`document`]: `FR-CTX-007` keeps the
/// columns, the indexes and the primary key in full, and the two reference
/// collections become the names of the tables at the far end.
fn embedded<'a>(table: &'a Table<'a>) -> EmbeddedTable<'a> {
    let mut foreign_keys: Vec<Cow<'a, str>> = table
        .foreign_keys()
        .iter()
        .map(|key| key.referenced_table.clone())
        .collect();
    order::sort_by_name(&mut foreign_keys);

    let mut referenced_by: Vec<Cow<'a, str>> = table
        .referenced_by()
        .iter()
        .map(|incoming| incoming.table.clone())
        .collect();
    order::sort_by_name(&mut referenced_by);

    EmbeddedTable {
        foreign_keys,
        referenced_by,
        ..shape(table)
    }
}

/// Everything a table carries that does not depend on how deep it sits.
///
/// The two reference collections are left empty and the caller fills them,
/// which is what makes the depth the only difference between the two tables the
/// document holds.
fn shape<'a, F, I>(table: &'a Table<'a>) -> TableShape<'a, F, I> {
    TableShape {
        name: Cow::Borrowed(table.name()),
        table_type: table.table_type(),
        engine: table.engine().map(Cow::Borrowed),
        collation: table.collation().map(Cow::Borrowed),
        comment: Cow::Borrowed(table.comment()),
        columns: order::as_given(table.columns()),
        indexes: order::by_name(table.indexes()),
        primary_key: table.primary_key().cloned(),
        foreign_keys: Vec::new(),
        referenced_by: Vec::new(),
        triggers: order::by_name(table.triggers()),
        check_constraints: order::by_name(table.check_constraints()),
        restricted: table.restricted().cloned(),
    }
}

/// One foreign key, with the one field the depth decides supplied by the
/// caller.
///
/// The parameter is what makes the two depths one function: at the first hop
/// the caller passes the embedded table, and beyond it the name.
fn key<'a, R>(outgoing: &'a ForeignKey<'a>, referenced_table: R) -> ForeignKeyShape<'a, R> {
    ForeignKeyShape {
        name: outgoing.name.clone(),
        columns: order::as_given(&outgoing.columns),
        referenced_table,
        referenced_key: outgoing.referenced_key.clone(),
        match_option: outgoing.match_option.clone(),
        on_update: outgoing.on_update,
        on_delete: outgoing.on_delete,
    }
}

/// A foreign key whose referenced table is carried as a name.
fn named<'a>(outgoing: &'a ForeignKey<'a>) -> NamedKey<'a> {
    key(outgoing, outgoing.referenced_table.clone())
}

/// The table `name` names, from the population the document is built over.
///
/// # Errors
///
/// Returns [`Error::InternalInvariant`] where the model carries no such table.
fn find<'a>(carried: &[&'a Table<'a>], name: &str) -> Result<&'a Table<'a>, Error> {
    match carried.binary_search_by(|table| order::compare(table.name(), name)) {
        Ok(at) => Ok(carried[at]),
        Err(_) => Err(dangling()),
    }
}

/// The `70` a dangling reference produces, raised through the one guard that
/// classifies a violated invariant.
#[track_caller]
fn dangling() -> Error {
    ensure_invariant(false, REFERENCE_IS_CARRIED)
        .expect_err("ensure_invariant(false, _) returns the condition it was given")
}

#[cfg(test)]
mod tests {
    use super::super::fixture;
    use super::{context, order};
    use crate::model::database::Database;
    use crate::model::index::PRIMARY_KEY_NAME;
    use crate::model::table::{Table, TableParts, TableType};
    use std::borrow::Cow;

    /// The `database` object `model` produces.
    ///
    /// The model is the caller's local, so that the document may borrow it:
    /// what the document holds of an embedded table is the model's own
    /// members, which is what `ADR-009` materialises the **structure** of.
    fn document<'a>(model: &'a Database<'a>) -> super::DatabaseDocument<'a> {
        context(model)
            .expect("every reference of a model built here is carried by it")
            .database
    }

    /// `vessel`, which references `voyage` and is referenced by it.
    fn vessel() -> Table<'static> {
        Table::assemble(TableParts {
            columns: vec![
                fixture::column("vessel", "vessel_imo", 1, fixture::identifier()),
                fixture::column("vessel", "current_voyage_id", 2, fixture::identifier()),
            ],
            indexes: vec![fixture::index(PRIMARY_KEY_NAME, &["vessel_imo"])],
            foreign_keys: vec![fixture::key(
                "fk_vessel_voyage",
                "voyage",
                &[("current_voyage_id", "voyage_id")],
            )],
            referenced_by: vec![fixture::incoming(
                "voyage",
                fixture::key(
                    "fk_voyage_vessel",
                    "vessel",
                    &[("vessel_imo", "vessel_imo")],
                ),
            )],
            ..TableParts::new(Cow::Borrowed("vessel"), TableType::Base)
        })
        .expect("every key names a column its table carries")
    }

    /// `voyage`, the other end of the same two-table cycle.
    fn voyage() -> Table<'static> {
        Table::assemble(TableParts {
            columns: vec![
                fixture::column("voyage", "voyage_id", 1, fixture::identifier()),
                fixture::column("voyage", "vessel_imo", 2, fixture::identifier()),
            ],
            indexes: vec![fixture::index(PRIMARY_KEY_NAME, &["voyage_id"])],
            foreign_keys: vec![fixture::key(
                "fk_voyage_vessel",
                "vessel",
                &[("vessel_imo", "vessel_imo")],
            )],
            referenced_by: vec![fixture::incoming(
                "vessel",
                fixture::key(
                    "fk_vessel_voyage",
                    "voyage",
                    &[("current_voyage_id", "voyage_id")],
                ),
            )],
            ..TableParts::new(Cow::Borrowed("voyage"), TableType::Base)
        })
        .expect("every key names a column its table carries")
    }

    /// `tariff`, which references itself.
    fn tariff() -> Table<'static> {
        let supersedes = fixture::key(
            "fk_tariff_supersedes",
            "tariff",
            &[("supersedes_id", "tariff_id")],
        );

        Table::assemble(TableParts {
            columns: vec![
                fixture::column("tariff", "tariff_id", 1, fixture::identifier()),
                fixture::column("tariff", "supersedes_id", 2, fixture::identifier()),
            ],
            indexes: vec![fixture::index(PRIMARY_KEY_NAME, &["tariff_id"])],
            foreign_keys: vec![supersedes.clone()],
            referenced_by: vec![fixture::incoming("tariff", supersedes)],
            ..TableParts::new(Cow::Borrowed("tariff"), TableType::SystemVersioned)
        })
        .expect("every key names a column its table carries")
    }

    #[test]
    fn fr_ctx_006_a_foreign_key_embeds_the_referenced_table_and_referenced_by_embeds_the_referencing_one()
     {
        // FR-CTX-006 and FR-CTX-010: the two directions are the same question
        // asked from the two ends, and a template that could reach the
        // referenced table's column type but only the referencing table's name
        // could not generate the has-many side of a relation.
        let model = fixture::database();
        let database = document(&model);
        let consignment = &database.tables[0];
        let leg = &database.tables[1];

        assert_eq!(consignment.name, "consignment");
        assert_eq!(consignment.referenced_by[0].table.name, "consignment_leg");
        assert_eq!(consignment.referenced_by[0].table.columns.len(), 2);

        assert_eq!(leg.foreign_keys[0].referenced_table.name, "consignment");
        assert_eq!(leg.foreign_keys[0].referenced_table.columns.len(), 4);
    }

    #[test]
    fn fr_ctx_007_an_embedded_table_carries_its_columns_its_indexes_and_its_primary_key_in_full() {
        // FR-CTX-007, against the summary embedding BR-CTX-001 rejected: a
        // name, a primary key and column names only would stop short of the one
        // field a foreign-key accessor needs, which is the referenced column's
        // type.
        let model = fixture::database();
        let database = document(&model);
        let embedded = &database.tables[1].foreign_keys[0].referenced_table;

        assert_eq!(embedded.columns[0].name, "consignment_id");
        assert_eq!(
            embedded.columns[0].column_type.raw(),
            "bigint(20) unsigned",
            "the referenced column's type is reachable, which is what the embedding is for"
        );
        assert_eq!(embedded.indexes.len(), 2);
        assert_eq!(
            embedded
                .primary_key
                .as_ref()
                .expect("consignment has a primary key")
                .name,
            PRIMARY_KEY_NAME
        );
        assert_eq!(embedded.triggers.len(), 1);
        assert_eq!(embedded.check_constraints.len(), 1);
    }

    #[test]
    fn fr_ctx_009_a_two_table_cycle_terminates_at_the_first_hop_with_names() {
        // FR-CTX-009, first shape: `A` references `B`, and `B` references `A`.
        // The embedded `A` inside `B` and the embedded `B` inside `A` are each
        // at the first hop and are each cut to names.
        let model = fixture::database_of(vec![vessel(), voyage()]);
        let database = document(&model);
        let vessel = &database.tables[0];
        let voyage = &database.tables[1];

        assert_eq!(vessel.name, "vessel");
        let embedded_voyage = &vessel.foreign_keys[0].referenced_table;
        assert_eq!(embedded_voyage.name, "voyage");
        assert_eq!(embedded_voyage.foreign_keys, ["vessel"]);
        assert_eq!(embedded_voyage.referenced_by, ["vessel"]);

        assert_eq!(voyage.name, "voyage");
        let embedded_vessel = &voyage.foreign_keys[0].referenced_table;
        assert_eq!(embedded_vessel.name, "vessel");
        assert_eq!(embedded_vessel.foreign_keys, ["voyage"]);
        assert_eq!(embedded_vessel.referenced_by, ["voyage"]);
    }

    #[test]
    fn fr_ctx_009_the_two_directions_of_one_relation_are_two_first_hops_and_not_a_path_of_length_two()
     {
        // FR-CTX-009, second shape: `A` references `B`, so `B` is
        // `referenced_by` `A`. What the fifth-edition amendment removed is the
        // reading under which an outgoing hop followed by an incoming one could
        // be taken as depth two.
        let model = fixture::database_of(vec![vessel(), voyage()]);
        let database = document(&model);

        let outgoing = &database.tables[0].foreign_keys[0].referenced_table;
        let incoming = &database.tables[0].referenced_by[0].table;

        assert_eq!(outgoing.name, "voyage");
        assert_eq!(incoming.name, "voyage");
        assert_eq!(outgoing.referenced_by, ["vessel"]);
        assert_eq!(incoming.foreign_keys, ["vessel"]);
        assert_eq!(
            database.tables[0].referenced_by[0].key.referenced_table, "vessel",
            "the key beside an embedded table carries its referenced table as a name"
        );
    }

    #[test]
    fn fr_ctx_009_a_self_reference_is_cut_in_both_collections_at_the_first_hop() {
        // FR-CTX-009, third shape: `A` references itself. The embedded `A` is
        // at the first hop and is cut, in both collections.
        let model = fixture::database_of(vec![tariff()]);
        let database = document(&model);
        let tariff = &database.tables[0];

        let outgoing = &tariff.foreign_keys[0].referenced_table;
        assert_eq!(outgoing.name, "tariff");
        assert_eq!(outgoing.foreign_keys, ["tariff"]);
        assert_eq!(outgoing.referenced_by, ["tariff"]);

        let incoming = &tariff.referenced_by[0].table;
        assert_eq!(incoming.name, "tariff");
        assert_eq!(incoming.foreign_keys, ["tariff"]);
        assert_eq!(incoming.referenced_by, ["tariff"]);
    }

    #[test]
    fn nfr_det_002_a_primary_key_keeps_the_column_order_the_catalogue_states() {
        // NFR-DET-002's first corrupting exception: `voyage_leg`'s key is
        // (vessel_imo, voyage_number, leg_sequence); sorted by name it becomes
        // (leg_sequence, vessel_imo, voyage_number), which is a different key.
        let declared = ["vessel_imo", "voyage_number", "leg_sequence"];
        let leg = Table::assemble(TableParts {
            columns: declared
                .iter()
                .enumerate()
                .map(|(at, name)| {
                    fixture::column(
                        "voyage_leg",
                        name,
                        u64::try_from(at).expect("three columns fit") + 1,
                        fixture::identifier(),
                    )
                })
                .collect(),
            indexes: vec![fixture::index(PRIMARY_KEY_NAME, &declared)],
            ..TableParts::new(Cow::Borrowed("voyage_leg"), TableType::Base)
        })
        .expect("every key names a column its table carries");

        let model = fixture::database_of(vec![leg]);
        let database = document(&model);
        let key = database.tables[0]
            .primary_key
            .as_ref()
            .expect("voyage_leg has a primary key");

        let carried: Vec<&str> = key
            .columns
            .iter()
            .map(|column| column.name.as_ref())
            .collect();
        assert_eq!(carried, declared);
        assert_eq!(
            database.tables[0].indexes[0].columns.len(),
            3,
            "and the same index inside the collection carries the same order"
        );
    }

    #[test]
    fn nfr_det_002_a_foreign_keys_column_pairing_survives_because_it_is_one_list_of_pairs() {
        // NFR-DET-002's second corrupting exception: the referencing and the
        // referenced list are paired positionally, so sorting either
        // independently pairs each column with the wrong counterpart. One list
        // of pairs makes that unrepresentable, and the declared order is kept.
        let pairs = [("zone_code", "region_code"), ("area_code", "zone_id")];
        let tariff = Table::assemble(TableParts {
            columns: vec![
                fixture::column("tariff", "zone_code", 1, fixture::varchar(8)),
                fixture::column("tariff", "area_code", 2, fixture::varchar(8)),
            ],
            foreign_keys: vec![fixture::key("fk_tariff_zone", "zone", &pairs)],
            ..TableParts::new(Cow::Borrowed("tariff"), TableType::Base)
        })
        .expect("every key names a column its table carries");
        let zone = Table::assemble(TableParts {
            columns: vec![
                fixture::column("zone", "region_code", 1, fixture::varchar(8)),
                fixture::column("zone", "zone_id", 2, fixture::varchar(8)),
            ],
            ..TableParts::new(Cow::Borrowed("zone"), TableType::Base)
        })
        .expect("a table with no keys satisfies FR-CAT-044 vacuously");

        let model = fixture::database_of(vec![tariff, zone]);
        let database = document(&model);
        let carried: Vec<(&str, &str)> = database.tables[0].foreign_keys[0]
            .columns
            .iter()
            .map(|pair| (pair.column.as_ref(), pair.referenced_column.as_ref()))
            .collect();

        assert_eq!(carried, pairs);
    }

    #[test]
    fn nfr_det_002_an_enum_member_list_keeps_the_ordinal_each_member_is_stored_as() {
        // NFR-DET-002's third corrupting exception: the order **is** the
        // meaning. Sorting enum('Draft','Booked','Loaded','Delivered') by name
        // renumbers every member, and a generator emitting a target-language
        // enumeration from it assigns every value the wrong discriminant, at
        // exit 0.
        let model = fixture::database();
        let database = document(&model);
        let status = &database.tables[0].columns[2];

        assert_eq!(status.name, "status");
        assert_eq!(
            status
                .column_type
                .values()
                .expect("an ENUM carries its member list"),
            ["Draft", "Booked", "Loaded", "Delivered"]
        );
    }

    #[test]
    fn nfr_det_002_every_collection_the_default_rule_governs_is_ordered_by_name_byte_wise() {
        // NFR-DET-002's default rule, applied where each collection is built
        // and not where it is emitted, so the answer is the same whatever order
        // the model arrived in.
        let unordered_members = Table::assemble(TableParts {
            columns: vec![fixture::column(
                "berth",
                "berth_id",
                1,
                fixture::identifier(),
            )],
            indexes: vec![
                fixture::index("uq_berth_code", &["berth_id"]),
                fixture::index(PRIMARY_KEY_NAME, &["berth_id"]),
            ],
            triggers: vec![
                fixture::trigger("trg_berth_stamp"),
                fixture::trigger("trg_berth_audit"),
            ],
            check_constraints: vec![
                fixture::check("ck_berth_depth"),
                fixture::check("ck_berth_code"),
            ],
            ..TableParts::new(Cow::Borrowed("berth"), TableType::Base)
        })
        .expect("every key names a column its table carries");

        let disordered = Database {
            tables: vec![
                fixture::consignment_leg(),
                unordered_members,
                fixture::consignment(),
            ],
            views: vec![fixture::view("v_tariff"), fixture::view("v_consignment")],
            routines: vec![
                fixture::routine("sp_rate_consignment"),
                fixture::routine("fn_normalise_reference"),
            ],
            ..fixture::database()
        };

        let database = document(&disordered);

        let names: Vec<&str> = database
            .tables
            .iter()
            .map(|table| table.name.as_ref())
            .collect();
        assert_eq!(names, ["berth", "consignment", "consignment_leg"]);

        let berth = &database.tables[0];
        let triggers: Vec<&str> = berth
            .triggers
            .iter()
            .map(|trigger| trigger.name.as_ref())
            .collect();
        assert_eq!(triggers, ["trg_berth_audit", "trg_berth_stamp"]);

        let checks: Vec<&str> = berth
            .check_constraints
            .iter()
            .map(|check| check.name.as_ref())
            .collect();
        assert_eq!(checks, ["ck_berth_code", "ck_berth_depth"]);

        let members: Vec<&str> = berth
            .indexes
            .iter()
            .map(|index| index.name.as_ref())
            .collect();
        assert_eq!(members, ["PRIMARY", "uq_berth_code"]);

        let views: Vec<&str> = database
            .views
            .iter()
            .map(|view| view.name.as_ref())
            .collect();
        assert_eq!(views, ["v_consignment", "v_tariff"]);

        let routines: Vec<&str> = database
            .routines
            .iter()
            .map(|routine| routine.name.as_ref())
            .collect();
        assert_eq!(routines, ["fn_normalise_reference", "sp_rate_consignment"]);

        let indexes: Vec<&str> = database.tables[1]
            .indexes
            .iter()
            .map(|index| index.name.as_ref())
            .collect();
        assert_eq!(indexes, ["PRIMARY", "uq_consignment_reference"]);
    }

    #[test]
    fn fr_ctx_023_a_reference_the_model_does_not_carry_is_the_seventy_of_the_internal_invariant() {
        // FR-CTX-023 promises that every object referenced from another object
        // in a document produced by a server read is present in it, which is
        // what makes this unreachable on that path. ADR-009's materialisation
        // is what makes it detectable rather than emitted.
        let dangling = fixture::database_of(vec![fixture::consignment_leg()]);
        let reported = context(&dangling).expect_err("`consignment` is not carried");

        assert_eq!(reported.exit_code(), 70);
        assert!(
            reported
                .to_string()
                .contains("every table a foreign key names")
        );
    }

    #[test]
    fn nfr_det_002_the_comparator_the_document_orders_by_is_the_one_the_module_states() {
        // One comparator, so the two-key order of `referenced_by` and every
        // single-key order are the same byte-wise comparison.
        assert!(order::compare("consignment", "consignment_leg").is_lt());
    }
}
