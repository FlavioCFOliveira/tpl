---
title: Help and Version
status: approved
last-reviewed: 2026-09-10
related: [cli-contract.md, global-flags.md, output-formats.md, errors-and-exit-codes.md]
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
and the JSON command tree.

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
  thing, which the JSON command tree separates anyway.

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
  the subtree rooted at the node that path names, at any depth, per
  `FR-HELP-029`.

- **FR-HELP-017**: The document SHALL be the envelope of `FR-OUT-024`, with
  `source` set to `binary` per `FR-OUT-026`, and a `data` carrying
  `tpl_version`, `global_flags`, and `commands`, in that order:

  ```json
  {"schema_version":1,"source":"binary","data":{"tpl_version":"0.1.0","global_flags":[…],"commands":[…]}}
  ```

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

- **FR-HELP-018**: `data.global_flags` SHALL carry the global flags once, and
  each command SHALL carry `"inherits_globals": true` instead of repeating
  them.

- **FR-HELP-019**: Each command in `data.commands` SHALL carry `path`,
  `aliases`, `description`, `arguments`, `options`, `examples`, and
  `exit_codes`.

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
  rooted at `tpl cfg database`, per `FR-HELP-016`.

## Dependencies

- [cli-contract.md](cli-contract.md) — group nodes, and the closed tree the
  document describes.
- [global-flags.md](global-flags.md) — the seven flags listed once under
  `global_flags`.
- [output-formats.md](output-formats.md) — the JSON contract rules the document
  obeys.

## Open questions

None specific to this module. `OQ-014` is answered by `FR-HELP-026` through
`FR-HELP-029` and is listed under [Closed](open-questions.md#closed).
