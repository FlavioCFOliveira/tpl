---
title: Output Formats
status: approved
last-reviewed: 2026-09-21
related: [cli-contract.md, schema-commands.md, errors-and-exit-codes.md, help-and-version.md]
---

# Output Formats

## Overview

`tpl` produces two kinds of output. `text` is laid out for a person to read and
is explicitly not a contract. `json` is the plumbing contract: versioned, stable,
and safe to parse. This file defines both, the `--pretty` rule, and how
catalogue bytes that are not valid text are handled on the way out.

Both formats describe a **result**. A failure has no result and is never
formatted: `FR-OUT-015` and `FR-ERR-033` keep every error in the four-line text
of `FR-ERR-008`, whatever `--format` was asked for.

## Scope

In scope: the `--format` and `--pretty` flags, the properties of `text` output,
the JSON contract and its compatibility rule, character encoding, and the
separation of stdout from stderr.

Out of scope: the content of any particular document, which belongs to the
module that owns the command producing it.

## Actors

- **Calling agent**, parsing `json` and never `text`.
- **Operator**, reading `text`.
- **Server catalogue**, the origin of the bytes the encoding rules govern.

## `--format`

- **FR-OUT-001**: `--format <text|json>` SHALL default to `text`.

- **FR-OUT-002**: The default SHALL be fixed. The system SHALL NOT consult
  `isatty()`, `TERM`, or any other terminal property to choose a format.

  *Rationale.* `tpl schema tables` and `tpl schema tables | cat` produce
  identical bytes; the help can state `Default: text.` and have it be true in
  every context; and there is one set of escaping rules for catalogue-derived
  text instead of one per terminal state.

- **FR-OUT-003**: `--format` SHALL be declared only by the commands listed in
  `FR-GLOB-021`. Every other command SHALL reject it as an unknown flag, per
  `FR-CLI-019`.

## `text`

- **FR-OUT-004**: `text` output SHALL NOT be a contract. Its shape may change
  without a version bump.

- **FR-OUT-005**: The specification and the help SHALL state that anything
  parsing output must use `--format json`.

- **FR-OUT-006**: `text` output SHALL be laid out for a person: aligned columns
  under a header row, carrying the useful information rather than only the name.

- **BR-OUT-001**: Making `text` a contract would freeze every listing forever —
  no column, header, or total could ever be added. Making it a bare list of
  names would compose better into `xargs`, at the cost of the information a
  reader wants. The choice is deliberate, and the consequence is that iterating
  over objects goes through `--format json`.

## `json`

- **FR-OUT-007**: JSON output SHALL be compact by default: one line, no
  superfluous whitespace, terminated by a single newline.

- **FR-OUT-008**: `--pretty` SHALL produce a two-space indent with one key per
  line.

- **FR-OUT-009**: On a command that declares `--format`, `--pretty` SHALL
  require `--format json`. IF `--pretty` is supplied without it, THEN the system
  SHALL exit `64` (`EX_USAGE`).

- **FR-OUT-010**: On a command whose only output is JSON, `--pretty` SHALL stand
  alone, without any accompanying `--format`. `tpl schema dump --pretty` is
  valid; `tpl schema dump --format json` is `64`, per `FR-SCH-019`.

- **FR-OUT-011**: Every JSON document `tpl` emits SHALL begin with
  `schema_version`, which versions the document contract independently of the
  binary version. Its position, and the two keys that follow it, are fixed by
  the envelope of `FR-OUT-024`.

- **FR-OUT-012**: An absent value SHALL be emitted as `null` and SHALL NOT be
  omitted, so that the shape of a document is constant.

- **FR-OUT-013**: Keys SHALL be emitted in a fixed order per structure. The
  emitting path SHALL NOT use an unordered map.

