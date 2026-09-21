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
        let path = self.find(&completed, name)?;

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
        self.find(name, name)
    }

    /// The six steps of `OD-15`, over `relative`, reporting against `named`.
    fn find(&self, relative: &str, named: &str) -> Result<PathBuf, Error> {
        // 1 — FR-TMPL-004 and FR-TMPL-005: a file that is not a template is
        // not resolved, and no extension is added here.
        if !relative.ends_with(EXTENSION) {
            return Err(self.missing(named));
        }

        // 2 — FR-TMPL-023. A `relative` that is absolute, or that climbs with
        // `..`, needs no guard of its own: it survives the join and is refused
        // at step 4, which is the one check `FR-TMPL-026` states.
        let root = self.canonical_root(named)?;
        let joined = root.join(relative);

        // 3 — FR-TMPL-025.
        let resolved = std::fs::canonicalize(&joined).map_err(|returned| {
            if returned.kind() == io::ErrorKind::NotFound {
                self.missing(named)
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
                self.missing(named)
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
            return Err(self.missing(named));
        }

        // 6 — the path that was checked.
        Ok(resolved)
    }

    /// The canonical template root of step 4 (`FR-TMPL-025`).
    ///
    /// A root that does not exist is a project with no templates, so every
    /// name is a name that does not exist rather than a failure of its own.
    fn canonical_root(&self, named: &str) -> Result<PathBuf, Error> {
        std::fs::canonicalize(&self.path).map_err(|returned| {
            if returned.kind() == io::ErrorKind::NotFound {
                self.missing(named)
            } else {
                Error::ProjectFileUnreadable {
                    path: self.path.clone(),
                    returned,
                }
            }
        })
    }

    /// The `66` of `FR-TMPL-027`.
    fn missing(&self, named: &str) -> Error {
        Error::TemplateNotFound {
            name: named.to_owned(),
            root: self.path.clone(),
        }
    }

    /// The `65` of `FR-TMPL-026`, which `FR-TMPL-024` also reaches.
    fn escaped(&self, named: &str) -> Error {
        Error::TemplateOutsideRoot {
            name: named.to_owned(),
            root: self.path.clone(),
        }
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
    use super::Root;
    use crate::error::Error;
    use crate::project::scratch::Scratch;

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
}
