//! The fold of a `mysql` read: the rows of the twelve statements become the
//! model (`FR-MYSQL-023` … `FR-MYSQL-041`, `FR-MYSQL-046`).
//!
//! Every relation between two object kinds is made here, as
//! `crate::mariadb::catalogue::fold` makes it, so that no statement is issued
//! per object. Two parts of that fold are taken as they are, because a `mysql`
//! read does the same: the attachment of both directions of every foreign key,
//! with the cross-check of `FR-PRIV-019` and the exclusion of `FR-CAT-057`
//! (`FR-MYSQL-033`, `FR-MYSQL-041`), and the ordering and assembly of each
//! table with its primary key read from the index named `PRIMARY`
//! (`FR-MYSQL-032`). Everything else is read here, because MySQL reports it
//! differently.
//!
//! | What differs from a `mariadb` read | Where | Fixed by |
//! |---|---|---|
//! | Coverage: `BASE TABLE` alone | [`tables`] | `FR-MYSQL-023` |
//! | The default: classified from three fields | [`default`] | `FR-MYSQL-029` |
//! | The attribute field: a sequence of phrases | [`Extra`] | `FR-MYSQL-030` |
//! | A generated column: the attribute field alone | [`column`] | `FR-MYSQL-031` |
//! | The type: three more `data_type` values | [`column`], [`declared_type`] | `FR-MYSQL-028` |
//! | An index with a functional key part: excluded | [`indexes`] | `FR-MYSQL-024` |
//! | `ignored`: the visibility field, inverted | [`indexes`] | `FR-MYSQL-032` |
//! | A `CHECK` constraint: joined to its table by name, no level | [`checks`] | `FR-MYSQL-034` |
//! | A view: no algorithm | [`views`] | `FR-MYSQL-035` |
//! | A routine of type `LIBRARY`: not covered | [`routines`] | `FR-MYSQL-023` |
//! | One layer of escaping removed from three texts | [`unescape`] | `FR-MYSQL-046` |
//!
//! **A short read says so, in the shapes `FR-MYSQL-041` records.** A view
//! whose definition is the empty string is marked under `FR-PRIV-011`, a
//! routine whose body is SQL `NULL` under `FR-PRIV-017`, and a key column with
//! no rules row under `FR-PRIV-019`; triggers are never marked, per
//! `FR-PRIV-020`.

use std::borrow::Cow;

use sqlx_mysql::MySqlRow;

use super::Catalogue;
use super::expression::{Extra, unescape, without_outer_parentheses};
use super::row::{integer, maybe_integer, maybe_text, text, text_or_empty};
use super::statements::Read;
use crate::error::{CatalogueObjectKind, Error};
use crate::mariadb::catalogue::completeness::{self, Marking, Property};
use crate::mariadb::catalogue::fold::{Positions, Tables, assemble, foreign_keys};
use crate::mariadb::fault;
use crate::model::check_constraint::CheckConstraint;
use crate::model::column::{Column, Generated};
use crate::model::column_default::ColumnDefault;
use crate::model::column_type::{CatalogueType, ColumnType};
use crate::model::database::{Database, Provider};
use crate::model::document::order;
use crate::model::index::{Index, IndexColumn, SortDirection};
use crate::model::routine::{Routine, RoutineKind, RoutineParameter};
use crate::model::table::{TableParts, TableType};
use crate::model::trigger::{Trigger, TriggerEvent, TriggerTiming};
use crate::model::view::View;

/// The value a `YES`/`NO` field takes for yes.
const YES: &str = "YES";

/// The value of the visibility field for an index the optimizer does not use
/// (`FR-MYSQL-032`).
const NOT_VISIBLE: &str = "NO";

/// The routine type `FR-MYSQL-023` does not cover.
const LIBRARY: &str = "LIBRARY";

/// Whether a `YES`/`NO` field says yes.
fn yes(row: &MySqlRow, field: &str) -> Result<bool, Error> {
    Ok(text(row, field)? == YES)
}

