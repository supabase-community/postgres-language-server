# unknownRelation

**Group** [`typecheck`](../rules.md#typecheck) · **Recommended** · **Needs a database connection** · **Since** `0.27.0`  
**Diagnostic** `lint/unknownRelation` · **Postgres error** `42P01`

## Description
A table, view, or materialized view does not exist.

The relation is looked up in the connected database and in the relations created earlier in
the same file, following the search path of the session (including `SET search_path` in the
file). Temporary tables are found in `pg_temp`.

Postgres raises `42P01 undefined_table` for these statements.

## Examples

### Invalid

```sql
select * from missing_table;
```

```sh
code-block.sql:1:15 lint/unknownRelation ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

  × Relation missing_table does not exist.
  
  > 1 │ select * from missing_table;
      │               ^^^^^^^^^^^^^
    2 │ 
  

```

### Valid

```sql
create temp table scratch (id int8);
select * from scratch;
```

## How to configure
```json

{
  "linter": {
    "rules": {
      "unknownRelation": "error"
    }
  }
}

```
## How to suppress

Suppress this diagnostic with a comment:

```sql
-- pgls-ignore unknownRelation
```
