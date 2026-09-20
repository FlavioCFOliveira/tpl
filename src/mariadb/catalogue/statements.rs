//! The fixed repertoire: every statement a catalogue read may issue, and the
//! plan one read is composed of (`FR-SRV-006`, `FR-SRV-037`, `NFR-PERF-001`,
//! `NFR-PERF-002`).
//!
//! **One statement per object kind, and the plan is a pure function of the
//! scope.** [`plan`] is given a schema name and a [`Scope`] and nothing else —
//! no row, no count, no server answer — so the number of statements a read
//! issues cannot depend on what the database holds. That is `NFR-PERF-001` and
//! `NFR-PERF-002` made structural rather than reviewed: the plan is complete
//! before the first statement is sent, and [`super::read`] issues exactly the
//! plan.
//!
//! | # | [`Read`] | Catalogue table | What it carries |
//! |---|---|---|---|
//! | 1 | [`Read::Schema`] | `SCHEMATA` | The three metadata fields of `FR-CTX-036` |
//! | 2 | [`Read::Tables`] | `TABLES` | The table row, whose type is the coverage predicate of `FR-CAT-001` … `FR-CAT-006` |
//! | 3 | [`Read::Columns`] | `COLUMNS` | `FR-CAT-009`, with the type fields of `FR-CTX-040` and the attribute field of `FR-CAT-041` |
//! | 4 | [`Read::Indexes`] | `STATISTICS` | `FR-CAT-042`, one row per column, folded by `FR-CAT-010`; it is also the primary key's only source, per `FR-CAT-043` |
//! | 5 | [`Read::ForeignKeyRules`] | `REFERENTIAL_CONSTRAINTS` | The rules half of `FR-CAT-045` |
//! | 6 | [`Read::KeyColumns`] | `KEY_COLUMN_USAGE` | The columns half of `FR-CAT-045`, and **both** directions of `FR-CAT-012` and `FR-CAT-013` |
//! | 7 | [`Read::Checks`] | `CHECK_CONSTRAINTS` | `FR-CAT-046` |
//! | 8 | [`Read::Triggers`] | `TRIGGERS` | `FR-CAT-050` |
//! | 9 | [`Read::Views`] | `VIEWS` | `FR-CAT-047` |
//! | 10 | [`Read::Routines`] | `ROUTINES` | `FR-CAT-048` |
//! | 11 | [`Read::Parameters`] | `PARAMETERS` | `FR-CAT-049`, the return row excluded in SQL |
//!
//! **There is no twelfth statement for the primary key.** `FR-CAT-043` makes
//! the index catalogue its authoritative source and bars the other two, so the
//! primary key is the index named `PRIMARY` in the rows statement 4 already
//! returned. A statement of its own would be a second source for one fact.
//!
//! **The incoming direction of a foreign key is not a statement either.**
//! `FR-CAT-045` obtains it by reading the **same** key-column-usage rows from
//! the other end, filtered on the referenced table name, so statement 6 serves
//! `FR-CAT-012` and `FR-CAT-013` together.
//!
//! # `FR-SRV-037`: the column lists are the ones common to all four series
//!
//! `FR-SRV-037` offers two answers and this module takes the second: no column
//! list below names a column that is absent from any series of `FR-SRV-015`,
//! so no list is selected from the resolved series and none has to be. The
//! three catalogue tables whose width differs — differences 5, 6 and 7 of
//! `FR-SRV-038` — are accommodated by naming none of the columns that differ:
//!
//! | Table | Columns that differ | Named here |
//! |---|---|---|
//! | `COLUMNS` | `IS_SYSTEM_TIME_PERIOD_START`, `IS_SYSTEM_TIME_PERIOD_END`, absent on `10.11` | no — the model carries neither |
//! | `PARAMETERS` | `PARAMETER_DEFAULT`, present on `12.3` alone | no — `FR-CAT-049` does not carry it |
//! | `PERIODS` | the whole table, absent on `10.11` | no — `FR-CAT-022` excludes application-time periods |
//!
//! *Observed, 2026-09-18, against the fixture of `scripts/mariadb/` on all four
//! series.* The column list of each of the eleven catalogue tables was read
//! back from `10.11`, `11.4`, `11.8` and `12.3` and compared; the three rows
//! above are the only differences between them, and none of the three is
//! named here. Each of the eleven statements was then executed on all four and
//! returned the same row count on each — 1, 23, 301, 77, 15, 17, 24, 6, 5, 7
//! and 18. `FR-SRV-023` is satisfied without a fall-back path, because there
//! is nothing to fall back from.
//!
//! # Why the text is built by a macro
//!
//! Every read has a schema-wide form and, where a command names one object, a
//! narrowed one; the forms differ by a predicate and share a column list, and
//! `FR-SRV-037` is a property **of that column list**. Writing it once per
//! form would make the requirement checkable in two places that can drift
//! apart, so [`forms!`] writes it once and appends each form's predicate. The
//! product is still a `&'static str`, which is what `sqlx`'s `SqlSafeStr`
//! admits without an assertion: no schema name, object name or routine kind is
//! ever composed into SQL, and all three travel as bind parameters.

use crate::model::routine::RoutineKind;

/// The most bind parameters any statement of the repertoire takes.
///
/// Three: the schema, an object name, and — for the narrowed forms that carry
/// the name twice, or that carry a routine kind beside it — one more.
const MAX_BINDS: usize = 3;