- **FR-OUT-014**: The JSON contract SHALL follow this compatibility rule:

  | Change | Breaking |
  |---|---|
  | Adding a field | No |
  | Adding a value to an enumerated field such as `source` | No |
  | Removing a field | Yes |
  | Renaming a field | Yes |
  | Changing the type of a field | Yes |

  What a **Yes** in the second column obliges is `FR-OUT-038`. This
  requirement classifies and does not act; that requirement says what the
  classification costs.

  *Amended in the thirty-first edition: the table says where its own verdict is
  acted on.* The sentence above is the whole of the change, and no cell moves.
  The table has classified changes as breaking since the first edition and no
  requirement anywhere said what being classified breaking obliges, so the
  column was a verdict with no sentence. `FR-OUT-038` is that sentence, and the
  pointer is here because this is the table a reader is looking at when the
  question arises.

- **FR-OUT-038**: `schema_version` SHALL carry one value across every document
  of the envelope of `FR-OUT-024`. A change `FR-OUT-014` classifies as breaking
  SHALL increase it by exactly one; a change that requirement classifies as
  non-breaking SHALL leave it unchanged; and it SHALL never decrease.

  The obligation SHALL bind from the **first release** of the binary. Before
  that release `schema_version` SHALL be `1`, whatever breaking changes the
  document shapes undergo, and a breaking change made before it SHALL NOT move
  the value.

  *Rationale.* `FR-OUT-011` says the key versions the document contract and
  `FR-OUT-025` says it is first, and between them nothing said what moves it.
  A version exists for one reader: a consumer that has read a document under
  one value and needs to know whether it may read the next the same way. The
  two clauses above are that reader's whole interest — the number moves exactly
  when the old way of reading stops working, and it does not move when it does
  not.

  *Why the release, and not the edition that makes the change.* Before the
  first release no document this project emits has reached such a consumer, so
  a value moved now would record this project's own edits rather than anything
  anybody read. A first release shipping `schema_version: 4` because three
  shapes were corrected while the arms were being built tells a consumer
  nothing it can act on, and it costs this corpus a rewrite of every worked
  example each time. `FR-SRV-019` already makes a release a gate this corpus
  states obligations against, so the term is one this specification uses rather
  than one it introduces here.

  *Observed, 2026-09-21.* `Column::column_type` was flattened from an object to
  a string in this sprint, which the fifth row of `FR-OUT-014` classifies as
  breaking, and `schema_version` stayed at `1`. Under this requirement that is
  correct and not an oversight: `tpl` has never been released and no version
  has ever been tagged, so the contract had no consumer to protect. The six
  files carrying `schema_version: 1` in a worked example —
  [output-formats.md](output-formats.md),
  [schema-commands.md](schema-commands.md),
  [cfg-commands.md](cfg-commands.md),
  [cache-commands.md](cache-commands.md),
  [help-and-version.md](help-and-version.md) and
  [template-commands.md](template-commands.md) — are therefore untouched, and
  the first breaking change after the first release moves all six together.

  *Accepted cost.* One value governs every document, so a breaking change to
  one command's `data` moves `schema_version` for the other sixteen, and a
  consumer of `tpl cfg list` re-checks a document that did not change. The
  alternative is a second version key, per document, and `FR-OUT-028` forbids
  the envelope a fourth key. The cost falls on the consumer that reads more
  than one document and it costs it one comparison; the alternative costs every
  consumer a key it has to find before it can read anything.

  *Rejected: obliging the increase from the first edition, with no release
  qualifier.* It obliges `schema_version` to move now, and again at every
  further pre-release change, and it makes the number a count of this project's
  own history. It also states an obligation this corpus could not have met: the
  document shapes were fixed across the third, fifth, seventh and twenty-first
  editions, and none of them moved the value.

  *Rejected: leaving `FR-OUT-014` with no obligation attached, on the ground
  that `FR-OUT-011` implies one.* It does not. A key that "versions the
  document contract" says what the key is for and not when it moves, and an
  implementer deriving the emitter from this corpus found no answer — which is
  how a breaking change shipped this sprint against a value that did not move,
  correctly as it turns out, and for no reason this corpus had written down.

  *Rejected: tying `schema_version` to the binary version.* `FR-OUT-011` makes
  the two independent in terms, and for the reason that survives here: a
  breaking change to the binary's command line is not a breaking change to a
  document, and a consumer that parses documents would be made to re-check on
  every release that touched a flag.

