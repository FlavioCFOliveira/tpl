//! The fold: the rows the repertoire returned become the model
//! (`FR-CAT-009` … `FR-CAT-015`, `FR-CAT-042` … `FR-CAT-053`, `FR-CTX-036`).
//!
//! Every statement of [`super::statements`] reads **one** catalogue table, so
//! every relation between two object kinds is made here rather than by the
//! server. That is what `NFR-PERF-001` buys: a join per table would be a
//! statement per table.
//!
//! | What the fold makes | From | Fixed by |
//! |---|---|---|
//! | A column, an index, a key, a constraint or a trigger on its table | The table name each row carries | `FR-CAT-009` … `FR-CAT-015` |
//! | One index from its one-row-per-column form | Adjacent rows of one index name, in the order the catalogue states | `FR-CAT-010` |
//! | The primary key | The index named `PRIMARY`, in that same collection | `FR-CAT-011`, `FR-CAT-043` |
//! | A foreign key | The rules row joined to its key-column rows by name | `FR-CAT-045` |
//! | The incoming direction | The same rows, read from the referenced end | `FR-CAT-013`, `FR-CAT-045` |
//! | A routine's parameters | The routine name **and kind**, because a procedure and a function may share a name | `FR-CAT-018`, `FR-SCH-008` |
//!
//! # The coverage filter, and where it is applied
//!
//! [`TableType::from_catalogue`] is the coverage predicate of `FR-CAT-001`
//! through `FR-CAT-006`, and it is applied to the rows of the table read: a
//! row whose type is `VIEW`, `SEQUENCE`, `SYSTEM VIEW` or `TEMPORARY` yields
//! no table. `FR-CAT-032` requires exactly that — a filter on the table type,
//! not a reliance on the catalogue omitting anything, which only `10.11` does
//! for a temporary table.
//!
//! **Every other read is filtered by the same predicate, once removed.** A row
//! of any member read names the object it hangs from, and the fold attaches it
//! to the table of that name or presents it nowhere. That is `FR-CAT-052` for
//! the column read, stated over the owning object rather than over the column:
//! the fixture's sequence carries eight rows in the column catalogue, and they
//! reach no table because no table of that name was covered. It is the same
//! filter for the index, key, constraint and trigger reads, which were
//! observed to carry no row for an uncovered object on any of the four series
//! and which are filtered all the same, for the reason `FR-CAT-032` gives.
//!
//! *Rejected: restricting the member reads in SQL, by joining each to the
//! table catalogue on the covered types.* It states the coverage predicate a
//! second time, in a second language, where the two can drift apart — and the
//! project's own fixture records why that matters, in as many words: a second
//! copy of a value is a second thing that can be wrong. It also turns every
//! member read into a self-join of `INFORMATION_SCHEMA`, whose cost is not
//! bounded by anything this reader controls. The price paid instead is the
//! rows an uncovered object contributes to the column read — 52 of the
//! fixture's 301 — which are decoded by nothing, because the fold drops each
//! before it reads a second field.
//!
//! # Where the fold refuses
//!
//! Five catalogue fields are closed enumerations that a requirement fixes from
//! an observation of all four series: a referential action (`FR-CAT-045`), a
//! constraint level (`FR-CAT-046`), a trigger event and a trigger timing
//! (`FR-CAT-050`), and a routine kind (`FR-CAT-016`). The model has a variant
//! for each observed value and for no other, so a value outside one of those
//! sets is a fact the model cannot carry.
//!
//! The fold reports it as [`Error::InternalInvariant`], `70`. The two
//! alternatives are worse in the same way: substituting a variant puts a value
//! in the document that the server did not state, and dropping the object
//! presents a table without a foreign key it has — both at exit `0`, which is
//! the silent corruption `NFR-DET-002`'s own amendment calls the class of
//! failure this corpus works hardest to prevent. A `70` says *`tpl` cannot
//! represent what this server returned*, which is what happened.
//!
//! **Its one foreseeable trigger is a server newer than the window.**
//! `FR-SRV-031` reads such a server and marks it, so a series that added a
//! sixth referential action would be read up to the row that carries it and
//! then refused. The cost is recorded here rather than discovered: the
//! alternative is a document that silently disagrees with the server.
//!
//! # Where the fold takes the completeness verdict
//!
//! The three checks of [`super::completeness`] are made **here**, on the row,
//! as the model is built — never on the finished model. Two of the three would
//! be wrong if they were made later: a routine's body reaches the model as the
//! empty string whether the catalogue returned SQL `NULL` or an empty body, and
//! a foreign key whose rules row is missing reaches the model as no foreign key
//! at all, which is indistinguishable from a table that declares none.
//!
//! | Where | Check | Marks | Fixed by |
//! |---|---|---|---|
//! | [`views`] | The definition is the empty string | That view, with `definition` | `FR-PRIV-011` |
//! | [`routine`] | The body is SQL `NULL` | That routine, with `body` | `FR-PRIV-017` |
//! | [`foreign_keys`] | A key column has no referential-constraint row | The referencing table with `foreign_keys`, and the referenced one with `referenced_by` | `FR-PRIV-019` |
//!
//! Because the fold is the one path every read takes, `FR-PRIV-012` holds for
//! the narrowed reads and for the dump without a call site of their own. No
//! other object kind is cross-checked, per `FR-PRIV-015`, and no marking this
//! fold writes ever names `triggers`, per `FR-PRIV-020`.

