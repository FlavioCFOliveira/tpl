#!/usr/bin/env python3
"""The Go worked example: a data layer for three schemas, and the gate that
decides whether it is correct.

``FR-EX-002`` fixes what this produces — source files that declare the tables of
``sakila``, ``world`` and ``freight`` as Go types — and ``FR-EX-009`` fixes what
makes it correct: not that ``tpl`` exited ``0``, but that the Go toolchain
accepts every file that was rendered.

The workflow itself is not here. ``FR-EX-004`` fixes it, ``_driver`` implements
it once for all four examples, and what this module contributes is the one thing
that is this example's own: which template renders into which file. That is
:func:`plan`, and it is the whole of the example-specific surface the driver
asks for.

Standard library only, like the driver. Run it with the fixture up and the
password in the environment::

    export FIXTURE_PW=tpl-root
    python3 examples/go-data-layer/run.py
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
TEMPLATES = EXAMPLE / "templates"

#: The workspace the driver is given. ``tpl init`` creates ``<workspace>/.tpl``
#: there and owns it; nothing else in the directory is the driver's, and the
#: rendered files land under :data:`OUTPUT`, which is inside it and is this
#: example's own. ``.tpl`` is not committed: it holds the database entries of a
#: run rather than anything the example is.
WORKSPACE = EXAMPLE

#: The rendered data layer, one Go package per schema. This is committed, so
#: that a reader sees what the example produces without running it.
OUTPUT = EXAMPLE / "out"


class UnsafeName(RuntimeError):
    """Raised when a catalogue name would be used as a path and must not be.

    ``BR-RND-003`` records the finding this guards: a filename built from
    catalogue data is a path-injection sink, and it is reachable from a
    ``--context`` document as well as from a server. ``tpl`` removed the
    file-writing surface for that reason, per ``FR-RND-028``, which puts the
    check where the writing now happens — here.
    """


class ToolchainMissing(RuntimeError):
    """Raised when the Go toolchain is not available to the compile gate."""


class CompileGateFailed(RuntimeError):
    """Raised when the Go toolchain refuses a file the example rendered.

    This is the outcome the gate exists for. ``FR-EX-009`` states why nothing
    earlier can report it: a render succeeds when the template evaluated, which
    is a fact about the template and not about the file.
    """


def _package_file(directory: Path, name: str) -> Path:
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

    One Go package per schema, and within it:

    * ``schema.go``, rendered once with the whole database in context and no
      object flag, per ``FR-RND-006``: the package documentation, the errors,
      the ordering direction and the aggregate of every repository;
    * three files per table — the row struct, the repository interface, and the
      ``database/sql`` implementation of that interface.

    One file is one invocation, per ``FR-EX-005``, so this returns
    ``3 x tables + schemas`` renders.
    """
    renders: list[Render] = []
    for schema in catalogue:
        package = OUTPUT / schema.name
        renders.append(
            Render(
                template="go/schema",
                destination=_package_file(package, "schema.go"),
                entry=schema.entry,
            )
        )
        for table in schema.tables:
            for template, suffix in (
                ("go/struct", ".go"),
                ("go/repository", "_repository.go"),
                ("go/sql", "_sql.go"),
            ):
                renders.append(
                    Render(
                        template=template,
                        destination=_package_file(package, table.name + suffix),
                        entry=table.entry,
                        table=table.name,
                    )
                )
    return renders


def reset_output() -> None:
    """Empty the rendered package tree, so that a rerun cannot leave a stale file.

    A table dropped from a schema leaves a file that still compiles, and a
    compile gate that passes over it says nothing true. Only ``OUTPUT`` is
    removed, only when it is a real directory, and never when it is a link.
    """
    if OUTPUT.is_symlink():
        raise UnsafeName(f"{OUTPUT} is a symbolic link; refusing to remove it")
    if OUTPUT.is_dir():
        shutil.rmtree(OUTPUT)
    OUTPUT.mkdir(parents=True)


def compile_gate() -> None:
    """Submit every rendered file to the Go toolchain, per ``FR-EX-009``.

    ``go build`` decides whether the package compiles and ``go vet`` decides
    whether it is sound in the ways the compiler does not check. Both run over
    the whole module, so every file the example rendered is read.

    Raises:
        ToolchainMissing: if ``go`` is not on the path. A missing toolchain is a
            failure that names what is missing, and never a skip: the gate's
            verdict is this example's acceptance signal, so a gate that did not
            run has produced no verdict.
        CompileGateFailed: if either command exits non-zero.
    """
    go = shutil.which("go")
    if go is None:
        raise ToolchainMissing(
            "the compile gate needs the Go toolchain and no `go` executable is "
            "on PATH. FR-EX-009 makes the toolchain's verdict this example's "
            "acceptance signal, so this is a failure rather than a skip: "
            "install Go from https://go.dev/dl/ and run this again."
        )

    version = subprocess.run(
        [go, "version"], cwd=EXAMPLE, capture_output=True, text=True, check=False
    )
    print(f"gate     {version.stdout.strip() or 'go version unavailable'}", file=sys.stderr)

    for arguments in (("build", "./..."), ("vet", "./...")):
        completed = subprocess.run(
            [go, *arguments],
            cwd=EXAMPLE,
            capture_output=True,
            text=True,
            check=False,
        )
        command = "go " + " ".join(arguments)
        if completed.stdout:
            print(completed.stdout, end="", file=sys.stderr)
        if completed.stderr:
            print(completed.stderr, end="", file=sys.stderr)
        if completed.returncode != 0:
            raise CompileGateFailed(
                f"{command} exited {completed.returncode} over the rendered "
                f"package. The render succeeded and the code is wrong, which is "
                f"the outcome FR-EX-009 states no earlier step can report.\n"
                f"{completed.stdout}{completed.stderr}"
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
        f"every one of them mapped by templates/go/_types.jinja",
        file=sys.stderr,
    )


def main() -> int:
    """Render the data layer and submit it to the Go toolchain."""
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
