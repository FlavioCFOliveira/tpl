# Changelog

All notable changes to `tpl` are recorded in this file.

The format is [Keep a Changelog](https://keepachangelog.com/en/1.1.0/) and the
versioning is [Semantic Versioning](https://semver.org/spec/v2.0.0.html). Neither
is chosen here: both were settled before this file existed, in `OD-03` of
[`docs/spec-technical/open-decisions.md`](docs/spec-technical/open-decisions.md#od-03--versioning-the-binary-the-document-the-cache-the-changelog),
which also fixes the pre-1.0 rule — **while `tpl` is below 1.0, a breaking change
is a minor bump.** The binary version has one source, the `version` field of
`Cargo.toml`; a release moves it there and nowhere else.

## Maintaining this file

A task writes its own entry during the documentation step of the project
workflow, before it closes.

- **Put the entry under `Unreleased`**, in the section that matches the change:
  `Added`, `Changed`, `Deprecated`, `Removed`, `Fixed` or `Security`. Create the
  section if it is absent, and keep the six in that order.
- **One line per change**, naming the observable thing it affects — a command, a
  flag, an exit code, a document key, a configuration key or a registered
  template name. State what a caller can now do, or can no longer do, not how it
  was implemented.
- **Add nothing when nothing a caller can observe changed.** Work confined to
  `/specification`, `docs/`, `tests/`, `scripts/`, the knowledge graph or the
  roadmap leaves this file untouched; `git log` and the roadmap already carry it.
- **Renaming or removing a name from the registered template surface MUST be
  recorded here.** `FR-ENV-029` makes it the one entry a release may never omit,
  and it is breaking: under the pre-1.0 rule above, it moves the minor number.
  Adding a name is not breaking and is an ordinary `Added` line.
- **A fix is a `Fixed` entry only when the defect shipped.** A defect present
  in a released version is recorded under `Fixed`. A defect corrected in work
  that has not yet been released is not: amend the `Unreleased` line it belongs
  to instead.

## Unreleased

### Added

- **SQLite as a second database provider.** An entry's new `provider` key is
  `mariadb` — the provider of an entry without the key — or `sqlite`; a
  `sqlite` entry names its database file in the new `path` key, read literally
  and resolved against the directory that holds `.tpl`. The eight `schema` subcommands,
  `tpl cache load`, `tpl render` and `tpl cfg database test` read such a file
  through the SQLite library built into `tpl`, opened read-only, never
  created, and hardened as untrusted input. SQLite has no routines, so
  `routines` is always `[]`, and a routine named on the command line exits
  `66`.
- **`--provider` and `--path` on `tpl cfg database add` and `update`**, and
  the keys `database.<name>.provider` and `database.<name>.path` for
  `tpl cfg set`, `get` and `unset`. Each of these exits `64`, writing
  nothing: a provider other than `mariadb`, `mysql` or `sqlite`; `--path`
  beside a server flag or beside `--provider mariadb` or `--provider mysql`,
  or given to `add` without `--provider sqlite`; an empty
  `--path` or one holding `${`; and a write that would leave an entry carrying
  a key its provider does not take. Writing or removing `provider` or `path`
  keeps the entry's cache and warns, as writing or removing a connection key
  does. The warning says the entry may now read another database where the
  write names `provider` or `path` or leaves a `sqlite` entry, and that it may
  now point at another server otherwise.
- **`database.provider` in every document.** The `database` object of the
  JSON documents and of the render context carries `provider`, `mariadb`,
  `mysql` or `sqlite`; a cache-served read carries the provider of the read
  that filled the cache. A `--context` document without the key is read as
  `mariadb`, and one with any other value exits `65`. `schema_version` and
  `cache_format` stay at `1`.
- **A `provider` row in `tpl schema info`.** The text output carries it
  between `collation` and `server`, for every provider.
- **The exit codes of a `sqlite` entry.** `74` when the path names nothing or
  the file cannot be opened or read; `78` when the path names something other
  than a regular file, the file is not a SQLite database, is damaged or holds
  a schema `tpl` refuses, the entry has no `path` or carries a key its
  provider does not take, or the file holds two tables or views — or two
  columns, indexes or triggers of one table — whose names differ only in bytes
  that are not valid UTF-8, which the document could not tell apart: no model
  is presented and nothing is cached; `69` when a writer keeps the database locked past
  `core.query_timeout`; `70` when a safeguard of the read does not take
  effect.
- **The write-ahead-log files of a SQLite database.** A read of a database in
  write-ahead-log mode may create `<file>-wal` and `<file>-shm` beside it,
  which may remain: the only files a read writes outside `.tpl`. Where they are
  absent and the directory refuses them, the read exits `74`.
- **Templates over a SQLite read.** The `numeric` test is true for the
  `data_type` values `integer`, `numeric` and `real`, and `primary_key` and
  `unique` answer from the table's `primary_key`, so a SQLite rowid key, which
  has no index, satisfies both. No answer over a MariaDB document changes.
- **MySQL as a third database provider.** An entry whose `provider` is
  `mysql` reads a MySQL server. It takes the keys of a `mariadb` entry —
  `dsn` or the discrete fields, `password` or `password_command`, `tls`,
  `ca_file` and `ca_path` — and refuses `path`; neither DSN scheme selects
  the provider. The eight `schema` subcommands, `tpl cache load`,
  `tpl render` and `tpl cfg database test` read its catalogue from twelve
  `INFORMATION_SCHEMA` tables, in a session set read-only and confirmed by
  reading `@@session.transaction_read_only` back. The supported series are
  MySQL's LTS series under Oracle Premier or Extended Support — `8.4` and
  `9.7` in this release — and no Innovation release.
- **`--provider mysql`** on `tpl cfg database add` and `update`, with the
  flags of a `mariadb` entry, and the value `mysql` for
  `tpl cfg set database.<name>.provider`. Every document a `mysql` read
  produces carries `database.provider` `mysql`, and a `--context` document
  carrying it is accepted.
- **The exit codes of a `mysql` entry.** `78` when the server reached is not
  MySQL, a MariaDB server included; when its series is older than the newest
  supported one and not supported, such as `8.0`; when the session cannot be
  made read-only; and when a login over an unencrypted session — `tls`
  `disabled`, or `preferred` against a server that offers no TLS — needs the
  server's RSA public key, which `tpl` never requests, with the `hint`
  `encrypt the session: tpl cfg database update <entry> --tls verify-identity`.
  A series newer than every supported one is read and marked
  `newer_than_supported`, at exit `0`. A `mariadb` entry that reaches a MySQL
  server is refused with `78`, as before.
- **What a `mysql` read carries.** A `CHECK` constraint's `level` and a
  view's `algorithm` are `null`, because MySQL records neither. The three
  `data_type` values only MySQL reports — `json`, `geomcollection` and
  `vector` — are decomposed like any other type; `json` satisfies the
  `textual` test, and the other two satisfy no family test. An expression
  default, a generated column's expression and a check clause are carried
  with one layer of MySQL's backslash escaping removed. An index with a
  functional key part is not carried.
- **An empty password for a `mysql` entry.** An entry whose password is the
  empty string — `password = ""`, a DSN whose password part is empty, or a
  `password_command` that prints nothing — and an entry that supplies no
  password log in to a MySQL account that has no password, as MySQL's own
  client does, whether or not TLS protects the session.

### Changed

- **Help text.** `tpl help cfg database add` and `update` document
  `--provider` and `--path`, with a SQLite example, and name `--path` among
  the flags each MariaDB flag excludes; the help of `--provider` names
  `mariadb`, `mysql` and `sqlite`, and `tpl help cfg database add` states that
  a MariaDB or a MySQL entry needs `--host` and `--schema` to connect; `tpl help`
  says that `tpl` reads a MariaDB or MySQL database or a SQLite database file,
  and defines a database entry, and the entry `-d/--database` selects, as a way
  to reach either; `--host`, `--port` and
  `--schema` name the MariaDB or MySQL server; `tpl help cfg database test`
  names which server answered, MariaDB or MySQL; and the `78` of every command
  that connects names a server that is not a supported version of the entry's
  provider; `tpl help cfg set` lists the
  two new keys, the provider's three values among them;
  `tpl help cfg database test` states what its four answers mean for a
  `sqlite` entry; the exit codes of the `schema` subcommands, `tpl render`,
  `tpl cache load` and `tpl cfg database test` name the conditions of a
  `sqlite` entry; and `tpl render --help` and the `context_variables` of
  `tpl help --format json` describe the `database` variable as "The whole
  database: name, charset, collation, provider, server, tables, views and
  routines."
- **Library API.** `tpl::Error` gained twelve variants, for the SQLite
  provider and its configuration, and nine for the MySQL provider and the
  classification of a failed server connection; `TlsHandshakeFailed` gained
  `provider`, `ReadOnlySessionNotEnforced` gained `read_back`,
  `ProviderIncoherentWrite` gained `own`, and `error::NetworkPhase` gained
  `Login`, `ReadOnlyStatement`, `ReadOnlyReadBack` and `VersionProbe`.
  `tpl::Error` also gained `DatabaseNameRefused`, `ValueHoldsNul` and
  `NulValueGiven`, with the new `error::NulField` and `error::NulPlace`, and
  `SqliteNamesAlike`, with the new `error::AlikeNames` and `error::AlikeKind`,
  and `ProjectFileNotDurable`. `tpl::model` gained `database::Provider`, with the values
  `Mariadb`, `Mysql` and `Sqlite`, `index::PrimaryKey` and the field
  `Database::provider`; `Table::primary_key` now returns a `PrimaryKey`
  rather than an `Index`; and fields a SQLite or a MySQL read cannot fill
  became `Option`, among them `Database::collation`, every view field but
  `name` and `definition`, and `CheckConstraint::level`. The library carries
  no stability promise (`DIV-032`).
- **A login that does not complete is reported as one.** For a `mariadb`
  entry, a connection that opens and fails before the login completes, and
  an answer that is not the MariaDB protocol, exited `69` with the message of
  a refused TCP connect. Both still exit `69`, now with
  `error: the login to <host>:<port> did not complete, for database entry '<entry>'`
  and a `cause` that names the login and which of the two happened. A login
  that needs the server's RSA public key over an unencrypted session also
  still exits `69`, now with
  `error: cannot authenticate user '<user>' over an unencrypted session, for database entry '<entry>'`,
  a `cause` naming the `tls` mode and the authentication method, and the
  `hint` `encrypt the session: tpl cfg database update <entry> --tls verify-identity`.
- **A failure after the session opened names that phase.** A version probe
  or a catalogue query that fails on an open session exited `69` with the
  message of a refused TCP connect, or of a failed TLS handshake where the
  TLS layer failed. It still exits `69`, now with
  `error: the version probe of <host>:<port> did not complete, …` or
  `error: a query reading the database structure from <host>:<port> did not complete, …`,
  a `cause` stating that the session was open and whether the connection
  failed, the answer did not follow the protocol, or the server answered
  with an error — naming its number — and the `hint`
  `the operation is read-only and can be run again; check the connection with: tpl cfg database test <entry>`.
- **A failed TCP connect names what the operating system reported.** Only a
  refusal, `ECONNREFUSED`, keeps
  `error: the server at <host>:<port> refused the connection, …` and its
  `cause` and `hint`, word for word. An unreachable host or network, a
  connect the operating system abandoned with no answer, and any other error
  now read `error: cannot reach <host>:<port>, for database entry '<entry>'`,
  with a `cause` naming which of those happened, or the error number, and a
  `hint` to check that the host is reachable or to change the address. Exit
  `69` throughout.
- **A failed TLS handshake is decided by the phase the connection reached.**
  Its two messages — a certificate that was not accepted, and a handshake
  refused before any certificate was checked — are unchanged. Every failure
  while the TLS handshake runs, a connection closed or reset during it
  included, is now reported as one of them; an I/O error outside the
  handshake no longer reads as a certificate not accepted; and a failure of
  the TLS layer on an open session is now a failed statement, above. Exit
  `69` throughout.
- **An error packet during the login is classified by its number.** Every
  packet but `1044` and `1049` on a connection naming a database exited `77`
  as a refused user or password. Now `1045` and `1698` keep that message;
  `1130`, `1862`, `4151` and `4150` (`3118` and `3955` for a `mysql` entry)
  exit `77` with a `cause` naming the reason — the client's host, an expired
  password, a locked account, an account blocked after failed logins — and a
  `hint` for it. **`3159` exits `78`, not `77`**: the server accepts only
  encrypted sessions, and the `hint` is
  `encrypt the session: tpl cfg database update <entry> --tls verify-identity`.
  **`1040`, `1203`, `1226` and `1129` exit `69`, not `77`**, with a `cause`
  naming the number and the limit on connections or the blocked host it
  reports; so does **every other number, and `1044` or `1049` on a connection
  that names no database**, as `tpl cfg database test` opens, with a `cause`
  stating that `tpl` does not classify the number further. `1044` and `1049`
  on a connection naming a database keep their `77` and message. **`1102`,
  a database name the server does not accept, exits `78`** on a connection
  naming a database: `error: the server at <host>:<port> refused the database name, …`,
  a `cause` naming the name between quotation marks, the key or the part of
  `dsn` that gives it and the file, and the `hint`
  `correct the database name: tpl cfg database update <entry> --schema <database>`.
- **`core.connect_timeout` bounds four phases, the login included.** An
  expired connection budget is reported against the phase the connection had
  reached — DNS resolution, the TCP connect, the TLS handshake or the login —
  where every expiry after DNS resolution was reported as the TCP connect.
  The `error` line reads
  `<phase> for <host>:<port> exceeded the connection budget of <value>, for database entry '<entry>'`,
  where it read `exceeded its own deadline of <value>`, and the `cause` states
  that DNS resolution, the TCP connect, the TLS handshake and the login share
  that budget. Exit `69`, unchanged.
- **A session statement that outlives `core.query_timeout` names itself.**
  The read-only session statement, its read-back and the version probe each
  have their own deadline of `core.query_timeout`. An expiry of one was
  reported as `a query reading the database structure`; it now reads
  `error: the version probe for <host>:<port> exceeded its own deadline of <value>, …`,
  or names the read-only session statement or its read-back, with a `cause`
  stating that `core.query_timeout` sets that deadline and bounds each
  statement on its own, and the `hint`
  `raise the deadline with: tpl cfg set core.query_timeout <seconds>`. An
  expired catalogue query's `cause` states the same. Exit `69`, unchanged,
  for every one of them, the read-only pair included.
- **`-v` reports each connection phase and each session statement.** For a
  `mariadb` and a `mysql` entry, `-v` writes one `phase:` line for DNS
  resolution, the TCP connect, the TLS handshake where the `tls` mode
  negotiated one, the login and each of the three session statements, each
  naming it and its time, a phase that failed or expired included. The TCP
  connect's line no longer carries the time of the TLS handshake and the
  login, and the session statements are no longer reported as `server
  query`. The `query:` lines, one per catalogue query, are unchanged.

- **The worked examples cover MySQL and SQLite.** Each of the four examples
  also renders, and passes its compile gate over, the data layer of the MySQL
  fixture's `freight` and of the SQLite fixture's main database, built from
  `scripts/sqlite/freight.sql` on every run; each type mapping covers `json`,
  `geomcollection`, `integer`, `real` and `numeric`. A table with no primary
  key gets its row types and no repository, and the MariaDB output of every
  example is unchanged.
- **A cached `tpl render` reads a table's `foreign_keys` and
  `referenced_by` when the template first reaches them.** A cache file whose
  `foreign_keys` or `referenced_by` is damaged is a miss only where the
  template reaches that part: the render is then abandoned, nothing of it is
  written, and the result is rendered from the server, as for any file found
  damaged during a render. A template that never reaches the part is served
  from the cache. The eight `schema` subcommands still treat such a file as a
  miss before they write anything.

### Fixed

- **A cached `tpl render` of a large database holds about half the heap it
  held.** Every table a render reached from the cache decoded the tables its
  foreign keys embed, so under the default `core.render_memory_limit` a
  whole-database render such as `rust/schema` or `go/schema` exited `65`
  from about 2 450 tables cached, while the same render with `--direct`
  passed. It now passes up to about 4 600 tables; past that a cached render
  still exits `65` where `--direct` passes.
- **A short or malformed packet from a server no longer ends `tpl` with
  `70`.** A greeting cut before its fields, an error packet cut before its
  number or its SQL state — in the greeting's place or in answer to the
  login — and a one-byte `0x01` packet in a `caching_sha2_password` login
  made the driver panic. Each now exits `69` with
  `error: the login to <host>:<port> did not complete, …` and the `cause` of
  an answer that is not the server's protocol. A server that sends
  continuation packets past 1 GiB is refused the same way, where `tpl` kept
  reading them into memory.
- **`tpl cfg database test` reads only a privilege error as
  `can_read_catalogue: false`.** An error packet carrying `1044`, `1142`,
  `1143` or `1227` answers the privilege probe with `false`, exit `0`, as
  before. Any other error packet answered it with `false` too; it now exits
  `69`, naming the error number, and writes no document.
- **A NUL character in an entry is refused.** A `host`, `user`, `password`
  or `database`, a part of a `dsn`, a `sqlite` `path`, or the output of a
  `password_command` holding a NUL character was cut at it, so a login
  could select another database than the one written. In `.tpl/.cfg` such a
  value now exits `78` before any connection is opened, the `cause` naming the
  key or the part of `dsn`, the file and the text before the NUL — never any
  part of a password — and the helper's output exits `78` naming the command.
  Given to `tpl cfg set`, or to `--dsn`, `--host`, `--user`, `--schema` or
  `--path` of `tpl cfg database add` and `update`, it exits `64` and writes
  nothing.
- **A read served from the cache needs no credential.** A `${VAR}` in `dsn`,
  `host`, `port`, `user`, `password` or `database` is expanded, and
  `password_command` run, only by an invocation that opens a connection: a
  read that misses the cache or carries `--direct`, the read a render returns
  to, `tpl cache load` and `tpl cfg database test`. A read the cache serves
  exited `78` when such a variable was unset; it now succeeds, with
  `"source":"cache"`, and the `78` is raised by the first invocation that
  connects.
- **An include suggestion names another template, or none.** A name an
  `{% include %}`, `{% import %}` or `{% extends %}` writes that does not exist
  is never answered with the template that holds it, and one that names a FIFO,
  a socket or a device is answered with no suggestion, as `tpl render`,
  `tpl template show`, `check` and `path` given such a name already were: an
  include of a FIFO `h.jinja` from `i.jinja` suggested `'i.jinja'`.
- **An account with no password logs in with an empty password.** A
  `mariadb` entry whose password is the empty string — `password = ""`, a DSN
  whose password part is empty, or a `password_command` that prints nothing —
  was refused by such an account with `77`, because `tpl` sent a scramble
  computed from the empty string. It is now presented as MariaDB's and
  MySQL's own clients present an empty password, and so is an entry that
  supplies no password when the server switches the authentication method.

### Security

- **A rewrite of `.tpl/.cfg` acts only in the `.tpl` the trust checks
  judged.** The five commands that rewrite the file start from the `.cfg` text
  the ownership and mode checks read, and create the temporary file, write it,
  give it mode `0600` and rename it over `.cfg` relative to the `.tpl` folder
  those checks opened and held, following no link; a folder without `.cfg` is
  judged on that same held folder. A link or a folder put in place of a
  component of its path after the checks therefore redirects none of those
  steps — another project's `.cfg` could be rewritten before — and a link at
  `.cfg` is replaced rather than written through. A rewrite that cannot act in
  that folder exits `74`, leaving the previous file in place.
- **A rewrite of `.tpl/.cfg` survives a crash.** The temporary file is
  `fsync`ed before it is renamed over `.cfg`, and the `.tpl` folder after the
  rename, so a power loss leaves the previous file or the new one, whole. A
  failure up to and including the rename exits `74` with the previous file in
  place, as before. A failure after it exits `74` with
  `error: .tpl/.cfg was written, and its durability could not be confirmed`, a
  `cause` stating that the file holds the new content and what the filesystem
  returned, and a `hint` carrying `tpl cfg list`.
- **C1 controls and bidirectional formatting characters are escaped.** In
  the text output of the read commands and in every diagnostic, a character
  from U+0080 to U+009F, and each of the twelve characters Unicode marks
  `Bidi_Control`, is now written as `\u{…}`, as a C0 control already was; in
  JSON output it is written as `\u` and four hexadecimal digits. This holds
  for every provider, so a name in a catalogue can no longer reorder what a
  terminal shows.
- **Dependencies moved to releases with security fixes.** `rustls` 0.23.46,
  `rustls-webpki` 0.103.16, `webpki-roots` 1.0.10 and `tokio` 1.53.3,
  which close RUSTSEC-2026-0346, RUSTSEC-2026-0347, RUSTSEC-2026-0348,
  RUSTSEC-2026-0349, RUSTSEC-2026-0381, RUSTSEC-2026-0382 and
  RUSTSEC-2026-0386.

## [0.0.2] - 2026-09-25

### Added

- **The Claude Code skill.** `skill/` makes Claude the sole operator of `tpl`
  for Claude Code agents. It is installed by `install-skill.sh`, or from a
  clone by linking or copying the folder into `~/.claude/skills/tpl`, and
  `skill/scripts/check-coverage.sh` checks its command map against
  `tpl help --format json`.
- **The skill installer and its release asset.** A release now carries
  `tpl-skill-<tag>.tar.gz`, listed in `SHA256SUMS`, and
  `curl -fsSL https://raw.githubusercontent.com/FlavioCFOliveira/tpl/main/install-skill.sh | sh`
  installs or updates the skill from it into `~/.claude/skills/tpl` — under
  `CLAUDE_CONFIG_DIR` when set, or `TPL_SKILL_DIR` — replacing an existing skill
  or symbolic link there.
- **`tpl::run_from(args)` in the library.** It runs `tpl` on an explicit
  argument vector instead of the process arguments, so a fuzz harness can drive
  `tpl` in process; `tpl::run` is `run_from` over the process arguments. The
  library API carries no stability promise (`DIV-032`): the contract is the
  command line and the JSON document.

### Changed

- **Publishing a release.** A pushed `v*` tag no longer publishes a release by
  itself: `release.yml` is started with
  `gh workflow run release.yml --ref vX.Y.Z`, and it refuses a ref that is not
  a tag.
- **Help banner.** Every text help — `tpl help`, `tpl help <path>` and
  `-h`/`--help` at every node — now opens with
  `tpl v<version> - Code Generation based on database schema` and one empty
  line. `tpl help --format json` and `tpl version` are unchanged.
- **`tpl cfg set core.database` requires the entry to exist.** A name no entry
  of `.tpl/.cfg` declares is refused with `66`, writing nothing, and the `hint`
  suggests the nearest declared entry.
- **`tpl cfg unset` of an absent block suggests the nearest one.** For `core`,
  `database` or `database.<name>`, the `hint` names the nearest block the file
  carries, and for an entry's block, `tpl cfg database show <name>`.
- **Breaking: the lookup functions are renamed.** In the registered template
  surface, `table(name)`, `view(name)` and `routine(name)` are now
  `table_named(name)`, `view_named(name)` and `routine_named(name)`; `column`
  is unchanged. A template that calls an old name fails with `65`. It is a
  breaking change of the registered template surface, released in `0.0.2` by
  the maintainer's decision rather than as a minor bump.
- **`.tpl/.cfg` must be a regular file.** A `.cfg` that is a symbolic link, a
  directory, a FIFO, a socket or a device is refused with `78`, naming the
  kind found and reading nothing, so a FIFO no longer makes `tpl` block. **A
  `.cfg` kept as a symbolic link, which was checked at its target, is now
  refused:** replace it with a regular file, or keep the project elsewhere and
  name its `.tpl` folder with `--tpl-dir`.

### Fixed

- **A template that is not a regular file is never opened.** A `.jinja` entry
  of `.tpl/templates/` that is a FIFO, a socket or a device is not listed or
  checked, is refused with `66` when named on the command line, and fails a
  render with `65` when an `include`, `import` or `extends` names it; a FIFO
  no longer makes `tpl` block.
- **Cache operations do not follow a `.tpl/.cache` swapped for a link
  mid-operation.** Every cache path is resolved relative to the directory
  already opened, so a component replaced by a symbolic link between two steps
  fails the next step instead of redirecting a read, write or removal outside
  the project.
- **`table`, `view` and `routine` are undefined without an object flag.** A
  `tpl render` given no `--table`, `--view` or `--routine` no longer sees them
  as defined: they resolved to the lookup functions of the same names.
- **`tpl render --context` refuses a document of another `schema_version`.**
  A document whose `schema_version` is not the one this `tpl` emits exits
  `65`, rendering nothing, and the `hint` carries `tpl -d <entry> schema dump`;
  it was rendered.
- **A backslash-newline in a `password_command` string is a line
  continuation.** Given to `tpl cfg set database.<name>.password_command` or
  to `--password-command`, a backslash followed by a newline outside single
  quotes is removed before the string is split into words, as POSIX does; it
  was kept.
- **`tpl cache clean` removes nothing through a symbolic link.** In every
  form, a linked `.tpl/.cache`, and for `--table`, `--view` or `--routine` a
  linked `.tpl/.cache/<name>` or collection folder, is refused with `78`,
  nothing removed, and a `hint` that removes the link alone; a link that is
  itself the folder or file removed is removed as a link, and no link beneath
  a removed folder is followed.
- **A partial `tpl cache clean` keeps `loaded_at`.** With `--table`, `--view`
  or `--routine`, it only marks the object's collection as not loaded whole in
  `meta.json`, and leaves an absent or unusable `meta.json` as it found it.
- **No cache path is read or written through a symbolic link.** The eight
  `schema` subcommands, `tpl render` without `--context`, `tpl cache load` and
  `tpl cache status` refuse a linked `.tpl/.cache`, entry folder or collection
  folder with `78`, reading, writing and connecting to nothing;
  `--direct --no-cache` still reads the server. `meta.json` and
  `database.json` are read only as regular files within a size bound, and a
  full write replaces a link there rather than following it.
- **`tpl cache status` no longer calls a cache with objects empty.** With
  `meta.json` absent or unusable beside cached objects, it reports
  `loaded_at` `null` and all three collections with their counts and `whole`
  `false`.

## [0.0.1] - 2026-09-24

The first release of `tpl`. No earlier version was ever released or tagged, so
this entry covers **the whole of the project's history up to the release** —
everything built from the first commit to `0.0.1`. It carries only `Added` for
that reason: with no previous release, there was nothing to change, deprecate,
remove, fix or secure.

### Added

- **The command surface.** Eight subcommands — `schema`, `template`, `render`,
  `cache`, `cfg`, `init`, `help` and `version` — over a closed tree of 34 nodes,
  29 of them invocable. Short aliases on the `schema` leaves (`tbls`, `tbl`,
  `vws`, `vw`, `rtns`, `rtn`) and on `cfg database` (`db`).
- **Seven global flags**, accepted in any position: `-d`/`--database`,
  `--tpl-dir`, `--timeout`, `-v`/`--verbose`, `-q`/`--quiet`, `-h`/`--help` and
  `-V`/`--version`.
- **Help and version.** Help renders seven sections at a fixed width of eighty
  columns, for every node of the tree. `tpl help --format json` publishes the
  command tree, the global flags and the template surface as one JSON document.
  `tpl version` prints `tpl 0.0.1`.
- **Projects.** `tpl init` creates a `.tpl` project. Commands find one by walking
  up from the working directory as far as the mount point; `--tpl-dir` names the
  `.tpl` folder itself and suppresses the walk, and has no effect on `init`,
  `help` and `version`. `.tpl/.cfg` is checked for ownership and
  permissions, at the target of a symbolic link, before it is read.
- **Configuration.** `cfg get`, `cfg set`, `cfg unset` and `cfg list` read and
  write `.tpl/.cfg`, preserving its comments and its ordering.
  `cfg database` adds, lists, shows, updates, removes and tests the connection
  entries, through `--dsn`, `--host`, `--port`, `--user`, `--schema`, `--tls`,
  `--ca-file`, `--ca-path` and `--password-command`.
  `password_command` runs as the leader of a process group of its own, and its
  phase ends only when it has exited and its standard output has ended: at
  `core.password_timeout`, or when it writes more than 4096 bytes, the whole
  group is ended, descendants included, and the invocation exits `78` — a helper
  that exits `0` while a descendant holds its output open no longer hangs the
  invocation.
- **Testing an entry.** `cfg database test` contacts the server an entry
  describes and reports four steps in one order: the connection and its
  authentication, the read-only session and its confirmation, the server's
  series against the supported window, and a probe of the reader's catalogue
  privileges. Exit `0` says the four steps ran, not that the entry is usable:
  `can_read_catalogue` in the document is what says whether a read through it
  would be complete. It is the one `cfg` subcommand that opens a connection, and
  the last leaf of the command tree to be written — every node of the tree now
  has an implementation.
- **Reading the catalogue.** One connection, opened as late as possible and
  closed when the read ends, with the read-only session enforced before any
  statement. The catalogue is read from `INFORMATION_SCHEMA` in a fixed
  repertoire of eleven statements. `tpl` never writes to the database.
- **The model.** Databases, tables, columns with their decomposed type and their
  classified default, generated columns, indexes, foreign keys in both
  directions, check constraints, views, routines with their parameters, triggers
  and the server. What the reader's privileges did not reach is marked in the
  model rather than silently omitted.
- **Schema commands.** `schema info`, `tables`, `table`, `views`, `view`,
  `routines`, `routine` and `dump`, with `--pattern` on the listing commands.
- **The catalogue cache.** `cache load`, `cache clean` and `cache status`. A
  cached document carries `schema_version` and `cache_format`, both at `1`; a
  document that cannot be decoded is a miss, and the read goes to the server.
  `--no-cache` and `--direct` bypass the cache on the commands that consult it.
  A cache write does not force its files to disk, so a file a crash leaves torn
  is that same miss; `cache status` counts the object files held without
  reading them, so a file that cannot be read is still counted. A cached
  `schema info` reads `database.json` and counts the object files, and the
  text listing of `schema tables` decodes four members of each table file; an
  object file that fails only in what they no longer decode is not a miss for
  them, and remains one for every other read. An object file that is a symbolic
  link is a miss for every read and is never read through, and a read of one
  named table, view or routine whose file holds another object — another kind,
  or a name that differs in any byte, as on a filesystem that folds case — is a
  miss, so the read goes to the server rather than answering that the object
  does not exist.
- **Templates.** `template list`, `template show`, `template check` and
  `template path`.
- **The render environment.** MiniJinja, with templates loaded and compiled at
  run time and never embedded at compile time. The registered surface is a
  contract of twenty-three names:
  - filters — `pascal`, `camel`, `snake`, `upper_snake`, `kebab`, `quote`,
    `sql_type`, `json`, `indent`, `comment`, `escape`;
  - tests — `nullable`, `primary_key`, `auto_increment`, `unique`, `numeric`,
    `temporal`, `textual`;
  - functions — `table`, `view`, `routine`, `column`, `fail`.

  Fourteen further filters inherited from the engine are pinned to its version
  rather than guaranteed by `tpl`.
- **Rendering.** `tpl render <TEMPLATE>` renders one template, once, to stdout.
  The context comes from the selected database or from `--context`, never from
  both; `--table`, `--view` or `--routine` binds one object, and a repeatable
  `--set KEY=VALUE` passes template variables. A `--context` document in which
  a foreign key names a table its `tables` does not carry exits `65` naming the
  table, the key and the table it names. Every render is bounded, besides its
  deadline, by render fuel (`core.render_fuel`, default 100 000 000 evaluation
  steps), by the render output limit (`core.render_output_limit`, default
  64 MiB) and by the render memory limit (`core.render_memory_limit`, default
  128 MiB, the heap the process holds while the render runs, observed every
  10 ms); the first crossed exits `65` naming the bound, its value and the key,
  and a render stopped by either limit writes nothing to stdout. When a render
  reads the server, the connection is closed and the driver's runtime shut down
  before the template starts.
- **Output.** Every JSON document shares one envelope, carrying `schema_version`,
  `source` and `data`. `--format json` and `--pretty` select it on the commands
  that emit a document.
- **Diagnostics.** Exit codes drawn from the `sysexits` set, one per condition;
  nearest-match suggestions on an unknown name; verbosity gated by `-v` and `-q`;
  and a panic hook that reports a defect in `tpl` as exit `70`.
- **Help and error texts a caller can act on alone.** Every help text and
  every error message states what failed, why, and a command that can succeed.
  - Help: every leaf's `DESCRIPTION`, and its JSON `description`, ends with
    four statements — whether it contacts the server, which database entry it
    needs, which files it writes, and what it prints. `tpl help render` lists
    the context variables and every guaranteed filter, test and function with
    its signature and purpose.
  - `tpl help --format json`: each `template_surface` name is an object with
    `name`, `signature`, `operand`, `arguments` and `purpose`; `data` gains
    `context_variables`; a flag `default` is `null` or one JSON string of the
    command-line text, so `--port` shows `"3306"`.
  - `cfg get`, `cfg list` and `cfg database show` with `--format json` keep
    TOML types: an integer is a number, `password_command` an array.
  - New refusals: `--tpl-dir` naming a directory that is not a `.tpl` folder
    exits `78`, and the `hint` names the `.tpl` folder it holds; a `.tpl`
    folder without `.cfg` not owned by the caller exits `78`; `tpl init` into
    a `.tpl` folder exits `64`; `tpl render --context --direct` exits `64`;
    `tpl cfg database update` with no field flag exits `64`; `tpl cfg get` of
    a block exits `64`; `tpl cache clean` naming an object that is not cached
    exits `66`.
  - New refusals of values, each `64` on the command line with nothing
    written, and `78` in `.tpl/.cfg` where the value can stand there: a `password_command` string that yields no word,
    leaves a quote unclosed, ends in a backslash or begins with `[`; `${` in
    `ca_file` or `ca_path`; an entry name, including `core.database`, outside
    `[A-Za-z0-9_]{1,64}`; a `${NAME}` reference whose name is not
    `[A-Za-z_][A-Za-z0-9_]*`; an empty or whitespace-only host or database.
    `${VAR}` in `password_command` is stored and passed to the program as
    written.
  - `tpl template check` reports every failing template, one message each,
    before exiting `65`.
  - `tpl -d <name> cache clean` removes `.tpl/.cache/<name>/` of a name no
    entry declares.
  - New stderr warnings, exit code unchanged, suppressed by `-q`: `--tpl-dir`
    given to `tpl init`; `-d/--database` given to `cfg database add` or
    `update`, naming `--schema` or `--dsn` where one would act; `tpl cfg unset` of a `dsn`, naming
    what it carried; and, naming `tpl -d <name> cache clean`, each entry
    removed or repointed by `cfg database remove`, `cfg database update`,
    `cfg set` or `cfg unset`, since any data cached for it is kept.
  - Hints: a `tpl` command in a `hint` carries the invocation's `--tpl-dir`
    and `-d`; no `hint` deletes or overwrites a value or cached object the
    invocation did not name, so an entry defined by `dsn` is changed through
    `--dsn`; a rewritten command keeps the whole invocation; a prefix of a
    command (`tpl sch`) and an undefined value in a render get a
    nearest-match suggestion; a suggestion must share a character with the
    name; `-d` or `--tpl-dir` that took a command name reports that the flag
    needs a value.
  - Library API: `tpl::Error` gained eighteen variants, and thirty-two
    existing variants gained fields; `LoadWithoutStoring` became a struct
    variant. `Error` is `#[non_exhaustive]`, so a `match` with a wildcard arm
    still compiles, but code that constructs or destructures a changed
    variant without `..` does not. In `tpl::error`, `ContextFault::Structure`
    replaced `rule` with `at` and `expected`, `ContextFault` gained `Empty`,
    and `EntryRepair::Rewrite` was replaced by `InsideDsn`,
    `DsnWithoutPassword` and `Discrete`; neither enum is `#[non_exhaustive]`.
- **Worked examples.** `examples/` holds four complete demonstrations that build
  an application's data layer — in Go, Rust, Python and Node.js — from three
  known schemas through the command line alone, each ending in a compile gate
  that submits every file it rendered to that language's own toolchain.
- **The shared example driver.** `examples/_driver/` runs the five-command
  workflow once for all four examples, checking every exit code, keeping stdout
  and stderr apart, and redirecting each render's bytes into a file unchanged.
- **The published datasets.** `sakila` and `world` are vendored verbatim under
  `scripts/mariadb/datasets/`, each with the source URL, date and checksums it
  was taken at, and `scripts/mariadb/seed-datasets.sh` loads them into a named
  fixture server, re-grants the reader on them, and verifies what arrived.
- **The install script.** `install.sh` installs and updates `tpl` in one
  command,
  `curl -fsSL https://raw.githubusercontent.com/FlavioCFOliveira/tpl/main/install.sh | sh`,
  into `/usr/local/bin` or the directory `TPL_INSTALL_DIR` names, after
  verifying the archive against the release's `SHA256SUMS`.
- **Release archives.** Pushing a `v*` tag publishes a GitHub Release carrying
  one `tpl-<tag>-<triple>.tar.gz` per supported target and a `SHA256SUMS` file
  covering them, only if the tag is annotated, on `main`, equal to the
  `Cargo.toml` version, and has its release notes, and the tagged commit passes
  validation. A pre-release tag publishes a GitHub pre-release, which
  `install.sh` never installs. The archives are not signed.

## The record before this file

This file begins at the present and does not reconstruct what preceded it.
Reconstructing it would mean writing version headings for releases that never
happened, and the version number has one source, which no document may
duplicate. The detailed record of the work already exists, and is authoritative
where this file is silent:

- **`git log`** — every change as it was made, with each sprint merge naming the
  work it closes.
- **The `tpl` roadmap in `rmp`** — the sprints and their tasks, each closed task
  naming the commit that records it, and each carrying the decisions, findings
  and tests logged while the work happened.
- **[`specification/README.md`](specification/README.md)** — the functional
  specification's per-edition ledger, each edition stating what it added or
  amended and on what evidence.
- **[`docs/adr/`](docs/adr/)** — the architecture decision records, each with the
  alternatives it rejected.