- **FR-OUT-015**: WHEN the outcome of an invocation is an error, the system
  SHALL ignore `--format`, SHALL write the four-line text diagnostic of
  `FR-ERR-008` to stderr, and SHALL leave stdout empty. No error is emitted as
  JSON, per `FR-ERR-033`.

  *Amended in the fifth edition.* The first edition read "WHEN an error is
  emitted in JSON, the document SHALL go to stderr". `FR-ERR-033` withdraws
  the JSON error document altogether, so the conditional has no case left. What
  survives from the original — stdout empty, diagnostic on stderr — is now
  unconditional, and `--format` no longer reaches an error at all.

- **FR-OUT-016**: `tpl render --context` SHALL accept a document in either
  compact or indented form. What it accepts is the envelope of `FR-OUT-024`
  carrying the `data` of `tpl schema dump`, per `FR-SCH-036`.

## The document envelope

Every JSON document `tpl` writes to stdout has the same outer shape. Only the
`data` payload differs between commands, and the module that owns a command
owns the shape of its `data`.

- **FR-OUT-024**: Every JSON document the system writes to stdout SHALL have
  exactly the following outer shape, carrying exactly these three keys, in this
  order:

  ```json
  {"schema_version":1,"source":"server","data":{…}}
  ```

  *Rationale.* A calling agent that has learned one document has learned the
  outer shape of all seventeen. The alternative the first edition left in place
  was seventeen independent documents of which only two were ever specified,
  which made `--format json` a set of transport rules with nothing under them.

  *Rejected.* A separate envelope for a listing and for a single object, which
  would oblige a caller to know which of the two a command returns before it
  can read the header. The distinction survives inside `data`, per `FR-OUT-030`
  and `FR-OUT-031`, where a caller reaches it after the header rather than
  before.

- **FR-OUT-025**: `schema_version` SHALL version the document contract, per
  `FR-OUT-011`, and SHALL be the first key. What moves it is `FR-OUT-038`.

  *Amended in the thirty-first edition.* The last sentence is a
  cross-reference and not a second rule. This requirement fixes the key's
  position and its subject and never said when its value changes; a reader
  resolving that question arrived here from `FR-OUT-011` and was sent back.

- **FR-OUT-026**: `source` SHALL state where the bytes of `data` came from, and
  SHALL take one of exactly the following values:

  | Value | Meaning |
  |---|---|
  | `server` | Read from a server on this invocation |
  | `cache` | Served from `.tpl/.cache/`, per `FR-CACHE-006` |
  | `project` | Read from `.tpl/` alone, with no catalogue involved |
  | `binary` | Produced by the binary from itself, with nothing read |

  *Rationale.* `source` answers one question — is this result live, or may it
  be stale — and it must answer it for every document, or a caller has to know
  which commands carry the field before it can read one. `cache-documents.md`
  owns what the value `cache` withdraws, per `FR-CDOC-015` and `FR-CDOC-016`;
  this requirement owns the position of the field and the set of values it
  takes. Adding a value to the set is not a breaking change, per `FR-OUT-014`
  and `FR-CDOC-010`.

  *Rejected.* `null` wherever neither `server` nor `cache` applies, which would
  say "not known" where the truth is "not applicable", and would leave a caller
  unable to tell a project read from a failure to record the source.

- **FR-OUT-027**: `data` SHALL carry the whole of the command's result. The
  module that owns a command SHALL fix the shape of that command's `data`, and
  the shape SHALL be the same whatever the value of `source`.

- **FR-OUT-028**: The system SHALL NOT place any part of a result beside
  `data`. The envelope SHALL NOT grow a fourth key for the benefit of one
  command.

- **FR-OUT-029**: `restricted`, per `FR-PRIV-005`, SHALL be a field of the
  object it qualifies, inside `data`. It SHALL NOT appear on the envelope, and
  a document SHALL NOT carry a single envelope-level flag standing in for the
  per-object marking, per `FR-PRIV-006`.

- **FR-OUT-030**: WHERE a command returns a collection, `data` SHALL be an
  object carrying one key, named for the collection in the plural, whose value
  is a JSON array of its members, per `FR-CTX-003`.

  ```json
  {"schema_version":1,"source":"server","data":{"tables":[…]}}
  ```

