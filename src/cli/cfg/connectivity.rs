//! `tpl cfg database test`: the one `cfg` subcommand that contacts a server.
//!
//! `FR-CFG-005` makes it the exception to the arm, `FR-CACHE-010` fixes what
//! the exception does — it always contacts the server, neither reads nor writes
//! the store, and reads nothing into the model — and `FR-CFG-024` fixes what it
//! performs: **exactly four steps, in one order**, each of which it reports.
//!
//! | Step | What runs | Requirement |
//! |---|---|---|
//! | 1 | Connect to the server the entry describes, and authenticate | `FR-CFG-024` |
//! | 2 | Enforce the read-only session, and confirm it | `FR-SRV-008`, `FR-SRV-009` |
//! | 3 | Verify the server is a supported MariaDB series | `FR-SRV-034`, `FR-SRV-020`, `FR-SRV-031` |
//! | 4 | Run the catalogue privilege probe | `FR-CFG-044` |
//!
//! Steps 1 to 3 are [`crate::mariadb::open`], unchanged and unconditional:
//! `FR-SRV-042` fixes the order of the three connection-start statements and
//! `FR-ERR-006` the order of the conditions they settle, and this command
//! reaches them through the same function every `schema` subcommand reaches
//! them through. There is no second path to a session and no flag that skips
//! one of them, per `FR-SRV-011`. Step 4 is [`catalogue::probe`].
//!
//! # The connection carries no default schema, and that is what step 4 is for
//!
//! [`Target::without_database`] states the reasoning in full and it is not
//! repeated here. In one line: `FR-CFG-044`'s probe answers whether this reader
//! "can see the named database in the catalogue at all", `FR-CFG-045` fixes a
//! false at exit `0`, and a connection that carried the database as its default
//! schema would have the server settle that question at the handshake instead —
//! as a `77`, which `FR-CFG-045` reserves for a refused authentication, and
//! which no success document could ever report.
//!
//! `FR-CONF-041` is satisfied where it asks to be: the database the probe
//! covers is the one the entry names, and an entry that names none is `78`
//! before a connection is opened — which that requirement states of this
//! command by name.
//!
//! # What exit `0` does and does not promise
//!
//! `FR-CFG-045`: exit `0` means the four steps ran and here is what each
//! returned. It does not mean the entry is fully usable, and a caller that
//! branches on the exit code alone and needs the fourth answer reads
//! `can_read_catalogue` — which is what the help of this command says and what
//! its second `EXAMPLES` entry shows. A server **newer** than the supported
//! window is exit `0` as well, read under `FR-SRV-031` and reported through the
//! `standing` field of `FR-CTX-034`.
//!
//! # No error of this command is a document
//!
//! `FR-ERR-033` withdraws the JSON error document outright, so `--format json`
//! changes what a **success** looks like and nothing else: a `69`, a `77` or a
//! `78` from any of the four steps is the four labelled lines of `FR-ERR-008`
//! on stderr, with stdout untouched. Nothing in this module chooses that; it
//! returns the condition and the binary renders it, as every other command
//! does.

use std::io::Write;

use serde::Serialize;

use super::super::layout::{Cell, PROPERTY, flag, text};
use super::super::source;
use super::{Supplied, form, project};
use crate::cli::local::Format;
use crate::error::Error;
use crate::mariadb::{self, Target, catalogue};
use crate::model::server::Server;
use crate::output::{self, Document, Order, Source, Table};
use crate::project::config::expand;
use crate::project::settings;

/// The flag of `FR-CFG-027` that writes `database.<name>.host`.
///
/// It is named here for the totality arm below and nowhere else: the refusal a
/// caller actually meets is [`source::connection_keys`]'s, which is the one
/// place `FR-CONF-040` and `FR-CONF-041` are applied.
const HOST_FLAG: &str = "--host";

/// The placeholder the hint of `FR-CONF-040` writes after it.
const HOST_PLACEHOLDER: &str = "<host>";

