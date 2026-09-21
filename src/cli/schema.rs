//! The first arm of the tree: `tpl schema`, which reads the database structure.
//!
//! The node is a group, per `FR-CLI-008`, so it carries no action of its own
//! (`FR-CLI-009`) and its child is optional — a bare `tpl schema` prints its
//! own help and exits `0`, per `FR-CLI-007`.
//!
//! Six of the seven aliases of `FR-CLI-011` are declared here. They are
//! declared as **visible** aliases because `FR-CLI-013` shows every alias in
//! the help of its parent node and in the JSON command tree, and because the
//! tree is what `FR-HELP-021` introspects to build that document.
//!
//! # The surface of the eight
//!
//! Three requirements fix it, and between them they leave no subcommand
//! undecided:
//!
//! | Requirement | What it gives, and to which of the eight |
//! |---|---|
//! | `FR-SCH-024` | `--direct` and `--no-cache`, to all eight |
//! | `FR-SCH-023` | `--format` and `--pretty`, to the seven that are not `dump` |
//! | `FR-SCH-020` | `--pretty` alone, to `dump` |
//!
//! `dump` is the exception in both directions and deliberately so:
//! `FR-SCH-019` withholds `--format` because a flag with one permitted value is
//! not a choice, and `FR-SCH-021` withholds `--pattern` because a partial
//! context would fail at render time on a missing object. Neither is declared,
//! so each is the ordinary unknown-flag `64` of `FR-CLI-019` rather than a case
//! of its own.
//!
//! `FR-SCH-005` gives `table`, `view` and `routine` one positional argument
//! each, and states the asymmetry it creates with `render` and `cache`, which
//! name an object by flag: here the subcommand already carries the object's
//! type, so the name needs no flag to disambiguate it.

pub(super) mod named;
mod pattern;
mod text;

use std::borrow::Cow;
use std::io::Write;

use clap::{ArgAction, Args, Subcommand};
use serde::Serialize;

use super::globals::Globals;
use super::local::{self, Format};
use super::source::Reader;
use crate::cache::{Look, paths::Collection};
use crate::error::Error;
use crate::model::document::DatabaseDocument;
use crate::model::document::shape::TableDocument;
use crate::model::routine::Routine;
use crate::model::server::Server;
use crate::model::view::View;
use crate::output::{self, Collection as Members, Document, Form, Source};

/// The `tpl schema` group node.
#[derive(Debug, Clone, PartialEq, Eq, Args)]
pub(crate) struct Schema {
    /// The subcommand invoked, or [`None`] where the group node was invoked
    /// bare.
    #[command(subcommand)]
    pub(crate) command: Option<Command>,
}

/// `--pattern`, declared by exactly the three listing subcommands of
/// `FR-SCH-011`.
///
/// It is declared once and flattened three times for the reason
/// [`local`] gives for the flags it holds. `FR-SCH-015` forbids it on every
/// other command, `dump` included.
#[derive(Debug, Clone, PartialEq, Eq, Args)]
pub(crate) struct Pattern {
    /// Filter the listing by object name, in MariaDB `LIKE` syntax
    /// (`FR-SCH-012`).
    ///
    /// `%` matches any sequence of characters including the empty one, `_`
    /// matches exactly one character, and `\%` and `\_` match those two
    /// literally. The pattern is evaluated in memory and is never sent to the
    /// server, per `FR-SCH-013`, and it folds case over ASCII `A-Z` and `a-z`
    /// only, per `FR-SCH-014`.
    ///
    /// Every occurrence, for the reason [`super::globals`] gives: `FR-CLI-014`
    /// refuses a second one over the occurrences this declaration
    /// accumulates.
    #[arg(long = "pattern", value_name = "PATTERN", action = ArgAction::Append)]
    pub(crate) pattern: Vec<String>,
}

/// The eight children of `tpl schema`.
#[derive(Debug, Clone, PartialEq, Eq, Subcommand)]
pub(crate) enum Command {
    /// Reports the server and the selected database.
    Info {
        /// `--format` and `--pretty`, per `FR-SCH-023`.
        #[command(flatten)]
        output: local::Output,

        /// `--direct` and `--no-cache`, per `FR-SCH-024`.
        #[command(flatten)]
        caching: local::Caching,
    },

