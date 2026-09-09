---
title: Security Rules Across the Surface
status: draft
last-reviewed: 2026-09-09
related: [configuration-model.md, project-and-discovery.md, errors-and-exit-codes.md, template-commands.md]
---

# Security Rules Across the Surface

## Overview

Several rules of this specification exist for security reasons and are enforced
in more than one place. This file collects them, states the threat each closes,
and points at the module that owns the requirement. It adds no requirement that
is not stated elsewhere; where a rule appears here with its own identifier, it
is because the rule is genuinely cross-cutting and has no single owning module.

## Trust boundaries

`tpl` treats the following as untrusted input:

| Input | Why it is untrusted |
|---|---|
| The argument vector | Written by a caller, possibly generated |
| `.tpl/.cfg` | May be inherited from an ancestor directory or arrive in a clone |
| The environment | Read only through `${VAR}`, and expandable into a connection |
| Catalogue values | Table, column, and routine names and comments are free text on the server |
| A `--context` document | An ordinary JSON file, from anywhere |
| Files under `.tpl/templates/` | May arrive in a clone and be read before being reviewed |

## Credentials

- **FR-SEC-001**: No flag named `password`, and no flag whose purpose is to
  carry a password, SHALL exist at any node. See `FR-CFG-030`.

- **FR-SEC-002**: Two paths remain open by which a caller may place a literal
  password on the command line — `--dsn` and `tpl cfg set` — and the help of
  each states that the value is visible in the process table and recommends
  `${VAR}`. `tpl` warns; it does not prevent. See `FR-CFG-031` through
  `FR-CFG-033`.

- **FR-SEC-003**: `tpl cfg list` and `tpl cfg database show` SHALL redact
  passwords and SHALL print `${VAR}` unexpanded, so that a password living in an
  environment variable never reaches stdout through them. See `FR-CFG-021`.

- **FR-SEC-004**: `tpl cfg get` is the single, deliberate exception to
  redaction, because it is a directed read of a named key. See `BR-CFG-002`.

- **FR-SEC-005**: The system SHALL NOT write the argument vector, the resolved
  DSN, the `password_command` or its stderr, the raw driver error, or the
  contents of `.tpl/.cfg` to any diagnostic stream, at any verbosity level. See
  `FR-GLOB-018`.

- **FR-SEC-006**: The system SHALL NOT include a credential in any error
  message, at any verbosity level. See `FR-ERR-013`.

## Environment expansion

- **FR-SEC-007**: `${VAR}` SHALL expand only in `dsn`, `host`, `port`, `user`,
  `password`, and `database`. See `FR-CONF-015`.

- **FR-SEC-008**: `${VAR}` SHALL NOT expand in `tls`, so that an injected value
  cannot turn off encryption; nor in `password_command`, so that the environment
  cannot alter the command executed. See `FR-CONF-016` and `FR-CONF-017`.

- **FR-SEC-009**: Inside a DSN, the system SHALL parse the URL, expand within
  the already-delimited field, and then percent-encode, so that an expanded
  value can never move the host, the port, or the database. See `FR-CONF-018`.

  *Threat closed.* `SHOP_PW=x@attacker.example.com/shop?#` would otherwise
  redirect the connection and hand the attacker's server the user and password
  during the handshake.

- **FR-SEC-010**: An undefined variable SHALL be `78`, never a silent empty
  substitution. See `FR-CONF-022`.

## Command execution

- **FR-SEC-011**: `password_command` SHALL be executed directly, without a
  shell, from an argument array. Shell metacharacters are literal arguments. See
  `FR-CONF-023` through `FR-CONF-026`.

  *Threat closed.* With a shell, reading a `.cfg` inherited from an ancestor
  directory or arriving in a clone would execute code chosen by whoever wrote
  it.

- **FR-SEC-012**: `password_command` SHALL be subject to a deadline, exceeding
  which is `78`. See `FR-CONF-028`.

## Project discovery

- **FR-SEC-013**: The upward walk SHALL stop at the home directory and at the
  mount point. See `FR-PROJ-005`.

  *Threat closed.* A `.tpl` planted in a world-writable ancestor such as `/tmp`
  can no longer supply the configuration.

