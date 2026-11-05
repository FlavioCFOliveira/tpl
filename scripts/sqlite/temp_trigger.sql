-- The fixture database of FR-LITE-038 whose schema table holds a trigger whose
-- text begins CREATE TEMP TRIGGER (FR-LITE-041, its seventh kind). SQLite
-- removes TEMP from the text it stores, and the library loads the row all the
-- same, while the derivation of FR-LITE-032 reads the trigger's name after
-- CREATE TRIGGER and would have misread it. SQLite writes no such text, so it
-- is written under writable_schema, by a connection whose defensive mode is
-- off and which sets none of the controls of FR-LITE-010. A read of it ends
-- with 78. It is built from this text at test time, by
-- tests/support/sqlite.rs.

CREATE TABLE consignment (
    consignment_id INTEGER PRIMARY KEY,
    reference TEXT NOT NULL,
    status TEXT NOT NULL DEFAULT 'booked'
);

CREATE TABLE consignment_audit (
    audit_id INTEGER PRIMARY KEY,
    consignment_id INTEGER,
    action TEXT NOT NULL
);

CREATE TRIGGER trg_consignment_after_update AFTER UPDATE OF status ON consignment
BEGIN
    INSERT INTO consignment_audit (consignment_id, action) VALUES (NEW.consignment_id, NEW.status);
END;

PRAGMA writable_schema = ON;
UPDATE sqlite_schema SET sql = 'CREATE TEMP TRIGGER' || substr(sql, length('CREATE TRIGGER') + 1)
    WHERE type = 'trigger' AND name = 'trg_consignment_after_update';
PRAGMA writable_schema = OFF;
