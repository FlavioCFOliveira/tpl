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

use super::escape;
use super::verbosity::{Level, emits};

/// The fixed leading token of the one line per catalogue query.
///
/// `FR-GLOB-017` requires exactly one line per catalogue query issued,
/// distinguishable from every other diagnostic line, and `NFR-PERF-008` makes
/// the query count observable from outside the process through it — which is
/// what lets `NFR-PERF-001` and `NFR-PERF-002` be checked at all. One function
/// writes this token and no other line begins with it.
const CATALOGUE_QUERY_TOKEN: &str = "query:";

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
#[allow(
    dead_code,
    reason = "the catalogue reader that calls this is a later sprint; the function is written \
              here because FR-GLOB-017 and OD-17 place the emission set in this module"
)]
pub(crate) fn catalogue_query() {
    if !emits(Level::Info) {
        return;
    }

    let mut composed = String::with_capacity(LINE_CAPACITY);
    composed.push_str(CATALOGUE_QUERY_TOKEN);
    composed.push_str(" read INFORMATION_SCHEMA");

    write_line(&composed);
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
    use super::{CATALOGUE_QUERY_TOKEN, WARNING_TOKEN, shadow_line};
    use std::path::Path;

    #[test]
    fn fr_glob_017_the_catalogue_query_token_is_distinguishable_from_every_label() {
        // FR-GLOB-017: the line must be distinguishable from every other
        // diagnostic line. The four labels of FR-ERR-008 and the warning token
        // are the whole of what else this module writes.
        for other in ["error: ", "cause: ", "hint:  ", "exit:  ", WARNING_TOKEN] {
            assert!(
                !other.starts_with(CATALOGUE_QUERY_TOKEN),
                "{other:?} would be read as a catalogue-query line"
            );
            assert!(!CATALOGUE_QUERY_TOKEN.starts_with(other));
        }
    }

    #[test]
    fn fr_proj_016_the_warning_names_both_projects() {
        let line = shadow_line(Path::new("/work/app/.tpl"), Path::new("/work/.tpl"));

        assert!(line.starts_with(WARNING_TOKEN), "{line}");
        assert!(line.contains("/work/app/.tpl"), "{line}");
        assert!(line.contains("/work/.tpl"), "{line}");
    }
}