use std::borrow::Cow;
use std::collections::BTreeMap;
use std::panic::Location;

use sqlx::mysql::MySqlRow;

use super::Catalogue;
use super::completeness::{self, Marking, Property};
use super::row::{count, maybe_count, maybe_signed, maybe_text, signed, text, text_or_empty};
use super::statements::Read;
use crate::error::Error;
use crate::model::check_constraint::{CheckConstraint, ConstraintLevel};
use crate::model::column::{Column, Generated, GeneratedStorage};
use crate::model::column_default::ColumnDefault;
use crate::model::column_type::{CatalogueType, ColumnType};
use crate::model::database::Database;
use crate::model::document::order;
use crate::model::foreign_key::{
    ForeignKey, ForeignKeyColumn, IncomingForeignKey, ReferentialAction,
};
use crate::model::index::{Index, IndexColumn, SortDirection};
use crate::model::routine::{Routine, RoutineKind, RoutineParameter};
use crate::model::table::{Table, TableParts, TableType};
use crate::model::trigger::{Trigger, TriggerEvent, TriggerTiming};
use crate::model::view::View;

/// The value the column attribute field of `FR-CAT-041` takes for an
/// auto-incremental column. Lower case, with an underscore.
const AUTO_INCREMENT: &str = "auto_increment";

/// The value it takes for an invisible column (`FR-CAT-035`). Upper case.
const INVISIBLE: &str = "INVISIBLE";

/// What it begins with where the column carries an `ON UPDATE` default
/// (`FR-CAT-041`). The whole field is carried, per `FR-CTX-020`.
const ON_UPDATE: &str = "on update ";

/// The value the index catalogue's ignored field takes for an index that is
/// not ignored (`FR-CAT-042`).
const NOT_IGNORED: &str = "NO";

/// The value a `YES`/`NO` field takes for yes.
const YES: &str = "YES";

/// The invariant an absent schema row violates.
const SCHEMA_ROW: &str = "the schema a connection selected has a row in the schema catalogue";

/// The invariant a catalogue value outside a closed enumeration violates.
const ENUMERATED: &str =
    "every enumerated catalogue value is one the requirement that fixes it records";

/// The invariant a key naming a column its table does not carry violates
/// (`FR-CAT-044`).
const KEY_NAMES_A_CARRIED_COLUMN: &str =
    "a key the catalogue reports names a column the same table's column list carries";

/// The condition a fact the model cannot represent produces.
const fn refused(invariant: &'static str, location: &'static Location<'static>) -> Error {
    Error::InternalInvariant {
        invariant,
        location,
    }
}

/// Whether a `YES`/`NO` field says yes.
fn yes(row: &MySqlRow, field: &str) -> Result<bool, Error> {
    Ok(text(row, field)? == YES)
}

/// A lookup from a catalogue key to the position of the object it names.
///
/// The entries are sorted once and answered by binary search, which is what
/// the document's own `order::pointers_by_name` does for the same reason: a
/// scan per row would make the fold quadratic in the number of objects, and
/// `NFR-PERF-001` is not the only place this reader must not scale with the
/// size of the database.
///
/// The order is [`Ord`]'s, which for `&str` and for a tuple of them is the
/// byte-wise comparison `NFR-DET-002` fixes. It is a lookup and not a
/// presented collection, so it goes through [`Ord`] rather than through
/// `order::compare`; nothing a caller sees is ordered by it.
struct Positions<K: Ord + Copy> {
    /// The key of each object, with its position in the collection.
    entries: Vec<(K, usize)>,
}

impl<K: Ord + Copy> Positions<K> {
    /// The lookup `entries` describes.
    fn of(mut entries: Vec<(K, usize)>) -> Self {
        entries.sort_unstable_by_key(|entry| entry.0);

        Self { entries }
    }

    /// Where the object `key` names stands, or [`None`] where the fold carries
    /// no object of that key.
    fn find(&self, key: K) -> Option<usize> {
        self.entries
            .binary_search_by(|(carried, _)| carried.cmp(&key))
            .ok()
            .and_then(|at| self.entries.get(at))
            .map(|(_, position)| *position)
    }
}

