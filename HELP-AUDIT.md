# Audit #259 — help texts and error messages, read for AI-agent comprehension

Binary: `target/release/tpl` 0.1.0. It was rebuilt at the start of the audit because `src/` was newer than the binary. Branch `feature/21-help-refinement`, HEAD `2280ecd`. Date 2026-09-23.
Method: every node's help was captured, as text and as JSON (`tpl help --format json`). Every error class reachable without a MariaDB server was provoked in scratch projects under the session scratchpad: usage, project, configuration, template, render against `--context`, render bounds, password_command, DNS, TCP refused, the overall deadline, stdout closure and TLS trust files. The rest were read in `src/error.rs`, `src/diagnostics/cause.rs`, `src/diagnostics/hint.rs`. The findings were compared with `specification/help-and-version.md`, `errors-and-exit-codes.md`, `cli-contract.md` and `output-formats.md`, plus the owning modules where needed (`cfg-commands.md`, `cache-commands.md`, `configuration-model.md`, `project-and-discovery.md`).

Both specs put the wording of individual texts out of scope. `help-and-version.md` says so under Scope, and `errors-and-exit-codes.md` says so at line 30. So most fixes below need **no spec change**.

Where the text lives:
- Help text is the typed table in `src/cli/help.rs`. That covers purposes, descriptions, examples and the `Outcome` tables at lines 953–1480.
- Each error line is the `#[error]` on its variant in `src/error.rs`.
- Cause lines are in `src/diagnostics/cause.rs`.
- Hint lines are in `src/diagnostics/hint.rs`.
- The `.cfg` header is in `src/project/init.rs`.
- Flag value descriptions are in `src/cli/intercept.rs`.
- The `--pretty` rule is in `src/cli/rules.rs`.

---

## 1. Coverage

### 1.1 Command paths (35 nodes, all run)

| Path | `--help` text | `help <path>` | JSON entry | Behaviour checked |
|---|---|---|---|---|
| `tpl` (root, also `tpl`, `-h`, `help`) | ran | ran | ran (global_flags) | ran |
| `schema` | ran | — | ran | ran (group help) |
| `schema info` | ran | — | ran | ran (69 path) |
| `schema tables` / `tbls` | ran | — | ran | ran (69, 66, 78) |
| `schema table` / `tbl` | ran | — | ran | ran (69) |
| `schema views` / `vws` | ran | — | ran | source only (needs DB) |
| `schema view` / `vw` | ran | — | ran | source only |
| `schema routines` / `rtns` | ran | — | ran | source only |
| `schema routine` / `rtn` | ran | — | ran | source only |
| `schema dump` | ran | — | ran | ran (69, --format refused) |
| `template` | ran | — | ran | ran |
| `template list` | ran | — | ran | ran |
| `template show` | ran | — | ran | ran |
| `template check` | ran | — | ran | ran |
| `template path` | ran | — | ran | ran |
| `render` | ran | ran (JSON) | ran | ran (--context, bounds, 69) |
| `cache` | ran | — | ran | ran |
| `cache load` | ran | — | ran | ran (69, --no-cache, --direct) |
| `cache clean` | ran | — | ran | ran |
| `cache status` | ran | — | ran | ran |
| `cfg` | ran | — | ran | ran |
| `cfg get` / `set` / `unset` / `list` | ran | — | ran | ran |
| `cfg database` / `db` | ran | ran (alias) | ran | ran |
| `cfg database add` / `list` / `show` / `update` / `remove` / `test` | ran | ran (add, JSON) | ran | ran (test → 69) |
| `init` | ran | — | ran | ran (73, shadow warning, artefacts) |
| `help` | ran | ran (bad segment) | ran | ran |
| `version` | ran | — | ran | ran |

### 1.2 Error variants (69 in `Error`, plus warning, panic, verbosity lines)