    /// Lists the tables of the selected database.
    #[command(visible_alias = "tbls")]
    Tables {
        /// `--pattern`, per `FR-SCH-011`.
        #[command(flatten)]
        filter: Pattern,

        /// `--format` and `--pretty`, per `FR-SCH-023`.
        #[command(flatten)]
        output: local::Output,

        /// `--direct` and `--no-cache`, per `FR-SCH-024`.
        #[command(flatten)]
        caching: local::Caching,
    },

    /// Describes one table.
    #[command(visible_alias = "tbl")]
    Table {
        /// The table to describe (`FR-SCH-005`).
        ///
        /// A name absent from the selected database is `66`, with a
        /// nearest-match suggestion over the tables that do exist, per
        /// `FR-SCH-010`.
        #[arg(value_name = "NAME")]
        name: String,

        /// `--format` and `--pretty`, per `FR-SCH-023`.
        #[command(flatten)]
        output: local::Output,

        /// `--direct` and `--no-cache`, per `FR-SCH-024`.
        #[command(flatten)]
        caching: local::Caching,
    },

    /// Lists the views of the selected database.
    #[command(visible_alias = "vws")]
    Views {
        /// `--pattern`, per `FR-SCH-011`.
        #[command(flatten)]
        filter: Pattern,

        /// `--format` and `--pretty`, per `FR-SCH-023`.
        #[command(flatten)]
        output: local::Output,

        /// `--direct` and `--no-cache`, per `FR-SCH-024`.
        #[command(flatten)]
        caching: local::Caching,
    },

    /// Describes one view.
    #[command(visible_alias = "vw")]
    View {
        /// The view to describe (`FR-SCH-005`).
        ///
        /// A name absent from the selected database is `66`, with a
        /// nearest-match suggestion over the views that do exist, per
        /// `FR-SCH-010`.
        #[arg(value_name = "NAME")]
        name: String,

        /// `--format` and `--pretty`, per `FR-SCH-023`.
        #[command(flatten)]
        output: local::Output,

        /// `--direct` and `--no-cache`, per `FR-SCH-024`.
        #[command(flatten)]
        caching: local::Caching,
    },

    /// Lists the routines — procedures and functions — of the selected
    /// database.
    #[command(visible_alias = "rtns")]
    Routines {
        /// `--pattern`, per `FR-SCH-011`.
        ///
        /// There is no `--type` beside it: `FR-SCH-008` refuses one, and
        /// selecting one kind from a listing is done downstream from
        /// `--format json`.
        #[command(flatten)]
        filter: Pattern,

        /// `--format` and `--pretty`, per `FR-SCH-023`.
        #[command(flatten)]
        output: local::Output,

        /// `--direct` and `--no-cache`, per `FR-SCH-024`.
        #[command(flatten)]
        caching: local::Caching,
    },

    /// Describes one routine.
    #[command(visible_alias = "rtn")]
    Routine {
        /// The routine to describe (`FR-SCH-005`).
        ///
        /// It accepts the qualified forms `procedure:<name>` and
        /// `function:<name>` as well as a bare name, per `FR-SCH-008`. A bare
        /// name matching both a procedure and a function is `64`, and a name
        /// matching neither is `66`, per `FR-SCH-010`.
        #[arg(value_name = "NAME")]
        name: String,

        /// `--format` and `--pretty`, per `FR-SCH-023`.
        #[command(flatten)]
        output: local::Output,

        /// `--direct` and `--no-cache`, per `FR-SCH-024`.
        #[command(flatten)]
        caching: local::Caching,
    },

    /// Writes the whole catalogue model of the selected database.
    Dump {
        /// `--pretty`, standing alone because the result is JSON and nothing
        /// else, per `FR-SCH-020` and `FR-OUT-010`.
        #[command(flatten)]
        pretty: local::Pretty,

        /// `--direct` and `--no-cache`, per `FR-SCH-024`.
        #[command(flatten)]
        caching: local::Caching,
    },
}

/// The invocation of `FR-ERR-022` the routine token of this arm belongs to.
///
/// It travels on the two conditions a token can raise — the prefix case of
/// `FR-SCH-008` and the ambiguity of `FR-SCH-010` — because each is answered
/// with the same invocation corrected, per `FR-ERR-009`.
const ROUTINE: &str = "schema routine";

