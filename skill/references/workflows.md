# Workflows

Each workflow gives the commands in order, the exit code to expect, and the decision points. Replace `shop` with the real entry name. Check the exit code after every step before moving on.

## Contents

1. Start from nothing
2. Register and verify a database entry
3. Explore a schema
4. Render one object
5. Render one file per table (or view, or routine)
6. Render without a server (`--context`)
7. Work from a warm cache
8. Repoint, rename or remove an entry
9. Author and iterate on a template
10. Operate on a project from outside it

## 1. Start from nothing

```sh
command -v tpl || echo "tpl missing"   # if missing, ask the user, then use the README installer (see SKILL.md)
tpl help --format json > /tmp/tpl-help.json   # optional: the whole surface, with no project needed
tpl init                  # or: tpl init path/to/dir
```

- Exit 0 prints nothing. It creates `.tpl/.cfg` (mode 0600, commented, with no entry), `.tpl/.gitignore` (`.cfg` and `.cache/`), and `.tpl/templates/` holding `example.jinja` and `rust/_types.jinja`.
- Exit 73 means a `.tpl` already exists there and nothing was changed. Use that project.
- Exit 64 on `tpl init proj/.tpl`: give the directory that will hold `.tpl` (`tpl init proj`).
- A project created inside another one succeeds, with a warning that it hides the one above. Tell the user.

## 2. Register and verify a database entry

Pick one shape. Both keep the password out of the file and off the command line.

```sh
# Discrete fields, with the password from the environment:
tpl cfg database add shop --host db.example.com --user reader --schema shop
tpl cfg set database.shop.password '${SHOP_PASSWORD}'      # single quotes: the shell must not expand it

# One URL, with the reference inside it:
tpl cfg database add reporting --dsn 'mysql://reader:${REPORTING_PASSWORD}@db.example.com:3306/reporting'

# A secret store, run without a shell:
tpl cfg database add staging --host db-staging.example.com --user reader --schema shop \
  --password-command "pass db/staging"
```

- `--schema` is the database name on the server. `NAME` is the entry's local name (1 to 64 letters, digits or underscores), and `-d NAME` selects it.
- For a local or development server without a trusted certificate, add `--tls disabled` (or `--tls preferred`). The default, `verify-identity`, fails against it.
- Exit 64 if the name is taken (the hint gives the `update` command instead), if `--dsn` is mixed with the discrete flags, or if a value is malformed.
- An entry without a host and a server database name is stored, but every command that connects with it exits 78.

Optionally make it the default, then verify it:

```sh
tpl cfg set core.database shop
SHOP_PASSWORD=… tpl cfg database test shop --format json
```

Read the result:

- Exit 0 means the four checks ran. Then read `data.can_read_catalogue`: `false` means the user logs in but can't see the catalogue, so request `SELECT` access to it. `data.server.standing` is `supported` or `newer_than_supported`.
- Exit 69: unreachable, refused, or timed out. Check the host and port. Retrying is safe.
- Exit 77: login refused. Fix the user or the password source.
- Exit 78: undefined `${VAR}`, `password_command` failed, unsupported server, or the session could not be made read-only.

`cfg database test` never reads or writes the cache.

## 3. Explore a schema

```sh
tpl -d shop schema info --format json
tpl -d shop schema tables --format json | jq -r '.data.tables[].name'
tpl -d shop schema tables --pattern 'order%' --format json
tpl -d shop schema table orders --format json --pretty
tpl -d shop schema views --format json
tpl -d shop schema view v_sales --format json | jq -r '.data.view.definition'
tpl -d shop schema routines --format json | jq -r '.data.routines[] | .kind + " " + .name'
tpl -d shop schema routine function:calc_vat --format json
```

- The first read connects and stores what it read. Later reads are served from `.tpl/.cache/` (`"source":"cache"`).
- `schema tables --format json` already returns full table objects (columns, indexes, keys). One call is often enough, with no need for one `schema table` call per table.
- A routine name shared by a procedure and a function is exit 64 until you qualify it as `procedure:NAME` or `function:NAME`.
- A name the cache does not hold is a cache miss and triggers a connection. With the server down, a mistyped name therefore reports 69, not 66. Check spelling against the listing first.

## 4. Render one object

```sh
tpl template list --format json                         # the names render accepts
tpl template check rust/struct                          # parse first: exit 65 names the line and column
tpl -d shop render rust/struct --table orders > src/models/orders.rs.tmp &&
  mv src/models/orders.rs.tmp src/models/orders.rs
tpl -d shop render docs/table.md --table orders --set title=Orders --set author=data-team
tpl -d shop render api/proc --routine procedure:refresh_stats
```

- stdout holds the rendered text and nothing else. Always go through a temporary file: a redirect empties its target before tpl runs, and a failed render prints nothing.
- With no object flag, the template gets the whole database only (`database`, `vars`, `tpl`, `now`).
- `--set` splits on the first `=`: `--set msg=a=b` gives `vars.msg == "a=b"`. Keys are letters, digits and `_`, not starting with a digit, and each key may be given only once.

