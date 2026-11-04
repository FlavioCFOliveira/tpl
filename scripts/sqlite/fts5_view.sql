-- The fixture database of FR-LITE-038 holding an fts5 table, a view over it
-- that compiles, and no view that does not: the database over which
-- FR-LITE-018 sees whether a statement connects a virtual table or compiles a
-- view. Connecting the fts5 table, whether a statement names it or compiles
-- the view, makes the module run a statement of its own against the file,
-- which the statement record reports. It is built from this text at test
-- time, by tests/support/sqlite.rs.

CREATE TABLE consignment (
    consignment_id INTEGER PRIMARY KEY,
    reference TEXT NOT NULL
);

CREATE VIRTUAL TABLE cargo_note USING fts5 (consignment_ref, body);

CREATE VIEW v_cargo_notes AS
    SELECT consignment_ref, body FROM cargo_note;

INSERT INTO consignment (consignment_id, reference) VALUES
    (40211, 'BKG-2026-40211'),
    (40212, 'BKG-2026-40212');

INSERT INTO cargo_note (consignment_ref, body) VALUES
    ('BKG-2026-40211', 'reefer container, pre-cooled to 4 C before stuffing'),
    ('BKG-2026-40212', 'dry van, fumigation certificate attached');
