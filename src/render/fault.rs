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
use minijinja::value::ValueKind;

use crate::diagnostics::suggest::{self, Population};
use crate::error::{Error, LookupKind, Missing, Position, RenderReason, Unresolved};

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

/// The context variables of `FR-RND-023`, in the order it names them.
const VARIABLES: [&str; 7] = ["database", "table", "view", "routine", "vars", "tpl", "now"];

/// The step that found nothing in an undefined expression rooted at a context
/// variable of `FR-RND-023`, WHERE the render bound that variable; [`None`]
/// where the expression has another root, or the variable is not bound.
///
/// `FR-RND-023` binds `table`, `view` and `routine` only when the flag of the
/// same name is given, so a bound one means the flag was given, and the hint
/// that asks for it would be false (finding Y-01 of the eighth re-audit of rmp
/// `#263`); the other four are bound in every render, and a step of them that
/// finds nothing is named in the same way (finding Z-01 of the ninth). The
/// walk reads the context itself, step by step, over the attributes and
/// decimal indexes the expression begins with, and stops at the first step
/// that finds nothing.
pub(super) fn missing(context: &minijinja::Value, expression: &str) -> Option<Missing> {
    let (root, rest) = identifier(expression)?;
    let root: &'static str = VARIABLES.into_iter().find(|&variable| variable == root)?;
    let current = bound(context, root)?;

    Some(walk(expression, root.len(), rest, current).unwrap_or(Missing::Elsewhere { root }))
}

/// The first step of `expression` after its first `consumed` bytes that finds
/// nothing in `current`, the value those bytes hold; [`None`] where every step
/// the walk can read finds something.
fn walk(
    expression: &str,
    mut consumed: usize,
    mut rest: &str,
    mut current: minijinja::Value,
) -> Option<Missing> {
    loop {
        let owner = &expression[..consumed];
        if let Some(after) = rest.strip_prefix('.') {
            let Some((name, tail)) = identifier(after) else {
                break;
            };
            let found = current
                .get_attr(name)
                .ok()
                .filter(|value| !value.is_undefined());
            let Some(found) = found else {
                return Some(attribute(owner, name, &current));
            };
            current = found;
            consumed += 1 + name.len();
            rest = tail;
        } else if let Some(after) = rest.strip_prefix('[') {
            let Some((index, length)) = decimal(after) else {
                break;
            };
            // The engine indexes more than lists — a string by character —
            // so a value of another kind is asked itself before it is called
            // not a list.
            if current.kind() != ValueKind::Seq {
                let Some(found) = current
                    .get_item(&minijinja::Value::from(index))
                    .ok()
                    .filter(|value| !value.is_undefined())
                else {
                    return Some(Missing::NotList {
                        owner: owner.to_owned(),
                        index,
                        kind: kind_of(&current),
                    });
                };
                current = found;
                consumed += 1 + length;
                rest = &after[length..];
                continue;
            }
            let items = current.len().unwrap_or(0);
            let position = if index < 0 {
                usize::try_from(index.unsigned_abs())
                    .ok()
                    .and_then(|back| items.checked_sub(back))
            } else {
                usize::try_from(index).ok().filter(|&at| at < items)
            };
            let found = position
                .and_then(|at| current.get_item(&minijinja::Value::from(at)).ok())
                .filter(|value| !value.is_undefined());
            let Some(found) = found else {
                return Some(Missing::Index {
                    owner: owner.to_owned(),
                    index,
                    length: items,
                });
            };
            current = found;
            consumed += 1 + length;
            rest = &after[length..];
        } else {
            break;
        }
    }

    None
}

