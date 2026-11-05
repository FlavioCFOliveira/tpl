//! The escaping of `FR-OUT-018`, applied to every cell of a `text` layout.
//!
//! `FR-OUT-018` escapes the C0 control characters, the C1 control characters
//! and the bidirectional formatting characters in the `text` output of the
//! read commands, for a `mariadb` and a `sqlite` entry alike, and excepts
//! **tab**; `FR-OUT-019` extends the rule to every value interpolated into
//! that output whatever its source — the catalogue, a `--context` document, or
//! the argument vector. Two outputs the same requirement exempts are emitted
//! byte for byte and never reach this module: the result of `tpl render`, and
//! the template source printed by `tpl template show`. Neither is a value
//! interpolated into a listing, and neither is written through [`super::text`].
//!
//! | Character | Emitted as | Width |
//! |---|---|---|
//! | `\n`, `\r` | `\n`, `\r` | 2 |
//! | Tab | Tab, unchanged | 1 |
//! | The other twenty-nine C0 controls | `\u{XX}`, two lowercase hexadecimal digits | 6 |
//! | A C1 control, U+0080 to U+009F | `\u{XX}`, the form of a C0 control: `\u{9b}` | 6 |
//! | A bidirectional formatting character | `\u{…}`, the code point in lowercase hexadecimal: `\u{61c}`, `\u{202e}` | 7 or 8 |
//! | Everything else, `U+007F` included | Unchanged | 1 |
//!
//! The bidirectional formatting characters are the twelve code points of
//! Unicode's `Bidi_Control` property (the glossary of the specification);
//! [`escaped_past_c0`] decides membership of the two sets past C0, and the
//! diagnostic renderer and the JSON formatter call it too, so the three paths
//! differ in the form they write and never in what they escape (`FR-ERR-024`,
//! `FR-OUT-018`, as amended in the sixty-ninth edition).
//!
//! **This is not `diagnostics::escape`, and the two do not share an
//! implementation.** They disagree on tab — escaped there, excepted here — and
//! `OD-05` keeps `output/` and `diagnostics/` apart on precisely that ground:
//! "One module holding two opposite escaping rules over two opposite promises
//! is the shape in which the wrong one gets applied." A shared routine taking a
//! flag would rebuild that shape in a third place, with the flag as the thing a
//! caller gets wrong and no compiler able to catch it. The two differ in more
//! than the flag would carry, besides: that one escapes a **composed line** as
//! a whole into a `String`, per `OD-06`, and this one escapes a **single cell**
//! into the buffered writer and must also measure it without writing.
//!
//! [`width`] and [`write()`] are therefore one rule stated twice, and a test
//! holds them to each other: for every value, the bytes [`write()`] emits carry
//! exactly [`width`] characters.

use std::io::{self, Write};

/// The lowercase hexadecimal digits, indexed by value.
const HEX: &[u8; 16] = b"0123456789abcdef";

/// The exclusive upper bound of the C0 control range, `U+0000`–`U+001F`.
const C0_END: u8 = 0x20;

/// Tab, the one C0 control `FR-OUT-018` excepts in `text` output.
const TAB: u8 = b'\t';

/// The character width of `\n` and `\r`.
const NAMED: usize = 2;

/// The character width of `\u{` and `}` around the digits of a code point.
const BRACES: usize = 4;

/// Whether `character` is a C1 control character — U+0080 to U+009F — or one
/// of the twelve bidirectional formatting characters of Unicode's
/// `Bidi_Control` property: U+061C, U+200E, U+200F, U+202A to U+202E and
/// U+2066 to U+2069 (`FR-ERR-024`, `FR-OUT-018`, as amended in the sixty-ninth
/// edition; the glossary's *bidirectional formatting character*).
///
/// It is the one predicate that decides what is escaped past the C0 range:
/// this module, the diagnostic renderer and the JSON formatter each call it,
/// and differ only in the form they write.
#[must_use]
pub(crate) const fn escaped_past_c0(character: char) -> bool {
    matches!(
        character,
        '\u{80}'..='\u{9f}'
            | '\u{61c}'
            | '\u{200e}'
            | '\u{200f}'
            | '\u{202a}'..='\u{202e}'
            | '\u{2066}'..='\u{2069}'
    )
}

