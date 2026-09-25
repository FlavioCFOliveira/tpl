//! The render environment: the engine, the loader, and the surface a template
//! is allowed to call.
//!
//! Three things live here and nowhere else. The engine is built from disk at
//! render time, never embedded at compile time, so a template changes without
//! the binary being rebuilt. One function resolves a template name to a path,
//! and it is the boundary `FR-TMPL-023` makes every lookup pass through. And
//! the filters, tests and global functions of `FR-ENV-001`'s first group are
//! registered from a single declaration, which is also what the JSON help
//! document publishes.
//!
//! | Submodule | What it owns |
//! |---|---|
//! | [`bounds`] | Render fuel, the render output limit and the render memory limit — `FR-RND-036` … `FR-RND-039`, `FR-CONF-045` |
//! | [`root`] | The template root and the six-step resolution of `OD-15` — `FR-TMPL-004` … `FR-TMPL-008`, `FR-TMPL-023` … `FR-TMPL-027` |
//! | [`engine`] | The five settings that are not the engine's defaults, and the loader — `FR-SEM-001` … `FR-SEM-004`, `FR-SEM-010`, `FR-SEM-011`, `FR-ENV-026`, `FR-ENV-027`, `OD-14` |
//! | [`surface`] | The one declaration of group 1, and the closed list of group 2 — `FR-ENV-005` … `FR-ENV-007`, `FR-ENV-014`, `FR-ENV-018` … `FR-ENV-020` |
//! | [`naming`] | The word list of `FR-ENV-030` … `FR-ENV-032` and the five filters of `FR-ENV-033` |
//! | [`code`] | The six code filters of `FR-ENV-007`, with the tables of `FR-ENV-035` … `FR-ENV-039` and `FR-ENV-044` |
//! | [`predicate`] | The seven tests of `FR-ENV-014`, and the three families of `FR-ENV-046` |
//! | [`function`] | The five global functions of `FR-ENV-020`, and the four capabilities `FR-ENV-022` … `FR-ENV-025` forbid |
//! | [`operand`] | What a name was handed, and the refusal of `FR-SEM-005` … `FR-SEM-009` when it is the wrong thing |
//! | [`lookup`] | Reaching back into the render context, for the two derived tests and the four lookups |
//! | [`fault`] | What the engine reported, read as a condition of this crate — `FR-ERR-011`, `FR-SEM-019` |
//!
//! # Nothing is built for an invocation that compiles no template
//!
//! [`Environment`] holds the engine in a cell that is filled on first use.
//! `tpl template list` and `tpl template path` resolve paths and never reach
//! it; `tpl cfg` and the eight `schema` subcommands never construct one at all.
//! That is `NFR-PERF-006` and the project's lazy-initialisation rule, and it is
//! the reason the engine is built here rather than at the entry point.
//!
//! # Each template is read and parsed once per process
//!
//! The engine caches what the loader returns, so a loop in a template does not
//! reparse it and a second `{% include %}` of one partial does not read it
//! again. The one duplicated step is deliberate: a name the command line
//! supplied is resolved before the engine is reached, so that a template that
//! does not exist is the `66` of `FR-TMPL-027` and not a render failure, and
//! the loader resolves it again when it reads it. `OD-15` fixes that split by
//! who asks, and the second resolution is one `realpath` against a read and a
//! parse.
//!
//! # Every render is bounded by a count as well as by its deadline
//!
//! `FR-SEC-025` bounds a render by render fuel and by the render output limit
//! besides its deadline, and [`Environment::render`] applies both to every
//! render it makes. Fuel is the engine's own, set once on the engine and
//! consumed per render. The output is counted by the writer the render writes
//! into, as it is produced: the writer refuses the write that would carry the
//! count past the limit, which ends the render there, so the text held is never
//! longer than the limit. Neither costs more than a counter per step or per
//! write.

mod bounds;
mod code;
mod engine;
mod fault;
mod function;
mod lookup;
mod naming;
mod operand;
mod predicate;
mod root;
mod surface;

