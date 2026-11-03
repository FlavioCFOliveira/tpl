//! The timing and the event of a trigger, read from its stored header
//! (`FR-LITE-032`, `BR-LITE-002`).
//!
//! It is the one fact a `sqlite` read derives from stored text, and the
//! derivation reads **the tokens between the trigger's name and the keyword
//! `ON`, and nothing else**. SQLite stores the header as
//! `CREATE TRIGGER <name> [BEFORE | AFTER | INSTEAD OF] (DELETE | INSERT |
//! UPDATE [OF <columns>]) ON <table>`, with `CREATE TRIGGER` upper-cased and
//! `TEMP`, `IF NOT EXISTS` and the schema qualifier removed
//! (`https://sqlite.org/lang_createtrigger.html` and
//! `https://sqlite.org/schematab.html`, consulted 2026-11-03; `IF NOT EXISTS`
//! observed). The other keywords keep the case they were written in, so they
//! are compared ignoring ASCII case.
//!
//! The tokens are read under SQLite's four identifier quotings — `"…"`,
//! `` `…` ``, `[…]` and `'…'`, each doubling its closing quote to escape it,
//! except the bracket — and the header under SQLite's comment syntax, `--` to
//! the end of the line and `/* … */`.

/// What the header states.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct Header {
    /// `BEFORE` or `AFTER`, `BEFORE` where neither is written.
    pub(crate) timing: Timing,
    /// `DELETE`, `INSERT` or `UPDATE`, an `UPDATE OF` list read as `UPDATE`.
    pub(crate) event: Event,
}

/// When a trigger fires.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Timing {
    /// Before the statement.
    Before,
    /// After it.
    After,
}

/// The statement a trigger fires on.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Event {
    /// `DELETE`.
    Delete,
    /// `INSERT`.
    Insert,
    /// `UPDATE`, with or without an `OF` list.
    Update,
}

/// One token of the header.
#[derive(Debug, Clone, PartialEq, Eq)]
enum Token<'s> {
    /// A bare word.
    Word(&'s str),
    /// A quoted identifier, whatever its quoting.
    Quoted,
    /// A `.`, which joins a schema qualifier to a name.
    Dot,
    /// A `,`, between the columns of an `UPDATE OF` list.
    Comma,
    /// Anything else, which a header never holds before `ON`.
    Other,
}

/// Reads the timing and the event of the trigger whose stored text is `sql`.
///
/// [`None`] where the header does not yield one timing and one event — which
/// `FR-LITE-032` makes a `70`, since the library accepted the text.
pub(crate) fn header(sql: &str) -> Option<Header> {
    let mut tokens = Tokens { rest: sql };

    expect_word(&mut tokens, "CREATE")?;
    let mut next = tokens.next()?;
    // TEMP is removed from the stored text, and read past where it is not.
    if is_word(&next, "TEMP") || is_word(&next, "TEMPORARY") {
        next = tokens.next()?;
    }
    if !is_word(&next, "TRIGGER") {
        return None;
    }

    // The name, past an `IF NOT EXISTS` and a schema qualifier the stored text
    // does not normally carry.
    let mut name = tokens.next()?;
    if is_word(&name, "IF") {
        expect_word(&mut tokens, "NOT")?;
        expect_word(&mut tokens, "EXISTS")?;
        name = tokens.next()?;
    }
    if !matches!(name, Token::Word(_) | Token::Quoted) {
        return None;
    }
    let mut token = tokens.next()?;
    if token == Token::Dot {
        let qualified = tokens.next()?;
        if !matches!(qualified, Token::Word(_) | Token::Quoted) {
            return None;
        }
        token = tokens.next()?;
    }

    let mut timing = None;
    let mut event = None;

    loop {
        match token {
            ref word if is_word(word, "ON") => break,
            ref word if is_word(word, "BEFORE") => set(&mut timing, Timing::Before)?,
            ref word if is_word(word, "AFTER") => set(&mut timing, Timing::After)?,
            ref word if is_word(word, "INSTEAD") => {
                // INSTEAD OF fires on a view, and a trigger on a view is not
                // read (FR-LITE-021): on a table it yields no timing.
                return None;
            }
            ref word if is_word(word, "DELETE") => set(&mut event, Event::Delete)?,
            ref word if is_word(word, "INSERT") => set(&mut event, Event::Insert)?,
            ref word if is_word(word, "UPDATE") => {
                set(&mut event, Event::Update)?;
                let after = tokens.next()?;
                if is_word(&after, "OF") {
                    // The column list, to the keyword ON.
                    loop {
                        let column = tokens.next()?;
                        if !matches!(column, Token::Word(_) | Token::Quoted) {
                            return None;
                        }
                        let separator = tokens.next()?;
                        match separator {
                            Token::Comma => {}
                            ref word if is_word(word, "ON") => {
                                token = separator;
                                break;
                            }
                            _ => return None,
                        }
                    }
                    continue;
                }
                token = after;
                continue;
            }
            _ => return None,
        }
        token = tokens.next()?;
    }

    Some(Header {
        timing: timing.unwrap_or(Timing::Before),
        event: event?,
    })
}

/// Records `value` in `slot`, or [`None`] where the slot is already filled.
fn set<T>(slot: &mut Option<T>, value: T) -> Option<()> {
    if slot.is_some() {
        return None;
    }
    *slot = Some(value);
    Some(())
}

/// Whether `token` is the keyword `keyword`, ignoring ASCII case.
fn is_word(token: &Token<'_>, keyword: &str) -> bool {
    matches!(token, Token::Word(word) if word.eq_ignore_ascii_case(keyword))
}

/// The next token, where it is the keyword `keyword`.
fn expect_word(tokens: &mut Tokens<'_>, keyword: &str) -> Option<()> {
    is_word(&tokens.next()?, keyword).then_some(())
}

