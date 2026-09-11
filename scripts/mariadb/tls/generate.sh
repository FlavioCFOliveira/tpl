#!/bin/sh
# Regenerate the TLS material the `tpl` MariaDB fixture serves.
#
# Produces, beside this script:
#
#   ca.pem          the root certificate, and the file a test passes as `ca_file`
#   server-cert.pem the leaf the server presents, naming the loopback host
#   server-key.pem  its private key
#
# The certificate's contents come from `openssl.cnf`; only the two distinguished
# names and the validity period are set here. The root's private key is written
# to a temporary file and destroyed once the leaf is signed: the fixture never
# needs to sign anything again, and a root key that does not exist cannot mint a
# second certificate that the committed `ca.pem` would vouch for.
#
# Regeneration is not byte-reproducible — each run makes fresh keys, a fresh
# root and fresh dates. What the repository reproduces is the material's
# meaning: the same names, the same key type and the same extensions, every run.
#
# Requires `openssl` (OpenSSL 1.1.1 or later, or LibreSSL 3.1 or later).
# Rebuild the images afterwards; the material is copied in at build time.

set -eu

cd "$(dirname "$0")"

days=3650
tmp_ca_key=$(mktemp)
trap 'rm -f "$tmp_ca_key"' EXIT INT TERM

openssl req -x509 -new -nodes \
    -newkey rsa:2048 -sha256 -days "$days" \
    -config openssl.cnf -extensions v3_ca \
    -subj "/O=tpl MariaDB test fixture/CN=tpl fixture root CA" \
    -keyout "$tmp_ca_key" -out ca.pem

openssl req -new -nodes \
    -newkey rsa:2048 -sha256 \
    -config openssl.cnf \
    -subj "/O=tpl MariaDB test fixture/CN=localhost" \
    -keyout server-key.pem -out server-req.pem

openssl x509 -req -in server-req.pem -days "$days" -sha256 \
    -CA ca.pem -CAkey "$tmp_ca_key" -set_serial 2 \
    -extfile openssl.cnf -extensions v3_server \
    -out server-cert.pem

rm -f server-req.pem
chmod 0644 ca.pem server-cert.pem
chmod 0600 server-key.pem

echo
echo "Regenerated. The leaf names:"
openssl x509 -in server-cert.pem -noout -ext subjectAltName
