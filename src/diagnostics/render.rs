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
//! hint:  list the available tables with: tpl -d shop schema tables
//! exit:  66 (EX_NOINPUT)
//! ```
//!
//! [`Label`] is the closed set of four labels and [`push_labelled`] is the only
//! way a line is produced, which is what makes "four lines, in one order"
//! structural rather than reviewed. Each line is composed and then escaped **as
//! a whole**, per `OD-06`, so an interpolation nobody remembered to escape
//! cannot forge a fifth line.
//!
//! The level of `FR-GLOB-014` and `FR-GLOB-015` is not consulted: `-q` lowers
//! the stream to errors **only**, so an error is emitted at every level, and
//! `FR-ERR-013` bars a credential from the message at every level too.
//!
//! `NFR-DET-003` and `NFR-DET-004` are satisfied by construction: no colour, no
//! ANSI escape sequence, and no terminal property consulted.

use std::io::Write as _;

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
    let rendered = render(error);

    // One locked write of the whole block. Stderr is unbuffered, so the
    // composed `String` is the buffer `OD-17` asks for: the four lines reach
    // the stream in one call and cannot be interleaved with another writer's.
    let mut stderr = std::io::stderr().lock();
    let _ = stderr.write_all(rendered.as_bytes());
    let _ = stderr.flush();
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
    use super::{Label, render, sysexits_name};
    use crate::error::{
        CatalogueObjectKind, ContextFault, DeadlineBound, Error, NetworkPhase, Position,
        ReadOnlyFault,
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
    fn the_four_labels_appear_in_the_order_fr_err_008_fixes() {
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
    fn the_labels_are_aligned_as_the_requirement_shows() {
        for label in [Label::Error, Label::Cause, Label::Hint, Label::Exit] {
            assert_eq!(label.as_str().len(), 7, "{label:?} is not aligned");
        }
    }

    #[test]
    fn the_nine_codes_of_the_table_carry_their_sysexits_name() {
        let named: BTreeSet<u8> = [64, 65, 66, 69, 70, 73, 74, 77, 78]
            .into_iter()
            .filter(|code| sysexits_name(*code).is_some())
            .collect();

        assert_eq!(named, BTreeSet::from([64, 65, 66, 69, 70, 73, 74, 77, 78]));
        assert_eq!(sysexits_name(0), None, "0 produces no message");
    }

    // ---------------------------------------- one test per FR-ERR-001 code ---

    #[test]
    fn code_64_names_the_token_and_why_it_was_rejected() {
        // FR-ERR-034, the 64 row.
        let rendered = render(&Error::UnknownCommand {
            token: "sch".to_owned(),
        });

        assert_eq!(
            rendered,
            "error: unknown command 'sch'\n\
             cause: 'sch' is not a name in the command tree, which is closed; tpl matches a \
             command exactly and never by a prefix of one\n\
             hint:  list the commands with: tpl help\n\
             exit:  64 (EX_USAGE)\n"
        );
    }

    #[test]
    fn code_64_names_both_members_of_a_mutually_exclusive_pair() {
        let rendered = render(&Error::MutuallyExclusiveFlags {
            first: "--dsn".to_owned(),
            second: "--host".to_owned(),
        });
        let cause = line(&rendered, Label::Cause);

        assert!(cause.contains("'--dsn'"), "{cause}");
        assert!(cause.contains("'--host'"), "{cause}");
    }

    #[test]
    fn code_64_names_the_value_and_the_type_expected() {
        let rendered = render(&Error::MalformedValue {
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
    fn code_65_names_the_template_the_position_and_the_engine_chain() {
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
             cause: 'example.jinja' could not be compiled; the parser stopped at line 7, column 3 \
             reporting unexpected end of input: caused by block 'columns' was never closed\n\
             hint:  parse the project's templates with: tpl template check\n\
             exit:  65 (EX_DATAERR)\n"
        );
    }

    #[test]
    fn code_65_names_the_context_path_and_the_position_of_the_malformed_json() {
        let rendered = render(&Error::ContextDocumentMalformed {
            path: PathBuf::from("context.json"),
            fault: ContextFault::NotJson(position()),
        });
        let cause = line(&rendered, Label::Cause);

        assert!(cause.contains("context.json"), "{cause}");
        assert!(cause.contains("line 7, column 3"), "{cause}");
    }

    #[test]
    fn code_65_names_which_deadline_expired_and_its_resolved_value() {
        let rendered = render(&Error::RenderDeadlineExceeded {
            bound: DeadlineBound::Overall,
            limit: Duration::from_secs(10),
        });
        let cause = line(&rendered, Label::Cause);

        assert!(cause.contains("the overall budget of 10s"), "{cause}");
        assert!(cause.contains("from process start"), "{cause}");
    }

    #[test]
    fn code_66_names_the_identifier_the_kind_and_the_population() {
        // FR-ERR-034, the 66 row: the database entry and the server-side
        // database are the population for a catalogue object.
        let rendered = render(&Error::CatalogueObjectNotFound {
            kind: CatalogueObjectKind::Table,
            name: "ordrs".to_owned(),
            entry: "shop".to_owned(),
            database: "shop_prod".to_owned(),
        });

        assert_eq!(
            rendered,
            "error: table 'ordrs' does not exist in database 'shop_prod'\n\
             cause: no row of INFORMATION_SCHEMA matches table 'ordrs' in database 'shop_prod', \
             read through database entry 'shop'\n\
             hint:  list the available tables with: tpl -d shop schema tables\n\
             exit:  66 (EX_NOINPUT)\n"
        );
    }

    #[test]
    fn code_69_names_the_phase_the_host_and_the_port() {
        // FR-ERR-034, the 69 row. What the phase returned is the
        // classification `mariadb/` made of it: OD-06 drops the driver value.
        let rendered = render(&Error::ConnectionRefused {
            host: "db.example.com".to_owned(),
            port: 3306,
        });

        assert_eq!(
            rendered,
            "error: the server at db.example.com:3306 refused the connection\n\
             cause: the TCP connect to db.example.com:3306 returned a refusal from the host, so no \
             session was opened\n\
             hint:  check that the server is listening, or repoint the entry with: tpl cfg \
             database update <entry> --port <port>\n\
             exit:  69 (EX_UNAVAILABLE)\n"
        );
    }

    #[test]
    fn code_69_names_the_phase_when_a_deadline_expires() {
        let rendered = render(&Error::NetworkDeadlineExceeded {
            phase: NetworkPhase::CatalogueQuery,
            host: "db.example.com".to_owned(),
            port: 3306,
            bound: DeadlineBound::Phase,
            limit: Duration::from_secs(30),
        });

        assert!(
            line(&rendered, Label::Cause).contains("the catalogue query for db.example.com:3306")
        );
        assert_eq!(
            line(&rendered, Label::Hint),
            "raise the deadline with: tpl cfg set core.query_timeout <seconds>"
        );
    }

    #[test]
    fn code_70_names_the_invariant_and_where_it_was_detected() {
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

    #[test]
    fn code_73_names_the_path_and_which_obstacle_it_met() {
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
    fn code_74_names_the_stream_the_operation_and_what_it_returned() {
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
    fn code_77_names_the_user_the_host_and_that_the_server_refused() {
        // FR-ERR-034, the 77 row.
        let rendered = render(&Error::AuthenticationRefused {
            user: "reader".to_owned(),
            host: "db.example.com".to_owned(),
        });

        assert_eq!(
            rendered,
            "error: the server at 'db.example.com' refused authentication for user 'reader'\n\
             cause: the server at 'db.example.com' rejected the credentials presented for user \
             'reader'; the refusal came from the server and not from tpl\n\
             hint:  correct the credentials with: tpl cfg database update <entry> --user <user>\n\
             exit:  77 (EX_NOPERM)\n"
        );
    }

    #[test]
    fn code_77_names_which_property_of_which_object() {
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
    fn code_78_names_the_key_the_file_and_the_value_expected() {
        // FR-ERR-034, the 78 row.
        let rendered = render(&Error::PasswordCommandNotAnArray {
            key: "database.shop.password_command".to_owned(),
            file: path(),
            position: position(),
            found: "string",
        });

        // FR-CONF-035's illustrative cause names a line of the file; the row
        // obliges the key and the file, and the position is what separates two
        // declarations of the same key in one file.
        assert_eq!(
            rendered,
            "error: database.shop.password_command is not an array\n\
             cause: .tpl/.cfg at line 7, column 3 declares database.shop.password_command as a \
             string; this key takes an array of strings\n\
             hint:  write it as an array: password_command = [\"security\", \
             \"find-generic-password\", \"-s\", \"tpl-shop\", \"-w\"]\n\
             exit:  78 (EX_CONFIG)\n"
        );
    }

    #[test]
    fn code_78_names_the_specific_condition_where_the_fault_is_not_a_key() {
        // The 78 row's second half: the directory the walk ended at, the name
        // of the undefined variable, the series found.
        let walk = render(&Error::ProjectNotFound {
            walk_ended_at: PathBuf::from("/"),
        });
        assert_eq!(
            line(&walk, Label::Cause),
            "the walk upward ended at / without meeting a .tpl folder"
        );
        // FR-PROJ-006 obliges this hint.
        assert_eq!(
            line(&walk, Label::Hint),
            "create a project here with: tpl init"
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
    fn the_unsafe_mode_diagnostic_is_the_one_fr_proj_011_shows() {
        let rendered = render(&Error::ConfigurationUnsafeMode {
            path: path(),
            mode: 0o644,
        });

        assert_eq!(
            rendered,
            "error: .tpl/.cfg has unsafe permissions\n\
             cause: mode 0644 grants access to group or other; tpl reads .tpl/.cfg only at mode \
             0600\n\
             hint:  chmod 600 .tpl/.cfg\n\
             exit:  78 (EX_CONFIG)\n"
        );
    }

    // ------------------------------------------------------- the escaping ---

    #[test]
    fn a_hostile_value_cannot_forge_a_fifth_line() {
        // FR-ERR-024: the value carries \n, \r, \t and a C0 control, and a
        // whole `exit:  0 (EX_OK)` line. Unescaped it would be read as a
        // success by a caller parsing the stream line by line.
        let rendered = render(&Error::UnknownCommand {
            token: HOSTILE.to_owned(),
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
    fn a_hostile_value_is_escaped_on_every_one_of_the_four_lines() {
        // The token reaches `error:` through Display, `cause:` through the
        // derivation, and `hint:` through the character set of FR-ERR-022;
        // `exit:` carries no interpolation at all. Each is escaped as a whole.
        let hostile_entry = format!("shop{HOSTILE}");
        let samples = [
            Error::UnknownCommand {
                token: HOSTILE.to_owned(),
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

    /// One value of every variant of [`Error`], carrying a hostile payload
    /// wherever the variant has a field that can hold one.
    fn samples() -> Vec<Error> {
        let hostile = || HOSTILE.to_owned();
        let hostile_path = || PathBuf::from(HOSTILE);

        vec![
            Error::UnknownCommand { token: hostile() },
            Error::UnknownFlag { token: hostile() },
            Error::MissingArgument {
                command: hostile(),
                argument: hostile(),
            },
            Error::MutuallyExclusiveFlags {
                first: hostile(),
                second: hostile(),
            },
            Error::MalformedValue {
                parameter: hostile(),
                value: hostile(),
                expected: "an integer",
            },
            Error::UnknownConfigurationKey { key: hostile() },
            Error::TemplateSyntax {
                template: hostile(),
                position: position(),
                chain: vec![hostile()],
            },
            Error::RenderFailed {
                template: hostile(),
                position: position(),
                chain: vec![],
            },
            Error::TemplateOutsideRoot {
                name: hostile(),
                root: hostile_path(),
            },
            Error::ContextDocumentMalformed {
                path: hostile_path(),
                fault: ContextFault::Structure {
                    rule: "the root value is an object",
                },
            },
            Error::RenderDeadlineExceeded {
                bound: DeadlineBound::Phase,
                limit: Duration::from_secs(30),
            },
            Error::CatalogueObjectNotFound {
                kind: CatalogueObjectKind::View,
                name: hostile(),
                entry: hostile(),
                database: hostile(),
            },
            Error::TemplateNotFound {
                name: hostile(),
                root: hostile_path(),
            },
            Error::DatabaseEntryNotFound {
                name: hostile(),
                file: hostile_path(),
            },
            Error::ConfigurationKeyNotFound {
                key: hostile(),
                file: hostile_path(),
            },
            Error::NameNotResolved {
                host: hostile(),
                port: 3306,
            },
            Error::ConnectionRefused {
                host: hostile(),
                port: 3306,
            },
            Error::TlsHandshakeFailed {
                host: hostile(),
                port: 3306,
            },
            Error::NetworkDeadlineExceeded {
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
            Error::StdoutUnwritable {
                returned: io::Error::from(io::ErrorKind::StorageFull),
            },
            Error::StdoutClosedMidDocument,
            Error::AuthenticationRefused {
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
                path: hostile_path(),
                position: position(),
            },
            Error::ConfigurationKeyOutsideSpace {
                key: hostile(),
                file: hostile_path(),
            },
            Error::PasswordCommandNotAnArray {
                key: hostile(),
                file: hostile_path(),
                position: position(),
                found: "string",
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
                command: vec![hostile()],
                bound: DeadlineBound::Phase,
                limit: Duration::from_secs(5),
            },
            Error::PasswordCommandOutputCapExceeded {
                command: vec![hostile()],
                cap: 4096,
            },
            Error::PasswordCommandFailed {
                command: vec![hostile()],
                status: None,
            },
            Error::ReadOnlySessionNotEnforced {
                entry: hostile(),
                fault: ReadOnlyFault::NotApplied,
            },
            Error::NoDatabaseEntrySelected {
                file: hostile_path(),
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
    fn the_sample_set_reaches_every_code_of_the_table() {
        // Per-variant coverage is not this set's job: the matches in `cause`
        // and `hint` are exhaustive and carry no wildcard arm, so a variant
        // added to `Error` fails to compile until both derive a line for it.
        let codes: BTreeSet<u8> = samples().iter().map(Error::exit_code).collect();

        assert_eq!(codes, BTreeSet::from([64, 65, 66, 69, 70, 73, 74, 77, 78]));
        assert_eq!(samples().len(), 43, "every variant of Error is sampled");
    }

    #[test]
    fn every_variant_renders_four_labelled_lines_and_no_control_character() {
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
    fn no_cause_restates_its_error_line() {
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
    fn no_line_is_empty_and_every_hint_says_something() {
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
    fn no_rendered_line_carries_an_ansi_escape_sequence() {
        // NFR-DET-004, and NFR-DET-003: no terminal property is consulted, so
        // the output does not vary with one.
        for error in samples() {
            let rendered = render(&error);
            assert!(!rendered.contains('\u{1b}'), "{rendered:?}");
        }
    }
}
