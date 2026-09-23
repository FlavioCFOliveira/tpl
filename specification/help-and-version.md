---
title: Help and Version
status: approved
last-reviewed: 2026-09-23
related: [cli-contract.md, global-flags.md, output-formats.md, errors-and-exit-codes.md, template-environment.md, context-document.md, render-command.md]
---

# Help and Version

## Overview

Help text is one of only three channels a calling agent has for understanding
`tpl`, alongside the exit code and what the command prints. It is therefore
treated as a contract in its own right: fixed structure, fixed width, complete
at every level, and available as a single machine-readable document.

## Scope

In scope: the six help and version forms and their equivalences, the nested
command path, the fixed help layout, the content obligations of each section,
and the JSON command tree — its envelope, the keys of its `data`, the shape of
`data.commands`, and how a subtree of it is selected.

Out of scope: the wording of any individual help text, which is written
alongside the command it documents.

## Actors

- **Calling agent**, for which help text is one of only three channels, and the
  JSON command tree the way the whole surface is loaded in one invocation.
- **Operator**, reading the same text at a shell prompt.

## Help and version forms

- **FR-HELP-001**: The system SHALL provide all six of the following forms:

  | Form | Purpose |
  |---|---|
  | `tpl help` | Top-level help |
  | `tpl --help`, `tpl -h` | Top-level help |
  | `tpl help <command path>` | Help for one command |
  | `tpl <command path> --help`, `tpl <command path> -h` | Help for one command |
  | `tpl version`, `tpl --version`, `tpl -V` | Version |
  | `tpl help --format json`, `tpl help <command path> --format json` | The command tree, or one subtree, as JSON |

  `<command path>` is the full path of a node, of any depth, per `FR-HELP-026`.

- **FR-HELP-002**: The output of `tpl help` SHALL be byte-identical to that of
  `tpl --help` and of `tpl -h`. The output of `tpl help <command path>` SHALL
  be byte-identical to that of `tpl <command path> --help` and of
  `tpl <command path> -h`, at every depth of the tree. The output of
  `tpl version` SHALL be byte-identical to that of `tpl --version` and of
  `tpl -V`.

- **FR-HELP-003**: `-h` SHALL NOT be a shortened or summarised form of
  `--help`. The two produce the same bytes.

  *Rationale.* An agent using `-h` out of habit would otherwise silently lose
  `EXAMPLES`, `EXIT CODES`, types, and defaults.

- **FR-HELP-004**: `help` SHALL be a real subcommand of the tree, listed in the
  top-level help and present in the JSON command tree.

- **FR-HELP-005**: `tpl version` SHALL write exactly `tpl <version>` followed by
  a single newline, and nothing else. For version 0.1.0 the output is exactly
  `tpl 0.1.0\n`.

- **BR-HELP-001**: Each equivalence in `FR-HELP-002` SHALL be guaranteed by a
  byte-for-byte snapshot test, at every depth of the tree.
  `tpl help <command path>` and `tpl <command path> --help` are two distinct
  code paths, and a test must keep them identical.

## Help layout

- **FR-HELP-006**: Every help text SHALL contain the following sections, in this
  order and no other: `USAGE`, `DESCRIPTION`, `ARGUMENTS`, `OPTIONS`,
  `EXAMPLES`, `EXIT CODES`, `SEE ALSO`.

- **FR-HELP-007**: The system SHALL omit an empty section, except `USAGE`,
  `DESCRIPTION`, `EXAMPLES`, and `EXIT CODES`, which SHALL always appear.

  *Rationale.* Printing all seven with `none` under the empty ones would be
  trivially parseable by position, but a simple command's help would grow from
  eight lines to eighteen, against the rule that help consumes the caller's
  context window.

- **FR-HELP-008**: The system SHALL list the children of a node inside
  `ARGUMENTS`, as the node's first positional argument:

  ```
  ARGUMENTS
    <subcommand>   one of:
      info      Database metadata
      tables    List tables
      ...
  ```

  *Rationale.* A subcommand is formally the node's first positional argument, so
  no eighth section is needed. The cost is that `ARGUMENTS` holds two kinds of
  thing, which the JSON command tree separates anyway: there, a child is an
  entry of `data.commands` of its own and is never a member of its parent's
  `arguments`, per `FR-HELP-019`.

