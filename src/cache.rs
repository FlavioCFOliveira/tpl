//! The catalogue cache: `.tpl/.cache/`, read through on every cached read and
//! written on every miss (`FR-CACHE-001` … `FR-CACHE-037`, `FR-CDOC-001` …
//! `FR-CDOC-016`).
//!
//! | Submodule | Subject | Forced by |
//! |---|---|---|
//! | [`paths`] | One folder per entry, one file per object, and the routine path | `FR-CACHE-001`, `FR-CACHE-002`, `FR-CDOC-014` |
//! | [`meta`] | The two versions, the load time, and the completeness record | `FR-CDOC-001` … `FR-CDOC-007`, `FR-CDOC-013` |
//!
//! # What is stored is the document, not the model
//!
//! Each file holds one member of the `database` object exactly as
//! [`crate::model::document`] builds it — a table with its references embedded
//! one hop, per `FR-CTX-006`; a view; a routine — beside one file carrying the
//! database's own three metadata fields and the `server` object of
//! `FR-CTX-031`. A read therefore reassembles a
//! [`DatabaseDocument`] and hands it
//! to the same presentation the server path hands one to, which is what makes
//! `BR-SCH-001` hold: a pattern, a listing and a named object select and print
//! the same thing whichever source served them.
//!
//! *Rejected: caching the model and rebuilding the document on every read.* The
//! embedding of `ADR-009` is materialised once, at the read that produced it,
//! and rebuilding it per cache hit would put a table's references together
//! from objects that were read at different instants — the very thing
//! `BR-CDOC-004` describes, done on every hit rather than only where the cache
//! genuinely holds objects of different ages.
//!
//! *Rejected: caching the whole dump as one document beside the per-object
//! files.* `BR-CDOC-004` rejects it in as many words: it stores everything
//! twice, and the two copies drift apart.
//!
//! # A cache that cannot be read or written is never an error
//!
//! Both directions fail silently, and the two requirements are deliberately
//! symmetric:
//!
//! - `FR-CACHE-033` and `FR-CDOC-004` — an unreadable file, or one written
//!   under a version this binary does not know, is a **miss**. The read goes to
//!   the server, the file is rewritten, and neither an error nor a warning is
//!   reported. Every lookup below therefore answers [`Option`] and never
//!   [`Result`].
//! - `FR-CACHE-036` — a cache that cannot be written leaves the answer
//!   untouched: exit `0`, stdout byte for byte what it would have been, the
//!   cache as it was found, and nothing said. [`Cache::write`] therefore
//!   returns `()`.
//!
//! The one operation that does report is [`Cache::clean`], and it reports
//! because removing is the whole of what the caller asked for: a `tpl cache
//! clean` that removed nothing and exited `0` would tell the caller the cache
//! is empty when it is not.
//!
//! # Nothing expires, and nothing invalidates
//!
//! There is no time-to-live and no automatic expiry (`FR-CACHE-008`), and
//! repointing an entry invalidates nothing (`FR-CACHE-029`) — the cache is
//! keyed by entry name and by nothing else, per `FR-CACHE-002`, so a repointed
//! entry keeps the previous server's catalogue and `BR-CACHE-003` accepts the
//! consequence. Only [`Cache::write`] and [`Cache::clean`] change what is
//! stored, which is `FR-CACHE-028`.
//!
//! # No lock, and one whole file or the other
//!
//! `FR-CACHE-031` takes no lock, and `FR-CACHE-030` writes each object through
//! a temporary file in the same directory, renamed over the target — so a
//! reader meets one whole version of a file or the other, never half of one,
//! and a killed process leaves nothing locked. What a **collection** offers is
//! weaker by the same requirement's design, and `FR-CDOC-015` states it: a
//! document served from the cache promises neither referential integrity nor a
//! point-in-time snapshot.

pub(crate) mod meta;
pub(crate) mod paths;

use std::borrow::Cow;
use std::collections::BTreeSet;
use std::fs;
use std::io::Write as _;
use std::os::unix::fs::OpenOptionsExt as _;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::error::Error;
use crate::model::document::DatabaseDocument;
use crate::model::document::order::{self, Named};
use crate::model::routine::RoutineKind;
use crate::model::server::Server;
use meta::Meta;
use paths::{Collection, Layout};