/// The model the fetched rows describe (`FR-MYSQL-025`).
///
/// # Errors
///
/// Returns [`Error::PropertyNotReadable`] — the `77` of `FR-PRIV-021` — where
/// the schema catalogue returned no row for the database the read covers, and
/// [`Error::InternalInvariant`] where a field does not decode or a key names a
/// column its table does not carry, as a `mariadb` read does.
pub(super) fn database(catalogue: &Catalogue) -> Result<Database<'_>, Error> {
    let Some(row) = catalogue.rows(Read::Schema).first() else {
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
    checks(
        catalogue.rows(Read::CheckOwners),
        catalogue.rows(Read::Checks),
        &mut tables,
    )?;
    triggers(catalogue.rows(Read::Triggers), &mut tables)?;

    Ok(Database {
        name: Cow::Borrowed(name),
        // FR-MYSQL-025 with BR-MYSQL-003: passed through, per FR-SRV-039.
        charset: Cow::Borrowed(text(row, "DEFAULT_CHARACTER_SET_NAME")?),
        collation: Some(Cow::Borrowed(text(row, "DEFAULT_COLLATION_NAME")?)),
        provider: Provider::Mysql,
        server: catalogue.server().clone(),
        tables: assemble(tables)?,
        views: views(catalogue.rows(Read::Views))?,
        routines: routines(
            catalogue.rows(Read::Routines),
            catalogue.rows(Read::Parameters),
        )?,
    })
}

/// The covered tables (`FR-MYSQL-023`, `FR-MYSQL-026`).
///
/// The coverage is a filter on the table type, per `FR-CAT-032`: a row is a
/// table where its type is `BASE TABLE`, and no other value covers one.
fn tables(rows: &[MySqlRow]) -> Result<Tables<'_>, Error> {
    let mut parts = Vec::with_capacity(rows.len());
    let mut positions = Vec::with_capacity(rows.len());

    for row in rows {
        if text(row, "TABLE_TYPE")? != TableType::Base.name() {
            continue;
        }

        let name = text(row, "TABLE_NAME")?;

        positions.push((name, parts.len()));
        parts.push(TableParts {
            engine: maybe_text(row, "ENGINE")?.map(Cow::Borrowed),
            collation: maybe_text(row, "TABLE_COLLATION")?.map(Cow::Borrowed),
            comment: Cow::Borrowed(text(row, "TABLE_COMMENT")?),
            ..TableParts::new(Cow::Borrowed(name), TableType::Base)
        });
    }

    Ok(Tables::new(parts, positions))
}

/// The table lookup of the previous row, answered again while the owner
/// repeats.
///
/// The member rows of one table arrive together on both series, so all but the
/// first row of a run compare one name with the last instead of searching the
/// covered tables. Any order is still answered: an owner that differs from the
/// last is looked up, and the answer is a function of the name alone.
#[derive(Default)]
struct Owner<'a> {
    /// The last owner looked up, with where its table stands.
    last: Option<(&'a str, Option<usize>)>,
}

impl<'a> Owner<'a> {
    /// Where the table `owner` names stands, as [`Tables::position`] answers.
    fn position(&mut self, tables: &Tables<'a>, owner: &'a str) -> Option<usize> {
        if let Some((last, at)) = self.last
            && last == owner
        {
            return at;
        }

        let at = tables.position(owner);
        self.last = Some((owner, at));

        at
    }
}

/// Attaches every column to its table, in the order of the values of
/// `ORDINAL_POSITION` (`FR-MYSQL-027`).
///
/// No statement orders the rows, so the fold does: each table's columns are
/// sorted by the value of the position once its rows are attached. It assumes
/// neither a first position of `1` nor contiguous positions, which a hidden
/// generated invisible primary key breaks, and nothing of the order the rows
/// arrived in. Two columns of one table never share a position, so the sort
/// has no tie to break. A column whose owning object is not covered is not
/// presented, per `FR-CAT-052`, and the rest of its row is never decoded.
///
/// *Observed on 2026-11-05, on the 2 200 tables of a schema of the profiling
/// surface of rmp `#351`:* the rows of one table arrive together on `8.4` and
/// `9.7`, in the order of the position on `8.4` and not on `9.7`. The rows are
/// therefore gathered by runs, as `crate::mariadb::catalogue::fold` gathers
/// them, and a run is sorted when it is attached; a table whose rows come back
/// in more than one run is sorted whole each time, so any order folds to the
/// same columns.
fn columns<'a>(rows: &'a [MySqlRow], tables: &mut Tables<'a>) -> Result<(), Error> {
    // PERF: one vector reused across runs, grown to the widest table once, and
    // one exactly sized allocation per table, rather than every table's vector
    // grown by doubling from empty.
    let mut run: Vec<Column<'a>> = Vec::new();
    let mut run_at: Option<usize> = None;
    let mut owners = Owner::default();

    for row in rows {
        let owner = text(row, "TABLE_NAME")?;

        let Some(at) = owners.position(tables, owner) else {
            continue;
        };
        let column = column(row, owner)?;

        if run_at != Some(at) {
            attach(tables, run_at, &mut run);
            run_at = Some(at);
        }
        run.push(column);
    }

    attach(tables, run_at, &mut run);

    Ok(())
}

