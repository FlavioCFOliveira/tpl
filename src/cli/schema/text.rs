//! The `text` half of the first arm: the listings of `FR-SCH-026`, and the
//! sections a named object is laid out in.
//!
//! `FR-OUT-004` makes none of it a contract and `FR-SCH-027` repeats that for
//! this arm in particular: anything parsing a result uses `--format json`. What
//! it is instead is `FR-OUT-006` — a layout for a person, carrying the useful
//! information rather than only the name — and `FR-SCH-026` states the six
//! clauses that produce it. Those six are [`crate::output`]'s and are not
//! restated here: this module chooses the columns of each listing and nothing
//! about how they are laid out.
//!
//! **A listing is one table.** `FR-SCH-026` fixes the columns of
//! `tpl schema tables` and states that the field list of any other listing is a
//! property of that listing; the two others are chosen here, each leading with
//! the name, per `FR-OUT-006`. `FR-SCH-007` obliges the routine listing to
//! carry the kind as a column, and it carries the catalogue's own string, which
//! is what `FR-CAT-016` fixes.
//!
//! **A named object is several.** `FR-SCH-026` governs a *listing*, and one
//! object is not one; `FR-SCH-009` obliges `tpl schema table` to be exhaustive
//! over what the catalogue holds for that table, which is more than one table of
//! columns can carry. Each part is laid out by the same rule, under a heading,
//! and the whole is written through [`crate::output`] in one go — so the bytes
//! are aggregated as `FR-OUT-021` requires rather than emitted a section at a
//! time.
//!
//! **Every cell is escaped, and that reaches a view's SQL.** `FR-OUT-018` and
//! `FR-OUT-019` escape every interpolated value whatever its source, and
//! `FR-OUT-019` exempts two outputs, neither of which is this one. A view
//! definition and a routine body therefore appear on one line with their line
//! breaks escaped. It is not pretty and it is the rule: a value that could
//! forge a line of the layout is exactly what the requirement exists to
//! prevent, and `--format json` is where the definition is read from.

use std::borrow::Cow;

use super::super::layout::{Cell, PROPERTY, Sections, count, emit, flag, optional, text};
use crate::error::Error;
use crate::model::document::DatabaseDocument;
use crate::model::document::shape::TableDocument;
use crate::model::routine::Routine;
use crate::model::view::View;
use crate::output::{self, Order, Table};

/// The columns of `tpl schema tables` (`FR-SCH-026`).
const TABLES: [&str; 4] = ["NAME", "ENGINE", "COLUMNS", "COMMENT"];

/// The columns of `tpl schema views`.
const VIEWS: [&str; 4] = ["NAME", "UPDATABLE", "ALGORITHM", "SECURITY"];

/// The columns of `tpl schema routines`, whose second is the kind
/// `FR-SCH-007` obliges.
const ROUTINES: [&str; 5] = ["NAME", "KIND", "RETURNS", "DETERMINISTIC", "COMMENT"];

/// The columns of a table's column list.
const COLUMNS: [&str; 6] = ["NAME", "POSITION", "TYPE", "NULLABLE", "DEFAULT", "COMMENT"];

/// The columns of a table's index list.
const INDEXES: [&str; 5] = ["NAME", "UNIQUE", "TYPE", "COLUMNS", "COMMENT"];

/// The columns of a table's outgoing keys.
const FOREIGN_KEYS: [&str; 5] = ["NAME", "COLUMNS", "REFERENCES", "ON UPDATE", "ON DELETE"];

/// The columns of a table's incoming keys.
const REFERENCED_BY: [&str; 3] = ["TABLE", "CONSTRAINT", "COLUMNS"];

/// The columns of a table's triggers.
const TRIGGERS: [&str; 4] = ["NAME", "TIMING", "EVENT", "ORIENTATION"];

/// The columns of a table's `CHECK` constraints.
const CHECKS: [&str; 3] = ["NAME", "LEVEL", "CLAUSE"];

/// The columns of a routine's parameters.
const PARAMETERS: [&str; 3] = ["NAME", "MODE", "TYPE"];

/// What the cell of a default whose kind is `null` reads as (`FR-CTX-012`).
const NULL: &str = "NULL";

// --------------------------------------------------------------- listings ---