#[cfg(test)]
mod fixture;

use std::cell::OnceCell;
use std::path::{Path, PathBuf};

use minijinja::Value;

use crate::error::Error;

pub(crate) use bounds::{RenderBounds, RenderFuel, RenderMemoryLimit, RenderOutputLimit};
pub(crate) use lookup::Query;
pub(crate) use root::Template;
pub(crate) use surface::{
    INHERITED_FILTERS, REGISTERED_FILTERS, REGISTERED_FUNCTIONS, REGISTERED_TESTS,
};

/// One project's render environment.
///
/// It is cheap to build — a path, and a cell that is empty — so a command may
/// construct one and never compile anything. The engine is built on the first
/// call that needs it and reused for every call after, which is where the
/// promise that a template is parsed once per process is kept.
#[derive(Debug)]
pub(crate) struct Environment {
    /// The template root, and the resolution it bounds.
    root: root::Root,

    /// The render fuel and the render output limit every render is held to.
    bounds: RenderBounds,

    /// The engine, built on first use.
    engine: OnceCell<minijinja::Environment<'static>>,
}

impl Environment {
    /// The environment of the project whose `.tpl` folder is `tpl_directory`.
    ///
    /// Nothing is read: the template root is composed and the engine is left
    /// unbuilt. A render it makes is held to the built-in defaults of the two
    /// render bounds, which is what a project whose `.tpl/.cfg` declares
    /// neither key resolves.
    pub(crate) fn new(tpl_directory: &Path) -> Self {
        Self::bounded(tpl_directory, RenderBounds::default())
    }

    /// The environment of the project whose `.tpl` folder is `tpl_directory`,
    /// every render of which is held to `bounds` (`FR-RND-036`, `FR-RND-037`).
    ///
    /// Nothing is read, as for [`Environment::new`].
    pub(crate) fn bounded(tpl_directory: &Path, bounds: RenderBounds) -> Self {
        Self {
            root: root::Root::new(tpl_directory),
            bounds,
            engine: OnceCell::new(),
        }
    }

    /// The render bounds every render of this environment is held to
    /// (`FR-RND-036` … `FR-RND-039`).
    pub(crate) const fn bounds(&self) -> RenderBounds {
        self.bounds
    }

    /// The template root, absolute (`FR-TMPL-021`, `FR-TMPL-023`).
    pub(crate) fn root(&self) -> &Path {
        self.root.path()
    }

    /// Every template of the project, in the order `FR-TMPL-013` fixes
    /// (`FR-TMPL-011`, `FR-TMPL-014`).
    ///
    /// No engine is built: the listing is a walk of the template root, which
    /// is what lets `tpl template list` answer without one. It is also the
    /// population `tpl template check` checks when it is given no name, per
    /// `FR-TMPL-018`, and the population the suggestion of `FR-TMPL-027` is
    /// selected from.
    ///
    /// # Errors
    ///
    /// Returns [`Error::ProjectFileUnreadable`] where the filesystem refused
    /// the walk. A project with no template directory carries no templates and
    /// is not a failure, per `FR-TMPL-031`.
    pub(crate) fn templates(&self) -> Result<Vec<Template>, Error> {
        self.root.templates()
    }

    /// The canonical path of the template `name` resolves to (`FR-TMPL-022`).
    ///
    /// The extension of `FR-TMPL-007` is completed where the caller omitted
    /// it. No engine is built, which is what lets `tpl template path` and
    /// `tpl template show` answer without one.
    ///
    /// # Errors
    ///
    /// Returns [`Error::TemplateNotFound`] for a name that resolves to nothing
    /// (`FR-TMPL-027`), [`Error::TemplateOutsideRoot`] for one that resolves
    /// outside the root or to a symbolic link (`FR-TMPL-024`, `FR-TMPL-026`),
    /// and [`Error::ProjectFileUnreadable`] where the filesystem refused the
    /// lookup.
    pub(crate) fn resolve(&self, name: &str) -> Result<PathBuf, Error> {
        Ok(self.root.resolve(name)?.path)
    }

