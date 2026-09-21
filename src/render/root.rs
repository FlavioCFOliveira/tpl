//! The template root, and the one resolution of a template name to a path.
//!
//! `FR-TMPL-023` makes `.tpl/templates/` of the resolved project "the boundary
//! of every template lookup", which is one boundary and is therefore enforced
//! in one place. [`Root::locate`] is that place: the loader closure of
//! [`super::engine`] calls it, and so do the `template` subcommands that
//! resolve a path without building an engine. `OD-15` settles the sequence and
//! forbids wrapping the engine's own path helper; the six steps below are that
//! sequence, and no step is applied twice anywhere else in the crate.
//!
//! | Step | What it does | Forced by |
//! |---|---|---|
//! | 1 | Refuses a name whose file does not end in `.jinja` | `FR-TMPL-004`, `FR-TMPL-005` |
//! | 2 | Joins the name to the template root | `FR-TMPL-023` |
//! | 3 | Canonicalises the joined path | `FR-TMPL-025` |
//! | 4 | Re-checks the canonical path against the canonicalised root | `FR-TMPL-026` |
//! | 5 | Refuses a symbolic link by its own metadata, without following it | `FR-TMPL-024` |
//! | 6 | Returns the path that was checked, and no other | `OD-15` |
//!
//! # Two names, and why the extension is completed by only one of them
//!
//! `FR-TMPL-007` lets the **command line** name a template with or without its
//! extension; `FR-TMPL-008` makes a name written **inside** a template literal,
//! with no extension resolved for it. The two obligations meet here as two
//! entry points over one sequence: [`Root::resolve`] completes the extension
//! and then locates, and [`Root::locate`] locates a name exactly as written.
//! The loader calls the second, so the engine never sees a name `tpl`
//! completed and `BR-TMPL-003` holds by construction.
//!
//! # The population, and why it is here rather than in the command
//!
//! [`Root::templates`] enumerates the templates the project carries, and it is
//! here for the same reason the lookup is: it applies `FR-TMPL-004`,
//! `FR-TMPL-005` and `FR-TMPL-024` — what a template is, what is invisible,
//! and what a symbolic link is not — and those rules are stated once in this
//! crate. `tpl template list` and `tpl template check` read it, and so does the
//! nearest-match suggestion of `FR-TMPL-027`, which `FR-ERR-021` makes this
//! component's to select because this component owns the names.
//!
//! # Which condition each refusal is
//!
//! `FR-TMPL-027` makes a template that does not exist `66` and `FR-TMPL-026`
//! makes a resolved path outside the root `65`, so the two are separate
//! variants rather than one. A name that is not a template under
//! `FR-TMPL-004` — one whose file does not end in `.jinja`, a directory, a
//! device — is reported as the first of the two: `FR-TMPL-005` makes such an
//! entry invisible, and an entry that is invisible is one that does not exist.

use std::io;
use std::path::{Path, PathBuf};

use crate::diagnostics::suggest::{self, Population};
use crate::error::Error;

/// The extension the file of every template ends in (`FR-TMPL-004`).
pub(crate) const EXTENSION: &str = ".jinja";

/// One project's template root, and the resolution it bounds.
///
/// It is cheap to clone — one [`PathBuf`] — because the loader closure of
/// [`super::engine`] owns a copy, which is what lets the same sequence run
/// for a name the command line supplied and for a name an `{% include %}`
/// wrote.
#[derive(Debug, Clone)]
pub(crate) struct Root {
    /// `.tpl/templates/` of the resolved project (`FR-TMPL-023`).
    path: PathBuf,
}

/// One template the project carries, as a listing presents it.
///
/// It holds the name of `FR-TMPL-006` — the path relative to the template
/// root, extension included — because that one string is both forms its
/// consumers need: the engine is given the name itself, and
/// [`Template::displayed`] is a borrow of it with the extension removed. A
/// listing of a large project therefore costs one allocation per template
/// rather than two.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct Template {
    /// The name of `FR-TMPL-006`, which ends in [`EXTENSION`].
    name: String,
}

impl Template {
    /// The name of `FR-TMPL-006`, carrying the extension.
    ///
    /// This is the name the engine knows the template by, and the name an
    /// `{% include %}` must write, per `FR-TMPL-008`.
    pub(crate) fn name(&self) -> &str {
        &self.name
    }

