---
title: Render Command (Third Arm)
status: approved
last-reviewed: 2026-09-21
related: [schema-commands.md, template-commands.md, cache-commands.md, output-formats.md]
---

# Render Command (Third Arm)

## Overview

The third arm composes the first two: it reads the catalogue, reads a template,
renders it, and prints the result. Its pipeline is fixed and has no other steps,
and one invocation produces exactly one rendered result on stdout.

## Scope

In scope: the positional argument, the object flags, `--set`, `--context`, how
the context is assembled from its sources, and what the command does not have.

Out of scope: the template language, the content of the context variables, the
filters and tests available to a template, and how the catalogue is read.

## Actors

- **Calling agent** or **operator**, invoking the command.
- **Project**, supplying the template and the database entry.
- **Context source** — either a database entry or a JSON context file, never
  both.

## Command surface

```
tpl render <template>                       whole database in context
tpl render <template> --table   <name>      binds table
tpl render <template> --view    <name>      binds view
tpl render <template> --routine <name>      binds routine
```

- **FR-RND-001**: `tpl render` SHALL take exactly one positional argument, the
  template name, resolved by the rules of `FR-TMPL-006` and `FR-TMPL-007`.

- **FR-RND-002**: One invocation of `tpl render` SHALL produce exactly one
  render.

  *Rationale.* A template may be composed of several files through `include`,
  `import`, and `extends`, but the invocation produces one result. This removes
  the separator question between concatenated renders, partial-output-on-failure
  semantics, canonical ordering across many objects, and the
  memory-versus-streaming trade-off of a multi-object render.

- **FR-RND-003**: The system SHALL name the render target with a flag, never
  positionally. The flags are `--table <name>`, `--view <name>`, and
  `--routine <name>`.

  *Rationale.* The object type is always explicit, so there is no name lookup
  across three catalogues, no cross-namespace collision to resolve, and the
  template knows which context variable is bound.

- **FR-RND-004**: Each of `--table`, `--view`, and `--routine` SHALL take
  exactly one value and SHALL be given at most once. A repetition is `64`, per
  `FR-CLI-014`.

- **FR-RND-005**: IF more than one kind of object flag is supplied in one
  invocation, THEN the system SHALL exit `64` (`EX_USAGE`).

- **FR-RND-006**: WHEN no object flag is supplied, the system SHALL render the
  template once with the whole database in context and no object variable bound.

- **FR-RND-007**: The system SHALL NOT provide `--all-tables`, `--all-views`,
  `--all-routines`, or `--pattern` on `tpl render`.

- **BR-RND-001**: Because kinds cannot be mixed and only one object is bound,
  the bound context variable is the same for the whole invocation. A template
  written for tables never receives a view, and therefore never has to defend
  itself with `{% if table is defined %}` against strict undefined-variable
  behaviour.

- **BR-RND-002**: Iterating over many objects is the caller's job, one
  invocation per object. The `EXAMPLES` section of `tpl render` SHALL show the
  canonical loop, which uses `tpl schema tables --format json`.

## `--set`

- **FR-RND-008**: `tpl render` SHALL declare `--set <key>=<value>`, which
  defines one entry under the `vars` context variable. It is repeatable with
  distinct keys. It SHALL carry no short form, per `FR-GLOB-024`.

- **FR-RND-009**: The system SHALL split a `--set` argument on the first `=`.
  Everything after that first `=` is the value, so `--set "msg=a=b"` sets
  `vars.msg` to `a=b`.

- **FR-RND-010**: An empty value SHALL be valid. `--set empty=` sets
  `vars.empty` to the empty string.

- **FR-RND-011**: IF a `--set` argument contains no `=`, THEN the system SHALL
  exit `64`.

- **FR-RND-012**: A `--set` key SHALL match `[A-Za-z_][A-Za-z0-9_]*`. IF it does
  not, THEN the system SHALL exit `64`.

  *Rationale.* The constraint guarantees that `vars.<key>` always resolves in a
  template. `my key` and `db.host` are rejected.

- **FR-RND-013**: The system SHALL NOT interpret a dotted key as nested
  structure. A dotted key is rejected under `FR-RND-012`, not split.

- **FR-RND-014**: IF the same `--set` key is supplied more than once, THEN the
  system SHALL exit `64`.

  *Rationale.* Last-wins would let a script emitting the same key twice pass
  unnoticed, with the result depending on ordering.

- **FR-RND-015**: A `--set` value SHALL always be a string. The system SHALL NOT
  infer a number, a boolean, or any other type from it; conversion is the
  template's job.

  *Rationale.* JSON-typed values would silently turn `--set version=1.0` into
  the number `1.0` and `--set name=true` into a boolean, which is a trap for
  generated command lines.

## `--context`

- **FR-RND-016**: `tpl render` SHALL declare `--context <path>`, which supplies
  the server-derived part of the context from a JSON file instead of from a
  database.

