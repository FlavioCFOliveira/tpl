---
title: The Template Environment
status: approved
last-reviewed: 2026-09-10
related: [render-semantics.md, render-command.md, context-document.md, help-and-version.md]
---

# The Template Environment

## Overview

A template reaches the model through the context variables and acts on it
through filters, tests, and global functions. This file defines that surface:
what `tpl` registers, what it guarantees from the engine underneath, what it
deliberately does not provide, and what it forbids.

The surface is contract. What is guaranteed here can be relied on by a template
committed to a repository; what is not guaranteed may work today and stop
working when the engine is updated, and the difference is stated rather than
left to be discovered.

## Scope

In scope: the three contract groups and the pin the second is guaranteed
against, the filters and tests `tpl` registers and what each of them does, the
global functions, the removals and their reasons, explicit escaping, and how
the surface is published.

Out of scope: evaluation semantics — the general rule for an operand of the
wrong type, what an interpolated `null` produces, and how whitespace is handled
— which belong to [render-semantics.md](render-semantics.md); and the content
of the context variables, which belongs to
[context-document.md](context-document.md).

## Actors

- **Template author**, writing against this surface.
- **Calling agent**, discovering it through `tpl help --format json`.
- **Template engine**, whose own surface underlies part of it.

## The three contract groups

- **FR-ENV-001**: The template surface SHALL consist of exactly three groups:

  | Group | Contents | Guarantee |
  |---|---|---|
  | 1 | The filters, tests, and functions `tpl` registers | Full contract |
  | 2 | An enumerated list of filters inherited from the engine | Guaranteed against a pinned engine minor version |
  | 3 | Everything else the engine offers | Works; no guarantee |

- **FR-ENV-002**: A name in group 1 SHALL NOT be renamed or removed without a
  breaking change. Adding a name to group 1 SHALL NOT be breaking.