/// The `data` of `tpl cfg database test` (`FR-CFG-039`).
///
/// ```json
/// {"entry":"shop","connected":true,"read_only_session":true,"server":{…},"can_read_catalogue":true}
/// ```
///
/// **Five members, in this order**, one per step of `FR-CFG-024` with the entry
/// in front of them. The key order is the field order, per `OD-18`, and it is
/// the order that requirement writes.
///
/// `connected` and `read_only_session` are fields rather than constants because
/// `FR-CFG-039` requires the outcome of every step to be reported and
/// `FR-CFG-024` numbers four of them; a value of this type exists only where
/// both steps succeeded, because either failing is a condition that leaves this
/// function before the report is built. That is not a reason to leave them out:
/// a caller reading the document branches on the field it needs without
/// inferring it from the exit code, which is what that requirement's own
/// rationale asks for.
///
/// `server` is the object of `FR-CTX-031` — the same type, the same three keys
/// and the same meanings, borrowed from the session rather than rebuilt, so the
/// two commands that report a server cannot disagree about what one is.
#[derive(Debug, Serialize)]
struct Report<'a> {
    /// The entry the invocation named.
    entry: &'a str,

    /// Step 1: the connection opened and the credentials were accepted.
    connected: bool,

    /// Step 2: the read-only session of `FR-SRV-008` was enforced and confirmed
    /// under `FR-SRV-009`.
    read_only_session: bool,

    /// Step 3: the server that answered, and its standing against the window
    /// (`FR-CTX-031`, `FR-SRV-034`).
    server: &'a Server<'a>,

    /// Step 4: what the probe of `FR-CFG-044` answered.
    can_read_catalogue: bool,
}

/// `tpl cfg database test <name>` (`FR-CFG-024`, `FR-CFG-039`).
///
/// The steps run in the order `FR-CFG-024` fixes, and each of the first three
/// is a refusal rather than a field where it fails — which is why the report is
/// built last and only once.
///
/// # Errors
///
/// Returns what opening the project returns; [`Error::NoDatabaseEntrySelected`]
/// and the `66` of `FR-GLOB-007` where `name` reaches no entry of the file;
/// [`Error::EntryKeyMissing`] where the entry names no host or no database, per
/// `FR-CONF-040` and `FR-CONF-041`; whatever [`mariadb::open`] returns for the
/// first three steps — the `69` of a server that did not answer, the `77` of a
/// refused credential, the `78` of a read-only session that could not be
/// enforced and the `78` of `FR-SRV-030` for a series outside the window, whose
/// `cause` states that the connection and the authentication succeeded, per
/// `FR-CFG-043`; what [`catalogue::probe`] returns; and
/// [`Error::StdoutUnwritable`] where the report could not be written.
pub(crate) fn test<W: Write>(
    out: &mut W,
    supplied: &Supplied<'_>,
    name: &str,
) -> Result<(), Error> {
    let project = project(supplied)?;
    let configuration = project.configuration()?;
    let clock = supplied.clock();

    // Step 5 of FR-ERR-006. The entry is the operand rather than `-d`: this
    // command tests the entry it was given a name for, so nothing here consults
    // `core.database`.
    let settings = settings::resolve(&configuration, Some(name), &clock, &expand::environment)?;

    source::connection_keys(&settings, &configuration)?;

    // FR-CONF-040 has just refused an entry that names no host, so the arm
    // below is this type's own totality rather than a reachable condition.
    let target = Target::of(&settings).ok_or_else(|| Error::EntryKeyMissing {
        entry: settings.entry().to_owned(),
        key: format!("database.{}.host", settings.entry()),
        file: configuration.file().to_owned(),
        flag: HOST_FLAG,
        placeholder: HOST_PLACEHOLDER,
        absence: crate::error::KeyAbsence::Absent,
    })?;
    // FR-CONF-041 has just refused an entry that names no database, on the same
    // terms.
    let database = settings.database().unwrap_or_default();
    let target = target.without_database();

    // Steps 1, 2 and 3 of FR-CFG-024, in the order FR-SRV-042 and FR-ERR-006
    // fix between them.
    let mut session = mariadb::open(&target, &clock)?;

    // Step 4. Its answer is a boolean and never a row: FR-CACHE-010 keeps this
    // command out of the model, and FR-CFG-044 reads the statement's outcome
    // for two facts and nothing else.
    let can_read_catalogue = catalogue::probe(&mut session, &target, &clock, database);

    // The session is closed whichever way the probe went: the connection of
    // NFR-PERF-004 is released as soon as the last statement has been answered,
    // and a condition on the way out is reported after the socket is gone
    // rather than while it is held.
    let server = session.server().clone();
    session.close();

    let report = Report {
        entry: settings.entry(),
        connected: true,
        read_only_session: true,
        server: &server,
        can_read_catalogue: can_read_catalogue?,
    };

    match supplied.format() {
        Format::Text => written(out, &report),
        // FR-CFG-035: `source` is `server` for this command and for no other of
        // the arm, because contacting the server is what it does.
        Format::Json => {
            output::emit_to(out, &Document::new(Source::Server, &report), form(supplied))
        }
    }
}

