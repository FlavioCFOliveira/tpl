//! The `database` variable of `FR-RND-023`, converted where a template reads
//! it and nowhere else.
//!
//! `FR-RND-023` makes the whole database **reachable** from every render; it
//! does not make a template pay for what it does not read. Converting the
//! document into `minijinja` values before the template ran cost 41.4% of a
//! render of the `example` template over `WL-001`, and most of a 26.7 MB heap
//! peak, for a result that reads one table and the database name
//! (`BENCHMARKS.md`, 2026-09-22, row 6 of the waste register). Here the three
//! outer levels — the database, its three collections and each table — are
//! [`Object`]s that convert a member the first time it is read, and keep it.
//!
//! # What a template sees is what the whole conversion produced
//!
//! Every behaviour of a value — iteration, `length`, `in`, truthiness,
//! equality, ordering, hashing, `|json`, `|pprint`, `|string` and item access
//! — is derived by the engine from four answers: the representation, the key
//! or index lookup, the enumeration and its length. Each object here gives the
//! four answers the value [`Value::from_serialize`] builds would give, so no
//! behaviour can differ:
//!
//! | Level | The whole conversion builds | This module answers as |
//! |---|---|---|
//! | the database, a table | a map of the struct's fields, in declaration order, looked up by name | [`Database`], [`Table`]: the same keys, in the same order, looked up the same way |
//! | a collection | a sequence, indexed by position | [`Members`]: the same length, indexed the same way |
//! | every member below a table's fields | itself | [`Value::from_serialize`] of that member, unchanged |
//!
//! The table's `restricted` key is carried only where the table carries the
//! marking, which is what `skip_serializing_if` does on the struct.
//!
//! **A member is converted once.** Each object caches what it converted, so a
//! member read twice is the same value both times: `is sameas` answers as it
//! answered over the whole conversion, and the linear scans of
//! `crate::render::lookup` clone the values they compare rather than rebuild
//! them.
//!
//! # A render served from the cache reads a member's file when it is reached
//!
//! `FR-CACHE-038` moves the read one level further out. Over the cache the
//! collections are not a decoded document but a [`Store`]: the members as the
//! directory listing names them, in the order an up-front read would give
//! them, and one slot per member for its one read. Reaching a member reads,
//! validates and decodes its file, and every level above and below answers as
//! it answers over a whole document, so the table above holds for both
//! sources. A member whose file is a miss reads as an invalid value and is
//! recorded, and the caller abandons the render, per `FR-CACHE-039`.
//!
//! *Rejected: reading a member's name from the listing and its file only when
//! a field other than the name is read.* It would leave a member reached for
//! its name alone — the scans of `crate::render::lookup` — unvalidated, which
//! is a looser reading of "reaches" than `FR-CACHE-038` states.
//!
//! # Why the document is copied only where the caller carries on
//!
//! `minijinja` holds an object behind an `Arc` with a `'static` bound, and the
//! document borrows the buffers its source was read into — the cache files,
//! the `--context` bytes, the rows of a server read. Where the process exits
//! when the command returns, those buffers are leaked and the document arrives
//! [`Served::leaked`], already borrowing `'static` data, and is held as it is.
//! Otherwise it arrives [`Served::borrowed`] and is copied once into
//! a [`DatabaseDocument<'static>`] through [`ToStatic`], which copies each
//! string and builds nothing else; the copy is freed with the context.
//!
//! *Rejected: converting the whole document, as before.* It is the cost this
//! module removes.
//!
//! *Rejected: copying on both endings, as `#239` did.* The copy cost 0.81 to
//! 1.08 ms and 3.65 MB of every render of `WL-001`, and made a template that
//! reads the whole database 1 ms slower than the whole conversion
//! (`BENCHMARKS.md`, 2026-09-22, row 3 of the waste register).
//!
//! *Rejected: ending the borrow without a copy or a leak.* It takes `unsafe`,
//! which this crate forbids, or a self-referential dependency, which the
//! budget refuses.
//!
//! *Rejected: lazy objects below a table's fields — a column, an index.* A
//! template that reads a table's columns reads them all, and the one member
//! whose conversion is expensive, `Column`'s flattened type, would be
//! converted all the same; a further level adds an object per member and
//! changes no outcome a measured template reaches.

