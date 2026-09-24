# World — provenance

| | |
|---|---|
| Source | `https://downloads.mysql.com/docs/world-db.tar.gz` |
| Taken on | 2026-09-22 |
| Archive SHA-256 | `57d727e06405337eae177ea7422ac95b14d7a055a67b5627605be4cbb9b6647c` |
| Archive size | 92 927 bytes |
| Version | A `mysqldump 10.13` of MySQL `8.0.19`, dated `2020-01-22 9:56:18` in the file's own trailer |
| Licence | **Not carried by the archive.** See below |

## What is vendored

The archive holds one file, vendored here **byte for byte and unmodified**:

| Archive member | Vendored as |
|---|---|
| `world-db/world.sql` | `world.sql` |

Verify it against upstream by re-fetching the archive:

```
shasum -a 256 world.sql
dc96faace01d61c3d571c45f0fa55a5c9dd26baa2a700f05a6fd74f382482b7b  world.sql
```

## There is no licence file

`world-db.tar.gz` contains exactly one member, `world-db/world.sql`, and that
file carries no copyright notice and no licence text — it is a plain
`mysqldump` whose header names only the tool, the host and the server version.
The archive therefore ships nothing to vendor as `LICENSE`, which is why no
such file sits beside this note, and the absence is recorded here rather than
filled in with a guess.

This is a difference from [sakila](../sakila/NOTICE.md), whose SQL files carry
the BSD 3-clause text in their header and whose `LICENSE` is that text lifted
out of it.

## What MariaDB does with it

MariaDB 12.3.3 accepts the file unmodified. Nothing here is patched, and the
server writes no `ERROR` to its log while loading it.

The dump uses executable comments, all of them below `50700`, so MariaDB runs
every one: `40000`, `40014`, `40101`, `40103`, `40111` and `50503`. None guards
a statement MariaDB lacks. `city` declares a foreign key to `country`, which
the dump creates afterwards, and the load works because the dump sets
`FOREIGN_KEY_CHECKS=0` around itself.

## What it holds

Three tables, no views, no routines, no triggers, no generated columns and no
table comments: `country` (239 rows), `city` (4 079 rows) and `countrylanguage`
(984 rows). The schema is small and flat on purpose — it is the dataset a
reader is most likely to already know by heart.
