# banAlterExtension

**Group** [`destructive`](../rules.md#destructive) · **Migrations only** · **Since** `Unreleased`  
**Diagnostic** `lint/banAlterExtension`

**Sources**: inspired by [`squawk/ban-alter-extension`](https://squawkhq.com/docs/ban-alter-extension)

## Description
Updating an extension or removing an object from it may break existing clients.

`ALTER EXTENSION ... UPDATE` runs the upgrade scripts of the extension, which may remove
functions, change their signatures, or change their results. `ALTER EXTENSION ... DROP`
removes an object from the extension, so later updates of the extension no longer manage
it.

Adding an object to an extension is not reported. Moving an extension to another schema
is reported by `banSetSchema`.

## Examples

### Invalid

```sql
alter extension postgis update to '3.4.0';
```

```sh
code-block.sql:1:1 lint/banAlterExtension ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

  ! Updating extension postgis may break existing clients.
  
  > 1 │ alter extension postgis update to '3.4.0';
      │ ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
    2 │ 
  
  i The upgrade scripts of the extension may remove functions, change their signatures, or change their results.
  
  i Read the upgrade notes of the extension and update clients before updating it.
  

```

```sql
alter extension postgis drop function legacy_distance(geometry, geometry);
```

```sh
code-block.sql:1:1 lint/banAlterExtension ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

  ! Removing an object from extension postgis may break existing clients.
  
  > 1 │ alter extension postgis drop function legacy_distance(geometry, geometry);
      │ ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
    2 │ 
  
  i Later updates of the extension no longer manage the object, so it keeps its old definition.
  
  i Make sure that nothing relies on the extension managing the object before removing it.
  

```

### Valid

```sql
alter extension postgis add function legacy_distance(geometry, geometry);
```

## How to configure
```json

{
  "linter": {
    "rules": {
      "banAlterExtension": "error"
    }
  }
}

```
## How to suppress

Suppress this diagnostic with a comment:

```sql
-- pgls-ignore banAlterExtension
```
