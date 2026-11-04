//! The one connection a `mysql` entry opens, and everything settled on it
//! before a catalogue row is read (`mysql-contract.md`).
//!
//! This module is to a `mysql` entry what `crate::mariadb` is to a `mariadb`
//! entry, and it is a module of its own because each engine is a provider of
//! its own (`FR-MYSQL-001`, `ADR-005`). It speaks to the server through the
//! driver `ADR-014` names, and nothing above it is asynchronous, holds a
//! connection or composes a statement.
//!
//! | Submodule | Subject | Forced by |
//! |---|---|---|
//! | [`session`] | The three connection-start statements and the verdicts they settle | `FR-MYSQL-006` … `FR-MYSQL-012`, `FR-MYSQL-015` … `FR-MYSQL-019` |
//! | [`window`] | The supported series, as one ordered table | `FR-MYSQL-002`, `FR-MYSQL-003`, `FR-MYSQL-009`, `FR-MYSQL-010` |
//! | [`fault`] | The refusal of `ADR-014`, recognised by its type | `FR-MYSQL-045` |
//! | [`catalogue`] | The twelve statements, and the model their rows fold into | `FR-MYSQL-015`, `FR-MYSQL-023` … `FR-MYSQL-042`, `FR-MYSQL-046` |
//!
//! **What it shares with `crate::mariadb`, and why only that.**
//! `FR-MYSQL-020` describes, resolves and connects a `mysql` entry exactly as
//! a `mariadb` entry, so the connection is opened by the same code — the
//! options the driver is told, the name resolution, the TLS mode of `ADR-002`,
//! the trust material and the budget — through
//! `crate::mariadb::connect::open_classified`, with one classification of its
//! own. The statement runner, the counting of what is alive, the comparison of
//! two series, the text accessors, and the attachment of foreign keys and the
//! assembly of tables are shared on the same ground: their behaviour is the
//! same for both providers. Everything a MySQL server answers differently —
//! the read-back's spelling, the product check, the window, the twelve
//! statements and every field they map — is this module's.
//!
//! **The runtime is built here for a `mysql` entry, and nowhere else.**
//! `ADR-005` confines the current-thread runtime to the module that opens the
//! connection to a server: [`open`] builds it when a command reaches a `mysql`
//! entry, and [`Session::close`] shuts it down after the socket. Its count and
//! the connection's are this module's own, so [`quiescent`] answers for a
//! `mysql` read as `crate::mariadb::quiescent` answers for a `mariadb` one
//! (`FR-RND-040`).
//!
//! **The session guarantee cannot be stepped around.** [`open`] issues the
//! read-only statement, its read-back and the version probe before it returns
//! anything, behind no condition (`FR-MYSQL-019`), and [`catalogue::read`]
//! takes the [`Session`] only [`open`] produces, so every catalogue statement
//! follows all three (`FR-MYSQL-017`).

pub(crate) mod catalogue;
pub(crate) mod fault;
pub(crate) mod session;
pub(crate) mod window;

use std::cell::Cell;
use std::panic::Location;

use sqlx_core::connection::Connection as _;
use sqlx_mysql::MySqlConnection;
use tokio::runtime::{Builder, Runtime};

use crate::deadline::Clock;
use crate::error::Error;
use crate::mariadb::Counted;
use crate::mariadb::connect::{self, Target};
use crate::model::server::Server;

thread_local! {
    /// The connections to a MySQL server this thread holds open.
    static CONNECTIONS: Cell<usize> = const { Cell::new(0) };

    /// The driver runtimes this thread has built for a `mysql` read and not
    /// yet shut down.
    static RUNTIMES: Cell<usize> = const { Cell::new(0) };
}

/// Whether this thread holds no open connection to a MySQL server and no
/// runtime built for one (`FR-RND-040`).
///
/// The counts are per thread for the reason `crate::mariadb::quiescent`
/// records: the runtime is a current-thread one, and the values that hold a
/// connection or a runtime cannot leave the thread that built them.
pub(crate) fn quiescent() -> bool {
    CONNECTIONS.with(Cell::get) == 0 && RUNTIMES.with(Cell::get) == 0
}

/// An open connection to a MySQL server whose session has been settled.
///
/// A value of this type is a connection that has passed every gate
/// `mysql-contract.md` puts before a catalogue read: the authentication did
/// not need the server's public key over a session TLS does not protect
/// (`FR-MYSQL-045`), the session is read only and the setting has been read
/// back (`FR-MYSQL-018`), the server is MySQL (`FR-MYSQL-007`), and its series
/// is one of `FR-MYSQL-003` or newer than all of them (`FR-MYSQL-010`).
#[derive(Debug)]
pub(crate) struct Session {
    /// The connection itself, declared before the runtime so that the socket
    /// is released while the runtime that registered it is still alive.
    connection: Option<Counted<MySqlConnection>>,

