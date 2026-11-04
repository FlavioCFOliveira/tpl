-- The fixture database of FR-LITE-038 whose schema holds a table named
-- PRAGMA_TABLE_XINFO: SQLite compares the name ignoring ASCII case, so the
-- table would take the place of the pragma table-valued function
-- pragma_table_xinfo (FR-LITE-041, its first kind). A read of it ends with 78.
-- It is built from this text at test time, by tests/support/sqlite.rs.

CREATE TABLE vessel (
    vessel_imo INTEGER PRIMARY KEY,
    vessel_name TEXT NOT NULL
);

CREATE TABLE "PRAGMA_TABLE_XINFO" (
    cid INTEGER,
    name TEXT,
    type TEXT,
    "notnull" INTEGER,
    dflt_value TEXT,
    pk INTEGER,
    hidden INTEGER
);

INSERT INTO vessel (vessel_imo, vessel_name) VALUES (9778791, 'Madrid Maersk');
INSERT INTO "PRAGMA_TABLE_XINFO" VALUES (0, 'forged', 'TEXT', 0, NULL, 0, 0);