/// Moves one run of columns into the table at `at`, sized for them, puts the
/// table's columns in the order of their positions, and leaves `run` empty.
fn attach<'a>(tables: &mut Tables<'a>, at: Option<usize>, run: &mut Vec<Column<'a>>) {
    if let Some(table) = at.and_then(|at| tables.at(at)) {
        table.columns.reserve_exact(run.len());
        table.columns.append(run);
        // FR-MYSQL-027: by the value of the position. The positions of one
        // table are distinct, so an unstable sort orders them as a stable one
        // would, and a run that arrived in order costs one pass.
        table.columns.sort_unstable_by_key(|column| column.position);
    } else {
        run.clear();
    }
}

/// One column (`FR-MYSQL-027` … `FR-MYSQL-031`).
fn column<'a>(row: &'a MySqlRow, owner: &'a str) -> Result<Column<'a>, Error> {
    let extra = Extra::parse(text(row, "EXTRA")?);
    let nullable = yes(row, "IS_NULLABLE")?;

    // FR-MYSQL-031: the storage kind from the attribute field, and the
    // expression from GENERATION_EXPRESSION, unescaped per FR-MYSQL-046. A
    // column carrying neither phrase is not generated, whatever that field
    // holds — the empty string on every other column.
    let generated = match extra.storage {
        Some(storage) => Some(Generated {
            expression: Some(unescape(text_or_empty(row, "GENERATION_EXPRESSION")?)),
            storage,
        }),
        None => None,
    };

    Ok(Column {
        name: Cow::Borrowed(text(row, "COLUMN_NAME")?),
        table_name: Cow::Borrowed(owner),
        position: integer(row, "ORDINAL_POSITION")?,
        column_type: ColumnType::decompose_mysql(&CatalogueType {
            column_type: text(row, "COLUMN_TYPE")?,
            data_type: text(row, "DATA_TYPE")?,
            numeric_precision: maybe_integer(row, "NUMERIC_PRECISION")?,
            datetime_precision: maybe_integer(row, "DATETIME_PRECISION")?,
            numeric_scale: maybe_integer(row, "NUMERIC_SCALE")?,
            character_maximum_length: maybe_integer(row, "CHARACTER_MAXIMUM_LENGTH")?,
            charset: maybe_text(row, "CHARACTER_SET_NAME")?,
            collation: maybe_text(row, "COLLATION_NAME")?,
        }),
        nullable,
        default: default(
            maybe_text(row, "COLUMN_DEFAULT")?,
            extra.default_generated,
            nullable,
        ),
        comment: Cow::Borrowed(text(row, "COLUMN_COMMENT")?),
        auto_increment: extra.auto_increment,
        invisible: extra.invisible,
        generated,
        // FR-MYSQL-030: the phrase as the server writes it, which is each
        // server's own spelling, as `FR-CAT-041` carries MariaDB's.
        on_update: extra.on_update.map(Cow::Borrowed),
    })
}

/// The default of a column, by the first row of the classification of
/// `FR-MYSQL-029` that matches.
///
/// | Shape | Emitted |
/// |---|---|
/// | SQL `NULL`, not nullable | [`None`] — the bare `null` of `FR-CTX-012` |
/// | SQL `NULL`, nullable | [`ColumnDefault::Null`] |
/// | `DEFAULT_GENERATED`, text in outer parentheses | an expression, the text between them, unescaped |
/// | `DEFAULT_GENERATED` | an expression, the text unescaped |
/// | Anything else | a literal, the text unchanged |
///
/// The rows are matched against the text as the catalogue returns it, and the
/// classification of `FR-CTX-037` is not applied: MySQL quotes no literal and
/// marks an expression in the attribute field.
fn default(text: Option<&str>, generated: bool, nullable: bool) -> Option<ColumnDefault<'_>> {
    match text {
        None if nullable => Some(ColumnDefault::Null),
        None => None,
        Some(text) if generated => Some(ColumnDefault::Expression(unescape(
            without_outer_parentheses(text),
        ))),
        Some(text) => Some(ColumnDefault::Literal(Cow::Borrowed(text))),
    }
}

/// Whether one index row is a functional key part: no column name, and an
/// expression (`FR-MYSQL-024`).
fn functional(row: &MySqlRow) -> Result<bool, Error> {
    Ok(maybe_text(row, "COLUMN_NAME")?.is_none() && maybe_text(row, "EXPRESSION")?.is_some())
}

