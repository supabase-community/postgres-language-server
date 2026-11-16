# requireOperatorSchema

**Group** [`style`](../rules.md#style) · **Since** `Unreleased`  
**Diagnostic** `lint/requireOperatorSchema`

## Description
Operators should be schema-qualified.

Without a schema, Postgres looks the operator up in every schema on the `search_path`
and uses the best match. An operator with the same name in a schema earlier on the
search path, or one in any of them whose argument types match better, is used instead of
the intended one. Anyone who can create operators in one of these schemas can run code
with the privileges of the caller. A schema-qualified operator is written as
`OPERATOR(schema.op)`, e.g. `a OPERATOR(pg_catalog.=) b`.

The rule is meant for code where this matters, like functions with `SECURITY DEFINER` and
extension scripts, as defence in depth next to setting the `search_path` explicitly.

The rule checks binary and unary operators, `op ANY (...)` and `op ALL (...)` with arrays
and subqueries, and `ORDER BY ... USING op`. It also reports the SQL constructs that
Postgres turns into operators that it looks up through the search path: `IN` and
`NOT IN` (`=` and `<>`), `LIKE` and `ILIKE` (`~~` and `~~*`), `SIMILAR TO` (`~`),
`IS [NOT] DISTINCT FROM` and `NULLIF` (`=`), and `BETWEEN` (`>=` and `<=`). They have no
schema-qualified form, so the comparison has to be written with `OPERATOR(...)` instead.

## Examples

### Invalid

```sql
select * from pg_catalog.pg_stat_io where backend_type = 'client backend';
```

```sh
code-block.sql:1:56 lint/requireOperatorSchema ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

  ! Operator = is not schema-qualified.
  
  > 1 │ select * from pg_catalog.pg_stat_io where backend_type = 'client backend';
      │                                                        ^
    2 │ 
  
  i Without a schema, the operator is resolved through the search_path: an operator with the same name earlier on the search_path, or one whose argument types match better, is used instead.
  
  i Specify the schema, e.g. OPERATOR(pg_catalog.=).
  

```

```sql
select * from pg_catalog.pg_class where relkind in ('r', 'p');
```

```sh
code-block.sql:1:49 lint/requireOperatorSchema ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

  ! IN uses the operator = without a schema.
  
  > 1 │ select * from pg_catalog.pg_class where relkind in ('r', 'p');
      │                                                 ^^
    2 │ 
  
  i Without a schema, the operator is resolved through the search_path: an operator with the same name earlier on the search_path, or one whose argument types match better, is used instead.
  
  i IN has no schema-qualified form. Write the comparison with OPERATOR(pg_catalog.=) instead.
  

```

### Valid

```sql
select * from pg_catalog.pg_stat_io where backend_type operator(pg_catalog.=) 'client backend';
select * from pg_catalog.pg_class where relkind operator(pg_catalog.=) any (array['r', 'p']);
```

## How to configure
```json

{
  "linter": {
    "rules": {
      "requireOperatorSchema": "error"
    }
  }
}

```
## How to suppress

Suppress this diagnostic with a comment:

```sql
-- pgls-ignore requireOperatorSchema
```
