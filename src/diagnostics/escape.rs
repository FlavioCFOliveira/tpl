//! The escaping of `FR-ERR-024`, applied to a composed line as a whole.
//!
//! `FR-ERR-024` escapes `\n`, `\r`, `\t`, every C0 control character, every
//! C1 control character and every bidirectional formatting character in every
//! value a message interpolates, for a `mariadb` and a `sqlite` entry alike.
//! `OD-06` fixes *where* the rule is applied: the renderer escapes each
//! **composed line as a whole** rather than each interpolation, so an
//! interpolation nobody remembered to escape is escaped anyway. [`push_line`]
//! is therefore the only way a line leaves this module, and the newline it
//! appends is the only newline any line can carry.
//!
//! | Character | Written as |
//! |---|---|
//! | `\n`, `\r`, `\t` | `\n`, `\r`, `\t` |
//! | The other twenty-nine C0 controls | `\u{XX}`, two lowercase hexadecimal digits |
//! | A C1 control, U+0080 to U+009F | `\u{XX}`, the form of a C0 control: `\u{9b}` |
//! | A bidirectional formatting character | `\u{…}`, the code point in lowercase hexadecimal: `\u{61c}`, `\u{202e}` |
//!
//! Which characters past C0 are escaped is decided by
//! [`crate::output::escaped_past_c0`], the one predicate the `text` output and
//! the JSON formatter call too; the three paths differ in the form they write
//! and never in what they escape (`FR-ERR-024`, `FR-OUT-018`, as amended in the
//! sixty-ninth edition).
//!
//! Tab is escaped here. It is excepted only in the `text` output of a read
//! command, which is `FR-OUT-018`'s rule and `output/`'s business: a listing is
//! laid out in aligned columns and this format has none, so a tab inside an
//! interpolated catalogue name could only misalign the labels a caller reads
//! on.

use crate::output::{escaped_past_c0, may_hold_past_c0};

/// The lowercase hexadecimal digits, indexed by value.
const HEX: &[u8; 16] = b"0123456789abcdef";

/// The exclusive upper bound of the C0 control range, `U+0000`–`U+001F`.
const C0_END: u32 = 0x20;

/// Whether `text` carries a character [`push_line`] or [`bounded`] rewrites.
///
/// A C0 control is a byte below `0x20` in UTF-8, and every continuation byte
/// is at or above `0x80`, so that half of the scan is exact and needs no
/// decoding; the characters past C0 are found by their lead bytes and
/// confirmed by decoding.
fn rewrites(text: &str) -> bool {
    text.bytes().any(|byte| u32::from(byte) < C0_END)
        || (may_hold_past_c0(text) && text.chars().any(escaped_past_c0))
}

/// Appends `character` to `out`, escaped where `FR-ERR-024` escapes it.
fn push_escaped(out: &mut String, character: char) {
    match character {
        '\n' => out.push_str("\\n"),
        '\r' => out.push_str("\\r"),
        '\t' => out.push_str("\\t"),
        control if u32::from(control) < C0_END || escaped_past_c0(control) => {
            push_unicode(out, u32::from(control));
        }
        plain => out.push(plain),
    }
}

/// Appends `\u{…}` for `code`: lowercase hexadecimal of at least two digits.
fn push_unicode(out: &mut String, code: u32) {
    let mut digits = 1;
    while digits < 8 && code >> (4 * digits) != 0 {
        digits += 1;
    }
    let digits = digits.max(2);

    out.push_str("\\u{");
    for at in (0..digits).rev() {
        out.push(char::from(HEX[((code >> (4 * at)) & 0xf) as usize]));
    }
    out.push('}');
}

