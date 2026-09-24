---
title: Configuration Model
status: approved
last-reviewed: 2026-09-24
related: [cfg-commands.md, global-flags.md, project-and-discovery.md, security.md]
---

# Configuration Model

## Overview

`.tpl/.cfg` is the only configuration `tpl` reads. It is a TOML file in the
project's `.tpl` folder, holding project options and the database entries
available to the project. This file defines its key space, the type and default
of each key, how `${VAR}` expands, how a DSN is parsed, and how a password is
obtained without being written to disk.

## Scope

In scope: the `.cfg` key space, value types and defaults, the strictness with
which the file is read, entry shape, what an entry must describe for a
connection and a read to be possible, DSN grammar, the five TLS modes and the
behaviour by which they are distinguished, the trust material the two
verifying modes read and how the directory holding it is resolved, timeouts,
the keys that set the render bounds, `${VAR}` expansion, and `password_command`
execution and its failure modes.

Out of scope: the commands that read and write the file, which belong to
[cfg-commands.md](cfg-commands.md); and the ownership and permission checks
applied before the file is trusted, which belong to
[project-and-discovery.md](project-and-discovery.md).

## Actors

- **Project**, whose `.tpl/.cfg` is the only configuration `tpl` reads.
- **Environment**, read only through `${VAR}` and treated as untrusted.
- **`password_command` child process**, which supplies a password without it
  being written to disk.

## The file

```toml
[core]
database = "shop"

[database.shop]
dsn = "mysql://alice:${SHOP_DB_PASSWORD}@db.example.com:3306/shop"

[database.reporting]
host     = "10.0.1.5"
port     = 3306
user     = "reader"
database = "reporting"
password_command = ["security", "find-generic-password", "-s", "tpl-reporting", "-w"]
tls      = "verify-identity"
```

- **FR-CONF-001**: `.tpl/.cfg` SHALL be a TOML document. Its name is `.cfg` — a
  leading dot, no extension — because it is meant to stay out of version
  control.

- **FR-CONF-002**: The system SHALL recognise exactly the following keys:

  | Key | Type | Default |
  |---|---|---|
  | `core.database` | entry name | none |
  | `core.connect_timeout` | seconds, positive integer | `10` |
  | `core.query_timeout` | seconds, positive integer | `30` |
  | `core.password_timeout` | seconds, positive integer | `5` |
  | `core.render_timeout` | seconds, positive integer | `30` |
  | `core.render_fuel` | evaluation steps, integer from `1` to `1000000000000` | `100000000` |
  | `core.render_output_limit` | bytes, integer from `1` to `1099511627776` | `67108864` |
  | `core.render_memory_limit` | bytes, integer from `8388608` to `1099511627776` | `134217728` |
  | `database.<name>.dsn` | connection URL | none |
  | `database.<name>.host` | hostname or address | none |
  | `database.<name>.port` | TCP port | `3306` |
  | `database.<name>.user` | user name | none |
  | `database.<name>.password` | password | none |
  | `database.<name>.password_command` | argument array | none |
  | `database.<name>.database` | server-side database name | none |
  | `database.<name>.tls` | TLS mode, see `FR-CONF-013` | `verify-identity` |
  | `database.<name>.ca_file` | path to a certificate file | none |
  | `database.<name>.ca_path` | path to a certificate directory | none |

  *Provenance.* The default port `3306` comes from the root `README.md` and is
  contradicted by no decision.

  *Amended in the forty-second edition.* The three rows `core.render_fuel`,
  `core.render_output_limit` and `core.render_memory_limit` are new, as
  decided for rmp `#255`. They set the render bounds of `FR-RND-036`,
  `FR-RND-037` and `FR-RND-039`, and `FR-CONF-045` states how they are
  resolved and why their defaults and ranges are what they are. The key space
  grows from fifteen key forms to eighteen, eight of them under `[core]`.

- **FR-CONF-003**: There SHALL be no global configuration. The system SHALL NOT
  read configuration from the home directory, from an XDG location, or from
  `/etc`.

- **FR-CONF-004**: The system SHALL resolve each phase deadline from the
  `[core]` key `FR-CONF-005` names for that phase, or from the built-in default
  declared for that key in `FR-CONF-002` where the key is absent. `--timeout`
  SHALL NOT participate in this resolution; it is an overall bound that
  composes with the result, per `FR-GLOB-011` and `FR-GLOB-012`.

  *Amended in the third edition.* The first edition put `--timeout` at the head
  of this precedence, which made the four `[core]` timeout keys unreachable
  because `FR-GLOB-001` gave the flag a default. The flag now has no default
  and is not a layer of this rule.

  *Amended in the ninth edition.* The rule said "the `[core]` key for that
  phase" against six phases and four keys, which left three phases with no
  referent. The mapping is now stated, once, in `FR-CONF-005`, and this rule
  points at it rather than implying a key per phase.

- **FR-CONF-005**: The system SHALL apply a deadline to every blocking phase:
  DNS resolution, TCP connect, TLS handshake, catalogue query,
  `password_command`, and render. Each phase SHALL take its deadline from the
  key named beside it:

  | Phase | Key of `FR-CONF-002` |
  |---|---|
  | DNS resolution | `core.connect_timeout` |
  | TCP connect | `core.connect_timeout` |
  | TLS handshake | `core.connect_timeout` |
  | Catalogue query | `core.query_timeout` |
  | `password_command` | `core.password_timeout` |
  | Render | `core.render_timeout` |

  The three connection phases SHALL share **one** budget of the resolved value
  of `core.connect_timeout`, measured from the start of the first of them that
  runs and consumed by them in the order they run. The system SHALL NOT give
  each of the three a budget of that value. The other three phases each have a
  budget of their own. IF the shared budget expires, THEN the system SHALL
  report the failure against the phase that was in progress when it expired,
  per `FR-ERR-034`.

  *Note added in the fortieth edition.* Under `FR-CACHE-038` the files a render
  reads lazily are read within the render phase and under its deadline. A
  render abandoned under `FR-CACHE-039` and the render that follows it each
  have the whole budget of `core.render_timeout`. The server read between them
  runs under the connection and catalogue-query deadlines above. The overall
  budget of `FR-GLOB-011` still bounds the whole invocation.

  *Note added in the fifth edition.* The DNS phase is the one phase whose
  behaviour depends on how the binary was linked. The Linux targets of
  `NFR-PERF-018` are statically linked against `musl`, whose static
  `getaddrinfo` does not load the platform's name-service modules, so a name
  resolvable only through such a module does not resolve. The deadline and the
  outcome are unchanged — the phase fails and `FR-ERR-027` routes it to `69` —
  but the `cause` line owes the caller the distinction, per `FR-ERR-034`: a
  name that did not resolve is not a host that refused a connection.

  *Amended in the ninth edition: the six phases are mapped onto the four keys,
  because three of them had no referent.* `FR-CONF-004` resolves each phase
  deadline from "the `[core]` key for that phase" and `FR-CONF-002` declares
  four timeout keys against these six phases, so DNS resolution, TCP connect
  and TLS handshake were left with no key of their own and one candidate
  between them. Two readings were available: three independent deadlines of
  `core.connect_timeout`, whose sum is three times the value the caller set; or
  one budget of that value shared by the three. This corpus takes the second,
  and now says so in its own text. It is what the key's name states — a caller
  who writes `connect_timeout = 10` is stating how long connecting may take —
  and under the first reading the configured value could not bound the thing it
  is named after. Nothing about which phases exist, which of them can fail, or
  which code a failure produces changes, and the obligation that no phase run
  unbounded held under either reading. `FR-GLOB-012` composes the shared budget
  with `--timeout` exactly as it composes the other three.

