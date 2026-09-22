---
id: ADR-002
title: The TLS mode mapping onto the database driver
status: accepted
decided: 2026-09-10
last-reviewed: 2026-09-22
requirements: [FR-CONF-013, FR-CONF-014, FR-CONF-036, FR-CONF-037, FR-CONF-038, FR-CONF-039, FR-CONF-044]
supersedes: []
superseded-by: null
---

# ADR-002 — The TLS mode mapping onto the database driver

## Status

Accepted, 2026-09-10.

## Context

`FR-CONF-013` fixes five values for the `tls` key of a database entry —
`disabled`, `preferred`, `required`, `verify-ca`, `verify-identity` — defaulting
to `verify-identity`, on the ground that encrypting and verifying are different
guarantees and must have different names. `FR-CONF-038` then fixes the
**observable behaviour** of each mode, cell by cell, against a server that
offers TLS and a server that does not, verified against running servers on
2026-09-10.

**`FR-CONF-038` delegates the mapping to this register.** In its own words the
mapping onto the chosen driver "SHALL be recorded in the project's architecture
decision records and cited from there, and SHALL NOT be restated in this
corpus", for the reason `BR-SRV-005` gives about the supported-series table and
`FR-ENV-003` about the engine pin. The corpus names no driver, and the behaviour
it fixes is what a caller can observe whichever one is chosen. **This record is
that home.**

Three further requirements constrain what the mapping may do. `FR-CONF-036`
makes the five modes normative over the driver: one that cannot express all five
distinctly is disqualified, and the choice may not be settled by reducing the
mode set to fit a candidate. `FR-CONF-037` requires the mode to be set
explicitly on every connection, forbidding reliance on the driver's default for
any mode including `disabled`. `FR-CONF-039` requires that trust material
supplied by `ca_file` or `ca_path` be **additional** to what the TLS
implementation already trusts, and forbids describing `verify-ca` or
`verify-identity` as exclusive trust in the supplied authority.

**`FR-CONF-014` names a key the driver has no mechanism for.** It makes
`ca_file` **and** `ca_path` supply the trust material the two validating modes
use, and says nothing about how either reaches a driver, because the corpus
names none. `ca_file` maps straight onto a driver method. `ca_path` does not:
the driver affords a file and a byte buffer, and no directory. The mapping of
that key onto the driver is the same kind of fact as the mapping of the five
modes, missing for the same reason, and it is held here for the same reason.

**What the directory holds is the corpus's, and it moved after this record first
described it.** The thirty-second edition amended `FR-CONF-014` to settle two
questions it had left open — an entry is resolved **through symbolic links** and
read at its target, and an entry that cannot be resolved is reported against its
own path in the directory rather than passed over — and added `FR-CONF-044`,
which refuses a `ca_path` that yields no regular file. This record's earlier
wording described the opposite behaviour and named no requirement as its author,
which made it a defect under rule R2 rather than a difference of emphasis. Both
requirements are cited below and neither is restated: what an entry is, and what
an absent one costs, are the corpus's to fix, and only the hand-off to the driver
is this record's.

**The driver choice itself is `ADR-003`.** That record holds the driver, the
rule that settled it — `FR-CONF-036`, which disqualified the candidate that
could not express all five modes — and the alternatives weighed. This record
maps the five modes onto the driver that record pins, and restates none of it.

`NFR-DET-001` and `BR-CLI-002` bear on the two remaining questions: where the
trust anchors come from, and in what order supplied material is assembled.
`BR-CLI-002` states that two identical command lines run in two different
shells, against the same project state, cannot read different databases.

## Decision

**Each of the five modes maps to exactly one named variant of
`sqlx::mysql::MySqlSslMode`:**

| `tls` mode (`FR-CONF-013`) | Driver variant |
|---|---|
| `disabled` | `MySqlSslMode::Disabled` |
| `preferred` | `MySqlSslMode::Preferred` |
| `required` | `MySqlSslMode::Required` |
| `verify-ca` | `MySqlSslMode::VerifyCa` |
| `verify-identity` | `MySqlSslMode::VerifyIdentity` |

The mapping is one-to-one and total in both directions: five modes, five
variants, no mode collapsed onto another and no variant left unnamed. That is
what `FR-CONF-036` requires of the driver, and it is why this driver satisfies
it where the rejected candidate did not.

