# unknownType

**Group** [`typecheck`](../rules.md#typecheck) · **Recommended** · **Needs a database connection** · **Since** `0.27.0`  
**Diagnostic** `lint/unknownType` · **Postgres error** `42704`

## Description
A type does not exist.

Types are looked up in the connected database and in the types created earlier in the same
file, following the search path.

Postgres raises `42704 undefined_object` for these statements.

## Examples

### Invalid

```sql
select null::missing_type;
```

```sh
code-block.sql:1:14 lint/unknownType ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

  × Type missing_type does not exist.
  
  > 1 │ select null::missing_type;
      │              ^^^^^^^^^^^^
    2 │ 
  

```

### Valid

```sql
create type mood as enum ('happy', 'sad');
select null::mood;
```

## How to configure
```json

{
  "linter": {
    "rules": {
      "unknownType": "error"
    }
  }
}

```
## How to suppress

Suppress this diagnostic with a comment:

```sql
-- pgls-ignore unknownType
```
