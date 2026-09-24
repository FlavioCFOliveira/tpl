#!/usr/bin/env python3
"""The Rust worked example: a data layer for three schemas, and the gate that
decides whether it is correct.

``FR-EX-002`` fixes what this produces — source files that declare the tables of
``sakila``, ``world`` and ``freight`` as Rust types — and ``FR-EX-009`` fixes
what makes it correct: not that ``tpl`` exited ``0``, but that the Rust
toolchain accepts every file that was rendered.

The workflow itself is not here. ``FR-EX-004`` fixes it, ``_driver`` implements
it once for all four examples, and what this module contributes is the one thing
that is this example's own: which template renders into which file. That is
:func:`plan`, and it is the whole of the example-specific surface the driver
asks for.

Standard library only, like the driver. Run it with the fixture up and the
password in the environment::

    export FIXTURE_PW=tpl-root
    python3 examples/rust-data-layer/run.py
"""

from __future__ import annotations

import shutil
import subprocess
import sys
from pathlib import Path

EXAMPLE = Path(__file__).resolve().parent
sys.path.insert(0, str(EXAMPLE.parent))

from _driver import (  # noqa: E402
    AccessRefused,
    BinaryNotFound,
    Catalogue,
    CommandFailed,
    MalformedEnvelope,
    PasswordNotInEnvironment,
    Render,
    UnexpectedDocument,
    run,
)

#: The templates this example owns, copied into the project by the driver.
#: ``rust/_types.jinja`` among them replaces the starter macro ``tpl init``
#: writes under ``FR-PROJ-017``, which is what ``FR-EX-008`` allows this example
#: to do.
TEMPLATES = EXAMPLE / "templates"

#: The workspace the driver is given. ``tpl init`` creates ``<workspace>/.tpl``
#: there and owns it; nothing else in the directory is the driver's, and the
#: rendered files land under :data:`OUTPUT`, which is inside it and is this
#: example's own. ``.tpl`` is not committed: it holds the database entries of a
#: run rather than anything the example is.
WORKSPACE = EXAMPLE

#: The rendered data layer, one Rust module per schema. This is committed, so
#: that a reader sees what the example produces without running it.
#:
#: It is the crate's source directory, because a Rust module's path is its
#: file's path: ``pub mod sakila;`` in :data:`CRATE_ROOT` resolves to
#: ``src/sakila.rs`` and that module's children to ``src/sakila/``. Rendering
#: elsewhere would need a ``#[path]`` attribute on every module declaration.
OUTPUT = EXAMPLE / "src"

#: The one Rust file of the crate this example does not render. It carries the
#: crate's attributes and names the three schema modules; everything else under
#: :data:`OUTPUT` is generated and is removed before each run.
CRATE_ROOT = OUTPUT / "lib.rs"


class UnsafeName(RuntimeError):
    """Raised when a catalogue name would be used as a path and must not be.

    ``BR-RND-003`` records the finding this guards: a filename built from
    catalogue data is a path-injection sink, and it is reachable from a
    ``--context`` document as well as from a server. ``tpl`` removed the
    file-writing surface for that reason, per ``FR-RND-028``, which puts the
    check where the writing now happens — here.
    """


class ToolchainMissing(RuntimeError):
    """Raised when the Rust toolchain is not available to the compile gate."""


class CompileGateFailed(RuntimeError):
    """Raised when the Rust toolchain refuses a file the example rendered.

    This is the outcome the gate exists for. ``FR-EX-009`` states why nothing
    earlier can report it: a render succeeds when the template evaluated, which
    is a fact about the template and not about the file.
    """


def _module_file(directory: Path, name: str) -> Path:
    """Return ``directory/name``, refusing a name that would leave it.

    Raises:
        UnsafeName: if the name carries a path separator, or resolves anywhere
            other than directly inside ``directory``.
    """
    candidate = directory / name
    if "/" in name or "\\" in name or name in {"", ".", ".."} or name.startswith("."):
        raise UnsafeName(f"the catalogue name {name!r} is not usable as a file name")
    if candidate.parent != directory:
        raise UnsafeName(f"the catalogue name {name!r} escapes {directory}")
    return candidate


def plan(catalogue: Catalogue) -> list[Render]:
    """Say what to render, which is this example's whole contribution.

    One Rust module per schema, and within it:

    * ``<schema>.rs``, rendered once with the whole database in context and no
      object flag, per ``FR-RND-006``: the module documentation, the errors, the
      ordering direction, the two column adapters the type mapping names, the
      connection, and the aggregate of every repository;
    * three files per table — the row, the repository trait, and the ``sqlx``
      implementation of that trait.

    One file is one invocation, per ``FR-EX-005``, so this returns
    ``3 x tables + schemas`` renders.
    """
    renders: list[Render] = []
    for schema in catalogue:
        module = _module_file(OUTPUT, f"{schema.name}.rs")
        renders.append(
            Render(
                template="rust/schema",
                destination=module,
                entry=schema.entry,
            )
        )
        directory = _module_file(OUTPUT, schema.name)
        for table in schema.tables:
            for template, suffix in (
                ("rust/struct", ".rs"),
                ("rust/repository", "_repository.rs"),
                ("rust/sqlx", "_sqlx.rs"),
            ):
                renders.append(
                    Render(
                        template=template,
                        destination=_module_file(directory, table.name + suffix),
                        entry=table.entry,
                        table=table.name,
                    )
                )
    return renders