- **FR-HELP-009**: Help text SHALL be laid out at a fixed width of 80 columns,
  with line breaks written into the text.

- **FR-HELP-010**: The system SHALL NOT read `COLUMNS`, SHALL NOT query the
  terminal width, and SHALL NOT reflow help text for any reason.

  *Rationale.* The same help must be byte-identical across machines and across
  the four targets of `NFR-PERF-018`.

- **FR-HELP-011**: The `EXIT CODES` section of a command's help SHALL list only
  the codes that command can produce.

- **FR-HELP-012**: Every command's help SHALL end with at least one complete,
  copyable, correct example in `EXAMPLES`.

  *Rationale.* Agents copy examples. One example is worth three paragraphs of
  prose.

- **FR-HELP-013**: For every flag and argument, help SHALL state the type of its
  value, its default, whether it is required, its permitted values where they
  are enumerated, whether it is repeatable, and any mutual exclusion. Those six
  facts describe the **value**; what the flag or the argument does is
  `FR-HELP-030`, and both are stated.

  *Amended in the thirty-first edition.* The last sentence is a
  cross-reference and not a seventh fact. This list was read as the whole of
  what help states about a flag, and a renderer derived from it emitted
  `-d, --database <NAME>` over `Type: string. No default. Optional. Not
  repeatable.` — six facts, every one correct, and nothing saying that the
  flag selects a `[database.<name>]` entry of `.tpl/.cfg`. `FR-HELP-030` adds
  the statement of purpose; nothing about these six changes.

- **FR-HELP-030**: For every flag and every positional argument, help SHALL
  state in one sentence what supplying it does — the object it acts on and the
  effect it has — beside the six facts of `FR-HELP-013`. The sentence SHALL
  name a thing this specification fixes rather than restating the flag's own
  spelling: for `-d/--database`, that it selects the `[database.<name>]` entry
  of `.tpl/.cfg` used by the invocation, per `FR-GLOB-004`.

  The text SHALL live in the typed table of `FR-HELP-022`, indexed by command
  path for a local flag or an argument and by flag name for a global flag,
  which `FR-GLOB-003` lists once at the root. That table feeds both channels,
  so the sentence SHALL appear in the `OPTIONS` and `ARGUMENTS` sections of the
  text help and in the `options` array of `FR-HELP-020`, the `arguments` array
  of `FR-HELP-019`, and `data.global_flags` of `FR-HELP-018`.

  It SHALL apply to the seven global flags of `FR-GLOB-001`, stated once in the
  `OPTIONS` section of `tpl --help` and repeated in no other node's, per
  `FR-GLOB-003`.

  It is **already satisfied** for the children a node lists inside `ARGUMENTS`,
  per `FR-HELP-008`: the one-line summary beside each child — `tables   List
  tables` — is this sentence, and nothing about that listing changes.

  *Rationale.* Help is one of only three channels the primary consumer has, and
  `FR-HELP-014` forbids sending a reader anywhere else for the rest. The six
  facts of `FR-HELP-013` are true of `-d/--database` and of
  `--pattern <PATTERN>` in the same words, so an agent choosing between two
  flags learns from them the shape of a value and nothing about which flag to
  write. A statement of purpose is the one fact that distinguishes them, and it
  is the fact a caller came to the help for.

  *Why the typed table and not the declarations.* The flag declarations cannot
  carry the text. Their doc comments name requirement identifiers in backticks,
  and `FR-HELP-014` bars help from referring to any document outside the help
  system — a `SEE ALSO` may name a `tpl` command and nothing else — so a
  renderer that lifted a doc comment into help would publish
  `FR-GLOB-004`-shaped citations to a caller who cannot resolve them. The
  typed table already exists for exactly this reason: `FR-HELP-022` holds
  `examples` and `exit_codes` there because help text is not a place to derive
  anything from, and a purpose sentence belongs beside them.

  *Weighed against `BR-HELP-002` and the context window.* The cost is one line
  per flag, at the one place that flag is declared. The seven global flags cost
  seven lines in `tpl --help` and nothing anywhere else, because `FR-GLOB-003`
  already forbids repeating them; a local flag costs one line in the single
  command that declares it. That is not the cost `FR-HELP-007`'s rationale
  refuses — a simple command's help growing from eight lines to eighteen for
  four empty section headings — and it does not repeat anything at a level
  where it has already been said, which is what `BR-HELP-002` bars. A caller
  who has to run a second invocation, or guess, because the first help did not
  say what a flag does spends more context than the line would have cost.

  *Rejected: leaving `FR-HELP-013` as the whole of it.* It is what produced the
  renderer that satisfies the requirement and tells the caller nothing, and the
  defect is invisible to every check this corpus can run on itself: the
  requirement was met.

  *Rejected: putting the purpose in the `DESCRIPTION` section as prose.* It is
  not addressable per flag, so the JSON document of `FR-HELP-016` could not
  carry it in `options` or `global_flags`, and the agent that loads the whole
  surface in one call — the caller that document exists for — would have to
  parse a paragraph to recover it. It would also repeat the seven global flags
  in every node's `DESCRIPTION`, which is the multiplication `FR-GLOB-003`
  exists to prevent.

  *Rejected: a sentence for a positional argument alone, leaving flags to the
  six facts.* An argument and a flag are the same problem — `<PATTERN>` says as
  little as `--pattern` — and `FR-HELP-008` makes a subcommand formally a
  positional argument, so the two populations are not separable in the first
  place.

