//! The `text` cells and the multi-part payload the commands of this tree share.
//!
//! [`crate::output`] owns the layout of `FR-OUT-006` — the widths, the padding,
//! the separator, the escaping and the ordering — and owns nothing about what a
//! particular result puts in a cell. This module is the other half, and it is
//! here rather than in one command's module because two arms need it: the first
//! arm lays a named object out in several parts, and `tpl cache status` reports
//! an entry, a load time and a count per collection, which is two parts as well.
//!
//! **Nothing here composes a line.** A cell arrives as text and is escaped on
//! the way out by the layout, per `FR-OUT-018` and `FR-OUT-019`; the only bytes
//! this module writes itself are the headings, which are its callers' literals.

use std::borrow::Cow;

use crate::error::Error;
use crate::output::{self, Order, Table};

/// The two columns an object's own properties are laid out in.
///
/// It is the shape a result that is **not** a listing takes: `FR-SCH-026`
/// governs a listing, and one object is not one, so its fields are rows and the
/// order they are given in is the order they are printed in.
pub(super) const PROPERTY: [&str; 2] = ["PROPERTY", "VALUE"];

/// What a boolean cell reads as.
const YES: &str = "yes";

/// What a boolean cell reads as when it is not [`YES`].
const NO: &str = "no";

/// One cell.
pub(super) type Cell<'a> = Cow<'a, str>;

/// `flag` as the cell a reader sees.
pub(super) fn flag(flag: bool) -> Cell<'static> {
    Cow::Borrowed(if flag { YES } else { NO })
}

/// `count` as the cell a reader sees.
pub(super) fn count(count: usize) -> Cell<'static> {
    Cow::Owned(count.to_string())
}

/// `text` as a cell, borrowed.
pub(super) const fn text(text: &str) -> Cell<'_> {
    Cow::Borrowed(text)
}

/// An optional value as a cell, empty where there is none.
///
/// `FR-SCH-026` clause 5 ends a row at its last non-empty cell, so an absent
/// value costs no trailing whitespace. `FR-OUT-012`'s `null` is the `json`
/// path's and does not reach this one.
pub(super) fn optional(value: Option<&str>) -> Cell<'_> {
    value.map_or(Cow::Borrowed(""), Cow::Borrowed)
}

/// Several tables under headings, written as one payload.
///
/// The parts are composed into one buffer and emitted in a single call, so the
/// bytes reach the consumer aggregated rather than one section at a time, which
/// is what `FR-OUT-021` and this project's own I/O rule require.
#[derive(Debug, Default)]
pub(super) struct Sections {
    /// The composed bytes.
    written: Vec<u8>,
}

impl Sections {
    /// Adds a heading, preceded by a blank line where anything precedes it.
    pub(super) fn heading(&mut self, title: &str) {
        if !self.written.is_empty() {
            self.written.push(b'\n');
        }
        self.written.extend_from_slice(title.as_bytes());
        self.written.push(b'\n');
    }

    /// Adds one laid-out table.
    ///
    /// # Errors
    ///
    /// Returns what [`output::emit_table_to`] returns. A buffer accepts every
    /// write, so the arm is the signature's own totality.
    pub(super) fn table<C, const N: usize>(&mut self, table: &Table<'_, C, N>) -> Result<(), Error>
    where
        C: AsRef<str>,
    {
        output::emit_table_to(&mut self.written, table)
    }

    /// Adds a heading and the table beneath it, where the table has rows.
    ///
    /// An empty part is left out entirely rather than printed as a header row
    /// with nothing beneath it: `FR-OUT-034` governs an empty **result**, and
    /// the result of a named read is the object, which is present. A command
    /// whose result *is* the collection writes the heading and the table
    /// itself, so that the header row of `FR-OUT-034` is printed.
    ///
    /// # Errors
    ///
    /// Returns what [`Sections::table`] returns.
    pub(super) fn part<C, const N: usize>(
        &mut self,
        title: &str,
        headers: [&'static str; N],
        rows: &[[C; N]],
        order: Order,
    ) -> Result<(), Error>
    where
        C: AsRef<str>,
    {
        if rows.is_empty() {
            return Ok(());
        }

        self.heading(title);
        self.table(&Table::new(headers, rows, order))
    }

    /// The composed payload.
    ///
    /// Every byte of it was written by the layout of [`crate::output`] or is
    /// one of the caller's own heading literals, so the result is UTF-8 by
    /// construction; a buffer that somehow were not is emitted with the
    /// replacement character rather than refused, which is the treatment
    /// `FR-OUT-017` gives a byte sequence that is not valid UTF-8.
    pub(super) fn finish(&self) -> Cow<'_, str> {
        String::from_utf8_lossy(&self.written)
    }
}

/// Writes `sections` to `out`.
///
/// # Errors
///
/// Returns [`Error::StdoutUnwritable`] where the stream refused the write.
pub(super) fn emit<W: std::io::Write>(out: W, sections: &Sections) -> Result<(), Error> {
    output::emit_verbatim(out, &sections.finish())
}

#[cfg(test)]
mod tests {
    use super::{Sections, flag, optional};
    use crate::output::{Order, Table};

    #[test]
    fn a_boolean_cell_reads_as_a_word_rather_than_as_a_number() {
        assert_eq!(flag(true), "yes");
        assert_eq!(flag(false), "no");
    }

    #[test]
    fn fr_sch_026_an_absent_value_is_an_empty_cell_and_carries_no_trailing_whitespace() {
        // FR-SCH-026 clause 5: a row ends at its last non-empty cell, so an
        // absent value at the end of a row costs nothing at all.
        assert_eq!(optional(None), "");
        assert_eq!(optional(Some("InnoDB")), "InnoDB");

        let rows = [[optional(Some("orders")), optional(None)]];
        let mut sections = Sections::default();
        sections
            .table(&Table::new(["NAME", "ENGINE"], &rows, Order::ByName))
            .expect("a buffer accepts every write");

        assert_eq!(sections.finish(), "NAME    ENGINE\norders\n");
    }

    #[test]
    fn a_heading_separates_two_parts_by_one_blank_line_and_never_leads_with_one() {
        let first = [["a"]];
        let second = [["b"]];
        let mut sections = Sections::default();

        sections
            .part("FIRST", ["N"], &first, Order::ByName)
            .expect("a buffer accepts every write");
        sections
            .part("SECOND", ["N"], &second, Order::ByName)
            .expect("a buffer accepts every write");

        assert_eq!(sections.finish(), "FIRST\nN\na\n\nSECOND\nN\nb\n");
    }

    #[test]
    fn an_empty_part_is_left_out_rather_than_printed_as_a_bare_header_row() {
        // FR-OUT-034 governs an empty result, and the result of a named read
        // is the object, which is present. A table with no trigger prints no
        // trigger section.
        let empty: [[&str; 1]; 0] = [];
        let mut sections = Sections::default();

        sections
            .part("TRIGGERS", ["N"], &empty, Order::ByName)
            .expect("a buffer accepts every write");

        assert_eq!(sections.finish(), "");
    }
}
