---
title: Schema Commands (First Arm)
status: approved
last-reviewed: 2026-09-21
related: [cli-contract.md, cache-commands.md, output-formats.md, context-document.md, render-command.md]
---

# Schema Commands (First Arm)

## Overview

The first arm reads the structure of a database and presents it. It is the only
arm that talks to a server on its own account, and every one of its subcommands
reads through the catalogue cache.

## Scope

In scope: the eight subcommands, their arguments and flags, the `--pattern`
filter, the shape of the dump document, the shape of the `tpl schema info`
document and how it differs from the dump, and the ordering and format of the
result.

Out of scope: the content of the catalogue itself — which fields a table, view,
or routine carries, how they are read, and how they map onto the render context.
Where a field list is named below it is named as a scope statement, not as a
data model.

## Actors

- **Calling agent** or **operator**, invoking the command.
- **Database entry**, selected by `-d/--database`, naming the server and the
  database to read.

## Command surface

```
tpl schema info                        Database metadata
tpl schema tables    [--pattern <p>]   List tables
tpl schema table     <name>            Full table metadata
tpl schema views     [--pattern <p>]   List views
tpl schema view      <name>            Full view metadata, including its SQL definition
tpl schema routines  [--pattern <p>]   List procedures and functions
tpl schema routine   <name>            Full routine metadata
tpl schema dump                        The whole database as one JSON document
```

- **FR-SCH-001**: `tpl schema` SHALL be a group node, per `FR-CLI-007`.

- **FR-SCH-002**: The system SHALL provide exactly the eight subcommands listed
  above, with the aliases declared in `FR-CLI-011`.

- **FR-SCH-003**: `tpl schema info` SHALL report the metadata of the selected
  database.

- **FR-SCH-004**: `tpl schema tables`, `tpl schema views`, and
  `tpl schema routines` SHALL list the objects of their kind in the selected
  database.

- **FR-SCH-005**: `tpl schema table <name>`, `tpl schema view <name>`, and
  `tpl schema routine <name>` SHALL each take exactly one positional argument
  naming the object.

  *Rationale.* The object is named positionally here because the subcommand
  already carries its type. This is a deliberate asymmetry with `render` and
  `cache`, where the type must come from the flag.

- **FR-SCH-006**: `tpl schema view <name>` SHALL include the SQL definition of
  the view in its result.

  *Provenance.* Root `README.md`; not contradicted by any decision.

- **FR-SCH-007**: `tpl schema routines` SHALL return stored procedures and
  stored functions together, and SHALL state the kind of each object — as a
  column in `text` output and as a field in `json` output.

