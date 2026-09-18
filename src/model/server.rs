//! The server the model was read from (`FR-CTX-031` … `FR-CTX-034`).
//!
//! This is one of the two objects of the model that is **not a catalogue row**.
//! `BR-CTX-006` says why it could be fixed before the catalogue field lists
//! were observed: it comes from the version probe of `FR-SRV-002`, so its shape
//! asserts nothing about what a catalogue table returns.
//!
//! | Requirement | What this module does with it |
//! |---|---|
//! | `FR-CTX-031` — exactly three keys | [`Server`] carries three fields and no fourth |
//! | `FR-SRV-040` — `series` is `<major>.<minor>` and nothing else | [`series_of`] reads those two components and stops; the build suffix reaches no field but `version` |
//! | `FR-CTX-032` — `series` is a field of its own | It is stored, not recomputed, so a template never parses `version` |
//! | `FR-CTX-034` — `standing` takes exactly two values | [`Standing`] is a closed enum of two, and the field is not an [`Option`] |
//! | `FR-CTX-033` — a supplied document's `series` is not validated | [`Server::stated`] carries what the document said; [`Server::probed`] derives |
//!
//! *The derived [`Deserialize`] **is** the check `FR-CTX-033` admits, and it is
//! derived so that no second check can be written beside it.* That requirement
//! asks for three things and forbids two. Three fields with no default make the
//! three keys required; two [`Cow<'a, str>`](Cow) fields make `version` and
//! `series` strings; and [`Standing`] being a closed enumeration is what refuses
//! a fourth value of `standing`. The two prohibitions — no check of `series`
//! against the window of `FR-SRV-015`, and none of `standing` against `series` —
//! hold because a derived implementation has nowhere to put one. A document
//! taken from a server the window has since left therefore still renders, which
//! is the accepted cost `FR-CTX-033` records.
//!
//! *The two constructors are two paths and not a convenience.* A server read
//! derives `series` from the string the probe returned, and a `--context`
//! document supplies all three keys already written down — `FR-CTX-033` bars
//! the system from checking the third against the first, because the document
//! is a record of a read that already happened rather than a server that can
//! be questioned.

use std::borrow::Cow;

use serde::{Deserialize, Serialize};

/// The separator between the components of a version string.
const COMPONENT: char = '.';

/// The standing of the server relative to the supported window (`FR-CTX-034`).
///
/// Two values and no third, because a series below the window never reaches a
/// document: `FR-SRV-020` refuses it before the catalogue is read. The type is
/// `#[non_exhaustive]` for the reason `FR-CTX-034` gives for choosing an
/// enumeration over a boolean — `FR-OUT-014` lets an enumerated field gain a
/// value, and gaining one is not a breaking change.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
#[non_exhaustive]
pub enum Standing {
    /// The series is one of `FR-SRV-015`.
    Supported,

    /// The series is newer than every series of `FR-SRV-015`, and the document
    /// was produced under `FR-SRV-031`.
    NewerThanSupported,
}

impl Standing {
    /// The value as `FR-CTX-034` spells it.
    ///
    /// The two spellings are contract surface: they are what the document
    /// carries and what a caller parsing it branches on.
    #[must_use]
    pub const fn name(self) -> &'static str {
        match self {
            Self::Supported => "supported",
            Self::NewerThanSupported => "newer_than_supported",
        }
    }
}

/// The series of a version string — `<major>.<minor>`, and nothing else
/// (`FR-SRV-040`).
///
/// The argument is the string the probe of `FR-SRV-002` returns, whose observed
/// form is `<major>.<minor>.<patch>-MariaDB` optionally followed by
/// `-<suffix>`. The answer borrows the leading `<major>.<minor>` of that
/// string, so nothing is allocated and nothing is reconstructed: the bytes
/// returned are the bytes the server sent.
///
/// [`None`] is returned for a string outside that form. Refusing a server on
/// that ground is not this function's business — `FR-SRV-041` tests the product
/// marker and `FR-SRV-003` refuses — and this answers only whether a series can
/// be read.
///
/// *The caller obligation `FR-SRV-040` states in the words `and from nothing
/// else`.* A server also announces a version when the connection opens, and on
/// `10.11` that announcement carries a `5.5.5-` prefix the probe's answer does
/// not. This function is defined over the **probe's** answer; given the
/// greeting it would read `5.5`, and `FR-SRV-020` would refuse a supported
/// server as older than the window.
#[must_use]
pub fn series_of(version: &str) -> Option<&str> {
    let mut components = version.split(COMPONENT);

    let major = components.next()?;
    let minor = components.next()?;

    // The patch component must exist: `11.4` on its own is not the form
    // `FR-SRV-040` fixes, and reading a series from it would be reading a
    // string this system has never observed a server return.
    components.next()?;

    if !numeric(major) || !numeric(minor) {
        return None;
    }

    Some(&version[..major.len() + COMPONENT.len_utf8() + minor.len()])
}

