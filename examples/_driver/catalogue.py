"""What a schema read hands a worked example.

The driver reads each schema once, with ``tpl schema tables --format json``,
and hands the example the result as data rather than as a stream to re-parse.
The model here is deliberately thin: an example needs to know which tables
exist, in which schema, and through which database entry to render them. What
each column *is* belongs to the template and to the example's type-mapping
macro, per ``FR-EX-008``, so nothing here interprets a type.

Every object keeps the JSON it came from, under ``raw``. An example that needs
a field this module does not model reaches it there instead of waiting for this
module to grow an opinion about it.

Standard library only.
"""

from __future__ import annotations

from collections.abc import Iterator, Mapping, Sequence
from dataclasses import dataclass
from typing import Any

__all__ = ["Catalogue", "Schema", "Table", "UnexpectedDocument"]


class UnexpectedDocument(RuntimeError):
    """Raised when a ``data`` payload is not shaped as the contract fixes it."""


@dataclass(frozen=True)
class Table:
    """One table of one schema, as ``tpl schema tables`` reported it."""

    #: The server-side schema this table belongs to, e.g. ``"sakila"``.
    schema: str
    #: The ``.tpl/.cfg`` entry that reads that schema. This is what an example
    #: passes to ``-d``, and it is why a :class:`Table` is enough on its own to
    #: build a render from.
    entry: str
    name: str
    #: ``"BASE TABLE"`` for a table. ``tpl schema tables`` lists tables, so
    #: this is not the place a view arrives; it is kept because the field is in
    #: the document and dropping it would be an interpretation.
    table_type: str
    #: The whole JSON object this table came from, columns included.
    raw: Mapping[str, Any]

    @property
    def qualified(self) -> str:
        """``schema.name``, for a message or a comment in a rendered file."""
        return f"{self.schema}.{self.name}"


@dataclass(frozen=True)
class Schema:
    """One schema, the entry that reads it, and the tables it holds."""

    #: The server-side schema name.
    name: str
    #: The ``.tpl/.cfg`` entry registered for it.
    entry: str
    tables: tuple[Table, ...]

    def table(self, name: str) -> Table:
        """Return the table called ``name``.

        Raises:
            KeyError: if this schema holds no such table. The message lists
                what it does hold, because the usual cause is a spelling.
        """
        for table in self.tables:
            if table.name == name:
                return table
        known = ", ".join(t.name for t in self.tables) or "(none)"
        raise KeyError(f"{self.name} has no table {name!r}. It has: {known}")


@dataclass(frozen=True)
class Catalogue:
    """The three schemas of ``FR-EX-006``, read once each."""

    schemas: tuple[Schema, ...]

    def __iter__(self) -> Iterator[Schema]:
        """Iterate the schemas in the order ``FR-EX-006`` names them."""
        return iter(self.schemas)

    def __len__(self) -> int:
        """The number of schemas read."""
        return len(self.schemas)

    @property
    def tables(self) -> tuple[Table, ...]:
        """Every table of every schema, schema order preserved."""
        return tuple(table for schema in self.schemas for table in schema.tables)

    def schema(self, name: str) -> Schema:
        """Return the schema called ``name``.

        Raises:
            KeyError: if no schema of that name was read.
        """
        for schema in self.schemas:
            if schema.name == name:
                return schema
        known = ", ".join(s.name for s in self.schemas) or "(none)"
        raise KeyError(f"no schema {name!r} was read. Read: {known}")


def tables_from(
    data: Mapping[str, Any], *, schema: str, entry: str
) -> tuple[Table, ...]:
    """Build the tables of one schema from a ``schema tables`` ``data``.

    The document's shape is checked rather than assumed: ``data.tables`` is a
    list of objects, each carrying ``name``. A document that is valid JSON and
    not that fails here, naming what was found, instead of raising an opaque
    ``TypeError`` further along.

    Raises:
        UnexpectedDocument: if ``data`` is not shaped as described.
    """
    listing: Any = data.get("tables")
    if not isinstance(listing, list):
        raise UnexpectedDocument(
            f"the read of {schema} carried a data.tables of type "
            f"{type(listing).__name__}, not a list"
        )

    built: list[Table] = []
    rows: Sequence[Any] = listing
    for position, row in enumerate(rows):
        if not isinstance(row, dict):
            raise UnexpectedDocument(
                f"the read of {schema} carried a data.tables[{position}] of "
                f"type {type(row).__name__}, not an object"
            )
        typed_row: dict[str, Any] = row
        name: Any = typed_row.get("name")
        if not isinstance(name, str) or not name:
            raise UnexpectedDocument(
                f"the read of {schema} carried a data.tables[{position}] "
                "without a usable name"
            )
        table_type: Any = typed_row.get("table_type")
        built.append(
            Table(
                schema=schema,
                entry=entry,
                name=name,
                table_type=table_type if isinstance(table_type, str) else "",
                raw=typed_row,
            )
        )
    return tuple(built)