/// Whether `value` may hold a character [`escaped_past_c0`] admits.
///
/// Every one of them is written in UTF-8 with the lead byte `0xC2` (the C1
/// range), `0xD8` (U+061C) or `0xE2` (the other eleven), so a value holding
/// none of those bytes holds none of them, and the scan decodes nothing. A
/// value that holds one of those bytes is decoded to find out.
#[must_use]
pub(crate) fn may_hold_past_c0(value: &str) -> bool {
    value.bytes().any(|byte| matches!(byte, 0xC2 | 0xD8 | 0xE2))
}

/// The number of lowercase hexadecimal digits a code point is written with in
/// the form `\u{…}`: as many as it needs, and at least two.
const fn digits(code: u32) -> usize {
    let mut needed = 1;
    let mut rest = code >> 4;
    while rest > 0 {
        needed += 1;
        rest >>= 4;
    }
    if needed < 2 { 2 } else { needed }
}

/// Whether `value` carries a character this rule rewrites.
///
/// A C0 control is a byte below `0x20` in UTF-8 and every continuation byte is
/// at or above `0x80`, so that half of the scan is exact and decodes nothing.
/// Tab is not one of them: it is excepted, and a value whose only control is a
/// tab takes the unescaped path and reaches the writer in a single call. The
/// characters past C0 are found by [`may_hold_past_c0`] and confirmed by
/// decoding.
fn rewrites(value: &str) -> bool {
    value.bytes().any(|byte| byte < C0_END && byte != TAB)
        || (may_hold_past_c0(value) && value.chars().any(escaped_past_c0))
}

/// The width of `character` once escaped, in characters.
const fn escaped(character: char) -> usize {
    match character {
        '\n' | '\r' => NAMED,
        // Excepted by `FR-OUT-018`, and therefore one character wide — which
        // is what it is, and not what it displays as. [`width`] says what that
        // costs.
        '\t' => 1,
        '\u{00}'..='\u{1f}' => BRACES + digits(character as u32),
        past if escaped_past_c0(past) => BRACES + digits(past as u32),
        _ => 1,
    }
}

/// The number of characters `value` occupies once escaped.
///
/// This is the number [`super::text`] lays a column out with, and it is a count
/// of **characters**, not of bytes and not of terminal cells. The three differ,
/// and each difference is a deliberate accepted cost:
///
/// - A multi-byte character counts once, which is right: `ç` is two bytes and
///   one column of the reader's terminal.
/// - A character the terminal draws two cells wide — most CJK, and most emoji —
///   counts once and displays as two, so a row carrying one is drawn one cell
///   wider than the layout allotted it and the columns to its right step right.
///   A combining mark is the same error in the other direction. Measuring this
///   correctly needs a Unicode width table, which is a dependency, and the
///   project's dependency budget does not admit one for a `text` output that
///   `FR-OUT-004` makes explicitly not a contract.
/// - A tab counts once and displays as a jump to the next tab stop, so a value
///   carrying one misaligns every column to its right. `FR-OUT-018` excepts tab
///   deliberately and `FR-OUT-019` sends every catalogue value through this
///   rule, so the two compose into exactly this: a tab inside a comment is
///   carried to the reader intact, and the row it sits in is no longer aligned.
///   Escaping it instead is not available — the requirement excepts it — and
///   rewriting it to spaces would change a value the same requirement carries.
#[must_use]
pub(super) fn width(value: &str) -> usize {
    if rewrites(value) {
        value.chars().map(escaped).sum()
    } else {
        value.chars().count()
    }
}