| Variant | Code | How | Finding |
|---|---|---|---|
| UnknownCommand | 64 | ran | E-09, E-10 |
| UnknownCommandPathSegment | 64 | ran | good |
| UnknownFlag | 64 | ran | E-07, E-11 |
| UnexpectedArgument | 64 | ran | good |
| RepeatedValueFlag | 64 | ran | E-09 (wording) |
| RepeatedFlag | 64 | source (bool repeats are accepted, per FR-CLI-025) | — |
| FlagValueMissing | 64 | ran | good |
| SeparateTokenValue | 64 | ran | good |
| ValueOutsideEnumeration | 64 | ran | E-07 |
| InvocationRejected | 64 | source | E-23 |
| MissingArgument | 64 | ran | E-12 (add), E-24 |
| MutuallyExclusiveFlags | 64 | ran | E-05 (--pretty case) |
| RoutinePrefixNotLowerCase | 64 | ran | E-07 |
| AmbiguousRoutineName | 64 | source | good |
| AmbiguousRoutineInContext | 64 | source | good |
| RepeatedSetKey | 64 | ran | E-09 |
| LoadWithoutStoring | 64 | ran | good |
| MalformedValue | 64 | ran (--timeout, --set, --port, --dsn, cfg set) | E-07, E-13 |
| UnknownConfigurationKey | 64 | ran | H-01 |
| DatabaseEntryAlreadyExists | 64 | ran | good |
| IncoherentEntryWrite | 64 | ran (Unset, Restate); Rewrite from source | E-09 |
| TemplateSyntax | 65 | ran | E-21 |
| RenderFailed | 65 | ran (undefined, fail(), include) | E-06 |
| TemplateOutsideRoot | 65 | ran (`..`, symlink) | good |
| ContextDocumentMalformed | 65 | ran (NotJson, Structure); DanglingReference from source | E-08 |
| RenderDeadlineExceeded | 65 | ran (Overall); Phase from source | good |
| RenderFuelExhausted | 65 | source | good |
| RenderOutputLimitExceeded | 65 | ran | good |
| RenderMemoryLimitExceeded | 65 | source | good |
| CatalogueObjectNotFound | 66 | source (needs DB) | good |
| ContextObjectNotFound | 66 | ran | E-27 |
| TemplateNotFound | 66 | ran | H-07 |
| DatabaseEntryNotFound | 66 | ran | good |
| ConfigurationKeyNotFound | 66 | ran | E-14 |
| NameNotResolved | 69 | ran | E-07, E-26 |
| ConnectionRefused | 69 | ran | E-07, E-26 |
| TlsHandshakeFailed | 69 | source | E-20 |
| NetworkDeadlineExceeded | 69 | ran (Overall); Phase from source | good |
| InternalInvariant | 70 | source | good |
| ProjectAlreadyExists | 73 | ran | good |
| ProjectNotCreated | 73 | ran | good |
| ProjectFileUnreadable | 74 | ran (--context, ca_file, ca_path) | E-04 |
| ProjectFileUnwritable | 74 | source | good |
| StdoutUnwritable | 74 | source (EBADF gives exit 0, per FR-ERR-025) | — |
| StdoutClosedMidDocument | 74 | ran (`--pretty` into `head`) | good |
| AuthenticationRefused | 77 | source | E-19 |
| PropertyNotReadable | 77 | source | E-19 |
| ProjectNotFound | 78 | ran (walk, --tpl-dir) | E-03 |
| ConfigurationNotOwned | 78 | source | E-22 |
| ConfigurationUnsafeMode | 78 | ran | E-22 |
| ConfigurationMalformed | 78 | ran | E-16 |
| ConfigurationKeyOutsideSpace | 78 | ran | E-17, E-01 |
| ConfigurationValueMalformed | 78 | ran | E-01, E-02 |
| DsnMalformed | 78 | ran (Scheme); Form from source | E-01 |
| UnclosedExpansion | 78 | ran | good |
| PasswordCommandNotAnArray | 78 | ran | good |
| ConflictingEntryKeys | 78 | ran | E-01 |
| DsnQueryParameter | 78 | ran | E-01 |
| UndefinedVariable | 78 | ran | good |
| PasswordCommandDeadlineExceeded | 78 | ran (Phase) | good |
| PasswordCommandOutputCapExceeded | 78 | ran | good |
| PasswordCommandNotExecutable | 78 | ran (NotStarted) | E-18 |
| PasswordCommandFailed | 78 | ran (Exited) | good |
| TrustDirectoryEmpty | 78 | ran | good |
| ReadOnlySessionNotEnforced | 78 | source | good |
| EntryKeyMissing | 78 | ran | E-15 |
| NoDatabaseEntrySelected | 78 | ran | E-28 |
| ServerNotMariaDb | 78 | source | good |
| SeriesNotSupported | 78 | source | good |
| `warning:` (init shadow) | 0 | ran, and `-q` suppresses it | good |
| panic → 70 (`diagnostics/panic.rs`) | 70 | source | good |
| `phase:` lines at `-vvv` | — | ran | good |

---

## 2. Already good

- **Help structure.** Every node's help has the fixed sections. Every node has copyable examples. Exit-code tables are per command. `-h`, `--help` and `help <path>` produce the same bytes, and aliases resolve (`help cfg db`).
- **Help content** that is clear and honest as it stands:
  - `cfg database test`: its exit 0 is not a promise of full usability, and the example shows how to read `can_read_catalogue`.
  - `template list`: the note about `{% include %}` needing the extension.
  - `cfg get`: the stated exception to redaction.
  - `schema routine`: when a routine name must be qualified.
  - `render`: "Redirect stdout …: this command has no --output".
  - `init`: its five artefacts, which match what `init` actually writes.
- **Errors that already meet all four criteria:**
  - render bounds (output limit, overall budget): the key named and a runnable hint;
  - UndefinedVariable (`export NAME=<value>`);
  - PasswordCommand deadline, output cap and exit status;
  - SeparateTokenValue (`--database=-shop`);
  - IncoherentEntryWrite, both repairs;
  - DatabaseEntryAlreadyExists;
  - TrustDirectoryEmpty (two concrete commands);
  - TemplateOutsideRoot;
  - UnknownCommandPathSegment, which names the parent and suggests `add`;
  - StdoutClosedMidDocument;
  - init ProjectAlreadyExists ("has changed nothing here");
  - the 70 hint.
- **Output channels.** Errors never go to stdout. `--format json` never turns an error into JSON. `-q` suppresses the warning and nothing else.

---

## 3. Help findings

### H-01 — `tpl help cfg set` does not list the keys, although three places send the reader there · **high** · honest, explicit
- **Surface:**
  - the `cfg set` help;
  - the generated `.tpl/.cfg` header;
  - the hints of UnknownConfigurationKey and ConfigurationKeyOutsideSpace.
- **Current text:**
  - `.cfg` line 2: "`tpl help cfg set` the keys it admits."
  - hint: "show what tpl cfg set accepts with: tpl help cfg set".
  - cfg set DESCRIPTION: "The key must be in the configuration key space and the value must conform to the type that key declares."
- **Broken:** No help node lists a key, its type or its default. The only place that shows them is the commented `.cfg` header. So an agent that follows the hint lands on text that does not answer it.
- **Fix:** add a key list to the `cfg set` DESCRIPTION, or to the `<KEY>` purpose, one line per key. For example:
  ```
  Keys (type, default when absent):
    core.database             entry name, none
    core.connect_timeout      seconds, 10
    core.query_timeout        seconds, 30
    core.password_timeout     seconds, 5
    core.render_timeout       seconds, 30
    core.render_fuel          steps 1..10^12, 100000000
    core.render_output_limit  bytes 1..2^40, 67108864
    core.render_memory_limit  bytes 8388608..2^40, 134217728
    database.<entry>.dsn|host|port(3306)|user|password|password_command|database|tls(verify-identity)|ca_file|ca_path
  ```
- **Spec change:** none. FR-HELP-014 requires help to be self-contained, which supports this.
- **Where:** `src/cli/help.rs:2097` (cfg set), `src/project/init.rs:74`, `src/diagnostics/hint.rs` (UnknownConfigurationKey, ConfigurationKeyOutsideSpace).

### H-02 — Template variables are not named in any help · **high** · explicit
- **Surface:** the `render` help, the `schema dump` help, and the JSON `template_surface`.
- **Current text:**
  - render: "One object at most is bound, named by --table, --view or --routine; with none, the whole database is in context."
  - dump: "carries the server-derived context alone — no vars, no tpl, no now".
