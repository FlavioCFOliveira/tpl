//! The operands and the render context the tests of this module are written
//! against.
//!
//! Nothing here builds a shape of its own. A column, a table and a whole
//! render context are produced by **serialising the model**, through the same
//! document the render context is assembled from (`FR-CTX-001`,
//! `FR-RND-023`), so a test that passes here is a test against the shape a
//! template actually sees. The model itself is
//! [`crate::model::document::fixture`]'s freight domain, which is already the
//! one the rest of the crate is tested over.
//!
//! Two builders are the exception and are deliberately hand-made: a column
//! whose decomposed type is the unrecognised one of `FR-CTX-018`, and a column
//! missing an attribute, which no read of a supported series produces and
//! which a hand-assembled `--context` document can, per `FR-SEM-018`.

use std::borrow::Cow;
use std::collections::BTreeMap;

use minijinja::{Value, context};

use crate::model::column::Column;
use crate::model::column_type::{CatalogueType, ColumnType};
use crate::model::document;
use crate::model::table::Table;

/// The render context of `FR-RND-023`, carrying the `database` variable.
pub(super) fn context() -> Value {
    let database = document::fixture::database();
    let built = document::context(&database).expect("the fixture model is coherent");

    context! { database => Value::from_serialize(&built) }
}

/// The render context of [`context`], carrying `operand` beside the database.
///
/// It is how a test or a filter that reaches the context is exercised through
/// a real render rather than through a state built by hand.
pub(super) fn context_with(operand: Value) -> Value {
    let database = document::fixture::database();
    let built = document::context(&database).expect("the fixture model is coherent");

    context! {
        database => Value::from_serialize(&built),
        operand => operand,
    }
}

/// A render context carrying `operand` and no `database` (`FR-SEM-018`).
pub(super) fn context_with_only(operand: Value) -> Value {
    context! { operand => operand }
}

/// The fixture's `consignment`, as the document carries it.
pub(super) fn table() -> Value {
    let database = document::fixture::database();
    let built = document::context(&database).expect("the fixture model is coherent");
    let table = built
        .tables
        .iter()
        .find(|table| table.name == "consignment")
        .expect("the fixture carries the table");

    Value::from_serialize(table)
}

/// One column of the fixture's `consignment`, as the document carries it.
pub(super) fn column(name: &str) -> Value {
    let table = consignment();
    let column = table
        .columns()
        .iter()
        .find(|column| column.name == name)
        .expect("the fixture carries the column");

    Value::from_serialize(column)
}

/// A column of `consignment` whose `data_type` is `data_type`.
///
/// It is how the three family tables of `FR-ENV-046` are exercised: one column
/// per observed value, built through the same decomposition a read uses.
pub(super) fn column_of_type(data_type: &str) -> Value {
    Value::from_serialize(&Column {
        column_type: ColumnType::decompose(&CatalogueType {
            column_type: data_type,
            data_type,
            ..CatalogueType::default()
        }),
        ..plain()
    })
}

/// A column whose type the system does not recognise (`FR-CTX-018`).
pub(super) fn column_of_unrecognised_type() -> Value {
    column_of_type("quaternion")
}

/// A column carrying `auto_increment`.
pub(super) fn auto_incremental_column() -> Value {
    Value::from_serialize(&Column {
        auto_increment: true,
        ..plain()
    })
}

/// A column admitting `NULL`.
pub(super) fn nullable_column() -> Value {
    Value::from_serialize(&Column {
        nullable: true,
        ..plain()
    })
}

/// A column of a table the render context does not carry (`FR-ENV-017`).
pub(super) fn column_of_absent_table() -> Value {
    Value::from_serialize(&Column {
        table_name: Cow::Borrowed("manifest"),
        ..plain()
    })
}

/// A map that identifies as a column and does not carry `absent`.
///
/// The raw type and the decomposed parts are **siblings**, per `FR-CTX-014`
/// and `FR-CTX-015`, so `column_type` here is the raw string a column carries
/// and `data_type` stands beside it.
pub(super) fn column_without(absent: &str) -> Value {
    let whole = context! {
        name => "consignment_id",
        table_name => "consignment",
        nullable => false,
        auto_increment => false,
        column_type => "bigint(20) unsigned",
        data_type => "bigint",
    };

    let kept: BTreeMap<&str, Value> = [
        "name",
        "table_name",
        "nullable",
        "auto_increment",
        "column_type",
        "data_type",
    ]
    .into_iter()
    .filter(|key| *key != absent)
    .map(|key| (key, whole.get_attr(key).expect("the map carries the key")))
    .collect();

    Value::from_serialize(&kept)
}

/// The fixture's `consignment` table.
fn consignment() -> Table<'static> {
    document::fixture::consignment()
}

/// A plain column of `consignment`, with every attribute at its quiet value.
fn plain() -> Column<'static> {
    document::fixture::column("consignment", "consignment_id", 1, identifier())
}

/// The `bigint(20) unsigned` of the fixture.
fn identifier() -> ColumnType<'static> {
    document::fixture::identifier()
}
