//! How the engine is built, and the five settings that are not its defaults.
//!
//! | Setting | Value | Forced by |
//! |---|---|---|
//! | Trailing newline | Preserved | `FR-SEM-003` |
//! | Undefined behaviour | Strict | `FR-SEM-012`, `OD-14` |
//! | Auto-escaping | Off, for every name | `FR-ENV-026`, `FR-ENV-027` |
//! | The formatter | `null` writes nothing | `FR-SEM-010`, `FR-SEM-011` |
//! | The loader | `tpl`'s own, over one resolution | `FR-TMPL-023` … `FR-TMPL-027`, `OD-15` |
//!
//! Everything else is left at the engine's default, which is what `BR-SEM-001`
//! asks for: a correct template produces here exactly what it produces
//! anywhere else, so the newline after a block tag and the whitespace before
//! one are both kept (`FR-SEM-001`, `FR-SEM-002`) and trimming is requested in
//! the template with `{%-` and `-%}` (`FR-SEM-004`).
//!
//! # Auto-escaping is set, not left
//!
//! The engine's default callback keys escaping on the **file extension**,
//! which `FR-ENV-027` forbids by name: no property of a template's name may
//! turn escaping on. The callback is therefore replaced rather than relied on,
//! and escaping happens only where a template asks for it, through the
//! registered `escape` of `FR-ENV-028`.
//!
//! # The formatter, and the one observation it makes unnecessary
//!
//! `FR-SEM-010` renders an interpolated `null` as the empty string and
//! `FR-SEM-011` forbids the words `none` and `null` in its place. Strictness
//! governs *undefined* values and says nothing about a defined `null`, so the
//! two requirements would otherwise rest on an engine behaviour nobody had
//! fixed. The formatter settles it inside `tpl`: a `null` writes nothing,
//! whatever the engine would have written, and `FR-SEM-013` is preserved
//! because an undefined never reaches the formatter at all — strictness fails
//! it first, which is `FR-SEM-012`.
//!
//! # The loader
//!
//! It calls [`super::root::Root::locate`] and opens the path that was checked,
//! per `OD-15`. Its three outcomes are that decision's table: a template that
//! does not exist is `Ok(None)`, so the engine raises its own not-found error
//! carrying the line and the column `FR-TMPL-009` requires; a path that
//! escapes the root is reported as an escape; and anything else is the source
//! the engine compiles. The engine caches what the loader returns, so each
//! template is read and parsed once per process.

use minijinja::{AutoEscape, Environment, ErrorKind, Output, State, UndefinedBehavior, Value};

use super::root::Root;
use super::surface;
use crate::error::Error;

/// Builds the engine over `root`.
///
/// It is called once per process and only by an invocation that compiles a
/// template: [`super::Environment`] holds the result in a cell that stays
/// empty otherwise, which is the lazy construction `NFR-PERF-006` requires.
pub(super) fn build(root: Root) -> Environment<'static> {
    let mut engine = Environment::new();

    // FR-SEM-003. The engine's default strips a single trailing newline, and
    // BR-SEM-001 departs from it deliberately: most formatters require a final
    // newline and a generator that omits it produces a diff on every file.
    engine.set_keep_trailing_newline(true);

    // OD-14, FR-SEM-012.
    engine.set_undefined_behavior(UndefinedBehavior::Strict);

    // FR-ENV-026, FR-ENV-027.
    engine.set_auto_escape_callback(|_| AutoEscape::None);

    // FR-SEM-010, FR-SEM-011.
    engine.set_formatter(format);

    // FR-ERR-011 obliges the line and the column of a failure, and the engine
    // carries the byte range and the source only as debug information.
    engine.set_debug(true);

    // OD-15, FR-TMPL-023 … FR-TMPL-027.
    engine.set_loader(move |name| load(&root, name));

    surface::register(&mut engine);

    engine
}

/// Writes one interpolated value (`FR-SEM-010`, `FR-SEM-011`).
fn format(
    out: &mut Output<'_>,
    state: &State<'_, '_>,
    value: &Value,
) -> Result<(), minijinja::Error> {
    if value.is_none() {
        return Ok(());
    }

    minijinja::escape_formatter(out, state, value)
}

/// Reads one template, through the one resolution of `OD-15`.
fn load(root: &Root, name: &str) -> Result<Option<String>, minijinja::Error> {
    let located = match root.locate(name) {
        Ok(path) => path,
        // The engine raises its own not-found error, which carries the line
        // and the column `FR-TMPL-009` requires of an `{% include %}` that did
        // not resolve. On the command-line path the name was already resolved
        // before the engine was reached, per `OD-15`, so this arm is the
        // include's.
        Err(Error::TemplateNotFound { .. }) => return Ok(None),
        Err(refused) => {
            return Err(minijinja::Error::new(
                ErrorKind::InvalidOperation,
                refused.to_string(),
            ));
        }
    };

    std::fs::read_to_string(&located)
        .map(Some)
        .map_err(|returned| {
            minijinja::Error::new(
                ErrorKind::InvalidOperation,
                format!("template '{name}' could not be read"),
            )
            .with_source(returned)
        })
}

#[cfg(test)]
mod tests {
    use minijinja::{Environment, UndefinedBehavior, Value, context};

    /// The engine as it comes, with only the strictness of `OD-14` applied.
    fn stock() -> Environment<'static> {
        let mut engine = Environment::new();
        engine.set_undefined_behavior(UndefinedBehavior::Strict);

        engine
    }

    #[test]
    fn fr_sem_011_the_formatter_is_what_keeps_a_null_from_reaching_the_output() {
        // The observation `OD-14` records as owed, made here so that the
        // formatter is not removed as redundant: under the strict behaviour a
        // **defined** `null` does not fail, and the engine writes the word
        // `None` for it — which `FR-SEM-011` forbids by name, in either case.
        // The formatter of this module is therefore load-bearing for
        // `FR-SEM-010` rather than a restatement of what the engine does.
        let written = stock()
            .render_str("[{{ value }}]", context! { value => Value::from(()) })
            .expect("a defined null does not fail under the strict behaviour");

        assert_eq!(written, "[None]");
    }

    #[test]
    fn fr_sem_012_the_strict_behaviour_is_what_fails_a_field_that_does_not_exist() {
        // The other half of `FR-SEM-013`: absence and `null` are different
        // failures, and only the first of them is the engine's to raise.
        assert!(
            stock()
                .render_str("{{ value.absent }}", context! { value => context! {} })
                .is_err()
        );
    }
}
