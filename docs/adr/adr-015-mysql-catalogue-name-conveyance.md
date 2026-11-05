---
id: ADR-015
title: How a caller-supplied name reaches a MySQL catalogue statement
status: accepted
decided: 2026-11-05
last-reviewed: 2026-11-05
requirements: [FR-MYSQL-048, FR-SRV-043, FR-MYSQL-015, FR-MYSQL-016, FR-CFG-044, BR-MYSQL-002]
supersedes: []
superseded-by: null
---

# ADR-015 — How a caller-supplied name reaches a MySQL catalogue statement

## Status

Accepted, 2026-11-05, for rmp `#352`.

The decision is the user's, of 2026-11-05, relayed by the session coordinator:
the MySQL reader moves from Prepare+Execute to the text protocol **only if**
`security-researcher` certified a conveyance rule. It did, the same day, and
the rule it certified is Decision point 2. The user's condition has a second
half that is not yet met: a test showing that a name holding a quotation mark,
a backslash and a backtick reads that database and no other (rmp `#352`,
comment 1635). *Consequences* states what follows if that test fails.

## Context

**The corpus fixes the property and hands out the means.** `FR-SRV-043`, which
`FR-MYSQL-048` holds for a `mysql` entry with the closed list of
`FR-MYSQL-015`, requires a name the system did not write itself to change no
part of a statement but the value its rows are compared with, whatever
characters it holds, and never to be read as syntax or as a pattern. Both
requirements leave the means open — "a value the driver binds and a value
written into the text under the server's quoting rules can both meet the
property" — and name the technical specification as where the means is
recorded. `docs/spec-technical/security.md` reports, as a recorded gap, that
no settled entry fixes it.

**Why this is a record and not an entry of the decision register.** Rule R4's
first limb does not admit it: the requirements send the means to the technical
specification, not to this register. The second limb does. The decision is
architectural: it changes the wire protocol of every catalogue statement of a
provider, from `COM_STMT_PREPARE` and `COM_STMT_EXECUTE` to `COM_QUERY`, with
the row format the driver decodes; it moves the defence of the one value a
caller supplies to the server from the driver's structural separation of value
and text to a composition rule the code must keep, at the seam `BR-MYSQL-002`
makes the preventing part of the read-only promise; and it makes the two server
providers convey the name differently. Its alternatives must survive it,
because the strongest of them — a quoted literal under the server's escaping —
is the one a later change reaches for first, and it is refuted only by evidence
an entry would not keep in full. The technical specification records the means
by citing this record.

**Where the name reaches the server.** For a `mysql` entry the only
caller-supplied value that reaches the server is the database name the entry
gives (`FR-CONF-041`), in the `database` key or the path of `dsn`. It reaches
the login packet as a protocol field, and the single `?` of each of the twelve
statements of the plan and of the probe of `FR-CFG-044`, all issued by
`src/mysql/catalogue.rs`, `issue`, which binds it today
(`query(...).bind(schema)`). Named tables, views and routines are matched in
memory, and the session statements are constants. The login field is outside
this record.

**What binding costs.** Prepare+Execute costs every MySQL catalogue read a
constant 2.8 to 3.8 ms; the text protocol cuts a direct read of `freight` by
18.8%, measured on `aarch64-apple-darwin` against MySQL `8.4` and `9.7`
(rmp `#352`, from `#351`). The driver of `ADR-003` sends a statement it is
handed with no arguments as `COM_QUERY` and decodes the answer in the text
format; a statement with arguments is prepared and executed.

**What the session can do to a composed text.** On the fixture's `8.4.13` and
`9.7.4` servers the session runs the server's global `sql_mode` plus
`IGNORE_SPACE`, with `utf8mb4` and `utf8mb4_general_ci`; the server's global
`sql_mode` and `init_connect` reach it. The compared catalogue columns are
`utf8mb3_bin`, of coercibility 2. A literal written into the text is read
under `character_set_connection` and the session's `sql_mode`, both of which
the server, not `tpl`, can set.

## Decision

1. **The MySQL reader sends its catalogue statements over the text protocol.**
   The twelve statements of the plan and the probe of `FR-CFG-044`, issued by
   `src/mysql/catalogue.rs`, `issue`, are handed to the driver with no
   arguments, and the driver sends each as `COM_QUERY`. The statements stay
   those of `FR-MYSQL-015`; only the conveyance of the name changes.

