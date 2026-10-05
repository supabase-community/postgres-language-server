# operatorTypeMismatch

**Group** [`typecheck`](../rules.md#typecheck) · **Recommended** · **Needs a database connection** · **Since** `0.27.0`  
**Diagnostic** `lint/operatorTypeMismatch` · **Postgres error** `42883`, `42725`

## Description
An operator exists by name but cannot be resolved for the operand types, or has
multiple equally suitable candidates.

Postgres raises `42883 undefined_function` when no operator matches, and
`42725 ambiguous_function` when the operator is ambiguous.

## Examples

### Invalid

```sql
select 1 + timestamp '2020-01-01';
```

```sh
code-block.sql:1:10 lint/operatorTypeMismatch ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

  × Operator does not exist: integer + timestamp without time zone
  
  > 1 │ select 1 + timestamp '2020-01-01';
      │          ^
    2 │ 
  
  i No operator matches the given name and argument types. You might need to add explicit type casts.
  

```

### Valid

```sql
select 1 + 2;
```

## How to configure
```json

{
  "linter": {
    "rules": {
      "operatorTypeMismatch": "error"
    }
  }
}

```
## How to suppress

Suppress this diagnostic with a comment:

```sql
-- pgls-ignore operatorTypeMismatch
```