/// Writes the listing of `tpl schema tables` (`FR-SCH-026`).
///
/// # Errors
///
/// Returns [`Error::StdoutUnwritable`] where the stream refused the write.
pub(super) fn tables<W: std::io::Write>(
    out: W,
    selected: &[&TableDocument<'_>],
) -> Result<(), Error> {
    let rows: Vec<[Cell<'_>; 4]> = selected
        .iter()
        .map(|table| {
            [
                text(&table.name),
                optional(table.engine.as_deref()),
                count(table.columns.len()),
                text(&table.comment),
            ]
        })
        .collect();

    output::emit_table_to(out, &Table::new(TABLES, &rows, Order::ByName))
}

/// Writes the listing of `tpl schema views`.
///
/// # Errors
///
/// Returns [`Error::StdoutUnwritable`] where the stream refused the write.
pub(super) fn views<W: std::io::Write>(out: W, selected: &[&View<'_>]) -> Result<(), Error> {
    let rows: Vec<[Cell<'_>; 4]> = selected
        .iter()
        .map(|view| {
            [
                text(&view.name),
                flag(view.is_updatable),
                text(&view.algorithm),
                text(&view.security_type),
            ]
        })
        .collect();

    output::emit_table_to(out, &Table::new(VIEWS, &rows, Order::ByName))
}

/// Writes the listing of `tpl schema routines` (`FR-SCH-007`).
///
/// # Errors
///
/// Returns [`Error::StdoutUnwritable`] where the stream refused the write.
pub(super) fn routines<W: std::io::Write>(out: W, selected: &[&Routine<'_>]) -> Result<(), Error> {
    let rows: Vec<[Cell<'_>; 5]> = selected
        .iter()
        .map(|routine| {
            [
                text(&routine.name),
                text(routine.kind.name()),
                optional(routine.return_type.as_ref().map(|kind| kind.raw())),
                flag(routine.is_deterministic),
                text(&routine.comment),
            ]
        })
        .collect();

    output::emit_table_to(out, &Table::new(ROUTINES, &rows, Order::ByName))
}

// ---------------------------------------------------------- named objects ---

/// Writes the metadata of `tpl schema info`.
///
/// It is the three metadata fields of `FR-CTX-036`, the `server` object of
/// `FR-CTX-031`, and the size of each of the three collections of
/// `FR-CTX-035`. The collections themselves are not listed: `tpl schema tables`
/// and its two siblings list them, and the `json` form of this command carries
/// the whole object, per `FR-SCH-031`.
///
/// # Errors
///
/// Returns [`Error::StdoutUnwritable`] where the stream refused the write.
pub(super) fn info<W: std::io::Write>(
    out: W,
    database: &DatabaseDocument<'_>,
) -> Result<(), Error> {
    let rows: Vec<[Cell<'_>; 2]> = vec![
        [text("name"), text(&database.name)],
        [text("charset"), text(&database.charset)],
        [text("collation"), text(&database.collation)],
        [text("server"), text(database.server.version())],
        [text("series"), text(database.server.series())],
        [text("standing"), text(database.server.standing().name())],
        [text("tables"), count(database.tables.len())],
        [text("views"), count(database.views.len())],
        [text("routines"), count(database.routines.len())],
    ];

    output::emit_table_to(out, &Table::new(PROPERTY, &rows, Order::AsGiven))
}

/// Writes one table, exhaustively over what the catalogue holds for it
/// (`FR-SCH-009`).
///
/// # Errors
///
/// Returns [`Error::StdoutUnwritable`] where the stream refused the write.
pub(super) fn table<W: std::io::Write>(out: W, table: &TableDocument<'_>) -> Result<(), Error> {
    let mut sections = Sections::default();

    let own: Vec<[Cell<'_>; 2]> = vec![
        [text("name"), text(&table.name)],
        [text("type"), text(table.table_type.name())],
        [text("engine"), optional(table.engine.as_deref())],
        [text("collation"), optional(table.collation.as_deref())],
        [text("comment"), text(&table.comment)],
    ];
    sections.table(&Table::new(PROPERTY, &own, Order::AsGiven))?;

    // NFR-DET-002 excepts a table's columns: the order is the ordinal
    // position, which the document already carries.
    let columns: Vec<[Cell<'_>; 6]> = table
        .columns
        .iter()
        .map(|column| {
            [
                text(&column.name),
                Cow::Owned(column.position.to_string()),
                text(column.column_type.raw()),
                flag(column.nullable),
                column
                    .default
                    .as_ref()
                    .map_or(Cow::Borrowed(""), |default| {
                        // A column with no `DEFAULT` and one whose default is the
                        // literal `NULL` are two states, per `FR-CTX-011`, and an
                        // empty cell for both would make them one.
                        default.value().map_or(Cow::Borrowed(NULL), Cow::Borrowed)
                    }),
                text(&column.comment),
            ]
        })
        .collect();
    sections.part("COLUMNS", COLUMNS, &columns, Order::AsGiven)?;

    let indexes: Vec<[Cell<'_>; 5]> = table
        .indexes
        .iter()
        .map(|index| {
            [
                text(&index.name),
                flag(index.unique),
                text(&index.index_type),
                Cow::Owned(
                    index
                        .columns
                        .iter()
                        .map(|column| column.name.as_ref())
                        .collect::<Vec<&str>>()
                        .join(", "),
                ),
                text(&index.comment),
            ]
        })
        .collect();
    sections.part("INDEXES", INDEXES, &indexes, Order::ByName)?;

    let outgoing: Vec<[Cell<'_>; 5]> = table
        .foreign_keys
        .iter()
        .map(|key| {
            [
                text(&key.name),
                Cow::Owned(
                    key.columns
                        .iter()
                        .map(|pair| pair.column.as_ref())
                        .collect::<Vec<&str>>()
                        .join(", "),
                ),
                // FR-CTX-006: a key that names no table to reference carries
                // `null` where the embedded table stands, and the `text` form
                // shows the same absence it shows everywhere else.
                Cow::Owned(format!(
                    "{}({})",
                    key.referenced_table
                        .as_ref()
                        .map_or(NULL, |referenced| referenced.name.as_ref()),
                    key.columns
                        .iter()
                        .map(|pair| pair.referenced_column.as_ref())
                        .collect::<Vec<&str>>()
                        .join(", ")
                )),
                text(key.on_update.name()),
                text(key.on_delete.name()),
            ]
        })
        .collect();
    sections.part("FOREIGN KEYS", FOREIGN_KEYS, &outgoing, Order::ByName)?;

    // NFR-DET-002's default rule is read over the two names this collection
    // has, and the document already carries that order.
    let incoming: Vec<[Cell<'_>; 3]> = table
        .referenced_by
        .iter()
        .map(|incoming| {
            [
                text(&incoming.table.name),
                text(&incoming.key.name),
                Cow::Owned(
                    incoming
                        .key
                        .columns
                        .iter()
                        .map(|pair| pair.column.as_ref())
                        .collect::<Vec<&str>>()
                        .join(", "),
                ),
            ]
        })
        .collect();
    sections.part("REFERENCED BY", REFERENCED_BY, &incoming, Order::AsGiven)?;

    let triggers: Vec<[Cell<'_>; 4]> = table
        .triggers
        .iter()
        .map(|trigger| {
            [
                text(&trigger.name),
                text(trigger.timing.name()),
                text(trigger.event.name()),
                text(&trigger.orientation),
            ]
        })
        .collect();
    sections.part("TRIGGERS", TRIGGERS, &triggers, Order::ByName)?;

    let checks: Vec<[Cell<'_>; 3]> = table
        .check_constraints
        .iter()
        .map(|check| {
            [
                text(&check.name),
                text(check.level.name()),
                text(&check.clause),
            ]
        })
        .collect();
    sections.part("CHECK CONSTRAINTS", CHECKS, &checks, Order::ByName)?;

    emit(out, &sections)
}

/// Writes one view, its SQL definition included (`FR-SCH-006`).
///
/// # Errors
///
/// Returns [`Error::StdoutUnwritable`] where the stream refused the write.
pub(super) fn view<W: std::io::Write>(out: W, view: &View<'_>) -> Result<(), Error> {
    let rows: Vec<[Cell<'_>; 2]> = vec![
        [text("name"), text(&view.name)],
        [text("updatable"), flag(view.is_updatable)],
        [text("algorithm"), text(&view.algorithm)],
        [text("check_option"), text(&view.check_option)],
        [text("security_type"), text(&view.security_type)],
        [text("definer"), text(&view.definer)],
        [
            text("character_set_client"),
            text(&view.character_set_client),
        ],
        [
            text("collation_connection"),
            text(&view.collation_connection),
        ],
        [text("definition"), text(&view.definition)],
    ];

    output::emit_table_to(out, &Table::new(PROPERTY, &rows, Order::AsGiven))
}

/// Writes one routine, its parameters and its body included.
///
/// # Errors
///
/// Returns [`Error::StdoutUnwritable`] where the stream refused the write.
pub(super) fn routine<W: std::io::Write>(out: W, routine: &Routine<'_>) -> Result<(), Error> {
    let mut sections = Sections::default();

    let own: Vec<[Cell<'_>; 2]> = vec![
        [text("name"), text(&routine.name)],
        [text("kind"), text(routine.kind.name())],
        [
            text("returns"),
            optional(routine.return_type.as_ref().map(|kind| kind.raw())),
        ],
        [text("deterministic"), flag(routine.is_deterministic)],
        [text("sql_data_access"), text(&routine.sql_data_access)],
        [text("security_type"), text(&routine.security_type)],
        [text("definer"), text(&routine.definer)],
        [text("comment"), text(&routine.comment)],
        [text("body"), optional(routine.body.as_deref())],
    ];
    sections.table(&Table::new(PROPERTY, &own, Order::AsGiven))?;

    // NFR-DET-002 excepts a routine's parameters: the order is the
    // declaration order, which the document already carries.
    let parameters: Vec<[Cell<'_>; 3]> = routine
        .parameters
        .iter()
        .map(|parameter| {
            [
                text(&parameter.name),
                text(&parameter.mode),
                text(parameter.parameter_type.raw()),
            ]
        })
        .collect();
    sections.part("PARAMETERS", PARAMETERS, &parameters, Order::AsGiven)?;

    emit(out, &sections)
}