- **FR-OUT-031**: WHERE a command returns one named object, `data` SHALL be an
  object carrying one key, named for the kind in the singular, whose value is
  that object.

  ```json
  {"schema_version":1,"source":"cache","data":{"table":{…}}}
  ```

- **FR-OUT-032**: The envelope of `FR-OUT-024` SHALL govern every JSON document
  the system emits, without exception. There is no JSON document outside it,
  because there is no JSON error document: `FR-ERR-033` withdraws it.

  *Amended in the fifth edition.* The first edition exempted one document, the
  JSON error of `FR-ERR-014`, on the ground that the envelope's `source` and
  `data` have no meaning for a failure. That ground was sound and is now
  settled the other way round: rather than an exempt document, there is no
  document. The consequence for a caller is that stdout is the only stream
  carrying JSON, and everything on it is enveloped.

- **BR-OUT-002**: The seventeen documents are enumerated by their owning
  modules and nowhere else: `FR-SCH-030` through `FR-SCH-036` for the eight
  commands of the first arm, `FR-TMPL-028` through `FR-TMPL-031` for the two of
  the second, `FR-CACHE-034` and `FR-CACHE-035` for `tpl cache status`,
  `FR-CFG-035` through `FR-CFG-040` for the five configuration commands, and
  `FR-HELP-017` through `FR-HELP-019` for the command tree. This file fixes the
  envelope they share and states no `data` shape of its own.

## Empty results

- **FR-OUT-033**: WHEN a command's result set is empty, the system SHALL
  succeed and SHALL exit `0`.

  *Rationale.* Empty is a state, not a failure. `66` (`EX_NOINPUT`) is reserved
  by `FR-ERR-001` for an object the caller *named* and that does not exist; a
  caller that asked what exists and was told "nothing" received a correct
  answer. `FR-CACHE-026` already settles the case this way for
  `tpl cache status`, and a fresh project has no database entry at all, per
  `FR-PROJ-018`, so an empty listing is the ordinary first state of a project
  rather than an edge case.

- **FR-OUT-034**: WHEN a `text` result set is empty, the system SHALL print the
  header row of `FR-OUT-006` and nothing beneath it.

  *Rationale.* The `text` shape then does not change with the data, and a
  reader sees which listing answered rather than a blank screen.

- **FR-OUT-035**: WHEN a `json` result set is empty, `data` SHALL carry its
  collection key with an empty array, per `FR-CTX-004`. The key SHALL NOT be
  omitted and its value SHALL NOT be `null`.

- **FR-OUT-036**: `FR-OUT-033` through `FR-OUT-035` SHALL apply to every
  command that returns a collection, naming no exception: `tpl schema tables`,
  `tpl schema views`, `tpl schema routines`, `tpl schema dump` over a database
  with no covered object, `tpl template list`, `tpl cfg list`,
  `tpl cfg database list`, and `tpl cache status`.

- **FR-OUT-037**: `FR-OUT-033` SHALL apply on its own to a command that
  returns no collection and finds nothing to do: `tpl template check` invoked
  with no positional argument in a project with no template checks nothing and
  exits `0`, writing nothing, per `BR-CLI-004`.

## Character encoding

- **FR-OUT-017**: WHEN a catalogue value contains a byte sequence that is not
  valid UTF-8, the system SHALL replace it with U+FFFD and SHALL continue.

  *Rationale.* One odd byte in a legacy comment must not prevent the other 199
  tables from being read. Failing the whole command would block `schema dump`
  entirely.

  *Accepted cost.* A printed value is then no longer byte-identical to what the
  server holds, and a template generating code from a comment receives U+FFFD
  rather than the original byte — which was not representable anyway.