/// The covered tables, and the lookup that attaches a member row to one.
struct Tables<'a> {
    /// The tables, in the order the table read returned them.
    parts: Vec<TableParts<'a>>,

    /// Where each table stands, by name.
    positions: Positions<&'a str>,
}

impl<'a> Tables<'a> {
    /// Where the table `name` names stands, or [`None`] where it is not
    /// covered.
    ///
    /// [`None`] is the coverage filter of `FR-CAT-052` seen from the member
    /// side: a row whose owning object the model does not carry is presented
    /// nowhere. It is answered before the rest of the row is decoded, so an
    /// uncovered object's rows cost one lookup each and nothing more.
    fn position(&self, name: &str) -> Option<usize> {
        self.positions.find(name)
    }

    /// The table at `position`, which [`Tables::position`] answered.
    fn at(&mut self, position: usize) -> Option<&mut TableParts<'a>> {
        self.parts.get_mut(position)
    }
}

/// A foreign key's rules row, with the columns the key-column read adds to it.
struct Rule<'a> {
    /// The referencing table — the table `FR-CAT-012` carries the key on.
    table: &'a str,

    /// The referenced table — the table `FR-CAT-013` carries it on, from the
    /// other end.
    referenced_table: &'a str,

    /// The key itself, whose column list the second read fills.
    key: ForeignKey<'a>,
}

/// The model the fetched rows describe (`FR-CTX-035`, `FR-CTX-036`).
///
/// # Errors
///
/// Returns [`Error::InternalInvariant`] where the catalogue returned a value
/// the model cannot represent — a decode that failed, an enumerated value
/// outside the set its requirement fixes, a key naming a column its table does
/// not carry (`FR-CAT-044`), or no row at all in the schema catalogue. Every
/// one of them is a `70` for the reason [`super::row`] records: the field
/// lists were observed on all four series of `FR-SRV-015`, so a shape outside
/// them is a defect in this reader's model of the catalogue rather than a
/// condition the caller can act on.
pub(super) fn database(catalogue: &Catalogue) -> Result<Database<'_>, Error> {
    let schema = catalogue.rows(Read::Schema);
    let row = schema.first().ok_or_else(|| Error::InternalInvariant {
        invariant: SCHEMA_ROW,
        location: Location::caller(),
    })?;

    let mut tables = tables(catalogue.rows(Read::Tables))?;

    columns(catalogue.rows(Read::Columns), &mut tables)?;
    indexes(catalogue.rows(Read::Indexes), &mut tables)?;
    foreign_keys(
        catalogue.rows(Read::ForeignKeyRules),
        catalogue.rows(Read::KeyColumns),
        &mut tables,
    )?;
    checks(catalogue.rows(Read::Checks), &mut tables)?;
    triggers(catalogue.rows(Read::Triggers), &mut tables)?;

    Ok(Database {
        name: Cow::Borrowed(text(row, "SCHEMA_NAME")?),
        charset: Cow::Borrowed(text(row, "DEFAULT_CHARACTER_SET_NAME")?),
        collation: Cow::Borrowed(text(row, "DEFAULT_COLLATION_NAME")?),
        server: catalogue.server().clone(),
        tables: assemble(tables)?,
        views: views(catalogue.rows(Read::Views))?,
        routines: routines(
            catalogue.rows(Read::Routines),
            catalogue.rows(Read::Parameters),
        )?,
    })
}

/// The covered tables of the table read (`FR-CAT-001` … `FR-CAT-006`,
/// `FR-CAT-053`).
fn tables(rows: &[MySqlRow]) -> Result<Tables<'_>, Error> {
    let mut parts = Vec::with_capacity(rows.len());
    let mut positions = Vec::with_capacity(rows.len());

    for row in rows {
        let name = text(row, "TABLE_NAME")?;

        // The coverage predicate of FR-CAT-001 through FR-CAT-006, which
        // FR-CAT-032 requires to be a filter on the table type.
        let Some(table_type) = TableType::from_catalogue(text(row, "TABLE_TYPE")?) else {
            continue;
        };

        positions.push((name, parts.len()));
        parts.push(TableParts {
            engine: maybe_text(row, "ENGINE")?.map(Cow::Borrowed),
            collation: maybe_text(row, "TABLE_COLLATION")?.map(Cow::Borrowed),
            comment: Cow::Borrowed(text(row, "TABLE_COMMENT")?),
            ..TableParts::new(Cow::Borrowed(name), table_type)
        });
    }

    Ok(Tables {
        parts,
        positions: Positions::of(positions),
    })
}