- **FR-RND-017**: `--context -` SHALL read the document from stdin.

  ```
  tpl schema dump | tpl render x.jinja --context - --table orders
  ```

- **FR-RND-018**: IF `--context` is supplied together with `-d/--database` given
  explicitly on the command line, THEN the system SHALL exit `64`.

  *Rationale.* Two conflicting context sources.

- **FR-RND-019**: WHEN `--context` is supplied and no `-d/--database` appears on
  the command line, the system SHALL ignore `core.database` rather than treat it
  as a conflict.

- **FR-RND-020**: IF the document supplied to `--context` is not well-formed
  JSON, or does not match the document contract — the outer shape of
  `FR-SCH-017` and the structural rules of
  [context-document.md](context-document.md) — THEN the system SHALL exit `65`
  (`EX_DATAERR`). Bytes that are not valid UTF-8 are bytes that are not
  well-formed JSON and are `65` under this requirement; the `cause` SHALL name
  the path and the offset of the first byte the decoder could not read, which
  is the position the `65` row of `FR-ERR-034` obliges beside the path.

  *Amended in the thirty-first edition: the encoding clause is stated rather
  than left to the implementation.* Which code a `--context` document in the
  wrong encoding produces was decided in code because no requirement fixed it,
  and `65` is the answer this requirement now carries. Nothing else about it
  changes: the two conditions it already stated are the two it states, and the
  encoding is a third way of failing the first of them rather than a fourth
  condition.

  *Source.* RFC 8259 (STD 90), section 8.1, makes UTF-8 the encoding of a JSON
  text exchanged outside a closed ecosystem, and `tpl` accepts this document
  from any writer — a `tpl schema dump` of its own, a file a caller wrote, or a
  producer at the other end of a pipe under `FR-RND-017`. Checked 2026-09-21.
  It does not decay the way the table of `FR-SRV-015` decays: an RFC is fixed
  once published, so what would change this is an IETF standard obsoleting that
  one, and the trigger is that event rather than a date.

  *Why this is not `FR-OUT-017`, which replaces and continues.* That rule
  governs a byte the **catalogue** returns, and its ground is that one odd byte
  in a legacy comment must not stop the other 199 tables from being read — the
  caller cannot fix the server's bytes and has no other way to get the read.
  A `--context` document is one artefact the caller supplies and can correct,
  and replacing a byte inside it with U+FFFD would change the model a template
  renders from, silently, at exit `0`. The two rules point the same way for the
  same reason: the byte the caller can fix is refused, and the byte the caller
  cannot fix is carried.

  *Rejected: `74`, on the ground that an encoding fault is a fault of the
  stream.* The read succeeded — every byte arrived — and what failed is the
  document, which is what `65` means and where `FR-RND-035` draws the line
  between the two.

- **FR-RND-035**: IF the file or the stream supplied to `--context` cannot be
  read — it does not exist, the filesystem refused the read, or the stream
  failed part-way — THEN the system SHALL exit `74` (`EX_IOERR`), and the
  `cause` SHALL name the path or the stream, the operation attempted on it, and
  what the filesystem or the stream returned, per the `74` row of
  `FR-ERR-034`.

  *Rationale.* Nothing had fixed this code, and the three candidates are `66`,
  `74` and `65`. It is not `65`: no document was obtained, so there is nothing
  to be malformed. It is not `66`, although a missing path looks like a named
  object that does not exist: every population that row of `FR-ERR-001` names
  is one this system can list — tables, views, routines, templates, database
  entries, configuration keys — and `FR-ERR-019` admits a candidate only from
  such a population. A filesystem path has none, so a `66` would arrive with no
  suggestion and with a `cause` unable to name the population it searched,
  which is what the `66` row of `FR-ERR-034` obliges. `74` names the path, the
  operation and what the filesystem returned, which is the whole of what the
  caller needs and is exactly that code's row.

  *Why the absence of the file is not separated from the other read failures.*
  The caller's next step is the same for all three — look at the path, and at
  what the filesystem said about it — and `FR-ERR-002` admits one code for two
  conditions exactly where that holds, obliging the `cause` to separate them
  for a reader. It does: `no such file or directory` and `permission denied`
  are two different `cause` lines under one code.

  *Rejected: `66` for a path that does not exist, and `74` for every other read
  failure.* It splits one fault across two codes on a distinction the caller
  cannot act on differently, and it puts `tpl render x --context gone.json` and
  `tpl render x --context locked.json` on two branches that lead to the same
  place. It would also oblige a nearest-match suggestion over a population this
  system never reads, which is the ground `FR-PRIV-021` already refuses a `66`
  on for a database.

  *Under `--context -` the stream is standard input*, and a read that fails
  there is this requirement's condition too. It is not the `EPIPE` of
  `FR-ERR-025` and `FR-ERR-026`, which govern a consumer closing **stdout**;
  this is `tpl` as the consumer, and the producer at the other end.

