"""The workflow every worked example runs, written once.

``FR-EX-004`` fixes it, in order:

===========================  ==================================
Create the project           ``tpl init``
Register the access          ``tpl cfg database add``
Verify the access            ``tpl cfg database test``
Read the schema              ``tpl schema tables``
Render                       ``tpl render``
===========================  ==================================

Four copies of that sequence would drift, and a drift would cost the set the
one property it is for: ``FR-EX-007`` requires the four examples to read the
same three schemas under the same conditions, so that a difference between two
rendered data layers is attributable to the templates and the type mappings and
to nothing else. Written once, that holds by construction.

What this module does **not** know is any example. There is no language here,
no template name, no file name and no table list: an example supplies those by
returning a sequence of :class:`Render` from its plan function, which this
module calls once the three schemas have been read.

Standard library only.
"""

from __future__ import annotations

import shutil
import sys
from collections.abc import Callable, Mapping, Sequence
from dataclasses import dataclass, field
from pathlib import Path
from typing import Any

from .catalogue import Catalogue, Schema, tables_from
from .cli import Invocation, Tpl
from .fixture import (
    PASSWORD_ENV,
    SCHEMAS,
    SERVER,
    ca_file,
    require_password,
)

__all__ = ["Access", "AccessRefused", "Outcome", "Plan", "Render", "run"]


@dataclass(frozen=True)
class Render:
    """One ``tpl render`` invocation, and the file its standard output becomes.

    ``tpl render`` declares no ``--output``: it writes its result to standard
    output and nowhere else, per ``FR-RND-028``, so a rendered file is a
    redirection the caller performs — ``FR-EX-005`` in as many words. Here the
    caller is the driver, and :attr:`destination` is where the redirection
    lands.

    One file rendered is one invocation, so a data layer of *n* tables is *n*
    of these plus whatever the example renders once for the whole database.

    At most one of :attr:`table`, :attr:`view` and :attr:`routine` may be set;
    with none, the whole database is in context.
    """

    #: The template, named as ``tpl template list`` names it: a path under the
    #: project's templates folder, without the ``.jinja`` extension.
    template: str
    #: Where the rendered bytes go. Relative paths resolve against the
    #: workspace. Parent directories are created.
    destination: Path
    #: The ``.tpl/.cfg`` entry to read through, which is what reaches ``-d``.
    #: A :class:`~._driver.catalogue.Table` carries the right one.
    entry: str
    table: str | None = None
    view: str | None = None
    routine: str | None = None
    #: Extra template variables, reaching the template as ``vars.<key>``.
    variables: Mapping[str, str] = field(default_factory=dict)

    def __post_init__(self) -> None:
        """Refuse a render that names more than one object.

        ``tpl render`` refuses it too, with exit ``64``, and catching it here
        names the offending render rather than an argv.
        """
        named = [
            label
            for label, value in (
                ("--table", self.table),
                ("--view", self.view),
                ("--routine", self.routine),
            )
            if value is not None
        ]
        if len(named) > 1:
            raise ValueError(
                f"render of {self.template!r} names {' and '.join(named)}; "
                "tpl render binds at most one object per invocation"
            )

    def arguments(self) -> tuple[str, ...]:
        """The argv after ``render``, in a fixed order.

        Fixed so that two runs of the same example produce the same commands,
        which is what makes a transcript comparable with the one before it.
        """
        argv: list[str] = ["-d", self.entry, self.template]
        for flag, value in (
            ("--table", self.table),
            ("--view", self.view),
            ("--routine", self.routine),
        ):
            if value is not None:
                argv += [flag, value]
        for key in sorted(self.variables):
            argv += ["--set", f"{key}={self.variables[key]}"]
        return tuple(argv)


@dataclass(frozen=True)
class Access:
    """What ``tpl cfg database test`` reported for one entry.

    The four answers of ``FR-CFG-024``. Exit ``0`` means the four steps ran and
    does not promise the entry is usable, so :attr:`can_read_catalogue` is read
    from the document rather than inferred from the code — ``FR-CFG-045``.
    """

    entry: str
    connected: bool
    read_only_session: bool
    can_read_catalogue: bool
    server: Mapping[str, Any]

    @property
    def usable(self) -> bool:
        """Are all three answers affirmative?"""
        return self.connected and self.read_only_session and self.can_read_catalogue


