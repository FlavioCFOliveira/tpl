//! The three render bounds other than the deadline: render fuel, the render
//! output limit and the render memory limit (`FR-RND-036` … `FR-RND-039`,
//! `FR-CONF-045`, `FR-SEC-025`).
//!
//! Fuel and output are counts, not times, so the same template over the same
//! context stops at the same point on every run; the memory limit is observed
//! periodically, so where it stops depends on when it is observed
//! (`FR-RND-039`). Each is resolved from one `[core]`
//! key of `.tpl/.cfg`, or from the built-in default of `FR-CONF-002` where the
//! key is absent; no flag and no environment variable sets either, and
//! `--timeout` does not participate.
//!
//! | Bound | Key | Default | Range | Counted by |
//! |---|---|---|---|---|
//! | Render fuel | `core.render_fuel` | 100 000 000 | 1 … 10^12 | The engine, per evaluation step |
//! | Render output limit | `core.render_output_limit` | 67 108 864 | 1 … 2^40 | [`super::Environment::render`], per byte produced |
//! | Render memory limit | `core.render_memory_limit` | 134 217 728 | 2^23 … 2^40 | The allocator of `ADR-011`, read by the render's watchdog every 10 ms |
//!
//! **None admits `0` nor any value meaning "no bound".** `FR-CONF-045`
//! gives the reason: a key that can remove a bound reopens the surface the
//! bound closes. The types below therefore cannot hold a value outside the
//! range, and every constructor checks it.

use std::io::{self, Write};

use crate::error::{Error, ensure_invariant};

/// The budget of evaluation steps one render may execute (`FR-RND-036`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct RenderFuel(u64);

impl RenderFuel {
    /// The built-in default of `core.render_fuel` (`FR-CONF-002`).
    pub(crate) const DEFAULT: Self = Self(100_000_000);

    /// The largest value `core.render_fuel` admits (`FR-CONF-002`).
    pub(crate) const MAX: u64 = 1_000_000_000_000;

    /// The fuel `steps` spells, or [`None`] where it is outside `1..=MAX`.
    pub(crate) const fn new(steps: u64) -> Option<Self> {
        if steps >= 1 && steps <= Self::MAX {
            Some(Self(steps))
        } else {
            None
        }
    }

    /// The number of evaluation steps.
    pub(crate) const fn get(self) -> u64 {
        self.0
    }
}

/// The number of bytes one render may produce (`FR-RND-037`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct RenderOutputLimit(u64);

impl RenderOutputLimit {
    /// The built-in default of `core.render_output_limit`: 64 MiB
    /// (`FR-CONF-002`).
    ///
    /// Half the default render memory limit, because the output is held in
    /// memory until the render ends and so counts toward it: at or above it, a
    /// render writing without end would be reported under the memory limit
    /// rather than this one (`FR-CONF-045`).
    pub(crate) const DEFAULT: Self = Self(67_108_864);

    /// The largest value `core.render_output_limit` admits: 1 TiB
    /// (`FR-CONF-002`).
    pub(crate) const MAX: u64 = 1_099_511_627_776;

    /// The limit `bytes` spells, or [`None`] where it is outside `1..=MAX`.
    pub(crate) const fn new(bytes: u64) -> Option<Self> {
        if bytes >= 1 && bytes <= Self::MAX {
            Some(Self(bytes))
        } else {
            None
        }
    }

    /// The number of bytes.
    pub(crate) const fn get(self) -> u64 {
        self.0
    }
}

/// The heap bytes the process may hold while one render runs (`FR-RND-039`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct RenderMemoryLimit(u64);

impl RenderMemoryLimit {
    /// The built-in default of `core.render_memory_limit`: 128 MiB
    /// (`FR-CONF-002`).
    pub(crate) const DEFAULT: Self = Self(134_217_728);

    /// The smallest value `core.render_memory_limit` admits: 8 MiB, because
    /// the process holds heap before the template runs (`FR-CONF-045`).
    pub(crate) const MIN: u64 = 8_388_608;