    /// The text of the template `name` resolves to, read on a descriptor typed
    /// as a regular file (`FR-TMPL-033`, `FR-SEC-027`).
    ///
    /// # Errors
    ///
    /// Returns what [`Environment::resolve`] returns;
    /// [`Error::TemplateNotRegular`] where the entry became something other
    /// than a regular file after it was resolved; and
    /// [`Error::ProjectFileUnreadable`] where the file could not be read or is
    /// not UTF-8.
    pub(crate) fn source(&self, name: &str) -> Result<String, Error> {
        let resolved = self.root.resolve(name)?;

        self.root
            .read(&resolved.path)
            .map_err(|unread| match unread {
                crate::at::Unread::NotRegular(kind) => Error::TemplateNotRegular {
                    name: name.to_owned(),
                    file: resolved.name.clone(),
                    kind: crate::at::kind_name(kind),
                },
                crate::at::Unread::Io(returned) => Error::ProjectFileUnreadable {
                    path: resolved.path.clone(),
                    returned,
                },
                crate::at::Unread::Oversized => Error::ProjectFileUnreadable {
                    path: resolved.path.clone(),
                    returned: std::io::Error::from(std::io::ErrorKind::InvalidData),
                },
            })
    }

    /// Compiles the template `name` without evaluating anything
    /// (`FR-TMPL-017`, `FR-TMPL-020`).
    ///
    /// This is syntax analysis alone: no expression is evaluated, no filter is
    /// called and nothing is connected to, which is what makes `BR-TMPL-001`'s
    /// guarantee — `tpl template check` is safe to run against a template you
    /// have not read — a property of the operation rather than a side effect.
    ///
    /// # Errors
    ///
    /// Returns what [`Environment::resolve`] returns for a name that does not
    /// resolve, and [`Error::TemplateSyntax`] for a template the engine could
    /// not parse.
    pub(crate) fn compile(&self, name: &str) -> Result<(), Error> {
        self.template(name).map(|_| ())
    }

    /// Renders the template `name` against `context` (`FR-RND-028`), held to
    /// the render fuel and the render output limit of this environment
    /// (`FR-RND-036`, `FR-RND-037`).
    ///
    /// The text is produced into a writer that counts every byte as it is
    /// written and refuses the write that would carry the count past the
    /// limit, so the render ends there and what is held never exceeds the
    /// limit. The text is returned whole, and nothing reaches stdout until it
    /// has been, which is what `FR-RND-034` and `FR-CACHE-039` rely on; it is
    /// held for those two and not in order to be counted.
    ///
    /// # Errors
    ///
    /// Returns what [`Environment::compile`] returns;
    /// [`Error::RenderFuelExhausted`] where the render exhausted its render
    /// fuel, and [`Error::RenderOutputLimitExceeded`] where it would have
    /// produced more bytes than its render output limit, both `65`; and
    /// [`Error::RenderFailed`] for every failure
    /// `specification/render-semantics.md` defines: an undefined variable, a
    /// filter or a test handed an operand it does not accept, a template the
    /// author ended with `fail`, and an `{% include %}` that did not resolve
    /// literally.
    pub(crate) fn render(&self, name: &str, context: &Value) -> Result<String, Error> {
        let resolved = self.root.resolve(name)?;
        let template = self
            .engine()
            .get_template(&resolved.name)
            .map_err(|reported| fault::during_compile(self.root(), &resolved.name, &reported))?;

        let mut produced = bounds::Limited::new(self.bounds.output_limit);
        let rendered = template.render_captured_to(context, &mut produced);

        match rendered {
            Ok(_) => produced.into_text(),
            // The writer refused the write past the limit, and the engine
            // reports that as a write failure: the refusal is the cause.
            Err(_) if produced.exceeded() => Err(Error::RenderOutputLimitExceeded {
                limit: self.bounds.output_limit.get(),
            }),
            Err(reported) if fault::out_of_fuel(&reported) => Err(Error::RenderFuelExhausted {
                fuel: self.bounds.fuel.get(),
            }),
            Err(reported) => {
                let mut condition = fault::during_render(&resolved.name, &reported);
                if let Error::RenderFailed {
                    undefined: Some(expression),
                    reason: reason @ None,
                    ..
                } = &mut condition
                {
                    *reason = fault::unresolved(self.engine(), context, expression)
                        .map(crate::error::RenderReason::Unresolved)
                        .or_else(|| {
                            fault::template_bound(self.engine(), context, expression, &reported)
                                .or_else(|| fault::missing(context, expression))
                                .or_else(|| fault::other_object(context, expression))
                                .or_else(|| {
                                    fault::unbound(context, expression, &reported, &resolved.name)
                                })
                                .map(crate::error::RenderReason::Missing)
                        })
                        .map(Box::new);
                }
                if let Error::RenderFailed {
                    reason: reason @ None,
                    ..
                } = &mut condition
                    && let Some(name) = fault::missing_include(&reported)
                {
                    // FR-TMPL-009 resolves an include literally, and the
                    // root's own resolution completes the extension: the
                    // second finding what the first did not is exactly the
                    // name that lacks it.
                    let lacks_extension =
                        !name.ends_with(".jinja") && self.root.resolve(&name).is_ok();
                    // Z-03: the nearest templates whether or not the include
                    // wrote the extension; the name that only lacks it is
                    // itself the one suggestion.
                    let nearest = if lacks_extension {
                        vec![format!("{name}.jinja")]
                    } else {
                        self.root.nearest_included(&name)
                    };
                    // FR-TMPL-033 item 3: an entry of that name that is a
                    // FIFO, a socket or a device is named for what it is.
                    let not_regular = self.root.special_entry(&name);
                    *reason = Some(Box::new(crate::error::RenderReason::IncludeNotFound {
                        name,
                        lacks_extension,
                        nearest,
                        not_regular,
                    }));
                }
                Err(condition)
            }
        }
    }

