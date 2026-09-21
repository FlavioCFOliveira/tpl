//! The six code filters of `FR-ENV-007`.
//!
//! | Filter | What it produces | Fixed by |
//! |---|---|---|
//! | `quote` | A MariaDB identifier, backtick-quoted, every backtick doubled | `FR-ENV-008`, `FR-ENV-035`, `FR-ENV-045` |
//! | `sql_type` | The column's normalised type name, and nothing else | `FR-ENV-039` |
//! | `json` | The operand as one line of JSON | `FR-ENV-036`, `FR-OUT-007` |
//! | `indent(n)` | Every line after the first prefixed with `n` spaces | `FR-ENV-037` |
//! | `comment(prefix)` | Every line prefixed with `prefix` and one space | `FR-ENV-038` |
//! | `escape(target)` | Five characters replaced, for `html` or for `xml` | `FR-ENV-028`, `FR-ENV-044` |
//!
//! Three of the six take a **required** argument with no default, and
//! `FR-ENV-037`, `FR-ENV-038` and `FR-ENV-044` each give the same reason: a
//! value nobody can see in the template is a value that changes the generated
//! bytes without appearing in the source. Two of those three names shadow an
//! engine built-in of a different arity, deliberately and as `ADR-001` records,
//! so a template written against the engine's signature fails loudly here
//! rather than emitting different bytes.
//!
//! # What each accepts
//!
//! `FR-SEM-008` fixes it and [`super::operand`] enforces it: five of the six
//! accept a string, `sql_type` accepts a column object, and `json` accepts any
//! type the render context can hold. None coerces, per `FR-SEM-009`, and none
//! answers with an empty string for an operand it does not accept.
//!
//! # Where a copy is avoided
//!
//! `indent` and `escape` return the operand **itself** where the transformation
//! would not change it — a single-line operand, a string carrying none of the
//! five characters. The value is reference-counted by the engine, so returning
//! it costs no copy of the string, which is what the project's
//! minimal-allocation rule asks for on a path a loop reaches.

use minijinja::{Error, ErrorKind, Value};

use super::operand::{self, Role};

/// The identifier delimiter MariaDB accepts, per `FR-ENV-045`.
const BACKTICK: char = '`';

/// The `target` of `FR-ENV-044` that escapes an apostrophe numerically.
const HTML: &str = "html";

/// The `target` of `FR-ENV-044` that escapes it by name.
const XML: &str = "xml";

/// `quote` (`FR-ENV-008`, `FR-ENV-035`, `FR-ENV-045`).
///
/// Every backtick within the identifier is doubled and the whole is enclosed
/// in a single pair, which is the form every series of `FR-SRV-015` was
/// observed to accept.
///
/// # Errors
///
/// Returns the refusal of `FR-SEM-008` for an operand that is not a string.
pub(super) fn quote(value: &Value) -> Result<Value, Error> {
    let text = operand::text(value, "quote")?;
    let mut quoted = String::with_capacity(text.len() + 2);

    quoted.push(BACKTICK);
    for character in text.chars() {
        if character == BACKTICK {
            quoted.push(BACKTICK);
        }
        quoted.push(character);
    }
    quoted.push(BACKTICK);

    Ok(Value::from(quoted))
}

/// `sql_type` (`FR-ENV-039`).
///
/// The value of the column's `data_type` field, exactly as `FR-CTX-015`
/// carries it, and nothing else: no parsing, no reassembled type clause, and
/// no reading of `column_type`. An unrecognised type carries `null` there, per
/// `FR-CTX-018`, and `null` is what the filter returns.
///
/// # Errors
///
/// Returns the refusal of `FR-ENV-040` for an operand that is not a column
/// object.
pub(super) fn sql_type(value: &Value) -> Result<Value, Error> {
    operand::Column::of(value, Role::Filter, "sql_type")?.data_type()
}

/// `json` (`FR-ENV-036`).
///
/// The compact form of `FR-OUT-007`: one line, no superfluous whitespace, and
/// no trailing newline.
///
/// # Errors
///
/// Returns the refusal of `FR-SEM-008` for an operand that is undefined, which
/// is absence rather than a type the context can hold, and an
/// [`ErrorKind::BadSerialization`] for a value the serialiser refuses.
pub(super) fn json(value: &Value) -> Result<Value, Error> {
    if value.is_undefined() {
        return Err(operand::refused(
            Role::Filter,
            "json",
            "any value the render context can hold",
            value,
        ));
    }

    serde_json::to_string(value)
        .map(Value::from)
        .map_err(|returned| {
            Error::new(
                ErrorKind::BadSerialization,
                "the filter 'json' could not serialise its operand",
            )
            .with_source(returned)
        })
}

