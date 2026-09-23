//! What the engine reports, read as a condition of this crate.
//!
//! `FR-ERR-011` makes the template engine the one dependency whose error
//! reaches the caller: a failure carries the template, the line, the column and
//! the **chain** of underlying engine errors, and `OD-06` admits that as the
//! deliberate exception to the rule that a dependency's error is classified and
//! dropped at the boundary.
//!
//! | The engine reports | The caller receives | Code |
//! |---|---|---|
//! | A syntax error | [`Error::TemplateSyntax`] | `65` |
//! | A template the loader did not supply | [`Error::TemplateNotFound`] | `66` |
//! | Fuel exhausted, per [`out_of_fuel`] — classified by [`super::Environment::render`], which holds the budget the `cause` names | [`Error::RenderFuelExhausted`] | `65` |
//! | Anything else | [`Error::RenderFailed`] | `65` |
//!
//! # Why a render maps the not-found row differently
//!
//! `FR-TMPL-009` makes an `{% include %}` that does not resolve a `65` naming
//! the template, the line and the column, while `FR-TMPL-027` makes a template
//! the **command line** named a `66`. `OD-15` settles the split by who asks:
//! the command line resolves the template itself before the engine is reached,
//! so a not-found raised during a render can only have come from inside a
//! template. [`during_render`] therefore carries every engine failure to
//! [`Error::RenderFailed`], and [`during_compile`] is the one that separates
//! the two rows.
//!
//! # The column
//!
//! The engine reports a line and a byte range rather than a column, so the
//! column is counted from the start of the line the range begins in, in
//! characters, as [`crate::error::Position`] counts it. The range and the
//! source are both debug information, which the engine attaches when it is
//! configured to — [`super::engine`] configures it to — and a failure that
//! carries neither is reported at the first column of the line rather than
//! without a position, because `FR-SEM-019` requires one.

use std::error::Error as _;
use std::path::Path;

use crate::error::{Error, Position};

/// The condition a failed compile is (`FR-TMPL-020`, `FR-RND-030`).
pub(super) fn during_compile(root: &Path, name: &str, reported: &minijinja::Error) -> Error {
    if reported.kind() == minijinja::ErrorKind::TemplateNotFound {
        return Error::TemplateNotFound {
            name: name.to_owned(),
            root: root.to_owned(),
            // The name reaching here was resolved before the engine was given
            // it, per `OD-15`, so this is a template that went away between
            // the two reads rather than one a caller misspelled — and there is
            // no misspelling for a suggestion to correct.
            nearest: Vec::new(),
        };
    }

    if reported.kind() == minijinja::ErrorKind::SyntaxError {
        return Error::TemplateSyntax {
            template: template(name, reported),
            position: position(reported),
            chain: chain(reported),
        };
    }

    during_render(name, reported)
}

/// The condition a failed render is (`FR-RND-031`, `FR-SEM-019`).
///
/// Every failure defined by `render-semantics.md` arrives here: an undefined
/// variable, a filter or a test handed the wrong operand, a template the
/// author ended with `fail`, and an `{% include %}` that did not resolve.
/// `FR-SEM-019` makes all of them one condition carrying the template, the
/// line and the column.
pub(super) fn during_render(name: &str, reported: &minijinja::Error) -> Error {
    if reported.kind() == minijinja::ErrorKind::SyntaxError {
        return Error::TemplateSyntax {
            template: template(name, reported),
            position: position(reported),
            chain: chain(reported),
        };
    }

    Error::RenderFailed {
        template: template(name, reported),
        position: position(reported),
        chain: chain(reported),
    }
}