/// Writes the report in `text` (`FR-OUT-006`).
///
/// It is the two-column layout `tpl schema info` uses, and the rows are the
/// five members of `FR-CFG-039` with the `server` object spread over the three
/// keys `FR-CTX-031` gives it — which is how that same object is laid out
/// there. `FR-OUT-004` makes none of it a contract: a caller that parses uses
/// `--format json`, which the second `EXAMPLES` entry of this command's help
/// shows.
///
/// # Errors
///
/// Returns [`Error::StdoutUnwritable`] where the stream refused the write.
fn written<W: Write>(out: W, report: &Report<'_>) -> Result<(), Error> {
    let rows: Vec<[Cell<'_>; 2]> = vec![
        [text("entry"), text(report.entry)],
        [text("connected"), flag(report.connected)],
        [text("read_only_session"), flag(report.read_only_session)],
        [text("server"), text(report.server.version())],
        [text("series"), text(report.server.series())],
        [text("standing"), text(report.server.standing().name())],
        [text("can_read_catalogue"), flag(report.can_read_catalogue)],
    ];

    output::emit_table_to(out, &Table::new(PROPERTY, &rows, Order::AsGiven))
}

#[cfg(test)]
mod tests {
    use super::test;
    use crate::cli::cfg::tests::Harness;
    use crate::cli::local::Format;
    use crate::error::Error;
    use crate::mariadb::catalogue::fixture;
    use crate::mariadb::window::{self, Series};

    /// The entry every project below defines.
    const ENTRY: &str = "fixture";

    /// The schema every fixture server carries.
    const SCHEMA: &str = "freight";

    /// The privileged account of the fixture, which sees every database.
    const ROOT: (&str, &str) = ("root", "tpl-root");

    /// The reduced-grant reader of the fixture, which holds
    /// `SELECT, EXECUTE ON freight.*` and nothing else.
    const REDUCED: (&str, &str) = ("tpl_reader", "tpl-reader-pw");

    /// A database name no fixture server carries.
    const ABSENT_SCHEMA: &str = "a_schema_no_server_of_the_fixture_carries";

    /// A database every fixture server carries and [`REDUCED`] holds no grant
    /// on.
    ///
    /// It is the arrangement `FR-CFG-044` is written for: a reader whose
    /// credentials the server accepts and who cannot see the database the entry
    /// names.
    const UNGRANTED_SCHEMA: &str = "mysql";

    /// Runs `body` against every series of `FR-SRV-015`, or skips with a
    /// printed reason where the fixture is not up.
    ///
    /// It is `src/mariadb/catalogue.rs`'s helper, written again here because it
    /// is four lines over the harness's own three-valued gate and importing it
    /// would make one test module's private helper another's interface.
    fn on_every_series(test: &str, body: impl Fn(&fixture::Server)) {
        let Some(series) = fixture::series(test) else {
            return;
        };
        let _exclusive = fixture::exclusive();

        for server in series {
            body(server);
        }
    }

