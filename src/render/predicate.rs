//! The seven tests of `FR-ENV-014`.
//!
//! The module is named for what a test *is* rather than for what a template
//! calls it, because `tests` is the name Rust gives a unit-test module and one
//! file may not mean both.
//!
//! Every one of the seven accepts a column object and nothing else, per
//! `FR-ENV-040`, and none of them answers `false` for an operand it does not
//! accept, per `FR-SEM-007`: the refusal is [`super::operand`]'s and it names
//! the test, the type received and the location.
//!
//! | Test | True when | Answers from |
//! |---|---|---|
//! | `nullable` | The column admits `NULL` | The column alone |
//! | `auto_increment` | The column carries the attribute of `FR-CAT-027` | The column alone |
//! | `primary_key` | The column is named in its table's primary key | The render context |
//! | `unique` | The column is named in an index of its table the model reports as unique, or in its primary key | The render context |
//! | `numeric` | The column's `data_type` is in the numeric family | The column alone |
//! | `temporal` | The column's `data_type` is in the date-and-time family | The column alone |
//! | `textual` | The column's `data_type` is in the character-string family | The column alone |
//!
//! # The three families
//!
//! `FR-ENV-046` fixes their membership over the `data_type` values a series of
//! `FR-SRV-015` was observed to produce, and `FR-ENV-042` makes them disjoint
//! and total: a column of a type in none of them satisfies none of the three
//! and does not fail the render. That is what a template depends on —
//! `{% if col is numeric %}…{% elif col is textual %}` must not take two
//! branches, and a spatial column must be able to fall through to neither.
//!
//! An unrecognised type carries `null` as its `data_type`, per `FR-CTX-018`,
//! and a `null` is a value in none of the three rows rather than a failure.
//!
//! # The two that reach the context
//!
//! `FR-ENV-015` makes `primary_key` and `unique` resolve `table_name` against
//! the render context, and `FR-ENV-043` makes a table that is absent a `65`
//! rather than a `false`. Both read the table's key from its `primary_key`,
//! never from an index found by the name `PRIMARY`: a SQLite key is named after
//! its index of origin or has no index at all (`FR-LITE-029`), and
//! `primary_key` is the one place every table's key is held. `unique` also
//! answers from the index collection (`FR-ENV-041`).

use minijinja::{Error, State, Value};

use super::lookup;
use super::operand::{Column, Role};

/// The `numeric` family of `FR-ENV-046`, with the three SQLite affinities
/// `FR-ENV-048` places in it, sorted.
const NUMERIC: [&str; 12] = [
    "bigint",
    "bit",
    "decimal",
    "double",
    "float",
    "int",
    "integer",
    "mediumint",
    "numeric",
    "real",
    "smallint",
    "tinyint",
];

/// The `temporal` family of `FR-ENV-046`, sorted.
const TEMPORAL: [&str; 5] = ["date", "datetime", "time", "timestamp", "year"];

/// The `textual` family of `FR-ENV-046`, sorted.
///
/// It is exactly the set of types for which the catalogue reports a character
/// set and a collation, per `FR-CTX-041`, which is why the binary and the blob
/// types are not in it although they carry a length.
const TEXTUAL: [&str; 8] = [
    "char",
    "enum",
    "longtext",
    "mediumtext",
    "set",
    "text",
    "tinytext",
    "varchar",
];

/// The `textual` row of `FR-ENV-049`: the one value a `mysql` read produces,
/// and no series of `FR-SRV-015` was observed to produce, that belongs to a
/// family.
///
/// `json` is placed by analogy with the `longtext` a `mariadb` read reports for
/// the same declaration. The other two values `FR-ENV-049` places,
/// `geomcollection` and `vector`, belong to no family, which the fall-through
/// of `FR-ENV-046` already gives them, so they need no entry. The row is kept
/// apart from [`TEXTUAL`] because it rests on another requirement and another
/// observation, and no answer for a `mariadb` or a `sqlite` read moves on
/// account of it.
const MYSQL_TEXTUAL: [&str; 1] = ["json"];

/// `nullable` (`FR-ENV-041`).
///
/// # Errors
///
/// Returns the refusal of `FR-ENV-040` for an operand that is not a column.
pub(super) fn nullable(value: &Value) -> Result<bool, Error> {
    Column::of(value, Role::Test, "nullable")?.flag("nullable")
}