- **FR-CONF-045**: The system SHALL resolve the render fuel of `FR-RND-036`
  from `core.render_fuel`, the render output limit of `FR-RND-037` from
  `core.render_output_limit`, and the render memory limit of `FR-RND-039` from
  `core.render_memory_limit`, or from the built-in default `FR-CONF-002`
  declares for each where the key is absent. No flag and no environment
  variable SHALL set any of them, and `--timeout` SHALL NOT participate. IF
  `.tpl/.cfg` carries any of the three keys with a value that is not an integer within the
  range `FR-CONF-002` declares for it, THEN the system SHALL exit `78`
  (`EX_CONFIG`), and the `cause` SHALL name the key, the file, the value found
  and the range expected, per the `78` row of `FR-ERR-034`. `tpl cfg set`
  refuses the same values with `64`, per `FR-CFG-010`.

  *Added in the forty-second edition, as decided for rmp `#255`.* The three
  keys follow the pattern of the four deadline keys above: a `[core]` key, a
  built-in default, and no layer above the file, per `FR-CONF-029` and
  `FR-CONF-030`.

  *Rationale for the fuel and output defaults.* Both are set where no
  legitimate render reaches them. `WL-001`, the large reference workload of
  [performance-requirements.md](performance-requirements.md), carries 2 400
  columns across 200 tables. A whole-database render that spends a thousand
  evaluation steps on every column — far more than generating one field of a
  struct takes — consumes 2 400 000 steps, about one fortieth of the render fuel
  default of 100 000 000; a database ten times that size still stays within a
  quarter of it. The same render emitting a kilobyte per column produces about
  2.4 MB, under a twentieth of the output default of 64 MiB (67 108 864
  bytes). A generated source file is measured in kilobytes, so 64 MiB is far
  beyond any file a code generator legitimately writes, and a runaway render
  stops long before it fills a disk.

  *Why the output default sits below the memory default.* The render holds its
  output in memory until it ends, because `FR-RND-034` with `FR-CACHE-039`
  forbids any byte of an abandoned render reaching stdout. Held output is heap
  the process holds, so it counts toward the render memory limit of
  `FR-RND-039`. With the output default at or above the memory default, a
  render writing without end would cross the memory limit first and be
  reported under the wrong cause. At 64 MiB, half the memory default of
  128 MiB, the output limit is reached first and names the fault truly.

  *Amended within the forty-second edition.* The output default was first
  256 MiB (268 435 456 bytes). The user lowered it to 64 MiB for the reason
  above; the range is unchanged.

  *Rationale for the memory default.* The default of 128 MiB (134 217 728
  bytes) is the user's decision. Its base is recorded in `BENCHMARKS.md`, in
  the entry of 2026-09-23 on the peak resident memory of every render the four
  worked examples perform: the largest is 6 094 848 bytes, `rust/schema` over
  the whole `freight` database. The user first set the rule at four times that
  peak, 24 379 392 bytes, and then chose 128 MiB, about twenty-two times the
  peak, as the default. The reading is informative, per `BR-PERF-008`; the
  default rests on the decision, not on the figure.

  *Rationale for the ranges.* The lower bound of render fuel and of the
  render output limit is `1`, so a test can set a small value and reach either
  bound on purpose. The lower bound of the render memory limit is 8 MiB
  (8 388 608 bytes), not `1`: the process holds heap before the template runs,
  and a limit below that would end even a trivial render. 8 MiB is above the
  whole resident memory of the largest render the worked examples perform, so
  a test can still set it low enough to reach the bound on purpose with a
  template that grows memory. No key admits `0` or any value meaning "no
  bound": a key that can remove a bound reopens the surface the bound closes,
  which is the argument `FR-CONF-031` makes for its cap. The upper bounds —
  1 000 000 000 000 steps, ten thousand times the fuel default; 1 TiB, 16 384
  times the output default; and 1 TiB, 8 192 times the memory default — leave
  room for any legitimate need and keep every admitted value a bound rather
  than a way of disabling one.

  *Why keys, when `FR-CONF-031` fixes its cap as a constant.* A password has a
  size that does not grow; a legitimate render grows with the database it
  renders. A constant would one day refuse a correct render and leave the
  caller no remedy, and a key raises the bound for the one project that needs
  it without widening it for any other.

  *Rejected: a flag.* `FR-GLOB-001` fixes seven global flags, and the four
  deadline keys have no flag either. A render bound describes what a project's
  templates need, which is a property of the project and belongs in its
  `.tpl/.cfg`.

## Strictness of the file

- **FR-CONF-034**: IF `.tpl/.cfg` contains a key outside the enumerated space
  of `FR-CONF-002`, anywhere in the file, THEN the system SHALL exit `78`
  (`EX_CONFIG`) with a nearest-match suggestion over the known keys, per
  `FR-ERR-021`.

  *Rationale.* `FR-CFG-009` already refuses an unknown key supplied to
  `tpl cfg set` with `64`, and the file may be hand-written. Ignoring a key the
  file contains would make a misspelled `passwrod_command` a silent no-op: the
  entry would authenticate as though no password source had been configured,
  and the failure would surface as a `77` naming the credentials rather than
  the typo. Refusing names the typo, and the suggestion corrects it.

  *Accepted cost, and it is the substantial one.* An older `tpl` cannot read a
  project written by a newer one. The moment a release adds a key,
  every binary released before it refuses the whole file with `78` rather than
  ignoring the key it does not know. Forward compatibility of the
  configuration is given up deliberately, and in exchange every key in a `.cfg`
  is a key that is in force. A project that must be read by two versions of
  `tpl` keeps its `.cfg` within the key space both recognise.

- **FR-CONF-035**: IF `password_command` in `.tpl/.cfg` is not a TOML array of
  strings, THEN the system SHALL exit `78`, and the `hint` SHALL show the array
  form.

  ```
  error: database.shop.password_command is not an array
  cause: .tpl/.cfg line 7 declares database.shop.password_command as a string; this key takes an array of arguments
  hint:  write it as an array: password_command = ["security", "find-generic-password", "-s", "tpl-shop", "-w"]
  exit:  78 (EX_CONFIG)
  ```

  *Rationale.* `FR-CONF-023` fixes the stored form as an array, and
  `FR-CONF-025` splits a string **supplied to a command** by POSIX quoting
  rules. Applying that splitting rule to a string found in the file would put a
  quoting engine on the path that reads an untrusted `.cfg`, and would make the
  argument vector of a child process depend on a rule the reader of the file
  cannot see. `FR-CONF-024` executes the array directly and without a shell;
  the file states the array.

  *Accepted cost.* A hand-written `.cfg` that copies the string form from a
  `tpl cfg set` invocation is refused rather than accepted. The `hint` carries
  the exact array to write, so the correction is one edit.

- **BR-CONF-004**: The file is strict in both directions and for one reason.
  `.tpl/.cfg` is untrusted input, per the trust boundaries of
  [security.md](security.md), and it decides which host is contacted, which
  credential is used, and which child process is executed. A reader that
  accepted what it did not understand, or that repaired what was written
  incorrectly, would be guessing at those three. Refusal with a nearest-match
  suggestion and a corrected form costs one invocation; a wrong guess costs a
  connection to somewhere else.

## Database entries

- **FR-CONF-006**: A `[database.<name>]` block SHALL be defined either by `dsn`
  or by the discrete connection fields, never by both. The discrete connection
  fields are `host`, `port`, `user`, `password`, and `database`.
  `password_command` SHALL NOT be one of them.

  *Amended in the third edition.* The list of discrete connection fields, and
  the exclusion of `password_command` from it, are new. The first edition said
  "the discrete fields" without enumerating them, which left it unstated
  whether `dsn` beside `password_command` was legal — the natural way to keep a
  secret out of a connection URL.

- **FR-CONF-007**: The system SHALL admit or refuse the combinations of
  connection and password keys in one entry as follows, and SHALL exit `78`
  (`EX_CONFIG`) for each refusal:

  | Combination in one entry | Outcome |
  |---|---|
  | `dsn` and any discrete connection field | `78` |
  | `dsn` carrying no password, and `password_command` | Admitted; the command supplies the password |
  | `dsn` carrying a password, and `password_command` | `78` |
  | Discrete fields and `password_command` | Admitted; the command supplies the password |
  | `password` and `password_command` | `78` |

  *Rationale.* `password_command` composes with either way of describing a
  connection, because its whole purpose is to keep the password out of the file
  and both forms otherwise write it there. Two password sources in one entry
  is a different matter: it is two answers to one question, and a precedence
  rule between them would be invisible on the command line, which is what
  `BR-CLI-002` exists to prevent.

  *Rejected.* Treating `password_command` as a discrete field, which would make
  the safest configuration there is — a DSN with no secret in it and a keychain
  lookup beside it — a configuration error. Also rejected: resolving
  `password` against `password_command` by a stated precedence, which leaves
  the losing key in the file looking as though it were in force.

- **FR-CONF-008**: The entry name SHALL be a label local to the project. It need
  not match the name of any database on the server.

## What an entry must describe

An entry must supply two facts that have no default: the host to connect to,
and the database to read. Each is carried either by its own key of
`FR-CONF-002` or by the `dsn` that stands in for the discrete fields, and an
entry supplying neither form of one of them describes no connection, or no
read. `FR-CFG-016` admits an entry created from any one discrete flag, so both
absences are reachable from a legal invocation. Both are decided at the
entry-resolution step of `FR-ERR-006`, before any connection is opened, and
neither adds a code: the `78` row of `FR-ERR-001` carries the condition as
*invalid entry*.

