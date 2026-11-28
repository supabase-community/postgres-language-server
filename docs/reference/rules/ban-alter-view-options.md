# banAlterViewOptions

**Group** [`destructive`](../rules.md#destructive) · **Migrations only** · **Since** `Unreleased`  
**Diagnostic** `lint/banAlterViewOptions`

**Sources**: inspired by [`squawk/ban-alter-view-options`](https://squawkhq.com/docs/ban-alter-view-options)

## Description
Changing the options of a view may change behaviour for existing clients.

View options such as `security_barrier`, `security_invoker` and `check_option` decide
with whose privileges the view reads its tables, which rows it returns, and which writes
through it Postgres accepts.

## Examples

### Invalid

```sql
alter view active_users set (security_invoker = true);
```

```sh
code-block.sql:1:1 lint/banAlterViewOptions ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

  ! Changing the options of a view may change behaviour for existing clients.
  
  > 1 │ alter view active_users set (security_invoker = true);
      │ ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
    2 │ 
  
  i The options decide with whose privileges the view runs, which rows it returns, and which writes through it Postgres accepts.
  
  i Check that existing clients work with the new options before changing them.
  

```

```sql
alter view active_users reset (security_barrier);
```

```sh
code-block.sql:1:1 lint/banAlterViewOptions ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

  ! Changing the options of a view may change behaviour for existing clients.
  
  > 1 │ alter view active_users reset (security_barrier);
      │ ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
    2 │ 
  
  i The options decide with whose privileges the view runs, which rows it returns, and which writes through it Postgres accepts.
  
  i Check that existing clients work with the new options before changing them.
  

```

### Valid

```sql
alter view active_users rename to current_users;
```

## How to configure
```json

{
  "linter": {
    "rules": {
      "banAlterViewOptions": "error"
    }
  }
}

```
## How to suppress

Suppress this diagnostic with a comment:

```sql
-- pgls-ignore banAlterViewOptions
```