/// `indent(n)` (`FR-ENV-037`).
///
/// Every line after the first is prefixed with `n` spaces. The first line is
/// not indented, an empty line is not indented, and no trailing whitespace is
/// added.
///
/// # Errors
///
/// Returns the refusal of `FR-SEM-008` for an operand that is not a string.
/// An invocation with no argument is the engine's own missing-argument
/// condition, which `FR-ENV-037` requires by making the argument required.
pub(super) fn indent(value: &Value, width: usize) -> Result<Value, Error> {
    let text = operand::text(value, "indent")?;

    if !text.contains('\n') {
        return Ok(value.clone());
    }

    let mut indented = String::with_capacity(text.len() + width * 4);

    for (index, line) in text.split_inclusive('\n').enumerate() {
        if index > 0 && !is_empty_line(line) {
            for _ in 0..width {
                indented.push(' ');
            }
        }
        indented.push_str(line);
    }

    Ok(Value::from(indented))
}

/// `comment(prefix)` (`FR-ENV-038`).
///
/// Every line is prefixed with `prefix` followed by a single space. The filter
/// chooses no comment syntax of its own, because a comment syntax is an
/// opinion about a target language and `BR-ENV-002` keeps such an opinion with
/// the project rather than in the binary.
///
/// An operand carrying no character has no line, so it produces no comment
/// marker: `{{ table.comment | comment("//") }}` over the empty string
/// `FR-CAT-039` gives a table with no comment emits nothing rather than a bare
/// marker.
///
/// # Errors
///
/// Returns the refusal of `FR-SEM-008` for an operand that is not a string.
pub(super) fn comment(value: &Value, prefix: &str) -> Result<Value, Error> {
    let text = operand::text(value, "comment")?;
    let mut commented = String::with_capacity(text.len() + prefix.len() + 1);

    for line in text.split_inclusive('\n') {
        commented.push_str(prefix);
        commented.push(' ');
        commented.push_str(line);
    }

    Ok(Value::from(commented))
}

/// `escape(target)` (`FR-ENV-028`, `FR-ENV-044`).
///
/// Five characters are replaced and no other. `&` is replaced first, which a
/// single pass over the operand guarantees: a replacement is written to the
/// output and never read again, so nothing is escaped twice. An
/// already-escaped entity is treated as text, because the filter escapes text
/// and does not inspect it for markup.
///
/// # Errors
///
/// Returns the refusal of `FR-SEM-008` for an operand that is not a string,
/// and a refusal naming the filter, the value received and the two permitted
/// values for a `target` that is neither `html` nor `xml`.
pub(super) fn escape(value: &Value, target: &str) -> Result<Value, Error> {
    let apostrophe = match target {
        HTML => "&#39;",
        XML => "&apos;",
        _ => {
            return Err(Error::new(
                ErrorKind::InvalidOperation,
                format!(
                    "the filter 'escape' was given the target '{target}', and accepts \
                     '{HTML}' or '{XML}'"
                ),
            ));
        }
    };

    let text = operand::text(value, "escape")?;

    if !text.contains(needs_escaping) {
        return Ok(value.clone());
    }

    let mut escaped = String::with_capacity(text.len() + 16);

    for character in text.chars() {
        match character {
            '&' => escaped.push_str("&amp;"),
            '<' => escaped.push_str("&lt;"),
            '>' => escaped.push_str("&gt;"),
            '"' => escaped.push_str("&quot;"),
            '\'' => escaped.push_str(apostrophe),
            other => escaped.push(other),
        }
    }

    Ok(Value::from(escaped))
}

/// Whether a line carries nothing but its terminator (`FR-ENV-037`).
fn is_empty_line(line: &str) -> bool {
    line.trim_end_matches('\n').is_empty()
}

/// The five characters of `FR-ENV-044`'s table.
const fn needs_escaping(character: char) -> bool {
    matches!(character, '&' | '<' | '>' | '"' | '\'')
}

#[cfg(test)]
mod tests {
    use super::{comment, escape, indent, json, quote, sql_type};
    use crate::render::fixture;
    use minijinja::Value;

    /// The string a filter returned.
    fn rendered(value: &Value) -> String {
        value
            .as_str()
            .expect("the filter returns a string")
            .to_owned()
    }

    #[test]
    fn fr_env_045_quote_doubles_every_backtick_inside_one_pair() {
        // FR-ENV-045, with the worked case the requirement states: the
        // identifier `back` + backtick + `tick` yields `back``tick`.
        assert_eq!(
            rendered(&quote(&Value::from("back`tick")).expect("it is a string")),
            "`back``tick`"
        );
        assert_eq!(
            rendered(&quote(&Value::from("consignment")).expect("it is a string")),
            "`consignment`"
        );
    }

