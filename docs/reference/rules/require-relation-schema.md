# requireRelationSchema

**Group** [`style`](../rules.md#style) · **Since** `Unreleased`  
**Diagnostic** `lint/requireRelationSchema`

## Description
Relations in queries should be schema-qualified.

Without a schema, Postgres looks the table or view up in the schemas on the
`search_path`, starting with `pg_temp`. A relation with the same name in a schema earlier
on the search path, like a temporary table that any user can create, is used instead of
the intended one.

The rule is meant for code where this matters, like functions with `SECURITY DEFINER` and
extension scripts, as defence in depth next to setting the `search_path` explicitly.

The rule checks the relations that queries and DML read or write: `FROM`, `JOIN`,
subqueries, CTEs, the targets of `INSERT`, `UPDATE`, `DELETE` and `MERGE`, and their
`USING`, as well as the queries of views, `CREATE TABLE AS`, `EXPLAIN`, `DECLARE` and
`PREPARE`, and the relations of `COPY`, `TRUNCATE` and `LOCK`. Names that refer to a CTE
and temporary tables, including the ones created earlier in the file, are not reported.
The tables that DDL creates, changes or drops are checked by `requireTableSchema`.

## Examples

### Invalid

```sql
select * from pg_statio_all_tables;
```

```sh
code-block.sql:1:15 lint/requireRelationSchema ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

  ! Relation pg_statio_all_tables is not schema-qualified.
  
  > 1 │ select * from pg_statio_all_tables;
      │               ^^^^^^^^^^^^^^^^^^^^
    2 │ 
  
  i Without a schema, the relation is resolved through the search_path: a relation with the same name earlier on the search_path, like a temporary table, is used instead.
  
  i Specify the schema, e.g. public.pg_statio_all_tables.
  

```

```sql
insert into users (name) values ('alice');
```

```sh
code-block.sql:1:13 lint/requireRelationSchema ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

  ! Relation users is not schema-qualified.
  
  > 1 │ insert into users (name) values ('alice');
      │             ^^^^^
    2 │ 
  
  i Without a schema, the relation is resolved through the search_path: a relation with the same name earlier on the search_path, like a temporary table, is used instead.
  
  i Specify the schema, e.g. public.users.
  

```

### Valid

```sql
select * from pg_catalog.pg_statio_all_tables;
with recent as (select * from public.posts) select * from recent;
```

## How to configure
```json

{
  "linter": {
    "rules": {
      "requireRelationSchema": "error"
    }
  }
}

```
## How to suppress

Suppress this diagnostic with a comment:

```sql
-- pgls-ignore requireRelationSchema
```
