"""Finding the two things on disk an example must not hard-code.

The ``tpl`` binary, because an example that named ``target/release/tpl``
outright would be wrong for anyone who installed the tool, and wrong again for
anyone who built it in debug; and the repository root, which is where the
fixture's trust material lives.

Standard library only.
"""

from __future__ import annotations

import os
import shutil
from pathlib import Path

__all__ = ["BinaryNotFound", "repository_root", "tpl_binary"]

#: Environment variable naming the binary outright, which overrides the search.
BINARY_ENV = "TPL_BIN"

#: Files and directories that together identify the repository root. A parent
#: holding all of them is the root; one holding some of them is a coincidence.
ROOT_MARKERS = ("Cargo.toml", "specification", "scripts/mariadb")

#: Build outputs searched, in order, before anything on ``PATH``. Release comes
#: first because it is what the project's validation pipeline produces; debug
#: is admitted so that an example is runnable mid-development.
BUILD_OUTPUTS = ("target/release/tpl", "target/debug/tpl")


class BinaryNotFound(RuntimeError):
    """Raised when no ``tpl`` executable could be located.

    The message names every place that was looked in, because the remedy
    differs per place and a bare "not found" leaves the caller guessing.
    """


def repository_root(start: Path | None = None) -> Path:
    """Return the repository root, found by walking up from ``start``.

    ``start`` defaults to the directory holding this file. The search stops at
    the first ancestor carrying every marker in :data:`ROOT_MARKERS`.

    Raises:
        BinaryNotFound: if no ancestor carries the markers.
    """
    here = (start or Path(__file__).resolve().parent).resolve()
    for candidate in (here, *here.parents):
        if all((candidate / marker).exists() for marker in ROOT_MARKERS):
            return candidate
    raise BinaryNotFound(
        f"no repository root above {here}: "
        f"no ancestor carries all of {', '.join(ROOT_MARKERS)}"
    )


def tpl_binary(explicit: Path | None = None) -> Path:
    """Return the ``tpl`` executable to invoke.

    The search order, and it is deliberate:

    1. ``explicit``, when the caller named one.
    2. The ``TPL_BIN`` environment variable.
    3. ``target/release/tpl`` then ``target/debug/tpl`` under the repository
       root — the binary this repository builds, which is the one an example
       run from the repository is demonstrating.
    4. ``tpl`` on ``PATH``, for an installed copy.

    Raises:
        BinaryNotFound: if every candidate is absent or not executable.
    """
    tried: list[str] = []

    if explicit is not None:
        resolved = explicit.expanduser().resolve()
        if _is_executable(resolved):
            return resolved
        tried.append(f"{resolved} (named by the caller)")

    from_env = os.environ.get(BINARY_ENV)
    if from_env:
        resolved = Path(from_env).expanduser().resolve()
        if _is_executable(resolved):
            return resolved
        tried.append(f"{resolved} (from ${BINARY_ENV})")

    root = repository_root()
    for relative in BUILD_OUTPUTS:
        candidate = root / relative
        if _is_executable(candidate):
            return candidate
        tried.append(str(candidate))

    on_path = shutil.which("tpl")
    if on_path is not None:
        return Path(on_path).resolve()
    tried.append("tpl on $PATH")

    raise BinaryNotFound(
        "no tpl executable found. Looked in:\n  "
        + "\n  ".join(tried)
        + "\nBuild it with `cargo build --release`, or name one in "
        + f"${BINARY_ENV}."
    )


def _is_executable(path: Path) -> bool:
    """Is ``path`` a regular file this process may execute?"""
    return path.is_file() and os.access(path, os.X_OK)
