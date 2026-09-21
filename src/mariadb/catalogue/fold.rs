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
//! # Where the fold does **not** refuse, and why it used to
//!
//! Five catalogue fields take a set of values that a requirement fixed from an
//! observation of all four series: a referential action (`FR-CAT-045`), a
//! constraint level (`FR-CAT-046`), a trigger event and a trigger timing
//! (`FR-CAT-050`), and a routine kind (`FR-CAT-016`). The first implementation
//! of this fold reported a value outside one of those sets as
//! [`Error::InternalInvariant`], `70`, on the ground that the model had a
//! variant for each observed value and for no other.
//!
//! **`FR-CAT-055` rejects that reading, and the fold now carries the
//! catalogue's own string.** Each of the five is an enumeration with a
//! recorded variant per observed value and one that carries an unrecorded
//! string verbatim, so a sixth referential action reaches the document as the
//! server spelled it. The object is not dropped, the read is not refused, and
//! the exit code does not move on account of it.
//!
//! Three grounds, and the requirement states all three:
//!
//! | Why `70` was wrong | Stated by |
//! |---|---|
//! | `70` is closed to a panic and to an invariant the system detects **in itself**, and nothing about `tpl` is defective when a server returns a value `tpl` has not seen | `FR-ERR-030`, and the same correction `FR-PRIV-021` made for the schema row below |
//! | The `70` row tells the caller the condition is not fixable by them, which makes a whole database unreadable over one rule of one key | `FR-ERR-001` |
//! | The one foreseeable server that returns such a value is one newer than the window, and that server is read and **marked** rather than refused | `FR-SRV-031`, `FR-SRV-032`, `FR-SRV-033` |
//!
//! The two alternatives the requirement also rejects are the ones a reader
//! reaches for next, and both are wrong documents at exit `0`: substituting the
//! nearest recorded value puts a value in the document the server did not
//! state, and dropping the object that carries the field presents a table
//! without a foreign key it has. `FR-CTX-034`'s `standing` is the field that
//! says such a read is unverified, and there is no second thing for an exit
//! code to add.
//!
//! **`table_type` is not one of the five, and the difference is the field's
//! job.** `FR-CAT-001` through `FR-CAT-006` make that value a coverage
//! predicate, so a value outside the recorded set selects no covered kind and
//! the object is presented nowhere — which is what
//! [`TableType::from_catalogue`] already does and what the section above
//! describes.
//!
//! # Where the fold takes the completeness verdict
//!
//! The three checks of [`super::completeness`] are made **here**, on the row,
//! as the model is built — never on the finished model. One of the three would
//! be wrong if it were made later: a foreign key whose rules row is missing
//! reaches the model as no foreign key at all, which is indistinguishable from
//! a table that declares none.
//!
//! | Where | Check | Marks | Fixed by |
//! |---|---|---|---|
//! | [`views`] | The definition is the empty string | That view, with `definition` | `FR-PRIV-011` |
//! | [`routine`] | The body is SQL `NULL` | That routine, with `body` | `FR-PRIV-017` |
//! | [`foreign_keys`] | A key column has no referential-constraint row | The referencing table with `foreign_keys`, and the referenced one with `referenced_by` | `FR-PRIV-019` |
//!
//! *The second reason this section used to give has been removed by
//! `FR-CAT-056`.* A routine's body reached the model as the empty string
//! whether the catalogue returned SQL `NULL` or an empty body, so a verdict
//! taken over the folded model could not have told the two apart; the body is
//! now [`None`] for SQL `NULL` and the model **can** tell them apart. The
//! verdict is still taken here all the same, because the foreign-key reason
//! above stands on its own and because moving one of the three checks
//! elsewhere would put `FR-PRIV-012` in two places.
//!
//! # Where the fold drops a key the catalogue did return
//!
//! `FR-CAT-057` excludes a foreign key that crosses a schema boundary, in
//! **both** directions, and bars reading a second schema in order to cover one.
//! The two directions are excluded in two different places:
//!
//! | Direction | Excluded by |
//! |---|---|
//! | Declared in another schema, referencing a table this read covers | The `TABLE_SCHEMA = ?` predicate of the key-column statement — the read never sees the row |
//! | Declared in this schema, referencing a table in another | [`foreign_keys`], on the referenced-schema field the statement now selects |
//!
//! The first was an accident of the statement's filter and is now a
//! requirement; the second needed the field, because a rules row for an
//! outgoing cross-schema key **is** returned — its constraint is declared here
//! — and nothing else on either row says where the referenced table lives.
//!
//! *The ground is `FR-CTX-006`, `FR-CTX-010` and `FR-CTX-023` together.* They
//! embed the table at each end of every key in full and require every object a
//! document references to be present in it, and a table in another schema is in
//! no model this read builds. A carried cross-schema key is therefore a
//! reference with nothing at the other end — the one condition those three
//! forbid — so the exclusion is what keeps the embedding materialisable.
//!
//! *`FR-PRIV-002` is not engaged, and the cost is silent.* A table at either
//! end of a cross-schema key is presented with one relation fewer than the
//! server holds, at exit `0`, and nothing in the document says so: the property
//! was read and excluded, not withheld. `FR-CAT-057` accepts that cost in its
//! own text, which is why no marking is written here.
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
use crate::error::{CatalogueObjectKind, Error};
use crate::mariadb::fault;
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

