---
id: ADR-012
title: Continuous integration and release distribution
status: accepted
decided: 2026-09-24
last-reviewed: 2026-09-24
requirements: [NFR-PERF-018, NFR-PERF-005, NFR-PERF-007, FR-SRV-019, BR-PERF-008, FR-RND-039, FR-CONF-028, FR-CONF-031]
supersedes: []
superseded-by: null
---

# ADR-012 — Continuous integration and release distribution

## Status

Accepted, 2026-09-24. Decided by the user for rmp `#292`, as relayed by the
session coordinator, including the release gate, the version pins and the
archive contents. The provenance gates of Decision point 3 were decided by the
user for rmp `#293`, on the same day. Decision point 10 was decided by the user
for rmp `#267`, the on-demand triggers for rmp `#299`, and the skill asset and
its installer (Decision points 4 and 11) for rmp `#303`, on the same day.

This record takes over two parts of `ADR-008`: its refusal to prescribe a
pipeline, with the rejection that argued it, and its open question on the form
of the release artefact. `ADR-008` keeps the build path, which is not
superseded. The register's lifecycle defines no partial supersession, so
`ADR-008` was edited in place and this record's `supersedes` field is empty.

## Context

`NFR-PERF-018` fixes exactly four targets and makes none second class: a result
that fails on one of the four fails. `ADR-008` fixes how each is built:
`cargo-zigbuild` for the two `musl` targets, native builds for the two Darwin
targets.

`ADR-008` recorded that no pipeline existed and rejected prescribing one,
because a pipeline that did not exist would have been described as though it
did. The consequence was a set of obligations carried by hand: the five-command
validation sequence of the root coordination document, `NFR-PERF-018`'s
no-second-class rule across all four targets, and `FR-SRV-019`'s
re-verification of the supported-series table before every release. The same
record left the release artefact's form open.

**Admission under rule R4.** No requirement delegates this fact: the
specification-manager judged on 2026-09-24 that distribution is release
engineering and that `/specification` needs no requirement for it. The record is
admitted under R4's second limb. It reverses a rejection recorded in an accepted
record, and a reversal whose alternatives are not kept leaves the earlier
rejection as the only argument on file. `ADR-008` sets the precedent: packaging
and the build path were admitted on the same limb.

Two constraints shape what a pipeline can verify. `BR-PERF-008` gives no figure
the power to fail a change, so a pipeline owes nothing to a measurement. The
integration suite gates every server-dependent assertion on
`scripts/mariadb/status.sh`: exit `0` runs it, `1` skips it with a printed
reason, and `2` (half a fixture) fails the run. The table under *What an
assertion needs in order to run*, in `specification/performance-requirements.md`,
states which assertions need the fixture.

## Decision

**`tpl` is distributed through GitHub Actions. Every GitHub Release publishes
all the artefacts continuous integration produces.** There are two workflows,
and both run correctness validations only. **Both are triggered by
`workflow_dispatch` only**: no `push`, `pull_request` or tag trigger exists, and
nothing runs on GitHub in reaction to a push or a tag.

1. **`ci.yml` runs on demand, against a chosen ref**, for example before a
   release is cut: `gh workflow run ci.yml --ref release/X.Y.Z`. It runs the five
   commands of the root coordination document, in order:
   `cargo fmt --all -- --check`,
   `cargo clippy --all-targets --all-features -- -D warnings`,
   `cargo build --release`, `cargo test --all-features`, `cargo audit`. They run
   on all four targets of `NFR-PERF-018`, as a matrix with one entry per target.

2. **No MariaDB fixture runs in continuous integration.** Tests that need the
   database skip themselves when the fixture is absent, through the gate
   described in Context. **No benchmark or performance step runs.**

