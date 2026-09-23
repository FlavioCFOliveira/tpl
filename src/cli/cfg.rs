//! The configuration arm: `tpl cfg`, and the database entries beneath it.
//!
//! This arm carries the only three-level path of the tree, and both of its
//! interior nodes are groups, per `FR-CLI-008`: `tpl cfg` and
//! `tpl cfg database` each have no action of their own (`FR-CLI-009`), an
//! optional child, and their own help at exit `0` when invoked bare
//! (`FR-CLI-007`).
//!
//! The seventh alias of `FR-CLI-011` — `db` for `database` — is declared here,
//! and it is the only alias of the arm. It is visible for the reason every
//! other alias is: `FR-CLI-013` shows each one in the help of its parent node
//! and in the JSON command tree.
//!
//! # The surface of the ten
//!
//! `FR-GLOB-021` gives `--format`, and with it `--pretty`, to the five
//! subcommands that answer with a document — `get`, `list`, `database list`,
//! `database show` and `database test`, the last by `FR-CFG-026`. The five that
//! change the file or name a key for removal have neither, so
//! `tpl cfg set core.database shop --format json` is the ordinary unknown-flag
//! `64` of `FR-CLI-019`.
//!
//! The nine flags of [`Entry`] are `FR-CFG-027` in full, and they are declared
//! by `database add` and `database update` and by nothing else. Two rules
//! between them are **not** declared on the arguments and are the command's:
//! `FR-CFG-016`, which requires `add` to be given either `--dsn` or at least
//! one discrete connection flag, and `FR-CFG-029`, which makes the two groups
//! mutually exclusive in one invocation. Both are refusals between arguments
//! rather than properties of one, and are left where [`super::local`] leaves
//! the two of its own.
//!
//! # The arm and the catalogue cache
//!
//! `FR-CACHE-011` forbids every subcommand of this arm **other than
//! `database test`** to contact a database or to touch `.tpl/.cache/`, and
//! `FR-CACHE-010` fixes what the exception does: `database test` always
//! contacts the server, neither reads nor writes the store, and reads nothing
//! into the model — the one statement it issues against the catalogue is the
//! privilege probe of `FR-CFG-044`, whose result is a boolean and not model
//! content.
//!
//! The first holds by absence, over the whole arm: no code path of this module,
//! of [`entries`], of [`keys`] or of [`connectivity`] reaches [`crate::cache`],
//! so no subcommand of the arm has a route to the store — including the one
//! that contacts a server.
//!
//! The second holds by absence over the nine: they reach [`crate::mariadb`] no
//! more than they reach the cache. [`connectivity`] is the exception
//! `FR-CACHE-010` makes, and it is a module of its own so that the exception is
//! one file rather than one arm of a match: the connection, the four steps of
//! `FR-CFG-024` and the report of `FR-CFG-039` are all there, and neither
//! [`entries`] nor [`keys`] names a type of [`crate::mariadb`].
//!
//! What is absent is a requirement in its own right. `FR-CFG-030` forbids a
//! `--password` or `-p` flag on any command, and `FR-GLOB-023` generalises it
//! to any flag whose purpose is to carry a password. A literal password reaches
//! the file through `tpl cfg set` or inside a `--dsn`, per `FR-CFG-031` and
//! `FR-CFG-032`, and `BR-CFG-003` states the position that makes those two
//! acceptable: `tpl` warns, and does not prevent.

pub(crate) mod coherence;
pub(crate) mod connectivity;
pub(crate) mod entries;
pub(crate) mod keys;

use std::path::{Path, PathBuf};

use clap::{ArgAction, Args, Subcommand};

use super::globals::Globals;
use super::local::{self, Format};
use crate::deadline::{Clock, Seconds};
use crate::error::Error;
use crate::output::Form;
use crate::project::Project;
use crate::project::config::entry::TlsMode;
use crate::project::settings;

