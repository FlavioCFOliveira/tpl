//! The one buffered writer a result reaches stdout through, and the
//! `0`-or-`74` distinction of `FR-ERR-025` and `FR-ERR-026`.
//!
//! A consumer that closes stdout is two different outcomes, and which one it is
//! depends on a fact only the writer holds:
//!
//! | When the stream closed | Outcome | Why |
//! |---|---|---|
//! | Before a byte of a document was emitted | Exit `0`, silently (`FR-ERR-025`) | `tpl … \| head -1` is not an error, and the consumer holds nothing it could misread |
//! | After a byte of a document was emitted | Exit `74` (`FR-ERR-026`) | The consumer holds truncated JSON and cannot tell that it is incomplete |
//!
//! A `text` listing is the third row, and it is not in the table because the
//! fact above does not decide it: `FR-ERR-026` names a **JSON document** and
//! nothing else, and the ground `FR-ERR-025` gives is this case outright — "in
//! `text`, a cut listing is exactly what `head` asked for". A listing is
//! therefore the silent `0` however much of it the consumer holds, which is
//! what [`Cut`] carries into the classification. Both paths share the buffer
//! all the same: the aggregation is required of every write, not of one format.
//!
//! So the writer records **whether any byte of the current document has been
//! accepted by the stream**, which is what `OD-18` places here rather than in
//! the encoder: it is a property of the writer, and `FR-OUT-021` and the
//! project's rule that I/O is aggregated require a buffered writer regardless,
//! so the state costs nothing extra.
//!
//! The flag has to be read at the **stream**, not at the buffer. Bytes handed
//! to a buffer have not been emitted, and a document small enough to fit in one
//! buffer is handed over in a single write at the flush — so a flag flipped on
//! entry to the buffer would report every closed stream as mid-document and
//! `FR-ERR-025` would have no case left. [`Tracked`] therefore sits **under**
//! the buffer, between it and the stream, and counts what the stream itself
//! accepted.
//!
//! "Accepted by the stream" is as far as a writer can see, and for standard
//! output the stream is the standard library's handle, which carries a line
//! buffer of its own. A document small enough to sit in that buffer has
//! therefore been emitted as far as this module can tell, even where the pipe
//! was already gone — so a consumer that closed during such a document is
//! reported as `74` rather than `0`. That is the conservative half of the pair
//! and the one the requirement's own ground asks for: the document was in
//! flight, and a caller told `74` declines to trust stdout, which is the
//! correct conclusion whether it received part of the document or none of it.
//!
//! This depends on a closed consumer arriving as [`io::ErrorKind::BrokenPipe`]
//! rather than as a signal that ends the process before any of this runs. Were
//! the process terminated by `SIGPIPE` instead, neither requirement could be
//! satisfied at all: there would be no frame left to decide between `0` and
//! `74`.

use std::io::{self, BufWriter, Write};

use serde::Serialize;

use super::envelope::Document;
use super::json::{self, Form};
use super::text::{self, Table};
use crate::error::Error;

/// What a consumer's close part-way through means for the output in flight.
///
/// `FR-ERR-025` and `FR-ERR-026` divide on the kind of output, not only on the
/// moment: `FR-ERR-026` names a **JSON document** and nothing else, and the
/// ground `FR-ERR-025` gives for the other half names the other kind outright —
/// "in `text`, a cut listing is exactly what `head` asked for". A listing has no
/// outer shape a consumer could be misled about the completeness of; a document
/// does, and a truncated one parses as nothing at all or, worse, as less than it
/// is.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Cut {
    /// A JSON document: a consumer that holds part of one cannot tell, so a
    /// close after the first byte is `74` (`FR-ERR-026`).
    Truncates,
    /// A `text` listing: a close is the silent `0` of `FR-ERR-025` however much
    /// of the listing the consumer holds.
    Harmless,
}