    /// The largest value `core.render_memory_limit` admits: 1 TiB
    /// (`FR-CONF-002`).
    pub(crate) const MAX: u64 = 1_099_511_627_776;

    /// The limit `bytes` spells, or [`None`] where it is outside `MIN..=MAX`.
    pub(crate) const fn new(bytes: u64) -> Option<Self> {
        if bytes >= Self::MIN && bytes <= Self::MAX {
            Some(Self(bytes))
        } else {
            None
        }
    }

    /// The number of bytes.
    pub(crate) const fn get(self) -> u64 {
        self.0
    }

    /// Whether `held` heap bytes cross the limit.
    pub(crate) fn crossed_by(self, held: usize) -> bool {
        u64::try_from(held).unwrap_or(u64::MAX) > self.0
    }
}

/// The three render bounds of one invocation, resolved (`FR-CONF-045`).
///
/// A value, not a budget: every render made under it starts with the whole of
/// each, which is what `FR-RND-038` requires of a render abandoned under
/// `FR-CACHE-039` and of the one that follows it. The engine's fuel is
/// consumed per render, the output count belongs to one render's writer, and
/// the memory limit is compared with the heap the process holds at each
/// observation — which the abandoned render no longer holds, because its
/// values are dropped before the following render starts.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct RenderBounds {
    /// The render fuel of `FR-RND-036`.
    pub(crate) fuel: RenderFuel,
    /// The render output limit of `FR-RND-037`.
    pub(crate) output_limit: RenderOutputLimit,
    /// The render memory limit of `FR-RND-039`.
    pub(crate) memory_limit: RenderMemoryLimit,
}

impl RenderBounds {
    /// The bounds `.tpl/.cfg` declares, each falling back to its built-in
    /// default where the key is absent (`FR-CONF-045`).
    pub(crate) fn resolve(
        fuel: Option<RenderFuel>,
        output_limit: Option<RenderOutputLimit>,
        memory_limit: Option<RenderMemoryLimit>,
    ) -> Self {
        Self {
            fuel: fuel.unwrap_or(RenderFuel::DEFAULT),
            output_limit: output_limit.unwrap_or(RenderOutputLimit::DEFAULT),
            memory_limit: memory_limit.unwrap_or(RenderMemoryLimit::DEFAULT),
        }
    }
}

impl Default for RenderBounds {
    /// The three built-in defaults of `FR-CONF-002`.
    fn default() -> Self {
        Self::resolve(None, None, None)
    }
}

/// The writer one render produces its text into, counting every byte as it
/// is written (`FR-RND-037`).
///
/// A write that would carry the count past the limit is refused whole and
/// leaves the text as it was, so what is held is never longer than the limit;
/// the engine reports the refusal as a failed write and stops, and
/// [`Limited::exceeded`] tells the caller that the limit, and not the engine,
/// ended the render. Below the limit a write costs one comparison and the
/// append.
#[derive(Debug)]
pub(super) struct Limited {
    /// The text produced so far.
    produced: Vec<u8>,
    /// The render output limit, in bytes.
    limit: u64,
    /// Whether a write was refused for passing the limit.
    exceeded: bool,
}

impl Limited {
    /// An empty writer held to `limit`.
    pub(super) const fn new(limit: RenderOutputLimit) -> Self {
        Self {
            produced: Vec::new(),
            limit: limit.get(),
            exceeded: false,
        }
    }

    /// Whether a write was refused for passing the limit.
    pub(super) const fn exceeded(&self) -> bool {
        self.exceeded
    }

    /// The text produced, as a string.
    ///
    /// # Errors
    ///
    /// Returns [`Error::InternalInvariant`] where the bytes are not UTF-8,
    /// which no render can produce: the engine writes nothing but string
    /// slices, and the writer appends each one whole or refuses it whole.
    pub(super) fn into_text(self) -> Result<String, Error> {
        String::from_utf8(self.produced).map_err(|_| {
            ensure_invariant(false, "the engine writes only whole UTF-8 string slices")
                .expect_err("ensure_invariant(false, _) returns the condition it was given")
        })
    }
}

