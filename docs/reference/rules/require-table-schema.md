# requireTableSchema

**Group** [`style`](../rules.md#style) · **Since** `Unreleased`  
**Diagnostic** `lint/requireTableSchema`

**Sources**: inspired by [`squawk/require-table-schema`](https://squawkhq.com/docs/require-table-schema)

## Description
Table names in DDL should be schema-qualified.

Without a schema, Postgres resolves the table through the `search_path`, which depends on
the session that runs the migration. This makes it ambiguous which table a statement acts
on. Temporary tables, including the ones created earlier in the file, are not reported.

The rule checks the statements that create, change, or drop a table:
`CREATE [FOREIGN] TABLE`, `CREATE TABLE AS`, `SELECT INTO`, `ALTER [FOREIGN] TABLE`
(including `RENAME` and `SET SCHEMA`), `CREATE INDEX`, and `DROP [FOREIGN] TABLE`. It also
checks the tables these statements reference by name, like `REFERENCES`, `PARTITION OF`,
`INHERITS`, `LIKE`, and `ATTACH PARTITION`. Queries, like the one of a view, are not
checked.

## Examples

### Invalid

```sql
create table posts (id bigint);
```

```sh
code-block.sql:1:14 lint/requireTableSchema ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

  ! Table posts is not schema-qualified.
  
  > 1 │ create table posts (id bigint);
      │              ^^^^^
    2 │ 
  
  i Without a schema, the table is resolved through the search_path of the session that runs the statement.
  
  i Specify the schema, e.g. public.posts.
  

```

```sql
alter table posts add column title text;
```

```sh
code-block.sql:1:13 lint/requireTableSchema ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

  ! Table posts is not schema-qualified.
  
  > 1 │ alter table posts add column title text;
      │             ^^^^^
    2 │ 
  
  i Without a schema, the table is resolved through the search_path of the session that runs the statement.
  
  i Specify the schema, e.g. public.posts.
  

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
