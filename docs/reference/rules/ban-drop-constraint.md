# banDropConstraint

**Group** [`destructive`](../rules.md#destructive) · **Migrations only** · **Since** `Unreleased`  
**Diagnostic** `lint/banDropConstraint`

**Sources**: inspired by [`squawk/ban-drop-constraint`](https://squawkhq.com/docs/ban-drop-constraint)

## Description
Dropping a constraint may break existing clients.

A constraint is a guarantee that clients can depend on. For example, an application that
uses `INSERT ... ON CONFLICT ON CONSTRAINT my_constraint`, or whose `ON CONFLICT (col)`
relies on a unique constraint, fails as soon as the constraint is gone.

Update clients to no longer depend on the constraint before dropping it.

## Examples

### Invalid

```sql
alter table users drop constraint users_email_key;
```

```sh
code-block.sql:1:1 lint/banDropConstraint ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

  ! Dropping a constraint may break existing clients.
  
  > 1 │ alter table users drop constraint users_email_key;
      │ ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
    2 │ 
  
  i Clients may depend on the guarantee of the constraint, e.g. as the arbiter of INSERT ... ON CONFLICT.
  
  i Update clients to no longer depend on the constraint, then drop it in a later migration.
  

```

```sql
alter domain positive_int drop constraint positive_int_check;
```

```sh
code-block.sql:1:1 lint/banDropConstraint ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

  ! Dropping a constraint may break existing clients.
  
  > 1 │ alter domain positive_int drop constraint positive_int_check;
      │ ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
    2 │ 
  
  i Clients may depend on the guarantee of the constraint, e.g. as the arbiter of INSERT ... ON CONFLICT.
  
  i Update clients to no longer depend on the constraint, then drop it in a later migration.
  

```

## How to configure
```json

{
  "linter": {
    "rules": {
      "banDropConstraint": "error"
    }
  }
}

```
## How to suppress

Suppress this diagnostic with a comment:

```sql
-- pgls-ignore banDropConstraint
```