    /// Resolves and compiles `name`, leaving the compiled template cached.
    fn template(&self, name: &str) -> Result<minijinja::Template<'_, '_>, Error> {
        let resolved = self.root.resolve(name)?;

        self.engine()
            .get_template(&resolved.name)
            .map_err(|reported| fault::during_compile(self.root(), &resolved.name, &reported))
    }

    /// The engine, built on first use.
    fn engine(&self) -> &minijinja::Environment<'static> {
        self.engine
            .get_or_init(|| engine::build(self.root.clone(), self.bounds.fuel))
    }
}

#[cfg(test)]
mod tests {
    use super::{Environment, RenderBounds, RenderFuel, RenderMemoryLimit, RenderOutputLimit};
    use crate::error::Error;
    use crate::project::scratch::Scratch;
    use minijinja::{Value, context};

    /// An environment over a project carrying `templates`.
    fn project(scratch: &Scratch, templates: &[(&str, &str)]) -> Environment {
        scratch.directory("project/.tpl/templates");

        for (name, source) in templates {
            scratch.file(&format!("project/.tpl/templates/{name}"), source);
        }

        Environment::new(&scratch.canonical("project/.tpl"))
    }

    /// An environment over a project carrying `templates`, held to `fuel` and
    /// `limit`.
    fn bounded(
        scratch: &Scratch,
        templates: &[(&str, &str)],
        fuel: u64,
        limit: u64,
    ) -> Environment {
        project(scratch, templates);
        let bounds = RenderBounds {
            fuel: RenderFuel::new(fuel).expect("the test writes a value in range"),
            output_limit: RenderOutputLimit::new(limit).expect("the test writes a value in range"),
            memory_limit: RenderMemoryLimit::DEFAULT,
        };

        Environment::bounded(&scratch.canonical("project/.tpl"), bounds)
    }

    /// A template that loops without end, as far as any fuel reaches.
    ///
    /// The engine refuses a `range` longer than 100 000, so the loop nests
    /// three of 10 000: 10^12 iterations.
    const RUNAWAY: &str = "{% for i in range(10000) %}{% for j in range(10000) %}\
                           {% for k in range(10000) %}{% endfor %}{% endfor %}{% endfor %}";

