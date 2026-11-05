-- The fixture database of FR-LITE-038 holding a table with two columns whose
-- names differ only in a byte that is not valid UTF-8 (FR-LITE-043, its
-- second set). A legacy tool wrote two column names of a customs table in
-- Latin-1: d\xE9signation (é) and d\xC9signation (É), neither of which is
-- UTF-8, so both convert to the one text d�signation under FR-OUT-017.
-- SQLite holds them as two columns. A statement of this text can only give
-- UTF-8, so the names are written under writable_schema, by a connection whose
-- defensive mode is off and which sets none of the controls of FR-LITE-010.
-- A read of it ends with 78. It is built from this text at test time, by
-- tests/support/sqlite.rs.

CREATE TABLE customs_line (
    line_id INTEGER PRIMARY KEY,
    designation_lower TEXT NOT NULL,
    designation_upper TEXT
);

INSERT INTO customs_line (line_id, designation_lower, designation_upper)
VALUES (1, 'poires Rocha', 'POIRES ROCHA');

PRAGMA writable_schema = ON;
UPDATE sqlite_schema SET
    sql = replace(replace(sql, 'designation_lower', 'd' || CAST(x'E9' AS TEXT) || 'signation'),
                  'designation_upper', 'd' || CAST(x'C9' AS TEXT) || 'signation')
WHERE type = 'table' AND name = 'customs_line';
PRAGMA writable_schema = OFF;
