//! The second arm of the tree: `tpl template`, which reads the project's
//! templates.
//!
//! The node is a group, per `FR-CLI-008`: no action of its own
//! (`FR-CLI-009`), an optional child, and its own help at exit `0` when
//! invoked bare (`FR-CLI-007`). None of its four children carries an alias —
//! `FR-CLI-011` declares seven aliases and none of them is here.
//!
//! # The surface of the four
//!
//! `FR-GLOB-021` gives `--format`, and with it `--pretty`, to `list` and
//! `path` and to neither of the other two: `show` prints template source
//! unaltered, per `FR-TMPL-015`, and `check` reports through its exit code, per
//! `FR-TMPL-020`. Neither has a second representation to choose between, so
//! `tpl template show x --format json` is the ordinary unknown-flag `64` of
//! `FR-CLI-019`.
//!
//! The four positional arguments differ in arity, and each difference is a
//! requirement rather than a convenience: `show` takes exactly one
//! (`FR-TMPL-016`); `check` takes any number, checking every template of the
//! project when given none (`FR-TMPL-018`, `FR-TMPL-019`); `path` takes at most
//! one, printing the template root when given none (`FR-TMPL-021`,
//! `FR-TMPL-022`); `list` takes none.
//!
//! A name is resolved by `FR-TMPL-006` and `FR-TMPL-007` throughout: it is the
//! path of the file relative to `.tpl/templates/`, with or without the `.jinja`
//! extension. That resolution is the command's work, not the parser's, so every
//! name here is a string.
//!
//! # What the four do, and what none of them does
//!
//! Every one of them is `.tpl` and nothing else. [`environment`] performs steps
//! 2 and 3 of `FR-ERR-006` — discovery, and the two trust checks — and stops
//! there: `.tpl/.cfg` is not read for its keys, no database entry is resolved,
//! and no connection is opened, which is `FR-TMPL-003` held by what is absent
//! rather than by a guard. `BR-TMPL-002` is held the same way: nothing here
//! writes.
//!
//! **The catalogue cache is absent for the same reason.** `FR-CACHE-011`
//! forbids every subcommand of this arm to contact a database **or** to touch
//! the store, and no code path here reaches [`crate::mariadb`] or
//! [`crate::cache`]: there is no [`Cache`](crate::cache::Cache) to consult,
//! nothing to write on a miss, and no entry to key one by. The half about the
//! store is the half a connection record cannot see, and it holds here because
//! the module has no route to `.tpl/.cache/` at all.
//!
//! **Three of the four build no engine.** [`crate::render::Environment`] holds
//! the engine in a cell filled on first use, and `list`, `show` and `path`
//! reach the template root alone, so the one invocation that compiles anything
//! is `check` — which is `NFR-PERF-006` and the project's lazy-initialisation
//! rule.
//!
//! **`check` evaluates nothing.** It compiles, which is syntax analysis, per
//! `FR-TMPL-017`; `BR-TMPL-001` makes that a guarantee rather than a side
//! effect, so a template ending in `fail` passes and a template that reads an
//! undefined variable passes.
//!
//! | Subcommand | What reaches stdout | Forced by |
//! |---|---|---|
//! | `list` | The aligned column of `FR-OUT-006`, or the `templates` array of `FR-TMPL-028` | `FR-TMPL-011` … `FR-TMPL-014` |
//! | `show` | The template's bytes, unescaped and unterminated | `FR-TMPL-015`, `FR-OUT-019` |
//! | `check` | Nothing at all; the exit code is the answer | `FR-TMPL-020`, `BR-CLI-004` |
//! | `path` | One absolute path, or the `path` key of `FR-TMPL-029` | `FR-TMPL-021`, `FR-TMPL-022` |

use std::borrow::Cow;
use std::io::Write;
use std::path::{Path, PathBuf};

use clap::{Args, Subcommand};
use serde::Serialize;

use super::globals::Globals;
use super::local::{self, Format};
use crate::error::Error;
use crate::output::{self, Collection, Document, Form, Order, Source, Table};
use crate::project::Project;
use crate::render::Environment;

/// The `tpl template` group node.
#[derive(Debug, Clone, PartialEq, Eq, Args)]
pub(crate) struct Template {
    /// The subcommand invoked, or [`None`] where the group node was invoked
    /// bare.
    #[command(subcommand)]
    pub(crate) command: Option<Command>,
}

