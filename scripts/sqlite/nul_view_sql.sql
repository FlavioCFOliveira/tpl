-- The fixture database of FR-LITE-038 whose schema table holds a view whose
-- sql is its definition followed by a NUL character and further text
-- (FR-LITE-041, its sixth kind). The library parses the text only up to the
-- NUL and loads the view, while the statement reading the stored text returns
-- the whole value as the view's definition, text the library never parsed
-- included. SQLite writes no such value, so it is written under
-- writable_schema, by a connection whose defensive mode is off and which sets
-- none of the controls of FR-LITE-010. A read of it ends with 78. It is built
-- from this text at test time, by tests/support/sqlite.rs.

CREATE TABLE vessel (
    vessel_imo INTEGER PRIMARY KEY,
    vessel_name TEXT NOT NULL
);

CREATE VIEW v_vessel_names AS SELECT vessel_imo, vessel_name FROM vessel;

INSERT INTO vessel (vessel_imo, vessel_name) VALUES (9778791, 'Madrid Maersk'), (9839131, 'MSC Gulsun');

PRAGMA writable_schema = ON;
UPDATE sqlite_schema SET sql = sql || char(0) || '; DROP TABLE vessel' WHERE type = 'view' AND name = 'v_vessel_names';
PRAGMA writable_schema = OFF;
