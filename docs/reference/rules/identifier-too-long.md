# identifierTooLong

**Group** [`correctness`](../rules.md#correctness) · **Recommended** · **Since** `Unreleased`  
**Diagnostic** `lint/identifierTooLong`

**Sources**: inspired by [`squawk/identifier-too-long`](https://squawkhq.com/docs/identifier-too-long)

## Description
Identifiers longer than 63 bytes are silently truncated.

Postgres truncates longer identifiers to 63 bytes and only raises a notice, so the names
you write aren't the names Postgres uses. Two long names that share their first 63 bytes
even end up as the same name.

## Examples

### Invalid

```sql
create table customer_subscription_billing_cycle_adjustments_history_archives_v2 (id bigint);
```

```sh
code-block.sql:1:14 lint/identifierTooLong ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

  ! Identifier customer_subscription_billing_cycle_adjustments_history_archives_v2 is longer than 63 bytes.
  
  > 1 │ create table customer_subscription_billing_cycle_adjustments_history_archives_v2 (id bigint);
      │              ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
    2 │ 
  
  i Postgres truncates it to `customer_subscription_billing_cycle_adjustments_history_archive`.
  
  i Use a shorter name.
  

```

### Valid

```sql
create table subscription_adjustments_archive (id bigint);
```

## How to configure
```json

{
  "linter": {
    "rules": {
      "identifierTooLong": "error"
    }
  }
}

```
## How to suppress

Suppress this diagnostic with a comment:

```sql
-- pgls-ignore identifierTooLong
```