3. **A release is published only from a tag of a commit on `main`, pushed to
   GitHub, and only if that commit passes the five-command validation.**
   The tag is produced by the `gitflow` procedure, over the version bump,
   changelog and release notes the `release-manager` procedure writes; neither
   procedure is restated here. Under `gitflow`, `main` is pushed and then the
   annotated tag, and the release is then started with
   `gh workflow run release.yml --ref vX.Y.Z`. The run's `github.ref`,
   `github.ref_name` and `GITHUB_SHA` are therefore the tag and its commit
   (Sources). The gate job refuses a ref that is not a tag: `github.ref_type`
   must be `tag`. Before any validation or publishing, the workflow enforces
   four gates:
   1. the tagged commit is an ancestor of `origin/main`;
   2. the tag is annotated;
   3. the tag is `v` followed by a valid Semantic Versioning 2.0.0 version,
      checked with the numbered-capture-group regular expression semver.org
      publishes, and that version equals `version` in `Cargo.toml`;
   4. exactly one file's path matches `release-notes/<tag>-<YYYYMMDD>.md`,
      anchored, where `<tag>` is the full tag including any pre-release
      identifier and `<YYYYMMDD>` is exactly eight digits; that file becomes
      the body of the GitHub Release. For `v0.2.0`,
      `v0.2.0-rc.1-20260924.md` does not match; for `v0.2.0-rc.1`, it does.
      This extends the `release-manager` naming from `v<MAJOR.MINOR.PATCH>` to
      the full tag.

   The workflow then runs the five-command validation itself, or makes the
   publish job depend on it. If any gate or the validation fails, the workflow
   fails and publishes nothing. On success it builds the four targets and
   creates the GitHub Release. A version that carries a pre-release identifier,
   such as `v0.2.0-rc.1`, creates a release marked as a pre-release. The rule
   for the version number itself is `OD-03`'s, in
   `docs/spec-technical/open-decisions.md`, and is not restated here.

4. **The release carries one archive per target, one skill archive and one
   `SHA256SUMS` file.**
   Each archive is named `tpl-<tag>-<triple>.tar.gz`, for example
   `tpl-v1.2.0-aarch64-apple-darwin.tar.gz`, and contains the `tpl` binary,
   `README.md`, `LICENSE` and `CHANGELOG.md`. The skill archive,
   `tpl-skill-<tag>.tar.gz`, contains the `skill/` tree at the tagged commit.
   It is platform-independent and is built once, in the publish job, so the
   skill's version is the version of the binary it describes. `SHA256SUMS`
   covers all five archives. **No artefact is signed.** Signing is not done,
   not left open.
   The two Darwin archives are created with
   `tar --no-mac-metadata --no-xattrs`, so they carry no extended attributes
   (Sources).

5. **The build path is `ADR-008`'s, unchanged, and is not restated here.** Each
   target is built and tested by its own path in both workflows. The two `musl`
   targets go through `cargo-zigbuild`, which provides `zigbuild`, `clippy` and
   `test` subcommands (Sources). The two Darwin targets are built natively.
   **Both workflows pin `cargo-zigbuild` and zig to the versions `ADR-008`
   records**, and a change to either is a change to that record.

6. **The runner for each target follows from the build path, and no runner
   label is fixed here.** A native Darwin build needs a macOS runner.
   `cargo test` on a target executes that target's test binaries, which takes a
   Linux host for a `musl` target and a macOS host for a Darwin target, each
   able to run the target's architecture.
   GitHub-hosted runners exist for Linux and macOS on both x64 and arm64
   (Sources). The labels are an implementation detail of the workflow files.

7. **The CI toolchain is the MSRV.** Both workflows install the `rust-version`
   that `Cargo.toml` declares, whose value `ADR-007` fixes.

8. **Every downloaded tool is pinned and hash-checked.** CI installs the
   prebuilt `cargo-audit` 0.22.2. Each tool the workflows download
   (`cargo-zigbuild`, zig, `cargo-audit`) is checked against a SHA-256
   hard-coded in the workflow before it is extracted, and a mismatch fails the
   job. The `cargo-zigbuild` hashes come from the publisher's `.sha256` files
   and the zig hashes from `ziglang.org`'s `index.json`. RustSec publishes no
   checksum for `cargo-audit`: its hashes were computed from the official
   downloads, and they match the `digest` GitHub reports for each asset.

