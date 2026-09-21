//! The five naming filters of `FR-ENV-006`, and the one word list they share.
//!
//! `FR-ENV-030` derives a word list from the operand by five rules applied
//! once, left to right, and `FR-ENV-033` makes each of the five filters a pure
//! function of that list. The derivation is therefore written **once**, in
//! [`words`], and each filter is the joining rule its column of the table
//! states:
//!
//! | Filter | Each word | Joined with |
//! |---|---|---|
//! | `pascal` | first character upper, the rest lower | nothing |
//! | `camel` | as `pascal`, except the first word, which is wholly lower | nothing |
//! | `snake` | wholly lower | `_` |
//! | `upper_snake` | wholly upper | `_` |
//! | `kebab` | wholly lower | `-` |
//!
//! Case folding is ASCII only, per `FR-ENV-031`: a character outside `A-Z` and
//! `a-z` passes through unchanged, so the output does not depend on the server,
//! on a collation or on a locale, which is what `NFR-DET-001` requires of every
//! byte a template produces.
//!
//! # The scanner, rule by rule
//!
//! [`words`] visits each word without materialising the list, so the five
//! filters allocate the output string and nothing else. Each rule of
//! `FR-ENV-030` is one arm of the loop:
//!
//! | Rule | Where it is |
//! |---|---|
//! | 1 — an underscore, a hyphen or a space ends the word and is dropped | [`is_separator`] |
//! | 2 — a lower-to-upper transition ends the word before the upper letter | [`breaks_before`], first arm |
//! | 3 — an upper-to-lower transition ends the word before the upper letter, where that letter follows another upper one | [`breaks_before`], second arm |
//! | 4 — a digit belongs to the word it follows and begins none | Neither arm matches a digit, so a digit never breaks |
//! | 5 — an empty word is discarded | The two guards that visit only a non-empty span |
//!
//! `FR-ENV-034` makes a non-string operand a `65` naming the filter, and
//! `FR-SEM-009` forbids coercing one: both are [`super::operand::text`]'s.

use minijinja::{Error, Value};

use super::operand;

/// `pascal` (`FR-ENV-006`, `FR-ENV-033`).
///
/// # Errors
///
/// Returns the refusal of `FR-ENV-034` for an operand that is not a string.
pub(super) fn pascal(value: &Value) -> Result<Value, Error> {
    let text = operand::text(value, "pascal")?;
    let mut joined = output(text);

    words(text, |_, word| push_capitalised(&mut joined, word));

    Ok(Value::from(joined))
}

/// `camel` (`FR-ENV-006`, `FR-ENV-033`).
///
/// # Errors
///
/// Returns the refusal of `FR-ENV-034` for an operand that is not a string.
pub(super) fn camel(value: &Value) -> Result<Value, Error> {
    let text = operand::text(value, "camel")?;
    let mut joined = output(text);

    words(text, |index, word| {
        if index == 0 {
            push_lower(&mut joined, word);
        } else {
            push_capitalised(&mut joined, word);
        }
    });

    Ok(Value::from(joined))
}

/// `snake` (`FR-ENV-006`, `FR-ENV-033`).
///
/// # Errors
///
/// Returns the refusal of `FR-ENV-034` for an operand that is not a string.
pub(super) fn snake(value: &Value) -> Result<Value, Error> {
    delimited(value, "snake", '_', push_lower)
}

/// `upper_snake` (`FR-ENV-006`, `FR-ENV-033`).
///
/// # Errors
///
/// Returns the refusal of `FR-ENV-034` for an operand that is not a string.
pub(super) fn upper_snake(value: &Value) -> Result<Value, Error> {
    delimited(value, "upper_snake", '_', push_upper)
}

/// `kebab` (`FR-ENV-006`, `FR-ENV-033`).
///
/// # Errors
///
/// Returns the refusal of `FR-ENV-034` for an operand that is not a string.
pub(super) fn kebab(value: &Value) -> Result<Value, Error> {
    delimited(value, "kebab", '-', push_lower)
}