/// Defines every form of one read from one column list.
///
/// The column list and the order clause are written once; each form adds its
/// own narrowing predicate between them. A form with no predicate is the
/// schema-wide one.
macro_rules! forms {
    (
        $columns:literal, $order:literal,
        $( $(#[$form:meta])* $name:ident => $narrowing:literal; )+
    ) => {
        $(
            $(#[$form])*
            const $name: &str = concat!($columns, $narrowing, $order);
        )+
    };
}

/// The schema row of `FR-CTX-036`: `name`, `charset` and `collation`.
///
/// The other three fields the schema catalogue returns are not read: the
/// catalogue-name field is row identity, and the SQL-path and schema-comment
/// fields were observed absent on every schema of every series, so
/// `BR-CAT-005` carries neither.
const SCHEMA: &str = "SELECT SCHEMA_NAME, DEFAULT_CHARACTER_SET_NAME, DEFAULT_COLLATION_NAME \
     FROM INFORMATION_SCHEMA.SCHEMATA WHERE SCHEMA_NAME = ?";

// The table read, whose properties are the index of `FR-CAT-053`.
//
// `TABLE_TYPE` is read because it is both the coverage predicate of
// `FR-CAT-001` through `FR-CAT-006` and the `table_type` of `FR-CAT-002`. The
// twelve volatile fields of `FR-CAT-024` this catalogue table also carries are
// named nowhere, so `FR-CAT-026` holds by construction: there is nothing to
// filter out downstream because nothing asked for them.
forms! {
    "SELECT TABLE_NAME, TABLE_TYPE, ENGINE, TABLE_COLLATION, TABLE_COMMENT \
     FROM INFORMATION_SCHEMA.TABLES WHERE TABLE_SCHEMA = ?",
    " ORDER BY TABLE_NAME",

    /// Every table of the schema, covered or not: the coverage filter is
    /// [`TableType::from_catalogue`](crate::model::table::TableType::from_catalogue),
    /// applied to these rows.
    TABLES => "";

    /// One named table.
    TABLE => " AND TABLE_NAME = ?";
}

// The column read (`FR-CAT-009`, `FR-CTX-040`, `FR-CAT-041`, `FR-CAT-051`).
//
// `CHARACTER_OCTET_LENGTH` is the ninth size field and is not a part:
// `FR-CTX-040` records that it is derivable from `length` and `charset`.
// `COLUMN_KEY` is not read — it is the restatement `FR-CTX-021` forbids, and
// `FR-CAT-043` bars it as a primary-key source — and neither is `PRIVILEGES`,
// which is not a property of the structure. `IS_GENERATED` is not read because
// `FR-CAT-051` forbids taking the storage kind from it: it says only
// *whether*, never *how*.
forms! {
    "SELECT TABLE_NAME, COLUMN_NAME, ORDINAL_POSITION, COLUMN_DEFAULT, IS_NULLABLE, DATA_TYPE, \
     CHARACTER_MAXIMUM_LENGTH, NUMERIC_PRECISION, NUMERIC_SCALE, DATETIME_PRECISION, \
     CHARACTER_SET_NAME, COLLATION_NAME, COLUMN_TYPE, EXTRA, COLUMN_COMMENT, \
     GENERATION_EXPRESSION FROM INFORMATION_SCHEMA.COLUMNS WHERE TABLE_SCHEMA = ?",
    " ORDER BY TABLE_NAME, ORDINAL_POSITION",

    /// Every column of the schema. `FR-CAT-052` is applied to these rows: a
    /// column whose owning object is not covered finds no table and is not
    /// presented.
    COLUMNS => "";

    /// The columns of one named table.
    COLUMNS_OF => " AND TABLE_NAME = ?";
}

// The index read (`FR-CAT-042`): nine of the seventeen fields.
//
// `CARDINALITY` is volatile (`FR-CAT-024`); `PACKED` was never observed
// populated; `NULLABLE` restates the column's own nullability (`FR-CTX-021`);
// the catalogue, schema and index-schema fields are row identity; and
// `COMMENT` — which is **not** `INDEX_COMMENT` — was never observed populated
// either. A reader that took `COMMENT` instead of the field beside it would
// report every index as uncommented.
//
// `SEQ_IN_INDEX` is ordered by and not selected: it is the column `FR-CAT-010`
// folds on, and the fold reads it as the order of the rows.
forms! {
    "SELECT TABLE_NAME, INDEX_NAME, NON_UNIQUE, COLUMN_NAME, COLLATION, SUB_PART, INDEX_TYPE, \
     INDEX_COMMENT, IGNORED FROM INFORMATION_SCHEMA.STATISTICS WHERE TABLE_SCHEMA = ?",
    " ORDER BY TABLE_NAME, INDEX_NAME, SEQ_IN_INDEX",

    /// Every index of the schema, the one named `PRIMARY` among them.
    INDEXES => "";

    /// The indexes of one named table.
    INDEXES_OF => " AND TABLE_NAME = ?";
}

// The rules half of a foreign key (`FR-CAT-045`). The catalogue, schema,
// unique-constraint catalogue and unique-constraint schema fields are row
// identity and are not read.
forms! {
    "SELECT TABLE_NAME, CONSTRAINT_NAME, UNIQUE_CONSTRAINT_NAME, MATCH_OPTION, UPDATE_RULE, \
     DELETE_RULE, REFERENCED_TABLE_NAME FROM INFORMATION_SCHEMA.REFERENTIAL_CONSTRAINTS \
     WHERE CONSTRAINT_SCHEMA = ?",
    " ORDER BY TABLE_NAME, CONSTRAINT_NAME",

    /// Every foreign key declared in the schema, which serves both the
    /// directions `FR-CAT-012` and `FR-CAT-013` present.
    RULES => "";

    /// The foreign keys one named table is at **either** end of. The
    /// disjunction is what makes one statement serve both directions for a
    /// named read, as the schema-wide form does for a full one.
    RULES_OF => " AND (TABLE_NAME = ? OR REFERENCED_TABLE_NAME = ?)";
}

// The columns half of a foreign key (`FR-CAT-045`).
//
// `ORDINAL_POSITION` is ordered by and not selected — it is the order of the
// referencing columns — and `POSITION_IN_UNIQUE_CONSTRAINT` is not selected
// because the pairing it states is already **on the row**: each row carries
// one referencing column beside the referenced column it points at, so the two
// lists cannot drift apart.
//
// `REFERENCED_TABLE_NAME IS NOT NULL` is what restricts the rows to foreign
// keys. The same catalogue table carries one row per column of every primary
// and unique key, and `FR-CAT-043` bars reading a primary key from it: on a
// system-versioned table those rows name the implicit period column, which the
// column catalogue carries for no table.
//
// `REFERENCED_TABLE_SCHEMA` is read for `FR-CAT-057`, and it is the one field
// of either foreign-key read that says where the referenced table lives. It is
// **selected rather than filtered on**, so the exclusion is made in the fold:
// a rules row for an outgoing cross-schema key is returned whatever this
// statement does — its constraint is declared in the schema the read covers,
// and the rules statement is filtered on that — so a key excluded here would
// otherwise reach a table through its rule with no columns at all. The fold
// excludes both halves together, and `super::fold` records why.
//
// *Rejected: excluding in SQL on the rules statement, through
// `UNIQUE_CONSTRAINT_SCHEMA`.* That field is the only cross-schema
// discriminant the rules table carries and `FR-CAT-056` records it as
// **nullable**; a predicate over it would therefore drop, silently, every key
// whose unique-constraint name is SQL `NULL` — which `FR-CTX-006` requires to
// be presented.
//
// `TABLE_SCHEMA = ?` excludes the **incoming** direction of `FR-CAT-057`: a key
// declared in another schema that references a table this read covers returns
// no row here at all. That was an accident of `FR-CAT-052`'s filter before
// `FR-CAT-057` was written and is now a requirement this predicate satisfies.
//
// The statement count is unchanged at eleven, which `NFR-PERF-001` and
// `NFR-PERF-002` fix: this is a column, not a statement.
forms! {
    "SELECT TABLE_NAME, CONSTRAINT_NAME, COLUMN_NAME, REFERENCED_TABLE_SCHEMA, \
     REFERENCED_TABLE_NAME, REFERENCED_COLUMN_NAME FROM INFORMATION_SCHEMA.KEY_COLUMN_USAGE \
     WHERE TABLE_SCHEMA = ? AND REFERENCED_TABLE_NAME IS NOT NULL",
    " ORDER BY TABLE_NAME, CONSTRAINT_NAME, ORDINAL_POSITION",

    /// Every foreign-key column of the schema.
    KEY_COLUMNS => "";

    /// The foreign-key columns of the keys one named table is at either end
    /// of.
    KEY_COLUMNS_OF => " AND (TABLE_NAME = ? OR REFERENCED_TABLE_NAME = ?)";
}

// The `CHECK` constraint read (`FR-CAT-046`): four of six fields, the other
// two being row identity.
forms! {
    "SELECT TABLE_NAME, CONSTRAINT_NAME, LEVEL, CHECK_CLAUSE \
     FROM INFORMATION_SCHEMA.CHECK_CONSTRAINTS WHERE CONSTRAINT_SCHEMA = ?",
    " ORDER BY TABLE_NAME, CONSTRAINT_NAME",

    /// Every `CHECK` constraint of the schema, the implicit `json_valid` ones
    /// of `FR-CAT-038` among them.
    CHECKS => "";

    /// The `CHECK` constraints of one named table.
    CHECKS_OF => " AND TABLE_NAME = ?";
}

// The trigger read (`FR-CAT-050`): fourteen of 22 fields.
//
// Four are row identity; the event object table is what the trigger is carried
// on; `ACTION_CONDITION` and the two action-reference **table** fields were
// never observed populated, so `BR-CAT-005` carries none of them; and
// `CREATED` is volatile under `FR-CAT-024` — a wall-clock time that differs
// between two containers of the same series, which would make `FR-SRV-026`
// unsatisfiable outright.
forms! {
    "SELECT EVENT_OBJECT_TABLE, TRIGGER_NAME, EVENT_MANIPULATION, ACTION_TIMING, ACTION_ORDER, \
     ACTION_STATEMENT, ACTION_ORIENTATION, ACTION_REFERENCE_OLD_ROW, ACTION_REFERENCE_NEW_ROW, \
     SQL_MODE, DEFINER, CHARACTER_SET_CLIENT, COLLATION_CONNECTION, DATABASE_COLLATION \
     FROM INFORMATION_SCHEMA.TRIGGERS WHERE EVENT_OBJECT_SCHEMA = ?",
    " ORDER BY EVENT_OBJECT_TABLE, TRIGGER_NAME",

    /// Every trigger of the schema.
    TRIGGERS => "";

    /// The triggers of one named table.
    TRIGGERS_OF => " AND EVENT_OBJECT_TABLE = ?";
}

// The view read (`FR-CAT-047`): nine of eleven fields, the other two being row
// identity.
//
// There is no comment among them, and the absence is `FR-CAT-040`: the row a
// view has in the table catalogue carries the literal `VIEW` in its comment
// field, which is a constant the server writes rather than text an author
// supplied.
forms! {
    "SELECT TABLE_NAME, VIEW_DEFINITION, CHECK_OPTION, IS_UPDATABLE, DEFINER, SECURITY_TYPE, \
     CHARACTER_SET_CLIENT, COLLATION_CONNECTION, ALGORITHM FROM INFORMATION_SCHEMA.VIEWS \
     WHERE TABLE_SCHEMA = ?",
    " ORDER BY TABLE_NAME",

    /// Every view of the schema (`FR-CAT-007`).
    VIEWS => "";

    /// One named view.
    VIEW => " AND TABLE_NAME = ?";
}

// The routine read (`FR-CAT-048`): 22 of 31 fields.
//
// Two are row identity; `SPECIFIC_NAME` restates the routine name;
// `CHARACTER_OCTET_LENGTH` is the size field `FR-CTX-040` does not carry;
// `EXTERNAL_NAME`, `EXTERNAL_LANGUAGE` and `SQL_PATH` were never observed
// populated; and `CREATED` and `LAST_ALTERED` are volatile under `FR-CAT-024`.
forms! {
    "SELECT ROUTINE_NAME, ROUTINE_TYPE, DTD_IDENTIFIER, DATA_TYPE, CHARACTER_MAXIMUM_LENGTH, \
     NUMERIC_PRECISION, NUMERIC_SCALE, DATETIME_PRECISION, CHARACTER_SET_NAME, COLLATION_NAME, \
     ROUTINE_BODY, ROUTINE_DEFINITION, PARAMETER_STYLE, IS_DETERMINISTIC, SQL_DATA_ACCESS, \
     SECURITY_TYPE, SQL_MODE, ROUTINE_COMMENT, DEFINER, CHARACTER_SET_CLIENT, \
     COLLATION_CONNECTION, DATABASE_COLLATION FROM INFORMATION_SCHEMA.ROUTINES \
     WHERE ROUTINE_SCHEMA = ?",
    " ORDER BY ROUTINE_NAME, ROUTINE_TYPE",

    /// Every routine of the schema — procedures and functions together, per
    /// `FR-CAT-008`.
    ROUTINES => "";

    /// Every routine of one name, whichever kind. A bare name can match two
    /// objects, which `FR-SCH-010` makes the caller's ambiguity to report.
    ROUTINES_NAMED => " AND ROUTINE_NAME = ?";

    /// One routine of one name and one kind — the qualified form of
    /// `FR-SCH-008`, narrowed in SQL rather than by discarding the other.
    ROUTINE_QUALIFIED => " AND ROUTINE_NAME = ? AND ROUTINE_TYPE = ?";
}

// The routine-parameter read (`FR-CAT-049`).
//
// `ORDINAL_POSITION > 0` is what excludes a function's **return row**, which
// `FR-CAT-049` states is not a parameter: it sits at position `0` with both the
// mode and the name SQL `NULL`, and the return type the model carries is the
// one the routine row states. Excluding it in SQL rather than in the fold
// keeps the shape `RoutineParameter` refuses — a parameter with no name —
// unreachable.
//
// `PARAMETER_DEFAULT` is not named: it exists on `12.3` alone, difference 6 of
// `FR-SRV-038`, and naming it would make this statement fail outright on the
// other three series.
forms! {
    "SELECT SPECIFIC_NAME, ROUTINE_TYPE, PARAMETER_NAME, PARAMETER_MODE, DTD_IDENTIFIER, \
     DATA_TYPE, CHARACTER_MAXIMUM_LENGTH, NUMERIC_PRECISION, NUMERIC_SCALE, DATETIME_PRECISION, \
     CHARACTER_SET_NAME, COLLATION_NAME FROM INFORMATION_SCHEMA.PARAMETERS \
     WHERE SPECIFIC_SCHEMA = ? AND ORDINAL_POSITION > 0",
    " ORDER BY SPECIFIC_NAME, ROUTINE_TYPE, ORDINAL_POSITION",

    /// Every declared parameter of every routine of the schema.
    PARAMETERS => "";

    /// The parameters of every routine of one name.
    PARAMETERS_NAMED => " AND SPECIFIC_NAME = ?";

    /// The parameters of one routine of one name and one kind.
    PARAMETERS_QUALIFIED => " AND SPECIFIC_NAME = ? AND ROUTINE_TYPE = ?";
}

/// What one catalogue statement read, which is how the fold finds its rows
/// again.
///
/// The discriminant is the object kind and not the statement text, because a
/// read has more than one text — the schema-wide one and the named ones — and
/// the fold treats their rows identically.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum Read {
    /// `SCHEMATA` (`FR-CTX-036`).
    Schema,
    /// `TABLES` (`FR-CAT-001`, `FR-CAT-053`).
    Tables,
    /// `COLUMNS` (`FR-CAT-009`).
    Columns,
    /// `STATISTICS` (`FR-CAT-042`, `FR-CAT-043`).
    Indexes,
    /// `REFERENTIAL_CONSTRAINTS` (`FR-CAT-045`).
    ForeignKeyRules,
    /// `KEY_COLUMN_USAGE` (`FR-CAT-045`).
    KeyColumns,
    /// `CHECK_CONSTRAINTS` (`FR-CAT-046`).
    Checks,
    /// `TRIGGERS` (`FR-CAT-050`).
    Triggers,
    /// `VIEWS` (`FR-CAT-047`).
    Views,
    /// `ROUTINES` (`FR-CAT-048`).
    Routines,
    /// `PARAMETERS` (`FR-CAT-049`).
    Parameters,
}

impl Read {
    /// Every member, in the order a full read issues them.
    pub(super) const ALL: [Self; 11] = [
        Self::Schema,
        Self::Tables,
        Self::Columns,
        Self::Indexes,
        Self::ForeignKeyRules,
        Self::KeyColumns,
        Self::Checks,
        Self::Triggers,
        Self::Views,
        Self::Routines,
        Self::Parameters,
    ];

    /// The number of object kinds, which is the length of the full repertoire
    /// and the number of slots the fetched rows occupy.
    pub(super) const COUNT: usize = Self::ALL.len();

    /// The slot this read's rows are kept in.
    pub(super) const fn slot(self) -> usize {
        match self {
            Self::Schema => 0,
            Self::Tables => 1,
            Self::Columns => 2,
            Self::Indexes => 3,
            Self::ForeignKeyRules => 4,
            Self::KeyColumns => 5,
            Self::Checks => 6,
            Self::Triggers => 7,
            Self::Views => 8,
            Self::Routines => 9,
            Self::Parameters => 10,
        }
    }
}

/// What a read covers (`FR-SCH-004`, `FR-SCH-005`, `NFR-PERF-002`).
///
/// A named scope narrows **in SQL**, through the predicate its statement
/// carries, and never by reading the whole catalogue and discarding what does
/// not match: `NFR-PERF-002` fixes the count of a named read independently of
/// how many objects the database holds, and a discard would make the rows
/// depend on it even where the statement count did not.
///
/// The type is [`Clone`] and not [`Copy`], because [`Scope::Routine`] carries a
/// [`RoutineKind`] and `FR-CAT-055` gives that enumeration a variant holding
/// the catalogue's own string. [`plan`] therefore takes it by reference.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum Scope<'a> {
    /// The whole schema: every covered table, every view, every routine.
    Everything,

    /// One table, with everything `FR-CAT-053` gives a table.
    ///
    /// No command constructs it, and **that is not a gap**. `FR-CTX-006` and
    /// `FR-CTX-010` embed, in full, the table at each end of every foreign key
    /// and `FR-CTX-023` requires every referenced object to be present, so a
    /// plan that reads one table row returns that table's keys without the
    /// tables they name and no document can be built from it. `NFR-PERF-002`
    /// settles what that costs: it counts **statements**, the rows such a read
    /// returns MAY be the whole catalogue, and a read that returns them SHALL
    /// NOT be taken to violate it. A whole-catalogue read is therefore the
    /// admissible plan rather than a shortfall, and the requirement's own
    /// *Accepted cost* records the wall-clock and memory price against
    /// `NFR-PERF-014` and the `WL-002` scalar.
    ///
    /// The plan is kept because it is the repertoire's own statement of what a
    /// narrowed table read would issue, and because `NFR-PERF-002` keeps the
    /// option open in as many words: a narrow read that returned the
    /// neighbours of the named table would also satisfy it, and choosing
    /// between the two is an architecture decision rather than a functional
    /// one.
    #[allow(
        dead_code,
        reason = "every command reads the whole catalogue because FR-CTX-006, FR-CTX-010 and \
                  FR-CTX-023 make a one-table plan unable to produce a document; NFR-PERF-002 \
                  as amended admits that read, counting statements rather than rows, so this \
                  plan records the narrowed read the requirement leaves open"
    )]
    Table(&'a str),

    /// One view.
    ///
    /// No command constructs it either, and the cause is **not** the one
    /// [`Scope::Table`] records: nothing about a view's document needs a second
    /// object. It is `FR-SCH-010`, which obliges a name that reaches no object
    /// to carry a nearest-match suggestion drawn from the objects of that kind
    /// that do exist — a population only a collection read has in hand.
    #[allow(
        dead_code,
        reason = "FR-SCH-010 draws its suggestion from the objects of that kind that exist, \
                  which a narrowed plan does not read"
    )]
    View(&'a str),

    /// One routine. [`None`] for the kind is the bare name of `FR-SCH-008`,
    /// which may match a procedure **and** a function; the caller decides what
    /// two matches mean, per `FR-SCH-010`.
    ///
    /// No command constructs it, for the cause [`Scope::View`] records and not
    /// the one [`Scope::Table`] does.
    #[allow(
        dead_code,
        reason = "FR-SCH-010 draws its suggestion from the objects of that kind that exist, \
                  which a narrowed plan does not read"
    )]
    Routine {
        /// The routine's name, without the qualifying prefix.
        name: &'a str,
        /// The kind the qualified form named, or [`None`] for a bare name.
        ///
        /// Only a kind `FR-SCH-008` admits reaches here, because the qualified
        /// form carries one of two prefixes and no third. [`plan`] nevertheless
        /// narrows on [`RoutineKind::recorded`] and falls back to the bare-name
        /// plan where a kind has no recorded spelling, which is `FR-CAT-055`'s
        /// own consequence: an unrecorded kind is carried in the document and
        /// is not reachable by a qualified name.
        kind: Option<RoutineKind<'a>>,
    },
}