/// The four children of `tpl template`.
#[derive(Debug, Clone, PartialEq, Eq, Subcommand)]
pub(crate) enum Command {
    /// Lists the templates of the project.
    List {
        /// `--format` and `--pretty`, per `FR-GLOB-021`.
        #[command(flatten)]
        output: local::Output,
    },

    /// Prints the source of one template.
    Show {
        /// The template to print, with or without its `.jinja` extension
        /// (`FR-TMPL-007`, `FR-TMPL-016`).
        ///
        /// A name that does not resolve is `66`, with a nearest-match
        /// suggestion over the template names that do exist, per
        /// `FR-TMPL-027`.
        #[arg(value_name = "NAME")]
        name: String,
    },

    /// Checks the syntax of templates without running them.
    Check {
        /// The templates to check, with or without their `.jinja` extension.
        ///
        /// Repeatable. Given none, every template of the project is checked,
        /// per `FR-TMPL-018`; given one or more, exactly those are, per
        /// `FR-TMPL-019`. The check is syntax analysis only and evaluates
        /// nothing, per `FR-TMPL-017`.
        #[arg(value_name = "NAME")]
        names: Vec<String>,
    },

    /// Prints the path of a template, or of the template folder.
    Path {
        /// The template whose path to print, with or without its `.jinja`
        /// extension (`FR-TMPL-022`).
        ///
        /// Optional: given none, the absolute path of the template root is
        /// printed instead, per `FR-TMPL-021`.
        #[arg(value_name = "NAME")]
        name: Option<String>,

        /// `--format` and `--pretty`, per `FR-GLOB-021`.
        #[command(flatten)]
        output: local::Output,
    },
}

/// The header of the `text` listing of `tpl template list`.
///
/// One column, and the name is the whole of it. `FR-OUT-006` asks a listing to
/// carry the useful information rather than only the name, and a template has
/// no second property this specification fixes: where it lives is
/// `tpl template path` and what it says is `tpl template show`. It is the shape
/// `tpl cfg database list` takes, for the same reason.
const NAMES: [&str; 1] = ["NAME"];

/// The plural key the `data` of `tpl template list` carries (`FR-OUT-030`,
/// `FR-TMPL-028`).
const TEMPLATES: &str = "templates";

/// One member of the `templates` array of `FR-TMPL-028`.
///
/// An object rather than a bare string, for the reason that requirement gives:
/// `FR-OUT-014` makes adding a field the only non-breaking way for a listing to
/// grow, and a string cannot gain one.
#[derive(Debug, Serialize)]
struct Named<'a> {
    /// The displayed name of `FR-TMPL-011`, which `FR-TMPL-012` makes usable
    /// verbatim as a positional argument.
    name: &'a str,
}

/// The `data` of `tpl template path` (`FR-TMPL-029`).
///
/// One key, whose value is the absolute path `FR-TMPL-021` or `FR-TMPL-022`
/// would print — the same bytes in both representations, so a caller reading
/// `data.path` and a caller reading stdout are told the same thing.
#[derive(Debug, Serialize)]
struct Located<'a> {
    /// The absolute path.
    path: &'a str,
}

/// Runs one `template` subcommand.
///
/// The project is opened once, before the match, because all four need it and
/// none needs anything else: `FR-TMPL-003` withholds the catalogue, the cache
/// and the database entry from every one of them, and `FR-TMPL-023` makes
/// `.tpl/templates/` of the resolved project the boundary all four work inside.
///
/// # Errors
///
/// Returns what [`environment`] returns — the `78` of a project that is not
/// found or a `.tpl/.cfg` that cannot be used — and then, per subcommand: the
/// `74` of a template root that could not be walked or a stream that refused
/// the write, the `66` of `FR-TMPL-027` for a name that resolves to nothing,
/// the `65` of `FR-TMPL-026` for one that resolves outside the root, and the
/// `65` of `FR-TMPL-020` for a checked template the engine could not parse.
pub(crate) fn run<W: Write>(
    out: &mut W,
    globals: &Globals,
    command: &Command,
) -> Result<(), Error> {
    let environment = environment(globals)?;

    match command {
        Command::List { output } => list(out, &environment, output),
        Command::Show { name } => show(out, &environment, name),
        Command::Check { names } => check(&environment, names),
        Command::Path { name, output } => path(out, &environment, name.as_deref(), output),
    }
}