- **FR-SEC-014**: `.tpl/.cfg` SHALL be owned by the current user and SHALL carry
  no group or other access bits. See `FR-PROJ-010` and `FR-PROJ-011`.

  *Threat closed.* A `.cfg` writable by anyone else can no longer choose the
  `password_command` that runs with the caller's privileges.

- **FR-SEC-015**: Paths SHALL be canonicalised before being checked, so that a
  symlinked `.tpl` is verified at its real target. See `FR-PROJ-009`.

- **FR-SEC-016**: `--tpl-dir` SHALL be subject to the same checks, without
  exemption. See `FR-PROJ-008`.

## Template containment

- **FR-SEC-017**: A symbolic link inside `.tpl/templates/` SHALL be refused, and
  every resolved template path SHALL be canonicalised and re-checked against the
  template root. See `FR-TMPL-024` and `FR-TMPL-025`.

  *Threat closed.* `ln -s ../.cfg .tpl/templates/leak.jinja` must not turn
  `tpl template show` into a credential dump.

- **FR-SEC-018**: `tpl template check` SHALL parse only, never evaluating an
  expression, calling a function, or connecting to a database, so that it is
  safe to run against a template that has not been read. See `FR-TMPL-017` and
  `BR-TMPL-001`.

## Output and diagnostics

- **FR-SEC-019**: A `hint` carrying a runnable command SHALL be built only from
  literals and from names matching `[A-Za-z0-9_]{1,64}`; any other name yields
  no executable suggestion and appears only as data. See `FR-ERR-022` and
  `FR-ERR-023`.

  *Threat closed.* A table name is free text on the server and can contain
  semicolons, quotes, and newlines; formatting one into a suggested command is
  command injection with the caller as the interpreter.

- **FR-SEC-020**: C0 control characters, tab excepted, SHALL be escaped in every
  value the system prints or interpolates, whatever its source. See
  `FR-OUT-018`, `FR-OUT-019`, and `FR-ERR-024`.

  *Threat closed.* An escape sequence in a column comment cannot rewrite what
  the user sees, and a newline in a catalogue value cannot forge a whole
  diagnostic line in the line-oriented error format.

## Transport

- **FR-SEC-021**: `tls` SHALL default to `verify-identity`, and no DSN parameter
  may contradict the `tls` key. See `FR-CONF-012` and `FR-CONF-013`.

  *Threat closed.* Defaulting to `preferred` would let an active intermediary
  answer "no TLS", after which the handshake carries the user and password in
  clear; a `required` that does not validate is indistinguishable from no TLS
  against the same adversary.

## Availability

- **FR-SEC-022**: Every blocking phase SHALL have a deadline. See `FR-CONF-005`
  and `FR-GLOB-013`.

  *Threat closed.* A connect to a silent address, a `password_command` waiting
  on a FIFO, and a runaway loop in a template all hang the caller with no
  diagnosis, which is exactly what the "never interactive" invariant exists to
  prevent.

- **FR-SEC-023**: `tpl` SHALL never prompt, never page, and never read stdin
  except for an explicitly requested `--context -`. See `BR-CLI-003`.

## Business rules

- **BR-SEC-001**: The specification states each of these rules where it is
  enforced. This file is a cross-reference and a threat model, not a second
  source of truth; where it and an owning module differ, the owning module
  governs and the difference is a defect to be corrected here.

- **BR-SEC-002**: `tpl` never issues a write statement against a database, and
  enforces a read-only session at the engine level on every connection it opens.
  Failure to establish that session refuses the connection with `78`. There is
  no flag that disables this. The mechanism is outside the scope of this
  edition.

## Dependencies

Every requirement above names its owning module. The four principal owners are
[configuration-model.md](configuration-model.md),
[project-and-discovery.md](project-and-discovery.md),
[errors-and-exit-codes.md](errors-and-exit-codes.md), and
[template-commands.md](template-commands.md).

## Open questions

- [OQ-012](open-questions.md#oq-012) — whether a sentinel test asserting that a
  known password never appears in any byte of output becomes mandatory.
