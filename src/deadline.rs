//! The phase clock: the four `[core]` deadlines, the overall budget, and the
//! composition of the two.
//!
//! `FR-CONF-005` puts a deadline on every blocking phase and `FR-SEC-022` gives
//! the reason: a connect to a silent address, a `password_command` waiting on a
//! FIFO and a runaway loop in a template all hang the caller with no diagnosis,
//! which is what the "never interactive" invariant exists to prevent.
//!
//! Two bounds of different kinds apply to one phase, and both apply:
//!
//! | Bound | Where it comes from | Fixed by |
//! |---|---|---|
//! | The phase deadline | The `[core]` key `FR-CONF-005` names for that phase, or the built-in default of `FR-CONF-002` | `FR-CONF-004` |
//! | The overall budget | `--timeout`, measured from process start, absent unless supplied | `FR-GLOB-011` |
//!
//! `FR-GLOB-012` composes them rather than letting either replace the other: a
//! phase ends at the first of the two to expire. [`Clock::bound`] is that
//! composition, and it returns **which** of the two bounds it was together with
//! that bound's resolved value, because `FR-ERR-034` obliges the `cause` line of
//! a `65` and of a `69` to name both.
//!
//! The three connection phases are the one exception to "a budget each", and it
//! is stated rather than derived: `FR-CONF-005` gives DNS resolution, TCP
//! connect and the TLS handshake **one** budget of `core.connect_timeout`,
//! measured from the start of the first of them that runs and consumed by them
//! in the order they run. [`Budget`] is that shared instant.
//!
//! `OD-05` places this module at the crate root rather than under `project/`,
//! because `OD-12` gives the construct three users — the runtime inside
//! `mariadb/`, the `password_command` child, and the render — and a budget
//! shared by three modules belongs to none of them. What `project/` owns is the
//! **resolution** of the four values from the file; what this module owns is
//! the clock they are applied through.
//!
//! No deadline crosses a boundary as a bare integer. [`Seconds`] carries the
//! unit `FR-CONF-002` states the four keys in, and [`Bound`] carries a
//! [`Duration`] and the discriminant that says which bound produced it.

use std::fmt;
use std::num::NonZeroU64;
use std::sync::OnceLock;
use std::time::{Duration, Instant};

use crate::error::DeadlineBound;

/// The instant `--timeout` is measured from, per `FR-GLOB-011`.
///
/// It is set once, as early in the process as a library function can be
/// reached, and read by every [`Clock`]. A process that never marks it — a unit
/// test, for instance — reads the instant of the first [`Clock`] built instead,
/// which is the same value for a process whose first clock is its only one.
static PROCESS_START: OnceLock<Instant> = OnceLock::new();

/// Records the instant the overall budget of `FR-GLOB-011` is measured from.
///
/// Called once, by the crate entry point, before anything blocking can run.
/// Calling it twice keeps the first instant, so a second call cannot move the
/// origin of a budget that is already being spent.
pub(crate) fn mark_process_start() {
    let _ = PROCESS_START.set(Instant::now());
}

/// The instant the overall budget is measured from.
fn process_start() -> Instant {
    *PROCESS_START.get_or_init(Instant::now)
}

/// A deadline, in the seconds `FR-CONF-002` states the four `[core]` keys in.
///
/// The type exists so that no bare integer of seconds crosses a boundary: a
/// value read from the file, a value supplied to `--timeout` and a value
/// compared against a [`Duration`] are all this type, and the conversion to a
/// [`Duration`] happens in exactly one place.
///
/// The inner value is a [`NonZeroU64`] because `FR-CONF-002` declares the four
/// keys as **positive** integers and `FR-GLOB-001` declares `--timeout` the
/// same way, so a deadline of zero seconds is refused where it is written
/// rather than where it would expire.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) struct Seconds(NonZeroU64);

impl Seconds {
    /// Names a number of seconds.
    pub(crate) const fn new(value: NonZeroU64) -> Self {
        Self(value)
    }