    #[test]
    fn fr_rnd_036_a_render_that_exhausts_its_fuel_is_65_naming_the_bound() {
        // FR-RND-036, FR-SEC-025, H-1: the loop ends at a count, not at the
        // deadline, and the condition carries the resolved budget.
        let scratch = Scratch::new();
        let environment = bounded(&scratch, &[("loop.jinja", RUNAWAY)], 10_000, 1_000_000);

        let condition = environment
            .render("loop", &context! {})
            .expect_err("the loop outruns its fuel");

        assert!(
            matches!(condition, Error::RenderFuelExhausted { fuel: 10_000 }),
            "{condition:?}"
        );
        assert_eq!(condition.exit_code(), 65);
    }

    #[test]
    fn fr_rnd_036_fuel_exhausted_inside_an_include_is_still_the_fuel() {
        let scratch = Scratch::new();
        let environment = bounded(
            &scratch,
            &[
                ("outer.jinja", "{% include \"inner.jinja\" %}"),
                ("inner.jinja", RUNAWAY),
            ],
            10_000,
            1_000_000,
        );

        let condition = environment
            .render("outer", &context! {})
            .expect_err("the included loop outruns its fuel");

        assert!(
            matches!(condition, Error::RenderFuelExhausted { fuel: 10_000 }),
            "{condition:?}"
        );
    }

    #[test]
    fn fr_rnd_037_a_render_that_would_pass_its_output_limit_is_65_naming_the_bound() {
        // FR-RND-037: the writer refuses the byte past the limit, and the
        // render ends there with the resolved limit in the condition.
        let scratch = Scratch::new();
        let environment = bounded(
            &scratch,
            &[(
                "wide.jinja",
                "{% for i in range(100) %}0123456789{% endfor %}",
            )],
            1_000_000,
            64,
        );

        let condition = environment
            .render("wide", &context! {})
            .expect_err("a thousand bytes pass a limit of 64");

        assert!(
            matches!(condition, Error::RenderOutputLimitExceeded { limit: 64 }),
            "{condition:?}"
        );
        assert_eq!(condition.exit_code(), 65);
    }

    #[test]
    fn fr_rnd_037_output_exactly_at_the_limit_is_produced() {
        let scratch = Scratch::new();
        let environment = bounded(
            &scratch,
            &[(
                "wide.jinja",
                "{% for i in range(100) %}0123456789{% endfor %}",
            )],
            1_000_000,
            1_000,
        );

        let produced = environment
            .render("wide", &context! {})
            .expect("a thousand bytes fit a limit of 1000");

        assert_eq!(produced.len(), 1_000);
    }

    #[test]
    fn fr_conf_045_raising_the_bounds_lets_a_large_legitimate_render_pass() {
        // The same template refused under small bounds renders under the
        // defaults, which sit far above any legitimate render.
        let scratch = Scratch::new();
        let template = [(
            "large.jinja",
            "{% for i in range(20000) %}row {{ i }}\n{% endfor %}",
        )];

        let small = bounded(&scratch, &template, 1_000, 1_000);
        assert!(small.render("large", &context! {}).is_err());

        let raised = Environment::new(&scratch.canonical("project/.tpl"));
        let produced = raised
            .render("large", &context! {})
            .expect("the defaults admit it");

        assert!(produced.len() > 100_000, "{}", produced.len());
    }

    #[test]
    fn nfr_perf_006_no_engine_is_built_for_an_invocation_that_compiles_nothing() {
        // The lazy construction of `architecture.md`: an environment that only
        // resolves a path leaves the cell empty.
        let scratch = Scratch::new();
        let environment = project(&scratch, &[("example.jinja", "hello\n")]);

        assert!(environment.resolve("example").is_ok());
        assert!(environment.engine.get().is_none());

        environment.compile("example").expect("it parses");

        assert!(environment.engine.get().is_some());
    }

