"""The data layer this example renders: one subpackage per read.

This is the one Python file of the package that ``tpl`` does not render. It
names the five subpackages and nothing else; everything under it —
``sakila/``, ``world/`` and ``freight/`` from MariaDB, ``freight_mysql/`` from
MySQL and ``freight_sqlite/`` from SQLite — is generated from the catalogue by
``run.py`` and is removed and rewritten on every run.

It is hand-written because a render sees one database at a time, per
``FR-RND-006``, so no invocation of ``tpl render`` is in a position to name all
five. ``examples/rust-data-layer/src/lib.rs`` is the same file for the same
reason.

Importing this package imports the five subpackages, and therefore PyMySQL. A
caller that wants one schema imports it directly::

    from datalayer import freight

    connection = freight.connect(host="127.0.0.1", port=13309, user="root")
    repositories = freight.Repositories(connection)
    vessel = repositories.vessel.read(freight.VesselKey(vessel_id=1))
"""

from __future__ import annotations

from . import freight, freight_mysql, freight_sqlite, sakila, world

__all__ = ["freight", "freight_mysql", "freight_sqlite", "sakila", "world"]
