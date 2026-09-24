//! The four labelled lines a failure writes, and the verbosity gate.
//!
//! `FR-ERR-033` withdraws the JSON error document, so the four labelled lines
//! of `FR-ERR-008` are an error's whole channel of detail and the exit code is
//! the caller's only machine-comparable signal. This module composes those
//! lines and writes them; `error.rs` owns the taxonomy and the code, and the
//! two are separated because `OD-06` separates them: four requirements pull
//! the lines apart from the type, and `OD-05` records that this module holds
//! transformations rather than a classification.
//!
//! The same module holds the verbosity gate, because `OD-17` puts the level and
//! the closed set of typed emission functions here. The two subjects share a
//! stream, an escaping rule and a prohibition — `FR-GLOB-018` and `BR-ERR-003`
//! bar the same six categories from a diagnostic and from an error message
//! alike.
//!
//! | Submodule | Subject | Forced by |
//! |---|---|---|
//! | [`escape`] | Escaping a composed line as a whole | `FR-ERR-024`, `OD-06` |
//! | [`cause`] | The `cause` line, per variant | `FR-ERR-010`, `FR-ERR-034` |
//! | [`hint`] | The generic `hint` line, and the character set a runnable command admits | `FR-ERR-009`, `FR-ERR-012`, `FR-ERR-022`, `FR-ERR-032` |
//! | [`suggest`] | The nearest-match selection, and the composed `hint` line | `FR-ERR-019` … `FR-ERR-023`, `FR-ERR-039`, `OD-20` |
//! | [`render`] | The four lines, their order, and the only writer of them | `FR-ERR-008`, `FR-ERR-033` |
//! | [`mod@panic`] | The panic hook, and the process exit it terminates with | `FR-ERR-030`, `FR-ERR-032`, `ADR-004` |
//! | [`verbosity`] | The level, resolved once and shared | `FR-GLOB-014`, `FR-GLOB-015`, `OD-17` |
//! | [`emit`] | The closed set of typed emission functions | `FR-GLOB-017`, `FR-GLOB-018`, `OD-17` |
//!
//! What is **not** here, and will not be: the eight candidate populations of
//! `FR-ERR-021`. Each belongs to the component that owns it — object names to
//! `mariadb/`, template names to `render/`, entry and key names to
//! `project/config.rs`, command and flag names to `cli/` — and [`suggest`] owns
//! the selection made over one of them and nothing else.
//!
//! `output/` is the deliberate neighbour, not the same module: `FR-OUT-018`
//! excepts tab in `text` read output and `FR-ERR-024` escapes it in every
//! message, on streams with opposite contracts — stdout is byte-identical under
//! `NFR-DET-001` and stderr is explicitly neither deterministic nor contract.

mod cause;
mod escape;
mod hint;
mod render;
mod restate;

pub(crate) mod emit;
pub(crate) mod panic;
pub(crate) mod suggest;
pub(crate) mod verbosity;

pub(crate) use restate::destination;
pub(crate) use restate::record as record_invocation;

pub(crate) use cause::invoked;
pub(crate) use render::report;

#[cfg(test)]
pub(crate) use render::rendered;

/// `text` with the caller's `--tpl-dir` and `-d/--database` carried into
/// every runnable `tpl` command it writes, exactly as a `hint` carries them
/// (`FR-ERR-043`).
///
/// For a line outside the four of `FR-ERR-008` that names a command to run:
/// copied without the flags, it would act on another project or another entry
/// (finding AB-01 of the eleventh re-audit of rmp `#263`).
pub(crate) fn carried(text: &'static str) -> std::borrow::Cow<'static, str> {
    restate::carried(
        std::borrow::Cow::Borrowed(text),
        restate::Carry {
            tpl_dir: true,
            database: true,
        },
    )
}
