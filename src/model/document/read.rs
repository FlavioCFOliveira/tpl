//! The inward direction: a supplied document becomes a model (`FR-CTX-033`,
//! `FR-RND-020`, `FR-SCH-036`).
//!
//! On this path the document is **untrusted input**. What that obliges is
//! narrow and is stated twice over, once as what is checked and once as what is
//! not:
//!
//! | Checked | Where |
//! |---|---|
//! | The three envelope keys of `FR-OUT-024`, with `source` one of the four values of `FR-OUT-026` | [`Document`](crate::output::Document)'s derived deserialisation |
//! | Every key the document contract names, with the type it fixes | Each type's derived deserialisation; an absent key is an error, because `FR-OUT-012` emits an absent **value** as `null` rather than omitting the key |
//! | `version`, `series` and `standing` present, strings, and `standing` one of the two values of `FR-CTX-034` | [`Server`](crate::model::server::Server)'s three fields and closed enumeration |
//! | No key of a table names a column the table does not carry (`FR-CAT-044`) | [`Table::assemble`], the model's only constructor for a table |
//! | A `restricted` marking names at least one property (`FR-PRIV-016`) | [`Restricted`](crate::model::restricted::Restricted)'s `TryFrom` |
//! | Every table a foreign key names, under `foreign_keys` or `referenced_by`, is a member of `tables` (`FR-CTX-042`) | [`references_are_carried`] |
//!
//! | **Not** checked | Why |
//! |---|---|
//! | `series` against the supported window of `FR-SRV-015` | `FR-CTX-033` forbids it. `FR-RND-022` opens no connection on this path, so there is no server to vouch for; validating the window would make every committed dump expire on a calendar date as the window moved |
//! | `standing` against `series` | `FR-CTX-033` forbids it in as many words |
//! | The value of `schema_version` | `FR-OUT-014` fixes what moves it and names no rule for refusing one, and refusing a number this binary has not seen would be a check the corpus does not ask for |
//! | `primary_key` against `indexes` | `FR-CAT-043` makes the index collection the authoritative source, so the model takes the primary key from there and this key is not read back at all |
//! | A column's `table_name` against `tables` | `FR-CTX-042` excludes it by name: `FR-SEM-017`, `FR-SEM-018` and `FR-ENV-017` make a column naming an absent table reachable on purpose, and the test that resolves it fails the render when it is applied |
//! | The embedded tables of `FR-CTX-006` and `FR-CTX-010` against the members they copy | `FR-CTX-042` does not check them apart; only the name each carries is read back, and that name is what the rule above checks |
//!
//! **Nothing is repaired.** A document that does not match the contract is an
//! [`Error`](crate::Error) at `65`, per `FR-RND-020`; the two conditions this
//! module detects itself each name the rule they failed, which is what the `65`
//! row of `FR-ERR-034` obliges the `cause` line to carry.

use super::order;
use super::shape::{DatabaseDocument, TableDocument, TableShape};
use crate::error::ContextFault;
use crate::model::database::Database;
use crate::model::foreign_key::{ForeignKey, IncomingForeignKey};
use crate::model::table::{Table, TableParts};

/// The rule `FR-CAT-044` states, as the `cause` line names it.
const KEYS_NAME_CARRIED_COLUMNS: &str = "breaks a rule: every column an index or a key of this table names must be one of its own \
     columns";

