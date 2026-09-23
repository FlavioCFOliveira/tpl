# `tpl` Security Audit

Sprint 20, task #251 — Hostile Security Laboratory.

## Scope and method

- **Target.** The `tpl` CLI built from this repository, and only that binary.
- **Commit.** `051f1a8` (branch `feature/20-hostile-security-laboratory`).
- **Platform.** `aarch64-apple-darwin`, macOS 26.6.2 (Darwin 25.6.0),
  `rustc` 1.98.1 (48a229cea, 2026-09-01), `cargo-fuzz` 0.13.1 on nightly
  `1.100.0` (495c385d0). Release build (`opt-level=3`, `lto=fat`,
  `panic=abort`, `strip=true`).
- **Tools.** White-box source review; black-box execution of
  `target/release/tpl`; the project MariaDB fixture driven only through its
  harness in `scripts/mariadb/`; live statement capture through the fixture's
  general query log; a `cargo-fuzz` crate depending on the `tpl` library by
  path; a time-bounded mutational black-box fuzzer over the parser/render
  surfaces; `cargo audit`; `cargo clippy`; the test suite.
- **Fixture.** Brought up through `scripts/mariadb/up.sh` (all four supported
  series `10.11`/`11.4`/`11.8`/`12.3` plus the `--skip-ssl` server), confirmed
  with `status.sh` (exit 0, every server answered), and torn down through
  `down.sh` (exit-checked: `docker ps` lists no `tpl-mariadb` container).
- **Authorisation.** Owner-authorised offensive audit against this repository's
  binary and its local test fixture. Every PoC is contained under
  `target/security-lab/` (git-ignored). No PoC was run against any external
  system. No tracked file was modified except this report (`git status` clean
  but for `SECURITY-AUDIT.md`).
- **Method phases exercised.** Reconnaissance, threat model, attack-surface
  enumeration, per-component and inter-component analysis, live proof of
  concept, impact and remediation.

## Attack-surface map

| # | Input | Reaches |
|---|---|---|
| 1 | argv, flags, environment (`${VAR}`) | `cli/` parser, `project::config::expand` |
| 2 | project folder, discovery walk, symlinks, permissions | `project::discover`, `project::trust` |
| 3 | `.tpl/.cfg`, connection defs, credentials, TLS material | `project::config`, `mariadb::connect`, `project::password` |
| 4 | catalogue cache files on disk | `cache` (read-back, write-through) |
| 5 | catalogue content from the server | `mariadb::catalogue`, `model`, `model::document`, `output::escape`, `diagnostics::escape` |
| 6 | templates (MiniJinja) rendered from disk | `render` |
| 7 | output destinations named on the command line | none — `tpl render` writes stdout only (`FR-RND-028`) |
| 8 | terminal output (control/escape sequences) | `output::escape` (`FR-OUT-018`), `diagnostics::escape` (`FR-ERR-024`) |

## Verdict per input

| # | Input | Verdict | Evidence |
|---|---|---|---|
| 1 | argv / flags / env | Not exposed | `${VAR}` expands only in whitelisted fields; DSN parsed into fields before expansion + percent-encoded; a DSN carrying any `?` query parameter is refused at config time (`78`), so TLS cannot be downgraded through a DSN parameter; 5 000 `--set` flags handled (rc 0); control characters in argv escaped in diagnostics. |
| 2 | project folder | Not exposed | Walk canonicalises + stops at mount point (`discover.rs`); `.tpl/.cfg` owner + `0o077`-mode check (`trust.rs`); symlink `.tpl` verified at target. |
| 3 | `.tpl/.cfg` / credentials / TLS | **SEC-01** (availability) | Redaction, no `password` flag, read-only session enforced + read back live; TLS matrix shows no silent downgrade and verification not bypassable (below); `password_command` deadline bypassable — SEC-01. |
| 4 | cache files | **SEC-03** (integrity, case-/normalisation-insensitive filesystems) | Object read decodes JSON (else a miss, `cache.rs:1043`); write is `symlink_metadata`-guarded + `rename` over target; component names reject `/`, `.`-prefix, NUL (`paths.rs:215`), and live `../../../../tmp/tpl_sec_lab_escape` / `..x` were not cached, nothing written outside `.tpl/.cache`. But object file names are the object name verbatim, so on APFS two names that differ only in case or Unicode normalisation share one file and a cached single-object read answers a false `66` — SEC-03. |
| 5 | catalogue content | **SEC-02** (input-validation) via `--context`; catalogue-sourced output Not exposed | Live hostile objects (classes a–f below) on the fixture: control characters escaped on text output, quotes/`{{ }}` never evaluated, max-length names and comments and a 4 MB routine body handled without fault. A hostile `--context` document with a dangling foreign-key reference drives an internal-invariant error — SEC-02. |
| 6 | templates | Not exposed (contained) | No env/file/network/clock function; include/import confined to the template root; `range`/recursion capped by the engine; render CPU hard-killed by the deadline. |
| 7 | output destinations | Not exposed | `tpl render` writes stdout only; no file-writing surface (`FR-RND-028`). |
| 8 | terminal output | Not exposed | C0 controls escaped in diagnostics and read-command output (inputs 5 and 8). |