/// The undefined expression rooted at a context variable's name that the
/// template binds itself — `{% for table in database.tables %}` — described
/// as the template's variable rather than as the render's.
///
/// The render's `table` is not what the expression reads, so the walk over
/// the context would describe the wrong value, and a hint asking for
/// `--table` would be false. Where the name has one binding whose value can be
/// evaluated against the context alone — the first item of a loop's iterable,
/// or the value of a `set` or a `with` — the walk runs over that value; the
/// step it finds is then what the variable holds. Otherwise the step is not
/// determined, and only the first attribute the expression reads is named.
pub(super) fn template_bound(
    engine: &minijinja::Environment<'_>,
    context: &minijinja::Value,
    expression: &str,
    reported: &minijinja::Error,
) -> Option<Missing> {
    let (root, rest) = identifier(expression)?;
    if !VARIABLES.contains(&root) {
        return None;
    }
    let found = bindings(reported.template_source()?, root);
    let first = found.first()?;
    let unique = found.iter().all(|binding| binding == first);
    let evaluate = |text: &str| {
        engine
            .compile_expression(text)
            .and_then(|compiled| compiled.eval(context))
            .ok()
            .filter(|value| !value.is_undefined())
    };
    let held = match (unique, first.keyword) {
        (true, "for") => first
            .value
            .and_then(evaluate)
            .and_then(|iterable| iterable.try_iter().ok()?.next()),
        (true, "set" | "with") => first.value.and_then(evaluate),
        _ => None,
    };
    let step = held
        .and_then(|value| walk(expression, root.len(), rest, value))
        .map(Box::new);
    let read = rest
        .strip_prefix('.')
        .and_then(identifier)
        .map(|(name, _)| name.to_owned());

    Some(Missing::Bound {
        name: root.to_owned(),
        loop_variable: found.iter().all(|binding| binding.keyword == "for"),
        step,
        read,
    })
}

/// One occurrence of a name in a template's source that reads as a binding.
#[derive(Debug, PartialEq, Eq)]
struct Binding<'a> {
    /// What precedes the name: `for`, `set`, `with`, `as`, `import`, `,` or
    /// `(`.
    keyword: &'static str,
    /// The expression the binding gives the name, WHERE it is the whole of a
    /// loop's iterable or of a `set`'s or a `with`'s value.
    value: Option<&'a str>,
}

/// Every occurrence of the word `name` in `source` that reads as a binding
/// the template makes itself: after `for`, `set`, `with`, `as` or `import`,
/// or after a `,` or a `(` that a loop target, a macro's parameters or a call
/// block's list would place it after.
fn bindings<'a>(source: &'a str, name: &str) -> Vec<Binding<'a>> {
    const KEYWORDS: [&str; 5] = ["for", "set", "with", "as", "import"];
    let bytes = source.as_bytes();
    let word = |byte: u8| byte.is_ascii_alphanumeric() || byte == b'_';

    source
        .match_indices(name)
        .filter_map(|(at, _)| {
            let end = at + name.len();
            if (at > 0 && word(bytes[at - 1])) || bytes.get(end).copied().is_some_and(word) {
                return None;
            }
            let before = source[..at].trim_end();
            let keyword = if before.ends_with(',') {
                ","
            } else if before.ends_with('(') {
                "("
            } else {
                KEYWORDS.into_iter().find(|keyword| {
                    before
                        .strip_suffix(keyword)
                        .is_some_and(|head| head.bytes().next_back().is_none_or(|byte| !word(byte)))
                })?
            };
            Some(Binding {
                keyword,
                value: bound_value(keyword, &source[end..]),
            })
        })
        .collect()
}

/// The expression a binding gives its name, read from just after the name:
/// the iterable of `for <name> in <expression> %}`, or the value of
/// `set <name> = <expression> %}` and `with <name> = <expression> %}`; [`None`]
/// for any other shape, a loop filtered with `if` or a `with` of several names
/// included.
fn bound_value<'a>(keyword: &str, after: &'a str) -> Option<&'a str> {
    let after = after.trim_start();
    let value = match keyword {
        "for" => after.strip_prefix("in")?,
        "set" | "with" => after
            .strip_prefix('=')
            .filter(|value| !value.starts_with('='))?,
        _ => return None,
    };
    if keyword == "for" && !value.starts_with(char::is_whitespace) {
        return None;
    }
    let (value, _) = value.split_once("%}")?;
    let value = value.trim().trim_end_matches('-').trim_end();
    let excluded = [" if ", " recursive"];
    (!value.is_empty()
        && !excluded.iter().any(|word| value.contains(word))
        && !(keyword == "with" && value.contains(',')))
    .then_some(value)
}