/// The sequence number the first member of an index, and the first column of a
/// key, carries.
const FIRST: u64 = 1;

/// Whether a member row at `sequence` keeps the rows read so far in an order
/// the fold can keep as it reads, given the sequence of the previous row
/// where that row belonged to the same index or key.
///
/// A row that continues its group must come later in the sequence, and a row
/// that opens a group must be its first member. That is enough for every
/// group to arrive in one run, in ascending sequence, whatever else the order
/// is: two runs of one group would both open at the first member, and the
/// catalogue gives no two members of one group the same sequence
/// (`FR-MYSQL-032`, `FR-MYSQL-033`). It is not necessary — a run that opens
/// later in its sequence fails it, and is then sorted rather than trusted.
const fn in_sequence(previous: Option<u64>, sequence: u64) -> bool {
    match previous {
        Some(previous) => sequence > previous,
        None => sequence == FIRST,
    }
}

/// The index the previous row of [`fold_indexes`] belonged to.
#[derive(Clone, Copy)]
struct Run<'a> {
    /// Where its table stands.
    at: usize,
    /// Its name.
    name: &'a str,
    /// The sequence of the previous row.
    sequence: u64,
    /// Where the index stands in its table's collection, once a row opened it.
    index: Option<usize>,
}

/// The order of the rows [`fold_indexes`] is given.
#[derive(Clone, Copy, PartialEq, Eq)]
enum Arrival {
    /// As the statement returned them, in no order it states: the fold
    /// verifies, row by row, that it can keep the order they came in.
    AsReturned,
    /// Sorted by table, index and sequence.
    Sorted,
}

/// What [`fold_indexes`] did.
#[derive(PartialEq, Eq)]
enum Folded {
    /// Every row is folded, in the order of its index's sequence.
    Whole,
    /// A row arrived out of its index's sequence, or a second run of an index
    /// began; the tables hold a partial fold.
    Disordered,
}

/// Folds the index read into one object per index (`FR-MYSQL-032`), its
/// members in the order of `SEQ_IN_INDEX`, less the indexes `FR-MYSQL-024`
/// excludes.
///
/// No statement orders the rows. *Observed on 2026-11-05, on the 2 200 tables
/// of a schema of the profiling surface of rmp `#351`, on `8.4` and `9.7`:*
/// the rows of one index arrive together and in the order of the sequence. The
/// fold therefore reads them as they arrive and verifies, row by row, that the
/// order holds ([`in_sequence`]); where it does not, every index is folded
/// again from the rows sorted by table, index and sequence, so any order folds
/// to the same indexes. The cost of the order is one field decoded per row,
/// and of a fold that cannot keep it, a sort.
fn indexes<'a>(rows: &'a [MySqlRow], tables: &mut Tables<'a>) -> Result<(), Error> {
    if fold_indexes(rows.iter(), tables, Arrival::AsReturned)? == Folded::Whole {
        return Ok(());
    }

    // Nothing but this function has attached an index yet, so the partial
    // fold is undone by emptying every table's collection.
    let mut at = 0;
    while let Some(table) = tables.at(at) {
        table.indexes.clear();
        at += 1;
    }

    let mut sorted = Vec::with_capacity(rows.len());
    let mut owners = Owner::default();

    for row in rows {
        let Some(at) = owners.position(tables, text(row, "TABLE_NAME")?) else {
            continue;
        };

        sorted.push((
            (at, text(row, "INDEX_NAME")?, integer(row, "SEQ_IN_INDEX")?),
            row,
        ));
    }

    // No two rows of one index share a sequence, so the keys are distinct and
    // an unstable sort orders them as a stable one would.
    sorted.sort_unstable_by(|left, right| left.0.cmp(&right.0));

    fold_indexes(
        sorted.into_iter().map(|(_, row)| row),
        tables,
        Arrival::Sorted,
    )?;

    Ok(())
}

