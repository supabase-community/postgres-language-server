# invalidDropTypeSignature

**Group** [`correctness`](../rules.md#correctness) · **Recommended** · **Since** `0.27.0`  
**Diagnostic** `lint/invalidDropTypeSignature` · **Postgres error** `42601`

## Description
`DROP TYPE` and `DROP DOMAIN` don't take a parameter list.

Unlike `DROP FUNCTION`, types are dropped by name only. Postgres parses the parameter list
as type modifiers and raises `42601 syntax_error` ("type modifier is not allowed").

## Examples

### Invalid

```sql
drop type if exists group_composite (int, text);
```

```sh
code-block.sql:1:1 lint/invalidDropTypeSignature ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

  × Types are dropped by name only, without a parameter list.
  
  > 1 │ drop type if exists group_composite (int, text);
      │ ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
    2 │ 
  
  i Remove the parameter list.
  

```

### Valid

```sql
drop type if exists group_composite;
```

## How to configure
```json

{
  "linter": {
    "rules": {
      "invalidDropTypeSignature": "error"
    }
  }
}

```
## How to suppress

Suppress this diagnostic with a comment:

```sql
-- pgls-ignore invalidDropTypeSignature
```