- **FR-HELP-031**: The `DESCRIPTION` section of every leaf SHALL end with four
  statements, in plain words and in this order:

  1. Whether the command connects to a database server: never, always, or only
     on a cache miss, per the requirement that owns the command.
  2. Whether it requires a `[database.<name>]` entry of `.tpl/.cfg`, selected
     by `-d/--database` or by `core.database`, per `FR-GLOB-025`.
  3. Whether it writes files, and if it does, which: `.tpl/.cfg`, the cache
     folder of the selected entry under `.tpl/.cache/`, the artefacts of
     `tpl init`, or a path the invocation names.
  4. What it writes to stdout.

  ```
  Connects to the server only when .tpl/.cache/ does not hold the tables.
  Needs a database entry (-d or core.database). Writes what it read to the
  entry's folder under .tpl/.cache/. Prints the tables, one row each.
  ```

  Each statement SHALL be true of the command as the requirement that owns its
  behaviour states it, and SHALL NOT name that requirement, per `FR-HELP-014`.
  Two statements MAY be joined in one sentence WHERE the order above is kept.
  A statement SHALL NOT be omitted because its answer is no:
  `Does not contact the server.` and `Writes no file.` are statements of this
  requirement. The same four statements SHALL end the `description` of the
  leaf's entry in the JSON command tree, per `FR-HELP-019`.

  *Rationale.* A caller reading one leaf's help must learn from it alone
  whether the command can run where it stands — with no server, with no entry,
  in a read-only checkout — and what it will leave behind. Before this
  requirement those facts were stated in the group help of `tpl schema` and
  `tpl cache` and in no leaf, so `tpl schema tables --help` did not say that it
  needs an entry, connects on a miss, and writes `.tpl/.cache/`; and
  `tpl cache status --help` did not say that it never contacts the server.

  *Weighed against `BR-HELP-002`.* The statements are repeated in every leaf,
  which is the level at which a caller reads them; the group help is the level
  a caller skips. The cost is two to four lines per leaf.

  *Rejected: stating the four facts only where the answer is yes.* The absence
  of a statement would then carry meaning, and a caller cannot tell an absent
  statement from an omitted one.

  *Added in the forty-third edition,* for rmp `#260`, from finding H-10 of the
  audit of rmp `#259`.

