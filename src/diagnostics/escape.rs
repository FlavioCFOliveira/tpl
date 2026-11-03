//! The escaping of `FR-ERR-024`, applied to a composed line as a whole.
//!
//! `FR-ERR-024` escapes `\n`, `\r`, `\t` and every C0 control character in
//! every value a message interpolates. `OD-06` fixes *where* the rule is
//! applied: the renderer escapes each **composed line as a whole** rather than
//! each interpolation, so an interpolation nobody remembered to escape is
//! escaped anyway. [`push_line`] is therefore the only way a line leaves this
//! module, and the newline it appends is the only newline any line can carry.
//!
//! Tab is escaped here. It is excepted only in the `text` output of a read
//! command, which is `FR-OUT-018`'s rule and `output/`'s business: a listing is
//! laid out in aligned columns and this format has none, so a tab inside an
//! interpolated catalogue name could only misalign the labels a caller reads
//! on.

/// The lowercase hexadecimal digits, indexed by value.
const HEX: &[u8; 16] = b"0123456789abcdef";

/// The exclusive upper bound of the C0 control range, `U+0000`–`U+001F`.
const C0_END: u32 = 0x20;

/// Appends `composed` to `out` with every C0 control character escaped, then
/// appends the single newline that terminates the line.
///
/// `\n`, `\r` and `\t` become `\n`, `\r` and `\t`; the remaining twenty-nine C0
/// controls become `\u{XX}`, with two lowercase hexadecimal digits. The
/// appended text therefore carries no control character of its own, which is
/// what makes a forged label line impossible: a caller reading the stream line
/// by line sees exactly the lines this module emitted.
pub(super) fn push_line(out: &mut String, composed: &str) {
    // A C0 control is a byte below 0x20 in UTF-8, and every continuation byte
    // is at or above 0x80, so this scan is exact and needs no decoding.
    if composed.bytes().any(|byte| u32::from(byte) < C0_END) {
        out.reserve(composed.len() + 1);
        for character in composed.chars() {
            match character {
                '\n' => out.push_str("\\n"),
                '\r' => out.push_str("\\r"),
                '\t' => out.push_str("\\t"),
                control if u32::from(control) < C0_END => {
                    let value = u32::from(control);
                    out.push_str("\\u{");
                    out.push(char::from(HEX[(value >> 4) as usize]));
                    out.push(char::from(HEX[(value & 0xf) as usize]));
                    out.push('}');
                }
                plain => out.push(plain),
            }
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
/// The value is escaped first — `\n`, `\r`, `\t` and every other C0 control
/// as `\u{XX}` — and, where the escaped form is longer than 256 characters,
/// replaced by at most its first 128 characters, the marker
/// `[... <n> characters omitted ...]` and at most its last 128, `<n>` counting
/// the escaped characters left out. A cut that would fall inside an escape
/// sequence moves to that sequence's edge outside the kept part, so neither end
/// carries half of one. A character is a Unicode scalar value, the unit
/// `FR-ERR-039` already counts in.
///
/// The result holds no C0 control, so the escape [`push_line`] applies to the
/// whole line leaves it unchanged.
pub(crate) fn bounded(value: &str) -> std::borrow::Cow<'_, str> {
    // Each unit is one character of the value, as escaped: one character, or
    // the two or six of an escape sequence, which are never split.
    let units: Vec<String> = value.chars().map(escaped_unit).collect();
    let length: usize = units.iter().map(|unit| unit.chars().count()).sum();

    if length <= BOUND {
        if value.bytes().any(|byte| u32::from(byte) < C0_END) {
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
    match character {
        '\n' => "\\n".to_owned(),
        '\r' => "\\r".to_owned(),
        '\t' => "\\t".to_owned(),
        control if u32::from(control) < C0_END => {
            let value = u32::from(control);
            let mut unit = String::from("\\u{");
            unit.push(char::from(HEX[(value >> 4) as usize]));
            unit.push(char::from(HEX[(value & 0xf) as usize]));
            unit.push('}');
            unit
        }
        plain => plain.to_string(),
    }
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
    fn fr_err_024_a_character_above_the_c0_range_is_left_alone() {
        // FR-ERR-024's set is the C0 range. A name is UTF-8 and its
        // non-control characters reach the caller as written.
        assert_eq!(
            escaped("ordens_de_compra \u{7f} \u{e9} \u{1f600}"),
            "ordens_de_compra \u{7f} \u{e9} \u{1f600}"
        );
    }
}
