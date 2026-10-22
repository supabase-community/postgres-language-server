# banDropExtension

**Group** [`destructive`](../rules.md#destructive) · **Migrations only** · **Since** `Unreleased`  
**Diagnostic** `lint/banDropExtension`

**Sources**: inspired by [`squawk/ban-drop-extension`](https://squawkhq.com/docs/ban-drop-extension)

## Description
Dropping an extension may break existing clients.

`DROP EXTENSION` also removes the functions, types, operators and other objects of the
extension, which existing clients may depend on.

## Examples

### Invalid

```sql
drop extension hstore;
```

```sh
code-block.sql:1:1 lint/banDropExtension ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

  ! Dropping an extension may break existing clients.
  
  > 1 │ drop extension hstore;
      │ ^^^^^^^^^^^^^^^^^^^^^^
    2 │ 
  
  i Dropping an extension also removes its functions, types, and other objects.
  
  i Update clients to no longer use the objects of the extension before dropping it.
  

```

## How to configure
```json

{
  "linter": {
    "rules": {
      "banDropExtension": "error"
    }
  }
}

```
## How to suppress

Suppress this diagnostic with a comment:

```sql
-- pgls-ignore banDropExtension
```
