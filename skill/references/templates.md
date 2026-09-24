# Templates

Templates are MiniJinja files under `.tpl/templates/`, loaded at render time. Edit them with ordinary file tools. No rebuild is needed.

The live list of variables, filters, tests and functions is published by the binary. Consult it when in doubt:

```sh
tpl help render                                        # prose, with every filter, test and function
tpl help --format json > /tmp/h.json
jq '.data.context_variables, .data.template_surface' /tmp/h.json
```

## Names, includes and imports

- A template name is its path under `.tpl/templates/` without `.jinja`: `rust/struct` is `rust/struct.jinja`. `render`, `template show`, `template check` and `template path` take the name (the `.jinja` suffix is optional there).
- Inside a template, `{% include %}` and `{% import %}` need the full file name, extension included: `{% import "rust/_types.jinja" as types %}`, then `{{ types.of(column) }}`. Without the extension, the render fails with exit 65, and the hint names the right file.
- By convention, a leading `_` marks a partial or macro file (`rust/_types`). It is still listed and checkable.
- A symbolic link, or any path that resolves outside `.tpl/templates/`, is refused with exit 65.

## Context variables

| Variable | Contents |
|---|---|
| `database` | Always present: `name`, `charset`, `collation`, `server` (`version`, `series`, `standing`), `tables`, `views`, `routines` |
| `table` | The object bound by `--table` |
| `view` | The object bound by `--view` |
| `routine` | The object bound by `--routine` |
| `vars` | The `--set` values, all strings. `{}` when none are given |
| `tpl` | `tpl.version` |
| `now` | Render time in UTC, e.g. `2026-09-10T08:14:22Z`. It is the only source of non-determinism |

### Trap: `table`, `view` and `routine` are also function names

The help says `table` is "undefined without --table". In tpl 0.0.1, the unbound name instead resolves to the global function of the same name. So `{{ table is defined }}` is `true` even without `--table`, and `{{ table }}` prints a function name. Test a field instead:

```jinja
{% if table.name is defined %}…bound…{% else %}{{ fail("render this with --table <name>") }}{% endif %}
```

`table.name is defined` is `false` when nothing is bound, and `true` when `--table` is given. The same holds for `view.name` and `routine.name`.

## The model

Use `tpl -d NAME schema table T --format json --pretty` (or a dump) to see real values. The field names below were observed in tpl 0.0.1 output.

| Object | Fields |
|---|---|
| table | `name`, `table_type`, `engine`, `collation`, `comment`, `columns`, `primary_key`, `indexes`, `foreign_keys`, `referenced_by`, `triggers`, `check_constraints` |
| column | `name`, `table_name`, `position`, `column_type` (raw, e.g. `smallint(5) unsigned`), `data_type` (e.g. `smallint`), `precision`, `scale`, `length`, `unsigned`, `charset`, `collation`, `values` (ENUM or SET members), `nullable`, `default`, `comment`, `auto_increment`, `invisible`, `generated`, `on_update` |
| index / primary_key | `name`, `unique`, `columns` (`name`, `direction`, `prefix_length`), `type`, `comment`, `ignored`. `primary_key` is `null` when the table has none |
| foreign key | `name`, `columns` (`column`, `referenced_column`), `referenced_table` (embedded one level: its columns and indexes, with nested `foreign_keys` cut short), `referenced_key`, `match_option`, `on_update`, `on_delete` |
| view | `name`, `definition`, `is_updatable`, `algorithm`, `check_option`, `security_type`, `definer`, `character_set_client`, `collation_connection` |
| routine | `name`, `kind` (`PROCEDURE` or `FUNCTION`), `return_type`, `parameters` (`name`, `mode`, `parameter_type`), `body`, `body_kind`, `is_deterministic`, `sql_data_access`, `security_type`, `definer`, `comment`, `parameter_style`, `sql_mode`, `character_set_client`, `collation_connection`, `database_collation` |

A column's `default` is `null` (NOT NULL with no default), `{"kind":"null"}`, `{"kind":"literal","value":…}` or `{"kind":"expression","value":…}`. An absent scalar is `null`, and it prints as an empty string. Test it with `is none`.

## Filters, tests and functions

The functions, and the tests and filters tpl registers, are a contract. The engine filters that are listed are pinned. Anything else the engine offers works, but may change when the engine version changes.

- **Naming filters:** `pascal`, `camel`, `snake`, `upper_snake`, `kebab`.
- **Code filters:** `quote` (MariaDB backticks), `sql_type` (a column's `data_type`), `json`, `indent(n)`, `comment(prefix)`, `escape("html"|"xml")`.
- **Pinned engine filters:** `default`, `join`, `length`, `map(attribute=…)`, `select`, `reject`, `first`, `last`, `reverse`, `sort(attribute=…, reverse=…, case_sensitive=…)`, `trim`, `upper`, `lower`, `replace`.
- **Column tests:** `nullable`, `primary_key`, `auto_increment`, `unique`, `numeric`, `temporal`, `textual`. Each fails the render on anything but a column.
- **Functions:** `table(name)`, `view(name)`, `routine(name)`, `column(table, name)` (using the result fails the render when the object is missing), and `fail(message)`, which stops the render with exit 65.

There is no per-language type filter (no `rust_type`). Write a type-mapping macro the project owns, as `rust/_types.jinja` does, and end it with `fail()` so an unmapped `data_type` stops the render instead of producing wrong code.

## Strictness: why renders fail and how to read it

- Undefined variables and misspelled fields fail with exit 65. The hint lists the object's real attributes and suggests the nearest match.
- Filters don't coerce: `{{ 42 | snake }}` fails. Convert explicitly, or check that the input is a string first.
- Auto-escaping is always off. Use `escape("html")` or `escape("xml")` where markup needs it.
- Each render is bounded by `core.render_timeout` (seconds), `core.render_fuel` (evaluation steps), `core.render_output_limit` (bytes) and `core.render_memory_limit` (bytes). Hitting any of them is exit 65, naming the key that raises the limit. Raise it with `tpl cfg set core.<key> <n>` only when the template is correct and the work really is large.
- `tpl template check` catches syntax errors only. Evaluation errors appear only in `tpl render`. Render against a dump (`--context`) to iterate quickly.

## Output hygiene

- Use Jinja whitespace control (`{%-`, `-%}`) to control blank lines. A leading `{# … #}` comment block still leaves its trailing newline: the shipped `example` renders with a leading blank line.
- Sort explicitly (`| sort(attribute="name")`) when order matters. The model's collections already come in a fixed, deterministic order.
- A render that exits 0 doesn't prove the output is correct. Compile, lint or test what you generated.