/// Whether a component is a non-empty run of ASCII digits.
fn numeric(component: &str) -> bool {
    !component.is_empty() && component.bytes().all(|byte| byte.is_ascii_digit())
}

/// The server a model was read from (`FR-CTX-031`).
///
/// The fields are private and the two constructors are the only way in, which
/// is what makes `FR-SRV-040` structural rather than conventional: on the
/// server path `series` cannot be populated from anything but the leading two
/// components of `version`, and the build suffix has no route to any field
/// other than `version`, which carries the whole string unaltered.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct Server<'a> {
    /// The string the probe of `FR-SRV-002` returned, unaltered.
    #[serde(borrow)]
    version: Cow<'a, str>,

    /// The series identifier of `FR-SRV-040`.
    #[serde(borrow)]
    series: Cow<'a, str>,

    /// The standing of `FR-CTX-034`, always present.
    standing: Standing,
}

impl<'a> Server<'a> {
    /// Builds the object from a server read, deriving `series` per
    /// `FR-SRV-040`.
    ///
    /// [`None`] is returned where the version string is outside the form that
    /// requirement fixes, which is the one case a series cannot be read from.
    ///
    /// A borrowed version yields a borrowed series, because the series is a
    /// slice of it; an owned version costs one short allocation, which is what
    /// keeps the value free of the source it was read from.
    #[must_use]
    pub fn probed(version: Cow<'a, str>, standing: Standing) -> Option<Self> {
        match version {
            Cow::Borrowed(probed) => Some(Self {
                series: Cow::Borrowed(series_of(probed)?),
                version: Cow::Borrowed(probed),
                standing,
            }),
            Cow::Owned(probed) => Some(Self {
                series: Cow::Owned(series_of(&probed)?.to_owned()),
                version: Cow::Owned(probed),
                standing,
            }),
        }
    }

    /// Builds the object from a document that already states all three keys.
    ///
    /// `FR-CTX-033` requires only that the three keys are present and are
    /// strings, and bars the system from checking `series` against `version` or
    /// `standing` against `series`: the document is a record of a read that
    /// already happened, and validating it against the window of `FR-SRV-015`
    /// would make every committed dump expire on a calendar date.
    #[must_use]
    pub const fn stated(version: Cow<'a, str>, series: Cow<'a, str>, standing: Standing) -> Self {
        Self {
            version,
            series,
            standing,
        }
    }

    /// The version string, exactly as the probe returned it, suffix and all.
    #[must_use]
    pub fn version(&self) -> &str {
        &self.version
    }

    /// The series identifier — `<major>.<minor>`, per `FR-SRV-040`.
    #[must_use]
    pub fn series(&self) -> &str {
        &self.series
    }

    /// The server's standing relative to the supported window (`FR-CTX-034`).
    #[must_use]
    pub const fn standing(&self) -> Standing {
        self.standing
    }
}

#[cfg(test)]
mod tests {
    use super::{Server, Standing, series_of};
    use std::borrow::Cow;

    /// The version `11.4` returned to the observation pass of `FR-SRV-040`.
    const PROBED: &str = "11.4.13-MariaDB-ubu2404";

    #[test]
    fn the_series_is_the_first_two_components_and_the_suffix_reaches_nothing() {
        // FR-SRV-040: `series` is `<major>.<minor>` and is derived from
        // nothing else. The four strings are the ones the four fixture servers
        // returned verbatim.
        assert_eq!(series_of(PROBED), Some("11.4"));
        assert_eq!(series_of("10.11.19-MariaDB-ubu2204"), Some("10.11"));
        assert_eq!(series_of("11.8.9-MariaDB-ubu2404"), Some("11.8"));
        assert_eq!(series_of("12.3.3-MariaDB-ubu2404"), Some("12.3"));
    }

