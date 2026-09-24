//! The closed set of typed emission functions, per `OD-17`.
//!
//! There is no general-purpose sink here that accepts arbitrary text. Every
//! function takes **typed arguments** and composes its own line, so there is no
//! `debug!("{e}")` to write because there is no function that would take one.
//! That is what makes `FR-GLOB-018` structural rather than reviewed: the
//! argument vector, the resolved DSN, the `password_command` and its stderr,
//! the raw driver error and the contents of `.tpl/.cfg` are denied a **home**,
//! not denied by a rule someone must remember. `OD-06` supplies the third leg
//! by dropping the driver's error at the `mariadb/` boundary, so it is not
//! present in the process to be emitted even by a function that would take it.
//!
//! Every line is escaped as a whole before it reaches the stream, on the same
//! terms as the four labelled lines: `FR-ERR-024` owns escaping in a
//! diagnostic message, and a catalogue byte reaches this stream too.
//!
//! `NFR-DET-001` puts stderr outside the contract, so the wording after a
//! line's leading token is free to change and no test reads it.

use std::io::Write as _;
use std::path::Path;
use std::time::Duration;

use super::escape;
use super::verbosity::{Level, emits};
use crate::deadline::Phase;

/// The fixed leading token of the one line per catalogue query.
///
/// `FR-GLOB-017` requires exactly one line per catalogue query issued,
/// distinguishable from every other diagnostic line, and `NFR-PERF-008` makes
/// the query count observable from outside the process through it — which is
/// what lets `NFR-PERF-001` and `NFR-PERF-002` be checked at all. One function
/// writes this token and no other line begins with it.
const CATALOGUE_QUERY_TOKEN: &str = "query:";

/// The fixed leading token of the one line per phase that ran.
///
/// `FR-GLOB-017` requires the level to report **which phases ran and how long
/// each took**, beside the one line per catalogue query. One function writes
/// this token and no other line begins with it, so the phases an invocation ran
/// are recoverable from the stream by the same reading that recovers the query
/// count.
const PHASE_TOKEN: &str = "phase:";

/// The leading token of a warning.
const WARNING_TOKEN: &str = "warning:";

/// A generous first guess at the size of one emitted line, in bytes.
const LINE_CAPACITY: usize = 160;

/// Writes the one line `FR-GLOB-017` requires for a catalogue query.
///
/// Called once per query issued against `INFORMATION_SCHEMA`, so that the
/// count of lines carrying [`CATALOGUE_QUERY_TOKEN`] is the count of queries.
///
/// The line carries the token and no identity of the query. `FR-GLOB-017`
/// constrains the existence of the line and its distinguishability, not its
/// wording, and the query identity `OD-17` anticipates is a type `mariadb/`
/// owns; adding it is a change this signature invites and does not need.
///
/// **It carries no credential and no driver message**, which is `FR-ERR-013`
/// and `FR-GLOB-018` obtained structurally rather than by rule: the function
/// takes no argument, so there is nothing a caller could hand it and nothing
/// composed into the line but a literal. The statement text is not carried
/// either — it would be the one thing a caller could mistake for the contract
/// `NFR-DET-001` puts outside stderr.
pub(crate) fn catalogue_query() {
    if !emits(Level::Info) {
        return;
    }

    let mut composed = String::with_capacity(LINE_CAPACITY);
    composed.push_str(CATALOGUE_QUERY_TOKEN);
    composed.push_str(" read INFORMATION_SCHEMA");

    write_line(&composed);
}

/// Reports that `phase` ran, and how long it took (`FR-GLOB-017`).
///
/// Called once by the code that runs a phase of `FR-CONF-005`, after that phase
/// has ended, whether it ended in an answer or in a refusal: a phase that failed
/// is a phase that ran, and how long it took before it failed is the half of the
/// report a caller diagnosing a slow invocation is looking for.
///
/// The two arguments are typed, per `OD-17`, and neither can carry anything
/// `FR-GLOB-018` bars: a [`Phase`] is one of six discriminants this crate
/// declares, and a [`Duration`] is a number. There is no host, no user, no
/// statement and no driver message on this line, and no argument through which
/// one could be passed.
///
/// The duration is written in milliseconds with one fractional digit, which is
/// the resolution a caller can act on; `NFR-DET-001` puts stderr outside the
/// contract, so the wording is free to change and no test reads it.
pub(crate) fn phase_ran(phase: Phase, took: Duration) {
    if !emits(Level::Info) {
        return;
    }

    write_line(&phase_line(phase, took));
}

