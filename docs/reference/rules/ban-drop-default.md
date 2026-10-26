# banDropDefault

**Group** [`destructive`](../rules.md#destructive) · **Migrations only** · **Since** `Unreleased`  
**Diagnostic** `lint/banDropDefault` · **Postgres error** `23502`

**Sources**: inspired by [`squawk/ban-drop-default`](https://squawkhq.com/docs/ban-drop-default)

## Description
Dropping a column or domain default may break existing clients.

Inserts that omit the column fail with `23502 not_null_violation` if the column is
`NOT NULL`, and silently write `NULL` otherwise.

Update your application code to provide a value for the column, then drop the default in a
later migration.

## Examples

### Invalid

```sql
alter table users alter column created_at drop default;
```

```sh
code-block.sql:1:43 lint/banDropDefault ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

  ! Dropping the default of column created_at may break existing clients.
  
  > 1 │ alter table users alter column created_at drop default;
      │                                           ^^^^^^^^^^^^
    2 │ 
  
  i Inserts that omit the column fail if it is NOT NULL, and write NULL otherwise.
  
  i Update your application code to provide a value, then drop the default in a later migration.
  

```

```sql
alter domain status drop default;
```

```sh
code-block.sql:1:21 lint/banDropDefault ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

  ! Dropping the default of domain status may break existing clients.
  
  > 1 │ alter domain status drop default;
      │                     ^^^^^^^^^^^^
    2 │ 
  
  i Inserts that omit the column fail if it is NOT NULL, and write NULL otherwise.
  
  i Update your application code to provide a value, then drop the default in a later migration.
  

```

### Valid

```sql
alter table users alter column created_at set default now();
```

## How to configure
```json

{
  "linter": {
    "rules": {
      "banDropDefault": "error"
    }
  }
}

```
## How to suppress

Suppress this diagnostic with a comment:

```sql
-- pgls-ignore banDropDefault
```