9. **`install.sh`, at the repository root, installs and updates `tpl`.** It is
   POSIX `sh`, and one command does both jobs:
   `curl -fsSL https://raw.githubusercontent.com/FlavioCFOliveira/tpl/main/install.sh | sh`,
   which the root `README.md` carries. The script:
   - maps the OS and CPU architecture to one of the four targets, and otherwise
     fails with a clear message and a non-zero exit;
   - resolves the latest GitHub Release's tag and compares it with the
     installed `tpl --version`; if they match, it says so and exits `0`
     without downloading anything;
   - otherwise downloads that release's archive for the target and its
     `SHA256SUMS`, verifies the archive, and installs `tpl` into `/usr/local/bin` on Linux and macOS, using
     `sudo` only when that directory is not writable. It creates the
     directory when it does not exist, again with `sudo` only when needed.
     `TPL_INSTALL_DIR` overrides the directory. It is a variable of the script,
     not of `tpl`, which never reads it.

10. **The release path runs correctness tests only.** The release path is the
    `cargo test --all-features` that both workflows run. A test whose verdict
    depends on timing, a wall-clock deadline, sampling or host load is a
    measurement test. It carries `#[ignore = "measurement: <reason>"]`, so
    `cargo test` never runs it, and it runs on demand with
    `cargo test --all-features -- --ignored`, outside both workflows.

11. **`install-skill.sh`, at the repository root, installs and updates the
    Claude Code skill.** It is POSIX `sh`, and runs as
    `curl -fsSL https://raw.githubusercontent.com/FlavioCFOliveira/tpl/main/install-skill.sh | sh`.
    It resolves the latest GitHub Release as `install.sh` does, downloads
    `tpl-skill-<tag>.tar.gz` and `SHA256SUMS`, and verifies the archive before
    it touches anything. The destination is
    `${TPL_SKILL_DIR:-${CLAUDE_CONFIG_DIR:-$HOME/.claude}/skills/tpl}`, the
    personal skills location, which loads in every project on the machine
    (Sources). An existing destination is removed and replaced; if it is a
    symbolic link, only the link is removed, never its target. The script never
    uses `sudo`, because the path belongs to the user. `TPL_SKILL_DIR` is a
    variable of the script, not of `tpl`.

## Alternatives rejected

- **Running the MariaDB fixture in continuous integration.** Rejected as
  heavier: the fixture's containers would be raised and torn down on every run.
  The assertions it would enable are carried by hand instead, as listed under
  Consequences.

- **Validating on one Linux host only.** Rejected because it breaks
  `NFR-PERF-018`'s no-second-class rule. A green run on one target says nothing
  about the other three, and the rule states that a failure on any of the four
  is a failure.

- **Publishing bare binaries instead of archives.** Rejected by the user in
  favour of one `.tar.gz` per target with a `SHA256SUMS` file.

- **An archive holding only the binary.** Rejected: the archive also carries
  the readme, the licence and the changelog.

- **Publishing from any `v*` tag on any commit.** Rejected: a tag on a branch
  other than `main`, a lightweight tag, or a tag that disagrees with the
  manifest would each publish.

- **Matching release notes with the glob `<tag>-*.md`.** Rejected: it
  collides across tags, since `v0.2.0-*.md` also matches the notes of
  `v0.2.0-rc.1`.

- **Exempting pre-releases from gate 4.** Rejected: a gate does not vary with
  the kind of tag.

- **GitHub's source tarball as the skill's release asset.** Rejected:
  `SHA256SUMS` does not cover it.

- **Installing the skill from the `main` branch.** Rejected: `main` is not a
  release, and it drifts.

- **The project's `.claude/skills/` as the skill's destination.** Rejected: it
  loads only in that repository, not everywhere.

- **Keeping an existing skill, or merging into it.** Rejected: the user wants
  a replacement.

- **Skipping the skill install when the version is unchanged.** Rejected: not
  asked for.

- **Retrying a measurement test, or tuning its thresholds.** Rejected: the rule
  forbids measurement on the release path, and a retry or a wider threshold
  keeps it there.

- **Checking the peak at the end of the render.** Rejected here: it needs
  `ADR-011` and `FR-RND-039` changed. It is recorded as backlog rmp `#297`.

- **A separate measurement workflow.** Rejected: the workflows stay very simple
  and correctness-only.

