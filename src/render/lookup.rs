//! Reaching back into the render context from a filter, a test or a global
//! function.
//!
//! `FR-RND-023` binds the model to the `database` variable of the render
//! context, and two of the seven tests must reach it: `FR-CTX-021` forbids
//! materialising on a column any fact its table already states, so
//! `primary_key` and `unique` resolve the column's `table_name` against the
//! context and ask the table, per `FR-CTX-022` and `FR-ENV-015`. The five
//! global functions of `FR-ENV-020` that resolve an object by name reach it the
//! same way, so the resolution is written once, here.
//!
//! The engine supplies the access: [`minijinja::State`] is passed to a filter,
//! a test or a function that declares it, and its lookup finds a variable of
//! the initial render context — which `database` is, under `FR-RND-023`.
//!
//! # Why the scan is linear
//!
//! `FR-CTX-003` makes every collection of the document a JSON **array**, and
//! `BR-CTX-001` records the rejection of maps keyed by object name: ordering
//! would then exist only in the emitter and `NFR-DET-002` would have no
//! observable subject. A lookup by name is therefore a scan, and it is one the
//! specification chose.
//!
//! # A table that is absent is not a table that answers `false`
//!
//! `FR-ENV-017` and `FR-SEM-017` make the absent table a `65` rather than a
//! negative answer, and `FR-SEM-018` makes the rule the same whether the column
//! came from a server or from a `--context` document assembled by hand.
//! [`table_of`] is where that is enforced; the five global functions do not
//! share it, because a lookup that finds nothing is answered in the form the
//! engine already has for absence — see [`super::function`].

use std::sync::OnceLock;

use minijinja::value::{Object, ValueKind};
use minijinja::{Error, ErrorKind, State, Value};

use super::operand::Column;

/// The context variable the model is bound to (`FR-RND-023`).
const DATABASE: &str = "database";

/// The collection of tables (`FR-CTX-036`).
pub(super) const TABLES: &str = "tables";

/// The collection of views (`FR-CTX-036`).
pub(super) const VIEWS: &str = "views";

/// The collection of routines (`FR-CTX-036`).
pub(super) const ROUTINES: &str = "routines";

/// The attribute every member of a collection is named by.
const NAME: &str = "name";

/// The member of `collection` named `name`, or [`None`].
///
/// [`None`] covers every way the lookup can fail to find one: no `database`
/// variable, a `database` that carries no such collection, and a collection
/// that carries no member of that name. A caller that must distinguish them
/// does not exist, because none of the three is a state a template can act on
/// differently.
pub(super) fn member(state: &State<'_, '_>, collection: &str, name: &str) -> Option<Value> {
    let members = state.lookup(DATABASE)?.get_attr(collection).ok()?;

    // FR-CACHE-038: a collection that knows its members' names without
    // reaching them answers the query, and reaches only the member it returns.
    let query = Value::from_object(Query {
        name: name.to_owned(),
        answer: OnceLock::new(),
    });
    let _ = members.get_item(&query);
    if let Some(answer) = query
        .downcast_object_ref::<Query>()
        .and_then(|asked| asked.answer.get())
    {
        return answer.clone();
    }

    members.try_iter().ok()?.find(|member| named(member, name))
}

/// A lookup by name, put to a collection as an item key (`FR-CACHE-038`).
///
/// A collection served from the cache knows the names of its members from the
/// listing, before any of them is read, and answers the query with the first
/// member of that name — reaching that member and no other — or with nothing.
/// A collection that cannot answer leaves the query unanswered, and
/// [`member`] then scans it, which over a whole document reads nothing it had
/// not already read.
///
/// The key is an object no template can construct, so the query is not part of
/// the surface `FR-ENV-001` fixes: a template indexing a collection with any
/// value it can write reaches the collection's ordinary item lookup.
///
/// The answer is the scan's own: the first member in the collection's order
/// whose name is `name`. For a procedure and a function of one name that is
/// the one the listing orders first, which is the one the scan meets first.
#[derive(Debug)]
pub(crate) struct Query {
    /// The name sought.
    name: String,
    /// The answer, set by the collection that could give one.
    answer: OnceLock<Option<Value>>,
}

impl Query {
    /// The name sought.
    pub(crate) fn name(&self) -> &str {
        &self.name
    }

    /// Records the collection's answer: the member found, or [`None`].
    pub(crate) fn answer(&self, found: Option<Value>) {
        let _ = self.answer.set(found);
    }
}

impl Object for Query {}

/// The table a column belongs to, resolved against the render context.
///
/// # Errors
///
/// Returns the `65` of `FR-ENV-017` and `FR-SEM-017` where the table named by
/// the column's `table_name` is absent from the context, naming the test and
/// the table. Answering `false` would report that the column is not part of a
/// primary key when the truth is that the question could not be asked.
pub(super) fn table_of(
    state: &State<'_, '_>,
    column: &Column<'_>,
    name: &str,
) -> Result<Value, Error> {
    let table_name = column.table_name()?;

    member(state, TABLES, &table_name).ok_or_else(|| {
        Error::new(
            ErrorKind::InvalidOperation,
            format!(
                "the test '{name}' could not resolve the table '{table_name}', which the \
                 render context does not carry"
            ),
        )
    })
}

/// The column of `table` named `name`, or [`None`].
pub(super) fn column_of(table: &Value, name: &str) -> Option<Value> {
    table
        .get_attr("columns")
        .ok()?
        .try_iter()
        .ok()?
        .find(|column| named(column, name))
}

/// The indexes of a table, in the order the document carries them.
///
/// The collection carries the primary key among the others, per `FR-CAT-010`
/// and `FR-CAT-043`, which is why both `primary_key` and `unique` answer from
/// this one collection and from no second source.
pub(super) fn indexes(table: &Value) -> impl Iterator<Item = Value> {
    table
        .get_attr("indexes")
        .ok()
        .and_then(|indexes| indexes.try_iter().ok())
        .into_iter()
        .flatten()
}

/// Whether an index names `column` among its columns.
pub(super) fn covers(index: &Value, column: &str) -> bool {
    index
        .get_attr("columns")
        .ok()
        .and_then(|columns| columns.try_iter().ok())
        .is_some_and(|mut columns| columns.any(|member| named(&member, column)))
}

/// Whether an index is one the model reports as unique (`FR-ENV-041`).
pub(super) fn unique(index: &Value) -> bool {
    index
        .get_attr("unique")
        .is_ok_and(|unique| unique.kind() == ValueKind::Bool && unique.is_true())
}

/// Whether a member of a collection carries this name.
fn named(member: &Value, name: &str) -> bool {
    member
        .get_attr(NAME)
        .is_ok_and(|carried| carried.as_str() == Some(name))
}
