---
title: Open Questions
status: approved
last-reviewed: 2026-09-10
related: [README.md, catalogue-coverage.md, context-document.md, server-contract.md]
---

# Open Questions

## Overview

Every entry here is a point this specification cannot yet fix. None is resolved
by inference: an open question stays open until the user settles it or a
measurement answers it. An entry states the question, where it came from, why
it cannot be answered now, and what it blocks. When one is closed, the
requirement it produces is written into the owning module and the entry moves
to the *Closed* table below; its identifier is retired and never reused, per
the identifier scheme of the [README](README.md#identifier-scheme).

**Nothing is open. The index is empty, and the specification is complete.**
All seventy-five entries this corpus has raised are closed. The twenty the
sixth edition left waiting for a recorded observation were observed against
all four series of `FR-SRV-015` on 2026-09-10 and written into their owning
modules; `OQ-075`, the one the evidence itself opened, was settled by the
product owner; and `OQ-042`, the last to stand, closed on the necessary
condition of `FR-SRV-041` with its limit written into that requirement.

**Complete means that no point of this specification is waiting on someone.**
It does not mean the corpus has stopped moving. Three obligations recur and
none of them is an open question: `FR-SRV-019` requires the supported-series
table to be re-verified before every release, `NFR-PERF-020` turns a
provisional budget into a measured one, and any requirement resting on the
fourth provenance of the [README](README.md#provenance) is falsifiable by a
later observation. Each is work with an owner and a trigger, which is what
distinguishes it from an entry here.

If a point of this specification appears unsettled, it is a defect to be
reported and corrected, or a new question to be raised with the next unused
identifier — never an old one revived.

## Index

**Empty.** No question is open. An entry is added here only when this
specification reaches a point it cannot fix, and it is added with the next
unused `OQ` number; the last assigned was `OQ-075`.

## Closed

An entry is closed when the requirement it produced is written into the owning
module. Two are closed differently. An entry is **dissolved** when the
mechanism it governed ceases to exist, so there is no answer to record because
there is no longer a question. An entry closes **on a stated limit** when the
evidence it asks for is shown to be unobtainable rather than merely
unrecorded: what would have been the answer becomes a limit written into the
requirement, in the form the [README](README.md#writing-conventions) fixes,
because an entry no achievable observation can close is not an open question.
`OQ-042` is the only one closed this way. In every case the identifier is
retired and never reused, so a gap in the sequence is expected rather than a
defect.

| Id | Question | Answered by | Closed in |
|---|---|---|---|
| OQ-001 | Output of `tpl cfg database test` | `FR-CFG-035`, `FR-CFG-039` | Third edition |
| OQ-008 | `--timeout` versus the per-phase keys | `FR-GLOB-011`, `FR-GLOB-012`, `FR-GLOB-013`, `FR-CONF-004` | Third edition |
| OQ-020 | Cache format version, and the cached-versus-live field | `FR-CDOC-001` through `FR-CDOC-005`, `FR-CDOC-009` through `FR-CDOC-011` | Second edition |
| OQ-022 | Shape of `tpl cache status` output | `FR-CACHE-034`, `FR-CACHE-035` | Third edition |
| OQ-061 | Whether `--help` runs before project discovery | `FR-PROJ-025`, `NFR-PERF-005`, `FR-ERR-006` | Third edition |
| OQ-062 | The `json` document shape of every read command | `FR-OUT-024` through `FR-OUT-032`, and the `data` shapes of `BR-OUT-002` | Third edition |
| OQ-063 | The order of the collections `NFR-DET-002` did not name | `NFR-DET-002` | Third edition |
| OQ-064 | The outcome of a read whose result set is empty | `FR-OUT-033` through `FR-OUT-037` | Third edition |
| OQ-065 | The durability of a `.tpl/.cfg` rewrite | `FR-CFG-041`, `FR-CFG-042` | Third edition |
| OQ-066 | `password` beside `password_command`, and either beside `dsn` | `FR-CONF-006`, `FR-CONF-007` | Third edition |
| OQ-067 | How `70` is produced, and how it is exercised | `FR-ERR-030` through `FR-ERR-032`, `BR-ERR-001` | Third edition |
| OQ-068 | What each registered filter and test does | `FR-ENV-030` through `FR-ENV-043`; residue in `OQ-070` through `OQ-072` | Third edition |
| OQ-069 | The shape of the `vars`, `tpl`, and `now` context variables | `FR-CTX-026` through `FR-CTX-030` | Third edition |
| OQ-044 | The outcome for a server below the supported window | `FR-SRV-020`, `FR-SRV-021` | Fourth edition |
| OQ-073 | The outcome for a server newer than the supported window | `FR-SRV-031` through `FR-SRV-033`, `BR-SRV-008`, `BR-SRV-009`, `FR-CTX-034` | Fourth edition |
| OQ-074 | Whether `tpl cfg database test` applies the version gate | `FR-SRV-002` as amended, `FR-SRV-034`, `FR-CFG-024`, `FR-CFG-039` | Fourth edition |
| OQ-002 | Whether `tpl cfg database test` reports effective privileges | `FR-CFG-044`, `FR-CFG-045`, the `can_read_catalogue` field of `FR-CFG-039`, and the fourth step of `FR-CFG-024` | Fifth edition |
| OQ-004 | Which query parameters a DSN may carry | `FR-CONF-011` as amended — none — and `FR-CONF-012`, now a special case of it | Fifth edition |
| OQ-005 | The cap on `password_command` output | `FR-CONF-031` — 4096 bytes; exceeding it is `78` | Fifth edition |
| OQ-006 | What happens to the child's stderr | `FR-CONF-032` — the null device | Fifth edition |
| OQ-007 | What happens when `password_command` exits non-zero | `FR-CONF-033` — `78` | Fifth edition |
| OQ-011 | Pre-scan on ambiguous `--format` forms | **Dissolved.** `FR-ERR-033` withdraws the JSON error document and with it the pre-scan of `FR-ERR-017`; no mechanism is left to specify | Fifth edition |
| OQ-012 | Whether a mandatory password sentinel test is contract | `BR-SEC-003` | Fifth edition |
| OQ-013 | The order of `tpl template list` | `FR-TMPL-013` | Fifth edition |
| OQ-014 | Whether `tpl help` accepts a nested command path | `FR-HELP-026` through `FR-HELP-029` | Fifth edition |
| OQ-015 | Where a global flag may appear on the command line | `FR-CLI-024`, with `FR-CLI-001` as amended | Fifth edition |
| OQ-016 | Short forms for local flags | `FR-GLOB-024` — there are none | Fifth edition |
| OQ-017 | Flags for `password_command`, `ca_file`, and `ca_path` | `FR-CFG-027` as amended, `FR-CFG-046`, `FR-CFG-047`, and `FR-CFG-033` as amended | Fifth edition |
| OQ-018 | An unknown key in a hand-written `.cfg` | `FR-CONF-034`, `BR-CONF-004` | Fifth edition |
| OQ-019 | `password_command` written as a TOML string in the file | `FR-CONF-035`, `BR-CONF-004` | Fifth edition |
| OQ-021 | The outcome of a failed cache write | `FR-CACHE-036` — silent success | Fifth edition |
| OQ-023 | Pre-scan on a command that declares no `--format` | **Dissolved.** The same mechanism, withdrawn by the same requirement; `FR-RND-027` records the ordinary outcome that remains | Fifth edition |
| OQ-043 | Whether `referenced_by` embeds | `FR-CTX-010`, with `FR-CTX-009` as amended | Fifth edition |
| OQ-046 | The read-back within the closed statement list | The fourth entry of `FR-SRV-006`, with `FR-SRV-012` as amended | Fifth edition |
| OQ-047 | The shape of the `restricted` field | `FR-PRIV-016` | Fifth edition |
| OQ-048 | Whether an incomplete object may be cached | `FR-CACHE-037` — it may not | Fifth edition |
| OQ-049 | The pinned engine minor version | `FR-ENV-003` as amended: the pin must exist, be recorded outside this corpus, and be cited; the number never enters the corpus | Fifth edition |
| OQ-050 | The contract group and behaviour of `escape` | `FR-ENV-007` as amended, and `FR-ENV-044` | Fifth edition |
| OQ-051 | Budget: `tpl --version` | `NFR-PERF-014`, carrying a provisional figure under `NFR-PERF-019` and the gate of `NFR-PERF-020` | Fifth edition |
| OQ-052 | Budget: `tpl --help` | The same | Fifth edition |
| OQ-053 | Budget: startup to the first useful byte | The same | Fifth edition |
| OQ-054 | Budget: `tpl schema dump` over `WL-001` | The same | Fifth edition |
| OQ-055 | Budget: cache-served read over `WL-003` | `NFR-PERF-014` and `NFR-PERF-015`; named and unvalued, per `BR-PERF-006` | Fifth edition |
| OQ-056 | Budget: the failure path | `NFR-PERF-014`; named and unvalued | Fifth edition |
| OQ-057 | Budget: `tpl help --format json` | `NFR-PERF-014`; named and unvalued | Fifth edition |
| OQ-058 | Budget: the 200-invocation loop | `NFR-PERF-014`; named and unvalued | Fifth edition |
| OQ-059 | Budget: peak resident memory over `WL-001` | `NFR-PERF-014`, carrying a provisional figure; `FR-CTX-010` records why it is the likeliest of them to move | Fifth edition |
| OQ-060 | The byte scalar N of `WL-002` | `WL-002` as amended: unvalued, and scheduled under `NFR-PERF-020` | Fifth edition |
| OQ-071 | What `sql_type` does beyond `column_type` | `FR-ENV-039` — the normalised type name, which is the `data_type` of `FR-CTX-015` | Fifth edition |
| OQ-003 | What each of the five TLS modes maps to in the chosen driver | `FR-CONF-038`, with `FR-CONF-037` and `FR-CONF-039` | Sixth edition |
| OQ-025 | The table types reported, and whether temporary tables appear | `FR-CAT-031`, `FR-CAT-032` | Sixth edition |
| OQ-041 | How an unreadable view is reported | `FR-PRIV-018`, and `FR-PRIV-011` as amended | Sixth edition |
| OQ-045 | Which fields differ among the four supported series | `FR-SRV-038`, and the register of `FR-SRV-036` | Sixth edition |
| OQ-070 | The escaping of a backtick inside an identifier | `FR-ENV-045` — doubling | Sixth edition |
| OQ-009 | Catalogue return types for comments and defaults | `FR-CAT-039`, with `FR-CAT-036` and `FR-CTX-037` | Seventh edition |
| OQ-010 | Field lists for routines, triggers, generated columns, and foreign-key rules | `FR-CAT-045`, `FR-CAT-048`, `FR-CAT-050`, `FR-CAT-051` — the four entries it was the umbrella over, and it had no query of its own | Seventh edition |
| OQ-024 | The metadata fields of the `database` object | `FR-CTX-036` | Seventh edition |
| OQ-026 | The column attribute string for auto-increment | `FR-CAT-041`, with `FR-CAT-035` | Seventh edition |
| OQ-027 | Literal, expression, and explicit `DEFAULT NULL` | `FR-CTX-037`, with `FR-CTX-012` and `FR-CTX-013` as amended | Seventh edition |
| OQ-028 | The textual form of `column_type` | `FR-CTX-038`, with `FR-CAT-038` | Seventh edition |
| OQ-029 | `ENUM` member delimiting and escaping | `FR-CTX-039`, with `FR-CAT-034` | Seventh edition |
| OQ-030 | Which type parts the catalogue populates | `FR-CTX-040` | Seventh edition |
| OQ-031 | Per-column character set and collation | `FR-CTX-041` | Seventh edition |
| OQ-032 | Index fields and the folding order column | `FR-CAT-042` | Seventh edition |
| OQ-033 | How the primary key is reported | `FR-CAT-043`, with `FR-CAT-011` as amended and `FR-CAT-044` | Seventh edition |
| OQ-034 | Foreign-key fields and the four reachable rule spellings | `FR-CAT-045`, with `FR-CAT-033` | Seventh edition |
| OQ-035 | `CHECK` constraint fields and the remaining level values | `FR-CAT-046`, with `FR-CAT-037` and `FR-CAT-038` | Seventh edition |
| OQ-036 | View fields beyond the definition | `FR-CAT-047` | Seventh edition |
| OQ-037 | Routine fields | `FR-CAT-048`, with `FR-PRIV-017` | Seventh edition |
| OQ-038 | Routine parameter fields and ordering | `FR-CAT-049` | Seventh edition |
| OQ-039 | Trigger fields | `FR-CAT-050` | Seventh edition |
| OQ-040 | Generated-column expression and storage kind | `FR-CAT-051` | Seventh edition |
| OQ-042 | How MariaDB is distinguished from a server reporting a MariaDB-compatible version string | `FR-SRV-040`, and `FR-SRV-041` — the necessary condition, closed **on a stated limit**: a server determined to pass as MariaDB will pass | Seventh edition |
| OQ-072 | The membership of the three type families | `FR-ENV-046` | Seventh edition |
| OQ-075 | A field whose value differs because the servers' defaults differ | `FR-SRV-039`, with `BR-SRV-006` and `FR-SRV-026` as amended | Seventh edition |

