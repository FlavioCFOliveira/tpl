# Test fixture image for `tpl`.
#
# A single Dockerfile parameterised by MariaDB release series. Build it once per
# supported series (see README.md); the DDL and the seed data are identical for
# all of them, which is what makes a difference between two series attributable
# to the server rather than to the fixture.
#
# The two SQL files are renamed on copy. The official entrypoint executes
# everything in /docker-entrypoint-initdb.d in the shell's collation order, and
# "seed.sql" sorts before "setup.sql"; the numeric prefixes force the intended
# order instead of relying on the names being lucky.

ARG MARIADB_SERIES=11.4
FROM mariadb:${MARIADB_SERIES}

# Repeated after FROM: an ARG declared before the first FROM is out of scope in
# the build stage, and the label below would otherwise be empty.
ARG MARIADB_SERIES=11.4

LABEL org.opencontainers.image.title="tpl MariaDB fixture" \
      org.opencontainers.image.description="freight schema used by tpl's catalogue tests" \
      org.opencontainers.image.vendor="tpl" \
      tpl.mariadb.series="${MARIADB_SERIES}"

COPY setup.sql /docker-entrypoint-initdb.d/01-setup.sql
COPY seed.sql  /docker-entrypoint-initdb.d/02-seed.sql

# TLS. `FR-CONF-013` defaults `tls` to `verify-identity`, so the fixture has to
# be able to present a certificate that names the host the tests reach it by.
# The material is produced by tls/generate.sh and copied in here, which is what
# makes it the same certificate on all four series: 10.11 offers no TLS at all
# without it, and on 11.4 and later it replaces the certificate the server
# generates by itself, which carries no subjectAltName and no name can match.
#
# The key is copied with an owner because COPY would otherwise land it as
# root:root, and the server, which drops to the mysql user, could not read a
# 0600 file it does not own. The two certificates are public and keep 0644.
COPY tls/ca.pem tls/server-cert.pem /etc/mysql/tls/
COPY --chown=mysql:mysql tls/server-key.pem /etc/mysql/tls/server-key.pem
COPY tls/server-tls.cnf /etc/mysql/conf.d/90-tls.cnf