2. **The name is conveyed by one rule, certified on 2026-11-05:**

   - one function, `literal(name: &str)`, emits `_utf8mb4 X'`, then two
     uppercase hexadecimal digits for each byte of `name.as_bytes()`, then
     `'`, and nothing else;
   - its output replaces the statement's single `?`, and only as the
     right-hand side of `<INFORMATION_SCHEMA column> =`;
   - it is never written beside `COLLATE`, `CONVERT`, `LIKE` or `REGEXP`, and
     never compared with another literal;
   - the composed text is handed to the driver through `AssertSqlSafe` at that
     one site, and nowhere else in the reader.

   **The rule needs no session precondition.** The composed text is pure ASCII
   and holds no `\`, `"`, `|`, `;` or `--`, so no `sql_mode` setting and no
   client character set changes how it is read: MySQL refuses a client
   character set that is not ASCII-compatible. The introducer fixes the
   literal's character set, and the absence of `COLLATE` leaves the column's
   collation deciding the comparison, as it decides it for the bound value
   today — the comparison `FR-SRV-043` leaves to the server.

3. **The scope is the MySQL reader.** The MariaDB reader keeps binding its
   values. The rule should serve there on the same grounds, and would carry
   the names of tables, views and routines there too; nothing was run against
   MariaDB, and this record decides nothing for it.

## Alternatives rejected

- **A quoted string literal, under any escaping.** Refused on evidence. Under
  quote doubling, under `NO_BACKSLASH_ESCAPES`, and under the client character
  sets `gbk`, `big5` and `gb18030`, injection queries returned all 7 schemata
  of the server. The server reads escapes per `character_set_connection`, so a
  multi-byte character whose trail byte is `0x5c` turns an escape the client
  wrote into a quotation mark the server closes on. No escaping the client
  applies can know the character set the server will read the text under,
  because the server's own settings reach the session.

- **A hexadecimal literal with an explicit `COLLATE`.** Refused: it overrides
  the column's collation, which `FR-SRV-043` leaves to the server, and under
  `lower_case_table_names` `1` or `2` it would change which spellings name one
  database. This was reasoned from the MySQL manual and not run.

- **Selecting the database at connect and comparing with `DATABASE()`.**
  Refused on evidence: the login packet's database field ends at a NUL byte,
  so an entry naming `freight\0x` read the 27 tables of `freight`, where the
  bind reads none. The name would then reach the server through a protocol
  field that truncates it.

- **Keeping the bind.** It has no gap in safety: the driver keeps the value
  out of the text by construction. Refused by the user for its cost, the 2.8
  to 3.8 ms per read the text protocol recovers, once a rule with no session
  precondition was certified.

- **Composing without a certified rule.** Refused by the user on 2026-11-05,
  together with keeping the bind unconditionally (rmp `#352`, comment 1635).

## Consequences

**The certification is evidence over a finite set, and it is what the safety
rests on.** 49 names, 12 statements and 16 session states — `NO_BACKSLASH_ESCAPES`,
`ANSI_QUOTES`, `ANSI`, `PIPES_AS_CONCAT`, `SET NAMES` with `gbk`, `big5`,
`sjis`, `cp932`, `gb18030`, `latin1`, `ascii` and `binary`, and collation
variants — on MySQL `8.4.13` and `9.7.4`, each compared with the bound result:
the same rows in 576 of 588 checks, the other 12 being `𠮷`, an error under
both conveyances and refused at login first; the name intact on its round trip
for 49 of 49; and 0 rows from each of the 128 injection queries. The bind was
safe by construction; the literal is safe by this argument and this evidence.
**The rule is re-verified at every change to the supported MySQL series and at
every move of `ADR-003`'s pin**, because either can change how the server
reads a literal or what the driver sends for a statement without arguments,
and this record is amended in place with the reading.

**The user's second condition is owed by the implementation.** The test of
`FR-MYSQL-048` reads the target database of item 17 of `FR-MYSQL-044` and its
four decoys, on `8.4` and `9.7`. If it shows the target read as any other
database, or any other database read in its place, the condition is not met:
the reader keeps binding, and this record is withdrawn.

