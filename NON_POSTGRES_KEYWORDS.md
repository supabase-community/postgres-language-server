# Non-PostgreSQL Keywords in Grammar

Keywords in `crates/pgls_completions/src/providers/keywords.rs` and the tree-sitter grammar that are **not** native PostgreSQL keywords, but come from other SQL dialects.

## Verification Sources

- [PostgreSQL SQL Key Words Documentation](https://www.postgresql.org/docs/current/sql-keywords-appendix.html)
- [PostgreSQL Data Types](https://www.postgresql.org/docs/current/datatype.html)
- [PostgreSQL SQL Commands](https://www.postgresql.org/docs/current/sql-commands.html)

---

## MySQL-Specific Keywords

These keywords are MySQL-specific and have no equivalent syntax in PostgreSQL:

| Keyword | Description | Verification |
|---------|-------------|--------------|
| `auto_increment` | MySQL auto-incrementing column syntax | PostgreSQL uses `SERIAL` or `GENERATED AS IDENTITY` |
| `change` | MySQL `ALTER TABLE ... CHANGE COLUMN` | PostgreSQL uses `ALTER COLUMN` |
| `delayed` | MySQL `INSERT DELAYED` | No PostgreSQL equivalent |
| `duplicate` | MySQL `ON DUPLICATE KEY` | PostgreSQL uses `ON CONFLICT` |
| `engine` | MySQL storage engine (e.g., `ENGINE=InnoDB`) | PostgreSQL has no storage engines |
| `escaped` | MySQL `LOAD DATA ... ESCAPED BY` | PostgreSQL COPY uses `ESCAPE` differently |
| `fields` | MySQL `LOAD DATA ... FIELDS` | PostgreSQL COPY uses different syntax |
| `follows` | MySQL trigger ordering (`FOLLOWS other_trigger`) | No PostgreSQL equivalent |
| `high_priority` | MySQL query priority hint | No PostgreSQL equivalent |
| `ignore` | MySQL `INSERT IGNORE` | PostgreSQL uses `ON CONFLICT DO NOTHING` |
| `lines` | MySQL `LOAD DATA ... LINES` | No PostgreSQL equivalent |
| `low_priority` | MySQL query priority hint | No PostgreSQL equivalent |
| `mediumint` | MySQL 3-byte integer type | [Not in PostgreSQL](https://dev.to/kellyblaire/real-life-examples-of-choosing-integer-types-in-mysql-postgresql-183m) - use `INTEGER` |
| `modify` | MySQL `ALTER TABLE ... MODIFY COLUMN` | PostgreSQL uses `ALTER COLUMN` |
| `optimize` | MySQL `OPTIMIZE TABLE` | PostgreSQL uses `VACUUM` |
| `separator` | MySQL `GROUP_CONCAT ... SEPARATOR` | PostgreSQL uses `string_agg()` with different syntax |
| `terminated` | MySQL/Hive `FIELDS TERMINATED BY` | No PostgreSQL equivalent |
| `unsigned` | MySQL unsigned integer modifier | [Not supported in PostgreSQL](https://github.com/jbranchaud/til/blob/master/postgres/postgres-does-not-support-unsigned-integers.md) |
| `use` | MySQL `USE database` | PostgreSQL uses `\c` or `SET search_path` |
| `wait` | MySQL locking hint | No PostgreSQL equivalent |
| `zerofill` | MySQL zero-padding display modifier | No PostgreSQL equivalent |

---

## SQL Server (T-SQL) Specific Keywords

| Keyword | Description | Verification |
|---------|-------------|--------------|
| `datetime` | SQL Server datetime type | [PostgreSQL uses `TIMESTAMP`](https://www.postgresql.org/docs/current/datatype-datetime.html) |
| `datetime2` | SQL Server datetime2 type | No PostgreSQL equivalent |
| `datetimeoffset` | SQL Server datetimeoffset type | PostgreSQL uses `TIMESTAMPTZ` |
| `image` | SQL Server legacy binary type | PostgreSQL uses `BYTEA` |
| `nvarchar` | SQL Server national varchar | PostgreSQL uses `VARCHAR` (no separate national type) |
| `object_id` | SQL Server system function/identifier | No PostgreSQL equivalent |
| `smalldatetime` | SQL Server smalldatetime type | No PostgreSQL equivalent |
| `smallmoney` | SQL Server smallmoney type | PostgreSQL uses `MONEY` or `NUMERIC` |
| `tinyint` | SQL Server/MySQL 8-bit integer | [Not in PostgreSQL](https://dev.to/kellyblaire/real-life-examples-of-choosing-integer-types-in-mysql-postgresql-183m) - use `SMALLINT` |

---

## Hive/Impala/Spark (Big Data) Specific Keywords

| Keyword | Description | Verification |
|---------|-------------|--------------|
| `avro` | Hive Avro file format | No PostgreSQL equivalent |
| `bin_pack` | Impala optimization hint | No PostgreSQL equivalent |
| `cached` | Impala table caching | No PostgreSQL equivalent |
| `compute` | Impala `COMPUTE STATS` | PostgreSQL uses `ANALYZE` |
| `delimited` | Hive `ROW FORMAT DELIMITED` | No PostgreSQL equivalent |
| `incremental` | Impala `COMPUTE INCREMENTAL STATS` | No PostgreSQL equivalent |
| `jsonfile` | Hive JSON file format | No PostgreSQL equivalent |
| `metadata` | Hive/Impala metadata operations | No PostgreSQL equivalent |
| `noscan` | Impala `COMPUTE STATS ... NOSCAN` | No PostgreSQL equivalent |
| `orc` | Hive ORC file format | No PostgreSQL equivalent |
| `overwrite` | Hive `INSERT OVERWRITE` | No PostgreSQL equivalent |
| `parquet` | Hive/Spark Parquet file format | No PostgreSQL equivalent |
| `rcfile` | Hive RCFile format | No PostgreSQL equivalent |
| `sequencefile` | Hive SequenceFile format | No PostgreSQL equivalent |
| `sort` | Hive `SORT BY` clause | PostgreSQL uses `ORDER BY` |
| `stats` | Impala/Hive statistics | PostgreSQL uses `ANALYZE` |
| `stored` | Hive `STORED AS` clause | No PostgreSQL equivalent |
| `tblproperties` | Hive table properties | No PostgreSQL equivalent |
| `textfile` | Hive text file format | No PostgreSQL equivalent |
| `uncached` | Impala table caching control | No PostgreSQL equivalent |
| `unload` | Redshift/Hive `UNLOAD` command | PostgreSQL uses `COPY TO` |

---

## PostGIS Extension Keywords

These are not core PostgreSQL keywords but come from the PostGIS spatial extension. They work in PostgreSQL only when PostGIS is installed:

| Keyword | Description |
|---------|-------------|
| `box2d` | PostGIS 2D bounding box type |
| `box3d` | PostGIS 3D bounding box type |
| `geography` | PostGIS geography type |
| `geometry` | PostGIS geometry type |

---

## Keywords in PostgreSQL's SQL Standard Appendix (Not Features)

The following keywords appear in [PostgreSQL's keyword appendix](https://www.postgresql.org/docs/current/sql-keywords-appendix.html) as SQL standard keywords that PostgreSQL recognizes for compatibility, but they don't correspond to implemented features:

| Keyword | Notes |
|---------|-------|
| `nchar` | SQL standard national character - recognized but functionally equivalent to `char` |
| `precedes` | SQL:2011 temporal - recognized but limited support |
| `string` | SQL standard - PostgreSQL uses `TEXT` or `VARCHAR` |
| `varbinary` | SQL standard - PostgreSQL uses `BYTEA` |
| `version` | SQL standard keyword - not a PostgreSQL feature keyword |
| `virtual` | SQL standard for generated columns - PostgreSQL uses `GENERATED ALWAYS AS ... STORED` |

---

## Summary

| Category | Count |
|----------|-------|
| MySQL-specific | 21 |
| SQL Server-specific | 9 |
| Hive/Impala/Spark | 21 |
| PostGIS extension | 4 |
| SQL Standard (not features) | 6 |
| **Total** | **61** |

---

## Notes

1. **PostgreSQL-specific keywords that ARE valid** (removed from this list):
   - `permissive`, `restrictive` - [Row-level security policy types](https://www.postgresql.org/docs/current/ddl-rowsecurity.html)
   - `brin`, `btree`, `gin`, `gist`, `spgist`, `hash` - [Index types](https://www.postgresql.org/docs/current/indexes-types.html)
   - `force_quote`, `force_null`, `force_not_null`, `freeze`, `header`, `delimiter` - [COPY options](https://www.postgresql.org/docs/current/sql-copy.html)
   - `leakproof`, `immutable`, `stable`, `volatile`, `strict`, `support` - [Function attributes](https://www.postgresql.org/docs/current/sql-createfunction.html)
   - `parallel` with `safe`/`unsafe`/`restricted` - Function parallelism options

2. **Type aliases that ARE valid in PostgreSQL**:
   - `bigserial`, `smallserial`, `serial` - Auto-incrementing types
   - `bytea`, `inet`, `jsonb`, `money`, `uuid`, `timestamptz` - PostgreSQL-specific types
   - `regclass`, `regtype`, `regproc`, `regnamespace`, `oid` - System catalog types