/// Attaches every column to its table (`FR-CAT-009`, `FR-CAT-052`).
///
/// The collections are not pre-sized: the row count gives the schema's total
/// and not any one table's, and counting per table would cost a second lookup
/// per row to save a handful of reallocations per table.
fn columns<'a>(rows: &'a [MySqlRow], tables: &mut Tables<'a>) -> Result<(), Error> {
    for row in rows {
        let owner = text(row, "TABLE_NAME")?;

        // FR-CAT-052: a column whose owning object is not covered is not
        // presented, and the rest of its row is never decoded. The fixture's
        // sequence reaches this line eight times and its views 44.
        let Some(at) = tables.position(owner) else {
            continue;
        };
        let column = column(row, owner)?;

        if let Some(table) = tables.at(at) {
            table.columns.push(column);
        }
    }

    Ok(())
}

/// One column (`FR-CAT-009`, `FR-CTX-019` … `FR-CTX-021`, `FR-CAT-041`).
fn column<'a>(row: &'a MySqlRow, owner: &'a str) -> Result<Column<'a>, Error> {
    // FR-CAT-041 fixes the six values this one field takes, and its bounded
    // claim records that no column of the fixture carries two at once — so the
    // whole field is compared, and a column carrying two attributes would
    // match none of them. The claim is the requirement's and is not narrowed
    // here.
    let attribute = text(row, "EXTRA")?;

    Ok(Column {
        name: Cow::Borrowed(text(row, "COLUMN_NAME")?),
        table_name: Cow::Borrowed(owner),
        position: count(row, "ORDINAL_POSITION")?,
        column_type: ColumnType::decompose(&CatalogueType {
            column_type: text(row, "COLUMN_TYPE")?,
            data_type: text(row, "DATA_TYPE")?,
            numeric_precision: maybe_count(row, "NUMERIC_PRECISION")?,
            datetime_precision: maybe_count(row, "DATETIME_PRECISION")?,
            numeric_scale: maybe_count(row, "NUMERIC_SCALE")?,
            character_maximum_length: maybe_count(row, "CHARACTER_MAXIMUM_LENGTH")?,
            charset: maybe_text(row, "CHARACTER_SET_NAME")?,
            collation: maybe_text(row, "COLLATION_NAME")?,
        }),
        nullable: yes(row, "IS_NULLABLE")?,
        default: ColumnDefault::classify(maybe_text(row, "COLUMN_DEFAULT")?),
        comment: Cow::Borrowed(text(row, "COLUMN_COMMENT")?),
        auto_increment: attribute == AUTO_INCREMENT,
        invisible: attribute == INVISIBLE,
        generated: generated(row, attribute)?,
        on_update: attribute
            .starts_with(ON_UPDATE)
            .then_some(Cow::Borrowed(attribute)),
    })
}

/// The generation expression and storage kind of a generated column, or
/// [`None`] for a column that is not one (`FR-CAT-051`).
///
/// The storage kind comes from the attribute field and never from the
/// is-generated field, which `FR-CAT-051` bars in terms: that field says only
/// *whether* a column is generated, never *how*, so the reader does not ask
/// for it.
fn generated<'a>(row: &'a MySqlRow, attribute: &str) -> Result<Option<Generated<'a>>, Error> {
    let Some(storage) = GeneratedStorage::from_attribute(attribute) else {
        return Ok(None);
    };

    Ok(Some(Generated {
        // The field is declared nullable and reads SQL `NULL` on every column
        // that is not generated; on the nine that are, it carries the
        // expression.
        expression: Cow::Borrowed(text_or_empty(row, "GENERATION_EXPRESSION")?),
        storage,
    }))
}

/// Folds the index read into one object per index (`FR-CAT-010`,
/// `FR-CAT-042`).
///
/// The rows arrive ordered by the sequence field, so a member is appended to
/// the index it belongs to in the order the catalogue states. The index is
/// found by its **exact** name rather than by adjacency: the statement's
/// `ORDER BY` runs under the server's own collation, and two names that
/// collate equal would interleave without the fold noticing.
fn indexes<'a>(rows: &'a [MySqlRow], tables: &mut Tables<'a>) -> Result<(), Error> {
    for row in rows {
        let owner = text(row, "TABLE_NAME")?;
        let name = text(row, "INDEX_NAME")?;

        let Some(at) = tables.position(owner) else {
            continue;
        };

        let member = IndexColumn {
            name: Cow::Borrowed(text(row, "COLUMN_NAME")?),
            direction: maybe_text(row, "COLLATION")?.and_then(SortDirection::from_catalogue),
            prefix_length: maybe_signed(row, "SUB_PART")?,
        };
        let carried = tables
            .at(at)
            .and_then(|table| table.indexes.iter().position(|index| index.name == name));

        if let Some(existing) = carried {
            // `position` answered `existing` over the same collection, so the
            // lookup cannot fail; it is written as one anyway, so that this
            // module carries no indexing that could panic.
            if let Some(index) = tables
                .at(at)
                .and_then(|table| table.indexes.get_mut(existing))
            {
                index.columns.push(member);
            }
        } else {
            let opened = index(row, name, member)?;

            if let Some(table) = tables.at(at) {
                table.indexes.push(opened);
            }
        }
    }

    Ok(())
}

