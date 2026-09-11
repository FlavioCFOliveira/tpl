---
id: ADR-002
title: The TLS mode mapping onto the database driver
status: accepted
decided: 2026-09-10
last-reviewed: 2026-09-11
requirements: [FR-CONF-013, FR-CONF-014, FR-CONF-036, FR-CONF-037, FR-CONF-038, FR-CONF-039]
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

**The driver choice itself is `ADR-003`.** That record holds the driver, the
rule that settled it — `FR-CONF-036`, which disqualified the candidate that
could not express all five modes — and the alternatives weighed. This record
maps the five modes onto the driver that record pins, and restates none of it.

`NFR-DET-001` and `BR-CLI-002` bear on the remaining question of where the trust
anchors come from. `BR-CLI-002` states that two identical command lines run in
two different shells, against the same project state, cannot read different
databases.

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

**Under R3, the mapping lives here alone.**
`docs/spec-technical/security.md` cites `ADR-002` for it rather than restating
it, and `docs/spec-technical/open-decisions.md` entry `OD-16` reduces to a
citation of this record.

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
</content>