- **FR-SCH-008**: The system SHALL NOT provide a `--type` flag on
  `tpl schema routines`. Selecting one kind from a listing is done downstream,
  from `--format json`. Wherever a command names one routine — `tpl schema
  routine`, `tpl render --routine`, `tpl cache load --routine`, and
  `tpl cache clean --routine` — the system SHALL accept the qualified forms
  `procedure:<name>` and `function:<name>` as well as the bare name.

  *Amended in the second edition.* The prohibition was written for the listing
  and is unchanged by the amendment. The addition is the disambiguator the
  singular forms lacked: procedures and functions occupy distinct namespaces on
  the server, so `calc_vat` can legally name two objects, and every command that
  names one routine was ambiguous. The cached form of the same rule is
  `FR-CDOC-014`.

  *Rejected.* A `--kind` flag to be supplied only when a name is ambiguous,
  which is easy to forget until the day an ambiguity appears; and resolving a
  bare ambiguous name in favour of the function with a warning on stderr, which
  exits `0` — so a caller checking the code never sees it — and leaves the
  procedure unreachable.

  **The prefix is not the `kind` the document carries.** `FR-CAT-016` fixes
  that field at `PROCEDURE` and `FUNCTION`, in upper case, because it is the
  catalogue's own string; the two prefixes here are lower case. A caller
  composing a qualified name from `kind` folds the case, and nothing else
  separates the two — the kinds are the same two and the spelling is otherwise
  identical. Stated here in the twenty-third edition, with the field, so that
  a reader arriving from either side is told once.

  **The prefix is matched as written, in lower case.** The system SHALL accept
  `procedure:` and `function:` and no other spelling of either. IF the segment
  before the first colon of a token that names one routine, folded over ASCII
  `A-Z` and `a-z` alone as `FR-SCH-014` folds, is `procedure` or `function`
  while the token does not carry it in lower case, THEN the system SHALL exit
  `64` (`EX_USAGE`). The `cause` SHALL name the token as written and the
  spelling expected, per the `64` row of `FR-ERR-034`; the `hint` SHALL carry
  the same invocation with the prefix in lower case, per `FR-ERR-009`, built
  under `FR-ERR-022` and dropped under `FR-ERR-023` where the routine name
  falls outside the character set that requirement applies to it. The token's
  shape is decidable without a server, so the condition is evaluated at step 1
  of `FR-ERR-006` and precedes every catalogue read.

  *Amended in the twenty-fifth edition: the casing of the prefix is stated,
  because the composition the twenty-third edition described produced a token
  this requirement neither admitted nor refused.* That edition fixed `kind` at
  `PROCEDURE` and `FUNCTION` and said that a caller composing a qualified name
  from the field folds the case. `procedure:calc_vat` was admitted in terms and
  `PROCEDURE:calc_vat` — which is what a caller that reads `kind` from the
  document and concatenates produces — was governed by nothing: a reader could
  take it for a prefix this requirement admits in another spelling, or for a
  bare name. Neither reading was available from the text, and the two build
  different programs. The obligation to fold now has a stated consequence, and
  the sentence above is what it always meant.

  *Rejected: matching the prefix case-insensitively, so that every case
  variant is admitted.* No spelling this corpus fixes is stated to be matched
  case-insensitively — not a command or its alias, per `FR-CLI-002`; not a
  flag, per `FR-GLOB-001`; not a TLS mode, per `FR-CONF-013`; not a key of the
  space of `FR-CONF-002` — so one token folded inside a command line that folds
  nothing else is a rule every reader has to memorise and every parser has to
  except. It also widens the set of tokens read as qualified from two spellings
  to every case variant of two words, and each of them shadows a routine that
  could legally carry it as a name: this requirement accepts that shadow for
  two spellings deliberately, and multiplying it buys nothing that one case
  fold, at the one place a qualified name is composed, does not buy.

  *Rejected: leaving the casing unstated, so that `PROCEDURE:calc_vat` falls
  through as a bare name.* It is refused, by `FR-SCH-010`, with `66` and a
  nearest-match suggestion over the routine names that exist — a message
  reporting that an object of that name is absent, over a population that
  cannot contain the name the caller meant, when what is wrong is the spelling
  of a prefix this corpus fixes. That diagnoses the wrong fault, which is what
  `FR-CONF-034` refuses for a misspelled configuration key and what
  `FR-ERR-034` bans a `cause` line for.

  *Accepted cost.* A routine whose own name begins with a case variant of
  `procedure:` or `function:` is not reachable by that name through the four
  commands above. The cost is already accepted for the two lower-case
  spellings, by the amendment that introduced them; this widens it to their
  case variants and to nothing else.