/// Appends `composed` to `out` with every character of `FR-ERR-024` escaped,
/// then appends the single newline that terminates the line.
///
/// `\n`, `\r` and `\t` become `\n`, `\r` and `\t`; the remaining twenty-nine
/// C0 controls and every character past C0 the rule names become `\u{…}`, in
/// lowercase hexadecimal of at least two digits. The appended text therefore
/// carries no control character of its own, which is what makes a forged
/// label line impossible: a caller reading the stream line by line sees
/// exactly the lines this module emitted.
pub(super) fn push_line(out: &mut String, composed: &str) {
    if rewrites(composed) {
        out.reserve(composed.len() + 1);
        for character in composed.chars() {
            push_escaped(out, character);
        }
    } else {
        out.push_str(composed);
    }

    out.push('\n');
}

/// The length past which `FR-ERR-046` bounds a value, in characters.
const BOUND: usize = 256;

/// How many characters of each end a bounded value keeps, at most.
const KEPT: usize = 128;

/// `value` escaped under `FR-ERR-024` and bounded under `FR-ERR-046`.
///
/// The value is escaped first — `\n`, `\r`, `\t`, and every other C0
/// control, C1 control and bidirectional formatting character as `\u{…}` —
/// and, where the escaped form is longer than 256 characters, replaced by at
/// most its first 128 characters, the marker `[... <n> characters omitted
/// ...]` and at most its last 128, `<n>` counting the escaped characters left
/// out. A cut that would fall inside an escape sequence moves to that
/// sequence's edge outside the kept part, so neither end carries half of one.
/// A character is a Unicode scalar value, the unit `FR-ERR-039` already counts
/// in.
///
/// The result holds no character the escape of the whole line rewrites, so
/// the escape [`push_line`] applies to the whole line leaves it unchanged.
pub(crate) fn bounded(value: &str) -> std::borrow::Cow<'_, str> {
    // Each unit is one character of the value, as escaped: one character, or
    // the two to eight of an escape sequence, which are never split.
    let units: Vec<String> = value.chars().map(escaped_unit).collect();
    let length: usize = units.iter().map(|unit| unit.chars().count()).sum();

    if length <= BOUND {
        if rewrites(value) {
            return std::borrow::Cow::Owned(units.concat());
        }
        return std::borrow::Cow::Borrowed(value);
    }

    let mut head = 0;
    let mut head_units = 0;
    for unit in &units {
        let width = unit.chars().count();
        if head + width > KEPT {
            break;
        }
        head += width;
        head_units += 1;
    }

    let mut tail = 0;
    let mut tail_units = 0;
    for unit in units[head_units..].iter().rev() {
        let width = unit.chars().count();
        if tail + width > KEPT {
            break;
        }
        tail += width;
        tail_units += 1;
    }

    let omitted = length - head - tail;
    let mut out = String::with_capacity(head + tail + 48);
    for unit in &units[..head_units] {
        out.push_str(unit);
    }
    out.push_str("[... ");
    out.push_str(&omitted.to_string());
    out.push_str(" characters omitted ...]");
    for unit in &units[units.len() - tail_units..] {
        out.push_str(unit);
    }

    std::borrow::Cow::Owned(out)
}

/// One character as `FR-ERR-024` escapes it.
fn escaped_unit(character: char) -> String {
    let mut unit = String::new();
    push_escaped(&mut unit, character);
    unit
}

#[cfg(test)]
mod tests {
    use super::{bounded, push_line};

    #[test]
    fn fr_err_046_a_value_of_256_characters_is_written_whole() {
        let value = "a".repeat(256);

        assert_eq!(bounded(&value), value);
    }

    #[test]
    fn fr_err_046_a_value_of_257_characters_keeps_both_ends_and_counts_what_was_left_out() {
        let value = format!("{}{}", "h".repeat(200), "t".repeat(57));
        let written = bounded(&value);

        let head = "h".repeat(128);
        let tail = format!("{}{}", "h".repeat(71), "t".repeat(57));
        assert_eq!(
            written,
            format!("{head}[... 1 characters omitted ...]{tail}")
        );
    }

