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

use super::function::Failed;
use crate::error::{Error, LookupKind, Position, RenderReason, Unresolved};

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
        invoked: name.to_owned(),
        undefined: undefined(reported),
        reason: failed(reported).map(|message| Box::new(RenderReason::Failed(message))),
        position: position(reported),
        chain: chain(reported),
    }
}

/// The lookup call an undefined expression begins with, WHERE its arguments
/// are literals and the call, evaluated again against the same context,
/// finds nothing.
///
/// `table("orders").name` is undefined either because there is no table
/// `orders` or because a table has no member `name`; evaluating the call alone
/// is what tells the two apart, with the very function the template called.
/// An argument that is not a literal — `table(t.name)` — names a value only
/// the template's own scope holds, so no claim is made.
pub(super) fn unresolved(
    engine: &minijinja::Environment<'_>,
    context: &minijinja::Value,
    expression: &str,
) -> Option<Unresolved> {
    let (function, rest) = expression.split_once('(')?;
    let kind = match function {
        "table" => LookupKind::Table,
        "view" => LookupKind::View,
        "routine" => LookupKind::Routine,
        "column" => LookupKind::Column,
        _ => return None,
    };
    let (arguments, length) = literals(rest)?;
    let call = &expression[..function.len() + 1 + length];
    let undefined = |call: &str| {
        engine
            .compile_expression(call)
            .and_then(|compiled| compiled.eval(context))
            .is_ok_and(|value| value.is_undefined())
    };

    if !undefined(call) {
        return None;
    }

    match (kind, arguments.as_slice()) {
        (LookupKind::Column, [table, name]) => {
            let table_call = format!("table({})", quoted(table));
            Some(if undefined(&table_call) {
                Unresolved {
                    call: call.to_owned(),
                    kind: LookupKind::Table,
                    name: table.clone(),
                    table: None,
                    document: None,
                }
            } else {
                Unresolved {
                    call: call.to_owned(),
                    kind,
                    name: name.clone(),
                    table: Some(table.clone()),
                    document: None,
                }
            })
        }
        (LookupKind::Column, _) | (_, []) => None,
        (_, [name, ..]) => Some(Unresolved {
            call: call.to_owned(),
            kind,
            name: name.clone(),
            table: None,
            document: None,
        }),
    }
}

/// The string literals of an argument list, read from just after its opening
/// parenthesis, and the length up to and including the closing one; [`None`]
/// where an argument is anything other than a string literal.
fn literals(rest: &str) -> Option<(Vec<String>, usize)> {
    let bytes = rest.as_bytes();
    let mut arguments = Vec::new();
    let mut index = 0;

    loop {
        while bytes.get(index).is_some_and(u8::is_ascii_whitespace) {
            index += 1;
        }
        match bytes.get(index)? {
            b')' if arguments.is_empty() => return Some((arguments, index + 1)),
            quote @ (b'"' | b'\'') => {
                let end = index + 1 + rest[index + 1..].find(char::from(*quote))?;
                let literal = &rest[index + 1..end];
                if literal.contains('\\') {
                    return None;
                }
                arguments.push(literal.to_owned());
                index = end + 1;
            }
            _ => return None,
        }
        while bytes.get(index).is_some_and(u8::is_ascii_whitespace) {
            index += 1;
        }
        match bytes.get(index)? {
            b',' => index += 1,
            b')' => return Some((arguments, index + 1)),
            _ => return None,
        }
    }
}

/// A name as a string literal the engine reads back as the same name.
fn quoted(name: &str) -> String {
    if name.contains('"') {
        format!("'{name}'")
    } else {
        format!("\"{name}\"")
    }
}

