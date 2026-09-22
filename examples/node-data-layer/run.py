#!/usr/bin/env python3
"""The Node.js worked example: a data layer for three schemas, and the gate that
decides whether it is correct.

``FR-EX-002`` fixes what this produces — source files that declare the tables of
``sakila``, ``world`` and ``freight`` as JavaScript types — and ``FR-EX-009``
fixes what makes it correct: not that ``tpl`` exited ``0``, but that the target
language's own readers accept every file that was rendered. JavaScript has no
compiler, so the clause's second reader applies, and it is two tools rather than
one: ``node --check`` decides whether each file is a JavaScript file at all, and
``tsc --checkJs`` decides whether the types the JSDoc annotations declare agree
with one another — which is the half that catches a rendered repository whose
signature drifted from the base class it extends.

The workflow itself is not here. ``FR-EX-004`` fixes it, ``_driver`` implements
it once for all four examples, and what this module contributes is the one thing
that is this example's own: which template renders into which file. That is
:func:`plan`, and it is the whole of the example-specific surface the driver
asks for.

Standard library only, like the driver. Run it with the fixture up, the
dependencies installed and the password in the environment::

    npm --prefix examples/node-data-layer install
    export FIXTURE_PW=tpl-root
    python3 examples/node-data-layer/run.py
"""

from __future__ import annotations

import shutil
import subprocess
import sys
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

#: The rendered data layer, one ES module directory per schema. This is
#: committed, so that a reader sees what the example produces without running
#: it.
#:
#: It is ``src/`` because that is what a Node package with no build step
#: publishes, and because nothing about Node forces the choice: a module
#: specifier is either a path relative to the importing file or a package
#: subpath resolved through ``exports`` in ``package.json``, and neither takes
#: its name from a directory. ``package.json`` maps ``./sakila`` onto
#: ``./src/sakila/index.js``, so the directory never appears in an import.
OUTPUT = EXAMPLE / "src"

#: Where the dependencies of the rendered package and of the gate are installed.
#: It is not committed, and a checkout without it is what ``npm install``
#: repairs.
MODULES = EXAMPLE / "node_modules"