- **FR-SCH-009**: `tpl schema table <name>` SHALL be exhaustive over what the
  catalogue holds for that table: its columns with position, type, nullability,
  default, comment, and generated-column status; its primary key, indexes, and
  foreign keys with their `ON UPDATE` and `ON DELETE` rules; its triggers; its
  `CHECK` constraints; and its engine, collation, and comment.

  *Amended in the seventh edition, because one item on the list does not
  exist.* The requirement named a table's **character set**, and the
  catalogue has no such field: the table catalogue carries a collation and no
  character set, observed on all four series of `FR-SRV-015`. A character set
  is reachable on the database, per `FR-CTX-036`, and on each individual
  column, per `FR-CTX-041` — and not on the table between them.
  Deriving one from the collation's leading segment would be an inference the
  catalogue does not state, and this specification does not write down what
  it has not observed, so the item is removed rather than reconstructed.
  `CHECK` constraints are added in the same amendment: `FR-CAT-015` has
  required them since the second edition and this list, written in the first,
  never named them.

  *Amended in the twenty-sixth edition: two of the three now cite the
  observation that established them.* The engine and the collation were
  required here from the first edition, and no requirement of this corpus said
  which catalogue field either is read from or what that field holds — two
  facts this command must print, fixed nowhere. `FR-CAT-054` records both,
  observed on 2026-09-18 against all four series of `FR-SRV-015`, and
  `FR-CAT-053` names it beside them. The comment's absent value was already
  fixed by `FR-CAT-039`. Nothing this requirement obliges changes: the three
  are as the seventh edition left them.

  *Corrected in the thirty-first edition: the sentence counted three places and
  named two.* It read *The three places a character set is reachable are the
  database, per `FR-CTX-036`, and each individual column, per `FR-CTX-041`*,
  which is a count with nothing under it — a reader looking for the third found
  no requirement of this corpus that gives one. The count is dropped rather
  than a third place found, and the sentence now reads as the same fact reads
  in [catalogue-coverage.md](catalogue-coverage.md), where the twenty-sixth
  edition stated it below the table of `FR-CAT-054` in these words and stated
  no number. Nothing this requirement obliges changes, and the seventh
  edition's amendment above stands in every other word.

  *Rejected: naming the session character set a routine, a routine parameter
  or a trigger carries as the third place.* `FR-CAT-048`, `FR-CAT-049` and
  `FR-CAT-050` each record a client character set the model carries, so a
  literal count of every place the word appears is larger than three and not
  smaller. They are not what this sentence is about: each records the session
  the object was created in, and this sentence is about the chain a character
  set describing **data** runs down — the database, then the table, then the
  column — which is the chain the removed item was wrongly placed on. Writing
  the larger count would answer the arithmetic and lose the point.

  *Rejected: leaving the number and finding a third place to fit it.* There is
  none, and manufacturing one — the collation's leading segment, or the
  schema's default read a second time at table level — is the inference the
  amendment above removed the item rather than make.

- **FR-SCH-010**: IF a named table, view, or routine does not exist in the
  selected database, THEN the system SHALL exit `66` (`EX_NOINPUT`) with a
  nearest-match suggestion over the objects of that kind that do exist. IF a
  bare routine name matches both a procedure and a function, THEN the system
  SHALL exit `64` (`EX_USAGE`), naming both candidates in the qualified form of
  `FR-SCH-008`.

  *Amended in the second edition.* The ambiguity case is new. The system SHALL
  NOT resolve it in favour of either kind under any circumstance: a first-wins
  rule would make one of the two objects permanently unreachable through a bare
  name, and which one it was would depend on the order the catalogue returned
  them.

## The `--pattern` filter

- **FR-SCH-011**: `tpl schema tables`, `tpl schema views`, and
  `tpl schema routines` SHALL each declare an optional `--pattern <p>` flag that
  filters the listing by object name.

- **FR-SCH-012**: `--pattern` SHALL use MariaDB `LIKE` syntax: `%` matches any
  sequence of characters including the empty sequence, `_` matches exactly one
  character, `\%` matches a literal `%`, and `\_` matches a literal `_`.

- **FR-SCH-013**: The system SHALL evaluate `--pattern` locally, in memory,
  against the names it has read. It SHALL NOT send the pattern to the server.

  *Rationale.* Local evaluation is the only way the same pattern selects the
  same set with a database, without one, and against any server. Server-side
  `LIKE` would make the result depend on the collation of the catalogue columns,
  which is invisible on the command line and therefore breaks determinism.
  Catalogue reads are already whole-list reads, so filtering in memory costs
  nothing.