    #[test]
    fn fr_err_046_a_cut_never_falls_inside_an_escape_sequence() {
        // FR-ERR-046 with FR-ERR-024: the value is escaped first, and a cut
        // that would fall inside `\n`, `\r`, `\t` or `\u{XX}` moves to the
        // sequence's edge outside the kept part.
        for (control, form) in [
            ('\n', "\\n"),
            ('\r', "\\r"),
            ('\t', "\\t"),
            ('\u{1}', "\\u{01}"),
        ] {
            // The head's 128th character falls inside the escape.
            let value: String = "x"
                .repeat(127)
                .chars()
                .chain(std::iter::once(control))
                .chain("y".repeat(300).chars())
                .chain(std::iter::once(control))
                .chain("z".repeat(127).chars())
                .collect();
            let written = bounded(&value);
            let (head, rest) = written
                .split_once("[... ")
                .expect("the value is longer than the bound");
            let (_, tail) = rest.split_once(" ...]").expect("the marker closes");

            assert_eq!(
                head,
                "x".repeat(127),
                "{form}: the head stops before the escape"
            );
            assert_eq!(
                tail,
                "z".repeat(127),
                "{form}: the tail starts after the escape"
            );
            assert!(!written.bytes().any(|byte| byte < 0x20));
            let omitted: usize = rest
                .split_once(' ')
                .and_then(|(count, _)| count.parse().ok())
                .expect("the marker counts");
            assert_eq!(omitted, 2 * form.chars().count() + 300);
        }
    }

    #[test]
    fn fr_err_046_a_short_value_is_escaped_and_nothing_else() {
        assert_eq!(bounded("no such table: a\nb"), "no such table: a\\nb");
        assert_eq!(bounded("plain"), "plain");
    }

    /// Renders `composed` through [`push_line`] and drops the terminator.
    fn escaped(composed: &str) -> String {
        let mut out = String::new();
        push_line(&mut out, composed);
        assert!(out.ends_with('\n'));
        out.truncate(out.len() - 1);
        out
    }

    #[test]
    fn fr_err_024_a_line_without_a_control_character_is_unchanged() {
        assert_eq!(
            escaped("error: table 'ordrs' does not exist"),
            "error: table 'ordrs' does not exist"
        );
    }

    #[test]
    fn fr_err_024_the_three_named_controls_take_their_readable_escape() {
        assert_eq!(escaped("a\nb\rc\td"), "a\\nb\\rc\\td");
    }

    #[test]
    fn fr_err_024_every_other_c0_control_takes_the_unicode_escape() {
        // FR-ERR-024 names three controls and then "every C0 control
        // character", which is U+0000 through U+001F.
        for value in 0x00u32..0x20 {
            let control = char::from_u32(value).expect("every C0 code point is a character");
            if matches!(control, '\n' | '\r' | '\t') {
                continue;
            }
            let rendered = escaped(&control.to_string());
            assert_eq!(rendered, format!("\\u{{{value:02x}}}"));
        }
    }

    #[test]
    fn fr_err_024_the_escaped_line_carries_no_control_character() {
        let hostile: String = (0x00u32..0x20)
            .filter_map(char::from_u32)
            .chain("plain text".chars())
            .collect();

        let rendered = escaped(&hostile);

        assert!(
            !rendered.bytes().any(|byte| byte < 0x20),
            "{rendered:?} still carries a control character"
        );
    }

    #[test]
    fn fr_err_024_a_character_outside_the_escaped_set_is_left_alone() {
        // FR-ERR-024's set is C0, C1 and the twelve Bidi_Control code points.
        // A name is UTF-8 and its other characters reach the caller as
        // written: U+007F, the neighbours of the C1 range and of every
        // Bidi_Control code point, and the zero-width joiner the sixty-ninth
        // edition declines to escape among them.
        let outside = "ordens_de_compra \u{7f} \u{e9} \u{a0} \u{61b}\u{61d} \u{200d}\u{2010} \u{2029}\u{202f} \u{2065}\u{206a} \u{1f600}";

        assert_eq!(escaped(outside), outside);
        assert_eq!(bounded(outside), outside);
    }