    /// The number of seconds, as written.
    pub(crate) const fn get(self) -> u64 {
        self.0.get()
    }

    /// The same deadline as a [`Duration`].
    pub(crate) const fn duration(self) -> Duration {
        Duration::from_secs(self.0.get())
    }
}

impl fmt::Display for Seconds {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}s", self.0.get())
    }
}

impl From<Seconds> for Duration {
    fn from(seconds: Seconds) -> Self {
        seconds.duration()
    }
}

/// One of the six blocking phases of `FR-CONF-005`.
///
/// The set is closed by that requirement, and each variant names the `[core]`
/// key its deadline is resolved from. Three of the six share one budget, which
/// is [`Budget`]'s subject rather than this type's.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Phase {
    /// Resolving the host name, bounded by `core.connect_timeout`.
    DnsResolution,
    /// Opening the TCP connection, bounded by `core.connect_timeout`.
    TcpConnect,
    /// Negotiating TLS, bounded by `core.connect_timeout`.
    ///
    /// It is never constructed, and `FR-CONF-005` is why it exists all the
    /// same: that requirement closes the set at six and this is one of them.
    /// The handshake is **inside** the driver's connect call and the driver
    /// affords no method that takes an already-connected socket, so nothing can
    /// bound it, time it or report it apart from the TCP connect it runs
    /// within — `mariadb::connect` says so where the two are run, and `OD-12`
    /// records that the report separates them by the driver's own discriminant
    /// rather than by a clock.
    #[allow(
        dead_code,
        reason = "FR-CONF-005 closes the set of phases at six and the TLS handshake is one of \
                  them; it is never constructed because the driver runs it inside the connect \
                  call, which is what mariadb::connect documents and what OD-12 records"
    )]
    TlsHandshake,
    /// One catalogue query, bounded by `core.query_timeout`.
    CatalogueQuery,
    /// The `password_command` child, bounded by `core.password_timeout`.
    PasswordCommand,
    /// The render, bounded by `core.render_timeout`.
    Render,
}

impl Phase {
    /// The phase's own name, as the report of `FR-GLOB-017` writes it.
    ///
    /// It is the discriminant spelled out rather than the `Debug` of the
    /// variant, because a report line is text a caller reads and `NFR-DET-001`
    /// leaves stderr outside the contract only as far as its wording: a name
    /// that changed with a rename of the variant would change what the reader
    /// sees for no reason the reader can see.
    pub(crate) const fn name(self) -> &'static str {
        match self {
            Self::DnsResolution => "dns resolution",
            Self::TcpConnect => "tcp connect",
            Self::TlsHandshake => "tls handshake",
            Self::CatalogueQuery => "catalogue query",
            Self::PasswordCommand => "password command",
            Self::Render => "render",
        }
    }

    /// The `[core]` key of `FR-CONF-002` this phase takes its deadline from,
    /// per the table of `FR-CONF-005`.
    #[allow(
        dead_code,
        reason = "this is the table of FR-CONF-005 stated as code — which key a phase resolves \
                  from — and a test asserts it against the requirement, row by row; the resolution \
                  itself reads the key space rather than this mapping, so it has no caller in the \
                  library build and gains one only if a diagnostic ever names the key a phase was \
                  bounded by"
    )]
    pub(crate) const fn key(self) -> &'static str {
        match self {
            Self::DnsResolution | Self::TcpConnect | Self::TlsHandshake => "core.connect_timeout",
            Self::CatalogueQuery => "core.query_timeout",
            Self::PasswordCommand => "core.password_timeout",
            Self::Render => "core.render_timeout",
        }
    }

    /// Whether this phase draws on the shared connection budget of
    /// `FR-CONF-005`.
    #[allow(
        dead_code,
        reason = "this is the other half of the FR-CONF-005 table stated as code — which three \
                  phases share one budget — and a test asserts it against the requirement; the \
                  connection opens that budget from `Clock::connection_budget` directly, so the \
                  predicate has no caller in the library build"
    )]
    pub(crate) const fn shares_the_connection_budget(self) -> bool {
        matches!(
            self,
            Self::DnsResolution | Self::TcpConnect | Self::TlsHandshake
        )
    }
}

