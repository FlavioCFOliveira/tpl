//! The `restricted` marking an incomplete object carries (`FR-PRIV-005` …
//! `FR-PRIV-007`, `FR-PRIV-016`).
//!
//! `FR-PRIV-016` fixes three properties of the marking, and two of them are
//! structural here. It is a collection of **property names**, so a caller
//! learns *what* could not be read rather than only *that* something could
//! not; it is **never empty**, which [`Restricted::new`] enforces by refusing
//! to build one; and it is **present only on an incomplete object**, which the
//! [`Option`] every marked type wraps it in expresses — [`None`] is the
//! complete object of `FR-PRIV-007`, and there is no empty array for it to be
//! confused with.
//!
//! *The byte-wise ordering `FR-PRIV-016` also requires is `NFR-DET-002`'s, and
//! it is applied where this marking becomes a **document collection** — the
//! projection below — and not in the constructor.* The value keeps the order it
//! is given, so nothing silently rearranges a marking a caller is holding; the
//! document applies the one comparator `model::document::order` states, exactly
//! as every other collection's build site does.
//!
//! The projection is the build site because `restricted` is the one collection
//! of the document whose members are bare strings rather than objects with
//! names of their own, and because it is carried inside three types — a table,
//! a view and a routine — that reach the document through their own derived
//! serialisation.

use std::borrow::Cow;

use serde::{Deserialize, Serialize};
use thiserror::Error;

use super::ToStatic;
use super::document::order;

/// A `restricted` marking that names no property (`FR-PRIV-016`).
///
/// It is the read-back counterpart of [`Restricted::new`] returning [`None`]:
/// the constructor cannot produce an empty marking, and a supplied document
/// that carries one is refused rather than repaired. The caller maps it to
/// [`Error::ContextDocumentMalformed`](crate::Error::ContextDocumentMalformed).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Error)]
#[error("a 'restricted' marking names no property")]
#[non_exhaustive]
pub struct EmptyMarking;

/// The properties of one object that could not be read (`FR-PRIV-016`).
///
/// The field is private and [`Restricted::new`] is the only constructor,
/// because the non-emptiness of `FR-PRIV-016` is the whole point: an empty
/// marking would say an object is incomplete and name nothing that is missing,
/// which is the boolean the requirement rejected in as many words.
///
/// The document shape is a **bare array of strings**, not an object, so the
/// serialisation is a projection onto [`Vec<Cow<'a, str>>`](Vec) rather than a
/// derive over the field. The projection is what applies the byte-wise order
/// `FR-PRIV-016` requires, and [`TryFrom`] is what re-establishes the
/// non-emptiness on the way back in.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(into = "Vec<Cow<'a, str>>", try_from = "Vec<Cow<'a, str>>")]
#[non_exhaustive]
pub struct Restricted<'a> {
    /// The property names, never empty.
    properties: Vec<Cow<'a, str>>,
}

impl<'a> From<Restricted<'a>> for Vec<Cow<'a, str>> {
    /// The document's array, ordered by name ascending, byte-wise
    /// (`FR-PRIV-016`, `NFR-DET-002`).
    fn from(marking: Restricted<'a>) -> Self {
        let mut properties = marking.properties;
        order::sort_by_name(&mut properties);
        properties
    }
}

impl<'a> TryFrom<Vec<Cow<'a, str>>> for Restricted<'a> {
    type Error = EmptyMarking;

    /// Reads the marking back, refusing the empty array `FR-PRIV-016` forbids.
    ///
    /// # Errors
    ///
    /// Returns [`EmptyMarking`] for an array that names no property. The
    /// alternative — reading it as a complete object — would turn a document
    /// the requirement forbids into one it permits, silently, which is the
    /// repair `FR-RND-020` does not ask for.
    fn try_from(properties: Vec<Cow<'a, str>>) -> Result<Self, Self::Error> {
        Self::new(properties).ok_or(EmptyMarking)
    }
}

impl<'a> Restricted<'a> {
    /// Builds the marking, or [`None`] where there is nothing to report.
    ///
    /// [`None`] is the complete object of `FR-PRIV-007`. A caller that
    /// collects the properties it could not read and passes the result here
    /// therefore gets the right shape without testing for emptiness itself,
    /// and cannot produce the empty array `FR-PRIV-016` forbids.
    #[must_use]
    pub fn new(properties: Vec<Cow<'a, str>>) -> Option<Self> {
        if properties.is_empty() {
            None
        } else {
            Some(Self { properties })
        }
    }

    /// The property names, in the order the marking was built with.
    #[must_use]
    pub fn properties(&self) -> &[Cow<'a, str>] {
        &self.properties
    }
}

/// A copy that borrows nothing, for the render context of `FR-RND-023`.
impl ToStatic for Restricted<'_> {
    type Static = Restricted<'static>;

    fn to_static(&self) -> Self::Static {
        Restricted {
            properties: self.properties.to_static(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::Restricted;
    use std::borrow::Cow;

    #[test]
    fn fr_priv_016_an_empty_marking_cannot_be_built() {
        // FR-PRIV-016: the array is never empty. The constructor is the only
        // way in, so the empty array is not a value that exists.
        assert_eq!(Restricted::new(Vec::new()), None);
    }

    #[test]
    fn fr_priv_016_a_marking_that_names_a_property_is_built_and_keeps_it() {
        // The example of FR-PRIV-016: an unreadable routine body.
        let marking = Restricted::new(vec![Cow::Borrowed("body")])
            .expect("a marking naming one property is not empty");

        assert_eq!(marking.properties(), [Cow::Borrowed("body")]);
    }

    #[test]
    fn fr_priv_016_the_order_the_marking_is_given_is_the_order_it_keeps() {
        // The byte-wise ordering of FR-PRIV-016 belongs to NFR-DET-002 and is
        // applied where every other collection is ordered, not here. This type
        // is a sequence, and it does not silently rearrange what it is given.
        let marking = Restricted::new(vec![Cow::Borrowed("definition"), Cow::Borrowed("body")])
            .expect("two properties are not empty");

        assert_eq!(
            marking.properties(),
            [Cow::Borrowed("definition"), Cow::Borrowed("body")]
        );
    }
}