/// The `data` of `tpl schema dump` (`FR-SCH-034`).
///
/// One key, named for the kind in the singular, whose value is the whole
/// `database` object [context-document.md](../../../specification/context-document.md)
/// fixes. The key order is the field order, per `OD-18`, and one field states
/// it.
#[derive(Debug, Serialize)]
struct DatabaseData<'a, 'd> {
    /// The whole model of the selected database.
    database: &'a DatabaseDocument<'d>,
}

/// The `data` of `tpl schema info` (`FR-SCH-031`).
///
/// One key, `database`, whose value is the **named subset** that requirement
/// fixes rather than the whole object [`DatabaseData`] carries.
#[derive(Debug, Serialize)]
struct InfoData<'a, 'd> {
    /// The metadata of the selected database.
    database: DatabaseMetadata<'a, 'd>,
}

/// The `database` object of `tpl schema info` (`FR-SCH-031`).
///
/// ```json
/// {"name":"freight","charset":"utf8mb4","collation":"utf8mb4_unicode_520_ci","server":{…}}
/// ```
///
/// **Exactly four members and no others**: the three metadata fields
/// `FR-CTX-036` fixes and the `server` object `FR-CTX-031` fixes. The three
/// collections of `FR-CTX-035` — `tables`, `views` and `routines` — are not
/// carried, which is what `FR-SCH-031` requires and is the whole of the
/// difference between this command and `tpl schema dump`.
///
/// *Why the reduction is a type of its own rather than a serialisation
/// attribute on [`DatabaseDocument`].* That type is the document
/// `tpl schema dump` emits under `FR-SCH-034` and `tpl render --context`
/// consumes under `FR-SCH-036`, and skipping three of its fields there would
/// change both. The fields below are **borrowed** from it, so the reduction
/// copies nothing and cannot disagree with the object it reduces: every member
/// is the same member, under the same name and with the same value, which is
/// what `FR-SCH-031` promises a caller that reads `data.database.name` from
/// either command.
///
/// The key order is the field order, per `OD-18`, and it is `FR-CTX-036`'s
/// order with the collections removed from the end.
///
/// *Rejected: a key of its own beside `database`, or a count of each
/// collection.* `FR-SCH-031` rejects both by name — a count is derivable from
/// the three listings of `FR-SCH-032` and from the dump, and inventing a field
/// the model does not carry would put a number in the plumbing contract that no
/// requirement of `catalogue-coverage.md` fixes.
#[derive(Debug, Serialize)]
struct DatabaseMetadata<'a, 'd> {
    /// The schema's name.
    name: &'a Cow<'d, str>,

    /// The schema's default character set.
    charset: &'a Cow<'d, str>,

    /// The schema's default collation.
    collation: &'a Cow<'d, str>,

    /// The server the read was made against (`FR-CTX-031`).
    server: &'a Server<'d>,
}

impl<'a, 'd> DatabaseMetadata<'a, 'd> {
    /// The four members of `document` this command presents.
    const fn of(document: &'a DatabaseDocument<'d>) -> Self {
        Self {
            name: &document.name,
            charset: &document.charset,
            collation: &document.collation,
            server: &document.server,
        }
    }
}

/// The `data` of `tpl schema table` (`FR-SCH-033`, `FR-OUT-031`).
#[derive(Debug, Serialize)]
struct TableData<'a, 'd> {
    /// The table the invocation named.
    table: &'a TableDocument<'d>,
}

/// The `data` of `tpl schema view` (`FR-SCH-033`, `FR-OUT-031`).
#[derive(Debug, Serialize)]
struct ViewData<'a, 'd> {
    /// The view the invocation named.
    view: &'a View<'d>,
}

/// The `data` of `tpl schema routine` (`FR-SCH-033`, `FR-OUT-031`).
#[derive(Debug, Serialize)]
struct RoutineData<'a, 'd> {
    /// The routine the invocation named.
    routine: &'a Routine<'d>,
}

