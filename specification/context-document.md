---
title: The Context Document
status: approved
last-reviewed: 2026-09-22
related: [catalogue-coverage.md, output-formats.md, schema-commands.md, render-command.md, server-contract.md]
---

# The Context Document

## Overview

One document carries the model. `tpl schema dump` emits it, `tpl render
--context` consumes it, and a template sees the same material as the `database`
context variable. This file fixes its structure: how collections are shaped, how
deep a reference is followed, how a column default and a column type are
expressed, and what the document promises about its own consistency.

The document is plumbing contract. Its compatibility rule is `FR-OUT-014`, and
every rule of [output-formats.md](output-formats.md) applies to it.

## Scope

In scope: the structural rules of the document — array shape, the collections
the `database` object carries, reference depth in both directions, the default
discriminant, the decomposition of a column type, the treatment of absence, the
server version and standing the document carries, and the consistency the
document promises; and the content of the three context variables that do not
come from a server.

Out of scope: which objects and fields the document carries, which is
[catalogue-coverage.md](catalogue-coverage.md); the transport rules of JSON,
which are [output-formats.md](output-formats.md); the cache-specific fields,
which are [cache-documents.md](cache-documents.md); and how each context
variable is bound to a source, which is
[render-command.md](render-command.md).

## Actors

- **Calling agent**, parsing the document.
- **Template**, reading the same material through the render context.
- **`tpl` itself**, on the `--context` path, where the document is untrusted
  input.

## Identity of the document

- **FR-CTX-001**: The document defined here SHALL be the server-derived part of
  the render context, per `FR-SCH-018`, and SHALL be the document `tpl schema
  dump` emits under the shape of `FR-SCH-017`.

- **FR-CTX-002**: The document SHALL obey `FR-OUT-011` for its version key,
  `FR-OUT-012` for absent values, `FR-OUT-013` for key order, and `FR-OUT-014`
  for compatibility.

## Collections

- **FR-CTX-003**: Every collection SHALL be a JSON array, ordered by the rules
  of `NFR-DET-002`.

- **FR-CTX-004**: An empty collection SHALL be emitted as `[]`. It SHALL NOT be
  emitted as `null` and SHALL NOT be omitted.

- **FR-CTX-005**: `null` SHALL be reserved for an absent scalar. A consumer may
  therefore test a collection for emptiness without first testing it for
  nullity.

  *Rejected.* Maps keyed by object name, which would give a template direct
  lookup without a helper. They were rejected because ordering would then exist
  only in the emitter and could not be read back from the document, and
  `NFR-DET-002` would have no observable subject.

## Reference depth

- **FR-CTX-006**: A foreign key SHALL embed the table it references, one level
  deep. WHERE a key names no table to reference — its `referenced_table` being
  `null`, per `FR-CAT-056` — the key SHALL still be carried, with its name, its
  columns and its rules, and the place the embedded table occupies SHALL carry
  `null`, per `FR-OUT-012`.

  *Amended in the twenty-seventh edition: the one case with no first hop.* The
  requirement was written over a key that names a table, which is every key the
  fixture holds and every key any server has been observed to return.
  `FR-CAT-056` records that the catalogue **declares** the referenced table name
  nullable, so the model has a shape for a key that names none, and this
  requirement had no reading for it: a reader could carry the key without an
  embedding, carry it with `null`, or drop it, and the three build different
  documents. Nothing about a key that names a table changes.

  *`FR-CTX-005` is not contradicted, and this states why once.* That
  requirement reserves `null` for an absent scalar so that a consumer may test
  a **collection** for emptiness without first testing it for nullity, and
  `FR-CTX-004` is the half that makes an empty collection `[]`. The embedding
  is neither: it is one object at one key, and an absent one is the case
  `FR-OUT-012` governs — emitted as `null` rather than omitted, so that
  `FR-SEM-012` does not fail a template that reads through it.

  *Why this does not weaken `FR-CTX-023`.* A key with no referenced table
  references no object, so there is no object for the document to be missing.
  The other way a reference could dangle — a key naming a table in another
  schema — cannot arise either: `FR-CAT-057` excludes a cross-schema foreign
  key from the model in both directions, so every table a carried key names is
  a table of the database the read covers.

  *Rejected: dropping such a key from the document.* It presents a table with
  one constraint fewer than the server holds, at exit `0`, and the key's name,
  its columns and its two rules are facts the catalogue did return. *Also
  rejected: omitting the embedding key from the object.* `FR-SEM-012` fails a
  render when a template reads a field that is not there, so a template written
  against every other key in the document would fail on this one — which is the
  argument `BR-SRV-008` used to make `standing` unconditional, arriving here
  and answered the same way.

- **FR-CTX-007**: The embedded table SHALL carry its columns, its indexes, and
  its primary key in full.

- **FR-CTX-008**: The `foreign_keys` of an embedded table SHALL hold the names
  of the referenced tables as strings, not as objects.

- **FR-CTX-009**: The cut of `FR-CTX-008` SHALL be one rule applied at the first
  hop, whatever the shape of the reference graph and whichever direction the
  reference is followed in. A cycle `A → B → A` therefore terminates at the
  first hop with a string, and no traversal can fail to terminate.

  *Amended in the fifth edition.* The clause about direction is new. As first
  written this requirement reasoned only about the outgoing direction, because
  that was the only one that embedded. `FR-CTX-010` now embeds the incoming
  direction under the same rule, which creates two further shapes the rule must
  cover, and covers both by being stated once over both directions:

  | Shape | Terminates because |
  |---|---|
  | `A` references `B`, and `B` references `A` | The embedded `A` inside `B`, and the embedded `B` inside `A`, are each at the first hop and are each cut to names |
  | `A` references `B`, so `B` is `referenced_by` `A` | The embedded `B` inside `A`'s `foreign_keys` and the embedded `A` inside `B`'s `referenced_by` are two first hops, not one path of length two |
  | `A` references itself | The embedded `A` is at the first hop and is cut, in both collections |

  The rule is unchanged: **one hop, then names**. What the amendment removes is
  the reading under which an outgoing hop followed by an incoming hop could be
  taken as depth two.