/// What one `cfg` invocation supplies that every subcommand of the arm reads.
///
/// The two are the project the command acts on and the representation it
/// answers in, and they are collected once so that a subcommand takes one
/// argument rather than two vectors it must reduce itself. `FR-CLI-014` has
/// already reduced each of the three to at most one occurrence by the time this
/// is built.
#[derive(Debug, Clone, Copy)]
pub(crate) struct Supplied<'a> {
    /// `--tpl-dir`, which names the `.tpl` folder and suppresses the walk
    /// (`FR-GLOB-009`).
    tpl_dir: Option<&'a Path>,
    /// The representation of the result (`FR-OUT-001`), for the five
    /// subcommands of the arm that declare `--format`.
    format: Format,
    /// Whether `--pretty` was given (`FR-OUT-008`).
    pretty: bool,
    /// `--timeout`, the overall budget of `FR-GLOB-011`.
    ///
    /// It is carried for [`connectivity`] alone, which is the one subcommand of
    /// the arm with a blocking phase to bound: `FR-GLOB-012` composes the
    /// budget with every phase deadline, and the four steps of `FR-CFG-024` run
    /// over three of them. The other nine touch `.tpl/` and nothing else, so
    /// they have nothing for it to compose with; carrying it once here rather
    /// than passing a second argument to one subcommand is what keeps
    /// `FR-CLI-014`'s reduction in one place.
    budget: Option<Seconds>,
}

impl<'a> Supplied<'a> {
    /// What `globals` and `output` supply.
    ///
    /// `output` is [`None`] for the five subcommands that declare neither flag,
    /// which `FR-GLOB-021` leaves without a representation to choose.
    pub(crate) fn new(globals: &'a Globals, output: Option<&local::Output>) -> Self {
        Self {
            tpl_dir: globals.tpl_dir.first().map(PathBuf::as_path),
            format: output
                .and_then(|output| output.format.first().copied())
                .unwrap_or(Format::Text),
            pretty: output.is_some_and(|output| output.pretty.pretty),
            budget: globals.timeout.first().copied().map(Seconds::new),
        }
    }

    /// The representation the result is written in.
    pub(crate) const fn format(self) -> Format {
        self.format
    }

    /// The clock the blocking phases of `tpl cfg database test` are bounded by
    /// (`FR-GLOB-011`, `FR-GLOB-012`).
    fn clock(self) -> Clock {
        settings::clock(self.budget)
    }
}

/// The project this invocation acts on.
///
/// It is steps 2 and 3 of `FR-ERR-006` in one call: discovery and the two trust
/// checks, before any subcommand of the arm reads a key of its own. No `cfg`
/// subcommand is among the four `FR-PROJ-025` excuses, so every one of them
/// performs it.
///
/// # Errors
///
/// Returns what [`Project::current`] returns.
fn project(supplied: &Supplied<'_>) -> Result<Project, Error> {
    Project::current(supplied.tpl_dir)
}

/// Which of the two forms of `FR-OUT-007` and `FR-OUT-008` a document is
/// written in.
const fn form(supplied: &Supplied<'_>) -> Form {
    if supplied.pretty {
        Form::Indented
    } else {
        Form::Compact
    }
}

/// The `tpl cfg` group node.
#[derive(Debug, Clone, PartialEq, Eq, Args)]
pub(crate) struct Cfg {
    /// The subcommand invoked, or [`None`] where the group node was invoked
    /// bare.
    #[command(subcommand)]
    pub(crate) command: Option<Command>,
}

/// The five children of `tpl cfg`.
#[derive(Debug, Clone, PartialEq, Eq, Subcommand)]
pub(crate) enum Command {
    /// Reads one configuration key.
    Get {
        /// The key to read, in the dotted form of `FR-CONF-002`.
        ///
        /// A key absent from the file is `66`, with a nearest-match suggestion
        /// over the keys that do exist, per `FR-CFG-007`.
        #[arg(value_name = "KEY")]
        key: String,

        /// `--format` and `--pretty`, per `FR-GLOB-021`.
        #[command(flatten)]
        output: local::Output,
    },

    /// Writes one configuration key.
    Set {
        /// The key to write, which `FR-CFG-009` restricts to the enumerated
        /// key space of `FR-CONF-002`.
        #[arg(value_name = "KEY")]
        key: String,

        /// The value to write, validated against the type `FR-CONF-002`
        /// declares for the key, per `FR-CFG-010`.
        ///
        /// A value given here is visible in the process table for the life of
        /// the invocation, per `FR-CFG-033`.
        #[arg(value_name = "VALUE")]
        value: String,
    },