/// The mode every file of the cache is created with.
///
/// It is the mode `FR-SEC-004` fixes for `.tpl/.cfg` and it is applied here for
/// a weaker but real reason: a cached catalogue is the structure of a database
/// this user was granted sight of, and `.tpl/` is a per-machine directory, so
/// the cache is written no wider than the configuration that reached it.
const MODE: u32 = 0o600;

/// The prefix of the temporary file of `FR-CACHE-030`.
///
/// It begins with `.` and carries no `.json` extension, so a concurrent
/// directory walk does not read it as an object — which is the test
/// [`paths::is_object`] applies.
const TEMPORARY: &str = ".tpl-cache";

/// The database's own metadata, as the cache holds it.
///
/// It is the part of the `database` object of `FR-CTX-036` that belongs to no
/// collection: the three metadata fields and the `server` object of
/// `FR-CTX-031`. The key order is the field order, per `OD-18`, and it is the
/// order that requirement writes them in.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
struct Metadata<'a> {
    /// The schema's name.
    #[serde(borrow)]
    name: Cow<'a, str>,
    /// The schema's default character set.
    #[serde(borrow)]
    charset: Cow<'a, str>,
    /// The schema's default collation.
    #[serde(borrow)]
    collation: Cow<'a, str>,
    /// The server the read was made against.
    #[serde(borrow)]
    server: Server<'a>,
}

/// How much of the catalogue a read covered, which decides what a write stores
/// and what it may record as whole.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Covered {
    /// The whole catalogue of the entry: every table, view and routine.
    Everything,
    /// One member of one collection, named on the command line.
    One(Collection),
}

/// What one cache hit holds: the bytes of every file it read.
///
/// The bytes are owned and the document borrows them, which is the same
/// arrangement [`crate::mariadb::catalogue`] uses for a server read: nothing is
/// copied out of a file on the way into the document, and the lifetime that
/// makes it sound is the caller's hold on this value.
#[derive(Debug)]
pub(crate) struct Loaded {
    /// The contents of `database.json`.
    metadata: String,
    /// The contents of each table file read.
    tables: Vec<String>,
    /// The contents of each view file read.
    views: Vec<String>,
    /// The contents of each routine file read.
    routines: Vec<String>,
}

impl Loaded {
    /// The `database` object these bytes describe, or [`None`] where any of
    /// them fails to decode.
    ///
    /// A file this binary cannot decode is a **miss**, per `FR-CACHE-033`, and
    /// a miss is decided over the whole hit rather than per file: serving a
    /// listing with one member dropped is the wrong answer wearing the
    /// appearance of a right one that `BR-CDOC-002` exists to prevent.
    ///
    /// **Each collection is ordered here**, per `NFR-DET-002`. One file per
    /// object means the order a collection is read back in is the order the
    /// directory walk produced, which is the filesystem's and not this
    /// project's — so `BR-SCH-001` would fail on the one property it is about:
    /// a listing served from the store would present the same objects in
    /// another order from the same listing served live.
    pub(crate) fn document(&self) -> Option<DatabaseDocument<'_>> {
        let metadata: Metadata<'_> = serde_json::from_str(&self.metadata).ok()?;

        Some(DatabaseDocument {
            name: metadata.name,
            charset: metadata.charset,
            collation: metadata.collation,
            server: metadata.server,
            tables: decode(&self.tables)?,
            views: Cow::Owned(decode(&self.views)?),
            routines: Cow::Owned(decode(&self.routines)?),
        })
    }
}

/// Decodes every member of one collection, ordered by name, or [`None`] where
/// any fails.
///
/// The ordering is `NFR-DET-002`'s default rule, applied through the one
/// comparator [`crate::model::document::order`] holds: the directory walk that
/// produced these bytes has an order of its own, and it is the filesystem's.
fn decode<'a, T: Deserialize<'a> + Named>(members: &'a [String]) -> Option<Vec<T>> {
    let mut decoded: Vec<T> = members
        .iter()
        .map(|bytes| serde_json::from_str(bytes).ok())
        .collect::<Option<Vec<T>>>()?;

    order::sort_by_name(&mut decoded);

    Some(decoded)
}

/// What `tpl cache status` reports (`FR-CACHE-025`, `FR-CACHE-034`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct Status {
    /// The load time from `meta.json`, or [`None`] where the cache is empty
    /// (`FR-CDOC-013`, `FR-CACHE-035`).
    pub(crate) loaded_at: Option<String>,
    /// One entry per collection, or empty where the cache is empty.
    pub(crate) collections: Vec<Held>,
}

