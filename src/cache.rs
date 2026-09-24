//! The catalogue cache: `.tpl/.cache/`, read through on every cached read and
//! written on every miss (`FR-CACHE-001` … `FR-CACHE-039`, `FR-CDOC-001` …
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
//! Two further files are a miss wherever they are read, and both for the same
//! reason: the file is not one this binary wrote for the object asked for.
//!
//! - `FR-CACHE-033`, `FR-CDOC-008` — a read that serves one named object from
//!   its own file ([`Cache::table`], [`Cache::view`], [`Cache::routine`]) and
//!   finds another object in it, differing in kind or, byte for byte, in name.
//!   On a filesystem that folds case or Unicode normalisation two objects share
//!   one file (finding SEC-03 of `SECURITY-AUDIT.md`), and serving the other
//!   one would answer that the requested object does not exist, naming a
//!   server that was never asked. The file-naming arrangement is unchanged.
//! - `FR-CACHE-033`, `FR-CACHE-030` — an object file that is a symbolic link
//!   (hardening observation H-2). It is never read through, whatever it points
//!   at; the rewrite that follows the miss replaces the link, which
//!   [`write_through`]'s rename already does.
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
//! a temporary file in the same directory, renamed over the target, or leaves
//! in place a file that already holds exactly the bytes the write would
//! produce — so a reader meets one whole version of a file or the other, never
//! half of one, and a killed process leaves nothing locked. What a **collection** offers is
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
use crate::model::ToStatic as _;
use crate::model::document::DatabaseDocument;
use crate::model::document::order::{self, Named};
use crate::model::document::shape::TableDocument;
use crate::model::routine::{Routine, RoutineKind};
use crate::model::server::Server;
use crate::model::view::View;
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
    /// The one object a read of one named object asked for, or [`None`] for a
    /// read of a collection or of everything.
    requested: Option<Requested>,
}

/// The object a read of one named object asked for (`FR-CDOC-008`).
///
/// A file at that object's path is **present**, in the sense `FR-CDOC-008`
/// serves, only where it holds that object: the same kind and, byte for byte,
/// the same name (`FR-CACHE-033`). The kind of a table or a view is the
/// collection the file sits in and the type it decodes as, so only a routine
/// carries a kind to compare.
#[derive(Debug)]
struct Requested {
    /// The collection the object belongs to.
    collection: Collection,
    /// The name asked for, as the command line wrote it.
    name: String,
    /// The routine kind asked for; [`None`] for a table or a view.
    kind: Option<RoutineKind<'static>>,
}

impl Requested {
    /// Whether `document`, reassembled from one object file, holds exactly the
    /// object asked for.
    fn held_by(&self, document: &DatabaseDocument<'_>) -> bool {
        match self.collection {
            Collection::Tables => {
                matches!(document.tables.as_slice(), [table] if table.name == self.name)
            }
            Collection::Views => {
                matches!(document.views.as_ref(), [view] if view.name == self.name)
            }
            Collection::Routines => matches!(
                document.routines.as_ref(),
                [routine] if routine.name == self.name && Some(&routine.kind) == self.kind.as_ref()
            ),
        }
    }
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
    ///
    /// **A read of one named object is a miss where its file holds another**
    /// (`FR-CACHE-033`, `FR-CDOC-008`): the check is made here, over the
    /// member already decoded, so it costs a comparison and no second parse.
    pub(crate) fn document(&self) -> Option<DatabaseDocument<'_>> {
        let metadata: Metadata<'_> = serde_json::from_str(&self.metadata).ok()?;

        let document = DatabaseDocument {
            name: metadata.name,
            charset: metadata.charset,
            collation: metadata.collation,
            server: metadata.server,
            tables: decode(&self.tables)?,
            views: Cow::Owned(decode(&self.views)?),
            routines: Cow::Owned(decode(&self.routines)?),
        };

        match &self.requested {
            Some(requested) if !requested.held_by(&document) => None,
            _ => Some(document),
        }
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

/// What a render served from the cache reads before it starts (`FR-CACHE-038`):
/// the bytes of `database.json`, and the members of each collection as the
/// directory listing names them.
///
/// It is [`Loaded`]'s counterpart for `tpl render`, which binds the whole
/// `database` but reaches, as a rule, a small part of it. No object file is
/// opened here: each [`Shelf`] names one, and the file is read and decoded only
/// when the template first reaches that member, through [`Shelf::read`] and one
/// of [`Shelf::table`], [`Shelf::view`] and [`Shelf::routine`].
///
/// **The members are ordered here**, by the one comparator
/// [`Loaded::document`] orders by, over the names the paths hold. For every
/// file this binary writes, the name a path holds is the name the file holds,
/// so the order, the length and the members of each collection are those an
/// up-front read of the same files produces. A file whose contents name
/// another object is refused when it is reached, which is the only point at
/// which the difference can be seen.
///
/// *Rejected: reading each file's name up front to order the collection.* The
/// open is most of the cost of a file (`BENCHMARKS.md`, 2026-09-23, `#243`),
/// and `FR-CACHE-038` forbids reading a member's file before it is reached.
#[derive(Debug)]
pub(crate) struct Shelved {
    /// The contents of `database.json`.
    metadata: String,
    /// The tables, ordered by name.
    tables: Vec<Shelf>,
    /// The views, ordered by name.
    views: Vec<Shelf>,
    /// The routines, ordered by name.
    routines: Vec<Shelf>,
}

impl Shelved {
    /// The database's own members — the three metadata fields and the `server`
    /// object — with its three collections empty, or [`None`] where
    /// `database.json` fails to decode, which is a miss per `FR-CACHE-033`.
    ///
    /// The collections are the shelves: [`Shelved::tables`],
    /// [`Shelved::views`] and [`Shelved::routines`].
    pub(crate) fn head(&self) -> Option<DatabaseDocument<'_>> {
        let metadata: Metadata<'_> = serde_json::from_str(&self.metadata).ok()?;

        Some(DatabaseDocument {
            name: metadata.name,
            charset: metadata.charset,
            collation: metadata.collation,
            server: metadata.server,
            tables: Vec::new(),
            views: Cow::Owned(Vec::new()),
            routines: Cow::Owned(Vec::new()),
        })
    }

    /// The tables, ordered by name.
    pub(crate) fn tables(&self) -> &[Shelf] {
        &self.tables
    }

    /// The views, ordered by name.
    pub(crate) fn views(&self) -> &[Shelf] {
        &self.views
    }

    /// The routines, ordered by name.
    pub(crate) fn routines(&self) -> &[Shelf] {
        &self.routines
    }
}

/// One member of a collection, known by its file and not yet read
/// (`FR-CACHE-038`).
#[derive(Debug)]
pub(crate) struct Shelf {
    /// The name the path holds.
    name: String,
    /// The kind the path holds, for a routine; [`None`] for a table or a view.
    kind: Option<RoutineKind<'static>>,
    /// The object file.
    file: PathBuf,
}