class AccessRefused(RuntimeError):
    """Raised when ``cfg database test`` exits ``0`` and says the entry is not usable.

    This is the failure the exit code does not carry, and the one ``UC-013``
    names in its second alternate flow: a reader that cannot see the catalogue
    would hand the example an incomplete model under ``FR-PRIV-001``, and the
    data layer would be short without anything saying so.
    """


@dataclass(frozen=True)
class Outcome:
    """What one run of the workflow produced."""

    #: The ``.tpl`` folder ``tpl init`` created.
    project: Path
    #: The workspace the example was given.
    workspace: Path
    #: The three schemas, read once each. Kept out of the default ``repr``: it
    #: carries every column of every table, so printing an :class:`Outcome`
    #: would print upwards of a megabyte of catalogue.
    catalogue: Catalogue = field(repr=False)
    #: Every file written, in render order, as absolute paths.
    written: tuple[Path, ...]
    #: What ``cfg database test`` reported, one per entry.
    access: tuple[Access, ...]
    #: Every ``tpl`` invocation the run made, in order.
    invocations: tuple[Invocation, ...]


#: An example's contribution: given the three schemas, say what to render.
#:
#: This is the whole of the example-specific surface. Everything the driver
#: knows is fixture; everything the example knows arrives through here.
Plan = Callable[[Catalogue], Sequence[Render]]