/// Folds index rows into their tables, in the order given.
///
/// An index any row of which is a functional key part is excluded whole, and
/// nothing is read to cover it: its other rows are not presented, so an index
/// mixing a column and an expression is absent too. A member is appended to
/// the index its run opened; the first row of a run finds the index by its
/// **exact** name, as `crate::mariadb::catalogue::fold` finds it, so that a
/// second run of one index, which only sorted rows may carry, joins the first.
fn fold_indexes<'a>(
    rows: impl Iterator<Item = &'a MySqlRow>,
    tables: &mut Tables<'a>,
    arrival: Arrival,
) -> Result<Folded, Error> {
    // One entry per (table, index) to exclude, built only where the read meets
    // a functional key part.
    let mut excluded: Vec<(usize, &'a str)> = Vec::new();
    let mut owners = Owner::default();
    let mut run: Option<Run<'a>> = None;

    for row in rows {
        let owner = text(row, "TABLE_NAME")?;
        let name = text(row, "INDEX_NAME")?;

        let Some(at) = owners.position(tables, owner) else {
            continue;
        };

        let sequence = integer(row, "SEQ_IN_INDEX")?;
        let continued = run.filter(|run| run.at == at && run.name == name);

        if arrival == Arrival::AsReturned
            && !in_sequence(continued.map(|run| run.sequence), sequence)
        {
            return Ok(Folded::Disordered);
        }

        let opened = continued.and_then(|run| run.index);

        if functional(row)? {
            excluded.push((at, name));
            run = Some(Run {
                at,
                name,
                sequence,
                index: opened,
            });
            continue;
        }

        let member = IndexColumn {
            name: Cow::Borrowed(text(row, "COLUMN_NAME")?),
            direction: maybe_text(row, "COLLATION")?.and_then(SortDirection::from_catalogue),
            prefix_length: maybe_integer(row, "SUB_PART")?,
        };

        let Some(table) = tables.at(at) else {
            continue;
        };

        let carried = opened.or_else(|| table.indexes.iter().position(|index| index.name == name));

        // A run that opens on an index already carried is a second run of it,
        // which `in_sequence` cannot see through a functional key part.
        if arrival == Arrival::AsReturned && continued.is_none() && carried.is_some() {
            return Ok(Folded::Disordered);
        }

        let index = if let Some(index) = carried.and_then(|at| table.indexes.get_mut(at)) {
            index.columns.push(member);
            carried
        } else {
            let opening = table.indexes.len();

            table.indexes.push(Index {
                name: Cow::Borrowed(name),
                // FR-MYSQL-032: unique where the non-unique field is 0.
                unique: integer(row, "NON_UNIQUE")? == 0,
                columns: vec![member],
                index_type: Some(Cow::Borrowed(text(row, "INDEX_TYPE")?)),
                comment: Cow::Borrowed(text(row, "INDEX_COMMENT")?),
                // FR-MYSQL-032: the visibility field, with its polarity
                // inverted onto the fact MariaDB reports as ignored.
                ignored: text(row, "IS_VISIBLE")? == NOT_VISIBLE,
            });

            Some(opening)
        };

        run = Some(Run {
            at,
            name,
            sequence,
            index,
        });
    }

    for (at, name) in excluded {
        if let Some(table) = tables.at(at) {
            table.indexes.retain(|index| index.name != name);
        }
    }

    Ok(Folded::Whole)
}

/// The key-column rows in an order the shared fold of `FR-MYSQL-033` keeps:
/// the rows of each key together, in the order of `ORDINAL_POSITION`.
///
/// The fold the keys go through is `crate::mariadb::catalogue::fold`'s, which
/// appends a key's columns in the order the rows stand in, and the model
/// borrows from the rows, so the order is made on the rows the read keeps,
/// once, as they are fetched. *Observed on 2026-11-05, as for the index read
/// in [`indexes`]:* the rows of one key arrive together and in the order of the
/// position, which [`in_sequence`] verifies row by row; rows that do not are
/// sorted by table, constraint and position.
///
/// # Errors
///
/// Returns [`Error::InternalInvariant`] where one of the three fields does not
/// decode, as the fold does for any field.
pub(super) fn key_columns(rows: Vec<MySqlRow>) -> Result<Vec<MySqlRow>, Error> {
    let disordered = {
        let mut previous: Option<((&str, &str), u64)> = None;
        let mut disordered = false;

        for row in &rows {
            let key = (text(row, "TABLE_NAME")?, text(row, "CONSTRAINT_NAME")?);
            let position = integer(row, "ORDINAL_POSITION")?;
            let continued = previous
                .filter(|(last, _)| *last == key)
                .map(|(_, position)| position);

            if !in_sequence(continued, position) {
                disordered = true;
                break;
            }

            previous = Some((key, position));
        }

        disordered
    };

    if !disordered {
        return Ok(rows);
    }

    let order: Vec<usize> = {
        let mut keyed = Vec::with_capacity(rows.len());

        for (at, row) in rows.iter().enumerate() {
            let key = (
                text(row, "TABLE_NAME")?,
                text(row, "CONSTRAINT_NAME")?,
                integer(row, "ORDINAL_POSITION")?,
            );
            keyed.push((key, at));
        }

        // No two columns of one key share a position, so the keys are distinct
        // and the place a row arrived at never decides.
        keyed.sort_unstable();
        keyed.into_iter().map(|(_, at)| at).collect()
    };

    let mut slots: Vec<Option<MySqlRow>> = rows.into_iter().map(Some).collect();

    Ok(order
        .into_iter()
        .filter_map(|at| slots.get_mut(at).and_then(Option::take))
        .collect())
}

/// Attaches every `CHECK` constraint to its table (`FR-MYSQL-034`).
///
/// The table-constraint rows of type `CHECK` name the table; the
/// check-constraint rows carry the clause; the two are joined on the
/// constraint name, which MySQL makes unique in a schema and both reads
/// restrict to the database the read covers. `level` is `null`: MySQL records
/// none. A constraint whose clause row is absent is not presented, because it
/// has no clause to carry; `FR-MYSQL-041` records that no reader observed
/// loses one without the other.
fn checks<'a>(
    owners: &'a [MySqlRow],
    clauses: &'a [MySqlRow],
    tables: &mut Tables<'a>,
) -> Result<(), Error> {
    let mut by_name = Vec::with_capacity(clauses.len());
    for (at, row) in clauses.iter().enumerate() {
        by_name.push((text(row, "CONSTRAINT_NAME")?, at));
    }
    let by_name = Positions::of(by_name);

    for row in owners {
        let owner = text(row, "TABLE_NAME")?;
        let name = text(row, "CONSTRAINT_NAME")?;

        let Some(at) = tables.position(owner) else {
            continue;
        };
        let Some(clause) = by_name.find(name).and_then(|at| clauses.get(at)) else {
            continue;
        };

        let constraint = CheckConstraint {
            name: Cow::Borrowed(name),
            level: None,
            // FR-MYSQL-046: the extra layer removed, the introducers and the
            // outer parentheses kept.
            clause: unescape(text(clause, "CHECK_CLAUSE")?),
        };

        if let Some(table) = tables.at(at) {
            table.check_constraints.push(constraint);
        }
    }

    Ok(())
}