    /// Removes one configuration key.
    Unset {
        /// The key or block to remove — either a leaf, such as
        /// `database.shop.host`, or a whole block, such as `database.shop`,
        /// per `FR-CFG-011`.
        #[arg(value_name = "KEY")]
        key: String,
    },

    /// Lists the configuration keys the project carries.
    List {
        /// `--format` and `--pretty`, per `FR-GLOB-021`.
        #[command(flatten)]
        output: local::Output,
    },

    /// The database entries of `.tpl/.cfg`.
    #[command(visible_alias = "db")]
    Database(Database),
}

/// The `tpl cfg database` group node.
#[derive(Debug, Clone, PartialEq, Eq, Args)]
pub(crate) struct Database {
    /// The subcommand invoked, or [`None`] where the group node was invoked
    /// bare.
    #[command(subcommand)]
    pub(crate) command: Option<DatabaseCommand>,
}

// The mode of `--tls` is the type `FR-CONF-013` closes and `FR-CONF-036` makes
// normative over the driver, and it is declared once, beside the entry it
// belongs to. It is a type rather than a string checked after parsing because
// `FR-HELP-013` obliges the help to state the permitted values of every flag
// and `FR-HELP-021` derives them by introspecting this tree; declaring a second
// copy here would let the flag and the file disagree about what a mode is.

/// The nine flags of `FR-CFG-027`, each mapping to one key of a
/// `[database.<name>]` block.
///
/// They are declared once and flattened into `add` and `update`, which is what
/// keeps one flag from meaning one thing to the command that creates an entry
/// and another to the command that changes it. None of them carries a short
/// form: `FR-GLOB-024` closes the short-flag space at five, and `FR-CFG-047`
/// says so again for the three the fifth edition added.
///
/// Every flag is optional here. `add` and `update` require different
/// combinations — `FR-CFG-016` and `FR-CFG-029` — and both are refusals
/// between arguments, left to the commands that make them.
///
/// Each of the nine carries a value, so each is declared repeatable and the
/// repetition is refused by [`super::rules`], for the reason `OD-08` gives and
/// [`super::globals`] states: `FR-CLI-014` obliges the message to name **both
/// values**, and only the accumulated occurrences put both in hand. A field
/// here is every occurrence in the order written, reduced to at most one
/// before `add` or `update` reads it.
#[derive(Debug, Clone, PartialEq, Eq, Args)]
pub(crate) struct Entry {
    /// The whole connection as one URL, written to `database.<name>.dsn`.
    ///
    /// Mutually exclusive with the discrete connection flags, per
    /// `FR-CFG-029`. It is stored verbatim, a literal password included, per
    /// `FR-CFG-031`, and a value given here is visible in the process table
    /// for the life of the invocation, per `FR-CFG-033`.
    #[arg(long = "dsn", value_name = "URL", action = ArgAction::Append)]
    pub(crate) dsn: Vec<String>,

    /// The server host, written to `database.<name>.host`.
    #[arg(long = "host", value_name = "HOST", action = ArgAction::Append)]
    pub(crate) host: Vec<String>,

