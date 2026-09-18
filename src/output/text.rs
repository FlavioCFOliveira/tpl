//! The `text` layout of `FR-OUT-006`: aligned columns under a header row.
//!
//! `FR-OUT-004` states what this output is not — a contract, whose shape may
//! change without a version bump — and `FR-OUT-005` sends anything that parses
//! to `--format json`. What it is instead is `FR-OUT-006`: a layout for a
//! person, carrying the useful information rather than only the name, with
//! every column as wide as the widest thing in it. `FR-OUT-034` keeps that
//! shape constant across the data by printing the header row of an empty
//! result and nothing beneath it, so a reader sees which listing answered.
//!
//! Three rules meet here, and each belongs to a different requirement:
//!
//! | Rule | Requirement | Where it is applied |
//! |---|---|---|
//! | Every interpolated value is escaped, tab excepted | `FR-OUT-018`, `FR-OUT-019` | [`super::escape`], over every cell including the header cells |
//! | The rows are ordered by name, ascending, byte-wise, unless the caller carries an excepted order | `NFR-DET-002` | [`Order`], stated at every call site |
//! | The bytes are aggregated and the consumer's close is classified | `FR-OUT-021`, `FR-ERR-025` | [`super::writer`], which owns the one buffer |
//!
//! **Escaping is applied to the whole table, not to the cells a caller
//! remembered to escape.** `FR-OUT-019` reaches every value whatever its
//! source, and the way to make that structural rather than reviewable is to
//! leave the caller no way to interpolate anything itself: a cell arrives as
//! text and is escaped on the way out. `OD-06` settled the same question the
//! same way for a diagnostic line. The header cells go through the rule too;
//! they are the system's own labels and carry nothing to rewrite, and exempting
//! them would only add a path with no rule on it.
//!
//! **It does not stream, and cannot.** A column is as wide as the widest cell
//! in it, so the first line cannot be written until the last row has been read;
//! and the default ordering is over the whole collection, so the first row is
//! not known until every row is. What it does instead is hold no copy: the rows
//! stay the caller's, the layout makes two passes over them — one to measure,
//! one to write — and allocates one `usize` per row, and only where it sorts.
//! Nothing per cell is allocated at all, and the bytes reach the consumer
//! through the buffer [`super::writer`] owns rather than one write per line.

use std::io::{self, Write};

use super::escape;

/// The gap between two columns.
///
/// Two spaces, which is the gap the worked listing of `FR-SCH-026` shows
/// between its columns. It is spaces rather than a tab because the columns are
/// aligned, per `FR-OUT-006`, and a tab aligns to the reader's tab stops rather
/// than to the content.
const SEPARATOR: &[u8] = b"  ";

/// A run of spaces, so padding costs one call per column rather than one per
/// space.
const SPACES: &[u8; 32] = &[b' '; 32];

/// The byte every line ends with.
const TERMINATOR: u8 = b'\n';

/// Which order the rows of a listing are presented in (`NFR-DET-002`).
///
/// The requirement asks for two things at once — that an ordering be
/// **explicit** and that it be **stable** — so this is a parameter of
/// [`Table::new`] rather than a default a call site can omit: a listing states
/// which of the two rules it falls under, and the choice is greppable.
/// [`Order::ByName`] is the default rule all the same, and the type says so.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub(crate) enum Order {
    /// By the first column, ascending, compared byte by byte.
    ///
    /// This is the rule `NFR-DET-002` applies to every collection it does not
    /// except. The first column is the name, per `FR-OUT-006`, which carries
    /// "the useful information rather than only the name" and therefore leads
    /// with it.
    #[default]
    ByName,
    /// The order the caller supplied, unchanged.
    ///
    /// This is for the six collections `NFR-DET-002` excepts — a table's
    /// columns, an index's columns, a primary key's columns, a foreign key's
    /// paired columns, an `ENUM` or `SET` member list, and a routine's
    /// parameters. Each of those orders is carried from `mariadb/` and is the
    /// meaning of the collection rather than a presentation of it; re-deriving
    /// one here is what the requirement's seventh-edition amendment records as
    /// a silent corruption of correct-looking output.
    AsGiven,
}

/// One `text` listing: a header row, and the rows beneath it.
///
/// The column count is a const parameter, so a row that does not match the
/// header row is a compile error rather than a ragged line: there is no
/// missing cell to render blank and no extra cell to drop.
///
/// `C` is the cell type, borrowed wherever the caller can borrow. A listing of
/// catalogue names passes `[&str; N]` rows and copies nothing; a listing
/// carrying a formatted count passes `[Cow<'_, str>; N]` and allocates only for
/// the cells it had to compose.
#[derive(Debug)]
pub(crate) struct Table<'a, C, const COLUMNS: usize> {
    /// The header row, per `FR-OUT-006`.
    headers: [&'static str; COLUMNS],
    /// The rows beneath it, in the order `order` presents them in.
    rows: &'a [[C; COLUMNS]],
    /// Which rule of `NFR-DET-002` those rows are presented under.
    order: Order,
}

