//! `tpl init`: the five artefacts of `FR-PROJ-017`, and nothing else.
//!
//! | Artefact | What it is |
//! |---|---|
//! | `.tpl/.cfg` | Mode `0600`; a `[core]` section and one commented-out `[database.*]` entry |
//! | `.tpl/.gitignore` | Two lines: `.cfg` and `.cache/` |
//! | `.tpl/templates/` | The project's template directory |
//! | `.tpl/templates/example.jinja` | A working example template |
//! | `.tpl/templates/rust/_types.jinja` | A macro file mapping a column to a Rust type |
//!
//! `.tpl/.cache/` is deliberately **not** among them, per `FR-PROJ-020`: it is
//! per-machine state derived from a server and does not exist until a read
//! populates it.
//!
//! The command writes nothing to stdout and exits `0`, per `FR-PROJ-022` and
//! `BR-CLI-004`. The one thing it may write is the warning of `FR-PROJ-016`, on
//! stderr, when the project it creates shadows one in an ancestor directory.
//!
//! It is the one command that creates anything outside `.tpl`, and the
//! exception is enumerated rather than general: `FR-PROJ-024` admits the
//! destination directory and its missing parents, per `FR-PROJ-013`, and no
//! file.
//!
//! It is also the one command that reaches `73`, per `FR-ERR-003`: a `.tpl`
//! already at the destination is `FR-PROJ-014` and a destination that cannot be
//! created is `FR-PROJ-015`.

use std::fs;
use std::io;
use std::os::unix::fs::OpenOptionsExt as _;
use std::path::Path;

use super::discover::{self, MARKER};
use super::edit::{CONFIGURATION, MODE};
use crate::diagnostics::emit;
use crate::error::Error;

/// The template directory inside `.tpl` (`FR-PROJ-017`).
///
/// It is the template root of `FR-TMPL-023`, so `crate::render` composes its
/// own boundary from this constant rather than from a second spelling of the
/// same directory: this module writes it, that one reads it, and one name
/// keeps the two from parting.
pub(crate) const TEMPLATES: &str = "templates";

/// The file `.tpl/.gitignore`, which keeps `.cfg` and `.cache/` out of version
/// control (`FR-PROJ-003`).
///
/// The second line is `FR-CACHE-004`, which requires the file this command
/// writes to exclude `.cache/` and defers the requirement itself to
/// `FR-PROJ-017`. The store holds a catalogue read from a server with one
/// machine's credentials, so it is per-machine state that must not travel with
/// the repository, and the exclusion is written **here** rather than left to
/// the caller's own `.gitignore`: `FR-CACHE-003` makes the folder appear on the
/// first read that populates it, long after anyone would have thought to
/// exclude it.
const GITIGNORE: &str = ".cfg\n.cache/\n";

/// The generated `.tpl/.cfg` (`FR-PROJ-017`, `FR-PROJ-018`).
///
/// It carries a `[core]` section and one commented-out `[database.*]` entry
/// showing the exact shape a real entry takes, and **no active database
/// entry**: a fresh project knows about no database until one is added.
///
/// The `[core]` header is the **last** line of the file, and the commented
/// example stands above it. That is not decoration: `FR-CFG-041` rewrites the
/// file through a parser that keeps every comment, and a comment after the last
/// key of the document is the document's trailer — so a header written last
/// leaves the first `tpl cfg set` writing its key directly beneath it, rather
/// than above a block of commented text that the rewrite would then push below
/// the entry it describes.
const CONFIGURATION_TEMPLATE: &str = "\
# The configuration of this project. `tpl help cfg` lists the commands that
# maintain it, and `tpl help cfg set` the keys it admits.
#
# This file is not versioned: it is per-machine access configuration and may
# hold credentials. `.tpl/.gitignore` excludes it.
#
# Everything below is commented out, and every commented key is shown with its
# default, or with an example where it has none. A fresh project knows about no
# database until one is added.
#
# The keys of [core]:
#
#   database            = \"shop\"     the entry every invocation uses when
#                                    -d/--database is absent; no default
#   connect_timeout     = 10         seconds, shared by DNS, TCP and TLS
#   query_timeout       = 30         seconds, per query reading the database
#                                    structure
#   password_timeout    = 5          seconds, for password_command
#   render_timeout      = 30         seconds, for one render
#   render_fuel         = 100000000  evaluation steps, for one render
#   render_output_limit = 67108864   bytes, for one render
#   render_memory_limit = 134217728  bytes of heap, while one render runs
#
# A database entry, in the two shapes it may take. Copy one, name it, and
# uncomment it — or let `tpl cfg database add` write it for you.
#
#   [database.shop]
#   dsn = \"mysql://alice:${SHOP_DB_PASSWORD}@db.example.com:3306/shop\"
#
#   [database.reporting]
#   host             = \"10.0.1.5\"
#   port             = 3306
#   user             = \"reader\"
#   database         = \"reporting\"
#   tls              = \"verify-identity\"
#   password_command = [\"security\", \"find-generic-password\", \"-s\", \"tpl\", \"-w\"]

