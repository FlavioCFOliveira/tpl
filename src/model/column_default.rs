//! A column's default: the discriminated structure of `FR-CTX-011`, and the
//! shape classifier of `FR-CTX-037` that decides which form it takes.
//!
//! The catalogue answers the question with one string, and `BR-CTX-002` is why
//! that string does not reach a template: the server separates a literal from
//! an expression by the **shape** of the field and by nothing else, so a
//! document carrying the raw value would make every template reimplement the
//! same classification, and each one would get a different case wrong — an
//! empty string, an apostrophe inside a literal, a function call whose name
//! looks like a word.
//!
//! | Requirement | What the type does with it |
//! |---|---|
//! | `FR-CTX-011` — a structure carrying a `kind`, or `null` | [`ColumnDefault::classify`] returns an [`Option`], whose [`None`] is the document's bare `null` |
//! | `FR-CTX-012` — exactly the forms listed, and no others | [`ColumnDefault`] is a closed enum of three variants, and the fourth form is the absent one |
//! | `FR-CTX-013` — `kind` takes exactly three values | [`DefaultKind`] is a closed enum of three, and the `null` kind has no field a value could occupy |
//! | `FR-CTX-037` — eight shapes, applied in the order written | [`ColumnDefault::classify`] tests them in that order and returns on the first match |
//!
//! The classification is **total**. The final row of `FR-CTX-037` matches every
//! string, so reading a default cannot fail and this module reports no error.
//!
//! *The residual risk is the requirement's own, and is recorded here because
//! the code cannot mitigate it.* The eight rows were derived from 43 distinct
//! values observed over 301 columns on all four series of `FR-SRV-015`. A value
//! outside that population is classified by the final row, and `FR-CTX-012`
//! rejected carrying the raw string alongside the structure, so a template has
//! no second view to fall back on. A misclassification is a defect to be
//! reported with the DDL that produced it.

use std::borrow::Cow;

use serde::{Deserialize, Serialize};

use super::{ToStatic, collapse_doubled_apostrophes};

/// The four characters the catalogue returns for a column defaulted to `NULL`.
///
/// `FR-CTX-037` observed them on 87 of the fixture's 301 columns, and
/// `FR-CTX-012` records what the second row of that table costs: a column
/// declaring `DEFAULT NULL` and a nullable column declaring no `DEFAULT` return
/// **identical bytes**, so the two are one state in the document because they
/// are one state in the catalogue.
const NULL: &str = "NULL";

/// The quote the catalogue delimits a string literal with.
const QUOTE: char = '\'';

/// The two characters a bit literal opens with, per the fifth row of
/// `FR-CTX-037`.
const BIT: &str = "b'";

/// The character a parenthesised expression opens with.
const OPEN: char = '(';

/// The character a parenthesised expression, and a function call, close with.
const CLOSE: char = ')';

/// The decimal point of the fourth row of `FR-CTX-037`.
const POINT: char = '.';

/// The `kind` discriminant of `FR-CTX-013`.
///
/// The three values are the whole enumeration. `FR-CTX-012` published four
/// forms of which two are indistinguishable in the catalogue, and the seventh
/// edition narrowed the contract to these three: a caller that branches on a
/// fourth is branching on a case no read can produce.
///
/// The type is `#[non_exhaustive]` because `FR-CTX-013` says so in as many
/// words — adding a value to it is not a breaking change, per `FR-OUT-014` —
/// and the attribute is what obliges a downstream match to say so too.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
#[non_exhaustive]
pub enum DefaultKind {
    /// The default is a value: a string, a number, or a bit literal.
    Literal,

    /// The default is evaluated by the server on each insert.
    Expression,

    /// The default is `NULL`, whether the column declared it or was nullable
    /// and declared nothing.
    Null,
}

impl DefaultKind {
    /// The value as `FR-CTX-013` spells it.
    ///
    /// The three spellings are contract surface: they are what a template
    /// branches on and what the document carries.
    #[must_use]
    pub const fn name(self) -> &'static str {
        match self {
            Self::Literal => "literal",
            Self::Expression => "expression",
            Self::Null => "null",
        }
    }
}

