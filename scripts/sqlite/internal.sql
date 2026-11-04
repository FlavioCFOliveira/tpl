-- The fixture database of FR-LITE-038 whose only objects are the internal
-- tables: sqlite_sequence and sqlite_stat1 survive the table and the index
-- that brought them into being. It is built from this text at test time, by
-- tests/support/sqlite.rs.

CREATE TABLE booking_draft (
    draft_id INTEGER PRIMARY KEY AUTOINCREMENT,
    reference TEXT NOT NULL
);

CREATE INDEX ix_booking_draft_reference ON booking_draft (reference);

INSERT INTO booking_draft (reference) VALUES
    ('DRAFT-2026-0001'),
    ('DRAFT-2026-0002');

ANALYZE;

DROP TABLE booking_draft;
