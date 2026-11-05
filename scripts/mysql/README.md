# MySQL test fixtures

The MySQL servers `tpl`'s `mysql` provider is validated against, and the
harness that drives and instruments them. `FR-MYSQL-044`, in
`specification/mysql-contract.md`, obliges this directory to exist and fixes
what it holds; the harness has the contract of the MariaDB fixture's in
`scripts/mariadb/`: scripts that start the fixture and verify it server by
server, stop it and prove that nothing of it is left running, answer whether it
is up before a server-dependent test runs, and instrument the observations the
specification requires to be made outside the process.

Every output shown below was produced by a command that was run against this
fixture on 2026-11-04, on an Apple Silicon host, from the `linux/arm64` images.
This file records what the fixture **is** and how it is operated. The
observations of MySQL's behaviour that the specification marks *Pending
observation* belong to rmp `#326`, which takes them through this fixture; the
few made while building it are listed under
[Observed while building the fixture](#observed-while-building-the-fixture) so
that they are not lost, and none of them is a substitute for that work.

## Contents

| File | Purpose |
|---|---|
| `Dockerfile` | One image definition, parameterised by the upstream image |
| `setup.sql` | The `freight`, `masterdata` and `freight_empty` schemas: DDL only |
| `seed.sql` | The data, plus the statements that make the triggers and the stored procedure actually run |
| `tls/generate.sh` | Regenerates the TLS material below |
| `tls/openssl.cnf` | The certificate profile: what the certificate says, including the names it carries |
| `tls/ca.pem` | The fixture's root certificate, and the file to pass as `ca_file` |
| `tls/server-cert.pem` | The certificate the servers present |
| `tls/server-key.pem` | Its private key |
| `tls/server-tls.cnf` | The three server settings that put the material into service |
| `series.env` | The inventory — the pinned images and one record per server — and the helpers the scripts share |
| `up.sh` | Starts every server and does not return until each is listening and verified |
| `down.sh` | Stops and removes them, and proves nothing of the fixture is left |
| `status.sh` | The gate: whether the fixture is up, answered without a client |
| `observe.sh` | The two server-side instruments of `NFR-PERF-007`, and the reading of the conditions a reading depends on |
| `auth-cache.sh` | Empties or fills a server's `caching_sha2_password` cache, which decides the outcome `FR-MYSQL-045` describes |
| `probe-session.sql` | The connection-start sequence of `FR-MYSQL-017`, for a substitute client |
| `README.md` | This file |

The Dockerfile copies `setup.sql` to `/docker-entrypoint-initdb.d/01-setup.sql`
and `seed.sql` to `02-seed.sql`. The rename is load-bearing: the official
entrypoint runs that directory in collation order, and `seed.sql` sorts before
`setup.sql`. It copies the three `tls/` certificate files to `/etc/mysql/tls/`
and `tls/server-tls.cnf` to `/etc/mysql/conf.d/90-tls.cnf`; see [TLS](#tls).

## The servers

The supported series are fixed by `FR-MYSQL-003`: `8.4` and `9.7`, the two MySQL
LTS series under Oracle Premier or Extended Support. `FR-MYSQL-044` adds two
more kinds of server beside them — one that offers no TLS, and one of a series
outside `FR-MYSQL-003` and older than its newest, for the refusal of
`FR-MYSQL-009`. Five servers, a container and a published port each:

| Server | Series | Upstream image | Image tag | Container | Host port | TLS | Standing |
|---|---|---|---|---|---|---|---|
| `8.4` | `8.4` | `mysql:8.4.13` | `tpl-mysql:8.4` | `tpl-mysql-8.4` | `13320` | yes | supported |
| `9.7` | `9.7` | `mysql:9.7.4` | `tpl-mysql:9.7` | `tpl-mysql-9.7` | `13321` | yes | supported |
| `notls-8.4` | `8.4` | `mysql:8.4.13` | `tpl-mysql:8.4` | `tpl-mysql-notls-8.4` | `13322` | no | supported |
| `notls-9.7` | `9.7` | `mysql:9.7.4` | `tpl-mysql:9.7` | `tpl-mysql-notls-9.7` | `13323` | no | supported |
| `8.0` | `8.0` | `mysql:8.0.46` | `tpl-mysql:8.0` | `tpl-mysql-8.0` | `13324` | yes | refused |

**The ports are `13320` to `13324`**, a contiguous block clear of `13306` to
`13310`, which the MariaDB fixture publishes, so both fixtures run side by side.

**Each upstream image is pinned by tag and by digest** in `series.env`. The tag
names the release — each supported series at its latest release on 2026-11-04,
the date `FR-MYSQL-003` was verified — and the digest names the build: the
official `mysql` tags are rebuilt in place when their base image changes, so a
tag alone would let two runs observe two builds. The digests are the
multi-architecture index digests Docker Hub reported for the three tags on
2026-11-04; each covers `linux/amd64` and `linux/arm64`, so the fixture runs
unmodified on both supported architectures.

**Why a no-TLS server at each supported series.** `FR-MYSQL-044` requires the
fixture to retain *a* server that offers no TLS. `FR-MYSQL-021` owes every cell
of its mode table an observation against servers of every series of
`FR-MYSQL-003`, and `FR-MYSQL-045` owes the logins under `preferred` against the
server offering no TLS an observation on every series; one no-TLS server could
serve only one series. Two servers satisfy both readings. They are not further
series: each runs its series' image with one extra argument.

**Why `8.0`.** `FR-MYSQL-044` asks for a series outside `FR-MYSQL-003` and older
than its newest, and names none. `8.0` is the series `FR-MYSQL-009`'s own
example refuses, and `8.0.46` is its last release. It presents the fixture
certificate, so that a `mysql` entry reaches the version check under the default
`tls` mode instead of failing on TLS first, and it carries the same schemas: the
shared DDL loads on it unchanged, which nothing requires but which lets `up.sh`
verify it like the others.

**The data directory is a tmpfs**, not the anonymous volume the image declares.
Nothing in it has to outlive the container — the entrypoint builds it from
`setup.sql` and `seed.sql` on every start — so in memory it leaves no volume
behind to be mistaken for a fresh schema, and it costs the Docker host's disk
nothing. That matters: a loaded server's data directory measured 214 MB, of
which 101 MB is the redo log MySQL pre-allocates, so five servers on volumes
need over a gigabyte of disk. `up.sh` bounds each tmpfs at 1 GB; a loaded server
uses 206 MB of it.

## Building

`up.sh` builds a missing image itself. By hand, one build per series, with the
reference `series.env` pins:

```sh
cd scripts/mysql
. ./series.env
for s in 8.4 9.7 8.0; do
  docker build --build-arg "MYSQL_IMAGE=$(tpl_mysql_base_image $s)" \
    --build-arg "MYSQL_SERIES=$s" -t "tpl-mysql:$s" .
done
```

Editing `setup.sql` or `seed.sql` requires a rebuild **and** a fresh container:
the entrypoint runs the init directory only when the data directory is empty.
`./down.sh --images && ./up.sh` does both. No preparatory step is needed: the
TLS material the build copies in is committed.

## The harness

| File | What it is |
|---|---|
| `series.env` | The inventory and the shared helpers. Sourced, never run |
| `up.sh` | Starts the servers and verifies each one |
| `down.sh` | Removes them and proves nothing is left |
| `status.sh` | The gate |
| `observe.sh` | The two instruments, and the conditions reading |
| `auth-cache.sh` | Sets the authentication cache before a test that depends on it |
| `probe-session.sql` | A substitute client's connection-start sequence |

`series.env` carries the inventory the table under [The servers](#the-servers)
states in prose, in the form the scripts read. The scripts read nothing else, so
a port changes in one executable place; the prose table is the second copy, and
it has to be changed with it. Its iterator selects servers by role, which is how
a test runner should ask for them rather than by name:

```sh
. ./series.env
emit() { printf '%s ' "$1"; }
for f in tls notls supported refused all; do printf '%-9s ' $f; tpl_mysql_each emit $f; echo; done
```
```
tls       8.4 9.7
notls     notls-8.4 notls-9.7
supported 8.4 9.7 notls-8.4 notls-9.7
refused   8.0
all       8.4 9.7 notls-8.4 notls-9.7 8.0
```

`tls` is the series of `FR-MYSQL-003`, one server each — the set an assertion
that holds "on every series" iterates. An unknown filter exits `2`.

### Bringing it up

```sh
cd scripts/mysql
./up.sh                    # all five
./up.sh 9.7 notls-9.7      # only the named servers
```

It is idempotent: it builds an image only when it is missing and starts a
container only when it is not already running. With the images built and
nothing running:

```
8.4
  image   tpl-mysql:8.4 already built
  server  tpl-mysql-8.4 started on :13320
  listen  tpl-mysql-8.4 answers on :13320 as 8.4.13
  ready   tpl-mysql-8.4: freight tables=22 views=5 routines=7 triggers=7 events=1; masterdata tables=2; freight_empty objects=0; TLS enabled=Yes; no [ERROR] in log
9.7
  image   tpl-mysql:9.7 already built
  server  tpl-mysql-9.7 started on :13321
  listen  tpl-mysql-9.7 answers on :13321 as 9.7.4
  ready   tpl-mysql-9.7: freight tables=22 views=5 routines=7 triggers=7 events=1; masterdata tables=2; freight_empty objects=0; TLS enabled=Yes; no [ERROR] in log
notls-8.4
  image   tpl-mysql:8.4 already built
  server  tpl-mysql-notls-8.4 started on :13322 (--tls-version=)
  listen  tpl-mysql-notls-8.4 answers on :13322 as 8.4.13
  ready   tpl-mysql-notls-8.4: freight tables=22 views=5 routines=7 triggers=7 events=1; masterdata tables=2; freight_empty objects=0; TLS enabled=No; no [ERROR] in log
notls-9.7
  image   tpl-mysql:9.7 already built
  server  tpl-mysql-notls-9.7 started on :13323 (--tls-version=)
  listen  tpl-mysql-notls-9.7 answers on :13323 as 9.7.4
  ready   tpl-mysql-notls-9.7: freight tables=22 views=5 routines=7 triggers=7 events=1; masterdata tables=2; freight_empty objects=0; TLS enabled=No; no [ERROR] in log
8.0
  image   tpl-mysql:8.0 already built
  server  tpl-mysql-8.0 started on :13324
  listen  tpl-mysql-8.0 answers on :13324 as 8.0.46
  ready   tpl-mysql-8.0: freight tables=22 views=5 routines=7 triggers=7 events=1; masterdata tables=2; freight_empty objects=0; TLS enabled=Yes; no [ERROR] in log

SERVER     SERIES  PORT   TLS  STANDING   STATE VERSION
8.4        8.4     13320  yes  supported  up    8.4.13
9.7        9.7     13321  yes  supported  up    9.7.4
notls-8.4  8.4     13322  no   supported  up    8.4.13
notls-9.7  9.7     13323  no   supported  up    9.7.4
8.0        8.0     13324  yes  refused    up    8.0.46
```

28 seconds for all five on this host with the three images built, and 28
seconds from nothing, the builds included. Each server is checked three ways,
because no one check is the check:

- `listen` is the published port answering with a MySQL greeting.
- `ready` is the log carrying no `[ERROR]` line, and every figure `series.env`
  states being what the catalogue holds: the three schemas present, the
  `freight` tables, views, routines, triggers and events, the `masterdata`
  tables, and no object in `freight_empty`. A failing statement in the init
  scripts stops the container, which `listen` catches; a statement that is
  lost without failing is what the counts catch.
- `TLS enabled` is read from the server's own channel status
  (`performance_schema.tls_channel_status`, property `Enabled`), and for a TLS
  server `ssl_cert` must name the fixture certificate. A no-TLS server is staged
  by configuration, so the staging is verified rather than assumed.

**Readiness is the published port, never a client inside the container.** The
official entrypoint runs a temporary server with `--skip-networking` while the
init scripts execute: it answers over the Unix socket, and it then goes away to
be replaced by the real one. The published port is up only when the server
under test is, so that is what `up.sh` waits for.

**A server that cannot be made healthy is removed, not left behind.** `up.sh`
prints the container's last 30 log lines and then removes it. An unknown server
name is an error before anything starts.

**A healthy start logs warnings, and none of them is a fault of the fixture.**
On every server: `Insecure configuration for --pid-file` and `root@localhost is
created with an empty password` (the entrypoint's own initialisation). On a TLS
server: `CA certificate /etc/mysql/tls/ca.pem is self signed`, which is what a
root certificate is. On a no-TLS server: `Failed to set up SSL because of the
following SSL library error: TLS version is invalid` and `Failed to initialize
TLS for channel: mysql_main`, which is how MySQL reports a `tls_version` set to
the empty string, and a matching warning from the X Plugin. On `8.0` alone: `The
syntax '--skip-host-cache' is deprecated`, from the image's own `my.cnf`.

### Taking it down

```sh
./down.sh              # all five
./down.sh 8.0          # only the named servers
./down.sh --images     # ...and drop the three images too
```

```
  removed   tpl-mysql-8.4
  removed   tpl-mysql-9.7
  removed   tpl-mysql-notls-8.4
  removed   tpl-mysql-notls-9.7
  removed   tpl-mysql-8.0

removed 5 container(s); docker ps -a lists no tpl-mysql container
```

The last line is not an intention: `down.sh` asks the daemon what is left of the
fixture (`docker ps -a --filter name=tpl-mysql-`) and exits `1` if anything is.
Named servers are a partial teardown: it then checks that every named server is
gone and lists the ones left standing, which is what was asked for. It removes
exactly the containers `series.env` names, because the host may be running
containers of other projects. It is safe to run when nothing is up and safe to
run twice.

### The gate

`status.sh` answers the question a server-dependent test has to ask before it
runs. It probes each published port the way a test reaches it — from the host,
over TCP — and needs **no client installed**: a server sends its greeting the
moment the socket opens, and the greeting carries the version, so what is
reported is "a MySQL of this version answered here", not "something is
listening". The probe reads 40 bytes of the greeting, which is 78 bytes long on
`8.4` and `8.0` and 77 on `9.7`; asking for more than the packet holds would
block until the server gives up on the handshake.

| Exit | Meaning | What a test runner does |
|---|---|---|
| `0` | Every requested server answered | Run the server tests |
| `1` | None answered | Skip them, and say why |
| `2` | Some answered and some did not, or an unknown server was named | A broken fixture, not an absent one. Do **not** skip |

All three were observed. With one of the five removed:

```
SERVER     SERIES  PORT   TLS  STANDING   STATE VERSION
8.4        8.4     13320  yes  supported  up    8.4.13
9.7        9.7     13321  yes  supported  up    9.7.4
notls-8.4  8.4     13322  no   supported  up    8.4.13
notls-9.7  9.7     13323  no   supported  up    9.7.4
8.0        8.0     13324  yes  refused    down  -
exit=2
```

For a runner that needs the answer as data:

```sh
./status.sh --export
```
```
TPL_MYSQL_8_4=127.0.0.1:13320; export TPL_MYSQL_8_4
TPL_MYSQL_9_7=127.0.0.1:13321; export TPL_MYSQL_9_7
TPL_MYSQL_NOTLS_8_4=127.0.0.1:13322; export TPL_MYSQL_NOTLS_8_4
TPL_MYSQL_NOTLS_9_7=127.0.0.1:13323; export TPL_MYSQL_NOTLS_9_7
TPL_MYSQL_8_0=127.0.0.1:13324; export TPL_MYSQL_8_0
TPL_MYSQL_READY=all; export TPL_MYSQL_READY
```

A server's name becomes a variable name by upper-casing it and turning every dot
and hyphen into an underscore. `TPL_MYSQL_READY` is `all`, `none` or `partial`,
matching the three exit codes. `./status.sh --quiet` prints nothing and exits
with the code alone.

**The gate is visible on the server.** One probe opens a TCP connection, reads
the greeting and closes without authenticating, so it adds 1 to `Connections`
and 1 to `Aborted_connects` on each server it asks. Measured on `8.4` around one
run, the second reading's own connection included:

```
before:  Aborted_connects 13   Connections 66
after:   Aborted_connects 14   Connections 68
```

Take a counter baseline **after** the gate, not before it.

## TLS

`FR-MYSQL-044` obliges this fixture to present, at each series of
`FR-MYSQL-003`, a server whose certificate names the host the project's tests
reach it by, and to retain a server that offers no TLS; `FR-MYSQL-021` states
the five modes' behaviour against both. A MySQL server left to itself generates
a self-signed certificate in its data directory at initialisation, and nothing
in the manual says that it names a host, so the fixture carries its own.

### The certificate

| | |
|---|---|
| Names | `DNS:localhost`, `IP:127.0.0.1`, `IP:::1` |
| Subject | `O=tpl MySQL test fixture, CN=localhost` |
| Issuer | `O=tpl MySQL test fixture, CN=tpl fixture root CA`, in `tls/ca.pem` |
| Key | RSA 2048, signed with SHA-256 |
| Validity | 3650 days from generation: 2026-11-04 to 2036-11-01 |

The three names are the three spellings of the loopback a test can write.
`tls/ca.pem` is the file to pass as `ca_file` (`FR-CONF-014`); the private key
that signed it was destroyed at generation time and is not in this repository.
The root is this fixture's own and not the MariaDB fixture's, so trust material
handed to a `mysql` entry vouches for these servers and for nothing else. The
material is committed and is not secret: the fixture's passwords are in this
directory too, and a certificate that names `localhost` is worth nothing
anywhere else.

### In the image

```
/etc/mysql/tls/ca.pem              root:root
/etc/mysql/tls/server-cert.pem     root:root
/etc/mysql/tls/server-key.pem      owned by mysql, the user the server runs as
/etc/mysql/conf.d/90-tls.cnf       ssl_ca, ssl_cert, ssl_key under [mysqld]
```

`require_secure_transport` is deliberately **not** set: the `disabled` and
`preferred` cells of `FR-MYSQL-021` against a server offering TLS expect a
plaintext connection to be accepted.

### The servers that offer no TLS

`--tls-version=` — `tls_version` set to the empty string — is how the MySQL
manual starts a server "with encrypted connections disabled on the main
interface", and it wins over the certificate `90-tls.cnf` configures. MySQL
8.4 and 9.7 have no `have_ssl` variable, so the state is read from the channel
status, which reports `Enabled No` while `Current_tls_cert` still names the
configured file.

### Verifying it

Chain and host name, from the host, with `openssl`:

```sh
cd scripts/mysql/tls
for p in 13320 13321 13324; do
  openssl s_client -starttls mysql -connect 127.0.0.1:$p \
    -CAfile ca.pem -verify_return_error -verify_hostname localhost -brief </dev/null
done
```

All three answered `CONNECTION ESTABLISHED`, `Protocol version: TLSv1.3`,
`Verification: OK` and `Verified peername: localhost`. The controls that prove
the check is real:

| Control | Result |
|---|---|
| `-verify_hostname db.example.com` against `13320` | `verify error:num=62:hostname mismatch` |
| No `-CAfile`, against `13321` | `verify error:num=19:self-signed certificate in certificate chain` |
| `-CAfile ../../mariadb/tls/ca.pem` against `13320` | `verify error:num=19:self-signed certificate in certificate chain` |
| Any of the above against `13322` or `13323` | `MySQL server does not support SSL.` |

Each series' own client verifies the same way from inside its container, where
`127.0.0.1:3306` is the server and the certificate's `IP:127.0.0.1` matches:

```sh
docker exec -e MYSQL_PWD=tpl-reader-pw tpl-mysql-8.4 mysql \
  --ssl-mode=VERIFY_IDENTITY --ssl-ca=/etc/mysql/tls/ca.pem \
  -h 127.0.0.1 -P 3306 -u tpl_reader -N -B \
  -e "SELECT VARIABLE_VALUE FROM performance_schema.session_status WHERE VARIABLE_NAME='Ssl_cipher'"
```

It answered `TLS_AES_128_GCM_SHA256` on `8.4`. `Ssl_cipher` is read from the live
session, so a non-empty value is the session's own evidence that it is
encrypted.

### Regenerating

```sh
cd scripts/mysql
tls/generate.sh
./down.sh --images && ./up.sh
```

`tls/generate.sh` needs `openssl` and nothing else, takes everything the
certificate says from `tls/openssl.cnf`, makes a root, signs the leaf with it,
and destroys the root's private key. Regeneration is not byte-reproducible; what
it reproduces is the material's meaning.

## Accounts and authentication

| User | Host | Password | Privileges |
|---|---|---|---|
| `root` | `%` | `tpl-root` | Everything. Created by the entrypoint from `MYSQL_ROOT_PASSWORD`; the privileged reader a test reaches over TCP |
| `root` | `localhost` | `tpl-root` | The same account name for the Unix socket, which is how the harness reaches a server |
| `tpl_reader` | `%` | `tpl-reader-pw` | `SELECT, EXECUTE ON freight.*` and on the target database of item 17, ``tpl_q'b\s`k``, and nothing more: the reduced-grant reader of `FR-MYSQL-044` item 16 |
| `tpl_nopass` | `%` | none | `SELECT, EXECUTE ON freight.*`: an account with an empty credential, for `FR-CONF-055` and `FR-MYSQL-047` |
| `tpl_limited` | `%` | `tpl-limited-pw` | `SELECT ON freight.*`, `MAX_USER_CONNECTIONS 1`: the second simultaneous login is refused with a limit on connections, for `FR-ERR-048` |

The names and passwords of `root` and `tpl_reader` are the variables
`TPL_MYSQL_ROOT_USER`, `TPL_MYSQL_ROOT_PASSWORD`, `TPL_MYSQL_READER_USER` and
`TPL_MYSQL_READER_PASSWORD` of `series.env`, which `tests/support/mysql.rs`
reads; no test writes an account name of its own.

`setup.sql` creates `tpl_reader` without naming an authentication plugin, so the
account takes the server's default, which is the method `FR-MYSQL-022` requires
`tpl` to authenticate with. `mysql.user` reports `caching_sha2_password` for all
three accounts on `8.4`, `9.7` and `8.0`. The reader sees 27 objects in
`freight` — its 22 tables and 5 views — on every server.

`SHOW DATABASES`, as `tpl_reader`, lists `freight`, `information_schema`,
`performance_schema` and ``tpl_q'b\s`k`` and nothing else, on `8.4`, `9.7`
and `8.0`; `mysql.db` holds the grant's database part as ``tpl\_q'b\\s`k``,
bytes `74706C5C5F7127625C5C73606B`. Observed on 2026-11-05.

The five databases of item 17 are not counted by `up.sh`, whose figures are
those of the three schemas above. As root, `information_schema.SCHEMATA`
holds twelve rows on each server, and the five names are distinct byte
sequences under `lower_case_table_names` `0`: the target
`74706C5F7127625C73606B`, ``tpl_q'bs`k`` `74706C5F71276273606B`,
``tpl_q'b\\s`k`` `74706C5F7127625C5C73606B`, `tpl_q_b_s_k`
`74706C5F715F625F735F6B` and ``TPL_Q'B\S`K`` `54504C5F5127425C53604B`.
Observed on 2026-11-05 on `8.4`, `9.7` and `8.0`.

### The cache, and `auth-cache.sh`

`FR-MYSQL-045` forbids `tpl` to request a server's RSA public key over a session
TLS does not protect. Whether a login needs that key depends on the server's
`caching_sha2_password` cache and on nothing in the entry, so a test of either
outcome has to put the cache in a known state first:

```sh
./auth-cache.sh flush <server>              # FLUSH PRIVILEGES: the cache holds no account
./auth-cache.sh prime <server> [<user>]     # one login of tpl_reader (or root) as user@'%'
```

`prime` logs in over TCP from inside the container, to `127.0.0.1`, so that the
login is the `%` account a test from the host uses. On a TLS server the login is
encrypted; on a no-TLS server it requests the server's public key, which is the
harness filling a cache, and precisely what `tpl` must not do.

**After `./up.sh` the cache holds no entry for `root@'%'` or
`tpl_reader@'%'`**: the entrypoint and every harness script connect as
`root@'localhost'` over the socket. Two things fill an entry besides
`auth-cache.sh prime`: any successful login over TLS, and a login as
`tpl_reader` over the socket — `tpl_mysql_sql_as <server> tpl_reader …` in
`series.env` — which the server takes as `tpl_reader@'%'`, a secure transport,
and caches. Observed on `8.4`: after `flush`, a `DISABLED` login over TCP failed
with `ERROR 2061`; after one socket login, the same TCP login succeeded. A test
of the uncached outcome therefore flushes first, whatever ran before it.

The matrix the specification describes, reproduced with the stock `mysql` client
as a substitute — it requests no public key unless told to — logging in as
`tpl_reader` over TCP, once after `flush` and once after `prime`:

| Server | Client mode | Cache empty | Cache filled |
|---|---|---|---|
| `8.4`, `9.7` | `DISABLED` | `ERROR 2061 (HY000): Authentication plugin 'caching_sha2_password' reported error: Authentication requires secure connection.` | logged in, `Ssl_cipher` empty |
| `8.4`, `9.7` | `PREFERRED` | logged in, `Ssl_cipher=TLS_AES_128_GCM_SHA256` | the same |
| `notls-8.4`, `notls-9.7` | `DISABLED` | `ERROR 2061`, as above | logged in, `Ssl_cipher` empty |
| `notls-8.4`, `notls-9.7` | `PREFERRED` | `ERROR 2061`, as above | logged in, `Ssl_cipher` empty |

Identical on both series. The rows with an empty cache are the logins that
`FR-MYSQL-045` turns into exit `78` for `tpl`.

## The observations made outside the process

`NFR-PERF-007` forbids verifying a requirement of form by reading the source, and
`BR-SRV-003` says why. `FR-MYSQL-042` holds a `mysql` entry to `NFR-PERF-001`,
`002`, `003`, `004`, `006` and `008`, verified from the server's records on
every series of `FR-MYSQL-003`, and `FR-MYSQL-043` adds `FR-SRV-012` to `014` for
the closed statement list, the read-back and the connection count.
`FR-MYSQL-044` obliges this fixture to instrument the two records a server
keeps — the statements it receives and the connections it accepts — and those
are the two instruments here. The syscall tracer of the MariaDB fixture is not:
no requirement a `mysql` entry carries is observed through it.

| Requirement | The property | Instrument |
|---|---|---|
| `NFR-PERF-001` | No query per object on a full read, compared over `freight` and `freight_empty` | statements |
| `NFR-PERF-002` | Reading one named object does not scale with the database | statements |
| `NFR-PERF-003` | A cache hit opens no connection and issues no query | connections and statements |
| `NFR-PERF-004`, `FR-SRV-014` | At most one connection per invocation, seen from the server | connections |
| `NFR-PERF-006` | A command needing no catalogue opens no connection | connections |
| `NFR-PERF-008` | The count is observable | statements |
| `FR-SRV-012` | The closed list of `FR-MYSQL-015`, in the order of `FR-MYSQL-017` | statements |
| `FR-SRV-013` | The read-back of `FR-MYSQL-018`, in its confirming outcome | statements, and the value the session reports |

**`freight_empty` is the database that holds no object** against which
`FR-MYSQL-042` takes the equality of `NFR-PERF-001`. It declares the character
set and collation `freight` declares, so the two differ in nothing but their
objects. `tpl_reader` holds no grant on it: the comparison is a privileged read.

### The statements a server receives

The server's general log, directed to the table `mysql.general_log` so that it
can be queried. Its `event_time` is `timestamp(6)` on every series, so the dump
orders by microsecond.

```sh
./observe.sh statements on   8.4     # log_output=TABLE, empty the log, general_log=ON
./observe.sh statements off  8.4
./observe.sh statements dump 8.4     # filters: --all --user <u> --kind <type> --queries --catalogue --count
```

`dump` drops connections made over the Unix socket, which is how the harness
reaches a server and not how anything under test reaches it; `--all` keeps
them. `--catalogue` keeps the `Query` and `Execute` rows naming
`INFORMATION_SCHEMA` and drops `Prepare`, which registers a statement text
rather than issuing it; `observe.sh` records why. The log's spelling of users
and transports is the MariaDB fixture's, which is what lets one filter serve
both: a socket connection is `root[root] @ localhost []`, and a `Connect` row
reads `using SSL/TLS`, `using TCP/IP` or `using Socket`, so the instrument also
answers, per connection and from the server side, whether the session was
encrypted.

`probe-session.sql` issues the four kinds of statement `FR-MYSQL-015` admits,
the three connection-start statements in the order `FR-MYSQL-017` fixes, then
a catalogue read:

```sh
./observe.sh statements on 8.4
docker exec -i -e MYSQL_PWD=tpl-reader-pw tpl-mysql-8.4 mysql --skip-comments \
  --ssl-mode=VERIFY_IDENTITY --ssl-ca=/etc/mysql/tls/ca.pem \
  -h 127.0.0.1 -P 3306 -u tpl_reader < probe-session.sql
./observe.sh statements off 8.4
./observe.sh statements dump 8.4
./observe.sh statements dump 8.4 --catalogue --count
```
```
thread_id	command_type	statement
23	Connect	tpl_reader@127.0.0.1 on  using SSL/TLS
23	Query	select @@version_comment limit 1
23	Query	SET SESSION TRANSACTION READ ONLY
23	Query	SELECT @@session.transaction_read_only
23	Query	SELECT VERSION()
23	Query	SELECT COUNT(*) FROM INFORMATION_SCHEMA.TABLES WHERE TABLE_SCHEMA = 'freight'
23	Quit
1
```

Identical on all five servers, but for the thread id and for `using TCP/IP` on
the two no-TLS servers. The read-back answered `1` on every one. Two things in
the dump are the substitute client's and not the probe's, and would not be in a
dump of `tpl`: every client of the three series sends `select @@version_comment
limit 1` on connecting, and the `8.4` and `9.7` clients forward each comment
line of a script to the server as a statement of its own unless run with
`--skip-comments` — a comment line of the probe names `INFORMATION_SCHEMA`,
and `--catalogue` counted `2` rather than `1` until the flag was added. The
`8.0` client strips comments either way.

### The connections a server accepts

```sh
./observe.sh connections 9.7
./observe.sh connections 9.7 --value      # Connections alone, for arithmetic
```
```
Variable_name	Value
Aborted_connects	2
Connections	31
Max_used_connections	1
Threads_connected	1
```

`Connections` is monotonic, so a test brackets the invocation under test with
two readings and subtracts. The value is read from
`performance_schema.global_status`; MySQL has no
`information_schema.GLOBAL_STATUS`, where the MariaDB fixture reads it. **The
observer's baseline is 1**, measured on each server as two consecutive readings
with nothing between them:

```
  8.4        two readings 32 -> 33, baseline delta=1
  9.7        two readings 29 -> 30, baseline delta=1
  notls-8.4  two readings 28 -> 29, baseline delta=1
  notls-9.7  two readings 28 -> 29, baseline delta=1
  8.0        two readings 32 -> 33, baseline delta=1
```

With three client connections between two readings on `8.4`:
`a=34 b=38 delta=4 => attributable to the three client connections: 3`.

### The conditions a reading depends on

`FR-MYSQL-044` requires the conditions each server runs under that a reading
depends on — its TLS material, its `lower_case_table_names` and its
`show_gipk_in_create_table_and_information_schema` — to be recorded with every
reading taken against it. `observe.sh conditions` reads them from the server:

```sh
./observe.sh conditions          # every server
./observe.sh conditions 9.7      # one of them
```
```
SERVER     CONDITION                                         VALUE
9.7        version                                           9.7.4
9.7        tls_enabled                                       Yes
9.7        tls_cert                                          /etc/mysql/tls/server-cert.pem
9.7        tls_cert_not_after                                Nov  1 14:45:36 2036 GMT
9.7        tls_cert_not_before                               Nov  4 14:45:36 2026 GMT
9.7        lower_case_table_names                            0
9.7        show_gipk_in_create_table_and_information_schema  ON
9.7        sql_generate_invisible_primary_key                OFF
```

All five servers answered `lower_case_table_names 0`,
`show_gipk_in_create_table_and_information_schema ON` and
`sql_generate_invisible_primary_key OFF`, the defaults, which the fixture does
not change. The certificate's validity dates name the generation of `tls/` the
image carries. On the no-TLS servers `tls_enabled` is `No` and the two dates are
empty. Every name is counted, and a variable missing from the answer is an
error rather than a blank: a name the server does not have comes back as no
row, which would otherwise read as a server that lacks the setting.

### What is not instrumented here

**The failing outcome of the read-back.** `FR-SRV-013`, which `FR-MYSQL-043`
applies, verifies a read-back that does not confirm the setting in process,
through the seam `FR-ERR-031` authorises; no server produces it, so nothing is
owed to this fixture for it. **The marked read of `FR-MYSQL-010`** is likewise
verified through a seam, per `FR-SRV-035`, and no server newer than `9.7` is
staged here.

## What the schema contains

`freight` is the MariaDB fixture's domain — a freight-forwarding and maritime
logistics operation, with real ports, UN/LOCODEs, ships and IMO numbers,
invented companies and plausible rates — adapted to MySQL. The table maps each
item of `FR-MYSQL-044` to the objects that realise it:

| Item | What `FR-MYSQL-044` asks for | Where it is |
|---|---|---|
| 1 | InnoDB and at least one other engine | every table InnoDB but `edi_message_staging`, MyISAM |
| 2 | Every native data type every series accepts | 37 distinct `DATA_TYPE` values: the five integer types signed and unsigned (`voyage.schedule_deviation_min` is the signed `int`, `voyage_leg.bunker_delta_kg` the signed `bigint`, `container_type.min_setpoint_celsius` the signed `tinyint`), `legacy_edi_field.element_position` `ZEROFILL`, every `BOOLEAN` as `tinyint(1)`; `decimal`, `float`, `double`, `bit`; `date`, `year`, `datetime`, `timestamp` and `time` with and without fractional seconds; `char`, `varchar`, the four text and four blob sizes, `binary`, `varbinary`; `enum` members with a comma (`cargo_item.imdg_class`) and an apostrophe (`vessel.class_society`), `set`; `json`; the eight geometry types, `port_facility.berth_outline` with `SRID 4326` |
| 3 | Every row of the default classification of `FR-MYSQL-029` | `consignment` holds no default on `NOT NULL` (`shipper_reference`) and on nullable (`broker_reference`), `DEFAULT NULL`, a string literal, the empty string (`handling_flags`), `CURRENT_TIMESTAMP(6)` and `CURRENT_TIMESTAMP(3)`; `legacy_edi_field` holds `''`, `'NULL'`, the binary literal `x'27'`; `container_type.height_ft_in` holds `'8''6"'`; numbers throughout; bit literals in `port` and `customer`; `customer.updated_at` `CURRENT_TIMESTAMP`; parenthesised expressions on `consignment.payment_due_on` (scalar), `consignment.terms_and_conditions` (`TEXT`, a string literal), `legacy_edi_field.una_service_string` (`BLOB`) and `customer.preferences` (`JSON`, string literals) |
| 4 | An auto-incremental and an invisible column, and each combination of phrases `FR-MYSQL-030` names | `auto_increment INVISIBLE` (`audit_event.audit_event_id`), `DEFAULT_GENERATED INVISIBLE` (`customer.crm_sync_token`), `VIRTUAL GENERATED INVISIBLE` (`consignment.insured_amount`), `DEFAULT_GENERATED on update CURRENT_TIMESTAMP` (`customer.updated_at`), `DEFAULT_GENERATED on update CURRENT_TIMESTAMP(3)` (`consignment.updated_at`), `INVISIBLE` alone (`customer.legacy_account_code`) |
| 5 | Virtual and stored generated columns, one with a string literal, an index on one | three `VIRTUAL` (`voyage_leg.moves_total`, `cargo_item.chargeable_weight_kg`, `consignment.insured_amount`) and seven `STORED`, `cargo_item.is_dangerous_goods` holding `'Not regulated'`; indexed: `idx_leg_moves_total` and `idx_cargo_item_chargeable` (virtual), `idx_container_vgm` (stored) |
| 6 | Simple and composite primary keys, no primary key, a `NOT NULL` unique index and no primary key | `port`; `voyage` (two columns), `voyage_leg` (three); `edi_message_staging`; `currency` |
| 7 | Unique, composite, descending, prefix, `FULLTEXT`, `SPATIAL`, invisible, commented, functional, `UNIQUE` functional and multi-valued indexes | `uq_port_un_locode`; `idx_port_country_name`; `idx_voyage_etd_desc`; `idx_vessel_name_prefix`; `ft_cargo_item_description`; `sp_facility_berth_outline`; `charge.idx_charge_invoice`; comments on many; `voyage.idx_voyage_departure_year`; `customer.uq_customer_vat_normalised`; `customer.idx_customer_milestone_alerts` |
| 8 | Foreign keys with distinct rules, one with neither clause, a composite key, a self-reference, and both directions across a second schema | the sixteen keys inside `freight` carry the sixteen `(ON DELETE, ON UPDATE)` pairs of `CASCADE`, `SET NULL`, `RESTRICT` and `NO ACTION`, one each; `fk_consignment_shipper` is written with neither clause and reports `NO ACTION`/`NO ACTION`; `fk_leg_voyage` and `fk_consignment_voyage` are composite; `fk_customer_parent` is the self-reference; `port.fk_port_country` references `masterdata.country`, and `masterdata.customer_mdm_link.fk_mdm_link_customer` references `freight.customer` |
| 9 | `CHECK` constraints at column and table level, named and unnamed, one `NOT ENFORCED`, one with a string literal | column level: `chk_vessel_capacity` (named), `cargo_item_chk_1` (unnamed); table level: `container_type_chk_1` (unnamed) and 23 named; `NOT ENFORCED`: `chk_vessel_dimensions`, `chk_customer_credit_policy`; string literals: `chk_voyage_direction` |
| 10 | Views with each check option, each security type, updatable and not, each algorithm | `v_port_directory` (`MERGE`, `INVOKER`), `v_active_vessel` (`UNDEFINED`, `DEFINER`), `v_reefer_container` (`LOCAL`), `v_reefer_container_deep_frozen` (`CASCADED`), `v_consignment_manifest` (`TEMPTABLE`, not updatable) |
| 11 | Procedures and functions with `IN`, `OUT` and `INOUT`, deterministic and not, each data access, a comment, each security type | four functions and three procedures; `NO SQL`, `CONTAINS SQL`, `READS SQL DATA` and `MODIFIES SQL DATA`; `INVOKER` on `fn_locode_country` and `sp_recalculate_freight`; a comment on every one |
| 12 | Triggers of each timing on each event, and two sharing one event and timing | seven on `consignment`; `trg_consignment_before_insert_currency` `FOLLOWS trg_consignment_before_insert`, `ACTION_ORDER` 2 |
| 13 | Comments on tables, columns, indexes and routines, one outside the basic multilingual plane | throughout; the column comment of `port.local_name` is written with `𠮷` (U+20BB7) — see [Observed while building the fixture](#observed-while-building-the-fixture) |
| 14 | Identifiers with a space, a backtick and a double quote | `legacy_edi_field`: `` `space in name` ``, `` `back``tick` ``, `` `double"quote` ``, and the index names `` `uq segment tag` `` and `` `idx``backtick` `` |
| 15 | A scheduled event and a partitioned table | `ev_purge_edi_staging`, created `DISABLE` so that the data does not move under a test; `vessel_position`, four `RANGE COLUMNS` partitions |
| 16 | A reduced-grant reader, and a schema declaring no collation of its own | `tpl_reader`, with `SELECT` and `EXECUTE` on `freight` and on the target of item 17 alone; `masterdata`, which inherits `utf8mb4_0900_ai_ci` |
| 17 | A target database whose name holds a quotation mark, a backslash and a backtick, and four decoys | the target ``tpl_q'b\s`k``, holding `consignee`, the view `v_consignee_by_place`, ``tariff'line`%`` and ``tariff'line`2026``, which the first matches as a `LIKE` pattern; the decoys ``tpl_q'bs`k`` (`decoy_escape_consumed`), ``tpl_q'b\\s`k`` (`decoy_backslash_doubled`), `tpl_q_b_s_k` (`decoy_like_pattern`) and ``TPL_Q'B\S`K`` (`decoy_letter_case`). `tpl_reader` holds no grant on a decoy: the `GRANT` escapes the target's `_` and `\`, because the database part of a `GRANT` is a pattern |
| — | The two cases `FR-MYSQL-044` lists apart from its seventeen items: a generated invisible primary key, and a foreign key declared `ON DELETE SET DEFAULT ON UPDATE SET DEFAULT` | `equipment_survey`, created with `sql_generate_invisible_primary_key` `ON` for that one statement, so the server adds `my_row_id` and `PRIMARY` on it; its `fk_survey_container_type` is the `SET DEFAULT` key, which both series accept and report and neither performs |

`seed.sql` ends by booking a consignment through `sp_book_consignment`, which
draws `BKG-000480001` from `reference_counter` and fires both `BEFORE INSERT`
triggers in their declared order, updating two consignments and deleting a
third, so that all seven triggers have fired — `audit_event` holds 12 rows — by
the time the container reports itself ready.

## Deliberate omissions

Anything a supported series refuses is absent, rather than hidden behind a
conditional, as `FR-MYSQL-044` requires; so is anything its seventeen items do
not ask for.

| Left out | Why |
|---|---|
| `VECTOR` columns | Refused by `8.4` with `ERROR 1064 (42000)`, a syntax error at `VECTOR(3)`; accepted by `9.7`, which reports `vector(3)`. `FR-MYSQL-044` asks for the types *every* series accepts, and requires the refusal to be recorded under `FR-MYSQL-014`, which `specification/` owns |
| A `SET` member containing a comma | `SET` values are stored comma-separated; the comma requirement is met by `ENUM` |
| Sequences, system-versioned tables, `UUID`, `INET4`, `INET6` | MariaDB's, and absent from MySQL, per `FR-MYSQL-039`. Booking references come from a counter row instead; identifiers and addresses are `BINARY(16)` and `VARBINARY(16)`, which is how a MySQL schema stores them |
| A non-transactional table written in one statement with an InnoDB one | `9.7` runs with `enforce_gtid_consistency` `ON` and refused the MariaDB fixture's arrangement — a MyISAM `audit_event` written by triggers on `consignment` — with `ERROR 1785 (HY000): Statement violates GTID consistency`. The MyISAM table is the one written only on its own |

## Observed while building the fixture

None of these is a requirement or a record of the specification. Each is what a
command run while building the fixture showed, kept here so that rmp `#326`
finds it.

1. **A character outside the basic multilingual plane does not survive in a
   comment.** On `8.4` and on `9.7`, a table, column, index or routine comment
   holding `𠮷` is accepted with `Warning 1300: Cannot convert string ... from
   utf8mb4 to utf8mb3`, and the catalogue then reports `?` (`0x3F`) in its
   place; `information_schema` declares every comment column `utf8mb3`. The DDL
   writes the character as `FR-MYSQL-044` item 13 asks; the catalogue cannot
   return it.
2. **The text of a functional key part differs between the series.** For
   `uq_customer_vat_normalised`, `STATISTICS.EXPRESSION` reads
   ``upper(replace(`vat_number`,_ascii' ',_utf8mb4''))`` on `8.4` and
   ``upper(replace(`vat_number`,_ascii' ',_ascii''))`` on `9.7`. It was the one
   difference in the summary compared to confirm the load — every column's
   attribute field, the functional, invisible, non-B-tree, descending and
   prefix index members, the foreign keys, the `CHECK` constraints, the views,
   the routines, the parameters, the triggers and the tables. That summary is
   not the field-by-field comparison `FR-MYSQL-014` asks of rmp `#326`.
3. **GTID mode differs by default:** `gtid_mode` and `enforce_gtid_consistency`
   are `OFF` on `8.4` and `8.0`, and `ON` on `9.7`. See the last row of
   [Deliberate omissions](#deliberate-omissions).
4. **A `GEOMETRYCOLLECTION` column reports `DATA_TYPE` `geomcollection`** on
   both series.
5. **A `SPATIAL` index member reports `SUB_PART` `32`** and `INDEX_TYPE`
   `SPATIAL` on both series.
6. **InnoDB accepts `SET DEFAULT`.** In a scratch schema, on `8.4.13` and on
   `9.7.4`, `FOREIGN KEY (pid) REFERENCES p (id) ON DELETE SET DEFAULT ON
   UPDATE SET DEFAULT` was accepted without error or warning, and
   `REFERENTIAL_CONSTRAINTS` reported `SET DEFAULT` for both rules. This
   contradicts the statement of `FR-MYSQL-033` that InnoDB rejects such a
   definition, and row 18 of `FR-MYSQL-040`, both marked *Pending
   observation*.
7. **`FLUSH PRIVILEGES` draws warning 1681**, "deprecated and will be removed
   in a future release", on `9.7`, and no warning on `8.4`. It empties the
   authentication cache on both, which is what `auth-cache.sh flush` relies
   on.

## Stopping and removing

`./down.sh` removes every server and proves nothing is left; see [Taking it
down](#taking-it-down). `./down.sh --images` drops the three images as well.
Leave no container running after a validation run.