    /// The displayed name of `FR-TMPL-011`, with the extension removed.
    ///
    /// It is what `tpl template list` prints, what `FR-TMPL-013` orders a
    /// listing by, and what `FR-TMPL-012` makes usable verbatim as the
    /// positional argument of every command that names a template.
    ///
    /// A name that somehow did not end in [`EXTENSION`] is returned whole
    /// rather than panicked over: only [`Root::templates`] builds this type
    /// and it admits nothing else, so the arm is unreachable, and a
    /// degradation is what this crate reaches for where an invariant would
    /// otherwise put a panic on a read path.
    pub(crate) fn displayed(&self) -> &str {
        self.name.strip_suffix(EXTENSION).unwrap_or(&self.name)
    }
}

/// Whether a name that reaches nothing carries the nearest-match suggestion of
/// `FR-TMPL-027`.
///
/// It is `OD-15`'s split by who asks, carried into the refusal: the command
/// line named a template and is owed the neighbours it might have meant, while
/// a name written inside a template is answered by `FR-TMPL-009` with the line
/// and the column the engine reports and never reaches a suggestion at all.
/// Withholding it there is not only correctness — it is what keeps an
/// `{% include %}` that does not resolve from walking the template root.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Suggest {
    /// The command line asked, so the population is enumerated and the nearest
    /// matches are offered.
    Offered,
    /// A template asked, so nothing is enumerated.
    Withheld,
}

/// A template name that resolved, with both forms the caller needs.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct Resolved {
    /// The name of `FR-TMPL-006` — the path relative to the template root,
    /// carrying the extension — which is the name the engine knows the
    /// template by and the name an `{% include %}` must write.
    pub(crate) name: String,

    /// The canonical path of the file (`FR-TMPL-025`), which is absolute and
    /// is what `FR-TMPL-022` prints.
    pub(crate) path: PathBuf,
}

impl Root {
    /// The template root of the project whose `.tpl` folder is `tpl_directory`.
    ///
    /// The folder is not read and is not required to exist: an invocation that
    /// resolves no template must touch the filesystem no further, per the
    /// project's lazy-initialisation rule, and a project with no template
    /// directory is a project with no templates rather than a failure.
    pub(crate) fn new(tpl_directory: &Path) -> Self {
        Self {
            path: tpl_directory.join(crate::project::init::TEMPLATES),
        }
    }

    /// The template root itself, absolute (`FR-TMPL-021`, `FR-TMPL-023`).
    pub(crate) fn path(&self) -> &Path {
        &self.path
    }

    /// Resolves a name as the **command line** wrote it (`FR-TMPL-007`).
    ///
    /// The `.jinja` extension is completed where the caller omitted it, so
    /// `rust/struct` and `rust/struct.jinja` name one template, and the
    /// completed form is the name of `FR-TMPL-006` the engine is then given.
    ///
    /// # Errors
    ///
    /// Returns what [`Root::locate`] returns, naming the template as the
    /// caller named it rather than as the completion rewrote it, which is what
    /// [`Error::TemplateNotFound`] and [`Error::TemplateOutsideRoot`] both
    /// document their name field to carry.
    pub(crate) fn resolve(&self, name: &str) -> Result<Resolved, Error> {
        let completed = complete(name);
        let path = self.find(&completed, name, Suggest::Offered)?;

        Ok(Resolved {
            name: completed.into_owned(),
            path,
        })
    }

    /// Resolves a name **literally** (`FR-TMPL-008`).
    ///
    /// This is the form the loader uses, so a name written inside a template
    /// resolves to the file it spells and to no other. `FR-TMPL-009` is the
    /// consequence: `{% include "_header" %}` does not reach `_header.jinja`.
    ///
    /// # Errors
    ///
    /// Returns [`Error::TemplateNotFound`] for a name that is not a template
    /// under `FR-TMPL-004`, for one whose file is absent, and for one whose
    /// entry is not a regular file; [`Error::TemplateOutsideRoot`] for a
    /// resolved path outside the root (`FR-TMPL-026`) and for a symbolic link
    /// (`FR-TMPL-024`); and [`Error::ProjectFileUnreadable`] where the
    /// filesystem refused the lookup for any other reason.
    pub(crate) fn locate(&self, name: &str) -> Result<PathBuf, Error> {
        self.find(name, name, Suggest::Withheld)
    }