/// Writes `value` to `writer` with the characters of `FR-OUT-018` escaped.
///
/// A value carrying nothing to rewrite reaches `writer` in one call, which is
/// the ordinary case for a catalogue name and keeps the aggregation the project
/// requires of every write.
///
/// # Errors
///
/// Returns what `writer` returned.
pub(super) fn write<W: Write>(writer: &mut W, value: &str) -> io::Result<()> {
    if !rewrites(value) {
        return writer.write_all(value.as_bytes());
    }

    let mut utf8 = [0_u8; 4];
    for character in value.chars() {
        match character {
            '\n' => writer.write_all(b"\\n")?,
            '\r' => writer.write_all(b"\\r")?,
            '\t' => writer.write_all(b"\t")?,
            '\u{00}'..='\u{1f}' => unicode(writer, u32::from(character))?,
            past if escaped_past_c0(past) => unicode(writer, u32::from(past))?,
            plain => writer.write_all(plain.encode_utf8(&mut utf8).as_bytes())?,
        }
    }

    Ok(())
}

/// Writes `\u{…}` for `code`, in lowercase hexadecimal of at least two digits,
/// in one call.
fn unicode<W: Write>(writer: &mut W, code: u32) -> io::Result<()> {
    // `\u{` + at most six digits + `}`: every code point fits.
    let mut form = [0_u8; 10];
    let count = digits(code);
    form[..3].copy_from_slice(b"\\u{");
    for at in 0..count {
        let shift = 4 * (count - 1 - at);
        form[3 + at] = HEX[((code >> shift) & 0xf) as usize];
    }
    form[3 + count] = b'}';

    writer.write_all(&form[..4 + count])
}

#[cfg(test)]
mod tests {
    use super::{width, write};

    /// The escaped form of `value`, as the reader receives it.
    fn escaped(value: &str) -> String {
        let mut out = Vec::new();
        write(&mut out, value).expect("a buffer accepts every write");

        String::from_utf8(out).expect("the rule emits UTF-8")
    }

    #[test]
    fn fr_out_018_a_value_with_no_control_is_unchanged() {
        assert_eq!(escaped("ordens_de_compra"), "ordens_de_compra");
    }

    #[test]
    fn fr_out_018_the_two_named_controls_take_their_readable_escape() {
        // FR-OUT-018: a newline inside a column comment cannot be allowed to
        // add a line to a listing.
        assert_eq!(escaped("one\ntwo\rthree"), "one\\ntwo\\rthree");
    }

    #[test]
    fn fr_out_018_tab_is_excepted_and_reaches_the_reader_intact() {
        // FR-OUT-018 excepts tab in `text` output alone, which is the one
        // point on which this rule and FR-ERR-024 disagree.
        assert_eq!(escaped("one\ttwo"), "one\ttwo");
        assert_eq!(width("one\ttwo"), 7);
    }

    #[test]
    fn fr_out_018_every_other_c0_control_takes_the_unicode_escape() {
        // The range the requirement names is U+0000 through U+001F. Three of
        // the thirty-two are spoken for; the other twenty-nine are not.
        let mut unicode_escaped = 0_u32;

        for code in 0x00_u32..0x20 {
            let control = char::from_u32(code).expect("every C0 code point is a character");
            if matches!(control, '\n' | '\r' | '\t') {
                continue;
            }

            assert_eq!(escaped(&control.to_string()), format!("\\u{{{code:02x}}}"));
            unicode_escaped += 1;
        }

        assert_eq!(unicode_escaped, 29);
    }