    /// The runtime of `ADR-005`, which every statement is entered through.
    runtime: Counted<Runtime>,

    /// The `server` object of `FR-CTX-031`, as `FR-MYSQL-006` fills it.
    server: Server<'static>,
}

impl Session {
    /// The server this connection reached (`FR-MYSQL-006`).
    pub(crate) const fn server(&self) -> &Server<'static> {
        &self.server
    }

    /// The runtime and the connection, for a statement on this session, or
    /// [`None`] where the session has been closed.
    fn parts(&mut self) -> (&Runtime, Option<&mut MySqlConnection>) {
        (&self.runtime, self.connection.as_deref_mut())
    }

    /// Closes the connection and shuts the runtime down, in that order.
    ///
    /// It is `crate::mariadb::Session::close` for a `mysql` read, on the same
    /// grounds: the driver sends the protocol's quit, the socket closes, and
    /// the runtime is dropped rather than shut down with a timeout, so that no
    /// thread of it outlives this call (`FR-RND-040`). A close that fails is
    /// not a condition.
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

/// The current-thread runtime of `ADR-005`, built for a `mysql` read.
///
/// # Errors
///
/// Returns [`Error::InternalInvariant`] where the runtime cannot be built, for
/// the reason `crate::mariadb` gives for the same guard.
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

/// Opens the one connection of `NFR-PERF-004` for a `mysql` entry and settles
/// the session on it.
///
/// The order is the one `FR-ERR-006` fixes among the conditions of step 6, as
/// its amendment of the seventy-fifth edition places `FR-MYSQL-045` first: the
/// authentication, then the read-only session of `FR-MYSQL-019`, then the
/// product of `FR-MYSQL-008`, then the series of `FR-MYSQL-009`.
///
/// # Errors
///
/// Returns what `crate::mariadb::connect::open_classified` returns where the
/// connection is not opened, [`Error::PublicKeyRequestRefused`] where the
/// authentication needs the server's public key over a session TLS does not
/// protect, and what [`session::start`] returns where a gate refuses the
/// server. On every one of those paths no catalogue statement is issued.
pub(crate) fn open(target: &Target<'_>, clock: &Clock) -> Result<Session, Error> {
    // FR-ERR-045, as `crate::mariadb::open` applies it.
    let marked = |error: Error| {
        if target.by_dsn() {
            error.of_dsn_entry()
        } else {
            error
        }
    };

    let runtime = runtime()?;
    let mut connection = Counted::new(
        connect::open_classified(&runtime, target, clock, |driver, user| {
            fault::connecting(target, driver, user)
        })
        .map_err(marked)?,
        &CONNECTIONS,
    );

    // FR-MYSQL-019: no condition around this call.
    let server = session::start(&runtime, &mut connection, target, clock).map_err(marked)?;

    Ok(Session {
        connection: Some(connection),
        runtime,
        server,
    })
}

/// One unit of this module's connection count, for a test that counts by hand.
#[cfg(test)]
fn counted_connection() -> crate::mariadb::Alive {
    crate::mariadb::Alive::of(&CONNECTIONS)
}

#[cfg(test)]
mod tests {
    use super::{counted_connection, quiescent, runtime};

    #[test]
    fn fr_rnd_040_a_mysql_runtime_and_connection_are_counted_on_their_own_thread() {
        // FR-RND-040: the render asks whether anything of a read is alive, and
        // a `mysql` read is counted in this module's own counts.
        assert!(quiescent(), "the test thread starts with nothing alive");

        let built = runtime().expect("the host can build a runtime");
        assert!(!quiescent(), "a built runtime is alive");
        assert!(
            crate::mariadb::quiescent(),
            "a mysql runtime is not a mariadb one"
        );

        let elsewhere = std::thread::spawn(quiescent)
            .join()
            .expect("the thread ran");
        assert!(elsewhere, "the count is per thread");

        drop(built);
        assert!(quiescent(), "a dropped runtime is gone");

        let connection = counted_connection();
        assert!(!quiescent(), "a counted connection is alive");
        drop(connection);
        assert!(quiescent());
    }

    #[test]
    fn adr_005_the_runtime_this_module_builds_is_a_current_thread_one() {
        let runtime = runtime().expect("the host can build a runtime");
        let entered = std::thread::current().id();
        let ran = runtime.block_on(async { std::thread::current().id() });

        assert_eq!(ran, entered);
    }
}
