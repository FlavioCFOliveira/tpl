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
use std::sync::{Arc, OnceLock};

use minijinja::Value;
use minijinja::value::{Enumerator, Object, ObjectRepr};

use crate::cli::source::Served;
use crate::model::ToStatic;
use crate::model::document::DatabaseDocument;

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

/// The `database` variable, over `document` (`FR-RND-023`).
pub(super) fn database(document: Served<'_, '_>) -> Value {
    let document = match document.leaked_document() {
        Some(leaked) => Held::Leaked(leaked),
        None => Held::Copied(Arc::new(document.to_static())),
    };

    Value::from_object(Database {
        document,
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

/// The `database` object: seven members, each converted when first read.
struct Database {
    /// The document every member is read from.
    document: Held,

    /// The converted members, by position in [`DATABASE_KEYS`].
    members: [OnceLock<Value>; DATABASE_KEYS.len()],
}

impl Database {
    /// The member at `position` of [`DATABASE_KEYS`], converted.
    fn convert(&self, position: usize) -> Value {
        let document = &self.document;

        match position {
            0 => Value::from_serialize(&document.name),
            1 => Value::from_serialize(&document.charset),
            2 => Value::from_serialize(&document.collation),
            3 => Value::from_serialize(&document.server),
            4 => Members::value(document, Collection::Tables, document.tables.len()),
            5 => Members::value(document, Collection::Views, document.views.len()),
            _ => Members::value(document, Collection::Routines, document.routines.len()),
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
            .field("name", &self.document.name)
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
    /// The document the collection is read from.
    document: Held,

    /// Which collection this is.
    collection: Collection,

    /// The converted members, by position.
    members: Box<[OnceLock<Value>]>,
}

impl Members {
    /// The collection as a value, holding `len` members not yet converted.
    fn value(document: &Held, collection: Collection, len: usize) -> Value {
        Value::from_object(Self {
            document: document.clone(),
            collection,
            members: (0..len).map(|_| OnceLock::new()).collect(),
        })
    }

    /// The member at `index`, converted.
    fn convert(&self, index: usize) -> Value {
        match self.collection {
            Collection::Tables => Value::from_object(Table {
                document: self.document.clone(),
                index,
                members: Default::default(),
            }),
            Collection::Views => Value::from_serialize(&self.document.views[index]),
            Collection::Routines => Value::from_serialize(&self.document.routines[index]),
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

/// One member of `database.tables`: the table's fields, each converted when
/// first read.
struct Table {
    /// The document the table is read from.
    document: Held,

    /// The table's position in `database.tables`.
    index: usize,

    /// The converted fields, by position in [`TABLE_KEYS`].
    members: [OnceLock<Value>; TABLE_KEYS.len()],
}

impl Table {
    /// How many of [`TABLE_KEYS`] this table carries: all of them where it
    /// carries the `restricted` marking, and all but that one otherwise.
    fn carried(&self) -> usize {
        if self.document.tables[self.index].restricted.is_some() {
            TABLE_KEYS.len()
        } else {
            RESTRICTED
        }
    }

    /// The field at `position` of [`TABLE_KEYS`], converted.
    fn convert(&self, position: usize) -> Value {
        let table = &self.document.tables[self.index];

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
            .field("name", &self.document.tables[self.index].name)
            .finish_non_exhaustive()
    }
}

#[cfg(test)]
mod tests {
    use super::database;
    use crate::cli::source::Served;
    use crate::model::ToStatic;
    use crate::model::document;
    use crate::model::restricted::Restricted;
    use minijinja::Value;
    use minijinja::tests::is_sameas;
    use minijinja::value::ValueKind;
    use std::borrow::Cow;

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
}
