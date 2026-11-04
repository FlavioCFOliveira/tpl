-- The fixture database of FR-LITE-038 whose schema table holds a table named
-- pragma_encoding, holding one row, whose type is TABLE (FR-LITE-041, its
-- fourth kind). The library loads a type written in another letter case and
-- uses its table, which then takes the place of the pragma table-valued
-- function pragma_encoding; a check that compared the type exactly for the
-- first kind alone let it forge the encoding at exit 0. SQLite writes no such
-- type, so it is written under writable_schema, by a connection whose
-- defensive mode is off and which sets none of the controls of FR-LITE-010. A
-- read of it ends with 78. It is built from this text at test time, by
-- tests/support/sqlite.rs.

CREATE TABLE port (
    port_code CHAR(5) PRIMARY KEY,
    port_name TEXT NOT NULL
);

CREATE TABLE pragma_encoding (
    encoding TEXT
);

INSERT INTO port (port_code, port_name) VALUES ('PTLIS', 'Lisbon'), ('NLRTM', 'Rotterdam');
INSERT INTO pragma_encoding (encoding) VALUES ('forged');

PRAGMA writable_schema = ON;
UPDATE sqlite_schema SET type = 'TABLE' WHERE type = 'table' AND name = 'pragma_encoding';
PRAGMA writable_schema = OFF;
