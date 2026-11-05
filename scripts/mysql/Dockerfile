# Test fixture image for `tpl`'s `mysql` provider.
#
# A single Dockerfile parameterised by the upstream image. Build it once per
# series of `series.env` (see README.md); the DDL and the seed data are
# identical for all of them, which is what makes a difference between two series
# attributable to the server rather than to the fixture.
#
# MYSQL_IMAGE is the whole upstream reference, tag and digest, exactly as
# `series.env` pins it: the tag names the release and the digest the build.
#
# The two SQL files are renamed on copy. The official entrypoint executes
# everything in /docker-entrypoint-initdb.d in the shell's collation order, and
# "seed.sql" sorts before "setup.sql"; the numeric prefixes force the intended
# order instead of relying on the names being lucky.

ARG MYSQL_IMAGE=mysql:8.4.13@sha256:a81d043930e51745522bd5c430961cf8286f0a23f0a9a2a6e0108f7ea71b0902
FROM ${MYSQL_IMAGE}

# Repeated after FROM: an ARG declared before the first FROM is out of scope in
# the build stage, and the labels below would otherwise be empty.
ARG MYSQL_IMAGE
ARG MYSQL_SERIES

LABEL org.opencontainers.image.title="tpl MySQL fixture" \
      org.opencontainers.image.description="freight schema used by tpl's mysql provider tests" \
      org.opencontainers.image.vendor="tpl" \
      tpl.mysql.series="${MYSQL_SERIES}" \
      tpl.mysql.base="${MYSQL_IMAGE}"

COPY setup.sql /docker-entrypoint-initdb.d/01-setup.sql
COPY seed.sql  /docker-entrypoint-initdb.d/02-seed.sql

# TLS. `FR-CONF-013` defaults `tls` to `verify-identity`, so the fixture has to
# present a certificate that names the host the tests reach it by. A MySQL
# server generates a certificate of its own at initialisation, and it names no
# such host; the material of tls/ replaces it, and is the same certificate on
# every series.
#
# The key is copied with an owner because COPY would otherwise land it as
# root:root, and the server, which drops to the mysql user, could not read a
# 0600 file it does not own. The two certificates are public.
COPY tls/ca.pem tls/server-cert.pem /etc/mysql/tls/
COPY --chown=mysql:mysql tls/server-key.pem /etc/mysql/tls/server-key.pem
COPY tls/server-tls.cnf /etc/mysql/conf.d/90-tls.cnf