/// A stream that records whether it has accepted a byte since the last
/// [`Tracked::begin`].
///
/// It wraps the stream rather than the buffer, so `emitted` answers "has any
/// byte of this document left the process" and not "has any byte been
/// buffered". The two differ by exactly the case `FR-ERR-025` exists for.
#[derive(Debug)]
struct Tracked<W> {
    /// The stream the bytes go to.
    stream: W,
    /// Whether `stream` has accepted a byte of the current document.
    emitted: bool,
}

impl<W> Tracked<W> {
    /// Wraps `stream`, with no byte of any document yet emitted.
    const fn new(stream: W) -> Self {
        Self {
            stream,
            emitted: false,
        }
    }

    /// Marks the start of a document, per `FR-ERR-025`.
    fn begin(&mut self) {
        self.emitted = false;
    }

    /// Whether a byte of the current document has been emitted.
    const fn emitted(&self) -> bool {
        self.emitted
    }
}

impl<W: Write> Write for Tracked<W> {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        // A partial write followed by a failure still emitted bytes, and the
        // consumer holds them: the flag is set from what the stream accepted,
        // before the failure is propagated on the next call.
        let written = self.stream.write(buf)?;
        self.emitted |= written > 0;

        Ok(written)
    }

    fn flush(&mut self) -> io::Result<()> {
        self.stream.flush()
    }
}

/// The writer every result is emitted through, in either format.
///
/// It owns one buffer, so a result reaches the stream in as few writes as the
/// buffer allows rather than one per line — the aggregation `FR-OUT-021` and
/// the project's own I/O rule require. The capacity is the standard library's
/// default: no measurement supports another number, and this project does not
/// tune without one.
///
/// [`Writer::document`] and [`Writer::table`] flush before they return, so the
/// writer holds nothing between results and there is no pending byte for a drop
/// to lose. That is
/// also what makes the flag of [`Tracked`] exact: a document's bytes have
/// either reached the stream or produced the failure being classified.
pub(super) struct Writer<W: Write> {
    /// The buffer, over the stream that records what it accepted.
    inner: BufWriter<Tracked<W>>,
    /// Whether the stream is finished — closed by the consumer, or refusing.
    finished: bool,
}

impl<W: Write> Writer<W> {
    /// Wraps `stream` in the buffer every result is written through.
    pub(super) fn new(stream: W) -> Self {
        Self {
            inner: BufWriter::new(Tracked::new(stream)),
            finished: false,
        }
    }

    /// Writes one document, in `form`, and flushes it.
    ///
    /// A stream a previous call found closed is not written to again: the
    /// process is on its way to the silent `0` of `FR-ERR-025`, and writing
    /// further documents into a dead stream would only convert that outcome
    /// into a different one.
    ///
    /// # Errors
    ///
    /// Returns [`Error::StdoutClosedMidDocument`] where the stream closed after
    /// a byte of this document had been emitted (`FR-ERR-026`), and
    /// [`Error::StdoutUnwritable`] where the stream refused the write for any
    /// other reason. A stream closed before the first byte returns `Ok(())`,
    /// which is the silent success of `FR-ERR-025`.
    pub(super) fn document<T: Serialize>(
        &mut self,
        document: &Document<T>,
        form: Form,
    ) -> Result<(), Error> {
        if self.finished {
            return Ok(());
        }

        self.inner.get_mut().begin();

        match self.write(document, form) {
            Ok(()) => Ok(()),
            Err(refused) => self.classify(refused, Cut::Truncates),
        }
    }

