//! The file layout of `.tpl/.cache/` (`FR-CACHE-001`, `FR-CACHE-002`,
//! `FR-CDOC-014`).
//!
//! One folder per database entry, keyed by entry name and by nothing else, and
//! one file per cached object beneath it:
//!
//! ```text
//! .tpl/.cache/shop/
//! ├── meta.json
//! ├── database.json
//! ├── tables/orders.json
//! ├── views/v_sales.json
//! └── routines/procedure.calc_vat.json
//! ```
//!
//! `FR-CACHE-001` draws the first, third, fourth and fifth of those and
//! `FR-CDOC-014` fixes the form of the fifth exactly — `routines/<kind>.<name>.json`,
//! with `<kind>` in **lower case** and no other spelling. `database.json` is
//! this module's own: `FR-SCH-025` obliges `tpl schema info` to read through
//! the cache, `FR-CTX-036` gives the database three metadata fields and a
//! `server` object that belong to no collection, and there is nowhere else for
//! them to live. `BR-CACHE-001` places the layout outside the plumbing
//! contract and `BR-CDOC-005` keeps `meta.json` for the versions the binary
//! decides with, so the model content goes in a file of its own rather than
//! into the metadata file.
//!
//! # A name that is not a path component is not cached
//!
//! A catalogue name may carry any byte an identifier admits, `/` included, and
//! a path built from one would name a file outside the collection's directory.
//! [`component`] is the gate: a name that is not a single, ordinary path
//! component yields [`None`], the object is not written, and a later read of it
//! finds no file and is therefore a **miss**, per `FR-CACHE-033`. A collection
//! holding such an object is not recorded whole, exactly as `FR-CACHE-037`
//! requires of a collection holding a restricted one, so no listing is ever
//! served short.
//!
//! *This corpus states no rule for such a name*, and the treatment above is
//! chosen to be the one the cache's own failure model already describes: a
//! cache that cannot hold something answers from the server and says nothing,
//! per `FR-CACHE-036`. It is reported rather than assumed correct.
//!
//! *Rejected: encoding the name into a filename.* `FR-CDOC-014` fixes
//! `routines/<kind>.<name>.json` with `<name>` the routine's name, and an
//! encoding makes the path form something other than what that requirement
//! writes. *Also rejected: refusing the whole read.* The read succeeded; only
//! an optimisation did not.

use std::path::{Path, PathBuf};

use crate::model::routine::RoutineKind;

/// The cache root inside `.tpl/`, per `FR-CACHE-001`.
const CACHE: &str = ".cache";

/// The file that carries the two versions, the load time, and the
/// per-collection completeness record (`FR-CDOC-001`, `FR-CDOC-006`,
/// `FR-CDOC-013`).
pub(super) const META: &str = "meta.json";

/// The file that carries the database's own metadata and the `server` object
/// (`FR-CTX-031`, `FR-CTX-036`).
pub(super) const DATABASE: &str = "database.json";

/// The extension every cached object carries.
const EXTENSION: &str = "json";

/// The suffix [`Layout::object`] appends, extension included.
const DOT_JSON: &str = ".json";

/// The three collections the cache holds, in the order `FR-CTX-036` writes
/// them.
///
/// The names are the plural ones `FR-OUT-030` requires a listing's one key to
/// carry, and they are the directory names of `FR-CACHE-001`. One set of three
/// strings serves the directory, the JSON key and the completeness record, so
/// the three cannot drift apart.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Collection {
    /// The tables of `FR-CAT-001`.
    Tables,
    /// The views of `FR-CAT-007`.
    Views,
    /// The routines of `FR-CAT-008`.
    Routines,
}

impl Collection {
    /// All three, in the order `FR-CTX-036` writes them.
    pub(crate) const ALL: [Self; 3] = [Self::Tables, Self::Views, Self::Routines];

    /// The plural name of `FR-OUT-030`, which is also the directory name.
    pub(crate) const fn name(self) -> &'static str {
        match self {
            Self::Tables => "tables",
            Self::Views => "views",
            Self::Routines => "routines",
        }
    }
}

/// Where one database entry's cache lives (`FR-CACHE-001`, `FR-CACHE-002`).
///
/// The entry name is the whole key. It is a path component like any other, so
/// an entry whose name is not one has no folder and is never cached — the same
/// gate [`component`] applies to an object name, applied one level up.
#[derive(Debug, Clone)]
pub(crate) struct Layout {
    /// `.tpl/.cache/<entry>`.
    entry: PathBuf,
}

