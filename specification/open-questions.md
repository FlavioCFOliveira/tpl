---
title: Open Questions
status: draft
last-reviewed: 2026-09-09
related: [README.md, configuration-model.md, cache-commands.md, errors-and-exit-codes.md]
---

# Open Questions

## Overview

Every entry here is a point this specification cannot yet fix. Each was left
open deliberately during the interview that settled the command surface, or is a
gap this edition found while writing. None is resolved by inference: an open
question stays open until the user settles it or a measurement answers it.

An entry states the question, where it came from, why it cannot be answered now,
and what it blocks. When one is closed, the requirement it produces is written
into the owning module and the entry is removed from this file.

## Index

| Id | Question | Owner module | Blocks |
|---|---|---|---|
| [OQ-001](#oq-001) | Output of `tpl cfg database test` | `cfg-commands.md` | Implementation of `test` |
| [OQ-002](#oq-002) | Whether `test` reports effective privileges | `cfg-commands.md` | Implementation of `test` |
| [OQ-003](#oq-003) | TLS mode to driver mapping | `configuration-model.md` | Freezing TLS semantics |
| [OQ-004](#oq-004) | Enumerated DSN query parameters | `configuration-model.md` | DSN validation |
| [OQ-005](#oq-005) | Cap on `password_command` output | `configuration-model.md` | `password_command` execution |
| [OQ-006](#oq-006) | Handling of the child's stderr | `configuration-model.md` | `password_command` execution |
| [OQ-007](#oq-007) | Handling of a non-zero child exit | `configuration-model.md` | `password_command` execution |
| [OQ-008](#oq-008) | `--timeout` versus per-phase keys | `global-flags.md` | Deadline resolution |
| [OQ-009](#oq-009) | Catalogue return types and collation | `schema-commands.md` | Freezing catalogue value types |
| [OQ-010](#oq-010) | Field lists for routines, triggers, generated columns, FK rules | `schema-commands.md` | `schema table` and `schema routine` |
| [OQ-011](#oq-011) | Pre-scan on ambiguous `--format` forms | `errors-and-exit-codes.md` | Error formatting |
| [OQ-012](#oq-012) | Mandatory password sentinel test | `security.md` | The test plan |
| [OQ-013](#oq-013) | Ordering of `tpl template list` | `template-commands.md` | Determinism of the listing |
| [OQ-014](#oq-014) | Nested command path for `tpl help` | `help-and-version.md` | `help` argument parsing |
| [OQ-015](#oq-015) | Position of a global flag | `global-flags.md` | Argument parsing |
| [OQ-016](#oq-016) | Short forms for local flags | `cfg-commands.md` | Flag declarations |
| [OQ-017](#oq-017) | Flags for `password_command`, `ca_file`, `ca_path` | `cfg-commands.md` | `cfg database add` and `update` |
| [OQ-018](#oq-018) | Unknown key in a hand-written `.cfg` | `configuration-model.md` | `.cfg` validation |
| [OQ-019](#oq-019) | `password_command` written as a string in the file | `configuration-model.md` | `.cfg` validation |
| [OQ-020](#oq-020) | Cache format version, and the cached-versus-live field | `cache-commands.md` | Cache documents |
| [OQ-021](#oq-021) | Exit code for a failed cache write | `cache-commands.md` | Cache write path |
| [OQ-022](#oq-022) | Shape of `tpl cache status` output | `cache-commands.md` | `cache status` |
| [OQ-023](#oq-023) | Pre-scan on a command without `--format` | `errors-and-exit-codes.md` | Error formatting for `render` |
| [OQ-024](#oq-024) | Field list of `tpl schema info` | `schema-commands.md` | `schema info` |

## OQ-001

**What does `tpl cfg database test` print, in `text` and in `json`?**

*Origin*: left open when `test` was kept inside the entry group.
*Why it is open*: the command reports on a connection, and no decision fixed the
fields of that report.
*Blocks*: `FR-CFG-024`, which states that `test` reports the result without
saying what the result contains.

## OQ-002

**Does `tpl cfg database test` report the effective privileges, or only that the
read-only session was established?**

*Origin*: left open in the same decision.
*Why it is open*: the two answers imply different queries and different failure
modes, and the choice was deferred.
*Blocks*: `FR-CFG-024`, and the set of exit codes that command can produce.

## OQ-003

**What does each of the five TLS modes map to in the chosen driver?**

*Origin*: flagged for verification when the five modes were adopted.
*Why it is open*: the database driver has not been chosen, and the mapping must
be verified against it rather than assumed.
*Blocks*: freezing the semantics of `FR-CONF-013`.

## OQ-004

**Which query parameters may a DSN carry?**

*Origin*: flagged as open when the DSN grammar was settled.
*Why it is open*: the list must be written out, and no decision enumerated it.
The one constraint already fixed is that it contains nothing TLS-related.
*Blocks*: `FR-CONF-011`.

## OQ-005

**What is the cap on the output of `password_command`?**

*Origin*: left to settle when `password_command` was made shell-free.
*Why it is open*: an unbounded read from a child process is a denial-of-service
surface, and no limit was chosen.
*Blocks*: `FR-CONF-027`.

## OQ-006

**What happens to the child's stderr?**

*Origin*: left to settle in the same decision.
*Why it is open*: it must not be logged, per `FR-SEC-005`, but whether it is
discarded, inherited, or captured for a diagnostic was not decided.
*Blocks*: `FR-CONF-024`.

## OQ-007

**What happens when `password_command` exits non-zero?**

*Origin*: left to settle in the same decision.
*Why it is open*: the deadline case is `78`, but the non-zero exit case has no
code assigned.
*Blocks*: `FR-CONF-027` and the exit code table.

## OQ-008

**How does `--timeout` compose with the per-phase keys?**

*Origin*: the decision states both that `--timeout` is an overall budget and
that the per-phase keys exist, with precedence flag, then `.cfg`, then default.
*Why it is open*: it is not stated whether `--timeout 5` caps each phase
individually, caps their sum, or replaces every per-phase value.
*Blocks*: `FR-GLOB-012` and `FR-CONF-004`.

## OQ-009

**What does the catalogue return for table comments, column comments, and column
defaults, and in which collation?**

*Origin*: flagged for verification against the container when the
invalid-UTF-8 policy was adopted.
*Why it is open*: it must be observed against a real MariaDB instance and
confirmed against the official documentation, not inferred from knowledge of
MySQL.
*Blocks*: freezing the value types behind `FR-OUT-017`.

## OQ-010

**What exactly does the catalogue return for routines, triggers, generated
columns, and foreign-key rules?**

*Origin*: flagged for verification against the container when the scope of
`schema table` and `schema routines` was settled.
*Why it is open*: the same reason as OQ-009.
*Blocks*: the field lists behind `FR-SCH-007` and `FR-SCH-009`.

## OQ-011

**How does the argument pre-scan behave on ambiguous forms?**

*Origin*: the decision states that the pre-scan is specified in detail,
including its behaviour on ambiguous forms, but the detail was not recorded.
*Why it is open*: cases such as `--format` as the last token with no value,
`--format json` appearing after `--`, or `--format=json` appearing twice, each
need a stated outcome.
*Blocks*: `FR-ERR-017`.

## OQ-012

**Does a mandatory sentinel test — asserting that a known password never appears
in any byte of output — become part of the contract?**

*Origin*: recorded as worth reconsidering when the test plan is written.
*Why it is open*: it was deliberately deferred to the test plan rather than
decided with the logging rules.
*Blocks*: nothing in the surface; it strengthens `FR-SEC-005`.

## OQ-013

**In what order does `tpl template list` print names?**

*Origin*: a gap found while writing this edition. The example listing in the
decision is not in lexicographic order.
*Why it is open*: determinism requires a fixed order, and no decision states
which.
*Blocks*: `FR-TMPL-013`.

## OQ-014

**Does `tpl help` accept a nested command path?**

*Origin*: a gap found while writing this edition.
*Why it is open*: the decision establishes `tpl help <cmd>`, but the tree now
reaches three levels, and it is not stated whether
`tpl help cfg database add` is valid.
*Blocks*: `FR-HELP-001` and `FR-HELP-016`.

## OQ-015

**Where may a global flag appear on the command line?**

*Origin*: a gap found while writing this edition.
*Why it is open*: every example shows a global flag before the command name, and
no decision states whether `tpl schema tables -d shop` is accepted.
*Blocks*: `FR-GLOB-002`.

## OQ-016

**Do local flags keep the short forms the root `README.md` declares?**

*Origin*: a gap found while writing this edition. The root document declares
`-s` for `--set`, and `-H`, `-P`, `-u` for the entry flags; the decision that
enumerated the entry flags listed no short forms.
*Why it is open*: the enumeration is the later and more specific source, so this
edition declares long forms only.
*Blocks*: `FR-RND-008` and `FR-CFG-027`.

## OQ-017

**Do `password_command`, `ca_file`, and `ca_path` have flags on
`tpl cfg database add` and `update`?**

*Origin*: a gap found while writing this edition. The three keys exist in the
key space, but the flag-to-key mapping covers only six flags.
*Why it is open*: either the three are set only through `tpl cfg set`, or three
flags are missing from the mapping.
*Blocks*: `FR-CFG-027`.

## OQ-018

**What happens when `.tpl/.cfg` contains a key outside the enumerated space?**

*Origin*: a gap found while writing this edition. `tpl cfg set` rejects an
unknown key with `64`, but the file may be hand-written.
*Why it is open*: rejecting the file with `78` and ignoring the key are both
defensible, and neither was decided.
*Blocks*: `FR-CONF-002`.

## OQ-019

**Is a `password_command` written as a TOML string, rather than an array,
accepted in a hand-edited `.cfg`?**

*Origin*: a gap found while writing this edition.
*Why it is open*: the decision fixes the stored form as an array and fixes the
splitting rule for a string supplied to a command, but says nothing about a
string found in the file.
*Blocks*: `FR-CONF-023` and `FR-CONF-025`.

## OQ-020

**What is the format version of a cached document, and which field reports that
a read was served from the cache?**

*Origin*: the decisions require both a format version and a way for `json`
output to expose cached versus live, without fixing either.
*Why it is open*: both are contract surface and must be named explicitly.
*Blocks*: `FR-CACHE-005` and `FR-CACHE-012`.

## OQ-021

**What exit code does a failed cache write produce?**

*Origin*: a gap found while writing this edition.
*Why it is open*: a read command that succeeds against the server but cannot
write `.tpl/.cache/` has no stated outcome. Failing the read and succeeding with
a warning are both defensible.
*Blocks*: `FR-CACHE-007`.

## OQ-022

**What is the `text` and `json` shape of `tpl cache status`?**

*Origin*: a gap found while writing this edition.
*Why it is open*: the fields are named — entry, load time, object counts — but
their layout, names, and types are not.
*Blocks*: `FR-CACHE-025`.

## OQ-023

**How does the error pre-scan behave for a command that does not declare
`--format`?**

*Origin*: a gap found while writing this edition. `tpl render` produces rendered
text and declares no `--format`, yet `tpl render x --format json` would trigger
the pre-scan and then fail as an unknown flag.
*Why it is open*: emitting that `64` as JSON is arguably correct and arguably
confusing, and no decision covers it.
*Blocks*: `FR-ERR-017` and `FR-RND-027`.

## OQ-024

**Which fields does `tpl schema info` report?**

*Origin*: a gap found while writing this edition. The root `README.md` names
"name, version, charset, collation"; no decision confirms the list.
*Why it is open*: the field list is contract surface in `json`, and it must be
verified against the container like every other catalogue field list.
*Blocks*: `FR-SCH-003`.