**Trust anchors are `webpki-roots`, bundled.** The crate supplies a compiled-in
copy of the root certificates trusted by Mozilla, embedded in the binary at
compile time rather than read from the operating system trust store (docs.rs,
verified 2026-09-10). `verify-ca` and `verify-identity` therefore behave
identically on every host and on all four supported targets.

**Trust material supplied by `ca_file` or `ca_path` is added to the bundled
roots, never substituted for them.** This is the behaviour `FR-CONF-039`
requires, and it is a property of how the crates build their root store rather
than a choice made here.

**`ca_path` reaches the driver as one bundle `tpl` assembles.** The driver takes
its trusted authorities through `ssl_ca`, which names a file, or
`ssl_ca_from_pem`, which takes PEM bytes; no method takes a directory (docs.rs,
`sqlx` 0.9.0, `sqlx::mysql::MySqlConnectOptions`, verified 2026-09-21).
`FR-CONF-014` makes `ca_path` supply trust material all the same, so `tpl` reads
both keys itself and hands the driver **one PEM bundle** through
`ssl_ca_from_pem`, composed in this order:

1. the bytes of `ca_file`, where the entry declares one;
2. then the entries of the `ca_path` directory that contribute, sorted into
   ascending path order — over the names the directory holds, and never over
   what those names resolve to — before any of them is resolved or read.

**Which entries contribute, and what an entry that cannot be resolved costs, is
`FR-CONF-014`'s and is not restated here**; nor is `FR-CONF-044`'s refusal of a
`ca_path` that yields no regular file. This record holds only the hand-off: that
the material is composed into one buffer at all, in that order, and reaches the
driver through `ssl_ca_from_pem`.

Each block ends before the next begins, so that a certificate whose last line
carries no newline cannot run into the block after it. The seam between
`ca_file` and the first block taken from `ca_path` is a seam like any other, and
is terminated like any other.

**The bundle is assembled only under `verify-ca` and `verify-identity`.** Those
are the two modes `FR-CONF-014` names. Under the other three the driver ignores
the material it is given — `FR-CONF-038` records `required` ignoring supplied
trust material as one of the three controls that separate the modes — and
reading a path whose contents cannot reach the connection would turn an
unreadable path into a failure of a mode that would never have looked at it.

**The mode is set explicitly on every connection `tpl` opens.** The driver's own
default is `MySqlSslMode::Preferred` (docs.rs, verified 2026-09-10), which
`FR-CONF-037` forbids relying on. The default is never reached, for any mode,
including `disabled`.

## Alternatives rejected

- **The platform trust store.** Refused because it makes the outcome of a
  connection a property of the machine rather than of the invocation. The same
  command could accept a certificate on one host and refuse it on another, with
  nothing in the invocation to say why — which is precisely what `BR-CLI-002`
  exists to prevent, and it would put the byte-identical stdout of
  `NFR-DET-001` at the mercy of the host's certificate configuration. A bundled
  root set is also the configuration the driver spike was measured under, so
  the decision keeps the evidence and the shipped behaviour aligned.

- **A configurable choice between the bundled set and the platform store.**
  Refused because it cannot be built from this side. It needs a configuration
  key, `FR-CONF-002` closes the key space, and `FR-CONF-034` makes an
  unrecognised key fatal. Introducing one would be a functional change, and an
  architecture decision may not make one.

- **Handing the directory to the driver.** Unavailable rather than refused. Both
  CA methods store the same type, whose only two shapes are a path to a file and
  an inline byte buffer (`vendor/sqlx-core-0.9.0/src/net/tls/mod.rs`, read
  2026-09-21). There is no third shape to reach for, and that is what makes the
  assembly `tpl`'s work rather than the driver's.

- **Supporting `ca_file` and refusing `ca_path`.** Refused for the reason the
  configurable trust store is refused above: it cannot be built from this side.
  `FR-CONF-014` names both keys and `FR-CONF-002` closes the key space, so
  dropping one is a functional change, and an architecture decision may not make
  one. The absence of a driver mechanism is a fact about the driver, and
  `FR-CONF-036` already settles which of the two yields when they disagree.