/// Whether the engine stopped because the render exhausted its fuel
/// (`FR-RND-036`).
///
/// The kind is read on the error and on every error in its chain, so a render
/// that ran out inside an `{% include %}`, a macro or a call block is
/// recognised whatever the engine wrapped the exhaustion in.
pub(super) fn out_of_fuel(reported: &minijinja::Error) -> bool {
    let mut current: Option<&(dyn std::error::Error + 'static)> = Some(reported);

    while let Some(error) = current {
        if error
            .downcast_ref::<minijinja::Error>()
            .is_some_and(|engine| engine.kind() == minijinja::ErrorKind::OutOfFuel)
        {
            return true;
        }
        current = error.source();
    }

    false
}

/// The template the failure arose in, which for an `{% include %}` is the
/// included one rather than the one the caller named.
fn template(name: &str, reported: &minijinja::Error) -> String {
    reported.name().unwrap_or(name).to_owned()
}

/// Where the engine stopped (`FR-ERR-011`).
fn position(reported: &minijinja::Error) -> Position {
    Position {
        line: reported.line().unwrap_or(1),
        column: column(reported),
    }
}

/// The column the engine's byte range begins at, counted from one.
fn column(reported: &minijinja::Error) -> usize {
    let (Some(range), Some(source)) = (reported.range(), reported.template_source()) else {
        return 1;
    };

    if !source.is_char_boundary(range.start) {
        return 1;
    }

    let head = &source[..range.start];
    let line = head.rfind('\n').map_or(0, |newline| newline + 1);

    head[line..].chars().count() + 1
}

/// The chain of underlying engine errors, outermost first (`FR-ERR-011`).
fn chain(reported: &minijinja::Error) -> Vec<String> {
    let mut chain = vec![reported.to_string()];
    let mut source = reported.source();

    while let Some(current) = source {
        chain.push(current.to_string());
        source = current.source();
    }

    chain
}

#[cfg(test)]
mod tests {
    use super::{during_compile, during_render};
    use crate::error::Error;
    use std::path::Path;

    /// The engine the two mappings are exercised against.
    fn engine() -> minijinja::Environment<'static> {
        let mut engine = minijinja::Environment::new();
        engine.set_undefined_behavior(minijinja::UndefinedBehavior::Strict);
        engine.set_debug(true);

        engine
    }

    #[test]
    fn fr_err_011_a_syntax_error_carries_the_template_the_line_and_the_column() {
        // FR-TMPL-020 and FR-ERR-011: the template, the line and the column,
        // with the engine's own chain beneath them.
        let reported = engine()
            .template_from_named_str("example.jinja", "ok\n{% if %}\n")
            .expect_err("the expression is missing");

        let condition = during_compile(Path::new(".tpl/templates"), "example.jinja", &reported);

        let Error::TemplateSyntax {
            template,
            position,
            chain,
        } = condition
        else {
            panic!("a syntax error is not a render failure");
        };

        assert_eq!(template, "example.jinja");
        assert_eq!(position.line, 2);
        assert!(position.column >= 1);
        assert!(!chain.is_empty());
    }

    #[test]
    fn fr_sem_019_a_render_failure_carries_the_template_the_line_and_the_column() {
        // FR-SEM-012 through FR-SEM-019: reading a field that does not exist
        // fails, and the failure carries where it stopped.
        let engine = engine();
        let template = engine
            .template_from_named_str("example.jinja", "one\n{{ value.absent }}\n")
            .expect("it compiles");

        let reported = template
            .render(minijinja::context! { value => minijinja::context! {} })
            .expect_err("the field does not exist");

        let condition = during_render("example.jinja", &reported);

        let Error::RenderFailed {
            template,
            position,
            chain,
        } = condition
        else {
            panic!("an evaluation failure is a render failure");
        };

        assert_eq!(template, "example.jinja");
        assert_eq!(position.line, 2);
        assert!(!chain.is_empty());
    }

    #[test]
    fn fr_tmpl_027_a_template_the_engine_did_not_find_is_66_on_the_compile_path() {
        // FR-TMPL-027 and OD-15: the command line asks, so the condition is
        // the missing template rather than a render failure.
        let reported = engine()
            .get_template("absent.jinja")
            .expect_err("nothing was loaded");

        let condition = during_compile(Path::new(".tpl/templates"), "absent.jinja", &reported);

        assert!(matches!(condition, Error::TemplateNotFound { .. }));
        assert_eq!(condition.exit_code(), 66);
    }

    #[test]
    fn fr_tmpl_009_a_template_the_engine_did_not_find_is_65_on_the_render_path() {
        // FR-TMPL-009: an `{% include %}` that does not resolve literally is a
        // render failure, naming the template, the line and the column.
        let reported = engine()
            .get_template("absent.jinja")
            .expect_err("nothing was loaded");

        let condition = during_render("caller.jinja", &reported);

        assert!(matches!(condition, Error::RenderFailed { .. }));
        assert_eq!(condition.exit_code(), 65);
    }
}