    /// The server port, written to `database.<name>.port`.
    ///
    /// Absent, the entry carries no port and the default of `FR-CONF-002`,
    /// `3306`, applies when the entry is read.
    // FR-CONF-002 types the key as a TCP port and the reader refuses `0`, so
    // the flag refuses it too: a value the flag admitted and the file then
    // refused would leave every later command, the repair included, a `78`.
    #[arg(
        long = "port",
        value_name = "PORT",
        action = ArgAction::Append,
        value_parser = clap::value_parser!(u16).range(1..)
    )]
    pub(crate) port: Vec<u16>,

    /// The user to authenticate as, written to `database.<name>.user`.
    #[arg(long = "user", value_name = "USER", action = ArgAction::Append)]
    pub(crate) user: Vec<String>,

    /// The database on the server, written to `database.<name>.database`.
    ///
    /// It is the one flag whose name differs from the key it writes, per
    /// `FR-CFG-028`: `schema` is the catalogue's own word for it, and it leaves
    /// the global `-d/--database` free to mean the entry label everywhere.
    #[arg(long = "schema", value_name = "NAME", action = ArgAction::Append)]
    pub(crate) schema: Vec<String>,

    /// The TLS mode, written to `database.<name>.tls`.
    ///
    /// Absent, the entry carries no mode and the default of `FR-CONF-013`,
    /// `verify-identity`, applies when the entry is read.
    #[arg(long = "tls", value_name = "MODE", value_enum, action = ArgAction::Append)]
    pub(crate) tls: Vec<TlsMode>,

    /// The command that produces the password, written to
    /// `database.<name>.password_command`.
    ///
    /// One string, split into the stored array by the POSIX quoting rules of
    /// `FR-CONF-025`. It accepts no array on the command line and is not
    /// repeatable, per `FR-CFG-046`. A value given here is visible in the
    /// process table for the life of the invocation, and `FR-CONF-017` admits
    /// no `${VAR}` expansion in this key, per `FR-CFG-033`.
    #[arg(
        long = "password-command",
        value_name = "COMMAND",
        action = ArgAction::Append
    )]
    pub(crate) password_command: Vec<String>,

    /// The certificate file trusted by `verify-ca` and `verify-identity`,
    /// written to `database.<name>.ca_file` (`FR-CONF-014`).
    #[arg(long = "ca-file", value_name = "PATH", action = ArgAction::Append)]
    pub(crate) ca_file: Vec<PathBuf>,

    /// The certificate directory trusted by `verify-ca` and `verify-identity`,
    /// written to `database.<name>.ca_path` (`FR-CONF-014`).
    #[arg(long = "ca-path", value_name = "PATH", action = ArgAction::Append)]
    pub(crate) ca_path: Vec<PathBuf>,
}

impl Entry {
    /// The nine flags, reduced to what one invocation supplied.
    ///
    /// Each field is every occurrence in the order written, per this module's
    /// own note, and [`super::rules::refuse_repetition`] has already refused a
    /// second one — so the first is the value in force and there is never a
    /// later one to lose.
    pub(crate) fn flags(&self) -> entries::Flags<'_> {
        entries::Flags {
            dsn: self.dsn.first().map(String::as_str),
            host: self.host.first().map(String::as_str),
            port: self.port.first().copied(),
            user: self.user.first().map(String::as_str),
            schema: self.schema.first().map(String::as_str),
            tls: self.tls.first().copied(),
            password_command: self.password_command.first().map(String::as_str),
            ca_file: self.ca_file.first().map(PathBuf::as_path),
            ca_path: self.ca_path.first().map(PathBuf::as_path),
        }
    }
}

/// The six children of `tpl cfg database`.
#[derive(Debug, Clone, PartialEq, Eq, Subcommand)]
pub(crate) enum DatabaseCommand {
    /// Adds a database entry.
    Add {
        /// The name of the entry to create, which becomes the
        /// `[database.<name>]` block (`FR-CFG-015`).
        ///
        /// A name that is already taken is `64`, with a hint pointing at
        /// `tpl cfg database update`, per `FR-CFG-017`.
        #[arg(value_name = "NAME")]
        name: String,

        /// The nine flags of `FR-CFG-027`.
        #[command(flatten)]
        entry: Entry,
    },

    /// Lists the database entries.
    List {
        /// `--format` and `--pretty`, per `FR-GLOB-021`.
        #[command(flatten)]
        output: local::Output,
    },

    /// Shows one database entry.
    Show {
        /// The entry to print, with passwords redacted and `${VAR}` left
        /// exactly as written, per `FR-CFG-019` and `FR-CFG-021`.
        #[arg(value_name = "NAME")]
        name: String,

        /// `--format` and `--pretty`, per `FR-GLOB-021`.
        #[command(flatten)]
        output: local::Output,
    },

    /// Changes one database entry.
    Update {
        /// The entry to change. Only the fields named by the flags supplied
        /// are changed, per `FR-CFG-020`.
        #[arg(value_name = "NAME")]
        name: String,

        /// The nine flags of `FR-CFG-027`.
        #[command(flatten)]
        entry: Entry,
    },

    /// Removes one database entry.
    Remove {
        /// The entry to delete (`FR-CFG-022`).
        #[arg(value_name = "NAME")]
        name: String,
    },

