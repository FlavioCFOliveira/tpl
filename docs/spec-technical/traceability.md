---
title: Traceability
status: draft
last-reviewed: 2026-09-11
related: [README.md, open-decisions.md]
---

# Traceability

## What this is

The mapping between the functional specification and this folder, in both
directions.

- **Forward**, sections 1 to 26: one section per file of `specification/`,
  stating the technical concerns that file's requirements **force** on the
  implementation, the requirement identifiers or file section each is drawn
  from, and the technical document that answers it.
- **Reverse**, at the end: from every file of `specification/` to the documents
  that answer it, and the five concerns the corpus forces as a whole rather
  than through any one file.

`specification/` holds **26 files**: 25 requirement modules and `README.md`,
the index. All 26 are covered. The mapping was harvested against the seventh
edition and reconciled against the **eighth** on 2026-09-11, which changed the
rows drawn from `cli-contract.md`, `configuration-model.md`,
`project-and-discovery.md`, `errors-and-exit-codes.md`, `server-contract.md`,
`performance-requirements.md`, `glossary.md` and `upstream-divergences.md`.

This file derives concerns. It states no requirement, adds no requirement, and
reproduces no requirement text. Where a concern is cited to an identifier, the
identifier is the authority and the wording here is a summary.

## Reading key

| Column | Meaning |
|---|---|
| Technical concern | A decision or mechanism the implementation must have, that the functional specification does not fix |
| Drawn from | The requirement identifiers or the file section that forces it |
| Doc | The technical document that answers it, by the short names of [README.md](README.md#the-documents) |
| `OD-nn` | An entry of [open-decisions.md](open-decisions.md). Its status — settled, open, or a conflict owed to the functional owner — is recorded there and nowhere else |

---

## 1. `README.md` — the index

| Technical concern | Drawn from | Doc |
|---|---|---|
| The technical specification is a **fourth** document set beside `/specification`, `rmp` and the knowledge graph; its boundary against each must be stated | *Still out of scope* — "The Rust implementation: its crates, its module layout, its types, and its library API"; CLAUDE.md *Fontes de Verdade* | `overview`, `README` (`OD-26`) |
| An **ADR register must exist**: two requirements cite it as the home of a fact this corpus refuses to hold | *Still out of scope* — "The pinned version of the template engine"; `FR-ENV-003`, `FR-CONF-038` | [`docs/adr/`](../adr/README.md) (`ADR-001`, `ADR-002`) |
| The library API carries **no compatibility guarantee**; only the JSON document and the command line are contract. The public surface is therefore an internal design choice, not a published one | *Still out of scope*; `DIV-032` | `overview`, `interfaces` |
| Requirement ids are the citation unit in commit messages, task descriptions and **test names** — the test suite's naming scheme is bound to them | *Identifier scheme* | `verification` |
| A requirement resting on the fourth provenance (direct observation) is falsifiable by a later observation: the technical spec must not re-derive catalogue facts, only cite them | *Provenance*, item 4 | `overview`, `data-model` |
| `FR-SRV-019` is a **release gate**: the supported-series table is re-verified before every release | *Maintenance debt*; `FR-SRV-019` | `operations` |

---

## 2. `cli-contract.md` — invocation grammar and the closed tree

| Technical concern | Drawn from | Doc |
|---|---|---|
| The parser must reject prefix inference for both commands and long flags, and must not consult `PATH` for external subcommands | `FR-CLI-004`, `FR-CLI-005`, `FR-CLI-006` | `interfaces` |
| A repeated single-value flag is `64` **and must name both values** — a last-wins parser cannot do this; the parser must retain both occurrences | `FR-CLI-014` | `interfaces` (`OD-07`, `OD-08`) |
| A global flag is accepted in **any position** at every depth, while a local flag is rejected at any node that does not declare it | `FR-CLI-024`, `FR-CLI-019`, `FR-GLOB-002` | `interfaces` |
| A separate-token flag value beginning with `-` is `64` **with a corrected `--flag=value` hint** | `FR-CLI-018` | `interfaces` |
| No environment variable may determine behaviour or the project location; `${VAR}` expansion inside `.cfg` is the single exception, and nothing a shell can set may decide which project, entry or server is reached | `FR-CLI-021`, `FR-CLI-023`, `BR-CLI-002` | `security`, `architecture` |
| Group nodes have no action and print their own help at exit `0`: the dispatcher needs a node kind, not a fallthrough | `FR-CLI-007`, `FR-CLI-008`, `FR-CLI-009` | `architecture`, `interfaces` |
| Never interactive: no prompt, no pager, no stdin read except `--context -` | `BR-CLI-003` | `architecture`, `security` |
| Byte-identical **stdout** for one invocation against one state; stderr is explicitly outside the contract | `NFR-DET-001` | `quality-attributes`, `verification` |
| Every collection is ordered explicitly, byte-wise by name, with six named exceptions — three of which the default rule would corrupt. The ordering must be applied by the emitter, not inherited from the server or the filesystem | `NFR-DET-002` | `interfaces`, `verification` |
| No `isatty()`, no terminal detection, no colour, no ANSI byte on either stream: clap's default `color` feature must be off | `NFR-DET-003`, `NFR-DET-004` | `technology-stack`, `operations` |
| `now` is the single documented source of non-reproducibility | `NFR-DET-005`, `FR-CTX-028` | `architecture` (`OD-25`) |

---

## 3. `global-flags.md` — the seven flags and precedence

| Technical concern | Drawn from | Doc |
|---|---|---|
| Two configuration layers above the built-in default, and **no environment layer**; the resolver has exactly three inputs | `FR-GLOB-001`, `FR-CONF-029`, `FR-CONF-030` | `architecture`, `interfaces` |
| An entry selected on the command line must be **distinguishable** from one resolved through `core.database`: the resolved value carries its provenance | `FR-GLOB-008`, `FR-RND-018`, `FR-RND-019` | `interfaces` |
| `--timeout` is an overall budget from **process start** that composes with per-phase deadlines; the first of the two to expire ends the phase | `FR-GLOB-011`, `FR-GLOB-012` | `architecture` (`OD-12`) |
| A timeout exits with the code of the **phase**, so the deadline machinery must know which phase it is in | `FR-GLOB-013`, `FR-ERR-027` | `architecture`, `interfaces` (`OD-12`) |
| Four diagnostic levels, one line per catalogue query at `INFO`, **distinguishable from every other line**, and cache hit/miss at `DEBUG` | `FR-GLOB-014` … `FR-GLOB-017`, `NFR-PERF-008` | `operations` (`OD-17`) |
| Six categories of content must never reach any diagnostic stream at any level: argv, resolved DSN, `password_command` and its stderr, raw driver error, `.cfg` contents. A raw driver error is what a naive `tracing` call on an `Err` emits | `FR-GLOB-018`, `FR-SEC-005` | `security`, `operations` (`OD-17`) |
| Five short forms exist in the whole tool and no local flag may declare one | `FR-GLOB-024` | `interfaces` |

---

## 4. `help-and-version.md` — help as contract

| Technical concern | Drawn from | Doc |
|---|---|---|
| Six forms, with **byte-identical** output between `tpl help <path>` and `tpl <path> --help` at every depth — two code paths held identical by snapshot test | `FR-HELP-001`, `FR-HELP-002`, `BR-HELP-001` | `interfaces`, `verification` (`OD-07`) |
| Seven fixed sections in a fixed order, including `EXAMPLES` and `EXIT CODES`, which no argument parser generates | `FR-HELP-006`, `FR-HELP-007`, `FR-HELP-008` | `interfaces` (`OD-07`) |
| Fixed 80-column layout with line breaks **written into the text**; `COLUMNS` never read, no reflow, identical across the four targets | `FR-HELP-009`, `FR-HELP-010` | `interfaces`, `technology-stack` (`OD-07`) |
| The JSON command tree is derived **at runtime by introspecting the parser**, so the parser's tree must be reflectable | `FR-HELP-021` | `interfaces` (`OD-07`) |
| `examples` and `exit_codes` come from a **typed table indexed by command path** feeding both text and JSON; never parsed out of help text | `FR-HELP-022` | `interfaces`, `data-model` |
| Declaration order preserved throughout the document, and no unordered map on the emitting path | `FR-HELP-023`, `FR-OUT-013` | `interfaces` (`OD-18`) |
| A nested command path resolves aliases and refuses prefixes; a bad segment is `64` with suggestions **over that node's children only** | `FR-HELP-026` … `FR-HELP-029` | `interfaces` |
| Three tests are part of the contract: every node/alias/flag present, every command has an example, **every example parses through the parser itself** | `BR-HELP-003` | `verification` |
| The version string is exactly `tpl <version>\n` and also appears as `tpl.version` in the context | `FR-HELP-005`, `FR-CTX-027` | `operations` (`OD-03`) |

---

## 5. `schema-commands.md` — the first arm

| Technical concern | Drawn from | Doc |
|---|---|---|
| Eight subcommands over one reader; each reads **through the cache** | `FR-SCH-002`, `FR-SCH-025`, `FR-CACHE-006` | `architecture` |
| A routine is named by `procedure:<name>` / `function:<name>` as well as bare, in four places; a bare ambiguous name is `64` naming both candidates | `FR-SCH-008`, `FR-SCH-010` | `interfaces`, `data-model` |
| `--pattern` is a **local** MariaDB `LIKE` matcher, never sent to the server, ASCII case-folded only | `FR-SCH-012` … `FR-SCH-014` | `interfaces` (`OD-20`) |
| Coverage is applied before the pattern, and to the **column read** as well as the object read | `FR-CAT-028`, `FR-CAT-052` | `architecture`, `interfaces` |
| The dump is one JSON document, complete by construction: `--pattern` on it is `64`, and it must round-trip through `--context` byte-identically against a live read | `FR-SCH-016`, `FR-SCH-021`, `FR-SCH-022`, `BR-SCH-004` | `verification`, `interfaces` (`OD-19`) |
| A `--context` document must carry the whole envelope; a bare `data` object is refused | `FR-SCH-036` | `interfaces` |
| `text` listings are aligned columns under a header row and are **not** contract; only `--format json` is | `FR-SCH-026`, `FR-SCH-027`, `FR-OUT-004` | `interfaces` |
| A read command **writes** to `.tpl/.cache/` on a miss; only `--direct --no-cache` touches no file | `BR-SCH-003`, `FR-CACHE-016` | `architecture`, `operations` |

---

## 6. `template-commands.md` — the second arm

| Technical concern | Drawn from | Doc |
|---|---|---|
| No `template` subcommand opens a connection, reads the cache, or requires an entry | `FR-TMPL-003`, `FR-CACHE-011`, `NFR-PERF-006` | `architecture` |
| A template is a regular file under the root ending `.jinja`; everything else is invisible to listing, render and include | `FR-TMPL-004`, `FR-TMPL-005` | `interfaces` |
| The extension is optional **on the command line only**; inside a template a name is literal, so the loader adds no resolution layer | `FR-TMPL-007`, `FR-TMPL-008`, `BR-TMPL-003` | `architecture` (`OD-15`) |
| Listing is ordered by the **displayed name** (extension removed), byte-wise, independent of directory iteration order | `FR-TMPL-013` | `interfaces` |
| Containment: symlinks refused, every resolved path canonicalised and re-checked against the root, escape is `65` | `FR-TMPL-024` … `FR-TMPL-026`, `FR-SEC-017` | `security` (`OD-15`) |
| `template check` is **syntax analysis only** — no expression evaluated, no filter called, no connection — so parse must be separable from render | `FR-TMPL-017`, `BR-TMPL-001`, `FR-SEC-018` | `architecture`, `interfaces` |
| `template show` is byte-for-byte, exempt from the escaping of `FR-OUT-018` | `FR-TMPL-015`, `FR-OUT-019` | `interfaces` |
| `template path` prints an **absolute** path | `FR-TMPL-021`, `FR-TMPL-022` | `interfaces` |

---

## 7. `render-command.md` — the third arm

| Technical concern | Drawn from | Doc |
|---|---|---|
| One invocation, one render, one result on stdout; no `--output` surface at all | `FR-RND-002`, `FR-RND-028` | `architecture`, `overview` |
| The context has five top-level variables from four sources; three are always injected and a supplied value for them is ignored | `FR-RND-023`, `FR-RND-024` | `interfaces`, `data-model` |
| `--set` splits on the **first** `=`, keys match `[A-Za-z_][A-Za-z0-9_]*`, values are always strings, duplicates are `64` | `FR-RND-009` … `FR-RND-015` | `interfaces` |
| `--context -` reads stdin; the document is untrusted input validated against the structural rules before use | `FR-RND-017`, `FR-RND-020`, `FR-CTX-033` | `security`, `interfaces` |
| With `--context`, no connection is opened and the cache is neither read nor written | `FR-RND-022`, `NFR-PERF-006` | `architecture` |
| A render failure carries template, line, column and the **chain of underlying engine errors**; stdout carries at most one incomplete result | `FR-RND-030`, `FR-RND-031`, `FR-RND-034` | `interfaces` (`OD-06`) |
| The render deadline is a hard requirement on a synchronous engine call | `FR-RND-033`, `FR-CONF-005` | `architecture` (`OD-12`) |
| Two findings must survive if file writing ever returns: filename expressions are a path-injection sink, and two objects can collapse onto one filename after a casing filter | `BR-RND-003` | `decisions` |

---

## 8. `cache-commands.md` — the catalogue cache

| Technical concern | Drawn from | Doc |
|---|---|---|
| Location `.tpl/.cache/<entry>/`, keyed by entry name **and nothing else**; created on first populating read, never by `init` | `FR-CACHE-001` … `FR-CACHE-003`, `FR-PROJ-020` | `data-model` |
| Read-through: hit serves from disk and opens no connection; miss reads, **writes, then answers** | `FR-CACHE-006`, `FR-CACHE-007`, `NFR-PERF-003` | `architecture` |
| No TTL, no automatic expiry, no invalidation on repointing an entry | `FR-CACHE-008`, `FR-CACHE-028`, `FR-CACHE-029` | `data-model` |
| `--direct` and `--no-cache` are orthogonal and compose into four behaviours; `--no-cache` on `cache load` is `64` | `FR-CACHE-015`, `FR-CACHE-019` | `interfaces` |
| One file per object, written through a temporary file in the same directory and renamed over the target; **no lock** | `FR-CACHE-030`, `FR-CACHE-031` | `data-model`, `architecture` |
| An unreadable file or unknown version is a **silent miss**; a failed write is a **silent success** at exit `0` with stdout unchanged | `FR-CACHE-033`, `FR-CACHE-036` | `architecture`, `interfaces` |
| An object marked `restricted` is never written, and a collection containing one is never recorded whole | `FR-CACHE-037` | `data-model` |
| `cache status` is the supported way to learn the cache's state; the on-disk layout is **not** plumbing contract | `BR-CACHE-001`, `FR-CACHE-034` | `data-model` (`OD-10`) |

---

## 9. `cache-documents.md` — what the cache stores about itself

| Technical concern | Drawn from | Doc |
|---|---|---|
| `meta.json` carries **two independent versions**: `cache_format` for the arrangement, `schema_version` for the content; neither is bumped for the other | `FR-CDOC-001` … `FR-CDOC-005` | `data-model` (`OD-03`) |
| Either version unknown to the binary is a miss: the reader must parse `meta.json` before trusting anything under it | `FR-CDOC-004` | `data-model` |
| Per-collection completeness decides whether a **listing** may be served; an individual object is served whenever present | `FR-CDOC-006` … `FR-CDOC-008` | `data-model` |
| `source` is an enumerated string on the envelope, never a boolean, never on an object | `FR-CDOC-009`, `FR-CDOC-010`, `FR-OUT-029` | `interfaces` |
| `loaded_at` appears in `meta.json` and in `cache status` and **nowhere else** | `FR-CDOC-012`, `FR-CDOC-013` | `data-model` |
| A cached routine's path carries its kind: `routines/<kind>.<name>.json` — the only on-disk name the functional spec fixes, and it implies a JSON encoding | `FR-CDOC-014` | `data-model` (`OD-10`) |
| A cache-served document promises neither referential integrity nor a snapshot; the assembler must not claim either | `FR-CDOC-015`, `FR-CDOC-016`, `BR-CDOC-004` | `data-model`, `interfaces` |
| The on-disk **encoding** is explicitly out of scope of the functional spec — it is the technical spec's to fix | *Scope*, "Out of scope: … the on-disk encoding" | `data-model` (`OD-10`) |

---

## 10. `cfg-commands.md` — configuration commands

| Technical concern | Drawn from | Doc |
|---|---|---|
| No `cfg` subcommand writes anywhere but `.tpl/.cfg`, and only `database test` contacts a server | `FR-CFG-004`, `FR-CFG-005` | `architecture` |
| `cfg get` is unredacted and unexpanded; `cfg list` and `database show` redact and leave `${VAR}` verbatim. Three distinct read paths over one file | `FR-CFG-006`, `FR-CFG-014`, `FR-CFG-021` | `security`, `interfaces` |
| `cfg set` validates against the **enumerated key space and the declared type** of each key | `FR-CFG-009`, `FR-CFG-010`, `FR-CONF-002` | `interfaces`, `data-model` |
| `cfg unset` accepts a leaf key **or a whole block** | `FR-CFG-011` | `interfaces` |
| `database update` changes only the named fields and leaves the rest of the entry untouched; `add` and `update` never do each other's job | `FR-CFG-020`, `BR-CFG-001` | `data-model` (`OD-09`) |
| Removing the entry named by `core.database` also clears that key, silently | `FR-CFG-023` | `interfaces` |
| A rewrite writes a temporary file in `.tpl/` at mode `0600` and renames it over the target; **no lock**; the mode is retained | `FR-CFG-034`, `FR-CFG-041`, `FR-CFG-042` | `data-model`, `security` |
| `--password-command` takes a single string and stores the split array; not repeatable | `FR-CFG-046`, `FR-CONF-025` | `interfaces` (`OD-20`) |
| `database test` performs **four ordered steps** and reports one field per step, with `server` the object of `FR-CTX-031` | `FR-CFG-024`, `FR-CFG-039` | `interfaces` |
| The privilege probe is **exactly one** `SELECT` against `INFORMATION_SCHEMA`, restricted to the named database, read for two facts only | `FR-CFG-044` | `interfaces`, `quality-attributes` |
| `can_read_catalogue` false does **not** change the exit code | `FR-CFG-045` | `interfaces` |
| `cfg list` is the first of exactly **two** exceptions to "absent is `null`": an absent key is absent from the document | `FR-CFG-037` | `interfaces` (`OD-18`) |

---

## 11. `configuration-model.md` — the `.cfg` file

| Technical concern | Drawn from | Doc |
|---|---|---|
| TOML, one file, no global configuration, no home/XDG/`/etc` fallback | `FR-CONF-001`, `FR-CONF-003` | `data-model` |
| Fifteen keys with declared types and defaults; the key space is the validator's population and `cfg list`'s shape | `FR-CONF-002` | `data-model` |
| **Strict in both directions**: an unrecognised key anywhere is `78` with a suggestion; `password_command` not an array is `78` with the array form in the hint | `FR-CONF-034`, `FR-CONF-035`, `BR-CONF-004` | `data-model`, `security` |
| An error must name the **line** of `.cfg` that carries the fault | `FR-CONF-035` example; `FR-ERR-034` row `78` | `data-model` (`OD-09`) |
| Five admitted/refused combinations of connection and password keys, decided before any connection | `FR-CONF-007` | `interfaces` |
| A DSN carries **no** query parameters; a `?` is `78` whatever follows it | `FR-CONF-011`, `FR-CONF-012` | `interfaces`, `security` |
| Five TLS modes, set **explicitly on every connection**, never inherited from the driver's default — including `disabled` | `FR-CONF-013`, `FR-CONF-037` | `security` (`OD-16`) |
| The five modes' observed behaviour against a TLS-offering and a TLS-less server is fixed; the **mapping onto the driver belongs in an ADR** and is cited, not restated | `FR-CONF-038` | `security`; [`ADR-002`](../adr/adr-002-tls-mode-mapping.md) |
| The **fixture is under obligation**: at each supported series it presents a server whose certificate names the host the tests reach it by, and retains a server offering no TLS. Without the first, the default mode of `FR-CONF-013` has no acceptance test; how either is provisioned is left to this folder | `FR-CONF-038` as amended in the eighth edition; `FR-SRV-015`, `FR-SRV-029` | `operations`, `verification` (the residual of `OD-22`) |
| Trust material is **additional** to the platform/bundled roots; exclusive trust is not deliverable and must not be claimed | `FR-CONF-039` | `security` (`OD-16`) |
| `${VAR}` expands in six fields only; inside a DSN the URL is **parsed first**, expanded within the delimited field, then percent-encoded | `FR-CONF-015`, `FR-CONF-018` | `security`, `interfaces` |
| Single-pass expansion, `$$` literal, unclosed brace `78`, undefined variable `78` | `FR-CONF-019` … `FR-CONF-022` | `interfaces` |
| `password_command` runs **without a shell** from an argument array; metacharacters are literal | `FR-CONF-024`, `FR-CONF-026` | `security` |
| The child is bounded and silent: 4096-byte stdout cap then terminate, stderr to the null device, non-zero exit is `78` naming the command and status | `FR-CONF-031` … `FR-CONF-033` | `security`, `architecture` |
| A deadline on every blocking phase: DNS, TCP connect, TLS handshake, catalogue query, `password_command`, render — six separately named phases | `FR-CONF-005` | `architecture` (`OD-12`) |
| The `musl` static linkage changes DNS behaviour, and the `cause` line owes the caller the distinction | `FR-CONF-005` note; `NFR-PERF-018` | `operations`, `interfaces` |

---

## 12. `project-and-discovery.md` — the project folder

| Technical concern | Drawn from | Doc |
|---|---|---|
| A project is any directory containing `.tpl`; discovery walks **up** and stops at the first one | `FR-PROJ-001`, `FR-PROJ-004` | `architecture` |
| The walk stops at the **filesystem mount point alone**, determined without reading any environment variable — so a directory is compared with its parent and no home directory is located | `FR-PROJ-005`, `FR-SEC-013` | `security`, `architecture` |
| Exactly four commands skip discovery entirely and must reach that decision **before** any filesystem access | `FR-PROJ-025`, `NFR-PERF-005` | `architecture`, `quality-attributes` |
| The `.tpl` path is canonicalised **before** any check, so a symlinked `.tpl` is verified at its target | `FR-PROJ-009`, `FR-SEC-015` | `security` |
| `.cfg` must be owned by the current user and carry no group or other bits: the current process's uid and the file's uid and mode are all needed, and `std` supplies only the file's | `FR-PROJ-010`, `FR-PROJ-011` | `security`, `technology-stack` (the residual of `OD-24`) |
| `tpl init` creates five artefacts, `.cfg` at `0600`, missing parents included, and refuses an existing `.tpl` with `73` changing nothing | `FR-PROJ-013` … `FR-PROJ-019` | `interfaces`, `operations` |
| Two artefacts are **shipped content**: `example.jinja` and `rust/_types.jinja`; the example must render against any table of any supported series and use at least one filter and one test | `FR-PROJ-017`, `FR-PROJ-021`, `FR-ENV-011` | `operations`, `verification` |
| The generated `.cfg` carries a **commented-out** example entry — the file must survive later rewrites with its comments | `FR-PROJ-017`, `FR-PROJ-018` | `data-model` (`OD-09`) |
| Exactly four writers inside `.tpl`, and exactly one file-system exception outside it | `FR-PROJ-023`, `FR-PROJ-024` | `architecture`, `security` |
| A nested project warns on stderr and exits `0` | `FR-PROJ-016` | `interfaces` |

---

## 13. `output-formats.md` — `text`, `json`, encoding, streams

| Technical concern | Drawn from | Doc |
|---|---|---|
| One envelope of three keys in a fixed order for **all seventeen** documents; nothing beside `data`, no fourth key | `FR-OUT-024`, `FR-OUT-028`, `FR-OUT-032` | `interfaces` |
| `source` takes exactly four values and is present on every document | `FR-OUT-026` | `interfaces` |
| `data` is one key named for the collection (plural) or for the kind (singular) | `FR-OUT-030`, `FR-OUT-031` | `interfaces` |
| Compact by default: one line, no superfluous whitespace, one terminating newline; `--pretty` is a **two-space** indent, one key per line | `FR-OUT-007`, `FR-OUT-008` | `interfaces` (`OD-18`) |
| Absent means `null`, never omitted — with exactly two stated exceptions (`FR-CFG-037`, `FR-PRIV-016`) | `FR-OUT-012` | `interfaces` (`OD-18`) |
| Fixed key order per structure and **no unordered map** on the emitting path | `FR-OUT-013` | `interfaces` (`OD-18`) |
| A five-row compatibility rule governs document evolution: adding a field or an enumerated value is not breaking | `FR-OUT-014` | `data-model`, `operations` (`OD-03`) |
| An error is never formatted: `--format` is ignored, stdout stays empty, the four lines go to stderr | `FR-OUT-015`, `FR-ERR-033` | `interfaces` |
| Invalid UTF-8 from the catalogue becomes U+FFFD and the read continues — so catalogue values must be read as **bytes** and converted lossily, not as validated strings | `FR-OUT-017` | `architecture`, `interfaces` |
| C0 escaping in read output (tab excepted) applies to **every interpolated value whatever its source**; `render` and `template show` are exempt and byte-for-byte | `FR-OUT-018`, `FR-OUT-019` | `security`, `interfaces` |
| An empty result is `0`: the header row alone in `text`, the collection key with `[]` in `json` | `FR-OUT-033` … `FR-OUT-037` | `interfaces` |
| Results to stdout, everything else to stderr, nothing else on stdout for a piped command | `FR-OUT-020`, `FR-OUT-021` | `architecture` |

---

## 14. `errors-and-exit-codes.md` — the machine channel

| Technical concern | Drawn from | Doc |
|---|---|---|
| Ten codes and no others; each distinct condition has its own code and is not collapsed | `FR-ERR-001`, `FR-ERR-002` | `interfaces` (`OD-06`) |
| A **fixed eight-step validation order** decides which code wins, with steps 2 and 3 skipped for four commands. This is the shape of the invocation pipeline | `FR-ERR-006`, `FR-ERR-007` | `architecture` |
| Four labelled lines on stderr: `error:`, `cause:`, `hint:`, `exit:` | `FR-ERR-008` | `interfaces` |
| The `cause` line has a **per-code obligation table** naming what it must contain; the error value must therefore carry the instance, not the category | `FR-ERR-034`, `FR-ERR-010`, `FR-ERR-012` | `interfaces` (`OD-06`) |
| An internal taxonomy with no external carrier was **explicitly rejected**, while an exit code still has to be derived from the error value | `FR-ERR-015` *Rejected*; `BR-ERR-002` | `interfaces` (`OD-06`) |
| `70` has exactly two producing conditions — a caught top-level panic and a detected invariant violation — and the first cannot exist under an aborting release profile, which `DIV-045` records as a contradiction owed to the root coordination document | `FR-ERR-030`, `FR-ERR-032`, `DIV-045` | `architecture`, `operations`, `technology-stack` (`OD-28`) |
| The deliberate `70` trigger is reachable **only from within the system's own test configuration** and from no invocation of the distributed binary, and appears in neither command tree nor any help text; `BR-ERR-001` excepts `70` from its integration test in consequence | `FR-ERR-031`, `BR-ERR-001` | `verification` (the residual of `OD-21`) |
| Nearest match: at most three candidates within edit distance two, ordered by distance then name, over eight populations | `FR-ERR-019` … `FR-ERR-021` | `interfaces`, `quality-attributes` (`OD-20`) |
| A runnable hint is built only from literals and `[A-Za-z0-9_]{1,64}`; a candidate outside that set is not presented **at all** | `FR-ERR-022`, `FR-ERR-023` | `security` |
| Every interpolated value in a message escapes `\n`, `\r`, `\t` and C0 — a different rule from read output | `FR-ERR-024` | `security`, `interfaces` |
| `EPIPE` is `0` normally and `74` if a JSON document was mid-flight: the writer must know whether it is inside a document | `FR-ERR-025`, `FR-ERR-026` | `architecture`, `interfaces` |
| Nine codes carry at least one integration test, part of the definition of done; `70` is the single exception and is exercised in process | `BR-ERR-001` | `verification` |

---

## 15. `security.md` — cross-cutting rules

| Technical concern | Drawn from | Doc |
|---|---|---|
| Six untrusted inputs are enumerated: argv, `.cfg`, the environment, catalogue values, a `--context` document, files under `templates/`. These are the trust boundaries the technical spec must draw | *Trust boundaries* | `security` |
| Two documented paths put a literal password in argv; `tpl` **warns and does not prevent** | `FR-SEC-002`, `BR-CFG-003` | `security` |
| Prohibitions must be **structural where possible**: the child's stderr goes to the null device precisely so that no rule has to be remembered | `FR-SEC-024`, `FR-CONF-032` | `security` |
| A known **sentinel password must never appear in any byte** `tpl` writes: every command of the tree, at maximum verbosity, on both streams. This is a whole-surface test, not a per-path one | `BR-SEC-003` | `verification`, `security` |
| The read-only promise has two parts, and only the closed statement list prevents; the session setting detects | `BR-SEC-002`, `BR-SRV-001`, `BR-SRV-002` | `security`, `architecture` |
| Every blocking phase has a deadline, because a hung process is the failure "never interactive" exists to prevent | `FR-SEC-022` | `architecture` (`OD-12`) |
| This file is a cross-reference, not a second source: the technical `security.md` must likewise cite its owning requirement and not restate it | `BR-SEC-001` | `security` |

---

## 16. `use-cases.md` — end-to-end flows

| Technical concern | Drawn from | Doc |
|---|---|---|
| Twelve flows are the acceptance-test skeleton and the source of every `EXAMPLES` section | `UC-001` … `UC-012`; *Overview* | `verification` |
| The canonical loop is **200 process startups**, not 200 iterations: startup cost is paid per object | `UC-008`, `BR-RND-002`, `BR-PERF-005` | `quality-attributes` |
| The offline flow is dump → commit → render, so the document must be stable enough to commit and diff | `UC-009` | `quality-attributes`, `data-model` |
| The agent's self-correction loop is exit code → `cause`/`hint` → retry: the diagnostic is a functional path, not decoration | `UC-012` | `interfaces` |
| `UC-004` requires all four `database test` steps to be separately reportable and separately failing | `UC-004` | `interfaces` |

---

## 17. `catalogue-coverage.md` — what enters the model

| Technical concern | Drawn from | Doc |
|---|---|---|
| Coverage is a **predicate over six table-type strings**, applied to the object read and to the column read; temporary tables must be filtered, never assumed absent | `FR-CAT-001` … `FR-CAT-006`, `FR-CAT-031`, `FR-CAT-032`, `FR-CAT-052` | `data-model`, `interfaces` |
| An index is **folded** from one row per column into one object with an ordered column list, on the sequence-in-index field | `FR-CAT-010`, `FR-CAT-042` | `interfaces`, `data-model` |
| The primary key is read from the **index table only**, and never from key-column-usage or the constraint table | `FR-CAT-043` | `interfaces`, `data-model` |
| No key may name a column absent from the same table's column list — an invariant the emitter must guarantee, since the catalogue does not | `FR-CAT-044` | `interfaces`, `verification` |
| A foreign key requires **joining two catalogue tables**; the incoming direction is the same key-column table filtered from the other end | `FR-CAT-045`, `FR-CAT-013` | `architecture`, `interfaces` |
| Carry by default, exclude on one of four stated grounds; "nothing observed" is not a fifth ground for admitting a field later without an observation | `BR-CAT-005` | `data-model` |
| Two **closed** exclusion lists with different grounds, tests and futures: sixteen volatile fields, and an ambiguous-meaning list that is currently empty | `FR-CAT-024`, `FR-CAT-025`, `FR-CAT-029`, `FR-CAT-030`, `BR-CAT-004` | `data-model` |
| Field-list requirements fix, per object kind, exactly which catalogue fields are read and which properties the model carries — the reader's column lists are specified material | `FR-CAT-039`, `FR-CAT-041` … `FR-CAT-051` | `data-model`, `interfaces` |
| Observed catalogue behaviour the reader must encode rather than rediscover: `SET DEFAULT` is unrepresentable; a `SET` member cannot contain a comma but an `ENUM` member can; a `JSON` column reads `longtext` and carries an implicit `json_valid` check; a view's table comment is the literal `VIEW`; a comment is `utf8mb3` and lossy in the server | `FR-CAT-033` … `FR-CAT-038`, `FR-CAT-040` | `data-model` |
| Generated-column storage kind comes from the **attribute** field, not the is-generated field | `FR-CAT-051` | `interfaces` |
| Five whole features are excluded by decision: events, sequences, partitions, application-time periods, spatial reference identifiers | `FR-CAT-019` … `FR-CAT-023` | `overview`, `data-model` |
| Identifiers arrive unquoted and unescaped, including hostile ones (backtick, space, reserved word, non-ASCII): quoting is the template's job | `FR-CAT-042` note; `FR-ENV-045` | `interfaces`, `security` |

---

## 18. `context-document.md` — the shape of the document

| Technical concern | Drawn from | Doc |
|---|---|---|
| Every collection is a JSON array; `[]` for empty; `null` reserved for an absent **scalar** | `FR-CTX-003` … `FR-CTX-005` | `interfaces` |
| **Both** foreign-key directions embed one level deep and are cut to names at the first hop; one rule, so no traversal can fail to terminate | `FR-CTX-006` … `FR-CTX-010` | `interfaces`, `architecture` (`OD-19`) |
| The double embedding roughly quadruples column volume and is the named pressure on the 32 MiB provisional budget | `FR-CTX-010` *Accepted cost*; `NFR-PERF-014` | `quality-attributes` (`OD-19`) |
| A column default is a **three-way discriminated structure or `null`**, classified by an ordered eight-row shape table over the raw catalogue value, first match wins | `FR-CTX-011` … `FR-CTX-013`, `FR-CTX-037` | `interfaces`, `data-model` |
| A doubled apostrophe collapses to one in a literal default and in an `ENUM` member; there is no backslash escape | `FR-CTX-037`, `FR-CTX-039` | `interfaces` |
| An `ENUM`/`SET` member list is parsed from the raw type string by **quote state**, never by splitting on the comma | `FR-CTX-039` | `interfaces`, `verification` |
| A type decomposes into eight parts read from named catalogue fields; the two precision fields merge losslessly; `unsigned` is derivable **only** from the raw type string | `FR-CTX-015`, `FR-CTX-038`, `FR-CTX-040` | `interfaces`, `data-model` |
| An unrecognised type keeps `column_type` and nulls every part — the raw string is the safety net | `FR-CTX-018` | `interfaces` |
| A part is `null` **exactly when** the catalogue field is SQL `NULL`; applicability is never decided by `tpl` | `FR-CTX-017`, `FR-CTX-041` | `interfaces` |
| Nothing the table already states is materialised on a column; `primary_key` and `unique` are computed by resolving `table_name` against the context | `FR-CTX-021`, `FR-CTX-022` | `interfaces`, `architecture` |
| The `database` object carries three metadata fields, a three-key `server` object, and three collections — `server` derived from the version probe, not the catalogue | `FR-CTX-031`, `FR-CTX-035`, `FR-CTX-036` | `data-model` |
| `standing` is enumerated and **unconditionally present**; a conditional marker would fail the guard that looks for it | `FR-CTX-034`, `BR-SRV-008`, `FR-SEM-012` | `interfaces` |
| A `--context` document is validated **structurally only**: three keys present, strings, `standing` in range; never against the current window | `FR-CTX-033` | `security`, `interfaces` |
| `now` is one RFC 3339 UTC second-precision string, evaluated **once per invocation** | `FR-CTX-028`, `FR-CTX-029` | `architecture` (`OD-25`) |
| A server read promises referential integrity and **not** a point-in-time snapshot; a cache-served document promises neither | `FR-CTX-023` … `FR-CTX-025` | `data-model` |

---

## 19. `template-environment.md` — the template surface

| Technical concern | Drawn from | Doc |
|---|---|---|
| Three contract groups, with group 2 guaranteed against a **pinned engine minor version** recorded in an ADR and cited from there | `FR-ENV-001` … `FR-ENV-003` | `technology-stack`; [`ADR-001`](../adr/adr-001-template-engine-pin.md) |
| All three groups are published through `tpl help --format json` | `FR-ENV-005` | `interfaces` |
| Eleven names are registered by `tpl`: five naming filters, six code filters | `FR-ENV-006`, `FR-ENV-007` | `interfaces` |
| Two registered names — `indent` and `escape` — **collide with engine built-ins of different signatures**, so the registration must deliberately shadow them | `FR-ENV-007`, `FR-ENV-037`, `FR-ENV-044`; minijinja built-ins (verified) | `technology-stack`, `interfaces` (`OD-13`) |
| The word-list tokeniser is a five-rule algorithm with a published eight-row test vector; all five naming filters are pure functions of it | `FR-ENV-030` … `FR-ENV-033` | `interfaces`, `verification` (`OD-20`) |
| Case folding is ASCII-only everywhere, independent of server, collation and locale | `FR-ENV-031`, `FR-SCH-014` | `interfaces` |
| `quote` doubles every backtick and wraps in one pair — testable without a server precisely because the mechanism is named | `FR-ENV-035`, `FR-ENV-045` | `interfaces`, `verification` |
| `json` emits the compact form with **no trailing newline** and accepts any context type | `FR-ENV-036` | `interfaces` |
| `indent(n)`, `comment(prefix)` and `escape(target)` all take a **required** argument with no default; a missing or bad argument is `65` | `FR-ENV-037`, `FR-ENV-038`, `FR-ENV-044` | `interfaces` |
| `sql_type` is a read of `data_type` and nothing else — no parsing, no consulting `column_type` | `FR-ENV-039` | `interfaces` |
| Seven tests accept a **column object and nothing else**; a wrong operand fails, never answers `false` | `FR-ENV-040`, `FR-SEM-005`, `FR-SEM-007` | `interfaces` |
| `primary_key` and `unique` resolve `table_name` **against the render context**, so a filter/test needs access to the context, and an absent table is `65` | `FR-ENV-015`, `FR-ENV-017`, `FR-ENV-043` | `architecture`, `interfaces` |
| Three disjoint type families over an exact 39-value partition; a value in none satisfies none and does not fail | `FR-ENV-042`, `FR-ENV-046` | `interfaces`, `data-model` |
| Fourteen inherited filters, closed; everything else the engine offers is group 3 and unguaranteed | `FR-ENV-018`, `FR-ENV-019` | `technology-stack`, `interfaces` |
| Five global functions and no more; four capability prohibitions — no environment, no file, no network, no clock | `FR-ENV-020` … `FR-ENV-025`, `BR-ENV-004` | `security`, `architecture` |
| Auto-escaping is **off always**, and must not be keyed on any property of a name — the engine's default callback does key on extension | `FR-ENV-026`, `FR-ENV-027`; minijinja `set_auto_escape_callback` (verified) | `architecture`, `security` (`OD-14`) |
| The behaviour tables are simultaneously specification and test vector; a changed cell is a breaking change | `BR-ENV-007`, `FR-ENV-029` | `verification`, `operations` |

---

## 20. `render-semantics.md` — what happens when a template is wrong

| Technical concern | Drawn from | Doc |
|---|---|---|
| Whitespace defaults are the engine's stock behaviour, **except** that a trailing newline in the source is preserved | `FR-SEM-001` … `FR-SEM-004`, `BR-SEM-001` | `architecture`, `interfaces` |
| No coercion anywhere: a wrong operand fails with `65` naming the filter or test, the type received, and the location | `FR-SEM-008`, `FR-SEM-009` | `interfaces` |
| An interpolated `null` renders as the **empty string**, and never as `none` or `null` | `FR-SEM-010`, `FR-SEM-011` | `interfaces` (`OD-14`) |
| Reading a field that does not exist **fails**, and this rule is load-bearing for three requirements elsewhere; relaxing it silently removes a guarantee | `FR-SEM-012`, `FR-SEM-013` | `architecture`, `verification` (`OD-14`) |
| `fail(message)` ends the render at `65` carrying the author's message with template, line and column | `FR-SEM-014`, `FR-SEM-015` | `interfaces` |
| There is **no** warn-and-continue path | `FR-SEM-016` | `interfaces` |
| The same strictness applies to a column arriving from `--context` as to one read from a server | `FR-SEM-018` | `verification` |
| Every failure here carries template, line and column, and leaves at most one incomplete result on stdout | `FR-SEM-019`, `FR-SEM-020` | `interfaces`, `architecture` |

---

## 21. `server-contract.md` — the server and the read-only promise

| Technical concern | Drawn from | Doc |
|---|---|---|
| The supported set is a **criterion**, not a list: two conditions, four series today, re-verified before every release | `FR-SRV-001`, `FR-SRV-015`, `FR-SRV-019`, `BR-SRV-004` | `operations`, `overview` |
| Series comparison is by `<major>.<minor>` only, derived from the version string; the build suffix derives nothing | `FR-SRV-021`, `FR-SRV-040` | `interfaces` |
| The product marker is a **necessary and not sufficient** condition; no impostor detection is claimed | `FR-SRV-041` | `security`, `overview` |
| The product and version are determined **on connecting**, before any other statement but the read-only pair; below the window is `78`, above it is a marked read | `FR-SRV-002`, `FR-SRV-003`, `FR-SRV-020`, `FR-SRV-031`, `FR-SRV-034` | `architecture`, `interfaces` |
| The treatment of every known difference is selected from the **resolved series**; discovery by attempt-and-handle-failure is forbidden | `FR-SRV-022`, `FR-SRV-023` | `architecture`, `interfaces` |
| A statement naming a fixed `INFORMATION_SCHEMA` column list must be common to all four series or selected per series — naming an absent column is a hard `ERROR 1054` | `FR-SRV-037`; differences 5, 6, 7 of `FR-SRV-038` | `interfaces`, `verification` |
| Four treatments of a difference: normalise, mark `null`, exclude, **pass through** — the fourth ranks fidelity above cross-server determinism for character sets and collations | `FR-SRV-024`, `FR-SRV-004`, `FR-SRV-025`, `FR-SRV-039`, `BR-SRV-006` | `data-model`, `quality-attributes` |
| Byte-identical documents across the four series from identical DDL, with three stated exceptions — the strongest testable form of "supported" | `FR-SRV-026`, `FR-SRV-029` | `verification`, `quality-attributes` |
| A **closed list of four statement kinds** and nothing else: no DDL, no DML, no `SHOW`, no non-`INFORMATION_SCHEMA` schema, no external process | `FR-SRV-006`, `FR-SRV-007` | `security`, `interfaces` |
| The session read-only setting is applied **and read back** at connection start, in that order, once each; failure of either refuses the connection with `78` | `FR-SRV-008` … `FR-SRV-010` | `architecture`, `security` |
| No flag, key or environment condition disables any of it | `FR-SRV-011` | `security` |
| Verification is from **outside the process**, on the server: the statements it receives, the connections it accepts | `FR-SRV-012` … `FR-SRV-014`, `BR-SRV-003` | `verification` |
| The newer-than-window path is verified through the in-process seam of `FR-ERR-031`, asserting that the read completes without error and that `standing` is `newer_than_supported`; `BR-SRV-003` states in its own text that it does not reach this requirement | `FR-SRV-035`, `BR-SRV-003` | `verification` (the residual of `OD-21`) |
| Eleven observed differences bound the reader's assumptions; one does **not** separate `10.11` from the rest, so tests must not model the window as one old server and three modern ones | `FR-SRV-038`, difference 8 | `verification` |

---

## 22. `privileges-and-completeness.md` — short reads

| Technical concern | Drawn from | Doc |
|---|---|---|
| A privilege-driven absence has **three shapes** — empty string, SQL `NULL`, zero rows — and a reader that looks for one finds none of the others | `FR-PRIV-018` | `interfaces`, `data-model` |
| Asymmetric outcome: a named object that is incomplete is `77`; a listing or dump marks the object and succeeds | `FR-PRIV-003` … `FR-PRIV-005`, `BR-PRIV-001` | `interfaces`, `architecture` |
| `restricted` is an ordered non-empty array of **model property names**, on the object, present only where there is something to report — the second of two exceptions to "absent is `null`" | `FR-PRIV-016`, `FR-OUT-012` | `interfaces` (`OD-18`) |
| Two cross-checks exist and only two: a view definition read as the empty string, and a key column naming a referenced table with no referential row. Both need **two reads of one population compared** | `FR-PRIV-011`, `FR-PRIV-015`, `FR-PRIV-019` | `interfaces`, `architecture` |
| A routine body read as SQL `NULL` is self-announcing and needs no cross-check | `FR-PRIV-017` | `interfaces` |
| Hidden triggers are **indistinguishable** from no triggers, and the limit must not be papered over by inferring grants | `FR-PRIV-020` | `overview`, `security` |
| Every cross-check runs on every read that presents the guarded property, dumps included | `FR-PRIV-012` | `architecture` |
| A marked document is refused as a `--context`, whatever the render selects | `FR-PRIV-008`, `FR-PRIV-009` | `interfaces` |
| Completeness is a property of a **read**, not of a server: nothing in the model records which reader produced it | `BR-PRIV-003` | `data-model` |

---

## 23. `performance-requirements.md` — properties and protocol

| Technical concern | Drawn from | Doc |
|---|---|---|
| Six **requirements of form** constrain the design from the first commit: query count independent of object count for a full read and for a single object, no connection on a cache hit, at most one connection, nothing at all for four commands, no connection for any command needing no catalogue | `NFR-PERF-001` … `NFR-PERF-006` | `quality-attributes`, `architecture` |
| Each is verified from **outside the process** — statements, connections, files opened — never by reading the source | `NFR-PERF-007`, `BR-SRV-003` | `verification` (the residual of `OD-22`) |
| The query count must be observable from the diagnostic stream, which makes one diagnostic line structurally load-bearing although stderr is not contract | `NFR-PERF-008`, `FR-GLOB-017` | `operations`, `verification` (`OD-17`) |
| Exactly four targets; Linux is `musl`, statically linked; no target is second class | `NFR-PERF-018` | `operations` |
| Nine budgets, one normative, five needing a server; a measurement names its target **and its server series** | `NFR-PERF-012`, `NFR-PERF-014` | `quality-attributes` |
| The measurement protocol is normative: 200-run median, 20 warmups, no shell, idle host, first run of a fresh binary discarded, RSD ≤ 5% | `NFR-PERF-009` … `NFR-PERF-011` | `quality-attributes` |
| Provisional figures are not limits; the ratification gate has four conditions and moves the figure to `BENCHMARKS.md` | `NFR-PERF-019`, `NFR-PERF-020`, `BR-PERF-006` | `quality-attributes` |
| Regression against a recorded baseline on the same target fails the change | `NFR-PERF-017` | `operations`, `quality-attributes` |
| Three reference workloads and a byte scalar; `WL-001` needs `seed-bench.sql`, which does not exist | `WL-001` … `WL-003`, `BR-PERF-002`, `BR-PERF-007` | `verification` (`OD-27`) |
| The failure path is a budget because nearest match computes an edit distance against every existing name | `BR-PERF-004` | `quality-attributes` (`OD-20`) |

---

## 24. `glossary.md` — terms

| Technical concern | Drawn from | Doc |
|---|---|---|
| The functional vocabulary is fixed and must be **reused, not paralleled**: arm, model, catalogue, context, context document, envelope, source, standing, restricted, coverage, target, budget, series. No second glossary is proposed, for this reason | whole file; *"A term used in a requirement without being defined here is a defect"* | `README` |
| `plumbing` / `porcelain` name the two output audiences; the technical spec's naming of modules and types should not invent a third vocabulary | *plumbing*, *porcelain* | `interfaces` |
| One term already carries two meanings by decision — `schema` (the arm, and the `--schema` flag) — and code naming must disambiguate rather than pick one | *schema (the word, two meanings)*; `FR-CFG-028` | `interfaces` |
| `target` is defined as one of four build targets: the word is reserved and must not be reused for a render target | *target*; `NFR-PERF-018` | `operations`, `quality-attributes` |
| The *DSN* entry was stale when this mapping was harvested and was corrected in the eighth edition; it now agrees with `FR-CONF-009` and `FR-CONF-011` and nothing is owed | *DSN*; `ED-01`, retired | — |

---

## 25. `open-questions.md` — the empty index

| Technical concern | Drawn from | Doc |
|---|---|---|
| **Nothing is open**: no technical decision may be deferred by citing an open functional question | *Index* — empty; all 75 closed | `overview` |
| Three obligations recur and are not questions: the series table re-verification, budget ratification, and the falsifiability of any observation-based requirement | *Overview*, "Complete means that no point of this specification is waiting on someone" | `operations`, `quality-attributes` |
| An unsettled point is a **defect to report**, never an old identifier revived | *Overview*, final paragraph | `README`, `decisions` |
| The closed table is the provenance trail for every fixed decision: a technical decision that contradicts one must cite the entry it reopens | *Closed* | `decisions` |

---

## 26. `upstream-divergences.md` — corrections owed to the root documents

| Technical concern | Drawn from | Doc |
|---|---|---|
| The library API and the `model/` structs are **not** a public surface: five questions the functional spec declines — owned versus borrowed types, public fields versus accessors, newtypes for names, whether the serialisation crate is a public dependency, `#[non_exhaustive]` — are named as **architecture decisions** | `DIV-032` | `interfaces`, `decisions` (`OD-05`) |
| The whole engine `Environment` surface is **not** contract; only the three groups are | `DIV-033` | `technology-stack` |
| The target matrix is fixed and Linux is `musl`; the `gnu` triples are not targets. The linkage has an observable DNS consequence that must not be presented as pure packaging | `DIV-041` | `operations` |
| The release profile's `panic = "abort"` removes one of the two producing conditions of `70` in the only artefact a caller runs. Either the profile leaves the panic path catchable, or `FR-ERR-030` is amended first through `specification-manager`; both statements may not stand together | `DIV-045` | `architecture`, `technology-stack`, `operations` (`OD-28`) |
| `CLAUDE.md`'s four performance figures are adopted as provisional and should exist in **no** third place | `DIV-035` | `quality-attributes` |
| `scripts/mariadb/` still owes `seed-bench.sql`, plus one line of the project tree and one of the testing section | `DIV-036` | `verification` (`OD-27`) |
| The catalogue is read through `INFORMATION_SCHEMA` only; the `SHOW` escape hatch `CLAUDE.md` allows does not exist | `DIV-031` | `security`, `interfaces` |
| Determinism is over **stdout** only | `DIV-039` | `quality-attributes` |
| Four writers inside `.tpl`, not three | `DIV-005` | `architecture` |
| `tpl cache` is a fourth auxiliary group that `CLAUDE.md` omits entirely | `DIV-040` | `architecture` |
| Every divergence is a correction owed to a file **the specification never edits** — the technical spec inherits that restraint and must not restate the corrected content either | *Overview* | `README`, `overview` |

---

## Reverse mapping

Every file of `specification/` is answered by at least one document of this
folder.

| Specification file | Answered principally in |
|---|---|
| `README.md` | `README`, `overview`, `decisions` |
| `cli-contract.md` | `architecture`, `interfaces`, `quality-attributes` |
| `global-flags.md` | `architecture`, `interfaces`, `operations` |
| `help-and-version.md` | `interfaces`, `verification` |
| `schema-commands.md` | `architecture`, `interfaces` |
| `template-commands.md` | `architecture`, `security`, `interfaces` |
| `render-command.md` | `architecture`, `interfaces` |
| `cache-commands.md` | `architecture`, `data-model` |
| `cache-documents.md` | `data-model` |
| `cfg-commands.md` | `interfaces`, `data-model`, `security` |
| `configuration-model.md` | `data-model`, `security`, `interfaces` |
| `project-and-discovery.md` | `architecture`, `security`, `operations` |
| `output-formats.md` | `interfaces` |
| `errors-and-exit-codes.md` | `interfaces`, `architecture` |
| `security.md` | `security` |
| `use-cases.md` | `verification` |
| `catalogue-coverage.md` | `data-model`, `interfaces` |
| `context-document.md` | `data-model`, `interfaces` |
| `template-environment.md` | `interfaces`, `technology-stack` |
| `render-semantics.md` | `interfaces`, `architecture` |
| `server-contract.md` | `architecture`, `security`, `verification`, `data-model` |
| `privileges-and-completeness.md` | `interfaces`, `architecture` |
| `performance-requirements.md` | `quality-attributes`, `operations` |
| `glossary.md` | `README` |
| `open-questions.md` | `overview`, `decisions` |
| `upstream-divergences.md` | `README`, `interfaces`, `operations` |

## Concerns that cross every file

Five concerns are forced by the corpus as a whole rather than by any one file,
and each is the reason a document of this folder exists.

| Concern | Forced by | Doc |
|---|---|---|
| **The invocation pipeline is specified.** `FR-ERR-006` fixes eight ordered steps, `FR-PROJ-025` exempts four commands from two of them, `NFR-PERF-005` makes the exemption observable from outside the process, and `FR-SRV-002` fixes what happens at connection start. The component decomposition has to realise that order | `FR-ERR-006`, `FR-PROJ-025`, `NFR-PERF-005`, `FR-SRV-002` | `architecture` |
| **One model, three sources.** A live read, a cached read and a `--context` document must present the same objects and fields, so the model is the single junction of three producers and three consumers — `text`, `json`, and the render | `catalogue-coverage.md` *Overview*; `BR-SCH-001`, `FR-SCH-022` | `architecture`, `interfaces` |
| **Silent wrongness is the failure mode the corpus is written against.** Every fail-loud rule — no coercion, strict field access, `77` for a named short read, refusal of a marked context, exclusion of volatile fields — is one design stance applied repeatedly | `BR-SEM-004`, `BR-PRIV-001`, `BR-CAT-002`, `FR-SRV-003` | `overview`, `architecture` |
| **The seventeen JSON documents share one envelope and one emitter.** Seventeen payload shapes, one outer shape, one ordering rule, one escaping rule, one encoding rule | `FR-OUT-024`, `BR-OUT-002` | `interfaces` |
| **Fourteen mandated tests are part of the contract, not of the plan.** `BR-ERR-001`, `BR-HELP-001`, `BR-HELP-003` (three), `BR-SCH-004`, `BR-SEC-003`, `FR-SRV-012`, `FR-SRV-013`, `FR-SRV-029` (two), `FR-SRV-035`, and the published test vectors of `FR-ENV-032`, `FR-ENV-033`, `FR-ENV-041` and `FR-ENV-046` | as cited | `verification` |

## Editorial defects found while harvesting

Both are closed. The two stale statements reported here — the *DSN* form in
`glossary.md` and the field count in `DIV-034` — were corrected in the eighth
edition of `/specification`, and neither is outstanding. What each said and what
each now says is recorded once, as `ED-01` and `ED-02` in
[open-decisions.md](open-decisions.md#editorial-defects-reported-and-corrected),
and is not repeated here.