impl<'a, C, const COLUMNS: usize> Table<'a, C, COLUMNS> {
    /// Lays `rows` out under `headers`, presented in `order`.
    ///
    /// `order` is [`Order::ByName`] unless this collection is one of the six
    /// `NFR-DET-002` excepts, in which case `rows` already carry the excepted
    /// order and it is [`Order::AsGiven`].
    pub(crate) const fn new(
        headers: [&'static str; COLUMNS],
        rows: &'a [[C; COLUMNS]],
        order: Order,
    ) -> Self {
        Self {
            headers,
            rows,
            order,
        }
    }
}

/// The order the rows are written in.
///
/// [`Sequence::Given`] allocates nothing, which is the whole reason this is an
/// enum rather than a permutation that is always built: the excepted
/// collections are presented in the order they arrived in, and building the
/// identity permutation for them would be an allocation with no effect.
#[derive(Debug)]
enum Sequence {
    /// The caller's own order.
    Given,
    /// A permutation of the caller's rows, by name.
    ByName(Vec<usize>),
}

impl Sequence {
    /// Works out the order `table`'s rows are written in.
    ///
    /// The comparison is over **bytes**, named as bytes: `NFR-DET-002` fixes
    /// "ascending, compared byte by byte", and a comparison written over
    /// `&[u8]` cannot quietly become a locale-aware one, because there is no
    /// collation of bytes to reach for. `Ord` for `str` would compare the same
    /// way today and states a weaker thing about what it will compare tomorrow.
    ///
    /// The comparator is **total**: two rows whose names are equal are ordered
    /// by the position they arrived in, so the result does not depend on the
    /// sort being stable and two equal names keep the caller's order.
    fn of<C: AsRef<str>, const COLUMNS: usize>(table: &Table<'_, C, COLUMNS>) -> Self {
        match table.order {
            Order::AsGiven => Self::Given,
            Order::ByName => {
                let mut places: Vec<usize> = (0..table.rows.len()).collect();
                places.sort_unstable_by(|&left, &right| {
                    name(table, left)
                        .cmp(name(table, right))
                        .then(left.cmp(&right))
                });

                Self::ByName(places)
            }
        }
    }

    /// The row that occupies `position` in the listing.
    fn at(&self, position: usize) -> usize {
        match self {
            Self::Given => position,
            Self::ByName(places) => places[position],
        }
    }
}

/// The bytes the row at `position` is ordered by.
///
/// The name is the first column, per `FR-OUT-006`. A table of no columns has
/// no name to order by and every row compares equal, which leaves them in the
/// order they arrived in.
fn name<'r, C: AsRef<str>, const COLUMNS: usize>(
    table: &'r Table<'_, C, COLUMNS>,
    position: usize,
) -> &'r [u8] {
    table.rows[position]
        .first()
        .map_or(&[][..], |cell| cell.as_ref().as_bytes())
}

/// The width of each column: the widest thing it holds, header included.
fn widths<C: AsRef<str>, const COLUMNS: usize>(table: &Table<'_, C, COLUMNS>) -> [usize; COLUMNS] {
    let mut widths = table.headers.map(escape::width);

    for row in table.rows {
        for (width, cell) in widths.iter_mut().zip(row) {
            *width = (*width).max(escape::width(cell.as_ref()));
        }
    }

    widths
}

/// Writes one line: each cell escaped, padded to its column, and separated.
///
/// The line ends at its last populated cell. Padding a trailing empty cell
/// would put invisible spaces at the end of every short line — which the worked
/// listing of `FR-SCH-026` does not carry, and which a reader diffing two
/// listings would have to strip.
fn line<W: Write, T: AsRef<str>>(writer: &mut W, cells: &[T], widths: &[usize]) -> io::Result<()> {
    if let Some(last) = cells.iter().rposition(|cell| !cell.as_ref().is_empty()) {
        for (at, cell) in cells.iter().enumerate().take(last + 1) {
            let cell = cell.as_ref();
            escape::write(writer, cell)?;

            if at < last {
                // `widths` is measured over these cells, so the column is
                // never narrower than the cell it holds.
                pad(writer, widths[at].saturating_sub(escape::width(cell)))?;
                writer.write_all(SEPARATOR)?;
            }
        }
    }

    writer.write_all(&[TERMINATOR])
}

