-- The fixture database of FR-LITE-038 whose schema holds a view named
-- pragma_table_list: SQLite resolves the name among the file's tables and
-- views before the pragma table-valued function of that name, so the view
-- would take the function's place (FR-LITE-041, its first kind). A read of it
-- ends with 78. It is built from this text at test time, by
-- tests/support/sqlite.rs.

CREATE TABLE consignment (
    consignment_id INTEGER PRIMARY KEY,
    reference TEXT NOT NULL
);

CREATE VIEW pragma_table_list AS
    SELECT 'main' AS schema, 'forged' AS name, 'table' AS type, 0 AS ncol, 0 AS wr, 0 AS strict;

INSERT INTO consignment (consignment_id, reference) VALUES
    (40211, 'BKG-2026-40211'),
    (40212, 'BKG-2026-40212');