/// One statement of the plan: its text, and the values bound to it.
///
/// The text is a `&'static str` and every value the statement varies by is a
/// bind, so nothing a caller supplies is ever composed into SQL.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) struct Statement<'a> {
    /// What this statement reads.
    read: Read,

    /// The text, which is one of the constants of this module.
    sql: &'static str,

    /// The values bound to it, in the order its placeholders appear. The
    /// leading [`Some`] entries are the binds; the rest are empty.
    binds: [Option<&'a str>; MAX_BINDS],
}

impl<'a> Statement<'a> {
    /// What this statement reads.
    pub(super) const fn read(&self) -> Read {
        self.read
    }

    /// The text, which is what an observation of the statements issued
    /// compares against.
    pub(super) const fn sql(&self) -> &'static str {
        self.sql
    }

    /// The values to bind, in placeholder order.
    pub(super) fn binds(&self) -> impl Iterator<Item = &'a str> {
        self.binds.into_iter().flatten()
    }
}

/// One statement with only the schema bound.
const fn one<'a>(read: Read, sql: &'static str, schema: &'a str) -> Statement<'a> {
    Statement {
        read,
        sql,
        binds: [Some(schema), None, None],
    }
}

/// One statement with the schema and one further value bound.
const fn two<'a>(read: Read, sql: &'static str, schema: &'a str, second: &'a str) -> Statement<'a> {
    Statement {
        read,
        sql,
        binds: [Some(schema), Some(second), None],
    }
}

