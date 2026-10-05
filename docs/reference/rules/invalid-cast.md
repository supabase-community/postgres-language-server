# invalidCast

**Group** [`typecheck`](../rules.md#typecheck) · **Recommended** · **Needs a database connection** · **Since** `0.27.0`  
**Diagnostic** `lint/invalidCast` · **Postgres error** `42846`

## Description
An explicit cast is not permitted between the source and target types.

Postgres raises `42846 cannot_coerce` for these statements.

## Examples

### Invalid

```sql
select timestamp '2020-01-01'::integer;
```

```sh
code-block.sql:1:30 lint/invalidCast ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

  × Cannot cast type timestamp without time zone to integer.
  
  > 1 │ select timestamp '2020-01-01'::integer;
      │                              ^^^^^^^^^
    2 │ 
  

```

### Valid

```sql
select 1::text;
```

## How to configure
```json

{
  "linter": {
    "rules": {
      "invalidCast": "error"
    }
  }
}

```
## How to suppress

Suppress this diagnostic with a comment:

```sql
-- pgls-ignore invalidCast
```