    #[test]
    fn fr_tmpl_020_a_template_with_a_syntax_error_is_65_naming_where_it_stopped() {
        // FR-TMPL-020, through the parse-only path of FR-TMPL-017.
        let scratch = Scratch::new();
        let environment = project(&scratch, &[("broken.jinja", "ok\n{% if %}\n")]);

        let condition = environment
            .compile("broken")
            .expect_err("it does not parse");

        assert!(matches!(condition, Error::TemplateSyntax { .. }));
        assert_eq!(condition.exit_code(), 65);
    }

    #[test]
    fn fr_env_026_auto_escaping_is_off_whatever_the_template_is_named() {
        // FR-ENV-026 and FR-ENV-027: off always, and never keyed on a name.
        // The engine's own default would escape the first of these three.
        let scratch = Scratch::new();
        let environment = project(
            &scratch,
            &[
                ("page.html.jinja", "{{ value }}"),
                ("feed.xml.jinja", "{{ value }}"),
                ("code.rs.jinja", "{{ value }}"),
            ],
        );
        let context = context! { value => "a & b < c" };

        for name in ["page.html", "feed.xml", "code.rs"] {
            assert_eq!(
                environment.render(name, &context).expect("it renders"),
                "a & b < c",
                "{name}"
            );
        }
    }

    #[test]
    fn fr_sem_010_an_interpolated_null_renders_as_the_empty_string() {
        // FR-SEM-010 and FR-SEM-011: the empty string, and never the words
        // `none` or `null`.
        let scratch = Scratch::new();
        let environment = project(&scratch, &[("null.jinja", "[{{ value }}]")]);
        let context = context! { value => Value::from(()) };

        assert_eq!(
            environment.render("null", &context).expect("it renders"),
            "[]"
        );
    }

    #[test]
    fn fr_sem_021_an_interpolated_boolean_renders_true_or_false_and_never_another_casing() {
        // FR-SEM-021: `tpl` is a code generator, and `True` is a token Rust,
        // Go, JSON and SQL all refuse. A template that writes a boolean into a
        // generated file must produce a file that builds, and must do so
        // without the author remembering a filter.
        let scratch = Scratch::new();
        let environment = project(
            &scratch,
            &[("flags.jinja", "{{ yes }} {{ no }} {{ maybe }}")],
        );
        let context = context! {
            yes => true,
            no => false,
            maybe => Value::from(1 == 1),
        };

        let written = environment.render("flags", &context).expect("it renders");

        assert_eq!(written, "true false true");
        assert!(!written.contains("True"), "{written}");
        assert!(!written.contains("False"), "{written}");
    }

    #[test]
    fn fr_sem_012_reading_a_field_that_does_not_exist_fails_the_render() {
        // FR-SEM-012 and FR-SEM-013: absence and `null` are different
        // failures, and this is the one that fails.
        let scratch = Scratch::new();
        let environment = project(&scratch, &[("absent.jinja", "{{ value.missing }}")]);
        let context = context! { value => context! { present => "x" } };

        let condition = environment
            .render("absent", &context)
            .expect_err("the field does not exist");

        assert!(matches!(condition, Error::RenderFailed { .. }));
        assert_eq!(condition.exit_code(), 65);
    }

    #[test]
    fn fr_sem_014_fail_ends_the_render_with_65_carrying_the_authors_message() {
        // FR-ENV-021, FR-SEM-014 and FR-SEM-015: the message reaches the
        // caller through the chain of FR-ERR-011, with the template and the
        // position beside it.
        let scratch = Scratch::new();
        let environment = project(
            &scratch,
            &[("stop.jinja", "one\n{{ fail('DECIMAL has no mapping') }}\n")],
        );

        let condition = environment
            .render("stop", &context! {})
            .expect_err("fail always fails");

        let Error::RenderFailed {
            template,
            position,
            chain,
            ..
        } = &condition
        else {
            panic!("fail is a render failure");
        };

        assert_eq!(condition.exit_code(), 65);
        assert_eq!(template, "stop.jinja");
        assert_eq!(position.line, 2);
        assert!(
            chain
                .iter()
                .any(|line| line.contains("DECIMAL has no mapping")),
            "{chain:?}"
        );
    }