/// One statement with the schema and two further values bound.
const fn three<'a>(
    read: Read,
    sql: &'static str,
    schema: &'a str,
    second: &'a str,
    third: &'a str,
) -> Statement<'a> {
    Statement {
        read,
        sql,
        binds: [Some(schema), Some(second), Some(third)],
    }
}

/// The statements one read issues, in the order they are issued
/// (`NFR-PERF-001`, `NFR-PERF-002`).
///
/// The plan is decided **here**, from the schema name and the scope alone. It
/// sees no row and no count, so the length of what it returns cannot depend on
/// the number of objects the database holds — which is the whole of what
/// `NFR-PERF-001` and `NFR-PERF-002` require, stated where it can be asserted
/// rather than reviewed.
///
/// | Scope | Statements | Why that many |
/// |---|---|---|
/// | [`Scope::Everything`] | 11 | One per object kind |
/// | [`Scope::Table`] | 8 | The schema, the table row, and the six reads of the members `FR-CAT-053` gives a table |
/// | [`Scope::View`] | 2 | The schema and the view row; `FR-CAT-047` gives a view no member collection |
/// | [`Scope::Routine`] | 3 | The schema, the routine row, and its parameters |
pub(super) fn plan<'a>(schema: &'a str, scope: &'a Scope<'a>) -> Vec<Statement<'a>> {
    let schema_row = one(Read::Schema, SCHEMA, schema);

    match *scope {
        Scope::Everything => vec![
            schema_row,
            one(Read::Tables, TABLES, schema),
            one(Read::Columns, COLUMNS, schema),
            one(Read::Indexes, INDEXES, schema),
            one(Read::ForeignKeyRules, RULES, schema),
            one(Read::KeyColumns, KEY_COLUMNS, schema),
            one(Read::Checks, CHECKS, schema),
            one(Read::Triggers, TRIGGERS, schema),
            one(Read::Views, VIEWS, schema),
            one(Read::Routines, ROUTINES, schema),
            one(Read::Parameters, PARAMETERS, schema),
        ],

        Scope::Table(name) => vec![
            schema_row,
            two(Read::Tables, TABLE, schema, name),
            two(Read::Columns, COLUMNS_OF, schema, name),
            two(Read::Indexes, INDEXES_OF, schema, name),
            three(Read::ForeignKeyRules, RULES_OF, schema, name, name),
            three(Read::KeyColumns, KEY_COLUMNS_OF, schema, name, name),
            two(Read::Checks, CHECKS_OF, schema, name),
            two(Read::Triggers, TRIGGERS_OF, schema, name),
        ],

        Scope::View(name) => vec![schema_row, two(Read::Views, VIEW, schema, name)],

        // FR-CAT-055: a kind with no recorded spelling narrows nothing, so the
        // qualified plan falls back to the bare-name one. The requirement
        // names this as the one place a third kind is not reachable, and the
        // fall-back is what keeps its bare name reaching it, per `FR-SCH-010`.
        Scope::Routine { name, ref kind } => match kind.as_ref().and_then(RoutineKind::recorded) {
            None => vec![
                schema_row,
                two(Read::Routines, ROUTINES_NAMED, schema, name),
                two(Read::Parameters, PARAMETERS_NAMED, schema, name),
            ],
            Some(recorded) => vec![
                schema_row,
                three(Read::Routines, ROUTINE_QUALIFIED, schema, name, recorded),
                three(
                    Read::Parameters,
                    PARAMETERS_QUALIFIED,
                    schema,
                    name,
                    recorded,
                ),
            ],
        },
    }
}

