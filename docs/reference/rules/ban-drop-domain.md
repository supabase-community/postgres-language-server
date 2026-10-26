# banDropDomain

**Group** [`destructive`](../rules.md#destructive) · **Migrations only** · **Since** `Unreleased`  
**Diagnostic** `lint/banDropDomain`

**Sources**: inspired by [`squawk/ban-drop-domain`](https://squawkhq.com/docs/ban-drop-domain)

## Description
Dropping a domain may break existing clients.

Clients that use the domain in casts or parameters fail. With `CASCADE`, columns of the
domain are dropped too.

Move clients to a replacement type before dropping the domain.

## Examples

### Invalid

```sql
drop domain email;
```

```sh
code-block.sql:1:1 lint/banDropDomain ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

  ! Dropping a domain may break existing clients.
  
  > 1 │ drop domain email;
      │ ^^^^^^^^^^^^^^^^^^
    2 │ 
  
  i Casts and parameters that name the domain fail once it is gone.
  
  i Move clients to a replacement type, then drop the domain in a later migration.
  

```

## How to configure
```json

{
  "linter": {
    "rules": {
      "banDropDomain": "error"
    }
  }
}

```
## How to suppress

Suppress this diagnostic with a comment:

```sql
-- pgls-ignore banDropDomain
```