- **FR-HELP-033**: The `DESCRIPTION` section of `tpl render` SHALL state,
  before the four statements of `FR-HELP-031`:

  1. Every context variable of `FR-HELP-032`, one line each, with what it holds
     and, for `table`, `view` and `routine`, the flag that binds it.
  2. Every filter, test and function of contract groups 1 and 2 of
     `FR-ENV-001`, grouped as filters, tests and functions, each with its
     `signature` and its `purpose` of `FR-ENV-047`, marked as contract or as
     pinned to the engine version.
  3. That everything else the engine offers works and carries no guarantee,
     per `FR-ENV-004`.

  ```
  A template sees these variables:
    database  The whole database: name, server, tables, views, routines.
    table     The table named by --table. Absent without --table.
    ...
  Filters (contract):
    value | indent(n)       Prefixes every line after the first with n spaces.
    ...
  ```

  The lines SHALL come from the typed table of `FR-HELP-022`, so the text and
  the JSON document state the same signatures and the same sentences. The list
  SHALL appear in no other node's help, per `BR-HELP-002`.

  *Rationale.* `FR-HELP-014` makes help self-contained, and a caller that reads
  text help and never requests JSON had no channel that named a variable or a
  filter signature. `tpl render` is the command that runs a template, so its
  help is where a caller writing one looks.

  *Why inside `DESCRIPTION`.* `FR-HELP-006` admits seven sections and no other.
  The list describes what the command gives the template it runs, which is
  description, and placing it there leaves the layout unchanged.

  *Accepted cost.* The help of `tpl render` grows by about forty lines. It is
  paid in one node, the one a template author reads, and nowhere else.

  *Rejected: an eighth section, `TEMPLATE SURFACE`.* It amends `FR-HELP-006`
  and `FR-HELP-007` for one node, and every parser of the layout would learn a
  section that only one help carries.

  *Added in the forty-third edition,* for rmp `#260`, from findings H-02 and
  H-03 of the audit of rmp `#259`.

- **FR-HELP-014**: Help SHALL be self-contained. It SHALL NOT refer the reader
  to a website, a manual page, a README, or any document outside the help
  system itself. `SEE ALSO` SHALL reference only other `tpl` commands.

- **FR-HELP-015**: Help text SHALL contain no colour, no emoji, and no
  decorative characters.

- **BR-HELP-002**: Help is concise. Short sentences, no marketing prose, and
  nothing repeated at a level where it has already been said.

## The JSON command tree

- **FR-HELP-016**: `tpl help --format json` SHALL emit the entire command tree
  as a single JSON document. `tpl help <command path> --format json` SHALL emit
  the same document with `data.commands` reduced to the subtree rooted at the
  node that path names — that node's own entry and the entry of every
  descendant of it, and no other — at any depth, per `FR-HELP-029`.

  *Amended in the twenty-first edition.* "The subtree rooted at" named a shape
  as well as a selection, and the shape it named is the one `FR-HELP-019`
  contradicts: a subtree is a tree, and every entry carries its full `path`.
  `FR-HELP-019` now settles the shape — `data.commands` is flat — and a subtree
  under it is a selection over that array rather than a nesting of it. What is
  emitted is otherwise the same document: `FR-HELP-017` requires every key
  of `data` in both forms, and `FR-HELP-029` states the selection.

