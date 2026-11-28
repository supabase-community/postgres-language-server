# banAlterDatabaseOptions

**Group** [`destructive`](../rules.md#destructive) · **Migrations only** · **Since** `Unreleased`  
**Diagnostic** `lint/banAlterDatabaseOptions`

**Sources**: inspired by [`squawk/ban-alter-database-options`](https://squawkhq.com/docs/ban-alter-database-options)

## Description
Changing the options or settings of a database may change behaviour for existing clients.

Database options such as `ALLOW_CONNECTIONS` or `CONNECTION LIMIT` decide whether clients
can connect. `SET` and `RESET` change the settings of every new session in the database,
e.g. the `search_path` or the time zone.

Renaming the database, changing its owner or its tablespace, and refreshing its collation
version are not reported.

## Examples

### Invalid

```sql
alter database app set search_path to app, public;
```

```sh
code-block.sql:1:1 lint/banAlterDatabaseOptions ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

  ! Changing the settings of a database may change behaviour for existing clients.
  
  > 1 │ alter database app set search_path to app, public;
      │ ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
    2 │ 
  
  i The settings apply to every new session in the database, e.g. which schemas its queries find.
  
  i Check that the clients of the database work with the change before making it.
  

```

```sql
alter database app with connection limit 5;
```

```sh
code-block.sql:1:1 lint/banAlterDatabaseOptions ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

  ! Changing the options of a database may change behaviour for existing clients.
  
  > 1 │ alter database app with connection limit 5;
      │ ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
    2 │ 
  
  i The options decide whether and how many clients can connect to the database.
  
  i Check that the clients of the database work with the change before making it.
  

```

### Valid

```sql
alter database app rename to app_v2;
```

## How to configure
```json

{
  "linter": {
    "rules": {
      "banAlterDatabaseOptions": "error"
    }
  }
}

```
## How to suppress

Suppress this diagnostic with a comment:

```sql
-- pgls-ignore banAlterDatabaseOptions
```