/// The index one row opens, carrying that row's column as its first
/// (`FR-CAT-042`).
///
/// The fields read here are the ones the catalogue repeats on every row of one
/// index; the two that differ row by row are the member's, per `FR-CAT-010`.
fn index<'a>(row: &'a MySqlRow, name: &'a str, first: IndexColumn<'a>) -> Result<Index<'a>, Error> {
    Ok(Index {
        name: Cow::Borrowed(name),
        // FR-CAT-042: there is no is-unique field, and the non-unique field is
        // `0` for the primary key and for a unique index.
        unique: signed(row, "NON_UNIQUE")? == 0,
        columns: vec![first],
        index_type: Cow::Borrowed(text(row, "INDEX_TYPE")?),
        comment: Cow::Borrowed(text(row, "INDEX_COMMENT")?),
        ignored: text(row, "IGNORED")? != NOT_IGNORED,
    })
}

/// Attaches both directions of every foreign key (`FR-CAT-012`, `FR-CAT-013`,
/// `FR-CAT-045`).
///
/// The two reads are joined here because neither catalogue table is
/// sufficient: the rules table carries the rules and names no column, and the
/// key-column table carries the columns and no rule.
///
/// That independence is also what `FR-PRIV-019` detects: a privilege that
/// removes the rules table entirely leaves the key-column table whole, so a
/// column that names a referenced table under no rules row is the third shape
/// of `FR-PRIV-018` — zero rows — seen from the only place the catalogue offers
/// a second view of the same population.
fn foreign_keys<'a>(
    rules: &'a [MySqlRow],
    key_columns: &'a [MySqlRow],
    tables: &mut Tables<'a>,
) -> Result<(), Error> {
    let mut carried = Vec::with_capacity(rules.len());
    let mut positions = Vec::with_capacity(rules.len());
    // One marking per table that lost a property, built only where there is a
    // shortfall: the map allocates nothing on a complete read, and its order
    // is its own rather than the rows'.
    let mut unreadable: BTreeMap<&'a str, Marking> = BTreeMap::new();

    for row in rules {
        let table = text(row, "TABLE_NAME")?;
        let name = text(row, "CONSTRAINT_NAME")?;

        positions.push(((table, name), carried.len()));
        carried.push(rule(row, table, name)?);
    }

    let positions = Positions::of(positions);

    for row in key_columns {
        let table = text(row, "TABLE_NAME")?;
        let key = (table, text(row, "CONSTRAINT_NAME")?);
        // FR-PRIV-019's antecedent, read as the field it is stated over. The
        // statement already restricts the read to rows that name a referenced
        // table, per `FR-CAT-045`, so this is the name the other end is marked
        // by rather than a second filter.
        let referenced = maybe_text(row, "REFERENCED_TABLE_NAME")?;
        let pair = ForeignKeyColumn {
            column: Cow::Borrowed(text(row, "COLUMN_NAME")?),
            referenced_column: Cow::Borrowed(text_or_empty(row, "REFERENCED_COLUMN_NAME")?),
        };
        let at = positions.find(key);

        // FR-PRIV-019: the column names a table it references, and no rules row
        // describes the constraint it belongs to. The two observations
        // contradict each other — a column cannot reference a table under no
        // constraint — and one explanation fits. Both ends lose a property,
        // because `FR-CAT-045` presents one key from two of them.
        if let Some(referenced) = referenced
            && completeness::no_row(at)
        {
            unreadable
                .entry(table)
                .or_default()
                .record(Property::ForeignKeys);
            unreadable
                .entry(referenced)
                .or_default()
                .record(Property::ReferencedBy);
        }

        // A column whose rules row is absent is not presented: there is no rule
        // for it to be a column of.
        let Some(rule) = at.and_then(|at| carried.get_mut(at)) else {
            continue;
        };

        rule.key.columns.push(pair);
    }

    mark(unreadable, tables);

    for rule in carried {
        // FR-CAT-013 carries the same key on the referenced table, so the two
        // directions are two values. The clone is the second of them: the
        // model presents one key from two ends, and building it a second time
        // from the rows would allocate the same column list again for no
        // clearer result.
        if let Some(at) = tables.position(rule.referenced_table)
            && let Some(table) = tables.at(at)
        {
            table.referenced_by.push(IncomingForeignKey {
                table: Cow::Borrowed(rule.table),
                key: rule.key.clone(),
            });
        }

        if let Some(at) = tables.position(rule.table)
            && let Some(table) = tables.at(at)
        {
            table.foreign_keys.push(rule.key);
        }
    }

    Ok(())
}