- **FR-HELP-017**: The document SHALL be the envelope of `FR-OUT-024`, with
  `source` set to `binary` per `FR-OUT-026`, and a `data` carrying
  `tpl_version`, `global_flags`, `commands`, `template_surface`, and
  `context_variables`, in that order:

  ```json
  {"schema_version":1,"source":"binary","data":{"tpl_version":"0.1.0","global_flags":[…],"commands":[…],"template_surface":{…},"context_variables":[…]}}
  ```

  All five keys SHALL be present in every document of this form, whatever path
  argument `FR-HELP-029` reduces `commands` by. What `context_variables`
  contains is fixed by `FR-HELP-032`.

  The `data` of this document SHALL be an **open** set of keys. A later edition
  MAY add a key to it, and SHALL place the new key after the last, so that the
  position of every key already present is unchanged.

  *Amended in the third edition.* The first edition put `schema_version` and
  `tpl_version` side by side as the document's first two keys, which made the
  command tree the second of only two JSON documents this specification fixed
  and gave it a header shared with nothing else. The envelope of `FR-OUT-024`
  now governs all seventeen documents, so `tpl_version` moves into `data`
  beside the material it describes, and `source` is `binary` because the
  document is derived from the command tree the binary parses with, per
  `FR-HELP-021`, and nothing is read to produce it.

  *Rejected.* Keeping `tpl_version` in the header as a third envelope key,
  which `FR-OUT-028` forbids: the envelope must not grow a key for the benefit
  of one command.

  *Amended in the twenty-first edition.* `template_surface` is new, and so is
  the statement that `data` is open. `FR-ENV-005` obliges this document to
  enumerate all three groups of the template surface and named no place in it
  for them; three keys listed "in that order", with nothing said beside them,
  read as the whole of what `data` may carry; and `FR-OUT-028` bars a fourth
  key beside `data` while saying nothing about a fourth key inside it. The
  surface is therefore a fourth key of `data`, and this requirement now says
  that `data` admits one — which is what `FR-OUT-027` already leaves to the
  module owning the command, and what `FR-OUT-014` already makes non-breaking.
  Nothing here touches the envelope: it stays closed at three keys, and
  `FR-OUT-028` is undiminished. What `template_surface` contains is fixed by
  `FR-ENV-005`, in [template-environment.md](template-environment.md), which
  owns the surface; this requirement owns only its name and its position.

  *Rejected.* Publishing the surface inside the entry of `tpl render` in
  `data.commands`, which places it beside the one command that uses it and
  needs no new key at all. It was rejected because the surface is not a
  property of one command and must survive the reduction of `FR-HELP-029`: a
  caller reading `tpl help cfg --format json` would lose the surface, and a
  caller reading `tpl help render --format json` would receive filters and
  tests as though `render` declared them, which `FR-HELP-020` reserves for the
  flags a command declares. Also rejected: a document of its own, reached by a
  command of its own, which adds a node to a tree `FR-CLI-002` closes in order
  to publish material `FR-ENV-005` already requires this document to carry.

  *Amended in the forty-third edition.* `context_variables` is new, and is
  placed after the last key, as this requirement's own rule for a new key
  requires. The document named every filter, test and function a template may
  call and none of the variables it reads, so an agent writing its first
  template had to guess `database`, `table`, `vars`, `tpl` and `now`, per
  finding H-02 of the audit of rmp `#259`. Adding a key is not breaking, per
  `FR-OUT-014`.

  *Rejected: a fourth key of `template_surface`.* That value holds the three
  contract groups of `FR-ENV-001`, in three objects of one shape, and a caller
  reads them with one routine. A list of variables is a different shape, and
  its content belongs to [context-document.md](context-document.md) and
  [render-command.md](render-command.md), not to the file that owns the
  surface.

- **FR-HELP-032**: `data.context_variables` SHALL be an array carrying one
  object per top-level context variable of `FR-RND-023` — `database`, `table`,
  `view`, `routine`, `vars`, `tpl`, and `now`, in that order — each carrying
  exactly `name`, `type`, `bound_by`, and `purpose`, in that order:

  | Key | Value |
  |---|---|
  | `name` | The variable's name |
  | `type` | `object` or `string`, as `FR-CTX-001`, `FR-CTX-026`, `FR-CTX-027` and `FR-CTX-028` fix it |
  | `bound_by` | `null` WHERE the variable is present in every render, per `FR-RND-023` and `FR-RND-024`; otherwise the object flag whose presence binds it: `--table`, `--view` or `--routine` |
  | `purpose` | One sentence stating what the variable holds |

  ```json
  {"name":"table","type":"object","bound_by":"--table","purpose":"The table named by --table: its columns, indexes and foreign keys."}
  ```

  Each `purpose` SHALL agree with the requirement that fixes the variable's
  content and SHALL name no requirement, file or document outside the help
  system, per `FR-HELP-014`. The values SHALL come from the typed table of
  `FR-HELP-022`.

  *Rationale.* The variables are the first thing a template reads, and the
  document is where a calling agent loads the whole surface, per `FR-HELP-016`.
  `bound_by` states as data what a template author otherwise learns from a
  render failure: that `table` exists only when `--table` is given.

  *Added in the forty-third edition,* for rmp `#260`.

- **FR-HELP-018**: `data.global_flags` SHALL carry the global flags once, and
  each command SHALL carry `"inherits_globals": true` instead of repeating
  them.

