-- The fixture database of FR-LITE-038 whose schema table holds the type of a
-- trigger as a BLOB, which the schema table's contract excludes (FR-LITE-041,
-- its second kind): a statement that selects the rows whose type is the text
-- 'trigger' would miss it. SQLite writes no such value, so it is written
-- under writable_schema, by a connection whose defensive mode is off and which
-- sets none of the controls of FR-LITE-010. A read of it ends with 78. It is
-- built from this text at test time, by tests/support/sqlite.rs.

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
UPDATE sqlite_schema SET type = CAST('trigger' AS BLOB) WHERE name = 'trg_voyage_after_insert';
PRAGMA writable_schema = OFF;
