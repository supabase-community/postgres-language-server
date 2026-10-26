# banDropFunction

**Group** [`destructive`](../rules.md#destructive) · **Migrations only** · **Since** `Unreleased`  
**Diagnostic** `lint/banDropFunction` · **Postgres error** `42883`

**Sources**: inspired by [`squawk/ban-drop-function`](https://squawkhq.com/docs/ban-drop-function)

## Description
Dropping a function, procedure, routine, or aggregate may break existing clients.

Calls from clients that still use it fail with `42883 undefined_function`. Rolling back
the application does not restore it.

Update your application code to no longer call the function, then drop it in a later
migration.

## Examples

### Invalid

```sql
drop function calculate_total(int);
```

```sh
code-block.sql:1:1 lint/banDropFunction ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

  ! Dropping a function may break existing clients.
  
  > 1 │ drop function calculate_total(int);
      │ ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
    2 │ 
  
  i Clients that still call it fail, and rolling back the application does not restore it.
  
  i Update your application code to no longer call it, then drop it in a later migration.
  

```

```sql
drop procedure archive_orders();
```

```sh
code-block.sql:1:1 lint/banDropFunction ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

  ! Dropping a procedure may break existing clients.
  
  > 1 │ drop procedure archive_orders();
      │ ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
    2 │ 
  
  i Clients that still call it fail, and rolling back the application does not restore it.
  
  i Update your application code to no longer call it, then drop it in a later migration.
  

```

## How to configure
```json

{
  "linter": {
    "rules": {
      "banDropFunction": "error"
    }
  }
}

```
## How to suppress

Suppress this diagnostic with a comment:

```sql
-- pgls-ignore banDropFunction
```
