//! The four labelled lines of `FR-ERR-008`, and the only writer of them.
//!
//! `FR-ERR-033` withdraws the JSON error document, so these four lines are an
//! error's whole channel of detail and the exit code is the caller's only
//! machine-comparable signal. The renderer answers the three questions
//! `FR-ERR-008` fixes — what failed, why, and what to do next — in that order,
//! and writes nothing else.
//!
//! ```text
//! error: table 'ordrs' does not exist in database 'shop'
//! cause: no row of INFORMATION_SCHEMA matches table 'ordrs' in database 'shop', …
//! hint:  list the available tables with: tpl schema tables
//! exit:  66 (EX_NOINPUT)
//! ```
//!
//! [`Label`] is the closed set of four labels and [`push_labelled`] is the only
//! way a line is produced, which is what makes "four lines, in one order"
//! structural rather than reviewed. Each line is composed and then escaped **as
//! a whole**, per `OD-06`, so an interpolation nobody remembered to escape
//! cannot forge a fifth line.
//!
//! Two conditions reach this module, because `FR-ERR-030` gives `70` two
//! producing conditions. [`report`] serves every [`Error`] value; [`report_panic`]
//! serves the panic, which `ADR-004` reports from the panic site and which
//! therefore has no [`Error`] to be rendered from. Both compose through
//! [`push_labelled`], so neither can emit a fifth line, a different order, or an
//! unescaped one.
//!
//! The level of `FR-GLOB-014` and `FR-GLOB-015` is not consulted: `-q` lowers
//! the stream to errors **only**, so an error is emitted at every level, and
//! `FR-ERR-013` bars a credential from the message at every level too.
//!
//! `NFR-DET-003` and `NFR-DET-004` are satisfied by construction: no colour, no
//! ANSI escape sequence, and no terminal property consulted.

use std::io::Write as _;
use std::panic::Location;

use super::{cause, escape, hint};
use crate::error::Error;

/// A label of `FR-ERR-008`, padded to the alignment the requirement shows.
///
/// The set is closed, and `FR-ERR-008` fixes the order the variants are
/// declared in.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Label {
    /// What failed.
    Error,
    /// Why it failed.
    Cause,
    /// What to do next.
    Hint,
    /// The status the process exits with.
    Exit,
}

impl Label {
    /// The label as it is written, including the padding that aligns the four.
    const fn as_str(self) -> &'static str {
        match self {
            Self::Error => "error: ",
            Self::Cause => "cause: ",
            Self::Hint => "hint:  ",
            Self::Exit => "exit:  ",
        }
    }
}

/// A generous first guess at the size of a whole diagnostic, in bytes.
const BLOCK_CAPACITY: usize = 512;

/// A generous first guess at the size of one composed line, in bytes.
const LINE_CAPACITY: usize = 192;

/// The code of `FR-ERR-001` that both producing conditions of `FR-ERR-030`
/// carry.
///
/// [`Error::exit_code`] assigns it to the invariant violation, which is the one
/// condition that is an error *value*. The panic path has no value to derive it
/// from, so it is named once here and read by the `exit` line this module
/// composes and by the status [`super::panic`] terminates with — the two cannot
/// disagree.
pub(super) const SOFTWARE: u8 = 70;

/// Writes the four labelled lines of `FR-ERR-008` for `error` to stderr.
///
/// This is the whole of what a caller receives on a failure: `FR-ERR-033`
/// leaves stdout empty, ignores `--format`, and makes the exit status of
/// [`Error::exit_code`] the machine-comparable half of the answer.
///
/// A stderr that refuses the write is not itself reported. There is nothing
/// further to say on a stream that will not carry a diagnostic, and the exit
/// code reaches the caller regardless.
pub(crate) fn report(error: &Error) {
    write_block(&render(error));
}

/// Writes the four labelled lines of `FR-ERR-008` for a panic to stderr.
///
/// This is the other producing condition of `FR-ERR-030`, and the one that
/// reaches the caller from a path where no [`Error`] exists: `ADR-004` composes
/// the message at the panic site rather than at a frame above it, because the
/// release profile aborts and no such frame runs. It goes through
/// [`push_labelled`] like every other diagnostic, so the four lines, their
/// order and their escaping are the same by construction and not by review.
///
/// `location` is what [`std::panic::PanicHookInfo::location`] yielded. The
/// panic's **payload** is deliberately absent: `FR-ERR-034`'s `70` row obliges
/// the `cause` line to name that a panic occurred and where, and requires
/// nothing of the text a panic carried, which is arbitrary and composed at a
/// site no reviewer reads before it runs. Withholding it is how `FR-GLOB-018`
/// stays true here structurally rather than vigilantly.
pub(super) fn report_panic(location: Option<&Location<'_>>) {
    write_block(&render_panic(location));
}

/// Writes one composed block to stderr in a single locked call.
fn write_block(rendered: &str) {
    // One locked write of the whole block. Stderr is unbuffered, so the
    // composed `String` is the buffer `OD-17` asks for: the four lines reach
    // the stream in one call and cannot be interleaved with another writer's.
    let mut stderr = std::io::stderr().lock();
    let _ = stderr.write_all(rendered.as_bytes());
    let _ = stderr.flush();
}

/// The four labelled lines [`report`] writes for `error`, as one block.
///
/// It exists so that a test may read what a caller would read, without
/// capturing a process's stderr. `#[cfg(test)]` is the reachability rule
/// `OD-21` applies to a test seam: the item is not compiled into the artefact
/// `cargo build` produces.
#[cfg(test)]
pub(crate) fn rendered(error: &Error) -> String {
    render(error)
}

/// Composes the four labelled lines for `error`, each terminated by a newline.
fn render(error: &Error) -> String {
    let mut out = String::with_capacity(BLOCK_CAPACITY);
    let mut composed = String::with_capacity(LINE_CAPACITY);

    push_labelled(&mut out, &mut composed, Label::Error, &error.to_string());
    push_labelled(&mut out, &mut composed, Label::Cause, &cause::cause(error));
    push_labelled(&mut out, &mut composed, Label::Hint, &hint::hint(error));
    push_labelled(
        &mut out,
        &mut composed,
        Label::Exit,
        &exit_content(error.exit_code()),
    );

    out
}

/// Composes the four labelled lines for a panic, each terminated by a newline.
fn render_panic(location: Option<&Location<'_>>) -> String {
    let mut out = String::with_capacity(BLOCK_CAPACITY);
    let mut composed = String::with_capacity(LINE_CAPACITY);

    push_labelled(
        &mut out,
        &mut composed,
        Label::Error,
        "tpl stopped on an internal defect",
    );
    push_labelled(
        &mut out,
        &mut composed,
        Label::Cause,
        &panic_cause(location),
    );
    push_labelled(&mut out, &mut composed, Label::Hint, hint::SOFTWARE_DEFECT);
    push_labelled(
        &mut out,
        &mut composed,
        Label::Exit,
        &exit_content(SOFTWARE),
    );

    out
}

/// The content of the `cause` line for a panic: that one occurred, and where.
///
/// The `70` row of `FR-ERR-034` obliges both facts, and the ninth edition reads
/// "where" as the location the condition arose at. A panic runtime that yields
/// no location leaves the second fact unavailable, and the line says so: that
/// is the one wording the ban on naming a category rather than an instance
/// cannot reach, because no instance existed to name.
fn panic_cause(location: Option<&Location<'_>>) -> String {
    match location {
        Some(location) => format!(
            "a panic occurred at {}:{}:{}",
            location.file(),
            location.line(),
            location.column()
        ),
        None => "a panic occurred; the panic runtime reported no location for it".to_owned(),
    }
}

/// Composes one labelled line into `composed`, escapes it as a whole, and
/// appends it to `out` with its terminator.
///
/// `composed` is a scratch buffer the four calls share, so one diagnostic
/// allocates one line buffer rather than four.
fn push_labelled(out: &mut String, composed: &mut String, label: Label, content: &str) {
    composed.clear();
    composed.push_str(label.as_str());
    composed.push_str(content);

    // FR-ERR-024, as OD-06 places it: the whole line, label included, goes
    // through the escaper, and `push_line` appends the only newline it can
    // carry.
    escape::push_line(out, composed);
}

/// The content of the `exit` line: the code, and the `sysexits.h` name of it.
fn exit_content(code: u8) -> String {
    match sysexits_name(code) {
        Some(name) => format!("{code} ({name})"),
        None => code.to_string(),
    }
}