use std::fmt;
use std::ops::Deref;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, OnceLock};

use minijinja::value::{Enumerator, Object, ObjectRepr};
use minijinja::{ErrorKind, Value};

use crate::cache::Shelved;
use crate::cli::Ending;
use crate::cli::source::Served;
use crate::model::ToStatic;
use crate::model::document::DatabaseDocument;
use crate::model::document::shape::TableDocument;
use crate::model::routine::Routine;
use crate::model::view::View;

/// The keys of the `database` object, in the declaration order of
/// [`DatabaseDocument`], which is the order `FR-CTX-036` writes them in and the
/// order a template iterates them in.
const DATABASE_KEYS: [&str; 7] = [
    "name",
    "charset",
    "collation",
    "server",
    "tables",
    "views",
    "routines",
];

/// The keys of a table, in the declaration order of
/// [`crate::model::document::shape::TableShape`]; the last is present only
/// where the table carries the marking of `FR-PRIV-016`.
const TABLE_KEYS: [&str; 13] = [
    "name",
    "table_type",
    "engine",
    "collation",
    "comment",
    "columns",
    "indexes",
    "primary_key",
    "foreign_keys",
    "referenced_by",
    "triggers",
    "check_constraints",
    "restricted",
];

/// The position of `restricted` in [`TABLE_KEYS`].
const RESTRICTED: usize = 12;

/// What a member that is a miss reads as, once [`Store`] has recorded it.
///
/// No template sees it: the render it is read in is abandoned under
/// `FR-CACHE-039`, whatever the render then does with the value.
const MISSED: &str = "a cache file this render reached is a miss, and the render is abandoned";

/// The `database` variable, over `document` (`FR-RND-023`).
pub(super) fn database(document: Served<'_, '_>) -> Value {
    let document = match document.leaked_document() {
        Some(leaked) => Held::Leaked(leaked),
        None => Held::Copied(Arc::new(document.to_static())),
    };

    Value::from_object(Database {
        contents: Contents::Document(document),
        members: Default::default(),
    })
}

/// The `database` variable, over the cache files `store` names
/// (`FR-RND-023`, `FR-CACHE-038`).
pub(super) fn shelved(store: &Arc<Store>) -> Value {
    Value::from_object(Database {
        contents: Contents::Shelved(Arc::clone(store)),
        members: Default::default(),
    })
}

/// The document every object of this module reads from: leaked, or a copy
/// shared among them and freed with the last.
#[derive(Debug, Clone)]
enum Held {
    /// A document that borrows buffers leaked for the rest of the process.
    Leaked(&'static DatabaseDocument<'static>),
    /// A copy of a document whose buffers the caller frees.
    Copied(Arc<DatabaseDocument<'static>>),
}

impl Deref for Held {
    type Target = DatabaseDocument<'static>;

    fn deref(&self) -> &Self::Target {
        match self {
            Self::Leaked(document) => document,
            Self::Copied(document) => document,
        }
    }
}

/// Where the members of the `database` object come from.
#[derive(Debug, Clone)]
enum Contents {
    /// A whole document, already decoded: a server read, a `--context`
    /// document.
    Document(Held),
    /// The cache files of `FR-CACHE-038`, each read when first reached.
    Shelved(Arc<Store>),
}

impl Contents {
    /// The document the database's own four members are read from.
    ///
    /// For [`Contents::Shelved`] its three collections are empty: the members
    /// are [`Store`]'s.
    fn head(&self) -> &DatabaseDocument<'static> {
        match self {
            Self::Document(document) => document,
            Self::Shelved(store) => &store.head,
        }
    }

    /// How many members `collection` holds.
    fn len(&self, collection: Collection) -> usize {
        match (self, collection) {
            (Self::Document(document), Collection::Tables) => document.tables.len(),
            (Self::Document(document), Collection::Views) => document.views.len(),
            (Self::Document(document), Collection::Routines) => document.routines.len(),
            (Self::Shelved(store), Collection::Tables) => store.shelved.tables().len(),
            (Self::Shelved(store), Collection::Views) => store.shelved.views().len(),
            (Self::Shelved(store), Collection::Routines) => store.shelved.routines().len(),
        }
    }
}