    /// A project whose one entry reaches `server` as `account` and selects
    /// `schema`.
    fn project(server: &fixture::Server, account: (&str, &str), schema: &str) -> Harness {
        Harness::new(&fixture::configuration(server, ENTRY, schema, account))
    }

    /// What `tpl cfg database test <ENTRY>` produced in `format`, for a run
    /// expected to succeed.
    fn reported(harness: &Harness, format: Format) -> String {
        let mut out = Vec::new();

        test(&mut out, &harness.supplied(format, false), ENTRY)
            .expect("the four steps of FR-CFG-024 ran");

        String::from_utf8(out).expect("the command writes UTF-8")
    }

    /// What `tpl cfg database test <ENTRY>` refused, and what it had written by
    /// then.
    fn refused(harness: &Harness) -> (Error, String) {
        let mut out = Vec::new();
        let condition = test(&mut out, &harness.supplied(Format::Json, false), ENTRY)
            .expect_err("the command is refused");

        (
            condition,
            String::from_utf8(out).expect("the command writes UTF-8"),
        )
    }

    #[test]
    fn fr_cfg_039_the_document_carries_the_five_members_in_the_order_the_requirement_fixes() {
        // FR-CFG-039 writes the document out in full, and the key order is part
        // of it: `entry`, `connected`, `read_only_session`, `server`,
        // `can_read_catalogue`, inside the envelope of FR-OUT-024 with `source`
        // set to `server` per FR-CFG-035. The assertion is over the bytes
        // rather than over a parsed value, because a parsed value is exactly
        // what loses the order.
        on_every_series(
            "fr_cfg_039_the_document_carries_the_five_members_in_the_order_the_requirement_fixes",
            |server| {
                let harness = project(server, ROOT, SCHEMA);
                let written = reported(&harness, Format::Json);
                let name = server.name();

                assert!(
                    written.starts_with(
                        "{\"schema_version\":1,\"source\":\"server\",\"data\":\
                         {\"entry\":\"fixture\",\"connected\":true,\
                         \"read_only_session\":true,\"server\":{\"version\":\""
                    ),
                    "{name}: {written}"
                );
                assert!(
                    written.contains("\"series\":\"") && written.contains("\"standing\":\""),
                    "{name}: the server object is not the three keys of FR-CTX-031: {written}"
                );
                assert!(
                    written.ends_with("\"can_read_catalogue\":true}}\n"),
                    "{name}: {written}"
                );
            },
        );
    }

    #[test]
    fn fr_cfg_024_every_series_of_the_window_reports_all_four_steps() {
        // FR-CFG-024: four steps, in one order, each reported. A supported
        // series reaches the fourth, so the `standing` of FR-CTX-034 is
        // `supported` and the probe of FR-CFG-044 answers true — on every
        // server of FR-SRV-015 and not on one of them.
        on_every_series(
            "fr_cfg_024_every_series_of_the_window_reports_all_four_steps",
            |server| {
                let harness = project(server, ROOT, SCHEMA);
                let written = reported(&harness, Format::Json);

                assert!(
                    written.contains("\"standing\":\"supported\""),
                    "{}: {written}",
                    server.name()
                );
                assert!(
                    written.contains("\"can_read_catalogue\":true"),
                    "{}: {written}",
                    server.name()
                );
            },
        );
    }

    #[test]
    fn fr_out_006_the_text_form_lays_the_five_members_out_under_the_two_columns() {
        // FR-OUT-006 with FR-OUT-004: the `text` form is a layout for a person
        // and not a contract, and it carries the same five answers with the
        // `server` object spread over the three keys FR-CTX-031 gives it. What
        // is asserted is that every answer is present and readable, not the
        // widths.
        on_every_series(
            "fr_out_006_the_text_form_lays_the_five_members_out_under_the_two_columns",
            |server| {
                let harness = project(server, ROOT, SCHEMA);
                let written = reported(&harness, Format::Text);
                let name = server.name();

                for row in [
                    "entry               fixture",
                    "connected           yes",
                    "read_only_session   yes",
                    "standing            supported",
                    "can_read_catalogue  yes",
                ] {
                    assert!(written.contains(row), "{name}: {row:?} is not in {written}");
                }
            },
        );
    }

