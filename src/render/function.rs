//! The five global functions of `FR-ENV-020`, and the four that do not exist.
//!
//! | Function | What it does |
//! |---|---|
//! | `table_named(name)` | Resolves a table by name in the render context |
//! | `view_named(name)` | Resolves a view by name |
//! | `routine_named(name)` | Resolves a routine by name |
//! | `column(table, name)` | Resolves a column of a named table |
//! | `fail(message)` | Ends the render with `65`, carrying the author's message |
//!
//! No function carries the name of a context variable (`FR-ENV-020`, sixtieth
//! edition): the lookups were once named `table`, `view` and `routine`, and a
//! template sees one namespace, so without an object flag `table is defined`
//! was `true` and with one the bound object hid the lookup.
//!
//! `FR-ENV-020` closes the set at those five, and `FR-ENV-022` through
//! `FR-ENV-025` state four prohibitions as prohibitions rather than as an
//! absence, so that adding one is visibly a change of position: no function
//! reads the environment, a file, the network or a clock. `BR-ENV-004` is the
//! reason — a template arrives in a clone and is read before anybody has
//! reviewed it, so it is semi-trusted input, and the four are what keep
//! `tpl render` from being a way to run arbitrary code with the caller's
//! privileges. The only source of time available to a template is the `now`
//! context variable of `FR-RND-023`.
//!
//! # What a lookup that finds nothing produces
//!
//! `undefined`, which the strict undefined behaviour of `OD-14` turns into the
//! `65` of `FR-SEM-012` at the point the template **uses** the answer. The
//! outcome is therefore the failure `BR-SEM-004` asks for, and a template that
//! wants to ask first can still write `{% if table_named("x") is defined %}` or
//! `{{ table_named("x") | default(…) }}` — the inherited `default` of `FR-ENV-018`
//! being guaranteed for exactly that.
//!
//! *Rejected: `none`.* It renders as the empty string under `FR-SEM-010`, so a
//! template that misspelled a table name would emit nothing and exit `0`,
//! which is the failure mode this tool exists to avoid.
//!
//! *Rejected: failing at the call.* It forecloses the guard above, and
//! `FR-ENV-020` fixes no such condition for the four lookups — `FR-ENV-017`
//! fixes one for the two derived tests, and that one is enforced where it is
//! stated.

use minijinja::{Error, ErrorKind, State, Value};

use super::lookup;

/// `table_named(name)` (`FR-ENV-020`).
pub(super) fn table_named(state: &State<'_, '_>, name: &str) -> Value {
    resolved(lookup::member(state, lookup::TABLES, name))
}

/// `view_named(name)` (`FR-ENV-020`).
pub(super) fn view_named(state: &State<'_, '_>, name: &str) -> Value {
    resolved(lookup::member(state, lookup::VIEWS, name))
}

/// `routine_named(name)` (`FR-ENV-020`).
///
/// The first routine of that name, in the order `NFR-DET-002` fixes for the
/// collection. A database may declare a procedure and a function under one
/// name, and `FR-ENV-020` gives the function one argument, so the name is what
/// it resolves on.
pub(super) fn routine_named(state: &State<'_, '_>, name: &str) -> Value {
    resolved(lookup::member(state, lookup::ROUTINES, name))
}

/// `column(table, name)` (`FR-ENV-020`).
pub(super) fn column(state: &State<'_, '_>, table: &str, name: &str) -> Value {
    let Some(table) = lookup::member(state, lookup::TABLES, table) else {
        return Value::UNDEFINED;
    };

    resolved(lookup::column_of(&table, name))
}

/// `fail(message)` (`FR-ENV-020`, `FR-ENV-021`, `FR-SEM-014`).
///
/// The render ends with `65`, carrying the message the template supplied, with
/// the template, the line and the column `FR-SEM-015` and `FR-ERR-011`
/// require. There is no counterpart that warns and continues: `FR-SEM-016`
/// forbids one, because a warning leaves the exit code at `0` and hands the
/// caller an incomplete generated file with nothing in the code to say so.
///
/// # Errors
///
/// Always. That is what it is for.
pub(super) fn fail(message: &str) -> Result<Value, Error> {
    Err(Error::new(ErrorKind::InvalidOperation, message.to_owned())
        .with_source(Failed(message.to_owned())))
}