/// The cache files one render is served from, read when the template first
/// reaches each member (`FR-CACHE-038`, `FR-CACHE-039`).
///
/// It holds what was read before the render started — `database.json` decoded,
/// and each collection as the directory listing names it — and, for each
/// member, the result of its one read. A member is read at most once per
/// render: the bound object's file, read before the render starts, is the one
/// every later reach of that member is served from, and a file changed after
/// its read is not read again.
///
/// **A member whose file is a miss is recorded here**, and reads as an invalid
/// value, which fails the render wherever the engine operates on it. The render
/// is abandoned whether or not it failed: [`Store::missed`] is what the caller
/// asks once the render has returned, and a render that tolerated the value
/// has still read a document the cache could not serve.
pub(in crate::cli::render) struct Store {
    /// The database's own four members, in a document whose collections are
    /// empty.
    head: Held,
    /// What was read before the render started.
    shelved: Shelved,
    /// Each table's read, by position in the listing.
    tables: Box<[OnceLock<Option<Arc<TableDocument<'static>>>>]>,
    /// Each view's read.
    views: Box<[OnceLock<Option<Arc<View<'static>>>>]>,
    /// Each routine's read.
    routines: Box<[OnceLock<Option<Arc<Routine<'static>>>>]>,
    /// Whether the files' bytes are leaked for the rest of the process or the
    /// members copied out of them (see [`Held`]).
    ending: Ending,
    /// Whether a member this render reached was a miss.
    missed: AtomicBool,
}

impl Store {
    /// The store over `shelved`, or [`None`] where `database.json` fails to
    /// decode, which is a miss per `FR-CACHE-033`.
    ///
    /// The database's own members are copied, whatever the ending: they are
    /// three strings and the `server` object, and the copy frees the store of
    /// a borrow of its own field.
    pub(in crate::cli::render) fn open(shelved: Shelved, ending: Ending) -> Option<Self> {
        let head = Held::Copied(Arc::new(shelved.head()?.to_static()));

        Some(Self {
            head,
            tables: slots(shelved.tables().len()),
            views: slots(shelved.views().len()),
            routines: slots(shelved.routines().len()),
            shelved,
            ending,
            missed: AtomicBool::new(false),
        })
    }

    /// The database's name.
    pub(in crate::cli::render) fn name(&self) -> &str {
        &self.head.name
    }

    /// What was read before the render started.
    pub(in crate::cli::render) const fn shelved(&self) -> &Shelved {
        &self.shelved
    }

    /// The table at `index` of the listing, read on the first call, or
    /// [`None`] where its file is a miss (`FR-CACHE-033`).
    pub(in crate::cli::render) fn table(
        &self,
        index: usize,
    ) -> Option<Arc<TableDocument<'static>>> {
        let shelf = self.shelved.tables().get(index)?;

        self.tables
            .get(index)?
            .get_or_init(|| {
                // Once a member was a miss the render is abandoned, so it reads
                // no further file: it fails at every member it reaches next.
                if self.missed() {
                    return None;
                }

                settled(
                    shelf.read()?,
                    self.ending,
                    |bytes| shelf.table(bytes),
                    |bytes| shelf.table(bytes).map(|table| table.to_static()),
                )
                .map(Arc::new)
            })
            .clone()
    }

    /// The view at `index` of the listing, on the terms of [`Store::table`].
    pub(in crate::cli::render) fn view(&self, index: usize) -> Option<Arc<View<'static>>> {
        let shelf = self.shelved.views().get(index)?;

        self.views
            .get(index)?
            .get_or_init(|| {
                // Once a member was a miss the render is abandoned, so it reads
                // no further file: it fails at every member it reaches next.
                if self.missed() {
                    return None;
                }

                settled(
                    shelf.read()?,
                    self.ending,
                    |bytes| shelf.view(bytes),
                    |bytes| shelf.view(bytes).map(|view| view.to_static()),
                )
                .map(Arc::new)
            })
            .clone()
    }

    /// The routine at `index` of the listing, on the terms of
    /// [`Store::table`].
    pub(in crate::cli::render) fn routine(&self, index: usize) -> Option<Arc<Routine<'static>>> {
        let shelf = self.shelved.routines().get(index)?;

        self.routines
            .get(index)?
            .get_or_init(|| {
                // Once a member was a miss the render is abandoned, so it reads
                // no further file: it fails at every member it reaches next.
                if self.missed() {
                    return None;
                }

                settled(
                    shelf.read()?,
                    self.ending,
                    |bytes| shelf.routine(bytes),
                    |bytes| shelf.routine(bytes).map(|routine| routine.to_static()),
                )
                .map(Arc::new)
            })
            .clone()
    }