    #[test]
    fn fr_cfg_044_a_database_the_reader_may_not_see_is_reported_rather_than_refused() {
        // FR-CFG-044 and FR-CFG-045, and the whole reason this command opens
        // its connection without a default schema.
        //
        // The arrangement is the fixture's own: `tpl_reader` holds
        // `SELECT, EXECUTE ON freight.*` and nothing else, so the server
        // accepts its credentials and shows it no row of the schema catalogue
        // for `mysql`. `src/mariadb/fault.rs` observed, on all four series, that
        // a connection carrying that database as its default schema is refused
        // at the handshake with `1044` — which `FR-PRIV-021` makes a `77`, and
        // which would make this command answer *your credentials were refused*
        // when they were accepted.
        //
        // What FR-CFG-044 asks for instead is exactly this: the connection
        // succeeds, the probe answers false, and FR-CFG-045 keeps the exit code
        // at `0` so the answer can be read from the document. A `77` here would
        // be the dead surface that requirement's rationale rejects.
        on_every_series(
            "fr_cfg_044_a_database_the_reader_may_not_see_is_reported_rather_than_refused",
            |server| {
                let harness = project(server, REDUCED, UNGRANTED_SCHEMA);
                let written = reported(&harness, Format::Json);
                let name = server.name();

                assert!(
                    written.contains("\"connected\":true"),
                    "{name}: the credentials were accepted and the document says otherwise: \
                     {written}"
                );
                assert!(
                    written.contains("\"can_read_catalogue\":false"),
                    "{name}: {written}"
                );

                // The control, on the same server and with the same account:
                // the database this reader *does* hold a grant on answers true,
                // so the false above is the reader's standing on that database
                // and not a probe that always answers false.
                let granted = project(server, REDUCED, SCHEMA);

                assert!(
                    reported(&granted, Format::Json).contains("\"can_read_catalogue\":true"),
                    "{name}: the probe answered false for a database this reader can see"
                );
            },
        );
    }

    #[test]
    fn fr_cfg_045_a_database_no_server_holds_is_false_at_exit_zero() {
        // FR-CFG-045: a `can_read_catalogue` of false does not change the exit
        // code. The second of the two shapes FR-PRIV-021 declines to separate —
        // a database that is not there — reaches the same answer as the first,
        // for the reason that requirement gives: the catalogue offers no second
        // view that tells them apart, and the probe does not claim to.
        on_every_series(
            "fr_cfg_045_a_database_no_server_holds_is_false_at_exit_zero",
            |server| {
                let harness = project(server, ROOT, ABSENT_SCHEMA);
                let written = reported(&harness, Format::Json);

                assert!(
                    written.contains("\"can_read_catalogue\":false"),
                    "{}: {written}",
                    server.name()
                );
            },
        );
    }

