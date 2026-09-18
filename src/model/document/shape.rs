//! The shape of the document: its types, and the key order they state
//! (`FR-CTX-031` … `FR-CTX-036`, `FR-OUT-013`, `OD-18`).
//!
//! Every type here derives [`Serialize`] and [`Deserialize`], so a struct's
//! **field declaration order is the emitted key order** and the order is stated
//! once, in the type, where it cannot drift from the type. `preserve_order` is
//! off and `indexmap` is not in the dependency graph, which is how `FR-OUT-013`
//! is met where it forbids an unordered map on the emitting path.
//!
//! Most of the document is the model's own types, serialised directly: a
//! column, an index, a trigger, a `CHECK` constraint, a view, a routine, the
//! decomposed type, the discriminated default, the server and the `restricted`
//! marking each state their own keys in their own module. Only four shapes
//! differ from the model, and every one of them differs for the same reason —
//! the **embedding** of `FR-CTX-006` and `FR-CTX-010`, which the model carries
//! as a name and the document carries as an object.
//!
//! # One hop, then names, and the type system is what terminates it
//!
//! `FR-CTX-009` makes the cut of `FR-CTX-008` one rule applied at the first
//! hop, in both directions, whatever the shape of the reference graph. Here the
//! rule is a **type parameter**:
//!
//! | Alias | What a reference resolves to |
//! |---|---|
//! | [`TableDocument`] | an [`EmbeddedTable`], one hop deep |
//! | [`EmbeddedTable`] | a [`Cow<'a, str>`](Cow) — the name, and nothing further |
//!
//! [`TableShape`] is written once and instantiated twice. An embedded table's
//! two reference collections hold strings, and a string cannot carry a table,
//! so a traversal terminates **by construction**: there is no depth counter to
//! get wrong, no visited set to maintain, and no cycle a value could express.
//! The three shapes `FR-CTX-009` enumerates — `A` and `B` referencing each
//! other, `A` referenced by `B`, and `A` referencing itself — are all the same
//! one hop under this rule.
//!
//! `FR-CTX-007` is what the embedded table still carries: its columns, its
//! indexes and its primary key in full. The cut reaches the two reference
//! collections and nothing else, which is also why `BR-CTX-001`'s rejected
//! *summary embedding* — a name, a primary key and column names only — is not
//! what this is.

use std::borrow::Cow;

use serde::{Deserialize, Serialize};

use super::order::Named;
use crate::model::check_constraint::CheckConstraint;
use crate::model::column::Column;
use crate::model::foreign_key::{ForeignKeyColumn, ReferentialAction};
use crate::model::index::Index;
use crate::model::restricted::Restricted;
use crate::model::routine::Routine;
use crate::model::server::Server;
use crate::model::table::TableType;
use crate::model::trigger::Trigger;
use crate::model::view::View;

/// The `data` of `tpl schema dump`, and what `--context` is handed
/// (`FR-SCH-017`, `FR-SCH-034`, `FR-OUT-031`).
///
/// ```json
/// {"schema_version":1,"source":"server","data":{"database":{…}}}
/// ```
///
/// One key, named for the kind in the singular, whose value is the whole model
/// of the selected database. The envelope around it is
/// [`Document`](crate::output::Document) and is not restated here.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct ContextData<'a> {
    /// The whole model of the selected database.
    #[serde(borrow)]
    pub(super) database: DatabaseDocument<'a>,
}

/// The `database` object (`FR-CTX-035`, `FR-CTX-036`).
///
/// ```json
/// {"name":"freight","charset":"utf8mb4","collation":"utf8mb4_unicode_520_ci","server":{…},"tables":[…],"views":[…],"routines":[…]}
/// ```
///
/// Three metadata fields, the `server` object, and the three collections — in
/// the order `FR-CTX-036` writes them, which is therefore the order of the
/// fields below.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct DatabaseDocument<'a> {
    /// The schema's name.
    #[serde(borrow)]
    pub(super) name: Cow<'a, str>,

    /// The schema's default character set.
    #[serde(borrow)]
    pub(super) charset: Cow<'a, str>,

    /// The schema's default collation.
    #[serde(borrow)]
    pub(super) collation: Cow<'a, str>,

    /// The server the read was made against (`FR-CTX-031`).
    #[serde(borrow)]
    pub(super) server: Server<'a>,

    /// The tables of `FR-CAT-001`, ordered by name (`NFR-DET-002`).
    #[serde(borrow)]
    pub(super) tables: Vec<TableDocument<'a>>,

    /// The views of `FR-CAT-007`, ordered by name.
    #[serde(borrow)]
    pub(super) views: Cow<'a, [View<'a>]>,

    /// The routines of `FR-CAT-008`, ordered by name.
    #[serde(borrow)]
    pub(super) routines: Cow<'a, [Routine<'a>]>,
}

/// A table at the first hop: its references are embedded objects.
pub(crate) type TableDocument<'a> = TableShape<'a, OutgoingKey<'a>, IncomingKey<'a>>;

/// A table one hop in: its references are names, per `FR-CTX-008`.
///
/// This alias is where a traversal stops. Both reference collections hold
/// [`Cow<'a, str>`](Cow), which carries no table, so no value of this type can
/// lead anywhere.
pub(crate) type EmbeddedTable<'a> = TableShape<'a, Cow<'a, str>, Cow<'a, str>>;