impl Shelf {
    /// The routine's kind, as its path holds it; [`None`] for a table or a
    /// view.
    pub(crate) const fn kind(&self) -> Option<&RoutineKind<'static>> {
        self.kind.as_ref()
    }

    /// The contents of the member's file, or [`None`] on a miss
    /// (`FR-CACHE-033`): absent — removed since the listing was read, which
    /// `FR-CACHE-039` names — a symbolic link, unreadable, or not UTF-8.
    pub(crate) fn read(&self) -> Option<String> {
        read_object(&self.file)
    }

    /// The table `bytes` hold, or [`None`] on a miss.
    ///
    /// The decode is [`Loaded::document`]'s, so a file is refused here exactly
    /// where an up-front read would have refused it. A file that decodes and
    /// names another table than its path is refused as well: the collection was
    /// ordered and counted by the path, so serving it would present a member
    /// out of the order and under a name the rest of the render has already
    /// seen. No write of this binary produces one.
    pub(crate) fn table<'a>(&self, bytes: &'a str) -> Option<TableDocument<'a>> {
        self.decoded(bytes)
    }

    /// The view `bytes` hold, or [`None`] on a miss, on the terms of
    /// [`Shelf::table`].
    pub(crate) fn view<'a>(&self, bytes: &'a str) -> Option<View<'a>> {
        self.decoded(bytes)
    }

    /// The routine `bytes` hold, or [`None`] on a miss, on the terms of
    /// [`Shelf::table`]; the kind must be the one the path holds as well.
    pub(crate) fn routine<'a>(&self, bytes: &'a str) -> Option<Routine<'a>> {
        let routine: Routine<'a> = self.decoded(bytes)?;

        (self.kind.as_ref() == Some(&routine.kind)).then_some(routine)
    }

    /// The member `bytes` hold, where it names this shelf's member.
    fn decoded<'a, T: Deserialize<'a> + Named>(&self, bytes: &'a str) -> Option<T> {
        let member: T = serde_json::from_str(bytes).ok()?;

        (member.name() == self.name).then_some(member)
    }
}

impl Named for Shelf {
    /// The member's name, as its path holds it.
    fn name(&self) -> &str {
        &self.name
    }
}

impl Loaded {
    /// The rows of the `text` listing of `tpl schema tables`, ordered by name,
    /// or [`None`] where any table file fails to decode.
    ///
    /// Each file is decoded into [`Listed`], which reads the four members the
    /// listing prints and parses the rest without building it; the order and
    /// the miss are [`Loaded::document`]'s.
    ///
    /// *Rejected: decoding every [`TableDocument`] in full and dropping all but
    /// four fields.* It was 62.0% of the samples of a cached text listing of
    /// `WL-001` and allocated 23 709 970 B (`BENCHMARKS.md`, 2026-09-22).
    pub(crate) fn listing(&self) -> Option<Vec<Listed<'_>>> {
        decode(&self.tables)
    }
}

/// One row of the `text` listing of `tpl schema tables` (`FR-SCH-026`): the
/// four members of a table that listing prints.
///
/// It is a reduced view of [`TableDocument`], decoded from the same file: the
/// four members are read as that type reads them, the columns are counted
/// rather than built, and every other member is parsed and dropped, as serde
/// drops a member no field names. The count is the length of the `columns`
/// array, which is what the full decode's collection holds.
///
/// A file this view decodes and the full decode would refuse — JSON of the
/// document's shape whose unprinted members break the contract — is served
/// here where [`Loaded::document`] would have missed. No file this binary
/// writes is one, and a torn or truncated file is not JSON and is a miss here
/// too.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub(crate) struct Listed<'a> {
    /// The table's name.
    #[serde(borrow)]
    pub(crate) name: Cow<'a, str>,

    /// The storage engine.
    #[serde(borrow)]
    pub(crate) engine: Option<Cow<'a, str>>,

    /// How many columns the table has.
    #[serde(rename = "columns", deserialize_with = "counted")]
    pub(crate) column_count: usize,

    /// The comment, the empty string where none was given.
    #[serde(borrow)]
    pub(crate) comment: Cow<'a, str>,
}

impl<'a> Listed<'a> {
    /// The row of `table`, for a listing the server served.
    pub(crate) fn of(table: &'a TableDocument<'_>) -> Self {
        Self {
            name: Cow::Borrowed(&table.name),
            engine: table.engine.as_deref().map(Cow::Borrowed),
            column_count: table.columns.len(),
            comment: Cow::Borrowed(&table.comment),
        }
    }
}

impl Named for Listed<'_> {
    fn name(&self) -> &str {
        &self.name
    }
}

/// The length of an array, each member parsed and dropped.
fn counted<'de, D: serde::Deserializer<'de>>(deserializer: D) -> Result<usize, D::Error> {
    /// The visitor that counts.
    struct Counting;

    impl<'de> serde::de::Visitor<'de> for Counting {
        type Value = usize;

        fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            formatter.write_str("a sequence")
        }

        fn visit_seq<A: serde::de::SeqAccess<'de>>(self, mut seq: A) -> Result<usize, A::Error> {
            let mut counted = 0;
            while seq.next_element::<serde::de::IgnoredAny>()?.is_some() {
                counted += 1;
            }
            Ok(counted)
        }
    }

    deserializer.deserialize_seq(Counting)
}

/// What a cache hit for `tpl schema info` holds: the bytes of `database.json`
/// and the number of object files of each collection.
///
/// It is [`Loaded`]'s counterpart for the one command that presents the
/// database's own metadata and the size of each collection, and nothing of any
/// member.
#[derive(Debug)]
pub(crate) struct Summarised {
    /// The contents of `database.json`.
    metadata: String,
    /// The object files of each collection, in [`Collection::ALL`]'s order.
    counts: [usize; 3],
}

impl Summarised {
    /// What these bytes describe, or [`None`] where `database.json` fails to
    /// decode, which is a miss per `FR-CACHE-033`.
    pub(crate) fn summary(&self) -> Option<Summary<'_>> {
        let metadata: Metadata<'_> = serde_json::from_str(&self.metadata).ok()?;
        let [tables, views, routines] = self.counts;

        Some(Summary {
            name: metadata.name,
            charset: metadata.charset,
            collation: metadata.collation,
            server: metadata.server,
            tables,
            views,
            routines,
        })
    }
}

/// What `tpl schema info` presents (`FR-SCH-003`, `FR-SCH-031`): the three
/// metadata fields of `FR-CTX-036`, the `server` object of `FR-CTX-031`, and
/// the size of each of the three collections of `FR-CTX-035`.
///
/// Both sources produce it: [`Summarised::summary`] from the cache, and
/// [`Summary::of`] from a document the server served. The members are the
/// same members, under the same names and with the same values, that the
/// `database` object of the context document carries, which is what
/// `FR-SCH-031` promises a caller that reads `data.database.name` from either
/// command.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct Summary<'a> {
    /// The schema's name.
    pub(crate) name: Cow<'a, str>,
    /// The schema's default character set.
    pub(crate) charset: Cow<'a, str>,
    /// The schema's default collation.
    pub(crate) collation: Cow<'a, str>,
    /// The server the read was made against.
    pub(crate) server: Server<'a>,
    /// How many tables the database holds.
    pub(crate) tables: usize,
    /// How many views it holds.
    pub(crate) views: usize,
    /// How many routines it holds.
    pub(crate) routines: usize,
}