    /// Whether a member this render reached was a miss, which abandons the
    /// render under `FR-CACHE-039`.
    ///
    /// It is read by the render's own thread once the render has returned,
    /// and by the deadline's timer thread while it runs (see
    /// `crate::cli::render`'s `bounded`). The flag guards no other data, so
    /// Acquire and Release are more than it needs; they are taken so that the
    /// timer's reading needs no argument.
    pub(in crate::cli::render) fn missed(&self) -> bool {
        self.missed.load(Ordering::Acquire)
    }

    /// Records a miss, and answers the value the member reads as.
    fn miss(&self) -> Value {
        self.missed.store(true, Ordering::Release);

        Value::from(minijinja::Error::new(ErrorKind::InvalidOperation, MISSED))
    }
}

impl fmt::Debug for Store {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Store")
            .field("name", &self.head.name)
            .field("missed", &self.missed())
            .finish_non_exhaustive()
    }
}

/// One empty slot per member of a collection of `len`.
fn slots<T>(len: usize) -> Box<[OnceLock<T>]> {
    (0..len).map(|_| OnceLock::new()).collect()
}

/// The member `bytes` hold, with the lifetime the render needs.
///
/// Under [`Ending::Process`] the bytes are leaked and the member borrows them
/// for the rest of the process, as a whole read's buffers are (see
/// `crate::cli::source`); under [`Ending::Caller`] the member is copied out of
/// them and they are freed.
fn settled<S>(
    bytes: String,
    ending: Ending,
    leaked: impl FnOnce(&'static str) -> Option<S>,
    copied: impl FnOnce(&str) -> Option<S>,
) -> Option<S> {
    match ending {
        Ending::Process => leaked(bytes.leak()),
        Ending::Caller => copied(&bytes),
    }
}

/// The `database` object: seven members, each converted when first read.
struct Database {
    /// Where the members are read from.
    contents: Contents,

    /// The converted members, by position in [`DATABASE_KEYS`].
    members: [OnceLock<Value>; DATABASE_KEYS.len()],
}

impl Database {
    /// The member at `position` of [`DATABASE_KEYS`], converted.
    fn convert(&self, position: usize) -> Value {
        let head = self.contents.head();

        match position {
            0 => Value::from_serialize(&head.name),
            1 => Value::from_serialize(&head.charset),
            2 => Value::from_serialize(&head.collation),
            3 => Value::from_serialize(&head.server),
            4 => Members::value(&self.contents, Collection::Tables),
            5 => Members::value(&self.contents, Collection::Views),
            _ => Members::value(&self.contents, Collection::Routines),
        }
    }
}

impl Object for Database {
    fn get_value(self: &Arc<Self>, key: &Value) -> Option<Value> {
        self.get_value_by_str(key.as_str()?)
    }

    fn get_value_by_str(self: &Arc<Self>, key: &str) -> Option<Value> {
        let position = DATABASE_KEYS.iter().position(|name| *name == key)?;

        Some(
            self.members[position]
                .get_or_init(|| self.convert(position))
                .clone(),
        )
    }

    fn enumerate(self: &Arc<Self>) -> Enumerator {
        Enumerator::Values(DATABASE_KEYS.iter().copied().map(Value::from).collect())
    }

    fn enumerator_len(self: &Arc<Self>) -> Option<usize> {
        Some(DATABASE_KEYS.len())
    }
}

impl fmt::Debug for Database {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Database")
            .field("name", &self.contents.head().name)
            .finish_non_exhaustive()
    }
}

/// Which of the three collections of `FR-CTX-036` a [`Members`] indexes.
#[derive(Debug, Clone, Copy)]
enum Collection {
    /// `database.tables`, whose members are [`Table`] objects.
    Tables,
    /// `database.views`, whose members are converted whole.
    Views,
    /// `database.routines`, whose members are converted whole.
    Routines,
}

