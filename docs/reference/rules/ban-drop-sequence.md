# banDropSequence

**Group** [`destructive`](../rules.md#destructive) · **Migrations only** · **Since** `Unreleased`  
**Diagnostic** `lint/banDropSequence`

**Sources**: inspired by [`squawk/ban-drop-sequence`](https://squawkhq.com/docs/ban-drop-sequence)

## Description
Dropping a sequence may break existing clients.

An earlier application revision may still call `nextval('my_seq')` after the migration
dropped the sequence. Rolling back the application does not restore the sequence, so that
call fails. With `CASCADE`, column defaults that use the sequence are dropped too.

Update clients to stop using the sequence, and drop it in a later migration.

## Examples

### Invalid

```sql
drop sequence order_number_seq;
```

```sh
code-block.sql:1:1 lint/banDropSequence ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

  ! Dropping a sequence may break existing clients.
  
  > 1 │ drop sequence order_number_seq;
      │ ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
    2 │ 
  
  i Clients that still call nextval() on the sequence fail, and rolling back the application does not restore it.
  
  i Update your application code to stop using the sequence, then drop it in a later migration.
  

```

## How to configure
```json

{
  "linter": {
    "rules": {
      "banDropSequence": "error"
    }
  }
}

```
## How to suppress

Suppress this diagnostic with a comment:

```sql
-- pgls-ignore banDropSequence
```