- **Selecting the directory's entries on the kind of the entry itself, so that a
  symbolic link never contributes.** Refused, and refused by the corpus rather
  than here: `FR-CONF-014` names this option and rejects it, on the ground that a
  `CApath` is conventionally a directory of links and that skipping them yields
  an empty bundle which nothing reports. This record described that behaviour as
  built between 2026-09-21 and this amendment, which is the defect this pass
  closes; the requirement carries the reasoning and it is not reproduced here.

- **Taking the directory's files in the order the directory yields them.**
  Refused. The order a directory read returns entries in "is platform and
  filesystem dependent" and "can change between calls", and the same
  documentation states the remedy — "if reproducible ordering is required, the
  entries should be explicitly sorted" (Rust standard library, `std::fs::read_dir`,
  verified 2026-09-21). One configuration would otherwise produce different
  bundles on two runs of one command, or on two hosts holding the same
  certificates, which is the ground on which the platform trust store is refused
  above, arriving through the filesystem instead of through the machine.

## Consequences

**`verify-ca` and `verify-identity` are reproducible across hosts and targets.**
Two identical command lines against the same project state reach the same
verdict on the same certificate wherever they run. This is the property
`BR-CLI-002` asks for, obtained by removing the host from the decision entirely.

**`FR-CONF-039`'s limit stands, and it is weaker than the words suggest.**
Because supplied material is added rather than substituted, pinning a private
authority **widens** the set of certificates that pass `verify-ca` instead of
narrowing it. Under `verify-ca`, a server presenting a certificate issued by any
authority in the bundled Mozilla set is accepted, whether or not the user
supplied one of their own. Neither driver measured can express "trust only this
authority"; the limitation is symmetric between them. `verify-ca` and
`verify-identity` must therefore never be documented or reported as exclusive
trust in the supplied authority.

**An unset mode is a fifth, unnamed mode, and it fails in the dangerous
direction.** If a connection is ever opened without the mode being set, the
driver's `Preferred` default takes effect: it upgrades where the server
advertises TLS and falls back to plaintext in silence where it does not. That is
`disabled` wearing the appearance of `required`. `BR-CONF-001` makes `tls` the
sole authority on encryption for an entry, and an inherited default would move
that authority out of the configuration without anything in the configuration
changing — and would move again on a dependency upgrade. Setting the mode
explicitly on every connection is what keeps the authority where the
configuration puts it.

**One configuration is one bundle, on every run and on every host.** The
directory's own order stops being an input: what the driver is handed is a
function of `ca_file`, of the names the `ca_path` directory holds, and of what
those names resolve to and carry — and of nothing else. That is the same
property the bundled root set
delivers for the anchors, obtained the same way — by removing from the decision
everything the invocation does not state.

**What the assembly obliges of the code.** Every block the bundle carries must
end before the next begins, or two certificates merge into one block that parses
as neither. The obligation covers every seam, including the one between the two
keys, which is the seam a fixture ending in a newline hides;
`src/mariadb/connect.rs` carries a test named for this record that asserts it
against a `ca_file` whose last byte is not a newline.

**Sorting before resolving is what keeps the bundle a function of the
configuration.** The order is fixed over the directory's own names, so it cannot
be changed by what a name points at, by a link being repointed, or by the order
the filesystem happens to yield. Two names resolving to one certificate
contribute it twice, which `FR-CONF-014` states and no requirement forbids.

**A bundled root set ages with the binary.** Refreshing the trusted roots
requires a dependency update and a rebuild, which is the accepted cost of
removing the host from the decision. The vendor documents this trade-off in the
same terms, recommending a platform verifier for applications that cannot be
readily rebuilt and redeployed; `tpl` is a small, frequently rebuilt CLI, which
is the case the bundled set is documented as suiting.

**This record pins no TLS crate version.** `BENCHMARKS.md` records `rustls`
0.23.44, `ring` 0.17.14 and `webpki-roots` 1.0.9 in the driver spike of
2026-09-10. Those are evidence of what was measured, not pins this decision
fixes; the decision names the *source* of the trust anchors, not a version of
the crate that supplies them.