/// A column default, classified (`FR-CTX-011`, `FR-CTX-012`).
///
/// The variant carries the value where there is one, which is what makes
/// `FR-CTX-012` structural rather than conventional: a default of kind `null`
/// has no field a value could occupy, so the form `{"kind":"null","value":…}`
/// — which the requirement does not list — cannot be built.
///
/// The value borrows the catalogue string. It is copied in exactly one case,
/// and for exactly one reason: a string literal carrying a doubled apostrophe,
/// where one byte has to be dropped.
///
/// *The document shape is the private `Tagged` enumeration below, and the
/// projection onto it is what keeps the three forms of `FR-CTX-012` the only
/// three that can be written.* The requirement puts the discriminant **inside**
/// the object, beside a `value` the `null` form does not carry, which is
/// neither of the shapes a derive over these variants would produce. `Tagged`
/// states that once, as a tagged enumeration whose `null` variant has no field,
/// and the `into` and `from` attributes route both directions through it — so
/// the key order is still a property of a type and no writer restates it.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(into = "Tagged<'a>", from = "Tagged<'a>")]
#[non_exhaustive]
pub enum ColumnDefault<'a> {
    /// A value, carrying it as the document does — the text between the outer
    /// quotes for a string, and the catalogue's own characters for a number or
    /// a bit literal.
    Literal(Cow<'a, str>),

    /// An expression the server evaluates, carrying it without its outer
    /// parentheses where it had them.
    Expression(Cow<'a, str>),

    /// `NULL`, carrying nothing.
    Null,
}

impl<'a> ColumnDefault<'a> {
    /// Classifies the column-default catalogue field, per `FR-CTX-037`.
    ///
    /// `catalogue` is the field as read: [`None`] where the read returned SQL
    /// `NULL`, and [`Some`] otherwise. `FR-CAT-039` observed that the field is
    /// **never the empty string** — an empty default arrives as the two
    /// characters `''` — so an empty [`Some`] is a shape the catalogue was not
    /// observed to produce and falls to the final row like any other.
    ///
    /// [`None`] is returned for the first row of the table and for that row
    /// alone: a `NOT NULL` column that declares no `DEFAULT`, which is the one
    /// case with no default at all and the one the document writes as a bare
    /// `null`.
    ///
    /// The eight rows are tested in the order the requirement writes them, and
    /// the first that matches is the one that applies. Two of them are ordered
    /// against each other rather than merely listed, and reversing either
    /// changes the answer: the quoted row precedes the bit-literal row, so
    /// `b'101'` is not read as an unquoted string; and the parenthesised row
    /// precedes the ends-with-`)` row, so `(curdate() + interval 30 day)` is
    /// unwrapped rather than carried whole.
    #[must_use]
    pub fn classify(catalogue: Option<&'a str>) -> Option<Self> {
        // Row 1 — SQL `NULL`: the column has no default at all.
        let written = catalogue?;

        // Row 2 — exactly the four characters `NULL`.
        if written == NULL {
            return Some(Self::Null);
        }

        // Row 3 — begins and ends with a single quote. The two strips also
        // reject the one-character value `'`, which begins and ends with the
        // same byte and delimits nothing.
        if let Some(quoted) = written
            .strip_prefix(QUOTE)
            .and_then(|rest| rest.strip_suffix(QUOTE))
        {
            return Some(Self::Literal(collapse_doubled_apostrophes(quoted)));
        }

        // Row 4 — a decimal number, unquoted and unwrapped.
        if is_decimal(written) {
            return Some(Self::Literal(Cow::Borrowed(written)));
        }

        // Row 5 — a bit literal, carried with its `b` and its quotes.
        if written
            .strip_prefix(BIT)
            .is_some_and(|rest| rest.ends_with(QUOTE))
        {
            return Some(Self::Literal(Cow::Borrowed(written)));
        }

        // Row 6 — parenthesised: the expression is what the parentheses hold.
        if let Some(inner) = written
            .strip_prefix(OPEN)
            .and_then(|rest| rest.strip_suffix(CLOSE))
        {
            return Some(Self::Expression(Cow::Borrowed(inner)));
        }

        // Rows 7 and 8 — a call the server wrote unparenthesised, and anything
        // else. They are one arm because they emit one thing: the requirement
        // gives both the same `kind` and the same `value`, and no third row
        // follows that the merge could reorder.
        Some(Self::Expression(Cow::Borrowed(written)))
    }

    /// The `kind` this default carries, per `FR-CTX-013`.
    #[must_use]
    pub const fn kind(&self) -> DefaultKind {
        match self {
            Self::Literal(_) => DefaultKind::Literal,
            Self::Expression(_) => DefaultKind::Expression,
            Self::Null => DefaultKind::Null,
        }
    }

    /// The `value` this default carries, or [`None`] where its kind carries
    /// none.
    ///
    /// [`None`] is returned for [`DefaultKind::Null`] and for nothing else,
    /// which is the whole of `FR-CTX-012`'s distinction between the form
    /// `{"kind":"null"}` and the two that carry a value.
    #[must_use]
    pub fn value(&self) -> Option<&str> {
        match self {
            Self::Literal(value) | Self::Expression(value) => Some(value),
            Self::Null => None,
        }
    }
}

/// The document shape of a column default (`FR-CTX-011` … `FR-CTX-013`).
///
/// ```json
/// {"kind":"literal","value":"0"}
/// {"kind":"expression","value":"current_timestamp()"}
/// {"kind":"null"}
/// ```
///
/// The discriminant is a key of the object rather than a wrapper around it,
/// which is why this is a tagged enumeration and not the derive over
/// [`ColumnDefault`]'s own variants. [`Tagged::Null`] carries no field, so the
/// form `{"kind":"null","value":…}` — which `FR-CTX-012` does not list — has no
/// value that would produce it, on either side of the round trip.
///
/// It is private: the shape is contract surface, the type is not.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "lowercase")]
enum Tagged<'a> {
    /// A value, carried as `FR-CTX-037` classifies it.
    Literal {
        /// The literal.
        value: Cow<'a, str>,
    },

    /// An expression the server evaluates.
    Expression {
        /// The expression, without its outer parentheses where it had them.
        value: Cow<'a, str>,
    },

    /// `NULL`, carrying nothing.
    Null,
}