## Verdict per threat class

| Threat class | Verdict | Evidence |
|---|---|---|
| Path traversal / symlink escape | Not exposed | Template root canonicalise + `starts_with` + per-component symlink refusal (`root.rs`); `../`, absolute, symlinked dir → 65. |
| Arbitrary file read | Not exposed | No file-reading template function; `include "/etc/passwd"`/`../../etc/passwd` → 65; cache read of a symlink yields only JSON-decodable content. |
| Arbitrary file write / overwrite / chmod | Not exposed | Writes only the requested project folder and cache under `.tpl/.cache/`; cache writes go to a `pid`-named temp then `rename`, mode `0o600`, never through a symlink. |
| Template sandbox escape | Not exposed | `range.__class__`, `self.__init__`, `get_env`, `environ` → 65. |
| Code / command execution | Not exposed | `password_command` shell-free from an argument array; templates cannot spawn. |
| Credential disclosure | Not exposed | `Secret` redacts; panic hook prints only `location()`; sentinel test `BR-SEC-003` present. Measured: a sentinel password supplied through `${VAR}` never appeared in stdout or stderr of 20 commands spanning the tree, run at `-vvv` (`cfg get` prints the unexpanded `${VAR}` reference, by design under `FR-SEC-004`). The password appears 0 times in the preserved general logs of every live run. |
| Denial of service | **SEC-01** (password_command hang); render peak-memory is H-1 | CPU/loops bounded by the render deadline and the engine's `range` cap; recursion capped (no stack overflow). |
| TOCTOU races | Not exposed | Cache `holds()` re-opens and compares `(dev,ino)`; write is atomic rename. |
| Cache integrity (wrong answer from the store) | **SEC-03** | Case-/normalisation-colliding object names share one cache file; the single-object read serves it without checking the member's name. |
| Integer / size overflow | Not exposed | `#![forbid(unsafe_code)]`; saturating arithmetic; 364 647 total fuzz execs, 0 memory crashes. |
| Panic-driven abort | Not exposed | 0 signal deaths across all fuzz campaigns; the one `70` observed is a returned `Error`, not a panic (SEC-02). |
| Terminal injection | Not exposed | Inputs 5 and 8. |
| **SQL injection** | Not exposed | **Live**: every value that reaches the server travels as a bound `?` parameter. See below. |
| **Any DB write** | Not exposed | **Live**: general log across the full surface × 4 series shows only reads + session setup; the read-only guard blocks writes even for a write-privileged account. See below. |
| TLS downgrade / cert-verify bypass | Not exposed | **Live** TLS matrix: no silent downgrade, verification not bypassable. See below. |
| Supply chain | Not exposed | `cargo audit` clean; `vendor/sqlx-core-0.9.0` is one documented one-statement divergence. |

## Live fixture findings

### Statement audit (all four TLS series, full command surface)

Every command of the tree was run against a live connection under the general
query log directed to a table (`observe.sh statements`). Reader entry
(`tpl_reader`, SELECT+EXECUTE) and root entry, on each of `10.11`, `11.4`,
`11.8`, `12.3`. Classification of every logged statement (798 log rows per
series):

- `Connect … using SSL/TLS`, then per connection, in this order:
  `SET SESSION TRANSACTION READ ONLY`, `SELECT @@session.tx_read_only`,
  `SELECT VERSION()` — matching the `FR-SRV-042` session-start order on every
  connection (verified: 31/31 connections).