    #[test]
    fn fr_sem_001_the_engine_s_whitespace_defaults_are_kept_and_the_newline_is_preserved() {
        // FR-SEM-001 and FR-SEM-002 keep the stock defaults; FR-SEM-003
        // departs from one deliberately, per BR-SEM-001; FR-SEM-004 makes
        // trimming explicit.
        let scratch = Scratch::new();
        let environment = project(
            &scratch,
            &[
                ("kept.jinja", "  {% if true %}\nbody\n{% endif %}\n"),
                ("trimmed.jinja", "  {%- if true -%}\nbody\n{%- endif %}\n"),
            ],
        );

        assert_eq!(
            environment
                .render("kept", &context! {})
                .expect("it renders"),
            "  \nbody\n\n"
        );
        assert_eq!(
            environment
                .render("trimmed", &context! {})
                .expect("it renders"),
            "body\n"
        );
    }

    #[test]
    fn fr_tmpl_009_an_include_that_does_not_resolve_literally_is_a_render_failure() {
        // FR-TMPL-008 and FR-TMPL-009, with the accepted cost the requirement
        // records: a name copied from `tpl template list` into an
        // `{% include %}` fails, because the listing omits the extension.
        let scratch = Scratch::new();
        let environment = project(
            &scratch,
            &[
                ("caller.jinja", "{% include \"_header\" %}"),
                ("correct.jinja", "{% include \"_header.jinja\" %}"),
                ("_header.jinja", "header"),
            ],
        );

        let condition = environment
            .render("caller", &context! {})
            .expect_err("the name does not resolve literally");

        assert!(matches!(condition, Error::RenderFailed { .. }));
        assert_eq!(condition.exit_code(), 65);
        assert_eq!(
            environment
                .render("correct", &context! {})
                .expect("it renders"),
            "header"
        );
    }

    #[test]
    fn fr_tmpl_024_an_include_of_a_symbolic_link_is_refused_as_an_escape() {
        // FR-TMPL-024: a symlinked entry is not includable either, and the
        // loader is where that is enforced for both paths.
        let scratch = Scratch::new();
        let environment = project(
            &scratch,
            &[("caller.jinja", "{% include \"leak.jinja\" %}")],
        );
        let secret = scratch.file("project/.tpl/.cfg", "[core]\n");
        scratch.link(&secret, &scratch.path("project/.tpl/templates/leak.jinja"));

        let condition = environment
            .render("caller", &context! {})
            .expect_err("a symbolic link is not a template");

        assert_eq!(condition.exit_code(), 65);
    }

    #[test]
    fn fr_tmpl_027_a_template_the_command_line_named_and_that_does_not_exist_is_66() {
        // FR-TMPL-027 and OD-15's table: the command line asks, so the code is
        // the missing template's and not the render's.
        let scratch = Scratch::new();
        let environment = project(&scratch, &[("example.jinja", "hello\n")]);

        for condition in [
            environment.compile("absent").expect_err("no such template"),
            environment
                .render("absent", &context! {})
                .expect_err("no such template"),
        ] {
            assert!(matches!(condition, Error::TemplateNotFound { .. }));
            assert_eq!(condition.exit_code(), 66);
        }
    }

    #[test]
    fn the_registered_surface_is_reachable_from_a_rendered_template() {
        // One render over one template, proving the registration reaches the
        // engine this module builds rather than a bare one.
        let scratch = Scratch::new();
        let environment = project(
            &scratch,
            &[(
                "surface.jinja",
                "{{ 'order_items' | pascal }} {{ 'a<b' | escape('xml') }} {{ 'x' | quote }}",
            )],
        );

        assert_eq!(
            environment
                .render("surface", &context! {})
                .expect("it renders"),
            "OrderItems a&lt;b `x`"
        );
    }
}