/// One collection of `FR-CTX-036`: a sequence whose members are converted when
/// first read.
struct Members {
    /// Where the collection is read from.
    contents: Contents,

    /// Which collection this is.
    collection: Collection,

    /// The converted members, by position.
    members: Box<[OnceLock<Value>]>,
}

impl Members {
    /// The collection as a value, holding its members not yet converted.
    fn value(contents: &Contents, collection: Collection) -> Value {
        Value::from_object(Self {
            contents: contents.clone(),
            collection,
            members: (0..contents.len(collection))
                .map(|_| OnceLock::new())
                .collect(),
        })
    }

    /// The member at `index`, converted.
    fn convert(&self, index: usize) -> Value {
        match &self.contents {
            Contents::Document(document) => match self.collection {
                Collection::Tables => Table::value(Row::Document(document.clone(), index)),
                Collection::Views => Value::from_serialize(&document.views[index]),
                Collection::Routines => Value::from_serialize(&document.routines[index]),
            },
            // FR-CACHE-038: the member's file is read here, the first time the
            // template reaches it, and a miss is FR-CACHE-039's.
            Contents::Shelved(store) => match self.collection {
                Collection::Tables => store
                    .table(index)
                    .map_or_else(|| store.miss(), |table| Table::value(Row::Alone(table))),
                Collection::Views => store
                    .view(index)
                    .map_or_else(|| store.miss(), |view| Value::from_serialize(&*view)),
                Collection::Routines => store
                    .routine(index)
                    .map_or_else(|| store.miss(), |routine| Value::from_serialize(&*routine)),
            },
        }
    }
}

impl Object for Members {
    fn repr(self: &Arc<Self>) -> ObjectRepr {
        ObjectRepr::Seq
    }

    fn get_value(self: &Arc<Self>, key: &Value) -> Option<Value> {
        let index = key.as_usize()?;
        let member = self.members.get(index)?;

        Some(member.get_or_init(|| self.convert(index)).clone())
    }

    fn enumerate(self: &Arc<Self>) -> Enumerator {
        Enumerator::Seq(self.members.len())
    }
}

impl fmt::Debug for Members {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Members")
            .field("collection", &self.collection)
            .field("len", &self.members.len())
            .finish_non_exhaustive()
    }
}

/// Where one [`Table`] is read from.
#[derive(Debug, Clone)]
enum Row {
    /// The member at this position of a whole document's tables.
    Document(Held, usize),
    /// A table read on its own, from its cache file.
    Alone(Arc<TableDocument<'static>>),
}

impl Deref for Row {
    type Target = TableDocument<'static>;

    fn deref(&self) -> &Self::Target {
        match self {
            Self::Document(document, index) => &document.tables[*index],
            Self::Alone(table) => table,
        }
    }
}

/// One member of `database.tables`: the table's fields, each converted when
/// first read.
struct Table {
    /// Where the table is read from.
    row: Row,

    /// The converted fields, by position in [`TABLE_KEYS`].
    members: [OnceLock<Value>; TABLE_KEYS.len()],
}

impl Table {
    /// The table `row` holds, as a value.
    fn value(row: Row) -> Value {
        Value::from_object(Self {
            row,
            members: Default::default(),
        })
    }

    /// How many of [`TABLE_KEYS`] this table carries: all of them where it
    /// carries the `restricted` marking, and all but that one otherwise.
    fn carried(&self) -> usize {
        if self.row.restricted.is_some() {
            TABLE_KEYS.len()
        } else {
            RESTRICTED
        }
    }

    /// The field at `position` of [`TABLE_KEYS`], converted.
    fn convert(&self, position: usize) -> Value {
        let table = &*self.row;

        match position {
            0 => Value::from_serialize(&table.name),
            1 => Value::from_serialize(table.table_type),
            2 => Value::from_serialize(&table.engine),
            3 => Value::from_serialize(&table.collation),
            4 => Value::from_serialize(&table.comment),
            5 => Value::from_serialize(&table.columns),
            6 => Value::from_serialize(&table.indexes),
            7 => Value::from_serialize(&table.primary_key),
            8 => Value::from_serialize(&table.foreign_keys),
            9 => Value::from_serialize(&table.referenced_by),
            10 => Value::from_serialize(&table.triggers),
            11 => Value::from_serialize(&table.check_constraints),
            _ => Value::from_serialize(&table.restricted),
        }
    }
}

impl Object for Table {
    fn get_value(self: &Arc<Self>, key: &Value) -> Option<Value> {
        self.get_value_by_str(key.as_str()?)
    }

