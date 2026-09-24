# `_driver` — the workflow every worked example runs

A worked example builds an application's data layer for one target language out
of a known database schema, using nothing but the command line. `FR-EX-004`
fixes the sequence:

```
tpl init  →  tpl cfg database add  →  tpl cfg database test  →  tpl schema …  →  tpl render …
```

This package is that sequence, written once.

## Why once

`FR-EX-007` requires the four examples to read the same three schemas from the
same server under the same conditions, "database entries that differ in nothing
a read can observe" included. The set is comparable only while one axis varies:
where two examples render different data layers, the difference must be
attributable to their templates and their type mappings, because that is what
the set exists to compare. Four copies of a host, a port, a user and a
five-command sequence would drift, and the drift would not announce itself.

Written once, the property holds by construction rather than by inspection.

## What it does not know

No language. No template name. No file name. No table list.

Everything example-specific arrives through one callback. The driver reads the
three schemas, hands them to the example's `plan` function, and performs the
renders that function returns. That is the whole of the example-specific
surface; everything else in this package is fixture.

## Using it

```python
from pathlib import Path
from _driver import Catalogue, Render, run

def plan(catalogue: Catalogue) -> list[Render]:
    return [
        Render(
            template="go/struct",
            destination=Path("out") / f"{table.name}.go",
            entry=table.entry,
            table=table.name,
        )
        for table in catalogue.schema("sakila").tables
    ]

outcome = run(Path("build"), plan, templates=Path("templates"))
```

`run` is the only entry point:

```python
def run(
    workspace: Path,
    plan: Plan,
    *,
    templates: Path | None = None,
    binary: Path | None = None,
    reset: bool = True,
    echo: bool = True,
) -> Outcome
```

| Parameter | What it is |
|---|---|
| `workspace` | The **build** directory. `tpl init` creates `<workspace>/.tpl` there, and that directory belongs to the driver |
| `plan` | `Callable[[Catalogue], Sequence[Render]]`, called once with the three schemas read |
| `templates` | A directory copied into `<workspace>/.tpl/templates/` after `init`, merging over what `init` wrote so an example may replace the macro of `FR-PROJ-017` as well as add to it |
| `binary` | A `tpl` executable to use instead of the one the search finds |
| `reset` | Remove an existing `<workspace>/.tpl` before `init`, so a rerun works. Only that directory, only directly under the workspace, never a link |
| `echo` | Write a line per step to standard error |

`Outcome` carries the project path, the catalogue, every file written in render
order, what `cfg database test` reported per entry, and every invocation made.

## What it asserts

The three channels an agent has into a command-line tool are the help text, the
exit code and the two streams. The driver treats the last two as a contract:

- **Every exit code is checked.** A non-zero code raises `CommandFailed`, which
  carries the argv, the code and the whole of `stderr`, and the run stops there.
- **`stdout` is the sole carrier of a result.** Nothing reads a result out of
  `stderr`. A render's bytes are captured and written to the file unchanged —
  no decode, no re-encode, no newline added or removed.
- **`stderr` is the sole carrier of a diagnostic.** The two streams are captured
  separately and never merged; merging them would make a diagnostic
  indistinguishable from a result. The driver's own progress lines go to
  `stderr` for the same reason.
- **`tpl render` has no `--output`.** It writes to standard output and nowhere
  else, per `FR-RND-028`, so a rendered file is a redirection the caller
  performs — `FR-EX-005`. `Tpl.render_to` is that redirection, and it creates
  the file only after the invocation has exited `0`, so a failed render leaves
  no truncated file for a compile gate to trip over.
- **The JSON envelope is checked.** `schema_version`, `source` and `data` must
  all be present, per `FR-OUT-024`, before `data` is used.
- **`cfg database test` exiting `0` is not taken as success.** Exit `0` means
  the four steps ran. `can_read_catalogue` is read from the document, per
  `FR-CFG-045`, and an entry that connects and cannot read raises
  `AccessRefused` — because a read through it would be incomplete under
  `FR-PRIV-001` and the data layer would be short without saying so, which is
  `UC-013`'s second alternate flow.

Nothing here reaches a library interface of `tpl`. `FR-EX-004` admits the
command line and nothing else, and the [specification](../../specification/examples.md)
says why: the primary consumer has three channels and only three, so an example
that reached past them would demonstrate a surface no such consumer can use.

## The access

The server and the three schemas live in `fixture.py`, not in an example.

| | |
|---|---|
| Server | `127.0.0.1:13309` — the fixture's most recent series, `12.3` |
| TLS | `verify-identity`, `tpl`'s own default and the strongest of the five, against `scripts/mariadb/tls/ca.pem` |
| User | `root` |
| Schemas | `sakila`, `world`, `freight`, each registered under an entry of its own name |
| Password | `${FIXTURE_PW}`, expanded from the environment |

**The password is never written into `.tpl/.cfg` and never passed on a command
line.** What `tpl cfg set` writes into the file is the literal string
`${FIXTURE_PW}`; `tpl` expands it from the environment at connection time, which
is the only mechanism by which it reads the environment at all
(`BR-CONF-003`). A value written into an argv is visible in the process table
for the life of the invocation, and every process on the host can read it —
`specification/security.md` is why this is not a stylistic preference.

The driver checks `$FIXTURE_PW` before it writes anything. Left to `tpl`, the
absence surfaces correctly but only at the third step, as exit `78`, after a
project and three entries have already been written.

**Why `root` and not `tpl_reader`.** The fixture's other user exists to make an
*incomplete* catalogue read reproducible, which
[`scripts/mariadb/README.md`](../../scripts/mariadb/README.md#credentials) says
in as many words. Measured on this fixture, `tpl_reader` sees every table and
every column of `sakila` and none of its 6 triggers, 40 table constraints or 22
referential constraints, so a data layer built through it would silently lose
every foreign key. An example is the demonstration that the workflow produces a
*complete* data layer, so it reads through the user that can see one.

## Finding `tpl`

No path is assumed. `discovery.tpl_binary` searches, in order:

1. a binary the caller named;
2. `$TPL_BIN`;
3. `target/release/tpl`, then `target/debug/tpl`, under the repository root —
   the binary this repository builds, which is what an example run here is
   demonstrating;
4. `tpl` on `$PATH`, for an installed copy.

Failing all four raises `BinaryNotFound`, naming every place that was looked in,
because the remedy differs per place.

The repository root is found by walking up for a directory carrying
`Cargo.toml`, `specification/` and `scripts/mariadb/` together — all three,
because one of them alone is a coincidence.

## Before running

```sh
cargo build --release
./scripts/mariadb/up.sh
./scripts/mariadb/seed-datasets.sh 12.3
export FIXTURE_PW=tpl-root
```

`sakila` and `world` are not in the fixture image; `seed-datasets.sh` is what
puts them in, and
[`scripts/mariadb/README.md`](../../scripts/mariadb/README.md#the-published-datasets)
documents both. `freight` is in the image and needs nothing.

## Dependencies

None. Standard library only: `subprocess`, `json`, `pathlib`, `shutil`,
`dataclasses`. The workflow is five command invocations, a JSON parse and a file
write, and no third-party package would carry its weight against that.

## Layout

| File | What it holds |
|---|---|
| `__init__.py` | The public surface, re-exported |
| `workflow.py` | `run`, the entry point, and `Render`, `Access`, `Outcome` |
| `cli.py` | `Tpl`: building an argv, running it, checking the exit code, keeping the streams apart |
| `fixture.py` | The server, the three schemas and the password arrangement |
| `catalogue.py` | What a schema read hands an example |
| `discovery.py` | Finding the binary and the repository root |
