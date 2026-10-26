# banDropGeneratedExpression

**Group** [`destructive`](../rules.md#destructive) · **Migrations only** · **Since** `Unreleased`  
**Diagnostic** `lint/banDropGeneratedExpression`

**Sources**: inspired by [`squawk/ban-drop-generated-expression`](https://squawkhq.com/docs/ban-drop-generated-expression)

## Description
Dropping the expression of a generated column may break existing clients.

`DROP EXPRESSION` turns a stored generated column into an ordinary column. Existing values
remain, but Postgres no longer computes the value for new or updated rows, so clients that
omit the column write `NULL` instead of the computed value.

## Examples

### Invalid

```sql
alter table line_items alter column total drop expression;
```

```sh
code-block.sql:1:1 lint/banDropGeneratedExpression ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

  ! Dropping the expression of generated column total may break existing clients.
  
  > 1 │ alter table line_items alter column total drop expression;
      │ ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
    2 │ 
  
  i Postgres no longer computes the column, so inserts and updates that omit it write NULL.
  
  i Update clients to write the column themselves before dropping the expression.
  

```

## How to configure
```json

{
  "linter": {
    "rules": {
      "banDropGeneratedExpression": "error"
    }
  }
}

```
## How to suppress

Suppress this diagnostic with a comment:

```sql
-- pgls-ignore banDropGeneratedExpression
```
