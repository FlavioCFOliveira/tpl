//! The template surface, declared once and registered from that declaration.
//!
//! `FR-ENV-001` divides the surface into three groups, and two of them are
//! named here:
//!
//! | Group | What it holds | Guarantee |
//! |---|---|---|
//! | 1 | The eleven filters, seven tests and five functions `tpl` registers | Full contract (`FR-ENV-002`) |
//! | 2 | The fourteen inherited filters of `FR-ENV-018`, closed by `FR-ENV-019` | Against the pin of `ADR-001` |
//! | 3 | Everything else the engine offers | None (`FR-ENV-004`) |
//!
//! `FR-ENV-005` requires the published surface to be **derived from the
//! registrations the environment actually performs**, for the reason
//! `FR-HELP-021` gives for the command tree: a second statement of one truth is
//! the statement that stops being true without saying so. The engine exposes no
//! way to read its filters back, so the derivation is made the other way round
//! — the `declare!` macro below takes one list and writes both the name arrays
//! `cli::help::document` publishes and the registration that installs them, so
//! neither can be edited without the other.
//!
//! # Order
//!
//! `FR-ENV-005` fixes it: `registered.filters` is the names of `FR-ENV-006`
//! followed by those of `FR-ENV-007`, `registered.tests` those of
//! `FR-ENV-014`, `registered.functions` those of `FR-ENV-020`, and
//! `inherited.filters` those of `FR-ENV-018`, each in the order its
//! requirement states them. The declaration below is written in those orders,
//! and a test holds it to them.
//!
//! # The two deliberate shadowings
//!
//! `indent` and `escape` are engine built-ins, and `tpl` registers its own over
//! them. `ADR-001` records the decision: each `tpl` filter takes a required
//! argument where the engine's takes none, so a template written against the
//! engine's signature fails loudly rather than emitting different bytes.
//! Neither name is in the closed list of `FR-ENV-018`, so neither shadowing
//! weakens a group 2 guarantee.

use minijinja::Environment;

use super::{code, function, naming, predicate};

/// Declares the registered surface once, and derives both statements of it.
///
/// The macro exists because the name arrays of `FR-ENV-005` and the
/// registration that installs the names are the same list read twice, and
/// `FR-ENV-005` requires the first to be derived from the second. Written out
/// by hand they would be two lists that a later edition could move apart.
macro_rules! declare {
    (
        filters { $($filter:literal => $filter_fn:path),+ $(,)? }
        tests { $($test:literal => $test_fn:path),+ $(,)? }
        functions { $($function:literal => $function_fn:path),+ $(,)? }
    ) => {
        /// The filters of group 1: those of `FR-ENV-006` followed by those of
        /// `FR-ENV-007`, in the order each requirement states them.
        pub(crate) const REGISTERED_FILTERS: &[&str] = &[$($filter),+];

        /// The tests of group 1, in the order of `FR-ENV-014`.
        pub(crate) const REGISTERED_TESTS: &[&str] = &[$($test),+];

        /// The global functions of group 1, in the order of `FR-ENV-020`.
        pub(crate) const REGISTERED_FUNCTIONS: &[&str] = &[$($function),+];

        /// Installs group 1 on `engine`.
        ///
        /// This is the registration `FR-ENV-005` requires the published
        /// surface to be derived from, and the three arrays above are that
        /// derivation.
        pub(super) fn register(engine: &mut Environment<'static>) {
            $(engine.add_filter($filter, $filter_fn);)+
            $(engine.add_test($test, $test_fn);)+
            $(engine.add_function($function, $function_fn);)+
        }
    };
}

declare! {
    filters {
        // FR-ENV-006, in the order it states them.
        "pascal" => naming::pascal,
        "camel" => naming::camel,
        "snake" => naming::snake,
        "upper_snake" => naming::upper_snake,
        "kebab" => naming::kebab,
        // FR-ENV-007, in the order it states them.
        "quote" => code::quote,
        "sql_type" => code::sql_type,
        "json" => code::json,
        "indent" => code::indent,
        "comment" => code::comment,
        "escape" => code::escape,
    }
    tests {
        // FR-ENV-014, in the order it states them.
        "nullable" => predicate::nullable,
        "primary_key" => predicate::primary_key,
        "auto_increment" => predicate::auto_increment,
        "unique" => predicate::unique,
        "numeric" => predicate::numeric,
        "temporal" => predicate::temporal,
        "textual" => predicate::textual,
    }
    functions {
        // FR-ENV-020, in the order of its table.
        "table" => function::table,
        "view" => function::view,
        "routine" => function::routine,
        "column" => function::column,
        "fail" => function::fail,
    }
}

/// The inherited filters of group 2 (`FR-ENV-018`), in the order it states
/// them.
///
/// The list is **closed**, per `FR-ENV-019`: a filter the engine offers that is
/// absent from it belongs to group 3 and carries no guarantee. It is not
/// registered here — every one of the fourteen is a built-in of the engine
/// `ADR-001` pins — so this array is a statement about the engine rather than
/// about `tpl`, and `FR-ENV-003` obliges whoever moves the pin to check that
/// each name still exists and still behaves as before.
pub(crate) const INHERITED_FILTERS: &[&str] = &[
    "default", "join", "length", "map", "select", "reject", "first", "last", "reverse", "sort",
    "trim", "upper", "lower", "replace",
];