[core]
";

/// The generated `.tpl/templates/example.jinja` (`FR-PROJ-017`,
/// `FR-PROJ-021`).
///
/// It renders against any table of any supported MariaDB database, walks the
/// columns of that table, uses the `snake` and `pascal` filters of
/// `FR-ENV-006`, the `length` filter of `FR-ENV-018` and the `nullable` test of
/// `FR-ENV-014`, and carries in its header the command that runs it.
const EXAMPLE_TEMPLATE: &str = r#"{#
  A worked example of what a template receives. Run it with:

      tpl render example --table <table>

  It walks the columns of the table named by --table and depends on nothing
  a particular schema carries, so it renders against any table of any
  supported server.
#}
// {{ table.name | pascal }}: {{ table.columns | length }} column(s), read from
// the database {{ database.name }}.
pub struct {{ table.name | pascal }} {
{%- for column in table.columns %}
    /// {{ column.column_type }}{% if column is nullable %}, nullable{% endif %}
    pub {{ column.name | snake }}: (),
{%- endfor %}
}
"#;

/// The generated `.tpl/templates/rust/_types.jinja` (`FR-PROJ-017`,
/// `FR-ENV-011`).
///
/// `FR-ENV-009` removes the `rust_type` filter from the binary and `BR-ENV-002`
/// gives the reason: a type mapping is an opinion, and it belongs to the
/// project that holds it. This file is that opinion, delivered editable.
const TYPES_TEMPLATE: &str = r#"{#
  The column-to-Rust-type mapping this project owns.

  `tpl` registers no `rust_type` filter: whether DECIMAL becomes a third-party
  decimal type, an f64 or a String is this project's decision, and it is made
  here. Edit it.

  Import it by its full name — a template name is literal, so the `.jinja` is
  written out:

      {% import "rust/_types.jinja" as types %}
      {{ types.of(column) }}
#}
{% macro of(column) -%}
{%- set mapped -%}
    {%- if column.data_type in ["tinyint", "smallint", "mediumint", "int", "year"] -%}
        {%- if column.unsigned -%}u32{%- else -%}i32{%- endif -%}
    {%- elif column.data_type == "bigint" -%}
        {%- if column.unsigned -%}u64{%- else -%}i64{%- endif -%}
    {%- elif column.data_type in ["float", "double"] -%}f64
    {%- elif column.data_type == "decimal" -%}String
    {%- elif column.data_type in ["bit", "binary", "varbinary", "tinyblob", "blob", "mediumblob", "longblob"] -%}Vec<u8>
    {%- elif column.data_type in ["date", "time", "datetime", "timestamp"] -%}String
    {%- elif column.data_type == "bool" -%}bool
    {%- else -%}String
    {%- endif -%}
{%- endset -%}
{%- if column is nullable -%}Option<{{ mapped }}>{%- else -%}{{ mapped }}{%- endif -%}
{%- endmacro %}
"#;

