#!/usr/bin/env python3
"""The Python worked example: a data layer for three schemas, and the gate that
decides whether it is correct.

``FR-EX-002`` fixes what this produces — source files that declare the tables of
``sakila``, ``world`` and ``freight`` as Python types — and ``FR-EX-009`` fixes
what makes it correct: not that ``tpl`` exited ``0``, but that Python's own
readers accept every file that was rendered. Python has no compiler, so the
clause's second reader applies: ``python -m compileall`` decides whether each
file is a Python file at all, and ``mypy --strict`` decides whether the types it
declares agree with one another — which is the half that catches a rendered
repository whose signature drifted from the protocol it claims to implement.

The workflow itself is not here. ``FR-EX-004`` fixes it, ``_driver`` implements
it once for all four examples, and what this module contributes is the one thing
that is this example's own: which template renders into which file. That is
:func:`plan`, and it is the whole of the example-specific surface the driver
asks for.

Standard library only, like the driver. Run it with the fixture up and the
password in the environment::

    export FIXTURE_PW=tpl-root
    python3 examples/python-data-layer/run.py
"""

from __future__ import annotations

import os
import shutil
import subprocess
import sys
import tempfile
from collections.abc import Sequence
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

#: The rendered data layer, one Python subpackage per schema. This is committed,
#: so that a reader sees what the example produces without running it.
#:
#: It is a package directory rather than an ``out/`` of its own because a Python
#: module's name *is* its import path: ``from datalayer import freight`` resolves
#: to ``datalayer/freight/__init__.py`` and nowhere else. Rendering into a
#: directory that is not a package would leave the files unimportable without a
#: ``sys.path`` entry per schema, which is the arrangement the language has an
#: answer to and the answer is a package.
OUTPUT = EXAMPLE / "datalayer"

#: The one Python file of the package this example does not render. It names the
#: three schema subpackages; everything else under :data:`OUTPUT` is generated
#: and is removed before each run.
PACKAGE_ROOT = OUTPUT / "__init__.py"

#: The oldest Python the rendered package is checked against.
#:
#: `mypy` is pinned to it rather than left to follow the interpreter running the
#: gate, so that the verdict is about the version the example claims to support
#: and not about the one that happened to be installed. It is the Rust example's
#: ``rust-version`` in another language.
TYPE_FLOOR = "3.11"


class UnsafeName(RuntimeError):
    """Raised when a catalogue name would be used as a path and must not be.

    ``BR-RND-003`` records the finding this guards: a filename built from
    catalogue data is a path-injection sink, and it is reachable from a
    ``--context`` document as well as from a server. ``tpl`` removed the
    file-writing surface for that reason, per ``FR-RND-028``, which puts the
    check where the writing now happens — here.
    """


class ToolchainMissing(RuntimeError):
    """Raised when the tools the compile gate needs are not available."""


class CompileGateFailed(RuntimeError):
    """Raised when a Python reader refuses a file the example rendered.

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

    One Python subpackage per schema, and within it:

    * ``_core.py``, rendered once with the whole database in context and no
      object flag, per ``FR-RND-006``: the schema name, the errors, the ordering
      direction, the translation of a driver failure, and the connection;
    * ``__init__.py``, rendered the same way: the package's public surface and
      the aggregate of every repository;
    * three files per table — the row and the types that address it, the
      ``Protocol`` it satisfies, and the PyMySQL implementation of that
      ``Protocol``.

    The schema's material is split across two files where the Rust example needs
    one, and the reason is the language: a Python package body executes, so a
    table module that imported its errors from ``__init__`` would depend on the
    order of the statements in ``__init__``. ``_core`` imports nothing from its
    own package and can therefore never be part of a cycle.

    One file is one invocation, per ``FR-EX-005``, so this returns
    ``3 x tables + 2 x schemas`` renders.
    """
    renders: list[Render] = []
    for schema in catalogue:
        package = _module_file(OUTPUT, schema.name)
        for template, name in (
            ("python/core", "_core.py"),
            ("python/package", "__init__.py"),
        ):
            renders.append(
                Render(
                    template=template,
                    destination=_module_file(package, name),
                    entry=schema.entry,
                )
            )
        for table in schema.tables:
            for template, suffix in (
                ("python/dataclass", ".py"),
                ("python/repository", "_repository.py"),
                ("python/pymysql", "_pymysql.py"),
            ):
                renders.append(
                    Render(
                        template=template,
                        destination=_module_file(package, table.name + suffix),
                        entry=table.entry,
                        table=table.name,
                    )
                )
    return renders


def reset_output() -> None:
    """Remove every rendered module, so that a rerun cannot leave a stale file.

    A table dropped from a schema leaves a file that still imports, and a gate
    that passes over it says nothing true. Only the contents of :data:`OUTPUT`
    are removed, only where they are real files and directories rather than
    links, and never :data:`PACKAGE_ROOT`, which is the one file of the package
    this example does not render.
    """
    if OUTPUT.is_symlink():
        raise UnsafeName(f"{OUTPUT} is a symbolic link; refusing to remove from it")
    if not PACKAGE_ROOT.is_file():
        raise UnsafeName(
            f"{PACKAGE_ROOT} is missing; it is the package root and is committed, so "
            "a checkout without it is incomplete"
        )
    OUTPUT.mkdir(parents=True, exist_ok=True)
    for entry in sorted(OUTPUT.iterdir()):
        if entry == PACKAGE_ROOT:
            continue
        if entry.is_symlink():
            raise UnsafeName(f"{entry} is a symbolic link; refusing to remove it")
        if entry.is_dir():
            shutil.rmtree(entry)
        else:
            entry.unlink()