    /// Every template of the project, in the order `FR-TMPL-013` fixes.
    ///
    /// The walk applies the same two rules step 1 applies to one name:
    /// `FR-TMPL-004` admits a regular file whose name ends in [`EXTENSION`],
    /// and `FR-TMPL-005` makes every other entry invisible — a `LICENSE`, a
    /// `.DS_Store`, a directory named as though it were a template. Partials
    /// are not among the invisible: `FR-TMPL-014` lists them beside every
    /// other template, without a flag to hide them, so nothing here looks at a
    /// leading underscore.
    ///
    /// **A symbolic link is skipped rather than followed**, per `FR-TMPL-024`
    /// — entry by entry, so a symlinked directory is not descended into
    /// either. The walk therefore never leaves the root, which is why steps 3
    /// and 4 of `OD-15` have nothing to do here: every path it builds is
    /// composed downwards from the root through real directories.
    ///
    /// **The order is the displayed name's**, ascending, compared byte by
    /// byte, which is what `FR-TMPL-013` fixes and what makes the listing
    /// independent of directory iteration order. It is applied once, here, so
    /// both representations of `tpl template list` and the checking order of
    /// `tpl template check` carry it without restating it.
    ///
    /// # Errors
    ///
    /// Returns [`Error::ProjectFileUnreadable`] where the filesystem refused a
    /// directory the walk had reached. A template root that does not exist is
    /// a project with no templates rather than a failure, on the same terms
    /// [`Root::new`] states, and is an empty listing — which `FR-TMPL-031`
    /// requires to be a success.
    pub(crate) fn templates(&self) -> Result<Vec<Template>, Error> {
        let mut found: Vec<Template> = Vec::new();
        let mut pending: Vec<(PathBuf, String)> = vec![(self.path.clone(), String::new())];

        while let Some((directory, prefix)) = pending.pop() {
            let listed = match std::fs::read_dir(&directory) {
                Ok(listed) => listed,
                // A directory that is not there is nothing to list. For the
                // root that is a project with no templates; for anything
                // beneath it, an entry that went away between the two reads.
                Err(returned) if returned.kind() == io::ErrorKind::NotFound => continue,
                Err(returned) => {
                    return Err(Error::ProjectFileUnreadable {
                        path: directory,
                        returned,
                    });
                }
            };

            for entry in listed {
                let entry = entry.map_err(|returned| Error::ProjectFileUnreadable {
                    path: directory.clone(),
                    returned,
                })?;
                let file_name = entry.file_name();

                // A name that is not valid UTF-8 is a name no caller can write
                // on the command line and no document can carry, so
                // `FR-TMPL-012` could not hold for it: it is left out of the
                // listing rather than printed in a form that would not resolve.
                let Some(name) = file_name.to_str() else {
                    continue;
                };

                let kind = entry
                    .file_type()
                    .map_err(|returned| Error::ProjectFileUnreadable {
                        path: entry.path(),
                        returned,
                    })?;

                // FR-TMPL-024: a symlinked entry is not a template and is not
                // listed. `file_type` here is the entry's own, unfollowed.
                if kind.is_symlink() {
                    continue;
                }

                if kind.is_dir() {
                    pending.push((entry.path(), joined(&prefix, name)));
                } else if kind.is_file() && name.ends_with(EXTENSION) {
                    found.push(Template {
                        name: joined(&prefix, name),
                    });
                }
            }
        }

        // FR-TMPL-013, over **bytes**, named as bytes for the reason
        // `crate::output`'s layout names them: a comparison written over
        // `&[u8]` cannot quietly become a locale-aware one. Two templates
        // cannot share a displayed name — it is the relative path with a fixed
        // suffix removed — so the comparison is already total.
        found.sort_unstable_by(|left, right| {
            left.displayed()
                .as_bytes()
                .cmp(right.displayed().as_bytes())
        });

        Ok(found)
    }