/// The name an `{% include %}` wrote, where a template it named is what the
/// loader did not hold.
///
/// The engine carries the name only in its message, `tried to include
/// non-existing template "<name>"`, written with Rust's quoting; a name that
/// the quoting escaped is not recovered, and yields nothing rather than a
/// guess.
pub(super) fn missing_include(reported: &minijinja::Error) -> Option<String> {
    let mut current: Option<&(dyn std::error::Error + 'static)> = Some(reported);

    while let Some(error) = current {
        if let Some(engine) = error.downcast_ref::<minijinja::Error>()
            && engine.kind() == minijinja::ErrorKind::TemplateNotFound
        {
            let detail = engine.detail()?;
            let (_, quoted) = detail.split_once('"')?;
            let name = quoted.strip_suffix('"')?;
            return (!name.is_empty() && !name.contains(['"', '\\'])).then(|| name.to_owned());
        }
        current = error.source();
    }

    None
}

/// The message the template gave `fail(message)`, where that call is what
/// ended the render.
fn failed(reported: &minijinja::Error) -> Option<String> {
    let mut current: Option<&(dyn std::error::Error + 'static)> = Some(reported);

    while let Some(error) = current {
        if let Some(Failed(message)) = error.downcast_ref::<Failed>() {
            return Some(message.clone());
        }
        current = error.source();
    }

    None
}

/// The source text of the expression the engine found undefined, where the
/// failure is an undefined value and the engine located it.
///
/// The engine reports the byte range of the expression it evaluated, which for
/// `{{ table.name }}` is `table.name`; the text is taken from the template's
/// own source, so it is what the author wrote. A range that does not fall on
/// the source, or that spans more than one line, yields nothing rather than a
/// fragment.
fn undefined(reported: &minijinja::Error) -> Option<String> {
    if reported.kind() != minijinja::ErrorKind::UndefinedError {
        return None;
    }

    let range = reported.range()?;
    let source = reported.template_source()?;
    let start = expression_start(source.as_bytes(), range.start)?;
    let expression = source.get(start..range.end)?.trim();

    // The engine's range can begin part-way into the expression — at
    // `.nosuch.x` of `database.nosuch.x`, or at `("orders").name` of
    // `table("orders").name` — so it is extended back to where the expression
    // begins; one that still does not begin with a name is quoted not at all
    // rather than as a fragment.
    let named = expression
        .bytes()
        .next()
        .is_some_and(|byte| byte.is_ascii_alphabetic() || byte == b'_');

    (named && !expression.contains('\n') && expression.len() <= 128).then(|| expression.to_owned())
}

/// Where the expression that contains the byte at `start` begins: back over
/// names, dots, and balanced call or subscript brackets.
fn expression_start(source: &[u8], start: usize) -> Option<usize> {
    let mut at = start.min(source.len());

    while let Some(before) = at.checked_sub(1).map(|index| source[index]) {
        match before {
            byte if byte.is_ascii_alphanumeric() || matches!(byte, b'_' | b'.') => at -= 1,
            b')' | b']' => at = opening(source, at - 1)?,
            _ => break,
        }
    }

    Some(at)
}

/// The index of the bracket that `close` closes, reading backwards.
fn opening(source: &[u8], close: usize) -> Option<usize> {
    let (open, shut) = if source[close] == b')' {
        (b'(', b')')
    } else {
        (b'[', b']')
    };
    let mut depth = 0_usize;
    let mut index = close;

    loop {
        if source[index] == shut {
            depth += 1;
        } else if source[index] == open {
            depth -= 1;
            if depth == 0 {
                return Some(index);
            }
        }
        index = index.checked_sub(1)?;
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
///
/// The engine ends each message with ` (in <template>:<line>)`. Where that
/// names the template and the line the diagnostic already states, it is cut:
/// the `error:` line and the `cause` carry both, and a third copy adds
/// nothing. A location naming another template — one an include reached — is
/// kept, because it is said nowhere else.
fn chain(reported: &minijinja::Error) -> Vec<String> {
    let stated = reported
        .name()
        .zip(reported.line())
        .map(|(name, line)| format!(" (in {name}:{line})"));
    let located = |message: String| match &stated {
        Some(suffix) => match message.strip_suffix(suffix.as_str()) {
            Some(head) => head.to_owned(),
            None => message,
        },
        None => message,
    };

    let mut chain = vec![unlabelled(located(reported.to_string()))];
    let mut source = reported.source();

    while let Some(current) = source {
        // The mark `fail` attaches repeats the message the error above it
        // already carries.
        if current.downcast_ref::<Failed>().is_none() {
            chain.push(unlabelled(located(current.to_string())));
        }
        source = current.source();
    }

    chain
}

/// The label the engine writes before the message of an invalid operation.
///
/// It names the engine's own error kind, which is nothing the author wrote:
/// `fail("boom")` reads `invalid operation: boom`, and a range past its cap
/// `invalid operation: range has too many elements`. The message after it is
/// the link of the chain `FR-ERR-011` carries.
const INVALID_OPERATION: &str = "invalid operation: ";

/// One link of the chain, without the engine's [`INVALID_OPERATION`] label.
fn unlabelled(message: String) -> String {
    match message.strip_prefix(INVALID_OPERATION) {
        Some(rest) => rest.to_owned(),
        None => message,
    }
}

#[cfg(test)]
mod tests {
    use super::{during_compile, during_render};
    use crate::error::{Error, RenderReason};
    use std::path::Path;

    /// The engine the two mappings are exercised against.
    fn engine() -> minijinja::Environment<'static> {
        let mut engine = minijinja::Environment::new();
        engine.set_undefined_behavior(minijinja::UndefinedBehavior::Strict);
        engine.set_debug(true);

        engine
    }

    #[test]
    fn rmp_274_no_link_of_the_chain_carries_the_engines_invalid_operation_label() {
        let mut engine = engine();
        engine
            .add_template("big", "{% for i in range(100000000) %}{% endfor %}")
            .expect("the template parses");
        let reported = engine
            .get_template("big")
            .expect("the template is there")
            .render(())
            .expect_err("the range is past the engine's cap");

        let Error::RenderFailed { chain, .. } = during_render("big", &reported) else {
            panic!("an evaluation failure is a RenderFailed");
        };
        assert!(
            chain
                .iter()
                .any(|link| link.starts_with("range has too many elements")),
            "{chain:?}"
        );
        assert!(
            chain.iter().all(|link| !link.contains("invalid operation")),
            "{chain:?}"
        );
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
            ..
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

    #[test]
    fn fr_err_034_an_undefined_value_names_the_expression_the_author_wrote() {
        // Finding E-06: the most common failure a template author meets named
        // no variable. The expression is taken from the template's source.
        let engine = engine();
        let template = engine
            .template_from_named_str("t/needtable.jinja", "{{ table.name }}\n")
            .expect("it compiles");

        let reported = template
            .render(minijinja::context! {})
            .expect_err("table is not defined");

        let Error::RenderFailed {
            undefined, invoked, ..
        } = during_render("t/needtable", &reported)
        else {
            panic!("an undefined value is a render failure");
        };

        assert_eq!(undefined.as_deref(), Some("table.name"));
        assert_eq!(invoked, "t/needtable");
    }

    /// The engine with the lookups of `FR-ENV-020` registered, and the
    /// fixture's context.
    fn surfaced() -> (minijinja::Environment<'static>, minijinja::Value) {
        let mut engine = engine();
        crate::render::surface::register(&mut engine);

        (engine, crate::render::fixture::context())
    }

    /// The condition a render of `source` against the fixture fails with.
    fn failed_with(source: &str) -> Error {
        let (engine, context) = surfaced();
        let reported = engine
            .render_named_str("t.jinja", source, &context)
            .expect_err("the render fails");

        during_render("t", &reported)
    }

    #[test]
    fn r_02_an_undefined_expression_is_quoted_from_where_it_begins() {
        // Finding R-02: the engine's range begins part-way into the
        // expression, and the quote lost its first name.
        for (source, expected) in [
            ("{{ database.nosuch.x }}", Some("database.nosuch.x")),
            ("{{ table('absent').name }}", Some("table('absent').name")),
            ("{{ \"x\".y.z }}", None),
        ] {
            let Error::RenderFailed { undefined, .. } = failed_with(source) else {
                panic!("{source} is a render failure");
            };
            assert_eq!(undefined.as_deref(), expected, "{source}");
        }
    }

    #[test]
    fn r_02_a_lookup_that_found_nothing_is_told_apart_from_a_member_that_is_absent() {
        use crate::error::{LookupKind, Unresolved};

        let (engine, context) = surfaced();
        let unresolved = |expression: &str| super::unresolved(&engine, &context, expression);

        assert_eq!(
            unresolved("table(\"absent\").name"),
            Some(Unresolved {
                call: "table(\"absent\")".to_owned(),
                kind: LookupKind::Table,
                name: "absent".to_owned(),
                table: None,
                document: None,
            })
        );
        assert_eq!(
            unresolved("column('consignment', 'absent').name"),
            Some(Unresolved {
                call: "column('consignment', 'absent')".to_owned(),
                kind: LookupKind::Column,
                name: "absent".to_owned(),
                table: Some("consignment".to_owned()),
                document: None,
            })
        );
        assert_eq!(
            unresolved("column('absent', 'reference')").map(|found| found.kind),
            Some(LookupKind::Table)
        );
        // The table exists, so the member is what is absent.
        assert_eq!(unresolved("table('consignment').nosuch"), None);
        // An argument only the template's scope holds is not evaluated.
        assert_eq!(unresolved("table(t.name).x"), None);
        assert_eq!(unresolved("database.nosuch"), None);
    }

    #[test]
    fn r_09_fail_carries_the_message_apart_and_the_chain_says_it_once() {
        let Error::RenderFailed { reason, chain, .. } = failed_with("{{ fail('boom') }}") else {
            panic!("fail is a render failure");
        };

        assert_eq!(
            reason.as_deref(),
            Some(&RenderReason::Failed("boom".to_owned()))
        );
        assert_eq!(chain.iter().filter(|line| line.contains("boom")).count(), 1);
        assert!(
            chain.iter().all(|line| !line.contains("(in t.jinja")),
            "{chain:?}"
        );
    }
}
