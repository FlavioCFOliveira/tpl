---
title: Configuration Model
status: approved
last-reviewed: 2026-09-10
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
which the file is read, entry shape, DSN grammar, TLS modes, timeouts, `${VAR}`
expansion, and `password_command` execution and its failure modes.

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

- **FR-CONF-003**: There SHALL be no global configuration. The system SHALL NOT
  read configuration from the home directory, from an XDG location, or from
  `/etc`.

- **FR-CONF-004**: The system SHALL resolve each phase deadline from the
  `[core]` key for that phase, or from the built-in default declared for that
  key in `FR-CONF-002` where the key is absent. `--timeout` SHALL NOT
  participate in this resolution; it is an overall bound that composes with the
  result, per `FR-GLOB-011` and `FR-GLOB-012`.

  *Amended in the third edition.* The first edition put `--timeout` at the head
  of this precedence, which made the four `[core]` timeout keys unreachable
  because `FR-GLOB-001` gave the flag a default. The flag now has no default
  and is not a layer of this rule.

- **FR-CONF-005**: The system SHALL apply a deadline to every blocking phase:
  DNS resolution, TCP connect, TLS handshake, catalogue query,
  `password_command`, and render.

  *Note added in the fifth edition.* The DNS phase is the one phase whose
  behaviour depends on how the binary was linked. The Linux targets of
  `NFR-PERF-018` are statically linked against `musl`, whose static
  `getaddrinfo` does not load the platform's name-service modules, so a name
  resolvable only through such a module does not resolve. The deadline and the
  outcome are unchanged — the phase fails and `FR-ERR-027` routes it to `69` —
  but the `cause` line owes the caller the distinction, per `FR-ERR-034`: a
  name that did not resolve is not a host that refused a connection.

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
  than partly honoured. The refusal names the `?` and the `hint` points at
  `tpl cfg database add` with the discrete flags.

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
  by `verify-ca` and `verify-identity`.

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

  *Known gap.* What each of the five modes maps to in the chosen driver is
  [OQ-003](open-questions.md#oq-003) and stays open until that task reports.
  This requirement fixes what the mapping must satisfy; it does not state one
  that has not been verified.

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
  resulting array.

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

- **FR-CONF-028**: IF `password_command` exceeds its deadline, THEN the system
  SHALL exit `78`.

- **FR-CONF-031**: The system SHALL read at most 4096 bytes from the standard
  output of `password_command`. IF the child writes more, THEN the system SHALL
  terminate the child and SHALL exit `78`.

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
  exit status of the child, per `FR-CONF-033`, and the remedy stated in the
  `hint` is to run the command directly, where its own stderr is visible.

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
  naming it in the `cause` cannot disclose a secret. The child's stderr is not
  available to name, per `FR-CONF-032`, and its stdout is the password and is
  never named, per `FR-ERR-013`.

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

## Open questions

- [OQ-003](open-questions.md#oq-003) — what each of the five TLS modes maps to
  in the chosen driver. `FR-CONF-036` fixes what the mapping must satisfy; the
  mapping itself is blocked on the driver being chosen.

The other six this file carried are closed and listed under
[Closed](open-questions.md#closed): `OQ-004` by `FR-CONF-011`, `OQ-005` by
`FR-CONF-031`, `OQ-006` by `FR-CONF-032`, `OQ-007` by `FR-CONF-033`, `OQ-018`
by `FR-CONF-034`, and `OQ-019` by `FR-CONF-035`.
