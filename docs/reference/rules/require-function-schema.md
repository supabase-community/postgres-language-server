# requireFunctionSchema

**Group** [`style`](../rules.md#style) · **Since** `Unreleased`  
**Diagnostic** `lint/requireFunctionSchema`

## Description
Function calls should be schema-qualified.

Without a schema, Postgres looks the function up in every schema on the `search_path` and
calls the best match. A function with the same name in a schema earlier on the search
path, or an overload in any of them whose argument types match the call better, is called
instead of the intended one. Anyone who can create functions in one of these schemas can
run code with the privileges of the caller.

The rule is meant for code where this matters, like functions with `SECURITY DEFINER` and
extension scripts, as defence in depth next to setting the `search_path` explicitly.

The rule checks every function call, including aggregates, window functions, `CALL` and
calls in DDL, like column defaults and index expressions. It doesn't report the SQL syntax
that Postgres turns into a call of a `pg_catalog` function itself, like `extract(...)`,
`position(... in ...)`, `substring(... from ...)`, `overlay`, `trim`, `current_date` or
`coalesce`.

## Examples

### Invalid

```sql
select abs(1);
```

```sh
code-block.sql:1:8 lint/requireFunctionSchema ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

  ! Function abs is not schema-qualified.
  
  > 1 │ select abs(1);
      │        ^^^
    2 │ 
  
  i Without a schema, the function is resolved through the search_path: a function with the same name earlier on the search_path, or an overload whose argument types match better, is called instead.
  
  i Specify the schema, e.g. pg_catalog.abs for a built-in function.
  

```

### Valid

```sql
select pg_catalog.abs(1);
select extract(year from pg_catalog.now());
```

## How to configure
```json

{
  "linter": {
    "rules": {
      "requireFunctionSchema": "error"
    }
  }
}

```
## How to suppress

Suppress this diagnostic with a comment:

```sql
-- pgls-ignore requireFunctionSchema
```
