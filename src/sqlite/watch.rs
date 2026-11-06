//! The bound on every statement of a read of a SQLite file (`FR-LITE-009`,
//! `OD-36`).
//!
//! Two means bound a statement, because each bounds what the other cannot: the
//! library's own wait for a lock, whose timeout is set to what remains of the
//! statement's bound before it runs, replacing `rusqlite`'s 5000 ms default
//! (`ADR-013`, *Consequences*); and one watchdog thread per open of the file,
//! which holds its `Connection`'s interrupt handle and interrupts a statement
//! still running when its bound expires.
//!
//! **Disarming is a rendezvous.** The thread interrupts only while it is
//! armed, under the lock [`Watchdog::disarm`] takes, so an interrupt reaches
//! the statement it was armed for or none: once `disarm` has returned, nothing
//! issued for that statement can reach the next. An interrupt issued after
//! the statement returned and before it was disarmed reaches no statement, by
//! SQLite's own rule — a call "when there are no running SQL statements is a
//! no-op" (`https://sqlite.org/c3ref/interrupt.html`, consulted 2026-11-03).
//!
//! **The thread is stopped and joined before the file is closed**, so no
//! interrupt is ever issued against a closed `Connection`, which the same
//! page states is not safe.
//!
//! **The deadline arithmetic saturates.** A `core.query_timeout` the
//! configuration admits can lie past what an [`Instant`] can represent from
//! now; such a bound arms the watchdog with no instant at all, so the statement
//! is never interrupted and nothing panics (`OD-36`, as amended for rmp
//! `#317`).
//!
//! The thread is not speculative parallelism, on the ground `OD-12` gives the
//! render's watchdog: it performs no work of the invocation and exists only on
//! the path that opens a database file.

use std::sync::{Arc, Condvar, Mutex, MutexGuard, PoisonError};
use std::thread::JoinHandle;
use std::time::{Duration, Instant};

use rusqlite::InterruptHandle;

/// What the watchdog is doing.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum State {
    /// No statement is running under it.
    Idle,
    /// A statement is running, and must be interrupted at this instant —
    /// or never, where its bound lies past every instant the clock can
    /// represent.
    Armed(Option<Instant>),
    /// The statement it was armed for was interrupted.
    Fired,
    /// The file is about to close; the thread ends.
    Stopped,
}

/// The state shared between the thread that reads the file and the watchdog's.
#[derive(Debug)]
struct Shared {
    /// The state, under the lock every transition takes.
    state: Mutex<State>,
    /// Woken on every transition.
    changed: Condvar,
}

impl Shared {
    /// The state, locked; a poisoned lock still holds a valid state, because
    /// every transition writes a whole value.
    fn lock(&self) -> MutexGuard<'_, State> {
        self.state.lock().unwrap_or_else(PoisonError::into_inner)
    }
}

/// One watchdog, for one open of the file.
#[derive(Debug)]
pub(crate) struct Watchdog {
    /// The state it shares with its thread.
    shared: Arc<Shared>,
    /// The thread, joined when the watchdog stops.
    thread: Option<JoinHandle<()>>,
}

impl Watchdog {
    /// Starts the watchdog of the open file whose interrupt handle is
    /// `handle`.
    ///
    /// # Errors
    ///
    /// Returns what the operating system returned where it could not start the
    /// thread.
    pub(crate) fn start(handle: InterruptHandle) -> std::io::Result<Self> {
        let shared = Arc::new(Shared {
            state: Mutex::new(State::Idle),
            changed: Condvar::new(),
        });
        let watched = Arc::clone(&shared);
        let thread = std::thread::Builder::new()
            .name("tpl-sqlite-watchdog".to_owned())
            .spawn(move || watch(&watched, &handle))?;

        Ok(Self {
            shared,
            thread: Some(thread),
        })
    }