#[cfg(test)]
mod tests {
    use super::{Read, Scope, Statement, plan};
    use crate::model::routine::RoutineKind;

    /// The eleven statements of a full read, which is the count
    /// `NFR-PERF-001` fixes.
    const FULL_READ: usize = 11;

    #[test]
    fn nfr_perf_001_a_full_read_issues_one_statement_per_object_kind_and_no_more() {
        // NFR-PERF-001: the count is the repertoire's size, and the repertoire
        // has one entry per object kind. FR-CAT-043 is why there is no twelfth
        // for the primary key, and FR-CAT-045 why there is none for the
        // incoming direction of a foreign key.
        let statements = plan("freight", &Scope::Everything);

        assert_eq!(statements.len(), FULL_READ);
        assert_eq!(statements.len(), Read::COUNT);

        let mut reads: Vec<usize> = statements
            .iter()
            .map(|statement| statement.read().slot())
            .collect();
        reads.sort_unstable();
        reads.dedup();

        assert_eq!(reads.len(), FULL_READ, "one statement per object kind");
    }

    #[test]
    fn fr_cat_057_the_key_column_read_names_the_referenced_schema_and_stays_one_statement() {
        // FR-CAT-057 needs to know where a referenced table lives, and the
        // referenced-schema field of the key-column table is the only place
        // either foreign-key read says so. It is a **column**, not a twelfth
        // statement: NFR-PERF-001 and NFR-PERF-002 fix the count, and the
        // exclusion is made in the fold.
        let statements = plan("freight", &Scope::Everything);

        assert_eq!(statements.len(), FULL_READ, "the count is unchanged");

        let key_columns = statements
            .iter()
            .find(|statement| statement.read() == Read::KeyColumns)
            .expect("the repertoire reads KEY_COLUMN_USAGE");

        assert!(
            key_columns.sql().contains("REFERENCED_TABLE_SCHEMA"),
            "{}",
            key_columns.sql()
        );

        // The incoming direction of FR-CAT-057 is excluded by this predicate
        // and by nothing else: a key declared in another schema that
        // references a table this read covers returns no row at all.
        assert!(
            key_columns.sql().contains("TABLE_SCHEMA = ?"),
            "{}",
            key_columns.sql()
        );

        // *Rejected: excluding in SQL on the rules statement.* Its only
        // cross-schema discriminant is the unique-constraint schema, which
        // FR-CAT-056 records as nullable, so a predicate over it would drop
        // every key whose unique-constraint name is SQL `NULL` — which
        // FR-CTX-006 requires to be presented.
        let rules = statements
            .iter()
            .find(|statement| statement.read() == Read::ForeignKeyRules)
            .expect("the repertoire reads REFERENTIAL_CONSTRAINTS");

        assert!(
            !rules.sql().contains("UNIQUE_CONSTRAINT_SCHEMA"),
            "{}",
            rules.sql()
        );
    }