/// Composes the line of [`phase_ran`], unescaped.
fn phase_line(phase: Phase, took: Duration) -> String {
    format!(
        "{PHASE_TOKEN} {} took {:.1}ms",
        phase.name(),
        took.as_secs_f64() * 1_000.0
    )
}

/// Warns that `tpl init` was given `--tpl-dir`, which has no effect on it
/// (`FR-PROJ-026`).
///
/// The requirement fixes the line and forbids it to reproduce the value given
/// to the flag, which is never examined, so the function takes no argument.
pub(crate) fn tpl_dir_has_no_effect_on_init() {
    if !emits(Level::Warnings) {
        return;
    }

    write_line(TPL_DIR_ON_INIT);
}

/// The line of [`tpl_dir_has_no_effect_on_init`], as `FR-PROJ-026` fixes it.
const TPL_DIR_ON_INIT: &str = "warning: --tpl-dir has no effect on tpl init; it takes its \
                               destination as an operand: tpl init <path>";

/// Where the database on the server of the entry an invocation writes comes
/// from, which decides the last clause of the line of `FR-CFG-051`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum ServerDatabase<'a> {
    /// The invocation gives `--dsn`, or `update` names an entry `.tpl/.cfg`
    /// defines by `dsn`: the database is the path part of the dsn, and
    /// `--schema` is refused (`FR-CFG-048`).
    Dsn,
    /// The invocation gives `--schema`, so nothing is left to advise.
    SchemaGiven,
    /// The value given to `-d/--database` is byte-for-byte the `<name>`
    /// operand: the caller used the flag to name the entry, and a `--schema`
    /// clause would advise overwriting the server database with that name
    /// (condition 3 of `FR-CFG-051`, the fifty-third edition).
    NamesEntry,
    /// Neither: the database is set with `--schema`, and the value given to
    /// `-d/--database` is the likely one.
    Schema(&'a str),
}

/// Warns that `tpl cfg database add` or `update` was given `-d/--database`,
/// which has no effect on it (`FR-CFG-051`).
///
/// `command` is the canonical command path below `tpl`, whatever alias the
/// invocation used. `server` chooses the last clause; the value it may carry
/// is reproduced as the value of `--schema` only where the set of
/// `FR-ERR-022` admits it.
pub(crate) fn database_has_no_effect(command: &str, server: ServerDatabase<'_>) {
    if !emits(Level::Warnings) {
        return;
    }

    write_line(&database_line(command, server));
}

/// Composes the line of [`database_has_no_effect`], unescaped.
fn database_line(command: &str, server: ServerDatabase<'_>) -> String {
    let last = match server {
        ServerDatabase::Dsn => "; the database on the server is the path part of --dsn".to_owned(),
        ServerDatabase::SchemaGiven => String::new(),
        ServerDatabase::NamesEntry => "; the name argument already names the entry".to_owned(),
        ServerDatabase::Schema(given) => {
            let schema = if super::hint::admits(given) {
                given
            } else {
                "<database>"
            };
            format!("; the database on the server is set with --schema {schema}")
        }
    };
    format!(
        "{WARNING_TOKEN} -d/--database has no effect on tpl {command}; it selects the entry for \
         commands that connect{last}"
    )
}

/// Warns that a project just created shadows one in an ancestor directory.
///
/// `FR-PROJ-016` obliges the warning and fixes what it says; the two paths are
/// the whole of what it can name.
pub(crate) fn project_shadows_ancestor(created: &Path, shadowed: &Path) {
    if !emits(Level::Warnings) {
        return;
    }

    write_line(&shadow_line(created, shadowed));
}

/// Composes the line of [`project_shadows_ancestor`], unescaped.
fn shadow_line(created: &Path, shadowed: &Path) -> String {
    format!(
        "{WARNING_TOKEN} the project created at {} shadows the project at {}",
        created.display(),
        shadowed.display()
    )
}

