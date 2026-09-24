//! The `--pattern` filter: MariaDB `LIKE` syntax, evaluated in memory
//! (`FR-SCH-011` … `FR-SCH-015`, `BR-SCH-001`).
//!
//! `FR-SCH-012` fixes four behaviours and no others:
//!
//! | Written | Matches |
//! |---|---|
//! | `%` | any sequence of characters, the empty one included |
//! | `_` | exactly one character |
//! | `\%` | a literal `%` |
//! | `\_` | a literal `_` |
//!
//! **Nothing is sent to the server** (`FR-SCH-013`). The pattern is compiled
//! here and applied to the names already read, which is the only way the same
//! pattern selects the same set with a database, without one, and against any
//! server: server-side `LIKE` would make the result depend on the collation of
//! the catalogue columns, which is invisible on the command line. It costs
//! nothing, because a catalogue read is a whole-list read either way. That is
//! also what makes `BR-SCH-001` hold across the three sources — a live read, a
//! cached read and a context file all reach this one matcher.
//!
//! **Case is folded over ASCII `A-Z` and `a-z` alone** (`FR-SCH-014`),
//! independently of the server, of the database collation and of the locale.
//! `order%` therefore also matches `Orders`, which the requirement records as
//! an accepted cost, and a character outside that range is compared as written
//! — `É` does not match `é`, because folding it would be a rule the requirement
//! does not state.
//!
//! **A backslash before anything else is a literal backslash.** `FR-SCH-012`
//! gives the escape exactly two meanings, so a third — a backslash that
//! disappears before an ordinary character, as a server-side `LIKE` would have
//! it — is behaviour this corpus does not fix and is not invented here. It is
//! reported rather than assumed correct.
//!
//! The matcher is the classic backtracking walk over the two sequences: a `%`
//! remembers where it was last tried and the walk returns to it one character
//! later when the rest of the pattern fails. It is linear in the common case
//! and allocates one character buffer per [`Pattern`] rather than one per name.

/// One element of a compiled pattern.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Token {
    /// `%`: any sequence of characters, the empty one included.
    Any,
    /// `_`: exactly one character.
    One,
    /// A character matched as written, already folded by [`fold`].
    Literal(char),
}

/// A compiled `--pattern` (`FR-SCH-012`).
///
/// The pattern is compiled once per invocation and applied to every name of the
/// listing, which is what keeps the escape rules in one place and out of the
/// matching loop.
#[derive(Debug)]
pub(super) struct Pattern {
    /// The compiled elements, in the order they were written.
    tokens: Vec<Token>,

    /// The characters of the name under test, folded.
    ///
    /// It is reused across names so that a listing of two hundred tables
    /// allocates once rather than two hundred times. `BR-PERF-004` makes the
    /// nearest-match path budgeted for the same reason, and the same buffer
    /// discipline is applied here.
    subject: Vec<char>,
}

impl Pattern {
    /// Compiles `written`.
    pub(super) fn compile(written: &str) -> Self {
        let mut tokens = Vec::with_capacity(written.len());
        let mut characters = written.chars();

        while let Some(character) = characters.next() {
            let token = match character {
                '%' => Token::Any,
                '_' => Token::One,
                // FR-SCH-012 gives the escape two meanings and no third, so a
                // backslash before anything else — including the end of the
                // pattern — is the character itself.
                '\\' => match characters.clone().next() {
                    Some(escaped @ ('%' | '_')) => {
                        characters.next();
                        Token::Literal(fold(escaped))
                    }
                    _ => Token::Literal('\\'),
                },
                other => Token::Literal(fold(other)),
            };

            tokens.push(token);
        }

        Self {
            tokens,
            subject: Vec::new(),
        }
    }

    /// Whether `name` matches.
    pub(super) fn matches(&mut self, name: &str) -> bool {
        self.subject.clear();
        self.subject.extend(name.chars().map(fold));

        matched(&self.tokens, &self.subject)
    }
}

/// `character` folded over ASCII `A-Z` and `a-z` alone (`FR-SCH-014`).
///
/// `char::to_ascii_lowercase` is exactly that rule: it maps `A`–`Z` and leaves
/// every other character, ASCII or not, as it was. Nothing here reaches
/// `to_lowercase`, which would fold `É` and every other range the requirement
/// keeps out.
const fn fold(character: char) -> char {
    character.to_ascii_lowercase()
}