- **Accepting any `v*` string as a version.** Rejected: the tag would not be
  guaranteed to be a version at all.

- **Publishing a pre-release as a stable release.** Rejected: it would become
  the latest release, and `install.sh` would install it.

- **Keeping the `push`, `pull_request` and tag triggers.** Rejected by the user
  as a waste of resources.

- **A `tag` input to `release.yml` instead of `--ref`.** Rejected: it
  duplicates the ref and opens a gap between the input and the commit that is
  checked out.

- **`repository_dispatch`.** Rejected as less simple: it needs a token and a
  raw API call.

- **Triggering on a push to `main` and reading the tag from `HEAD`.** Rejected
  as fragile: under `gitflow` the tag is pushed after `main`, so the `main` push
  runs before the tag exists on GitHub.

- **Reopening rmp `#292` for the provenance gates.** Rejected: they are
  recorded under rmp `#293`.

- **No release gate, with a person checking `ci.yml` before tagging.**
  Rejected: a check a person can forget would let a failing commit become a
  release.

- **Running clippy on stable in CI.** Rejected: CI validates the toolchain the
  crate declares as its floor.

- **An unpinned or latest `cargo-audit`.** Rejected: the validation tool would
  drift.

- **Pinning downloads by URL only.** Rejected: a URL fixes where a file comes
  from, not what it contains.

- **`~/.local/bin` as the default install directory.** Rejected: it is not
  always on `PATH`, for example on macOS.

- **Failing when the install directory is missing.** Rejected in favour of
  creating it.

- **`--no-mac-metadata` alone for the Darwin archives.** Rejected: on bsdtar it
  drops the AppleDouble data but still writes the extended attributes as pax
  headers (Sources).

- **A checksum-only script that always reinstalls.** Rejected in favour of a
  version check that exits `0` when nothing needs installing.

- **A two-step download-then-run example in the README.** Rejected in favour of
  the one-line command.

- **Installing the latest `cargo-zigbuild` and zig in CI.** Rejected: the build
  path would drift from the one `ADR-008` records, which `ADR-008` says breaks
  the comparability of every recorded figure.

- **Keeping `ADR-008`'s refusal to prescribe a pipeline.** That refusal rested
  on the pipeline being aspiration. With the user's decision, the pipeline is
  the chosen mechanism for two of the obligations `ADR-008` listed as manual,
  and recording it is the accurate statement.

## Consequences

**Nothing validates a pushed commit automatically.** `ci.yml` runs the
five-command sequence on all four targets only when someone dispatches it.
`release.yml` runs the same validation on the tagged commit before it
publishes, so no release escapes it. The root coordination document's rule that
work is not complete until the five commands pass is not changed by this
record.

**The server-side checks remain an obligation carried by hand.** Without a
fixture, `ci.yml` skips every assertion observed through the server's statement
record or connection record. In particular it skips:

- the connection clause of `NFR-PERF-005`, which that requirement makes
  verifiable on every target of `NFR-PERF-018` from the server side; and
- the two server-side instruments of `NFR-PERF-007`, the statement record and
  the connection record, which that requirement binds to all four targets.

A green `ci.yml` run therefore does not verify these checks. Whoever prepares a
release runs them against the fixture, through its harness, on every target.

**`FR-SRV-019` stays manual, and it is done before tagging.** The supported-series
table is re-verified against its source before the `v*` tag is created, so that
any change it forces is in the tagged commit.

**The file-open observation of `NFR-PERF-005` depends on the runner image.** On
the two Linux targets, the suite skips that assertion with a printed reason when
the host has no `strace`. Whether the GitHub-hosted Linux images provide it is
**unverified**. The differential runs of `NFR-PERF-007` need no server and run in
`ci.yml` on all four targets.

**A pushed tag publishes nothing until `release.yml` is dispatched.** Only
`release.yml`'s gated validation decides whether a release is published.

**Dispatch after `main` is pushed.** If `release.yml` is dispatched before
`main` reaches GitHub, gate 1 fails and nothing is published. The remedy is to
dispatch it again after `main` is pushed; gate 1 reads `origin/main` as it
stands when the gate runs.