def run(
    workspace: Path,
    plan: Plan,
    *,
    templates: Path | None = None,
    binary: Path | None = None,
    reset: bool = True,
    echo: bool = True,
) -> Outcome:
    """Drive the whole workflow of ``FR-EX-004`` and return what it produced.

    This is the entry point every worked example calls, and the only one.

    Args:
        workspace: the **build** directory the project is created in. It is
            created if absent, and ``<workspace>/.tpl`` belongs to this
            function: an example keeps its own artefacts somewhere else and
            passes this a directory it is content to see rewritten.
        plan: called once, with the three schemas read, and returns the renders
            to perform. This is where an example's own knowledge lives.
        templates: a directory whose contents are copied into
            ``<workspace>/.tpl/templates/`` after ``tpl init``, merging over
            what ``init`` wrote so that an example may replace the macro of
            ``FR-PROJ-017`` as well as add to it. Nothing here reads a template
            or cares what is in one; the example owns the directory and this
            function only moves it into place, because ``tpl init`` creates the
            project and the renders need the templates there before step five.
        binary: a ``tpl`` executable to use instead of the one
            :func:`~._driver.discovery.tpl_binary` would find.
        reset: remove an existing ``<workspace>/.tpl`` before ``tpl init``, so
            that an example is re-runnable. Only that one directory, only when
            it is directly under the workspace and is a real directory rather
            than a link; nothing else under the workspace is touched, and no
            path outside it ever is.
        echo: write a line per step to standard error. The driver's own
            diagnostics go to standard error for the same reason it holds
            ``tpl`` to that split: standard output belongs to results.

    Raises:
        PasswordNotInEnvironment: if ``$FIXTURE_PW`` is not set.
        CommandFailed: on the first ``tpl`` invocation that exits non-zero.
        AccessRefused: if an entry connects and cannot read the catalogue.
    """
    from .discovery import tpl_binary

    require_password()
    executable = tpl_binary(binary)

    workspace = workspace.expanduser().resolve()
    project = workspace / ".tpl"

    def say(message: str) -> None:
        if echo:
            print(message, file=sys.stderr, flush=True)

    say(f"tpl      {executable}")
    say(f"project  {workspace}")

    # ------------------------------------------------- 1. create the project --
    if reset and project.is_dir() and not project.is_symlink():
        # Guarded: only ever the `.tpl` directly under the workspace.
        if project.name != ".tpl" or project.parent != workspace:
            raise RuntimeError(f"refusing to remove {project}")
        shutil.rmtree(project)
    workspace.mkdir(parents=True, exist_ok=True)

    tpl = Tpl(binary=executable)
    tpl.run("init", str(workspace))
    say(f"init     {project}")

    if templates is not None:
        source = templates.expanduser().resolve()
        if not source.is_dir():
            raise NotADirectoryError(f"no template directory at {source}")
        shutil.copytree(source, project / "templates", dirs_exist_ok=True)
        say(f"templates {source} -> {project / 'templates'}")

    # From here on every invocation names the project outright, so nothing
    # depends on the working directory or on the upward search finding this
    # `.tpl` rather than one above it.
    tpl.tpl_dir = project

    schemas: list[Schema] = []
    access: list[Access] = []

    for schema in SCHEMAS:
        entry = schema

        # ------------------------------------------- 2. register the access --
        tpl.run(
            "cfg",
            "database",
            "add",
            entry,
            "--host",
            SERVER.host,
            "--port",
            str(SERVER.port),
            "--user",
            SERVER.user,
            "--schema",
            schema,
            "--tls",
            SERVER.tls,
            "--ca-file",
            str(ca_file()),
        )
        # The password is the one field never written into the file and never
        # passed as a value: what goes in is the literal `${FIXTURE_PW}`, which
        # `tpl` expands from the environment at connection time. The single
        # quotes a shell would need are not needed here — no shell is involved.
        tpl.run("cfg", "set", f"database.{entry}.password", f"${{{PASSWORD_ENV}}}")
        say(f"add      {entry} -> {SERVER.user}@{SERVER.host}:{SERVER.port}/{schema}")

        # --------------------------------------------- 3. verify the access --
        reported = _verify(tpl, entry)
        access.append(reported)
        say(
            f"test     {entry}: connected={reported.connected} "
            f"read_only_session={reported.read_only_session} "
            f"can_read_catalogue={reported.can_read_catalogue} "
            f"server={reported.server.get('version')}"
        )

        # ------------------------------------------------ 4. read the schema --
        data = tpl.envelope("-d", entry, "schema", "tables")
        tables = tables_from(data, schema=schema, entry=entry)
        schemas.append(Schema(name=schema, entry=entry, tables=tables))
        say(f"schema   {entry}: {len(tables)} tables")

    catalogue = Catalogue(schemas=tuple(schemas))

    # ---------------------------------------------------------- 5. render ----
    renders = plan(catalogue)
    written: list[Path] = []
    for render in renders:
        destination = render.destination
        if not destination.is_absolute():
            destination = workspace / destination
        tpl.render_to(destination, *render.arguments())
        written.append(destination)
        # `Render.destination` documents only that a *relative* path resolves
        # against the workspace, so an absolute one may land outside it. That
        # is a legitimate destination and `relative_to` raises on it, so the
        # echo falls back to the absolute path rather than failing the run for
        # the sake of a progress line.
        try:
            shown: Path | str = destination.relative_to(workspace)
        except ValueError:
            shown = destination
        say(f"render   {render.template} -> {shown}")

    say(f"done     {len(written)} files, {len(tpl.log)} invocations")

    return Outcome(
        project=project,
        workspace=workspace,
        catalogue=catalogue,
        written=tuple(written),
        access=tuple(access),
        invocations=tuple(tpl.log),
    )


def _verify(tpl: Tpl, entry: str) -> Access:
    """Run ``cfg database test`` and refuse an entry that cannot read.

    Exit ``0`` from that command means the four steps ran, not that the entry
    is usable, so the fourth answer is read from the document. An entry that
    connects and cannot read the catalogue stops the run here, because the
    alternative is a data layer that is silently short.
    """
    data = tpl.envelope("cfg", "database", "test", entry)

    def flag(key: str) -> bool:
        value: Any = data.get(key)
        return value is True

    server: Any = data.get("server")
    reported = Access(
        entry=entry,
        connected=flag("connected"),
        read_only_session=flag("read_only_session"),
        can_read_catalogue=flag("can_read_catalogue"),
        server=server if isinstance(server, dict) else {},
    )
    if not reported.usable:
        raise AccessRefused(
            f"entry {entry!r} exited 0 and is not usable: "
            f"connected={reported.connected}, "
            f"read_only_session={reported.read_only_session}, "
            f"can_read_catalogue={reported.can_read_catalogue}. "
            "A read through it would be incomplete under FR-PRIV-001 and the "
            "data layer would be short without saying so."
        )
    return reported
