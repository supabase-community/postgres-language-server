# banAlterGeneratedExpression

**Group** [`destructive`](../rules.md#destructive) · **Migrations only** · **Since** `Unreleased`  
**Diagnostic** `lint/banAlterGeneratedExpression`

**Sources**: inspired by [`squawk/ban-alter-generated-expression`](https://squawkhq.com/docs/ban-alter-generated-expression)

## Description
Adding a generated column or changing its expression may break existing clients.

`SET EXPRESSION` changes the values that existing clients read from the column. A new
generated column changes the columns of `SELECT *`, and statements that copy rows with
`INSERT ... SELECT *` then try to write the generated column, which fails.

Dropping the expression is reported by `banDropGeneratedExpression`. `SET EXPRESSION` is
only checked on Postgres 17 and later, which added it.

## Examples

### Invalid

```sql
alter table line_items alter column total set expression as (price * quantity);
```

```sh
code-block.sql:1:1 lint/banAlterGeneratedExpression ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

  ! Changing the expression of generated column total may break existing clients.
  
  > 1 │ alter table line_items alter column total set expression as (price * quantity);
      │ ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
    2 │ 
  
  i Existing clients read different values from the column.
  
  i Check that clients can handle the new values before changing the expression.
  

```

```sql
alter table line_items add column total numeric generated always as (price * quantity) stored;
```

```sh
code-block.sql:1:1 lint/banAlterGeneratedExpression ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

  ! Adding generated column total may break existing clients.
  
  > 1 │ alter table line_items add column total numeric generated always as (price * quantity) stored;
      │ ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
    2 │ 
  
  i SELECT * returns an additional column, and INSERT ... SELECT * tries to write the generated column, which fails.
  
  i Make sure that clients list the columns they insert and select before adding the column.
  

```

### Valid

```sql
alter table line_items add column total numeric;
```

## How to configure
```json

{
  "linter": {
    "rules": {
      "banAlterGeneratedExpression": "error"
    }
  }
}

```
## How to suppress

Suppress this diagnostic with a comment:

```sql
-- pgls-ignore banAlterGeneratedExpression
```