    /// Writes one `text` listing, in the layout of `FR-OUT-006`, and flushes
    /// it.
    ///
    /// It is written through the same buffer a document is, for the same
    /// reason: `FR-OUT-021` and the project's own rule require the bytes to be
    /// aggregated rather than emitted a line at a time. What differs is the
    /// classification of a consumer that goes away, which is [`Cut`].
    ///
    /// A stream a previous call found closed is not written to again.
    ///
    /// # Errors
    ///
    /// Returns [`Error::StdoutUnwritable`] where the stream refused the write
    /// for a reason other than a close. A close is not an error on this path at
    /// all: `FR-ERR-025` makes a cut listing the silent success its own
    /// rationale names, so this returns `Ok(())` whatever had been emitted.
    pub(super) fn table<C, const COLUMNS: usize>(
        &mut self,
        table: &Table<'_, C, COLUMNS>,
    ) -> Result<(), Error>
    where
        C: AsRef<str>,
    {
        if self.finished {
            return Ok(());
        }

        // Nothing on this path reads the flag, but the flag describes whatever
        // the writer is emitting and a stale one would describe the last
        // document instead.
        self.inner.get_mut().begin();

        match self.write_table(table) {
            Ok(()) => Ok(()),
            Err(refused) => self.classify(refused, Cut::Harmless),
        }
    }

    /// Writes one document and empties the buffer into the stream.
    fn write<T: Serialize>(&mut self, document: &Document<T>, form: Form) -> io::Result<()> {
        json::write_document(&mut self.inner, document, form)?;
        self.inner.flush()
    }

    /// Writes one listing and empties the buffer into the stream.
    fn write_table<C, const COLUMNS: usize>(
        &mut self,
        table: &Table<'_, C, COLUMNS>,
    ) -> io::Result<()>
    where
        C: AsRef<str>,
    {
        text::write_table(&mut self.inner, table)?;
        self.inner.flush()
    }