    #[test]
    fn fr_out_018_a_c1_control_and_a_bidirectional_formatting_character_take_the_c0_form() {
        // FR-OUT-018 as amended in the sixty-ninth edition: the C1 range and
        // the twelve Bidi_Control code points are escaped in the form a C0
        // control is — `\u{`, the code point in lowercase hexadecimal of at
        // least two digits, `}` — for a `mariadb` and a `sqlite` entry alike.
        for code in 0x80_u32..=0x9f {
            let control = char::from_u32(code).expect("every C1 code point is a character");
            assert_eq!(escaped(&control.to_string()), format!("\\u{{{code:02x}}}"));
        }
        for (character, form) in [
            ('\u{61c}', "\\u{61c}"),
            ('\u{200e}', "\\u{200e}"),
            ('\u{200f}', "\\u{200f}"),
            ('\u{202a}', "\\u{202a}"),
            ('\u{202b}', "\\u{202b}"),
            ('\u{202c}', "\\u{202c}"),
            ('\u{202d}', "\\u{202d}"),
            ('\u{202e}', "\\u{202e}"),
            ('\u{2066}', "\\u{2066}"),
            ('\u{2067}', "\\u{2067}"),
            ('\u{2068}', "\\u{2068}"),
            ('\u{2069}', "\\u{2069}"),
        ] {
            assert_eq!(escaped(&character.to_string()), form);
        }
        assert_eq!(
            escaped("na\u{9b}31mme\u{202e}gpj.exe"),
            "na\\u{9b}31mme\\u{202e}gpj.exe"
        );
    }

    #[test]
    fn fr_out_018_a_character_outside_the_escaped_set_is_left_alone() {
        // U+007F is outside the C0 range, and the neighbours of the C1 range
        // and of every Bidi_Control code point are outside the set too.
        let outside = "caf\u{e9} \u{7f} \u{a0} \u{61b}\u{61d} \u{200d}\u{2010} \u{2029}\u{202f} \u{2065}\u{206a} \u{4e2d} \u{1f600}";

        assert_eq!(escaped(outside), outside);
        assert_eq!(width(outside), outside.chars().count());
    }

    #[test]
    fn fr_out_018_the_escaped_form_carries_no_control_but_tab() {
        let hostile: String = (0x00_u32..0x20)
            .filter_map(char::from_u32)
            .chain("plain text".chars())
            .collect();

        let rendered = escaped(&hostile);

        assert_eq!(
            rendered
                .bytes()
                .filter(|&byte| byte < 0x20)
                .collect::<Vec<u8>>(),
            vec![b'\t'],
            "{rendered:?} carries a control the requirement does not except"
        );
    }

    #[test]
    fn fr_out_006_the_width_is_the_number_of_characters_the_rule_emits() {
        // The layout aligns a column with `width` and fills it with `write`.
        // A disagreement between the two is a misaligned listing, so the two
        // are held to each other here rather than by review.
        let mut corpus: Vec<String> = (0x00_u32..0x20)
            .filter_map(char::from_u32)
            .map(|control| format!("a{control}b"))
            .collect();

        corpus.extend(
            [
                "",
                "orders",
                "caf\u{e9}",
                "\u{1f600}",
                "one\ttwo",
                "a\u{0}\u{1f}\n\r\tz",
                "a\u{80}\u{9b}\u{9f}b",
                "\u{61c}\u{200e}\u{202e}\u{2069}",
                "mixed\u{1}\u{9b}\u{202e}caf\u{e9}\t",
            ]
            .into_iter()
            .map(str::to_owned),
        );

        for value in corpus {
            assert_eq!(
                width(&value),
                escaped(&value).chars().count(),
                "{value:?} is measured at a width it is not written at"
            );
        }
    }

    #[test]
    fn fr_out_004_a_wide_character_is_measured_at_one_character_and_not_at_two_cells() {
        // The accepted cost `width` states: the layout counts characters, and
        // a terminal draws this one two cells wide. FR-OUT-004 makes `text`
        // explicitly not a contract, and correcting it needs a Unicode width
        // table the dependency budget does not admit.
        assert_eq!(width("\u{4e2d}\u{6587}"), 2);
    }
}