/// Warns that `tpl cfg unset` removed the `dsn` of an entry, and with it the
/// five facts it carried (`FR-CFG-050`).
///
/// The line names the key and the entry and never any part of the dsn, per
/// `BR-ERR-003`. It is written after the rewrite succeeded.
pub(crate) fn dsn_unset(entry: &str) {
    if !emits(Level::Warnings) {
        return;
    }

    write_line(&dsn_unset_line(entry));
}

/// Composes the line of [`dsn_unset`], unescaped.
fn dsn_unset_line(entry: &str) -> String {
    format!(
        "{WARNING_TOKEN} removed database.{entry}.dsn; entry '{entry}' no longer holds the host, \
         port, user, password or database that dsn carried"
    )
}

/// Warns that a `cfg` command deleted a database entry and left the data
/// cached for it (`FR-CFG-052`).
///
/// It is written after the rewrite succeeded, and nothing under
/// `.tpl/.cache/` is read to decide it: the line is the same whether or not a
/// cache exists. `entry` was declared in `.tpl/.cfg`, so it has passed
/// `FR-CONF-048` and is reproduced under `FR-ERR-022`. The command it carries
/// carries the caller's `--tpl-dir`, per item 2.
pub(crate) fn entry_removed(entry: &str) {
    if !emits(Level::Warnings) {
        return;
    }

    let project = super::restate::tpl_dir();
    write_line(&entry_removed_line(
        entry,
        project.as_ref().map(Option::as_deref),
    ));
}

/// Composes the line of [`entry_removed`], unescaped. `project` is the
/// `--tpl-dir` the caller wrote, as [`clean_command`] takes it.
fn entry_removed_line(entry: &str, project: Option<Option<&str>>) -> String {
    let (command, placeholder) = clean_command(entry, project);
    format!(
        "{WARNING_TOKEN} removed entry '{entry}'; data cached for it under .tpl/.cache/{entry}/ \
         is kept, and an entry added later as '{entry}' reads it; clear it with: \
         {command}{placeholder}"
    )
}

/// Warns that `tpl cfg database update` repointed an entry and left the data
/// cached for it (`FR-CFG-053`).
///
/// It is written after the rewrite succeeded, and after the line of
/// `FR-CFG-051` where the invocation writes both. Nothing under
/// `.tpl/.cache/` is read to decide it. The command it carries carries the
/// caller's `--tpl-dir`, per item 2.
pub(crate) fn entry_repointed(entry: &str) {
    if !emits(Level::Warnings) {
        return;
    }

    let project = super::restate::tpl_dir();
    write_line(&entry_repointed_line(
        entry,
        project.as_ref().map(Option::as_deref),
    ));
}

/// Composes the line of [`entry_repointed`], unescaped.
fn entry_repointed_line(entry: &str, project: Option<Option<&str>>) -> String {
    let (command, placeholder) = clean_command(entry, project);
    format!(
        "{WARNING_TOKEN} entry '{entry}' was repointed; data cached for it under \
         .tpl/.cache/{entry}/ is kept, and reads still serve it; clear it with: \
         {command}{placeholder}"
    )
}

/// The placeholder item 2 of `FR-CFG-052` writes for a `--tpl-dir` value the
/// set of `FR-ERR-041` refuses.
const TPL_DIR_PATH: &str = "<path>";

/// `tpl -d <entry> cache clean`, led by `--tpl-dir` where the caller wrote
/// one (item 2 of `FR-CFG-052`), and the clause that states in words what
/// the placeholder stands for, empty where none was written.
///
/// `project` is [`None`] where no `--tpl-dir` was given, `Some(None)` where
/// its value was refused, and `Some(Some(value))` otherwise.
fn clean_command(entry: &str, project: Option<Option<&str>>) -> (String, &'static str) {
    match project {
        None => (format!("tpl -d {entry} cache clean"), ""),
        Some(Some(path)) => (format!("tpl --tpl-dir {path} -d {entry} cache clean"), ""),
        Some(None) => (
            format!("tpl --tpl-dir {TPL_DIR_PATH} -d {entry} cache clean"),
            ", where <path> is the --tpl-dir this invocation was given",
        ),
    }
}