/// `auto_increment` (`FR-ENV-041`, `FR-CAT-027`).
///
/// This is the column attribute. The table-level counter of the same name is
/// excluded outright by `FR-CAT-024` and exists nowhere in the model.
///
/// # Errors
///
/// Returns the refusal of `FR-ENV-040` for an operand that is not a column.
pub(super) fn auto_increment(value: &Value) -> Result<bool, Error> {
    Column::of(value, Role::Test, "auto_increment")?.flag("auto_increment")
}

/// `numeric` (`FR-ENV-041`, `FR-ENV-046`).
///
/// # Errors
///
/// Returns the refusal of `FR-ENV-040` for an operand that is not a column.
pub(super) fn numeric(value: &Value) -> Result<bool, Error> {
    family(value, "numeric", &NUMERIC)
}

/// `temporal` (`FR-ENV-041`, `FR-ENV-046`).
///
/// # Errors
///
/// Returns the refusal of `FR-ENV-040` for an operand that is not a column.
pub(super) fn temporal(value: &Value) -> Result<bool, Error> {
    family(value, "temporal", &TEMPORAL)
}

/// `textual` (`FR-ENV-041`, `FR-ENV-046`, `FR-ENV-049`).
///
/// # Errors
///
/// Returns the refusal of `FR-ENV-040` for an operand that is not a column.
pub(super) fn textual(value: &Value) -> Result<bool, Error> {
    Ok(family(value, "textual", &TEXTUAL)? || family(value, "textual", &MYSQL_TEXTUAL)?)
}

/// `primary_key` (`FR-ENV-041`, `FR-ENV-015`).
///
/// # Errors
///
/// Returns the refusal of `FR-ENV-040` for an operand that is not a column,
/// and the `65` of `FR-ENV-017` and `FR-ENV-043` where the table the column
/// names is absent from the render context.
pub(super) fn primary_key(state: &State<'_, '_>, value: &Value) -> Result<bool, Error> {
    let column = Column::of(value, Role::Test, "primary_key")?;
    let table = lookup::table_of(state, &column, "primary_key")?;
    let name = column.name()?;

    // FR-ENV-041: the table's `primary_key`, never an index found by the
    // name `PRIMARY` — a SQLite key is named after its index of origin, or
    // has no index at all (FR-LITE-029).
    Ok(lookup::primary_key(&table).is_some_and(|key| lookup::covers(&key, &name)))
}

/// `unique` (`FR-ENV-041`, `FR-ENV-015`).
///
/// True where the column is named in an index the model reports unique, or in
/// the table's primary key (`FR-ENV-041`, as amended in the sixty-seventh
/// edition): a SQLite rowid alias is a primary key with no index
/// (`FR-LITE-029`). On a MariaDB document every primary key is also the unique
/// index `PRIMARY` (`FR-CAT-043`), so the second clause changes no answer
/// there.
///
/// # Errors
///
/// Returns the refusal of `FR-ENV-040` for an operand that is not a column,
/// and the `65` of `FR-ENV-017` and `FR-ENV-043` where the table the column
/// names is absent from the render context.
pub(super) fn unique(state: &State<'_, '_>, value: &Value) -> Result<bool, Error> {
    let column = Column::of(value, Role::Test, "unique")?;
    let table = lookup::table_of(state, &column, "unique")?;
    let name = column.name()?;

    Ok(lookup::indexes(&table)
        .filter(lookup::unique)
        .any(|index| lookup::covers(&index, &name))
        || lookup::primary_key(&table).is_some_and(|key| lookup::covers(&key, &name)))
}

/// The three family tests, written once.
fn family(value: &Value, name: &str, members: &[&str]) -> Result<bool, Error> {
    let data_type = Column::of(value, Role::Test, name)?.data_type()?;

    // FR-CTX-018 and FR-ENV-046: a type the system does not recognise carries
    // `null` here, and a value in none of the four rows satisfies none of the
    // three tests rather than failing the render.
    Ok(data_type
        .as_str()
        .is_some_and(|data_type| members.binary_search(&data_type).is_ok()))
}

#[cfg(test)]
mod tests {
    use super::{NUMERIC, TEMPORAL, TEXTUAL, auto_increment, nullable, numeric, temporal, textual};
    use crate::render::{fixture, surface};
    use minijinja::{Environment, UndefinedBehavior, Value};

