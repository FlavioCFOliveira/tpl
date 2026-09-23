//! The one connection an invocation opens, and everything settled on it before
//! a catalogue row is read.
//!
//! This module is the only part of `tpl` that speaks to a server. `FR-SRV-007`
//! admits no statement outside the closed list of `FR-SRV-006` and no external
//! process at all, and `OD-05` gives that rule a home: nothing above this
//! module is asynchronous, nothing above it holds a connection, and nothing
//! above it composes a statement.
//!
//! | Submodule | Subject | Forced by |
//! |---|---|---|
//! | [`connect`] | What the driver is told, the name resolution, and the one connection | `NFR-PERF-004`, `FR-CONF-037`, `FR-CONF-038`, `OD-12` |
//! | [`session`] | The three connection-start statements, and the verdicts they settle | `FR-SRV-006`, `FR-SRV-008` … `FR-SRV-010`, `FR-SRV-002`, `FR-SRV-020` |
//! | [`window`] | The supported version window, as one ordered table | `FR-SRV-015`, `FR-SRV-021`, `FR-SRV-031` |
//! | [`fault`] | Where the driver's error stops and a condition of `FR-ERR-001` begins | `FR-GLOB-018`, `OD-06`, `FR-ERR-034` |
//! | [`catalogue`] | The fixed repertoire of catalogue queries, and the model their rows fold into | `FR-SRV-006`, `FR-SRV-037`, `NFR-PERF-001`, `NFR-PERF-002`, `FR-CAT-001` … `FR-CAT-053` |
//!
//! **The runtime is built here and nowhere else.** `ADR-005` makes the process
//! synchronous and scopes the asynchronous runtime to this module: it is a
//! current-thread runtime, it comes into existence only when a command
//! actually reaches [`open`], and `block_on` at this boundary is the one
//! synchronisation point in the crate. That is what lets `NFR-PERF-005` be
//! satisfied by **observation** rather than by argument — a command that never
//! reaches this module starts no runtime, opens no socket, and has nothing for
//! `NFR-PERF-007`'s instruments to find.
//!
//! **Nothing of a read outlives it.** [`Session::close`] ends the session with
//! the protocol's quit, closes the socket, and then shuts the runtime down —
//! explicitly, in that order, and waiting for every thread the runtime
//! started. `FR-RND-040` requires that no connection be open and no runtime of
//! the driver be alive while a template is evaluated, and [`quiescent`] is how
//! the render asks: each connection and each runtime is counted from the
//! moment it exists to the moment it is gone, and the render refuses to start
//! while either count is above zero.
//!
//! **The session guarantee cannot be stepped around.** [`connect::open`] is
//! visible to this module alone, so the only route from the crate to an open
//! connection is [`open`], and [`open`] issues the read-only statement of
//! `FR-SRV-008`, its read-back under `FR-SRV-009` and the version probe of
//! `FR-SRV-002` before it returns anything. `FR-SRV-011` admits no flag,
//! configuration key or environment condition that disables any of it, and
//! there is none to find: the three statements are on the one path, behind no
//! condition.
//!
//! **The catalogue read follows all three, and never precedes one.**
//! `FR-SRV-042` fixes that order and [`catalogue::read`] takes a [`Session`],
//! which is the value [`open`] produces once the three have answered — so the
//! series every series-dependent treatment is selected from is resolved before
//! the first `SELECT` is composed, as `FR-SRV-022` requires, rather than
//! discovered from a read that failed.
//!
//! **A read a privilege truncated says so.** The three shapes a shortfall
//! reaches a reader in — the empty string, SQL `NULL` and zero rows, per
//! `FR-PRIV-018` — are found by the `completeness` submodule of [`catalogue`] as the
//! rows are mapped, and the object that is short carries the `restricted`
//! marking of `FR-PRIV-016` while the object that is whole carries none.
//!
//! **What this module still does not do is answer with the `77` of
//! `FR-PRIV-003`.** The verdict exists and is a value; the exit code is emitted
//! where every other one is, by the binary. The callers exist:
//! `crate::cli::schema::named` takes that verdict once the object it was given
//! has been found, and every read that names one object goes through it — the
//! three subcommands of `tpl schema` that name one, `tpl cache load`, and the
//! object flags of `tpl render`.

pub(crate) mod catalogue;
pub(crate) mod connect;
pub(crate) mod fault;
pub(crate) mod session;
pub(crate) mod window;

use std::cell::Cell;
use std::marker::PhantomData;
use std::ops::{Deref, DerefMut};
use std::panic::Location;
use std::thread::LocalKey;

use sqlx::Connection as _;
use sqlx::mysql::MySqlConnection;
use tokio::runtime::{Builder, Runtime};

use crate::deadline::Clock;
use crate::error::Error;
use crate::model::server::Server;

pub(crate) use connect::Target;