/// Creates a project at `destination`.
///
/// # Errors
///
/// Returns [`Error::ProjectAlreadyExists`] where the destination already holds
/// a `.tpl` (`FR-PROJ-014`), and [`Error::ProjectNotCreated`] where the
/// destination or any of the five artefacts could not be created
/// (`FR-PROJ-015`).
pub(crate) fn create(destination: &Path) -> Result<(), Error> {
    // FR-PROJ-029, when the destination is first examined: a destination that
    // names a `.tpl` folder is refused before anything else is asked of it.
    refuse_tpl_folder(destination)?;

    let marker = destination.join(MARKER);

    // FR-PROJ-014, before anything is created: a destination that already holds
    // a `.tpl` is refused with nothing changed. It does not merge, complete
    // partially, or overwrite.
    if marker.symlink_metadata().is_ok() {
        return Err(Error::ProjectAlreadyExists { path: marker });
    }

    // FR-PROJ-016, as amended in the forty-ninth edition: the walk starts at
    // the deepest directory that exists before anything is created, so a
    // `.tpl` folder this invocation creates as a missing parent is never
    // reported as a project above the destination. Between that directory and
    // the destination there is nothing yet, so no pre-existing project is
    // missed.
    let shadowed =
        deepest_existing(destination).and_then(|existing| discover::locate(None, existing).ok());

    // FR-PROJ-013, and the one exception FR-PROJ-024 enumerates: the
    // destination and its missing parents, which are directories and never a
    // file.
    fs::create_dir_all(destination).map_err(|returned| Error::ProjectNotCreated {
        path: destination.to_owned(),
        returned,
    })?;

    write(&marker, destination)?;

    if let Some(above) = shadowed {
        emit::project_shadows_ancestor(&marker, &above);
    }

    Ok(())
}

/// The deepest of `destination` and its ancestors that exists, or [`None`]
/// where none does.
///
/// A relative destination whose every segment is missing is resolved against
/// the current directory, which is its implicit first ancestor.
fn deepest_existing(destination: &Path) -> Option<&Path> {
    destination
        .ancestors()
        .map(|ancestor| {
            if ancestor.as_os_str().is_empty() {
                Path::new(".")
            } else {
                ancestor
            }
        })
        .find(|ancestor| ancestor.is_dir())
}

/// Refuses a destination that names a `.tpl` folder (`FR-PROJ-029`).
///
/// Two tests, in the order the requirement gives them: the last segment of
/// the path as written, where [`Path::file_name`] already drops trailing
/// separators and `.` segments; and, where the destination exists, the last
/// segment of its canonical path, which is the case of `tpl init` run inside a
/// `.tpl` folder.
///
/// # Errors
///
/// Returns [`Error::InitDestinationIsTplFolder`] with the parent directory the
/// hint names, or [`None`] for it where that parent is the current directory.
fn refuse_tpl_folder(destination: &Path) -> Result<(), Error> {
    let is_marker = |path: &Path| path.file_name() == Some(std::ffi::OsStr::new(MARKER));

    if is_marker(destination) {
        let parent = destination
            .parent()
            .filter(|parent| {
                parent
                    .components()
                    .any(|component| component != std::path::Component::CurDir)
            })
            .map(Path::to_path_buf);

        return Err(Error::InitDestinationIsTplFolder {
            written: destination.to_owned(),
            canonical: None,
            parent,
        });
    }

    let Ok(canonical) = fs::canonicalize(destination) else {
        return Ok(());
    };
    if !is_marker(&canonical) {
        return Ok(());
    }

    let here = std::env::current_dir()
        .ok()
        .and_then(|here| fs::canonicalize(here).ok());
    let parent = canonical
        .parent()
        .filter(|parent| here.as_deref() != Some(*parent))
        .map(Path::to_path_buf);

    Err(Error::InitDestinationIsTplFolder {
        written: destination.to_owned(),
        canonical: Some(canonical),
        parent,
    })
}