impl<'a> From<ColumnDefault<'a>> for Tagged<'a> {
    fn from(default: ColumnDefault<'a>) -> Self {
        match default {
            ColumnDefault::Literal(value) => Self::Literal { value },
            ColumnDefault::Expression(value) => Self::Expression { value },
            ColumnDefault::Null => Self::Null,
        }
    }
}

impl<'a> From<Tagged<'a>> for ColumnDefault<'a> {
    fn from(tagged: Tagged<'a>) -> Self {
        match tagged {
            Tagged::Literal { value } => Self::Literal(value),
            Tagged::Expression { value } => Self::Expression(value),
            Tagged::Null => Self::Null,
        }
    }
}

/// Whether a value is a decimal number, unquoted and unwrapped.
///
/// The fourth row of `FR-CTX-037` was observed over values written as digits,
/// with or without a fractional part — `0`, `0.0000`, `18.5`, `1.000000`. The
/// test is **exactly that shape and no wider**: a value the server wrote in any
/// other form falls to the final row and is classified as an expression, which
/// is the direction the requirement's own rationale calls the safer one.
fn is_decimal(written: &str) -> bool {
    let (whole, fraction) = match written.split_once(POINT) {
        Some((whole, fraction)) => (whole, Some(fraction)),
        None => (written, None),
    };

    digits(whole) && fraction.is_none_or(digits)
}

/// Whether a run is one or more ASCII digits and nothing else.
fn digits(run: &str) -> bool {
    !run.is_empty() && run.bytes().all(|byte| byte.is_ascii_digit())
}

