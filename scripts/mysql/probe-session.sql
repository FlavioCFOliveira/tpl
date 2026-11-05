-- The connection-start sequence of FR-MYSQL-017, issued by a substitute client.
--
-- FR-MYSQL-043 requires the closed statement list of FR-MYSQL-015 to be
-- verified by observing what the server actually receives. This file is what
-- the statement instrument is demonstrated against: a MySQL client reading it
-- sends the four kinds of statement the list admits, with the three
-- connection-start statements once each in the order FR-MYSQL-017 fixes — the
-- read-only session statement, its read-back, then the version probe — and a
-- catalogue read after them.
--
-- It is a stand-in for `tpl`, not a specification of it: FR-MYSQL-015 fixes
-- the kinds and the variable the read-back names, and leaves the text of every
-- statement to the implementation. The client under test is `tpl`; the
-- instrument is the same either way.
--
-- Run it under observation:
--     ./observe.sh statements on 8.4
--     docker exec -i -e MYSQL_PWD=tpl-reader-pw tpl-mysql-8.4 mysql --skip-comments \
--       --ssl-mode=VERIFY_IDENTITY --ssl-ca=/etc/mysql/tls/ca.pem \
--       -h 127.0.0.1 -P 3306 -u tpl_reader < probe-session.sql
--     ./observe.sh statements off 8.4
--     ./observe.sh statements dump 8.4
--
-- --skip-comments is not optional. The 8.4 and 9.7 clients forward every
-- comment line of this file to the server as a statement of its own, so
-- without it every comment line becomes a row of the dump, and a comment line
-- that names INFORMATION_SCHEMA is counted by --catalogue. The 8.0 client
-- strips them either way. Every client of the three also sends
-- `select @@version_comment limit 1` on connecting, which is the client's and
-- not this file's.

-- 1. The read-only session statement (FR-MYSQL-018). Once, at connection
--    start, and before the probe.
SET SESSION TRANSACTION READ ONLY;

-- 2. The read-back (FR-MYSQL-018), reading the session read-only state and
--    nothing else. @@session.transaction_read_only is the one spelling MySQL
--    has: tx_read_only was removed in 8.0.
SELECT @@session.transaction_read_only;

-- 3. The server version probe (FR-MYSQL-006). Once, after the read-only pair.
SELECT VERSION();

-- 4. A catalogue read: SELECT against one of the twelve INFORMATION_SCHEMA
--    tables of FR-MYSQL-015.
SELECT COUNT(*) FROM INFORMATION_SCHEMA.TABLES WHERE TABLE_SCHEMA = 'freight';
