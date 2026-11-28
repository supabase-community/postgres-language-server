# banAlterSystemOptions

**Group** [`destructive`](../rules.md#destructive) · **Migrations only** · **Since** `Unreleased`  
**Diagnostic** `lint/banAlterSystemOptions`

**Sources**: inspired by [`squawk/ban-alter-system-options`](https://squawkhq.com/docs/ban-alter-system-options)

## Description
Changing the server configuration may change behaviour for existing clients.

`ALTER SYSTEM` writes settings to `postgresql.auto.conf`, which apply to every database
and session of the server once the configuration is reloaded.

## Examples

### Invalid

```sql
alter system set timezone = 'UTC';
```

```sh
code-block.sql:1:1 lint/banAlterSystemOptions ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

  ! Changing the server configuration may change behaviour for existing clients.
  
  > 1 │ alter system set timezone = 'UTC';
      │ ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
    2 │ 
  
  i The settings apply to every database and session of the server once the configuration is reloaded.
  
  i Change the setting for a database or role instead, or check that every client of the server works with it.
  

```

```sql
alter system reset all;
```

```sh
code-block.sql:1:1 lint/banAlterSystemOptions ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

  ! Changing the server configuration may change behaviour for existing clients.
  
  > 1 │ alter system reset all;
      │ ^^^^^^^^^^^^^^^^^^^^^^^
    2 │ 
  
  i The settings apply to every database and session of the server once the configuration is reloaded.
  
  i Change the setting for a database or role instead, or check that every client of the server works with it.
  

```

### Valid

```sql
set timezone = 'UTC';
```

## How to configure
```json

{
  "linter": {
    "rules": {
      "banAlterSystemOptions": "error"
    }
  }
}

```
## How to suppress

Suppress this diagnostic with a comment:

```sql
-- pgls-ignore banAlterSystemOptions
```