/// Writes each shortfall onto the table it belongs to (`FR-PRIV-005` …
/// `FR-PRIV-007`).
///
/// `FR-PRIV-006` requires the marking to be **per object**, so each table is
/// given the properties it lost and no table is given another's. A table the
/// coverage filter of `FR-CAT-052` did not carry is marked nowhere, because it
/// is presented nowhere.
///
/// This is the only writer of a table's marking, so the assignment replaces
/// nothing: `FR-PRIV-015` admits no second cross-check for a table, and the
/// three other member reads mark nothing at all.
fn mark(unreadable: BTreeMap<&str, Marking>, tables: &mut Tables<'_>) {
    for (name, marking) in unreadable {
        if let Some(at) = tables.position(name)
            && let Some(table) = tables.at(at)
        {
            table.restricted = marking.sealed();
        }
    }
}

/// One foreign key's rules row, with an empty column list (`FR-CAT-045`).
fn rule<'a>(row: &'a MySqlRow, table: &'a str, name: &'a str) -> Result<Rule<'a>, Error> {
    let location = Location::caller();
    let action = |field: &str| {
        ReferentialAction::from_catalogue(text(row, field)?)
            .ok_or_else(|| refused(ENUMERATED, location))
    };
    let referenced_table = text_or_empty(row, "REFERENCED_TABLE_NAME")?;

    Ok(Rule {
        table,
        referenced_table,
        key: ForeignKey {
            name: Cow::Borrowed(name),
            columns: Vec::new(),
            referenced_table: Cow::Borrowed(referenced_table),
            referenced_key: Cow::Borrowed(text_or_empty(row, "UNIQUE_CONSTRAINT_NAME")?),
            match_option: Cow::Borrowed(text(row, "MATCH_OPTION")?),
            on_update: action("UPDATE_RULE")?,
            on_delete: action("DELETE_RULE")?,
        },
    })
}

/// Attaches every `CHECK` constraint to its table (`FR-CAT-015`,
/// `FR-CAT-046`).
fn checks<'a>(rows: &'a [MySqlRow], tables: &mut Tables<'a>) -> Result<(), Error> {
    for row in rows {
        let owner = text(row, "TABLE_NAME")?;

        let Some(at) = tables.position(owner) else {
            continue;
        };
        let constraint = CheckConstraint {
            name: Cow::Borrowed(text(row, "CONSTRAINT_NAME")?),
            level: ConstraintLevel::from_catalogue(text(row, "LEVEL")?)
                .ok_or_else(|| refused(ENUMERATED, Location::caller()))?,
            clause: Cow::Borrowed(text(row, "CHECK_CLAUSE")?),
        };

        if let Some(table) = tables.at(at) {
            table.check_constraints.push(constraint);
        }
    }

    Ok(())
}

/// Attaches every trigger to its table (`FR-CAT-014`, `FR-CAT-050`).
fn triggers<'a>(rows: &'a [MySqlRow], tables: &mut Tables<'a>) -> Result<(), Error> {
    for row in rows {
        let owner = text(row, "EVENT_OBJECT_TABLE")?;

        let Some(at) = tables.position(owner) else {
            continue;
        };
        let trigger = trigger(row)?;

        if let Some(table) = tables.at(at) {
            table.triggers.push(trigger);
        }
    }

    Ok(())
}

/// One trigger (`FR-CAT-050`).
fn trigger(row: &MySqlRow) -> Result<Trigger<'_>, Error> {
    let location = Location::caller();

    Ok(Trigger {
        name: Cow::Borrowed(text(row, "TRIGGER_NAME")?),
        event: TriggerEvent::from_catalogue(text(row, "EVENT_MANIPULATION")?)
            .ok_or_else(|| refused(ENUMERATED, location))?,
        timing: TriggerTiming::from_catalogue(text(row, "ACTION_TIMING")?)
            .ok_or_else(|| refused(ENUMERATED, location))?,
        action_order: signed(row, "ACTION_ORDER")?,
        statement: Cow::Borrowed(text_or_empty(row, "ACTION_STATEMENT")?),
        orientation: Cow::Borrowed(text(row, "ACTION_ORIENTATION")?),
        old_row_alias: Cow::Borrowed(text(row, "ACTION_REFERENCE_OLD_ROW")?),
        new_row_alias: Cow::Borrowed(text(row, "ACTION_REFERENCE_NEW_ROW")?),
        sql_mode: Cow::Borrowed(text(row, "SQL_MODE")?),
        definer: Cow::Borrowed(text_or_empty(row, "DEFINER")?),
        character_set_client: Cow::Borrowed(text(row, "CHARACTER_SET_CLIENT")?),
        collation_connection: Cow::Borrowed(text(row, "COLLATION_CONNECTION")?),
        database_collation: Cow::Borrowed(text(row, "DATABASE_COLLATION")?),
    })
}