- **FR-SCH-014**: `--pattern` SHALL fold case over ASCII `A-Z` and `a-z` only,
  independently of the server, of the database collation, and of the locale.

  *Accepted cost.* `order%` also matches `Orders`, which a binary server
  collation would not match.

- **FR-SCH-015**: `--pattern` SHALL NOT be declared by any command other than
  `tpl schema tables`, `tpl schema views`, and `tpl schema routines`.

- **BR-SCH-001**: A pattern selects the same set whatever the source of the
  catalogue: a live read, a cached read, or a context file.

## `tpl schema dump`

- **FR-SCH-016**: `tpl schema dump` SHALL emit the whole selected database as a
  single JSON document.

- **FR-SCH-017**: The document SHALL be the envelope of `FR-OUT-024` carrying
  a `data` of one key, `database`:

  ```json
  {"schema_version":1,"source":"server","data":{"database":{…}}}
  ```

  *Amended in the third edition.* The first edition gave the dump the shape
  `{"schema_version":1,"database":{…}}`, which was one of only two JSON
  documents this specification ever fixed and which shared nothing with the
  other fifteen. The envelope of `FR-OUT-024` now governs all seventeen, so the
  dump gains `source` and its `database` key moves inside `data`. Nothing about
  the `database` object itself changed; [context-document.md](context-document.md)
  fixes it, per `FR-CTX-001`.

- **FR-SCH-018**: The document SHALL contain only the server-derived part of the
  render context. It SHALL NOT contain `vars`, `tpl`, or `now`.

  *Rationale.* Three of the five top-level context variables do not come from
  the database. Including `now` would make two dumps of an unchanged database
  differ, and a committed snapshot would produce a diff on every regeneration.

- **FR-SCH-019**: `tpl schema dump` SHALL NOT declare `--format`. Supplying
  `--format` to it is an unknown-flag error under `FR-CLI-019`, exit `64`.

  *Rationale.* A flag with a single permitted value is not a choice, and
  declaring it would suggest an alternative exists. The `DESCRIPTION` section of
  its help states that the output is JSON.

- **FR-SCH-020**: `tpl schema dump` SHALL declare `--pretty`, which SHALL apply
  without any accompanying `--format`, per `FR-OUT-009`.

- **FR-SCH-021**: IF `--pattern` is supplied to `tpl schema dump`, THEN the
  system SHALL exit `64`.

  *Rationale.* A partial context would fail at render time on a missing object,
  and the round-trip contract depends on a dump being complete. A context file
  must be able to say for itself that it is whole.

- **FR-SCH-022**: The document emitted by `tpl schema dump` SHALL be accepted by
  `tpl render --context`, per `FR-RND-016`.