    #[test]
    fn fr_env_036_json_is_the_compact_form_of_four_operands() {
        // FR-ENV-036, every row of its table.
        assert_eq!(
            rendered(&json(&Value::from("a\"b")).expect("a string is a value")),
            r#""a\"b""#
        );
        assert_eq!(
            rendered(&json(&Value::from(())).expect("null is a value")),
            "null"
        );
        assert_eq!(
            rendered(&json(&Value::from(vec!["a", "b"])).expect("a list is a value")),
            r#"["a","b"]"#
        );
        assert_eq!(
            rendered(&json(&minijinja::context! { k => "v" }).expect("a map is a value")),
            r#"{"k":"v"}"#
        );
    }

    #[test]
    fn fr_env_037_indent_leaves_the_first_line_and_every_empty_line_alone() {
        // FR-ENV-037: no first line, no empty line, no trailing whitespace.
        let indented = indent(&Value::from("one\n\ntwo\n"), 2).expect("it is a string");

        assert_eq!(rendered(&indented), "one\n\n  two\n");
    }

    #[test]
    fn fr_env_037_indent_returns_a_single_line_operand_unchanged() {
        let value = Value::from("one line");

        assert_eq!(
            rendered(&indent(&value, 4).expect("it is a string")),
            "one line"
        );
    }

    #[test]
    fn fr_env_038_comment_prefixes_every_line_and_chooses_no_syntax() {
        // FR-ENV-038, with the two prefixes the requirement shows.
        let value = Value::from("one\ntwo\n");

        assert_eq!(
            rendered(&comment(&value, "//").expect("it is a string")),
            "// one\n// two\n"
        );
        assert_eq!(
            rendered(&comment(&value, "--").expect("it is a string")),
            "-- one\n-- two\n"
        );
        assert_eq!(
            rendered(&comment(&Value::from(""), "//").expect("it is a string")),
            ""
        );
    }

    #[test]
    fn fr_env_044_escape_replaces_five_characters_and_differs_in_one_row() {
        // FR-ENV-044, every row of its table, for both targets.
        let value = Value::from("&<>\"'");

        assert_eq!(
            rendered(&escape(&value, "html").expect("it is a string")),
            "&amp;&lt;&gt;&quot;&#39;"
        );
        assert_eq!(
            rendered(&escape(&value, "xml").expect("it is a string")),
            "&amp;&lt;&gt;&quot;&apos;"
        );
    }

    #[test]
    fn fr_env_044_escape_escapes_an_entity_again_and_nothing_twice() {
        // FR-ENV-044: an input `&amp;` becomes `&amp;amp;`, because the filter
        // escapes text and does not inspect it for markup; and `&` is replaced
        // before the others, so no replacement is escaped twice.
        assert_eq!(
            rendered(&escape(&Value::from("&amp;"), "html").expect("it is a string")),
            "&amp;amp;"
        );
    }

    #[test]
    fn fr_env_044_escape_refuses_a_target_outside_the_two() {
        // FR-ENV-044: naming the filter, the value received, and the two
        // permitted values.
        let condition = escape(&Value::from("x"), "latex").expect_err("latex is not a target");
        let message = condition.to_string();

        assert!(message.contains("escape"), "{message}");
        assert!(message.contains("latex"), "{message}");
        assert!(
            message.contains("html") && message.contains("xml"),
            "{message}"
        );
    }

    #[test]
    fn fr_env_039_sql_type_returns_the_normalised_type_name() {
        // FR-ENV-039, rows one and three of its table.
        let bigint = sql_type(&fixture::column("consignment_id")).expect("it is a column");
        let unrecognised =
            sql_type(&fixture::column_of_unrecognised_type()).expect("it is a column");

        assert_eq!(rendered(&bigint), "bigint");
        assert!(unrecognised.is_none());
    }

    #[test]
    fn fr_env_039_sql_type_accepts_a_column_object_and_nothing_else() {
        // FR-ENV-039: applied to a table, a string, a number or null the
        // render fails, naming the filter and the type received. It never
        // returns an empty string for an operand it does not accept.
        for other in [
            fixture::table(),
            Value::from("varchar"),
            Value::from(1),
            Value::from(()),
        ] {
            let condition = sql_type(&other).expect_err("it is not a column object");

            assert!(condition.to_string().contains("sql_type"));
        }
    }

    #[test]
    fn fr_sem_008_every_string_filter_refuses_a_number() {
        // FR-SEM-008 over the table of what each filter accepts, and
        // FR-SEM-009: no coercion, and never an empty string in place of a
        // refusal.
        let number = Value::from(42);

        assert!(quote(&number).is_err());
        assert!(indent(&number, 2).is_err());
        assert!(comment(&number, "//").is_err());
        assert!(escape(&number, "html").is_err());
    }
}