/// Orders every collection and assembles each table (`NFR-DET-002`,
/// `FR-CAT-044`).
///
/// The five collections ordered here are the ones the **default** rule of
/// `NFR-DET-002` governs — name, ascending, byte-wise. A table's columns are
/// not among them: their order is the ordinal position, which the statement's
/// own `ORDER BY` already produced, and it is one of the six exceptions the
/// requirement names. Neither is an index's column list, a foreign key's, or a
/// routine's parameters.
fn assemble(tables: Tables<'_>) -> Result<Vec<Table<'_>>, Error> {
    let mut assembled = Vec::with_capacity(tables.parts.len());

    for mut parts in tables.parts {
        order::sort_by_name(&mut parts.indexes);
        order::sort_by_name(&mut parts.triggers);
        order::sort_by_name(&mut parts.check_constraints);
        parts
            .foreign_keys
            .sort_by(|left, right| order::compare(&left.name, &right.name));
        // An entry of this collection is a pair and has no single name, so the
        // default rule is read over the two names it does have — the
        // referencing table first, the constraint second — exactly as the
        // document's own build site reads it.
        parts.referenced_by.sort_by(|left, right| {
            order::compare(&left.table, &right.table)
                .then_with(|| order::compare(&left.key.name, &right.key.name))
        });

        // FR-CAT-044. On this path the violation is an internal invariant:
        // FR-CAT-043 chose the index catalogue as the primary key's source so
        // that no key could name the implicit period column, and the
        // key-column read is restricted to foreign-key rows for the same
        // reason.
        assembled.push(
            Table::assemble(parts)
                .map_err(|_| refused(KEY_NAMES_A_CARRIED_COLUMN, Location::caller()))?,
        );
    }

    order::sort_by_name(&mut assembled);

    Ok(assembled)
}

/// The views of the view read (`FR-CAT-007`, `FR-CAT-047`, `FR-PRIV-011`).
fn views(rows: &[MySqlRow]) -> Result<Vec<View<'_>>, Error> {
    let mut views = Vec::with_capacity(rows.len());

    for row in rows {
        // FR-PRIV-011, the first shape of FR-PRIV-018. The row is present and
        // carries every other field; only the definition is short, and it is
        // short by being the empty string rather than by being absent.
        let definition = text(row, "VIEW_DEFINITION")?;
        let mut marking = Marking::default();

        if completeness::empty_string(definition) {
            marking.record(Property::Definition);
        }

        views.push(View {
            name: Cow::Borrowed(text(row, "TABLE_NAME")?),
            definition: Cow::Borrowed(definition),
            check_option: Cow::Borrowed(text(row, "CHECK_OPTION")?),
            is_updatable: yes(row, "IS_UPDATABLE")?,
            definer: Cow::Borrowed(text(row, "DEFINER")?),
            security_type: Cow::Borrowed(text(row, "SECURITY_TYPE")?),
            character_set_client: Cow::Borrowed(text(row, "CHARACTER_SET_CLIENT")?),
            collation_connection: Cow::Borrowed(text(row, "COLLATION_CONNECTION")?),
            algorithm: Cow::Borrowed(text(row, "ALGORITHM")?),
            // FR-PRIV-005 marks the view that is short; FR-PRIV-007 leaves
            // every other view of the same document unmarked, which is what
            // sealing an empty marking answers.
            restricted: marking.sealed(),
        });
    }

    order::sort_by_name(&mut views);

    Ok(views)
}

/// The routines of the routine read, with their parameters (`FR-CAT-008`,
/// `FR-CAT-048`, `FR-CAT-049`).
///
/// A parameter is attached by the routine's name **and kind**, because
/// `FR-SCH-008` records that procedures and functions occupy distinct
/// namespaces and one name can therefore be two objects.
fn routines<'a>(
    rows: &'a [MySqlRow],
    parameters: &'a [MySqlRow],
) -> Result<Vec<Routine<'a>>, Error> {
    let mut routines = Vec::with_capacity(rows.len());
    let mut positions = Vec::with_capacity(rows.len());

    for row in rows {
        let name = text(row, "ROUTINE_NAME")?;
        let declared = text(row, "ROUTINE_TYPE")?;

        positions.push(((name, declared), routines.len()));
        routines.push(routine(row, name, declared)?);
    }

    let positions = Positions::of(positions);

    for row in parameters {
        let key = (text(row, "SPECIFIC_NAME")?, text(row, "ROUTINE_TYPE")?);
        let parameter = RoutineParameter {
            name: Cow::Borrowed(text(row, "PARAMETER_NAME")?),
            mode: Cow::Borrowed(text(row, "PARAMETER_MODE")?),
            parameter_type: declared_type(row)?,
        };

        let Some(routine) = positions.find(key).and_then(|at| routines.get_mut(at)) else {
            continue;
        };

        routine.parameters.push(parameter);
    }

    order::sort_by_name(&mut routines);

    Ok(routines)
}

