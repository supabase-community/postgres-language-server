# functionArgumentMismatch

**Group** [`typecheck`](../rules.md#typecheck) · **Recommended** · **Needs a database connection** · **Since** `0.27.0`  
**Diagnostic** `lint/functionArgumentMismatch` · **Postgres error** `42883`, `42725`

## Description
A function name and argument count exist, but its argument types do not select exactly
one overload.

Postgres raises `42883 undefined_function` when no function matches, and
`42725 ambiguous_function` when the call is ambiguous.

## Examples

### Invalid

```sql
create function example_fn(value integer) returns integer language sql as 'select value';
select example_fn(timestamp '2020-01-01');
```

```sh
code-block.sql:2:8 lint/functionArgumentMismatch ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

  × Function example_fn(timestamp without time zone) does not exist.
  
    1 │ create function example_fn(value integer) returns integer language sql as 'select value';
  > 2 │ select example_fn(timestamp '2020-01-01');
      │        ^^^^^^^^^^
    3 │ 
  
  i No function matches the given name and argument types. You might need to add explicit type casts.
  

```

### Valid

```sql
create function example_fn(value integer) returns integer language sql as 'select value';
select example_fn(1);
```

## How to configure
```json

{
  "linter": {
    "rules": {
      "functionArgumentMismatch": "error"
    }
  }
}

```
## How to suppress

Suppress this diagnostic with a comment:

```sql
-- pgls-ignore functionArgumentMismatch
```