- **Broken:**
  - Neither channel says that a template reads `database`, `table` / `view` / `routine`, `vars`, `tpl` or `now`.
  - `vars`, `tpl` and `now` appear only as unexplained words in `schema dump`.
  - "Bound" is jargon.
  - An agent writing its first template must guess, or read `example.jinja`.
- **Fix:** add this to the `render` DESCRIPTION:
  > "A template sees these variables: `database` (the whole database: name, tables, views, routines, server); `table`, `view` or `routine` (only with --table, --view or --routine); `vars` (the --set values); `tpl` and `now`. Without an object flag, `table` is undefined."
- **Spec change:**
  - The top-level names in the render DESCRIPTION need none.
  - To publish them in the JSON tree as well, amend **FR-ENV-005** / **FR-HELP-017**, which name only filters, tests and functions in `template_surface`.
- **Where:** `src/cli/help.rs:1858` (render), `:1726` (dump).

### H-03 — The template surface is JSON-only, and gives names without meaning · **high** · explicit, text/JSON parity
- **Surface:** `data.template_surface` in the JSON tree; the text help has no equivalent.
- **Current:** `"registered":{"filters":["pascal","camel",…,"sql_type","json","indent","comment","escape"],"tests":[…],"functions":["table","view","routine","column","fail"]}`.
- **Broken:**
  - No text node lists the filters, tests or functions.
  - The JSON gives no arguments and no one-line purpose. For example, what `table(...)` takes, what `comment` takes, or what `fail` does.
  - An agent cannot use `column(...)` or `indent` correctly from this.
- **Fix:**
  - Give each item a signature and one sentence, e.g. `{"name":"table","signature":"table(name)","purpose":"Returns the table named, or fails."}`.
  - Render the same list in the text help of `tpl render`, inside DESCRIPTION.
- **Spec change:**
  - **FR-ENV-005**, for the per-item signature and purpose.
  - **FR-HELP-006** / **FR-HELP-007**, if a new text section is wanted, since they forbid any section beyond the seven. Placing the list inside `render` DESCRIPTION avoids changing FR-HELP-006.
- **Where:** `src/cli/help/document.rs` (template_surface), render entry in `src/cli/help.rs`.

### H-04 — `--tls` and `--port` say "No default", but an absent value means `verify-identity` and `3306` · **high** · honest
- **Surface:** `cfg database add --help` and `cfg database update --help`.
- **Current text:** `--tls <MODE> … Type: one of disabled, preferred, required, verify-ca, verify-identity. No default.` and `--port <PORT> … Type: integer. No default.`
- **Broken:** `configuration-model.md` (line 81) makes an absent `tls` mean `verify-identity` and an absent `port` mean `3306`. An agent that registers a local development server without `--tls` gets the strictest mode and a TLS failure, and nothing in the help warns of it.
- **Fix:**
  - `--tls`: "Default when not given: verify-identity (encrypted, certificate chain and host name checked). Use disabled or preferred for a local server without a trusted certificate."
  - `--port`: "Default when not given: 3306. Range 1–65535."
- **Spec change:** none.
- **Where:** `src/cli/help.rs:406`, `:424`.

### H-05 — `cfg database show` / `update` / `remove` share one exit-code table that is wrong for two of them · **medium** · honest
- **Current text:**
  - 0: "The command succeeded."
  - 64: "… or --dsn together with a discrete connection flag."
  - 74: "Reading .tpl/.cfg failed, rewriting it failed, or the result could not be written to stdout."
- **Broken:**
  - `show` and `remove` declare no `--dsn`. `tpl cfg database show shop --dsn x` gives "unknown flag".
  - `show` never rewrites anything.
  - `update` and `remove` write nothing to stdout.
  - "The command succeeded." says nothing.
- **Fix:** give each command its own table. For example:
  - show: 0 "The entry was written to stdout, passwords redacted."; 64 "unknown or repeated flag, or a missing NAME"; 74 "Reading .tpl/.cfg failed, or stdout could not be written."
  - update: 0 "The entry was changed in .tpl/.cfg."
  - remove: 0 "The entry was removed from .tpl/.cfg."
- **Spec change:** none. FR-HELP-011 already requires listing only the codes the command can produce.
- **Where:** `src/cli/help.rs:1384` (`CFG_DATABASE_ENTRY`).

### H-06 — `template path` lists "a missing NAME" under 64, but NAME is optional · **medium** · honest
- **Current text:** 64: "The invocation is not valid: an unknown or repeated flag, or a missing NAME."
- **Broken:** `tpl template path` with no NAME succeeds and prints the template root. The shared `TEMPLATE_NAMED` table is wrong for `path`.
- **Fix:** give `path` its own 64 row: "an unknown or repeated flag, or more than one NAME."
- **Spec change:** none.
- **Where:** `src/cli/help.rs:1170`.

### H-07 — "the nearest matches are suggested" is false for any template name containing `/` · **high** · honest
- **Surface:** the help of `template show`, `template path` and `template check`, and the hint of TemplateNotFound.
- **Current text:** 66: "NAME names no template of the project; the nearest matches are suggested."
- **Broken:** tested:
  - `tpl template show exampl` → "did you mean 'example'?"
  - `tpl template show rust/_type` → no suggestion, although `rust/_types` is one edit away.
  - `t/okk` → no suggestion.

  The filter `hint::admits` (FR-ERR-022: `[A-Za-z0-9_]{1,64}`) rejects `/`. So nested templates, which include every example in help (`rust/struct`, `docs/table.md`), never get a suggestion.
- **Fix:** choose one of these:
  - (a) Change the help wording to "a nearest match is suggested when the name holds only letters, digits and _".
  - (b) Admit `/` and `.` in template-name suggestions, since they are shell-safe inside quotes.
- **Spec change:** (a) none. (b) **FR-ERR-022** / **FR-ERR-023**, which define the admitted character set.
- **Where:** `src/cli/help.rs:1182`, `:1212`, `src/diagnostics/hint.rs` (`admits`).

### H-08 — `render` exit codes leave out conditions it produces · **medium** · honest
- **Current text:**
  - 65: "The template has a syntax error, the render failed, the --context document is malformed, or the render deadline expired."
  - 74: "Reading .tpl failed, or the result could not be written to stdout."