/// One routine, with an empty parameter list (`FR-CAT-048`, `FR-PRIV-017`).
fn routine<'a>(row: &'a MySqlRow, name: &'a str, declared: &str) -> Result<Routine<'a>, Error> {
    let kind = RoutineKind::from_catalogue(declared)
        .ok_or_else(|| refused(ENUMERATED, Location::caller()))?;
    // FR-PRIV-017, the second shape of FR-PRIV-018. The **nullity** is the
    // observation, and it is read from the row rather than from the model:
    // `Routine::body` cannot carry SQL `NULL`, and the empty string it becomes
    // is what a body that is genuinely empty carries too, so a verdict taken
    // over the folded model could not tell the two apart. The substitution
    // itself is unchanged and stays the recorded limit `super::row` states.
    let body = maybe_text(row, "ROUTINE_DEFINITION")?;
    let mut marking = Marking::default();

    if completeness::sql_null(body) {
        marking.record(Property::Body);
    }

    Ok(Routine {
        name: Cow::Borrowed(name),
        kind,
        // FR-CAT-048: a procedure has no return type, and the model emits it
        // as `null` in full. The two absent-value shapes the catalogue writes
        // beside each other — the empty string in the data-type field and SQL
        // `NULL` in the DTD identifier — are not read for the answer, because
        // the kind already gives it.
        return_type: match kind {
            RoutineKind::Procedure => None,
            RoutineKind::Function => Some(declared_type(row)?),
        },
        parameters: Vec::new(),
        body: Cow::Borrowed(body.unwrap_or_default()),
        body_kind: Cow::Borrowed(text(row, "ROUTINE_BODY")?),
        parameter_style: Cow::Borrowed(text(row, "PARAMETER_STYLE")?),
        is_deterministic: yes(row, "IS_DETERMINISTIC")?,
        sql_data_access: Cow::Borrowed(text(row, "SQL_DATA_ACCESS")?),
        security_type: Cow::Borrowed(text(row, "SECURITY_TYPE")?),
        sql_mode: Cow::Borrowed(text(row, "SQL_MODE")?),
        comment: Cow::Borrowed(text(row, "ROUTINE_COMMENT")?),
        definer: Cow::Borrowed(text(row, "DEFINER")?),
        character_set_client: Cow::Borrowed(text(row, "CHARACTER_SET_CLIENT")?),
        collation_connection: Cow::Borrowed(text(row, "COLLATION_CONNECTION")?),
        database_collation: Cow::Borrowed(text(row, "DATABASE_COLLATION")?),
        // FR-PRIV-005 and FR-PRIV-007, as the view read applies them: this
        // routine alone, and only where the body did not come back.
        restricted: marking.sealed(),
    })
}

/// A routine's return type or a parameter's type, decomposed exactly as
/// `FR-CTX-015` decomposes a column's (`FR-CAT-048`, `FR-CAT-049`).
///
/// The catalogue reports both through the same fields a column's type is
/// reported through, with the DTD identifier standing where a column's raw
/// type string stands — and it declares three of those fields **signed** in
/// the routine and parameter catalogues where the column catalogue declares
/// them unsigned. [`super::row`] records the observation; this is the one
/// place the difference reaches, and it is why a routine's type is not read by
/// the same function a column's is.
fn declared_type(row: &MySqlRow) -> Result<ColumnType<'_>, Error> {
    Ok(ColumnType::decompose(&CatalogueType {
        column_type: text_or_empty(row, "DTD_IDENTIFIER")?,
        data_type: text(row, "DATA_TYPE")?,
        numeric_precision: maybe_signed(row, "NUMERIC_PRECISION")?,
        datetime_precision: maybe_count(row, "DATETIME_PRECISION")?,
        numeric_scale: maybe_signed(row, "NUMERIC_SCALE")?,
        character_maximum_length: maybe_signed(row, "CHARACTER_MAXIMUM_LENGTH")?,
        charset: maybe_text(row, "CHARACTER_SET_NAME")?,
        collation: maybe_text(row, "COLLATION_NAME")?,
    }))
}
