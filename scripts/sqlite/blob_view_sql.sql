-- The fixture database of FR-LITE-038 whose schema table holds the sql of a
-- view as a BLOB, which the schema table's contract excludes (FR-LITE-041,
-- its second kind). SQLite writes no such value, so it is written under
-- writable_schema, by a connection whose defensive mode is off and which sets
-- none of the controls of FR-LITE-010. A read of it ends with 78. It is built
-- from this text at test time, by tests/support/sqlite.rs.

CREATE TABLE port (
    port_code CHAR(5) PRIMARY KEY,
    port_name TEXT NOT NULL
);

CREATE VIEW v_port_names AS SELECT port_code, port_name FROM port;

INSERT INTO port (port_code, port_name) VALUES ('PTLIS', 'Lisbon'), ('NLRTM', 'Rotterdam');

PRAGMA writable_schema = ON;
UPDATE sqlite_schema SET sql = CAST(sql AS BLOB) WHERE type = 'view' AND name = 'v_port_names';
PRAGMA writable_schema = OFF;