- **FR-HELP-019**: `data.commands` SHALL be a **flat** array carrying one entry
  per command of the tree of `FR-CLI-002` — every node below `tpl`, group nodes
  included — ordered so that each node is followed by its own children before
  the next node at its level, preserving declaration order throughout per
  `FR-HELP-023`. Each entry SHALL carry `path`, `aliases`, `description`,
  `arguments`, `options`, `examples`, and `exit_codes`, and SHALL NOT carry its
  children: a child is reached through its own entry, whose `path` extends the
  parent's.

  *Amended in the twenty-first edition.* The requirement listed the members of
  an entry and left unstated the array those entries sit in, so `path` on every
  entry read as a flat array and "the subtree rooted at" of `FR-HELP-016` read
  as a nested one, with nothing in the corpus to choose between them. Flat is
  chosen, and the two requirements that name a subtree now say what one is
  under it. The `path` every entry already carries **is** the parent-child
  relation written out: a nested array would carry the same relation twice, and
  a caller loading the whole surface — which `FR-HELP-016` says this document is
  for — would have to recurse to reach a leaf it can otherwise reach by index.
  It is also what `FR-HELP-026` assumes, and said so before this amendment: its
  rationale rests on every node carrying its full `path` so that a caller can
  hand the path straight back. One consequence is stated where it is read:
  `arguments` holds a command's own arguments alone, and the children that the
  text help of `FR-HELP-008` lists inside `ARGUMENTS` are entries of this array
  instead, which is what the rationale of `FR-HELP-008` means by separating
  them.

  *Rejected.* Nesting, each entry carrying a `commands` member holding its
  children. It names the tree in the shape the tree has, and `FR-HELP-016` reads
  literally under it without amendment. It was rejected because it obliges every
  consumer to recurse for a relation `path` already states, and because the tree
  is closed at three levels by `FR-CLI-002`, so the nesting buys a structure
  this surface is too small to need.

  *Accepted cost.* A caller that wants the children of one node filters the
  array on `path` rather than reading a member. That filter is exactly the
  operation `FR-HELP-029` states, so the cost is paid once, by the requirement,
  rather than by each caller.

- **FR-HELP-020**: The `options` array of a command SHALL list every local flag
  that command declares, including `--format`, `--pretty`, `--direct`, and
  `--no-cache` where declared.

- **FR-HELP-021**: The system SHALL derive the document at runtime by
  introspecting the command tree it actually parses with.

  *Rationale.* Maintaining the JSON by hand would create two sources for one
  truth, and the test comparing them would end up performing the introspection
  anyway.

- **FR-HELP-022**: `examples`, `exit_codes` and the statement of purpose of
  `FR-HELP-030` SHALL come from a typed table indexed by command path, which
  feeds both the text help and the JSON document. The same table SHALL hold the
  item values of `FR-ENV-047`, indexed by name, and the variable values of
  `FR-HELP-032`, indexed by variable. The system SHALL NOT derive any of these
  by parsing help text.

  *Amended in the forty-third edition.* The signatures and purposes of the
  template surface and the meanings of the context variables join the table,
  for the reason the rest is held there: each is written once and read by the
  text help of `FR-HELP-033` and by the JSON document.

  *Amended in the thirty-first edition.* The purpose sentence joins the two
  this requirement already held, and for the reason they are held here: it is
  written once and read by two channels, and neither channel may be derived
  from the other. `FR-HELP-030` states what the sentence must cover and where
  each kind of entry is indexed — by command path for a local flag or an
  argument, and by flag name for a global flag, which `FR-GLOB-003` lists once
  at the root and which therefore has no command path of its own.

- **FR-HELP-023**: The document SHALL preserve declaration order throughout, and
  the emitting path SHALL NOT use any unordered map.

- **FR-HELP-024**: The document SHALL obey every rule of the JSON output
  contract in [output-formats.md](output-formats.md), including the envelope of
  `FR-OUT-024`, compactness by default, and `--pretty`.

- **BR-HELP-003**: The JSON command tree is a contract and carries three tests
  that are part of it:

  1. Every command, alias, and flag in the tree appears in the document.
  2. Every command has at least one example.
  3. Every example parses through the command parser itself.

  *Rationale.* An example that does not parse is worse than no example.

## Use of help by a group node

- **FR-HELP-025**: WHEN a group node is invoked with no child, per `FR-CLI-007`,
  the system SHALL print exactly the text that `tpl help <node>` would print,
  and SHALL exit `0`.

## The nested command path

