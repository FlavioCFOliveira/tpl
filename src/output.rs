//! The envelope every JSON document carries, the emitter, and the writer that
//! puts one on stdout.
//!
//! `FR-OUT-024` fixes one outer shape of exactly three keys for **every**
//! document the system writes, and `FR-OUT-032` admits no exception to it:
//! there is no JSON document outside the envelope, because `FR-ERR-033`
//! withdraws the JSON error document. A calling agent that has learned one
//! document has therefore learned the outer shape of all seventeen, and the
//! module expresses that structurally rather than by review — [`emit`] accepts
//! a [`Document`] and nothing else, so an unenveloped value has no route to
//! stdout.
//!
//! The payload is the other half. `FR-OUT-027` leaves the shape of `data` to
//! the module that owns each command, and the two shapes every command shares
//! are here: [`Collection`] for a listing, per `FR-OUT-030`, and the envelope
//! itself. A payload module therefore states its own fields and re-derives
//! neither.
//!
//! | Submodule | Subject | Forced by |
//! |---|---|---|
//! | [`envelope`] | The three keys, the four values of `source`, and the collection shape | `FR-OUT-024` … `FR-OUT-032`, `FR-OUT-035` |
//! | [`json`] | The two forms, and the only call into `serde_json` | `FR-OUT-007`, `FR-OUT-008`, `FR-OUT-013`, `OD-18` |
//! | [`writer`] | The buffered writer, and the `0`-or-`74` distinction | `FR-ERR-025`, `FR-ERR-026`, `FR-OUT-020`, `FR-OUT-021` |
//!
//! Key order is stated in the types and nowhere else. `OD-18` settles
//! **derived** `Serialize`, so a struct's field declaration order **is** the
//! emitted key order; `preserve_order` is off and `indexmap` is not in the
//! dependency graph, which is what `FR-OUT-013` requires when it forbids an
//! unordered map on the emitting path.
//!
//! `diagnostics/` is the deliberate neighbour, not the same module. It writes
//! to stderr, which `NFR-DET-001` puts outside the contract, and it escapes tab
//! in every message; this module writes to stdout, which is byte-identical
//! under the same requirement, and on the `json` path the escape JSON itself
//! defines is what satisfies `FR-OUT-018`. `OD-05` keeps the two apart for
//! exactly that reason.
//!
//! What is **not** here: the seventeen payload shapes, each owned by the module
//! that owns its command per `BR-OUT-002`; and the `text` layouts of
//! `FR-OUT-006` with the escaping rule that is theirs alone.

mod envelope;
mod json;
mod writer;

use serde::Serialize;

use crate::error::Error;
use writer::Writer;

// `Document` is this module's own; the other two are what a command builds a
// payload from, and the first of those commands is a later sprint. The
// suppression is this one statement's, so a stale import anywhere else in the
// module is still reported.
#[allow(
    unused_imports,
    reason = "no command builds a payload yet; the re-export is the path those commands will use"
)]
pub(crate) use envelope::{Collection, Document, Source};
pub(crate) use json::Form;

/// Writes one document to standard output, in `form`.
///
/// This is the only route a result takes to stdout, per `FR-OUT-020`, and it
/// carries a [`Document`] rather than an arbitrary value so that `FR-OUT-032`
/// holds by construction: nothing unenveloped can be emitted, because nothing
/// unenveloped can be passed. A command with no result calls nothing and leaves
/// stdout empty, which is `FR-OUT-023`.
///
/// The stream is locked for the whole document and the bytes are aggregated
/// behind one buffer, so a document reaches the consumer in as few writes as
/// the buffer allows rather than one per line.
///
/// # Errors
///
/// Returns [`Error::StdoutClosedMidDocument`] where the consumer closed stdout
/// after a byte of this document had been emitted (`FR-ERR-026`), and
/// [`Error::StdoutUnwritable`] where the stream refused the write for any other
/// reason. A consumer that closed stdout **before** the first byte is not a
/// failure at all: `FR-ERR-025` makes it a silent success and this returns
/// `Ok(())`.
pub(crate) fn emit<T: Serialize>(document: &Document<T>, form: Form) -> Result<(), Error> {
    Writer::new(std::io::stdout().lock()).document(document, form)
}