    /// Decides which of the two outcomes a refused write is.
    ///
    /// The stream is finished either way, so nothing further is attempted on
    /// it. What separates `FR-ERR-025` from `FR-ERR-026` is `cut` — the kind of
    /// output in flight — and, where that kind can be truncated, the one fact
    /// [`Tracked`] holds.
    fn classify(&mut self, refused: io::Error, cut: Cut) -> Result<(), Error> {
        self.finished = true;

        if refused.kind() != io::ErrorKind::BrokenPipe {
            return Err(Error::StdoutUnwritable { returned: refused });
        }

        match cut {
            // FR-ERR-026: the consumer holds truncated JSON.
            Cut::Truncates if self.inner.get_ref().emitted() => Err(Error::StdoutClosedMidDocument),
            // FR-ERR-025: nothing of the document reached the consumer, or the
            // output was a listing, which a consumer may cut where it likes.
            Cut::Truncates | Cut::Harmless => Ok(()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::Writer;
    use crate::error::Error;
    use crate::output::envelope::{Collection, Document, Source};
    use crate::output::json::Form;
    use crate::output::text::{Order, Table};
    use std::io::{self, Write};

    /// A stream that accepts a fixed number of bytes and then refuses.
    ///
    /// It stands for a consumer that has gone away: `fail_after` bytes reach
    /// it, and every byte after that is refused with `kind`. What it accepted
    /// stays readable afterwards, so a test can state how much of a document
    /// the consumer actually holds.
    struct Refusing {
        accepted: Vec<u8>,
        fail_after: usize,
        kind: io::ErrorKind,
    }

    impl Refusing {
        const fn new(fail_after: usize, kind: io::ErrorKind) -> Self {
            Self {
                accepted: Vec::new(),
                fail_after,
                kind,
            }
        }
    }

    impl Write for Refusing {
        fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
            let remaining = self.fail_after - self.accepted.len();
            if remaining == 0 {
                return Err(io::Error::from(self.kind));
            }

            let take = buf.len().min(remaining);
            self.accepted.extend_from_slice(&buf[..take]);

            Ok(take)
        }

        fn flush(&mut self) -> io::Result<()> {
            Ok(())
        }
    }

    /// A document with one member, for a stream to accept or refuse.
    fn sample() -> Document<Collection<'static, &'static str>> {
        Document::new(Source::Server, Collection::new("tables", &["orders"]))
    }

    /// The rows of the sample listing.
    const LISTING: [[&str; 2]; 2] = [["orders", "InnoDB"], ["customers", "InnoDB"]];

    /// A listing of two rows, for a stream to accept or refuse.
    fn listing() -> Table<'static, &'static str, 2> {
        Table::new(["NAME", "ENGINE"], &LISTING, Order::ByName)
    }

    #[test]
    fn a_document_reaches_the_stream_whole() {
        let mut emitted = Vec::new();
        Writer::new(&mut emitted)
            .document(&sample(), Form::Compact)
            .expect("a buffer accepts every write");

        assert_eq!(
            String::from_utf8(emitted).expect("the encoder emits UTF-8"),
            "{\"schema_version\":1,\"source\":\"server\",\"data\":{\"tables\":[\"orders\"]}}\n"
        );
    }

    #[test]
    fn a_stream_closed_before_the_first_byte_is_a_silent_success() {
        // FR-ERR-025: `tpl … | head -1` is not an error. Nothing of the
        // document reached the consumer, so there is nothing it could misread
        // and the process ends silently at 0.
        let mut stream = Refusing::new(0, io::ErrorKind::BrokenPipe);

        let outcome = Writer::new(&mut stream).document(&sample(), Form::Compact);

        assert!(outcome.is_ok(), "a close before the first byte is exit 0");
        assert!(stream.accepted.is_empty(), "no byte was emitted");
    }

    #[test]
    fn a_stream_closed_after_the_first_byte_is_the_mid_document_condition() {
        // FR-ERR-026: the consumer holds truncated JSON and cannot tell.
        let mut stream = Refusing::new(12, io::ErrorKind::BrokenPipe);

        let refused = Writer::new(&mut stream)
            .document(&sample(), Form::Compact)
            .expect_err("a close part-way through a document is 74");

        assert!(
            matches!(refused, Error::StdoutClosedMidDocument),
            "{refused:?} is not the condition of FR-ERR-026"
        );
        assert_eq!(refused.exit_code(), 74);
        assert_eq!(stream.accepted, b"{\"schema_ver", "a byte was emitted");
    }

    #[test]
    fn a_document_larger_than_the_buffer_is_classified_the_same_way() {
        // A document that does not fit the buffer is refused inside the
        // encoder rather than at the final flush, so the refusal comes back
        // wrapped in the encoder's own error type. Both requirements have to
        // survive that unwrapping: a close before the first byte is still the
        // silent 0 of FR-ERR-025 and not the 74 of FR-ERR-026.
        let members: Vec<String> = (0..2_000).map(|at| format!("table_{at:06}")).collect();
        let large = Document::new(Source::Server, Collection::new("tables", &members));

        let mut closed = Refusing::new(0, io::ErrorKind::BrokenPipe);
        assert!(
            Writer::new(&mut closed)
                .document(&large, Form::Compact)
                .is_ok(),
            "FR-ERR-025 holds whatever the size of the document"
        );
        assert!(closed.accepted.is_empty(), "no byte was emitted");

        let mut cut = Refusing::new(9_000, io::ErrorKind::BrokenPipe);
        let refused = Writer::new(&mut cut)
            .document(&large, Form::Compact)
            .expect_err("FR-ERR-026 holds whatever the size of the document");

        assert!(
            matches!(refused, Error::StdoutClosedMidDocument),
            "{refused:?} is not the condition of FR-ERR-026"
        );
        assert_eq!(cut.accepted.len(), 9_000, "the consumer holds part of it");
    }

    #[test]
    fn a_stream_that_refuses_for_another_reason_is_unwritable() {
        // The 74 row of FR-ERR-001: standard output could not be written. It
        // is not the EPIPE rule and does not consult the mid-document state.
        let mut stream = Refusing::new(12, io::ErrorKind::StorageFull);

        let refused = Writer::new(&mut stream)
            .document(&sample(), Form::Compact)
            .expect_err("a stream that refuses the write is 74");

        assert!(
            matches!(refused, Error::StdoutUnwritable { .. }),
            "{refused:?} is not the condition of the 74 row"
        );
        assert_eq!(refused.exit_code(), 74);
    }

    #[test]
    fn a_refusal_before_the_first_byte_is_unwritable_when_it_is_not_a_close() {
        // The distinction FR-ERR-025 draws is about a *closed* stream. A stream
        // that refuses for another reason is a failure wherever it arises.
        let mut stream = Refusing::new(0, io::ErrorKind::PermissionDenied);

        let refused = Writer::new(&mut stream)
            .document(&sample(), Form::Compact)
            .expect_err("a refusal that is not a close is 74");

        assert!(matches!(refused, Error::StdoutUnwritable { .. }));
        assert_eq!(refused.exit_code(), 74);
    }

    #[test]
    fn nothing_further_is_written_after_the_consumer_closed_the_stream() {
        // FR-ERR-025 ends the process; a second document written into a dead
        // stream could only turn that silent 0 into a different outcome.
        let mut stream = Refusing::new(0, io::ErrorKind::BrokenPipe);
        {
            let mut writer = Writer::new(&mut stream);

            writer
                .document(&sample(), Form::Compact)
                .expect("the first document is the silent success");
            writer
                .document(&sample(), Form::Compact)
                .expect("the second is not attempted at all");
        }

        assert!(stream.accepted.is_empty());
    }

    #[test]
    fn a_listing_reaches_the_stream_whole() {
        // FR-OUT-006, FR-OUT-020: the `text` result goes to stdout through the
        // same buffer the JSON result does.
        let mut emitted = Vec::new();
        Writer::new(&mut emitted)
            .table(&listing())
            .expect("a buffer accepts every write");

        assert_eq!(
            String::from_utf8(emitted).expect("the layout emits UTF-8"),
            "NAME       ENGINE\ncustomers  InnoDB\norders     InnoDB\n"
        );
    }

    #[test]
    fn a_listing_cut_by_the_consumer_is_the_silent_success() {
        // FR-ERR-025, against the same stream state that makes a JSON document
        // the 74 of FR-ERR-026: the requirement names a *document*, and its own
        // ground names this case — "in `text`, a cut listing is exactly what
        // `head` asked for".
        let mut stream = Refusing::new(12, io::ErrorKind::BrokenPipe);

        let outcome = Writer::new(&mut stream).table(&listing());

        assert!(outcome.is_ok(), "a cut listing is exit 0, not 74");
        assert_eq!(stream.accepted.len(), 12, "the consumer holds part of it");
    }

    #[test]
    fn a_listing_into_a_stream_closed_before_the_first_byte_is_a_silent_success_too() {
        let mut stream = Refusing::new(0, io::ErrorKind::BrokenPipe);

        assert!(Writer::new(&mut stream).table(&listing()).is_ok());
        assert!(stream.accepted.is_empty(), "no byte was emitted");
    }

    #[test]
    fn a_listing_refused_for_another_reason_is_unwritable() {
        // The 74 row of FR-ERR-001 reaches the `text` path unchanged: only a
        // *close* is excused, and only because the consumer asked for it.
        let mut stream = Refusing::new(12, io::ErrorKind::StorageFull);

        let refused = Writer::new(&mut stream)
            .table(&listing())
            .expect_err("a stream that refuses the write is 74");

        assert!(
            matches!(refused, Error::StdoutUnwritable { .. }),
            "{refused:?} is not the condition of the 74 row"
        );
        assert_eq!(refused.exit_code(), 74);
    }

    #[test]
    fn the_indented_form_reaches_the_stream_too() {
        // FR-OUT-008: --pretty changes the whitespace and nothing else about
        // how a document is written.
        let mut emitted = Vec::new();
        Writer::new(&mut emitted)
            .document(&sample(), Form::Indented)
            .expect("a buffer accepts every write");

        let document = String::from_utf8(emitted).expect("the encoder emits UTF-8");

        assert!(
            document.starts_with("{\n  \"schema_version\": 1,\n"),
            "{document}"
        );
        assert!(document.ends_with("}\n"), "{document}");
    }
}