    /// The six steps of `OD-15`, over `relative`, reporting against `named`.
    fn find(&self, relative: &str, named: &str, suggesting: Suggest) -> Result<PathBuf, Error> {
        // 1 — FR-TMPL-004 and FR-TMPL-005: a file that is not a template is
        // not resolved, and no extension is added here.
        if !relative.ends_with(EXTENSION) {
            return Err(self.missing(named, suggesting));
        }

        // 2 — FR-TMPL-023. A `relative` that is absolute, or that climbs with
        // `..`, needs no guard of its own: it survives the join and is refused
        // at step 4, which is the one check `FR-TMPL-026` states.
        let root = self.canonical_root(named, suggesting)?;
        let joined = root.join(relative);

        // 3 — FR-TMPL-025.
        let resolved = std::fs::canonicalize(&joined).map_err(|returned| {
            if returned.kind() == io::ErrorKind::NotFound {
                self.missing(named, suggesting)
            } else {
                Error::ProjectFileUnreadable {
                    path: joined.clone(),
                    returned,
                }
            }
        })?;

        // 4 — FR-TMPL-026.
        if !resolved.starts_with(&root) {
            return Err(self.escaped(named));
        }

        // 5 — FR-TMPL-024. The metadata is read from the joined path and not
        // from the canonical one, and through `symlink_metadata`, which does
        // not follow the entry: a link that was followed is a link that cannot
        // be seen.
        let metadata = std::fs::symlink_metadata(&joined).map_err(|returned| {
            if returned.kind() == io::ErrorKind::NotFound {
                self.missing(named, suggesting)
            } else {
                Error::ProjectFileUnreadable {
                    path: joined.clone(),
                    returned,
                }
            }
        })?;

        if metadata.file_type().is_symlink() {
            return Err(self.escaped(named));
        }

        if !metadata.is_file() {
            return Err(self.missing(named, suggesting));
        }

        // 6 — the path that was checked.
        Ok(resolved)
    }

    /// The canonical template root of step 4 (`FR-TMPL-025`).
    ///
    /// A root that does not exist is a project with no templates, so every
    /// name is a name that does not exist rather than a failure of its own.
    fn canonical_root(&self, named: &str, suggesting: Suggest) -> Result<PathBuf, Error> {
        std::fs::canonicalize(&self.path).map_err(|returned| {
            if returned.kind() == io::ErrorKind::NotFound {
                self.missing(named, suggesting)
            } else {
                Error::ProjectFileUnreadable {
                    path: self.path.clone(),
                    returned,
                }
            }
        })
    }

    /// The `66` of `FR-TMPL-027`, with the suggestion `suggesting` admits.
    fn missing(&self, named: &str, suggesting: Suggest) -> Error {
        Error::TemplateNotFound {
            name: named.to_owned(),
            root: self.path.clone(),
            nearest: match suggesting {
                Suggest::Withheld => Vec::new(),
                Suggest::Offered => self.nearest(named),
            },
        }
    }

    /// The nearest matches among the templates that do exist (`FR-TMPL-027`,
    /// `FR-ERR-019`).
    ///
    /// The population is the **displayed** names of `FR-TMPL-011`, because
    /// those are the names `FR-TMPL-012` makes usable verbatim: a suggestion
    /// carrying the extension would offer a spelling the caller did not write
    /// and does not need. `FR-ERR-021` names templates among its eight
    /// populations and `crate::diagnostics` places each population in the
    /// component that owns it, which for this one is here; the line the caller
    /// reads is composed there, from the names the condition carries.
    ///
    /// A walk that fails leaves the population empty, which `FR-ERR-020` turns
    /// into no suggestion at all. That is deliberate: the condition being
    /// reported is the template that does not exist, and a failure to
    /// enumerate its neighbours must not replace it with a different one.
    fn nearest(&self, named: &str) -> Vec<String> {
        let population = self.templates().unwrap_or_default();
        let nearest = suggest::suggestions(
            named,
            population.iter().map(Template::displayed),
            Population::Names,
        );

        nearest.names().map(str::to_owned).collect()
    }

    /// The `65` of `FR-TMPL-026`, which `FR-TMPL-024` also reaches.
    fn escaped(&self, named: &str) -> Error {
        Error::TemplateOutsideRoot {
            name: named.to_owned(),
            root: self.path.clone(),
        }
    }
}