    fn get_value_by_str(self: &Arc<Self>, key: &str) -> Option<Value> {
        let position = TABLE_KEYS[..self.carried()]
            .iter()
            .position(|name| *name == key)?;

        Some(
            self.members[position]
                .get_or_init(|| self.convert(position))
                .clone(),
        )
    }

    fn enumerate(self: &Arc<Self>) -> Enumerator {
        Enumerator::Values(
            TABLE_KEYS[..self.carried()]
                .iter()
                .copied()
                .map(Value::from)
                .collect(),
        )
    }

    fn enumerator_len(self: &Arc<Self>) -> Option<usize> {
        Some(self.carried())
    }
}

impl fmt::Debug for Table {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Table")
            .field("name", &self.row.name)
            .finish_non_exhaustive()
    }
}

#[cfg(test)]
mod tests {
    use super::{Store, database, shelved};
    use crate::cache::{Cache, Covered};
    use crate::cli::Ending;
    use crate::cli::source::Served;
    use crate::model::ToStatic;
    use crate::model::document;
    use crate::model::restricted::Restricted;
    use crate::project::scratch::Scratch;
    use minijinja::Value;
    use minijinja::tests::is_sameas;
    use minijinja::value::ValueKind;
    use std::borrow::Cow;
    use std::sync::Arc;

    /// Walks `lazy` and `eager` together and asserts that every level answers
    /// alike: the kind, the length, the truthiness, the keys in order, each
    /// member, a key neither carries, and the three renderings a template can
    /// ask for.
    fn same(lazy: &Value, eager: &Value, path: &str) {
        assert_eq!(lazy.kind(), eager.kind(), "{path}: kind");
        assert_eq!(lazy.len(), eager.len(), "{path}: len");
        assert_eq!(lazy.is_true(), eager.is_true(), "{path}: truthiness");
        assert_eq!(lazy, eager, "{path}: equality");
        assert_eq!(
            lazy.cmp(eager),
            std::cmp::Ordering::Equal,
            "{path}: ordering"
        );
        assert_eq!(format!("{lazy}"), format!("{eager}"), "{path}: display");
        assert_eq!(format!("{lazy:#?}"), format!("{eager:#?}"), "{path}: debug");
        assert_eq!(
            serde_json::to_string(lazy).expect("a value serialises"),
            serde_json::to_string(eager).expect("a value serialises"),
            "{path}: serialisation"
        );

        match eager.kind() {
            ValueKind::Map => {
                let keys: Vec<Value> = lazy.try_iter().expect("a map iterates").collect();
                let expected: Vec<Value> = eager.try_iter().expect("a map iterates").collect();

                assert_eq!(keys, expected, "{path}: keys");
                for key in &expected {
                    let name = key.as_str().expect("every key is a string");
                    same(
                        &lazy.get_attr(name).expect("the key is readable"),
                        &eager.get_attr(name).expect("the key is readable"),
                        &format!("{path}.{name}"),
                    );
                }
                for absent in ["nosuch", "restricted"] {
                    assert_eq!(
                        lazy.get_attr(absent).expect("a map answers").is_undefined(),
                        eager
                            .get_attr(absent)
                            .expect("a map answers")
                            .is_undefined(),
                        "{path}.{absent}"
                    );
                }
                assert_eq!(
                    lazy.get_item(&Value::from(0))
                        .expect("a map answers")
                        .is_undefined(),
                    eager
                        .get_item(&Value::from(0))
                        .expect("a map answers")
                        .is_undefined(),
                    "{path}[0]"
                );
            }
            ValueKind::Seq => {
                let len = eager.len().expect("a sequence has a length");

                for index in 0..len {
                    same(
                        &lazy
                            .get_item_by_index(index)
                            .expect("the index is readable"),
                        &eager
                            .get_item_by_index(index)
                            .expect("the index is readable"),
                        &format!("{path}[{index}]"),
                    );
                }
                for key in [Value::from(-1), Value::from(len), Value::from("name")] {
                    assert_eq!(
                        lazy.get_item(&key).expect("a sequence answers"),
                        eager.get_item(&key).expect("a sequence answers"),
                        "{path}[{key:?}]"
                    );
                }
            }
            _ => {}
        }
    }