- **BR-SCH-004**: The round-trip of `FR-SCH-022` is a contract and carries a
  mandated test, as the other two contracts of this specification do. The test
  SHALL dump the reference database, feed the dump back through
  `tpl render --context`, and assert that the rendered result is byte-identical
  to the result of the same render against a live read of the same database.

  *Rationale.* This is the only property that makes rendering without a
  database safe to rely on, and it is the one most likely to break silently
  when the document shape changes. The precedents are `BR-ERR-001`, which
  mandates a test per exit code, `BR-HELP-001` and `BR-HELP-003`, which mandate
  four for the help forms and the command tree, and `FR-SRV-012` and
  `FR-SRV-013`, which mandate three for the read-only promise. The round-trip
  was the only contract with none.

  *Accepted cost.* The test needs the container of
  [performance-requirements.md](performance-requirements.md), which now
  exists at all four series of `FR-SRV-015`.

  *Amended in the twenty-sixth edition: the note named a condition that does
  not hold.* It said the test was blocked only by `tpl` not existing. The
  binary exists — the package landed at `d8e7e8a` and the repository carries
  both a library and a binary — so the note named no block at all, on a test
  this rule mandates. What blocks the test is that **neither half of the
  round-trip is implemented**: `tpl schema dump`, per `FR-SCH-016`, and
  `tpl render --context`, per `FR-RND-016`. Both are declared in the command
  tree and neither executes — each raises the violated-invariant `70` of
  `FR-ERR-030` instead — as checked at `90af569`. The two halves belong to
  different arms, the dump to this one and `--context` to the third, so the
  test becomes writable when the later of the two lands and not before. This
  note states something about a file this corpus does not own, so it is
  re-read whenever either command gains an implementation.

  *Rejected: recording that the block is gone.* It is not. The test cannot be
  written against two commands that neither dump nor render, and a rule that
  mandates a test owes its reader a note saying what the test waits on.

  *Rejected: writing the dump half now and asserting it against a stored
  snapshot.* That asserts that the dump has not changed, which is a
  determinism property `NFR-DET-001` already owns. The whole subject of this
  rule is that a render from a dump and a render from a live read agree, and
  neither render can be performed.

  *Amended in the twenty-eighth edition: the count in the precedent list
  follows the requirement it counts.* The clause said **two**, one test each,
  which was true until `FR-SRV-013` was split by the test form that can reach
  each of its two outcomes: its confirming outcome is an integration test
  against every series, its failing outcome an in-process test through the seam
  that requirement authorises, and `FR-SRV-012` is unchanged. Only the count
  changes here, and nothing about this rule or the round-trip it mandates.

## Flags and output

- **FR-SCH-023**: Every `schema` subcommand except `dump` SHALL declare
  `--format <text|json>`, defaulting to `text`, and `--pretty`.

- **FR-SCH-024**: Every `schema` subcommand, `dump` included, SHALL declare
  `--direct` and `--no-cache`, with the meanings defined in
  [cache-commands.md](cache-commands.md).

- **FR-SCH-025**: Every `schema` subcommand SHALL read through the catalogue
  cache, per `FR-CACHE-006`.

- **FR-SCH-026**: In `text` output, a listing SHALL be presented as aligned
  columns under a header row, laid out by the following rule and by no other:

  1. One column per field, in a fixed order, under a header row carrying each
     field's name in upper case. The fields and their order are a property of
     the listing rather than of this rule; for `tpl schema tables` they are
     those shown below.
  2. Each column SHALL be as wide as the widest cell it holds, its header cell
     included, measured in the characters a reader is shown — that is, after
     the escaping of `FR-OUT-018`.
  3. Every cell SHALL be left-aligned and padded on its right with spaces to
     its column's width. A column of numbers SHALL be laid out exactly as any
     other column and SHALL NOT be right-aligned.
  4. Two adjacent columns SHALL be separated by exactly two spaces.
  5. A row SHALL end at its last non-empty cell. That cell SHALL carry neither
     padding nor a separator after it, and no line SHALL carry trailing
     whitespace.
  6. Every line, the header row included, SHALL be terminated by one `\n`.

  Applied to a database holding three tables, the rule yields exactly this:

  ```
  tpl -d shop schema tables

  NAME         ENGINE  COLUMNS  COMMENT
  customers    InnoDB  14       Registered buyers
  order_items  InnoDB  7
  orders       InnoDB  21       One row per order
  ```

  The rows are ordered by name, ascending, byte-wise, per `FR-SCH-028` and
  `NFR-DET-002`, which is why `order_items` precedes `orders`. A listing with
  no rows prints the header row and nothing beneath it, per `FR-OUT-034`.

  *Amended in the second edition.* The listing previously carried a `ROWS`
  column, which is the server's row estimate. The storage engine revises that
  estimate without any change to the structure, so two reads of an unchanged
  database differ — which contradicts `NFR-DET-001` and the argument
  `FR-SCH-018` used to keep `now` out of the dump. `COLUMNS` is a structural
  count and is stable. The general rule is `FR-CAT-024`.

  *Amended in the twenty-fifth edition: the rule is stated, and the listing is
  now the rule applied to its own data.* As it stood, no single layout rule
  reproduced it. Its `NAME` column was twelve characters wide against a widest
  cell of eleven; its `COLUMNS` column was seven wide in the header row and
  eight in the rows beneath it, so the header's `COMMENT` began one column to
  the left of every comment under it; and it right-aligned `COLUMNS`, which no
  requirement of this corpus stated. `FR-OUT-006` fixes that the output is
  aligned columns under a header row and fixes nothing further, and
  `FR-OUT-004` makes it no contract, so nothing else here could settle the
  question — and this is the only worked `text` listing this corpus carries.
  The six clauses above are `FR-OUT-006` made reproducible, and they are stated
  here, beside the listing that demonstrates them.

  *Rejected: right-aligning a column of numbers, which the listing as it stood
  did.* It obliges the layout to carry an alignment per column, and obliges
  this corpus to say of every listing it fixes which of its columns hold
  numbers — a second field list beside each of the ones
  [catalogue-coverage.md](catalogue-coverage.md) already fixes, written for a
  surface `FR-SCH-027` and `FR-OUT-004` declare is not a contract. One rule
  applied to every cell alike is reproducible by a reader who knows nothing
  about what a column holds, which is the whole of what a worked listing is
  for.

  *Rejected: keeping the listing as it stood and stating the rule that
  produces it.* There is none. It would take three — a width per column that
  the data does not determine, an alignment per column, and a header row laid
  out to a different width from the rows beneath it — and none of the three is
  derivable from the values shown, so a reader could not apply any of them to
  a second listing.