**The text protocol changes what the reader decodes.** Every value arrives in
the text format rather than the binary one, so the decoding of
`src/mysql/catalogue/row.rs` is re-verified against both series; the twelve
tables' fields reach the read as before, under `FR-MYSQL-015`.

**The statement is no longer a constant of the crate.** Its text is composed
at the one site of point 2, from a constant and the literal. The technical
specification's statements that each MySQL statement is a constant with the
name bound as a parameter (`docs/spec-technical/architecture.md`,
`interfaces.md`, `security.md`) become false of the `mysql` reader when the
code changes, and are owed by the owner of those documents, citing this
record.

**`AssertSqlSafe` is the driver's own marker of a text the caller vouches
for.** Confining it to one site keeps the vouching reviewable: a second use,
or a use whose text holds anything but a constant and one literal, is outside
this record.

**The MariaDB reader is unchanged**, so the two server providers convey the
name differently. The `mariadb` fixture holds no database whose name a quoting
or a pattern could read as another (`FR-SRV-043`, note of the eightieth
edition), so a move of the MariaDB reader would first need that fixture.

**Under R3, these facts live here alone:** that the MySQL reader uses the text
protocol for its catalogue statements, the conveyance rule and its limits, the
scope, the rejected alternatives and their evidence. The property is
`FR-SRV-043`'s and `FR-MYSQL-048`'s, cited and not restated.

## Sources

| Claim | Source | Consulted |
|---|---|---|
| The property, its rationale, and the statement that the means is the technical specification's to record | `specification/server-contract.md`, `FR-SRV-043`; `specification/mysql-contract.md`, `FR-MYSQL-048`; eightieth edition at `a4f81c9` | 2026-11-05 |
| The gap: no settled entry fixes how a caller-supplied name is conveyed | `docs/spec-technical/security.md`, *Recorded gap — the one caller-supplied value that reaches the server*, at `a4f81c9` | 2026-11-05 |
| The user's decision: compose only if a rule is certified and a test shows the target read and no other; keeping the bind unconditionally and composing unconditionally refused | rmp `#352` work log, `DECISION` entry of 2026-11-05 (comment 1635); the session coordinator's brief | 2026-11-05 |
| The cost of Prepare+Execute, 2.8 to 3.8 ms per read; the 18.8% cut of a direct read of `freight`; `issue` binds the name | rmp `#352`, functional requirements, from rmp `#351` (comment 1619); `src/mysql/catalogue.rs` line 264, at `a4f81c9` | 2026-11-05 |
| The surface: the database name alone reaches the server, in the login packet and the single `?` of the twelve statements and the probe; the session's `sql_mode`, character set and collation; the compared columns `utf8mb3_bin`, coercibility 2 | rmp `#352` work log, `FINDING` entry (comment 1686) | 2026-11-05 |
| The certification: 49 names × 12 statements × 16 session states on MySQL `8.4` and `9.7`; 576 of 588 checks equal to the bind, 49 of 49 round trips intact, 0 rows from 128 injection queries | rmp `#352` work log, `TEST` entry (comment 1687) | 2026-11-05 |
| The rule; the rejected quoted literal, `COLLATE` and `DATABASE()`, with their evidence; no session precondition | rmp `#352` work log, `DECISION` entry (comment 1688); the session coordinator's brief | 2026-11-05 |
| The fixture the proving test needs; the row format changes | rmp `#352` work log, `FINDING` entry (comment 1689); `specification/mysql-contract.md`, `FR-MYSQL-044` item 17 (comment 1696) | 2026-11-05 |
| MariaDB recorded only, nothing run | rmp `#352` work log, `NOTE` entry (comment 1690) | 2026-11-05 |
| A statement with arguments is prepared and executed and decoded in the binary format; one with none is sent as `COM_QUERY` and decoded in the text format; `raw_sql` takes no arguments; `AssertSqlSafe` wraps a text the caller vouches for | `vendor/sqlx-mysql-0.9.0/src/connection/executor.rs` lines 123–179; `vendor/sqlx-core-0.9.0/src/raw_sql.rs` lines 119 and 132–138, `src/sql_str.rs` lines 52–114 | 2026-11-05 |