impl<'a> Summary<'a> {
    /// The summary of `document`, for a read the server served.
    pub(crate) fn of(document: &'a DatabaseDocument<'_>) -> Self {
        Self {
            name: Cow::Borrowed(&document.name),
            charset: Cow::Borrowed(&document.charset),
            collation: Cow::Borrowed(&document.collation),
            server: document.server.clone(),
            tables: document.tables.len(),
            views: document.views.len(),
            routines: document.routines.len(),
        }
    }
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
    /// It is what `tpl schema dump` asks for: the answer is the whole
    /// database, so a collection that was never loaded whole makes it a
    /// listing served short. `tpl schema info` asks [`Cache::summary`]
    /// instead, under the same rule.
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

/// What `.tpl/.cache/` recorded for the path [`Cache::clean_orphan`]
/// removed (`FR-CACHE-041`, item 2).
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct Recorded {
    /// The member of `.tpl/.cache/` equal to the name given byte for byte,
    /// or, where none is, the member the path resolved to; [`None`] where
    /// neither could be found or read as text.
    pub(crate) name: Option<String>,
}

/// The name `.tpl/.cache/` lists for `path`, whose metadata, read without
/// following a link, is `found`.
///
/// The member equal to the last component byte for byte wins; otherwise the
/// member with the same device and inode, which is the one a filesystem that
/// ignores case resolved the path to.
fn recorded_name(path: &Path, found: &fs::Metadata) -> Option<String> {
    use std::os::unix::fs::MetadataExt as _;

    let given = path.file_name()?;
    let members = fs::read_dir(path.parent()?).ok()?;
    let mut resolved = None;

    for member in members.flatten() {
        let name = member.file_name();
        if name == given {
            return name.into_string().ok();
        }
        if resolved.is_none()
            && let Ok(metadata) = fs::symlink_metadata(member.path())
            && metadata.dev() == found.dev()
            && metadata.ino() == found.ino()
        {
            resolved = Some(name);
        }
    }

    resolved?.into_string().ok()
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
            requested: None,
        })
    }

    /// What a render reads before it starts, or [`None`] on a miss
    /// (`FR-CACHE-038`).
    ///
    /// It is a hit exactly where [`Cache::everything`] would reach its object
    /// files: the record is read and checked first, all three collections must
    /// be recorded whole, `database.json` must be readable, and each
    /// collection's directory must be walkable. The object files are listed
    /// and not opened. A listed file whose path no write of this binary
    /// composes — see [`paths::member_of`] — is a miss here, because its member
    /// cannot be named or ordered without opening it.
    pub(crate) fn shelved(&self) -> Option<Shelved> {
        let layout = self.layout.as_ref()?;
        let meta = self.meta()?;

        if !Collection::ALL
            .iter()
            .all(|collection| meta.whole(*collection))
        {
            return None;
        }

        Some(Shelved {
            metadata: read(&layout.database())?,
            tables: shelve(layout, Collection::Tables)?,
            views: shelve(layout, Collection::Views)?,
            routines: shelve(layout, Collection::Routines)?,
        })
    }

    /// What `tpl schema info` presents, or [`None`] on a miss (`FR-SCH-031`).
    ///
    /// It is a hit exactly where [`Cache::everything`] is one for a store this
    /// binary wrote: the record is read and checked first, all three
    /// collections must be recorded whole, `database.json` must be readable,
    /// and each collection's directory must be walkable. The collections are
    /// **counted** rather than read, as `tpl cache status` counts them — the
    /// same walk, admitted by the same [`paths::is_object`] — because the
    /// command presents their sizes and nothing of their members. For a file
    /// every write of this binary produces, the count is the number of members
    /// [`Loaded::document`] decodes.
    ///
    /// **One difference from reading everything.** An object file that cannot
    /// be read or decoded no longer makes this command a miss, because the
    /// command no longer reads it: the file is counted, and it remains the
    /// miss of `FR-CACHE-033` for every command that does read it.
    /// `FR-SCH-031` puts the `text` form outside that requirement, and the
    /// `json` form carries no count.
    ///
    /// *Rejected: decoding all 272 files of `WL-001` to present four members
    /// and three counts.* It took 12.2 ms and allocated 23 945 725 B, where
    /// `tpl schema table` from the same cache took 2.0 ms (`BENCHMARKS.md`,
    /// 2026-09-22), which is what `FR-SCH-031` rejects as making this "the most
    /// expensive command of this arm while presenting the least".
    ///
    /// *Rejected: skipping the three walks for the `json` form.* A store whose
    /// collection directory is gone would then be a hit for one form and a
    /// miss for the other, and the `source` of `FR-SCH-035` would depend on
    /// `--format`. Three directory walks are the whole of the difference.
    pub(crate) fn summary(&self) -> Option<Summarised> {
        let layout = self.layout.as_ref()?;
        let meta = self.meta()?;

        if !Collection::ALL
            .iter()
            .all(|collection| meta.whole(*collection))
        {
            return None;
        }

        Some(Summarised {
            metadata: read(&layout.database())?,
            counts: [
                count(&layout.collection(Collection::Tables))?,
                count(&layout.collection(Collection::Views))?,
                count(&layout.collection(Collection::Routines))?,
            ],
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

        Some(self.one(collection, read_metadata(layout)?, read, None))
    }

    /// One table, or [`None`] on a miss (`FR-CDOC-008`).
    ///
    /// The file at the table's path is a miss where it is a symbolic link, and
    /// the hit it produces is a miss at [`Loaded::document`] where the file
    /// holds a table of another name (`FR-CACHE-033`).
    pub(crate) fn table(&self, name: &str) -> Option<Loaded> {
        let layout = self.layout.as_ref()?;
        self.member(Collection::Tables, layout.table(name)?, name, None)
    }

    /// One view, or [`None`] on a miss, on the terms of [`Cache::table`].
    pub(crate) fn view(&self, name: &str) -> Option<Loaded> {
        let layout = self.layout.as_ref()?;
        self.member(Collection::Views, layout.view(name)?, name, None)
    }

    /// One routine of a known kind, or [`None`] on a miss.
    ///
    /// The kind is part of the path, per `FR-CDOC-014`, so a bare name that
    /// could denote either is looked up as both by the caller: a cache that
    /// holds one of the two answers, and a cache that holds both puts the
    /// ambiguity of `FR-SCH-010` in the caller's hands exactly as a server read
    /// does.
    ///
    /// It is a miss on the terms of [`Cache::table`], and where the file holds
    /// a routine of the other kind as well (`FR-CACHE-033`).
    pub(crate) fn routine(&self, kind: &RoutineKind<'_>, name: &str) -> Option<Loaded> {
        let layout = self.layout.as_ref()?;
        self.member(
            Collection::Routines,
            layout.routine(kind, name)?,
            name,
            Some(kind.to_static()),
        )
    }

    /// The member of `collection` named `name` — of `kind`, for a routine —
    /// from `file`.
    fn member(
        &self,
        collection: Collection,
        file: PathBuf,
        name: &str,
        kind: Option<RoutineKind<'static>>,
    ) -> Option<Loaded> {
        let layout = self.layout.as_ref()?;

        // The versions govern every file of the folder, per FR-CDOC-004, so
        // they are checked before a member is read as well as before a
        // collection is.
        self.meta()?;

        let bytes = read_object(&file)?;
        let requested = Requested {
            collection,
            name: name.to_owned(),
            kind,
        };

        Some(self.one(
            collection,
            read_metadata(layout)?,
            vec![bytes],
            Some(requested),
        ))
    }

    /// A hit carrying `read` as the members of `collection`, and the one
    /// object a named read asked for, where it was one.
    fn one(
        &self,
        collection: Collection,
        metadata: String,
        read: Vec<String>,
        requested: Option<Requested>,
    ) -> Loaded {
        let mut loaded = Loaded {
            metadata,
            tables: Vec::new(),
            views: Vec::new(),
            routines: Vec::new(),
            requested,
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
        let mut buffers = Buffers::default();
        if !store_object(&layout.database(), &metadata, &mut buffers) {
            return;
        }

        // `meta.json` is not an object file, and `FR-CACHE-030` does not let
        // it be left in place: it is always written through `store`.
        match covered {
            Covered::Everything => {
                let whole = [
                    replace(
                        layout,
                        Collection::Tables,
                        &document.tables,
                        table_file,
                        &mut buffers,
                    ),
                    replace(
                        layout,
                        Collection::Views,
                        document.views.as_ref(),
                        view_file,
                        &mut buffers,
                    ),
                    replace(
                        layout,
                        Collection::Routines,
                        document.routines.as_ref(),
                        routine_file,
                        &mut buffers,
                    ),
                ];

                let _ = store(&layout.meta(), &Meta::new(whole));
            }
            Covered::One(_) => {
                for table in &document.tables {
                    one(layout, table, table_file, &mut buffers);
                }
                for view in document.views.as_ref() {
                    one(layout, view, view_file, &mut buffers);
                }
                for routine in document.routines.as_ref() {
                    one(layout, routine, routine_file, &mut buffers);
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

    /// Removes `.tpl/.cache/<name>` for a name no entry declares, where that
    /// path exists (`FR-CACHE-041`); answers [`None`] where it did not, and
    /// otherwise the name `.tpl/.cache/` records for what was removed.
    ///
    /// The caller has already established conditions 1 and 2 of that
    /// requirement; this is condition 3 and the removal. The path is examined
    /// with `symlink_metadata`, so a symbolic link at `.tpl/.cache/<name>` is
    /// removed itself and never followed, and a directory is removed with
    /// [`fs::remove_dir_all`], which removes a link beneath it rather than
    /// what it points at. Anything else there — a file, a socket — is
    /// removed as one file, because the condition is "exists, as a file of
    /// any kind".
    ///
    /// The recorded name is read before the removal, per item 2: on a
    /// filesystem that ignores case it may differ from the name given.
    ///
    /// # Errors
    ///
    /// Returns [`Error::ProjectFileUnwritable`] — `74` — where the filesystem
    /// refused to say whether the path exists, or refused the removal.
    pub(crate) fn clean_orphan(&self) -> Result<Option<Recorded>, Error> {
        let Some(layout) = self.layout.as_ref() else {
            return Ok(None);
        };
        let folder = layout.folder();
        let refused = |returned: std::io::Error| Error::ProjectFileUnwritable {
            path: folder.to_owned(),
            returned,
        };

        let found = match fs::symlink_metadata(folder) {
            Ok(found) => found,
            Err(returned) if returned.kind() == std::io::ErrorKind::NotFound => return Ok(None),
            Err(returned) => return Err(refused(returned)),
        };
        let kind = found.file_type();
        let recorded = Recorded {
            name: recorded_name(folder, &found),
        };

        let removed = if kind.is_dir() {
            fs::remove_dir_all(folder)
        } else {
            fs::remove_file(folder)
        };

        match removed {
            // Gone between the two calls: the state the caller asked for.
            Err(returned) if returned.kind() != std::io::ErrorKind::NotFound => {
                Err(refused(returned))
            }
            _ => Ok(Some(recorded)),
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

    /// The names of the objects `collection` holds, in the order of
    /// `order::sort_by_name`, and none where the store holds nothing.
    ///
    /// It is the population of the nearest-match suggestion `FR-CACHE-040`
    /// obliges for a clean that names nothing cached. The directory is listed
    /// and no file is opened.
    pub(crate) fn names(&self, collection: Collection) -> Vec<String> {
        self.layout
            .as_ref()
            .and_then(|layout| shelve(layout, collection))
            .unwrap_or_default()
            .into_iter()
            .map(|shelf| shelf.name)
            .collect()
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
                    count: count(&layout.collection(*collection)).unwrap_or(0),
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

/// The contents of the object file `file`, or [`None`] on a miss
/// (`FR-CACHE-033`).
///
/// It is [`read`] with the guard `FR-CACHE-033` adds for an object file, which
/// is the guard [`holds`] applies on the write: a file that is not a regular
/// file — a symbolic link above all, whatever it points at — is a miss, and is
/// never read through. The file opened must be the one inspected, by device
/// and inode, so a regular file replaced by a link between the inspection and
/// the open is not read either: the open would follow the link, and what it
/// opened is not what was inspected.
///
/// *Rejected: `O_NOFOLLOW` on the open.* It is one system call rather than
/// two, but the flag's value differs between the supported targets and the
/// crate carries it only through a further feature of `rustix`; and an open
/// that follows no link still opens a FIFO, which blocks, where the inspection
/// first refuses it. The inspection costs one `lstat` per object file read.
fn read_object(file: &Path) -> Option<String> {
    use std::io::Read as _;
    use std::os::unix::fs::MetadataExt as _;

    let inspected = fs::symlink_metadata(file).ok()?;
    if !inspected.file_type().is_file() {
        return None;
    }

    let mut handle = fs::File::open(file).ok()?;
    let opened = handle.metadata().ok()?;
    if opened.dev() != inspected.dev() || opened.ino() != inspected.ino() {
        return None;
    }

    let mut text = String::with_capacity(usize::try_from(opened.len()).unwrap_or(0));
    handle.read_to_string(&mut text).ok()?;

    Some(text)
}

/// The contents of every object file of `directory`, or [`None`] where the
/// directory could not be walked.
///
/// A file that is not an object — the temporary of a write in flight, anything
/// a hand left there — is skipped, per [`paths::is_object`]. A file that is an
/// object and cannot be read, or is a symbolic link, makes the whole collection
/// a miss: serving the rest would be a listing short one member, which
/// `BR-CDOC-002` refuses.
fn members(directory: &Path) -> Option<Vec<String>> {
    let mut held = Vec::new();

    for entry in fs::read_dir(directory).ok()? {
        let path = entry.ok()?.path();

        if paths::is_object(&path) {
            held.push(read_object(&path)?);
        }
    }

    Some(held)
}

/// The members of one collection, named and ordered by their paths, or
/// [`None`] where the directory could not be walked or a listed object file
/// holds no member this layout can name.
///
/// The walk is [`members`]'s, admitted by the same [`paths::is_object`], and
/// the order is [`decode`]'s: a stable sort by the one comparator, over the
/// walk's own order, so two members of one name keep the order an up-front
/// read would have kept them in.
fn shelve(layout: &Layout, collection: Collection) -> Option<Vec<Shelf>> {
    let mut held = Vec::new();

    for entry in fs::read_dir(layout.collection(collection)).ok()? {
        let file = entry.ok()?.path();

        if paths::is_object(&file) {
            let (kind, name) = paths::member_of(collection, &file)?;
            let name = name.to_owned();

            held.push(Shelf { name, kind, file });
        }
    }

    order::sort_by_name(&mut held);

    Some(held)
}

/// How many object files `directory` holds, or [`None`] where the directory
/// could not be walked.
///
/// [`Cache::summary`] takes the size of each collection from it as well.
///
/// It is what `tpl cache status` reports (`FR-CACHE-025`, `FR-CACHE-034`):
/// the count of objects **held**, which the directory listing answers without
/// opening a file. The files it counts are the ones [`members`] reads — the
/// same walk, admitted by the same [`paths::is_object`]. No requirement makes
/// the count a validation, so a file is counted whatever its contents: one
/// whose JSON would not decode was counted when the contents were read, and
/// still is. The one file counted now and not before is one that cannot be
/// opened or is not UTF-8, which used to turn the whole collection's count to
/// `0` — a count of nothing beside a collection recorded whole.
///
/// *Rejected: reading every file to count it.* It read 3.2 MB for `WL-001` and
/// was 93.3% of the command's samples (`BENCHMARKS.md`, 2026-09-22), for bytes
/// that were dropped unread.
fn count(directory: &Path) -> Option<usize> {
    let mut held = 0;

    for entry in fs::read_dir(directory).ok()? {
        if paths::is_object(&entry.ok()?.path()) {
            held += 1;
        }
    }

    Some(held)
}

/// Writes `value` to `file`, through a temporary file in the same directory
/// renamed over the target (`FR-CACHE-030`), whatever the target holds.
///
/// It is the write of `meta.json`, which is not an object file and which
/// `FR-CACHE-030` does not let be left in place. Every object file goes through
/// [`store_object`], which may leave an identical file where it is.
///
/// Answers whether the file is now stored. Every failure answers `false` and
/// reports nothing, per `FR-CACHE-036`.
fn store<T: Serialize>(file: &Path, value: &T) -> bool {
    write_through(file, |handle| serialise(handle, value))
}

/// The two buffers a write of the cache reuses from one object to the next.
///
/// They grow to the largest object written and are then reused, so a whole
/// write allocates for its largest object rather than once per object.
#[derive(Debug, Default)]
struct Buffers {
    /// The bytes the write would produce.
    encoded: Vec<u8>,
    /// The bytes the target already holds, read only where the lengths match.
    held: Vec<u8>,
}

/// Stores one cached object in `file`: leaves the file in place where it
/// already holds exactly the bytes the write would produce, and otherwise
/// writes it through [`write_through`] (`FR-CACHE-030`).
///
/// Both paths leave a whole file, so `FR-CACHE-031` holds on either: a file left
/// in place was whole, and a file renamed over is whole. A target that cannot
/// be read, or that differs in any byte, is written, which is also what makes a
/// corrupted file a miss that is rewritten, per `FR-CACHE-033`.
///
/// Answers whether the object is now stored. Every failure answers `false` and
/// reports nothing, per `FR-CACHE-036`.
fn store_object<T: Serialize>(file: &Path, value: &T, buffers: &mut Buffers) -> bool {
    buffers.encoded.clear();
    if encode(&mut buffers.encoded, value).is_err() {
        return false;
    }

    // PERF: rewriting an object whose file already holds the same bytes cost
    // 114 µs per file in the open of the temporary, its write and close, and
    // the rename, and was half of a `WL-001` `cache load` (`BENCHMARKS.md`,
    // 2026-09-23, `#243` row 2). The comparison costs a `lstat` where the
    // lengths differ, and a read where they match.
    if holds(file, &buffers.encoded, &mut buffers.held) {
        return true;
    }

    let encoded = &buffers.encoded;
    write_through(file, |mut handle| handle.write_all(encoded))
}

/// Whether `file` is a regular file of this cache's [`MODE`] that holds exactly
/// `encoded`, read into `held`.
///
/// Anything short of certainty answers `false`, and the caller then writes the
/// file: a target that is absent, cannot be read, is not a regular file, is a
/// symbolic link, carries another mode, or differs in length or in any byte. The
/// mode is compared because a rename would have replaced the file with one of
/// [`MODE`], and a file left in place must be no wider than one written. The
/// file opened must be the one inspected, by device and inode, and one byte
/// past the expected length is asked for, so a file replaced or grown between
/// the inspection and the read is not taken for identical.
fn holds(file: &Path, encoded: &[u8], held: &mut Vec<u8>) -> bool {
    use std::io::Read as _;
    use std::os::unix::fs::{MetadataExt as _, PermissionsExt as _};

    let Ok(inspected) = fs::symlink_metadata(file) else {
        return false;
    };
    let Ok(length) = u64::try_from(encoded.len()) else {
        return false;
    };
    if !inspected.file_type().is_file()
        || inspected.permissions().mode() & 0o7777 != MODE
        || inspected.len() != length
    {
        return false;
    }

    let Ok(handle) = fs::File::open(file) else {
        return false;
    };
    match handle.metadata() {
        Ok(opened) if opened.dev() == inspected.dev() && opened.ino() == inspected.ino() => {}
        _ => return false,
    }

    held.clear();
    held.reserve(encoded.len().saturating_add(1));

    handle
        .take(length.saturating_add(1))
        .read_to_end(held)
        .is_ok()
        && held.as_slice() == encoded
}

/// Writes a file through a temporary file in the same directory renamed over
/// `file` (`FR-CACHE-030`), with `fill` writing the bytes into the temporary.
///
/// The rename is what makes a concurrent reader see one whole version of the
/// file or the other, per `FR-CACHE-031`, and it is why no lock is taken. The
/// temporary carries the process id in its name, so two processes writing the
/// same object write two temporaries and race only on the rename — which the
/// operating system makes atomic.
///
/// **The file is not synced to disk before the rename, and nothing requires
/// it.** No requirement asks the cache for durability across a crash or a
/// power loss: `FR-CACHE-030` and `FR-CACHE-031` ask for atomicity against
/// concurrent writers, which the rename gives, and a file that a crash leaves
/// empty or torn does not decode — which `FR-CACHE-033` already makes a miss,
/// read from the server and rewritten, with nothing reported. Every read of
/// the store answers [`None`] for such a file: [`Cache::meta`],
/// [`Loaded::document`] and [`Cache::database`] all decode what they read, and
/// an empty or truncated JSON text does not decode.
///
/// Answers whether the file is now stored. Every failure answers `false`,
/// leaves the target untouched and reports nothing, per `FR-CACHE-036`.
fn write_through<F>(file: &Path, fill: F) -> bool
where
    F: FnOnce(fs::File) -> std::io::Result<()>,
{
    let Some(directory) = file.parent() else {
        return false;
    };
    let temporary = directory.join(format!("{TEMPORARY}.{}.tmp", std::process::id()));

    let written = || -> std::io::Result<()> {
        let handle = fs::OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(true)
            .mode(MODE)
            .open(&temporary)?;

        // The handle is consumed, so the file is closed before the rename.
        fill(handle)?;

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

/// Appends `value` to `sink` as one line of compact JSON: the bytes
/// [`serialise`] writes, into memory.
///
/// # Errors
///
/// Returns the error the serialisation met.
fn encode<T: Serialize>(sink: &mut Vec<u8>, value: &T) -> serde_json::Result<()> {
    serde_json::to_writer(&mut *sink, value)?;
    sink.push(b'\n');
    Ok(())
}

/// Writes `value` to `sink` as one line of compact JSON, through a buffer, and
/// flushes it.
///
/// It is separate from [`store`] so that a sink refusing the bytes is
/// observable from a test without a filesystem that refuses them.
///
/// # Errors
///
/// Returns the first error the serialisation, the write or the flush met.
fn serialise<W: std::io::Write, T: Serialize>(sink: W, value: &T) -> std::io::Result<()> {
    // PERF: `serde_json` writes one token at a time, so an unbuffered `File`
    // costs one `write` syscall per token. Those syscalls and the per-file
    // sync `store` no longer makes were 97.4% of a `WL-001` cache write
    // (`BENCHMARKS.md`, 2026-09-22). The buffer turns a file into a handful of
    // writes; the bytes are identical.
    let mut buffered = std::io::BufWriter::new(sink);
    serde_json::to_writer(&mut buffered, value).map_err(std::io::Error::from)?;
    buffered.write_all(b"\n")?;

    // Explicit, so that a failure is reported here: the drop of a `BufWriter`
    // swallows it, and the rename that follows would then store an object
    // whose last bytes were never written.
    buffered.flush()
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
fn replace<T, F>(
    layout: &Layout,
    collection: Collection,
    members: &[T],
    file_of: F,
    buffers: &mut Buffers,
) -> bool
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
            Some(file) if store_object(&file, member, buffers) => {
                written.insert(file);
            }
            _ => complete = false,
        }
    }

    complete && prune(&directory, &written) == members.len()
}

/// Writes one named member, or leaves the cache as it was.
fn one<T, F>(layout: &Layout, member: &T, file_of: F, buffers: &mut Buffers)
where
    T: Serialize,
    F: Fn(&Layout, &T) -> Option<PathBuf>,
{
    if let Some(file) = file_of(layout, member)
        && let Some(parent) = file.parent()
        && fs::create_dir_all(parent).is_ok()
    {
        let _ = store_object(&file, member, buffers);
    }
}

#[cfg(test)]
mod tests {
    use super::{
        Buffers, Cache, Collection, Layout, Listed, MODE, Meta, Summary, TEMPORARY, count,
        serialise, store, store_object,
    };
    use crate::project::scratch::Scratch;
    use std::io::{Error, ErrorKind, Write};
    use std::path::Path;

    /// A sink that refuses every byte it is handed.
    struct Refusing;

    impl Write for Refusing {
        fn write(&mut self, _: &[u8]) -> std::io::Result<usize> {
            Err(Error::new(ErrorKind::StorageFull, "refused"))
        }

        fn flush(&mut self) -> std::io::Result<()> {
            Ok(())
        }
    }

    /// The layout of the entry `shop`, under a `.tpl` folder in `scratch`.
    fn layout(scratch: &Scratch) -> Layout {
        Layout::of(&scratch.path(".tpl"), "shop").expect("shop is a path component")
    }

    /// Whether `directory` holds a temporary file of `FR-CACHE-030`.
    fn holds_temporary(directory: &Path) -> bool {
        std::fs::read_dir(directory)
            .expect("the directory exists")
            .flatten()
            .any(|entry| entry.file_name().to_string_lossy().starts_with(TEMPORARY))
    }

    #[test]
    fn a_failure_the_buffer_meets_only_at_the_flush_is_reported() {
        // The value is far smaller than the buffer, so no byte reaches the
        // sink before the flush: the refusal surfaces there or nowhere, and a
        // flush left to the drop of the buffer would swallow it.
        let returned = serialise(Refusing, &"a value").expect_err("the sink refused the bytes");

        assert_eq!(returned.kind(), ErrorKind::StorageFull);
    }

    #[test]
    fn fr_cache_030_a_stored_object_is_one_line_of_compact_json() {
        let scratch = Scratch::new();
        let file = scratch.directory("store").join("object.json");
        let value = serde_json::json!({"name": "orders", "columns": [1, 2, 3]});

        assert!(store(&file, &value));

        let mut expected = serde_json::to_vec(&value).expect("the value serialises");
        expected.push(b'\n');
        assert_eq!(
            std::fs::read(&file).expect("the object was stored"),
            expected
        );
        assert!(!holds_temporary(&scratch.path("store")));
    }

    /// The inode of `file`: a rename over it changes it, a file left in place
    /// keeps it.
    fn inode(file: &Path) -> u64 {
        std::os::unix::fs::MetadataExt::ino(
            &std::fs::symlink_metadata(file).expect("the file exists"),
        )
    }

    #[test]
    fn fr_cache_030_an_object_is_stored_as_the_bytes_the_record_is_stored_as() {
        let scratch = Scratch::new();
        let directory = scratch.directory("store");
        let value = serde_json::json!({"name": "orders", "comment": "é \u{1f600}"});

        assert!(store(&directory.join("streamed.json"), &value));
        assert!(store_object(
            &directory.join("encoded.json"),
            &value,
            &mut Buffers::default()
        ));

        assert_eq!(
            std::fs::read(directory.join("encoded.json")).expect("stored"),
            std::fs::read(directory.join("streamed.json")).expect("stored"),
        );
    }

    #[test]
    fn fr_cache_030_an_object_whose_file_holds_the_same_bytes_is_left_in_place() {
        // Implementation, not contract: FR-CACHE-030 permits either path, and
        // this pins the one taken.
        let scratch = Scratch::new();
        let file = scratch.directory("store").join("object.json");
        let value = serde_json::json!({"name": "orders", "columns": [1, 2, 3]});
        let mut buffers = Buffers::default();

        assert!(store_object(&file, &value, &mut buffers));
        let first = inode(&file);
        let bytes = std::fs::read(&file).expect("stored");

        assert!(store_object(&file, &value, &mut buffers));

        assert_eq!(inode(&file), first, "an identical object was rewritten");
        assert_eq!(std::fs::read(&file).expect("stored"), bytes);
        assert!(!holds_temporary(&scratch.path("store")));
    }

    #[test]
    fn fr_cache_030_an_object_whose_file_differs_or_cannot_be_trusted_is_rewritten() {
        use std::os::unix::fs::PermissionsExt as _;

        let value = serde_json::json!({"name": "orders", "columns": [1, 2, 3]});
        let mut expected = serde_json::to_vec(&value).expect("the value serialises");
        expected.push(b'\n');
        let mut flipped = expected.clone();
        flipped[3] ^= 0x01;
        let mut grown = expected.clone();
        grown.push(b' ');

        // Each case leaves the target in one state the write must not trust.
        type Prepare = fn(&Path, &[u8], &[u8], &[u8]);
        let cases: [(&str, Prepare); 7] = [
            ("absent", |file, _, _, _| {
                let _ = std::fs::remove_file(file);
            }),
            ("one byte differs, same length", |file, _, flipped, _| {
                std::fs::write(file, flipped).expect("writable");
            }),
            ("one byte longer", |file, _, _, grown| {
                std::fs::write(file, grown).expect("writable");
            }),
            ("truncated", |file, expected, _, _| {
                std::fs::write(file, &expected[..expected.len() / 2]).expect("writable");
            }),
            ("the same bytes, unreadable", |file, expected, _, _| {
                std::fs::write(file, expected).expect("writable");
                std::fs::set_permissions(file, std::fs::Permissions::from_mode(0o000))
                    .expect("ours");
            }),
            ("the same bytes, a wider mode", |file, expected, _, _| {
                std::fs::write(file, expected).expect("writable");
                std::fs::set_permissions(file, std::fs::Permissions::from_mode(0o644))
                    .expect("ours");
            }),
            (
                "the same bytes, through a symbolic link",
                |file, expected, _, _| {
                    let elsewhere = file.with_extension("held");
                    std::fs::write(&elsewhere, expected).expect("writable");
                    std::fs::set_permissions(&elsewhere, std::fs::Permissions::from_mode(MODE))
                        .expect("ours");
                    let _ = std::fs::remove_file(file);
                    std::os::unix::fs::symlink(&elsewhere, file).expect("writable");
                },
            ),
        ];

        for (case, prepare) in cases {
            let scratch = Scratch::new();
            let file = scratch.directory("store").join("object.json");
            let mut buffers = Buffers::default();
            assert!(store_object(&file, &value, &mut buffers));
            prepare(&file, &expected, &flipped, &grown);
            let before = std::fs::symlink_metadata(&file)
                .ok()
                .map(|found| std::os::unix::fs::MetadataExt::ino(&found));

            assert!(store_object(&file, &value, &mut buffers), "{case}");

            let written = std::fs::symlink_metadata(&file).expect("stored");
            assert!(written.file_type().is_file(), "{case}: not a regular file");
            assert_eq!(written.permissions().mode() & 0o7777, MODE, "{case}");
            assert_ne!(Some(inode(&file)), before, "{case}: not renamed over");
            assert_eq!(std::fs::read(&file).expect("stored"), expected, "{case}");
            assert!(!holds_temporary(&scratch.path("store")), "{case}");
        }
    }

    #[test]
    fn fr_cache_036_a_failed_rename_stores_nothing_and_leaves_no_temporary() {
        // A non-empty directory at the target makes the rename fail after the
        // temporary was written and flushed.
        let scratch = Scratch::new();
        let target = scratch.directory("store/object.json");
        scratch.file("store/object.json/held", "untouched");

        assert!(!store(&target, &"a value"));

        assert!(target.is_dir());
        assert_eq!(
            std::fs::read_to_string(target.join("held")).expect("the target is untouched"),
            "untouched"
        );
        assert!(!holds_temporary(&scratch.path("store")));
    }

    #[test]
    fn fr_cache_034_a_count_is_of_the_object_files_held_whatever_they_contain() {
        let scratch = Scratch::new();
        let layout = layout(&scratch);
        let tables = layout.collection(Collection::Tables);
        std::fs::create_dir_all(&tables).expect("the scratch directory is writable");

        std::fs::write(tables.join("orders.json"), "{\"name\":\"orders\"}\n")
            .expect("the scratch directory is writable");
        // A corrupted object, one that is not even UTF-8, the in-flight
        // temporary of a write, and a file that is not an object at all.
        std::fs::write(tables.join("lines.json"), "{\"name\":\"li").expect("writable");
        std::fs::write(tables.join("rates.json"), [0xff, 0xfe]).expect("writable");
        std::fs::write(tables.join(format!("{TEMPORARY}.1.tmp")), "{").expect("writable");
        std::fs::write(tables.join("notes.txt"), "by hand").expect("writable");

        assert_eq!(count(&tables), Some(3));
        assert_eq!(count(&layout.collection(Collection::Views)), None);
    }

    #[test]
    fn fr_cache_034_status_counts_a_corrupted_object_beside_the_record() {
        let scratch = Scratch::new();
        let layout = layout(&scratch);
        let tables = layout.collection(Collection::Tables);
        std::fs::create_dir_all(&tables).expect("the scratch directory is writable");
        assert!(store(&layout.meta(), &Meta::new([true, false, false])));
        std::fs::write(tables.join("orders.json"), "{}\n").expect("writable");
        std::fs::write(tables.join("lines.json"), "").expect("writable");

        let status = Cache::of(&scratch.path(".tpl"), "shop").status();

        assert!(status.loaded_at.is_some());
        let counted: Vec<(&str, usize, bool)> = status
            .collections
            .iter()
            .map(|held| (held.name, held.count, held.whole))
            .collect();
        assert_eq!(
            counted,
            [
                ("tables", 2, true),
                ("views", 0, false),
                ("routines", 0, false)
            ]
        );
    }

    /// A store holding the whole fixture, as a server read writes it.
    fn stored(scratch: &Scratch) -> Cache {
        let model = crate::model::document::fixture::database();
        let document = crate::model::document::context(&model).expect("the fixture builds");
        let cache = Cache::of(&scratch.path(".tpl"), "shop");
        cache.write(&document, super::Covered::Everything);
        cache
    }

    #[test]
    fn fr_sch_026_a_listed_row_is_the_four_members_of_the_table_decoded_in_full() {
        let scratch = Scratch::new();
        let cache = stored(&scratch);
        let loaded = cache
            .collection(Collection::Tables)
            .expect("the tables are recorded whole");

        let listed = loaded.listing().expect("every table file decodes");
        let document = loaded.document().expect("every table file decodes");
        let full: Vec<Listed<'_>> = document.tables.iter().map(Listed::of).collect();

        assert!(!listed.is_empty());
        assert_eq!(listed, full);
    }

    #[test]
    fn fr_sch_026_a_table_file_that_is_not_json_is_a_miss_for_the_listing_too() {
        let scratch = Scratch::new();
        let cache = stored(&scratch);
        let tables = layout(&scratch).collection(Collection::Tables);
        let file = std::fs::read_dir(&tables)
            .expect("the tables are stored")
            .flatten()
            .map(|entry| entry.path())
            .find(|path| super::paths::is_object(path))
            .expect("one table is stored");
        let whole = std::fs::read(&file).expect("the table was stored");
        std::fs::write(&file, &whole[..whole.len() / 2]).expect("writable");

        let loaded = cache
            .collection(Collection::Tables)
            .expect("the files are readable");
        assert!(loaded.listing().is_none());
        assert!(loaded.document().is_none());
    }

    #[test]
    fn fr_sch_031_a_summary_is_the_metadata_and_the_sizes_of_the_whole_document() {
        // The fixture's restricted view and routine are not stored, so their
        // collections are not recorded whole; the record is marked whole so
        // that both lookups compared here are hits over the same files.
        let scratch = Scratch::new();
        let cache = stored(&scratch);
        assert!(store(
            &layout(&scratch).meta(),
            &Meta::new([true, true, true])
        ));

        let held = cache.summary().expect("every collection is recorded whole");
        let summary = held.summary().expect("the metadata decodes");
        let loaded = cache
            .everything()
            .expect("every collection is recorded whole");
        let document = loaded.document().expect("every file decodes");

        assert_eq!(summary, Summary::of(&document));
    }

    #[test]
    fn fr_sch_031_a_summary_is_a_miss_where_reading_everything_is_one_for_want_of_a_record() {
        let scratch = Scratch::new();
        let cache = stored(&scratch);
        assert!(store(
            &layout(&scratch).meta(),
            &Meta::new([true, false, true])
        ));

        assert!(cache.summary().is_none());
        assert!(cache.everything().is_none());

        std::fs::remove_file(layout(&scratch).database()).expect("the metadata was stored");
        assert!(store(
            &layout(&scratch).meta(),
            &Meta::new([true, true, true])
        ));
        assert!(cache.summary().is_none());
        assert!(cache.everything().is_none());
    }

    #[test]
    fn fr_cache_033_an_empty_or_truncated_record_is_an_empty_cache() {
        // A crash after the rename and before the data reached the disk can
        // leave `meta.json` empty or torn, and no sync guards against it: the
        // record must then read as absent, which is the miss of FR-CACHE-033.
        let scratch = Scratch::new();
        let layout = layout(&scratch);
        std::fs::create_dir_all(layout.folder()).expect("the scratch directory is writable");
        assert!(store(&layout.meta(), &Meta::new([true, true, true])));
        let whole = std::fs::read(layout.meta()).expect("the record was stored");

        for torn in [
            &whole[..0],
            &whole[..whole.len() / 2],
            &whole[..whole.len() - 2],
        ] {
            std::fs::write(layout.meta(), torn).expect("writable");

            let cache = Cache::of(&scratch.path(".tpl"), "shop");
            let status = cache.status();

            assert_eq!(status.loaded_at, None);
            assert!(status.collections.is_empty());
            assert!(cache.everything().is_none());
        }
    }

    /// A store of the entry `shop` whose three collections are recorded whole.
    fn whole(scratch: &Scratch) -> Cache {
        let model = crate::model::document::fixture::whole();
        let document = crate::model::document::context(&model).expect("the fixture builds");
        let cache = Cache::of(&scratch.path(".tpl"), "shop");
        cache.write(&document, super::Covered::Everything);
        cache
    }

    /// Copies the object file `from` over the path `to`, as a filesystem that
    /// folds two names onto one file would present it.
    fn copied(from: &Path, to: &Path) {
        std::fs::copy(from, to).expect("the store is writable");
    }

    #[test]
    fn fr_cache_033_a_named_table_read_whose_file_holds_another_table_is_a_miss() {
        // FR-CACHE-033, FR-CDOC-008, finding SEC-03: the file at the path of
        // the table asked for holds another table. It is not present, so the
        // read misses and goes to the server rather than answering that the
        // table does not exist.
        let scratch = Scratch::new();
        let cache = whole(&scratch);
        let layout = layout(&scratch);
        copied(
            &layout.table("carrier").expect("a component"),
            &layout.table("carrier_other").expect("a component"),
        );

        assert!(
            cache
                .table("carrier")
                .is_some_and(|loaded| loaded.document().is_some()),
            "the table the file holds is served"
        );
        assert!(
            cache
                .table("carrier_other")
                .and_then(|loaded| loaded.document().map(|_| ()))
                .is_none(),
            "a file holding another table is a miss"
        );
        // The shape the audit found: a name differing only in case. On a
        // filesystem that folds case it reaches the file of `carrier`; on one
        // that does not it reaches no file. It is a miss on both.
        assert!(
            cache
                .table("CARRIER")
                .and_then(|loaded| loaded.document().map(|_| ()))
                .is_none(),
            "a name differing in case is a miss"
        );
    }

    #[test]
    fn fr_cache_033_a_named_view_read_whose_file_holds_another_view_is_a_miss() {
        let scratch = Scratch::new();
        let cache = whole(&scratch);
        let layout = layout(&scratch);
        copied(
            &layout.view("v_carrier_directory").expect("a component"),
            &layout.view("v_other").expect("a component"),
        );

        assert!(
            cache
                .view("v_carrier_directory")
                .is_some_and(|loaded| loaded.document().is_some())
        );
        assert!(
            cache
                .view("v_other")
                .and_then(|loaded| loaded.document().map(|_| ()))
                .is_none()
        );
    }

    #[test]
    fn fr_cache_033_a_named_routine_read_whose_file_holds_another_name_or_kind_is_a_miss() {
        use crate::model::routine::RoutineKind;

        let scratch = Scratch::new();
        let cache = whole(&scratch);
        let layout = layout(&scratch);
        let name = "sp_book_consignment";
        let procedure = layout
            .routine(&RoutineKind::Procedure, name)
            .expect("a component");

        assert!(
            cache
                .routine(&RoutineKind::Function, name)
                .is_some_and(|loaded| loaded.document().is_some()),
            "the function the file holds is served"
        );

        // Another name: `procedure:sp_other` holding `sp_book_consignment`.
        copied(
            &procedure,
            &layout
                .routine(&RoutineKind::Procedure, "sp_other")
                .expect("a component"),
        );
        assert!(
            cache
                .routine(&RoutineKind::Procedure, "sp_other")
                .and_then(|loaded| loaded.document().map(|_| ()))
                .is_none(),
            "a file holding another name is a miss"
        );

        // Another kind: `function:sp_book_consignment` holding the procedure.
        copied(
            &procedure,
            &layout
                .routine(&RoutineKind::Function, name)
                .expect("a component"),
        );
        assert!(
            cache
                .routine(&RoutineKind::Function, name)
                .and_then(|loaded| loaded.document().map(|_| ()))
                .is_none(),
            "a file holding the other kind is a miss"
        );
    }

    #[test]
    fn fr_cache_033_an_object_file_that_is_a_symbolic_link_is_a_miss_for_every_read() {
        // FR-CACHE-033, H-2: whatever the link points at — here, the very
        // bytes the file held — it is not read through, by a named read, a
        // collection, the whole store, or a render reaching it.
        let scratch = Scratch::new();
        let cache = whole(&scratch);
        let layout = layout(&scratch);
        let file = layout.table("carrier").expect("a component");
        let elsewhere = scratch.path("carrier.held");
        std::fs::rename(&file, &elsewhere).expect("the store is writable");
        std::os::unix::fs::symlink(&elsewhere, &file).expect("the store is writable");

        assert!(cache.table("carrier").is_none(), "a named read");
        assert!(
            cache.collection(Collection::Tables).is_none(),
            "a collection"
        );
        assert!(cache.everything().is_none(), "the whole store");

        let shelved = cache.shelved().expect("the listing opens no object file");
        let shelf = shelved
            .tables()
            .iter()
            .find(|shelf| crate::model::document::order::Named::name(*shelf) == "carrier")
            .expect("the listing names the link");
        assert!(shelf.read().is_none(), "a render reaching it");

        // The other members of the collection are untouched by the guard.
        assert!(
            cache
                .table("consignment")
                .is_some_and(|loaded| loaded.document().is_some())
        );
    }

    #[test]
    fn fr_cache_038_the_listing_names_and_orders_what_an_up_front_read_decodes() {
        // The names, their order and the count of each collection, from the
        // paths alone, against the document the same files decode into.
        use crate::model::document::order::Named as _;

        let scratch = Scratch::new();
        let cache = whole(&scratch);
        let loaded = cache.everything().expect("the store is whole");
        let expected = loaded.document().expect("every file decodes");
        let listed = cache.shelved().expect("the store is whole");
        let names = |shelves: &[super::Shelf]| -> Vec<String> {
            shelves
                .iter()
                .map(|shelf| shelf.name().to_owned())
                .collect()
        };

        assert_eq!(
            names(listed.tables()),
            expected
                .tables
                .iter()
                .map(|table| table.name.to_string())
                .collect::<Vec<_>>()
        );
        assert_eq!(
            names(listed.views()),
            expected
                .views
                .iter()
                .map(|view| view.name.to_string())
                .collect::<Vec<_>>()
        );
        assert_eq!(
            names(listed.routines()),
            expected
                .routines
                .iter()
                .map(|routine| routine.name.to_string())
                .collect::<Vec<_>>()
        );
        for (shelf, routine) in listed.routines().iter().zip(expected.routines.iter()) {
            assert_eq!(shelf.kind(), Some(&routine.kind));
        }
        assert_eq!(
            listed.head().expect("database.json decodes").name,
            expected.name
        );
    }

    #[test]
    fn fr_cache_038_a_damaged_object_file_leaves_the_listing_a_hit() {
        // FR-CACHE-033 as amended: the listing opens no object file, so a
        // damaged one is not a miss of it. The same store read up front is.
        let scratch = Scratch::new();
        let cache = whole(&scratch);
        std::fs::write(
            layout(&scratch)
                .collection(Collection::Views)
                .join("v_carrier_directory.json"),
            "{ torn",
        )
        .expect("writable");

        assert!(cache.shelved().is_some());
        assert!(cache.everything().expect("readable").document().is_none());
    }

    #[test]
    fn fr_cache_038_the_listing_is_a_miss_where_the_record_or_a_path_is() {
        // Before the render: the record of FR-CDOC-006, `database.json`, a
        // collection directory, and a path no write composes are each a miss.
        let scratch = Scratch::new();
        let cache = whole(&scratch);
        let layout = layout(&scratch);

        std::fs::write(
            layout.collection(Collection::Routines).join("stray.json"),
            "{}",
        )
        .expect("writable");
        assert!(cache.shelved().is_none(), "a path no write composes");

        std::fs::remove_file(layout.collection(Collection::Routines).join("stray.json"))
            .expect("removable");
        assert!(cache.shelved().is_some(), "the control");

        std::fs::remove_dir_all(layout.collection(Collection::Views)).expect("removable");
        assert!(cache.shelved().is_none(), "a collection directory");

        let scratch = Scratch::new();
        let cache = whole(&scratch);
        std::fs::remove_file(
            super::paths::Layout::of(&scratch.path(".tpl"), "shop")
                .expect("a component")
                .database(),
        )
        .expect("removable");
        assert!(cache.shelved().is_none(), "database.json");

        let scratch = Scratch::new();
        let cache = whole(&scratch);
        cache
            .clean_one(Collection::Tables, cache.table_file("carrier"))
            .expect("removable");
        assert!(cache.shelved().is_none(), "a collection not recorded whole");
    }
}