## 5. Render one file per table (or view, or routine)

Iteration is the caller's job. Run one render per object:

```sh
mkdir -p src/models
tpl -d shop schema tables --format json |
  jq -r '.data.tables[].name' |
  while read -r table; do
    f="src/models/$table.rs"
    tpl -d shop render rust/struct --table "$table" > "$f.tmp" && mv "$f.tmp" "$f" ||
      { echo "render failed for $table" >&2; rm -f "$f.tmp"; }
  done
```

- Warm the cache first (`tpl -d shop cache load`) so the loop opens no connection per table.
- For views use `.data.views[].name` with `--view`. For routines, use `.data.routines[] | "\(.kind|ascii_downcase):\(.name)"` with `--routine`, which avoids the procedure and function name clash.
- Stop, or report, on the first non-zero exit. Don't leave partial `.tmp` files behind.
- A render that exits 0 proves the template evaluated, not that the output is correct. Compile, lint or test the generated code afterwards.

## 6. Render without a server (`--context`)

```sh
tpl -d shop schema dump > shop.json               # once, with server or cache access
tpl render rust/struct --context shop.json --table orders
tpl -d shop schema dump | tpl render rust/struct --context - --table orders   # through stdin
```

- `--context` uses no entry, no password, no cache and no network, so it suits CI, reviewing a snapshot, or a down server. Don't also pass `-d` or `--direct` (exit 64).
- The document must be `schema dump` output (compact or `--pretty`). A malformed document is exit 65, and an unreadable file is exit 74.
- An object missing from the document is exit 66. The hint suggests `jq -r '.data.database.tables[].name' FILE`.
- A dump is a snapshot. Regenerate it after schema changes.

## 7. Work from a warm cache

```sh
tpl -d shop cache load                   # whole database. Exit 69 leaves the old cache unchanged
tpl -d shop cache load --table orders    # refresh one object
tpl -d shop cache status --format json   # loaded_at, and per-collection count and "whole"
tpl -d shop schema tables --direct       # force a live read, and refresh the cache
tpl -d shop schema dump --direct --no-cache > live.json   # live read that touches no file
tpl -d shop cache clean                  # delete everything cached for the entry
tpl -d shop cache clean --view v_sales   # delete one object (exit 66 if it is not cached)
```

- Nothing expires. `loaded_at` in `cache status` is the only staleness signal. If the schema may have changed, reload or use `--direct`.
- **A cache hit still resolves the entry.** Every `${VAR}` it references must be defined, and `password_command` must succeed, even when no connection is opened. Otherwise the command exits 78.
- Cleaning one object marks its collection as not whole. After that, a listing, a dump or a render needs the server again until you run `tpl -d shop cache load`. Prefer `cache load --table X` for refreshing one object.
- Read the cache only through `tpl`. Its on-disk layout is not a contract.

## 8. Repoint, rename or remove an entry

Point an entry at another server:

```sh
tpl cfg database update shop --host db-staging.example.com   # warns that the old cache is kept
tpl -d shop cache clean                                      # required, or reads keep serving the old server
tpl cfg database test shop
```

Remove an entry:

```sh
tpl cfg database remove staging        # also clears core.database if it named the entry. Warns about its cache
tpl -d staging cache clean             # removes the orphaned cache (allowed although the entry is gone)
```

- "Rename" means `add` the new name, `remove` the old one, then clean the old name's cache. An entry added later under an old name reads that name's leftover cache, so clean before its first read.
- `update` changes only the fields named by the flags given. Switching an entry between dsn and discrete fields is refused (exit 64) while the other shape's keys remain. Follow the hint, or `tpl cfg unset` the conflicting keys first.
- `tpl cfg unset database.shop` removes the whole entry, the same as `cfg database remove shop`.

## 9. Author and iterate on a template

```sh
tpl template path                        # absolute path of .tpl/templates/
# write .tpl/templates/<dir>/<name>.jinja with your file tools
tpl template check <dir>/<name>          # syntax only, safe, no server
tpl -d shop schema table orders --format json --pretty   # see the exact field names the template gets
tpl -d shop render <dir>/<name> --table orders           # evaluate against the cache
```

- Create and edit template files directly. No `tpl` command writes templates.
- Iterate against a dump (`--context shop.json`) so you need no server and no password while authoring.
- Exit 65 names the template, line and column, plus what was undefined or mistyped. Fix the template and render again. The cache filled by the failed attempt is reused.
- For template language details and traps, read `templates.md`.

## 10. Operate on a project from outside it

```sh
tpl --tpl-dir /path/to/project/.tpl template list
tpl --tpl-dir /path/to/project/.tpl -d shop schema tables --format json
```

`--tpl-dir` must name the `.tpl` folder itself (exit 78 otherwise, with the right path in the hint). It turns off the upward search, and the folder still has to pass the ownership and mode checks.