/// Reads the `database` object back as a model.
///
/// # Errors
///
/// Returns the [`ContextFault`] of the first structural rule the document
/// fails. The caller adds the path, which is the other half of what
/// `FR-ERR-034` obliges.
pub(super) fn database(document: DatabaseDocument<'_>) -> Result<Database<'_>, ContextFault> {
    let mut tables = Vec::with_capacity(document.tables.len());
    for (index, table) in document.tables.into_iter().enumerate() {
        tables.push(self::table(table, index)?);
    }

    references_are_carried(&tables)?;

    Ok(Database {
        name: document.name,
        charset: document.charset,
        collation: document.collation,
        server: document.server,
        tables,
        views: document.views.into_owned(),
        routines: document.routines.into_owned(),
    })
}

/// Reads one table back, through the constructor that enforces `FR-CAT-044`.
///
/// The embedding is undone here, and undoing it is what makes the round trip of
/// `FR-SCH-022` a property of one representation: the model carries the name of
/// a referenced table, so each embedded object contributes exactly the name it
/// was built from and nothing else of it is read. The embedded table itself is
/// present in `tables` in its own right, per `FR-CTX-023`.
fn table(document: TableDocument<'_>, index: usize) -> Result<Table<'_>, ContextFault> {
    let TableShape {
        name,
        table_type,
        engine,
        collation,
        comment,
        columns,
        indexes,
        primary_key: _,
        foreign_keys,
        referenced_by,
        triggers,
        check_constraints,
        restricted,
    } = document;

    Table::assemble(TableParts {
        name,
        table_type,
        engine,
        collation,
        comment,
        columns: columns.into_owned(),
        indexes: indexes.into_owned(),
        foreign_keys: foreign_keys
            .into_iter()
            .map(|key| ForeignKey {
                name: key.name,
                columns: key.columns.into_owned(),
                // The embedding is undone by taking the name the embedded
                // object was built from, and `null` there is a key that named
                // no table, per `FR-CTX-006` and `FR-CAT-056`: it reads back
                // as the `None` the model carries.
                referenced_table: key.referenced_table.map(|referenced| referenced.name),
                referenced_key: key.referenced_key,
                match_option: key.match_option,
                on_update: key.on_update,
                on_delete: key.on_delete,
            })
            .collect(),
        referenced_by: referenced_by
            .into_iter()
            .map(|incoming| IncomingForeignKey {
                table: incoming.table.name,
                key: ForeignKey {
                    name: incoming.key.name,
                    columns: incoming.key.columns.into_owned(),
                    referenced_table: incoming.key.referenced_table,
                    referenced_key: incoming.key.referenced_key,
                    match_option: incoming.key.match_option,
                    on_update: incoming.key.on_update,
                    on_delete: incoming.key.on_delete,
                },
            })
            .collect(),
        triggers: triggers.into_owned(),
        check_constraints: check_constraints.into_owned(),
        restricted,
    })
    .map_err(|_| ContextFault::Structure {
        at: format!("data.database.tables[{index}]"),
        expected: KEYS_NAME_CARRIED_COLUMNS.to_owned(),
    })
}

/// The collection of a table that lists the keys it references others by.
const FOREIGN_KEYS: &str = "foreign_keys";

/// The collection of a table that lists the keys others reference it by.
const REFERENCED_BY: &str = "referenced_by";

/// Applies `FR-CTX-042`: every table a foreign key of a member of `tables`
/// names is itself a member of `tables`.
///
/// It is checked here, on the inward path, because the outward builder needs
/// the same condition to materialise the embedding of `ADR-009` and holds it as
/// an invariant: `FR-CTX-023` guarantees it for a document a server read
/// produced, and nothing guarantees it for a document a caller supplied. A
/// document that breaks it is the caller's fault to correct — `65` — and never
/// the `70` the invariant would report.
///
/// The names are resolved exactly as the builder resolves them: byte for byte,
/// by binary search over the members ordered by the one comparator of
/// [`order`]. A name that passes here is therefore a name the builder finds.
/// A key whose referenced table is `null` names no table, per `FR-CTX-006`,
/// and is passed over.
///
/// # Errors
///
/// Returns [`ContextFault::DanglingReference`] for the first key, in document
/// order — each member in turn, its `foreign_keys` before its
/// `referenced_by` — that names a table `tables` does not carry.
fn references_are_carried(tables: &[Table<'_>]) -> Result<(), ContextFault> {
    let carried = order::pointers_by_name(tables);
    let present = |name: &str| {
        carried
            .binary_search_by(|table| order::compare(table.name(), name))
            .is_ok()
    };

    for table in tables {
        for outgoing in table.foreign_keys() {
            if let Some(named) = outgoing.referenced_table.as_deref()
                && !present(named)
            {
                return Err(ContextFault::DanglingReference {
                    table: table.name().to_owned(),
                    collection: FOREIGN_KEYS,
                    key: outgoing.name.as_ref().to_owned(),
                    names: named.to_owned(),
                });
            }
        }

        for incoming in table.referenced_by() {
            if !present(&incoming.table) {
                return Err(ContextFault::DanglingReference {
                    table: table.name().to_owned(),
                    collection: REFERENCED_BY,
                    key: incoming.key.name.as_ref().to_owned(),
                    names: incoming.table.as_ref().to_owned(),
                });
            }
        }
    }

    Ok(())
}
