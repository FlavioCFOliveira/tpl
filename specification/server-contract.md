---
title: Server Contract
status: draft
last-reviewed: 2026-09-09
related: [catalogue-coverage.md, privileges-and-completeness.md, security.md, performance-requirements.md]
---

# Server Contract

## Overview

This file states what `tpl` requires of a server and what it promises in return.
Two things are settled here. The first is which servers are supported: MariaDB
from a stated minimum version, and nothing else. The second is the read-only
promise, which the first edition stated as a session setting and which is
restated here in the two parts it actually has — a closed list of statements
`tpl` will issue, and a session setting that catches what the list already
prevents.

## Scope

In scope: the minimum server version, the refusal of a server that is not
MariaDB, the treatment of fields introduced after the floor, the closed
statement list, the read-only session and its read-back, and the connection
count.

Out of scope: which fields are read, which belongs to
[catalogue-coverage.md](catalogue-coverage.md); the handling of a read that
returns less than it should, which belongs to
[privileges-and-completeness.md](privileges-and-completeness.md); connection
settings and TLS, which belong to
[configuration-model.md](configuration-model.md); and the text of any statement.

## Actors

- **Server**, which must satisfy the version requirement.
- **`tpl`**, which must satisfy the statement restriction.
- **Database administrator**, who can verify both from the server side.

## Supported servers

- **FR-SRV-001**: The minimum supported server SHALL be MariaDB 10.6.

  *Rationale.* A floor of 11.4 would remove every version-conditional field, but
  10.11 is a long-term-support release and is still in wide use. 10.6 is the
  oldest floor that does not require the model to carry a version-conditional
  field for something a template would need every day.

- **FR-SRV-002**: The system SHALL determine the server product and version
  before reading any catalogue data.

- **FR-SRV-003**: IF the server is not MariaDB, THEN the system SHALL exit `78`
  (`EX_CONFIG`) with `kind: server_not_mariadb`, and SHALL NOT read the
  catalogue.

  *Rationale.* MySQL is refused rather than attempted because three verified
  divergences make the model **silently wrong** rather than empty. The catalogue
  quotes a string default in MariaDB and does not in MySQL, so the
  literal-versus-expression discriminant of `FR-CTX-012` inverts. MySQL's check
  constraint table carries no table name, so `FR-CAT-015` cannot attribute a
  constraint to its table. MySQL's column table carries no generated-column
  flag, so `FR-CAT-009` cannot distinguish a generated column from an ordinary
  one. Each of the three produces plausible output that is wrong.

  *Rejected.* Attempting a non-MariaDB server with a warning on stderr, which
  leaves the exit code at `0` and hands the caller a wrong model.

- **FR-SRV-004**: WHERE the model defines a field that the connected server does
  not provide because the field was introduced after the floor of `FR-SRV-001`,
  the system SHALL emit that field as `null`.

- **FR-SRV-005**: The shape of the document SHALL be constant across every
  supported server version. A field SHALL NOT be omitted because the server does
  not provide it, per `FR-OUT-012`.

## The closed statement list

- **FR-SRV-006**: The system SHALL issue only statements drawn from the
  following closed list:

  | Statement | Purpose |
  |---|---|
  | `SELECT` against `information_schema.*` | Reading the catalogue |
  | The server version probe | `FR-SRV-002` |
  | The read-only session statement issued at connection start | `FR-SRV-008` |

- **FR-SRV-007**: The system SHALL NOT issue any other statement. It SHALL issue
  no DDL, no DML, no `SHOW`, no statement against any schema other than
  `information_schema`, and SHALL NOT invoke an external process such as a dump
  utility to read structure.

- **BR-SRV-001**: The closed list is the read-only guarantee. It is a property of
  what `tpl` is built to send, it holds whatever the server permits, and it is
  the only part of the promise that prevents rather than detects.

## The read-only session

- **FR-SRV-008**: The system SHALL additionally set the session read-only at the
  engine level on every connection it opens.

- **FR-SRV-009**: The system SHALL read the session state back and SHALL confirm
  that the setting took effect.

- **FR-SRV-010**: IF the setting cannot be applied, or the read-back does not
  confirm it, THEN the system SHALL exit `78` and SHALL NOT read the catalogue.

- **FR-SRV-011**: There SHALL be no flag, configuration key, or environment
  condition that disables `FR-SRV-006` through `FR-SRV-010`.

- **BR-SRV-002**: The session setting is defence in depth and is stated as such.
  It makes a write **fail**; it does not stop the connection from attempting
  one, and it constrains the transaction rather than the session. Describing it
  as prevention, which the first edition and the root documents do, overstates
  it: a connection under that setting can still send a write and receive an
  error, and nothing about the setting alone tells a reader that `tpl` never
  sends one.

  *Rejected.* Keeping the promise as the first edition stated it — a session
  setting presented as prevention — which reads as a guarantee and is a
  detection.

## Verification

- **FR-SRV-012**: The closed list of `FR-SRV-006` SHALL be verified by an
  integration test that observes the statements the server actually receives.

- **FR-SRV-013**: The read-back of `FR-SRV-009` SHALL be verified by an
  integration test that exercises both outcomes: the setting taking effect, and
  the setting failing to take effect.

- **FR-SRV-014**: The connection count of an invocation SHALL be as fixed by
  `NFR-PERF-004`, and SHALL be verifiable from the server side.

- **BR-SRV-003**: All three verifications are observations made from outside the
  process, on the server. A promise about what a process sends that can only be
  checked by reading that process's own source is not a promise a caller can
  rely on.

## Dependencies

- [catalogue-coverage.md](catalogue-coverage.md) — the fields whose absence
  `FR-SRV-004` turns into `null`.
- [context-document.md](context-document.md) — `FR-CTX-012`, the discriminant
  that a non-MariaDB server would invert.
- [privileges-and-completeness.md](privileges-and-completeness.md) — the other
  reason a read can return less than the model defines.
- [performance-requirements.md](performance-requirements.md) — `NFR-PERF-003`
  and `NFR-PERF-004`, the connection invariants.
- [security.md](security.md) — `BR-SEC-002`, the cross-cutting statement of the
  read-only promise.
- [errors-and-exit-codes.md](errors-and-exit-codes.md) — `69`, `77`, and `78`.

## Open questions

- [OQ-042](open-questions.md#oq-042) — what the version probe returns, and how
  MariaDB is distinguished from a server reporting a MariaDB-compatible version
  string.
- [OQ-044](open-questions.md#oq-044) — the outcome for a MariaDB server older
  than the floor of `FR-SRV-001`.
- [OQ-045](open-questions.md#oq-045) — which fields of the model were introduced
  after 10.6 and therefore fall under `FR-SRV-004`.
- [OQ-046](open-questions.md#oq-046) — how the session read-back of `FR-SRV-009`
  is performed within the closed statement list of `FR-SRV-006`.