thread_local! {
    /// The connections to a server this thread holds open.
    static CONNECTIONS: Cell<usize> = const { Cell::new(0) };

    /// The driver runtimes this thread has built and not yet shut down.
    static RUNTIMES: Cell<usize> = const { Cell::new(0) };
}

/// Whether this thread holds no open connection and no driver runtime
/// (`FR-RND-040`).
///
/// The counts are **per thread**, and that is exact rather than approximate:
/// a runtime of `ADR-005` is a current-thread runtime built by [`open`] on the
/// thread that calls it, every statement is entered through `block_on` on that
/// thread, and the values that hold a connection or a runtime cannot leave it
/// — [`Counted`] is not [`Send`]. A process-wide count would be the same in
/// the binary, which runs one command on one thread, and wrong in the test
/// suite, where another test's connection on another thread is not this
/// invocation's.
pub(crate) fn quiescent() -> bool {
    CONNECTIONS.with(Cell::get) == 0 && RUNTIMES.with(Cell::get) == 0
}

/// One unit of a thread's count, held for as long as what it counts exists.
#[derive(Debug)]
struct Alive {
    /// The count this unit belongs to.
    count: &'static LocalKey<Cell<usize>>,
    /// Keeps the unit, and whatever holds it, on the thread it was counted on.
    here: PhantomData<*const ()>,
}

impl Alive {
    /// Adds one to `count`.
    fn of(count: &'static LocalKey<Cell<usize>>) -> Self {
        count.with(|held| held.set(held.get().saturating_add(1)));

        Self {
            count,
            here: PhantomData,
        }
    }
}

impl Drop for Alive {
    fn drop(&mut self) {
        self.count
            .with(|held| held.set(held.get().saturating_sub(1)));
    }
}

/// A connection or a runtime, counted for as long as it exists.
///
/// The value is declared before its unit of count, so it is dropped first:
/// the count falls only once the socket is closed or the runtime's threads
/// have stopped, never while either is still going.
#[derive(Debug)]
struct Counted<T> {
    /// What is counted.
    value: T,
    /// Its unit of count.
    alive: Alive,
}

impl<T> Counted<T> {
    /// `value`, counted in `count`.
    fn new(value: T, count: &'static LocalKey<Cell<usize>>) -> Self {
        Self {
            value,
            alive: Alive::of(count),
        }
    }
}

impl<T> Deref for Counted<T> {
    type Target = T;

    fn deref(&self) -> &T {
        &self.value
    }
}

impl<T> DerefMut for Counted<T> {
    fn deref_mut(&mut self) -> &mut T {
        &mut self.value
    }
}

/// An open connection whose session has been settled.
///
/// A value of this type is a connection that has passed every gate
/// `FR-SRV-002` puts before a catalogue read: the session is read only and the
/// setting has been read back, the server is MariaDB, and its series is one
/// `FR-SRV-015` admits or newer than all of them. There is no constructor that
/// skips any of it.
#[derive(Debug)]
pub(crate) struct Session {
    /// The connection itself.
    ///
    /// Declared before the runtime **on purpose**: a struct's fields are
    /// dropped in declaration order, so the socket is released while the
    /// runtime that registered it is still alive.
    ///
    /// It is an [`Option`] because [`Session::close`] takes it out, and the
    /// driver's own close consumes the connection.
    connection: Option<Counted<MySqlConnection>>,

    /// The runtime of `ADR-005`, which every statement on this connection is
    /// entered through.
    runtime: Counted<Runtime>,

    /// The `server` object of `FR-CTX-031`, as `FR-SRV-028` carries it into
    /// the model.
    server: Server<'static>,

    /// The database entry this connection was opened from.
    ///
    /// Nothing reads it yet: every condition this module raises names the
    /// entry from the [`Target`] it was given, which carries it borrowed, and
    /// the one caller that holds an open session names it from the settings it
    /// resolved.
    #[allow(
        dead_code,
        reason = "every condition names the entry from the Target it was given, and the caller \
                  that holds a session names it from the settings it resolved"
    )]
    entry: String,
}

impl Session {
    /// The server this connection reached (`FR-SRV-028`, `FR-CTX-031`).
    pub(crate) const fn server(&self) -> &Server<'static> {
        &self.server
    }

    /// The database entry this connection was opened from.
    #[allow(
        dead_code,
        reason = "every condition names the entry from the Target it was given, and the caller \
                  that holds a session names it from the settings it resolved"
    )]
    pub(crate) fn entry(&self) -> &str {
        &self.entry
    }

    /// Closes the connection and shuts the runtime down.
    ///
    /// The root coordination document closes the connection as soon as the
    /// read ends, and `NFR-PERF-004` makes the count of them observable from
    /// the server, so the close is a polite one: the driver sends the
    /// protocol's own quit and shuts the socket down. `FR-RND-040` then
    /// requires every runtime of the driver to be gone before a template is
    /// evaluated, so the runtime is shut down here, **explicitly and after the
    /// socket**, rather than whenever the value happens to go out of scope.
    ///
    /// The runtime is dropped rather than shut down with a timeout. Dropping
    /// it waits for every thread its blocking pool started; a timeout would
    /// return with such a thread possibly still running, which is a runtime
    /// alive while the template evaluates.
    ///
    /// A close that fails is not a condition. Nothing is left to report by the
    /// time it runs — the answer is already produced — and what `FR-RND-040`
    /// and `NFR-PERF-004` require is that the session be gone, which it is
    /// either way: the socket is closed when the connection the failed close
    /// was handed is dropped.
    pub(crate) fn close(self) {
        let Self {
            connection,
            runtime,
            ..
        } = self;

        if let Some(Counted { value, alive }) = connection {
            let _ = runtime.block_on(value.close());
            drop(alive);
        }

        let Counted { value, alive } = runtime;
        drop(value);
        drop(alive);
    }
}

