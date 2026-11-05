//! The two readings of a `mysql` read that rewrite a catalogue text: the
//! attribute field of a column (`FR-MYSQL-030`), and the one layer of
//! backslash escaping `FR-MYSQL-046` removes from an expression.
//!
//! Both are free functions over a string, so each rule is exercised without a
//! server, on the texts the specification records as observed.

use std::borrow::Cow;

use crate::model::column::GeneratedStorage;

/// The escape character of `FR-MYSQL-046`.
const BACKSLASH: char = '\\';

/// The quote it escapes.
const QUOTE: char = '\'';

/// Removes exactly one layer of backslash escaping (`FR-MYSQL-046`).
///
/// Read from the first character to the last: `\\` is written as `\` and `\'`
/// as `'`, each consuming the pair; any other character, a backslash followed
/// by any other character, and a backslash that ends the text are written
/// unchanged. Every introducer is kept, and no other character changes.
///
/// A text holding no backslash is returned borrowed, which is every expression
/// the specification observed that holds no string literal.
pub(super) fn unescape(text: &str) -> Cow<'_, str> {
    if !text.contains(BACKSLASH) {
        return Cow::Borrowed(text);
    }

    let mut written = String::with_capacity(text.len());
    let mut characters = text.chars().peekable();

    while let Some(character) = characters.next() {
        if character == BACKSLASH
            && let Some(&next) = characters.peek()
            && (next == BACKSLASH || next == QUOTE)
        {
            written.push(next);
            characters.next();
            continue;
        }

        written.push(character);
    }

    Cow::Owned(written)
}

/// The text between the outer parentheses where `text` begins with `(` and
/// ends with `)`, and `text` otherwise (`FR-MYSQL-029`, third row).
///
/// The rule is the row's own: the first and the last character are the outer
/// pair. Every text the specification observed in that shape has matching
/// outer parentheses.
pub(super) fn without_outer_parentheses(text: &str) -> &str {
    text.strip_prefix('(')
        .and_then(|inner| inner.strip_suffix(')'))
        .unwrap_or(text)
}

/// What the attribute field of a column states (`FR-MYSQL-030`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub(super) struct Extra<'a> {
    /// `auto_increment`.
    pub(super) auto_increment: bool,
    /// `INVISIBLE`.
    pub(super) invisible: bool,
    /// `VIRTUAL GENERATED` or `STORED GENERATED`.
    pub(super) storage: Option<GeneratedStorage>,
    /// `on update` and the one word after it, as the field writes the phrase.
    pub(super) on_update: Option<&'a str>,
    /// `DEFAULT_GENERATED`: the default is an expression.
    pub(super) default_generated: bool,
}

/// Whether two tokens are one phrase, ignoring ASCII letter case.
fn phrase(first: &str, second: &str, expected: (&str, &str)) -> bool {
    first.eq_ignore_ascii_case(expected.0) && second.eq_ignore_ascii_case(expected.1)
}

impl<'a> Extra<'a> {
    /// The attribute field read as a sequence of the phrases of
    /// `FR-MYSQL-030`, separated by single spaces, in any order, each
    /// recognised ignoring ASCII letter case.
    ///
    /// Text no phrase matches is skipped, and its neighbours are still read:
    /// the requirement takes the facts of the phrases it recognises and does
    /// not fail the read, on the ground `FR-CAT-055` gives. `MASKING POLICY` is
    /// recognised and carries no fact the model has a property for.
    ///
    /// No phrase is longer than three tokens, so the field is read through a
    /// window of three over the split, each token paired with the byte it
    /// begins at, and a phrase advances the window by its length.
    pub(super) fn parse(field: &'a str) -> Self {
        // PERF: a window over the split rather than a vector of every token,
        // which allocated once per column (rmp #351).
        let mut start = 0;
        let mut tokens = field.split(' ').map(|token| {
            let begins = start;
            start += token.len() + 1;
            (begins, token)
        });
        let mut window = [tokens.next(), tokens.next(), tokens.next()];
        let mut extra = Self::default();

        while let [Some((begins, token)), next, third] = window {
            let second = next.map_or("", |(_, token)| token);
            let mut length = 1;

            if token.eq_ignore_ascii_case("auto_increment") {
                extra.auto_increment = true;
            } else if token.eq_ignore_ascii_case("INVISIBLE") {
                extra.invisible = true;
            } else if token.eq_ignore_ascii_case("DEFAULT_GENERATED") {
                extra.default_generated = true;
            } else if phrase(token, second, ("VIRTUAL", "GENERATED")) {
                extra.storage = Some(GeneratedStorage::Virtual);
                length = 2;
            } else if phrase(token, second, ("STORED", "GENERATED")) {
                extra.storage = Some(GeneratedStorage::Stored);
                length = 2;
            } else if phrase(token, second, ("MASKING", "POLICY")) {
                length = 2;
            } else if phrase(token, second, ("on", "update"))
                && let Some((word_begins, word)) = third
                && !word.is_empty()
            {
                extra.on_update = field.get(begins..word_begins + word.len());
                length = 3;
            }

            for _ in 0..length {
                window = [window[1], window[2], tokens.next()];
            }
        }

        extra
    }
}