#: The flags every `mypy` invocation of the gate carries.
_STRICT = ("--strict", "--python-version", TYPE_FLOOR)


def _mypy() -> list[str]:
    """Return the argv that runs `mypy`, or fail naming what is missing.

    ``sys.executable -m mypy`` is preferred over a ``mypy`` on the path, because
    it is the one that shares this interpreter's installed packages and
    therefore resolves the PyMySQL stubs the rendered code is checked against.

    Raises:
        ToolchainMissing: if neither form runs. A missing toolchain is a failure
            that names what is missing, and never a skip: the gate's verdict is
            this example's acceptance signal, so a gate that did not run has
            produced no verdict.
    """
    for candidate in ([sys.executable, "-m", "mypy"], [shutil.which("mypy") or ""]):
        if not candidate[0]:
            continue
        probe = subprocess.run(
            [*candidate, "--version"],
            cwd=EXAMPLE,
            capture_output=True,
            text=True,
            check=False,
        )
        if probe.returncode == 0:
            print(f"gate     {probe.stdout.strip()}", file=sys.stderr)
            return candidate
    raise ToolchainMissing(
        "the compile gate needs mypy and neither "
        f"`{Path(sys.executable).name} -m mypy` nor a `mypy` on PATH would run. "
        "FR-EX-009 makes the type checker's verdict half of this example's "
        "acceptance signal, so this is a failure rather than a skip:\n"
        f"    {Path(sys.executable).name} -m pip install mypy PyMySQL types-PyMySQL"
    )


def _stubs(mypy: Sequence[str], cache: Path) -> None:
    """Fail unless `mypy` can resolve PyMySQL, naming what to install.

    The rendered implementation imports PyMySQL, so a checkout without the
    package and its stubs makes every one of the 36 implementation modules
    unresolvable. That is an incomplete environment and not a defect in the
    rendered code, and the two are reported differently on purpose.

    Raises:
        ToolchainMissing: if a one-line module importing PyMySQL does not check.
    """
    with tempfile.TemporaryDirectory() as directory:
        probe = Path(directory) / "probe.py"
        probe.write_text("import pymysql\n\nCLIENT: str = pymysql.__name__\n", "utf-8")
        completed = subprocess.run(
            [*mypy, *_STRICT, "--cache-dir", str(cache), str(probe)],
            cwd=EXAMPLE,
            capture_output=True,
            text=True,
            check=False,
        )
    if completed.returncode != 0:
        raise ToolchainMissing(
            "mypy cannot resolve PyMySQL, so it cannot check the rendered "
            "implementation. PyMySQL ships no py.typed and its stubs are a "
            "package of their own:\n"
            f"    {Path(sys.executable).name} -m pip install PyMySQL types-PyMySQL\n"
            f"{completed.stdout}{completed.stderr}"
        )


def compile_gate() -> None:
    """Submit every rendered file to Python's own readers, per ``FR-EX-009``.

    ``python -m compileall`` decides whether each file is a Python file, and
    ``mypy --strict`` decides whether the types it declares agree — including
    whether each rendered implementation satisfies the ``Protocol`` rendered
    beside it, which is the one failure the example is most likely to produce
    and the one nothing else would report.

    Neither tool writes into the tree: the bytecode cache and the type-check
    cache both go to a temporary directory, so a rerun cannot leave anything
    behind for the next one to read.

    Raises:
        ToolchainMissing: if mypy is absent, or cannot resolve PyMySQL.
        CompileGateFailed: if either reader refuses a rendered file.
    """
    print(f"gate     {sys.implementation.name} {sys.version.split()[0]}", file=sys.stderr)
    mypy = _mypy()

    with tempfile.TemporaryDirectory() as scratch:
        cache = Path(scratch) / "mypy"
        _stubs(mypy, cache)

        environment = dict(os.environ)
        environment["PYTHONPYCACHEPREFIX"] = str(Path(scratch) / "pycache")

        commands: tuple[tuple[str, Sequence[str]], ...] = (
            (
                "python -m compileall",
                [sys.executable, "-m", "compileall", "-q", str(OUTPUT)],
            ),
            (
                f"mypy --strict --python-version {TYPE_FLOOR}",
                [*mypy, *_STRICT, "--cache-dir", str(cache), str(OUTPUT)],
            ),
        )
        for label, argv in commands:
            completed = subprocess.run(
                list(argv),
                cwd=EXAMPLE,
                capture_output=True,
                text=True,
                check=False,
                env=environment,
            )
            if completed.stdout:
                print(completed.stdout, end="", file=sys.stderr)
            if completed.stderr:
                print(completed.stderr, end="", file=sys.stderr)
            if completed.returncode != 0:
                raise CompileGateFailed(
                    f"{label} exited {completed.returncode} over the rendered "
                    "package. The render succeeded and the code is wrong, which "
                    "is the outcome FR-EX-009 states no earlier step can report."
                )
            print(f"gate     {label}: accepted", file=sys.stderr)


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
        f"every one of them mapped by templates/python/_types.jinja",
        file=sys.stderr,
    )


def main() -> int:
    """Render the data layer and submit it to Python's own readers."""
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