- **Broken:**
  - The render fuel, output and memory limits also give 65 (the output limit was observed).
  - A missing `--context` file gives 74 (observed: `nofile.json could not be read`, exit 74). Neither is mentioned.
- **Fix:**
  - 65: "… or a render limit was hit (render_timeout, render_fuel, render_output_limit, render_memory_limit)."
  - 74: "Reading .tpl or the --context file failed, or stdout could not be written."
- **Spec change:** none.
- **Where:** `src/cli/help.rs:1108–1121`.

### H-09 — `--direct` hides its side effect, and `cache load` lists a flag it refuses · **medium** · explicit, honest
- **Current text:**
  - `--direct`: "Reads the server for this invocation and ignores whatever the project's cache already holds."
  - `cache load` OPTIONS show `--no-cache` with its generic purpose ("Leaves the project's cache as it was …"), while its DESCRIPTION says the flag is refused.
- **Broken:**
  - FR-CACHE-015: `--direct` **writes** the cache. The help implies a read that touches nothing.
  - On `cache load`, the OPTIONS entry contradicts the DESCRIPTION.
- **Fix:**
  - `--direct`: "Read from the server even when the cache holds the object, and replace the cached copy. Add --no-cache to leave the cache untouched."
  - Give per-command purposes on `cache load`:
    - `--direct`: "Accepted and has no effect: this command always reads the server."
    - `--no-cache`: "Refused (exit 64): this command exists to store."
- **Spec change:** none. FR-HELP-030 indexes local purposes by command path.
- **Where:** `src/cli/help.rs:364` and the `cache load` entry at `:1983`.

### H-10 — Leaf helps do not say whether they need a server, need an entry, or write files · **medium** · explicit
- **Surface:** every `schema` leaf, `render`, and `cache status` / `clean`.
- **Current text:** e.g. `schema tables`: "Lists the tables of the selected database, one row each." That caching happens, and writes files, is said only in the `schema` and `cache` group helps.
- **Broken:** a leaf's help should be enough on its own. An agent reading `schema tables --help` does not learn three things:
  - that it needs `-d` or `core.database`;
  - that it connects on a cache miss;
  - that it writes `.tpl/.cache/`.

  Conversely, `cache status` and `cache clean` do not say that they never contact the server.
- **Fix:** add one fixed closing sentence to every leaf DESCRIPTION. For example:
  - "Needs a database entry (-d or core.database). Uses .tpl/.cache/ when it holds the data; otherwise connects to the server and stores the result there."
  - "Does not contact the server."
- **Spec change:** none. Optionally, a new FR-HELP requirement could make the sentence mandatory everywhere.
- **Where:** `src/cli/help.rs` (leaf descriptions, from `:1551`).

### H-11 — `schema` claims it "issues no statement but a SELECT against INFORMATION_SCHEMA" · **medium** · honest
- **Broken:** the session also issues `SET SESSION TRANSACTION READ ONLY`, `SELECT @@session.tx_read_only` and `SELECT VERSION()` (`src/mariadb/session.rs:62–69`).
- **Fix:** "It only reads: the session is set read-only first, and every query is a SELECT, against INFORMATION_SCHEMA for the structure."
- **Spec change:** none.
- **Where:** `src/cli/help.rs:1510`.

### H-12 — The top-level help has no first-run workflow · **medium** · explicit
- **Current examples:** `tpl`, `tpl help --format json`, `tpl init`.
- **Broken:** a weak agent does not learn the minimal sequence that gets it to a render.
- **Fix:** add one multi-line example, "Set up a project and render a table":
  ```
  tpl init
  tpl cfg database add shop --host db.example.com --user reader --schema shop --password-command "pass db/shop"
  tpl cfg set core.database shop
  tpl schema tables
  tpl render example --table orders
  ```
- **Spec change:** none.
- **Where:** the root entry in `src/cli/help.rs`, near `:1496`.

### H-13 — Spec-speak and jargon in purpose sentences · **medium** · simple
| Flag or place | Current | Proposed |
|---|---|---|
| `-d` | "Selects the [database.<name>] entry of .tpl/.cfg this invocation reads through, in place of the one core.database names." | "The database entry (a named connection in .tpl/.cfg; list them with tpl cfg database list) to use. Overrides core.database." |
| `--timeout` | "Bounds the whole invocation in seconds, measured from process start, beside the per-phase deadlines the project sets." | "Fail if the whole command takes longer than SECONDS. The per-step limits in .tpl/.cfg still apply." |
| `--format` | "Chooses the representation of the result: aligned columns laid out for a person, or the JSON document anything parsing the output must read." | "text: aligned columns for people, whose layout may change. json: a stable document; use it whenever a program reads the output." |
| `--help` | "Writes the help of the node it is written at, instead of doing that node's work." | "Print this command's help and exit 0." |
| `--routine` et al. | "Narrows the invocation to the one table named, in place of the whole catalogue." | "Use only this table. In render, the template then sees it as `table`." |
| `cfg` DESCRIPTION | "It has two arms: dotted keys … so that registering a connection is one invocation rather than five." | "Use get/set/unset/list for single keys, and cfg database for whole connections." |
| common words | "catalogue", "bound", "this reader", "template root", "entry" | Define once in the root help: "database entry = a named connection in .tpl/.cfg"; "reader" → "the database user" |
- **Spec change:** none.
- **Where:** `src/cli/help.rs:349`, `:604`, `:617`, `:2053`, and the object-flag purposes.

### H-14 — Root 70 row carries spec prose · **low** · simple
- **Current:** "… report it. It is listed here, once, and under no command of the tree."
- **Fix:** "70 EX_SOFTWARE  A bug in tpl, which any command can hit. Report it with the command and the output of tpl version."
- **Spec change:** none.
- **Where:** `src/cli/help.rs:975`.

### H-15 — The word "database" has three meanings · **medium** · clear
- **Surface:** `-d/--database`, which means the entry label; `--schema`, which writes the key `database.<entry>.database`; and the `schema` command group.
- **Current:** `--schema`: "Sets the server-side database the entry reads the catalogue of."
- **Broken:** FR-CFG-028 says "Help text must disambiguate wherever both could be meant". Today it does not. The error in E-15 shows the confusion that results.
- **Fix:** `--schema <NAME>`: "Name of the database on the MariaDB server, stored as database.<entry>.database. Not the entry name that -d selects."
- **Spec change:** none.
- **Where:** `src/cli/help.rs:418`.

