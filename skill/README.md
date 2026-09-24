# tpl skill for Claude Code

This folder is a Claude Code skill. It makes Claude the operator of the `tpl` CLI. With it, Claude initialises projects, manages database entries and credentials, explores MariaDB schemas, manages the cache, checks templates and renders them. It does all of this by invoking `tpl`, never by reimplementing it.

The skill carries workflows, decision rules, an exit-code playbook, security rules and a map of every command. For exact flags and values it defers to the binary's own help (`tpl help <path> --format json`), so the skill does not become a stale copy of the contract.

```
skill/
├── SKILL.md                  # entry point: operator rules, decisions, exit codes
├── references/               # loaded on demand
│   ├── commands.md           # every command node, with aliases, needs and JSON data key
│   ├── workflows.md          # end-to-end procedures
│   ├── configuration.md      # keys, entries, credentials, TLS, read-only guarantee
│   ├── templates.md          # context, model fields, filters, tests, traps
│   └── errors.md             # recovery action for each exit code
└── scripts/check-coverage.sh # checks commands.md against the installed binary
```

## Install globally

Link the folder, so that updates to this repository reach Claude at once:

```sh
mkdir -p ~/.claude/skills
ln -s "$PWD/skill" ~/.claude/skills/tpl      # run from the repository root
```

Or copy it, which gives a snapshot you must refresh yourself:

```sh
cp -R skill ~/.claude/skills/tpl
```

Claude Code discovers the skill at the start of its next session.

## Update

- **Symlinked:** run `git pull` in this repository. Nothing else is needed.
- **Copied:** replace the copy: `rm -rf ~/.claude/skills/tpl && cp -R skill ~/.claude/skills/tpl`.

The skill needs a `tpl` binary on `PATH`. To install or update `tpl` itself:

```sh
curl -fsSL https://raw.githubusercontent.com/FlavioCFOliveira/tpl/main/install.sh | sh
```

## Check coverage

After `tpl` gains or renames a command, confirm that the command map still names every command path and alias the binary publishes:

```sh
sh skill/scripts/check-coverage.sh                          # uses tpl on PATH
TPL=target/release/tpl sh skill/scripts/check-coverage.sh   # or a specific binary
```

The check exits 0 when everything is covered, 1 when an item is missing (each one is listed on stderr), and 2 when `tpl` or the map cannot be used. It uses `jq` when available, and a pure `sh`/`awk` parser otherwise.