    #[test]
    fn fr_rnd_023_the_database_answers_exactly_as_its_whole_conversion() {
        // Every level of the fixture — whose view and routine carry the
        // `restricted` marking, and whose tables reference each other in both
        // directions — walked against `Value::from_serialize` of the same
        // document. One table is given the marking too, because the fixture's
        // tables are complete and the key is the one a table carries only
        // sometimes.
        let model = document::fixture::database();
        let mut built = document::context(&model).expect("the fixture model is coherent");

        same(
            &database(Served::borrowed(&built)),
            &Value::from_serialize(&built),
            "database",
        );

        built.tables[0].restricted = Restricted::new(vec![Cow::Borrowed("triggers")]);
        same(
            &database(Served::borrowed(&built)),
            &Value::from_serialize(&built),
            "restricted",
        );
    }

    #[test]
    fn fr_rnd_023_a_leaked_document_answers_as_a_copied_one() {
        // The document the exiting process hands over is held as it is, not
        // copied; what it answers must not depend on which. The one fixture
        // copy leaked here is the price of a `'static` document in a test.
        let model = document::fixture::database();
        let built = document::context(&model).expect("the fixture model is coherent");
        let leaked: &'static _ = Box::leak(Box::new(built.to_static()));

        same(
            &database(Served::leaked(leaked)),
            &Value::from_serialize(&built),
            "leaked",
        );
    }

    #[test]
    fn a_member_read_twice_is_the_same_value() {
        // `is sameas` over the whole conversion answers `true` for a member
        // read twice, because both reads clone one value; the cache keeps it
        // so.
        let model = document::fixture::database();
        let built = document::context(&model).expect("the fixture model is coherent");
        let lazy = database(Served::borrowed(&built));
        let first = lazy.get_attr("tables").expect("the collection is readable");
        let second = lazy.get_attr("tables").expect("the collection is readable");

        assert!(is_sameas(&first, &second));

        let table = first.get_item_by_index(0).expect("the fixture has a table");
        let again = second
            .get_item_by_index(0)
            .expect("the fixture has a table");

        assert!(is_sameas(&table, &again));
        assert!(is_sameas(
            &table.get_attr("columns").expect("a table has columns"),
            &again.get_attr("columns").expect("a table has columns"),
        ));
    }

    /// A store of the entry `shop` in `scratch`, written from
    /// [`document::fixture::whole`], whose three collections are recorded
    /// whole.
    fn written(scratch: &Scratch) -> Cache {
        let model = document::fixture::whole();
        let built = document::context(&model).expect("the fixture model is coherent");
        let cache = Cache::of(&scratch.path(".tpl"), "shop");
        cache.write(&built, Covered::Everything);

        cache
    }

    /// The store `cache` serves a render from, under `ending`.
    fn opened(cache: &Cache, ending: Ending) -> Arc<Store> {
        let listed = cache.shelved().expect("every collection is recorded whole");

        Arc::new(Store::open(listed, ending).expect("database.json decodes"))
    }

    #[test]
    fn fr_cache_038_a_shelved_database_answers_as_an_up_front_read_of_the_same_files() {
        // FR-CACHE-038: the same members, names and order, and the same value
        // under every rendering — `json` is the serialisation `same` compares
        // — against the document `Cache::everything` decodes from the same
        // files, on both endings. The two routines share a name, which is the
        // one tie the order has to keep as the up-front read keeps it.
        let scratch = Scratch::new();
        let cache = written(&scratch);
        let up_front = cache.everything().expect("the store is whole");
        let expected = up_front.document().expect("every file decodes");

        assert_eq!(expected.routines.len(), 2, "the tie is in the store");

        for ending in [Ending::Caller, Ending::Process] {
            let store = opened(&cache, ending);

            same(
                &shelved(&store),
                &Value::from_serialize(&expected),
                &format!("{ending:?}"),
            );
            assert!(!store.missed(), "{ending:?}: nothing was a miss");
        }
    }