impl Write for Limited {
    fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
        // `usize` is at most 64 bits on every target of NFR-PERF-018, so both
        // conversions are exact; saturation keeps the comparison total anyway.
        let held = u64::try_from(self.produced.len()).unwrap_or(u64::MAX);
        let added = u64::try_from(bytes.len()).unwrap_or(u64::MAX);

        if held.saturating_add(added) > self.limit {
            self.exceeded = true;
            return Err(io::Error::other("the render output limit was reached"));
        }

        self.produced.extend_from_slice(bytes);
        Ok(bytes.len())
    }

    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::{Limited, RenderBounds, RenderFuel, RenderMemoryLimit, RenderOutputLimit};
    use std::io::Write as _;

    #[test]
    fn fr_rnd_037_the_writer_holds_up_to_the_limit_and_refuses_the_write_past_it() {
        let mut writer = Limited::new(RenderOutputLimit::new(5).expect("in range"));

        writer.write_all(b"abc").expect("under the limit");
        writer.write_all(b"de").expect("exactly at the limit");
        assert!(!writer.exceeded());

        assert!(writer.write_all(b"f").is_err(), "one byte past the limit");
        assert!(writer.exceeded());
        assert_eq!(writer.into_text().expect("UTF-8"), "abcde");
    }

    #[test]
    fn fr_conf_002_the_defaults_are_the_ones_the_table_declares() {
        let bounds = RenderBounds::default();

        assert_eq!(bounds.fuel.get(), 100_000_000);
        assert_eq!(bounds.output_limit.get(), 67_108_864);
        assert_eq!(bounds.memory_limit.get(), 134_217_728);
    }

    #[test]
    fn fr_conf_045_each_bound_admits_exactly_its_range_and_no_value_meaning_none() {
        assert_eq!(RenderFuel::new(0), None);
        assert_eq!(RenderFuel::new(1).map(RenderFuel::get), Some(1));
        assert_eq!(
            RenderFuel::new(1_000_000_000_000).map(RenderFuel::get),
            Some(1_000_000_000_000)
        );
        assert_eq!(RenderFuel::new(1_000_000_000_001), None);
        assert_eq!(RenderFuel::new(u64::MAX), None);

        assert_eq!(RenderOutputLimit::new(0), None);
        assert_eq!(
            RenderOutputLimit::new(1).map(RenderOutputLimit::get),
            Some(1)
        );
        assert_eq!(
            RenderOutputLimit::new(1_099_511_627_776).map(RenderOutputLimit::get),
            Some(1_099_511_627_776)
        );
        assert_eq!(RenderOutputLimit::new(1_099_511_627_777), None);

        assert_eq!(RenderMemoryLimit::new(0), None);
        assert_eq!(RenderMemoryLimit::new(8_388_607), None);
        assert_eq!(
            RenderMemoryLimit::new(8_388_608).map(RenderMemoryLimit::get),
            Some(8_388_608)
        );
        assert_eq!(
            RenderMemoryLimit::new(1_099_511_627_776).map(RenderMemoryLimit::get),
            Some(1_099_511_627_776)
        );
        assert_eq!(RenderMemoryLimit::new(1_099_511_627_777), None);
    }

    #[test]
    fn fr_rnd_039_the_limit_is_crossed_by_a_count_above_it_and_not_by_one_at_it() {
        let limit = RenderMemoryLimit::new(8_388_608).expect("in range");

        assert!(!limit.crossed_by(8_388_608));
        assert!(limit.crossed_by(8_388_609));
    }

    #[test]
    fn fr_conf_045_a_declared_bound_is_taken_and_an_absent_one_is_the_default() {
        let bounds =
            RenderBounds::resolve(RenderFuel::new(7), None, RenderMemoryLimit::new(9_000_000));

        assert_eq!(bounds.fuel.get(), 7);
        assert_eq!(bounds.output_limit, RenderOutputLimit::DEFAULT);
        assert_eq!(bounds.memory_limit.get(), 9_000_000);
    }
}
