//! One model the tests of this module are written against.
//!
//! It is the freight domain the project's own MariaDB fixture models, reduced
//! to what the document rules need to be observable: two tables that reference
//! each other's counterpart, a three-column primary key, a composite foreign
//! key whose two column lists are ordered differently, an `ENUM` whose member
//! order is its ordinal, a view and a routine that each carry a `restricted`
//! marking, and collections in the order `NFR-DET-002` fixes for them.
//!
//! Every collection is built **already in that order**, so a round trip can be
//! compared against the model it started from. The tests that are about the
//! ordering build their own disordered values.

use std::borrow::Cow;

use crate::model::check_constraint::{CheckConstraint, ConstraintLevel};
use crate::model::column::Column;
use crate::model::column_default::ColumnDefault;
use crate::model::column_type::{CatalogueType, ColumnType};
use crate::model::database::Database;
use crate::model::foreign_key::{
    ForeignKey, ForeignKeyColumn, IncomingForeignKey, ReferentialAction,
};
use crate::model::index::{Index, IndexColumn, PRIMARY_KEY_NAME, SortDirection};
use crate::model::restricted::Restricted;
use crate::model::routine::{Routine, RoutineKind, RoutineParameter};
use crate::model::server::{Server, Standing};
use crate::model::table::{Table, TableParts, TableType};
use crate::model::trigger::{Trigger, TriggerEvent, TriggerTiming};
use crate::model::view::View;

/// The `ENUM` of the fixture's `consignment.status`, whose member order is the
/// ordinal each member is stored as.
pub(crate) const STATUS: &str = "enum('Draft','Booked','Loaded','Delivered')";

/// The version the `11.4` fixture server returned to the observation pass.
pub(crate) const PROBED: &str = "11.4.13-MariaDB-ubu2404";

/// The server of `FR-CTX-031`.
pub(crate) fn server() -> Server<'static> {
    Server::probed(Cow::Borrowed(PROBED), Standing::Supported)
        .expect("the observed form yields a series")
}

/// A `bigint(20) unsigned`.
pub(crate) fn identifier() -> ColumnType<'static> {
    ColumnType::decompose(&CatalogueType {
        column_type: "bigint(20) unsigned",
        data_type: "bigint",
        numeric_precision: Some(20),
        numeric_scale: Some(0),
        ..CatalogueType::default()
    })
}

/// A `varchar` of `length` characters.
pub(crate) fn varchar(length: u64) -> ColumnType<'static> {
    ColumnType::decompose(&CatalogueType {
        column_type: "varchar(64)",
        data_type: "varchar",
        character_maximum_length: Some(length),
        charset: Some("utf8mb4"),
        collation: Some("utf8mb4_unicode_520_ci"),
        ..CatalogueType::default()
    })
}

/// The `ENUM` of [`STATUS`].
pub(crate) fn status() -> ColumnType<'static> {
    ColumnType::decompose(&CatalogueType {
        column_type: STATUS,
        data_type: "enum",
        character_maximum_length: Some(9),
        charset: Some("utf8mb4"),
        collation: Some("utf8mb4_unicode_520_ci"),
        ..CatalogueType::default()
    })
}

/// A `datetime` with no fractional part.
pub(crate) fn timestamp() -> ColumnType<'static> {
    ColumnType::decompose(&CatalogueType {
        column_type: "datetime",
        data_type: "datetime",
        datetime_precision: Some(0),
        ..CatalogueType::default()
    })
}

/// One column of `table`, at `position`, of `column_type`.
pub(crate) fn column(
    table: &'static str,
    name: &'static str,
    position: u64,
    column_type: ColumnType<'static>,
) -> Column<'static> {
    Column {
        name: Cow::Borrowed(name),
        table_name: Cow::Borrowed(table),
        position,
        column_type,
        nullable: false,
        default: None,
        comment: Cow::Borrowed(""),
        auto_increment: false,
        invisible: false,
        generated: None,
        on_update: None,
    }
}

/// One index over `columns`, in the order given.
pub(crate) fn index(name: &'static str, columns: &[&'static str]) -> Index<'static> {
    Index {
        name: Cow::Borrowed(name),
        unique: true,
        columns: columns
            .iter()
            .map(|column| IndexColumn {
                name: Cow::Borrowed(column),
                direction: Some(SortDirection::Ascending),
                prefix_length: None,
            })
            .collect(),
        index_type: Cow::Borrowed("BTREE"),
        comment: Cow::Borrowed(""),
        ignored: false,
    }
}

