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
