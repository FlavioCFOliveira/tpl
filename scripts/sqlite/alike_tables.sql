-- The fixture database of FR-LITE-038 holding two tables whose names differ
-- only in a byte that is not valid UTF-8 (FR-LITE-043, its first set). A
-- legacy tool wrote the warehouse names in Latin-1: entrep\xF4t (ô) and
-- entrep\xD4t (Ô), neither of which is UTF-8, so both convert to the one text
-- entrep�t under FR-OUT-017. SQLite holds them as two tables, and before
-- rmp #349 the read folded the rows of both under that text, each table
-- carrying the other's columns at exit 0. SQLite writes a name as the bytes
-- its statement gave, but a statement of this text can only give UTF-8, so
-- the names are written under writable_schema, by a connection whose
-- defensive mode is off and which sets none of the controls of FR-LITE-010.
-- A read of it ends with 78. It is built from this text at test time, by
-- tests/support/sqlite.rs.

CREATE TABLE entrepot_lower (
    warehouse_id INTEGER PRIMARY KEY,
    port_code CHAR(5) NOT NULL,
    capacity_teu INTEGER NOT NULL
);

CREATE TABLE entrepot_upper (
    warehouse_id INTEGER PRIMARY KEY,
    bonded INTEGER NOT NULL DEFAULT 0
);

INSERT INTO entrepot_lower (warehouse_id, port_code, capacity_teu) VALUES (1, 'PTLIS', 1200);
INSERT INTO entrepot_upper (warehouse_id, bonded) VALUES (1, 1);

PRAGMA writable_schema = ON;
UPDATE sqlite_schema SET
    name = replace(replace(name, 'entrepot_lower', 'entrep' || CAST(x'F4' AS TEXT) || 't'),
                   'entrepot_upper', 'entrep' || CAST(x'D4' AS TEXT) || 't'),
    tbl_name = replace(replace(tbl_name, 'entrepot_lower', 'entrep' || CAST(x'F4' AS TEXT) || 't'),
                       'entrepot_upper', 'entrep' || CAST(x'D4' AS TEXT) || 't'),
    sql = replace(replace(sql, 'entrepot_lower', 'entrep' || CAST(x'F4' AS TEXT) || 't'),
                  'entrepot_upper', 'entrep' || CAST(x'D4' AS TEXT) || 't');
PRAGMA writable_schema = OFF;