impl Layout {
    /// The layout of `entry` under the `.tpl` folder at `tpl`, or [`None`]
    /// where the entry name is not a path component.
    pub(crate) fn of(tpl: &Path, entry: &str) -> Option<Self> {
        Some(Self {
            entry: tpl.join(CACHE).join(component(entry)?),
        })
    }

    /// The entry's own folder.
    pub(super) fn folder(&self) -> &Path {
        &self.entry
    }

    /// `meta.json`.
    pub(super) fn meta(&self) -> PathBuf {
        self.entry.join(META)
    }

    /// `database.json`.
    pub(super) fn database(&self) -> PathBuf {
        self.entry.join(DATABASE)
    }

    /// One collection's directory.
    pub(super) fn collection(&self, collection: Collection) -> PathBuf {
        self.entry.join(collection.name())
    }

    /// The file one table is held in, or [`None`] where its name is not a path
    /// component.
    pub(super) fn table(&self, name: &str) -> Option<PathBuf> {
        self.object(Collection::Tables, name)
    }

    /// The file one view is held in.
    pub(super) fn view(&self, name: &str) -> Option<PathBuf> {
        self.object(Collection::Views, name)
    }

    /// The file one routine is held in (`FR-CDOC-014`).
    ///
    /// The path carries the kind as well as the name, because procedures and
    /// functions occupy distinct namespaces on the server and a path keyed on
    /// the name alone would collide. `<kind>` is written in **lower case**, per
    /// that requirement as the twenty-sixth edition amended it, which is the
    /// spelling `FR-SCH-008` fixes for the qualified prefix on the command
    /// line — and not the upper-case string `FR-CAT-016` fixes for the emitted
    /// `kind`, which is the catalogue's own and belongs in a document rather
    /// than in a path this system composes.
    pub(super) fn routine(&self, kind: &RoutineKind<'_>, name: &str) -> Option<PathBuf> {
        let name = component(name)?;
        let kind = lower(kind)?;

        Some(
            self.collection(Collection::Routines)
                .join(format!("{kind}.{name}{DOT_JSON}")),
        )
    }

    /// The file one member of `collection` is held in.
    fn object(&self, collection: Collection, name: &str) -> Option<PathBuf> {
        let name = component(name)?;

        Some(
            self.collection(collection)
                .join(format!("{name}{DOT_JSON}")),
        )
    }
}