/// Joins a directory prefix and an entry name into the relative name of
/// `FR-TMPL-006`.
///
/// The separator is `/` rather than the platform's, because a template name is
/// a name in the document contract and on the command line rather than a path
/// the filesystem composed — `FR-TMPL-006` spells one `rust/struct.jinja`, and
/// `Path::join` is what turns it back into a path at step 2.
fn joined(prefix: &str, name: &str) -> String {
    if prefix.is_empty() {
        name.to_owned()
    } else {
        format!("{prefix}/{name}")
    }
}

/// Completes the optional extension of `FR-TMPL-007`.
///
/// A name that already carries it is borrowed rather than rebuilt, which is
/// the spelling `tpl render` and `tpl template check` are given most often.
fn complete(name: &str) -> std::borrow::Cow<'_, str> {
    if name.ends_with(EXTENSION) {
        std::borrow::Cow::Borrowed(name)
    } else {
        std::borrow::Cow::Owned(format!("{name}{EXTENSION}"))
    }
}

#[cfg(test)]
mod tests {
    use super::{Root, Template};
    use crate::error::Error;
    use crate::project::scratch::Scratch;

    /// The displayed names `root` lists, in the order it listed them.
    fn listed(root: &Root) -> Vec<String> {
        root.templates()
            .expect("the root is readable")
            .iter()
            .map(|template| template.displayed().to_owned())
            .collect()
    }

    /// A project whose `.tpl/templates/` carries `example.jinja`.
    fn project(scratch: &Scratch) -> Root {
        scratch.file("project/.tpl/templates/example.jinja", "hello\n");

        Root::new(&scratch.canonical("project/.tpl"))
    }

    #[test]
    fn fr_tmpl_007_a_name_resolves_identically_with_and_without_the_extension() {
        // FR-TMPL-007: `rust/struct` and `rust/struct.jinja` name the same
        // template, and FR-TMPL-006 makes the resolved name the one carrying
        // the extension in both cases.
        let scratch = Scratch::new();
        let root = project(&scratch);

        let bare = root.resolve("example").expect("the template exists");
        let spelt = root.resolve("example.jinja").expect("the template exists");

        assert_eq!(bare, spelt);
        assert_eq!(bare.name, "example.jinja");
        assert_eq!(
            bare.path,
            scratch.canonical("project/.tpl/templates/example.jinja")
        );
    }

    #[test]
    fn fr_tmpl_008_a_literal_name_has_no_extension_completed_for_it() {
        // FR-TMPL-008 and BR-TMPL-003: only the command line completes the
        // extension. This is the loader's entry point, and FR-TMPL-009 is what
        // it produces for `{% include "example" %}`.
        let scratch = Scratch::new();
        let root = project(&scratch);

        assert!(root.locate("example.jinja").is_ok());

        let condition = root
            .locate("example")
            .expect_err("a literal name is not completed");

        assert!(matches!(condition, Error::TemplateNotFound { .. }));
    }

    #[test]
    fn fr_tmpl_027_a_name_that_does_not_exist_is_66() {
        // FR-TMPL-027.
        let scratch = Scratch::new();
        let root = project(&scratch);

        let condition = root.resolve("absent").expect_err("no such template");

        assert!(matches!(condition, Error::TemplateNotFound { .. }));
        assert_eq!(condition.exit_code(), 66);
    }

    #[test]
    fn fr_tmpl_027_a_project_with_no_template_directory_resolves_nothing() {
        // The root itself is absent, which is a project with no templates
        // rather than a condition of its own.
        let scratch = Scratch::new();
        scratch.directory("bare/.tpl");
        let root = Root::new(&scratch.canonical("bare/.tpl"));

        let condition = root.resolve("example").expect_err("there is no root");

        assert!(matches!(condition, Error::TemplateNotFound { .. }));
        assert_eq!(condition.exit_code(), 66);
    }