/// Attaches every trigger to its table (`FR-MYSQL-038`).
fn triggers<'a>(rows: &'a [MySqlRow], tables: &mut Tables<'a>) -> Result<(), Error> {
    for row in rows {
        let owner = text(row, "EVENT_OBJECT_TABLE")?;

        let Some(at) = tables.position(owner) else {
            continue;
        };
        let trigger = Trigger {
            name: Cow::Borrowed(text(row, "TRIGGER_NAME")?),
            event: TriggerEvent::from_catalogue(text(row, "EVENT_MANIPULATION")?),
            timing: TriggerTiming::from_catalogue(text(row, "ACTION_TIMING")?),
            action_order: Some(integer(row, "ACTION_ORDER")?),
            // FR-MYSQL-038: the body as written, with no layer to remove.
            statement: maybe_text(row, "ACTION_STATEMENT")?.map(Cow::Borrowed),
            orientation: Cow::Borrowed(text(row, "ACTION_ORIENTATION")?),
            old_row_alias: Cow::Borrowed(text(row, "ACTION_REFERENCE_OLD_ROW")?),
            new_row_alias: Cow::Borrowed(text(row, "ACTION_REFERENCE_NEW_ROW")?),
            sql_mode: Some(Cow::Borrowed(text(row, "SQL_MODE")?)),
            definer: maybe_text(row, "DEFINER")?.map(Cow::Borrowed),
            character_set_client: Some(Cow::Borrowed(text(row, "CHARACTER_SET_CLIENT")?)),
            collation_connection: Some(Cow::Borrowed(text(row, "COLLATION_CONNECTION")?)),
            database_collation: Some(Cow::Borrowed(text(row, "DATABASE_COLLATION")?)),
        };

        if let Some(table) = tables.at(at) {
            table.triggers.push(trigger);
        }
    }

    Ok(())
}