- **FR-HELP-026**: `tpl help` SHALL accept the full path of any node of the
  command tree of `FR-CLI-002`, at any depth, as a sequence of positional
  arguments. `tpl help cfg database add` is valid and SHALL be byte-identical
  to `tpl cfg database add --help`.

  ```
  tpl help cfg                    same bytes as  tpl cfg --help
  tpl help cfg database           same bytes as  tpl cfg database --help
  tpl help cfg database add       same bytes as  tpl cfg database add --help
  ```

  *Rationale.* `FR-HELP-002` states the equivalence for "one command", and the
  tree reaches three levels. Accepting only the first level would leave the
  equivalence true of some nodes and false of others, with nothing in the help
  saying which — and `tpl help cfg database add` is exactly the invocation a
  calling agent writes after reading the tree of `FR-HELP-016`, where every
  node carries its full `path`, per `FR-HELP-019`. A caller that has the path
  must be able to hand it back.

  *Closes* `OQ-014`, now listed under [Closed](open-questions.md#closed).

- **FR-HELP-027**: `tpl help` SHALL resolve each segment of the path by the
  rules of the tree: an alias of `FR-CLI-011` SHALL resolve to its canonical
  node, and no segment SHALL be inferred from a prefix, per `FR-CLI-004`.
  `tpl help cfg db add` and `tpl help cfg database add` therefore print the
  same bytes.

- **FR-HELP-028**: IF a segment of the path does not name a child of the node
  the preceding segments resolved to, THEN the system SHALL exit `64`
  (`EX_USAGE`) with a nearest-match suggestion over the children of that node,
  per `FR-ERR-019`. The `cause` SHALL name the segment that failed and the node
  it was looked for under, per `FR-ERR-034`.

  *Rationale.* `64` and not `66`, because a command is not a named object of a
  catalogue: it is part of the invocation, and `FR-CLI-003` already gives `64`
  for a first token that is not a command. Suggesting over the children of the
  node reached, rather than over the whole tree, is what makes the suggestion
  short and right — `tpl help cfg database ad` should propose `add`, not every
  node named `add` anywhere.

- **FR-HELP-029**: `tpl help --format json` SHALL apply `FR-HELP-026` to its
  own argument: `tpl help cfg database --format json` SHALL emit the subtree
  rooted at `tpl cfg database`, per `FR-HELP-016`. That subtree SHALL be
  `data.commands` reduced to the entry whose `path` is the path given, together
  with every entry whose `path` extends that path segment by segment, in the
  order those entries hold in the unreduced document. The system SHALL emit
  every other key of `data` unreduced, per `FR-HELP-017`.

  *Amended in the twenty-first edition.* This requirement named a subtree and
  `FR-HELP-016` named it too, and neither said what one is once `FR-HELP-019`
  settles that `data.commands` is flat. It is a selection over that array, and
  the selection is stated here rather than left to be inferred from `path`. Two
  rules already in force reach it before it applies: a path resolves through an
  alias to its canonical node, per `FR-HELP-027`, so the entry selected is the
  canonical one; and a path that names no node exits `64` before any document
  is emitted, per `FR-HELP-028`.

  *Rejected.* Reducing the whole of `data` to the subtree, dropping
  `tpl_version`, `global_flags`, and `template_surface`. It makes the reduced
  document smaller, and it would make the shape of `data` depend on whether an
  argument was given — leaving a caller unable to read one document the way it
  reads the other, which is the whole ground on which `FR-OUT-024` fixed a
  single envelope.

## Dependencies

- [cli-contract.md](cli-contract.md) — group nodes, and the closed tree the
  document describes.
- [global-flags.md](global-flags.md) — the seven flags listed once under
  `global_flags`.
- [output-formats.md](output-formats.md) — the JSON contract rules the document
  obeys.
- [template-environment.md](template-environment.md) — `FR-ENV-005` and
  `FR-ENV-047`, which own what `data.template_surface` contains.
- [render-command.md](render-command.md) and
  [context-document.md](context-document.md) — `FR-RND-023` and the
  requirements of the context variables, which `FR-HELP-032` publishes.

## Open questions

None specific to this module. `OQ-014` is answered by `FR-HELP-026` through
`FR-HELP-029` and is listed under [Closed](open-questions.md#closed).