/// The value `variable` holds in `context`, WHERE the render bound it.
fn bound(context: &minijinja::Value, variable: &str) -> Option<minijinja::Value> {
    context
        .get_attr(variable)
        .ok()
        .filter(|value| !value.is_undefined())
}

/// The root of an undefined expression that is no context variable, with the
/// bound context variables nearest to it (finding Z-04 of the ninth re-audit
/// of rmp `#263`); [`None`] where none is near, or where the name may be one
/// the template binds itself.
///
/// The engine reports `c.nme`, for a loop variable `c`, over the same range as
/// `tabel.name`, so the root alone does not tell a misspelt variable from a
/// template's own. A name is therefore taken for a misspelling only WHERE the
/// failure arose in the template the caller named — an included template or
/// an imported macro sees names its caller bound — and no occurrence of the
/// name in its source reads as a binding: after `for`, `set`, `with`, `as` or
/// `import`, or after a `,` or a `(` that a loop target, a macro's parameters
/// or a call block's list would place it after.
pub(super) fn unbound(
    context: &minijinja::Value,
    expression: &str,
    reported: &minijinja::Error,
    invoked: &str,
) -> Option<Missing> {
    let (root, _) = identifier(expression)?;
    if VARIABLES.contains(&root) || reported.name().is_some_and(|name| name != invoked) {
        return None;
    }
    if reported
        .template_source()
        .is_some_and(|source| binds(source, root))
    {
        return None;
    }

    // AA-03: `table`, `view` and `routine` are candidates whether or not the
    // render bound them, since a misspelt one is most often written without
    // its flag too. A bound candidate is kept only where it holds the
    // attribute the expression reads next, so `tbl.name` is not steered to
    // `tpl`, which has no `name`; an unbound one cannot be read, and is kept.
    let (_, rest) = identifier(expression)?;
    let next = rest
        .strip_prefix('.')
        .and_then(identifier)
        .map(|(attribute, _)| attribute);
    let mut unbound: Vec<&'static str> = Vec::new();
    let variables: Vec<&'static str> = VARIABLES
        .into_iter()
        .filter(|variable| match bound(context, variable) {
            Some(value) => next.is_none_or(|attribute| holds(&value, attribute)),
            None => {
                let object = matches!(*variable, "table" | "view" | "routine");
                if object {
                    unbound.push(variable);
                }
                object
            }
        })
        .collect();
    let nearest: Vec<&'static str> =
        suggest::suggestions(root, variables.iter().copied(), Population::Names)
            .names()
            .collect();
    unbound.retain(|variable| nearest.contains(variable));

    (!nearest.is_empty()).then(|| Missing::Variable {
        name: root.to_owned(),
        nearest,
        unbound,
    })
}

/// Whether `value` holds the attribute `attribute`.
fn holds(value: &minijinja::Value, attribute: &str) -> bool {
    value
        .get_attr(attribute)
        .is_ok_and(|held| !held.is_undefined())
}

/// Whether an occurrence of the word `name` in `source` reads as a binding the
/// template makes itself, per [`unbound`].
fn binds(source: &str, name: &str) -> bool {
    !bindings(source, name).is_empty()
}

/// The name an expression fragment begins with, and what follows it.
fn identifier(fragment: &str) -> Option<(&str, &str)> {
    let end = fragment
        .find(|c: char| !(c.is_ascii_alphanumeric() || c == '_'))
        .unwrap_or(fragment.len());
    let name = &fragment[..end];
    name.bytes()
        .next()
        .is_some_and(|byte| byte.is_ascii_alphabetic() || byte == b'_')
        .then(|| (name, &fragment[end..]))
}

/// The decimal index an expression fragment begins with, read from just after
/// its `[`, and the length up to and including the `]`.
fn decimal(fragment: &str) -> Option<(i64, usize)> {
    let close = fragment.find(']')?;
    let digits = &fragment[..close];
    let unsigned = digits.strip_prefix('-').unwrap_or(digits);
    if unsigned.is_empty() || !unsigned.bytes().all(|byte| byte.is_ascii_digit()) {
        return None;
    }
    Some((digits.parse().ok()?, close + 1))
}

