# requireColumnQualifier

**Group** [`style`](../rules.md#style) · **Since** `Unreleased`  
**Diagnostic** `lint/requireColumnQualifier`

## Description
Columns in queries should be qualified with their table.

Without a table name or alias, Postgres looks the column up in every relation of the
query, and then in the ones of the queries around it. If another relation gets a column
with the same name, the query fails, or a subquery silently refers to the column of the
outer query. In the body of a SQL function, a column with the name of a parameter takes
its place.

The rule is meant for code where this matters, like functions with `SECURITY DEFINER` and
extension scripts, as defence in depth next to qualifying relations, functions, types and
operators with their schema.

The rule checks the column references of queries and DML that read from or write to a
relation. It does not report:

- queries without relations, like `select 1`
- references to output columns or their positions in `ORDER BY` and `GROUP BY`, and the
`ORDER BY` of `UNION`, `INTERSECT` and `EXCEPT`
- whole-row references, like `select u from public.users u`
- the parameters of the SQL function the statement is in
- the column lists of `INSERT`, `USING (...)` and `ON CONFLICT (...)`, and the targets of
`UPDATE ... SET`, which can only name columns of one table
- expressions outside queries, like column definitions, defaults, `CHECK` constraints and
index expressions

## Examples

### Invalid

```sql
select relid from pg_catalog.pg_statio_all_tables;
```

```sh
code-block.sql:1:8 lint/requireColumnQualifier ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

  ! Column relid is not qualified with its table.
  
  > 1 │ select relid from pg_catalog.pg_statio_all_tables;
      │        ^^^^^
    2 │ 
  
  i Without a table, the column is looked up in every relation of the query and of the queries around it, so a column with the same name in another relation can take its place.
  
  i Qualify the column with the name or alias of its table, e.g. t.relid.
  

```

### Valid

```sql
select at.relid from pg_catalog.pg_statio_all_tables as at;
select at.relid as id from pg_catalog.pg_statio_all_tables as at order by id;
insert into public.users (id, name) values (1, 'alice') on conflict (id) do nothing;
```

## How to configure
```json

{
  "linter": {
    "rules": {
      "requireColumnQualifier": "error"
    }
  }
}

```
## How to suppress

Suppress this diagnostic with a comment:

```sql
-- pgls-ignore requireColumnQualifier
```