- **FR-OUT-018**: The system SHALL escape C0 control characters in the `text`
  and `json` output of the read commands — `schema`, `template`, `cfg`, and
  `cache`. Tab is excepted in `text` output alone. In `json` output there is no
  exception to make: the format admits no raw control character inside a
  string, so a tab SHALL be emitted as the escape JSON defines for it, which is
  how this requirement is satisfied there.

  *Rationale.* No catalogue byte may reach the terminal uninterpreted. An escape
  character in a column comment must not be able to rewrite what the user sees.
  Tab is excepted because the `text` listings of `FR-OUT-006` are laid out in
  aligned columns and a tab is layout there rather than content. A JSON
  document has no columns to align, and nothing to except: the format itself
  keeps the byte from reaching a consumer raw.

  *Amended in the third edition.* The first edition extended this rule to
  "every diagnostic message", where it contradicted `FR-ERR-024`: that
  requirement escapes `\t` in a message and this one excepted it. Diagnostics
  are now governed by `FR-ERR-024` alone, which escapes tab, because the
  four-line `error:` / `cause:` / `hint:` / `exit:` format has no columns to
  align and a tab inside an interpolated catalogue name can only misalign the
  labels a caller reads on.

  *Amended in the ninth edition: the exception is confined to the output that
  needs it.* The rule read "tab excepted" across `text` and `json` alike, and
  no JSON document can honour that. JSON admits no raw control character inside
  a string, so a document with an unescaped tab in one is not JSON, and
  `FR-OUT-024` makes the document contract. Read literally the rule obliged an
  invalid document; read for its purpose it obliged nothing on that path,
  because the format already keeps the byte from arriving raw. The exception
  now names the output it governs and the `json` path states what satisfies the
  rule there. No emitted byte changes: this is the wording catching up with the
  only behaviour either format ever admitted.

- **FR-OUT-019**: The escaping of `FR-OUT-018` SHALL apply to every value
  interpolated into that output, whatever its source: the catalogue, a
  `--context` document, or the argument vector. It SHALL NOT apply to the
  result of `tpl render`, nor to the template source printed by
  `tpl template show`, both of which SHALL be emitted byte for byte.

  *Amended in the second edition.* The first edition applied the escaping to all
  output, which made `tpl render` unable to emit a view definition: a newline is
  a C0 control and was not excepted, so `{{ view.definition }}` would have
  produced literal escape sequences instead of line breaks — and the same for a
  routine body and for any multi-line comment. The rule was written for read
  output and for diagnostics, where the line-forgery threat is real; the render
  result is by definition text destined for a source file, and the rule was
  never meant to reach it.

  *Rejected.* Excepting newline and carriage return alongside tab everywhere,
  which reopens the forged `hint:` line; and a `raw` filter the author applies
  to opt out, which every multi-line comment, every routine body, and every
  expression default would need.

  *Amended in the third edition.* `tpl template show` joins `tpl render` in the
  exception, on the ground the second-edition amendment already used. Its
  result is not a catalogue value interpolated into a listing; it is a project
  file the caller asked to see verbatim, per `FR-TMPL-015`, and an escaped
  `show` cannot be piped back into a file. The credential-dump path that
  motivated the rule is closed by `FR-TMPL-024` and `FR-TMPL-025` instead. The
  clause about diagnostic messages moved to `FR-ERR-024`, per the amendment to
  `FR-OUT-018`.

## Streams

- **FR-OUT-020**: Results SHALL go to stdout. Warnings, diagnostics, and log
  output SHALL go to stderr.

- **FR-OUT-021**: A command whose output is meant to be piped SHALL write
  nothing else to stdout.

- **FR-OUT-022**: Help output is a result and goes to stdout, per `BR-CLI-005`.

- **FR-OUT-023**: WHEN a command writes no result — `tpl init`,
  `tpl cfg database add` — stdout SHALL remain empty.

## Dependencies

- [cli-contract.md](cli-contract.md) — determinism, and the stream separation
  rule this file details.
- [errors-and-exit-codes.md](errors-and-exit-codes.md) — `FR-ERR-033`, which
  keeps every error out of JSON, and the `EPIPE` rule that `FR-ERR-025` and
  `FR-ERR-026` divide on whether a JSON document is **in flight**, which
  [glossary.md](glossary.md#in-flight) defines.
- [help-and-version.md](help-and-version.md) — the JSON command tree, which
  obeys every rule here.

## Open questions

None specific to this module. The `data` shape of an individual document may
carry an open question of its own, listed by the module that owns the command
per `BR-OUT-002`; none does, and the index of
[open-questions.md](open-questions.md) is empty.