/// The built-in default of `core.connect_timeout`, in seconds
/// (`FR-CONF-002`).
const DEFAULT_CONNECT: u64 = 10;

/// The built-in default of `core.query_timeout`, in seconds (`FR-CONF-002`).
const DEFAULT_QUERY: u64 = 30;

/// The built-in default of `core.password_timeout`, in seconds
/// (`FR-CONF-002`).
const DEFAULT_PASSWORD: u64 = 5;

/// The built-in default of `core.render_timeout`, in seconds (`FR-CONF-002`).
const DEFAULT_RENDER: u64 = 30;

/// The seconds of a built-in default, which `FR-CONF-002` declares positive.
///
/// The four constants above are literals of this file, so the conversion cannot
/// fail; it is written as a fallback of one rather than as an `expect`, so the
/// module carries no panic at all.
const fn positive(seconds: u64) -> Seconds {
    match NonZeroU64::new(seconds) {
        Some(value) => Seconds::new(value),
        None => Seconds::new(NonZeroU64::MIN),
    }
}

/// The four phase deadlines, each resolved from its `[core]` key or from the
/// built-in default declared for that key (`FR-CONF-004`).
///
/// `--timeout` is deliberately absent: `FR-CONF-004` states that it does not
/// participate in this resolution, and `FR-GLOB-012` composes it with the
/// result instead. That is what [`Clock`] does.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct Deadlines {
    /// `core.connect_timeout`, shared by the three connection phases.
    connect: Seconds,
    /// `core.query_timeout`.
    query: Seconds,
    /// `core.password_timeout`.
    password: Seconds,
    /// `core.render_timeout`.
    render: Seconds,
}

impl Deadlines {
    /// Resolves the four deadlines, taking the built-in default of
    /// `FR-CONF-002` wherever the file declares no value.
    pub(crate) fn resolve(
        connect: Option<Seconds>,
        query: Option<Seconds>,
        password: Option<Seconds>,
        render: Option<Seconds>,
    ) -> Self {
        Self {
            connect: connect.unwrap_or(positive(DEFAULT_CONNECT)),
            query: query.unwrap_or(positive(DEFAULT_QUERY)),
            password: password.unwrap_or(positive(DEFAULT_PASSWORD)),
            render: render.unwrap_or(positive(DEFAULT_RENDER)),
        }
    }

    /// The deadline `phase` takes, per the table of `FR-CONF-005`.
    pub(crate) const fn of(self, phase: Phase) -> Seconds {
        match phase {
            Phase::DnsResolution | Phase::TcpConnect | Phase::TlsHandshake => self.connect,
            Phase::CatalogueQuery => self.query,
            Phase::PasswordCommand => self.password,
            Phase::Render => self.render,
        }
    }
}

impl Default for Deadlines {
    /// The four built-in defaults of `FR-CONF-002`, which is what a project
    /// whose `[core]` section declares no deadline resolves to.
    fn default() -> Self {
        Self::resolve(None, None, None, None)
    }
}

/// The overall wall-clock budget of `FR-GLOB-011`, and the composition rule of
/// `FR-GLOB-012`.
///
/// The budget has no default. Absent `--timeout` the invocation carries none
/// and every phase is bounded by its own deadline alone, which is what
/// `FR-GLOB-011` states in its own words and what the third edition amended it
/// to say.
#[derive(Debug, Clone, Copy)]
pub(crate) struct Clock {
    /// The instant the budget is measured from, per `FR-GLOB-011`.
    started: Instant,
    /// The budget `--timeout` set, or [`None`] where the flag was absent.
    budget: Option<Seconds>,
}