/// The missing attribute `name` of `owner`, with the attributes it holds.
fn attribute(owner: &str, name: &str, value: &minijinja::Value) -> Missing {
    let attributes: Vec<String> = if value.kind() == ValueKind::Map {
        value
            .try_iter()
            .map(|keys| {
                keys.filter_map(|key| key.as_str().map(str::to_owned))
                    .collect()
            })
            .unwrap_or_default()
    } else {
        Vec::new()
    };
    let nearest = suggest::suggestions(
        name,
        attributes.iter().map(String::as_str),
        Population::Names,
    )
    .names()
    .map(str::to_owned)
    .collect();

    Missing::Attribute {
        owner: owner.to_owned(),
        name: name.to_owned(),
        kind: kind_of(value),
        attributes,
        nearest,
    }
}

/// What a value is, in words.
fn kind_of(value: &minijinja::Value) -> &'static str {
    match value.kind() {
        ValueKind::None => "null",
        ValueKind::Bool => "a boolean",
        ValueKind::Number => "a number",
        ValueKind::String => "a string",
        ValueKind::Bytes => "a byte string",
        ValueKind::Seq => "a list",
        ValueKind::Map | ValueKind::Plain => "an object",
        _ => "a value",
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
    if reported.kind() == minijinja::ErrorKind::InvalidOperation
        && reported
            .detail()
            .is_some_and(|detail| detail.ends_with(super::operand::GIVEN_UNDEFINED))
    {
        return operand(reported);
    }

    if reported.kind() != minijinja::ErrorKind::UndefinedError {
        return None;
    }

    let range = reported.range()?;
    let source = reported.template_source()?;
    let start = expression_start(source.as_bytes(), range.start)?;

    // A built-in filter or test the engine refused an undefined operand to
    // reports the range of its own name, as a registered one does.
    let before = source[..start].trim_end();
    if before.ends_with('|') || before.ends_with(" is") || before.ends_with(" not") {
        return operand(reported);
    }

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

/// The source text of the operand a filter or a test was handed undefined
/// (finding T-02 of the third re-audit of rmp `#263`).
///
/// The engine's range covers the filter's or the test's name, so the operand
/// is the expression that ends before the `|`, or before the `is` or `is not`,
/// that precedes it. An operand that is itself the result of another filter —
/// `x|lower|pascal` — is quoted not at all rather than as its last filter.
fn operand(reported: &minijinja::Error) -> Option<String> {
    let range = reported.range()?;
    let source = reported.template_source()?;
    let bytes = source.as_bytes();
    let skip_space = |mut at: usize| {
        while at > 0 && bytes[at - 1].is_ascii_whitespace() {
            at -= 1;
        }
        at
    };
    let word_before = |at: usize, word: &[u8]| {
        at >= word.len()
            && &bytes[at - word.len()..at] == word
            && (at == word.len() || !bytes[at - word.len() - 1].is_ascii_alphanumeric())
    };

    let mut end = skip_space(range.start.min(bytes.len()));
    if end > 0 && bytes[end - 1] == b'|' {
        end -= 1;
    } else {
        if word_before(end, b"not") {
            end = skip_space(end - 3);
        }
        if !word_before(end, b"is") {
            return None;
        }
        end -= 2;
    }
    let end = skip_space(end);
    let start = expression_start(bytes, end)?;
    let expression = source.get(start..end)?.trim();

    let piped = skip_space(start) > 0 && bytes[skip_space(start) - 1] == b'|';
    let named = expression
        .bytes()
        .next()
        .is_some_and(|byte| byte.is_ascii_alphabetic() || byte == b'_');

    (named && !piped && !expression.contains('\n') && expression.len() <= 128)
        .then(|| expression.to_owned())
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
    fn t_02_an_undefined_operand_of_a_filter_or_a_test_is_quoted() {
        for (source, expected) in [
            ("{{ table.name|pascal }}", Some("table.name")),
            ("x\n  {{ vars.title | pascal }}", Some("vars.title")),
            ("{% if nothing is nullable %}{% endif %}", Some("nothing")),
            (
                "{% if nothing is not nullable %}{% endif %}",
                Some("nothing"),
            ),
            ("{{ nothing|lower|pascal }}", Some("nothing")),
            ("{{ nothing|lower }}", Some("nothing")),
        ] {
            let Error::RenderFailed { undefined, .. } = failed_with(source) else {
                panic!("{source} is a render failure");
            };
            assert_eq!(undefined.as_deref(), expected, "{source}");
        }
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

    /// Finding Y-01 of the eighth re-audit of rmp `#263`: the walk names the
    /// step that found nothing, and only for a bound object variable.
    #[test]
    fn the_walk_names_the_step_of_a_bound_variable_that_found_nothing() {
        use crate::error::Missing;

        let context = minijinja::Value::from_serialize(serde_json::json!({
            "table": {"name": "orders", "columns": [{"name": "id"}]},
        }));

        assert_eq!(super::missing(&context, "view.name"), None);
        assert_eq!(super::missing(&context, "database.x"), None);
        assert_eq!(
            super::missing(&context, "table.columns[-2].name"),
            Some(Missing::Index {
                owner: "table.columns".to_owned(),
                index: -2,
                length: 1,
            })
        );
        assert_eq!(
            super::missing(&context, "table.columns[-1].nme"),
            Some(Missing::Attribute {
                owner: "table.columns[-1]".to_owned(),
                name: "nme".to_owned(),
                kind: "an object",
                attributes: vec!["name".to_owned()],
                nearest: vec!["name".to_owned()],
            })
        );
        assert_eq!(
            super::missing(&context, "table.columns[0].name[9]"),
            Some(Missing::NotList {
                owner: "table.columns[0].name".to_owned(),
                index: 9,
                kind: "a string",
            })
        );
        assert_eq!(
            super::missing(&context, "table.name|upper"),
            Some(Missing::Elsewhere { root: "table" })
        );
    }

    /// Finding Z-01 of the ninth re-audit of rmp `#263`: the walk reaches
    /// every context variable, `vars` included.
    #[test]
    fn z_01_the_walk_reaches_every_bound_context_variable() {
        use crate::error::Missing;

        let context = minijinja::Value::from_serialize(serde_json::json!({
            "database": {"name": "shop", "tables": [{"name": "a"}, {"name": "b"}]},
            "vars": {"title": "x"},
            "tpl": {"version": "1.0.0"},
            "now": "1970-01-01T00:00:00Z",
        }));

        assert_eq!(
            super::missing(&context, "database.tables[3].name"),
            Some(Missing::Index {
                owner: "database.tables".to_owned(),
                index: 3,
                length: 2,
            })
        );
        assert_eq!(
            super::missing(&context, "tpl.vrsion"),
            Some(Missing::Attribute {
                owner: "tpl".to_owned(),
                name: "vrsion".to_owned(),
                kind: "an object",
                attributes: vec!["version".to_owned()],
                nearest: vec!["version".to_owned()],
            })
        );
        assert_eq!(
            super::missing(&context, "vars.titl"),
            Some(Missing::Attribute {
                owner: "vars".to_owned(),
                name: "titl".to_owned(),
                kind: "an object",
                attributes: vec!["title".to_owned()],
                nearest: vec!["title".to_owned()],
            })
        );
        assert!(matches!(
            super::missing(&context, "now.year"),
            Some(Missing::Attribute {
                kind: "a string",
                ..
            })
        ));
        assert_eq!(super::missing(&context, "table.name"), None);
    }

    /// The undefined root of a render of `source` as `name`, per
    /// [`super::unbound`].
    fn root_of(source: &str, invoked: &str) -> Option<crate::error::Missing> {
        let context = minijinja::Value::from_serialize(serde_json::json!({
            "database": {"name": "shop"},
            "table": {"name": "orders", "columns": [{"name": "id"}]},
            "vars": {},
            "tpl": {"version": "1.0.0"},
            "now": "1970-01-01T00:00:00Z",
        }));
        let mut engine = engine();
        engine
            .add_template("t.jinja", source)
            .expect("the template parses");
        let reported = engine
            .get_template("t.jinja")
            .expect("the template is there")
            .render(&context)
            .expect_err("the render fails");
        let Error::RenderFailed {
            undefined: Some(expression),
            ..
        } = during_render("t.jinja", &reported)
        else {
            panic!("{source} quotes no undefined expression");
        };

        super::unbound(&context, &expression, &reported, invoked)
    }

    /// Finding Z-04 of the ninth re-audit of rmp `#263`.
    #[test]
    fn z_04_a_misspelt_root_is_matched_only_where_the_template_binds_no_such_name() {
        use crate::error::Missing;

        assert_eq!(
            root_of("{{ tabel.name }}", "t.jinja"),
            Some(Missing::Variable {
                name: "tabel".to_owned(),
                nearest: vec!["table"],
                unbound: vec![],
            })
        );
        // AA-03: an object variable the render did not bind is a candidate,
        // and a bound one that lacks the attribute read next is not.
        assert_eq!(
            root_of("{{ viw.name }}", "t.jinja"),
            Some(Missing::Variable {
                name: "viw".to_owned(),
                nearest: vec!["view"],
                unbound: vec!["view"],
            })
        );
        assert_eq!(
            root_of("{{ tbl.name }}", "t.jinja"),
            Some(Missing::Variable {
                name: "tbl".to_owned(),
                nearest: vec!["table"],
                unbound: vec![],
            })
        );
        assert_eq!(
            root_of("{{ tbl.version }}", "t.jinja"),
            Some(Missing::Variable {
                name: "tbl".to_owned(),
                nearest: vec!["tpl"],
                unbound: vec![],
            })
        );
        assert_eq!(
            root_of("{{ tabel }}{{ tabel.name }}", "t.jinja").map(|_| ()),
            Some(())
        );
        // Another template than the one invoked may see its caller's names.
        assert_eq!(root_of("{{ tabel.name }}", "other.jinja"), None);
        // Nothing near.
        assert_eq!(root_of("{{ zzzzzz.name }}", "t.jinja"), None);
        for bound in [
            "{% for row in table.columns %}{{ row.nme }}{% endfor %}",
            "{% for i, row in [[0, table]] %}{{ row.nme }}{% endfor %}",
            "{% macro m(row) %}{{ row.nme }}{% endmacro %}{{ m(table) }}",
            "{% set row = table %}{{ row.nme }}",
            "{% with row = table %}{{ row.nme }}{% endwith %}",
        ] {
            assert_eq!(root_of(bound, "t.jinja"), None, "{bound}");
        }
    }

    #[test]
    fn z_04_a_binding_is_read_only_where_the_whole_word_follows_the_keyword() {
        assert!(super::binds("{% for row in x %}", "row"));
        assert!(super::binds("{% set a, row = x %}", "row"));
        assert!(!super::binds("{{ arrow.x }}{{ row.y }}", "row"));
        assert!(!super::binds("{% set rows = x %}{{ row.y }}", "row"));
        assert!(!super::binds("{{ offset }}{{ row.y }}", "row"));
    }

    #[test]
    fn a_binding_carries_its_value_only_where_the_whole_of_it_is_one_expression() {
        fn value(source: &str) -> Vec<(&'static str, Option<&str>)> {
            super::bindings(source, "table")
                .into_iter()
                .map(|binding| (binding.keyword, binding.value))
                .collect()
        }

        assert_eq!(
            value("{% for table in database.tables -%}"),
            [("for", Some("database.tables"))]
        );
        assert_eq!(
            value("{% for table in database.tables if table.name %}"),
            [("for", None)]
        );
        assert_eq!(
            value("{% set table = database.tables[0] %}"),
            [("set", Some("database.tables[0]"))]
        );
        assert_eq!(value("{% with table = a, b = c %}"), [("with", None)]);
        assert_eq!(value("{% macro m(table) %}"), [("(", None)]);
        assert_eq!(value("{{ table.name }}"), []);
    }
}
