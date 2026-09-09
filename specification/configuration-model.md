---
title: Configuration Model
status: draft
last-reviewed: 2026-09-09
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

In scope: the `.cfg` key space, value types and defaults, entry shape, DSN
grammar, TLS modes, timeouts, `${VAR}` expansion, and `password_command`
execution.

Out of scope: the commands that read and write the file, which belong to
[cfg-commands.md](cfg-commands.md); and the ownership and permission checks
applied before the file is trusted, which belong to
[project-and-discovery.md](project-and-discovery.md).

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
  | `database.<name>.tls` | TLS mode, see `FR-CONF-011` | `verify-identity` |
  | `database.<name>.ca_file` | path to a certificate file | none |
  | `database.<name>.ca_path` | path to a certificate directory | none |

  *Provenance.* The default port `3306` comes from the root `README.md` and is
  contradicted by no decision.

- **FR-CONF-003**: There SHALL be no global configuration. The system SHALL NOT
  read configuration from the home directory, from an XDG location, or from
  `/etc`.

- **FR-CONF-004**: The system SHALL resolve every phase deadline strongest
  first: the `--timeout` flag, then the `[core]` key for that phase, then the
  built-in default of `FR-CONF-002`.

- **FR-CONF-005**: The system SHALL apply a deadline to every blocking phase:
  DNS resolution, TCP connect, TLS handshake, catalogue query,
  `password_command`, and render.

## Database entries

- **FR-CONF-006**: A `[database.<name>]` block SHALL be defined either by `dsn`
  or by the discrete fields, never by both.

- **FR-CONF-007**: IF one entry carries both `dsn` and any discrete connection
  field, THEN the system SHALL exit `78` (`EX_CONFIG`).

- **FR-CONF-008**: The entry name SHALL be a label local to the project. It need
  not match the name of any database on the server.

## DSN

- **FR-CONF-009**: A DSN SHALL have the form
  `scheme://[user[:password]@]host[:port]/database[?params]`.

- **FR-CONF-010**: The system SHALL accept the schemes `mysql://` and
  `mariadb://`, treating them as equivalent.

- **FR-CONF-011**: The system SHALL accept query parameters drawn from an
  enumerated list, and SHALL exit `78` for any parameter outside it.

- **FR-CONF-012**: The enumerated list SHALL contain no TLS-related parameter.
  IF a DSN carries a TLS parameter, THEN the system SHALL exit `78`.

  *Rationale.* Encryption is configured by the `tls` key alone, and a DSN may
  not contradict it.

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

## Precedence

- **FR-CONF-029**: The system SHALL resolve every setting through exactly two
  layers, strongest first:

  ```
  flag  >  .tpl/.cfg  >  built-in default
  ```

- **FR-CONF-030**: There SHALL be no environment layer in that precedence. No
  `TPL_DIR`, no `TPL_DATABASE`, and no per-flag environment variable.

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

- [OQ-003](open-questions.md#oq-003) — what each TLS mode maps to in the chosen
  driver.
- [OQ-004](open-questions.md#oq-004) — the enumerated DSN query-parameter list.
- [OQ-005](open-questions.md#oq-005) — the cap on `password_command` output.
- [OQ-006](open-questions.md#oq-006) — the handling of the child's stderr.
- [OQ-007](open-questions.md#oq-007) — the handling of a non-zero exit from
  `password_command`.
- [OQ-008](open-questions.md#oq-008) — how `--timeout` composes with the
  per-phase keys.
- [OQ-018](open-questions.md#oq-018) — the treatment of an unknown key found in
  a hand-written `.cfg`.
- [OQ-019](open-questions.md#oq-019) — whether a `password_command` written as a
  TOML string is accepted in a hand-edited file.