- **FR-CONF-040**: An entry SHALL name a host. IF the entry an invocation
  selects carries neither `dsn` nor `host`, THEN the system SHALL exit `78`
  (`EX_CONFIG`) and SHALL NOT open a connection.

  The `cause` SHALL name `.tpl/.cfg`, the entry, and `database.<name>.host` as
  the key the entry does not carry, per the `78` row of `FR-ERR-034`. The
  `hint` SHALL carry `tpl cfg database update <entry> --host <host>`, per
  `FR-ERR-009` and `FR-CFG-027`, built under `FR-ERR-022` and dropped under
  `FR-ERR-023` where the entry name falls outside the character set that
  requirement applies to it.

  *Why `dsn` satisfies it.* The grammar of `FR-CONF-009` makes `host`
  mandatory in a DSN, so an entry defined by `dsn` names a host by carrying
  one. `FR-CONF-006` makes the two ways of describing a connection mutually
  exclusive, so exactly one of them answers this requirement for any entry
  `FR-CONF-007` admits.

  *Added in the twenty-fifth edition.* `FR-CONF-002` gives `host` no default
  and `FR-CFG-016` requires one discrete flag and not this one, so
  `tpl cfg database add reporting --user reader` writes an entry that is legal
  in the file, passes every check `FR-CONF-007` makes, and describes no
  connection. Nothing in this corpus said what an invocation selecting it
  produces.

  *Rejected: composing the refusal where the connection is assembled.* That is
  where the absence is met — the layer that resolves an entry into the settings
  a connection is opened from — and it is the one layer that cannot satisfy
  the `78` row of `FR-ERR-034`, which obliges the `cause` to name the key and
  the file. Neither the file nor the position that declared the entry is a
  thing that layer holds. The condition is therefore decided where `.tpl/.cfg`
  is open, which is entry resolution, and the layer below it is reached only by
  an entry that has already answered this requirement.

  *Rejected: refusing at step 3, whenever the file carries such an entry.* It
  refuses a file for an entry no invocation selected, and no `cfg` subcommand
  is among the commands `FR-PROJ-025` excuses from reading and validating the
  file — so the `tpl cfg database update` that would repair the entry is itself
  refused, and `.tpl/.cfg` becomes repairable only by hand. That is the defect
  the twenty-second edition closed in three places, arriving from the reading
  side.

  *Rejected: `64`.* The invocation is legal and the file is what is wrong, and
  `78` is the code that sends a caller to `.tpl/.cfg`, per its row of
  `FR-ERR-001`. `FR-CFG-048` decides the mirror case the other way for the same
  reason: there the file is valid and the invocation is not.

- **FR-CONF-041**: The database a read covers SHALL be the one the selected
  entry names — the `database` key of `FR-CONF-002`, or the `/database` segment
  of `dsn`, per `FR-CONF-009` — and the system SHALL NOT derive it from any
  other source. IF the entry an invocation selects names none and the
  invocation reads the catalogue, THEN the system SHALL exit `78` (`EX_CONFIG`)
  and SHALL NOT open a connection.

  The `cause` SHALL name `.tpl/.cfg`, the entry, and `database.<name>.database`
  as the key the entry does not carry, per the `78` row of `FR-ERR-034`. The
  `hint` SHALL carry `tpl cfg database update <entry> --schema <database>`, per
  `FR-ERR-009` and `FR-CFG-027`, under the same construction rules as
  `FR-CONF-040`.

  *What it reaches.* Every `schema` subcommand, which cannot select a database
  without it; every read `tpl render` makes against a live or cached source;
  `tpl cache load`; and the privilege probe of `FR-CFG-044`, which is a
  `SELECT` against `INFORMATION_SCHEMA` restricted to the database the entry
  names and has nothing to restrict itself to without one.
  `tpl cfg database test` therefore exits `78` on such an entry, which is one
  of the four outcomes `FR-CFG-043` already records for that command, and it
  reports none of the four steps of `FR-CFG-024` — exactly as it reports none
  of them for an entry `FR-GLOB-007` refuses with `66`, because entry
  resolution precedes them both.

  *Added in the twenty-fifth edition.* `FR-CONF-002` gives the key no default
  and no requirement said what its absence produces, so which database a read
  covers was answerable only by an implementation choosing one. The choice is
  made here instead, and it is the only source this corpus has: the entry.

  *Rejected: taking the session's own default schema.* Where the entry names a
  database there is nothing to take that the entry did not supply, and where it
  names none the session has no default to take, because the entry is what
  selects one. A default reaching the session another way — a server-side
  initialisation, a proxy — would decide which database is read without
  appearing anywhere on the command line or in the file, which is what
  `BR-CLI-002` exists to prevent, and it would cost a round trip to obtain.

  *Rejected: reading the databases the reader can see and choosing among
  them.* It is the guess `BR-CONF-004` refuses, made over the one value that
  decides what the whole document contains, and a wrong guess presents a
  caller with the structure of a database they did not name. It also costs a
  second catalogue statement on every read, against the counts `NFR-PERF-001`
  and `NFR-PERF-002` fix.

  *Rejected: a flag that names the database on the command line.* `-d` and
  `--database` already name the **entry**, per `FR-GLOB-004`, so a second flag
  spelt from the same word would be read as the first by everyone who met it;
  and `FR-GLOB-001` closes the global set at seven. Adding one is a change to
  the command surface and is not what this gap needs: the file already has a
  key for the value, and what was missing was the rule that the key is the
  answer.

## DSN

- **FR-CONF-009**: A DSN SHALL have the form
  `scheme://[user[:password]@]host[:port]/database`.

  *Amended in the fifth edition.* The grammar previously ended `[?params]`.
  `FR-CONF-011` admits no parameter, so the optional group is removed rather
  than left in a grammar that nothing may fill.

- **FR-CONF-010**: The system SHALL accept the schemes `mysql://` and
  `mariadb://`, treating them as equivalent.

- **FR-CONF-011**: A DSN SHALL carry no query parameters. IF a DSN carries a
  `?`, THEN the system SHALL exit `78` (`EX_CONFIG`), whatever follows it.

  *Amended in the fifth edition.* The first edition admitted "query parameters
  drawn from an enumerated list" and left the list to be written, which is
  `OQ-004`. The list is empty, so the rule is stated directly rather than as an
  admission with nothing in it.

  *Rationale.* Every setting a DSN parameter could carry already has a key in
  `FR-CONF-002`, and a key is visible to `tpl cfg get`, `tpl cfg list`, and
  `tpl cfg database show` while a parameter buried in a URL is visible to none
  of the three. An empty list also means the DSN grammar of `FR-CONF-009` has
  exactly one reading, and `FR-CONF-018` — which parses the URL before
  expanding inside it — has no parameter field to defend.

  *Accepted cost.* A DSN copied from another tool, which commonly carries
  parameters such as a connection charset or a socket path, is refused rather
  than partly honoured. The refusal names the `?`, and the `hint` names
  `.tpl/.cfg`, the entry's `dsn` key, and the edit that removes the `?` and
  everything after it.

  *Amended in the forty-third edition.* The `hint` pointed at
  `tpl cfg database add` with the discrete flags. That command cannot succeed
  for an entry the file already carries, per `FR-CFG-017`, and `BR-ERR-004`
  bars a `hint` naming a command that cannot
  succeed.

- **FR-CONF-012**: A TLS parameter in a DSN SHALL be a special case of
  `FR-CONF-011` and SHALL exit `78` for the same reason as any other parameter.

  *Rationale.* Encryption is configured by the `tls` key alone, and a DSN may
  not contradict it. This requirement is kept, rather than folded into
  `FR-CONF-011`, because `BR-CONF-001` and `FR-SEC-021` both cite it as the
  place the prohibition is stated, and because a TLS parameter is the one
  parameter whose acceptance would be a security regression rather than a
  configuration surprise.

## TLS

- **FR-CONF-013**: `tls` SHALL take one of five modes, defaulting to
  `verify-identity`:

  | Mode | Meaning |
  |---|---|
  | `disabled` | No TLS |
  | `preferred` | Encrypt if the server allows it |
  | `required` | Always encrypt, without validating |
  | `verify-ca` | Also validate the certificate chain |
  | `verify-identity` | Also validate the hostname |

  *Rationale.* Encrypting and verifying are different guarantees and must have
  different names. A single `required` that does not validate is
  indistinguishable from no TLS against an active intermediary, while the word
  tells the reader the opposite. Defaulting to `preferred` would let an active
  intermediary simply answer "no TLS", after which the handshake carries the
  user and password in clear.

  *Accepted cost.* A development server with a self-signed certificate now fails
  by default and needs `tls = "required"` or a `ca_file`.