/// One foreign key, over the column pairs given in the order given.
pub(crate) fn key(
    name: &'static str,
    referenced_table: &'static str,
    pairs: &[(&'static str, &'static str)],
) -> ForeignKey<'static> {
    ForeignKey {
        name: Cow::Borrowed(name),
        columns: pairs
            .iter()
            .map(|(column, referenced)| ForeignKeyColumn {
                column: Cow::Borrowed(column),
                referenced_column: Cow::Borrowed(referenced),
            })
            .collect(),
        referenced_table: Cow::Borrowed(referenced_table),
        referenced_key: Cow::Borrowed(PRIMARY_KEY_NAME),
        match_option: Cow::Borrowed("NONE"),
        on_update: ReferentialAction::Restrict,
        on_delete: ReferentialAction::Cascade,
    }
}

/// The same key seen from the table it references.
pub(crate) fn incoming(
    table: &'static str,
    key: ForeignKey<'static>,
) -> IncomingForeignKey<'static> {
    IncomingForeignKey {
        table: Cow::Borrowed(table),
        key,
    }
}

/// One trigger of the fixture.
pub(crate) fn trigger(name: &'static str) -> Trigger<'static> {
    Trigger {
        name: Cow::Borrowed(name),
        event: TriggerEvent::Insert,
        timing: TriggerTiming::Before,
        action_order: 1,
        statement: Cow::Borrowed("BEGIN SET NEW.reference = upper(NEW.reference); END"),
        orientation: Cow::Borrowed("ROW"),
        old_row_alias: Cow::Borrowed("OLD"),
        new_row_alias: Cow::Borrowed("NEW"),
        sql_mode: Cow::Borrowed("STRICT_TRANS_TABLES"),
        definer: Cow::Borrowed("freight_owner@localhost"),
        character_set_client: Cow::Borrowed("utf8mb4"),
        collation_connection: Cow::Borrowed("utf8mb4_uca1400_ai_ci"),
        database_collation: Cow::Borrowed("utf8mb4_unicode_520_ci"),
    }
}

/// One `CHECK` constraint of the fixture.
pub(crate) fn check(name: &'static str) -> CheckConstraint<'static> {
    CheckConstraint {
        name: Cow::Borrowed(name),
        level: ConstraintLevel::Table,
        clause: Cow::Borrowed("char_length(`reference`) > 0"),
    }
}

/// One view, carrying the marking `FR-PRIV-016` gives as its example.
pub(crate) fn view(name: &'static str) -> View<'static> {
    View {
        name: Cow::Borrowed(name),
        definition: Cow::Borrowed("select `freight`.`consignment`.`reference` AS `reference`"),
        check_option: Cow::Borrowed("NONE"),
        is_updatable: false,
        definer: Cow::Borrowed("freight_owner@localhost"),
        security_type: Cow::Borrowed("DEFINER"),
        character_set_client: Cow::Borrowed("utf8mb4"),
        collation_connection: Cow::Borrowed("utf8mb4_uca1400_ai_ci"),
        algorithm: Cow::Borrowed("UNDEFINED"),
        restricted: Restricted::new(vec![Cow::Borrowed("definition")]),
    }
}

/// One routine, with two parameters in declaration order.
pub(crate) fn routine(name: &'static str) -> Routine<'static> {
    Routine {
        name: Cow::Borrowed(name),
        kind: RoutineKind::Procedure,
        return_type: None,
        parameters: vec![
            RoutineParameter {
                name: Cow::Borrowed("p_reference"),
                mode: Cow::Borrowed("IN"),
                parameter_type: varchar(64),
            },
            RoutineParameter {
                name: Cow::Borrowed("p_consignment_id"),
                mode: Cow::Borrowed("OUT"),
                parameter_type: identifier(),
            },
        ],
        body: Cow::Borrowed("BEGIN INSERT INTO consignment (reference) VALUES (p_reference); END"),
        body_kind: Cow::Borrowed("SQL"),
        parameter_style: Cow::Borrowed("SQL"),
        is_deterministic: false,
        sql_data_access: Cow::Borrowed("MODIFIES SQL DATA"),
        security_type: Cow::Borrowed("DEFINER"),
        sql_mode: Cow::Borrowed("STRICT_TRANS_TABLES"),
        comment: Cow::Borrowed(""),
        definer: Cow::Borrowed("freight_owner@localhost"),
        character_set_client: Cow::Borrowed("utf8mb4"),
        collation_connection: Cow::Borrowed("utf8mb4_uca1400_ai_ci"),
        database_collation: Cow::Borrowed("utf8mb4_unicode_520_ci"),
        restricted: Restricted::new(vec![Cow::Borrowed("body")]),
    }
}

/// `consignment`: the referenced end of the fixture's one relation.
pub(crate) fn consignment() -> Table<'static> {
    Table::assemble(TableParts {
        engine: Some(Cow::Borrowed("InnoDB")),
        collation: Some(Cow::Borrowed("utf8mb4_unicode_520_ci")),
        comment: Cow::Borrowed("one consignment per row"),
        columns: vec![
            column("consignment", "consignment_id", 1, identifier()),
            // The three forms of `FR-CTX-012`, one per column, so that the
            // round trip carries all three.
            Column {
                default: ColumnDefault::classify(Some("'EUR'")),
                ..column("consignment", "reference", 2, varchar(64))
            },
            Column {
                nullable: true,
                default: ColumnDefault::classify(Some("NULL")),
                ..column("consignment", "status", 3, status())
            },
            Column {
                default: ColumnDefault::classify(Some("current_timestamp()")),
                ..column("consignment", "booked_at", 4, timestamp())
            },
        ],
        indexes: vec![
            index(PRIMARY_KEY_NAME, &["consignment_id"]),
            index("uq_consignment_reference", &["reference"]),
        ],
        referenced_by: vec![incoming(
            "consignment_leg",
            key(
                "fk_leg_consignment",
                "consignment",
                &[("consignment_id", "consignment_id")],
            ),
        )],
        triggers: vec![trigger("trg_consignment_stamp")],
        check_constraints: vec![check("ck_consignment_reference")],
        ..TableParts::new(Cow::Borrowed("consignment"), TableType::Base)
    })
    .expect("every key of the fixture names a column its table carries")
}

/// `consignment_leg`: the referencing end.
pub(crate) fn consignment_leg() -> Table<'static> {
    Table::assemble(TableParts {
        engine: Some(Cow::Borrowed("InnoDB")),
        collation: Some(Cow::Borrowed("utf8mb4_unicode_520_ci")),
        comment: Cow::Borrowed(""),
        columns: vec![
            column("consignment_leg", "leg_id", 1, identifier()),
            column("consignment_leg", "consignment_id", 2, identifier()),
        ],
        indexes: vec![index(PRIMARY_KEY_NAME, &["leg_id"])],
        foreign_keys: vec![key(
            "fk_leg_consignment",
            "consignment",
            &[("consignment_id", "consignment_id")],
        )],
        ..TableParts::new(Cow::Borrowed("consignment_leg"), TableType::Base)
    })
    .expect("every key of the fixture names a column its table carries")
}

/// `carrier`: a table that references nothing and is referenced by nothing.
pub(crate) fn carrier() -> Table<'static> {
    Table::assemble(TableParts {
        engine: Some(Cow::Borrowed("InnoDB")),
        columns: vec![
            column("carrier", "carrier_id", 1, identifier()),
            column("carrier", "scac", 2, varchar(4)),
        ],
        indexes: vec![index(PRIMARY_KEY_NAME, &["carrier_id"])],
        ..TableParts::new(Cow::Borrowed("carrier"), TableType::Base)
    })
    .expect("every key of the fixture names a column its table carries")
}

/// The whole model, with every collection already in the order `NFR-DET-002`
/// fixes for it.
pub(crate) fn database() -> Database<'static> {
    Database {
        name: Cow::Borrowed("freight"),
        charset: Cow::Borrowed("utf8mb4"),
        collation: Cow::Borrowed("utf8mb4_unicode_520_ci"),
        server: server(),
        tables: vec![consignment(), consignment_leg()],
        views: vec![view("v_consignment_manifest")],
        routines: vec![routine("sp_book_consignment")],
    }
}

/// A model carrying `tables` and nothing else.
pub(crate) fn database_of(tables: Vec<Table<'static>>) -> Database<'static> {
    Database {
        tables,
        views: Vec::new(),
        routines: Vec::new(),
        ..database()
    }
}
