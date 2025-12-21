# Non-PostgreSQL Keywords in Grammar

Keywords in `crates/pgls_completions/src/providers/keywords.rs` and the tree-sitter grammar that are **not** PostgreSQL keywords, but come from other SQL dialects.

Reference: [PostgreSQL SQL Key Words Documentation](https://www.postgresql.org/docs/current/sql-keywords-appendix.html)

---

## MySQL-Specific Keywords

| Keyword | Description |
|---------|-------------|
| `auto_increment` | MySQL auto-incrementing column syntax (PostgreSQL uses `SERIAL` or `GENERATED AS IDENTITY`) |
| `change` | MySQL `ALTER TABLE ... CHANGE COLUMN` syntax |
| `delayed` | MySQL `INSERT DELAYED` syntax |
| `duplicate` | MySQL `ON DUPLICATE KEY` syntax |
| `engine` | MySQL storage engine specification (e.g., `ENGINE=InnoDB`) |
| `escaped` | MySQL `LOAD DATA ... ESCAPED BY` syntax |
| `fields` | MySQL `LOAD DATA ... FIELDS` syntax (PostgreSQL uses `COLUMNS` in some contexts) |
| `follows` | MySQL trigger ordering (`FOLLOWS other_trigger`) |
| `high_priority` | MySQL query priority hint |
| `ignore` | MySQL `INSERT IGNORE` / `LOAD DATA ... IGNORE` syntax |
| `lines` | MySQL `LOAD DATA ... LINES` syntax |
| `low_priority` | MySQL query priority hint |
| `mediumint` | MySQL-specific integer type (24-bit) |
| `modify` | MySQL `ALTER TABLE ... MODIFY COLUMN` syntax |
| `optimize` | MySQL `OPTIMIZE TABLE` command |
| `separator` | MySQL `GROUP_CONCAT ... SEPARATOR` syntax |
| `terminated` | MySQL/Hive `FIELDS TERMINATED BY` syntax |
| `unsigned` | MySQL unsigned integer modifier |
| `use` | MySQL `USE database` command |
| `wait` | MySQL locking hint |
| `zerofill` | MySQL zero-padding display modifier |

---

## SQL Server (T-SQL) Specific Keywords

| Keyword | Description |
|---------|-------------|
| `datetime` | SQL Server datetime type (PostgreSQL uses `TIMESTAMP`) |
| `datetime2` | SQL Server datetime2 type |
| `datetimeoffset` | SQL Server datetimeoffset type (PostgreSQL uses `TIMESTAMPTZ`) |
| `image` | SQL Server legacy binary type (deprecated, use `VARBINARY(MAX)`) |
| `nvarchar` | SQL Server national varchar type |
| `object_id` | SQL Server system function/identifier |
| `smalldatetime` | SQL Server smalldatetime type |
| `smallmoney` | SQL Server smallmoney type |
| `tinyint` | SQL Server/MySQL 8-bit integer (PostgreSQL uses `SMALLINT`) |

---

## Hive/Impala/Spark (Big Data) Specific Keywords

| Keyword | Description |
|---------|-------------|
| `avro` | Hive file format |
| `bin_pack` | Impala optimization hint |
| `cached` | Impala table caching |
| `compute` | Impala `COMPUTE STATS` command |
| `delimited` | Hive `ROW FORMAT DELIMITED` syntax |
| `incremental` | Impala `COMPUTE INCREMENTAL STATS` |
| `jsonfile` | Hive JSON file format |
| `metadata` | Hive/Impala metadata operations |
| `noscan` | Impala `COMPUTE STATS ... NOSCAN` option |
| `orc` | Hive ORC file format |
| `overwrite` | Hive `INSERT OVERWRITE` syntax |
| `parquet` | Hive/Spark Parquet file format |
| `rcfile` | Hive RCFile format |
| `sequencefile` | Hive SequenceFile format |
| `sort` | Hive `SORT BY` clause |
| `stats` | Impala/Hive statistics operations |
| `stored` | Hive `STORED AS` clause |
| `tblproperties` | Hive table properties |
| `textfile` | Hive text file format |
| `uncached` | Impala table caching control |
| `unload` | Redshift/Hive `UNLOAD` command |

---

## PostGIS Extension Keywords

These are not core PostgreSQL keywords but come from the PostGIS spatial extension:

| Keyword | Description |
|---------|-------------|
| `box2d` | PostGIS 2D bounding box type |
| `box3d` | PostGIS 3D bounding box type |
| `geography` | PostGIS geography type |
| `geometry` | PostGIS geometry type |

---

## Other Non-Standard Keywords

| Keyword | Description | Origin |
|---------|-------------|--------|
| `restricted` | Non-standard security option | Various |
| `safe` | Non-standard function attribute | Various |
| `unsafe` | Non-standard function attribute | Various |

---

## Notes

1. Some keywords like `string` appear in the grammar but are listed in PostgreSQL's keyword appendix as SQL standard keywords that PostgreSQL recognizes.

2. PostgreSQL-specific types like `bigserial`, `smallserial`, `bytea`, `inet`, `jsonb`, `money`, `oid`, `regclass`, `regtype`, `timestamptz`, `uuid` are PostgreSQL types but not listed as reserved keywords—they can be used as identifiers.

3. Index types like `brin`, `btree`, `gin`, `gist`, `spgist`, and `hash` are PostgreSQL-specific but are recognized tokens, not reserved keywords.

---

## Summary

| Category | Count |
|----------|-------|
| MySQL-specific | 21 |
| SQL Server-specific | 9 |
| Hive/Impala/Spark | 21 |
| PostGIS extension | 4 |
| Other non-standard | 3 |
| **Total** | **58** |
