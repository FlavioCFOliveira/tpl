"""The workflow every worked example runs, and the only entry point into it.

A worked example builds an application's data layer for one target language out
of a known database schema, using nothing but the command line. The sequence it
drives is fixed by ``FR-EX-004``::

    tpl init  ->  tpl cfg database add  ->  tpl cfg database test
              ->  tpl schema ...        ->  tpl render ...

This package is that sequence, written once. ``FR-EX-007`` requires the four
examples to read the same three schemas from the same server under the same
conditions, so that a difference between two rendered data layers is
attributable to their templates and their type mappings and to nothing else.
Four copies of the sequence would drift, and the drift would not announce
itself.

**Nothing here knows any example.** No language, no template name, no file name,
no table list. An example supplies all of that by returning a sequence of
:class:`Render` from its plan function, which :func:`run` calls once the three
schemas have been read.

Usage, and this is the whole of it::

    from pathlib import Path
    from _driver import Catalogue, Render, run

    def plan(catalogue: Catalogue) -> list[Render]:
        return [
            Render(
                template="go/struct",
                destination=Path("out") / f"{table.name}.go",
                entry=table.entry,
                table=table.name,
            )
            for table in catalogue.schema("sakila").tables
        ]

    outcome = run(Path("build"), plan, templates=Path("templates"))

The password is never written into ``.tpl/.cfg`` and never passed on a command
line. The entries reference it as ``${FIXTURE_PW}`` and ``tpl`` expands it from
the environment, per ``BR-CONF-003``; set ``FIXTURE_PW`` before running.

Standard library only. No dependency has been taken, and none is needed: the
workflow is five command invocations, a JSON parse and a file write, all of
which ``subprocess``, ``json`` and ``pathlib`` do.
"""

from __future__ import annotations

from .catalogue import Catalogue, Schema, Table, UnexpectedDocument
from .cli import CommandFailed, Invocation, MalformedEnvelope, Tpl
from .discovery import BinaryNotFound, repository_root, tpl_binary
from .fixture import (
    PASSWORD_ENV,
    SCHEMAS,
    SERVER,
    PasswordNotInEnvironment,
    Server,
    ca_file,
)
from .workflow import Access, AccessRefused, Outcome, Plan, Render, run

__all__ = [
    "PASSWORD_ENV",
    "SCHEMAS",
    "SERVER",
    "Access",
    "AccessRefused",
    "BinaryNotFound",
    "Catalogue",
    "CommandFailed",
    "Invocation",
    "MalformedEnvelope",
    "Outcome",
    "PasswordNotInEnvironment",
    "Plan",
    "Render",
    "Schema",
    "Server",
    "Table",
    "Tpl",
    "UnexpectedDocument",
    "ca_file",
    "repository_root",
    "run",
    "tpl_binary",
]
