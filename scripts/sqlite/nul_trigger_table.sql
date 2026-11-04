-- The fixture database of FR-LITE-038 whose schema table holds a trigger
-- whose tbl_name is its table's name followed by a NUL character
-- (FR-LITE-041, its sixth kind). The library compares the tbl_name with the
-- trigger's text only up to the NUL and loads the row, while the read, which
-- takes the whole value, matched it to no table and would have dropped the
-- trigger at exit 0. SQLite writes no such value, so it is written under
-- writable_schema, by a connection whose defensive mode is off and which sets
-- none of the controls of FR-LITE-010. A read of it ends with 78. It is built
-- from this text at test time, by tests/support/sqlite.rs.

CREATE TABLE port (
    port_code CHAR(5) PRIMARY KEY,
    port_name TEXT NOT NULL
);

CREATE TRIGGER trg_port_code_length BEFORE INSERT ON port
BEGIN
    SELECT RAISE(ABORT, 'a port code has five characters') WHERE length(NEW.port_code) <> 5;
END;

PRAGMA writable_schema = ON;
UPDATE sqlite_schema SET tbl_name = tbl_name || char(0) WHERE type = 'trigger' AND name = 'trg_port_code_length';
PRAGMA writable_schema = OFF;