/// The render environment of the project this invocation acts on.
///
/// It is steps 2 and 3 of `FR-ERR-006` and then nothing:
/// [`Environment::new`](crate::render::Environment::new) composes the template
/// root and reads no file, so a subcommand that resolves a path has built no
/// engine. The configuration is read and validated, and then dropped:
/// `FR-TMPL-003` requires no database entry to be selected, yet a `.cfg` that
/// is not TOML, holds an unknown key or a value of the wrong type ends the
/// invocation with `78` before any template is resolved, as for every command
/// outside `FR-PROJ-025`.
///
/// # Errors
///
/// Returns what [`Project::current`] and [`Project::configuration`] return:
/// the `78` of a project that is not found or not trusted, and of a `.cfg`
/// that fails validation.
fn environment(globals: &Globals) -> Result<Environment, Error> {
    let project = Project::current(globals.tpl_dir.first().map(PathBuf::as_path))?;
    project.configuration()?;

    Ok(Environment::new(project.root()))
}

/// `tpl template list` (`FR-TMPL-011` … `FR-TMPL-014`, `FR-TMPL-028`).
///
/// The listing arrives in the order `FR-TMPL-013` fixes, and both
/// representations present it in that order: the `json` one because
/// `FR-TMPL-028`'s composition note requires the `templates` array to carry it,
/// and the `text` one because [`Order::ByName`] is that same rule declared at
/// the call site — the displayed name is the first column, so the layout orders
/// the rows by the column a reader sees them sorted on.
///
/// A project with no template writes the header row and nothing beneath it in
/// `text`, and `"templates":[]` in `json`, and exits `0` — which is
/// `FR-TMPL-031` through `FR-OUT-033`, `FR-OUT-034` and `FR-OUT-035`, none of
/// them a branch here.
///
/// # Errors
///
/// Returns what [`Environment::templates`](crate::render::Environment::templates)
/// returns, and the write conditions of [`output`].
fn list<W: Write>(
    out: &mut W,
    environment: &Environment,
    output: &local::Output,
) -> Result<(), Error> {
    let templates = environment.templates()?;
    let (format, form) = representation(output);

    match format {
        Format::Json => {
            let members: Vec<Named<'_>> = templates
                .iter()
                .map(|template| Named {
                    name: template.displayed(),
                })
                .collect();

            output::emit_to(
                out,
                &Document::new(Source::Project, Collection::new(TEMPLATES, &members)),
                form,
            )
        }
        Format::Text => {
            let rows: Vec<[&str; 1]> = templates
                .iter()
                .map(|template| [template.displayed()])
                .collect();

            output::emit_table_to(out, &Table::new(NAMES, &rows, Order::ByName))
        }
    }
}

/// `tpl template show <name>` (`FR-TMPL-015`, `FR-TMPL-016`).
///
/// The source reaches stdout unaltered: the escaping of `FR-OUT-018` does not
/// apply to it, per `FR-OUT-019`, and nothing is appended — a template that
/// ends without a newline is printed without one, which is what "unaltered"
/// leaves no room to decide otherwise.
///
/// The file is read as text rather than as bytes because the engine reads it as
/// text: a template that is not valid UTF-8 is one no render could compile, so
/// refusing it here with the `74` of a file that could not be read is the same
/// answer the render path gives it.
///
/// # Errors
///
/// Returns what [`Environment::resolve`](crate::render::Environment::resolve)
/// returns, [`Error::ProjectFileUnreadable`] where the file could not be read,
/// and the write conditions of [`output`].
fn show<W: Write>(out: &mut W, environment: &Environment, name: &str) -> Result<(), Error> {
    let located = environment.resolve(name)?;
    let source =
        std::fs::read_to_string(&located).map_err(|returned| Error::ProjectFileUnreadable {
            path: located,
            returned,
        })?;

    output::emit_verbatim(out, &source)
}

