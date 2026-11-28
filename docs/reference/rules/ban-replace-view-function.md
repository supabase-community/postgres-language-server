# banReplaceViewFunction

**Group** [`destructive`](../rules.md#destructive) · **Migrations only** · **Since** `Unreleased`  
**Diagnostic** `lint/banReplaceViewFunction`

**Sources**: inspired by [`squawk/ban-replace-view-function`](https://squawkhq.com/docs/ban-replace-view-function)

## Description
Replacing a view, function, procedure, trigger, rule or aggregate may change behaviour for
existing clients.

`CREATE OR REPLACE` changes what existing clients get back or what their statements do,
without any change to their SQL.

`CREATE OR REPLACE AGGREGATE` is only checked on Postgres 12 and later, and
`CREATE OR REPLACE TRIGGER` on Postgres 14 and later, which added them.

## Examples

### Invalid

```sql
create or replace view active_users as select id, email from users where active;
```

```sh
code-block.sql:1:1 lint/banReplaceViewFunction ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

  ! Replacing a view may change behaviour for existing clients.
  
  > 1 │ create or replace view active_users as select id, email from users where active;
      │ ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
    2 │ 
  
  i Existing clients get different results or side effects without any change to their SQL.
  
  i Check that existing clients work with the new definition, or create a new object and move clients to it.
  

```

```sql
create or replace function total(a int, b int) returns int language sql as $$ select a + b $$;
```

```sh
code-block.sql:1:1 lint/banReplaceViewFunction ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

  ! Replacing a function may change behaviour for existing clients.
  
  > 1 │ create or replace function total(a int, b int) returns int language sql as $$ select a + b $$;
      │ ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
    2 │ 
  
  i Existing clients get different results or side effects without any change to their SQL.
  
  i Check that existing clients work with the new definition, or create a new object and move clients to it.
  

```

### Valid

```sql
create view active_users as select id, email from users where active;
```

## How to configure
```json

{
  "linter": {
    "rules": {
      "banReplaceViewFunction": "error"
    }
  }
}

```
## How to suppress

Suppress this diagnostic with a comment:

```sql
-- pgls-ignore banReplaceViewFunction
```