- **BR-CTX-001**: One constant rule was chosen over a rule that adapts to the
  graph because a template author must be able to know, without inspecting the
  database, exactly how deep the object in hand goes.

  *Rejected.* Names only everywhere, with a `table()` lookup function to resolve
  them — the smallest document and a trivial round-trip, but every template that
  wants a referenced column has to perform the lookup itself. Also rejected: a
  summary embedding of name, primary key, and column names only, which grows the
  document by roughly a sixth instead of roughly doubling it, but which stops
  short of exactly the field — the referenced column's type — that a foreign-key
  accessor needs.

  *Accepted cost.* Over a database with 180 foreign keys across 200 tables, the
  embedding roughly doubles the column volume of the document. That cost is
  carried by the peak-memory measurement point and by the `WL-002` scalar of
  [performance-requirements.md](performance-requirements.md).

- **FR-CTX-010**: `referenced_by` SHALL be a collection, per `FR-CAT-013` and
  `FR-CTX-003`, and SHALL embed the **referencing** table one level deep, under
  the rule of `FR-CTX-006`. The embedded table SHALL carry its columns, its
  indexes, and its primary key in full, per `FR-CTX-007`, and the cut of
  `FR-CTX-008` SHALL apply to it at the first hop, per `FR-CTX-009`.

  *Rationale.* The two directions are the same question asked from the two
  ends, and a template that can reach the referenced table's column type
  through `foreign_keys` but only the referencing table's *name* through
  `referenced_by` cannot generate the has-many side of a relation without a
  lookup the belongs-to side never needed. `FR-CAT-013` added the incoming
  direction precisely so that both sides of a relation are reachable;
  embedding one and not the other would deliver half of that. Symmetry is also
  the only form a template author can hold in mind: `BR-CTX-001` chose one
  constant depth over a depth that adapts to the graph, and a depth that
  differs by direction is a depth that adapts.

  *Closes* `OQ-043`, now listed under [Closed](open-questions.md#closed).

  *Accepted cost, and it is the largest single cost in the document.*
  `FR-CTX-006` already roughly doubles the column volume of the document over a
  database with 180 foreign keys across 200 tables, per `BR-CTX-001`. This
  doubles it again: every table that is referenced now carries a full copy of
  every table that references it, in addition to a full copy of every table it
  references. Two consequences follow, and both are recorded rather than
  discovered:

  - The adopted peak-memory figure of `NFR-PERF-014` — `< 32 MiB` over
    `WL-001` — was supplied by the root `CLAUDE.md` before either embedding
    existed. It is the adopted figure most likely to be superseded upward by
    the first real measurement, and `NFR-PERF-019` is what allows that to
    happen without the figure ever having been a limit — as, since
    `BR-PERF-008`, no figure of this corpus is.
  - The `WL-002` scalar of `N` bytes is unvalued for this reason among others,
    and the amendment to `WL-002` states it.

  *Rejected.* Names only, which is what the second edition left in place by not
  composing the two decisions. It is the cheapest document and it makes the
  has-many accessor the one thing a generator cannot write from the object in
  hand. Also rejected: embedding the incoming direction to a *shallower* depth
  than the outgoing one — a name and a primary key — which halves the cost and
  reintroduces exactly the asymmetry a template author would have to remember.

## Column defaults

- **FR-CTX-011**: A column default SHALL be a structure carrying a `kind`
  discriminant, or `null`.

- **FR-CTX-012**: `kind` SHALL be a **three-way** discriminant, and the system
  SHALL emit exactly the following forms, and no others:

  | Case | Emitted |
  |---|---|
  | The column is `NOT NULL` and declares no `DEFAULT` | `null` |
  | The default is a literal | `{"kind":"literal","value":"0"}` |
  | The default is an expression | `{"kind":"expression","value":"current_timestamp()"}` |
  | The column declares `DEFAULT NULL`, **or** is nullable and declares no `DEFAULT` | `{"kind":"null"}` |

  Which catalogue value produces which of the four rows is fixed by
  `FR-CTX-037`.

  *Amended in the seventh edition, because the requirement could not be
  implemented as written.* It required four distinguishable cases and the
  catalogue draws three. Observed on all four series of `FR-SRV-015`: a column
  declaring `DEFAULT NULL` and a nullable column declaring no `DEFAULT` return
  **identical bytes** — the four-character string `NULL`, hex `4E554C4C` — and
  the fixture's `hs_code_override` and `broker_reference` are the two columns
  that prove it. The only value that returns SQL `NULL` is a `NOT NULL` column
  with no default. The case the requirement called *the column has no default*
  in fact split on nullability, with one branch colliding with `DEFAULT NULL`
  and the other alone producing SQL `NULL`.

  *The catalogue is right and the requirement was wrong.* In MariaDB a
  nullable column with no `DEFAULT` clause **is** defaulted to `NULL`; the two
  are one state, and the four-way split drew a distinction the engine does not
  make. The document's bare `null` is now reserved for the one case that
  genuinely has no default at all.

  *Rejected.* Reconstructing the fourth case by combining the default field
  with the nullability field. The information is not in the catalogue under
  any combination of fields, so the reconstruction would be an invention.
  Also rejected: keeping four cases and documenting the collision, which
  leaves in force a requirement no implementation can satisfy.

- **FR-CTX-013**: The `kind` of a column default SHALL be an enumerated field
  taking exactly the three values `literal`, `expression`, and `null`. Adding a
  value to it is not a breaking change, per `FR-OUT-014`.

  *Amended in the seventh edition, and this is a **narrowing of contract
  surface**.* `kind` was enumerated without its values being stated, and
  `FR-CTX-012` published four forms of which two are indistinguishable in the
  catalogue. The three values above are the whole enumeration. A caller that
  branched on a fourth form was branching on a case no read can produce; a
  caller that treats `{"kind":"null"}` as covering both `DEFAULT NULL` and a
  nullable column with no default is correct.

  *Amended in the twenty-third edition: the enumeration names its subject.* It
  read "`kind` SHALL be an enumerated field", and the document carries a second
  field of that name — a routine's, which `FR-CAT-016` fixes at `PROCEDURE` and
  `FUNCTION`. The three values here were always the column default's:
  `FR-CTX-011` and `FR-CTX-012` state that subject above and the section
  heading states it again, so nothing about the field changes. What the
  amendment removes is the reading under which one enumeration governs both
  fields, which would have put this requirement and `FR-CAT-016` in
  contradiction the moment either stated its strings.

- **FR-CTX-037**: The system SHALL classify the value of the column-default
  catalogue field by its observed shape, as follows, and SHALL derive the
  `kind` and the `value` of `FR-CTX-012` from that classification alone:

  | Shape of the catalogue value | Example | `kind` | `value` carries |
  |---|---|---|---|
  | SQL `NULL` | — | the default is `null`, per `FR-CTX-012` | — |
  | Exactly the four characters `NULL` | `NULL` | `null` | — |
  | Begins and ends with a single quote | `'EUR'`, `''`, `'8''6"'` | `literal` | the text between the outer quotes, with each **doubled apostrophe collapsed to one** |
  | A decimal number, unquoted and unwrapped | `0`, `0.0000`, `18.5`, `1.000000` | `literal` | the value unchanged |
  | Begins with `b'` and ends with a single quote | `b'0'`, `b'101'` | `literal` | the value unchanged |
  | Begins with `(` and ends with `)` | `(curdate() + interval 30 day)` | `expression` | the text between the outer parentheses |
  | Ends with `)` and does not begin with `(` | `current_timestamp()`, `current_timestamp(3)` | `expression` | the value unchanged |
  | Anything else | — | `expression` | the value unchanged |

  The rows SHALL be applied in the order written, and the **first row that
  matches** is the one that applies.

  *Observed.* The fixture produced **43 distinct values** of this field over
  301 columns, byte-identical on all four series of `FR-SRV-015`: SQL `NULL`
  on 132 columns, the four-character string `NULL` on 87, and 41 further
  distinct strings falling into the five shapes above. The provenance of each
  shape is known from the DDL that produced it, which is what allows the
  shapes to be classified rather than guessed: `DEFAULT 'EUR'` returns
  `'EUR'`, `DEFAULT 0` returns `0`, `DEFAULT b'101'` returns `b'101'`,
  `DEFAULT CURRENT_TIMESTAMP` returns `current_timestamp()`, and
  `DEFAULT (CURRENT_DATE + INTERVAL 30 DAY)` returns
  `(curdate() + interval 30 day)` — parenthesised, unquoted, and **rewritten
  by the server**, which lower-cases the function names and replaces
  `CURRENT_DATE` with `curdate()`.

  *An apostrophe inside a literal is doubled, and there is no backslash.*
  `container_type.height_ft_in`, whose default is the four characters
  `8'6"`, returns
  `'8''6"'`; the double quote in the same value is left alone. This is the
  same convention the `ENUM` member list uses, per `FR-CTX-039`, and the same
  one `FR-ENV-045` fixes for a backtick inside an identifier.

  *Closes* `OQ-027`, now listed under
  [Closed](open-questions.md#closed).

  *Rationale for the final row.* Every literal shape the catalogue was
  observed to produce is matched positively above, so a value matching none of
  them is one the server wrote as something other than a literal. Classifying
  it as an expression also fails in the safer direction for the one consumer
  that matters: a template that quotes an expression produces SQL that fails
  loudly, where a template that emits a literal unquoted can produce SQL that
  parses and means something else.

  *Residual risk, stated because the document offers no way to recover from
  it.* The rule is derived from 43 observed values, and a value outside that
  population would be classified by the final row. `FR-CTX-012` rejected a
  `default_raw` field carried alongside the structure, so a template has no
  second view of the field to fall back on — unlike an unrecognised **type**,
  where `FR-CTX-018` keeps `column_type` as exactly that safety net. A
  misclassification is therefore a defect to be reported with the DDL that
  produced it, and is the case this requirement is most likely to be amended
  for.

- **BR-CTX-002**: The server distinguishes a string literal from an expression
  by quoting the literal, and the distinction is carried in the shape of the
  field and nowhere else. Emitting the raw string would make every template
  reimplement the same unquoting heuristic, and each one would get a different
  case wrong — an empty string, a quote inside a literal, a function call whose
  name looks like a word.

  *Amended in the seventh edition, because the rule as written was too
  simple.* It read "by quoting the literal, and that distinction is the whole
  content of the field", which is true of a **string** literal and false of
  the field as a whole: a numeric literal is returned unquoted, a bit literal
  in a `b'…'` form, and an unparenthesised function call is an **expression**
  that carries no quotes at all. Quoting separates a string literal from
  everything else; it does not separate literal from expression. The
  argument the rule makes is unaffected and is strengthened by the
  correction — the heuristic a template would have to reimplement is larger
  than one quote test, which is `FR-CTX-037`.

  *Rejected.* The raw server string alone; and the structure with a `default_raw`
  field carried alongside it, which would offer two answers to one question and
  guarantee that some templates read the one that was not intended.

## Column types

- **FR-CTX-014**: A column SHALL carry `column_type`, the type exactly as the
  server writes it.

- **FR-CTX-015**: A column SHALL additionally carry the decomposed parts of its
  type: `data_type`, `precision`, `scale`, `length`, `unsigned`, `charset`,
  `collation`, and `values`. Each part SHALL be read from the catalogue field
  `FR-CTX-040` names for it, and from no other.

- **FR-CTX-038**: `column_type` SHALL be the raw type string exactly as the
  catalogue returns it, and the system SHALL take its grammar to be the
  following, as observed over the fixture's 301 columns and 105 distinct type
  strings, byte-identical on all four series of `FR-SRV-015`:

  | Observed | Example |
  |---|---|
  | Every integer type carries a **display width in parentheses**, including where the DDL declared none | `int(10)`, `mediumint(9)`, `smallint(6)`, `tinyint(1)` |
  | `unsigned` appears **inside this string only**, after the display width, separated by a single space, in lower case | `bigint(20) unsigned` |
  | `data_type` never carries the unsigned attribute: it reads the same for the signed and the unsigned form | `bigint`, `smallint`, `tinyint` |
  | A decimal carries precision and scale | `decimal(12,3)` |
  | Fractional-second precision appears as a parenthesised digit; the zero-precision form carries **no parentheses at all** | `datetime(6)`, `time(3)`, `timestamp(3)`, against `datetime`, `time`, `timestamp` |
  | A bit type carries its width | `bit(1)`, `bit(8)` |
  | The eight geometry types, `inet4`, `inet6` and `uuid` return a string **equal to `data_type`**, with no parentheses and no width | `point`, `multipolygon`, `uuid` |
  | `enum` and `set` carry the whole member list, per `FR-CTX-039` | `enum('EXW','FCA',…)` |
  | A column declared `NUMERIC` is reported as `decimal`, one declared `JSON` as `longtext` per `FR-CAT-038`, and one declared `YEAR` as `year(4)` | — |

  `unsigned` SHALL therefore be derived from `column_type` and SHALL NOT be
  derived from `data_type`, which does not carry it.

  *Closes* `OQ-028`, with `FR-CAT-038`, now listed
  under [Closed](open-questions.md#closed).

  *Rationale for deriving `unsigned` here rather than from a field of its
  own.* The catalogue has no unsigned field. The attribute exists in exactly
  one place, as a suffix of this string, and `FR-CTX-018` already makes the
  raw string the part a reader falls back to — so the one part that can only
  be read from it is read from it, and the requirement says so rather than
  leaving an implementer to discover that no other field carries it.

- **FR-CTX-016**: `values` SHALL carry the member list of an `ENUM` or `SET`
  column, in the order the catalogue states, and SHALL be `null` for every
  other type.

  *Observed, and it makes the two types unlike each other.* A `SET` member can
  never contain a comma — the server rejects the declaration on all four
  series of `FR-SRV-015` — while an `ENUM` member can contain a comma and an
  apostrophe, per `FR-CAT-034`. Splitting the member list of a `SET` on the
  comma is therefore safe, and doing the same to an `ENUM` corrupts it. How
  the catalogue delimits and escapes the members is `FR-CTX-039`.

  *Amended in the seventh edition: the order is now stated.* The member order
  of an `ENUM` is its declaration order and carries the ordinal each member
  is stored as, so it is meaning rather than presentation. Without the clause
  the default rule of `NFR-DET-002` would have sorted the list by name and
  destroyed it. `NFR-DET-002` now names this collection among its exceptions.

- **FR-CTX-039**: The member list of an `ENUM` or `SET` SHALL be read from the
  raw type string under the following rules, and SHALL NOT be obtained by
  splitting that string on the comma:

  | Rule | Observed |
  |---|---|
  | The list begins after `enum(` or `set(` and ends at the final `)` | `enum('INSERT','UPDATE','DELETE')` |
  | Each member is delimited by a **single quote** on each side | — |
  | Members are separated by a comma **between** a closing and an opening quote | — |
  | An apostrophe inside a member is **doubled**, and is never backslash-escaped | `enum('Lloyd''s Register','DNV',…)` |
  | A comma inside a member is a **bare comma**, byte-identical to the separator | `enum('Not regulated','Class 3, Flammable liquids',…)` |
  | Every other byte is passed through unchanged, including multi-byte UTF-8 | `enum('Method 1 — weighbridge',…)` carries the em dash intact |

  The value each member takes in `values` is the text between its delimiting
  quotes with each doubled apostrophe collapsed to one.

  *Observed.* The fixture's 19 `ENUM` columns and one `SET` column were read
  as bytes on all four series and the four series returned byte-identical
  output. `vessel.class_society` begins `656E756D28` (`enum(`), `27` (`'`),
  `4C6C6F7964` (`Lloyd`), then **`2727`** — two apostrophe bytes — then `73`
  (`s`); there is no backslash byte, `5C`, anywhere in the value.
  `cargo_item.imdg_class` contains `'Class 3, Flammable liquids'`, in which
  the comma is the byte `2C`, identical to the `2C` that separates one member
  from the next, and only the surrounding quotes tell them apart. The `SET`
  shows the same doubling.

  *Rationale, and it is the failure this requirement exists to prevent.* The
  separator and a member's own content are the same byte, so nothing but the
  quote state distinguishes them. A reader that splits on the comma turns the
  fixture's four-member `imdg_class` into seven members, three of which are
  fragments, and reports it at exit `0`. `FR-CAT-034` establishes that a
  `SET` cannot reach this state and an `ENUM` can, which is exactly why one
  rule must be written for both: a reader that learns the safe rule from the
  `SET` and applies it to the `ENUM` corrupts the `ENUM` silently.

  *Closes* `OQ-029`, with `FR-CAT-034`, now listed
  under [Closed](open-questions.md#closed).

- **FR-CTX-017**: A part that does not apply to a type SHALL be `null`, per
  `FR-CTX-005`.

  *What "does not apply" means is fixed by `FR-CTX-040`*, which records which
  parts the catalogue populates for each type. A part is `null` in the
  document exactly WHEN the catalogue returns SQL `NULL` for the field behind
  it; the system SHALL NOT decide applicability on its own account.

- **FR-CTX-018**: IF the system does not recognise a type, THEN it SHALL emit
  `column_type` unchanged and SHALL emit every decomposed part as `null`.

  *Rationale.* The raw string is the safety net. A type introduced by a later
  server reaches the template intact rather than being lost or guessed at, and a
  template can always fall back to matching on `column_type`.

  *Rejected.* The raw string alone, which would force the tests of
  [template-environment.md](template-environment.md) to classify a column by
  parsing a string; and the decomposition alone, which would erase an
  unrecognised type entirely.

- **FR-CTX-040**: The parts of `FR-CTX-015` SHALL be read from the following
  catalogue fields, and a part SHALL be `null` exactly WHEN the field behind
  it returns SQL `NULL`:

  | Part | Catalogue field |
  |---|---|
  | `data_type` | the data-type field |
  | `precision` | the numeric-precision field, or the datetime-precision field where that is the one the catalogue populates |
  | `scale` | the numeric-scale field |
  | `length` | the character-maximum-length field |
  | `unsigned` | derived from `column_type`, per `FR-CTX-038` |
  | `charset`, `collation` | the character-set and collation fields, per `FR-CTX-041` |
  | `values` | derived from `column_type`, per `FR-CTX-039` |

  The two precision fields SHALL be read into one part because **no type
  populates both**: over the fixture's 39 distinct data types, every type that
  returns a numeric precision returns SQL `NULL` for the datetime precision
  and every type that returns a datetime precision returns SQL `NULL` for the
  numeric one. The merge is therefore lossless, and it is what keeps
  `datetime(6)`'s `6` reachable through a part list that names `precision`
  once.

  Which parts each type family populates, as observed on all four series:

  | Type family, as `data_type` | `length` | `precision` | `scale` |
  |---|---|---|---|
  | `bigint` `int` `mediumint` `smallint` `tinyint` | `null` | set | `0` |
  | `decimal` | `null` | set | set |
  | `float` | `null` | `12` | `null` |
  | `double` | `null` | `22` | `null` |
  | `bit` | `null` | set — the bit width | `null` |
  | `datetime` `time` `timestamp` | `null` | set, and **`0` where the type has no fractional part** | `null` |
  | `date` `year` | `null` | `null` | `null` |
  | `char` `varchar` `text` `tinytext` `mediumtext` `longtext` `enum` `set` | set | `null` | `null` |
  | `binary` `varbinary` `blob` `tinyblob` `mediumblob` `longblob` | set | `null` | `null` |
  | the eight geometry types, `inet4`, `inet6`, `uuid` | `null` | `null` | `null` |

  *Closes* `OQ-030`, now listed under
  [Closed](open-questions.md#closed).

  *Three values in that table were written by the server and not by any
  author, and a template will read them as though they were declared.* A
  `FLOAT` reports a precision of `12` and a `DOUBLE` a precision of `22`,
  neither of which appears in the DDL; and a `DATETIME` declared with no
  fractional part reports a precision of `0` rather than `null`, so a template
  testing `precision` for absence must test the `date` row's `null` and not a
  falsy zero. `FR-CTX-017` is satisfied in every case, because the part is
  `null` exactly where the catalogue is.

  *The character octet length is not a part.* `FR-CTX-015` names eight parts
  and the catalogue offers a ninth size field, the octet length, which for a
  textual column is the character maximum length multiplied by the maximum
  bytes per character of that column's own character set — `varchar(255)`
  reads `255 / 765` under `utf8mb3` and `255 / 1020` under `utf8mb4`, and an
  `ascii` column reads the two equal. It is derivable from `length` and
  `charset` and is not carried.

  *For `enum` and `set`, `length` is the length of the longest member*, not
  the length of the type string: the fixture's four-member `imdg_class` reads
  `38`.

- **FR-CTX-041**: `charset` and `collation` SHALL be read from the catalogue's
  character-set and collation fields for the column, and SHALL be `null`
  together WHERE those fields return SQL `NULL`.

  *Observed, over all 301 columns on all four series.* The two fields are set
  together or SQL `NULL` together and are **never split**; **neither is ever
  the empty string**. They are set for exactly eight data types — `char`,
  `varchar`, `text`, `tinytext`, `mediumtext`, `longtext`, `enum` and `set` —
  and SQL `NULL` for every other type in the fixture, the binary and blob
  types included.

  *An inherited value is indistinguishable from a declared one.* A column
  that declares no character set of its own reports the table's default
  **explicitly** — `customer.legal_name` reads `utf8mb4` /
  `utf8mb4_unicode_520_ci`, which is exactly what the table declares — rather
  than `null` or the empty string. The model therefore cannot say whether a
  column's collation was written on the column or inherited from the table,
  and does not claim to.

  *Closes* `OQ-031`, now listed under
  [Closed](open-questions.md#closed).

  *These two values are passed through verbatim, per `FR-SRV-039`*, like every
  other character-set and collation value the model carries. They were
  observed to be byte-identical on all four series, because they follow the
  declared collation of the column or of its table rather than the server's
  own default — unlike the session collation recorded against a view, a
  routine or a trigger, which does differ.

## Column identity and derived facts

- **FR-CTX-019**: Every column SHALL carry `table_name`, naming the table it
  belongs to.

  *Rationale.* A column reached through a loop, through an index, or through a
  foreign key must be able to find its own table without the template having
  carried a reference to it.

- **FR-CTX-020**: A column SHALL carry the static attributes the catalogue
  states for it, including whether it is auto-incremental, whether it is
  generated, and whether it is invisible, per `FR-CAT-009` and `FR-CAT-027`.

- **FR-CTX-021**: The document SHALL NOT materialise on a column any fact the
  table already states. `is_primary_key` and `is_unique` SHALL NOT be fields.

- **FR-CTX-022**: The facts of `FR-CTX-021` SHALL be reachable through the tests
  `primary_key` and `unique`, which resolve `table_name` against the render
  context, per `FR-ENV-015`.

- **BR-CTX-003**: Calculating rather than materialising keeps one statement of
  each fact. A column that says it is not part of the primary key while the
  table says it is would be a document that contradicts itself, and nothing in
  the emitter could prevent it once both are written.

  *Rejected.* Passing the table to the test as an argument,
  `col is primary_key(table)`, which is pure and needs no context lookup, but
  which would leave `tpl` with two grammatical classes of test and a template
  author guessing which class a given test belongs to.

## The server version

- **FR-CTX-031**: The `database` object SHALL carry a `server` object holding
  exactly three keys, `version`, `series`, and `standing`:

  ```json
  {"server":{"version":"11.4.5-MariaDB","series":"11.4","standing":"supported"}}
  ```

  `version` SHALL be the string the probe of `FR-SRV-002` returns, unaltered,
  whose observed form is fixed by `FR-SRV-040`. `series` SHALL be the series
  identifier — the major family and the series number joined by a dot —
  derived from `version` as `FR-SRV-040` requires. `standing` SHALL be as
  `FR-CTX-034` fixes it.

  *Amended in the seventh edition.* The example above shows
  `11.4.5-MariaDB`, which is the form the fourth edition assumed. The four
  fixture servers were observed to return a **further suffix** —
  `11.4.13-MariaDB-ubu2404` — and `FR-SRV-040` records that the suffix is a
  property of the build rather than of the series. `version` carries it
  unaltered and `series` ignores it.

- **FR-CTX-032**: `series` SHALL be a field of its own. A template SHALL NOT be
  required to derive it by parsing `version`.

  *Rationale.* It could not. The inherited filter list of `FR-ENV-018` is closed
  by `FR-ENV-019` and contains nothing that splits a string, so a template given
  `11.4.5-MariaDB` alone has no way to reach `11.4`. The series is the field a
  template branches on, per `BR-SRV-007`, and a field that cannot be read is not
  exposed.

- **FR-CTX-033**: WHEN the document is produced by a server read, `series` SHALL
  be one of the series of `FR-SRV-015`, or newer than every one of them under
  `FR-SRV-031`. WHEN the document is supplied to `tpl render --context`, the
  system SHALL NOT validate `series` against `FR-SRV-015`, SHALL NOT check
  `standing` against `series`, and SHALL require only that all three keys of
  `FR-CTX-031` are present, are strings, and that `standing` holds one of the
  values of `FR-CTX-034`.

  *Rationale.* The refusal of `FR-SRV-020` is about a server `tpl` would have to
  read and cannot vouch for. `FR-RND-022` opens no connection on the `--context`
  path, so there is no such server: the document is a record of a read that
  already happened, and `FR-CTX-025` already establishes that such a record may
  carry a weaker promise than a live read. Validating it against the window would
  also make every committed dump expire on a calendar date as the window moved,
  which would break a working pipeline for a reason nothing in the pipeline
  changed.

  *Rejected.* Applying the window to `--context` as well, for symmetry with the
  server path. It is symmetry bought with a time bomb, and the asymmetry it
  removes is not real: one case is a server being read, the other is bytes
  already on disk.

  *Accepted cost.* A dump taken from a server the window has since left still
  renders. A template written against a newer model may generate wrongly from it,
  and nothing in the document warns of that beyond the `series` field itself,
  which a template can read.

- **FR-CTX-034**: `standing` SHALL be an enumerated field stating the server's
  standing relative to the supported window of `FR-SRV-001`, and SHALL take
  exactly the following values:

  | Value | Meaning |
  |---|---|
  | `supported` | The series is one of `FR-SRV-015` |
  | `newer_than_supported` | The series is newer than every series of `FR-SRV-015`, and the document was produced under `FR-SRV-031` |

  It SHALL be present in every document, whatever the series. It SHALL NOT be
  omitted, SHALL NOT be `null`, and SHALL NOT be conditional on the server.

  *Rationale.* Two values and no third, because a series below the window never
  reaches a document: `FR-SRV-020` refuses it before the catalogue is read. The
  field is enumerated rather than boolean for the reason `FR-CDOC-010` gives for
  `source` — `FR-OUT-014` lets an enumerated field gain a value and does not let
  a boolean gain a state — and it is always present for the reason `BR-SRV-008`
  gives, which is the one that decides the question: `FR-SEM-012` fails the
  render when a template reads a field that is not there, so a marker that
  appeared only when something was wrong would make the guard that looks for it
  fail on every server where nothing was.

  *Rejected.* A boolean `supported`, which cannot later say *how* a server sits
  outside the window; and omitting the field on a supported server, which would
  break `FR-SRV-005`, `FR-OUT-012`, and every template that tested it.

- **FR-CTX-035**: The `database` object SHALL carry the collections `tables`,
  `views`, and `routines`, each a JSON array per `FR-CTX-003`, each ordered by
  `NFR-DET-002`, and each emitted as `[]` when empty per `FR-CTX-004`. They
  stand alongside `server`, per `FR-CTX-031`.

  *Rationale.* These are not catalogue fields and were never blocked by the
  absence of an observation. They are the three collections
  [catalogue-coverage.md](catalogue-coverage.md) covers — `FR-CAT-001`,
  `FR-CAT-007`, and `FR-CAT-008` — and every command of the first arm already
  presents one of them: `FR-SCH-032` names `tables`, `views`, and `routines` as
  the `data` keys of the three listings. What was missing was the statement
  that the `database` object of a dump carries the same three under the same
  names, which is the whole of what makes `tpl schema dump` the round-trip
  partner of `tpl render --context` under `FR-SCH-022`. Nothing about what a
  member of one of those arrays contains is settled here; that is
  [catalogue-coverage.md](catalogue-coverage.md), which fixes it.

  *Corrected in the thirty-first edition: the second half of the last clause
  named nothing.* It read *that is [catalogue-coverage.md] and the open
  questions it carries*, which was true when it was written and stopped being
  true in the seventh edition: that file's ten entries were all closed by the
  observation pass of 2026-09-10, and its index has been empty since. A
  reader following the clause arrived at a heading reading **None**, and read
  it as a gap where the member shape is unsettled — which is the opposite of
  what that file now holds. The clause names the file alone, because the file
  alone is where the answer is.

  *Rejected: keeping the clause and qualifying it — "and any open question it
  carries".* A pointer that hedges whether its target has anything at the end
  of it is a pointer the reader has to check before following, and the check is
  the same one this correction made once. Also rejected: dropping the whole
  sentence. Its first half is the boundary this requirement needs — the
  collections are fixed here and their members are not — and losing it would
  leave a reader of `FR-CTX-035` with no statement of where the member shape
  is fixed at all.

  *Narrowed* `OQ-024` to the **metadata fields** of the `database` object,
  which `FR-CTX-036` now fixes.

  *One command emits a reduction of this object, and it is named here so that
  a reader arriving from this file is told once.* `FR-SCH-031` gives
  `tpl schema info` a `database` value carrying the three metadata fields of
  `FR-CTX-036` and the `server` object of `FR-CTX-031`, and **not** these three
  collections. It is that object with three members removed: every member it
  does carry is this member, under this name, with this value. Nothing about
  this requirement changes — it fixes the document `tpl schema dump` emits and
  `tpl render --context` consumes, per `FR-CTX-001`, and `tpl schema info`
  emits neither.

- **FR-CTX-036**: The `database` object SHALL carry exactly three metadata
  fields beside the `server` object of `FR-CTX-031` and the three collections
  of `FR-CTX-035`:

  | Field | Read from | Observed for the fixture |
  |---|---|---|
  | `name` | the schema-name field | `freight` |
  | `charset` | the schema's default character-set field | `utf8mb4` |
  | `collation` | the schema's default collation field | `utf8mb4_unicode_520_ci` |

  ```json
  {"database":{"name":"freight","charset":"utf8mb4","collation":"utf8mb4_unicode_520_ci","server":{…},"tables":[…],"views":[…],"routines":[…]}}
  ```

  *Observed.* The schema catalogue table returns **six columns and no more**,
  identically on all four series of `FR-SRV-015`. Three of them become the
  fields above. Of the other three: the catalogue-name field reads `def` for
  every schema on every series and locates the row rather than describing the
  database, per `BR-CAT-005`; the SQL-path field is SQL `NULL` for every
  schema on every series, so nothing has been observed for the model to
  carry; and the schema-comment field is the empty string for every schema on
  every series, and the fixture declares no schema comment, so whether an
  authored one reaches it has **not been observed** and admitting the field
  would need an observation rather than a decision.

  *Closes* `OQ-024`, now listed under
  [Closed](open-questions.md#closed). The root `README.md` named four fields,
  `name, version, charset, collation`; three are confirmed and `version` is
  superseded by the `server` object of `FR-CTX-031`, which carries three keys
  where the root document expected one string. The correction owed to that
  document is `DIV-044`.

  *These two values are passed through verbatim, per `FR-SRV-039`.* The
  `freight` row of the fixture reads identically on all four series because
  the fixture declares its collation explicitly; the `mysql` schema of the
  same servers, which declares none, reads `utf8mb4_general_ci` on `10.11`
  and `utf8mb4_uca1400_ai_ci` on the other three. A database created without
  an explicit collation therefore yields a different `collation` from two
  supported servers, and `FR-SRV-026` excepts it for that reason. Recorded as
  difference 11 of `FR-SRV-038`.

- **BR-CTX-006**: `server` was fixed two editions before the metadata fields
  of the `database` object beside it, and it could be fixed early for a reason
  particular to it: it does not come from the catalogue. It comes from the version probe of `FR-SRV-002`,
  which is a statement of its own in the closed list of `FR-SRV-006`, so fixing
  its shape asserts nothing about what a catalogue table returns and was never
  blocked by the absence of an observation.

  The same is true, for a different reason, of the three collections
  `FR-CTX-035` fixes: a collection is a structural rule of this file, and its
  name, its array shape, and its order are settled without asserting anything
  about what a catalogue table returns.

  An object rather than a bare string, for the reason `FR-CTX-027` gives for
  `tpl`: `FR-OUT-014` makes gaining a field the only non-breaking way to grow.
  A cache-served document carries the `version`, `series`, and `standing` of the
  server the read was made against rather than of any server reachable now,
  which is what the `source` field of `FR-CDOC-009` already signals about every
  other field in it. A cached document therefore keeps
  `standing: "newer_than_supported"` for as long as it is served, even from a
  binary whose window has since caught up — the field describes the read, and
  `tpl cache clean` is how a caller discards it.

  Because `server` is a structural rule of this file, a `--context` document that
  omits it does not match the document contract and is `65` under `FR-RND-020`.
  That is intended, and it costs a hand-written document three keys: the
  document is a record of a read against a server, and `FR-SCH-022` makes the
  ordinary way to obtain one a `tpl schema dump`, which always carries them.

## The non-server variables

`FR-RND-023` names five top-level context variables and `FR-RND-024` requires
three of them always to be injected. The two that come from a server are
everything above; these are the other three. They are in this file because a
template reads all five through one context and needs one place that says what
each holds, and not because they appear in the document `tpl schema dump`
emits — `FR-SCH-018` keeps them out of it.

- **FR-CTX-026**: `vars` SHALL be a JSON object whose keys are the `--set` keys
  of the invocation, per `FR-RND-008`, and whose values are strings, per
  `FR-RND-015`. WHEN no `--set` is supplied, `vars` SHALL be `{}`.

- **FR-CTX-027**: `tpl` SHALL be a JSON object carrying exactly one key,
  `version`, whose value is the version string `FR-HELP-005` prints:

  ```json
  {"version":"0.1.0"}
  ```

  *Rationale.* An object rather than the bare string, so that the variable can
  gain a field without breaking a template, which `FR-OUT-014` makes the only
  non-breaking way to grow. `{{ tpl.version }}` in a generated file header is
  the case it exists for.

- **FR-CTX-028**: `now` SHALL be a string carrying the render time as an
  RFC 3339 timestamp in UTC, with a `Z` offset and second precision:

  ```
  2026-09-10T08:14:22Z
  ```

  *Rationale.* A string in a fixed format is the only form a template can use
  today. `FR-ENV-018` guarantees no date filter, and `FR-ENV-025` forbids a
  function that reads a clock, so a structured `now` would give a template
  parts it has no way to format and no way to reassemble. UTC with a fixed
  precision also means the one documented source of non-reproducibility varies
  in exactly one way — the instant — rather than also with the machine's zone
  and locale.

  *Rejected.* A structured value carrying named parts, which needs a formatting
  filter this specification does not provide; and the engine's own datetime
  type with a date filter added to `FR-ENV-018`, which would put a filter into
  group 2 for the sake of one variable and tie its behaviour to the pinned
  engine version of `FR-ENV-003`.

- **FR-CTX-029**: `now` SHALL be evaluated once per invocation, at render time,
  and every reference to it in one render SHALL yield the same value.

  *Rationale.* A template that interpolates `now` twice must not produce two
  timestamps, and `FR-RND-002` gives one render per invocation, so there is one
  instant to record.

- **FR-CTX-030**: `now` SHALL be the single documented source of
  non-reproducibility, per `NFR-DET-005`. A template that does not reference it
  SHALL produce byte-identical output between runs against unchanged inputs.

- **BR-CTX-005**: These three are not part of the document this file otherwise
  fixes, and a `--context` document that carries any of them has its value
  ignored, per `FR-RND-024`. They are specified here because a template author
  reading the context needs one file that answers "what is in each of the five
  variables", and splitting three of the five into `render-command.md` would
  have answered where each *comes from* without ever saying what it *holds*.

## Consistency

- **FR-CTX-023**: WHEN the document is produced by a server read, every object
  referenced from another object in it SHALL be present in it.

  *Amended in the twenty-seventh edition: the two requirements that make this
  satisfiable are named.* The promise was stated and nothing said what keeps
  it. Two things do, and both are outside this file. `FR-CAT-057` excludes a
  foreign key that crosses a schema boundary, in either direction, so no
  carried key names a table the read does not cover; and `FR-CTX-006` as
  amended carries a key that names no table at all without an embedding, so
  there is no reference for that case either. A third is inside this file:
  `FR-CTX-008` cuts an embedded table's own keys to names, so the population a
  reference can point into is the document's own `tables` collection and
  nothing deeper. Nothing this requirement promises changes.

- **FR-CTX-024**: The document SHALL NOT promise to be a point-in-time snapshot,
  and the specification SHALL state that it is not one.

  *Rationale.* A snapshot of the catalogue is not attainable through the
  catalogue tables under any transaction mode the server offers. Promising one
  would be a promise `tpl` cannot keep, and a caller that believed it would
  build on a guarantee that does not exist.

- **FR-CTX-025**: WHEN the document is served wholly or partly from the cache,
  it SHALL promise neither the referential integrity of `FR-CTX-023` nor a
  snapshot. The `source` field of `FR-CDOC-009` is the signal, and
  [cache-documents.md](cache-documents.md) governs.

- **BR-CTX-004**: The three levels are distinct and are stated separately
  because a caller's next step differs between them: referential integrity means
  a traversal will not dangle, a snapshot would mean two objects were read at
  one instant, and a cache-served document means neither is claimed.

## Dependencies

- [catalogue-coverage.md](catalogue-coverage.md) — the objects and fields this
  document carries.
- [output-formats.md](output-formats.md) — the JSON rules the document obeys.
- [schema-commands.md](schema-commands.md) — `tpl schema dump`, which emits it.
- [render-command.md](render-command.md) — `--context`, which consumes it, and
  `FR-RND-020`, the code for a document that does not match this contract.
- [cache-documents.md](cache-documents.md) — the fields a cached read adds.
- [server-contract.md](server-contract.md) — `FR-SRV-002`, the probe whose
  result `FR-CTX-031` carries; `FR-SRV-015`, the series `series` may hold on the
  server path; and `FR-SRV-028`, which requires the version to reach this
  document at all.

## Open questions

**None.** The six entries this file carried are closed, five of them by
requirements written here and one by the observation recorded in
[server-contract.md](server-contract.md):

| Entry | Closed by |
|---|---|
| [OQ-024](open-questions.md#closed) | `FR-CTX-036`, with `FR-CTX-031` and `FR-CTX-035` |
| [OQ-027](open-questions.md#closed) | `FR-CTX-037`, with `FR-CTX-012` and `FR-CTX-013` as amended |
| [OQ-028](open-questions.md#closed) | `FR-CTX-038`, with `FR-CAT-038` |
| [OQ-029](open-questions.md#closed) | `FR-CTX-039`, with `FR-CAT-034` |
| [OQ-030](open-questions.md#closed) | `FR-CTX-040` |
| [OQ-031](open-questions.md#closed) | `FR-CTX-041` |

[OQ-042](open-questions.md#closed) reached this file through one field and is
closed too. `FR-SRV-040` fixes the form of the version string and the
derivation of `series`, which is what `FR-CTX-031` needed. Its second half —
how a server reporting a MariaDB-compatible version string is separated from
a real one — closed on a **stated limit** in `FR-SRV-041`: a server
determined to pass as MariaDB will pass. That limit belongs to `FR-SRV-003`
and not to this document, and it does not qualify any field here; `version`
and `series` carry what the server reported, which is what `FR-CTX-031` says
they carry.

`OQ-043` is answered by `FR-CTX-010` and was closed in the fifth edition.