/// The invariant a key naming a column its table does not carry violates
/// (`FR-CAT-044`).
const KEY_NAMES_A_CARRIED_COLUMN: &str =
    "a key the catalogue reports names a column the same table's column list carries";

/// Whether a foreign key crosses a schema boundary (`FR-CAT-057`).
///
/// `referenced` is the referenced-schema field of a key-column row and
/// `covered` is the database the read covers, per `FR-CONF-041`. Only the
/// **outgoing** direction reaches this predicate: the incoming one returns no
/// row at all, because the statement is filtered on the schema the key is
/// declared in.
///
/// SQL `NULL` is **not** a crossing. A referenced schema that names nothing
/// names no *other* schema, so the key stays in the model and `FR-CAT-056`
/// governs what it carries; excluding it here would drop a key `FR-CTX-006`
/// requires to be presented, and would do so on a field that says nothing
/// about where the referenced table is.
///
/// It is a free function over the two values rather than a branch inside the
/// fold so that the decision is exercised without a server, on the same terms
/// as the three checks of [`super::completeness`]. No read of the fixture
/// reaches the true case — observed on 2026-09-20 on all four series of
/// `FR-SRV-015`, where every one of the 17 foreign-key rows names `freight` at
/// both ends — which is what `FR-CAT-057` records and why the requirement
/// states the feature is excluded rather than observed.
fn crosses_a_schema(referenced: Option<&str>, covered: &str) -> bool {
    referenced.is_some_and(|named| named != covered)
}

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
    /// other end — or [`None`] where the rules row named none (`FR-CAT-056`).
    referenced_table: Option<&'a str>,

    /// Whether the key crosses a schema boundary and is therefore excluded
    /// from the model in both directions (`FR-CAT-057`).
    ///
    /// It is decided on the **key-column** rows, which are the only rows of the
    /// two reads that carry the referenced schema, and it is recorded here
    /// because the rules row is what the key is finally built from: a key
    /// excluded on one of its columns must not reach a table through its rule
    /// either, or the document would carry a key with no columns.
    crosses_a_schema: bool,

    /// The key itself, whose column list the second read fills.
    key: ForeignKey<'a>,
}