#: The type checker's configuration. It is hand-written and committed, as
#: ``go.mod`` is in the Go example and ``Cargo.toml`` in the Rust one.
TSCONFIG = EXAMPLE / "tsconfig.json"


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
    """Raised when a JavaScript reader refuses a file the example rendered.

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

    One ES module directory per schema, and within it:

    * ``core.js``, rendered once with the whole database in context and no
      object flag, per ``FR-RND-006``: the schema name, the errors, the
      ordering direction, the translation of a driver failure, and the
      connection;
    * ``index.js``, rendered the same way: the package's public surface and the
      aggregate of every repository;
    * three files per table — the row and the types that address it, the base
      class that is the contract, and the mysql2 implementation of it.

    The schema's material is split across two files where the Rust example
    needs one, and the reason is the language: an ES module's classes are
    initialised in source order, so a table module that took its errors from
    ``index.js`` would be in a cycle with it. ``core.js`` imports nothing from
    its own directory and can therefore never be in one.

    A rendered file is named for the table exactly as the catalogue names it,
    and every import this example writes names it the same way, so no
    transformation stands between a module's name and its file's.

    One file is one invocation, per ``FR-EX-005``, so this returns
    ``3 x tables + 2 x schemas`` renders.
    """
    renders: list[Render] = []
    for schema in catalogue:
        directory = _module_file(OUTPUT, schema.name)
        for template, name in (("node/core", "core.js"), ("node/index", "index.js")):
            renders.append(
                Render(
                    template=template,
                    destination=_module_file(directory, name),
                    entry=schema.entry,
                )
            )
        for table in schema.tables:
            for template, suffix in (
                ("node/model", ".js"),
                ("node/repository", "_repository.js"),
                ("node/mysql2", "_mysql2.js"),
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
    """Empty the rendered tree, so that a rerun cannot leave a stale file.

    A table dropped from a schema leaves a file that still parses, and a gate
    that passes over it says nothing true. Only :data:`OUTPUT` is removed, only
    when it is a real directory, and never when it is a link.
    """
    if OUTPUT.is_symlink():
        raise UnsafeName(f"{OUTPUT} is a symbolic link; refusing to remove it")
    if OUTPUT.is_dir():
        shutil.rmtree(OUTPUT)
    OUTPUT.mkdir(parents=True)


def _node() -> str:
    """Return the ``node`` executable, or fail naming what is missing.

    Raises:
        ToolchainMissing: if no ``node`` is on the path. A missing toolchain is
            a failure that names what is missing, and never a skip: the gate's
            verdict is this example's acceptance signal, so a gate that did not
            run has produced no verdict.
    """
    node = shutil.which("node")
    if node is None:
        raise ToolchainMissing(
            "the compile gate needs Node.js and no `node` executable is on "
            "PATH. FR-EX-009 makes the language's own readers this example's "
            "acceptance signal, so this is a failure rather than a skip: "
            "install Node.js from https://nodejs.org/ and run this again."
        )
    return node


def _tsc(node: str) -> list[str]:
    """Return the argv that runs `tsc`, or fail naming what is missing.

    The copy installed under this example is preferred over anything else,
    because it is the one sharing a tree with the mysql2 type definitions the
    rendered code is checked against. ``npx --no-install`` is the fallback and
    resolves the same tree; neither form reaches the network, and nothing is
    installed globally.

    Raises:
        ToolchainMissing: if neither form runs.
    """
    local = MODULES / "typescript" / "bin" / "tsc"
    candidates: list[list[str]] = []
    if local.is_file():
        candidates.append([node, str(local)])
    npx = shutil.which("npx")
    if npx is not None:
        candidates.append([npx, "--no-install", "tsc"])

    for candidate in candidates:
        probe = subprocess.run(
            [*candidate, "--version"],
            cwd=EXAMPLE,
            capture_output=True,
            text=True,
            check=False,
        )
        if probe.returncode == 0:
            print(f"gate     TypeScript {probe.stdout.strip()}", file=sys.stderr)
            return candidate

    raise ToolchainMissing(
        "the compile gate needs the TypeScript compiler and neither the copy "
        f"under {MODULES} nor `npx --no-install tsc` would run. FR-EX-009 makes "
        "the type checker's verdict half of this example's acceptance signal, "
        "so this is a failure rather than a skip:\n"
        "    npm --prefix examples/node-data-layer install"
    )


#: The diagnostics that mean the environment is incomplete rather than the
#: rendered code wrong. TS2307 is an unresolved module specifier and TS2688 an
#: unresolved entry of `types`, and both are what a checkout without
#: `node_modules` produces — 114 times over, which would otherwise read as 114
#: defects in the rendered code.
_MISSING_DEPENDENCY = ("error TS2307", "error TS2688")


def _syntax_gate(node: str, files: Sequence[Path]) -> None:
    """Submit every rendered file to ``node --check``.

    ``node --check`` parses one file and takes one file, so this is one
    invocation per rendered file. It reads `package.json` to decide whether the
    file is a module, which is why `"type": "module"` is committed beside the
    rendered tree: parsed as a script, every one of these files is a syntax
    error at its first `import`.

    Raises:
        CompileGateFailed: on the first file Node refuses.
    """
    for path in files:
        completed = subprocess.run(
            [node, "--check", str(path)],
            cwd=EXAMPLE,
            capture_output=True,
            text=True,
            check=False,
        )
        if completed.returncode != 0:
            raise CompileGateFailed(
                f"node --check exited {completed.returncode} over "
                f"{path.relative_to(EXAMPLE)}. The render succeeded and the "
                "code is wrong, which is the outcome FR-EX-009 states no "
                f"earlier step can report.\n{completed.stdout}{completed.stderr}"
            )
    print(f"gate     node --check: accepted {len(files)} files", file=sys.stderr)


def _type_gate(tsc: Sequence[str]) -> None:
    """Submit the rendered package to ``tsc --checkJs --noEmit``.

    The flags are passed outright as well as set in :data:`TSCONFIG`, so that
    the two commands this gate runs are readable from this file alone. The
    configuration carries the rest: `strict`, `noImplicitOverride` — which is
    what makes a rendered `@override` tag a checked claim rather than a comment
    — and the module resolution a Node ES module package needs.

    Raises:
        ToolchainMissing: if the diagnostics say the dependencies are absent.
        CompileGateFailed: if the checker refuses a rendered file.
    """
    completed = subprocess.run(
        [*tsc, "--checkJs", "--noEmit", "--project", str(TSCONFIG)],
        cwd=EXAMPLE,
        capture_output=True,
        text=True,
        check=False,
    )
    output = completed.stdout + completed.stderr
    if completed.returncode != 0:
        if any(marker in output for marker in _MISSING_DEPENDENCY):
            raise ToolchainMissing(
                "tsc cannot resolve the packages the rendered code imports, so "
                "it cannot check it. That is an incomplete environment and not "
                "a defect in the rendered code, and the two are reported "
                "differently on purpose:\n"
                "    npm --prefix examples/node-data-layer install\n"
                f"{output}"
            )
        raise CompileGateFailed(
            f"tsc --checkJs --noEmit exited {completed.returncode} over the "
            "rendered package. The render succeeded and the code is wrong, "
            "which is the outcome FR-EX-009 states no earlier step can "
            f"report.\n{output}"
        )
    if output.strip():
        print(output, end="", file=sys.stderr)
    print("gate     tsc --checkJs --noEmit: accepted", file=sys.stderr)


def compile_gate(written: Sequence[Path]) -> None:
    """Submit every rendered file to JavaScript's own readers, per ``FR-EX-009``.

    Neither reader is sufficient alone. ``node --check`` accepts a file whose
    every type annotation is a lie, and it is the only one of the two that
    proves the file is a JavaScript file rather than a text that resembles one;
    ``tsc --checkJs`` reads the annotations and is what catches a rendered
    implementation whose signature drifted from its base class.

    Raises:
        ToolchainMissing: if Node or the type checker is absent.
        CompileGateFailed: if either reader refuses a rendered file.
    """
    node = _node()
    version = subprocess.run(
        [node, "--version"], cwd=EXAMPLE, capture_output=True, text=True, check=False
    )
    print(f"gate     Node {version.stdout.strip() or 'version unavailable'}", file=sys.stderr)
    tsc = _tsc(node)

    _syntax_gate(node, written)
    _type_gate(tsc)


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
        f"every one of them mapped by templates/node/_types.jinja",
        file=sys.stderr,
    )


def main() -> int:
    """Render the data layer and submit it to JavaScript's own readers."""
    try:
        reset_output()
        outcome = run(WORKSPACE, plan, templates=TEMPLATES)
        compile_gate(outcome.written)
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