    #[test]
    fn fr_tmpl_024_a_symbolic_link_under_the_root_is_refused() {
        // FR-TMPL-024, with the exploit FR-TMPL-026's rationale names:
        // `ln -s ../.cfg .tpl/templates/leak.jinja` must not read the
        // configuration. The link resolves inside nothing the caller may see,
        // and the refusal is the escape of FR-TMPL-026.
        let scratch = Scratch::new();
        let root = project(&scratch);
        let secret = scratch.file("project/.tpl/.cfg", "[core]\npassword = \"s\"\n");
        scratch.link(&secret, &scratch.path("project/.tpl/templates/leak.jinja"));

        let condition = root
            .resolve("leak")
            .expect_err("a symbolic link is not a template");

        assert!(matches!(condition, Error::TemplateOutsideRoot { .. }));
        assert_eq!(condition.exit_code(), 65);
    }

    #[test]
    fn fr_tmpl_024_a_symbolic_link_to_a_template_inside_the_root_is_refused_too() {
        // FR-TMPL-024 refuses the link rather than its target: "a symlinked
        // entry is not a template". Step 4 passes here — the canonical path is
        // inside the root — so this is the case step 5 exists for.
        let scratch = Scratch::new();
        let root = project(&scratch);
        let inside = scratch.canonical("project/.tpl/templates/example.jinja");
        scratch.link(&inside, &scratch.path("project/.tpl/templates/alias.jinja"));

        let condition = root
            .resolve("alias")
            .expect_err("a symbolic link is not a template");

        assert!(matches!(condition, Error::TemplateOutsideRoot { .. }));
    }

    #[test]
    fn fr_tmpl_026_a_resolved_path_outside_the_root_is_65() {
        // FR-TMPL-026 over a name that climbs out of the root with `..`. It is
        // step 4 that refuses it, which is why step 2 needs no guard of its own.
        // The file exists, so the refusal is the escape and not the absence.
        let scratch = Scratch::new();
        let root = project(&scratch);
        scratch.file("project/.tpl/outside.jinja", "leaked\n");

        let condition = root
            .resolve("../outside")
            .expect_err("the name climbs out of the root");

        assert!(matches!(condition, Error::TemplateOutsideRoot { .. }));
        assert_eq!(condition.exit_code(), 65);
    }

    #[test]
    fn fr_tmpl_026_an_absolute_name_is_refused_as_an_escape() {
        // A name that is absolute replaces the root on the join, so it is the
        // second half of what step 4 catches.
        let scratch = Scratch::new();
        let root = project(&scratch);
        let outside = scratch.file("outside.jinja", "leaked\n");

        let condition = root
            .locate(outside.to_str().expect("the scratch path is UTF-8"))
            .expect_err("an absolute name is outside the root");

        assert!(matches!(condition, Error::TemplateOutsideRoot { .. }));
        assert_eq!(condition.exit_code(), 65);
    }

    #[test]
    fn fr_tmpl_004_an_entry_that_is_not_a_regular_file_is_not_a_template() {
        // FR-TMPL-004 requires a regular file; FR-TMPL-005 makes everything
        // else invisible, which is `66` and not a condition of its own.
        let scratch = Scratch::new();
        let root = project(&scratch);
        scratch.directory("project/.tpl/templates/folder.jinja");

        let condition = root
            .resolve("folder")
            .expect_err("a directory is not a template");

        assert!(matches!(condition, Error::TemplateNotFound { .. }));
    }

    #[test]
    fn fr_tmpl_005_a_file_that_does_not_end_in_the_extension_is_not_resolved() {
        // FR-TMPL-005: a LICENSE beside the templates is not renderable. The
        // literal entry point is where that shows, because the command-line one
        // would complete the extension and look for `LICENSE.jinja`.
        let scratch = Scratch::new();
        let root = project(&scratch);
        scratch.file("project/.tpl/templates/LICENSE", "not a template\n");

        let condition = root.locate("LICENSE").expect_err("it is not a template");

        assert!(matches!(condition, Error::TemplateNotFound { .. }));
    }

    #[test]
    fn fr_tmpl_006_a_name_is_the_path_relative_to_the_root() {
        // FR-TMPL-006: `rust/struct.jinja` names the file of that path under
        // the root, at any depth.
        let scratch = Scratch::new();
        let root = project(&scratch);
        scratch.file("project/.tpl/templates/rust/struct.jinja", "struct\n");

        let resolved = root.resolve("rust/struct").expect("the template exists");

        assert_eq!(resolved.name, "rust/struct.jinja");
        assert_eq!(
            resolved.path,
            scratch.canonical("project/.tpl/templates/rust/struct.jinja")
        );
    }