    #[test]
    fn fr_err_024_a_c1_control_and_a_bidirectional_formatting_character_take_the_c0_form() {
        // FR-ERR-024 as amended in the sixty-ninth edition: each is escaped in
        // the form a C0 control is — `\u{`, the code point in lowercase
        // hexadecimal of at least two digits, `}`.
        for code in 0x80_u32..=0x9f {
            let control = char::from_u32(code).expect("every C1 code point is a character");
            assert_eq!(escaped(&control.to_string()), format!("\\u{{{code:02x}}}"));
        }
        for code in [
            0x61c_u32, 0x200e, 0x200f, 0x202a, 0x202b, 0x202c, 0x202d, 0x202e, 0x2066, 0x2067,
            0x2068, 0x2069,
        ] {
            let character = char::from_u32(code).expect("a Bidi_Control code point");
            assert_eq!(escaped(&character.to_string()), format!("\\u{{{code:x}}}"));
        }
        // A line forged to reorder what a reader sees, and to introduce a
        // control sequence, is written so that both are seen.
        assert_eq!(
            escaped("error: table '\u{202e}cod.exe\u{9b}31m' does not exist"),
            "error: table '\\u{202e}cod.exe\\u{9b}31m' does not exist"
        );
    }

    #[test]
    fn fr_err_024_the_escaped_line_carries_no_character_of_the_set() {
        let hostile: String = (0x00_u32..0x20)
            .chain(0x80..0xa0)
            .chain([0x61c, 0x200e, 0x200f, 0x202a, 0x202e, 0x2066, 0x2069])
            .filter_map(char::from_u32)
            .chain("plain text".chars())
            .collect();

        let rendered = escaped(&hostile);

        assert!(
            !rendered.chars().any(|character| u32::from(character) < 0x20
                || crate::output::escaped_past_c0(character)),
            "{rendered:?} still carries a character of the set"
        );
        assert_eq!(
            bounded(&hostile).chars().filter(|c| c.is_control()).count(),
            0
        );
    }

    #[test]
    fn fr_err_046_a_cut_never_falls_inside_a_c1_or_bidirectional_escape() {
        // FR-ERR-046 with FR-ERR-024 as amended in the sixty-ninth edition:
        // the count is taken after escaping, and a cut that would fall inside
        // `\u{9b}`, `\u{61c}` or `\u{202e}` moves to the sequence's edge
        // outside the kept part.
        for (character, form) in [
            ('\u{9b}', "\\u{9b}"),
            ('\u{61c}', "\\u{61c}"),
            ('\u{202e}', "\\u{202e}"),
        ] {
            let value: String = "x"
                .repeat(127)
                .chars()
                .chain(std::iter::once(character))
                .chain("y".repeat(300).chars())
                .chain(std::iter::once(character))
                .chain("z".repeat(127).chars())
                .collect();
            let written = bounded(&value);
            let (head, rest) = written
                .split_once("[... ")
                .expect("the value is longer than the bound");
            let (_, tail) = rest.split_once(" ...]").expect("the marker closes");

            assert_eq!(head, "x".repeat(127), "{form}: the head stops before it");
            assert_eq!(tail, "z".repeat(127), "{form}: the tail starts after it");
            let omitted: usize = rest
                .split_once(' ')
                .and_then(|(count, _)| count.parse().ok())
                .expect("the marker counts");
            assert_eq!(omitted, 2 * form.chars().count() + 300);
        }

        // Counted after escaping: 250 characters and one C1 control are 256
        // escaped characters, written whole; one more is cut.
        let whole = format!("{}\u{9b}", "a".repeat(250));
        assert_eq!(bounded(&whole), format!("{}\\u{{9b}}", "a".repeat(250)));
        let cut = format!("{}\u{9b}", "a".repeat(251));
        assert!(
            bounded(&cut).contains("characters omitted"),
            "{}",
            bounded(&cut)
        );
    }
}