/// The lower-case spelling of a routine kind, per `FR-CDOC-014` and
/// `FR-SCH-008`, or [`None`] where the kind has none.
///
/// It is written out rather than derived from
/// [`RoutineKind::name`](crate::model::routine::RoutineKind::name) by folding
/// case, so that the two spellings this project uses are each stated where they
/// are fixed: the catalogue's own string on the model, and this one on the
/// path and on the command line.
///
/// [`None`] is `FR-CAT-055`'s own consequence, and this is one of the two
/// callers that requirement names. `FR-CDOC-014` builds a path from the two
/// kinds `FR-SCH-008` admits and no third, so a routine whose kind is outside
/// them has no file of its own: it is carried in a document and is not
/// reachable by a qualified name. The caller answers a miss, which is what an
/// absent file already means everywhere else in this module.
pub(crate) fn lower(kind: &RoutineKind<'_>) -> Option<&'static str> {
    match kind {
        RoutineKind::Procedure => Some("procedure"),
        RoutineKind::Function => Some("function"),
        RoutineKind::Unrecorded(_) => None,
    }
}

/// `name` as a single ordinary path component, or [`None`] where it is not one.
///
/// Four things disqualify a name, and between them they leave nothing that
/// escapes the directory it is joined to or collides with this module's own
/// files: the empty string, a name carrying the path separator or a NUL, the
/// two directory entries `.` and `..`, and a name beginning with `.` — which
/// would otherwise be able to spell `meta.json` and the temporary files of
/// `FR-CACHE-030`.
fn component(name: &str) -> Option<&str> {
    let ordinary = !name.is_empty()
        && !name.starts_with('.')
        && !name.contains(std::path::MAIN_SEPARATOR)
        && !name.contains('/')
        && !name.contains('\0');

    ordinary.then_some(name)
}

/// The name, and for a routine the kind, that the object file at `path` holds
/// under this layout, or [`None`] where no object of `collection` is stored
/// there.
///
/// It is the inverse of [`Layout::table`], [`Layout::view`] and
/// [`Layout::routine`]: every path those three compose answers the name and the
/// kind it was composed from, and a path none of them can compose answers
/// [`None`] — a file name that is not UTF-8, a name [`component`] refuses, or a
/// routine file whose first segment is neither of the two kinds of
/// `FR-CDOC-014`. It is what lets a render learn the members of a collection
/// from the directory listing alone, per `FR-CACHE-038`.
pub(super) fn member_of(
    collection: Collection,
    path: &Path,
) -> Option<(Option<RoutineKind<'static>>, &str)> {
    let stem = path.file_name()?.to_str()?.strip_suffix(DOT_JSON)?;

    match collection {
        Collection::Tables | Collection::Views => Some((None, component(stem)?)),
        Collection::Routines => {
            let (spelled, name) = stem.split_once('.')?;
            // The spelling is read back through `lower`, the one place it is
            // written, so the two directions cannot drift apart.
            let kind = [RoutineKind::Procedure, RoutineKind::Function]
                .into_iter()
                .find(|kind| lower(kind) == Some(spelled))?;

            Some((Some(kind), component(name)?))
        }
    }
}

/// Whether `path` is one of this layout's object files.
///
/// It is what a directory walk keeps: a regular file whose name ends in the
/// extension every cached object carries, so the temporary file of a write
/// that is in flight — which carries a name beginning with `.` and no such
/// extension — is not read as an object.
pub(super) fn is_object(path: &Path) -> bool {
    path.extension().is_some_and(|found| found == EXTENSION)
        && path
            .file_name()
            .and_then(|name| name.to_str())
            .is_some_and(|name| !name.starts_with('.'))
}

#[cfg(test)]
mod tests {
    use super::{Collection, Layout, component, is_object, lower, member_of};
    use crate::model::routine::RoutineKind;
    use std::path::Path;

    /// The `.tpl` folder every layout below is taken under.
    fn tpl() -> &'static Path {
        Path::new("/project/.tpl")
    }

    #[test]
    fn fr_cache_001_the_store_lives_under_the_cache_folder_of_the_tpl_directory() {
        // FR-CACHE-001: `.tpl/.cache/`, one folder per database entry.
        let layout = Layout::of(tpl(), "shop").expect("shop is a path component");

        assert_eq!(layout.folder(), Path::new("/project/.tpl/.cache/shop"));
        assert_eq!(
            layout.meta(),
            Path::new("/project/.tpl/.cache/shop/meta.json")
        );
        assert_eq!(
            layout.collection(Collection::Tables),
            Path::new("/project/.tpl/.cache/shop/tables")
        );
    }

    #[test]
    fn fr_cache_002_the_key_is_the_entry_name_and_nothing_else() {
        // FR-CACHE-002: two entries pointing at the same server have two
        // folders, and one entry repointed keeps its own — which is what
        // FR-CACHE-029 then makes a consequence rather than an accident.
        let first = Layout::of(tpl(), "shop").expect("a component");
        let second = Layout::of(tpl(), "staging").expect("a component");

        assert_ne!(first.folder(), second.folder());
        assert!(first.folder().ends_with("shop"));
        assert!(second.folder().ends_with("staging"));
    }

    #[test]
    fn fr_cdoc_014_a_cached_routine_carries_its_kind_in_lower_case() {
        // FR-CDOC-014 as amended in the twenty-sixth edition: `<kind>` is
        // `procedure` or `function`, and no other spelling of either.
        let layout = Layout::of(tpl(), "shop").expect("a component");

        assert_eq!(
            layout.routine(&RoutineKind::Procedure, "calc_vat"),
            Some("/project/.tpl/.cache/shop/routines/procedure.calc_vat.json".into())
        );
        assert_eq!(
            layout.routine(&RoutineKind::Function, "calc_vat"),
            Some("/project/.tpl/.cache/shop/routines/function.calc_vat.json".into())
        );
    }

    #[test]
    fn fr_cdoc_014_the_two_kinds_of_one_name_are_two_files() {
        // The whole reason the path carries the kind: one name can denote two
        // objects, and a path keyed on the name alone would collide.
        let layout = Layout::of(tpl(), "shop").expect("a component");

        assert_ne!(
            layout.routine(&RoutineKind::Procedure, "calc_vat"),
            layout.routine(&RoutineKind::Function, "calc_vat")
        );
    }

    #[test]
    fn fr_sch_008_the_path_kind_is_the_invocation_prefix_and_not_the_emitted_field() {
        // FR-SCH-008 fixes the prefix in lower case and FR-CAT-016 the emitted
        // `kind` in upper case. The two layers carry the same two kinds under
        // the two spellings each requirement fixes.
        for kind in [RoutineKind::Procedure, RoutineKind::Function] {
            assert_eq!(lower(&kind), Some(kind.name().to_lowercase().as_str()));
            assert_ne!(lower(&kind), Some(kind.name()));
        }
    }

    #[test]
    fn fr_cat_055_a_kind_outside_the_recorded_two_has_no_file_of_its_own() {
        // FR-CAT-055 names this as one of the two places a third kind is not
        // reachable: FR-CDOC-014 builds a path from the two kinds FR-SCH-008
        // admits and no third, so such a routine is carried in a document and
        // has no file. The caller reads the `None` as the miss an absent file
        // already is everywhere else here.
        let layout = Layout::of(tpl(), "shop").expect("a component");
        let package = RoutineKind::Unrecorded(std::borrow::Cow::Borrowed("PACKAGE"));

        assert_eq!(lower(&package), None);
        assert_eq!(layout.routine(&package, "calc_vat"), None);
    }

    #[test]
    fn a_name_that_is_not_a_path_component_yields_no_file() {
        // A catalogue name may carry any byte an identifier admits. None of
        // these is written, and a read of one is therefore a miss.
        let layout = Layout::of(tpl(), "shop").expect("a component");

        for refused in ["", ".", "..", "a/b", ".hidden", "a\0b"] {
            assert_eq!(layout.table(refused), None, "{refused:?}");
            assert_eq!(layout.view(refused), None, "{refused:?}");
            assert_eq!(
                layout.routine(&RoutineKind::Procedure, refused),
                None,
                "{refused:?}"
            );
        }

        assert_eq!(Layout::of(tpl(), "a/b").map(|_| ()), None);
        assert_eq!(component("orders"), Some("orders"));
    }

    #[test]
    fn a_directory_walk_keeps_object_files_and_skips_a_write_in_flight() {
        // FR-CACHE-030 writes through a temporary file in the same directory,
        // so a concurrent listing meets one. It is not an object.
        assert!(is_object(Path::new("/c/tables/orders.json")));
        assert!(!is_object(Path::new("/c/tables/.orders.json.tmp")));
        assert!(!is_object(Path::new("/c/tables/orders")));
        assert!(!is_object(Path::new("/c/tables/.orders.json")));
    }

    #[test]
    fn fr_cache_038_a_path_answers_the_name_and_kind_it_was_composed_from() {
        // The listing of FR-CACHE-038 names each member from its path, so
        // every path the layout composes must read back as what composed it,
        // a dotted name included.
        let layout = Layout::of(tpl(), "shop").expect("a component");
        let table = layout.table("order.lines").expect("a component");
        let view = layout.view("v_sales").expect("a component");

        assert_eq!(
            member_of(Collection::Tables, &table),
            Some((None, "order.lines"))
        );
        assert_eq!(member_of(Collection::Views, &view), Some((None, "v_sales")));

        for kind in [RoutineKind::Procedure, RoutineKind::Function] {
            let routine = layout.routine(&kind, "calc.vat").expect("a component");

            assert_eq!(
                member_of(Collection::Routines, &routine),
                Some((Some(kind), "calc.vat"))
            );
        }
    }

    #[test]
    fn fr_cache_038_a_path_no_write_composes_names_no_member() {
        // A routine file without one of the two kinds of FR-CDOC-014, in
        // either case, and a name the component gate refuses.
        for (collection, path) in [
            (Collection::Routines, "/c/routines/calc_vat.json"),
            (Collection::Routines, "/c/routines/PROCEDURE.calc_vat.json"),
            (Collection::Routines, "/c/routines/package.calc_vat.json"),
            (Collection::Routines, "/c/routines/procedure..json"),
            (Collection::Tables, "/c/tables/orders"),
        ] {
            assert_eq!(member_of(collection, Path::new(path)), None, "{path}");
        }
    }
}