    #[test]
    fn changing_only_the_suffix_changes_nothing_the_series_is_read_from() {
        // The suffix is a property of the build: three of the four fixture
        // servers carry `ubu2404` and one carries `ubu2204`, on the same
        // series. A build that changed it must move no field but `version`.
        let ubuntu = Server::probed(Cow::Borrowed(PROBED), Standing::Supported)
            .expect("the observed form yields a series");
        let other = Server::probed(
            Cow::Borrowed("11.4.13-MariaDB-something-else"),
            Standing::Supported,
        )
        .expect("the observed form yields a series");

        assert_eq!(ubuntu.series(), other.series());
        assert_eq!(ubuntu.series(), "11.4");
        assert_ne!(ubuntu.version(), other.version());
    }

    #[test]
    fn the_version_is_carried_unaltered_including_the_suffix() {
        // FR-CTX-031: `version` is the string the probe returns, unaltered.
        let server = Server::probed(Cow::Borrowed(PROBED), Standing::Supported)
            .expect("the observed form yields a series");

        assert_eq!(server.version(), PROBED);
    }

    #[test]
    fn the_series_borrows_the_version_rather_than_rebuilding_it() {
        // The bytes reported are the bytes the server sent: the series is a
        // slice of the version and is not reconstructed from its components.
        let series = series_of(PROBED).expect("the observed form yields a series");

        assert!(std::ptr::eq(series.as_ptr(), PROBED.as_ptr()));
    }

    #[test]
    fn a_string_outside_the_observed_form_yields_no_series() {
        // FR-SRV-040 fixes one form. Refusing a server is FR-SRV-041's and
        // FR-SRV-003's; this reports only that no series can be read.
        assert_eq!(series_of(""), None);
        assert_eq!(series_of("11"), None);
        assert_eq!(series_of("11.4"), None);
        assert_eq!(series_of("11.x.13-MariaDB"), None);
        assert_eq!(series_of("..-MariaDB"), None);
        assert_eq!(
            Server::probed(Cow::Borrowed("not a version"), Standing::Supported),
            None
        );
    }

    #[test]
    fn the_greeting_is_not_the_probes_string_and_this_reads_the_probes_string() {
        // Difference 13 of FR-SRV-038: on `10.11` the connection greeting
        // carries a `5.5.5-` prefix the probe's answer does not. Fed the
        // greeting, the derivation reads `5.5` — which is exactly why
        // FR-SRV-040 says *and from nothing else*, and why the caller must
        // pass the probe's answer. The trap is pinned here rather than left to
        // be met by a supported server being refused as older than the window.
        assert_eq!(series_of("5.5.5-10.11.19-MariaDB-ubu2204"), Some("5.5"));
    }

    #[test]
    fn an_owned_version_yields_an_owned_series_and_borrows_nothing() {
        // The value must survive the buffer it was read from: a model built
        // from owned bytes carries owned strings and no borrow of the source.
        let server = Server::probed(Cow::Owned(PROBED.to_owned()), Standing::Supported)
            .expect("the observed form yields a series");

        assert_eq!(server.series(), "11.4");
        assert_eq!(server.version(), PROBED);
    }

    #[test]
    fn a_stated_server_carries_what_the_document_said_and_is_not_checked() {
        // FR-CTX-033: on the `--context` path the system does not validate
        // `series` against `version`, nor `standing` against `series`. A
        // document that disagrees with itself is carried as written.
        let server = Server::stated(
            Cow::Borrowed("11.4.13-MariaDB-ubu2404"),
            Cow::Borrowed("10.11"),
            Standing::NewerThanSupported,
        );

        assert_eq!(server.version(), "11.4.13-MariaDB-ubu2404");
        assert_eq!(server.series(), "10.11");
        assert_eq!(server.standing(), Standing::NewerThanSupported);
    }

    #[test]
    fn the_standing_carries_the_two_spellings_of_the_requirement() {
        assert_eq!(Standing::Supported.name(), "supported");
        assert_eq!(Standing::NewerThanSupported.name(), "newer_than_supported");
    }
}