    #[test]
    fn fr_cache_038_no_object_file_is_read_before_the_template_reaches_it() {
        // FR-CACHE-038 and FR-CACHE-033: the listing is read up front and the
        // files are not. Every object file is removed after the store is
        // opened, and the name, the length and the database's own members are
        // still served; only reaching a member reads its file, and finds the
        // miss FR-CACHE-039 abandons the render on.
        let scratch = Scratch::new();
        let cache = written(&scratch);
        let store = opened(&cache, Ending::Caller);

        for collection in ["tables", "views", "routines"] {
            std::fs::remove_dir_all(scratch.path(&format!(".tpl/.cache/shop/{collection}")))
                .expect("the store is ours");
        }

        let lazy = shelved(&store);
        let tables = lazy.get_attr("tables").expect("the collection is readable");

        assert_eq!(
            lazy.get_attr("name").expect("readable").as_str(),
            Some("freight")
        );
        assert_eq!(tables.len(), Some(3));
        assert_eq!(lazy.get_attr("routines").expect("readable").len(), Some(2));
        assert!(!store.missed(), "no member has been reached");

        let reached = tables.get_item_by_index(0).expect("an index answers");

        assert_eq!(reached.kind(), ValueKind::Invalid);
        assert!(store.missed(), "FR-CACHE-039: the reached file is a miss");
    }

    #[test]
    fn fr_cache_033_a_damaged_file_is_a_miss_when_reached_and_not_before() {
        // FR-CACHE-033 as the fortieth edition amended it: a damaged file the
        // render never reaches is not a miss, and one it reaches is — and the
        // members read before it are served as they were.
        let scratch = Scratch::new();
        let cache = written(&scratch);
        let damaged = scratch.path(".tpl/.cache/shop/tables/consignment_leg.json");
        std::fs::write(&damaged, "{ torn").expect("the store is ours");

        let store = opened(&cache, Ending::Caller);
        let tables = shelved(&store)
            .get_attr("tables")
            .expect("the collection is readable");

        for index in 0..2 {
            let table = tables.get_item_by_index(index).expect("an index answers");
            assert_eq!(table.kind(), ValueKind::Map, "{index}");
        }
        assert!(!store.missed(), "the damaged file has not been reached");

        assert_eq!(
            tables
                .get_item_by_index(2)
                .expect("an index answers")
                .kind(),
            ValueKind::Invalid
        );
        assert!(store.missed());
    }

    #[test]
    fn a_file_that_names_another_object_than_its_path_is_a_miss_when_reached() {
        // The collection is named and ordered by the paths, so a file whose
        // contents name another member would be served out of that order.
        // No write of this binary produces one; a hand that does is refused.
        let scratch = Scratch::new();
        let cache = written(&scratch);
        let carrier = scratch.path(".tpl/.cache/shop/tables/carrier.json");
        let renamed = std::fs::read_to_string(&carrier)
            .expect("the store is ours")
            .replacen(r#""name":"carrier""#, r#""name":"zz_carrier""#, 1);
        std::fs::write(&carrier, renamed).expect("the store is ours");

        let store = opened(&cache, Ending::Caller);
        let first = shelved(&store)
            .get_attr("tables")
            .expect("the collection is readable")
            .get_item_by_index(0)
            .expect("an index answers");

        assert_eq!(first.kind(), ValueKind::Invalid);
        assert!(store.missed());
    }

    #[test]
    fn a_member_is_read_once_and_the_value_is_the_same_value() {
        // A member reached twice is served from its one read, so a file
        // changed between two reaches is not read again, and `is sameas`
        // answers as it answers over the whole conversion.
        let scratch = Scratch::new();
        let cache = written(&scratch);
        let store = opened(&cache, Ending::Caller);
        let lazy = shelved(&store);
        let first = lazy
            .get_attr("views")
            .expect("readable")
            .get_item_by_index(0)
            .expect("an index answers");

        std::fs::remove_dir_all(scratch.path(".tpl/.cache/shop/views")).expect("ours");

        let again = lazy
            .get_attr("views")
            .expect("readable")
            .get_item_by_index(0)
            .expect("an index answers");

        assert!(is_sameas(&first, &again));
        assert!(!store.missed());
        assert!(Arc::ptr_eq(
            &store.view(0).expect("read once"),
            &store.view(0).expect("read once"),
        ));
    }
}
