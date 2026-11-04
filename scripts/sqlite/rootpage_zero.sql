-- The fixture database of FR-LITE-038 whose schema table holds a CREATE TABLE
-- row with a root page of 0, which is a virtual table's (FR-LITE-041, its
-- third kind): a read that trusted the root page alone would exclude the
-- table as virtual, at exit 0. SQLite writes no such row, so it is written
-- under writable_schema, by a connection whose defensive mode is off and which
-- sets none of the controls of FR-LITE-010. A read of it ends with 78. It is
-- built from this text at test time, by tests/support/sqlite.rs.

CREATE TABLE carrier (
    carrier_id INTEGER PRIMARY KEY,
    scac_code VARCHAR(4) NOT NULL UNIQUE
);

CREATE TABLE berth (
    berth_no INTEGER PRIMARY KEY,
    port_code CHAR(5) NOT NULL
);

PRAGMA writable_schema = ON;
UPDATE sqlite_schema SET rootpage = 0 WHERE type = 'table' AND name = 'berth';
PRAGMA writable_schema = OFF;