/// The views (`FR-MYSQL-035`, `FR-PRIV-011`).
fn views(rows: &[MySqlRow]) -> Result<Vec<View<'_>>, Error> {
    let mut views = Vec::with_capacity(rows.len());

    for row in rows {
        // FR-MYSQL-035 with FR-PRIV-011: a definition that is the empty string
        // on a row that is present is a missing privilege.
        let definition = text(row, "VIEW_DEFINITION")?;
        let mut marking = Marking::default();

        if completeness::empty_string(definition) {
            marking.record(Property::Definition);
        }

        views.push(View {
            name: Cow::Borrowed(text(row, "TABLE_NAME")?),
            definition: Cow::Borrowed(definition),
            check_option: Some(Cow::Borrowed(text(row, "CHECK_OPTION")?)),
            is_updatable: Some(yes(row, "IS_UPDATABLE")?),
            definer: Some(Cow::Borrowed(text(row, "DEFINER")?)),
            security_type: Some(Cow::Borrowed(text(row, "SECURITY_TYPE")?)),
            character_set_client: Some(Cow::Borrowed(text(row, "CHARACTER_SET_CLIENT")?)),
            collation_connection: Some(Cow::Borrowed(text(row, "COLLATION_CONNECTION")?)),
            // FR-MYSQL-035: MySQL's view catalogue has no algorithm field.
            algorithm: None,
            restricted: marking.sealed(),
        });
    }

    order::sort_by_name(&mut views);

    Ok(views)
}

/// The routines, with their parameters (`FR-MYSQL-036`, `FR-MYSQL-037`).
///
/// A row of type `LIBRARY` is not covered, per `FR-MYSQL-023`, and its
/// parameters find no routine to attach to and are not presented either.
fn routines<'a>(
    rows: &'a [MySqlRow],
    parameters: &'a [MySqlRow],
) -> Result<Vec<Routine<'a>>, Error> {
    let mut routines = Vec::with_capacity(rows.len());
    let mut positions = Vec::with_capacity(rows.len());

    for row in rows {
        let declared = text(row, "ROUTINE_TYPE")?;

        if declared == LIBRARY {
            continue;
        }

        let name = text(row, "ROUTINE_NAME")?;

        positions.push(((name, declared), routines.len()));
        routines.push(routine(row, name, declared)?);
    }

    let positions = Positions::of(positions);

    for row in parameters {
        let key = (text(row, "SPECIFIC_NAME")?, text(row, "ROUTINE_TYPE")?);

        let Some(at) = positions.find(key) else {
            continue;
        };
        let parameter = RoutineParameter {
            name: Cow::Borrowed(text(row, "PARAMETER_NAME")?),
            mode: Cow::Borrowed(text(row, "PARAMETER_MODE")?),
            parameter_type: declared_type(row)?,
        };

        if let Some(routine) = routines.get_mut(at) {
            routine.parameters.push(parameter);
        }
    }

    order::sort_by_name(&mut routines);

    Ok(routines)
}

/// One routine, with an empty parameter list (`FR-MYSQL-036`, `FR-PRIV-017`).
fn routine<'a>(row: &'a MySqlRow, name: &'a str, declared: &'a str) -> Result<Routine<'a>, Error> {
    let kind = RoutineKind::from_catalogue(declared);
    // FR-MYSQL-036: the body is `null` where the field is SQL `NULL`, and the
    // routine is then incomplete, per FR-PRIV-017.
    let body = maybe_text(row, "ROUTINE_DEFINITION")?;
    let mut marking = Marking::default();

    if completeness::sql_null(body) {
        marking.record(Property::Body);
    }

    // FR-MYSQL-036: a procedure's return type is `null` in full, whatever its
    // type fields hold; any other kind carries the type the row states, as
    // `FR-CAT-048` reads it.
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
        restricted: marking.sealed(),
    })
}

/// A routine's return type or a parameter's type, decomposed as a column's is
/// (`FR-MYSQL-036`, `FR-MYSQL-037`, `FR-MYSQL-028`).
fn declared_type(row: &MySqlRow) -> Result<ColumnType<'_>, Error> {
    Ok(ColumnType::decompose_mysql(&CatalogueType {
        column_type: text_or_empty(row, "DTD_IDENTIFIER")?,
        data_type: text(row, "DATA_TYPE")?,
        numeric_precision: maybe_integer(row, "NUMERIC_PRECISION")?,
        datetime_precision: maybe_integer(row, "DATETIME_PRECISION")?,
        numeric_scale: maybe_integer(row, "NUMERIC_SCALE")?,
        character_maximum_length: maybe_integer(row, "CHARACTER_MAXIMUM_LENGTH")?,
        charset: maybe_text(row, "CHARACTER_SET_NAME")?,
        collation: maybe_text(row, "COLLATION_NAME")?,
    }))
}

#[cfg(test)]
mod tests {
    use super::{FIRST, default, in_sequence};
    use crate::model::column_default::ColumnDefault;
    use std::borrow::Cow;

