-- The connection-start sequence of FR-SRV-006, issued by a substitute client.
--
-- `tpl` does not exist yet, and FR-SRV-012 requires its closed statement list
-- to be checked by observing what the server actually receives. This file is
-- what the observation is demonstrated against in the meantime: a MariaDB
-- client reading it sends exactly the four kinds the list admits, with the
-- three connection-start statements once each in the order the list states.
--
-- It is a stand-in for `tpl`, not a specification of it. When `tpl` exists the
-- instrument stays and the client changes.
--
-- Run it under observation:
--     ./observe.sh statements on 11.8
--     docker exec -i tpl-mariadb-11.8 mariadb --ssl-ca=/etc/mysql/tls/ca.pem \
--       --ssl-verify-server-cert -h 127.0.0.1 -P 3306 \
--       -u tpl_reader -ptpl-reader-pw < probe-session.sql
--     ./observe.sh statements off 11.8
--     ./observe.sh statements dump 11.8

-- 1. The server version probe (FR-SRV-002). Once, at connection start.
SELECT VERSION();

-- 2. The read-only session statement (FR-SRV-008). Once, immediately after.
SET SESSION TRANSACTION READ ONLY;

-- 3. The read-back (FR-SRV-009), reading the session read-only state and
--    nothing else. Once, immediately after the statement it confirms.
--
--    @@session.tx_read_only is the spelling, and the only one of the two that
--    every supported series has: 10.11 answers ERROR 1193 (HY000) Unknown
--    system variable 'transaction_read_only', while 11.4, 11.8 and 12.3 accept
--    both spellings. See README.md, "Differences observed between the series".
SELECT @@session.tx_read_only;

-- 4. A catalogue read: SELECT against INFORMATION_SCHEMA, as the command
--    requires.
SELECT COUNT(*) FROM INFORMATION_SCHEMA.TABLES WHERE TABLE_SCHEMA = 'freight';