/// The tokens of a header, comments and whitespace skipped.
struct Tokens<'s> {
    /// What is left to read.
    rest: &'s str,
}

impl<'s> Tokens<'s> {
    /// The next token, or [`None`] at the end or inside an unclosed quote or
    /// comment.
    fn next(&mut self) -> Option<Token<'s>> {
        self.skip()?;

        let mut characters = self.rest.char_indices();
        let (_, first) = characters.next()?;

        match first {
            '"' | '`' | '\'' => {
                self.rest = &self.rest[1..];
                self.quoted(first)?;
                Some(Token::Quoted)
            }
            '[' => {
                let close = self.rest.find(']')?;
                self.rest = &self.rest[close + 1..];
                Some(Token::Quoted)
            }
            '.' => {
                self.rest = &self.rest[1..];
                Some(Token::Dot)
            }
            ',' => {
                self.rest = &self.rest[1..];
                Some(Token::Comma)
            }
            character
                if character == '_' || character.is_alphanumeric() || !character.is_ascii() =>
            {
                let end = self
                    .rest
                    .char_indices()
                    .find(|(_, character)| {
                        !(*character == '_'
                            || *character == '$'
                            || character.is_alphanumeric()
                            || !character.is_ascii())
                    })
                    .map_or(self.rest.len(), |(at, _)| at);
                let word = &self.rest[..end];
                self.rest = &self.rest[end..];
                Some(Token::Word(word))
            }
            other => {
                self.rest = &self.rest[other.len_utf8()..];
                Some(Token::Other)
            }
        }
    }

    /// Skips whitespace and comments; [`None`] inside an unclosed comment.
    fn skip(&mut self) -> Option<()> {
        loop {
            self.rest = self.rest.trim_start();
            if let Some(after) = self.rest.strip_prefix("--") {
                self.rest = after.find('\n').map_or("", |end| &after[end + 1..]);
            } else if let Some(after) = self.rest.strip_prefix("/*") {
                let end = after.find("*/")?;
                self.rest = &after[end + 2..];
            } else {
                return Some(());
            }
        }
    }

    /// Reads past a quoted identifier opened by `quote`, a doubled quote
    /// escaping one.
    fn quoted(&mut self, quote: char) -> Option<()> {
        let mut characters = self.rest.char_indices().peekable();
        while let Some((at, character)) = characters.next() {
            if character == quote {
                if characters.peek().map(|(_, next)| *next) == Some(quote) {
                    characters.next();
                    continue;
                }
                self.rest = &self.rest[at + quote.len_utf8()..];
                return Some(());
            }
        }
        None
    }
}

#[cfg(test)]
mod tests {
    use super::{Event, Header, Timing, header};

    fn read(sql: &str) -> Option<Header> {
        header(sql)
    }

    #[test]
    fn fr_lite_032_every_timing_on_every_event_is_read_from_the_header() {
        for (timing, written) in [(Timing::Before, "BEFORE"), (Timing::After, "AFTER")] {
            for (event, verb) in [
                (Event::Insert, "INSERT"),
                (Event::Update, "UPDATE"),
                (Event::Delete, "DELETE"),
            ] {
                let sql = format!(
                    "CREATE TRIGGER trg_leg {written} {verb} ON voyage_leg BEGIN SELECT 1; END"
                );
                assert_eq!(read(&sql), Some(Header { timing, event }), "{sql}");
            }
        }
    }

    #[test]
    fn fr_lite_032_a_header_with_no_timing_is_before() {
        assert_eq!(
            read("CREATE TRIGGER t_default_timing INSERT ON a1 BEGIN SELECT 1; END"),
            Some(Header {
                timing: Timing::Before,
                event: Event::Insert
            })
        );
    }

    #[test]
    fn fr_lite_032_an_update_of_list_is_read_as_update_and_its_columns_are_skipped() {
        assert_eq!(
            read(
                "CREATE TRIGGER \"t q\" AFTER UPDATE OF eta, \"on\", [after] ON a1 BEGIN SELECT 1; END"
            ),
            Some(Header {
                timing: Timing::After,
                event: Event::Update
            })
        );
    }

    #[test]
    fn fr_lite_032_the_four_quotings_comments_and_lower_case_keywords_are_read() {
        for sql in [
            "CREATE TRIGGER \"before\" after delete ON t BEGIN SELECT 1; END",
            "CREATE TRIGGER `on ``x``` After Delete ON t BEGIN SELECT 1; END",
            "CREATE TRIGGER [insert] /* the timing */ AFTER -- the event\n DELETE ON t BEGIN END",
            "CREATE TRIGGER 'q''uoted' AFTER DELETE ON t WHEN old.id = 1 BEGIN SELECT 1; END",
            "CREATE TRIGGER IF NOT EXISTS main.trg AFTER DELETE ON t BEGIN SELECT 1; END",
        ] {
            assert_eq!(
                read(sql),
                Some(Header {
                    timing: Timing::After,
                    event: Event::Delete
                }),
                "{sql}"
            );
        }
    }

    #[test]
    fn fr_lite_032_a_header_without_one_timing_and_one_event_yields_none() {
        for sql in [
            "CREATE TRIGGER t ON a1 BEGIN SELECT 1; END",
            "CREATE TRIGGER t BEFORE AFTER INSERT ON a1 BEGIN END",
            "CREATE TRIGGER t INSERT DELETE ON a1 BEGIN END",
            "CREATE TRIGGER t INSTEAD OF INSERT ON v BEGIN END",
            "CREATE TRIGGER \"unclosed AFTER INSERT ON a1",
            "CREATE VIEW v AS SELECT 1",
            "",
        ] {
            assert_eq!(read(sql), None, "{sql}");
        }
    }
}
