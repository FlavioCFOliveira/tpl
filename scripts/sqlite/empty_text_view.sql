-- The fixture database of FR-LITE-038 whose schema table holds, for an index
-- the database holds, a second row whose sql is the empty text and whose type
-- is view (FR-LITE-041, its fifth kind). The library reads a row whose text
-- is empty as an index's, as it reads one whose text is NULL, and loads it,
-- while every statement that selects the rows whose type is view reads it as
-- a view with no definition, which the file does not hold. SQLite writes no
-- such row, so it is written under writable_schema, by a connection whose
-- defensive mode is off and which sets none of the controls of FR-LITE-010. A
-- read of it ends with 78. It is built from this text at test time, by
-- tests/support/sqlite.rs.

CREATE TABLE container_size (
    size_code INTEGER PRIMARY KEY,
    teu REAL NOT NULL DEFAULT 1.0,
    description TEXT
);

CREATE INDEX ix_container_teu ON container_size (teu);

INSERT INTO container_size (size_code, teu, description) VALUES (20, 1.0, 'twenty-foot'), (40, 2.0, 'forty-foot');

PRAGMA writable_schema = ON;
INSERT INTO sqlite_schema (type, name, tbl_name, rootpage, sql)
    SELECT 'view', name, tbl_name, rootpage, ''
    FROM sqlite_schema WHERE type = 'index' AND name = 'ix_container_teu';
PRAGMA writable_schema = OFF;