- `Prepare` + `Execute` of `SELECT … FROM INFORMATION_SCHEMA.*` only
  (parameterised, `WHERE … = ?`).
- `Quit`.
- **Non-read statements: 0.** No `INSERT`/`UPDATE`/`DDL`/`SHOW`/`SET` beyond the
  read-only session statement, on any series.

### Read-only guard with a write-privileged account

The fixture provides `tpl_reader` (read-only); no write-capable application
account exists and none was created. Using the write-capable `root` account, a
session was set `SET SESSION TRANSACTION READ ONLY` (as `tpl` does) and every
write attempted directly: `INSERT`, `UPDATE`, `CREATE TABLE`, `ALTER`, `DROP`,
`CREATE DATABASE`, `CREATE USER`, `GRANT` were all refused with error `1792`
(`ER_CANT_EXECUTE_IN_READ_ONLY_TRANSACTION`) on all four series, with
`@@session.tx_read_only = 1`. The guard prevents writes even when the account
holds write privileges (`BR-SEC-002`, `FR-SRV-008`).

### SQL injection (live)

Injection payloads were fed through every user-supplied value that reaches a
catalogue query and the general log inspected:

- **Object name** (`schema table|view|routine <name>`, `render --table <name>`):
  `consignment' OR '1'='1`, `x' UNION SELECT user,password FROM mysql.user -- `,
  `consignment\' OR 1=1 -- `, ``consignment`; DROP TABLE t; --``. Each arrives
  as the bound value of a prepared `… WHERE TABLE_NAME = ?` (or the render
  path's lookup), produces no structural effect, and `tpl` returns `66`
  (not found). No payload appeared as SQL in any `Prepare` text.
- **Schema name** (entry `database` key): `freight' OR '1'='1` arrives as the
  connection default database (`Connect … on freight' OR '1'='1`) and as the
  bound `SCHEMA_NAME = ?` value; the server treats it as a literal name, returns
  nothing, and `tpl` reports `77` — the same outcome as a plainly missing schema
  (`nosuchdb`). No injection.
- `--pattern` never reaches the server (matched in memory, `FR-SCH-013`,
  `pattern.rs`).

Parameterisation is total; no user value is concatenated into SQL.

### TLS mode matrix (live)

Each `tls` mode set against the TLS-offering server (`11.8`, port 13308) and
the `--skip-ssl` server (13310), transport confirmed from each server's
general-log `Connect` line (`using SSL/TLS` vs `using TCP/IP`):

- **No silent downgrade.** The secure demand modes — default (`verify-identity`),
  `required`, `verify-ca`, `verify-identity` — all **fail** (`69`) against the
  no-TLS server rather than falling back to plaintext. The DSN entry with no
  `tls` key (default `verify-identity`) also failed against the no-TLS server.
- **Verification not bypassable.** `verify-identity`/`verify-ca` **without** a
  `ca_file`, and with a **rogue** CA, all fail the handshake (`69`) against the
  TLS server; `verify-identity` against a hostname absent from the certificate
  (a LAN IP, cert carries `127.0.0.1`/`::1`/`localhost`) fails on the hostname
  check.
- **Documented weaker semantics confirmed (not vulnerabilities).** `required`
  connects with a rogue CA (encrypt, no chain verification) and `verify-ca`
  connects to a mismatched hostname (chain, no hostname) — exactly the two
  limits `FR-SEC-021` states and the reason `verify-identity` is the default.
- **DSN cannot smuggle a TLS parameter.** Any DSN with a `?` query parameter
  (e.g. `?ssl-mode=disabled`) is refused at config validation (`78`,
  `FR-CONF-013`), so a DSN parameter cannot contradict the `tls` key.

### Hostile catalogue objects (live)

With the owner's explicit authorisation, the coordinator created the hostile
objects on all five fixture servers in a scratch schema `sec_lab`
(`target/security-lab/live/hostile.py`; project
`target/security-lab/live/sec/`, entries `r1011`, `r123`, `w1011`), drove the
schema commands, cache load and render over them, then dropped `sec_lab` and
took the fixture down. Class (g) was then independently re-verified by the
auditor from the source and the preserved cache (below).

| Class | Content | Result |
|---|---|---|
| (a) | ESC/CSI/OSC/BEL/CR/LF in table and column names and comments | Escaped (`\u{1b}`, `\r`, `\n`) on `schema tables/table/routines/routine` text output. Not exposed. |
| (b) | Quotes, backticks and `{{ }}`/`{% %}` in names and defaults | Printed literally; never evaluated by render. Not exposed. |
| (c) | `../../../../tmp/tpl_sec_lab_escape`, `..x`, `a\..\b`, `x..` | The first two not cached (component gate); nothing written outside `.tpl/.cache`; the last two cached as ordinary names. Not exposed. |
| (d) | 64-character ASCII and 40-character CJK names | Cached, listed, rendered. Not exposed. |
| (e) | Maximum-length comments (2048 table / 1024 column) | No fault. Not exposed. |
| (f) | 4 MB routine body; hostile view and procedure bodies | Cached as a 4.2 MB file, 24 MB RSS, 0.03 s; bodies escaped on text output. A reader without body privilege caches 0 routines and records the collection as not whole. Not exposed. |
| (g) | Names differing only in case (`Acct`/`acct`) or Unicode normalisation (`café` NFC/NFD) | 12 tables → 9 cache files; a cached single-object read answers a false `66`. **SEC-03.** |

**Accepted risk (by design, not a finding).** `tpl render` writes catalogue
values byte for byte, so control bytes from an object name or comment reach
stdout raw when a template interpolates them. `FR-OUT-019` excludes the render
result and `tpl template show` from the C0 escaping of `FR-SEC-020`, because a
render is code the caller asked for. A template author who needs escaping asks
for it explicitly (`escape`, `json`). A caller who renders to a terminal rather
than to a file accepts this.

## Findings register

### SEC-01 — `password_command` deadline bypassed by a descendant holding the stdout pipe (indefinite hang)

- **Severity.** Medium. CVSS 3.1 **5.5** — `AV:L/AC:L/PR:N/UI:R/S:U/C:N/I:N/A:H`.
- **CWE.** CWE-410 / CWE-400 — a blocking phase without an effective deadline.
- **Location.** `src/project/password.rs:186` (success path) and `:252`
  (`reap`), both `let _ = reader.join();`, joining the reader thread whose
  `stdout.take(OUTPUT_CAP+1).read_to_end(...)` at `:116` never returns while a
  write end of the pipe stays open.
- **Requirement broken.** `FR-SEC-022` (every blocking phase SHALL have a
  deadline) and `FR-CONF-028` / `FR-SEC-012`. `FR-SEC-024` caps the child's
  bytes (4096) but not its time.
- **Attack path.** A `.tpl/.cfg` names a `password_command` whose program writes
  ≤4096 bytes to stdout, forks a descendant that inherits the stdout
  descriptor, and exits 0. `obtain()` reaps the direct child within the deadline,
  breaks the deadline-checked loop, and reaches `reader.join()`; the reader is
  blocked in `read_to_end` (limit not reached, EOF never arrives) and the join —
  outside the loop, no timeout — blocks for as long as the descendant runs.
- **Reproduction.** `target/security-lab/pw/` — `holder.sh` = `printf secret;
  sleep 120 &; exit 0`; `.cfg` with `password_timeout=2`, `connect_timeout=3`.
  `perl -e 'alarm 20; exec @ARGV' tpl -d shop schema dump`. Measured: clean
  helper 322 ms (rc 78); daemonising helper ran until the 20 s guard fired
  (rc 142), i.e. hung past both the 2 s password and 3 s connect deadlines, with
  the `sleep 120` descendant left holding the pipe.
- **Observed vs expected.** Observed: indefinite hang, no diagnosis. Expected:
  abandon at the `password_command` deadline with exit 78.
- **Mitigating factor.** `FR-SEC-014` requires the `.cfg` at mode ≤600 owned by
  the invoking user, so a clone at git's `0644` is refused first; reachability
  needs a `.cfg` the victim actually uses (including one adopted from a clone and
  `chmod 600`-ed) — which is the untrusted-`.cfg` case the project's threat model
  already assumes.
- **Remediation (not applied).** Bound the reader join to `remaining()`, or make
  the read non-blocking / run the child in its own process group and `killpg`,
  so no `join`/`recv` in this function blocks past the phase deadline.

### SEC-02 — hostile `--context` document with a dangling reference is reported as an internal `tpl` defect (`70 EX_SOFTWARE`)

- **Severity.** Low. CVSS 3.1 **3.3** — `AV:L/AC:L/PR:N/UI:R/S:U/C:N/I:N/A:L`.
- **CWE.** CWE-20 (Improper Input Validation) / CWE-703 (Improper Handling of
  Exceptional Conditions).
- **Location.** `src/model/document/build.rs:227` (`find` → `dangling()` →
  `ensure_invariant(false, REFERENCE_IS_CARRIED)`), reached when the model is
  rebuilt from a `--context` document.
- **Requirement bearing.** The `--context` document is untrusted input
  (`specification/security.md`, "an ordinary JSON file, from anywhere"). A
  malformed-but-well-typed document is a caller/input error and should be a
  document-validation exit, not `70`, which the help text reserves for a defect
  the caller "cannot correct" and asks them to "report".
- **Attack path.** A `--context` document whose table declares a foreign key
  whose `referenced_table` names a table absent from the document's `tables`
  array. Document deserialisation validates shape but not cross-reference
  integrity, so the builder trips its own internal invariant and returns
  `Error::InternalInvariant` (exit 70). It is a returned error, not a panic — no
  memory unsafety and no crash.
- **Reproduction.** `target/security-lab/poc_dangling_fk.json` — a one-table
  document (`address`, FK `fk_address_city → city`) with `city` removed.
  `tpl render id --context poc_dangling_fk.json` →
  `error: the invariant 'every table a foreign key names is carried …' does not
  hold … exit: 70 (EX_SOFTWARE)`. Found by the context fuzz target and minimised.
- **Observed vs expected.** Observed: `70 EX_SOFTWARE` with "this is a defect in
  tpl … report it". Expected: a document-error exit that attributes the fault to
  the input.
- **Impact.** An attacker who supplies a `--context` document can make `tpl`
  claim it is internally broken and solicit a bug report; a single invocation
  fails deterministically, no crash, no unsafety, no disclosure.
- **Remediation (not applied).** Validate document referential integrity at
  deserialisation (or classify a dangling reference as a `--context`
  document-validation error) rather than as an internal invariant.

### SEC-03 — case- or normalisation-colliding object names share one cache file; a cached single-object read answers a false "does not exist"

- **Severity.** Low. CVSS 3.1 **3.7** — `AV:N/AC:H/PR:L/UI:R/S:U/C:N/I:L/A:L`
  (adversary: anyone able to create an object in the schema; trigger: the
  victim loads the cache on a case- or normalisation-insensitive filesystem and
  then reads the object by name). The more common trigger is accidental: a
  schema that legitimately holds two such names.
- **CWE.** CWE-178 (Improper Handling of Case Sensitivity) and CWE-176
  (Improper Handling of Unicode Encoding); the effect is CWE-706 (Use of
  Incorrectly-Resolved Name).
- **Root cause.**
  1. `src/cache/paths.rs:174` (`Layout::object`) and `:163`
     (`Layout::routine`) build the file name from the object name verbatim
     (`tables/<name>.json`). On APFS, which is case-insensitive and
     normalisation-insensitive by default, `Acct.json`/`acct.json` and the
     NFC/NFD spellings of `café.json` are one file, and the name written last
     overwrites the other. The write side notices: `prune` (`cache.rs:1329`)
     counts fewer files than it wrote and records `tables whole: false`.
  2. The single-object read does not check what it read.
     `Cache::table`/`view`/`routine` (`cache.rs:736-759`) open
     `layout.<kind>(name)` and `Cache::member` (`cache.rs:760`) returns the
     file as a **hit**. The file holds a member whose `name` differs from the
     one requested, and nothing checks that.
  3. `named::member` (`src/cli/schema/named.rs:266-276`) then searches that
     one-member population for the requested name, fails, and `absent`
     (`named.rs:352-366`) raises `Error::CatalogueObjectNotFound` with
     `Sought::Catalogue`. That error's cause is the text at
     `src/diagnostics/cause.rs:286`: "no row of INFORMATION_SCHEMA matches
     table '…'". The server is never consulted.
- **Platforms.** macOS on default APFS (case-insensitive,
  normalisation-insensitive). Also any case-insensitive volume: HFS+,
  exFAT/FAT, SMB/NFS mounts of such volumes, and ext4 directories with
  `casefold` enabled. **Not** default Linux ext4/xfs/btrfs, which are
  case-sensitive and byte-exact. Windows is out of scope.
- **Requirements broken.**
  - `FR-CDOC-008`: serve an individual object from the cache "whenever **it**
    is present". The requested object is not present; another object is.
  - `FR-CACHE-007`: a miss SHALL read the server. The read is treated as a hit,
    so the server is never read.
  - `FR-ERR-010` / `FR-ERR-034`: the `cause` SHALL be factual. The cause
    asserts that no row of `INFORMATION_SCHEMA` matches, when the server holds
    that row and was never asked.
  - The principle of `BR-CDOC-002`: a wrong answer wearing the appearance of a
    right one, which the caller cannot detect.
- **Commands affected.**
  - Confirmed live: `schema table <name>`.
  - Affected by construction, through the same `Cache::member` and the same
    verbatim naming, but not reproduced live because no colliding views or
    routines were created: `schema view <name>` and the qualified
    `schema routine procedure:<name>` / `function:<name>`.
  - **Not** affected: `schema tables/views/routines`, the bare
    `schema routine <name>`, `schema dump` and a render of the whole database.
    All of them go through `Cache::collection`/`everything`, which require the
    collection to be recorded whole, and it is not. Reproduced: with the
    fixture down, `schema tables` and `schema dump` go to the server and fail
    with `69`.
  - `render --table Acct`: the coordinator observed it fall back to the server
    and answer correctly.
- **Reproduction.** Cache evidence preserved in
  `target/security-lab/live/sec/.tpl/.cache/r1011/`: `tables/Acct.json` holds
  `"name":"acct"`, `tables/café.json` holds the NFC spelling, and `meta.json`
  records `tables whole:false`. With the **fixture down** and a **wrong
  password** (so any server contact would surface as a connect failure):

  ```
  cd target/security-lab/live/sec
  TPL_READER_PW=wrong ../../../release/tpl -d r1011 schema table Acct
  # error: table 'Acct' does not exist in database 'sec_lab'
  # cause: no row of INFORMATION_SCHEMA matches table 'Acct' ...   exit 66
  TPL_READER_PW=wrong ../../../release/tpl -d r1011 schema table "$(printf 'cafe\xcc\x81')"
  # error: table 'café' does not exist ...                          exit 66  (NFD form)
  TPL_READER_PW=wrong ../../../release/tpl -d r1011 schema table acct   # served, exit 0
  TPL_READER_PW=wrong ../../../release/tpl -d r1011 schema tables       # miss -> server, exit 69
  ```

  Both `Acct` and the NFD `café` exist on the server.
- **Observed vs expected.** Observed: exit `66`, a cause claiming the server
  holds no row, and the server never contacted. Expected: a miss (the cached
  file holds a different object), a server read under `FR-CACHE-007`, and the
  real object returned.
- **Impact.** A false negative with false provenance. A code-generating agent
  concludes that a real table does not exist, and nothing in the output says
  the answer came from a collided cache file. Anyone who can create an object
  in the schema can hide a table from cached reads by creating its
  case-variant: files are written in byte order, the last write wins, and the
  lower-case spelling sorts after the upper-case one. The effect lasts until
  `cache clean` or `--direct`. No disclosure, no escape from the cache
  directory, and no wrong object is ever returned under another name, because
  the name comparison is exact.
- **Remediation (not applied).** In `Cache::member`, treat a member whose
  decoded `name` (and kind) differs from the requested one as a miss. That
  closes the false answer on every filesystem. To stop the collision itself,
  encode file names injectively and in a case-folded form (for example a hash,
  or percent-encoding of every non-`[a-z0-9_]` byte, including upper case).

## Hardening observations (not vulnerabilities)

- **H-1. Render peak memory is bounded only by wall-clock, not by bytes.** A
  hostile template accumulating through a `namespace` grows memory exponentially
  and is killed by the render deadline (`nsbomb`: rc 65 at 5.2 s with a 5 s
  `render_timeout`); under memory pressure the OOM killer can fire first and the
  process dies by SIGKILL (rc 137) rather than 65. Contained (single-invocation);
  a byte/element cap would make the failure deterministic.
- **H-2. Cache object read follows symlinks.** `read_to_string` on a cache
  object follows a symlink; it grants nothing, but a `symlink_metadata` guard on
  read would match the write-path discipline.

## Fuzz campaign

Two complementary campaigns.

**Coverage-guided (`cargo-fuzz`).** A standalone crate under
`target/security-lab/fuzz/` depends on the `tpl` library by path and builds it
under sanitizer coverage + ASan (build verified: `cargo +nightly fuzz build`
compiles `tpl` and the target). **Result: in-process coverage-guided fuzzing of
`tpl`'s parsers is not achievable through the public API without a source
change.** The library exposes only `run()`, `install_panic_hook()` and `Error`;
`run()` hardcodes `std::env::args_os()`, and the `probe_argv` target proved
empirically that inside a libFuzzer harness those args are the fuzzer's own
(`["…/probe_argv", "-artifact_prefix=…", "-runs=1", "…/corpus"]`), so `run()`
would fail at `clap` on `-artifact_prefix` before reaching any parser and cannot
be steered per input. Driving it would require an argv-accepting entry point
(e.g. `run_from(args, cwd)`), a one-line change to `src/lib.rs` that this task
forbids. `cli::parse` already takes an arbitrary iterator but is `pub(crate)`,
invisible across the crate boundary.

**Black-box, time-bounded (the feasible in-depth campaign).** The mutational
fuzzer drives the release binary over each attacker-reachable surface, fork-per-
input (each execution is a fresh process). Oracle: rc 70 that is a panic, or
signal death (rc ≥ 132); hangs flagged separately (rc 124/142).

| Target | Entry point | Execs | Time | Crashes | Hangs |
|---|---|---|---|---|---|
| template | `render <t> --context` (hostile `.jinja`) | 113 710 | 620 s | 0 | 0 |
| config | `cfg list` (mutated/random `.tpl/.cfg`) | 128 643 | 600 s | 0 | 0 |
| context | `render id --context` (mutated document) | 122 294 | 600 s | 1 | 0 |

Total 364 647 executions. The single "crash" flag in the context target was
triaged to **SEC-02**: a returned `Error::InternalInvariant` (exit 70), not a
memory fault — reproduced and minimised to `poc_dangling_fk.json`. The cache
object-file decode path is the same `serde_json` + model deserialisation the
context target exercises. No memory-unsafety, abort, or hang was found on any
surface. (An earlier warm-up run added ~37 200 execs, also clean bar the same
SEC-02 class.)

## `cargo audit`

Clean. `cargo-audit` 0.22.2, 1266 advisories, 181 dependencies scanned, exit 0,
no vulnerabilities, no unmaintained/yanked warnings.

## Mandatory pipeline

| Step | Result |
|---|---|
| `cargo fmt --all -- --check` | pass |
| `cargo clippy --all-targets --all-features -- -D warnings` | pass (0 warnings) |
| `cargo build --release` | pass |
| `cargo test --all-features` | pass — 1085 passed, 0 failed, 0 ignored |
| `cargo audit` | pass — 0 vulnerabilities |

## Limitations

- **No coverage-guided fuzzing of `tpl`'s parsers.** Cause: the library's only
  entry point, `run()`, reads `std::env::args_os()` instead of taking an
  argument vector. Inside a libFuzzer harness those arguments are the fuzzer's
  own, as the `probe_argv` target showed, so `run()` stops at `clap` before any
  parser. `cli::parse` takes an iterator but is `pub(crate)`. Closing this
  needs an argv-taking public entry point (for example
  `run_from(args, cwd)`), which is a source change and outside this task. The
  fuzzing delivered is black-box, one fresh process per input, at scale
  (364 647 executions).
- **Hostile catalogue objects were created by the coordinator, not the
  auditor.** The auditor's own attempts to script them were stopped by the
  environment's safety classifier and were not reworked. The coordinator ran
  classes (a)–(g) with the owner's authorisation; the auditor independently
  re-verified (g) from the source and the preserved cache. Classes (a)–(f) are
  reported as the coordinator observed them.
- **SEC-03 was reproduced for tables only.** Views and qualified routines are
  affected by construction (the same code path) but were not reproduced live.
- **`aarch64-apple-darwin` only.** Linux libc-specific paths not exercised.
- The audit covers commit `051f1a8`; any later change re-opens the surface.