/// The three filters that join with a delimiter, written once.
fn delimited(
    value: &Value,
    name: &str,
    delimiter: char,
    push: fn(&mut String, &str),
) -> Result<Value, Error> {
    let text = operand::text(value, name)?;
    let mut joined = output(text);

    words(text, |index, word| {
        if index > 0 {
            joined.push(delimiter);
        }
        push(&mut joined, word);
    });

    Ok(Value::from(joined))
}

/// The output buffer for an operand of this length.
///
/// The transformed operand is the operand's own characters, less the
/// separators the word list drops, plus one delimiter per word boundary. The
/// operand's length plus a small margin therefore covers every identifier the
/// catalogue produces without a second allocation, and the string grows where
/// one does not.
fn output(text: &str) -> String {
    String::with_capacity(text.len() + 4)
}

/// Visits each word of `operand` in order, per `FR-ENV-030`.
///
/// `visit` is called with the word's index in the list and the word itself,
/// borrowed from the operand. An empty word is never visited, which is rule 5.
fn words<'a>(operand: &'a str, mut visit: impl FnMut(usize, &'a str)) {
    let mut index = 0;
    let mut start = 0;
    let mut previous = None;
    let mut characters = operand.char_indices().peekable();

    while let Some((at, current)) = characters.next() {
        let next = characters.peek().map(|&(_, character)| character);

        if is_separator(current) {
            if at > start {
                visit(index, &operand[start..at]);
                index += 1;
            }
            start = at + current.len_utf8();
        } else if current.is_ascii_uppercase() && breaks_before(previous, next) {
            if at > start {
                visit(index, &operand[start..at]);
                index += 1;
            }
            start = at;
        }

        previous = Some(current);
    }

    if start < operand.len() {
        visit(index, &operand[start..]);
    }
}

/// Rule 1 of `FR-ENV-030`: the three characters that end a word and are
/// dropped.
const fn is_separator(character: char) -> bool {
    matches!(character, '_' | '-' | ' ')
}

/// Rules 2 and 3 of `FR-ENV-030`, at an upper-case character.
///
/// The current character is known to be an upper-case ASCII letter. Rule 2 is
/// the lower-to-upper transition, which ends the word before it. Rule 3 is the
/// upper-to-lower transition, which ends the word before the **upper** letter
/// where that letter follows another upper one — which is this position, seen
/// one character early: `HTTPServer` breaks at `S` because `P` precedes it and
/// `e` follows it.
fn breaks_before(previous: Option<char>, next: Option<char>) -> bool {
    match previous {
        Some(character) if character.is_ascii_lowercase() => true,
        Some(character) if character.is_ascii_uppercase() => {
            next.is_some_and(|following| following.is_ascii_lowercase())
        }
        Some(_) | None => false,
    }
}

/// The first character upper-cased, the rest lower-cased (`FR-ENV-031`).
fn push_capitalised(joined: &mut String, word: &str) {
    let mut characters = word.chars();

    if let Some(first) = characters.next() {
        joined.push(first.to_ascii_uppercase());
        joined.extend(characters.map(|character| character.to_ascii_lowercase()));
    }
}

/// Every character lower-cased (`FR-ENV-031`).
fn push_lower(joined: &mut String, word: &str) {
    joined.extend(word.chars().map(|character| character.to_ascii_lowercase()));
}

/// Every character upper-cased (`FR-ENV-031`).
fn push_upper(joined: &mut String, word: &str) {
    joined.extend(word.chars().map(|character| character.to_ascii_uppercase()));
}

#[cfg(test)]
mod tests {
    use super::{camel, kebab, pascal, snake, upper_snake, words};
    use minijinja::Value;

    /// The word list of `operand`, materialised for comparison.
    fn listed(operand: &str) -> Vec<&str> {
        let mut list = Vec::new();
        words(operand, |_, word| list.push(word));

        list
    }

    /// The five filters applied to `operand`, in the order of `FR-ENV-033`.
    fn five(operand: &str) -> [String; 5] {
        let value = Value::from(operand);

        [pascal, camel, snake, upper_snake, kebab].map(|filter| {
            filter(&value)
                .expect("the operand is a string")
                .as_str()
                .expect("a naming filter returns a string")
                .to_owned()
        })
    }