**The workflow files must be on the default branch.** `workflow_dispatch`
triggers a run only if the workflow file exists on the default branch, and the
run uses the workflow file as it stands at the dispatched ref (Sources). A tag
therefore runs the `release.yml` its own commit carries.

**`install.sh` never installs a pre-release.** GitHub's latest release is the
most recent release that is neither a draft nor a pre-release, and a
pre-release cannot be set as latest (Sources). `/releases/latest`, which
`install.sh` follows, therefore never resolves to one. A pre-release is
installed only by hand.

**Some requirements lose their automatic check on the release path.** The
measurement tests cover the memory limit of `FR-RND-039`, the deadlines of
`FR-CONF-028` and `FR-CONF-031`, and the render deadlines. They are checked by
hand, on demand. The evidence for the rule comes from rmp `#267`:
`fr_conf_045_raising_the_memory_limit_lets_a_legitimately_large_render_pass`
let a render escape the 16 MiB limit in 3 of 300 runs on an idle host and 29 of
400 under load. The limit is enforced by sampling (`ADR-011`, Decision point 2,
at the interval `docs/spec-technical/architecture.md` fixes), and
`FR-RND-039` acts on the observed count, so the escape is not a product
defect.

**Gate 4 matches the tag literally.** A tag carries `.` and may carry `+`,
which are metacharacters in a regular expression, so the tag is compared as a
literal string and only the date part as eight digits.

**Gates 1 and 2 need more than the default checkout.** `actions/checkout`
fetches a single commit by default, so the ancestry check needs `main`'s
history. An open `actions/checkout` issue reports that tag annotations are not
preserved, so gate 2 cannot trust the tag object the checkout leaves locally
(Sources). How the workflow meets both is an implementation detail.

**The checksum file proves integrity, not origin.** `SHA256SUMS` lets a reader
check an archive against the release page it came from. Because nothing is
signed, nothing in the release proves who produced it. GitHub also attaches the repository's source archives to every release
automatically (Sources). `release.yml` does not produce them, and `SHA256SUMS`
does not cover them.

**A lint the floor raises fails CI.** Clippy's lints differ between toolchains,
so CI can flag code that clippy on stable accepts. The first run
on 1.94.0 did: it flagged `clippy::nonminimal_bool` (Sources).

**The hash pins are only as good as their origin.** A hard-coded hash proves a
download is the file that was hashed. For `cargo-zigbuild` and zig it is the
publisher's figure. For `cargo-audit` it is this project's own computation,
cross-checked against GitHub's asset `digest` and attested by no publisher.
Moving any tool version means replacing its hashes.

**`curl | sh` trusts the `main` branch and TLS.** Whoever can change
`install.sh` on `main`, or intercept the download, controls what runs. The
checksum proves the archive matches the release's `SHA256SUMS`. Because nothing
is signed, it does not prove who produced either file. `install-skill.sh`
follows the same trust path.

**`install-skill.sh` works only from the first release that carries the skill
archive.** `v0.0.1` does not carry it.

**A symlinked skill is replaced by a copy.** Claude Code documents that a
personal skill entry can be a symbolic link to a directory elsewhere (Sources),
which is how `skill/README.md` suggests linking the repository. The script
removes such a link and installs a copy, leaving the link's target untouched.

**The replacement reloads live, with one exception.** Claude Code picks up an
added, edited or removed skill under the personal skills directory without a
restart; if that top-level directory did not exist when the session started, a
restart is needed (Sources).

**Not fixed by this record:** the runner labels; whether `release.yml` runs the
validation itself or depends on a validation job.

**No performance figure is produced or consumed.** This is consistent with
`BR-PERF-008`, and `NFR-PERF-012`'s attribution of a figure to a target and a
build path is unaffected, because continuous integration records no figure.

**Under R3, this decision lives here alone.** `docs/spec-technical/` cites
`ADR-012` for the pipeline and the release artefact, and cites `ADR-008` only for
the build path.

## Sources