- **FR-ENV-003**: A name in group 2 SHALL behave as the pinned engine minor
  version defines it. The pin SHALL exist, SHALL be recorded in the project's
  architecture decision records, and SHALL be cited from there by this
  requirement rather than restated in this specification. Changing it SHALL be
  a deliberate decision recorded in the same place, and SHALL be accompanied by
  a check that every name of `FR-ENV-018` still exists and still behaves as
  before.

  *Amended in the fifth edition.* The requirement guaranteed group 2 "against a
  pinned engine minor version" and nothing obliged that pin to exist, to be
  written down, or to be findable — so the guarantee named a fact nobody was
  required to establish. Three obligations replace one: the pin exists, it is
  recorded in a named place, and this requirement cites that place. The version
  number itself never enters this corpus, for the reason `BR-SRV-005` gives
  about the supported-series table and `BR-PERF-006` about a measured figure: a
  number copied into a second file is the copy that stops being true without
  saying so, and a dependency version moves on a schedule this specification
  does not set.

  *Closes* `OQ-049`, now listed under [Closed](open-questions.md#closed). The
  question asked which minor version the pin names. That is the one thing this
  requirement will not answer, and the amendment is the recognition that it was
  the wrong question for this corpus to hold open: what a specification can
  require is that the pin exist and be findable, and what it cannot own is a
  dependency version.

  *Accepted cost.* A reader of this file cannot see which engine version group
  2 is guaranteed against without opening another document. That is the same
  cost `BR-PERF-006` accepts for a measured figure, and it is paid for the same
  reason.

- **FR-ENV-004**: The specification and the help SHALL state that group 3
  carries no guarantee, so that a template author knows which side of the line a
  name falls on before depending on it.

- **BR-ENV-001**: Inheriting the engine's whole surface as contract was
  rejected: it would let an engine update break `tpl` without anyone having
  decided anything. Guaranteeing only `tpl`'s own registrations was also
  rejected: it would leave `{{ cols | join(", ") }}`, the most common idiom
  there is, with no guarantee at all.

- **FR-ENV-005**: `tpl help --format json` SHALL enumerate all three groups: the
  names of group 1, the names of group 2, and a statement that group 3 exists
  and is unguaranteed.

  *Rationale.* A calling agent loads the whole surface in one invocation, per
  `FR-HELP-016`. A surface it cannot enumerate is one it will discover by trial
  and error.

## Filters that `tpl` registers

- **FR-ENV-006**: The system SHALL register the naming filters `pascal`,
  `camel`, `snake`, `upper_snake`, and `kebab`.

- **FR-ENV-007**: The system SHALL register the code filters `quote`,
  `sql_type`, `json`, `indent`, `comment`, and `escape`.

  *Amended in the fifth edition.* `escape` is added to the enumeration. It was
  required by `FR-ENV-028` and named in no group, so as written it fell into
  the unguaranteed group 3 — which cannot be right for the one filter the
  escaping decision of `FR-ENV-026` depends on. `FR-ENV-044` fixes what it
  does. The inherited list of `FR-ENV-018` is untouched: `escape` is registered
  by `tpl`, not inherited.

- **FR-ENV-008**: `quote` SHALL quote an identifier for MariaDB using backticks,
  and SHALL be correct for MariaDB rather than for SQL in general.

## Filters that do not exist

- **FR-ENV-009**: The system SHALL NOT provide a `rust_type` filter.

- **FR-ENV-010**: The system SHALL NOT provide a `go_type` filter, nor a filter
  mapping a column to the types of any other target language.

- **FR-ENV-011**: The mapping those filters performed SHALL instead be delivered
  as a template macro at `.tpl/templates/rust/_types.jinja`, written by
  `tpl init` per `FR-PROJ-017`.

- **BR-ENV-002**: A type mapping is an opinion, and it belongs to the project
  that holds it rather than to the binary. Whether `DECIMAL` becomes a
  third-party decimal type, an `f64`, or a `String` is a decision each project
  makes once and edits when it changes its mind; frozen into `tpl` it would be
  editable only by releasing a new version of `tpl`. Delivering it as a macro
  also stops the tool privileging one target language, which is what a
  `rust_type` in the binary and a `go_type` beside it would institutionalise.

  *Rejected.* Keeping `rust_type` with a standard-library-only mapping, which
  answers the easy cases and leaves `DECIMAL`, `JSON`, and the temporal types
  exactly where they were. Also rejected: keeping it with third-party crate
  types, which would oblige `tpl` to track those crates' major versions for as
  long as it exists.

- **FR-ENV-012**: The system SHALL NOT provide a `plural` filter.

- **FR-ENV-013**: The system SHALL NOT provide a `singular` filter.

- **BR-ENV-003**: Correct English inflection is a project in itself —
  person/people, status/statuses, index/indices or indexes, data/data — and a
  wrong plural applied to a table name that is not English is guaranteed noise
  in generated code. Removing the two filters before anything depends on them is
  free; removing them later would be a compatibility break.

  *Rejected.* Keeping them with a closed rule set and an irregulars list
  published as a test vector, which makes the behaviour predictable without
  making it right. Also rejected: a naive suffix-only version, which would
  document `persons` and `indexs` as correct output.

## Tests

- **FR-ENV-014**: The system SHALL register the tests `nullable`,
  `primary_key`, `auto_increment`, `unique`, `numeric`, `temporal`, and
  `textual`.

- **FR-ENV-015**: `primary_key` and `unique` SHALL resolve the operand's
  `table_name` against the render context, per `FR-CTX-019` and `FR-CTX-022`,
  and SHALL answer from what the table states.

- **FR-ENV-016**: The tests of `FR-ENV-015` therefore depend on the render
  context and not only on their operand. The specification and the help SHALL
  state this.

- **FR-ENV-017**: IF the table named by a column's `table_name` is absent from
  the render context, THEN the system SHALL fail the render with `65`
  (`EX_DATAERR`), per `FR-SEM-017`.

  *Rationale.* The case is reachable: a `--context` document assembled by hand,
  or a column carried across from one context into another, can name a table
  that is not there. Answering `false` would report "this column is not a
  primary key" when the truth is that the question could not be asked.

## Behaviour of the registered surface

`FR-ENV-002` makes every name of group 1 full contract, and `NFR-DET-001`
requires its output to be byte-identical between runs. This section fixes what
each one does. The tables are a specification and a test vector at once: each
row is a case the implementation SHALL satisfy.

### The word list

- **FR-ENV-030**: The five naming filters SHALL act on a word list derived from
  the operand by exactly the following rule, applied once, left to right:

  1. An underscore, a hyphen, or a space SHALL end the current word and SHALL
     NOT appear in the output.
  2. A transition from a lower-case ASCII letter to an upper-case ASCII letter
     SHALL end the current word before the upper-case letter.
  3. A transition from an upper-case ASCII letter to a lower-case ASCII letter
     SHALL end the current word before the upper-case letter, WHERE the
     upper-case letter is preceded by another upper-case letter.
  4. A digit SHALL belong to the word it follows and SHALL NOT begin a word.
  5. An empty word SHALL be discarded.

- **FR-ENV-031**: Each word of the list SHALL be case-folded over ASCII `A-Z`
  and `a-z` only, independently of the server, the database collation, and the
  locale. A character outside that range SHALL pass through unchanged.

  *Rationale.* The same rule `FR-SCH-014` applies to `--pattern`, for the same
  reason: a case transformation that depended on the locale would produce
  different generated code on two machines and break `NFR-DET-001`.

- **FR-ENV-032**: The word list SHALL be these values for these operands:

  | Operand | Word list |
  |---|---|
  | `order_items` | `order`, `items` |
  | `orderItems` | `order`, `Items` |
  | `HTTP_server` | `HTTP`, `server` |
  | `HTTPServer` | `HTTP`, `Server` |
  | `order_2_items` | `order`, `2`, `items` |
  | `utf8mb4` | `utf8mb4` |
  | `__orders__` | `orders` |
  | `` (empty) | (empty list) |

### The naming filters

- **FR-ENV-033**: The five naming filters SHALL join the case-folded word list
  as follows, and SHALL produce exactly these outputs:

  | Operand | `pascal` | `camel` | `snake` | `upper_snake` | `kebab` |
  |---|---|---|---|---|---|
  | `order_items` | `OrderItems` | `orderItems` | `order_items` | `ORDER_ITEMS` | `order-items` |
  | `orderItems` | `OrderItems` | `orderItems` | `order_items` | `ORDER_ITEMS` | `order-items` |
  | `HTTP_server` | `HttpServer` | `httpServer` | `http_server` | `HTTP_SERVER` | `http-server` |
  | `HTTPServer` | `HttpServer` | `httpServer` | `http_server` | `HTTP_SERVER` | `http-server` |
  | `order_2_items` | `Order2Items` | `order2Items` | `order_2_items` | `ORDER_2_ITEMS` | `order-2-items` |
  | `utf8mb4` | `Utf8mb4` | `utf8mb4` | `utf8mb4` | `UTF8MB4` | `utf8mb4` |
  | `orders` | `Orders` | `orders` | `orders` | `ORDERS` | `orders` |
  | `` (empty) | `` | `` | `` | `` | `` |

  The joining rules the table realises: `pascal` upper-cases the first
  character of every word, lower-cases the rest, and joins with nothing;
  `camel` does the same but lower-cases the whole of the first word; `snake`
  lower-cases every word and joins with `_`; `upper_snake` upper-cases every
  word and joins with `_`; `kebab` lower-cases every word and joins with `-`.

  *Rationale.* Every filter is a pure function of the word list, so a template
  author who has learned `FR-ENV-030` can predict all five, and
  `snake(pascal(x))` returns `snake(x)` for every `x`.

  *Accepted cost.* An acronym loses its case: `HTTP_server` becomes
  `HttpServer` and not `HTTPServer`. That is the Rust convention for an
  acronym in `UpperCamelCase`, and preserving the acronym would need a list of
  acronyms — which is the same objection `BR-ENV-003` raised against `plural`
  and `singular`, and it is refused here for the same reason.

- **FR-ENV-034**: IF a naming filter is applied to a value that is not a
  string, THEN the render SHALL fail with `65`, per `FR-SEM-008`. There is no
  coercion, per `FR-SEM-009`.

### The code filters

- **FR-ENV-035**: `quote` SHALL return its operand as a quoted MariaDB
  identifier, using backticks. For every string that is a legal MariaDB
  identifier — including one that contains a backtick — the output SHALL be a
  single quoted identifier that MariaDB parses back to exactly that string.

  *Known gap.* The exact escaping mechanism for a backtick inside an
  identifier is [OQ-070](open-questions.md#oq-070). It is a fact about MariaDB
  and this specification does not state one that has not been observed: there
  is no `scripts/mariadb/` in the repository, so there is no container to
  observe against. The requirement above is a product requirement and needs no
  observation; the mechanism that satisfies it does.

- **FR-ENV-036**: `json` SHALL serialise its operand as a JSON value in the
  compact form of `FR-OUT-007`: one line, no superfluous whitespace, and no
  trailing newline. It SHALL accept an operand of any type the context can
  hold.

  | Operand | Output |
  |---|---|
  | `"a\"b"` | `"a\"b"` |
  | `null` | `null` |
  | a list of two strings | `["a","b"]` |
  | an object of one key | `{"k":"v"}` |

- **FR-ENV-037**: `indent(n)` SHALL prefix every line of its operand after the
  first with `n` space characters. It SHALL NOT indent the first line, SHALL
  NOT indent an empty line, and SHALL NOT add trailing whitespace. `n` SHALL be
  a required argument with no default.

  *Rationale.* A required argument rather than an invisible default, for the
  reason `FR-OUT-002` gives for the `--format` default and `BR-CLI-002` for the
  command line: a value nobody can see in the template is a value that changes
  the generated bytes without appearing in the source. Empty lines are left
  alone because trailing whitespace fails a formatter, which is the argument
  `BR-SEM-001` already used to keep the final newline.

- **FR-ENV-038**: `comment(prefix)` SHALL prefix every line of its operand with
  `prefix` followed by a single space. `prefix` SHALL be a required argument
  with no default, and the filter SHALL NOT choose a comment syntax of its own.

  ```
  {{ table.comment | comment("//") }}
  {{ table.comment | comment("--") }}
  ```

  *Rationale.* A comment syntax is an opinion about a target language, and
  `BR-ENV-002` already settled where such an opinion lives: with the project,
  not in the binary. A `comment` that emitted `//` would privilege one target
  language exactly as `rust_type` did.

- **FR-ENV-039**: `sql_type` SHALL return the **normalised type name** of its
  operand: the value of that column's `data_type` field, exactly as
  `FR-CTX-015` carries it, and nothing else. It SHALL NOT parse, SHALL NOT
  reassemble a type clause, and SHALL NOT consult `column_type`.

  | Operand | Its `data_type` | `sql_type` returns |
  |---|---|---|
  | a column | `"varchar"` | `varchar` |
  | a column | `"decimal"` | `decimal` |
  | a column of a type the system does not recognise | `null`, per `FR-CTX-018` | `null` |

  `sql_type` SHALL accept an operand that is a column object and nothing else.
  IF it is applied to any other operand — a table, a view, a routine, a string,
  a number, `null` — THEN the render SHALL fail with `65`, per `FR-SEM-008`,
  naming the filter, the type received, and the location. It SHALL NOT return
  an empty string for an operand it does not accept. This is the same rule
  `FR-ENV-040` states for the seven tests, which accept a column object and
  nothing else for the same reason.

  WHERE the operand's `data_type` is `null`, `sql_type` returns `null`, and an
  interpolated `null` renders as the empty string, per `FR-SEM-010`. That is
  the one case in which the filter produces nothing, and it is the case
  `FR-CTX-018` already describes: the type was not recognised and
  `column_type` is the safety net a template falls back to.

  *Closes* `OQ-071`, now listed under [Closed](open-questions.md#closed). The
  filter is kept rather than withdrawn.

  *Rationale.* Defined as a convenience over a field that already exists, the
  filter **cannot diverge from the model**: there is no second derivation to
  keep in step, no grammar to get wrong, and nothing for it to be right about
  that `data_type` is wrong about. That is what makes keeping the name
  defensible where a `sql_type` that parsed `column_type` would not be — a
  parser would depend on [OQ-028](open-questions.md#oq-028), the textual form
  of `column_type`, which is unobserved, and it would be a second place the
  type could be misread. It also earns its place beside a plain field read:
  `{{ col | sql_type }}` composes into a filter chain and reads as one, where
  `{{ col.data_type }}` does not, and a template that wants a cased or quoted
  form writes `{{ col | sql_type | upper }}` without reaching past the filter
  vocabulary.

  *Rejected.* Withdrawing the name, which was the recommendation and which the
  user declined. Also rejected, and this is what the definition above forecloses:
  a full DDL type clause including length, nullability, and default, which is
  a fourth representation of a type beside `column_type`, the decomposition of
  `FR-CTX-015`, and this filter; and a form adjusted for a target dialect,
  which is the opinion `BR-ENV-002` keeps out of the binary.

### The tests

- **FR-ENV-040**: Every test of `FR-ENV-014` SHALL accept an operand that is a
  column object and nothing else. IF a test is applied to an operand of any
  other type — a table, a view, a routine, a string, a number, `null` — THEN
  the render SHALL fail with `65`, per `FR-SEM-005`, naming the test, the type
  received, and the location, per `FR-SEM-006`. A test SHALL NOT answer `false`
  for an operand it does not accept, per `FR-SEM-007`.

- **FR-ENV-041**: The seven tests SHALL answer as follows:

  | Test | True when | Answers from |
  |---|---|---|
  | `nullable` | The column's nullability, as the model states it, says the column admits `NULL` | The column alone |
  | `auto_increment` | The column carries the auto-increment attribute of `FR-CAT-027` | The column alone |
  | `primary_key` | The column is named in its table's primary key | `table_name`, resolved against the render context, per `FR-ENV-015` |
  | `unique` | The column is named in an index of its table that the model reports as unique | `table_name`, resolved against the render context, per `FR-ENV-015` |
  | `numeric` | The column's `data_type` belongs to the numeric family | The column alone |
  | `temporal` | The column's `data_type` belongs to the date-and-time family | The column alone |
  | `textual` | The column's `data_type` belongs to the character-string family | The column alone |

- **FR-ENV-042**: The three families of `FR-ENV-041` SHALL be disjoint. A
  column SHALL satisfy at most one of `numeric`, `temporal`, and `textual`, and
  a column of a type in none of the three SHALL satisfy none of them rather
  than fail.

  *Rationale.* Disjointness is testable today and is the property a template
  actually depends on: `{% if col is numeric %}…{% elif col is textual %}` must
  not take two branches, and a spatial or JSON column must be able to fall
  through to neither without failing the render.

  *Known gap.* Which `data_type` values fall in each family is
  [OQ-072](open-questions.md#oq-072). It is a fact about what MariaDB writes in
  the catalogue, the textual form of `data_type` is itself
  [OQ-028](open-questions.md#oq-028), and neither may be carried over from
  MySQL knowledge — there is no container to observe against. The three tests
  are fixed here as disjoint predicates over a family; the memberships are
  not.

- **FR-ENV-043**: `primary_key` and `unique` SHALL fail the render with `65`
  WHEN the table named by the operand's `table_name` is absent from the render
  context, per `FR-ENV-017` and `FR-SEM-017`. The other five tests SHALL NOT
  consult the render context and SHALL NOT fail for that reason.

- **BR-ENV-007**: The tables of this section are the test vector. A change to
  any cell is a change to the contract of `FR-ENV-002` and is breaking, and a
  row added for a case the tables do not cover must be consistent with the
  rules stated beside them rather than settle a new one.

## Inherited filters

- **FR-ENV-018**: The system SHALL guarantee the following inherited filters,
  and no others, under the terms of `FR-ENV-003`:

  `default`, `join`, `length`, `map`, `select`, `reject`, `first`, `last`,
  `reverse`, `sort`, `trim`, `upper`, `lower`, `replace`.

- **FR-ENV-019**: The list of `FR-ENV-018` SHALL be closed. A filter the engine
  offers that is absent from it belongs to group 3.

## Global functions

- **FR-ENV-020**: The system SHALL provide exactly the following global
  functions, and no others:

  | Function | Purpose |
  |---|---|
  | `table(name)` | Resolve a table by name in the render context |
  | `view(name)` | Resolve a view by name in the render context |
  | `routine(name)` | Resolve a routine by name in the render context |
  | `column(table, name)` | Resolve a column of a named table |
  | `fail(message)` | End the render with `65`, carrying the author's message |

- **FR-ENV-021**: `fail(message)` SHALL end the render with `65` and SHALL carry
  the message the template supplied, per `FR-SEM-014`.

- **FR-ENV-022**: The system SHALL NOT provide a function that reads the
  environment.

- **FR-ENV-023**: The system SHALL NOT provide a function that reads a file.

- **FR-ENV-024**: The system SHALL NOT provide a function that performs network
  access.

- **FR-ENV-025**: The system SHALL NOT provide a function that reads a clock.
  The `now` context variable of `FR-RND-023` is the only source of time
  available to a template.

- **BR-ENV-004**: A template arrives in a clone and is read by `tpl` before
  anybody has reviewed it, so it is semi-trusted input. The four prohibitions
  above are what keep `tpl render` from being a way to run arbitrary code with
  the caller's privileges, and they are stated as prohibitions rather than left
  as an absence so that adding one is visibly a change of position.

  *Rejected.* Lookups only, without `fail`, which would leave a generator that
  meets an unmappable type unable to say why it stopped. Also rejected: adding
  a `warn` function, which would let a render finish with exit `0` and hand the
  caller an incomplete file with nothing in the exit code to say so.

## Escaping

- **FR-ENV-026**: Auto-escaping SHALL be off, always.

- **FR-ENV-027**: The system SHALL NOT key escaping on a file extension, on a
  penultimate extension, or on any other property of a name.

- **FR-ENV-028**: `escape` SHALL be available as an explicit filter, registered
  in contract group 1 per `FR-ENV-007`. Escaping happens where a template asks
  for it and nowhere else.

- **FR-ENV-044**: `escape(target)` SHALL take a required `target` argument with
  no default, and SHALL replace the following characters in its operand:

  | Character | `escape("html")` | `escape("xml")` |
  |---|---|---|
  | `&` | `&amp;` | `&amp;` |
  | `<` | `&lt;` | `&lt;` |
  | `>` | `&gt;` | `&gt;` |
  | `"` | `&quot;` | `&quot;` |
  | `'` | `&#39;` | `&apos;` |

  It SHALL replace no other character, SHALL replace `&` before the others so
  that no replacement is escaped twice, and SHALL leave an already-escaped
  entity in the operand alone in no special way — an input `&amp;` becomes
  `&amp;amp;`, because the filter escapes text and does not inspect it for
  markup.

  IF `target` is absent, THEN the render SHALL fail with `65`. IF `target` is
  any value other than `html` or `xml`, THEN the render SHALL fail with `65`,
  naming the filter, the value received, and the two permitted values. IF the
  operand is not a string, THEN the render SHALL fail with `65`, per
  `FR-SEM-008` and `FR-SEM-009`; there is no coercion.

  *Rationale for the required argument.* Modelled exactly on `FR-ENV-038`'s
  treatment of `comment`. A default target would be a value nobody can see in
  the template that changes the generated bytes, which is what `FR-ENV-037`
  refuses for `indent`, `FR-OUT-002` for the `--format` default, and
  `BR-CLI-002` for the command line. It matters more here than for `comment`:
  the two targets differ in exactly one row of the table, so a wrong default
  produces output that is correct almost everywhere and wrong in the one place
  an author would not think to check.

  *Rationale for the two targets, and only two.* They are the two `FR-ENV-027`
  refuses to infer from a filename, and the pair the rejected
  penultimate-extension rule of `BR-ENV-005` covered. A third target would be a
  target this tool does not have, which is the reason `BR-ENV-005` gives for
  refusing an `{% autoescape %}` block.

  *Rationale for the one row that differs.* `&apos;` is defined by XML. In HTML
  it is not available in every version, so the numeric character reference
  `&#39;` is the form that is correct in all of them. A filter that emitted
  `&apos;` for both targets would produce, for HTML, output that renders
  correctly in current browsers and is not what the target's older
  specifications define — which is the class of "plausible and wrong" that
  `BR-SEM-004` refuses.

  *Closes* `OQ-050`, now listed under [Closed](open-questions.md#closed).
  `escape` is in group 1 and is full contract under `FR-ENV-002`; the
  inherited list of `FR-ENV-018` is untouched.

  *Accepted cost.* A template generating a document type that is neither HTML
  nor XML — a JSON string, a shell argument, a CSV field — gets no filter for
  it. `json` is registered separately, per `FR-ENV-036`, and the other two are
  the project's own business under `BR-ENV-002`.

- **BR-ENV-005**: Silent escaping in a code generator produces `&amp;`
  inside a SQL string and `&lt;` inside a generic type parameter, and the
  author sees it only in the generated file. A rule keyed on the penultimate
  extension would additionally make the same command emit different bytes
  depending on the filename — invisible on the command line, which is precisely
  what was rejected for the `--format` default in `FR-OUT-002`.

  *Rejected.* The penultimate-extension rule covering `.html`, `.htm`, and
  `.xml`; and an explicit `{% autoescape %}` block, which keeps a second mode
  alive for a target this tool does not have.

## Compatibility

- **FR-ENV-029**: Adding a name to group 1 SHALL NOT break the contract.
  Renaming or removing one SHALL, and SHALL be recorded in the project
  changelog.

- **BR-ENV-006**: The compatibility rule of the template surface mirrors that of
  the JSON document in `FR-OUT-014`, for the same reason: a caller may depend on
  what is present and must not depend on what is absent.

## Dependencies

- [render-semantics.md](render-semantics.md) — what happens when a filter, a
  test, or a function is given the wrong thing.
- [context-document.md](context-document.md) — the material the filters and
  tests act on.
- [help-and-version.md](help-and-version.md) — the JSON document in which the
  three groups are published.
- [project-and-discovery.md](project-and-discovery.md) — `FR-PROJ-017`, which
  writes `.tpl/templates/rust/_types.jinja`.

## Open questions

- [OQ-070](open-questions.md#oq-070) — how a backtick inside an identifier is
  escaped, so that `FR-ENV-035` can be satisfied.
- [OQ-072](open-questions.md#oq-072) — the membership of the numeric,
  temporal, and textual type families.

Both are facts about MariaDB that must be observed against the container of
`scripts/mariadb/`, which does not exist. The three design questions this file
carried are closed and listed under [Closed](open-questions.md#closed):
`OQ-049` by `FR-ENV-003`, `OQ-050` by `FR-ENV-044`, and `OQ-071` by
`FR-ENV-039`.
