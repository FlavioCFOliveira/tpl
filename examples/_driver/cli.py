"""Invoking ``tpl``, and holding it to the contract it publishes.

An agent has three channels into a command-line tool and only three — the help
text, the exit code and the two streams — and this module is where a worked
example uses the last two as a contract rather than as a convenience:

* **Every** exit code is checked. A non-zero code stops the run and reports the
  command, the code and the whole of ``stderr``.
* ``stdout`` is the sole carrier of a result. Nothing here reads a result out
  of ``stderr``, and a render's bytes reach the file byte for byte.
* ``stderr`` is the sole carrier of a diagnostic. Nothing here reads a
  diagnostic out of ``stdout``, and ``stderr`` is never merged into it.

Nothing in this module reaches a library interface of ``tpl``: ``FR-EX-004``
admits the command line and nothing else.

Standard library only.
"""

from __future__ import annotations

import json
import subprocess
from collections.abc import Mapping
from dataclasses import dataclass, field
from pathlib import Path
from typing import Any

__all__ = ["CommandFailed", "Invocation", "MalformedEnvelope", "Tpl"]

#: Seconds any one invocation may take before it is killed. A worked example
#: that hangs is a worked example nobody can run in CI; the figure is generous
#: against the largest of the three schemas and is not a performance claim.
DEFAULT_TIMEOUT = 120.0


@dataclass(frozen=True)
class Invocation:
    """One ``tpl`` invocation and what the three channels carried.

    ``stdout`` is kept as bytes because a render's result becomes a file
    verbatim, and decoding it here would put an encoding decision between the
    tool and the file it is meant to produce.
    """

    argv: tuple[str, ...]
    returncode: int
    stdout: bytes
    stderr: str

    @property
    def command(self) -> str:
        """The invocation as a single line, for a message a person reads."""
        return " ".join(self.argv)

    def text(self) -> str:
        """``stdout`` decoded as UTF-8.

        ``FR-OUT`` fixes UTF-8 for every document ``tpl`` writes, so a decode
        failure here is a defect and is allowed to raise rather than be
        papered over with a replacement character.
        """
        return self.stdout.decode("utf-8")


class CommandFailed(RuntimeError):
    """Raised when a ``tpl`` invocation exits non-zero.

    It carries the invocation, so a caller that wants the code or the stream
    can reach them instead of parsing the message.
    """

    def __init__(self, invocation: Invocation) -> None:
        self.invocation = invocation
        stderr = invocation.stderr.rstrip()
        detail = f"\n{stderr}" if stderr else "\n(stderr was empty)"
        super().__init__(
            f"command exited {invocation.returncode}: {invocation.command}{detail}"
        )


class MalformedEnvelope(RuntimeError):
    """Raised when a JSON document is not the envelope ``FR-OUT-024`` fixes."""


@dataclass
class Tpl:
    """A located ``tpl`` binary bound to one project.

    Every method here builds an argv, runs it, checks the exit code and returns
    the :class:`Invocation`. The project is addressed with ``--tpl-dir`` on
    every call, so nothing depends on the working directory or on the upward
    search finding the intended ``.tpl`` rather than one above it.
    """

    binary: Path
    tpl_dir: Path | None = None
    timeout: float = DEFAULT_TIMEOUT
    #: Every invocation made through this object, in order, for the record.
    log: list[Invocation] = field(default_factory=list)

    # ------------------------------------------------------------ running ---

    def run(
        self,
        *arguments: str,
        check: bool = True,
        stdin: bytes | None = None,
    ) -> Invocation:
        """Run one ``tpl`` invocation and return what it produced.

        The two streams are captured separately and never merged: a caller that
        merged them would make a diagnostic indistinguishable from a result,
        which is the distinction the whole command-line contract rests on.

        Args:
            arguments: the argv after the binary. ``--tpl-dir`` is prepended
                when this object is bound to a project.
            check: raise :class:`CommandFailed` on a non-zero exit. Passing
                ``False`` hands the caller the code to decide on, and is for
                the caller that means to inspect a failure.
            stdin: bytes to feed the process, or ``None`` for no input.

        Raises:
            CommandFailed: when ``check`` is set and the exit code is non-zero.
            subprocess.TimeoutExpired: when the invocation outlives
                :attr:`timeout`.
        """
        argv = [str(self.binary)]
        if self.tpl_dir is not None:
            argv += ["--tpl-dir", str(self.tpl_dir)]
        argv += list(arguments)

        # No shell: argv is a list, built here, and never a string handed
        # to an interpreter. Nothing a schema name carries can become a
        # command.
        completed = subprocess.run(
            argv,
            input=stdin,
            stdout=subprocess.PIPE,
            stderr=subprocess.PIPE,
            timeout=self.timeout,
            check=False,
        )
        invocation = Invocation(
            argv=tuple(argv),
            returncode=completed.returncode,
            stdout=completed.stdout,
            stderr=completed.stderr.decode("utf-8", errors="replace"),
        )
        self.log.append(invocation)
        if check and invocation.returncode != 0:
            raise CommandFailed(invocation)
        return invocation

    def envelope(self, *arguments: str) -> Mapping[str, Any]:
        """Run an invocation that writes JSON and return its ``data``.

        The three keys of ``FR-OUT-024`` are checked before ``data`` is handed
        back, so a document that is merely valid JSON — a proxy's error page,
        a truncated write — fails here rather than three frames later as a
        missing key.
        """
        invocation = self.run(*arguments, "--format", "json")
        try:
            document: Any = json.loads(invocation.text())
        except (UnicodeDecodeError, json.JSONDecodeError) as error:
            raise MalformedEnvelope(
                f"{invocation.command} did not write JSON: {error}"
            ) from error

        if not isinstance(document, dict):
            raise MalformedEnvelope(
                f"{invocation.command} wrote a {type(document).__name__}, "
                "not the JSON object FR-OUT-024 fixes"
            )
        missing = [k for k in ("schema_version", "source", "data") if k not in document]
        if missing:
            raise MalformedEnvelope(
                f"{invocation.command} wrote a document without "
                f"{', '.join(missing)}: FR-OUT-024 fixes all three"
            )
        data: Any = document["data"]
        if not isinstance(data, dict):
            raise MalformedEnvelope(
                f"{invocation.command} wrote a data of type "
                f"{type(data).__name__}, not an object"
            )
        typed: dict[str, Any] = data
        return typed

    # -------------------------------------------------------- redirection ---

    def render_to(self, destination: Path, *arguments: str) -> Invocation:
        """Render once and write the result to ``destination``.

        ``tpl render`` declares no ``--output`` and writes to standard output
        and nowhere else, per ``FR-RND-028``, so obtaining a file is the
        caller's redirection — ``FR-EX-005`` says so in as many words. This is
        that redirection, done in Python: the bytes are captured and written
        unchanged, with no decode, no re-encode and no trailing newline added
        or removed.

        The file is only created once the invocation has exited ``0``. A
        failed render therefore leaves no truncated file behind for a compile
        gate to trip over.
        """
        invocation = self.run("render", *arguments)
        destination.parent.mkdir(parents=True, exist_ok=True)
        destination.write_bytes(invocation.stdout)
        return invocation