| Claim | Source | Consulted |
|---|---|---|
| The decision, its rejected alternatives, and the build-path constraint; the release gate, the version pins, the archive name and contents, and the absence of signing | The user's decisions of 2026-09-24, relayed for rmp `#292` | 2026-09-24 |
| On bsdtar 3.5.3 (libarchive 3.7.4), macOS 26 development host: `tar -czf` with `--no-mac-metadata` still writes a test attribute and `com.apple.provenance` as `LIBARCHIVE.xattr` and `SCHILY.xattr` pax headers, and extraction restores `com.apple.provenance`; adding `--no-xattrs` leaves no attribute in the archive | Probe archive of one file carrying a test attribute, inspected with `strings` | 2026-09-24 |
| The tar on the macOS runners behaves the same | unverified; untested | — |
| The four provenance gates, their order, the release-notes body, and the three rejected alternatives | The user's decision of 2026-09-24, relayed for rmp `#293` | 2026-09-24 |
| Gate 4's anchored `release-notes/<tag>-<YYYYMMDD>.md` pattern and its two rejected alternatives | The user's decision of 2026-09-24, relayed for rmp `#293` | 2026-09-24 |
| The measurement-test rule, its three rejected alternatives, and the escape counts 3/300 idle and 29/400 under load | The user's decision of 2026-09-24 and the findings of rmp `#267`, relayed; counts not re-run | 2026-09-24 |
| `#[ignore]` accepts a reason in the name-value form; `--ignored` runs only ignored tests | The Rust Reference, *Testing attributes*, `ignore`; *The rustc book*, *Tests*, `--ignored` | 2026-09-24 |
| SemVer as the default versioning rule; gate 3's SemVer check; pre-release tags published as pre-releases; the two rejected alternatives | The user's decision of 2026-09-24, relayed for rmp `#293` | 2026-09-24 |
| The numbered-capture-group regular expression for a Semantic Versioning 2.0.0 version, compatible with ECMAScript, PCRE, Python and Go | semver.org, *Semantic Versioning 2.0.0*, FAQ "Is there a suggested regular expression (RegEx) to check a SemVer string?"; the same text in GitHub `semver/semver`, `semver.md` | 2026-09-24 |
| The latest release is "the most recent non-prerelease, non-draft release"; "Drafts and prereleases cannot be set as latest"; `releases/latest` links to the latest release | docs.github.com, REST API *Releases*, "Get the latest release" and "Create a release" (`make_latest`); *Linking to releases* | 2026-09-24 |
| The `workflow_dispatch`-only triggers, the dispatch commands, the `ref_type` refusal, and the three rejected alternatives | The user's decision of 2026-09-24, relayed for rmp `#299` | 2026-09-24 |
| The skill archive, `install-skill.sh`, its destination, replacement and symlink rules, and the five rejected alternatives | The user's decision of 2026-09-24, relayed for rmp `#303` | 2026-09-24 |
| Personal skills live at `~/.claude/skills/<skill-name>/SKILL.md` and load in "all your projects on this machine"; a skill entry can be a symlink to a directory elsewhere; changes under `~/.claude/skills/` are picked up in the current session, except for a top-level skills directory created after the session started | code.claude.com, `docs/en/skills.md`, *Choose where skills load* and *Edit a skill during a session* | 2026-09-24 |
| With `CLAUDE_CONFIG_DIR` set, every `~/.claude` path lives under that directory instead; its default is `~/.claude` | code.claude.com, `docs/en/claude-directory.md` and `docs/en/env-vars.md`, `CLAUDE_CONFIG_DIR` | 2026-09-24 |
| `v0.0.1` is the only tag | `git tag -l` in this repository | 2026-09-24 |
| For `workflow_dispatch`, `GITHUB_REF` is the branch or tag that received the dispatch and `GITHUB_SHA` the last commit on it; the event triggers a run only if the workflow file exists on the default branch | docs.github.com, *Events that trigger workflows*, `workflow_dispatch` | 2026-09-24 |
| `github.ref` is `refs/tags/<tag_name>` for a tag; `github.ref_type` is `branch` or `tag` | docs.github.com, *Contexts reference*, `github` context | 2026-09-24 |
| `gh workflow run --ref` names the "branch or tag name which contains the version of the workflow file you'd like to run" | GitHub `cli/cli`, `pkg/cmd/workflow/run/run.go` | 2026-09-24 |
| The default branch of `FlavioCFOliveira/tpl` is `main` | GitHub REST API, `repos/FlavioCFOliveira/tpl` | 2026-09-24 |
| `actions/checkout` fetches a single commit by default; `fetch-depth: 0` fetches all history | GitHub `actions/checkout`, `README.md` | 2026-09-24 |
| Tag annotations are not preserved by `actions/checkout`; the issue is open | GitHub `actions/checkout`, issue `#290`, *Preserve tag annotations* | 2026-09-24 |
| `README.md`, `LICENSE` and `CHANGELOG.md` exist at the repository root | Repository listing | 2026-09-24 |
| The CI toolchain, the `cargo-audit` pin, the download hash checks and their origins, and `install.sh` with its rejected alternatives | The user's decisions of 2026-09-24, relayed for rmp `#292` | 2026-09-24 |
| `rust-version` is `1.94.0` in `Cargo.toml` | `Cargo.toml` | 2026-09-24 |
| Clippy on 1.94.0 flagged `nonminimal_bool` in `src/render/fault.rs` | Relayed for rmp `#292`; not re-run | 2026-09-24 |
| `cargo-audit` 0.22.2 is the latest release (published 2026-06-05); its prebuilt assets carry no checksum file, and GitHub reports a `sha256` `digest` for each. That the computed hashes match those digests is relayed, not re-computed here | GitHub API, `rustsec/rustsec` releases, tag `cargo-audit/v0.22.2`; crates.io API, `cargo-audit` | 2026-09-24 |
| `cargo-zigbuild` v0.23.4 publishes a `.sha256` file beside each archive | GitHub API, `rust-cross/cargo-zigbuild` releases, tag `v0.23.4` | 2026-09-24 |
| `ziglang.org/download/index.json` carries a `shasum` per 0.16.0 tarball | `https://ziglang.org/download/index.json`, key `0.16.0` | 2026-09-24 |
| `/specification` needs no requirement for distribution; the connection clause of `NFR-PERF-005` and the server-side checks of `NFR-PERF-007` skip without a fixture; `FR-SRV-019` stays manual | Findings of the specification-manager, relayed for rmp `#292` | 2026-09-24 |
| `jobs.<job_id>.strategy.matrix` defines a matrix of job configurations | docs.github.com, *Workflow syntax for GitHub Actions* | 2026-09-24 |
| GitHub-hosted runners exist for Linux x64, Linux arm64, macOS Intel and macOS arm64 | docs.github.com, *GitHub-hosted runners*, supported runners table (`github/docs`, `data/reusables/actions/supported-github-runners.md`) | 2026-09-24 |
| Releases are based on Git tags and carry binary files; GitHub automatically adds a zip file and a tarball of the repository at the tag | docs.github.com, *About releases* | 2026-09-24 |
| `cargo-zigbuild` 0.23.4 provides the `zigbuild`, `clippy` and `test` subcommands | GitHub `rust-cross/cargo-zigbuild`, tag `v0.23.4`, `src/bin/cargo-zigbuild.rs` | 2026-09-24 |
| Server-dependent tests are gated on `status.sh`: `0` runs, `1` skips with a printed reason, `2` fails; the file-open trace skips with a reason off Linux and on a Linux host without `strace` | `tests/outside_the_process.rs`, module documentation and the `nfr_perf_007` file-open test; `scripts/mariadb/status.sh` | 2026-09-24 |
| Whether the GitHub-hosted Linux images include `strace` | unverified | — |
| Which assertions need the fixture; the connection clause of `NFR-PERF-005`; the four instruments of `NFR-PERF-007` and their targets; the four targets and the no-second-class rule | `specification/performance-requirements.md`, *What an assertion needs in order to run*, `NFR-PERF-005`, `NFR-PERF-007`, `NFR-PERF-018` | 2026-09-24 |
| No figure fails, blocks, rejects or gates a change | `specification/performance-requirements.md`, `BR-PERF-008` | 2026-09-24 |
| The supported-series table is re-verified against its source before every release | `specification/server-contract.md`, `FR-SRV-019` | 2026-09-24 |
