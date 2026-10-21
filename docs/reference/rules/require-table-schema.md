# requireTableSchema

**Group** [`style`](../rules.md#style) · **Since** `Unreleased`  
**Diagnostic** `lint/requireTableSchema`

**Sources**: inspired by [`squawk/require-table-schema`](https://squawkhq.com/docs/require-table-schema)

## Description
Table names in DDL should be schema-qualified.

Without a schema, Postgres resolves the table through the `search_path`, which depends on
the session that runs the migration. This makes it ambiguous which table a statement acts
on. Temporary tables are not reported.

The rule checks `CREATE TABLE`, `CREATE TABLE AS`, `ALTER TABLE`, and `DROP TABLE`.

## Examples

### Invalid

```sql
create table posts (id bigint);
```

```sh
code-block.sql:1:1 lint/requireTableSchema ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

  ! Table posts is not schema-qualified.
  
  > 1 │ create table posts (id bigint);
      │ ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
    2 │ 
  
  i Without a schema, the table is resolved through the search_path of the session that runs the statement.
  
  i Specify the schema, e.g. public.my_table.
  

```

```sql
alter table posts add column title text;
```

```sh
code-block.sql:1:1 lint/requireTableSchema ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

  ! Table posts is not schema-qualified.
  
  > 1 │ alter table posts add column title text;
      │ ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
    2 │ 
  
  i Without a schema, the table is resolved through the search_path of the session that runs the statement.
  
  i Specify the schema, e.g. public.my_table.
  

```

### Valid

```sql
create table public.posts (id bigint);
alter table public.posts add column title text;
drop table public.posts;
create temp table scratch (id bigint);
```

## How to configure
```json

{
  "linter": {
    "rules": {
      "requireTableSchema": "error"
    }
  }
}

```
## How to suppress

Suppress this diagnostic with a comment:

```sql
-- pgls-ignore requireTableSchema
```