/// `tpl template check [<name> …]` (`FR-TMPL-017` … `FR-TMPL-020`,
/// `FR-TMPL-032`).
///
/// Given no name it checks every template of the project, per `FR-TMPL-018`;
/// given names it checks exactly those, per `FR-TMPL-019`. Nothing reaches
/// stdout in either case: `FR-TMPL-020` reports through the exit code and
/// `BR-CLI-004` makes the `0` the message.
///
/// **Every selected template is checked before anything is reported**, per
/// `FR-TMPL-032`, so one invocation names every template with a syntax error.
/// The order is fixed: the population arrives in the order `FR-TMPL-013` fixes
/// and the named form is checked in the order the caller wrote, each template
/// once, at its first position. Every positional name is resolved before any
/// template is checked, because resolution is step 4 of `FR-ERR-006` and a
/// name that resolves to nothing is reported alone.
///
/// A condition other than a syntax error stops the check at once and is the
/// only one reported, per point 5 of `FR-TMPL-032`: the syntax errors met
/// before it are dropped, so every `exit` line written is the code returned.
///
/// # Errors
///
/// Returns what [`Environment::templates`](crate::render::Environment::templates),
/// [`Environment::resolve`](crate::render::Environment::resolve) and
/// [`Environment::compile`](crate::render::Environment::compile) return: the
/// `66` of a name that resolves to nothing, the `65` of one that resolves
/// outside the root, and the `65` of `FR-TMPL-020`. Where several templates
/// have a syntax error, every one but the last is reported here and the last
/// is returned, for [`crate::run`] to report as it reports every condition.
fn check(environment: &Environment, names: &[String]) -> Result<(), Error> {
    let mut failures = Vec::new();

    if names.is_empty() {
        // FR-TMPL-018, and FR-TMPL-031 where the project carries none: an
        // empty population checks nothing and falls through to `Ok(())`.
        for template in environment.templates()? {
            syntax_only(environment.compile(template.name()), &mut failures)?;
        }
    } else {
        // FR-TMPL-019: exactly those, each named as the caller wrote it so that
        // a refusal reproduces the caller's own spelling. `x` and `x.jinja`
        // are one template, so a repetition is recognised by its path.
        let mut seen = Vec::with_capacity(names.len());
        let mut selected = Vec::with_capacity(names.len());
        for name in names {
            let path = environment.resolve(name)?;
            if !seen.contains(&path) {
                seen.push(path);
                selected.push(name);
            }
        }
        for name in selected {
            syntax_only(environment.compile(name), &mut failures)?;
        }
    }

    let Some(last) = failures.pop() else {
        return Ok(());
    };
    for failure in &failures {
        crate::diagnostics::report(failure);
    }
    Err(last)
}

/// Keeps a syntax error of `FR-TMPL-020` for the report of `FR-TMPL-032`, and
/// passes every other condition on, which stops the check.
fn syntax_only(outcome: Result<(), Error>, failures: &mut Vec<Error>) -> Result<(), Error> {
    match outcome {
        Err(failure @ Error::TemplateSyntax { .. }) => {
            failures.push(failure);
            Ok(())
        }
        other => other,
    }
}

/// `tpl template path [<name>]` (`FR-TMPL-021`, `FR-TMPL-022`,
/// `FR-TMPL-029`).
///
/// With no name it writes the template root, which is absolute because the
/// project's own `.tpl` is canonical; with one, the canonical path of that
/// template's file. The root is **not** required to exist for the first form:
/// `FR-TMPL-021` asks where templates would be read from, and a project that
/// has not been given a template directory still has an answer.
///
/// A path that is not valid UTF-8 is written with the replacement character, on
/// the same terms `FR-OUT-017` fixes for a catalogue value: neither
/// representation carries an arbitrary byte sequence, and refusing the whole
/// command would withhold an answer the caller can act on.
///
/// # Errors
///
/// Returns what [`Environment::resolve`](crate::render::Environment::resolve)
/// returns for a name that was given, and the write conditions of [`output`].
fn path<W: Write>(
    out: &mut W,
    environment: &Environment,
    name: Option<&str>,
    output: &local::Output,
) -> Result<(), Error> {
    let (format, form) = representation(output);
    let located: Cow<'_, Path> = match name {
        None => Cow::Borrowed(environment.root()),
        Some(name) => Cow::Owned(environment.resolve(name)?),
    };
    let written = located.to_string_lossy();

    match format {
        Format::Json => output::emit_to(
            out,
            &Document::new(Source::Project, Located { path: &written }),
            form,
        ),
        Format::Text => output::emit_line(out, &written),
    }
}

/// The representation a subcommand answers in (`FR-GLOB-021`, `FR-OUT-001`).
///
/// `FR-CLI-014` has already reduced the occurrences to at most one, so the
/// first is the format in force and the declaration's own default occupies the
/// place where the flag was absent. [`super::schema`] carries the same
/// function over its own seven declarers; the two are written once per arm
/// rather than shared, so that each states the requirement that gave its
/// commands the flags — `FR-GLOB-021` here, `FR-SCH-023` there.
fn representation(output: &local::Output) -> (Format, Form) {
    (
        output.format.first().copied().unwrap_or(Format::Text),
        if output.pretty.pretty {
            Form::Indented
        } else {
            Form::Compact
        },
    )
}