    /// The three affinities of `FR-ENV-048` the numeric family gains, which no
    /// MariaDB read produces and which [`fixture::column_of_affinity`] builds.
    const AFFINITIES: [&str; 3] = ["integer", "numeric", "real"];

    /// The `data_type` values of `FR-ENV-046` that belong to no family.
    const NEITHER: [&str; 17] = [
        "binary",
        "blob",
        "geometry",
        "geometrycollection",
        "inet4",
        "inet6",
        "linestring",
        "longblob",
        "mediumblob",
        "multilinestring",
        "multipoint",
        "multipolygon",
        "point",
        "polygon",
        "tinyblob",
        "uuid",
        "varbinary",
    ];

    /// Whether `operand` satisfies the registered test `name`, in `context`.
    ///
    /// The two derived tests answer from the render context, so they are
    /// exercised through a render rather than through a state built by hand:
    /// the engine is the only party that can supply one. The answer is read
    /// from a branch rather than from an interpolated boolean, because no
    /// requirement fixes how the engine writes one.
    fn answer(name: &str, context: &Value) -> Result<bool, minijinja::Error> {
        let mut engine = Environment::new();
        engine.set_undefined_behavior(UndefinedBehavior::Strict);
        surface::register(&mut engine);

        let source = format!("{{% if operand is {name} %}}yes{{% else %}}no{{% endif %}}");

        engine
            .render_str(&source, context)
            .map(|written| written == "yes")
    }

    #[test]
    fn fr_env_046_every_observed_data_type_lands_in_exactly_one_row() {
        // FR-ENV-046, every row of the table, and FR-ENV-042: the three
        // families are disjoint, and a value in none of them satisfies none of
        // the three rather than failing.
        let rows = NUMERIC
            .iter()
            .filter(|data_type| !AFFINITIES.contains(data_type))
            .map(|data_type| (*data_type, [true, false, false]))
            .chain(
                TEMPORAL
                    .iter()
                    .map(|data_type| (*data_type, [false, true, false])),
            )
            .chain(
                TEXTUAL
                    .iter()
                    .map(|data_type| (*data_type, [false, false, true])),
            )
            .chain(
                NEITHER
                    .iter()
                    .map(|data_type| (*data_type, [false, false, false])),
            );

        for (data_type, expected) in rows {
            let column = fixture::column_of_type(data_type);
            let answered = [
                numeric(&column).expect("it is a column"),
                temporal(&column).expect("it is a column"),
                textual(&column).expect("it is a column"),
            ];

            assert_eq!(answered, expected, "{data_type}");
        }
    }

    #[test]
    fn fr_env_046_the_four_rows_partition_the_thirty_nine_observed_values() {
        // The observation records nine, five, eight and seventeen; FR-ENV-048
        // adds the three numeric affinities no MariaDB read produces, and
        // places `text` and `blob` where FR-ENV-046 already has them.
        assert_eq!(NUMERIC.len(), 9 + 3);
        assert_eq!(TEMPORAL.len(), 5);
        assert_eq!(TEXTUAL.len(), 8);
        assert_eq!(NEITHER.len(), 17);
        assert_eq!(
            NUMERIC.len() + TEMPORAL.len() + TEXTUAL.len() + NEITHER.len(),
            39 + 3
        );

        for family in [NUMERIC.as_slice(), TEMPORAL.as_slice(), TEXTUAL.as_slice()] {
            let mut sorted = family.to_vec();
            sorted.sort_unstable();

            assert_eq!(sorted, family, "the binary search requires a sorted family");
        }
    }

    #[test]
    fn fr_env_049_the_three_mysql_values_land_in_the_families_the_requirement_fixes() {
        // FR-ENV-049: json is textual; geomcollection and vector are in none.
        assert_eq!(super::MYSQL_TEXTUAL, ["json"]);

        for (data_type, expected) in [
            ("json", [false, false, true]),
            ("geomcollection", [false, false, false]),
            ("vector", [false, false, false]),
        ] {
            let column = fixture::column_of_mysql_type(data_type);
            let answered = [
                numeric(&column).expect("it is a column"),
                temporal(&column).expect("it is a column"),
                textual(&column).expect("it is a column"),
            ];

            assert_eq!(answered, expected, "{data_type}");
        }
    }