### H-16 — Flag facts are verbose boilerplate · **low** · simple, redundancy
- **Current:** each boolean flag spends two lines on "Takes no value. Off unless given. Optional. Repeatable, and further occurrences have the effect of the first."
- **Fix:** keep the six facts of FR-HELP-013 in a compact form, e.g. `No value. Optional. Repeating it changes nothing.` and `String. Optional. No default. Once.`
- **Spec change:** none.
- **Where:** the fact renderer in `src/cli/help/render.rs`.

### H-17 — The `--set` purpose leaves out where the value goes · **medium** · explicit
- **Current:** "Defines one extra variable for the template, written as key=value, and may be given once per key."
- **Broken:** the purpose never says `vars`; only an example caption does. It also does not say the value is a string, or that the key must match `[A-Za-z_][A-Za-z0-9_]*`.
- **Fix:** "Adds a template variable: --set title=Orders makes vars.title equal "Orders" (always a string). Keys are letters, digits and _, not starting with a digit. Give each key once."
- **Spec change:** none.
- **Where:** `src/cli/help.rs:378`.

### H-18 — `cfg database add` does not say which flags count as "discrete" · **medium** · clear
- **Current:** "Either --dsn or at least one discrete connection flag is required".
- **Broken:** the error shows the set is `--host`, `--port`, `--user`, `--schema`. `--tls` and `--password-command` do not count.
- **Fix:** "Give --dsn, or at least one of --host, --port, --user, --schema (not both kinds)."
- **Spec change:** none.
- **Where:** the `cfg database add` description in `src/cli/help.rs`.

### H-19 — Incomplete 64 rows · **low** · honest
- **Broken:**
  - `cfg set` 64 omits unknown flags and a missing KEY or VALUE.
  - `cfg database add` 64 omits a malformed `--dsn`, `--port` or `--tls`, and a DSN with a password together with `--password-command`.
- **Spec change:** none.
- **Where:** `src/cli/help.rs:1318`, `:1369`.

### H-20 — JSON examples carry shell quoting inside `invocation` · **low** · parity
- **Current:** `"invocation":["tpl",…,"--table","\"$table\""]`.
- **Fix:** strip the shell quotes, or null the invocation on lines that expand variables.
- **Spec change:** none (BR-HELP-003 only requires that examples parse).
- **Where:** `src/cli/help.rs` (example tokenizer).

### H-21 — `template check` says "compile" in one place and "parse" in the other · **low** · consistency
- **Current:** the argument says "Names a template to compile", while the DESCRIPTION says "Parses templates … syntax analysis only".
- **Fix:** use "check" or "parse" throughout.
- **Spec change:** none.
- **Where:** `src/cli/help.rs:510`.

### H-22 — `cache status` text output on an empty cache · **low** · explicit (output rather than help)
- **Current:** `loaded_at` is printed blank, followed by an empty `COLLECTIONS` table.
- **Fix:** print `loaded_at  never (the cache is empty; fill it with tpl cache load)`.
- **Spec change:** none (FR-OUT-004: text is not a contract).
- **Where:** `src/cli/cache.rs`.

### H-23 — `cfg` and `schema dump` descriptions carry rationale prose · **low** · simple
- **Current:**
  - `schema dump`: "The output is JSON and nothing else, so this command declares no --format."
  - `cfg`: "rather than five".
- **Fix:** keep the facts ("Always JSON; --format is refused."), and drop the reasons.
- **Spec change:** none (BR-HELP-002).

---

## 4. Error findings

### E-01 — Hints for a bad `.tpl/.cfg` point at `tpl cfg …`, which cannot run until the file is fixed · **high** · honest, explicit
- **Surface:** ConfigurationValueMalformed, ConflictingEntryKeys, DsnQueryParameter, DsnMalformed (Form hint) and ConfigurationKeyOutsideSpace.
- **Observed** (after `cfg database add s6 --host h --port 0`):
  ```
  error: database.s6.port is not a TCP port between 1 and 65535
  cause: …/.tpl/.cfg at line 47, column 8 declares database.s6.port as 0; this key takes a TCP port between 1 and 65535
  hint:  write a conforming value with: tpl cfg set database.s6.port <value>
  exit:  78 (EX_CONFIG)
  ```
  Following the hint (`tpl cfg set database.s6.port 3306`) fails with the same 78. So do `cfg unset database.s6.port` and `cfg database remove s6`.
  The other hints are dead ends in the same way. ConflictingEntryKeys says "remove the other with: tpl cfg unset <key>". DsnQueryParameter says "tpl cfg database add <entry> …", which is for an entry that already exists.
- **Broken:** FR-ERR-035 says no cfg subcommand is excused from validating the file (step 3). So the hint can never work.
- **Fix:**
  - Hint: "edit <abs path>/.tpl/.cfg at line 47 and set port to a number from 1 to 65535 (tpl cfg cannot run until the file is valid)".
  - For ConflictingEntryKeys: "delete either `dsn` or `host` under [database.x] in <file>".
- **Spec change:**
  - None for the hint wording.
  - Optionally, amend **FR-PROJ-025** / **FR-ERR-035** so that `cfg unset` and `cfg set` can repair a file whose only fault is a value. That would make the current hints true.
- **Where:** `src/diagnostics/hint.rs` (the listed variants).

### E-02 — `cfg database add --port 0` writes a file the loader then refuses · **high** · honest (defect surfaced by the messages)
- **Observed:** the flag accepts 0. Its error text says "which takes a whole number from 0 to 65535" (`src/cli/intercept.rs:291`, `:1007`). The file validator requires 1–65535 (`src/project/config/keys.rs:76`). One accepted command leaves the project broken for every command (E-01).
- **Fix:** validate the flag as 1–65535 and state "a whole number from 1 to 65535".
- **Spec change:** none (configuration-model: "TCP port").
- **Where:** `src/cli/intercept.rs:291`, `:1007`.

### E-03 — A wrong `--tpl-dir` is reported as a failed upward walk · **high** · honest
- **Observed:**
  ```
  $ tpl --tpl-dir /nonexistent/.tpl template list
  error: no .tpl project found
  cause: the walk upward ended at /nonexistent/.tpl without meeting a .tpl folder
  hint:  create a project here with: tpl init
  ```
  The same happens when `--tpl-dir` names a regular file.