/// Runs one `schema` subcommand.
///
/// Every one of them reads through the cache, per `FR-SCH-025`, and every one
/// of them presents a part of the one `database` object the read produced —
/// which is what makes `BR-SCH-001` hold across the eight: a listing, a named
/// object and the dump select from the same document, and the document is the
/// same shape whether the cache or the server served it.
///
/// # Errors
///
/// Returns what the read returns — the `78` of a project, a configuration or an
/// entry that does not describe a read, the `69` of a server that did not
/// answer, the `77` of a session that could not be made read only — and, for a
/// subcommand that names an object, the `64` of `FR-SCH-008` and `FR-SCH-010`,
/// the `66` of `FR-SCH-010` and the `77` of `FR-PRIV-003`.
pub(crate) fn run<W: Write>(
    out: &mut W,
    globals: &Globals,
    command: &Command,
) -> Result<(), Error> {
    match command {
        Command::Info { output, caching } => {
            let (format, form) = representation(output);

            Reader::new(globals, Some(caching)).serve(&Look::Everything, |document, source, _| {
                match format {
                    Format::Text => text::info(&mut *out, document),
                    // FR-SCH-031: the named subset, not the whole object. The
                    // two commands answer different questions and this is what
                    // keeps them from emitting the same bytes.
                    Format::Json => enveloped(
                        &mut *out,
                        source,
                        form,
                        InfoData {
                            database: DatabaseMetadata::of(document),
                        },
                    ),
                }
            })
        }

        Command::Tables {
            filter,
            output,
            caching,
        } => {
            let (format, form) = representation(output);
            let mut selector = selector(filter);

            Reader::new(globals, Some(caching)).serve(
                &Look::Collection(Collection::Tables),
                |document, source, _| {
                    let selected = filtered(selector.as_mut(), document.tables.iter(), |table| {
                        &table.name
                    });

                    match format {
                        Format::Text => text::tables(&mut *out, &selected),
                        Format::Json => {
                            enveloped(&mut *out, source, form, Members::new(TABLES, &selected))
                        }
                    }
                },
            )
        }

        Command::Views {
            filter,
            output,
            caching,
        } => {
            let (format, form) = representation(output);
            let mut selector = selector(filter);

            Reader::new(globals, Some(caching)).serve(
                &Look::Collection(Collection::Views),
                |document, source, _| {
                    let selected =
                        filtered(selector.as_mut(), document.views.iter(), |view| &view.name);

                    match format {
                        Format::Text => text::views(&mut *out, &selected),
                        Format::Json => {
                            enveloped(&mut *out, source, form, Members::new(VIEWS, &selected))
                        }
                    }
                },
            )
        }

        Command::Routines {
            filter,
            output,
            caching,
        } => {
            let (format, form) = representation(output);
            let mut selector = selector(filter);

            Reader::new(globals, Some(caching)).serve(
                &Look::Collection(Collection::Routines),
                |document, source, _| {
                    let selected =
                        filtered(selector.as_mut(), document.routines.iter(), |routine| {
                            &routine.name
                        });

                    match format {
                        Format::Text => text::routines(&mut *out, &selected),
                        Format::Json => {
                            enveloped(&mut *out, source, form, Members::new(ROUTINES, &selected))
                        }
                    }
                },
            )
        }

        Command::Table {
            name,
            output,
            caching,
        } => {
            let (format, form) = representation(output);

            Reader::new(globals, Some(caching)).serve(
                &Look::Table(name),
                |document, source, entry| {
                    let found = named::table(document, name, sought(entry, document))?;

                    match format {
                        Format::Text => text::table(&mut *out, found),
                        Format::Json => {
                            enveloped(&mut *out, source, form, TableData { table: found })
                        }
                    }
                },
            )
        }

        Command::View {
            name,
            output,
            caching,
        } => {
            let (format, form) = representation(output);

            Reader::new(globals, Some(caching)).serve(
                &Look::View(name),
                |document, source, entry| {
                    let found = named::view(document, name, sought(entry, document))?;

                    match format {
                        Format::Text => text::view(&mut *out, found),
                        Format::Json => {
                            enveloped(&mut *out, source, form, ViewData { view: found })
                        }
                    }
                },
            )
        }

        Command::Routine {
            name,
            output,
            caching,
        } => {
            let (format, form) = representation(output);
            // FR-SCH-008 decides the token's shape without a server, so it is
            // decided at step 1 of FR-ERR-006 — before the read below.
            let wanted = named::routine_token(name, ROUTINE)?;

            Reader::new(globals, Some(caching)).serve(
                &look_for(&wanted),
                |document, source, entry| {
                    let found =
                        named::routine(document, &wanted, sought(entry, document), ROUTINE)?;

                    match format {
                        Format::Text => text::routine(&mut *out, found),
                        Format::Json => {
                            enveloped(&mut *out, source, form, RoutineData { routine: found })
                        }
                    }
                },
            )
        }

        // FR-SCH-019 withholds `--format`, so the representation is JSON and
        // `--pretty` stands alone, per FR-SCH-020 and FR-OUT-010.
        Command::Dump { pretty, caching } => {
            let form = form(pretty.pretty);

            Reader::new(globals, Some(caching)).serve(&Look::Everything, |document, source, _| {
                enveloped(&mut *out, source, form, DatabaseData { database: document })
            })
        }
    }
}