    #[test]
    fn fr_env_032_the_word_list_is_the_eight_rows_of_the_table() {
        // FR-ENV-032, every row. BR-ENV-007 makes the table the test vector.
        assert_eq!(listed("order_items"), ["order", "items"]);
        assert_eq!(listed("orderItems"), ["order", "Items"]);
        assert_eq!(listed("HTTP_server"), ["HTTP", "server"]);
        assert_eq!(listed("HTTPServer"), ["HTTP", "Server"]);
        assert_eq!(listed("order_2_items"), ["order", "2", "items"]);
        assert_eq!(listed("utf8mb4"), ["utf8mb4"]);
        assert_eq!(listed("__orders__"), ["orders"]);
        assert_eq!(listed(""), Vec::<&str>::new());
    }

    #[test]
    fn fr_env_030_a_hyphen_and_a_space_end_a_word_as_an_underscore_does() {
        // Rule 1 names three characters and the table exercises one of them.
        assert_eq!(listed("order-items"), ["order", "items"]);
        assert_eq!(listed("order items"), ["order", "items"]);
    }

    #[test]
    fn fr_env_033_the_five_filters_produce_the_eight_rows_of_the_table() {
        // FR-ENV-033, every row and every column. BR-ENV-007 makes a change to
        // any cell a breaking change to the contract of FR-ENV-002.
        assert_eq!(
            five("order_items"),
            [
                "OrderItems",
                "orderItems",
                "order_items",
                "ORDER_ITEMS",
                "order-items"
            ]
        );
        assert_eq!(
            five("orderItems"),
            [
                "OrderItems",
                "orderItems",
                "order_items",
                "ORDER_ITEMS",
                "order-items"
            ]
        );
        assert_eq!(
            five("HTTP_server"),
            [
                "HttpServer",
                "httpServer",
                "http_server",
                "HTTP_SERVER",
                "http-server"
            ]
        );
        assert_eq!(
            five("HTTPServer"),
            [
                "HttpServer",
                "httpServer",
                "http_server",
                "HTTP_SERVER",
                "http-server"
            ]
        );
        assert_eq!(
            five("order_2_items"),
            [
                "Order2Items",
                "order2Items",
                "order_2_items",
                "ORDER_2_ITEMS",
                "order-2-items"
            ]
        );
        assert_eq!(
            five("utf8mb4"),
            ["Utf8mb4", "utf8mb4", "utf8mb4", "UTF8MB4", "utf8mb4"]
        );
        assert_eq!(
            five("orders"),
            ["Orders", "orders", "orders", "ORDERS", "orders"]
        );
        assert_eq!(five(""), ["", "", "", "", ""]);
    }

    #[test]
    fn fr_env_031_case_folding_is_ascii_only() {
        // FR-ENV-031: a character outside A-Z and a-z passes through
        // unchanged, whatever a locale would do with it. `É` is therefore
        // neither lower-cased by `snake` nor upper-cased by `upper_snake`, and
        // `pascal` and `camel` agree because the first character of the first
        // word is one the fold does not reach.
        assert_eq!(listed("ÉTAT_civil"), ["ÉTAT", "civil"]);
        assert_eq!(
            five("ÉTAT_civil"),
            [
                "ÉtatCivil",
                "ÉtatCivil",
                "État_civil",
                "ÉTAT_CIVIL",
                "État-civil"
            ]
        );
    }

    #[test]
    fn fr_env_034_a_naming_filter_refuses_an_operand_that_is_not_a_string() {
        // FR-ENV-034, with FR-SEM-008 and FR-SEM-009: `{{ 42 | snake }}` fails
        // and names the filter.
        for filter in [pascal, camel, snake, upper_snake, kebab] {
            let condition = filter(&Value::from(42)).expect_err("a number is not a string");

            assert!(condition.to_string().contains("number"));
        }
    }
}