#[cfg(test)]
mod tests {
    use super::{Extra, unescape, without_outer_parentheses};
    use crate::model::column::GeneratedStorage;
    use std::borrow::Cow;

    #[test]
    fn fr_mysql_046_one_layer_is_removed_from_the_observed_texts() {
        // FR-MYSQL-046's table, byte for byte: as the server writes it, and as
        // the model carries it.
        for (written, carried) in [
            (r"_utf8mb4\'it\\\'s\'", r"_utf8mb4'it\'s'"),
            (r"_utf8mb4\'back\\\\slash\'", r"_utf8mb4'back\\slash'"),
            (r"_utf8mb4\'line1\\nline2\'", r"_utf8mb4'line1\nline2'"),
            (
                r"concat(_utf8mb4\'it\\\'s \',`l_quote`)",
                r"concat(_utf8mb4'it\'s ',`l_quote`)",
            ),
            (
                r"(`l_quote` <> _utf8mb4\'o\\\'clock\')",
                r"(`l_quote` <> _utf8mb4'o\'clock')",
            ),
            (
                r"(`imdg_class` <> _utf8mb4\'Not regulated\')",
                r"(`imdg_class` <> _utf8mb4'Not regulated')",
            ),
        ] {
            assert_eq!(unescape(written), carried, "{written}");
        }
    }

