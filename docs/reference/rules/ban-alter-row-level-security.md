# banAlterRowLevelSecurity

**Group** [`destructive`](../rules.md#destructive) · **Migrations only** · **Since** `Unreleased`  
**Diagnostic** `lint/banAlterRowLevelSecurity`

**Sources**: inspired by [`squawk/ban-alter-row-level-security`](https://squawkhq.com/docs/ban-alter-row-level-security)

## Description
Changing row level security on a table may change access for existing clients.

Enabling row level security hides every row from clients that no policy lets through,
and disabling it gives them access to all rows again. `FORCE` and `NO FORCE` decide
whether the policies apply to the owner of the table too.

## Examples

### Invalid

```sql
alter table accounts enable row level security;
```

```sh
code-block.sql:1:1 lint/banAlterRowLevelSecurity ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

  ! Changing row level security may change access for existing clients.
  
  > 1 │ alter table accounts enable row level security;
      │ ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
    2 │ 
  
  i Existing clients may no longer see rows that no policy lets through, or see rows the policies hid.
  
  i Create the policies that existing clients need before enabling row level security, and check who relies on it before disabling it.
  

```

```sql
alter table accounts no force row level security;
```

```sh
code-block.sql:1:1 lint/banAlterRowLevelSecurity ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

  ! Changing row level security may change access for existing clients.
  
  > 1 │ alter table accounts no force row level security;
      │ ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
    2 │ 
  
  i Existing clients may no longer see rows that no policy lets through, or see rows the policies hid.
  
  i Create the policies that existing clients need before enabling row level security, and check who relies on it before disabling it.
  

```

### Valid

```sql
alter table accounts add column tenant_id int;
```

## How to configure
```json

{
  "linter": {
    "rules": {
      "banAlterRowLevelSecurity": "error"
    }
  }
}

```
## How to suppress

Suppress this diagnostic with a comment:

```sql
-- pgls-ignore banAlterRowLevelSecurity
```