/// The mark `fail` attaches to the error it returns, so that the diagnostic
/// can tell the author's own stop from an invalid operation the engine
/// reported, and put the author's message on the `error:` line
/// (`FR-SEM-015`).
#[derive(Debug)]
pub(super) struct Failed(pub(super) String);

impl std::fmt::Display for Failed {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}

impl std::error::Error for Failed {}

/// A lookup's answer, or the `undefined` this module's header argues for.
fn resolved(found: Option<Value>) -> Value {
    found.unwrap_or(Value::UNDEFINED)
}

#[cfg(test)]
mod tests {
    use crate::render::{fixture, surface};
    use minijinja::{Environment, UndefinedBehavior};

    /// Renders `source` against the fixture's render context.
    fn render(source: &str) -> Result<String, minijinja::Error> {
        let mut engine = Environment::new();
        engine.set_undefined_behavior(UndefinedBehavior::Strict);
        surface::register(&mut engine);

        engine.render_str(source, fixture::context())
    }

    #[test]
    fn fr_env_020_the_four_lookups_resolve_an_object_of_the_render_context() {
        // FR-ENV-020, rows one to four, over the fixture's model.
        assert_eq!(
            render("{{ table_named('consignment').name }}").expect("it exists"),
            "consignment"
        );
        assert_eq!(
            render("{{ view_named('v_consignment_manifest').name }}").expect("it exists"),
            "v_consignment_manifest"
        );
        assert_eq!(
            render("{{ routine_named('sp_book_consignment').name }}").expect("it exists"),
            "sp_book_consignment"
        );
        assert_eq!(
            render("{{ column('consignment', 'reference').name }}").expect("it exists"),
            "reference"
        );
    }

    #[test]
    fn fr_sem_012_a_lookup_that_finds_nothing_fails_the_render_where_it_is_used() {
        // The answer is `undefined`, and OD-14's strict behaviour makes every
        // use of it the `65` of FR-SEM-012 — while leaving the two guards the
        // module header names available.
        assert!(render("{{ table_named('absent') }}").is_err());
        assert!(render("{{ table_named('absent').name }}").is_err());
        assert!(render("{{ column('consignment', 'absent') }}").is_err());
        assert!(render("{{ column('absent', 'reference') }}").is_err());

        assert_eq!(
            render("{% if table_named('absent') is defined %}yes{% else %}no{% endif %}")
                .expect("the guard is allowed"),
            "no"
        );
        assert_eq!(
            render("{{ table_named('absent') | default('none of them') }}")
                .expect("the guard is allowed"),
            "none of them"
        );
    }

    #[test]
    fn fr_env_021_fail_ends_the_render_carrying_the_authors_message() {
        // FR-ENV-021 and FR-SEM-014. The exit code is asserted where the
        // condition becomes one, in `super::super`.
        let condition =
            render("{{ fail('this type has no mapping') }}").expect_err("fail always fails");

        assert!(
            condition.to_string().contains("this type has no mapping"),
            "{condition}"
        );
    }

    #[test]
    fn fr_env_022_the_environment_offers_no_function_that_reaches_outside_the_process() {
        // FR-ENV-022 through FR-ENV-025, as names a template author would
        // reach for. None of them is a global, and calling one is an unknown
        // function rather than a capability.
        for name in [
            "env",
            "environ",
            "getenv",
            "open",
            "read_file",
            "file",
            "include_file",
            "fetch",
            "http",
            "request",
            "url",
            "now",
            "today",
            "utcnow",
            "time",
        ] {
            assert!(
                render(&format!("{{{{ {name}('x') }}}}")).is_err(),
                "'{name}' is callable"
            );
        }
    }

    #[test]
    fn fr_env_020_the_globals_are_the_five_and_the_engine_s_own() {
        // FR-ENV-020 closes the set `tpl` provides. What the engine itself
        // declares is group 3 of FR-ENV-001, and none of its names is one the
        // four prohibitions reach.
        let mut engine = Environment::new();
        surface::register(&mut engine);

        let declared: Vec<&str> = engine.globals().map(|(name, _)| name).collect();

        for name in surface::REGISTERED_FUNCTIONS {
            assert!(declared.contains(name), "'{name}' is not registered");
        }

        for name in ["env", "open", "fetch", "now"] {
            assert!(!declared.contains(&name), "'{name}' is declared");
        }
    }
}