    #[test]
    fn fr_cfg_043_a_series_outside_the_window_is_seventy_eight_and_writes_no_document() {
        // FR-CFG-043: a server that is not a supported series is `78` with the
        // message of FR-SRV-030, whose `cause` states that the connection and
        // the authentication succeeded — which is what separates it, for a
        // caller, from the `69` of an unreachable host and the `77` of a
        // refused credential.
        //
        // The fixture holds no such server: `series.env` carries the four of
        // FR-SRV-015 and all four are inside the window. The condition is
        // reached the way FR-SRV-035 mandates and `src/mariadb/window.rs`
        // provides — by narrowing the **reader** through the `#[cfg(test)]`
        // seam of FR-ERR-031, so that a real server of a real series falls
        // below it. That seam is why this body is a unit test: an integration
        // test links the library compiled without that configuration and cannot
        // see it.
        //
        // FR-ERR-033 is asserted beside it: no error of this command is a
        // document, so stdout is empty however `--format` was given.
        on_every_series(
            "fr_cfg_043_a_series_outside_the_window_is_seventy_eight_and_writes_no_document",
            |server| {
                let name = server.name();
                let harness = project(server, ROOT, SCHEMA);

                // Every series of FR-SRV-015 is older than 13.0, so a window
                // holding it alone puts all four below.
                let _narrowed = window::narrow_to(Series::parse("13.0").expect("two components"));

                let (condition, written) = refused(&harness);

                assert_eq!(condition.exit_code(), 78, "{name}: {condition:?}");
                assert!(
                    matches!(condition, Error::SeriesNotSupported { .. }),
                    "{name}: {condition:?}"
                );
                assert!(
                    written.is_empty(),
                    "{name}: FR-ERR-033, yet stdout carries {written:?}"
                );

                let rendered = crate::diagnostics::rendered(&condition);

                assert!(
                    rendered.contains("connected and authenticated"),
                    "{name}: the cause does not state that both succeeded: {rendered}"
                );

                // The control: with the window restored the same server, the
                // same entry and the same account answer `0`.
                drop(_narrowed);

                assert!(
                    reported(&harness, Format::Json).contains("\"standing\":\"supported\""),
                    "{name}: the series was refused for a reason other than the window"
                );
            },
        );
    }

    #[test]
    fn fr_srv_031_a_series_newer_than_the_window_is_reported_at_exit_zero() {
        // FR-CFG-043's closing paragraph: a server newer than the supported
        // window is not a failure here. It is read under FR-SRV-031, so this
        // command exits `0` and reports it — and `standing` is the one field
        // that separates the two ways it can exit `0`, per FR-CFG-039.
        //
        // The window is narrowed for the reason the body above gives, in the
        // other direction: every series of FR-SRV-015 is 10.11 or newer, so a
        // window holding 10.6 alone puts all four above it.
        on_every_series(
            "fr_srv_031_a_series_newer_than_the_window_is_reported_at_exit_zero",
            |server| {
                let name = server.name();
                let harness = project(server, ROOT, SCHEMA);
                let _narrowed = window::narrow_to(Series::parse("10.6").expect("two components"));

                let written = reported(&harness, Format::Json);

                assert!(
                    written.contains("\"standing\":\"newer_than_supported\""),
                    "{name}: {written}"
                );
                // The fourth step still ran: a server above the window is read,
                // not refused.
                assert!(
                    written.contains("\"can_read_catalogue\":true"),
                    "{name}: {written}"
                );
            },
        );
    }

    #[test]
    fn fr_conf_041_an_entry_that_names_no_database_is_seventy_eight_before_a_connection() {
        // FR-CONF-041 names this command: the probe of FR-CFG-044 "has nothing
        // to restrict itself to" without a database, so the invocation is `78`
        // and reports none of the four steps. It needs no server, because no
        // connection is opened — which is the half of the requirement that a
        // body reaching one could not show.
        let harness =
            Harness::new("[database.fixture]\nhost = \"db.example.com\"\nuser = \"reader\"\n");
        let (condition, written) = refused(&harness);

        assert_eq!(condition.exit_code(), 78, "{condition:?}");
        assert!(
            matches!(&condition, Error::EntryKeyMissing { key, .. }
                if key == "database.fixture.database"),
            "{condition:?}"
        );
        assert!(written.is_empty(), "{written:?}");
    }

    #[test]
    fn fr_glob_007_a_name_that_reaches_no_entry_is_sixty_six() {
        // The `66` the help of this command lists: NAME names no entry of
        // .tpl/.cfg. Entry resolution precedes the four steps, so nothing is
        // reported and no connection is opened.
        let harness = Harness::new("[database.other]\nhost = \"db.example.com\"\n");
        let (condition, written) = refused(&harness);

        assert_eq!(condition.exit_code(), 66, "{condition:?}");
        assert!(written.is_empty(), "{written:?}");
    }
}