- **FR-RND-021**: The system SHALL accept a `--context` document in either
  compact or indented form.

- **FR-RND-022**: WHEN `--context` is supplied, the system SHALL NOT open a
  database connection and SHALL NOT read or write the cache.

## Context assembly

- **FR-RND-023**: The system SHALL assemble the render context from four
  sources:

  | Variable | Source |
  |---|---|
  | `database` | the context source — a catalogue read or the `--context` document |
  | `table` / `view` / `routine` | the context source, selected by the object flag |
  | `vars` | the `--set` flags of this invocation |
  | `tpl` | the binary |
  | `now` | the clock, at render time |

- **FR-RND-024**: The system SHALL always inject `vars`, `tpl`, and `now`. IF
  any of the three appears in a `--context` document, THEN the system SHALL
  ignore the supplied value. What each of the three holds is fixed by
  `FR-CTX-026` through `FR-CTX-028`.

  *Rationale.* A dump cannot produce "exactly the JSON the render receives as
  context", because three of the five top-level variables do not come from the
  database.

## Flags, cache, and output

- **FR-RND-025**: `tpl render` SHALL declare `--direct` and `--no-cache`, with
  the meanings defined in [cache-commands.md](cache-commands.md).

- **FR-RND-026**: WHEN no `--context` is supplied, `tpl render` SHALL read the
  catalogue through the cache, per `FR-CACHE-006`.

  *Rationale.* Sending `render` to the server while caching the first arm would
  let a stale catalogue produce wrong code that then gets committed; caching
  both keeps the two consistent.

- **FR-RND-027**: `tpl render` SHALL NOT declare `--format` or `--pretty`. Its
  result is the rendered text, which has no alternative representation.
  `tpl render x --format json` is therefore `64`, per `FR-CLI-019`, and the
  diagnostic is text like every other, per `FR-ERR-033`.

  *Amended in the fifth edition.* The last sentence is new and settles what
  `OQ-023` asked. The question only existed because the pre-scan of
  `FR-ERR-017` would have seen `--format json` on a command that declares no
  such flag and emitted the resulting `64` as JSON. `FR-ERR-017` is withdrawn
  and no error is JSON, so the outcome is the ordinary unknown-flag error with
  nothing special about it.

- **FR-RND-028**: `tpl render` SHALL write the rendered result to stdout and
  SHALL NOT write it anywhere else. The system SHALL NOT provide `--output`,
  `--output-dir`, `--output-name`, `--no-clobber`, or `--dry-run`.

  ```
  tpl -d shop render rust/struct.jinja --table orders > src/models/orders.rs
  ```

  *Rationale.* Removing the file-writing surface removes, at a stroke, atomic
  write and its temporary-file mode, the regular-file check, FIFO and device
  destinations, filename expressions as a path-injection sink, filename
  collisions between objects, overwrite policy, and the split between `73` and
  `74` for destination errors.

- **BR-RND-003**: If writing to disk is ever brought back into scope, two
  findings already established must be re-applied rather than re-derived: a
  filename expression over catalogue data is a path-injection sink, reachable
  from `--context` as well as from the database; and two objects can collapse
  onto one filename after a casing filter, silently losing a write and breaking
  determinism.

## Failure

- **FR-RND-029**: IF the named template does not exist, THEN the system SHALL
  exit `66` with a nearest-match suggestion, per `FR-TMPL-027`.

- **FR-RND-030**: IF the template contains a syntax error, THEN the system SHALL
  exit `65`, naming the template, the line, and the column, and propagating the
  chain of underlying template-engine errors.

- **FR-RND-031**: IF the render fails at evaluation time — an undefined
  variable, a failing filter, an escape from the template root — THEN the system
  SHALL exit `65`, with the same location information.

- **FR-RND-032**: IF the named object does not exist in the context source, THEN
  the system SHALL exit `66` with a nearest-match suggestion. IF `--routine` is
  given a bare name matching both a procedure and a function, THEN the system
  SHALL exit `64`, per `FR-SCH-010`, which applies under any circumstance.

- **FR-RND-033**: IF the render deadline is exceeded, THEN the system SHALL exit
  `65`.

- **FR-RND-034**: WHEN a render fails, stdout SHALL carry at most one incomplete
  result.

## Dependencies

- [template-commands.md](template-commands.md) — template naming, resolution,
  and containment.
- [schema-commands.md](schema-commands.md) — the dump document that
  `--context` consumes.
- [cache-commands.md](cache-commands.md) — read-through behaviour and the two
  cache flags.
- [errors-and-exit-codes.md](errors-and-exit-codes.md) — `64`, `65`, `66`, and
  the `74` of `FR-RND-035`.

## Open questions

None specific to this module. `OQ-016` is answered by `FR-GLOB-024` — `--set`
has no short form — and `OQ-023` is **dissolved** with the pre-scan of
`FR-ERR-017`; both are listed under [Closed](open-questions.md#closed).