    /// Opens a connection with one database entry and reports the outcome.
    Test {
        /// The entry to test, over the four steps of `FR-CFG-024`.
        #[arg(value_name = "NAME")]
        name: String,

        /// `--format` and `--pretty`, per `FR-CFG-026`.
        #[command(flatten)]
        output: local::Output,
    },
}

#[cfg(test)]
pub(crate) mod tests {
    use std::path::{Path, PathBuf};

    use super::{Supplied, entries, keys};
    use crate::cli::local::Format;
    use crate::error::Error;
    use crate::project::edit::MODE;
    use crate::project::scratch::Scratch;

    /// A project on disk, and the ten subcommands driven against it.
    ///
    /// Every subcommand is reached through `--tpl-dir`, which `FR-PROJ-008`
    /// makes subject to every trust check without exemption — so the harness
    /// exercises the same path a caller does and never moves the process's
    /// working directory, which `cargo test` shares between threads.
    #[derive(Debug)]
    pub(crate) struct Harness {
        /// The temporary tree, removed when the harness goes out of scope.
        scratch: Scratch,
        /// The `.tpl` folder inside it.
        tpl: PathBuf,
    }

    impl Harness {
        /// A project whose `.tpl/.cfg` holds `configuration`.
        pub(crate) fn new(configuration: &str) -> Self {
            let scratch = Scratch::new();
            let tpl = scratch.directory(".tpl");
            let file = scratch.file(".tpl/.cfg", configuration);
            scratch.chmod(&file, MODE);

            Self { scratch, tpl }
        }

        /// What one subcommand of the arm was supplied.
        ///
        /// No budget: `--timeout` is `FR-GLOB-011`'s and only [`connectivity`]
        /// has a blocking phase for it to bound, so a harness that supplies one
        /// would be supplying it to nine subcommands that cannot read it.
        /// [`connectivity`]'s own bodies compose the value they need.
        pub(crate) fn supplied(&self, format: Format, pretty: bool) -> Supplied<'_> {
            Supplied {
                tpl_dir: Some(self.tpl.as_path()),
                format,
                pretty,
                budget: None,
            }
        }

        /// The `.tpl/.cfg` as it now stands.
        pub(crate) fn written(&self) -> String {
            std::fs::read_to_string(self.tpl.join(".cfg")).expect("the file is there")
        }

        /// The `.tpl` folder, for a test that inspects it directly.
        pub(crate) fn tpl(&self) -> &Path {
            &self.tpl
        }

        /// The temporary tree the project sits in.
        pub(crate) const fn scratch(&self) -> &Scratch {
            &self.scratch
        }

        /// What a command wrote, for a run that is expected to succeed.
        fn wrote<F>(&self, run: F) -> String
        where
            F: FnOnce(&mut Vec<u8>) -> Result<(), Error>,
        {
            let mut out = Vec::new();
            run(&mut out).expect("the command succeeds");

            String::from_utf8(out).expect("the command writes UTF-8")
        }

        /// `tpl cfg get <key>`, in `text`.
        pub(crate) fn get(&self, key: &str) -> String {
            self.wrote(|out| keys::get(out, &self.supplied(Format::Text, false), key))
        }

        /// `tpl cfg get <key> --format json`.
        pub(crate) fn get_json(&self, key: &str) -> String {
            self.wrote(|out| keys::get(out, &self.supplied(Format::Json, false), key))
        }

        /// What `tpl cfg get <key>` refused.
        pub(crate) fn get_refused(&self, key: &str) -> Error {
            let mut out = Vec::new();

            keys::get(&mut out, &self.supplied(Format::Text, false), key)
                .expect_err("the command is refused")
        }

        /// `tpl cfg set <key> <value>`.
        pub(crate) fn set(&self, key: &str, value: &str) -> Result<(), Error> {
            keys::set(&self.supplied(Format::Text, false), key, value)
        }

        /// What `tpl cfg set` wrote to stdout, which `FR-OUT-023` makes empty.
        pub(crate) fn set_output(&self, key: &str, value: &str) -> String {
            self.wrote(|_| keys::set(&self.supplied(Format::Text, false), key, value))
        }