- **FR-CONF-014**: `ca_file` and `ca_path` SHALL supply the trust material used
  by `verify-ca` and `verify-identity`. The system SHALL resolve each entry of
  the directory `ca_path` names **through symbolic links**: an entry that
  resolves to a regular file SHALL be read at its target and SHALL contribute
  to the trust material, and an entry that resolves to anything else SHALL be
  skipped, a directory being skipped rather than descended into. An entry the
  system cannot resolve, or cannot read at its target, SHALL be reported
  against that entry's own path in the directory and SHALL NOT be passed over,
  per the `74` row of `FR-ERR-034`. What becomes of a directory that yields no
  regular file at all is `FR-CONF-044`.

  *Amended in the thirty-second edition: the key says whether it follows a
  link, because the convention it exists to serve is a directory of links.*
  The requirement had one clause and settled neither question a reader of it
  has to answer. A `CApath` directory, in the sense OpenSSL and MariaDB give
  the word, is conventionally a set of hash-named **symbolic links** created by
  `c_rehash` or `openssl rehash` beside the certificates they point at. A
  `ca_path` that skipped links would take nothing from such a directory and
  would say nothing about having taken nothing — and that is the arrangement
  the key is most likely to be pointed at.

  *Weighed against what this corpus already says about links, and the two are
  not one rule.* This corpus states links twice and in opposite directions,
  each on the ground of what a link would let somebody do. `FR-TMPL-024` and
  `FR-SEC-017` **refuse** a symbolic link inside `.tpl/templates/`: that
  directory is versioned and shared, its contents are printed by
  `tpl template show`, and `FR-TMPL-026` names the attack in its own
  rationale — `ln -s ../.cfg .tpl/templates/leak.jinja` must not become a
  credential dump. `FR-PROJ-009` and `FR-SEC-015` **follow** a link: the `.tpl`
  path is canonicalised so that the ownership and mode checks of `FR-PROJ-010`
  and `FR-PROJ-011` land on the real file rather than on a pointer to it, and
  following is what makes the check sound. `ca_path` has the second shape and
  not the first, on three grounds that hold together. The directory is named by
  `.tpl/.cfg`, which those same two requirements oblige to be the invoking
  user's alone at mode `0600`, so a link inside it is one the caller put there
  or pointed at. What is read is certificates, which are public by
  construction and are the one kind of material in this file that is not a
  secret. And no byte of the bundle is ever printed: it is handed to the TLS
  layer, and `FR-ERR-013` and `FR-SEC-005` keep it out of every stream, so
  there is no disclosure for a link to arrange. Following therefore widens
  nothing this corpus protects, and skipping costs the key its own convention.

  *Rejected: selecting each entry on the kind of the entry itself, so that a
  symbolic link never contributes.* A hash-named `CApath` then yields an empty
  bundle. It is rejected because its outcome is the worst available: not a
  refusal and not a warning, but a
  `verify-ca` or `verify-identity` connection attempted on the public roots
  alone, which then fails — where it fails at all — with a transport error
  naming a certificate and never naming the key that was meant to admit it.
  `FR-CONF-039` is why it may not fail at all: supplied material is additional
  to the bundled roots, so a publicly signed server validates whether or not
  the pinned authority was ever loaded. The two questions are answered together
  because either answer alone leaves the arrangement half-diagnosed.

  *Observed, 2026-09-21, and recorded as a reading of that date rather than as
  a standing claim.* `trust`, in `src/mariadb/connect.rs`, selected directory
  entries on `DirEntry::file_type`, which does not traverse a link. The option
  rejected above was therefore the behaviour built when this amendment was
  drafted, and that is why the amendment was drafted; it is not what makes the
  option wrong, and the paragraph above gives the ground that does.

  *Rejected: passing over an entry that cannot be resolved.* A dangling link in
  a `CApath` is what a removed certificate leaves behind, and any reader that
  selects an entry on the kind of the entry itself passes over it in silence,
  because that reading never asks what the link points at. Under this
  requirement the resolution is attempted and its failure is reported. The
  ground is the one `BR-CONF-004` states for
  the whole file: a reader that accepts what it does not understand is guessing
  at which authority the connection trusts. A skipped dangling link is a trust
  anchor the operator believes is loaded and is not, which is the failure this
  amendment exists to make visible, arriving one entry at a time instead of all
  at once.

  *Amended in the thirty-fourth edition: the two clauses above name the option
  and not the code.* The first read *Rejected: skipping a symbolic link, which
  is the behaviour built*, and asserted in the present tense, undated, what
  `src/mariadb/connect.rs` did; the second said a dangling link *is skipped in
  silence today*. Both were false in the commit that wrote them — `455e48d`
  carries this amendment and the correction of that file together — and the
  first had already misled a reader: on 2026-09-22 a reading made for the
  project's architecture decision records took it as a standing claim and
  concluded that the code still skipped links, an error caught only by reading
  `src/mariadb/connect.rs`. A rationale that describes an implementation stops
  being true the moment the implementation is corrected, and it stops without
  any signal. The remedy this corpus already had is
  [project-and-discovery.md](project-and-discovery.md)'s, where the two
  readings of `src/project/init.rs` each carry an `*Observed, 2026-09-18.*`
  note: a rejected option is named by what it does, its ground is the outcome
  it produces, and a reading of the implementation is kept beside it as a dated
  observation, which a later reader can weigh against the date instead of
  taking on trust.

  *The determinism is untouched, and the order that delivers it is cited rather
  than restated.* One configuration produces one bundle on every run and on
  every host, exactly as before: what reaches the driver is a function of
  `ca_file`, of the names the `ca_path` directory holds, and of what those
  names carry — and of nothing the filesystem decides. The order in which the
  entries are assembled is fixed by **the project's architecture decision
  record for the TLS mode mapping**, which that register's index lists as
  serving this requirement. Two entries resolving to one certificate contribute
  it twice; no requirement of this corpus forbids that, and none is amended
  here to forbid it.

  *Amended in the thirty-fourth edition: the order is cited and no longer
  restated.* The note said *Entries are sorted into ascending path order before
  any of them is read, over the names the directory holds and never over the
  targets they resolve to* — which is, clause for clause, the decision that
  record states, one fact held in two places. The copy here carried neither the
  alternative that record rejected, taking the entries in the order the
  directory yields them, nor the published documentation that grounds the
  rejection, so a reader who found only this note got the rule without the
  reason it is not the other rule, and a maintainer changing the order had two
  places to change and one trigger to re-read. The citation is by role and not
  by number, which is the form `FR-ENV-003` already uses for the template
  engine's pin and the form that survives the register being renumbered.

  *Rejected: keeping the sentence here and letting the record cite this
  requirement for it.* It is the live alternative, because it costs a reader no
  hop and because this corpus is meant to be self-sufficient. It is rejected on
  two grounds. The order is a **mechanism** and a requirement states the
  outcome a caller observes — the outcome is the sentence above, and it stays
  here — which is the ground the ninth edition used when `FR-ERR-030` named a
  mechanism the shipped binary cannot have. And the two copies had already
  begun to differ in content rather than merely repeat each other, which is the
  point at which one of them is the owner and the other is a summary nobody
  re-reads.

  *Accepted cost.* A reader of this requirement alone no longer learns which
  order is used, and the citation points into a folder this corpus does not own
  and has no trigger to re-read — the fifth validation rule's exposure, taken
  deliberately. What bounds it is that the property this requirement guarantees
  never left it, and that the record is subordinate: where it and this file
  disagree, this file governs.

  *Provenance of the convention, stated because it is the one input this
  amendment did not verify.* That a `CApath` directory is conventionally a set
  of hash-named symbolic links is taken as given from the task that raised the
  question. It is not cited to a published authority, because no document in
  this repository states it and the project's scope rule admits no reading
  outside the repository; `ADR-002` records only that no driver method takes a
  directory, which is why `tpl` reads it. **The decision does not rest on the
  convention alone**, and stands if it is ever shown to be wrong: the weighing
  above turns on there being no disclosure for a link to arrange, which makes
  following the more useful behaviour for any directory a caller curates, and
  `FR-CONF-044` makes the empty outcome visible either way. What the convention
  adds is the reason the defect is likely rather than possible. Verifying it
  against OpenSSL's documentation for `c_rehash` and `openssl rehash`, and
  against MariaDB's for `ssl-capath`, would make it a provenance of the third
  kind and is one line of work for whoever may read outside this repository.