def reset_output() -> None:
    """Remove every rendered module, so that a rerun cannot leave a stale file.

    A table dropped from a schema leaves a file that still compiles, and a
    compile gate that passes over it says nothing true. Only the contents of
    :data:`OUTPUT` are removed, only where they are real files and directories
    rather than links, and never :data:`CRATE_ROOT`, which is the one file of
    the crate this example does not render.
    """
    if OUTPUT.is_symlink():
        raise UnsafeName(f"{OUTPUT} is a symbolic link; refusing to remove from it")
    if not CRATE_ROOT.is_file():
        raise UnsafeName(
            f"{CRATE_ROOT} is missing; it is the crate root and is committed, so a "
            "checkout without it is incomplete"
        )
    OUTPUT.mkdir(parents=True, exist_ok=True)
    for entry in sorted(OUTPUT.iterdir()):
        if entry == CRATE_ROOT:
            continue
        if entry.is_symlink():
            raise UnsafeName(f"{entry} is a symbolic link; refusing to remove it")
        if entry.is_dir():
            shutil.rmtree(entry)
        else:
            entry.unlink()


def _cargo() -> str:
    """Return the ``cargo`` executable, or fail naming what is missing.

    Raises:
        ToolchainMissing: if ``cargo`` is not on the path, or if the ``clippy``
            component is not installed. A missing toolchain is a failure that
            names what is missing, and never a skip: the gate's verdict is this
            example's acceptance signal, so a gate that did not run has produced
            no verdict.
    """
    cargo = shutil.which("cargo")
    if cargo is None:
        raise ToolchainMissing(
            "the compile gate needs the Rust toolchain and no `cargo` executable "
            "is on PATH. FR-EX-009 makes the toolchain's verdict this example's "
            "acceptance signal, so this is a failure rather than a skip: install "
            "Rust from https://rustup.rs and run this again."
        )
    probe = subprocess.run(
        [cargo, "clippy", "--version"],
        cwd=EXAMPLE,
        capture_output=True,
        text=True,
        check=False,
    )
    if probe.returncode != 0:
        raise ToolchainMissing(
            "the compile gate needs clippy and `cargo clippy --version` failed:\n"
            f"{probe.stdout}{probe.stderr}"
            "Install it with `rustup component add clippy` and run this again."
        )
    print(f"gate     {probe.stdout.strip()}", file=sys.stderr)
    return cargo


def compile_gate() -> None:
    """Submit every rendered file to the Rust toolchain, per ``FR-EX-009``.

    ``cargo build`` decides whether the crate compiles and
    ``cargo clippy -- -D warnings`` decides whether it is idiomatic in the ways
    the compiler does not check. Both read every file of the crate, so every
    file the example rendered is read.

    Raises:
        ToolchainMissing: if the toolchain is not available.
        CompileGateFailed: if either command exits non-zero.
    """
    cargo = _cargo()
    version = subprocess.run(
        [cargo, "--version"], cwd=EXAMPLE, capture_output=True, text=True, check=False
    )
    print(f"gate     {version.stdout.strip() or 'cargo version unavailable'}", file=sys.stderr)

    for arguments in (
        ("build",),
        ("clippy", "--all-targets", "--all-features", "--", "-D", "warnings"),
    ):
        completed = subprocess.run(
            [cargo, *arguments],
            cwd=EXAMPLE,
            capture_output=True,
            text=True,
            check=False,
        )
        command = "cargo " + " ".join(arguments)
        if completed.stdout:
            print(completed.stdout, end="", file=sys.stderr)
        if completed.stderr:
            print(completed.stderr, end="", file=sys.stderr)
        if completed.returncode != 0:
            raise CompileGateFailed(
                f"{command} exited {completed.returncode} over the rendered crate. "
                f"The render succeeded and the code is wrong, which is the outcome "
                f"FR-EX-009 states no earlier step can report."
            )
        print(f"gate     {command}: accepted", file=sys.stderr)


def report(catalogue: Catalogue, written: int) -> None:
    """Write what was rendered, and over how much of the type surface.

    The count of distinct ``data_type`` values is the measure that matters for
    ``FR-EX-008``: the mapping is obliged over every value the model reports for
    a column of the three schemas, and this says how many that was.
    """
    types: set[str] = set()
    columns = 0
    for table in catalogue.tables:
        for column in table.raw.get("columns", []):
            columns += 1
            data_type = column.get("data_type")
            if isinstance(data_type, str):
                types.add(data_type)

    print(f"\nrendered {written} files into {OUTPUT}", file=sys.stderr)
    for schema in catalogue:
        print(f"  {schema.name:8s} {len(schema.tables):3d} tables", file=sys.stderr)
    print(
        f"  {columns} columns carrying {len(types)} distinct data_type values, "
        f"every one of them mapped by templates/rust/_types.jinja",
        file=sys.stderr,
    )


def main() -> int:
    """Render the data layer and submit it to the Rust toolchain."""
    try:
        reset_output()
        outcome = run(WORKSPACE, plan, templates=TEMPLATES)
        compile_gate()
    except (
        AccessRefused,
        BinaryNotFound,
        CommandFailed,
        CompileGateFailed,
        MalformedEnvelope,
        PasswordNotInEnvironment,
        ToolchainMissing,
        UnexpectedDocument,
        UnsafeName,
    ) as failure:
        print(f"\n{type(failure).__name__}: {failure}", file=sys.stderr)
        return 1

    report(outcome.catalogue, len(outcome.written))
    return 0


if __name__ == "__main__":
    sys.exit(main())