    /// Arms the watchdog for a statement that may run for `remaining`.
    ///
    /// The instant is `Instant::checked_add`'s: a bound past what the clock
    /// can represent arms the watchdog with none, and never panics.
    pub(crate) fn arm(&self, remaining: Duration) {
        let mut state = self.shared.lock();
        *state = State::Armed(Instant::now().checked_add(remaining));
        self.shared.changed.notify_all();
    }

    /// Disarms it once the statement returned, and answers whether it
    /// interrupted the statement.
    pub(crate) fn disarm(&self) -> bool {
        let mut state = self.shared.lock();
        let fired = *state == State::Fired;
        *state = State::Idle;
        self.shared.changed.notify_all();
        fired
    }

    /// Stops the thread and joins it (`OD-36`, point 4).
    pub(crate) fn stop(mut self) {
        self.halt();
    }

    /// Stops and joins the thread, once.
    fn halt(&mut self) {
        {
            let mut state = self.shared.lock();
            *state = State::Stopped;
            self.shared.changed.notify_all();
        }
        if let Some(thread) = self.thread.take() {
            // The thread panics on nothing it does; a join that reports one
            // has nothing left to stop.
            let _ = thread.join();
        }
    }
}

impl Drop for Watchdog {
    /// A watchdog dropped without [`Watchdog::stop`] — on an early return — is
    /// stopped and joined here, before the `Connection` beside it is dropped.
    fn drop(&mut self) {
        self.halt();
    }
}