/// Writes the five artefacts under `marker`.
///
/// # Errors
///
/// Returns [`Error::ProjectNotCreated`] naming the artefact that could not be
/// written (`FR-PROJ-015`).
fn write(marker: &Path, destination: &Path) -> Result<(), Error> {
    let failed = |path: &Path, returned: io::Error| Error::ProjectNotCreated {
        path: path.to_owned(),
        returned,
    };

    let templates = marker.join(TEMPLATES);
    let rust = templates.join("rust");

    // `create_dir_all` of the deepest directory makes `.tpl` and
    // `.tpl/templates` on the way, and FR-PROJ-020 keeps `.tpl/.cache/` out of
    // the set: nothing here names it.
    fs::create_dir_all(&rust).map_err(|returned| failed(destination, returned))?;

    // FR-PROJ-019: the configuration is created at 0600, by the open itself,
    // so it never exists at the umask's mode.
    let configuration = marker.join(CONFIGURATION);
    fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .mode(MODE)
        .open(&configuration)
        .and_then(|mut handle| {
            std::io::Write::write_all(&mut handle, CONFIGURATION_TEMPLATE.as_bytes())
        })
        .map_err(|returned| failed(&configuration, returned))?;

    for (path, contents) in [
        (marker.join(".gitignore"), GITIGNORE),
        (templates.join("example.jinja"), EXAMPLE_TEMPLATE),
        (rust.join("_types.jinja"), TYPES_TEMPLATE),
    ] {
        fs::write(&path, contents).map_err(|returned| failed(&path, returned))?;
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::{CONFIGURATION_TEMPLATE, MARKER, create};
    use crate::error::Error;
    use crate::project::edit::MODE;
    use crate::project::scratch::Scratch;
    use std::path::Path;

    /// Every path under `root`, relative to it, sorted.
    fn tree(root: &Path) -> Vec<String> {
        fn walk(at: &Path, base: &Path, found: &mut Vec<String>) {
            let Ok(entries) = std::fs::read_dir(at) else {
                return;
            };
            for entry in entries.flatten() {
                let path = entry.path();
                let relative = path
                    .strip_prefix(base)
                    .unwrap_or(&path)
                    .to_string_lossy()
                    .into_owned();
                found.push(relative);
                if path.is_dir() {
                    walk(&path, base, found);
                }
            }
        }

        let mut found = Vec::new();
        walk(root, root, &mut found);
        found.sort();

        found
    }

    #[test]
    fn fr_proj_017_init_creates_exactly_the_five_artefacts_the_requirement_names() {
        // FR-PROJ-017, FR-PROJ-020: five artefacts, and no `.cache/`.
        let scratch = Scratch::new();
        let destination = scratch.path("project");

        create(&destination).expect("the project is created");

        assert_eq!(
            tree(&destination.join(MARKER)),
            [
                ".cfg",
                ".gitignore",
                "templates",
                "templates/example.jinja",
                "templates/rust",
                "templates/rust/_types.jinja",
            ]
        );
    }

    #[test]
    fn fr_proj_019_the_configuration_is_created_at_six_hundred() {
        // FR-PROJ-019.
        let scratch = Scratch::new();
        let destination = scratch.path("project");

        create(&destination).expect("the project is created");

        assert_eq!(scratch.mode(&destination.join(MARKER).join(".cfg")), MODE);
    }

    #[test]
    fn fr_proj_018_the_generated_configuration_carries_no_active_database_entry() {
        // FR-PROJ-018: a fresh project knows about no database until one is
        // added, and the commented example shows the shape all the same.
        let scratch = Scratch::new();
        let destination = scratch.path("project");

        create(&destination).expect("the project is created");
        let written = std::fs::read_to_string(destination.join(MARKER).join(".cfg"))
            .expect("the file is there");

        assert_eq!(written, CONFIGURATION_TEMPLATE);

        let document = crate::project::config::load(&destination.join(MARKER).join(".cfg"))
            .expect("the generated file is valid");
        assert_eq!(document.names().len(), 0);
        assert!(document.core().database.is_none());
        assert!(document.keys().is_empty());
    }

    #[test]
    fn fr_proj_017_the_gitignore_carries_the_two_lines_the_requirement_names() {
        // FR-PROJ-017, FR-PROJ-003.
        let scratch = Scratch::new();
        let destination = scratch.path("project");

        create(&destination).expect("the project is created");

        assert_eq!(
            std::fs::read_to_string(destination.join(MARKER).join(".gitignore"))
                .expect("the file is there"),
            ".cfg\n.cache/\n"
        );
    }

    #[test]
    fn fr_proj_021_the_example_template_names_the_command_that_runs_it_and_walks_the_columns() {
        // FR-PROJ-021: the header carries the command, the body walks the
        // columns, and at least one filter and one test are used.
        let scratch = Scratch::new();
        let destination = scratch.path("project");

        create(&destination).expect("the project is created");
        let example = std::fs::read_to_string(
            destination
                .join(MARKER)
                .join("templates")
                .join("example.jinja"),
        )
        .expect("the file is there");

        assert!(example.contains("tpl render example --table"), "{example}");
        assert!(example.contains("for column in table.columns"), "{example}");
        assert!(example.contains("| pascal"), "{example}");
        assert!(example.contains("is nullable"), "{example}");
    }

    #[test]
    fn fr_proj_017_the_macro_file_maps_a_column_to_a_rust_type() {
        // FR-PROJ-017, FR-ENV-011: the mapping the binary no longer performs.
        let scratch = Scratch::new();
        let destination = scratch.path("project");

        create(&destination).expect("the project is created");
        let types = std::fs::read_to_string(
            destination
                .join(MARKER)
                .join("templates")
                .join("rust")
                .join("_types.jinja"),
        )
        .expect("the file is there");

        assert!(types.contains("macro of(column)"), "{types}");
        assert!(types.contains("column.data_type"), "{types}");
        assert!(types.contains("Option<"), "{types}");
    }

    #[test]
    fn fr_proj_013_init_creates_the_destination_and_its_missing_parents() {
        // FR-PROJ-013.
        let scratch = Scratch::new();
        let destination = scratch.path("a/b/c");

        create(&destination).expect("the project is created");

        assert!(destination.join(MARKER).is_dir());
    }

    #[test]
    fn fr_proj_014_a_destination_that_already_holds_a_project_is_refused_with_nothing_changed() {
        // FR-PROJ-014: 73, and it does not merge, complete partially, or
        // overwrite.
        let scratch = Scratch::new();
        let destination = scratch.directory("project");
        let marker = scratch.directory("project/.tpl");
        let kept = scratch.file("project/.tpl/.cfg", "# written by hand\n");

        let condition = create(&destination).expect_err("a project is already there");

        assert!(matches!(condition, Error::ProjectAlreadyExists { .. }));
        assert_eq!(condition.exit_code(), 73);
        assert_eq!(
            std::fs::read_to_string(&kept).expect("the file is untouched"),
            "# written by hand\n"
        );
        assert_eq!(tree(&marker), [".cfg"]);
    }

    #[test]
    fn fr_proj_015_a_destination_that_cannot_be_created_is_refused() {
        // FR-PROJ-015: 73.
        let scratch = Scratch::new();
        let blocking = scratch.file("a-file", "");
        let destination = blocking.join("under-a-file");

        let condition = create(&destination).expect_err("the destination cannot be created");

        assert!(matches!(condition, Error::ProjectNotCreated { .. }));
        assert_eq!(condition.exit_code(), 73);
    }

    #[test]
    fn fr_proj_016_a_nested_project_is_created_and_shadows_the_one_above_it() {
        // FR-PROJ-016: the nested project is created and the invocation
        // succeeds; the warning goes to stderr, which NFR-DET-001 puts outside
        // the contract.
        let scratch = Scratch::new();
        scratch.directory("outer/.tpl");
        let inner = scratch.path("outer/inner");

        create(&inner).expect("the nested project is created");

        assert!(inner.join(MARKER).is_dir());
    }

    #[test]
    fn fr_proj_001_a_project_discovered_after_init_is_the_one_init_created() {
        // FR-PROJ-001, FR-PROJ-004: what init writes is what discovery finds.
        let scratch = Scratch::new();
        let destination = scratch.directory("project");

        create(&destination).expect("the project is created");
        let found = crate::project::discover::locate(None, &destination)
            .expect("the project is discovered");

        assert_eq!(found, scratch.canonical("project").join(MARKER));
    }
}