/// Reports that `tpl cache clean` removed the data cached for a name that no
/// entry of `.tpl/.cfg` declares (`FR-CACHE-041`).
///
/// `name` has passed `FR-CONF-048`, which is condition 1 of that requirement,
/// so it is reproduced under `FR-ERR-022`. `recorded` is the name the
/// filesystem records for the folder removed, [`None`] where it could not be
/// read as text; it is reproduced only under the set of `FR-ERR-022`.
pub(crate) fn orphan_cache_removed(name: &str, recorded: Option<&str>) {
    if !emits(Level::Warnings) {
        return;
    }

    write_line(&orphan_cache_line(name, recorded));
}

/// Composes the line of [`orphan_cache_removed`], unescaped.
fn orphan_cache_line(name: &str, recorded: Option<&str>) -> String {
    let removed = match recorded {
        Some(folder) if folder == name => {
            format!("removed the data cached for that name under .tpl/.cache/{name}/")
        }
        Some(folder) if super::hint::admits(folder) => {
            format!("removed the data cached under .tpl/.cache/{folder}/")
        }
        _ => "removed the data cached in the folder of .tpl/.cache/ that the name resolved to, \
              whose name differs from the name given"
            .to_owned(),
    };
    format!("{WARNING_TOKEN} no entry '{name}' is declared in .tpl/.cfg; {removed}")
}

/// Escapes a composed line as a whole and writes it to stderr.
///
/// The escaped line is the buffer `OD-17` asks for: one locked write, and
/// stderr is unbuffered, so the line reaches the stream in one call and cannot
/// be interleaved with another writer's. A stream that refuses the write is not
/// itself reported — a diagnostic that cannot be written has nowhere to say so.
fn write_line(composed: &str) {
    let mut out = String::with_capacity(composed.len() + 1);
    escape::push_line(&mut out, composed);

    let mut stderr = std::io::stderr().lock();
    let _ = stderr.write_all(out.as_bytes());
    let _ = stderr.flush();
}

#[cfg(test)]
mod tests {
    use super::{
        CATALOGUE_QUERY_TOKEN, PHASE_TOKEN, ServerDatabase, WARNING_TOKEN, database_line,
        dsn_unset_line, entry_removed_line, entry_repointed_line, orphan_cache_line, phase_line,
        shadow_line,
    };
    use crate::deadline::Phase;
    use std::path::Path;
    use std::time::Duration;

    #[test]
    fn fr_glob_017_the_catalogue_query_token_is_distinguishable_from_every_label() {
        // FR-GLOB-017: the line must be distinguishable from every other
        // diagnostic line. The four labels of FR-ERR-008, the phase token and
        // the warning token are the whole of what else this module writes.
        for other in [
            "error: ",
            "cause: ",
            "hint:  ",
            "exit:  ",
            WARNING_TOKEN,
            PHASE_TOKEN,
        ] {
            assert!(
                !other.starts_with(CATALOGUE_QUERY_TOKEN),
                "{other:?} would be read as a catalogue-query line"
            );
            assert!(!CATALOGUE_QUERY_TOKEN.starts_with(other));
        }
    }

    #[test]
    fn fr_glob_017_a_phase_report_names_the_phase_and_how_long_it_took() {
        // FR-GLOB-017: at INFO the system reports which phases ran and how long
        // each took. The line carries both, under a token of its own.
        let line = phase_line(Phase::TcpConnect, Duration::from_micros(12_300));

        assert!(line.starts_with(PHASE_TOKEN), "{line}");
        assert!(line.contains(Phase::TcpConnect.name()), "{line}");
        assert!(line.contains("12.3ms"), "{line}");
    }

    #[test]
    fn fr_glob_017_every_phase_of_the_closed_set_reports_under_a_name_of_its_own() {
        // FR-CONF-005 closes the set at six, and a report that named two of
        // them alike would make the two indistinguishable on the stream.
        let named: Vec<&str> = [
            Phase::DnsResolution,
            Phase::TcpConnect,
            Phase::TlsHandshake,
            Phase::CatalogueQuery,
            Phase::PasswordCommand,
            Phase::Render,
        ]
        .into_iter()
        .map(Phase::name)
        .collect();

        let mut unique = named.clone();
        unique.sort_unstable();
        unique.dedup();

        assert_eq!(unique.len(), named.len(), "{named:?}");
    }