- **Broken:** `--tpl-dir` suppresses the walk (FR-PROJ-008), so the cause is false. The hint creates a project in the wrong place.
- **Fix:**
  - error: "the folder named by --tpl-dir does not exist: /nonexistent/.tpl"
  - hint: "correct --tpl-dir, or create the project with: tpl init /nonexistent"
  - Without `--tpl-dir`, add to the current hint: "or name one with --tpl-dir <path>/.tpl".
- **Spec change:** recommended. Amend **FR-ERR-034** (the 78 row names only "the directory the walk ended at") to cover the path `--tpl-dir` named.
- **Where:** `src/error.rs:1015`, `src/diagnostics/cause.rs` / `hint.rs` (ProjectNotFound).

### E-04 — A read failure on a file outside `.tpl` gets a `.tpl` hint · **high** · honest
- **Observed:** `--context nofile.json`, `ca_file="/nonexistent.pem"` and `ca_path="/nonexistent_dir"` all give:
  `hint:  make .tpl and its contents readable by the invoking user, then run the command again`
- **Broken:** none of these paths is in `.tpl`. The error line ("/nonexistent.pem could not be read") also does not say which setting named the file.
- **Fix:** carry the origin on the variant and state it:
  - "the --context file 'nofile.json' does not exist" / hint "check the path given to --context"
  - "the ca_file of database entry 'y' (/nonexistent.pem) cannot be read" / hint "tpl cfg set database.y.ca_file <path>"
- **Spec change:** none.
- **Where:** `src/error.rs:926` (ProjectFileUnreadable), `hint.rs`.

### E-05 — `--pretty` without `--format json` blames a flag the caller never wrote · **high** · honest, clear
- **Observed** (for `tpl template list --pretty` and `tpl help --pretty`):
  ```
  error: '--pretty' and '--format text' cannot be given together
  cause: the invocation supplies both '--pretty' and '--format text'; …
  hint:  give one of the two flags named above, and not both
  ```
- **Broken:** the invocation does not supply `--format text`; it is the default. The hint is generic because `admits_flag` rejects the space in "--format text". FR-OUT-009 says `--pretty` **requires** `--format json`.
- **Fix:**
  - error: "'--pretty' needs '--format json'"
  - cause: "--pretty indents JSON, and this invocation writes text (the default)"
  - hint: "add --format json: tpl template list --format json --pretty"
- **Spec change:** none.
- **Where:** `src/cli/rules.rs:57–65`.

### E-06 — A render "undefined value" error does not name the variable or the likely fix · **high** · explicit
- **Observed** (`{{ table.name }}` rendered with no `--table`, or `{{ vars.title }}` with no `--set`):
  ```
  error: rendering template 't/needtable.jinja' failed at line 1, column 4
  cause: 't/needtable.jinja' compiled and then failed while being evaluated; evaluation stopped at line 1, column 4 reporting undefined value (in t/needtable.jinja:1)
  hint:  print the template's source with: tpl template show <template>
  ```
- **Broken:** this is the most common failure a template author meets. The message does not name `table` or `vars.title`. The hint does not substitute the known template name, and it does not mention `--table` or `--set`.
- **Fix:**
  - Name the undefined expression, where the engine span allows it.
  - Add a context-aware hint:
    - no object flag given → "bind the object the template reads, e.g.: tpl render t/needtable --table <name>"
    - a `vars.` read → "pass it with --set title=<value>"
  - Always write the real template name.
- **Spec change:** none (FR-ERR-011 is already met; this adds to it).
- **Where:** `src/diagnostics/cause.rs` / `hint.rs` (RenderFailed), `src/render.rs`.

### E-07 — Hints use placeholders where the value is known · **medium** · explicit
- **Observed:**
  - UnknownFlag, ValueOutsideEnumeration and MalformedValue → "tpl help <command>". The command is known, e.g. `template list`. For a global flag (`--timeout`), the right node is `tpl help`.
  - RenderFailed → `<template>`.
  - RoutinePrefixNotLowerCase → `tpl render <template> --routine procedure:x`.
  - ConnectionRefused, NameNotResolved, TlsHandshakeFailed and AuthenticationRefused → `<entry>`. For example, `tpl cfg database update <entry> --port <port>` was printed for entry `refused`.
- **Broken:** FR-ERR-009 asks for a concrete runnable command wherever one exists.
- **Fix:** carry the command path, template and entry on the variants, and substitute them (subject to FR-ERR-022).
- **Spec change:** none.
- **Where:** `src/diagnostics/hint.rs`, and `src/error.rs` for the new fields.

### E-08 — The `--context` structure error cites a spec file and does not name the fault · **medium** · simple, explicit
- **Observed:**
  `cause: 'struct.json' is well-formed JSON and does not satisfy the context-document contract: the document carries every key context-document.md fixes, each with the type it fixes`
- **Broken:**
  - It names a document the caller does not have. FR-HELP-014 applies in spirit.
  - It restates the rule instead of naming the missing key (`data.database`).
- **Fix:**
  - cause: "'struct.json' has no key data.database (expected an object)"
  - hint: "produce a valid document with: tpl -d <entry> schema dump > struct.json"
- **Spec change:** **FR-ERR-034**, 65 row. It currently asks for "the structural rule of context-document.md it failed". Amend it to ask for the key path and the expected type.
- **Where:** `src/cli/render/context*` (rule strings), `cause.rs`.

