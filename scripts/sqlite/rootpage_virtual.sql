-- The fixture database of FR-LITE-038 whose schema table holds a
-- CREATE VIRTUAL TABLE row with a root page other than 0 (FR-LITE-041, its
-- third kind): a read that trusted the root page alone would read the virtual
-- table as an ordinary one, and connecting it would run its module's code.
-- SQLite writes no such row, so it is written under writable_schema, by a
-- connection whose defensive mode is off and which sets none of the controls
-- of FR-LITE-010. A read of it ends with 78. It is built from this text at
-- test time, by tests/support/sqlite.rs.

CREATE TABLE consignment (
    consignment_id INTEGER PRIMARY KEY,
    reference TEXT NOT NULL
);

CREATE VIRTUAL TABLE cargo_note USING fts5 (consignment_ref, body);

PRAGMA writable_schema = ON;
UPDATE sqlite_schema
    SET rootpage = (SELECT rootpage FROM sqlite_schema WHERE type = 'table' AND name = 'consignment')
    WHERE type = 'table' AND name = 'cargo_note';
PRAGMA writable_schema = OFF;