/// The model the fetched rows describe (`FR-CTX-035`, `FR-CTX-036`).
///
/// # Errors
///
/// Returns [`Error::InternalInvariant`] — `70` — where the catalogue returned a
/// value the model cannot represent: a decode that failed, or a key naming a
/// column its table does not carry (`FR-CAT-044`). Both are a `70` for the
/// reason [`super::row`] records — the field lists were observed on all four
/// series of `FR-SRV-015`, so a shape outside them is a defect in this reader's
/// model of the catalogue rather than a condition the caller can act on.
///
/// Returns [`Error::PropertyNotReadable`] — `77` — where the schema catalogue
/// returned no row for the database the read covers. `FR-PRIV-021` fixes that
/// code and rejects the `70` this fold used to report in its own words: the
/// shortfall is in what the reader was shown, not in `tpl`, and the caller can
/// fix it with a grant or by correcting `database.<name>.database`. The
/// condition is not reachable through the distributed binary today, because
/// `FR-CONF-041` puts the database on the connection and the handshake
/// classifies `1049` and `1044` first — [`super::super::fault`] routes that
/// pair to this same condition — but the divergence between this path and that
/// one is the one route by which a later change would resurrect the rejected
/// `70`.
///
/// An enumerated value outside the set its requirement fixes is **no longer**
/// among them: `FR-CAT-055` carries it verbatim, per this module's own header.
pub(super) fn database(catalogue: &Catalogue) -> Result<Database<'_>, Error> {
    let schema = catalogue.rows(Read::Schema);
    let Some(row) = schema.first() else {
        // FR-PRIV-021. The `cause` names the database and states that its
        // metadata could not be read, and the database is the one the selected
        // entry names, per FR-CONF-041 — which is the name this read was
        // issued with, so the condition always has an instance to name.
        return Err(Error::PropertyNotReadable {
            kind: CatalogueObjectKind::Database,
            object: catalogue.schema().to_owned(),
            property: fault::METADATA,
        });
    };

    let name = text(row, "SCHEMA_NAME")?;
    let mut tables = tables(catalogue.rows(Read::Tables))?;

    columns(catalogue.rows(Read::Columns), &mut tables)?;
    indexes(catalogue.rows(Read::Indexes), &mut tables)?;
    foreign_keys(
        catalogue.rows(Read::ForeignKeyRules),
        catalogue.rows(Read::KeyColumns),
        name,
        &mut tables,
    )?;
    checks(catalogue.rows(Read::Checks), &mut tables)?;
    triggers(catalogue.rows(Read::Triggers), &mut tables)?;

    Ok(Database {
        name: Cow::Borrowed(name),
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
///
/// `schema` is the database the read covers, and it is here for one reason:
/// `FR-CAT-057` excludes a key whose referenced table lives in another schema,
/// and the referenced-schema field of the key-column row is the only place
/// either read says where that table is.
fn foreign_keys<'a>(
    rules: &'a [MySqlRow],
    key_columns: &'a [MySqlRow],
    schema: &str,
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
        let at = positions.find(key);

        // FR-CAT-057, the outgoing direction. A referenced schema that is named
        // and is not the one this read covers puts the referenced table outside
        // every model this read builds, so the key is excluded here and its
        // rules row is excluded with it — before the cross-check below, because
        // an excluded key is not a property the reader lost but one the model
        // does not cover, and `FR-PRIV-002` is not engaged by it.
        //
        // A referenced schema that is SQL `NULL` names no other schema and is
        // not this case: the key is carried, and `FR-CAT-056` governs what it
        // carries. Dropping it here would drop a key `FR-CTX-006` requires to
        // be presented.
        if crosses_a_schema(maybe_text(row, "REFERENCED_TABLE_SCHEMA")?, schema) {
            if let Some(rule) = at.and_then(|at| carried.get_mut(at)) {
                rule.crosses_a_schema = true;
            }

            continue;
        }

        // FR-PRIV-019's antecedent, read as the field it is stated over. The
        // statement already restricts the read to rows that name a referenced
        // table, per `FR-CAT-045`, so this is the name the other end is marked
        // by rather than a second filter.
        let referenced = maybe_text(row, "REFERENCED_TABLE_NAME")?;
        let pair = ForeignKeyColumn {
            column: Cow::Borrowed(text(row, "COLUMN_NAME")?),
            // FR-CAT-056: `text` and not `text_or_empty`. The statement selects
            // exactly the rows on which this field is populated — all 17 of the
            // fixture's 54 — so no row carrying SQL `NULL` reaches the model
            // and there is no absence to substitute for.
            referenced_column: Cow::Borrowed(text(row, "REFERENCED_COLUMN_NAME")?),
        };

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
        // FR-CAT-057: excluded in both directions, so the rule contributes to
        // neither collection.
        if rule.crosses_a_schema {
            continue;
        }

        // FR-CAT-013 carries the same key on the referenced table, so the two
        // directions are two values. The clone is the second of them: the
        // model presents one key from two ends, and building it a second time
        // from the rows would allocate the same column list again for no
        // clearer result.
        //
        // A key that names no referenced table has no other end to be carried
        // on, which `FR-CAT-056` states and `FR-CTX-006` reads from the
        // document's side: it is carried on the referencing table alone.
        if let Some(at) = rule.referenced_table.and_then(|name| tables.position(name))
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
///
/// The two rule fields are read through `FR-CAT-055`'s total reading: a
/// spelling outside the four `FR-CAT-033` recorded is carried as the catalogue
/// wrote it rather than refused. The two nullable fields are read through
/// `FR-CAT-056`'s: SQL `NULL` reaches the model as [`None`], not as the empty
/// string.
fn rule<'a>(row: &'a MySqlRow, table: &'a str, name: &'a str) -> Result<Rule<'a>, Error> {
    let action = |field: &str| Ok(ReferentialAction::from_catalogue(text(row, field)?));
    let referenced_table = maybe_text(row, "REFERENCED_TABLE_NAME")?;

    Ok(Rule {
        table,
        referenced_table,
        crosses_a_schema: false,
        key: ForeignKey {
            name: Cow::Borrowed(name),
            columns: Vec::new(),
            referenced_table: referenced_table.map(Cow::Borrowed),
            referenced_key: maybe_text(row, "UNIQUE_CONSTRAINT_NAME")?.map(Cow::Borrowed),
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
            // FR-CAT-055: a third level is carried as the catalogue wrote it.
            level: ConstraintLevel::from_catalogue(text(row, "LEVEL")?),
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
///
/// The event and the timing are read through `FR-CAT-055`'s total reading, and
/// the action statement and the definer through `FR-CAT-056`'s: both are
/// declared nullable on all four series and reach the model as [`Option`].
fn trigger(row: &MySqlRow) -> Result<Trigger<'_>, Error> {
    Ok(Trigger {
        name: Cow::Borrowed(text(row, "TRIGGER_NAME")?),
        event: TriggerEvent::from_catalogue(text(row, "EVENT_MANIPULATION")?),
        timing: TriggerTiming::from_catalogue(text(row, "ACTION_TIMING")?),
        action_order: signed(row, "ACTION_ORDER")?,
        statement: maybe_text(row, "ACTION_STATEMENT")?.map(Cow::Borrowed),
        orientation: Cow::Borrowed(text(row, "ACTION_ORIENTATION")?),
        old_row_alias: Cow::Borrowed(text(row, "ACTION_REFERENCE_OLD_ROW")?),
        new_row_alias: Cow::Borrowed(text(row, "ACTION_REFERENCE_NEW_ROW")?),
        sql_mode: Cow::Borrowed(text(row, "SQL_MODE")?),
        definer: maybe_text(row, "DEFINER")?.map(Cow::Borrowed),
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
fn routine<'a>(row: &'a MySqlRow, name: &'a str, declared: &'a str) -> Result<Routine<'a>, Error> {
    // FR-CAT-055: a third kind is carried as the catalogue wrote it. It is not
    // reachable by the qualified name of `FR-SCH-008`, which is that
    // requirement's own stated consequence and is enforced where a qualified
    // name is composed rather than here.
    let kind = RoutineKind::from_catalogue(declared);
    // FR-PRIV-017, the second shape of FR-PRIV-018. The nullity is the
    // observation, and it is both what the model carries under `FR-CAT-056`
    // and what the marking is taken from: `null` says the body is absent and
    // the marking says why it is absent.
    let body = maybe_text(row, "ROUTINE_DEFINITION")?;
    let mut marking = Marking::default();

    if completeness::sql_null(body) {
        marking.record(Property::Body);
    }

    // FR-CAT-048: a procedure has no return type, and the model emits it as
    // `null` in full. The two absent-value shapes the catalogue writes beside
    // each other — the empty string in the data-type field and SQL `NULL` in
    // the DTD identifier — are not read for the answer, because the kind
    // already gives it.
    //
    // A kind outside the recorded two is read as the row states it rather than
    // as an absence: `FR-CAT-048` gives `null` to a **procedure**, and an
    // unrecorded kind is not one, so substituting an absent return type for it
    // would put in the document a fact the server did not state. *Rejected:
    // `null` for every kind but `FUNCTION`*, which reads that requirement as
    // naming a default rather than a case.
    let return_type = match &kind {
        RoutineKind::Procedure => None,
        _ => Some(declared_type(row)?),
    };

    Ok(Routine {
        name: Cow::Borrowed(name),
        kind,
        return_type,
        parameters: Vec::new(),
        body: body.map(Cow::Borrowed),
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

#[cfg(test)]
mod tests {
    use super::crosses_a_schema;

    #[test]
    fn fr_cat_057_a_key_referencing_another_schema_crosses_a_boundary_and_one_referencing_none_does_not()
     {
        // FR-CAT-057 excludes a foreign key that crosses a schema boundary in
        // both directions. The outgoing direction is decided here, on the
        // referenced-schema field the key-column statement selects.
        assert!(
            crosses_a_schema(Some("billing"), "freight"),
            "a key naming a table in another schema is excluded: the embedding of FR-CTX-006 \
             and FR-CTX-010 has nothing at the other end, which FR-CTX-023 forbids"
        );

        // The schema the read covers is not a crossing, which is every key the
        // fixture holds.
        assert!(!crosses_a_schema(Some("freight"), "freight"));

        // SQL `NULL` names no other schema. FR-CAT-056 gives that key a
        // `referenced_table` of `null` and FR-CTX-006 requires it to be carried
        // all the same, so reading the absence as a crossing would drop a key
        // the corpus obliges the document to present.
        assert!(!crosses_a_schema(None, "freight"));

        // The comparison is byte-wise, as every other name comparison in this
        // reader is: MariaDB schema names are case-sensitive on the platforms
        // `NFR-PERF-018` names, and folding here would silently admit a key
        // this requirement excludes.
        assert!(crosses_a_schema(Some("FREIGHT"), "freight"));
        assert!(crosses_a_schema(Some(""), "freight"));
    }
}