#[cfg(test)]
mod tests {
    use super::{
        INHERITED_FILTERS, REGISTERED_FILTERS, REGISTERED_FUNCTIONS, REGISTERED_TESTS, register,
    };
    use minijinja::{Environment, UndefinedBehavior};

    /// An environment carrying group 1 and the engine's own surface.
    fn engine() -> Environment<'static> {
        let mut engine = Environment::new();
        engine.set_undefined_behavior(UndefinedBehavior::Strict);
        register(&mut engine);

        engine
    }

    /// Whether `name` is installed on `engine` as a filter or as a test.
    ///
    /// The engine exposes no iterator over either, so the question is asked in
    /// the template language: the engine's own `filter` and `test` tests
    /// answer it, and both are group 3 of `FR-ENV-001` — which is what a test
    /// of this crate may depend on and a template may not.
    fn installed(engine: &Environment<'static>, kind: &str, name: &str) -> bool {
        let source = format!("{{% if '{name}' is {kind} %}}yes{{% else %}}no{{% endif %}}");

        engine
            .render_str(&source, ())
            .expect("the engine declares the test")
            == "yes"
    }

    #[test]
    fn fr_env_005_the_three_arrays_are_the_names_their_requirements_state() {
        // FR-ENV-006 and FR-ENV-007, then FR-ENV-014, then FR-ENV-020 — each
        // in the order the requirement states them, per FR-ENV-005 and
        // FR-HELP-023.
        assert_eq!(
            REGISTERED_FILTERS,
            [
                "pascal",
                "camel",
                "snake",
                "upper_snake",
                "kebab",
                "quote",
                "sql_type",
                "json",
                "indent",
                "comment",
                "escape",
            ]
        );
        assert_eq!(
            REGISTERED_TESTS,
            [
                "nullable",
                "primary_key",
                "auto_increment",
                "unique",
                "numeric",
                "temporal",
                "textual",
            ]
        );
        assert_eq!(
            REGISTERED_FUNCTIONS,
            ["table", "view", "routine", "column", "fail"]
        );
    }

    #[test]
    fn fr_env_001_group_one_is_eleven_filters_seven_tests_and_five_functions() {
        assert_eq!(REGISTERED_FILTERS.len(), 11);
        assert_eq!(REGISTERED_TESTS.len(), 7);
        assert_eq!(REGISTERED_FUNCTIONS.len(), 5);
    }

    #[test]
    fn fr_env_005_every_declared_filter_and_test_is_installed_on_the_engine() {
        // The arrays are derived from the registration, and this holds the
        // derivation to the engine: a name that is published and not installed
        // would be a surface the binary does not have.
        let engine = engine();

        for filter in REGISTERED_FILTERS {
            assert!(
                installed(&engine, "filter", filter),
                "'{filter}' is not installed"
            );
        }

        for test in REGISTERED_TESTS {
            assert!(
                installed(&engine, "test", test),
                "'{test}' is not installed"
            );
        }
    }

    #[test]
    fn fr_env_018_every_inherited_filter_still_exists_on_the_pinned_engine() {
        // FR-ENV-003 obliges this check, and ADR-001 states it as the
        // obligation on whoever moves the pin. It is made here so that moving
        // the pin fails a test rather than a template.
        let engine = engine();

        assert_eq!(INHERITED_FILTERS.len(), 14);

        for filter in INHERITED_FILTERS {
            assert!(
                installed(&engine, "filter", filter),
                "the inherited filter '{filter}' is gone"
            );
        }
    }

    #[test]
    fn fr_env_019_the_inherited_list_is_closed_and_names_nothing_tpl_registers() {
        // FR-ENV-019: a filter the engine offers that is absent from the list
        // belongs to group 3. `escape` is the case ADR-001 records — a name
        // `tpl` registers, which therefore belongs to group 1 and not to the
        // inherited list, although the engine offers one too.
        for filter in REGISTERED_FILTERS {
            assert!(
                !INHERITED_FILTERS.contains(filter),
                "'{filter}' is in both groups"
            );
        }
    }

    #[test]
    fn fr_env_009_the_four_forbidden_filters_are_absent() {
        // FR-ENV-009 through FR-ENV-013. The mapping `rust_type` and `go_type`
        // performed is delivered as a template macro, per FR-ENV-011, and the
        // two inflection filters are refused outright by BR-ENV-003.
        let engine = engine();

        for filter in ["rust_type", "go_type", "plural", "singular"] {
            assert!(!installed(&engine, "filter", filter), "'{filter}' exists");

            let applied = format!("{{{{ 'orders' | {filter} }}}}");

            assert!(
                engine.render_str(&applied, ()).is_err(),
                "'{filter}' is callable"
            );
        }
    }
}