/// One table of the document, parameterised by what a reference resolves to
/// (`FR-CAT-009` … `FR-CAT-015`, `FR-CTX-006` … `FR-CTX-010`).
///
/// `F` is what an outgoing foreign key resolves to and `I` an incoming one. The
/// two aliases above are the only two instantiations, and they are the two
/// sides of the one rule of `FR-CTX-009`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct TableShape<'a, F, I> {
    /// The table's name, returned unescaped.
    #[serde(borrow)]
    pub(super) name: Cow<'a, str>,

    /// Which of the two covered types it is (`FR-CAT-002`).
    pub(super) table_type: TableType,

    /// The storage engine (`FR-SCH-009`).
    #[serde(borrow)]
    pub(super) engine: Option<Cow<'a, str>>,

    /// The table's collation (`FR-SCH-009`). There is no character set beside
    /// it: the table catalogue carries none on any of the four series.
    #[serde(borrow)]
    pub(super) collation: Option<Cow<'a, str>>,

    /// The comment, the empty string where none was given (`FR-CAT-039`).
    #[serde(borrow)]
    pub(super) comment: Cow<'a, str>,

    /// The columns, in **ordinal position** order — an exception of
    /// `NFR-DET-002` (`FR-CAT-009`).
    #[serde(borrow)]
    pub(super) columns: Cow<'a, [Column<'a>]>,

    /// The indexes, ordered by name, the primary key among them
    /// (`FR-CAT-010`).
    #[serde(borrow)]
    pub(super) indexes: Cow<'a, [Index<'a>]>,

    /// The primary key — the index named `PRIMARY` (`FR-CAT-011`,
    /// `FR-CAT-043`), or `null` where the table has none.
    ///
    /// It is a projection of [`TableShape::indexes`] and not a second source.
    /// The read-back path takes the primary key from the index collection, as
    /// `FR-CAT-043` requires, and does not read this key: presenting it is what
    /// spares a template a match on a name, and reading it back would give the
    /// model a second place the two could disagree from.
    #[serde(borrow)]
    pub(super) primary_key: Option<Index<'a>>,

    /// The outgoing foreign keys, ordered by name (`FR-CAT-012`).
    pub(super) foreign_keys: Vec<F>,

    /// The incoming foreign keys (`FR-CAT-013`), ordered by the referencing
    /// table's name and then by the constraint's.
    pub(super) referenced_by: Vec<I>,

    /// The triggers, ordered by name (`FR-CAT-014`).
    #[serde(borrow)]
    pub(super) triggers: Cow<'a, [Trigger<'a>]>,

    /// The `CHECK` constraints, ordered by name (`FR-CAT-015`).
    #[serde(borrow)]
    pub(super) check_constraints: Cow<'a, [CheckConstraint<'a>]>,

    /// The properties that could not be read (`FR-PRIV-005` … `FR-PRIV-007`).
    ///
    /// The key is absent rather than `null` on a complete table, which is the
    /// one exception `OD-18` admits to `FR-OUT-012`.
    #[serde(borrow, default, skip_serializing_if = "Option::is_none")]
    pub(super) restricted: Option<Restricted<'a>>,
}

impl<F, I> Named for TableShape<'_, F, I> {
    fn name(&self) -> &str {
        &self.name
    }
}

/// A foreign key at the first hop: the referenced table is embedded in full,
/// per `FR-CTX-006` and `FR-CTX-007`.
pub(crate) type OutgoingKey<'a> = ForeignKeyShape<'a, EmbeddedTable<'a>>;

/// A foreign key whose referenced table is a name.
///
/// It is the key carried inside an [`IncomingKey`], where the referenced table
/// is the table carrying `referenced_by` — already at hand, and one hop further
/// than `FR-CTX-009` admits.
pub(crate) type NamedKey<'a> = ForeignKeyShape<'a, Cow<'a, str>>;

/// One foreign key of the document, parameterised by what its referenced table
/// resolves to (`FR-CAT-045`).
///
/// `columns` is one list of **pairs** and not two parallel lists, exactly as
/// the model carries it: `NFR-DET-002` excepts the pairing because it is
/// positional, and a shape with one list has nothing for a second to drift
/// from.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct ForeignKeyShape<'a, R> {
    /// The constraint name the DDL gave.
    #[serde(borrow)]
    pub(super) name: Cow<'a, str>,

    /// The referencing and referenced columns, **paired by position** and in
    /// the order the catalogue states — an exception of `NFR-DET-002`.
    #[serde(borrow)]
    pub(super) columns: Cow<'a, [ForeignKeyColumn<'a>]>,

    /// The referenced table: an object at the first hop, a name beyond it.
    pub(super) referenced_table: R,

    /// The key on the referenced table the foreign key points at.
    #[serde(borrow)]
    pub(super) referenced_key: Cow<'a, str>,

    /// The match option, carried verbatim.
    #[serde(borrow)]
    pub(super) match_option: Cow<'a, str>,

    /// The `ON UPDATE` rule.
    pub(super) on_update: ReferentialAction,

    /// The `ON DELETE` rule.
    pub(super) on_delete: ReferentialAction,
}

impl<R> Named for ForeignKeyShape<'_, R> {
    fn name(&self) -> &str {
        &self.name
    }
}

/// A foreign key seen from the table it references (`FR-CAT-013`,
/// `FR-CTX-010`).
///
/// ```json
/// {"table":{…the referencing table, embedded…},"key":{…}}
/// ```
///
/// The embedded table and the embedded table of an outgoing key are **two first
/// hops, not one path of length two**, which is the reading `FR-CTX-009`'s
/// fifth-edition amendment removed the ambiguity from. The key beside it
/// carries the referenced table as a name, because that table is the one
/// carrying this collection.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct IncomingKey<'a> {
    /// The referencing table, embedded one level deep.
    #[serde(borrow)]
    pub(super) table: EmbeddedTable<'a>,

    /// The key itself, as the referencing table carries it.
    #[serde(borrow)]
    pub(super) key: NamedKey<'a>,
}
