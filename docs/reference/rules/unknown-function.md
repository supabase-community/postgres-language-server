# unknownFunction

**Group** [`typecheck`](../rules.md#typecheck) · **Recommended** · **Needs a database connection** · **Since** `0.27.0`  
**Diagnostic** `lint/unknownFunction` · **Postgres error** `42883`

## Description
No function with this name accepts this number of arguments.

Functions are looked up in the connected database and in the functions created earlier in
the same file, following the search path. Default arguments and `VARIADIC` parameters are
taken into account. Argument types are not checked yet.

Postgres raises `42883 undefined_function` for these statements.

## Examples

### Invalid

```sql
select missing_function();
```

```sh
code-block.sql:1:8 lint/unknownFunction ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

  × Function missing_function does not exist.
  
  > 1 │ select missing_function();
      │        ^^^^^^^^^^^^^^^^
    2 │ 
  

```

### Valid

```sql
create function add_one(value int8) returns int8 language sql as 'select value + 1';
select add_one(1);
```

## How to configure
```json

{
  "linter": {
    "rules": {
      "unknownFunction": "error"
    }
  }
}

```
## How to suppress

Suppress this diagnostic with a comment:

```sql
-- pgls-ignore unknownFunction
```