    // ------------------------------------------------------ the listing ---

    #[test]
    fn fr_tmpl_013_the_listing_is_ordered_by_the_displayed_name_compared_byte_by_byte() {
        // FR-TMPL-013, over the worked case its own fifth-edition correction
        // records: `_` (0x5F) sorts before `s` (0x73), so `rust/_types` comes
        // before `rust/struct`. FR-TMPL-014 puts the partial in the listing at
        // all, with no flag to hide it, and FR-TMPL-011 removes the extension
        // from every name. The entries are created in an order that is not the
        // answer, so a listing that returned directory order would fail here.
        let scratch = Scratch::new();
        let root = project(&scratch);

        for relative in [
            "rust/struct.jinja",
            "docs/table.md.jinja",
            "rust/_types.jinja",
            "Docs.jinja",
        ] {
            scratch.file(&format!("project/.tpl/templates/{relative}"), "body\n");
        }

        // `Docs` before `docs` and `_types` before `struct` are the two
        // consequences FR-TMPL-013 states of comparing bytes rather than
        // collating.
        assert_eq!(
            listed(&root),
            [
                "Docs",
                "docs/table.md",
                "example",
                "rust/_types",
                "rust/struct"
            ]
        );
    }

    #[test]
    fn fr_tmpl_005_the_listing_carries_no_entry_that_is_not_a_template() {
        // FR-TMPL-004 and FR-TMPL-005: a regular file whose name ends in
        // `.jinja`, and nothing else — not a LICENSE, not a `.DS_Store`, and
        // not a directory named as though it were a template.
        let scratch = Scratch::new();
        let root = project(&scratch);

        scratch.file("project/.tpl/templates/LICENSE", "not a template\n");
        scratch.file("project/.tpl/templates/.DS_Store", "not a template\n");
        scratch.file("project/.tpl/templates/notes.md", "not a template\n");
        scratch.directory("project/.tpl/templates/folder.jinja");

        assert_eq!(listed(&root), ["example"]);
    }

    #[test]
    fn fr_tmpl_024_a_symbolic_link_is_not_listed_whichever_side_of_the_root_it_points_at() {
        // FR-TMPL-024: "a symlinked entry is not a template and is not listed".
        // Three entries, and none of them is listed: a link out of the root,
        // a link to a template inside it, and a link to a directory of
        // templates — which is not descended into, because the entry the walk
        // meets is the link.
        let scratch = Scratch::new();
        let root = project(&scratch);
        let secret = scratch.file("project/.tpl/.cfg", "[core]\n");
        let inside = scratch.canonical("project/.tpl/templates/example.jinja");
        let elsewhere = scratch.directory("elsewhere");
        scratch.file("elsewhere/outside.jinja", "leaked\n");

        scratch.link(&secret, &scratch.path("project/.tpl/templates/leak.jinja"));
        scratch.link(&inside, &scratch.path("project/.tpl/templates/alias.jinja"));
        scratch.link(&elsewhere, &scratch.path("project/.tpl/templates/linked"));

        assert_eq!(listed(&root), ["example"]);
    }

    #[test]
    fn fr_tmpl_031_a_project_with_no_template_root_lists_nothing_and_does_not_fail() {
        // FR-TMPL-031 through FR-OUT-033: empty is a state, not a failure, and
        // the root not being there at all is the first state of a project the
        // walk has to answer for.
        let scratch = Scratch::new();
        scratch.directory("bare/.tpl");
        let root = Root::new(&scratch.canonical("bare/.tpl"));

        assert_eq!(
            root.templates().expect("an absent root is empty"),
            Vec::new()
        );
    }

    #[test]
    fn fr_tmpl_012_every_listed_name_resolves_verbatim_and_carries_its_own_full_name() {
        // FR-TMPL-012: a listed name is usable as the positional argument, at
        // any depth. The full name of FR-TMPL-006 is the same string with the
        // extension, which is what the engine is given and what an
        // `{% include %}` must write, per FR-TMPL-008.
        let scratch = Scratch::new();
        let root = project(&scratch);
        scratch.file("project/.tpl/templates/rust/struct.jinja", "struct\n");

        for template in root.templates().expect("the root is readable") {
            let resolved = root
                .resolve(template.displayed())
                .expect("a listed name resolves");

            assert_eq!(resolved.name, template.name(), "{:?}", template.displayed());
            assert!(
                root.locate(template.name()).is_ok(),
                "{:?}",
                template.name()
            );
        }
    }