- **FR-CONF-044**: IF an entry declares `ca_path` under `verify-ca` or
  `verify-identity` and no entry of the directory it names resolves to a
  regular file, THEN the system SHALL exit `78`, and the `cause` SHALL name the
  directory as the entry declared it and SHALL state that it yielded no
  certificate file.

  *Rationale.* This is the answer to the second of `FR-CONF-014`'s two
  questions, and it is what makes the first one's failure visible. A declared
  key that contributes nothing is a fault in `.tpl/.cfg` and nowhere else: the
  path is wrong, or the directory is empty, or its contents are of a kind this
  key does not take. `78` puts it with `FR-CONF-034`, `FR-CONF-035` and the
  three `password_command` refusals, and the remedy `FR-ERR-001` gives that
  code — fix `.tpl/.cfg` — is the remedy here. The condition is decided while
  the trust material is assembled, before any connection is opened, so it costs
  no round trip and reaches the caller before the server is contacted.

  *The condition is exactly stated, and no wider.* It is *no entry resolves to
  a regular file*, which is decidable before a byte is read. A regular file
  that is empty, or that holds no PEM block, is **not** this condition: `tpl`
  does not parse the bytes it assembles, `FR-CONF-014` takes them as they are,
  and a requirement that refused on their content would oblige this corpus to
  fix a certificate format it names nowhere. The condition is also per key: a
  `ca_path` that yields nothing is refused whether or not `ca_file` is declared
  beside it, because the operator asked for both and only one was honoured.

  *Rejected: a warning on stderr, and it was the close alternative.* It refuses
  nothing, so a connection that would have succeeded still succeeds, which is
  its whole appeal. It was rejected because
  stderr is not contract — the [README](README.md#writing-conventions) and
  `NFR-DET-001` keep the guarantee on stdout — a warning is lowered out of sight
  by `-q` under `FR-GLOB-015`, and a caller redirecting stderr in a build log
  is exactly the caller this condition exists for. The corpus's own posture on
  a `.tpl/.cfg` fault is refusal and not repair, per `BR-CONF-004`, and a key
  that does nothing is not a lesser fault than a key that is misspelled, which
  `FR-CONF-034` refuses outright.

  *Rejected: `69` (`EX_UNAVAILABLE`), and `74` (`EX_IOERR`).* `69` is what
  `FR-CONF-038` gives a verifying mode against a server that offers no TLS,
  which is a fact about the server discovered on the wire; this is decided from
  the project before anything is contacted, and reporting it as `69` would send
  a caller to look at a server that is not at fault. `74` is an I/O failure, and
  nothing failed: the directory was read successfully and is empty of what the
  key promises. An entry that could not be read **is** `74`, and
  `FR-CONF-014` states that separately.

  *Neither mode that ignores the key is reached.* `disabled`, `preferred` and
  `required` never read `ca_file` or `ca_path` — `FR-CONF-014` names the two
  modes the material serves, and `FR-CONF-038` records `required` ignoring
  supplied trust material as one of the three controls that separate the
  modes — so a `ca_path` declared beside one of them is stored, printed and
  validated as a path and is never opened. Refusing there would turn an
  unread key into a failure of a mode that would never have looked at it.

- **FR-CONF-036**: The five modes of `FR-CONF-013` SHALL be normative over the
  database driver. A driver that cannot express all five distinctly SHALL be
  disqualified from the choice, and the choice SHALL NOT be settled by reducing
  the mode set to what a candidate driver offers.

  *Rationale.* The five modes exist because encrypting and verifying are
  different guarantees, per `FR-CONF-013`, and a driver that collapses
  `required` into `verify-ca` — or `verify-ca` into `verify-identity` — makes
  `tpl` report a guarantee it is not giving. That is the failure `FR-CONF-013`
  and `FR-SEC-021` are written to prevent, arriving through the dependency
  rather than through the configuration.

  *Provenance.* This is an acceptance criterion of the roadmap task that
  chooses the driver by measurement, and is recorded here so that the criterion
  is a requirement rather than a preference held by whoever runs the
  measurement. The specification names no driver: which one is chosen is an
  architecture decision, resolved by measuring startup, peak resident memory,
  and stripped binary size, and the outcome does not change any requirement of
  this file.

  *The gap is closed.* What each of the five modes maps to in the chosen driver
  was `OQ-003`, and the driver was chosen by
  measurement on 2026-09-10. The mapping was verified mode by mode against
  running servers and satisfies this requirement: all five modes are
  expressible and all five behave distinctly. `FR-CONF-038` states the
  behaviour that was observed, `FR-CONF-037` states what `tpl` must do with it,
  and `FR-CONF-039` states the one guarantee the mapping does **not** deliver.

- **FR-CONF-037**: The system SHALL set the TLS mode of `FR-CONF-013`
  explicitly on every connection it opens, and SHALL NOT rely on the database
  driver's default for any mode, including `disabled`.

  *Rationale, and it is empirical rather than defensive.* Both drivers
  measured on 2026-09-10 default to the wrong posture for `tpl`, in opposite
  directions. One never negotiates TLS at all, so a connection to a server
  that offers it is plaintext unless the caller intervenes. The other defaults
  to a preferring mode that upgrades where the server advertises TLS and
  **falls back to plaintext in silence** where it does not — which is
  `disabled` wearing the appearance of `required`. An inherited default is
  therefore not a neutral starting point but a fifth, unnamed mode, and
  `BR-CONF-001` already makes `tls` the sole authority on encryption for an
  entry. A default that changes with a dependency upgrade would move that
  authority out of the configuration without anything in the configuration
  changing.

- **FR-CONF-038**: The five modes SHALL be distinguishable by observed
  behaviour, and their behaviour SHALL be the following against a server that
  offers TLS and a server that does not:

  | Mode | Server offering TLS | Server offering no TLS |
  |---|---|---|
  | `disabled` | plaintext | plaintext |
  | `preferred` | encrypted | plaintext |
  | `required` | encrypted, chain not validated | `69` (`EX_UNAVAILABLE`) |
  | `verify-ca` | encrypted, chain validated against the trust material of `FR-CONF-014` | `69` |
  | `verify-identity` | encrypted, chain and hostname validated | `69` |

  The fixture of `scripts/mariadb/` SHALL be able to present, at each series
  of `FR-SRV-015`, a server whose certificate names the host by which the
  project's tests reach it, and SHALL retain a server that offers no TLS. The
  first is what an acceptance test for the default mode of `FR-CONF-013`
  requires; the second is the right-hand column of the table above.

  *Amended in the eighth edition: the fixture obligation is a requirement
  rather than a consequence.* The *Observed* note below records that `tpl`
  with default configuration could not reach the fixture over TCP on any
  series **as that fixture then stood**, because `10.11` offers no TLS until
  something configures it and the certificate the other three generate
  automatically carries no `subjectAltName`. Left as a consequence, that made
  the success cell of `verify-identity` — the default of `FR-CONF-013`, and
  therefore the mode every caller meets first — the one cell of the table with
  no test, and it obliged every other test that reaches a server to set `tls`
  away from its default. The obligation is the fixture's, so it is stated
  here, in the requirement whose cells it makes demonstrable.

  *Rejected.* Dropping the acceptance test for the default mode and stating
  the cost. The cost is not one this corpus can state and keep its own rule
  that a requirement which cannot be tested or demonstrated is not a
  requirement: `FR-CONF-036` makes the five modes normative over the driver
  precisely so that each is distinguishable, and a default that nothing
  exercises end to end is the one a caller reaches without asking for it.
  Also rejected: configuring the certificate on the three TLS-capable series
  alone, which leaves `10.11` — supported until 2028-02-16 — unable to run an
  acceptance test that `FR-SRV-029` requires against every series.

  *What this obligation is not.* It is not a change to any of the ten cells,
  which stand exactly as observed, and it is not a relaxation of
  `verify-identity`: a certificate that names the host is what the mode has
  always required, and supplying one makes the requirement demonstrable rather
  than weaker. How the certificate is generated, where the fixture keeps it,
  and how the no-TLS server is retained alongside it are the fixture's own
  work and are not specified here.

  *Observed.* Each cell was verified against two running servers of
  `FR-SRV-015` — one reporting `have_ssl=YES` and one reporting
  `have_ssl=DISABLED` — and encryption was read from the live session rather
  than from the configuration that requested it. Three controls separate the
  modes from one another: `verify-ca` without trust material fails on an
  unknown issuer, so it genuinely validates the chain; `preferred` without
  trust material still connects encrypted, so it is not a disguised
  `verify-ca`; and `required` with trust material ignores it, so it is not one
  either.

  *One cell was observed only in its failing form, and the reason matters for
  testing rather than for the requirement.* `verify-identity` against a
  TLS-offering server was observed to fail on the hostname, because the
  certificate MariaDB generates automatically from `11.4` onward carries **no
  `subjectAltName`** — it is self-signed, has no extensions at all, and is
  regenerated on every server start. No hostname can match it. That is what
  separates `verify-identity` from `verify-ca` in the observation and is why
  the two are distinct rather than collapsed; what it does not do is
  demonstrate the success cell, which needs a server certificate carrying a
  name. **Consequence for the project's own tests, now an obligation in the
  text above and discharged in the fixture:** `tpl` with default configuration
  could not connect to the fixture of `scripts/mariadb/` over TCP on any
  series **as that fixture then stood**, and an acceptance test for the
  default mode needs TLS configured in it with a certificate that names the
  host. The fixture carries material of its own and configures it at all four
  series, which is what the obligation above requires.

  *Amended in the fourteenth edition: the consequence says when it was true.*
  It was written in the present tense, and the fixture has since been given
  the certificate the obligation demands, so the sentence asserted of today's
  fixture the opposite of what that fixture does. The observation it draws on
  is unchanged and keeps its date; only the tense and the discharge are added.
  The same defect in the observation record of `FR-SRV-038` — difference 3,
  which cites the note below — is corrected in the same edition.

  *A supported series may offer no TLS at all*, so the right-hand column is
  not hypothetical: `10.11` reports `have_ssl=DISABLED` unless an
  administrator configures a certificate, and it is supported until
  2028-02-16. That condition is the one difference 3 of `FR-SRV-038` records
  and cites rather than restates. The `69` in three of its cells is correct
  behaviour and not a defect — `FR-CONF-013` defaults to `verify-identity`,
  and a server that cannot encrypt cannot satisfy it. `69` is the code because
  the failure is in the TLS handshake phase, per `FR-ERR-001`, and
  `FR-ERR-034` requires the `cause` to name that phase and what it returned.

  *The mapping onto the chosen driver* SHALL be recorded in the project's
  architecture decision records and cited from there, and SHALL NOT be
  restated in this corpus, for the reason `BR-SRV-005` gives about the
  supported-series table and `FR-ENV-003` about the engine pin. This
  specification names no driver, and the behaviour above is what a caller can
  observe whichever one is chosen.

  *Closes* `OQ-003`, now listed under [Closed](open-questions.md#closed).

- **FR-CONF-039**: The trust material supplied by `ca_file` or `ca_path` SHALL
  be **additional** to the trust anchors the TLS implementation already
  trusts, and `verify-ca` and `verify-identity` SHALL NOT be described,
  documented, or reported as exclusive trust in the supplied authority.

  *Observed.* Both drivers measured add a supplied certificate to the public
  root bundle rather than substituting it for the bundle. Neither can express
  "trust only this authority", and the limitation is symmetric — it is a
  property of how the two crates build their root store, not a difference
  between them.

  *Consequence, stated plainly because it is a weaker guarantee than the words
  suggest.* Under `verify-ca`, a server presenting a certificate issued by any
  publicly trusted authority validates, even though the operator pinned a
  private one. Pinning therefore **widens** the set of certificates that pass;
  it does not narrow it. `verify-identity` adds the hostname check on top,
  which is what an operator pinning a private CA is usually reaching for, and
  it is the default of `FR-CONF-013`.

  *What would change this.* Exclusive trust is a stronger requirement than
  `FR-CONF-036` states, and no candidate driver satisfies it as shipped, so
  requiring it would leave the project with no driver at all. If it is ever
  required, it is an amendment to this requirement and to `FR-CONF-036`
  together, and it re-opens the driver choice.

  *A stated limit.* This requirement names where a guarantee stops, in the
  form the [README](README.md#writing-conventions) fixes for all three:
  `FR-PRIV-020`, where a table whose triggers are hidden cannot be told from
  a table that has none, and `FR-SRV-041`, where a server determined to pass
  as MariaDB will pass.

- **BR-CONF-001**: The `tls` key is the sole authority on encryption for an
  entry. No other key, flag, or DSN parameter may weaken or override it.

## `${VAR}` expansion

- **FR-CONF-015**: The system SHALL expand `${VAR}` only in the following
  fields: `dsn`, `host`, `port`, `user`, `password`, and `database`.

- **FR-CONF-016**: The system SHALL NOT expand `${VAR}` in `tls`.

  *Rationale.* An injected `TLS_MODE=disabled` must not be able to turn off
  encryption.

- **FR-CONF-017**: The system SHALL NOT expand `${VAR}` in `password_command`.

  *Rationale.* The environment must not be able to alter the command executed.

- **FR-CONF-018**: WHEN expanding inside a `dsn`, the system SHALL parse the URL
  first, SHALL expand within the already-delimited field, and SHALL then
  percent-encode the result.

  *Rationale.* In that order, an expanded value can never move the host, the
  port, or the database. Expanding textually before parsing would let
  `SHOP_PW=x@attacker.example.com/shop?#` redirect the connection and hand the
  attacker's server the user and password in the handshake.

- **FR-CONF-019**: Expansion SHALL be a single pass. The system SHALL NOT
  re-expand the result of an expansion.

- **FR-CONF-020**: `$$` SHALL denote a literal `$`.

- **FR-CONF-021**: IF an expansion is unclosed — `${VAR` with no closing brace —
  THEN the system SHALL exit `78`.

- **FR-CONF-022**: IF a referenced environment variable is undefined, THEN the
  system SHALL exit `78`.

- **BR-CONF-002**: An undefined variable is never a silent empty substitution.

- **BR-CONF-003**: `${VAR}` is the only mechanism by which `tpl` reads the
  environment, and it exists so that a secret need not be written to disk.

## `password_command`

- **FR-CONF-023**: `password_command` SHALL be stored in `.cfg` as an array of
  arguments:

  ```toml
  password_command = ["security", "find-generic-password", "-s", "tpl", "-w"]
  ```

- **FR-CONF-024**: The system SHALL execute `password_command` directly, without
  a shell.

  *Rationale.* With a shell, reading a `.cfg` would be able to execute anything,
  and a `.tpl` inherited from an ancestor directory or from a clone would run
  code chosen by whoever wrote it.

- **FR-CONF-025**: WHEN a `password_command` is supplied as a single string —
  for instance to `tpl cfg set` — the system SHALL split it into words by POSIX
  quoting rules, honouring single and double quotes, and SHALL store the
  resulting array. `FR-CONF-046` states the strings it SHALL refuse instead.

  *Amended in the forty-seventh edition.* The second sentence is new, for rmp
  `#276`.

- **FR-CONF-046**: IF a `password_command` supplied as a single string — the
  value given to `tpl cfg set database.<name>.password_command`, or to
  `--password-command` under `FR-CFG-046` — meets any condition of the table
  below, THEN the system SHALL exit `64` (`EX_USAGE`), as `FR-CFG-010` does for
  a value that does not conform to its type, and SHALL write nothing to
  `.tpl/.cfg`. The `cause` SHALL name the key or the flag and the condition
  met. The `hint` SHALL show the value written as one command line.

  | The string | Why it is refused |
  |---|---|
  | Yields no word: it is empty or holds only whitespace | There is no program to execute |
  | Leaves a single or a double quote unclosed | The POSIX quoting rules admit no unclosed quote, and a shell refuses one |
  | Ends in a backslash outside any quoted run, so the backslash escapes no character | The backslash would be dropped without trace; a shell reads it as a line continuation, not as a character |
  | Has `[` as its first character that is not whitespace | It is how an array written on the command line arrives: `["pass","db/shop"]` would be stored as the one word `[pass,db/shop]`, which names no program |

  Where a string meets more than one condition, the `cause` SHALL name the
  first in the order of the table.

  ```
  tpl cfg set database.shop.password_command '["pass","db/shop"]'
  error: invalid value for database.shop.password_command
  cause: the value begins with '[', which is how an array arrives; this key takes one command line written as one string, which tpl splits into words
  hint:  write the command as one command line: tpl cfg set database.shop.password_command '<command line>', as in 'pass db/shop'
  exit:  64 (EX_USAGE)
  ```

  The wording of each line is the implementation's, under `FR-ERR-008`
  through `FR-ERR-012`. The example fixes the condition named and the code.
  The placeholder stands for a value only the caller knows, which
  `BR-ERR-004` admits.

  *The table states the rule the implementation follows, and adds two rows.*
  The first two rows are what `tpl cfg set` and `--password-command` already
  refused, and no requirement stated them. The third and the fourth are new:
  a trailing backslash was dropped, and a string beginning with `[` was stored
  as one word, and both exited `0`.

  *Why `[` costs no working command.* `[` is a program on every supported
  system — `/bin/[`, the `test` utility — but POSIX specifies that `test`
  writes nothing to its standard output, so it can never supply the password
  `FR-CONF-027` reads from there. A program whose name begins with `[` remains
  expressible: the rule tests the first character as written, so a quoted or
  escaped first word — `'[x]/get' db`, `\[x]/get db` — and an absolute path —
  `/bin/[` — pass it.

  *Rejected: reading a string that begins with `[` as a JSON array.* It gives
  one value two grammars chosen by its first character, and `FR-CFG-046` bars
  an array on the command line. Also rejected: keeping the behaviour and
  documenting it. The stored value names no program, so the fault would appear
  only at the first connection, as the `78` of `FR-CONF-042`, far from the
  command that wrote it.

  *Added in the forty-seventh edition,* for rmp `#276`.

- **FR-CONF-026**: The system SHALL pass shell metacharacters through as literal
  arguments. Pipes, `;`, `&&`, `$(…)`, redirection, and globbing SHALL NOT be
  interpreted.

  These work:

  ```
  security find-generic-password -s tpl -w
  op read op://vault/db/password
  /usr/local/bin/get-secret --profile prod
  ```

- **FR-CONF-027**: The system SHALL use the trimmed standard output of
  `password_command` as the password.

- **FR-CONF-028**: The system SHALL start `password_command` as the leader of
  a process group of its own. The `password_command` phase SHALL end when the
  child has exited and its standard output has reached end of file. IF the
  phase has not ended when its deadline expires, THEN the system SHALL
  terminate every process of that group — the child and every descendant still
  in the group — SHALL NOT wait on the child's standard output past the
  deadline, and SHALL exit `78`. A descendant that has left the group is not
  terminated, and the system does not wait for it either.

  *Amended in the forty-second edition, as decided for rmp `#252`.* The
  requirement said only that exceeding the deadline is `78`, and did not say
  what the deadline covers or which processes it ends. The security audit
  recorded in `SECURITY-AUDIT.md` at the repository root found, as its finding
  SEC-01, a helper that writes its output, starts a descendant holding the
  standard output open, and exits `0`. The child was reaped within the
  deadline and the read of its standard output then waited for an end of file
  that the descendant withheld, so the invocation hung past every deadline with
  no diagnosis, against `FR-SEC-022`. The phase now ends only when the output
  is complete, the deadline bounds all of it, and terminating the group ends
  the descendants that would otherwise keep running.

  *Consequence, stated plainly.* A helper that exits `0` and leaves a
  descendant holding its standard output past the deadline is a failure with
  `78`, even when it has already written a password. The password is not used,
  because the output it came on never ended.

  *Where the termination stops.* A descendant that puts itself in another
  process group or session is outside the group and survives it. The
  invocation still ends at the deadline with `78`, because the system does not
  wait on the pipe past it; what it cannot do is end a process that has left
  the group it controls.

- **FR-CONF-031**: The system SHALL read at most 4096 bytes from the standard
  output of `password_command`. IF the child writes more, THEN the system SHALL
  terminate every process of the child's process group, established under
  `FR-CONF-028` — the child and every descendant still in the group — and
  SHALL exit `78`. A descendant that has left the group is not terminated.

  *Amended in the forty-second edition, as decided with rmp `#252`.* The
  requirement terminated the child alone. A descendant the child started could
  then keep running after the invocation ended, and keep the pipe open, which
  is the shape `FR-CONF-028` closes at the deadline. Both paths on which the
  system ends the helper now end the same group, and both stop where
  `FR-CONF-028` states.

  *Rationale.* An unbounded read from a child process is a denial-of-service
  surface reachable from an untrusted `.cfg`, and the bound has to be a
  refusal rather than a truncation: a password truncated at the cap would be
  sent to the server, refused, and reported as a `77` naming the credentials,
  which is a wrong diagnosis of a configuration fault. 4096 bytes is far above
  any credential and far below anything that costs memory, and the value is
  fixed here rather than made configurable because a key that raises it would
  be a key that reopens the surface.

  *Accepted cost.* A `password_command` whose helper prints a banner before the
  secret can exceed the cap and be refused. The `cause` line names the cap and
  the command, per `FR-ERR-034`, and never the bytes read, per `FR-SEC-005`.

- **FR-CONF-032**: The system SHALL direct the standard error of
  `password_command` to the null device. It SHALL NOT inherit the parent's
  standard error, SHALL NOT capture it, and SHALL NOT include any part of it in
  a diagnostic or in an error message.

  *Rationale.* `FR-GLOB-018` and `FR-SEC-005` already forbid writing the
  child's stderr anywhere. Inheriting would break that by construction, because
  the child writes straight to the parent's descriptor and `tpl` never sees the
  bytes it is obliged to withhold. Capturing would satisfy the prohibition and
  would then hold a buffer that a later diagnostic must be careful never to
  print — a rule enforced by discipline rather than by structure. The null
  device makes the prohibition unbreakable: there is nothing to leak.

  *Accepted cost.* A `password_command` that fails and explains itself on
  stderr explains itself to nobody. The diagnosis a caller receives is the
  outcome of the child — its exit status under `FR-CONF-033`, or the condition
  of `FR-CONF-042` or `FR-CONF-043` where it returned none — and the remedy
  stated in the `hint` is to run the command directly, where its own stderr is
  visible.

  *Amended in the thirty-second edition: the clause named one outcome of
  three.* It read *the exit status of the child, per `FR-CONF-033`*, which was
  the whole of what this corpus governed when it was written and is now one of
  three. Nothing about this requirement changes; the note names the two
  conditions the same edition added.

- **FR-CONF-042**: IF the system obtains no exit status from
  `password_command` — because the child could not be started, or because the
  system could not read the status of a child it had started — THEN the system
  SHALL exit `78`, and the `cause` SHALL name the command as stored, SHALL say
  which of the two occurred, and SHALL name what the operating system returned.

  *Why this is a condition of its own.* `FR-CONF-033` routes a child that
  **exits** non-zero, and its `cause` is obliged to name the exit status the
  child returned. A child that never ran, or whose outcome the system cannot
  read, returns none, so that requirement's condition is not raised and its
  `cause` obligation cannot be met. The two are the same failure to a
  caller — the configured way of obtaining a password produced no password —
  and they differ in what the message can say, which is why the code is shared
  and the wording is not.

  *Why `78`.* It joins `FR-CONF-022`, `FR-CONF-028`, `FR-CONF-031` and
  `FR-CONF-033` on the ground `FR-CONF-033` states for all of them: the
  configured way of obtaining a password failed to produce one, which is a
  fault in `.tpl/.cfg` and not in the network or the credentials. Routing it
  to `77` would tell the caller a server refused an authentication that was
  never attempted, and routing it to `70` would name a defect in `tpl` for a
  program the caller chose.

  *Two conditions on one code, and `FR-ERR-002` permits it.* That requirement
  bars collapsing two conditions onto one code **where the caller's next step
  would differ**, and here it does not. The remedy in both is the `hint`
  `FR-CONF-032` promises: run the command directly, where its own stderr is
  visible. For the first it reproduces the failure outright — the program is
  absent, or is not executable, or the file it names is not a program. For the
  second it rules the configured command out and leaves the machine as what to
  look at, which is the next thing a caller has to know. What `FR-ERR-002` does
  forbid is one wording for both, and this requirement obliges the `cause` to
  say which occurred.

  *What the `cause` may name, and what it may not.* The array as stored, under
  the one exception `FR-CONF-033` states over it — that exception is written
  over the array **wherever a requirement of this corpus obliges a `cause` to
  name it**, so it reaches this requirement without amending `FR-GLOB-018`,
  `FR-SEC-005` or `BR-ERR-003`. The operating system's own report of the
  failure, which is neither a credential nor a content of `.tpl/.cfg`. And
  nothing else: there is no standard output to name, because there is no
  password, and the child's standard error went to the null device under
  `FR-CONF-032`, so there is nothing held to withhold.

  *Written in the thirty-second edition, over a condition the diagnostic
  renderer at `src/diagnostics/cause.rs` already produced and no requirement
  reached.* `FR-ERR-002` obliges every failing condition to carry a code named
  by the requirement that owns it, and this one was owned by nobody. The
  implementation routes it to `78`, which this requirement confirms rather than
  decides. What it does **not** confirm is the wording: the message renders
  *could not be started* for both cases, which is false of the second, and
  correcting it is work this requirement obliges rather than a defect of this
  corpus.

- **FR-CONF-033**: IF `password_command` exits non-zero, THEN the system SHALL
  exit `78`, and the `cause` SHALL name the command as stored and the exit
  status the child returned.

  *Rationale.* `78` matches `FR-CONF-022`, the undefined `${VAR}`, and
  `FR-CONF-028`, the deadline: all three are the configured way of obtaining a
  password failing to produce one, which is a fault in `.tpl/.cfg` and not in
  the network or the credentials. Routing it to `77` would tell the caller the
  server refused an authentication that was never attempted.

  *Composition.* The command as stored is the argument array of
  `FR-CONF-023`, which `FR-CONF-017` guarantees carries no expanded value, so
  naming it in the `cause` cannot disclose a secret the **environment** holds.
  The child's stderr is not available to name, per `FR-CONF-032`, and its
  stdout is the password and is never named, per `FR-ERR-013`.

  *This requirement governs, and two general prohibitions yield to it.*
  `FR-GLOB-018` bars the `password_command` from every diagnostic stream at
  every verbosity, and `BR-ERR-003` bars the contents of `.tpl/.cfg` from every
  error message; the array this `cause` names is both. The three were in flat
  contradiction and this one wins, for three reasons stated together. It is the
  specific rule over the general, and it is the only one of the three written
  against this condition. The two general rules protect a **credential**, which
  is the whole subject of the section each sits in, and the array is the one
  part of an entry that exists in order not to be one — `FR-CONF-007`'s
  rationale says its whole purpose is to keep the password out of the file.
  And without it the `cause` names a category where an instance is available,
  which `FR-ERR-034` bans in terms, and the `hint` `FR-CONF-032` promises —
  run the command directly, where its own stderr is visible — cannot be
  written at all. `FR-GLOB-018`, `FR-SEC-005` and `BR-ERR-003` are amended in
  the same edition to state the exception rather than to be read past.

  The exception is **the array as stored, and nothing else**. Everything the
  three rules bar is still barred on this path: the child's stdout, which is
  the password; the child's stderr, which `FR-CONF-032` sends to the null
  device; every other key of `.tpl/.cfg`; the resolved DSN; and the argument
  vector. And it is not confined to this requirement's own condition:
  `FR-CONF-031` obliges the same `cause` to name the cap **and the command**,
  `FR-CONF-028` reaches the same array through the `78` row of `FR-ERR-034`,
  and `FR-CONF-042` and `FR-CONF-043` oblige it by name, so the exception is
  stated over the array wherever a requirement of this corpus obliges a
  `cause` to name it.

  *Accepted cost, stated plainly.* `${VAR}` cannot put a secret into this
  array, per `FR-CONF-017`, but a caller can: a `password_command` written as
  `["sh", "-c", "echo hunter2"]` is printed verbatim in the `cause`, on the
  stream a caller may be redirecting into a build log, a CI transcript or an
  issue report. Three things bound the cost and none of them removes it. The
  array is bytes the caller wrote into a file `FR-PROJ-010` and `FR-PROJ-011`
  require to be theirs alone at mode `0600`, so the message widens the audience
  for a secret that is already on that caller's disk rather than disclosing one
  they did not have. A literal secret there is a misuse of the key and not a
  supported configuration: the key that carries a secret in the file is
  `password`, `FR-CONF-007` refuses the two together, and an entry that wants
  neither has `${VAR}` in `password`. And the condition is a failure — the
  child exited non-zero — so the `cause` is reached only where the caller is
  already reading the message.

  *What would change this.* A rule that a `cause` names the array's first
  element and the count of its remaining arguments, rather than the array. It
  was rejected here because the executable alone identifies almost nothing a
  real entry uses — `security`, `op` and `sh` are the common first elements,
  and two entries whose helpers differ only in their arguments would produce
  the same `cause`, which is the wording `FR-ERR-034` bans — and because the
  `hint` would then name a command the caller cannot run. Taking it would be an
  amendment to this requirement, to `FR-CONF-031` and to `FR-ERR-034`
  together.

  *This note joins neither family of the
  [README](README.md#writing-conventions), and says so rather than letting a
  reader count.* The three-member family of `FR-SRV-041`, `FR-PRIV-020` and
  `FR-CONF-039` is limits on what `tpl` guarantees about a server, a table or a
  trust store, and each cites the other two; the four requirements beside it
  limit the **evidence** for a guarantee. This is a limit on a **redaction** —
  what a message is allowed to carry — which is a third kind, and it takes the
  note shapes both families use without joining either. The families stay at
  three and at four.

- **FR-CONF-043**: IF `password_command` is ended by a signal the system did
  not send, and therefore returns no exit status, THEN the system SHALL exit
  `78`, and the `cause` SHALL name the command as stored and the signal number
  the operating system reports.

  *Why it is not `FR-CONF-033`.* That requirement's condition is a non-zero
  **exit** and its `cause` is obliged to name the exit status the child
  returned. A signalled child returns no exit status at all, so the condition
  is a different one and the obligation is a different one. The code is the
  same, for the reason `FR-CONF-042` states: to a caller this is the
  configured way of obtaining a password failing to produce one.

  *The signal the system sends is not this condition.* `FR-CONF-028`
  terminates the child's process group at a deadline, and `FR-CONF-031`
  terminates it at the output cap. Both of those end the child by a
  signal and both own their outcome, so this requirement is reached only where
  something outside `tpl` ended the child — the terminal's process group, a supervisor, the
  out-of-memory killer, or a `kill` from elsewhere. Without this clause the
  requirement would swallow two conditions in force and report them with the
  wrong `cause`.

  *Why the signal is named.* `FR-ERR-034` bans a `cause` that names a category
  where an instance is available, and the instance is available: every target
  of `NFR-PERF-018` is a Unix, and a Unix reports the signal that ended a
  child. A `cause` saying only that the child was signalled would read
  identically for a deadline enforced from outside, a memory limit and an
  interactive interrupt, which are three different next steps for the caller.

  *What the `cause` may name, and what it may not.* The array as stored, under
  the exception `FR-CONF-033` states over it and which reaches every
  requirement that obliges a `cause` to name the array. The signal number,
  which is neither a credential nor a content of `.tpl/.cfg`. Not the child's
  standard error, which `FR-CONF-032` sent to the null device; not its
  standard output, which is the password and is barred by `FR-ERR-013`.

  *Written in the thirty-second edition, over a condition the diagnostic
  renderer at `src/diagnostics/cause.rs` already produced and no requirement
  reached.* The implementation raises it and routes it to `78`, which this
  requirement confirms, and it names no signal, which this requirement now
  obliges it to.

## Precedence

- **FR-CONF-029**: The system SHALL resolve every setting through exactly two
  configuration layers above the built-in default, strongest first:

  ```
  flag  >  .tpl/.cfg  >  built-in default
  ```

- **FR-CONF-030**: There SHALL be no environment layer in that precedence. No
  `TPL_DIR`, no `TPL_DATABASE`, and no per-flag environment variable, per
  `FR-CLI-021`. `${VAR}` expansion, per `FR-CONF-015`, is not a layer: it
  supplies the value of a key that is already in the file.

  *Rationale.* An invocation is fully described by what you can see of it. Seven
  invisible values would each need their own truthiness rule, and the behaviour
  of `tpl schema tables` would depend on them.

## Dependencies

- [cfg-commands.md](cfg-commands.md) — the commands that read and write these
  keys, and the flag-to-key mapping.
- [project-and-discovery.md](project-and-discovery.md) — the ownership and mode
  checks applied to `.cfg` before it is trusted.
- [security.md](security.md) — these rules restated as a cross-cutting threat
  model.
- [server-contract.md](server-contract.md) — `FR-SRV-015`, the four supported
  series, one of which may be configured to offer no TLS at all, which is the
  right-hand column of `FR-CONF-038`.
- [errors-and-exit-codes.md](errors-and-exit-codes.md) — `69`, the code a TLS
  handshake failure produces, and `FR-ERR-034`, which fixes what its `cause`
  must name and whose `74` row fixes what a trust-material path that cannot be
  read must name under `FR-CONF-014`; `FR-ERR-002`, which obliges every
  condition here to carry a code; `FR-ERR-006`, at whose entry-resolution step
  `FR-CONF-040` and `FR-CONF-041` are decided.
- [template-commands.md](template-commands.md) — `FR-TMPL-024`, the other rule
  this corpus states about a symbolic link, and the one `FR-CONF-014` is
  weighed against.
- [schema-commands.md](schema-commands.md) — the arm `FR-CONF-041` supplies
  with the database every one of its subcommands reads.

## Open questions

**None.** All seven this file carried are closed and listed under
[Closed](open-questions.md#closed): `OQ-003` by `FR-CONF-038`, `OQ-004` by
`FR-CONF-011`, `OQ-005` by `FR-CONF-031`, `OQ-006` by `FR-CONF-032`, `OQ-007`
by `FR-CONF-033`, `OQ-018` by `FR-CONF-034`, and `OQ-019` by `FR-CONF-035`.
