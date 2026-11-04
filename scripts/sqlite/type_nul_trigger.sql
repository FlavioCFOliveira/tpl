-- The fixture database of FR-LITE-038 whose schema table holds a trigger
-- whose type is trigger followed by a NUL character (FR-LITE-041, its fourth
-- kind). The library reads the type only up to the NUL, loads the trigger and
-- fires it, while a statement that selects the rows whose type is trigger
-- misses it: the trigger was missing from the read at exit 0. A comparison
-- folding letter case alone would miss it too, which is why the check takes
-- the whole value. SQLite writes no such type, so it is written under
-- writable_schema, by a connection whose defensive mode is off and which sets
-- none of the controls of FR-LITE-010. A read of it ends with 78. It is built
-- from this text at test time, by tests/support/sqlite.rs.

CREATE TABLE voyage (
    vessel_imo INTEGER NOT NULL,
    voyage_no INTEGER NOT NULL,
    status TEXT NOT NULL DEFAULT 'planned',
    PRIMARY KEY (vessel_imo, voyage_no)
);

CREATE TABLE voyage_audit (
    audit_id INTEGER PRIMARY KEY,
    vessel_imo INTEGER,
    voyage_no INTEGER,
    action TEXT NOT NULL
);

CREATE TRIGGER trg_voyage_after_insert AFTER INSERT ON voyage
BEGIN
    INSERT INTO voyage_audit (vessel_imo, voyage_no, action) VALUES (NEW.vessel_imo, NEW.voyage_no, 'insert');
END;

PRAGMA writable_schema = ON;
UPDATE sqlite_schema SET type = 'trigger' || char(0) WHERE type = 'trigger' AND name = 'trg_voyage_after_insert';
PRAGMA writable_schema = OFF;