        /// `tpl cfg unset <key>`.
        pub(crate) fn unset(&self, key: &str) -> Result<(), Error> {
            keys::unset(&self.supplied(Format::Text, false), key)
        }

        /// `tpl cfg list`, in `text`.
        pub(crate) fn list(&self) -> String {
            self.wrote(|out| keys::list(out, &self.supplied(Format::Text, false)))
        }

        /// `tpl cfg list --format json`.
        pub(crate) fn list_json(&self) -> String {
            self.wrote(|out| keys::list(out, &self.supplied(Format::Json, false)))
        }

        /// `tpl cfg database add <name>`.
        pub(crate) fn add(&self, name: &str, flags: entries::Flags<'_>) -> Result<(), Error> {
            entries::add(&self.supplied(Format::Text, false), name, &flags)
        }

        /// What `tpl cfg database add` wrote to stdout.
        pub(crate) fn add_output(&self, name: &str, flags: entries::Flags<'_>) -> String {
            self.wrote(|_| entries::add(&self.supplied(Format::Text, false), name, &flags))
        }

        /// `tpl cfg database update <name>`.
        pub(crate) fn update(&self, name: &str, flags: entries::Flags<'_>) -> Result<(), Error> {
            entries::update(&self.supplied(Format::Text, false), name, &flags)
        }

        /// `tpl cfg database remove <name>`.
        pub(crate) fn remove(&self, name: &str) -> Result<(), Error> {
            entries::remove(&self.supplied(Format::Text, false), name)
        }

        /// `tpl cfg database list`, in `text`.
        pub(crate) fn database_list(&self) -> String {
            self.wrote(|out| entries::list(out, &self.supplied(Format::Text, false)))
        }

        /// `tpl cfg database list --format json`.
        pub(crate) fn database_list_json(&self) -> String {
            self.wrote(|out| entries::list(out, &self.supplied(Format::Json, false)))
        }

        /// `tpl cfg database show <name>`, in `text`.
        pub(crate) fn show(&self, name: &str) -> String {
            self.wrote(|out| entries::show(out, &self.supplied(Format::Text, false), name))
        }

        /// `tpl cfg database show <name> --format json`.
        pub(crate) fn show_json(&self, name: &str) -> String {
            self.wrote(|out| entries::show(out, &self.supplied(Format::Json, false), name))
        }

        /// What `tpl cfg database show <name>` did, for a run that is expected
        /// to be refused.
        pub(crate) fn show_refused(&self, name: &str) -> Result<(), Error> {
            let mut out = Vec::new();

            entries::show(&mut out, &self.supplied(Format::Text, false), name)
        }
    }

    #[test]
    fn fr_out_007_a_pretty_document_is_indented_and_a_plain_one_is_not() {
        // FR-OUT-007, FR-OUT-008: --pretty changes the whitespace and nothing
        // else.
        let harness = Harness::new("[core]\ndatabase = \"shop\"\n");

        let compact = harness.wrote(|out| keys::list(out, &harness.supplied(Format::Json, false)));
        let indented = harness.wrote(|out| keys::list(out, &harness.supplied(Format::Json, true)));

        assert_eq!(compact.matches('\n').count(), 1);
        assert!(
            indented.contains("\n  \"schema_version\": 1,\n"),
            "{indented}"
        );
    }

    #[test]
    fn fr_proj_025_every_subcommand_of_the_arm_performs_discovery_and_the_trust_checks() {
        // FR-PROJ-025: no cfg subcommand is among the four it excuses, and
        // FR-GLOB-010 subjects --tpl-dir to every check without exemption.
        let harness = Harness::new("[core]\ndatabase = \"shop\"\n");
        harness.scratch().chmod(&harness.tpl().join(".cfg"), 0o644);

        assert_eq!(
            harness.get_refused("core.database").exit_code(),
            78,
            "an unsafe mode is refused before the key is read"
        );
        assert_eq!(
            harness
                .set("core.database", "other")
                .expect_err("the mode is unsafe")
                .exit_code(),
            78
        );
        assert_eq!(
            harness
                .remove("shop")
                .expect_err("the mode is unsafe")
                .exit_code(),
            78
        );
    }
}