/// The watchdog's thread: waits for each armed instant and interrupts the
/// statement still running at it.
fn watch(shared: &Shared, handle: &InterruptHandle) {
    let mut state = shared.lock();

    loop {
        match *state {
            State::Stopped => return,
            // A bound past every representable instant is never reached: the
            // statement runs until it returns, and the thread waits as idle.
            State::Idle | State::Fired | State::Armed(None) => {
                state = shared
                    .changed
                    .wait(state)
                    .unwrap_or_else(PoisonError::into_inner);
            }
            State::Armed(Some(at)) => {
                let now = Instant::now();
                if now >= at {
                    // The interrupt is issued under the lock disarm takes, so
                    // it reaches the statement it was armed for or none.
                    handle.interrupt();
                    *state = State::Fired;
                    shared.changed.notify_all();
                } else {
                    state = shared
                        .changed
                        .wait_timeout(state, at - now)
                        .map_or_else(|poisoned| poisoned.into_inner().0, |(state, _)| state);
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::Watchdog;
    use rusqlite::Connection;
    use rusqlite::trace::{TraceEvent, TraceEventCodes};
    use std::sync::{Condvar, Mutex, PoisonError};
    use std::time::Duration;

    /// A statement that never returns on its own: only an interrupt ends it.
    const ENDLESS: &str =
        "WITH RECURSIVE n(i) AS (SELECT 1 UNION ALL SELECT i + 1 FROM n) SELECT count(*) FROM n";

    /// Whether [`ENDLESS`] has begun running, as the statement trace reports
    /// it; only [`fr_lite_009_a_statement_past_its_bound_is_interrupted_and_the_next_is_not`]
    /// runs it on a traced `Connection`.
    static BEGUN: Mutex<bool> = Mutex::new(false);

    /// Woken when [`BEGUN`] is set.
    static BEGAN: Condvar = Condvar::new();

    /// The statement trace: records that [`ENDLESS`] has begun.
    ///
    /// SQLite reports a statement when it begins running, after it has
    /// counted it as running, so an interrupt issued from here on reaches it
    /// rather than being the no-op an interrupt with no statement running is
    /// (`https://sqlite.org/c3ref/interrupt.html`).
    fn note(event: TraceEvent<'_>) {
        if let TraceEvent::Stmt(_, text) = event
            && text == ENDLESS
        {
            *BEGUN.lock().unwrap_or_else(PoisonError::into_inner) = true;
            BEGAN.notify_all();
        }
    }

    #[test]
    fn fr_lite_009_a_statement_past_its_bound_is_interrupted_and_the_next_is_not() {
        // rmp #366: nothing here depends on how fast the host runs. The
        // statements within their bound are armed with a bound no clock
        // reaches, and the endless statement is armed only once the trace has
        // reported it running, so the interrupt cannot be issued before it
        // starts, and the statement cannot end before the interrupt.
        let mut connection = Connection::open_in_memory().expect("a memory database opens");
        connection
            .trace_v2(TraceEventCodes::SQLITE_TRACE_STMT, Some(note))
            .expect("the statement trace installs");
        let watchdog =
            Watchdog::start(connection.get_interrupt_handle()).expect("the thread starts");

        // A statement that returns within its bound.
        watchdog.arm(Duration::MAX);
        let answered: i64 = connection
            .query_row("SELECT 1", [], |row| row.get(0))
            .expect("the statement answers");
        assert_eq!(answered, 1);
        assert!(!watchdog.disarm());

        // A statement that runs past a bound of 50 ms.
        let running = std::thread::spawn(move || {
            let answered = connection.query_row(ENDLESS, [], |row| row.get::<_, i64>(0));
            (answered, connection)
        });
        {
            let mut begun = BEGUN.lock().unwrap_or_else(PoisonError::into_inner);
            while !*begun {
                begun = BEGAN.wait(begun).unwrap_or_else(PoisonError::into_inner);
            }
        }
        watchdog.arm(Duration::from_millis(50));
        let (interrupted, connection) = running.join().expect("the statement thread returns");
        assert!(watchdog.disarm(), "the watchdog fired");
        let code = match interrupted {
            Err(rusqlite::Error::SqliteFailure(failure, _)) => failure.extended_code,
            other => panic!("the statement was not interrupted: {other:?}"),
        };
        assert_eq!(code, rusqlite::ffi::SQLITE_INTERRUPT);

        // The next statement is not reached by the interrupt.
        watchdog.arm(Duration::MAX);
        let after: i64 = connection
            .query_row("SELECT 2", [], |row| row.get(0))
            .expect("the next statement answers");
        assert_eq!(after, 2);
        assert!(!watchdog.disarm());

        watchdog.stop();
    }

    #[test]
    #[ignore = "measurement: needs `SELECT 1` to return within a 30 s wall-clock bound"]
    fn fr_lite_009_a_statement_within_a_bound_the_clock_reaches_is_not_interrupted() {
        // The armed path with an instant the clock reaches, which the
        // deterministic test above does not take for a statement within its
        // bound: the watchdog waits for the instant and does not fire before
        // it.
        let connection = Connection::open_in_memory().expect("a memory database opens");
        let watchdog =
            Watchdog::start(connection.get_interrupt_handle()).expect("the thread starts");

        watchdog.arm(Duration::from_secs(30));
        let answered: i64 = connection
            .query_row("SELECT 1", [], |row| row.get(0))
            .expect("the statement answers");
        assert_eq!(answered, 1);
        assert!(!watchdog.disarm());

        watchdog.stop();
    }

    #[test]
    fn fr_lite_009_a_bound_past_every_representable_instant_arms_without_panicking() {
        // OD-36 as amended for rmp #317: a core.query_timeout the
        // configuration admits can lie past what an Instant can represent;
        // arming with it neither panics nor interrupts the statement.
        let connection = Connection::open_in_memory().expect("a memory database opens");
        let watchdog =
            Watchdog::start(connection.get_interrupt_handle()).expect("the thread starts");

        for remaining in [Duration::MAX, Duration::from_secs(u64::MAX / 2)] {
            watchdog.arm(remaining);
            let answered: i64 = connection
                .query_row("SELECT 3", [], |row| row.get(0))
                .expect("the statement answers");
            assert_eq!(answered, 3);
            assert!(!watchdog.disarm(), "the watchdog fired");
        }

        watchdog.stop();
    }
}