impl Clock {
    /// Starts the clock for this invocation.
    ///
    /// The origin is the instant [`mark_process_start`] recorded, so the budget
    /// is measured from process start rather than from the moment a phase is
    /// about to run.
    pub(crate) fn new(budget: Option<Seconds>) -> Self {
        Self {
            started: process_start(),
            budget,
        }
    }

    /// How long this invocation has been running.
    pub(crate) fn elapsed(&self) -> Duration {
        self.started.elapsed()
    }

    /// What remains of the overall budget, or [`None`] where there is none.
    fn remaining(&self) -> Option<Duration> {
        self.budget
            .map(|budget| budget.duration().saturating_sub(self.elapsed()))
    }

    /// Bounds one phase, composing its own deadline with the overall budget.
    ///
    /// `FR-GLOB-012`: the phase ends at the first of the two to expire. The
    /// returned [`Bound`] carries which of the two that is and its resolved
    /// value, because `FR-ERR-034` obliges the `cause` line to name both, and
    /// the effective duration to wait, which is the smaller of the two.
    ///
    /// The phase deadline wins a tie. The two bounds are equal only where the
    /// budget is untouched and both were written the same, and naming the
    /// phase is the more specific of two equally true answers.
    pub(crate) fn bound(&self, phase: Seconds) -> Bound {
        let own = phase.duration();

        match self.remaining() {
            Some(left) if left < own => Bound {
                bound: DeadlineBound::Overall,
                limit: self.budget.map_or(own, Seconds::duration),
                remaining: left,
            },
            _ => Bound {
                bound: DeadlineBound::Phase,
                limit: own,
                remaining: own,
            },
        }
    }

    /// Opens the budget the three connection phases of `FR-CONF-005` share.
    ///
    /// The instant is taken here, which is the start of the first of the three
    /// that runs, and the three then consume one budget of `connect` in the
    /// order they run rather than one budget each.
    pub(crate) fn connection_budget(&self, connect: Seconds) -> Budget {
        Budget {
            clock: *self,
            opened: Instant::now(),
            limit: connect,
        }
    }
}

/// What bounds one phase, once `FR-GLOB-012` has composed the two.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct Bound {
    /// Which of the two bounds of `FR-GLOB-012` applies.
    bound: DeadlineBound,
    /// The resolved value of that bound — what `FR-ERR-034` obliges the `cause`
    /// line to name beside it.
    limit: Duration,
    /// How long the phase may actually run, which is the smaller of the two.
    remaining: Duration,
}

impl Bound {
    /// Which of the two bounds of `FR-GLOB-012` applies.
    pub(crate) const fn bound(self) -> DeadlineBound {
        self.bound
    }

    /// The resolved value of the bound that applies.
    pub(crate) const fn limit(self) -> Duration {
        self.limit
    }

    /// How long the phase may run before it is abandoned.
    pub(crate) const fn remaining(self) -> Duration {
        self.remaining
    }

    /// Whether the bound has already expired, so the phase must not start.
    pub(crate) const fn expired(self) -> bool {
        self.remaining.is_zero()
    }

    /// A phase bound of `remaining`, for a test that must see one expire
    /// without waiting out a deadline measured in whole seconds.
    #[cfg(test)]
    pub(crate) const fn lasting(remaining: Duration) -> Self {
        Self {
            bound: DeadlineBound::Phase,
            limit: remaining,
            remaining,
        }
    }

    /// A bound of `limit` with nothing left of it.
    ///
    /// The composition of `FR-GLOB-012` produces one whenever the overall
    /// budget is spent before a phase begins, and a phase that refuses to start
    /// is a branch a test has to be able to reach without waiting out a budget
    /// measured in whole seconds.
    #[cfg(test)]
    pub(crate) const fn spent(limit: Seconds) -> Self {
        Self {
            bound: DeadlineBound::Overall,
            limit: limit.duration(),
            remaining: Duration::ZERO,
        }
    }
}

