"""The one server and the three schemas every worked example reads.

``FR-EX-006`` has all four examples read ``sakila``, ``world`` and ``freight``
from one server of the most recent supported series, and ``FR-EX-007`` has them
read those schemas "under the same conditions: the same server, the same three
schemas at the same state, and database entries that differ in nothing a read
can observe".

That is why the access lives here and not in an example. Four copies of a host,
a port and a user would drift, and a difference between two rendered data
layers would then be attributable to two servers or two privilege sets rather
than to the templates and the type mappings the set exists to compare — which
is precisely the comparison ``FR-EX-007`` protects, and the drift would not
announce itself.

Standard library only.
"""

from __future__ import annotations

import os
from collections.abc import Sequence
from dataclasses import dataclass
from pathlib import Path

from .discovery import repository_root

__all__ = [
    "PASSWORD_ENV",
    "SCHEMAS",
    "SERVER",
    "PasswordNotInEnvironment",
    "Server",
    "ca_file",
    "require_password",
]

#: The environment variable the entries' password is read from.
#:
#: ``FR-CONF-021`` and ``specification/security.md`` are why it is read from
#: there and not passed on a command line: a value written into an argv is
#: visible in the process table for the life of the invocation, and every
#: process on the host can read it. ``${VAR}`` is the only mechanism by which
#: ``tpl`` reads the environment (``BR-CONF-003``), and it exists for exactly
#: this. What is written into ``.tpl/.cfg`` is the literal string
#: ``${FIXTURE_PW}``; the value never touches the file or the command line.
PASSWORD_ENV = "FIXTURE_PW"


@dataclass(frozen=True)
class Server:
    """The one server of ``FR-EX-006``, as a database entry must address it."""

    host: str
    port: int
    user: str
    #: TLS mode. ``verify-identity`` is ``tpl``'s own default and the strongest
    #: of the five: it validates the chain *and* requires the certificate to
    #: name the host. The fixture certificate carries ``IP:127.0.0.1``, so the
    #: examples demonstrate the strong mode rather than stepping down to make a
    #: local fixture work.
    tls: str = "verify-identity"


#: The fixture's most recent series, published on this port by
#: ``scripts/mariadb/up.sh``. The port, the user and the password belong to the
#: fixture and are documented in ``scripts/mariadb/README.md``; none of it is
#: secret, and the password is still taken from the environment because the
#: examples demonstrate the arrangement a real deployment needs.
SERVER = Server(host="127.0.0.1", port=13309, user="root")

#: The three schemas, in the order ``FR-EX-006`` names them.
#:
#: The entry each is registered under is the schema's own name, so an example
#: writes ``-d sakila`` and reads ``sakila``. The three entries differ in the
#: schema and in nothing else, which is ``FR-EX-007`` satisfied by construction
#: rather than by inspection.
#:
#: ``root`` is the user, and the choice is deliberate. The fixture's other user,
#: ``tpl_reader``, exists to make an *incomplete* catalogue read reproducible:
#: measured on this fixture it sees every table and every column of a schema and
#: none of its triggers, table constraints or referential constraints, so a data
#: layer built through it would silently lose every foreign key. An example is
#: the demonstration that the workflow produces a complete data layer, so it
#: reads through the user that can see one.
SCHEMAS: Sequence[str] = ("sakila", "world", "freight")


class PasswordNotInEnvironment(RuntimeError):
    """Raised when :data:`PASSWORD_ENV` is absent from the environment.

    Caught here rather than left to ``tpl``, which would report it correctly
    but only at the third step, as exit ``78`` from ``cfg database test``,
    after a project and three entries had already been written.
    """


def require_password() -> None:
    """Fail now if the password is not in the environment.

    The value is *not* returned. Nothing in this package reads it, writes it or
    passes it on: the entries reference it as ``${FIXTURE_PW}`` and the child
    process inherits the environment that holds it.
    """
    if not os.environ.get(PASSWORD_ENV):
        raise PasswordNotInEnvironment(
            f"${PASSWORD_ENV} is not set. The database entries reference it as "
            f'"${{{PASSWORD_ENV}}}", so the password stays out of .tpl/.cfg and '
            "out of every command line. Set it to the fixture's password for "
            f"{SERVER.user}, which scripts/mariadb/README.md documents:\n"
            f"    export {PASSWORD_ENV}=<password>"
        )


def ca_file() -> Path:
    """Return the fixture's root certificate, for ``--ca-file``.

    ``verify-identity`` validates the server's chain, and the fixture's
    certificate is signed by a root of its own, so the trust material has to be
    named. The file is the one ``scripts/mariadb/README.md`` names; it is
    committed, and the key that signed it was destroyed at generation time.

    Raises:
        FileNotFoundError: if the fixture's trust material is absent.
    """
    path = repository_root() / "scripts" / "mariadb" / "tls" / "ca.pem"
    if not path.is_file():
        raise FileNotFoundError(
            f"the fixture's root certificate is missing: {path}. "
            "It is committed; a checkout without it is incomplete."
        )
    return path
