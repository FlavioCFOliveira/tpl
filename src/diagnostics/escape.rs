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

#[cfg(test)]
mod tests {
    use super::push_line;

    /// Renders `composed` through [`push_line`] and drops the terminator.
    fn escaped(composed: &str) -> String {
        let mut out = String::new();
        push_line(&mut out, composed);
        assert!(out.ends_with('\n'));
        out.truncate(out.len() - 1);
        out
    }

    #[test]
    fn a_line_without_a_control_character_is_unchanged() {
        assert_eq!(
            escaped("error: table 'ordrs' does not exist"),
            "error: table 'ordrs' does not exist"
        );
    }

    #[test]
    fn the_three_named_controls_take_their_readable_escape() {
        assert_eq!(escaped("a\nb\rc\td"), "a\\nb\\rc\\td");
    }

    #[test]
    fn every_other_c0_control_takes_the_unicode_escape() {
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
    fn the_escaped_line_carries_no_control_character() {
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
    fn a_character_above_the_c0_range_is_left_alone() {
        // FR-ERR-024's set is the C0 range. A name is UTF-8 and its
        // non-control characters reach the caller as written.
        assert_eq!(
            escaped("ordens_de_compra \u{7f} \u{e9} \u{1f600}"),
            "ordens_de_compra \u{7f} \u{e9} \u{1f600}"
        );
    }
}
