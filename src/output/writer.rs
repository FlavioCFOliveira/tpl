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
//!
//! **A classified failure seals the stream**, which is what keeps `FR-ERR-033`
//! true on the way out. A write that fails leaves the bytes it had not yet
//! emitted inside the buffer, and a buffer flushes itself when it is dropped —
//! so the truncated remainder of a document whose outcome is already an error
//! would reach stdout after the failure had been reported, on a path the
//! requirement obliges to leave stdout empty. [`Tracked::seal`] closes that:
//! once [`Writer::classify`] has decided the outcome, the tracked stream
//! absorbs what the buffer hands it instead of forwarding it, and the drop
//! empties the buffer into nothing.

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
    /// Whether the outcome has been classified and the stream closed to
    /// anything further.
    sealed: bool,
}

impl<W> Tracked<W> {
    /// Wraps `stream`, with no byte of any document yet emitted.
    const fn new(stream: W) -> Self {
        Self {
            stream,
            emitted: false,
            sealed: false,
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

    /// Closes the stream to everything the buffer still holds.
    ///
    /// Called once the outcome of a write has been classified. What the buffer
    /// holds at that moment is the part of a document that never reached the
    /// consumer, and emitting it afterwards would put a truncated document on
    /// stdout on a path `FR-ERR-033` obliges to leave stdout empty — which is
    /// what a buffer's own drop would otherwise do.
    fn seal(&mut self) {
        self.sealed = true;
    }
}

impl<W: Write> Write for Tracked<W> {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        // A sealed stream reports the write as accepted without making it, so
        // the buffer above empties into nothing rather than retrying a stream
        // whose outcome is already settled. Reporting a refusal instead would
        // leave the same bytes in the buffer for the next attempt; this is the
        // behaviour of `io::Sink`, applied to a stream that is finished with.
        if self.sealed {
            return Ok(buf.len());
        }

        // A partial write followed by a failure still emitted bytes, and the
        // consumer holds them: the flag is set from what the stream accepted,
        // before the failure is propagated on the next call.
        let written = self.stream.write(buf)?;
        self.emitted |= written > 0;

        Ok(written)
    }

    fn flush(&mut self) -> io::Result<()> {
        if self.sealed {
            return Ok(());
        }

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

    /// Writes one help text, as composed, and flushes it.
    ///
    /// `BR-CLI-005` makes help a legitimate stdout payload, and it is written
    /// through this buffer for the reason every other payload is: the bytes are
    /// aggregated rather than emitted a line at a time. A consumer that goes
    /// away is classified as [`Cut::Harmless`], which is the row a `text`
    /// listing occupies and for the ground `FR-ERR-025` gives it: `FR-ERR-026`
    /// names a **JSON document** and nothing else, and a help text a consumer
    /// cut short is exactly what `tpl --help | head -20` asked for.
    ///
    /// A stream a previous call found closed is not written to again.
    ///
    /// # Errors
    ///
    /// Returns [`Error::StdoutUnwritable`] where the stream refused the write
    /// for a reason other than a close. A close returns `Ok(())`, which is the
    /// silent success of `FR-ERR-025`.
    pub(super) fn help(&mut self, text: &str) -> Result<(), Error> {
        if self.finished {
            return Ok(());
        }

        self.inner.get_mut().begin();

        match self.write_help(text) {
            Ok(()) => Ok(()),
            Err(refused) => self.classify(refused, Cut::Harmless),
        }
    }

    /// Writes one document and empties the buffer into the stream.
    fn write<T: Serialize>(&mut self, document: &Document<T>, form: Form) -> io::Result<()> {
        json::write_document(&mut self.inner, document, form)?;
        self.inner.flush()
    }

    /// Writes one help text and empties the buffer into the stream.
    fn write_help(&mut self, text: &str) -> io::Result<()> {
        self.inner.write_all(text.as_bytes())?;
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
    ///
    /// The stream is sealed **before** the outcome is decided, and for every
    /// outcome alike: the bytes the buffer still holds belong to output that
    /// failed, and none of the three outcomes wants them emitted afterwards.
    /// Sealing here rather than in the caller is what makes that hold for the
    /// drop as well as for the return, because the drop is the only writer left
    /// once this function has run.
    fn classify(&mut self, refused: io::Error, cut: Cut) -> Result<(), Error> {
        self.finished = true;
        self.inner.get_mut().seal();

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
    use serde::ser::{Error as _, SerializeStruct as _};
    use serde::{Serialize, Serializer};
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
    fn a_help_text_reaches_the_stream_as_the_renderer_composed_it() {
        // BR-CLI-005: help is a legitimate stdout payload, and this path lays
        // nothing out — FR-HELP-009 puts the line breaks in the text.
        let mut emitted = Vec::new();
        Writer::new(&mut emitted)
            .help("USAGE\n  tpl version [options]\n")
            .expect("a buffer accepts every write");

        assert_eq!(
            String::from_utf8(emitted).expect("the renderer emits UTF-8"),
            "USAGE\n  tpl version [options]\n"
        );
    }

    #[test]
    fn a_help_text_cut_by_the_consumer_is_the_silent_success() {
        // FR-ERR-025, on the same ground a cut listing rests on: FR-ERR-026
        // names a JSON document and nothing else, and `tpl --help | head -1`
        // is what the consumer asked for.
        let mut stream = Refusing::new(6, io::ErrorKind::BrokenPipe);

        let outcome = Writer::new(&mut stream).help("USAGE\n  tpl version [options]\n");

        assert!(outcome.is_ok(), "a cut help text is exit 0, not 74");
        assert_eq!(stream.accepted.len(), 6, "the consumer holds part of it");
    }

    #[test]
    fn a_help_text_refused_for_another_reason_is_unwritable() {
        // The 74 row of FR-ERR-001: only a close is excused on this path.
        let mut stream = Refusing::new(6, io::ErrorKind::StorageFull);

        let refused = Writer::new(&mut stream)
            .help("USAGE\n  tpl version [options]\n")
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

    // ------------------------------- what a failure leaves on stdout ---

    /// A payload that serialises two fields and then refuses.
    ///
    /// `FR-OUT-027` leaves the shape of `data` to the module that owns each
    /// command, so a payload with a `Serialize` of its own is admissible and
    /// `Collection` is already one. This is such a payload failing part-way,
    /// which is the condition `json::write_document` reports as
    /// [`io::ErrorKind::InvalidData`].
    struct FailsPartWay;

    impl Serialize for FailsPartWay {
        fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
            let mut payload = serializer.serialize_struct("FailsPartWay", 3)?;
            payload.serialize_field("tables", &["orders", "customers"])?;
            payload.serialize_field("count", &2_u32)?;

            Err(S::Error::custom("the payload could not be serialised"))
        }
    }

    /// A stream that refuses the write crossing `fails_at` once, and accepts
    /// every write before and after it.
    ///
    /// This is a consumer that returned `EAGAIN` on a descriptor it had put in
    /// non-blocking mode and then drained: the first write is refused, and a
    /// later one would succeed.
    struct Recovering {
        accepted: Vec<u8>,
        fails_at: usize,
        refused: bool,
    }

    impl Recovering {
        const fn new(fails_at: usize) -> Self {
            Self {
                accepted: Vec::new(),
                fails_at,
                refused: false,
            }
        }
    }

    impl Write for Recovering {
        fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
            if !self.refused && self.accepted.len() + buf.len() > self.fails_at {
                self.refused = true;
                return Err(io::Error::from(io::ErrorKind::WouldBlock));
            }

            self.accepted.extend_from_slice(buf);

            Ok(buf.len())
        }

        fn flush(&mut self) -> io::Result<()> {
            Ok(())
        }
    }

    #[test]
    fn a_payload_that_fails_part_way_leaves_stdout_empty() {
        // FR-ERR-033: the outcome is an error, so stdout is left empty. The
        // part of the document the encoder had composed is still inside the
        // buffer when the failure is classified, and the buffer's own drop
        // would emit it — after the failure had been reported, as a truncated
        // document no consumer can tell is incomplete.
        let mut emitted = Vec::new();
        let document = Document::new(Source::Server, FailsPartWay);

        let refused = Writer::new(&mut emitted)
            .document(&document, Form::Compact)
            .expect_err("a payload that refuses is not a success");

        assert!(
            matches!(refused, Error::StdoutUnwritable { .. }),
            "{refused:?} is not the condition of the 74 row"
        );
        assert_eq!(refused.exit_code(), 74);
        assert!(
            emitted.is_empty(),
            "a truncated document reached stdout: {:?}",
            String::from_utf8_lossy(&emitted)
        );
    }

    #[test]
    fn a_stream_that_recovers_receives_nothing_after_the_failure() {
        // The same rule where the refusal is the stream's rather than the
        // payload's, and the stream would accept the retry: nothing of the
        // document may follow the failure that was already reported.
        let members: Vec<String> = (0..2_000).map(|at| format!("table_{at:06}")).collect();
        let large = Document::new(Source::Server, Collection::new("tables", &members));
        let mut stream = Recovering::new(9_000);

        let refused = Writer::new(&mut stream)
            .document(&large, Form::Compact)
            .expect_err("a stream that refuses the write is not a success");

        assert!(
            matches!(refused, Error::StdoutUnwritable { .. }),
            "{refused:?} is not the condition of the 74 row"
        );

        let held = stream.accepted.len();
        assert!(held <= 9_000, "the stream accepted more than it admitted");
        assert!(
            !stream.accepted.ends_with(b"}\n"),
            "the document cannot have been completed on a refused stream"
        );
    }

    #[test]
    fn a_listing_refused_part_way_leaves_nothing_further_on_stdout() {
        // A listing is the silent 0 of FR-ERR-025 when the consumer closes,
        // and FR-ERR-033's empty stdout when the stream refuses for another
        // reason. Neither wants the remainder of the buffer afterwards.
        let mut stream = Recovering::new(4);

        let refused = Writer::new(&mut stream)
            .table(&listing())
            .expect_err("a refusal that is not a close is unwritable");

        assert!(
            matches!(refused, Error::StdoutUnwritable { .. }),
            "{refused:?}"
        );
        assert!(
            stream.accepted.len() <= 4,
            "the buffer reached the stream after the failure: {:?}",
            String::from_utf8_lossy(&stream.accepted)
        );
    }
}
