# banAlterFunctionOptions

**Group** [`destructive`](../rules.md#destructive) · **Migrations only** · **Since** `Unreleased`  
**Diagnostic** `lint/banAlterFunctionOptions`

**Sources**: inspired by [`squawk/ban-alter-function-options`](https://squawkhq.com/docs/ban-alter-function-options)

## Description
Changing the options of a function, procedure or routine may change behaviour for existing
clients.

Options such as `SECURITY DEFINER`, `SET search_path`, the volatility or `STRICT` change
with which privileges the function runs, which objects it finds, and what it returns.

Renaming the function, changing its owner or its schema are not reported.

## Examples

### Invalid

```sql
alter function add_user(text) security definer;
```

```sh
code-block.sql:1:1 lint/banAlterFunctionOptions ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

  ! Changing the options of a function may change behaviour for existing clients.
  
  > 1 │ alter function add_user(text) security definer;
      │ ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
    2 │ 
  
  i The options decide with which privileges and settings it runs, and how Postgres calls it.
  
  i Check that existing clients work with the new options before changing them.
  

```

```sql
alter procedure archive_orders() set search_path to private;
```

```sh
code-block.sql:1:1 lint/banAlterFunctionOptions ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

  ! Changing the options of a procedure may change behaviour for existing clients.
  
  > 1 │ alter procedure archive_orders() set search_path to private;
      │ ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
    2 │ 
  
  i The options decide with which privileges and settings it runs, and how Postgres calls it.
  
  i Check that existing clients work with the new options before changing them.
  

```

### Valid

```sql
alter function add_user(text) rename to create_user;
```

## How to configure
```json

{
  "linter": {
    "rules": {
      "banAlterFunctionOptions": "error"
    }
  }
}

```
## How to suppress

Suppress this diagnostic with a comment:

```sql
-- pgls-ignore banAlterFunctionOptions
```