- **FR-SCH-027**: The `text` output of any `schema` subcommand is not a
  contract, per `FR-OUT-004`. Anything parsing a listing must use
  `--format json`.

- **FR-SCH-030**: Every `schema` subcommand, `dump` included, SHALL emit its
  `json` output in the envelope of `FR-OUT-024`.

- **FR-SCH-031**: The `data` of `tpl schema info` SHALL be an object carrying
  one key, `database`, whose value carries exactly four members and no others:
  the three metadata fields `FR-CTX-036` fixes — `name`, `charset`, and
  `collation` — and the `server` object `FR-CTX-031` fixes. It SHALL NOT carry
  the collections `tables`, `views`, and `routines` of `FR-CTX-035`.

  ```json
  {"schema_version":1,"source":"server","data":{"database":{"name":"freight","charset":"utf8mb4","collation":"utf8mb4_unicode_520_ci","server":{"version":"11.4.13-MariaDB-ubu2404","series":"11.4","standing":"supported"}}}}
  ```

  **`tpl schema info --format json` and `tpl schema dump` SHALL NOT emit the
  same bytes.** The two commands answer different questions, and this
  requirement is the one that keeps them apart.

  **How this object relates to the one `FR-CTX-001` fixes.** Every member it
  carries is the same member, under the same name and with the same value, that
  the `database` object of the context document carries: a caller reading
  `data.database.name`, `.charset`, `.collation` or `.server` receives the same
  answer from `tpl schema info` and from `tpl schema dump`. This object is that
  object **without** the three collections, and it is the only place this
  corpus emits a reduction of it. `FR-CTX-001`, `FR-CTX-035` and `FR-CTX-036`
  are unchanged: they fix the context document, which is what
  `tpl schema dump` emits under `FR-SCH-034` and what `tpl render --context`
  consumes under `FR-SCH-036`, and `tpl schema info` emits neither.

  The envelope and the `data` key are fixed here, per `FR-SCH-030` and
  `FR-OUT-024`.

  *Amended in the fourth edition.* One field of that object is now fixed:
  `server`, carrying the probed version, the series, and the standing, per
  `FR-CTX-031` and `FR-CTX-034`. It is outside `OQ-024` because it is not a
  catalogue field — it comes from the version probe of `FR-SRV-002` — so
  `BR-CTX-006` could fix it without observing anything.

  *Amended in the fifth edition, and reversed in the twenty-seventh.* That
  edition read three more fields into this requirement — the collections
  `tables`, `views`, and `routines`, per `FR-CTX-035` — on the ground that they
  are outside `OQ-024` because a collection is a structural rule of
  [context-document.md](context-document.md) rather than a catalogue field.
  The ground was sound about `FR-CTX-035`; the step from it to this
  requirement was not. It is recorded rather than deleted, because the reading
  it created stood for twenty-two editions.

  *Amended in the seventh edition, and the gap is closed.* The schema
  catalogue was observed against all four series and returns six columns.
  `FR-CTX-036` takes three of them as the metadata fields and states, field by
  field, why the other three are not carried. `OQ-024` is now listed under
  [Closed](open-questions.md#closed).

  *Amended in the twenty-seventh edition: two commands were emitting the same
  bytes.* Reading this requirement against its own amendments showed that the
  fifth edition's step had made `tpl schema info --format json` emit, byte for
  byte, what `tpl schema dump` emits: the collections are the whole of the
  model, so a `database` object carrying them is the dump. `FR-SCH-002`
  provides eight subcommands and `FR-SCH-016` gives one of them the whole
  database; a second command emitting the same document is not a second
  command, which is the argument `FR-SCH-019` already made in this file about
  a flag with a single permitted value. The reduction is what this requirement
  always described — `tpl schema info` reports **the metadata of the selected
  database**, per `FR-SCH-003` — and the amendment restores that reading with
  the field list stated rather than delegated.

  *Rejected: leaving the collections in and accepting the byte-identity.* It
  costs a calling agent the whole model to ask a database's name and its
  server's standing, on the command whose line in the surface above reads
  *Database metadata*, and it leaves two of the eight subcommands
  indistinguishable to a caller that reads only the bytes. It also makes
  `tpl schema info` the most expensive command of this arm while presenting the
  least, which no reader of that surface would predict.

  *Rejected: giving `tpl schema info` a key of its own beside `database`, or a
  count of each collection.* A count is derivable from the three listings of
  `FR-SCH-032` and from the dump, and inventing a field the model does not
  carry would put a number in the plumbing contract that no requirement of
  [catalogue-coverage.md](catalogue-coverage.md) fixes. The `text` form of this
  command is not a contract, per `FR-SCH-027` and `FR-OUT-004`, and what it
  chooses to show is outside this requirement.

  *Rejected: composing a reduced object under a different key, so that
  `data.database` always means the whole object.* It would cost a caller the
  one property this amendment preserves — that `data.database.name` answers the
  same from either command — and `FR-OUT-031` names the key for the kind in the
  singular, which for a database is `database`.

  *Accepted cost, stated plainly.* Two shapes are emitted under the key
  `database`, distinguished by the command that produced them and by nothing in
  the envelope. A caller that reads `data.database.tables` from
  `tpl schema info` finds no such key, and `FR-SEM-012` would fail a template
  that did — though no template reads this document, because
  `tpl render --context` takes the dump and refuses anything else, per
  `FR-SCH-036`. The cost is one sentence in a caller's notes; the cost of the
  rejected option is the whole model on every metadata query.

- **FR-SCH-032**: The `data` of `tpl schema tables`, `tpl schema views`, and
  `tpl schema routines` SHALL follow `FR-OUT-030`, carrying one key named for
  the collection — `tables`, `views`, or `routines` — whose value is the array
  of its members:

  ```json
  {"schema_version":1,"source":"server","data":{"tables":[…]}}
  ```

  Each member is an object of the kind [catalogue-coverage.md](catalogue-coverage.md)
  defines, shaped as [context-document.md](context-document.md) fixes it.

- **FR-SCH-033**: The `data` of `tpl schema table`, `tpl schema view`, and
  `tpl schema routine` SHALL follow `FR-OUT-031`, carrying one key named for
  the kind — `table`, `view`, or `routine` — whose value is that object:

  ```json
  {"schema_version":1,"source":"cache","data":{"table":{…}}}
  ```

- **FR-SCH-034**: The `data` of `tpl schema dump` SHALL be the object fixed by
  `FR-SCH-017`: one key, `database`, carrying the whole model of the selected
  database.

- **FR-SCH-035**: `source` on a `schema` document SHALL be `server` or `cache`,
  per `FR-OUT-026`, according to which served the read.

- **FR-SCH-036**: A document supplied to `tpl render --context` SHALL carry the
  whole envelope of `FR-OUT-024`, and the system SHALL NOT accept a bare `data`
  object in its place.

  *Rationale.* Accepting both would give the round-trip of `FR-SCH-022` two
  input forms, and a caller that had stripped the envelope would have discarded
  the `source` field that `FR-CDOC-016` makes the signal of what the document
  does not promise.

- **FR-SCH-028**: The system SHALL order tables by name, columns by ordinal
  position, and indexes by name, per `NFR-DET-002`, and every other collection
  it presents by the default rule of that requirement.

- **FR-SCH-029**: The `EXAMPLES` section of `tpl schema tables` SHALL show the
  canonical loop over every table, which uses `--format json`.

  *Rationale.* `--all-tables` no longer exists, so iterating over objects is the
  caller's job and the help must show how.

## Business rules

- **BR-SCH-002**: The first arm is read-only with respect to the database under
  every circumstance. The guarantee is the closed statement list of
  `FR-SRV-006`, which is what prevents a write from being sent at all; the
  read-only session of `FR-SRV-008`, read back and confirmed under
  `FR-SRV-009`, is defence in depth. Failure to establish or confirm that
  session refuses the connection with `78`, per `FR-SRV-010`. There is no flag
  that disables either part, per `FR-SRV-011`.

- **BR-SCH-003**: The first arm is not read-only with respect to the filesystem.
  A cache miss writes to `.tpl/.cache/`, per `FR-CACHE-007`. Only
  `--direct --no-cache` guarantees that no file is touched.

## Dependencies

- [cache-commands.md](cache-commands.md) — read-through behaviour, `--direct`,
  `--no-cache`.
- [output-formats.md](output-formats.md) — `text` and `json` rules, `--pretty`.
- [render-command.md](render-command.md) — the `--context` half of the
  dump round-trip.
- [configuration-model.md](configuration-model.md) — `FR-CONF-041`, which fixes
  which database a read covers, and `FR-CONF-040`, which refuses an entry that
  names no host. Every subcommand of this arm depends on the first.
- [privileges-and-completeness.md](privileges-and-completeness.md) —
  `FR-PRIV-021`, the outcome where the schema catalogue returns no row for that
  database.
- [errors-and-exit-codes.md](errors-and-exit-codes.md) — `64`, `66`, `69`, `77`,
  `78`.

## Open questions

**None.** The three entries this file carried are closed, and the requirements
that answer them live in the two files that own the catalogue rather than in
this one, which is where the corresponding partial answers already were:

| Entry | Closed by | In |
|---|---|---|
| [OQ-009](open-questions.md#closed) | `FR-CAT-039`, with `FR-CAT-036` and `FR-CTX-037` | [catalogue-coverage.md](catalogue-coverage.md), [context-document.md](context-document.md) |
| [OQ-010](open-questions.md#closed) | `FR-CAT-045`, `FR-CAT-048`, `FR-CAT-050`, `FR-CAT-051` | [catalogue-coverage.md](catalogue-coverage.md) |
| [OQ-024](open-questions.md#closed) | `FR-CTX-036` | [context-document.md](context-document.md) |

The field lists behind `FR-SCH-007` and `FR-SCH-009` are therefore frozen, and
`FR-SCH-009` was corrected in the same edition: it named a table character set
the catalogue does not report.
