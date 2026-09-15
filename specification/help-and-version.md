---
title: Help and Version
status: approved
last-reviewed: 2026-09-15
related: [cli-contract.md, global-flags.md, output-formats.md, errors-and-exit-codes.md, template-environment.md]
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
  are enumerated, whether it is repeatable, and any mutual exclusion.

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
  emitted is otherwise the same document: `FR-HELP-017` requires all four keys
  of `data` in both forms, and `FR-HELP-029` states the selection.

- **FR-HELP-017**: The document SHALL be the envelope of `FR-OUT-024`, with
  `source` set to `binary` per `FR-OUT-026`, and a `data` carrying
  `tpl_version`, `global_flags`, `commands`, and `template_surface`, in that
  order:

  ```json
  {"schema_version":1,"source":"binary","data":{"tpl_version":"0.1.0","global_flags":[…],"commands":[…],"template_surface":{…}}}
  ```

  All four keys SHALL be present in every document of this form, whatever path
  argument `FR-HELP-029` reduces `commands` by.

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

- **FR-HELP-022**: `examples` and `exit_codes` SHALL come from a typed table
  indexed by command path, which feeds both the text help and the JSON document.
  The system SHALL NOT derive either by parsing help text.

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
- [template-environment.md](template-environment.md) — `FR-ENV-005`, which owns
  what `data.template_surface` contains.

## Open questions

None specific to this module. `OQ-014` is answered by `FR-HELP-026` through
`FR-HELP-029` and is listed under [Closed](open-questions.md#closed).
