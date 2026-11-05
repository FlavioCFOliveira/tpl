-- The fixture database of FR-LITE-038 holding a table whose name holds a byte
-- that is not valid UTF-8, and a second table with a foreign key whose parent
-- table's name differs from the first table's only in another such byte
-- (FR-LITE-043). A legacy tool wrote the warehouse name in Latin-1,
-- entrep\xF4t (ô), and the key of stock_lot names entrep\xD4t (Ô): both
-- convert to the one text entrep�t under FR-OUT-017, and SQLite holds no
-- table entrep\xD4t, so the key is dangling and FR-LITE-021 excludes it. A
-- read of it succeeds, carrying the first table as entrep�t and
-- stock_lot with no foreign key. A statement of this text can only give
-- UTF-8, so the names are written under writable_schema, by a connection whose
-- defensive mode is off and which sets none of the controls of FR-LITE-010.
-- It is built from this text at test time, by tests/support/sqlite.rs.

-- The builder's library enforces foreign keys by default, and the row of
-- stock_lot names a parent no table holds; the key is the subject, and is
-- kept dangling, so enforcement is off for this connection alone.
PRAGMA foreign_keys = OFF;

CREATE TABLE entrepot_lower (
    warehouse_id INTEGER PRIMARY KEY,
    port_code CHAR(5) NOT NULL
);

CREATE TABLE stock_lot (
    lot_id INTEGER PRIMARY KEY,
    warehouse_id INTEGER NOT NULL REFERENCES entrepot_upper (warehouse_id),
    pallets INTEGER NOT NULL
);

INSERT INTO entrepot_lower (warehouse_id, port_code) VALUES (1, 'PTLIS');
INSERT INTO stock_lot (lot_id, warehouse_id, pallets) VALUES (1, 1, 22);

PRAGMA writable_schema = ON;
UPDATE sqlite_schema SET
    name = replace(name, 'entrepot_lower', 'entrep' || CAST(x'F4' AS TEXT) || 't'),
    tbl_name = replace(tbl_name, 'entrepot_lower', 'entrep' || CAST(x'F4' AS TEXT) || 't'),
    sql = replace(replace(sql, 'entrepot_lower', 'entrep' || CAST(x'F4' AS TEXT) || 't'),
                  'entrepot_upper', 'entrep' || CAST(x'D4' AS TEXT) || 't');
PRAGMA writable_schema = OFF;