    /// Every order of `rows`, by Heap's algorithm.
    fn orders<T: Copy>(rows: &[T]) -> Vec<Vec<T>> {
        fn heap<T: Copy>(rows: &mut Vec<T>, length: usize, orders: &mut Vec<Vec<T>>) {
            if length <= 1 {
                orders.push(rows.clone());
                return;
            }
            for at in 0..length {
                heap(rows, length - 1, orders);
                let swapped = if length.is_multiple_of(2) { at } else { 0 };
                rows.swap(swapped, length - 1);
            }
        }
        let mut rows = rows.to_vec();
        let mut orders = Vec::new();
        let length = rows.len();
        heap(&mut rows, length, &mut orders);
        orders
    }

    /// Whether every row of `order`, a group and a sequence each, passes
    /// [`in_sequence`] against the row before it, as the fold reads them.
    fn kept(order: &[(char, u64)]) -> bool {
        let mut previous: Option<(char, u64)> = None;
        order.iter().all(|&(group, sequence)| {
            let continued = previous
                .filter(|(last, _)| *last == group)
                .map(|(_, sequence)| sequence);
            previous = Some((group, sequence));
            in_sequence(continued, sequence)
        })
    }

    #[test]
    fn fr_mysql_032_033_an_order_the_fold_keeps_is_one_run_per_group_in_sequence() {
        // rmp #351: over every order of the rows of three indexes (or keys) of
        // three, two and one members — 720 orders — an order every row of
        // which passes is one in which each group arrived in one run, in
        // ascending sequence; and every such order passes.
        let rows = [('a', 1), ('a', 2), ('a', 3), ('b', 1), ('b', 2), ('c', 1)];
        let all = orders(&rows);
        assert_eq!(all.len(), 720);

        let mut admitted = 0;
        for order in &all {
            let runs = order
                .windows(2)
                .filter(|pair| pair[0].0 != pair[1].0)
                .count()
                + 1;
            let ascending = order
                .windows(2)
                .all(|pair| pair[0].0 != pair[1].0 || pair[0].1 < pair[1].1);
            let grouped = runs == 3;

            assert_eq!(kept(order), grouped && ascending, "{order:?}");
            admitted += usize::from(kept(order));
        }
        // The three groups in any order, each whole and ascending.
        assert_eq!(admitted, 6);

        // A run that opens later in its sequence is not trusted, even where
        // the rest of its order is ascending: it is sorted instead.
        assert!(!in_sequence(None, FIRST + 1));
        assert!(!kept(&[('a', 2), ('a', 3)]));
        // A gap in a run is admitted: only the order is relied on.
        assert!(kept(&[('a', 1), ('a', 3), ('b', 1)]));
    }

    #[test]
    fn fr_mysql_029_every_row_of_the_classification_is_reached_by_its_shape() {
        // SQL NULL, by the nullability.
        assert_eq!(default(None, false, false), None);
        assert_eq!(default(None, false, true), Some(ColumnDefault::Null));

        // A literal, carried unchanged: no quote to remove, and 'NULL' is the
        // four characters, never a null default.
        for literal in [
            "EUR",
            "",
            "NULL",
            "it's",
            r"back\slash",
            "8'6\"",
            "1500",
            "b'101'",
            "0x6162",
        ] {
            assert_eq!(
                default(Some(literal), false, true),
                Some(ColumnDefault::Literal(Cow::Borrowed(literal))),
                "{literal}"
            );
        }

        // An expression, marked DEFAULT_GENERATED, unchanged or between its
        // outer parentheses, and unescaped per FR-MYSQL-046.
        for (written, carried) in [
            ("CURRENT_TIMESTAMP", "CURRENT_TIMESTAMP"),
            ("CURRENT_TIMESTAMP(3)", "CURRENT_TIMESTAMP(3)"),
            (
                "(curdate() + interval 30 day)",
                "curdate() + interval 30 day",
            ),
            ("((1 + 2) * (3 + 4))", "(1 + 2) * (3 + 4)"),
            ("now()", "now()"),
            ("-(1)", "-(1)"),
            ("0x554e413a2b2e3f2027", "0x554e413a2b2e3f2027"),
            (r"_utf8mb4\'it\\\'s\'", r"_utf8mb4'it\'s'"),
        ] {
            assert_eq!(
                default(Some(written), true, false),
                Some(ColumnDefault::Expression(Cow::Borrowed(carried))),
                "{written}"
            );
        }
    }
}