    #[test]
    fn fr_proj_016_the_warning_names_both_projects() {
        let line = shadow_line(Path::new("/work/app/.tpl"), Path::new("/work/.tpl"));

        assert!(line.starts_with(WARNING_TOKEN), "{line}");
        assert!(line.contains("/work/app/.tpl"), "{line}");
        assert!(line.contains("/work/.tpl"), "{line}");
    }

    #[test]
    fn fr_cfg_050_the_warning_names_the_key_the_entry_and_the_five_facts() {
        let line = dsn_unset_line("ds");

        assert_eq!(
            line,
            "warning: removed database.ds.dsn; entry 'ds' no longer holds the host, port, user, \
             password or database that dsn carried"
        );
    }

    #[test]
    fn fr_cfg_051_a_value_equal_to_the_entry_name_names_neither_schema_nor_a_value() {
        let line = database_line("cfg database update", ServerDatabase::NamesEntry);

        assert_eq!(
            line,
            "warning: -d/--database has no effect on tpl cfg database update; it selects the \
             entry for commands that connect; the name argument already names the entry"
        );
        assert!(!line.contains("--schema"), "{line}");
    }

    #[test]
    fn fr_cfg_052_the_warning_names_the_entry_its_cache_and_the_clean_that_removes_it() {
        assert_eq!(
            entry_removed_line("shop", None),
            "warning: removed entry 'shop'; data cached for it under .tpl/.cache/shop/ is kept, \
             and an entry added later as 'shop' reads it; clear it with: tpl -d shop cache clean"
        );
        // Item 2: the caller's --tpl-dir, before -d.
        assert_eq!(
            entry_removed_line("shop", Some(Some("/srv/shop/.tpl"))),
            "warning: removed entry 'shop'; data cached for it under .tpl/.cache/shop/ is kept, \
             and an entry added later as 'shop' reads it; clear it with: tpl --tpl-dir \
             /srv/shop/.tpl -d shop cache clean"
        );
        // A refused value is the placeholder, stated in words.
        assert!(entry_removed_line("shop", Some(None)).ends_with(
            "clear it with: tpl --tpl-dir <path> -d shop cache clean, where <path> is the \
                 --tpl-dir this invocation was given"
        ));
    }

    #[test]
    fn fr_cfg_053_the_warning_names_the_entry_its_cache_and_the_clean_that_removes_it() {
        assert_eq!(
            entry_repointed_line("shop", None),
            "warning: entry 'shop' was repointed; data cached for it under .tpl/.cache/shop/ is \
             kept, and reads still serve it; clear it with: tpl -d shop cache clean"
        );
        assert!(
            entry_repointed_line("shop", Some(Some("../w/.tpl")))
                .ends_with("clear it with: tpl --tpl-dir ../w/.tpl -d shop cache clean")
        );
        assert!(entry_repointed_line("shop", Some(None)).ends_with(
            "tpl --tpl-dir <path> -d shop cache clean, where <path> is the --tpl-dir this \
                 invocation was given"
        ));
    }

    #[test]
    fn fr_cache_041_the_line_names_the_folder_as_the_filesystem_records_it() {
        assert_eq!(
            orphan_cache_line("shop", Some("shop")),
            "warning: no entry 'shop' is declared in .tpl/.cfg; removed the data cached for that \
             name under .tpl/.cache/shop/"
        );
        assert_eq!(
            orphan_cache_line("Shop", Some("shop")),
            "warning: no entry 'Shop' is declared in .tpl/.cfg; removed the data cached under \
             .tpl/.cache/shop/"
        );
        // A recorded name outside FR-ERR-022, or none, is not reproduced.
        for recorded in [Some("sh\u{1b}op"), None] {
            let line = orphan_cache_line("Shop", recorded);
            assert!(
                line.contains("whose name differs from the name given"),
                "{line}"
            );
            assert!(!line.contains('\u{1b}'), "{line}");
        }
    }
}