    #[test]
    fn fr_cat_055_a_routine_kind_with_no_recorded_spelling_falls_back_to_the_bare_name_plan() {
        // FR-CAT-055 names the qualified name of FR-SCH-008 as the one place a
        // kind outside the recorded two is not reachable. The plan reads the
        // recorded spelling, so such a kind narrows on the name alone rather
        // than binding a string no requirement fixes to ROUTINE_TYPE.
        let unrecorded = plan(
            "freight",
            &Scope::Routine {
                name: "fn_chargeable_weight",
                kind: Some(RoutineKind::Unrecorded(std::borrow::Cow::Borrowed(
                    "PACKAGE",
                ))),
            },
        );
        let bare = plan(
            "freight",
            &Scope::Routine {
                name: "fn_chargeable_weight",
                kind: None,
            },
        );

        assert_eq!(unrecorded.len(), bare.len());

        for (unrecorded, bare) in unrecorded.iter().zip(&bare) {
            assert_eq!(unrecorded.sql(), bare.sql());
        }
    }

    #[test]
    fn nfr_perf_001_the_plan_of_a_full_read_does_not_depend_on_what_the_database_holds() {
        // NFR-PERF-001 requires the count over WL-001 to equal the count over
        // WL-003 — a database of 200 tables against one of a single table.
        // The plan is a function of the schema name and the scope and of
        // nothing else, so two plans differ in the bound schema name and in
        // nothing besides.
        let large = plan("two_hundred_tables", &Scope::Everything);
        let small = plan("one_table", &Scope::Everything);

        assert_eq!(large.len(), small.len());

        for (large, small) in large.iter().zip(&small) {
            assert_eq!(large.read(), small.read());
            assert_eq!(large.sql(), small.sql());
        }
    }

