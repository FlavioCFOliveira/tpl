# Sakila — provenance

| | |
|---|---|
| Source | `https://downloads.mysql.com/docs/sakila-db.tar.gz` |
| Taken on | 2026-09-22 |
| Archive SHA-256 | `ad246d78271690412375ad6de8da2cbfa64eb535040d5c5a36f7af59e24e0649` |
| Archive size | 732 423 bytes |
| Version | Sakila Sample Database 1.5, as the first line of each SQL file states |
| Upstream owner | Oracle and/or its affiliates |
| Licence | BSD 3-clause, in [LICENSE](LICENSE) |

## What is vendored

The archive holds three files. Two are vendored here, **byte for byte and
unmodified**, and the third is not:

| Archive member | Vendored as | Why |
|---|---|---|
| `sakila-db/sakila-schema.sql` | `sakila-schema.sql` | The DDL: 16 tables, 7 views, 6 routines, 3 triggers |
| `sakila-db/sakila-data.sql` | `sakila-data.sql` | The rows, and 3 further triggers |
| `sakila-db/sakila.mwb` | — | A MySQL Workbench binary model. It is not SQL, nothing loads it, and `seed-datasets.sh` would not know what to do with it |

Verify the two against upstream by re-fetching the archive:

```
shasum -a 256 sakila-schema.sql sakila-data.sql
b32170e1e2ad5828749b61a5ec896155bcd143104b076e5ee8a3a3b013f44915  sakila-schema.sql
8c228c678cec6ea9e5145ea868f48be87982252e806a547dcddb67758cadf174  sakila-data.sql
```

## The licence file

Upstream ships no separate licence file. `LICENSE` beside this note is the
copyright notice and the BSD 3-clause text that both SQL files carry in their
header, lifted verbatim and stripped of the leading `--` SQL comment marker.
The two headers are identical; the text was taken from `sakila-schema.sql`.

## What MariaDB does with it

MariaDB 12.3.3 accepts both files unmodified. Nothing here is patched, and the
server writes no `ERROR` to its log while loading them.

One difference from MySQL is worth knowing before it is mistaken for a defect.
`sakila.address` declares a `location GEOMETRY` column and a spatial index
behind executable comments:

```sql
  /*!50705 location GEOMETRY */ /*!80003 SRID 0 */ /*!50705 NOT NULL,*/
```

MariaDB ignores executable comments whose version number lies between `50700`
and `99999`, so it declares neither the column nor the index, and errors on
neither. `sakila.address` therefore carries **8 columns on MariaDB against 9 on
MySQL 5.7.5 and later**, and the geometry values in `sakila-data.sql` — which
sit behind the same `/*!50705 */` comment, 603 times — are skipped with it. The
two files stay consistent with each other either way; that is why the comment is
in both.

The threshold was measured on this fixture, not assumed:

```
$ . ./series.env
$ tpl_mariadb_sql 12.3 -N -B -e "
    SET @a=1; /*!50699 SET @a=2*/; SELECT '50699', @a;
    SET @b=1; /*!50700 SET @b=2*/; SELECT '50700', @b;
    SET @c=1; /*!80003 SET @c=2*/; SELECT '80003', @c;
    SET @d=1; /*!99999 SET @d=2*/; SELECT '99999', @d;
    SET @e=1; /*!100000 SET @e=2*/; SELECT '100000', @e;"
50699   2
50700   1
80003   1
99999   1
100000  2
```