    #[test]
    fn a_template_carries_the_extension_in_its_name_and_not_in_its_displayed_form() {
        // The one invariant the type holds, over a name that carries the
        // extension twice: only the last one is the extension.
        let scratch = Scratch::new();
        let root = project(&scratch);
        scratch.file("project/.tpl/templates/odd.jinja.jinja", "body\n");

        let found: Vec<(String, String)> = root
            .templates()
            .expect("the root is readable")
            .iter()
            .map(|template| (template.name().to_owned(), template.displayed().to_owned()))
            .collect();

        assert!(
            found.contains(&("odd.jinja.jinja".to_owned(), "odd.jinja".to_owned())),
            "{found:?}"
        );
    }

    // --------------------------------------------------- the suggestion ---

    #[test]
    fn fr_tmpl_027_a_name_the_command_line_wrote_carries_the_nearest_matches() {
        // FR-TMPL-027 with FR-ERR-019: at most three, within a distance of two,
        // ordered by distance and then by name, drawn from the **displayed**
        // names, which are the spellings FR-TMPL-012 makes usable.
        let scratch = Scratch::new();
        let root = project(&scratch);
        scratch.file("project/.tpl/templates/exemple.jinja", "body\n");
        scratch.file("project/.tpl/templates/unrelated.jinja", "body\n");

        let Error::TemplateNotFound { nearest, .. } =
            root.resolve("exmple").expect_err("no such template")
        else {
            panic!("a name that reaches nothing is TemplateNotFound");
        };

        assert_eq!(nearest, ["example", "exemple"]);
    }

    #[test]
    fn fr_err_020_a_name_nothing_is_near_carries_no_suggestion_at_all() {
        // FR-ERR-020: the suggestion is omitted rather than weakened.
        let scratch = Scratch::new();
        let root = project(&scratch);

        let Error::TemplateNotFound { nearest, .. } = root
            .resolve("something_entirely_different")
            .expect_err("no such template")
        else {
            panic!("a name that reaches nothing is TemplateNotFound");
        };

        assert!(nearest.is_empty(), "{nearest:?}");
    }

    #[test]
    fn fr_tmpl_009_a_name_written_inside_a_template_carries_no_suggestion() {
        // OD-15's split by who asks, carried into the refusal: FR-TMPL-027
        // owes the suggestion to the command line, and FR-TMPL-009 answers an
        // `{% include %}` with the engine's own line and column instead. The
        // loader's entry point therefore enumerates nothing.
        let scratch = Scratch::new();
        let root = project(&scratch);

        let Error::TemplateNotFound { nearest, .. } =
            root.locate("exmple.jinja").expect_err("no such template")
        else {
            panic!("a name that reaches nothing is TemplateNotFound");
        };

        assert!(nearest.is_empty(), "{nearest:?}");
    }

    #[test]
    fn fr_err_020_a_project_with_no_template_root_suggests_nothing_rather_than_failing() {
        // The population cannot be enumerated, which FR-ERR-020 turns into no
        // suggestion — never into a second condition replacing the first.
        let scratch = Scratch::new();
        scratch.directory("bare/.tpl");
        let root = Root::new(&scratch.canonical("bare/.tpl"));

        let condition = root.resolve("example").expect_err("there is no root");

        assert!(matches!(
            condition,
            Error::TemplateNotFound { ref nearest, .. } if nearest.is_empty()
        ));
        assert_eq!(condition.exit_code(), 66);
    }

    #[test]
    fn the_displayed_name_of_a_template_is_its_name_without_the_extension() {
        // The type's two accessors, stated once over a value built here so
        // that the invariant is readable without a filesystem.
        let template = Template {
            name: "rust/struct.jinja".to_owned(),
        };

        assert_eq!(template.name(), "rust/struct.jinja");
        assert_eq!(template.displayed(), "rust/struct");
    }
}