/// Writes `spaces` spaces.
fn pad<W: Write>(writer: &mut W, mut spaces: usize) -> io::Result<()> {
    while spaces > 0 {
        let run = spaces.min(SPACES.len());
        writer.write_all(&SPACES[..run])?;
        spaces -= run;
    }

    Ok(())
}

/// Writes one listing to `writer`: the header row, then the rows beneath it.
///
/// An empty listing writes the header row and stops, per `FR-OUT-034`, so the
/// shape does not change with the data.
///
/// Nothing is flushed here. The buffer belongs to [`super::writer`], which owns
/// it because it also owns the consequence of a failed flush.
///
/// # Errors
///
/// Returns what `writer` returned.
pub(super) fn write_table<W, C, const COLUMNS: usize>(
    writer: &mut W,
    table: &Table<'_, C, COLUMNS>,
) -> io::Result<()>
where
    W: Write,
    C: AsRef<str>,
{
    let widths = widths(table);
    let sequence = Sequence::of(table);

    line(writer, &table.headers, &widths)?;

    for position in 0..table.rows.len() {
        line(writer, &table.rows[sequence.at(position)], &widths)?;
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use std::borrow::Cow;

    use super::{Order, Table, write_table};

    /// The listing `table` reaches a reader as.
    fn rendered<C: AsRef<str>, const COLUMNS: usize>(table: &Table<'_, C, COLUMNS>) -> String {
        let mut out = Vec::new();
        write_table(&mut out, table).expect("a buffer accepts every write");

        String::from_utf8(out).expect("the layout emits UTF-8")
    }

    #[test]
    fn fr_out_006_a_listing_is_aligned_columns_under_a_header_row() {
        // FR-OUT-006: aligned columns under a header row, carrying the useful
        // information rather than only the name.
        let rows = [
            ["customers", "InnoDB", "Registered buyers"],
            ["order_items", "InnoDB", ""],
            ["orders", "InnoDB", "One row per order"],
        ];

        assert_eq!(
            rendered(&Table::new(
                ["NAME", "ENGINE", "COMMENT"],
                &rows,
                Order::ByName
            )),
            concat!(
                "NAME         ENGINE  COMMENT\n",
                "customers    InnoDB  Registered buyers\n",
                "order_items  InnoDB\n",
                "orders       InnoDB  One row per order\n",
            )
        );
    }

    #[test]
    fn fr_out_034_an_empty_listing_prints_the_header_row_and_nothing_beneath_it() {
        // FR-OUT-034: the shape does not change with the data, and a reader
        // sees which listing answered rather than a blank screen.
        let empty: [[&str; 3]; 0] = [];

        assert_eq!(
            rendered(&Table::new(
                ["NAME", "ENGINE", "COMMENT"],
                &empty,
                Order::ByName
            )),
            "NAME  ENGINE  COMMENT\n"
        );
    }

    #[test]
    fn fr_out_006_a_value_wider_than_its_header_sets_the_column_width() {
        let rows = [["a", "x"], ["a_considerably_longer_name", "y"]];

        assert_eq!(
            rendered(&Table::new(["NAME", "FLAG"], &rows, Order::ByName)),
            concat!(
                "NAME                        FLAG\n",
                "a                           x\n",
                "a_considerably_longer_name  y\n",
            )
        );
    }

    #[test]
    fn fr_out_006_a_header_wider_than_every_value_sets_the_column_width() {
        let rows = [["a", "x"], ["b", "y"]];

        assert_eq!(
            rendered(&Table::new(["COLLATION", "FLAG"], &rows, Order::ByName)),
            concat!("COLLATION  FLAG\n", "a          x\n", "b          y\n")
        );
    }

    #[test]
    fn fr_out_018_a_c0_control_in_a_value_is_escaped_and_a_tab_is_not() {
        // FR-OUT-018 and FR-OUT-019: every interpolated value is escaped
        // whatever its source, and tab alone is excepted because the columns
        // are laid out with spacing rather than escaped away.
        let rows = [["forged\nline", "a\u{1b}[31mb"], ["kept\ttab", "plain"]];

        assert_eq!(
            rendered(&Table::new(["NAME", "COMMENT"], &rows, Order::AsGiven)),
            concat!(
                "NAME          COMMENT\n",
                "forged\\nline  a\\u{1b}[31mb\n",
                "kept\ttab      plain\n",
            )
        );
    }

    #[test]
    fn fr_out_018_a_value_cannot_forge_a_line_of_the_listing() {
        // The line-forgery ground of FR-OUT-018: a reader splitting the
        // listing on newlines sees exactly the lines the layout emitted.
        let rows = [["a\nb\rc\u{7}d", "\u{0}"]];
        let listing = rendered(&Table::new(["NAME", "COMMENT"], &rows, Order::AsGiven));

        assert_eq!(listing.lines().count(), 2, "{listing:?}");
        assert!(
            !listing.bytes().any(|byte| byte < 0x20 && byte != b'\n'),
            "{listing:?} carries a control the requirement does not except"
        );
    }

    #[test]
    fn fr_out_006_an_escaped_value_is_measured_at_the_width_it_is_printed_at() {
        // The column is as wide as what the reader sees, not as wide as what
        // the catalogue held: the escape is six characters and the column
        // accommodates six.
        let rows = [["a\u{0}b", "x"], ["cc", "y"]];

        assert_eq!(
            rendered(&Table::new(["N", "F"], &rows, Order::AsGiven)),
            concat!("N         F\n", "a\\u{00}b  x\n", "cc        y\n")
        );
    }

    #[test]
    fn nfr_det_002_the_default_order_is_by_name_ascending_and_byte_wise() {
        // NFR-DET-002: byte-wise, which puts every uppercase letter before
        // every lowercase one. A locale-aware collation orders these the other
        // way round — `en_US` sorts `apple`, `Banana`, `cherry` — so this
        // assertion fails the moment a collation reaches the comparison.
        let rows = [["cherry"], ["apple"], ["Banana"]];

        assert_eq!(
            rendered(&Table::new(["NAME"], &rows, Order::ByName)),
            "NAME\nBanana\napple\ncherry\n"
        );
    }

    #[test]
    fn nfr_det_002_an_accented_name_sorts_where_its_bytes_put_it_and_not_where_a_collation_would() {
        // `é` is U+00E9, which is 0xc3 0xa9 in UTF-8 and therefore after every
        // ASCII letter. A collation sorts it beside `e`, which would place
        // `época` first of the three.
        let rows = [["zebra"], ["\u{e9}poca"], ["escala"]];

        assert_eq!(
            rendered(&Table::new(["NAME"], &rows, Order::ByName)),
            "NAME\nescala\nzebra\n\u{e9}poca\n"
        );
    }

    #[test]
    fn nfr_det_002_an_excepted_collection_keeps_the_order_it_was_given() {
        // NFR-DET-002's exceptions: a primary key's columns are the order the
        // catalogue states, and sorting them by name is a different key.
        let rows = [["vessel_imo"], ["voyage_number"], ["leg_sequence"]];

        assert_eq!(
            rendered(&Table::new(["COLUMN"], &rows, Order::AsGiven)),
            "COLUMN\nvessel_imo\nvoyage_number\nleg_sequence\n"
        );
    }

    #[test]
    fn nfr_det_002_two_equal_names_keep_the_order_they_arrived_in() {
        // The comparator is total, so the listing does not depend on the sort
        // being stable.
        let rows = [["same", "second"], ["same", "first"], ["other", "third"]];

        assert_eq!(
            rendered(&Table::new(["NAME", "NOTE"], &rows, Order::ByName)),
            concat!(
                "NAME   NOTE\n",
                "other  third\n",
                "same   second\n",
                "same   first\n",
            )
        );
    }

    #[test]
    fn no_line_carries_trailing_whitespace() {
        let rows = [["a", ""], ["bb", "y"], ["", ""]];
        let listing = rendered(&Table::new(["NAME", "NOTE"], &rows, Order::AsGiven));

        for line in listing.lines() {
            assert_eq!(line, line.trim_end(), "{listing:?}");
        }
    }

    #[test]
    fn every_line_is_terminated() {
        let rows = [["a"], ["b"]];
        let listing = rendered(&Table::new(["NAME"], &rows, Order::ByName));

        assert!(listing.ends_with('\n'));
        assert_eq!(listing.matches('\n').count(), 3);
    }

    #[test]
    fn nfr_det_002_by_name_is_the_default_rule_the_requirement_states() {
        // NFR-DET-002: every collection it does not except is ordered by name.
        assert_eq!(Order::default(), Order::ByName);
    }

    #[test]
    fn fr_out_006_a_cell_the_caller_composed_is_laid_out_like_any_other() {
        // The cell type is borrowed wherever the caller can borrow, and owned
        // where it had to compose the value — a count formatted for a column.
        let rows = [
            [Cow::Borrowed("customers"), Cow::Owned(14.to_string())],
            [Cow::Borrowed("orders"), Cow::Owned(21.to_string())],
        ];

        assert_eq!(
            rendered(&Table::new(["NAME", "COLUMNS"], &rows, Order::ByName)),
            concat!("NAME       COLUMNS\n", "customers  14\n", "orders     21\n")
        );
    }
}