/// Whether `subject` matches `tokens`.
///
/// The walk keeps the position of the last `%` and the position in `subject` it
/// was last satisfied at, so a failure further along resumes from there rather
/// than from the beginning. A pattern of `n` tokens against a name of `m`
/// characters therefore costs `O(n · m)` in the worst case and `O(n + m)` in
/// every case a catalogue produces.
fn matched(tokens: &[Token], subject: &[char]) -> bool {
    let (mut at, mut token) = (0_usize, 0_usize);
    let mut resume: Option<(usize, usize)> = None;

    loop {
        match tokens.get(token) {
            Some(Token::Any) => {
                // The `%` is tried against the empty sequence first, and
                // `resume` is what lengthens it one character at a time.
                resume = Some((token, at));
                token += 1;
            }
            Some(Token::One) if at < subject.len() => {
                at += 1;
                token += 1;
            }
            Some(Token::Literal(literal)) if subject.get(at) == Some(literal) => {
                at += 1;
                token += 1;
            }
            // The pattern is exhausted and so is the name: they match.
            None if at == subject.len() => return true,
            // Either the element did not match, or the pattern ran out with
            // characters left over. Both resume at the last `%`, if there was
            // one, with one more character consumed by it.
            _ => match resume {
                Some((star, consumed)) if consumed < subject.len() => {
                    at = consumed + 1;
                    token = star + 1;
                    resume = Some((star, at));
                }
                _ => return false,
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::Pattern;

    /// Whether `pattern` selects `name`.
    fn selects(pattern: &str, name: &str) -> bool {
        Pattern::compile(pattern).matches(name)
    }

    #[test]
    fn fr_sch_012_percent_matches_any_sequence_including_the_empty_one() {
        assert!(selects("%", ""));
        assert!(selects("%", "orders"));
        assert!(selects("order%", "orders"));
        assert!(selects("order%", "order"));
        assert!(selects("%items", "order_items"));
        assert!(selects("%der%", "orders"));
        assert!(selects("%%", "orders"));
        assert!(!selects("order%", "invoices"));
    }

    #[test]
    fn fr_sch_012_underscore_matches_exactly_one_character() {
        assert!(selects("order_", "orders"));
        assert!(!selects("order_", "order"));
        assert!(!selects("order_", "orderly"));
        assert!(selects("_____", "order"));
    }

    #[test]
    fn fr_sch_012_an_escaped_wildcard_matches_the_character_itself() {
        // The two escapes the requirement fixes, and the two they refuse.
        assert!(selects(r"100\%", "100%"));
        assert!(!selects(r"100\%", "100x"));
        assert!(selects(r"order\_items", "order_items"));
        assert!(!selects(r"order\_items", "orderxitems"));
    }

    #[test]
    fn a_backslash_before_anything_else_is_a_literal_backslash() {
        // FR-SCH-012 gives the escape two meanings and no third, so this is
        // the behaviour chosen where the corpus fixes none. It is reported
        // rather than assumed correct.
        assert!(selects(r"a\b", r"a\b"));
        assert!(!selects(r"a\b", "ab"));
        assert!(selects(r"trailing\", r"trailing\"));
    }

    #[test]
    fn fr_sch_014_case_is_folded_over_ascii_and_over_nothing_else() {
        // FR-SCH-014, with the accepted cost the requirement records: `order%`
        // also matches `Orders`.
        assert!(selects("order%", "Orders"));
        assert!(selects("ORDER%", "orders"));
        assert!(selects("OrDeRs", "oRdErS"));

        // And a character outside A-Z and a-z is compared as written: folding
        // it would be a rule the requirement does not state.
        assert!(!selects("\u{c9}poca", "\u{e9}poca"));
        assert!(selects("\u{e9}poca", "\u{e9}poca"));
    }

    #[test]
    fn fr_sch_013_a_pattern_is_a_property_of_the_names_and_of_nothing_else() {
        // FR-SCH-013: the same pattern selects the same set whatever produced
        // the names, because the only input is the name. The assertion that
        // nothing reaches the server is made where the statements are issued.
        let mut compiled = Pattern::compile("order%");
        let selected: Vec<&str> = ["orders", "order_items", "invoices", "Orders"]
            .into_iter()
            .filter(|name| compiled.matches(name))
            .collect();

        assert_eq!(selected, ["orders", "order_items", "Orders"]);
    }

    #[test]
    fn a_pattern_of_many_wildcards_terminates_on_a_name_that_does_not_match() {
        // The backtracking walk's worst case: every `%` is retried at every
        // position and the answer is still reached.
        assert!(!selects("%a%a%a%a%a%a%b", &"a".repeat(64)));
        assert!(selects("%a%a%a%a%a%a%b", &format!("{}b", "a".repeat(64))));
    }

    #[test]
    fn an_empty_pattern_selects_the_empty_name_alone() {
        assert!(selects("", ""));
        assert!(!selects("", "orders"));
    }
}