/// The one budget the three connection phases of `FR-CONF-005` share.
///
/// It is an instant rather than a duration, because the three consume it in the
/// order they run: each asks what is left, and the last of them may find
/// nothing. `FR-CONF-005` forbids giving each of the three a budget of the
/// configured value, and this type is how that is prevented rather than
/// remembered.
#[derive(Debug, Clone, Copy)]
pub(crate) struct Budget {
    /// The overall budget the shared one still composes with.
    clock: Clock,
    /// The instant the first of the three phases started.
    opened: Instant,
    /// The resolved value of `core.connect_timeout`.
    limit: Seconds,
}

impl Budget {
    /// What is left of the shared budget, composed with the overall one.
    ///
    /// The result is a [`Bound`] like any other phase's, so a caller that
    /// reports an expiry names the same two facts `FR-ERR-034` obliges
    /// everywhere else.
    pub(crate) fn remaining(&self) -> Bound {
        let own = self.limit.duration().saturating_sub(self.opened.elapsed());
        let composed = self.clock.bound(self.limit);

        if composed.bound() == DeadlineBound::Overall {
            return composed;
        }

        Bound {
            bound: DeadlineBound::Phase,
            limit: self.limit.duration(),
            remaining: own,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{Clock, Deadlines, Phase, Seconds, positive};
    use crate::error::DeadlineBound;
    use std::num::NonZeroU64;
    use std::time::Duration;

    /// A deadline of `seconds`, for a test that knows the value is positive.
    fn seconds(value: u64) -> Seconds {
        Seconds::new(NonZeroU64::new(value).expect("the test writes a positive value"))
    }

    #[test]
    fn fr_conf_002_the_built_in_defaults_are_the_four_the_key_space_declares() {
        // FR-CONF-002: the defaults of the four [core] timeout keys.
        let resolved = Deadlines::default();

        assert_eq!(resolved.of(Phase::TcpConnect).get(), 10);
        assert_eq!(resolved.of(Phase::CatalogueQuery).get(), 30);
        assert_eq!(resolved.of(Phase::PasswordCommand).get(), 5);
        assert_eq!(resolved.of(Phase::Render).get(), 30);
    }

    #[test]
    fn fr_conf_004_every_phase_resolves_from_the_key_the_requirement_names() {
        // FR-CONF-004, FR-CONF-005: the six phases map onto the four keys.
        assert_eq!(Phase::DnsResolution.key(), "core.connect_timeout");
        assert_eq!(Phase::TcpConnect.key(), "core.connect_timeout");
        assert_eq!(Phase::TlsHandshake.key(), "core.connect_timeout");
        assert_eq!(Phase::CatalogueQuery.key(), "core.query_timeout");
        assert_eq!(Phase::PasswordCommand.key(), "core.password_timeout");
        assert_eq!(Phase::Render.key(), "core.render_timeout");
    }

    #[test]
    fn fr_conf_005_the_three_connection_phases_share_one_budget_and_the_other_three_do_not() {
        // FR-CONF-005: the system SHALL NOT give each of the three a budget of
        // the configured value.
        assert!(Phase::DnsResolution.shares_the_connection_budget());
        assert!(Phase::TcpConnect.shares_the_connection_budget());
        assert!(Phase::TlsHandshake.shares_the_connection_budget());
        assert!(!Phase::CatalogueQuery.shares_the_connection_budget());
        assert!(!Phase::PasswordCommand.shares_the_connection_budget());
        assert!(!Phase::Render.shares_the_connection_budget());
    }

    #[test]
    fn fr_conf_004_a_configured_value_displaces_the_built_in_default_for_that_key_alone() {
        // FR-CONF-004: each phase resolves from its own key, or from the
        // built-in default where the key is absent.
        let resolved = Deadlines::resolve(Some(seconds(3)), None, Some(seconds(7)), None);

        assert_eq!(resolved.of(Phase::TcpConnect).get(), 3);
        assert_eq!(resolved.of(Phase::CatalogueQuery).get(), 30);
        assert_eq!(resolved.of(Phase::PasswordCommand).get(), 7);
        assert_eq!(resolved.of(Phase::Render).get(), 30);
    }

    #[test]
    fn fr_glob_011_without_the_flag_a_phase_is_bounded_by_its_own_deadline_alone() {
        // FR-GLOB-011: absent --timeout the invocation carries no overall
        // budget.
        let bound = Clock::new(None).bound(seconds(30));

        assert_eq!(bound.bound(), DeadlineBound::Phase);
        assert_eq!(bound.limit(), Duration::from_secs(30));
        assert_eq!(bound.remaining(), Duration::from_secs(30));
    }

    #[test]
    fn fr_glob_012_the_overall_budget_composes_with_the_phase_deadline_rather_than_replacing_it() {
        // FR-GLOB-012: a phase ends when the first of the two expires. A
        // --timeout of 1 bounds a phase whose own deadline is 30, and a
        // --timeout of 600 does not bound it at all.
        let short = Clock::new(Some(seconds(1))).bound(seconds(30));
        assert_eq!(short.bound(), DeadlineBound::Overall);
        assert_eq!(short.limit(), Duration::from_secs(1));
        assert!(short.remaining() <= Duration::from_secs(1));

        let long = Clock::new(Some(seconds(600))).bound(seconds(30));
        assert_eq!(long.bound(), DeadlineBound::Phase);
        assert_eq!(long.limit(), Duration::from_secs(30));
        assert_eq!(long.remaining(), Duration::from_secs(30));
    }

    #[test]
    fn fr_conf_004_the_flag_is_not_a_layer_of_the_per_phase_resolution() {
        // FR-CONF-004: --timeout SHALL NOT participate in the resolution of a
        // phase deadline. The four [core] keys are reachable whatever the flag
        // says, which is what the third edition's amendment restored.
        let resolved = Deadlines::resolve(None, Some(seconds(2)), None, None);
        let clock = Clock::new(Some(seconds(600)));

        assert_eq!(resolved.of(Phase::CatalogueQuery).get(), 2);
        assert_eq!(
            clock.bound(resolved.of(Phase::CatalogueQuery)).limit(),
            Duration::from_secs(2)
        );
    }

    #[test]
    fn fr_glob_012_a_spent_budget_leaves_a_phase_no_time_at_all() {
        // FR-GLOB-012: the phase ends at the first of the two, and a budget
        // already exhausted is the first.
        let clock = Clock::new(Some(seconds(1)));
        let bound = super::Bound {
            bound: DeadlineBound::Overall,
            limit: Duration::from_secs(1),
            remaining: Duration::ZERO,
        };

        assert!(bound.expired());
        assert!(!clock.bound(seconds(30)).expired());
    }

    #[test]
    fn fr_conf_002_a_built_in_default_is_positive() {
        // FR-CONF-002 declares the four keys positive integers, and the
        // fallback of `positive` is never reached by a literal of this file.
        assert_eq!(positive(10).get(), 10);
        assert_eq!(positive(0).get(), 1);
    }

    #[test]
    fn fr_conf_005_the_shared_connection_budget_is_one_budget_and_not_three() {
        // FR-CONF-005: the three connection phases share one budget of
        // core.connect_timeout, consumed in the order they run.
        let clock = Clock::new(None);
        let budget = clock.connection_budget(seconds(10));

        let first = budget.remaining();
        let second = budget.remaining();

        assert_eq!(first.bound(), DeadlineBound::Phase);
        assert_eq!(first.limit(), Duration::from_secs(10));
        assert!(
            second.remaining() <= first.remaining(),
            "the budget is consumed by the phases that draw on it"
        );
    }

    #[test]
    fn fr_conf_004_a_deadline_carries_its_unit() {
        // The type is the unit: no bare integer of seconds crosses a boundary.
        assert_eq!(seconds(5).to_string(), "5s");
        assert_eq!(Duration::from(seconds(5)), Duration::from_secs(5));
    }
}