/// What one collection holds (`FR-CACHE-034`).
///
/// It is [`Serialize`] because `FR-CACHE-034` fixes the `json` form of
/// `tpl cache status` as exactly these three keys, in this order, and `OD-18`
/// makes a struct's field declaration order the emitted key order. A second
/// type mirroring this one in the command would state the same three keys in a
/// second place, which is the drift `OD-18` exists to prevent.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub(crate) struct Held {
    /// The collection's plural name.
    pub(crate) name: &'static str,
    /// How many objects it holds.
    pub(crate) count: usize,
    /// Whether it was loaded whole (`FR-CDOC-006`).
    pub(crate) whole: bool,
}

/// What a cached read asks the store for (`FR-CDOC-007`, `FR-CDOC-008`).
///
/// The two requirements draw the line and this type is that line: a **listing**
/// is served only where its own collection is recorded whole, and an
/// **individual object** is served whenever it is present, whatever its
/// collection's record says. A lookup that asked for more than the command
/// needs would turn a warm cache into a miss, and one that asked for less would
/// serve a listing short — which is the wrong answer `BR-CDOC-002` exists to
/// prevent.
///
/// It is [`Clone`] and not [`Copy`], because [`Look::Routine`] carries a
/// [`RoutineKind`] and `FR-CAT-055` gives that enumeration a variant holding
/// the catalogue's own string.
#[derive(Debug, Clone)]
pub(crate) enum Look<'a> {
    /// The whole catalogue, all three collections recorded whole.
    ///
    /// It is what `tpl schema info` and `tpl schema dump` ask for: the answer
    /// is the whole database, so a collection that was never loaded whole makes
    /// it a listing served short.
    Everything,

    /// One whole collection (`FR-CDOC-007`).
    Collection(Collection),

    /// One named table (`FR-CDOC-008`).
    Table(&'a str),

    /// One named view (`FR-CDOC-008`).
    View(&'a str),

    /// One routine of a known kind (`FR-CDOC-008`, `FR-CDOC-014`).
    ///
    /// A **bare** name is not this: the kind is part of the path, so a bare
    /// name reaches two files, and deciding between them is the ambiguity
    /// `FR-SCH-010` refuses over the population of the database. A store
    /// holding one of the two says nothing about whether the other exists on
    /// the server, so a bare name asks for [`Look::Collection`] instead and is
    /// served only from a routines collection recorded whole.
    Routine(RoutineKind<'a>, &'a str),
}

/// One database entry's cache.
///
/// It exists whether or not anything has been written: `FR-CACHE-003` keeps
/// `tpl init` from creating `.tpl/.cache/`, so the folder appears on the first
/// read that populates it and every lookup before that is an ordinary miss.
#[derive(Debug)]
pub(crate) struct Cache {
    /// Where this entry's files live, or [`None`] where the entry name is not
    /// a path component — in which case nothing is ever read or written and
    /// every read goes to the server.
    layout: Option<Layout>,
}

impl Cache {
    /// The cache of `entry`, under the `.tpl` folder at `tpl`.
    pub(crate) fn of(tpl: &Path, entry: &str) -> Self {
        Self {
            layout: Layout::of(tpl, entry),
        }
    }

    // ------------------------------------------------------------ reads ---

    /// The whole catalogue, or [`None`] on a miss (`FR-CACHE-006`).
    ///
    /// All three collections must be recorded whole, per `FR-CDOC-007`: the
    /// answer is the whole database, so a collection that was never loaded
    /// whole makes it a listing served short.
    pub(crate) fn everything(&self) -> Option<Loaded> {
        let layout = self.layout.as_ref()?;
        let meta = self.meta()?;

        if !Collection::ALL
            .iter()
            .all(|collection| meta.whole(*collection))
        {
            return None;
        }

        Some(Loaded {
            metadata: read(&layout.database())?,
            tables: members(&layout.collection(Collection::Tables))?,
            views: members(&layout.collection(Collection::Views))?,
            routines: members(&layout.collection(Collection::Routines))?,
        })
    }

    /// What `look` asks for, or [`None`] on a miss.
    ///
    /// It is the one entry point a read-through takes, so that the choice
    /// `FR-CDOC-007` and `FR-CDOC-008` make between a collection and a member
    /// is stated where the command decides it and applied in one place here.
    pub(crate) fn look(&self, look: &Look<'_>) -> Option<Loaded> {
        match look {
            Look::Everything => self.everything(),
            Look::Collection(collection) => self.collection(*collection),
            Look::Table(name) => self.table(name),
            Look::View(name) => self.view(name),
            Look::Routine(kind, name) => self.routine(kind, name),
        }
    }

    /// One whole collection, or [`None`] on a miss (`FR-CDOC-007`).
    pub(crate) fn collection(&self, collection: Collection) -> Option<Loaded> {
        let layout = self.layout.as_ref()?;

        if !self.meta()?.whole(collection) {
            return None;
        }

        let read = members(&layout.collection(collection))?;

        Some(self.one(collection, read_metadata(layout)?, read))
    }

    /// One table, or [`None`] on a miss (`FR-CDOC-008`).
    pub(crate) fn table(&self, name: &str) -> Option<Loaded> {
        let layout = self.layout.as_ref()?;
        self.member(Collection::Tables, layout.table(name)?)
    }

    /// One view, or [`None`] on a miss.
    pub(crate) fn view(&self, name: &str) -> Option<Loaded> {
        let layout = self.layout.as_ref()?;
        self.member(Collection::Views, layout.view(name)?)
    }

    /// One routine of a known kind, or [`None`] on a miss.
    ///
    /// The kind is part of the path, per `FR-CDOC-014`, so a bare name that
    /// could denote either is looked up as both by the caller: a cache that
    /// holds one of the two answers, and a cache that holds both puts the
    /// ambiguity of `FR-SCH-010` in the caller's hands exactly as a server read
    /// does.
    pub(crate) fn routine(&self, kind: &RoutineKind<'_>, name: &str) -> Option<Loaded> {
        let layout = self.layout.as_ref()?;
        self.member(Collection::Routines, layout.routine(kind, name)?)
    }

    /// One named member of `collection`, from `file`.
    fn member(&self, collection: Collection, file: PathBuf) -> Option<Loaded> {
        let layout = self.layout.as_ref()?;

        // The versions govern every file of the folder, per FR-CDOC-004, so
        // they are checked before a member is read as well as before a
        // collection is.
        self.meta()?;

        let bytes = read(&file)?;

        Some(self.one(collection, read_metadata(layout)?, vec![bytes]))
    }

    /// A hit carrying `read` as the members of `collection`.
    fn one(&self, collection: Collection, metadata: String, read: Vec<String>) -> Loaded {
        let mut loaded = Loaded {
            metadata,
            tables: Vec::new(),
            views: Vec::new(),
            routines: Vec::new(),
        };

        match collection {
            Collection::Tables => loaded.tables = read,
            Collection::Views => loaded.views = read,
            Collection::Routines => loaded.routines = read,
        }

        loaded
    }

    /// The record `meta.json` carries, or [`None`] where it is absent,
    /// unreadable, or written under a version this binary does not know.
    fn meta(&self) -> Option<Meta> {
        let layout = self.layout.as_ref()?;
        let meta: Meta = serde_json::from_str(&read(&layout.meta())?).ok()?;

        meta.usable().then_some(meta)
    }

    // ----------------------------------------------------------- writes ---

    /// Stores what `document` carries, as far as `covered` reaches
    /// (`FR-CACHE-007`, `FR-CACHE-030`, `FR-CACHE-037`).
    ///
    /// It answers nothing, because there is nothing a caller may do with the
    /// answer: `FR-CACHE-036` requires a cache that cannot be written to leave
    /// the exit code and every byte of stdout exactly as they would have been,
    /// and to report neither an error nor a warning. Every failure below is
    /// therefore swallowed at the point it arises, and the cache is left as
    /// whole as it was found.
    ///
    /// An object marked `restricted` is never written, per `FR-CACHE-037`, and
    /// a collection holding one is not recorded whole — otherwise
    /// `FR-CDOC-007` would serve a listing from the cache with that object
    /// simply missing.
    pub(crate) fn write(&self, document: &DatabaseDocument<'_>, covered: Covered) {
        let Some(layout) = self.layout.as_ref() else {
            return;
        };
        if fs::create_dir_all(layout.folder()).is_err() {
            return;
        }

        let metadata = Metadata {
            name: document.name.clone(),
            charset: document.charset.clone(),
            collation: document.collation.clone(),
            server: document.server.clone(),
        };
        if !store(&layout.database(), &metadata) {
            return;
        }

        match covered {
            Covered::Everything => {
                let whole = [
                    replace(layout, Collection::Tables, &document.tables, table_file),
                    replace(
                        layout,
                        Collection::Views,
                        document.views.as_ref(),
                        view_file,
                    ),
                    replace(
                        layout,
                        Collection::Routines,
                        document.routines.as_ref(),
                        routine_file,
                    ),
                ];

                let _ = store(&layout.meta(), &Meta::new(whole));
            }
            Covered::One(_) => {
                for table in &document.tables {
                    one(layout, table, table_file);
                }
                for view in document.views.as_ref() {
                    one(layout, view, view_file);
                }
                for routine in document.routines.as_ref() {
                    one(layout, routine, routine_file);
                }

                // A named read refreshes one object and settles nothing about
                // the collection it belongs to: `FR-CACHE-028` admits no
                // automatic invalidation, so a collection that was whole stays
                // whole and one that was not stays not.
                let record = self.meta().map_or_else(
                    || Meta::new([false; Collection::ALL.len()]),
                    |previous| previous.refreshed(),
                );

                let _ = store(&layout.meta(), &record);
            }
        }
    }

    // ---------------------------------------------------------- removal ---

    /// Removes everything stored for this entry (`FR-CACHE-023`).
    ///
    /// # Errors
    ///
    /// Returns [`Error::ProjectFileUnwritable`] where the removal failed. This
    /// is the one cache operation that reports: the caller asked for the data
    /// to be gone, and a `0` over a cache that is still there would be a wrong
    /// answer rather than a missed optimisation.
    pub(crate) fn clean(&self) -> Result<(), Error> {
        let Some(layout) = self.layout.as_ref() else {
            return Ok(());
        };

        match fs::remove_dir_all(layout.folder()) {
            Ok(()) => Ok(()),
            // Nothing stored is the state the caller asked for, per the
            // reading FR-CACHE-026 applies to an empty cache: empty is a state,
            // not a failure.
            Err(returned) if returned.kind() == std::io::ErrorKind::NotFound => Ok(()),
            Err(returned) => Err(Error::ProjectFileUnwritable {
                path: layout.folder().to_owned(),
                returned,
            }),
        }
    }

    /// Removes one cached object, and records that its collection is no longer
    /// whole (`FR-CACHE-023`).
    ///
    /// Clearing the flag is what `BR-CDOC-002` requires: a listing served from
    /// a collection one object has been removed from would return every other
    /// object and exit `0`, which is the failure the completeness record
    /// exists to prevent.
    ///
    /// # Errors
    ///
    /// Returns [`Error::ProjectFileUnwritable`] where the removal failed.
    pub(crate) fn clean_one(
        &self,
        collection: Collection,
        file: Option<PathBuf>,
    ) -> Result<(), Error> {
        let Some(layout) = self.layout.as_ref() else {
            return Ok(());
        };
        let Some(file) = file else {
            return Ok(());
        };

        // An object that is not there is the state the caller asked for, by
        // the reading `FR-CACHE-026` applies to an empty cache.
        if let Err(returned) = fs::remove_file(&file)
            && returned.kind() != std::io::ErrorKind::NotFound
        {
            return Err(Error::ProjectFileUnwritable {
                path: file,
                returned,
            });
        }

        if let Some(previous) = self.meta() {
            let mut record = previous.refreshed();
            for recorded in &mut record.collections {
                if recorded.name == collection.name() {
                    recorded.whole = false;
                }
            }
            let _ = store(&layout.meta(), &record);
        }

        Ok(())
    }

    /// The name of the database this store was read from, or [`None`] where
    /// nothing readable is stored.
    ///
    /// It is the one field of a cached document a command needs **without**
    /// serving a read: `tpl cache clean --routine <bare name>` refuses an
    /// ambiguity under `FR-CACHE-024`, and the `cause` of that refusal names
    /// the database the two objects were found in, per the `64` row of
    /// `FR-ERR-034`. Resolving the entry far enough to learn it would run the
    /// `password_command` of `FR-CONF-023` for a command that authenticates to
    /// nothing, so the store is asked instead.
    pub(crate) fn database(&self) -> Option<String> {
        let layout = self.layout.as_ref()?;
        let bytes = read_metadata(layout)?;
        let metadata: Metadata<'_> = serde_json::from_str(&bytes).ok()?;

        Some(metadata.name.into_owned())
    }

    /// Whether the object at `file` is stored.
    ///
    /// It is the test `FR-CACHE-024` needs for a bare routine name: procedures
    /// and functions occupy distinct namespaces, so a bare name may reach two
    /// files, and the command must know which of them exist before it removes
    /// one or refuses both.
    pub(crate) fn holds(file: Option<&Path>) -> bool {
        file.is_some_and(Path::is_file)
    }

    /// The file one table is held in, for [`Cache::clean_one`].
    pub(crate) fn table_file(&self, name: &str) -> Option<PathBuf> {
        self.layout.as_ref()?.table(name)
    }

    /// The file one view is held in, for [`Cache::clean_one`].
    pub(crate) fn view_file(&self, name: &str) -> Option<PathBuf> {
        self.layout.as_ref()?.view(name)
    }

    /// The file one routine is held in, for [`Cache::clean_one`].
    pub(crate) fn routine_file(&self, kind: &RoutineKind<'_>, name: &str) -> Option<PathBuf> {
        self.layout.as_ref()?.routine(kind, name)
    }

    // ----------------------------------------------------------- status ---

    /// What this entry's cache holds (`FR-CACHE-025`, `FR-CACHE-034`,
    /// `FR-CACHE-035`).
    ///
    /// An empty cache — one with no usable record — reports `loaded_at` absent
    /// and no collections, and that is a success: `FR-CACHE-026` makes empty a
    /// state rather than a failure, and `FR-OUT-033` generalises it.
    pub(crate) fn status(&self) -> Status {
        let (Some(layout), Some(meta)) = (self.layout.as_ref(), self.meta()) else {
            return Status {
                loaded_at: None,
                collections: Vec::new(),
            };
        };

        Status {
            loaded_at: Some(meta.loaded_at.clone()),
            collections: Collection::ALL
                .iter()
                .map(|collection| Held {
                    name: collection.name(),
                    count: members(&layout.collection(*collection)).map_or(0, |held| held.len()),
                    whole: meta.whole(*collection),
                })
                .collect(),
        }
    }
}

/// The contents of `file`, or [`None`] where it could not be read.
///
/// Every failure is one answer, per `FR-CACHE-033`: absent, unreadable,
/// refused by the operating system, or not valid UTF-8 are all a miss, and none
/// of them is reported.
fn read(file: &Path) -> Option<String> {
    fs::read_to_string(file).ok()
}

/// The contents of `database.json`, or [`None`] on a miss.
fn read_metadata(layout: &Layout) -> Option<String> {
    read(&layout.database())
}

/// The contents of every object file of `directory`, or [`None`] where the
/// directory could not be walked.
///
/// A file that is not an object — the temporary of a write in flight, anything
/// a hand left there — is skipped, per [`paths::is_object`]. A file that is an
/// object and cannot be read makes the whole collection a miss: serving the
/// rest would be a listing short one member, which `BR-CDOC-002` refuses.
fn members(directory: &Path) -> Option<Vec<String>> {
    let mut held = Vec::new();

    for entry in fs::read_dir(directory).ok()? {
        let path = entry.ok()?.path();

        if paths::is_object(&path) {
            held.push(read(&path)?);
        }
    }

    Some(held)
}

/// Writes `value` to `file`, through a temporary file in the same directory
/// renamed over the target (`FR-CACHE-030`).
///
/// The rename is what makes a concurrent reader see one whole version of the
/// file or the other, per `FR-CACHE-031`, and it is why no lock is taken. The
/// temporary carries the process id in its name, so two processes writing the
/// same object write two temporaries and race only on the rename — which the
/// operating system makes atomic.
///
/// Answers whether the object is now stored. Every failure answers `false` and
/// reports nothing, per `FR-CACHE-036`.
fn store<T: Serialize>(file: &Path, value: &T) -> bool {
    let Some(directory) = file.parent() else {
        return false;
    };
    let temporary = directory.join(format!("{TEMPORARY}.{}.tmp", std::process::id()));

    let written = || -> std::io::Result<()> {
        let mut handle = fs::OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(true)
            .mode(MODE)
            .open(&temporary)?;

        serde_json::to_writer(&mut handle, value).map_err(std::io::Error::from)?;
        handle.write_all(b"\n")?;
        handle.sync_all()?;
        drop(handle);

        fs::rename(&temporary, file)
    }();

    if written.is_err() {
        // The target is untouched by a failed write, per FR-CACHE-036, and the
        // temporary is not left behind to be walked over.
        let _ = fs::remove_file(&temporary);
        return false;
    }

    true
}

/// Removes every object file of `directory` that `written` does not name, and
/// answers how many object files remain.
///
/// The count is the filesystem's own and is what decides whether a collection
/// may be recorded whole: on a case-insensitive filesystem two names that
/// differ only in case are one file, and the difference between the paths
/// written and the files that exist is the only way to find out.
fn prune(directory: &Path, written: &BTreeSet<PathBuf>) -> usize {
    let Ok(entries) = fs::read_dir(directory) else {
        return usize::MAX;
    };
    let mut held = 0;

    for entry in entries.flatten() {
        let path = entry.path();

        if !paths::is_object(&path) {
            continue;
        }

        if written.contains(&path) {
            held += 1;
        } else if fs::remove_file(&path).is_err() {
            // A stale object that cannot be removed would be served in every
            // later listing, so the collection is not whole while it is there.
            held += 1;
        }
    }

    held
}

/// Where one table is stored, or [`None`] where it must not be stored at all.
fn table_file(
    layout: &Layout,
    table: &crate::model::document::shape::TableDocument<'_>,
) -> Option<PathBuf> {
    table
        .restricted
        .is_none()
        .then(|| layout.table(&table.name))?
}

/// Where one view is stored, or [`None`] where it must not be stored at all.
fn view_file(layout: &Layout, view: &crate::model::view::View<'_>) -> Option<PathBuf> {
    view.restricted.is_none().then(|| layout.view(&view.name))?
}

/// Where one routine is stored, or [`None`] where it must not be stored at all
/// (`FR-CDOC-014`).
fn routine_file(layout: &Layout, routine: &crate::model::routine::Routine<'_>) -> Option<PathBuf> {
    routine
        .restricted
        .is_none()
        .then(|| layout.routine(&routine.kind, &routine.name))?
}

/// Replaces one collection with `members`, and answers whether it may be
/// recorded whole (`FR-CACHE-037`, `FR-CDOC-006`).
///
/// The order is: write every member, then remove every object file the write
/// did not produce, then count what the directory holds. The count is what
/// decides completeness, and it is taken from the filesystem rather than from
/// the loop: two names differing only in case are two paths and one file on a
/// case-insensitive filesystem, and only the directory knows which of the two
/// this host is.
///
/// The prune is what keeps a whole collection whole. Without it an object
/// dropped on the server since the last read would stay in the cache and be
/// served in every later listing, at exit `0`, with nothing to say so.
///
/// `file_of` answers [`None`] for a member that must not be stored — one marked
/// `restricted`, per `FR-CACHE-037`, or one whose name is not a path component.
/// Both leave the collection incomplete, and for the same reason: the listing
/// the cache would serve is short one member.
fn replace<T, F>(layout: &Layout, collection: Collection, members: &[T], file_of: F) -> bool
where
    T: Serialize,
    F: Fn(&Layout, &T) -> Option<PathBuf>,
{
    let directory = layout.collection(collection);
    if fs::create_dir_all(&directory).is_err() {
        return false;
    }

    let mut written: BTreeSet<PathBuf> = BTreeSet::new();
    let mut complete = true;

    for member in members {
        match file_of(layout, member) {
            Some(file) if store(&file, member) => {
                written.insert(file);
            }
            _ => complete = false,
        }
    }

    complete && prune(&directory, &written) == members.len()
}

/// Writes one named member, or leaves the cache as it was.
fn one<T, F>(layout: &Layout, member: &T, file_of: F)
where
    T: Serialize,
    F: Fn(&Layout, &T) -> Option<PathBuf>,
{
    if let Some(file) = file_of(layout, member)
        && let Some(parent) = file.parent()
        && fs::create_dir_all(parent).is_ok()
    {
        let _ = store(&file, member);
    }
}