### E-09 — Spec-speak in cause lines · **medium** · simple
| Variant | Current fragment | Proposed |
|---|---|---|
| UnknownCommand | "is not a name in the command tree, which is closed; tpl matches a command exactly and never by a prefix of one" | "'sch' is not a tpl command (commands must be typed in full)" |
| UnknownFlag | "tpl matches a long flag exactly and never by a prefix of one" (also printed for `-x`) | "'--formt' is not a flag of 'tpl template list' (flags must be typed in full)" |
| UnknownConfigurationKey | "outside the key space tpl cfg set writes into; the space is closed and a key is never created" | "'core.databse' is not a configuration key" |
| RepeatedValueFlag / RepeatedSetKey | "tpl refuses the repetition rather than letting one of them silently win" | drop the clause |
| AuthenticationRefused | "the refusal came from the server and not from tpl" | "the server rejected the password for 'reader'@'host'" |
| Ambiguous routine | "tpl resolves a bare name in favour of neither" | "both a procedure and a function are named x" |
| ProjectNotFound | "the walk upward ended at /" | "no .tpl folder in <cwd> or any parent folder" |
| RenderFuel | "the render exhausted its render fuel" | "the render took more than 100000000 steps (likely an endless loop)" |
| IncoherentEntryWrite | "which may state its connection and its password one way or the other and never both" | "an entry uses either dsn or host/port/user/database, not both" |
- **Spec change:** none.
- **Where:** `src/diagnostics/cause.rs`.

### E-10 — An unknown subcommand does not name its group and points at the root · **medium** · explicit, consistency
- **Observed:** `tpl schema tabls` → `error: unknown command 'tabls'` … `hint:  did you mean 'table', 'tables' or 'tbls'? list the commands with: tpl help`.
- **Broken:** `tpl help cfg database ad` says "under 'tpl cfg database'", and its hint is `tpl help cfg database`. The two paths are inconsistent.
- **Fix:** "unknown command 'tabls' under 'tpl schema'", with the hint "… list them with: tpl help schema".
- **Spec change:** none.
- **Where:** `src/error.rs:311`, `hint.rs` (UnknownCommand).

### E-11 — A value that starts with `-` is reported as an unknown flag, with noise suggestions · **medium** · clear
- **Observed:** `tpl cfg set core.query_timeout -5` → `unknown flag '-5'` … `did you mean '-V', '-d' or '-h'?`.
- **Broken:** FR-CLI-017 accepts `--`, and `tpl cfg set core.query_timeout -- -5` works. The hint does not mention it. Suggestions for one-letter short flags always match every short flag.
- **Fix:**
  - When the command takes a positional in that slot, add: "if '-5' is a value, write it after --: tpl cfg set core.query_timeout -- -5".
  - Suppress short-flag suggestions at a distance of 1 from another single letter.
- **Spec change:** none.
- **Where:** `hint.rs` (UnknownFlag), `src/diagnostics/suggest.rs`.

### E-12 — `cfg database add` with no connection flag calls the flags an "argument" · **medium** · clear
- **Observed:** `error: the command 'cfg database add' requires the argument '--dsn, or one of --host, --port, --user and --schema'`.
- **Fix:**
  - error: "tpl cfg database add needs connection details"
  - cause: "no --dsn, and none of --host, --port, --user, --schema, was given"
  - hint: "tpl cfg database add shop --host <host> --user <user> --schema <database>"
- **Spec change:** none.
- **Where:** `src/cli/cfg/entries.rs:95`.

### E-13 — `--dsn` value errors are generic, while the same fault in the file is specific · **medium** · explicit
- **Observed:** `--dsn http://h/d`, `--dsn mysql://` and `--dsn 'mysql://u@h/d?ssl=true'` all give "which takes a connection URL" and the hint "tpl help <command>". The file path (DsnMalformed, DsnQueryParameter) already says "mysql:// or mariadb://", "scheme://[user[:password]@]host[:port]/database" and "a DSN takes none [query parameters]".
- **Fix:** reuse the DsnFault causes for the flag, and use the hint "e.g. --dsn mysql://reader@db.example.com:3306/shop".
- **Spec change:** none.
- **Where:** `src/cli/intercept.rs` (MalformedValue for `--dsn`).

### E-14 — `cfg get` on an existing block says "is not set" · **medium** · honest
- **Observed:** `tpl cfg get database.shop` (the entry exists) → "configuration key 'database.shop' is not set", exit 66.
- **Fix:** "'database.shop' is a whole entry, not one value; show it with: tpl cfg database show shop".
- **Spec change:** **FR-CFG-007**, which only states the absent case, needs a stated condition and code for a block key (64 or 66).
- **Where:** `src/cli/cfg/keys.rs`.

### E-15 — EntryKeyMissing says "database" three times · **medium** · simple, clear
- **Observed:** `error: database entry 'noschema' does not carry 'database.noschema.database'` · `hint:  tpl cfg database update noschema --schema <database>`.
- **Fix:**
  - error: "database entry 'noschema' has no server database name (key database.noschema.database)"
  - hint: "set it with: tpl cfg database update noschema --schema <name>"
- **Spec change:** none.
- **Where:** `src/error.rs:1277`, `hint.rs`.

### E-16 — The TOML parse cause gives a position but no reason · **medium** · explicit
- **Observed:** `cause: the TOML parser stopped at line 38, column 6 of …/.cfg`.
- **Fix:** add the parser's message, e.g. "…: expected `]` to close the table header".
- **Spec change:** none. FR-ERR-034's 78 row asks for "the value found and the value expected".
- **Where:** `src/error.rs:1043`, `cause.rs`.

### E-17 — A known key in the wrong TOML table gets no suggestion · **medium** · explicit
- **Observed:** `password_timeout = 1` above `[core]` → "declares the unknown key 'password_timeout'", with no "did you mean 'core.password_timeout'?". The generated header's comment line "# The keys of [core]:" makes this mistake easy.
- **Fix:** let the nearest-match search try every table prefix.
- **Spec change:** none (FR-ERR-019).
- **Where:** `src/project/config.rs` (ConfigurationKeyOutsideSpace candidates).

### E-18 — "password_command yielded no exit status" is shown when the command never started · **low** · honest
- **Observed:** error line "password_command yielded no exit status", with the cause "could not be started: No such file or directory".
- **Fix:** error "password_command could not be started". Name the entry in every password_command error.
- **Spec change:** none.
- **Where:** `src/error.rs:1213`.

### E-19 — 77 hints are thin (source-read) · **medium** · explicit
- **AuthenticationRefused:** the hint only offers `--user`, although the usual cause is the password. Proposed: "check the password of entry 'shop' (password, password_command or the DSN), or test it with: tpl cfg database test shop".
- **PropertyNotReadable:** "request read access to the catalogue for this user". Proposed: name the privilege the property needs (e.g. SHOW VIEW for a view definition), and add "check with: tpl cfg database test shop".
- **Spec change:** none.
- **Where:** `src/diagnostics/hint.rs`.