    #[test]
    fn fr_env_048_the_five_affinities_land_in_the_families_the_requirement_fixes() {
        // FR-ENV-048: integer, numeric and real are numeric; text is textual;
        // blob is in none; and no affinity is temporal.
        for (affinity, expected) in [
            ("integer", [true, false, false]),
            ("numeric", [true, false, false]),
            ("real", [true, false, false]),
            ("text", [false, false, true]),
            ("blob", [false, false, false]),
        ] {
            let column = fixture::column_of_affinity(affinity);
            let answered = [
                numeric(&column).expect("it is a column"),
                temporal(&column).expect("it is a column"),
                textual(&column).expect("it is a column"),
            ];

            assert_eq!(answered, expected, "{affinity}");
        }
    }

    #[test]
    fn fr_ctx_018_a_type_the_system_does_not_recognise_satisfies_no_family() {
        // FR-CTX-018 carries `null` as the `data_type`, and FR-ENV-042 makes
        // that fall through rather than fail.
        let column = fixture::column_of_unrecognised_type();

        assert!(!numeric(&column).expect("it is a column"));
        assert!(!temporal(&column).expect("it is a column"));
        assert!(!textual(&column).expect("it is a column"));
    }

    #[test]
    fn fr_env_041_nullable_and_auto_increment_answer_from_the_column_alone() {
        // FR-ENV-041, rows one and two.
        assert!(nullable(&fixture::nullable_column()).expect("it is a column"));
        assert!(!nullable(&fixture::column("consignment_id")).expect("it is a column"));

        assert!(auto_increment(&fixture::auto_incremental_column()).expect("it is a column"));
        assert!(!auto_increment(&fixture::column("consignment_id")).expect("it is a column"));
    }

    #[test]
    fn fr_env_041_primary_key_and_unique_answer_from_the_table_the_context_carries() {
        // FR-ENV-041, rows three and four, over the fixture's `consignment`:
        // `consignment_id` is its primary key, `reference` carries a unique
        // index that is not the primary key, and `status` carries neither.
        let expected = [
            ("consignment_id", true, true),
            ("reference", false, true),
            ("status", false, false),
        ];

        for (column, primary, unique) in expected {
            let context = fixture::context_with(fixture::column(column));

            assert_eq!(
                answer("primary_key", &context).expect("it is a column"),
                primary,
                "{column}"
            );
            assert_eq!(
                answer("unique", &context).expect("it is a column"),
                unique,
                "{column}"
            );
        }
    }

    #[test]
    fn fr_env_043_an_absent_table_fails_rather_than_answering_false() {
        // FR-ENV-017, FR-ENV-043 and FR-SEM-017: the two derived tests fail
        // where the table is absent, and the other five do not consult the
        // context at all and do not fail for that reason.
        let orphan = fixture::column_of_absent_table();
        let context = fixture::context_with(orphan.clone());

        assert!(answer("primary_key", &context).is_err());
        assert!(answer("unique", &context).is_err());

        assert!(nullable(&orphan).is_ok());
        assert!(auto_increment(&orphan).is_ok());
        assert!(numeric(&orphan).is_ok());
        assert!(temporal(&orphan).is_ok());
        assert!(textual(&orphan).is_ok());
    }

    #[test]
    fn fr_sem_018_a_context_carrying_no_database_fails_the_two_derived_tests() {
        // FR-SEM-018: the rule is the same for a context assembled by hand.
        let context = fixture::context_with_only(fixture::column("consignment_id"));

        assert!(answer("primary_key", &context).is_err());
        assert!(answer("unique", &context).is_err());
    }

    #[test]
    fn fr_env_040_every_test_refuses_an_operand_that_is_not_a_column() {
        // FR-ENV-040 and FR-SEM-007: never `false` for an operand the test
        // does not accept, for any of the seven.
        for other in [
            fixture::table(),
            Value::from("consignment"),
            Value::from(1),
            Value::from(()),
        ] {
            assert!(nullable(&other).is_err());
            assert!(auto_increment(&other).is_err());
            assert!(numeric(&other).is_err());
            assert!(temporal(&other).is_err());
            assert!(textual(&other).is_err());

            let context = fixture::context_with(other);

            assert!(answer("primary_key", &context).is_err());
            assert!(answer("unique", &context).is_err());
        }
    }
}