    #[test]
    fn nfr_perf_002_a_named_read_issues_a_count_of_its_own_and_narrows_in_sql() {
        // NFR-PERF-002: the count of a named read does not depend on how many
        // objects the database holds either. Every statement of a named plan
        // but the schema row carries a predicate naming the object, so nothing
        // is read and discarded.
        let table = plan("freight", &Scope::Table("consignment"));
        let view = plan("freight", &Scope::View("v_consignment_manifest"));
        let bare = plan(
            "freight",
            &Scope::Routine {
                name: "fn_chargeable_weight",
                kind: None,
            },
        );
        let qualified = plan(
            "freight",
            &Scope::Routine {
                name: "fn_chargeable_weight",
                kind: Some(RoutineKind::Function),
            },
        );

        assert_eq!(table.len(), 8);
        assert_eq!(view.len(), 2);
        assert_eq!(bare.len(), 3);
        assert_eq!(qualified.len(), 3);

        for plan in [&table, &view, &bare, &qualified] {
            for statement in plan.iter().skip(1) {
                assert!(
                    statement.binds().count() > 1,
                    "{} narrows nothing",
                    statement.sql()
                );
            }
        }
    }

    #[test]
    fn fr_sch_008_a_qualified_routine_name_binds_the_kind_and_a_bare_one_does_not() {
        // FR-SCH-008 admits `procedure:<name>` and `function:<name>` beside
        // the bare name, and FR-SCH-010 makes two matches on a bare name the
        // caller's ambiguity to report — so the reader narrows by kind only
        // where the caller supplied one.
        let qualified = plan(
            "freight",
            &Scope::Routine {
                name: "sp_recalculate_freight",
                kind: Some(RoutineKind::Procedure),
            },
        );
        let routines = of(&qualified, Read::Routines);

        assert!(routines.sql().contains("ROUTINE_TYPE = ?"));
        assert_eq!(
            routines.binds().collect::<Vec<_>>(),
            ["freight", "sp_recalculate_freight", "PROCEDURE"]
        );

        let bare = plan(
            "freight",
            &Scope::Routine {
                name: "sp_recalculate_freight",
                kind: None,
            },
        );

        assert!(!of(&bare, Read::Routines).sql().contains("ROUTINE_TYPE = ?"));
    }

