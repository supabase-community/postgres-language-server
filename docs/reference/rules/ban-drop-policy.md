# banDropPolicy

**Group** [`destructive`](../rules.md#destructive) · **Migrations only** · **Since** `Unreleased`  
**Diagnostic** `lint/banDropPolicy`

**Sources**: inspired by [`squawk/ban-drop-policy`](https://squawkhq.com/docs/ban-drop-policy)

## Description
Dropping a policy or rule may change behaviour for existing clients.

Dropping a row level security policy changes which rows existing clients can read and
write. Dropping a rule changes what their queries on the table or view do.

## Examples

### Invalid

```sql
drop policy if exists tenant_isolation on accounts;
```

```sh
code-block.sql:1:1 lint/banDropPolicy ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

  ! Dropping a policy may change access for existing clients.
  
  > 1 │ drop policy if exists tenant_isolation on accounts;
      │ ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
    2 │ 
  
  i Existing clients may be able to read or write different rows once the policy is gone.
  
  i Update clients that depend on it before dropping it.
  

```

```sql
drop rule if exists protect_accounts on accounts;
```

```sh
code-block.sql:1:1 lint/banDropPolicy ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

  ! Dropping a rule may change behaviour for existing clients.
  
  > 1 │ drop rule if exists protect_accounts on accounts;
      │ ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
    2 │ 
  
  i Postgres no longer rewrites the queries of existing clients on the table or view.
  
  i Update clients that depend on it before dropping it.
  

```

### Valid

```sql
create policy tenant_isolation on accounts using (tenant_id = current_setting('app.tenant_id')::int);
```

## How to configure
```json

{
  "linter": {
    "rules": {
      "banDropPolicy": "error"
    }
  }
}

```
## How to suppress

Suppress this diagnostic with a comment:

```sql
-- pgls-ignore banDropPolicy
```