**The driver-choice citation is discharged.** This record's Context cited
`FR-CONF-036` and `BENCHMARKS.md` directly because no record for the driver
choice existed. `ADR-003` now holds it, and the citation was repointed in the
commit that created that record. Moving the pin `ADR-003` holds obliges a
re-check of the five-variant mapping above, and `ADR-003` carries that
obligation on its own side.

**Under R3, the mapping lives here alone, and so does the assembly.**
`docs/spec-technical/security.md` cites `ADR-002` for the mapping rather than
restating it, and `docs/spec-technical/open-decisions.md` entry `OD-16` reduces
to a citation of this record. `FR-CONF-014` states that both keys supply the
material and says nothing about how, which is what it should say: the corpus
names no driver, and the assembly exists only because the driver `ADR-003` pins
has no mechanism for a directory. No other document of this repository states
the composition, and none may. The **resolution** rule and the
empty-directory refusal run the other way: they were the corpus's from the
thirty-second edition, so this record cites `FR-CONF-014` and `FR-CONF-044` for
them and holds neither.

## Sources

| Claim | Source | Consulted |
|---|---|---|
| `MySqlSslMode` has exactly five variants: `Disabled`, `Preferred`, `Required`, `VerifyCa`, `VerifyIdentity` | docs.rs, `sqlx` 0.9.0, `sqlx::mysql::MySqlSslMode` | 2026-09-10 |
| `Preferred` is the driver default when `ssl_mode` is not specified, and falls back to an unencrypted connection where an encrypted one cannot be established | docs.rs, `sqlx` 0.9.0, `sqlx::mysql::MySqlSslMode` | 2026-09-10 |
| `VerifyCa` verifies the server CA certificate against the configured CA certificates; `VerifyIdentity` additionally performs host name identity verification | docs.rs, `sqlx` 0.9.0, `sqlx::mysql::MySqlSslMode` | 2026-09-10 |
| `webpki-roots` supplies a compiled-in copy of the root certificates trusted by Mozilla, embedded at compile time rather than read from the OS trust store | docs.rs, `webpki-roots` 1.0.9, crate documentation | 2026-09-10 |
| The vendor recommends a platform verifier instead for applications that cannot be readily recompiled and redeployed | docs.rs, `webpki-roots` 1.0.9, crate documentation | 2026-09-10 |
| The driver, the rule that settled the choice, and the candidate it disqualified | `ADR-003` | 2026-09-11 |
| `rustls` 0.23.44, `ring` 0.17.14, `webpki-roots` 1.0.9 linked by both candidates in the spike | `BENCHMARKS.md`, "2026-09-10 — MariaDB driver selection", Crates | 2026-09-10 |
| The behaviour of all five modes against a TLS-offering and a TLS-less server, verified cell by cell against running servers | `specification/configuration-model.md`, `FR-CONF-038` | 2026-09-10 |
| The driver takes trusted authorities as `ssl_ca`, naming a file, or `ssl_ca_from_pem`, taking PEM bytes, and no method takes a directory | docs.rs, `sqlx` 0.9.0, `sqlx::mysql::MySqlConnectOptions` | 2026-09-21 |
| Both CA methods store one type whose only two shapes are `File(PathBuf)` and `Inline(Vec<u8>)` | `vendor/sqlx-core-0.9.0/src/net/tls/mod.rs`, `CertificateInput` | 2026-09-21 |
| A directory read's order "is platform and filesystem dependent", "can change between calls", and reproducible ordering requires the entries to be explicitly sorted | Rust standard library documentation, `std::fs::read_dir` | 2026-09-21 |
| An entry of `ca_path` is resolved through symbolic links and read at its target; an entry that cannot be resolved is reported against its own path in the directory | `specification/configuration-model.md`, `FR-CONF-014`, as the thirty-second edition amended it | 2026-09-22 |
| A `ca_path` that yields no regular file is refused before anything is contacted | `specification/configuration-model.md`, `FR-CONF-044` | 2026-09-22 |
| Both keys are read under the two validating modes alone, `ca_file` first and then the directory's entries sorted by the directory's own names before any is resolved, each block terminated, and the bundle handed over through `ssl_ca_from_pem` | `src/mariadb/connect.rs`, `trust` and `options`, with the seven tests named for `FR-CONF-014`, `FR-CONF-044` and this record | 2026-09-22 |
</content>