    #[test]
    fn fr_srv_037_no_statement_names_a_column_absent_from_a_series_of_the_window() {
        // FR-SRV-037's second answer: the column list is restricted to the
        // columns common to all four series, so no list is selected from the
        // resolved series and none has to be. The three columns below are the
        // ones differences 5 and 6 of FR-SRV-038 record as present on some
        // series and not on others; none is named, and a statement naming one
        // would be ERROR 1054 on a supported server.
        const ABSENT_SOMEWHERE: [&str; 3] = [
            "IS_SYSTEM_TIME_PERIOD_START",
            "IS_SYSTEM_TIME_PERIOD_END",
            "PARAMETER_DEFAULT",
        ];

        for statement in every_statement() {
            for column in ABSENT_SOMEWHERE {
                assert!(
                    !statement.contains(column),
                    "{statement} names {column}, which a series of FR-SRV-015 does not have"
                );
            }

            // Difference 7: INFORMATION_SCHEMA.PERIODS is absent on 10.11
            // altogether, and FR-CAT-022 excludes what it would carry.
            assert!(!statement.contains("PERIODS"), "{statement}");
        }
    }

    #[test]
    fn fr_srv_007_every_statement_is_one_select_against_information_schema() {
        // FR-SRV-007: no DDL, no DML, no SHOW, and no statement against any
        // schema other than INFORMATION_SCHEMA. FR-SRV-006's first entry is
        // the only one this module writes.
        for statement in every_statement() {
            assert!(statement.starts_with("SELECT "), "{statement}");
            assert_eq!(
                statement.matches("FROM ").count(),
                1,
                "{statement} reads more than one table"
            );
            assert!(
                statement.contains("FROM INFORMATION_SCHEMA."),
                "{statement}"
            );

            // The comparison is over whole words: `DELETE_RULE` and
            // `UPDATE_RULE` are catalogue fields of `FR-CAT-045`, and a
            // substring test would read either as a statement keyword.
            let words: Vec<&str> = statement
                .split(|character: char| {
                    !(character.is_ascii_alphanumeric() || character == '_' || character == '.')
                })
                .collect();

            for barred in [
                "SHOW", "INSERT", "UPDATE", "DELETE", "CREATE", "DROP", "ALTER", "REPLACE",
                "TRUNCATE", "GRANT", "SET",
            ] {
                assert!(
                    !words.contains(&barred),
                    "{statement} carries the statement keyword {barred}"
                );
            }
        }
    }

    #[test]
    fn fr_cat_024_no_statement_asks_for_a_volatile_field() {
        // FR-CAT-024 and FR-CAT-026: the exclusion holds for every consumer
        // without exception, and it holds here by construction — the fields
        // are not asked for, so there is nothing to filter out later.
        // FR-CAT-027 keeps the column attribute `auto_increment`, which
        // arrives inside EXTRA and is a value rather than a catalogue column,
        // so excluding the column of that name costs the model nothing.
        const VOLATILE: [&str; 16] = [
            "TABLE_ROWS",
            "AVG_ROW_LENGTH",
            "DATA_LENGTH",
            "MAX_DATA_LENGTH",
            "INDEX_LENGTH",
            "DATA_FREE",
            "AUTO_INCREMENT",
            "CREATE_TIME",
            "UPDATE_TIME",
            "CHECK_TIME",
            "CHECKSUM",
            "VERSION",
            "CARDINALITY",
            "CREATED",
            "LAST_ALTERED",
            "PACKED",
        ];

        for statement in every_statement() {
            for field in VOLATILE {
                assert!(
                    !statement.contains(field),
                    "{statement} asks for the volatile field {field}"
                );
            }
        }
    }

    #[test]
    fn fr_cat_043_the_primary_key_has_no_statement_of_its_own() {
        // FR-CAT-043: the index catalogue is the authoritative source and the
        // other two are barred. The key-column-usage read is restricted to
        // foreign-key rows, so the rows that would carry a primary key — and
        // that name the implicit period column the model does not carry — are
        // never returned; and no statement reads the constraint table at all.
        let statements = plan("freight", &Scope::Everything);

        assert!(
            of(&statements, Read::KeyColumns)
                .sql()
                .contains("REFERENCED_TABLE_NAME IS NOT NULL")
        );

        for statement in every_statement() {
            assert!(!statement.contains("TABLE_CONSTRAINTS"), "{statement}");
        }
    }

    #[test]
    fn fr_cat_042_the_index_read_takes_the_index_comment_and_not_the_comment_beside_it() {
        // FR-CAT-042: the two fields are different fields, and the text a DDL
        // writes as `KEY … COMMENT '…'` lands in the second. A reader that
        // took the first would report every index as uncommented.
        let statements = plan("freight", &Scope::Everything);
        let indexes = of(&statements, Read::Indexes);

        assert!(indexes.sql().contains("INDEX_COMMENT"));
        assert_eq!(indexes.sql().matches("COMMENT").count(), 1);
    }

    /// The one statement of `plan` that performs `read`.
    fn of<'a>(statements: &'a [Statement<'a>], read: Read) -> &'a Statement<'a> {
        statements
            .iter()
            .find(|statement| statement.read() == read)
            .expect("the plan performs this read")
    }

    /// Every statement text the repertoire can issue, over every scope.
    fn every_statement() -> Vec<&'static str> {
        let scopes = [
            Scope::Everything,
            Scope::Table("consignment"),
            Scope::View("v_consignment_manifest"),
            Scope::Routine {
                name: "fn_chargeable_weight",
                kind: None,
            },
            Scope::Routine {
                name: "fn_chargeable_weight",
                kind: Some(RoutineKind::Function),
            },
        ];

        scopes
            .iter()
            .flat_map(|scope| plan("freight", scope))
            .map(|statement| statement.sql())
            .collect()
    }
}