/// The plural key a listing's one `data` key carries (`FR-OUT-030`,
/// `FR-SCH-032`).
const TABLES: &str = "tables";

/// The plural key of the view listing.
const VIEWS: &str = "views";

/// The plural key of the routine listing.
const ROUTINES: &str = "routines";

/// What the store is asked for on behalf of a token naming one routine.
///
/// A qualified token names one file, per `FR-CDOC-014`, and `FR-CDOC-008`
/// serves it whenever it is present. A bare token names two, and choosing
/// between them is the ambiguity `FR-SCH-010` refuses over the population of
/// the database — so it asks for the collection, which `FR-CDOC-007` serves
/// only where it was loaded whole and which is therefore the population itself.
fn look_for<'a>(wanted: &named::Wanted<'a>) -> Look<'a> {
    match wanted {
        named::Wanted::Bare(_) => Look::Collection(Collection::Routines),
        // The kind is cloned rather than copied: `FR-CAT-055` gives
        // `RoutineKind` a variant carrying the catalogue's own string. Only a
        // recorded kind reaches here, so the clone is of a unit variant and
        // allocates nothing.
        named::Wanted::Qualified(kind, name) => Look::Routine(kind.clone(), name),
    }
}

/// Where a named read was made, for the conditions whose `cause` names the
/// population.
///
/// The database is the document's own name, which is the database the entry
/// named, per `FR-CONF-041`: the read covered it and nothing else, so no
/// second source is consulted for it.
fn sought<'a>(entry: &'a str, document: &'a DatabaseDocument<'_>) -> named::Sought<'a> {
    named::Sought::Catalogue {
        entry,
        database: &document.name,
    }
}

/// The representation a subcommand answers in (`FR-SCH-023`, `FR-OUT-001`).
///
/// `FR-CLI-014` has already reduced the occurrences to at most one, so the
/// first is the format in force and the declaration's own default occupies the
/// place where the flag was absent.
fn representation(output: &local::Output) -> (Format, Form) {
    (
        output.format.first().copied().unwrap_or(Format::Text),
        form(output.pretty.pretty),
    )
}

/// Which of the two forms of `FR-OUT-007` and `FR-OUT-008` a document is
/// written in.
const fn form(pretty: bool) -> Form {
    if pretty {
        Form::Indented
    } else {
        Form::Compact
    }
}

/// The compiled `--pattern`, or [`None`] where the flag was absent
/// (`FR-SCH-011`).
fn selector(filter: &Pattern) -> Option<pattern::Pattern> {
    filter
        .pattern
        .first()
        .map(|written| pattern::Pattern::compile(written))
}

/// The members `selector` admits, in the order the document carries them.
///
/// The document's collections are already in the order `NFR-DET-002` fixes and
/// the coverage of `FR-CAT-052` has already been applied to them, so `FR-CAT-028`
/// holds by construction: the filter runs over what the read presented, after
/// coverage and before nothing.
fn filtered<'d, T>(
    selector: Option<&mut pattern::Pattern>,
    members: impl Iterator<Item = &'d T>,
    name: impl Fn(&'d T) -> &'d str,
) -> Vec<&'d T> {
    match selector {
        None => members.collect(),
        Some(selector) => members
            .filter(|member| selector.matches(name(member)))
            .collect(),
    }
}

/// Writes one payload in the envelope of `FR-OUT-024` (`FR-SCH-030`).
///
/// # Errors
///
/// Returns what [`output::emit_to`] returns.
fn enveloped<W: Write, T: Serialize>(
    out: W,
    source: Source,
    form: Form,
    data: T,
) -> Result<(), Error> {
    output::emit_to(out, &Document::new(source, data), form)
}