/// A copy that borrows nothing, for the render context of `FR-RND-023`.
impl ToStatic for ColumnDefault<'_> {
    type Static = ColumnDefault<'static>;

    fn to_static(&self) -> Self::Static {
        match self {
            Self::Literal(value) => ColumnDefault::Literal(value.to_static()),
            Self::Expression(value) => ColumnDefault::Expression(value.to_static()),
            Self::Null => ColumnDefault::Null,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{ColumnDefault, DefaultKind};

    /// Classifies one catalogue value and compares the `kind` and the `value`
    /// it yields against the requirement's own two columns.
    ///
    /// The classification is held in a local for the length of the comparison
    /// because a `value` borrows the default that carries it, which a doubled
    /// apostrophe makes owned.
    #[track_caller]
    fn classified(catalogue: Option<&str>, expected: Option<(&str, Option<&str>)>, row: &str) {
        let default = ColumnDefault::classify(catalogue);
        let emitted = default
            .as_ref()
            .map(|default| (default.kind().name(), default.value()));

        assert_eq!(emitted, expected, "{row}");
    }

    #[test]
    fn fr_ctx_037_the_eight_rows_of_the_classifier_are_applied_in_the_order_written() {
        // FR-CTX-037, row by row, over the values the requirement quotes. The
        // final row quotes none, because it is the row for a shape that was not
        // observed; `current_timestamp` supplies one, matching none of the
        // seven positive shapes above it.
        classified(None, None, "row 1 — SQL NULL");
        classified(
            Some("NULL"),
            Some(("null", None)),
            "row 2 — the four characters NULL",
        );
        classified(
            Some("'EUR'"),
            Some(("literal", Some("EUR"))),
            "row 3 — a quoted string",
        );
        classified(
            Some("''"),
            Some(("literal", Some(""))),
            "row 3 — the empty string, which is a legal default",
        );
        classified(
            Some(r#"'8''6"'"#),
            Some(("literal", Some("8'6\""))),
            "row 3 — the doubled apostrophe collapses, and the double quote does not",
        );
        classified(
            Some("0"),
            Some(("literal", Some("0"))),
            "row 4 — a decimal number",
        );
        classified(
            Some("b'101'"),
            Some(("literal", Some("b'101'"))),
            "row 5 — a bit literal, carried unchanged",
        );
        classified(
            Some("(curdate() + interval 30 day)"),
            Some(("expression", Some("curdate() + interval 30 day"))),
            "row 6 — parenthesised, and unwrapped",
        );
        classified(
            Some("current_timestamp(3)"),
            Some(("expression", Some("current_timestamp(3)"))),
            "row 7 — a call the server wrote unparenthesised",
        );
        classified(
            Some("current_timestamp"),
            Some(("expression", Some("current_timestamp"))),
            "row 8 — anything else",
        );
    }

    #[test]
    fn fr_ctx_037_the_four_decimal_shapes_the_requirement_quotes_are_literals() {
        // FR-CTX-037, row 4: digits, with or without a fractional part, and the
        // value carried unchanged.
        for written in ["0", "0.0000", "18.5", "1.000000"] {
            classified(Some(written), Some(("literal", Some(written))), written);
        }
    }

    #[test]
    fn fr_ctx_037_a_value_that_is_not_digits_and_a_point_is_not_a_decimal_number() {
        // The row is read no wider than it was observed: anything else reaches
        // the final row and is an expression carrying the value unchanged.
        for written in ["1.2.3", "1.", ".5", "1e6", "-1", "current_user"] {
            classified(Some(written), Some(("expression", Some(written))), written);
        }
    }

    #[test]
    fn fr_ctx_037_the_bit_row_carries_its_prefix_and_its_quotes_into_the_value() {
        // FR-CTX-037 orders the rows, and a bit literal begins with `b` rather
        // than with a quote, so the third row and the fifth cannot collide.
        // What the fifth row does require is that the value is carried
        // **unchanged**, prefix and quotes included.
        classified(
            Some("b'0'"),
            Some(("literal", Some("b'0'"))),
            "the b and the quotes are part of the value",
        );
    }

    #[test]
    fn fr_ctx_037_a_lone_quote_delimits_nothing_and_falls_to_the_final_row() {
        // The third row needs an opening quote and a closing one. A single
        // character is one quote, not two, and the value is not a literal.
        classified(
            Some("'"),
            Some(("expression", Some("'"))),
            "one quote is not a delimited literal",
        );
    }

    #[test]
    fn fr_ctx_012_a_null_default_carries_no_value_and_the_other_two_kinds_carry_one() {
        // FR-CTX-012: `{"kind":"null"}` has no `value`, and the form
        // `{"kind":"null","value":…}` is not one the enumeration can express.
        assert_eq!(ColumnDefault::Null.value(), None);
        assert_eq!(ColumnDefault::Null.kind(), DefaultKind::Null);

        let literal = ColumnDefault::classify(Some("'EUR'")).expect("a quoted literal classifies");

        assert_eq!(literal.kind(), DefaultKind::Literal);
        assert_eq!(literal.value(), Some("EUR"));
    }

    #[test]
    fn fr_ctx_013_the_kind_takes_exactly_the_three_spellings_of_the_requirement() {
        // FR-CTX-013: `literal`, `expression`, `null`, and no fourth.
        assert_eq!(DefaultKind::Literal.name(), "literal");
        assert_eq!(DefaultKind::Expression.name(), "expression");
        assert_eq!(DefaultKind::Null.name(), "null");
    }
}
