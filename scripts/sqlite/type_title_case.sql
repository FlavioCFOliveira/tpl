-- The fixture database of FR-LITE-038 whose schema table holds an ordinary
-- table whose type is Table (FR-LITE-041, its fourth kind). The library loads
-- the row and uses the table, while every statement that selects the rows
-- whose type is table misses it: the table was missing from the model at
-- exit 0. SQLite writes no such type, so it is written under writable_schema,
-- by a connection whose defensive mode is off and which sets none of the
-- controls of FR-LITE-010. A read of it ends with 78. It is built from this
-- text at test time, by tests/support/sqlite.rs.

CREATE TABLE carrier (
    carrier_id INTEGER PRIMARY KEY,
    scac_code VARCHAR(4) NOT NULL UNIQUE
);

CREATE TABLE vessel (
    vessel_imo INTEGER PRIMARY KEY,
    vessel_name TEXT NOT NULL,
    carrier_id INTEGER REFERENCES carrier (carrier_id)
);

INSERT INTO carrier (carrier_id, scac_code) VALUES (1, 'MAEU');
INSERT INTO vessel (vessel_imo, vessel_name, carrier_id) VALUES (9778791, 'Madrid Maersk', 1);

PRAGMA writable_schema = ON;
UPDATE sqlite_schema SET type = 'Table' WHERE type = 'table' AND name = 'vessel';
PRAGMA writable_schema = OFF;