/// The current-thread runtime of `ADR-005`.
///
/// It is built here rather than over the entry point, so that a command which
/// never reaches this module never pays for one: `NFR-PERF-005` requires
/// `tpl init`, every form of `help` and every form of `version` to do no work
/// of this kind, and `NFR-PERF-007` requires that to be observable from
/// outside the process rather than argued from the source.
///
/// # Errors
///
/// Returns [`Error::InternalInvariant`] where the runtime cannot be built. It
/// is the guard of `FR-ERR-030` rather than a condition a caller can act on:
/// the process could not obtain the machinery it needs to speak to a server at
/// all, which is a defect in `tpl` or an exhausted host, and there is no
/// second way to try.
fn runtime() -> Result<Counted<Runtime>, Error> {
    Builder::new_current_thread()
        .enable_all()
        .build()
        .map(|built| Counted::new(built, &RUNTIMES))
        .map_err(|_| Error::InternalInvariant {
            invariant: "the current-thread runtime of ADR-005 can be built",
            location: Location::caller(),
        })
}

/// Opens the one connection of `NFR-PERF-004` and settles the session on it.
///
/// The order is the one `FR-ERR-006` fixes among the three conditions decided
/// once a connection is open and before any catalogue read — the read-only
/// session of `FR-SRV-010`, then the product of `FR-SRV-003`, then the series
/// of `FR-SRV-020` — so the strongest guarantee this tool makes is confirmed
/// before the server is characterised.
///
/// # Errors
///
/// Returns [`Error::NameNotResolved`], [`Error::ConnectionRefused`],
/// [`Error::TlsHandshakeFailed`] or [`Error::NetworkDeadlineExceeded`] where
/// the connection is not opened; [`Error::AuthenticationRefused`] where the
/// server refuses the credentials; and
/// [`Error::ReadOnlySessionNotEnforced`], [`Error::ServerNotMariaDb`] or
/// [`Error::SeriesNotSupported`] where a gate of `FR-SRV-002` refuses the
/// server that answered. On every one of those paths no catalogue statement is
/// issued, because no [`Session`] is produced to issue one with.
pub(crate) fn open(target: &Target<'_>, clock: &Clock) -> Result<Session, Error> {
    let runtime = runtime()?;
    let mut connection = Counted::new(connect::open(&runtime, target, clock)?, &CONNECTIONS);

    // FR-SRV-011: there is no condition around this call, and no caller that
    // reaches a connection without it — `connect::open` is visible to this
    // module alone.
    let server = session::start(&runtime, &mut connection, target, clock)?;

    Ok(Session {
        connection: Some(connection),
        runtime,
        server,
        entry: target.entry().to_owned(),
    })
}

#[cfg(test)]
mod tests {
    use super::{quiescent, runtime};

    #[test]
    fn fr_rnd_040_a_runtime_is_counted_until_it_is_shut_down_and_on_its_own_thread() {
        // FR-RND-040's in-process observation rests on this count.
        assert!(quiescent(), "the test thread starts with nothing alive");

        let built = runtime().expect("the host can build a runtime");
        assert!(!quiescent(), "a built runtime is alive");

        // Another thread holds nothing of this one.
        let elsewhere = std::thread::spawn(quiescent)
            .join()
            .expect("the thread ran");
        assert!(elsewhere, "the count is per thread");

        drop(built);
        assert!(quiescent(), "a dropped runtime is gone");
    }

    #[test]
    fn adr_005_the_runtime_this_module_builds_is_a_current_thread_one() {
        // ADR-005 rejects the multi-thread flavour: with one connection and
        // one statement sequence there is no work to distribute, and a worker
        // pool started for an ephemeral process is startup cost with no
        // counterpart. A current-thread runtime runs everything on the thread
        // that entered it, which is what this observes.
        let runtime = runtime().expect("the host can build a runtime");
        let entered = std::thread::current().id();
        let ran = runtime.block_on(async { std::thread::current().id() });

        assert_eq!(ran, entered);
    }
}
