# banAlterRoleOptions

**Group** [`destructive`](../rules.md#destructive) · **Migrations only** · **Since** `Unreleased`  
**Diagnostic** `lint/banAlterRoleOptions`

**Sources**: inspired by [`squawk/ban-alter-role-options`](https://squawkhq.com/docs/ban-alter-role-options)

## Description
Changing the options or settings of a role may change access or behaviour for existing
clients.

Role options such as `LOGIN`, `BYPASSRLS` or `CONNECTION LIMIT` decide whether clients
can connect as the role and what they can access. `SET` and `RESET` change the settings
of their sessions, e.g. the `search_path`, including for a single database with
`IN DATABASE`. `ALTER USER` is an alias of `ALTER ROLE` and is reported too.

Renaming a role and changing the members of a group with `ALTER GROUP` are not reported.

## Examples

### Invalid

```sql
alter role app_user nologin;
```

```sh
code-block.sql:1:1 lint/banAlterRoleOptions ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

  ! Changing the options of a role may change access for existing clients.
  
  > 1 │ alter role app_user nologin;
      │ ^^^^^^^^^^^^^^^^^^^^^^^^^^^^
    2 │ 
  
  i The options decide whether clients can connect as the role and what they can access.
  
  i Check that the clients that use the role work with the change before making it.
  

```

```sql
alter user app_user in database app_db set search_path to private;
```

```sh
code-block.sql:1:1 lint/banAlterRoleOptions ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

  ! Changing the settings of a role may change behaviour for existing clients.
  
  > 1 │ alter user app_user in database app_db set search_path to private;
      │ ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
    2 │ 
  
  i The settings apply to every new session of the role, e.g. which schemas its queries find.
  
  i Check that the clients that use the role work with the change before making it.
  

```

### Valid

```sql
alter role app_user rename to api_user;
```

## How to configure
```json

{
  "linter": {
    "rules": {
      "banAlterRoleOptions": "error"
    }
  }
}

```
## How to suppress

Suppress this diagnostic with a comment:

```sql
-- pgls-ignore banAlterRoleOptions
```