    #[test]
    fn fr_mysql_046_any_other_backslash_is_kept_and_a_text_without_one_is_borrowed() {
        assert_eq!(unescape(r"a\nb"), r"a\nb");
        assert_eq!(unescape(r"ends\"), r"ends\");
        assert_eq!(unescape(r"\\\'"), r"\'");
        assert!(matches!(
            unescape("((`base_rate` + `bunker_surcharge`))"),
            Cow::Borrowed(_)
        ));
    }

    #[test]
    fn fr_mysql_029_the_outer_pair_is_removed_where_the_text_opens_and_closes_with_one() {
        assert_eq!(
            without_outer_parentheses("(curdate() + interval 30 day)"),
            "curdate() + interval 30 day"
        );
        assert_eq!(
            without_outer_parentheses("((1 + 2) * (3 + 4))"),
            "(1 + 2) * (3 + 4)"
        );
        assert_eq!(without_outer_parentheses("now()"), "now()");
        // The seventy-sixth edition: the first and the last character, removed
        // once, whether or not they are a matching pair.
        assert_eq!(without_outer_parentheses("(a) + (b)"), "a) + (b");
        assert_eq!(without_outer_parentheses("((x))"), "(x)");
        assert_eq!(
            without_outer_parentheses("CURRENT_TIMESTAMP"),
            "CURRENT_TIMESTAMP"
        );
    }

    #[test]
    fn fr_mysql_030_the_eleven_observed_values_yield_their_facts() {
        // FR-MYSQL-030's table of the eleven values the fixture's columns took.
        let none = Extra::parse("");
        assert_eq!(none, Extra::default());

        assert!(Extra::parse("auto_increment").auto_increment);

        let gipk = Extra::parse("auto_increment INVISIBLE");
        assert!(gipk.auto_increment && gipk.invisible);

        assert!(Extra::parse("DEFAULT_GENERATED").default_generated);

        let hidden = Extra::parse("DEFAULT_GENERATED INVISIBLE");
        assert!(hidden.default_generated && hidden.invisible);

        let updated = Extra::parse("DEFAULT_GENERATED on update CURRENT_TIMESTAMP");
        assert!(updated.default_generated);
        assert_eq!(updated.on_update, Some("on update CURRENT_TIMESTAMP"));

        let fractional = Extra::parse("DEFAULT_GENERATED on update CURRENT_TIMESTAMP(3)");
        assert_eq!(fractional.on_update, Some("on update CURRENT_TIMESTAMP(3)"));

        assert!(Extra::parse("INVISIBLE").invisible);
        assert_eq!(
            Extra::parse("STORED GENERATED").storage,
            Some(GeneratedStorage::Stored)
        );
        assert_eq!(
            Extra::parse("VIRTUAL GENERATED").storage,
            Some(GeneratedStorage::Virtual)
        );

        let invisible_virtual = Extra::parse("VIRTUAL GENERATED INVISIBLE");
        assert_eq!(invisible_virtual.storage, Some(GeneratedStorage::Virtual));
        assert!(invisible_virtual.invisible);
        assert!(!invisible_virtual.auto_increment);
    }

    #[test]
    fn fr_mysql_030_phrases_read_in_any_order_ignoring_case_and_unknown_text_is_skipped() {
        let reordered = Extra::parse("invisible stored generated AUTO_INCREMENT");
        assert!(reordered.invisible && reordered.auto_increment);
        assert_eq!(reordered.storage, Some(GeneratedStorage::Stored));

        // FR-MYSQL-030 as amended: the phrase as EXTRA writes it, letter case
        // included, from `on update` to the end of the word after it.
        assert_eq!(
            Extra::parse("ON UPDATE current_timestamp(6) INVISIBLE").on_update,
            Some("ON UPDATE current_timestamp(6)")
        );

        let masked = Extra::parse("MASKING POLICY DEFAULT_GENERATED something-new INVISIBLE");
        assert!(masked.default_generated && masked.invisible);
        assert_eq!(masked.storage, None);
        assert_eq!(masked.on_update, None);
    }

    /// The reading [`Extra::parse`] replaced in rmp `#351`, kept as the oracle
    /// of its replacement: the field split into a vector of tokens, each with
    /// the byte it begins at, read by index.
    fn parsed_through_a_vector(field: &str) -> Extra<'_> {
        let mut tokens: Vec<(usize, &str)> = Vec::new();
        let mut start = 0;

        for token in field.split(' ') {
            tokens.push((start, token));
            start += token.len() + 1;
        }

        let mut extra = Extra::default();
        let mut at = 0;

        while let Some(&(begins, token)) = tokens.get(at) {
            let second = tokens.get(at + 1).map_or("", |&(_, token)| token);
            let third = tokens.get(at + 2);

            if token.eq_ignore_ascii_case("auto_increment") {
                extra.auto_increment = true;
            } else if token.eq_ignore_ascii_case("INVISIBLE") {
                extra.invisible = true;
            } else if token.eq_ignore_ascii_case("DEFAULT_GENERATED") {
                extra.default_generated = true;
            } else if super::phrase(token, second, ("VIRTUAL", "GENERATED")) {
                extra.storage = Some(GeneratedStorage::Virtual);
                at += 1;
            } else if super::phrase(token, second, ("STORED", "GENERATED")) {
                extra.storage = Some(GeneratedStorage::Stored);
                at += 1;
            } else if super::phrase(token, second, ("MASKING", "POLICY")) {
                at += 1;
            } else if super::phrase(token, second, ("on", "update"))
                && let Some(&(word_begins, word)) = third
                && !word.is_empty()
            {
                extra.on_update = field.get(begins..word_begins + word.len());
                at += 2;
            }

            at += 1;
        }

        extra
    }

    #[test]
    fn fr_mysql_030_the_window_reads_every_field_as_the_vector_of_tokens_did() {
        // rmp #351: every field of up to four tokens drawn from the words of
        // FR-MYSQL-030's phrases, in both cases, with the empty token a
        // leading, trailing or doubled space makes and one word no phrase
        // holds, is read alike by both: the 41 371 fields of 0 to 4 words.
        const WORDS: [&str; 14] = [
            "auto_increment",
            "INVISIBLE",
            "DEFAULT_GENERATED",
            "VIRTUAL",
            "STORED",
            "GENERATED",
            "generated",
            "MASKING",
            "POLICY",
            "on",
            "UPDATE",
            "CURRENT_TIMESTAMP(3)",
            "",
            "x",
        ];

        let mut fields = vec![String::new()];
        let mut shorter: Vec<Vec<&str>> = vec![Vec::new()];
        for _ in 0..4 {
            let longer: Vec<Vec<&str>> = shorter
                .iter()
                .flat_map(|words| {
                    WORDS.iter().map(move |word| {
                        let mut longer = words.clone();
                        longer.push(word);
                        longer
                    })
                })
                .collect();
            fields.extend(longer.iter().map(|words| words.join(" ")));
            shorter = longer;
        }
        assert_eq!(fields.len(), 1 + 14 + 196 + 2_744 + 38_416);

        let mut facts = 0;
        for field in &fields {
            let read = Extra::parse(field);
            assert_eq!(read, parsed_through_a_vector(field), "{field:?}");
            facts += usize::from(read != Extra::default());
        }

        // The corpus reaches every fact, so the comparison is not vacuous.
        assert!(
            fields
                .iter()
                .any(|field| Extra::parse(field).on_update.is_some())
        );
        assert!(facts > fields.len() / 2, "{facts} of {}", fields.len());
    }
}