### E-20 — The TLS handshake cause does not say what the handshake returned (source-read) · **medium** · explicit
- **Current:** "the TLS handshake with h:p did not complete, so no session was opened and no catalogue statement was issued".
- **Broken:** FR-ERR-034's 69 row asks for "what that phase returned", e.g. an untrusted certificate, a host-name mismatch, or no TLS on the server. The hint gives `<entry>` and `<mode>` with no values.
- **Fix:**
  - Carry the TLS error text on the variant.
  - Hint: "for a server without a trusted certificate: tpl cfg set database.shop.tls preferred; to trust a private CA: tpl cfg set database.shop.ca_file <pem>".
- **Spec change:** none. This is an existing requirement that is not met.
- **Where:** `src/error.rs:859`, `cause.rs`, `hint.rs`.

### E-21 — The syntax-error cause repeats the position, and `template check`'s hint points at itself · **low** · simple
- **Observed:** "at line 1, column 13 … stopped at line 1, column 13 reporting … (in t/syntax.jinja:1)". The hint "parse the project's templates with: tpl template check" is printed by `tpl template check` itself.
- **Fix:**
  - cause: "unexpected end of block"
  - hint: "fix line 1 of <root>/t/syntax.jinja, then run: tpl template check t/syntax"
- **Spec change:** none.
- **Where:** `cause.rs`, `hint.rs` (TemplateSyntax).

### E-22 — Hints for `.cfg` owner and mode use a relative path · **low** · honest
- **Observed:** run from `a/b` under the project root, the hint is `chmod 600 .tpl/.cfg`, which fails from that directory.
- **Fix:** use the absolute path: `chmod 600 '/abs/.tpl/.cfg'`, and the same for chown.
- **Spec change:** optional. The FR-PROJ-011 example shows the relative form.
- **Where:** `src/diagnostics/hint.rs` (ConfigurationNotOwned, ConfigurationUnsafeMode).

### E-23 — InvocationRejected uses category wording (source-read) · **low** · explicit
- **Current:** "the invocation was rejected" / "tpl does not classify the refusal further". FR-ERR-034 bans a cause that fits any failure.
- **Fix:** map the remaining clap error kinds, or at least quote the parser's message.
- **Spec change:** none.
- **Where:** `src/error.rs:461`, `cause.rs`.

### E-24 — Command names are written three ways in errors · **low** · consistency
- **Observed:** `the command 'template show'`, `'tpl template list'` and `'tpl cache load'`.
- **Fix:** always write the full `tpl …` form.
- **Spec change:** none.
- **Where:** `src/error.rs:471`.

### E-25 — `cfg database update <name>` with no flag exits 0 and changes nothing · **low** · honest
- **Fix:** exit 64: "nothing to change: give at least one of --dsn, --host, …".
- **Spec change:** **FR-CFG-020**, for the new condition.
- **Where:** `src/cli/cfg/entries.rs`.

### E-26 — Network errors do not name the entry · **low** · explicit
- **Observed:** "the server at 127.0.0.1:1 refused the connection" does not say entry 'refused'.
- **Fix:** append ", for database entry 'refused'". This also fixes the `<entry>` placeholder in E-07.
- **Spec change:** none.
- **Where:** `src/error.rs:837–870`.

### E-27 — ContextObjectNotFound gives no way to list names · **low** · explicit
- **Current:** "name an object the --context document carries, then run the command again".
- **Fix:** add "list them with: jq -r '.data.database.tables[].name' ctx.json", and suggest the nearest name from the document.
- **Spec change:** none.
- **Where:** `hint.rs`.

### E-28 — The NoDatabaseEntrySelected cause is wrong for `cache status` and `cache clean` · **low** · honest
- **Current:** "… and this command reads the catalogue through one".
- **Broken:** `cache status` and `cache clean` never read the catalogue.
- **Fix:** "… and this command needs an entry to know which database to use".
- **Spec change:** none.
- **Where:** `cause.rs`.

---

## 5. Totals

| Severity | Help | Errors | Total |
|---|---|---|---|
| high | 5 (H-01, H-02, H-03, H-04, H-07) | 6 (E-01 … E-06) | **11** |
| medium | 11 (H-05, H-06, H-08, H-09, H-10, H-11, H-12, H-13, H-15, H-17, H-18) | 13 (E-07 … E-17, E-19, E-20) | **24** |
| low | 7 (H-14, H-16, H-19, H-20, H-21, H-22, H-23) | 9 (E-18, E-21 … E-28) | **16** |
| **all** | 23 | 28 | **51** |

## 6. Findings that need, or offer, a spec change

| Finding | Requirement | Required or optional |
|---|---|---|
| H-02 | FR-ENV-005, FR-HELP-017 (only to publish context variables in JSON) | optional: the text-help part needs none |
| H-03 | FR-ENV-005 (signature and purpose per item); FR-HELP-006 / FR-HELP-007 (only for a new text section) | required for the JSON part |
| H-07 | FR-ERR-022 / FR-ERR-023 (admit `/` in template suggestions) | alternative to rewording the help |
| E-01 | FR-PROJ-025 / FR-ERR-035 (let `cfg set` / `unset` repair a value fault) | optional: a hint rewrite needs none |
| E-03 | FR-ERR-034, 78 row (cover the path `--tpl-dir` named) | recommended |
| E-08 | FR-ERR-034, 65 row (name the key path and type, not the rule of context-document.md) | required |
| E-14 | FR-CFG-007 (condition and code for a block key given to `cfg get`) | required |
| E-22 | FR-PROJ-011 example (absolute path) | optional |
| E-25 | FR-CFG-020 (update with no field flag) | required |
| H-10 | a new FR-HELP requirement to make "needs a server / writes files" mandatory | optional |

## 7. Observations outside this audit's scope (recorded, not judged as text)

- `cfg database add 'a.b' --host h` and `cfg database add 'bad name' --host h` are accepted. An entry name containing `.` cannot be addressed by dotted keys (`database.a.b.host`).
- `--tpl-dir <any existing folder>` is accepted as a project, and `cfg set` then creates `.cfg` in it.
- `-d <anything>` is accepted silently by commands that use no entry (`template list`, `init`, `help`).