/// The `sysexits.h` name of a code of `FR-ERR-001`.
///
/// `0` is the tenth code of that table and produces no message, so it has no
/// entry here. A value outside the table cannot arise from
/// [`Error::exit_code`], whose match is exhaustive over the nine; the `None`
/// it would yield degrades the line to the bare number rather than inventing a
/// name for it.
const fn sysexits_name(code: u8) -> Option<&'static str> {
    match code {
        64 => Some("EX_USAGE"),
        65 => Some("EX_DATAERR"),
        66 => Some("EX_NOINPUT"),
        69 => Some("EX_UNAVAILABLE"),
        70 => Some("EX_SOFTWARE"),
        73 => Some("EX_CANTCREAT"),
        74 => Some("EX_IOERR"),
        77 => Some("EX_NOPERM"),
        78 => Some("EX_CONFIG"),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::{Label, SOFTWARE, exit_content, render, render_panic, sysexits_name};
    use crate::error::{
        CatalogueObjectKind, ChildEnd, ContextFault, DeadlineBound, DsnFault, EntryRepair, Error,
        LookupKind, NetworkPhase, PasswordCommandFault, Position, ReadOnlyFault, RenderReason,
        Unresolved, trigger_internal_invariant,
    };
    use std::collections::BTreeSet;
    use std::io;
    use std::panic::Location;
    use std::path::PathBuf;
    use std::time::Duration;

    /// A value carrying `\n`, `\r`, `\t`, a C0 control, and a forged `exit`
    /// line that would tell a caller the invocation succeeded.
    const HOSTILE: &str = "a\nexit:  0 (EX_OK)\r\tb\u{7}c";

    /// `HOSTILE` as it must appear once escaped.
    const HOSTILE_ESCAPED: &str = "a\\nexit:  0 (EX_OK)\\r\\tb\\u{07}c";

    fn path() -> PathBuf {
        PathBuf::from(".tpl/.cfg")
    }

    fn position() -> Position {
        Position { line: 7, column: 3 }
    }

    /// A supported window for a sample.
    ///
    /// It is deliberately not the table of `FR-SRV-015`. `BR-SRV-005` states
    /// that set once and forbids a second copy, and the property under test is
    /// that the renderer prints the window it is handed — which a window of
    /// this file's own invention demonstrates and the real one would not.
    const WINDOW: &[&str] = &["Z.9", "Z.8", "Z.7"];

    /// The content of one labelled line of a rendered diagnostic.
    fn line(rendered: &str, label: Label) -> String {
        let wanted = label.as_str();
        let found = rendered
            .lines()
            .find(|line| line.starts_with(wanted))
            .unwrap_or_else(|| panic!("no line carries the label {wanted:?}"));
        found[wanted.len()..].to_owned()
    }

    // -------------------------------------------------------- the shape ---

    #[test]
    fn fr_help_028_both_lines_name_the_node_by_one_rule_at_the_root_and_beneath_it() {
        // FR-HELP-028 requires the node the segment was looked for under to be
        // named, and FR-ERR-010 reads an overlap between the two lines as the
        // reader's signal that both are about the same thing. The `error` line
        // spelled the root itself and produced `under ''` where the `cause`
        // line said `'tpl'`; both now read `cause::invoked`, so the two cannot
        // name one node two ways again.
        for (node, named) in [("", "'tpl'"), ("cfg database", "'tpl cfg database'")] {
            let condition = Error::UnknownCommandPathSegment {
                segment: "nosuchcommand".to_owned(),
                node: node.to_owned(),
                nearest: Vec::new(),
            };
            let rendered = render(&condition);

            assert!(
                line(&rendered, Label::Error).contains(named),
                "the error line of {node:?} does not name the node: {rendered}"
            );
            assert!(
                line(&rendered, Label::Cause).contains(named),
                "the cause line of {node:?} does not name the node: {rendered}"
            );
            assert!(
                !rendered.contains("under ''"),
                "the root reached a line as the empty name: {rendered}"
            );
        }
    }

    #[test]
    fn fr_conf_042_the_two_conditions_that_yield_no_exit_status_are_distinguishable() {
        // FR-CONF-042 obliges the `cause` to say which of the two occurred, and
        // `could not be started` is false of a child that had started. The two
        // lines name the command as stored and what the operating system
        // returned, and neither claims the other's condition.
        let stored = vec!["security".to_owned(), "find-generic-password".to_owned()];

        let not_started = render(&Error::PasswordCommandNotExecutable {
            entry: "shop".to_owned(),
            command: stored.clone(),
            fault: PasswordCommandFault::NotStarted,
            returned: io::Error::from(io::ErrorKind::NotFound),
        });
        let unreadable = render(&Error::PasswordCommandNotExecutable {
            entry: "shop".to_owned(),
            command: stored.clone(),
            fault: PasswordCommandFault::StatusUnreadable,
            returned: io::Error::from(io::ErrorKind::NotFound),
        });

        let first = line(&not_started, Label::Cause);
        let second = line(&unreadable, Label::Cause);

        assert_ne!(first, second);
        assert!(first.contains("could not be started"), "{first}");
        assert!(!second.contains("could not be started"), "{second}");
        assert!(
            second.contains("could not read the status it ended with"),
            "{second}"
        );

        for stated in [&first, &second] {
            assert!(stated.contains("security"), "{stated}");
            assert!(stated.contains("find-generic-password"), "{stated}");
        }
    }

    #[test]
    fn fr_conf_043_the_signal_cause_names_the_number_the_operating_system_reported() {
        // FR-CONF-043 obliges the `cause` to name the signal number: a line
        // saying only that the child was signalled reads identically for a
        // supervisor, a memory limit and an interactive interrupt, which
        // FR-ERR-034 bans.
        let stored = vec!["op".to_owned(), "read".to_owned()];

        let signalled = line(
            &render(&Error::PasswordCommandFailed {
                entry: "shop".to_owned(),
                command: stored.clone(),
                end: ChildEnd::Signalled(9),
            }),
            Label::Cause,
        );

        assert!(signalled.contains("signal 9"), "{signalled}");
        assert!(signalled.contains("tpl did not send"), "{signalled}");

        // The exit is a different condition with a different obligation, and
        // the two do not read alike.
        let exited = line(
            &render(&Error::PasswordCommandFailed {
                entry: "shop".to_owned(),
                command: stored,
                end: ChildEnd::Exited(1),
            }),
            Label::Cause,
        );

        assert!(exited.contains("status 1"), "{exited}");
        assert_ne!(signalled, exited);
    }

    #[test]
    fn fr_err_008_the_four_labels_appear_in_the_order_the_requirement_fixes() {
        let rendered = render(&Error::StdoutClosedMidDocument);
        let lines: Vec<&str> = rendered.lines().collect();

        assert_eq!(lines.len(), 4, "a diagnostic is four lines and no more");
        assert!(lines[0].starts_with("error: "));
        assert!(lines[1].starts_with("cause: "));
        assert!(lines[2].starts_with("hint:  "));
        assert!(lines[3].starts_with("exit:  "));
        assert!(rendered.ends_with('\n'), "the block is newline-terminated");
    }

    #[test]
    fn fr_err_008_the_labels_are_aligned_as_the_requirement_shows() {
        for label in [Label::Error, Label::Cause, Label::Hint, Label::Exit] {
            assert_eq!(label.as_str().len(), 7, "{label:?} is not aligned");
        }
    }

    #[test]
    fn fr_err_001_the_nine_codes_of_the_table_carry_their_sysexits_name() {
        let named: BTreeSet<u8> = [64, 65, 66, 69, 70, 73, 74, 77, 78]
            .into_iter()
            .filter(|code| sysexits_name(*code).is_some())
            .collect();

        assert_eq!(named, BTreeSet::from([64, 65, 66, 69, 70, 73, 74, 77, 78]));
        assert_eq!(sysexits_name(0), None, "0 produces no message");
    }

    // ---------------------------------------- one test per FR-ERR-001 code ---

    #[test]
    fn fr_err_034_code_64_names_the_token_and_why_it_was_rejected() {
        // FR-ERR-034, the 64 row.
        let rendered = render(&Error::UnknownCommand {
            node: String::new(),
            token: "sch".to_owned(),
            nearest: Vec::new(),
        });

        assert_eq!(
            rendered,
            "error: unknown command 'sch'\n\
             cause: 'sch' is not a subcommand of 'tpl' (commands are matched in full, never by a \
             prefix)\n\
             hint:  list the commands with: tpl help\n\
             exit:  64 (EX_USAGE)\n"
        );
    }

    #[test]
    fn fr_err_034_code_64_names_both_members_of_a_mutually_exclusive_pair() {
        let rendered = render(&Error::MutuallyExclusiveFlags {
            first: "--dsn".to_owned(),
            second: "--host".to_owned(),
        });
        let cause = line(&rendered, Label::Cause);

        assert!(cause.contains("'--dsn'"), "{cause}");
        assert!(cause.contains("'--host'"), "{cause}");
    }

    #[test]
    fn fr_err_034_code_64_names_the_value_and_the_type_expected() {
        let rendered = render(&Error::MalformedValue {
            command: String::new(),
            parameter: "--timeout".to_owned(),
            value: "soon".to_owned(),
            expected: "a positive integer number of seconds",
        });
        let cause = line(&rendered, Label::Cause);

        assert_eq!(
            cause,
            "'soon' was supplied for '--timeout', which takes a positive integer number of seconds"
        );
    }

    #[test]
    fn fr_err_034_code_65_names_the_template_the_position_and_the_engine_chain() {
        // FR-ERR-034, the 65 row, with FR-ERR-011.
        let rendered = render(&Error::TemplateSyntax {
            template: "example.jinja".to_owned(),
            position: position(),
            chain: vec![
                "unexpected end of input".to_owned(),
                "block 'columns' was never closed".to_owned(),
            ],
        });

        assert_eq!(
            rendered,
            "error: template 'example.jinja' has a syntax error at line 7, column 3\n\
             cause: 'example.jinja' is not valid template syntax at line 7, column 3: unexpected \
             end of input: caused by block 'columns' was never closed\n\
             hint:  correct line 7 of template 'example.jinja', then check it with: tpl template \
             check example.jinja\n\
             exit:  65 (EX_DATAERR)\n"
        );
    }

    #[test]
    fn fr_err_034_code_65_names_the_context_path_and_the_position_of_the_malformed_json() {
        let rendered = render(&Error::ContextDocumentMalformed {
            path: std::path::Path::new("context.json").into(),
            fault: ContextFault::NotJson(position()),
            default_entry: false,
        });
        let cause = line(&rendered, Label::Cause);

        assert!(cause.contains("context.json"), "{cause}");
        assert!(cause.contains("line 7, column 3"), "{cause}");
    }

    #[test]
    fn fr_ctx_042_the_cause_names_the_path_the_table_the_key_and_the_table_it_names() {
        let rendered = render(&Error::ContextDocumentMalformed {
            path: std::path::Path::new("context.json").into(),
            fault: ContextFault::DanglingReference {
                table: "address".to_owned(),
                collection: "foreign_keys",
                key: "fk_address_city".to_owned(),
                names: "city".to_owned(),
            },
            default_entry: false,
        });
        let cause = line(&rendered, Label::Cause);

        for named in [
            "'context.json'",
            "'address'",
            "'fk_address_city'",
            "foreign_keys",
            "'city'",
        ] {
            assert!(cause.contains(named), "{named} is missing from {cause}");
        }
        assert!(rendered.ends_with("exit:  65 (EX_DATAERR)\n"), "{rendered}");
    }

    #[test]
    fn fr_ctx_042_every_name_of_a_dangling_reference_is_escaped() {
        // FR-ERR-024: all four names come from the untrusted document.
        let rendered = render(&Error::ContextDocumentMalformed {
            path: std::path::Path::new(HOSTILE).into(),
            fault: ContextFault::DanglingReference {
                table: HOSTILE.to_owned(),
                collection: "referenced_by",
                key: HOSTILE.to_owned(),
                names: HOSTILE.to_owned(),
            },
            default_entry: false,
        });

        assert_eq!(rendered.lines().count(), 4, "{rendered:?}");
        assert!(
            line(&rendered, Label::Cause).contains(HOSTILE_ESCAPED),
            "{rendered:?}"
        );
        assert!(
            !rendered.bytes().any(|byte| byte < 0x20 && byte != b'\n'),
            "{rendered:?}"
        );
    }

    #[test]
    fn fr_err_034_code_65_names_which_deadline_expired_and_its_resolved_value() {
        let rendered = render(&Error::RenderDeadlineExceeded {
            bound: DeadlineBound::Overall,
            limit: Duration::from_secs(10),
        });
        let cause = line(&rendered, Label::Cause);

        assert!(cause.contains("the overall budget of 10s"), "{cause}");
        assert!(cause.contains("from process start"), "{cause}");
    }

    #[test]
    fn fr_err_034_code_65_names_the_render_bound_its_resolved_value_and_its_key() {
        // FR-RND-036, FR-RND-037, the 65 row of FR-ERR-034.
        for (error, bound, value, key) in [
            (
                Error::RenderFuelExhausted { fuel: 5000 },
                "render fuel",
                "5000",
                "core.render_fuel",
            ),
            (
                Error::RenderOutputLimitExceeded { limit: 64 },
                "render output limit",
                "64",
                "core.render_output_limit",
            ),
            (
                Error::RenderMemoryLimitExceeded { limit: 16_777_216 },
                "render memory limit",
                "16777216",
                "core.render_memory_limit",
            ),
        ] {
            let rendered = render(&error);
            let cause = line(&rendered, Label::Cause);

            assert!(cause.contains(bound), "{cause}");
            assert!(cause.contains(value), "{cause}");
            assert!(cause.contains(key), "{cause}");
            assert!(line(&rendered, Label::Hint).contains(key), "{rendered}");
            assert!(rendered.ends_with("exit:  65 (EX_DATAERR)\n"), "{rendered}");
        }
    }

    #[test]
    fn fr_err_034_code_66_names_the_identifier_the_kind_and_the_population() {
        // FR-ERR-034, the 66 row: the database entry and the server-side
        // database are the population for a catalogue object.
        let rendered = render(&Error::CatalogueObjectNotFound {
            kind: CatalogueObjectKind::Table,
            name: "ordrs".to_owned(),
            entry: "shop".to_owned(),
            database: "shop_prod".to_owned(),
            nearest: Vec::new(),
        });

        assert_eq!(
            rendered,
            "error: table 'ordrs' does not exist in database 'shop_prod'\n\
             cause: no row of INFORMATION_SCHEMA matches table 'ordrs' in database 'shop_prod', \
             read through database entry 'shop'\n\
             hint:  list the available tables with: tpl schema tables\n\
             exit:  66 (EX_NOINPUT)\n"
        );
    }

    #[test]
    fn fr_err_034_code_69_names_the_phase_the_host_and_the_port() {
        // FR-ERR-034, the 69 row. What the phase returned is the
        // classification `mariadb/` made of it: OD-06 drops the driver value.
        let rendered = render(&Error::ConnectionRefused {
            entry: String::from("shop"),
            host: "db.example.com".to_owned(),
            port: 3306,
        });

        assert_eq!(
            rendered,
            "error: the server at db.example.com:3306 refused the connection, for database entry \
             'shop'\n\
             cause: the TCP connect to db.example.com:3306 did not open a session: the host \
             refused it, or nothing is listening on that port\n\
             hint:  check that the server is running and listening on port 3306, or change the \
             address with: tpl cfg database update shop --host <host> --port <port>\n\
             exit:  69 (EX_UNAVAILABLE)\n"
        );
    }

    #[test]
    fn fr_err_034_code_69_names_the_phase_when_a_deadline_expires() {
        let rendered = render(&Error::NetworkDeadlineExceeded {
            entry: String::from("shop"),
            phase: NetworkPhase::CatalogueQuery,
            host: "db.example.com".to_owned(),
            port: 3306,
            bound: DeadlineBound::Phase,
            limit: Duration::from_secs(30),
        });

        assert!(
            line(&rendered, Label::Cause)
                .contains("a query reading the database structure for db.example.com:3306")
        );
        assert_eq!(
            line(&rendered, Label::Hint),
            "raise the deadline with: tpl cfg set core.query_timeout <seconds>"
        );
    }

    #[test]
    fn fr_err_034_code_70_names_the_invariant_and_where_it_was_detected() {
        // FR-ERR-034, the 70 row, and FR-ERR-032 for the hint.
        let location = Location::caller();
        let rendered = render(&Error::InternalInvariant {
            invariant: "the key column has no column",
            location,
        });
        let cause = line(&rendered, Label::Cause);
        let hint = line(&rendered, Label::Hint);

        assert_eq!(
            cause,
            format!(
                "the invariant 'the key column has no column' was found violated at {}:{}:{}",
                location.file(),
                location.line(),
                location.column()
            )
        );
        assert!(hint.contains("a defect in tpl"), "{hint}");
        assert!(hint.contains("not correctable by the caller"), "{hint}");
        assert_eq!(line(&rendered, Label::Exit), "70 (EX_SOFTWARE)");
    }

    // ------------------------------- the other producing condition of 70 ---

    #[test]
    fn fr_err_030_a_panic_is_reported_as_the_four_lines_and_names_where_it_arose() {
        // FR-ERR-030 and ADR-004: the panic path produces the same outcome as
        // the invariant violation, from a site where no Error exists.
        // FR-ERR-034, the 70 row, obliges that a panic occurred and where.
        let location = Location::caller();
        let rendered = render_panic(Some(location));
        let lines: Vec<&str> = rendered.lines().collect();

        assert_eq!(lines.len(), 4, "a diagnostic is four lines and no more");
        assert!(lines[0].starts_with("error: "));
        assert!(lines[1].starts_with("cause: "));
        assert!(lines[2].starts_with("hint:  "));
        assert!(lines[3].starts_with("exit:  "));

        let cause = line(&rendered, Label::Cause);
        assert_eq!(
            cause,
            format!(
                "a panic occurred at {}:{}:{}",
                location.file(),
                location.line(),
                location.column()
            )
        );
        assert_eq!(line(&rendered, Label::Exit), "70 (EX_SOFTWARE)");
    }

    #[test]
    fn fr_err_032_a_panic_carries_the_hint_the_requirement_requires() {
        let hint = line(&render_panic(Some(Location::caller())), Label::Hint);

        assert!(hint.contains("a defect in tpl"), "{hint}");
        assert!(hint.contains("not correctable by the caller"), "{hint}");
        assert_eq!(
            hint,
            line(
                &render(&Error::InternalInvariant {
                    invariant: "the key column has no column",
                    location: Location::caller(),
                }),
                Label::Hint
            ),
            "the two producing conditions of FR-ERR-030 carry one hint"
        );
    }

    #[test]
    fn fr_glob_018_a_panic_carries_no_payload() {
        // ADR-004 withholds the payload on FR-GLOB-018: nothing composed at the
        // panic site reaches the stream, only the location the runtime yields.
        let rendered = render_panic(Some(Location::caller()));

        assert!(!rendered.contains("panicked at"), "{rendered}");
        assert_eq!(
            rendered.lines().count(),
            4,
            "a payload could only arrive as a fifth line: {rendered}"
        );
    }

    #[test]
    fn fr_err_034_a_panic_without_a_location_says_so_rather_than_naming_a_category() {
        // FR-ERR-034 bans a cause that would read identically for a different
        // failure. Where the runtime yields no location, the absence is the
        // fact, and stating it is not naming a category over an instance.
        let cause = line(&render_panic(None), Label::Cause);

        assert_eq!(
            cause,
            "a panic occurred; the panic runtime reported no location for it"
        );
        assert_eq!(line(&render_panic(None), Label::Exit), "70 (EX_SOFTWARE)");
    }

    #[test]
    fn fr_err_030_the_constant_the_panic_path_exits_with_is_the_code_it_prints() {
        // The `exit` line and the process status cannot disagree, because
        // `panic::install` terminates with this same constant.
        assert_eq!(SOFTWARE, 70);
        assert_eq!(exit_content(SOFTWARE), "70 (EX_SOFTWARE)");
        assert_eq!(
            Error::InternalInvariant {
                invariant: "sample",
                location: Location::caller(),
            }
            .exit_code(),
            SOFTWARE
        );
    }

    #[test]
    fn fr_err_031_the_trigger_produces_the_condition_of_fr_err_030() {
        // The composition OD-21 admits for 70, and the whole of what is
        // executed: the guard is exercised in process and observed to produce
        // the condition of FR-ERR-030 carrying the message FR-ERR-032 requires.
        let error = trigger_internal_invariant()
            .expect_err("the trigger of FR-ERR-031 exists to produce the condition");

        assert_eq!(error.exit_code(), 70);

        let Error::InternalInvariant {
            invariant,
            location,
        } = error
        else {
            panic!("the trigger produced {error:?}, not the condition of FR-ERR-030");
        };

        // The guard is `#[track_caller]`, so the location is the trigger's own
        // call site and not a frame inside the guard.
        assert!(!location.file().is_empty());
        assert!(location.line() > 0 && location.column() > 0);

        let rendered = render(&error);
        assert_eq!(rendered.lines().count(), 4);
        assert_eq!(
            line(&rendered, Label::Cause),
            format!(
                "the invariant '{invariant}' was found violated at {}:{}:{}",
                location.file(),
                location.line(),
                location.column()
            )
        );

        let hint = line(&rendered, Label::Hint);
        assert!(hint.contains("a defect in tpl"), "{hint}");
        assert!(hint.contains("not correctable by the caller"), "{hint}");
        assert_eq!(line(&rendered, Label::Exit), "70 (EX_SOFTWARE)");
    }

    #[test]
    fn fr_err_034_code_73_names_the_path_and_which_obstacle_it_met() {
        // FR-ERR-034, the 73 row: an existing .tpl, or a failure the
        // filesystem reported. The two arms must not read alike.
        let existing = render(&Error::ProjectAlreadyExists {
            path: PathBuf::from("/work/.tpl"),
        });
        let refused = render(&Error::ProjectNotCreated {
            path: PathBuf::from("/work/.tpl"),
            returned: io::Error::from(io::ErrorKind::PermissionDenied),
        });

        assert_eq!(
            existing,
            "error: a .tpl folder already exists at /work/.tpl\n\
             cause: /work/.tpl already exists; tpl init creates a project only where there is \
             none, and has changed nothing here\n\
             hint:  use the project that is already here, or create one elsewhere with: tpl init \
             <path>\n\
             exit:  73 (EX_CANTCREAT)\n"
        );
        assert!(
            line(&refused, Label::Cause)
                .starts_with("the filesystem refused to create /work/.tpl: ")
        );
        assert_ne!(line(&existing, Label::Cause), line(&refused, Label::Cause));
    }

    #[test]
    fn fr_err_034_code_74_names_the_stream_the_operation_and_what_it_returned() {
        // FR-ERR-034, the 74 row.
        let rendered = render(&Error::ProjectFileUnreadable {
            path: path(),
            returned: io::Error::from(io::ErrorKind::PermissionDenied),
        });
        let cause = line(&rendered, Label::Cause);

        assert!(
            cause.starts_with("the read of .tpl/.cfg returned: "),
            "{cause}"
        );
        assert_eq!(line(&rendered, Label::Exit), "74 (EX_IOERR)");
    }

    #[test]
    fn fr_err_034_code_77_names_the_user_the_host_and_that_the_server_refused() {
        // FR-ERR-034, the 77 row.
        let rendered = render(&Error::AuthenticationRefused {
            entry: String::from("shop"),
            user: "reader".to_owned(),
            host: "db.example.com".to_owned(),
        });

        assert_eq!(
            rendered,
            "error: the server at 'db.example.com' refused authentication for user 'reader', \
             for database entry 'shop'\n\
             cause: the server at 'db.example.com' rejected the user 'reader' or its password; \
             the refusal came from the server\n\
             hint:  check the user and the password of the entry (password, password_command, \
             or the password inside dsn), then test it with: tpl cfg database test shop\n\
             exit:  77 (EX_NOPERM)\n"
        );
    }

    #[test]
    fn fr_priv_013_code_77_names_which_property_of_which_object() {
        // FR-PRIV-013.
        let rendered = render(&Error::PropertyNotReadable {
            kind: CatalogueObjectKind::Routine,
            object: "recalculate_totals".to_owned(),
            property: "body",
        });
        let cause = line(&rendered, Label::Cause);

        assert!(
            cause.contains("the body of routine 'recalculate_totals'"),
            "{cause}"
        );
    }

    #[test]
    fn fr_err_034_code_78_names_the_key_the_file_and_the_value_expected() {
        // FR-ERR-034, the 78 row.
        let rendered = render(&Error::PasswordCommandNotAnArray {
            key: "database.shop.password_command".to_owned(),
            file: path(),
            position: position(),
            found: "string",
            element: None,
        });

        // FR-CONF-035's illustrative cause names a line of the file; the row
        // obliges the key and the file, and the position is what separates two
        // declarations of the same key in one file.
        assert_eq!(
            rendered,
            "error: database.shop.password_command is not an array\n\
             cause: .tpl/.cfg at line 7, column 3 declares database.shop.password_command as a \
             string; this key takes an array of strings\n\
             hint:  edit <project>/.tpl/.cfg at line 7 and write it as an array: \
             password_command = [\"security\", \
             \"find-generic-password\", \"-s\", \"tpl-shop\", \"-w\"]\n\
             exit:  78 (EX_CONFIG)\n"
        );
    }

    #[test]
    fn fr_err_034_code_78_names_the_specific_condition_where_the_fault_is_not_a_key() {
        // The 78 row's second half: the directory the walk ended at, the name
        // of the undefined variable, the series found.
        let walk = render(&Error::ProjectNotFound {
            walk_ended_at: PathBuf::from("/"),
        });
        assert_eq!(
            line(&walk, Label::Cause),
            "no .tpl folder in the working directory or in any parent of it, up to /"
        );
        // FR-PROJ-006 obliges this hint.
        assert_eq!(
            line(&walk, Label::Hint),
            "create a project here with: tpl init, or name an existing one with: tpl --tpl-dir \
             <path>/.tpl <command>"
        );

        let variable = render(&Error::UndefinedVariable {
            name: "SHOP_PW".to_owned(),
            key: "database.shop.password".to_owned(),
            file: path(),
        });
        assert!(line(&variable, Label::Cause).contains("'SHOP_PW'"));
        assert_eq!(
            line(&variable, Label::Hint),
            "define it with: export SHOP_PW=<value>"
        );

        let series = render(&Error::SeriesNotSupported {
            entry: "shop".to_owned(),
            series: "10.6".to_owned(),
            supported: WINDOW,
        });
        let series_cause = line(&series, Label::Cause);
        assert!(series_cause.contains("series '10.6'"), "{series_cause}");
        // FR-SRV-030 obliges the cause to list the series that are supported.
        // BR-SRV-005 keeps that list out of this crate, so the assertion is
        // that the window handed in is the window rendered.
        assert!(
            series_cause.ends_with("tpl supports Z.9, Z.8 and Z.7"),
            "{series_cause}"
        );
        // FR-SRV-030 obliges the entry name to be filled in.
        assert_eq!(
            line(&series, Label::Hint),
            "repoint the entry at a supported server: tpl cfg database update shop --host <host>"
        );
    }

    #[test]
    fn fr_proj_011_the_unsafe_mode_diagnostic_is_the_one_the_requirement_shows() {
        // FR-PROJ-011, as the forty-third edition amends it: the hint carries
        // the absolute path, built under FR-ERR-041, so that it succeeds from
        // any directory; a relative path stands in for none.
        let rendered = render(&Error::ConfigurationUnsafeMode {
            path: PathBuf::from("/home/ana/shop/.tpl/.cfg"),
            mode: 0o644,
        });

        assert_eq!(
            rendered,
            "error: /home/ana/shop/.tpl/.cfg has unsafe permissions\n\
             cause: mode 0644 grants access to group or other; tpl reads \
             /home/ana/shop/.tpl/.cfg only when group and other have no access, as at mode \
             0600\n\
             hint:  chmod 600 /home/ana/shop/.tpl/.cfg\n\
             exit:  78 (EX_CONFIG)\n"
        );

        // A path FR-ERR-041 refuses leaves the placeholder in its position.
        let spaced = render(&Error::ConfigurationUnsafeMode {
            path: PathBuf::from("/home/ana/my shop/.tpl/.cfg"),
            mode: 0o644,
        });

        assert_eq!(line(&spaced, Label::Hint), "chmod 600 <project>/.tpl/.cfg");
    }

    // ------------------------------------------------------- the escaping ---

    #[test]
    fn fr_err_024_a_hostile_value_cannot_forge_a_fifth_line() {
        // FR-ERR-024: the value carries \n, \r, \t and a C0 control, and a
        // whole `exit:  0 (EX_OK)` line. Unescaped it would be read as a
        // success by a caller parsing the stream line by line.
        let rendered = render(&Error::UnknownCommand {
            node: String::new(),
            token: HOSTILE.to_owned(),
            nearest: Vec::new(),
        });
        let lines: Vec<&str> = rendered.lines().collect();

        assert_eq!(
            lines.len(),
            4,
            "the forged line reached the stream: {rendered:?}"
        );
        assert_eq!(lines[3], "exit:  64 (EX_USAGE)");
        assert!(
            !rendered.bytes().any(|byte| byte < 0x20 && byte != b'\n'),
            "a control character survived: {rendered:?}"
        );
        assert!(lines[0].contains(HOSTILE_ESCAPED), "{:?}", lines[0]);
    }

    #[test]
    fn fr_err_022_a_hostile_value_is_escaped_on_every_one_of_the_four_lines() {
        // The token reaches `error:` through Display, `cause:` through the
        // derivation, and `hint:` through the character set of FR-ERR-022;
        // `exit:` carries no interpolation at all. Each is escaped as a whole.
        let hostile_entry = format!("shop{HOSTILE}");
        let samples = [
            Error::UnknownCommand {
                node: String::new(),
                token: HOSTILE.to_owned(),
                nearest: Vec::new(),
            },
            Error::MutuallyExclusiveFlags {
                first: HOSTILE.to_owned(),
                second: HOSTILE.to_owned(),
            },
            Error::MissingArgument {
                command: HOSTILE.to_owned(),
                argument: HOSTILE.to_owned(),
            },
            Error::CatalogueObjectNotFound {
                kind: CatalogueObjectKind::Table,
                name: HOSTILE.to_owned(),
                entry: hostile_entry,
                database: HOSTILE.to_owned(),
                nearest: vec![HOSTILE.to_owned()],
            },
            Error::UndefinedVariable {
                name: HOSTILE.to_owned(),
                key: HOSTILE.to_owned(),
                file: PathBuf::from(HOSTILE),
            },
            Error::TemplateSyntax {
                template: HOSTILE.to_owned(),
                position: position(),
                chain: vec![HOSTILE.to_owned()],
            },
        ];

        for error in samples {
            let rendered = render(&error);
            let lines: Vec<&str> = rendered.lines().collect();

            assert_eq!(lines.len(), 4, "{rendered:?}");
            for (index, label) in [Label::Error, Label::Cause, Label::Hint, Label::Exit]
                .into_iter()
                .enumerate()
            {
                assert!(
                    lines[index].starts_with(label.as_str()),
                    "line {index} is not the {label:?} line: {rendered:?}"
                );
            }
            assert!(
                !rendered.bytes().any(|byte| byte < 0x20 && byte != b'\n'),
                "a control character survived: {rendered:?}"
            );
        }
    }

    // --------------------------------------------------- the whole variant set ---

    /// One value of every variant of [`Error`], carrying [`HOSTILE`] wherever
    /// the variant has a field that can hold one.
    fn samples() -> Vec<Error> {
        samples_carrying(HOSTILE)
    }

    /// One value of every variant of [`Error`], carrying `payload` wherever the
    /// variant has a field that can hold one.
    ///
    /// The payload is a parameter because two properties are checked over the
    /// same set with different values: that a control character is escaped,
    /// which [`HOSTILE`] demonstrates, and that a value `FR-ERR-022` governs
    /// never reaches a `hint` line, which [`UNGATED`] demonstrates.
    fn samples_carrying(payload: &str) -> Vec<Error> {
        let hostile = || payload.to_owned();
        let hostile_path = || PathBuf::from(payload);

        vec![
            Error::UnknownCommand {
                node: String::new(),
                token: hostile(),
                nearest: vec![hostile()],
            },
            Error::UnknownFlag {
                positional: false,
                command: String::new(),
                token: hostile(),
                nearest: vec![hostile()],
            },
            Error::UnexpectedArgument {
                command: hostile(),
                token: hostile(),
            },
            Error::RepeatedValueFlag {
                flag: hostile(),
                first: hostile(),
                second: hostile(),
            },
            Error::RepeatedFlag { flag: hostile() },
            Error::FlagValueMissing {
                flag: hostile(),
                permitted: vec![hostile()],
            },
            Error::SeparateTokenValue {
                flag: hostile(),
                value: hostile(),
            },
            Error::ValueOutsideEnumeration {
                command: String::new(),
                flag: hostile(),
                value: hostile(),
                permitted: vec![hostile()],
            },
            Error::InvocationRejected {
                reason: "the value is not one the argument accepts",
                command: String::new(),
                token: Some(hostile()),
            },
            Error::MissingArgument {
                command: hostile(),
                argument: hostile(),
            },
            Error::MutuallyExclusiveFlags {
                first: hostile(),
                second: hostile(),
            },
            Error::MalformedValue {
                command: String::new(),
                parameter: hostile(),
                value: hostile(),
                expected: "an integer",
            },
            Error::UnknownConfigurationKey {
                key: hostile(),
                nearest: vec![hostile()],
            },
            Error::DatabaseEntryAlreadyExists {
                name: hostile(),
                file: hostile_path(),
            },
            Error::IncoherentEntryWrite {
                entry: hostile(),
                written: hostile(),
                conflicting: hostile(),
                repair: EntryRepair::Restate(hostile()),
            },
            Error::InitDestinationIsTplFolder {
                written: hostile_path(),
                canonical: Some(hostile_path()),
                parent: Some(hostile_path()),
            },
            Error::InvalidEntryName {
                given: crate::error::EntryNameGiven::Key("host"),
                name: hostile(),
            },
            Error::InvalidReference {
                parameter: hostile(),
                command: hostile(),
                fault: crate::error::ReferenceFault::Name(hostile()),
            },
            Error::EmptyValue {
                parameter: hostile(),
                command: hostile(),
            },
            Error::InvalidReferenceName {
                key: hostile(),
                file: hostile_path(),
                name: hostile(),
            },
            Error::ConfigurationEntryName {
                file: hostile_path(),
                name: hostile(),
                core: true,
                position: crate::error::Position { line: 1, column: 1 },
            },
            Error::TemplateSyntax {
                template: hostile(),
                position: position(),
                chain: vec![hostile()],
            },
            Error::RenderFailed {
                undefined: None,
                reason: None,
                invoked: String::from("example"),
                template: hostile(),
                position: position(),
                chain: vec![],
            },
            Error::TemplateOutsideRoot {
                name: hostile(),
                root: hostile_path(),
            },
            Error::ContextDocumentMalformed {
                path: hostile_path().into(),
                fault: ContextFault::Structure {
                    at: hostile(),
                    expected: hostile(),
                },
                default_entry: false,
            },
            Error::RenderDeadlineExceeded {
                bound: DeadlineBound::Phase,
                limit: Duration::from_secs(30),
            },
            Error::RenderFuelExhausted { fuel: 100_000_000 },
            Error::RenderOutputLimitExceeded { limit: 67_108_864 },
            Error::RenderMemoryLimitExceeded { limit: 134_217_728 },
            Error::CatalogueObjectNotFound {
                kind: CatalogueObjectKind::View,
                name: hostile(),
                entry: hostile(),
                database: hostile(),
                nearest: vec![hostile()],
            },
            Error::TemplateNotFound {
                name: hostile(),
                root: hostile_path(),
                nearest: vec![hostile()],
            },
            Error::DatabaseEntryNotFound {
                name: hostile(),
                file: hostile_path(),
                nearest: vec![hostile()],
                by_default: true,
            },
            Error::ConfigurationKeyNotFound {
                key: hostile(),
                known: false,
                default: None,
                entry_missing: true,
                file: hostile_path(),
                nearest: vec![(hostile(), false)],
            },
            Error::NameNotResolved {
                entry: String::from("shop"),
                host: hostile(),
                port: 3306,
            },
            Error::ConnectionRefused {
                entry: String::from("shop"),
                host: hostile(),
                port: 3306,
            },
            Error::TlsHandshakeFailed {
                fault: crate::error::TlsFault::Refused,
                entry: String::from("shop"),
                host: hostile(),
                port: 3306,
            },
            Error::NetworkDeadlineExceeded {
                entry: String::from("shop"),
                phase: NetworkPhase::DnsResolution,
                host: hostile(),
                port: 3306,
                bound: DeadlineBound::Overall,
                limit: Duration::from_secs(10),
            },
            Error::InternalInvariant {
                invariant: "the key column has no column",
                location: Location::caller(),
            },
            Error::ProjectAlreadyExists {
                path: hostile_path(),
            },
            Error::ProjectNotCreated {
                path: hostile_path(),
                returned: io::Error::from(io::ErrorKind::PermissionDenied),
            },
            Error::ProjectFileUnreadable {
                path: hostile_path(),
                returned: io::Error::from(io::ErrorKind::PermissionDenied),
            },
            Error::ProjectFileUnwritable {
                path: hostile_path(),
                returned: io::Error::from(io::ErrorKind::StorageFull),
            },
            Error::StdoutUnwritable {
                returned: io::Error::from(io::ErrorKind::StorageFull),
            },
            Error::StdoutClosedMidDocument,
            Error::ContextDocumentUnreadable {
                path: hostile_path(),
                returned: io::Error::from(io::ErrorKind::NotFound),
            },
            Error::TrustMaterialUnreadable {
                entry: hostile(),
                key: "ca_file",
                path: hostile_path(),
                returned: io::Error::from(io::ErrorKind::NotFound),
            },
            Error::ProjectDirUnusable {
                path: hostile_path(),
                fault: crate::error::TplDirFault::NotDirectory,
            },
            Error::ProjectDirUnusable {
                path: hostile_path(),
                fault: crate::error::TplDirFault::HoldsTplFolder,
            },
            Error::ProjectDirUnusable {
                path: hostile_path(),
                fault: crate::error::TplDirFault::NotTplFolder,
            },
            Error::ProjectFolderNotOwned {
                path: hostile_path(),
                owner: 0,
                expected: 501,
            },
            Error::ConfigurationPathReference {
                key: hostile(),
                file: hostile_path(),
                position: position(),
                value: format!("${{{payload}}}"),
            },
            Error::MalformedValue {
                parameter: format!("{payload}.ca_file"),
                command: "cfg set".to_owned(),
                value: format!("${{{payload}}}"),
                expected: "a filesystem path",
            },
            Error::MalformedValue {
                parameter: format!("{payload}.dsn"),
                command: "cfg set".to_owned(),
                value: format!("${{{payload}}}"),
                expected: "a connection URL",
            },
            Error::PrettyWithoutJson {
                command: hostile(),
                complete: true,
            },
            Error::ConnectionDetailsMissing { entry: hostile() },
            Error::NothingToUpdate { entry: hostile() },
            Error::BlockKeyGiven {
                key: hostile(),
                entry: Some(hostile()),
            },
            Error::AuthenticationRefused {
                entry: String::from("shop"),
                user: hostile(),
                host: hostile(),
            },
            Error::PropertyNotReadable {
                kind: CatalogueObjectKind::Routine,
                object: hostile(),
                property: "body",
            },
            Error::ProjectNotFound {
                walk_ended_at: hostile_path(),
            },
            Error::ConfigurationNotOwned {
                path: hostile_path(),
                owner: 0,
                expected: 501,
            },
            Error::ConfigurationUnsafeMode {
                path: hostile_path(),
                mode: 0o644,
            },
            Error::ConfigurationMalformed {
                reason: hostile(),
                path: hostile_path(),
                position: position(),
            },
            Error::ConfigurationKeyOutsideSpace {
                key: hostile(),
                file: hostile_path(),
                position: position(),
                nearest: vec![hostile()],
            },
            Error::ConfigurationValueMalformed {
                key: hostile(),
                file: hostile_path(),
                position: position(),
                found: hostile(),
                expected: "a positive integer number of seconds",
                expanded_from: Some(Box::new(hostile())),
            },
            Error::DsnMalformed {
                key: hostile(),
                file: hostile_path(),
                fault: DsnFault::Form,
            },
            Error::UnclosedExpansion {
                key: hostile(),
                file: hostile_path(),
            },
            Error::PasswordCommandNotAnArray {
                key: hostile(),
                file: hostile_path(),
                position: position(),
                found: "string",
                element: None,
            },
            Error::ConflictingEntryKeys {
                entry: hostile(),
                file: hostile_path(),
                first: hostile(),
                second: hostile(),
            },
            Error::DsnQueryParameter {
                key: hostile(),
                file: hostile_path(),
            },
            Error::UndefinedVariable {
                name: hostile(),
                key: hostile(),
                file: hostile_path(),
            },
            Error::PasswordCommandDeadlineExceeded {
                entry: "shop".to_owned(),
                command: vec![hostile()],
                bound: DeadlineBound::Phase,
                limit: Duration::from_secs(5),
            },
            Error::PasswordCommandOutputCapExceeded {
                entry: "shop".to_owned(),
                command: vec![hostile()],
                cap: 4096,
            },
            Error::PasswordCommandNotExecutable {
                entry: "shop".to_owned(),
                command: vec![hostile()],
                fault: PasswordCommandFault::NotStarted,
                returned: io::Error::from(io::ErrorKind::NotFound),
            },
            Error::PasswordCommandNotExecutable {
                entry: "shop".to_owned(),
                command: vec![hostile()],
                fault: PasswordCommandFault::StatusUnreadable,
                returned: io::Error::from(io::ErrorKind::NotFound),
            },
            Error::PasswordCommandFailed {
                entry: "shop".to_owned(),
                command: vec![hostile()],
                end: ChildEnd::Signalled(9),
            },
            Error::PasswordCommandFailed {
                entry: "shop".to_owned(),
                command: vec![hostile()],
                end: ChildEnd::Unreported,
            },
            Error::TrustDirectoryEmpty {
                entry: hostile(),
                path: hostile_path(),
            },
            Error::ReadOnlySessionNotEnforced {
                entry: hostile(),
                fault: ReadOnlyFault::NotApplied,
            },
            Error::NoDatabaseEntrySelected {
                file: hostile_path(),
                has_entries: true,
            },
            Error::ServerNotMariaDb {
                entry: hostile(),
                product: hostile(),
            },
            Error::SeriesNotSupported {
                entry: hostile(),
                series: hostile(),
                supported: &[HOSTILE],
            },
        ]
    }

    #[test]
    fn fr_err_001_the_sample_set_reaches_every_code_of_the_table() {
        // Per-variant coverage is not this set's job: the matches in `cause`
        // and `hint` are exhaustive and carry no wildcard arm, so a variant
        // added to `Error` fails to compile until both derive a line for it.
        let codes: BTreeSet<u8> = samples().iter().map(Error::exit_code).collect();

        assert_eq!(codes, BTreeSet::from([64, 65, 66, 69, 70, 73, 74, 77, 78]));
        assert_eq!(samples().len(), 82, "every variant of Error is sampled");
    }

    #[test]
    fn fr_err_008_every_variant_renders_four_labelled_lines_and_no_control_character() {
        for error in samples() {
            let rendered = render(&error);
            let lines: Vec<&str> = rendered.lines().collect();

            assert_eq!(lines.len(), 4, "{rendered:?}");
            assert!(lines[0].starts_with("error: "), "{rendered:?}");
            assert!(lines[1].starts_with("cause: "), "{rendered:?}");
            assert!(lines[2].starts_with("hint:  "), "{rendered:?}");
            assert!(lines[3].starts_with("exit:  "), "{rendered:?}");
            assert!(
                !rendered.bytes().any(|byte| byte < 0x20 && byte != b'\n'),
                "a control character survived: {rendered:?}"
            );
        }
    }

    #[test]
    fn fr_err_010_no_cause_restates_its_error_line() {
        // FR-ERR-010.
        for error in samples() {
            let rendered = render(&error);
            assert_ne!(
                line(&rendered, Label::Error),
                line(&rendered, Label::Cause),
                "the cause restates the error: {rendered:?}"
            );
        }
    }

    #[test]
    fn fr_err_008_no_line_is_empty_and_every_hint_says_something() {
        // FR-ERR-008 answers three questions; FR-ERR-012 forbids vague advice,
        // and an empty line is the vaguest of all.
        for error in samples() {
            let rendered = render(&error);
            for label in [Label::Error, Label::Cause, Label::Hint, Label::Exit] {
                assert!(
                    !line(&rendered, label).trim().is_empty(),
                    "the {label:?} line is empty: {rendered:?}"
                );
            }
        }
    }

    #[test]
    fn nfr_det_004_no_rendered_line_carries_an_ansi_escape_sequence() {
        // NFR-DET-004, and NFR-DET-003: no terminal property is consulted, so
        // the output does not vary with one.
        for error in samples() {
            let rendered = render(&error);
            assert!(!rendered.contains('\u{1b}'), "{rendered:?}");
        }
    }

    // ---------------------------------------- the character set of FR-ERR-022 ---

    /// A payload made only of characters no `hint` of [`super::hint`] holds.
    ///
    /// Every one of them is outside `[A-Za-z0-9_]`, so `FR-ERR-022` governs a
    /// value carrying them and `FR-ERR-023` keeps it off the line. None appears
    /// in any literal the module holds, so one on a `hint` line is proof that a
    /// value reached it ungated rather than evidence of a literal.
    const UNGATED: &str = "%!@^|&*?~`\\ \u{1b}[31m";

    /// The characters of [`UNGATED`] the assertion looks for.
    ///
    /// The space and the `[31m` of the escape sequence are left out because a
    /// literal may hold either. The backslash covers two cases at once: it is
    /// one of the characters, and it is also what `FR-ERR-024` puts in front of
    /// an escaped control, so a control that reached the line is caught by it
    /// even in its escaped form.
    const NEVER_IN_A_HINT: &[char] = &[
        '%', '!', '@', '^', '|', '&', '*', '?', '~', '`', '\\', '\u{1b}',
    ];

    #[test]
    fn fr_err_022_no_hint_line_carries_a_value_the_character_set_governs() {
        // FR-ERR-022 and FR-ERR-023, over every variant at once: a hint is
        // built from literals and from names matching `[A-Za-z0-9_]{1,64}`, so
        // a value outside the set reaches no hint line in any form — neither as
        // a runnable command nor as prose.
        for error in samples_carrying(UNGATED) {
            let rendered = render(&error);
            let hint = line(&rendered, Label::Hint);

            assert!(
                !hint.contains(NEVER_IN_A_HINT),
                "a value FR-ERR-022 governs reached the hint line: {hint:?}"
            );
        }
    }

    #[test]
    fn fr_err_022_a_name_the_character_set_admits_still_reaches_its_hint() {
        // The control that makes the test above mean something: the gate lets
        // an admissible name through, so a hint that carried nothing at all
        // would not pass for a hint that refused a hostile name. The entry is
        // not written: FR-ERR-043 writes -d only where the caller gave it.
        let rendered = render(&Error::CatalogueObjectNotFound {
            kind: CatalogueObjectKind::Table,
            name: "ordrs".to_owned(),
            entry: "shop".to_owned(),
            database: "shop".to_owned(),
            nearest: vec!["orders".to_owned()],
        });

        assert_eq!(
            line(&rendered, Label::Hint),
            "did you mean 'orders'? list the available tables with: tpl schema tables"
        );
    }

    #[test]
    fn fr_err_022_a_flag_outside_the_spelling_the_corpus_enumerates_is_not_reproduced() {
        // The pair of a mutually exclusive refusal is prose rather than a
        // runnable command, so FR-ERR-022 does not demand the test; it is the
        // same defensive assertion `admits_path` is for the command path. A
        // token that is not a flag this corpus enumerates came from somewhere
        // other than the flag table, and the hint names neither member.
        let refused = render(&Error::MutuallyExclusiveFlags {
            first: "--host; rm -rf /".to_owned(),
            second: "--dsn".to_owned(),
        });

        assert_eq!(
            line(&refused, Label::Hint),
            "give one of the two flags named above, and not both"
        );

        // The ordinary pair, which the flag table does produce, is named in
        // full: both spellings of this corpus pass the test, hyphen included.
        let named = render(&Error::MutuallyExclusiveFlags {
            first: "--dsn".to_owned(),
            second: "--ca-file".to_owned(),
        });

        assert_eq!(
            line(&named, Label::Hint),
            "give '--dsn' or '--ca-file', and not both"
        );
    }

    // ------------------------------------------------ the forty-third edition ---

    fn hint_of(error: &Error) -> String {
        line(&render(error), Label::Hint)
    }

    #[test]
    fn v_04_a_tcp_connect_that_times_out_names_the_address_before_the_deadline() {
        let timed_out = |bound| Error::NetworkDeadlineExceeded {
            entry: String::from("far"),
            phase: NetworkPhase::TcpConnect,
            host: "10.255.255.1".to_owned(),
            port: 3306,
            bound,
            limit: Duration::from_secs(1),
        };

        assert_eq!(
            hint_of(&timed_out(DeadlineBound::Phase)),
            "check that the host and port named above are the server's address and that it is \
             reachable from here, or change them with: tpl cfg database update far --host \
             <host> --port <port>; to wait longer, raise the deadline with: tpl cfg set \
             core.connect_timeout <seconds>"
        );
        assert!(hint_of(&timed_out(DeadlineBound::Overall)).ends_with(
            "to wait longer, run the same command again with a larger --timeout <seconds>"
        ));
    }

    #[test]
    fn fr_proj_028_a_folder_without_cfg_owned_by_another_user_names_it_and_points_at_tpl_dir() {
        let rendered = render(&Error::ProjectFolderNotOwned {
            path: PathBuf::from("/tmp/.tpl"),
            owner: 0,
            expected: 501,
        });

        assert_eq!(
            line(&rendered, Label::Error),
            "the .tpl folder at /tmp/.tpl cannot be used as a project"
        );
        let cause = line(&rendered, Label::Cause);
        assert!(cause.starts_with("/tmp/.tpl holds no .cfg"), "{cause}");
        assert!(cause.contains("owned by another user"), "{cause}");
        assert_eq!(
            line(&rendered, Label::Hint),
            "name your own project's .tpl folder with: tpl --tpl-dir <project>/.tpl <command>"
        );
        assert!(line(&rendered, Label::Exit).starts_with("78 "));
    }

    #[test]
    fn br_err_004_an_undefined_object_or_var_names_the_flag_that_defines_it() {
        let failed = |undefined: &str| Error::RenderFailed {
            template: "t/needtable.jinja".to_owned(),
            invoked: "t/needtable".to_owned(),
            undefined: Some(undefined.to_owned()),
            reason: None,
            position: position(),
            chain: vec!["undefined value".to_owned()],
        };

        assert_eq!(
            hint_of(&failed("table.name")),
            "'table' exists only when the render names one: add --table <name> to the tpl render \
             command"
        );
        assert_eq!(
            hint_of(&failed("vars.title")),
            "'vars.title' is set with --set: add --set title=<value> to the tpl render command"
        );
        assert!(
            line(&render(&failed("vars.title")), Label::Cause).contains("reads 'vars.title'"),
            "the cause names the expression"
        );

        // Anything else falls back to the source, named for the template.
        assert_eq!(
            hint_of(&failed("x.y")),
            "print the template's source with: tpl template show t/needtable.jinja"
        );
    }

    #[test]
    fn br_err_004_a_network_hint_names_the_entry_and_the_remedy_for_what_failed() {
        let tls = |fault| Error::TlsHandshakeFailed {
            entry: "shop".to_owned(),
            host: "db.example.com".to_owned(),
            port: 3306,
            fault,
        };

        assert!(
            hint_of(&tls(crate::error::TlsFault::Refused))
                .ends_with("tpl cfg set database.shop.tls preferred")
        );
        assert!(
            hint_of(&tls(crate::error::TlsFault::CertificateRejected))
                .contains("tpl cfg set database.shop.ca_file <pem-file>")
        );
        assert_eq!(
            hint_of(&Error::NameNotResolved {
                entry: "shop".to_owned(),
                host: "db".to_owned(),
                port: 3306,
            }),
            "check the host name, or change it with: tpl cfg database update shop --host <host>"
        );
    }

    #[test]
    fn fr_err_041_a_nested_template_candidate_is_admitted_and_a_hostile_one_is_not() {
        assert_eq!(
            hint_of(&Error::TemplateNotFound {
                name: "rust/_type".to_owned(),
                root: PathBuf::from("/p/.tpl/templates"),
                nearest: vec!["rust/_types".to_owned(), "-rf".to_owned(), "a b".to_owned()],
            }),
            "did you mean 'rust/_types'? list the project's templates with: tpl template list"
        );
    }

    #[test]
    fn fr_proj_008_the_hint_creates_a_project_only_where_the_flag_pointed() {
        assert_eq!(
            hint_of(&Error::ProjectDirUnusable {
                path: PathBuf::from("/srv/shop/.tpl"),
                fault: crate::error::TplDirFault::Missing,
            }),
            "correct --tpl-dir, or create the project with: tpl init /srv/shop"
        );
        assert_eq!(
            hint_of(&Error::ProjectDirUnusable {
                path: PathBuf::from("/srv/shop/other"),
                fault: crate::error::TplDirFault::NotDirectory,
            }),
            "correct --tpl-dir so that it names an existing .tpl folder"
        );
    }

    #[test]
    fn e_18_a_password_command_that_never_started_says_so_on_the_error_line() {
        let rendered = render(&Error::PasswordCommandNotExecutable {
            entry: "shop".to_owned(),
            command: vec!["absent".to_owned()],
            fault: PasswordCommandFault::NotStarted,
            returned: io::Error::from(io::ErrorKind::NotFound),
        });

        assert_eq!(
            line(&rendered, Label::Error),
            "the password_command of database entry 'shop' could not be started"
        );
        assert_eq!(
            line(&rendered, Label::Hint),
            "make the first word of database.shop.password_command an executable program on PATH \
             or its full path, e.g.: tpl cfg set database.shop.password_command '<program> \
             <argument>'"
        );
    }

    #[test]
    fn r_03_a_dsn_beside_password_command_is_refused_for_the_password_it_carries() {
        let rendered = render(&Error::IncoherentEntryWrite {
            entry: "s10".to_owned(),
            written: "database.s10.dsn".to_owned(),
            conflicting: "database.s10.password_command".to_owned(),
            repair: EntryRepair::Restate("tpl cfg database add s10".to_owned()),
        });
        assert_eq!(
            line(&rendered, Label::Error),
            "database entry 's10' would get its password twice: database.s10.dsn carries a \
             password, and database.s10.password_command gives another"
        );
        assert!(
            line(&rendered, Label::Cause).contains("takes its password from one place only"),
            "{rendered}"
        );

        // The connection-form conflict names that rule alone.
        let rendered = render(&Error::IncoherentEntryWrite {
            entry: "shop".to_owned(),
            written: "database.shop.dsn".to_owned(),
            conflicting: "database.shop.host".to_owned(),
            repair: EntryRepair::Rewrite {
                unset: vec![
                    "database.shop.host".to_owned(),
                    "database.shop.user".to_owned(),
                ]
                .into(),
                command: "cfg database update",
            },
        });
        assert_eq!(
            line(&rendered, Label::Error),
            "database entry 'shop' cannot declare both database.shop.dsn and database.shop.host"
        );
        assert!(
            !line(&rendered, Label::Cause).contains("takes its password"),
            "{rendered}"
        );
    }

    #[test]
    fn s_03_several_conflicting_keys_are_unset_one_by_one_and_the_entry_is_never_removed() {
        let rendered = render(&Error::IncoherentEntryWrite {
            entry: "f".to_owned(),
            written: "database.f.dsn".to_owned(),
            conflicting: "database.f.host".to_owned(),
            repair: EntryRepair::Rewrite {
                unset: vec!["database.f.host".to_owned(), "database.f.port".to_owned()].into(),
                command: "cfg set",
            },
        });

        assert_eq!(
            line(&rendered, Label::Hint),
            "remove the keys it conflicts with, then write it again: tpl cfg unset \
             database.f.host; tpl cfg unset database.f.port; then tpl cfg set database.f.dsn \
             <url>"
        );
        assert!(!rendered.contains("database remove"), "{rendered}");
    }

    #[test]
    fn s_06_a_lookup_under_context_lists_the_documents_names_with_jq() {
        let failed = |kind, table: Option<&str>, document: &str| Error::RenderFailed {
            template: "t.jinja".to_owned(),
            invoked: "t".to_owned(),
            undefined: Some("x".to_owned()),
            reason: Some(Box::new(RenderReason::Unresolved(Unresolved {
                call: "column(\"orders\", \"nope\")".to_owned(),
                kind,
                name: "nope".to_owned(),
                table: table.map(str::to_owned),
                document: Some(PathBuf::from(document)),
            }))),
            position: position(),
            chain: vec!["undefined value".to_owned()],
        };

        assert_eq!(
            line(
                &render(&failed(LookupKind::Column, Some("orders"), "ok.json")),
                Label::Hint
            ),
            "list the columns of table 'orders' in the --context document with jq, if it is \
             installed: jq -r '.data.database.tables[] | select(.name==\"orders\") | \
             .columns[].name' ok.json"
        );
        assert_eq!(
            line(
                &render(&failed(LookupKind::View, None, "ok.json")),
                Label::Hint
            ),
            "list the views of the --context document with jq, if it is installed: jq -r \
             '.data.database.views[].name' ok.json"
        );
        // Standard input cannot be read twice, so the member is named instead.
        assert_eq!(
            line(
                &render(&failed(LookupKind::Routine, None, "-")),
                Label::Hint
            ),
            "name one the --context document lists under data.database.routines"
        );
        assert!(
            !render(&failed(LookupKind::Table, None, "ok.json")).contains("tpl schema"),
            "the server's listing is not the document's"
        );
    }

    #[test]
    fn r_13_an_entry_that_core_database_names_says_so_and_how_to_change_it() {
        let missing = |by_default| Error::DatabaseEntryNotFound {
            name: "nope".to_owned(),
            file: PathBuf::from("/srv/p/.tpl/.cfg"),
            nearest: Vec::new(),
            by_default,
        };

        let rendered = render(&missing(true));
        assert!(
            line(&rendered, Label::Cause)
                .ends_with(", and core.database names it as the entry to use when -d is not given"),
            "{rendered}"
        );
        assert_eq!(
            line(&rendered, Label::Hint),
            "list the entries with: tpl cfg database list, or change the default with: tpl cfg \
             set core.database <entry>"
        );

        let rendered = render(&missing(false));
        assert!(
            !line(&rendered, Label::Cause).contains("core.database"),
            "{rendered}"
        );
        assert_eq!(
            line(&rendered, Label::Hint),
            "list the entries with: tpl cfg database list"
        );
    }

    #[test]
    fn e_07_a_routine_prefix_refused_in_render_names_the_template_in_the_hint() {
        let rendered = render(&Error::RoutinePrefixNotLowerCase {
            token: "Procedure:x".to_owned(),
            prefix: "procedure",
            name: "x".to_owned(),
            invocation: "render <template> --routine",
            template: Some("t/db".to_owned()),
        });

        assert_eq!(
            line(&rendered, Label::Hint),
            "write it as: tpl render t/db --routine procedure:x"
        );
    }

    #[test]
    fn e_27_an_object_missing_from_a_context_document_names_a_way_to_list_them() {
        let rendered = render(&Error::ContextObjectNotFound {
            kind: CatalogueObjectKind::Table,
            name: "ordrs".to_owned(),
            path: PathBuf::from("ctx.json"),
            database: "shop".to_owned(),
            nearest: Vec::new(),
        });

        assert!(
            line(&rendered, Label::Hint).ends_with("jq -r '.data.database.tables[].name' ctx.json"),
            "{rendered}"
        );
    }

    #[test]
    fn fr_err_041_a_context_path_outside_the_set_is_not_written() {
        // FR-ERR-041, forty-seventh edition: the --context path is governed.
        for path in ["/a b/ctx.json", "-ctx.json", "ctx;rm.json"] {
            let rendered = render(&Error::ContextObjectNotFound {
                kind: CatalogueObjectKind::Table,
                name: "ordrs".to_owned(),
                path: PathBuf::from(path),
                database: "shop".to_owned(),
                nearest: Vec::new(),
            });
            let hint = line(&rendered, Label::Hint);
            assert!(!hint.contains("jq -r"), "{hint}");
            assert!(!hint.contains(path), "{hint}");
            assert!(hint.contains("data.database.tables"), "{hint}");
        }
    }

    #[test]
    fn fr_conf_046_a_refused_password_command_names_the_condition() {
        use crate::project::config::entry::SplitFault;

        let refused = |parameter: &str, fault: SplitFault| {
            render(&Error::MalformedValue {
                parameter: parameter.to_owned(),
                command: "cfg set".to_owned(),
                value: "[\"pass\",\"db/shop\"]".to_owned(),
                expected: fault.condition(),
            })
        };

        let rendered = refused("database.shop.password_command", SplitFault::LeadingBracket);
        assert_eq!(
            line(&rendered, Label::Cause),
            "the value begins with '[', which is how an array arrives; \
             database.shop.password_command takes one command line written as one string, which \
             tpl splits into words"
        );
        assert_eq!(
            line(&rendered, Label::Hint),
            "write the command as one command line, e.g.: tpl cfg set \
             database.shop.password_command 'pass db/shop'"
        );
        assert!(rendered.ends_with("exit:  64 (EX_USAGE)\n"), "{rendered}");

        for (fault, named) in [
            (SplitFault::NoWord, "holds no word"),
            (SplitFault::UnclosedQuote, "leaves a quote unclosed"),
            (
                SplitFault::TrailingBackslash,
                "ends in a backslash outside quotes",
            ),
        ] {
            let rendered = refused("--password-command", fault);
            let cause = line(&rendered, Label::Cause);
            assert!(cause.contains(named), "{cause}");
            assert!(
                cause.contains("--password-command takes one command line"),
                "{cause}"
            );
        }
    }

    #[test]
    fn r_14_a_type_name_that_begins_with_a_vowel_takes_an() {
        let rendered = render(&Error::PasswordCommandNotAnArray {
            key: "database.x.password_command".to_owned(),
            file: PathBuf::from("/srv/p/.tpl/.cfg"),
            position: position(),
            found: "integer",
            element: None,
        });

        assert!(
            line(&rendered, Label::Cause).contains("as an integer;"),
            "{rendered}"
        );
    }

    #[test]
    fn e_07_an_ambiguous_routine_in_render_names_the_template_in_the_hint() {
        let rendered = render(&Error::AmbiguousRoutineInContext {
            name: "calc".to_owned(),
            path: PathBuf::from("ctx.json"),
            database: "shop".to_owned(),
            invocation: "render <template> --routine",
            template: Some("t/db".to_owned()),
        });

        assert_eq!(
            line(&rendered, Label::Hint),
            "name the kind you mean: tpl render t/db --routine procedure:calc, or tpl render t/db \
             --routine function:calc"
        );
    }

    #[test]
    fn r_03_a_file_that_gives_the_password_twice_says_so_and_other_pairs_keep_declares_both() {
        let conflict = |first: &str, second: &str| Error::ConflictingEntryKeys {
            entry: "z".to_owned(),
            file: PathBuf::from("/srv/p/.tpl/.cfg"),
            first: first.to_owned(),
            second: second.to_owned(),
        };

        let rendered = render(&conflict("password", "password_command"));
        assert_eq!(
            line(&rendered, Label::Error),
            "database entry 'z' gets its password twice: password gives one, and \
             password_command gives another"
        );
        assert!(
            line(&rendered, Label::Cause).contains("a password through both password and"),
            "{rendered}"
        );

        let rendered = render(&conflict("dsn", "host"));
        assert_eq!(
            line(&rendered, Label::Error),
            "database entry 'z' declares both dsn and host"
        );
        assert!(
            !line(&rendered, Label::Cause).contains("takes its password"),
            "{rendered}"
        );
    }
}
