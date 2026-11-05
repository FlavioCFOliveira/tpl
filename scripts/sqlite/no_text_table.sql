-- The fixture database of FR-LITE-038 whose schema table holds, for an index
-- the database holds, a second row with no text — its sql NULL — whose type
-- is table (FR-LITE-041, its fifth kind). The library reads every row with no
-- text as an index's, finds the index the row names and loads it, while every
-- statement that selects the rows whose type is table reads it as a table:
-- a table with no column, which the file does not hold. SQLite writes no such
-- row, so it is written under writable_schema, by a connection whose
-- defensive mode is off and which sets none of the controls of FR-LITE-010. A
-- read of it ends with 78. It is built from this text at test time, by
-- tests/support/sqlite.rs.

CREATE TABLE berth (
    berth_no INTEGER PRIMARY KEY,
    port_code CHAR(5) NOT NULL,
    max_draught_m REAL
);

CREATE INDEX ix_berth_port ON berth (port_code);

INSERT INTO berth (berth_no, port_code, max_draught_m) VALUES (1, 'PTSIE', 17.5), (2, 'NLRTM', 24.0);

PRAGMA writable_schema = ON;
INSERT INTO sqlite_schema (type, name, tbl_name, rootpage, sql)
    SELECT 'table', name, tbl_name, rootpage, NULL
    FROM sqlite_schema WHERE type = 'index' AND name = 'ix_berth_port';
PRAGMA writable_schema = OFF;
