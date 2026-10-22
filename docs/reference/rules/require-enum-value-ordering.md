# requireEnumValueOrdering

**Group** [`style`](../rules.md#style) · **Since** `Unreleased`  
**Diagnostic** `lint/requireEnumValueOrdering`

**Sources**: inspired by [`squawk/require-enum-value-ordering`](https://squawkhq.com/docs/require-enum-value-ordering)

## Description
`ALTER TYPE ... ADD VALUE` without `BEFORE` or `AFTER` appends the value to the end of the enum.

Enum values are ordered, and queries like `where status < 'approved'` depend on that
order. Appending a value that belongs in the middle silently changes what those queries
return.

Say where the new value belongs with `BEFORE` or `AFTER`.

## Examples

### Invalid

```sql
alter type order_status add value 'in_review';
```

```sh
code-block.sql:1:1 lint/requireEnumValueOrdering ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

  ! Enum value in_review is appended to the end of the enum.
  
  > 1 │ alter type order_status add value 'in_review';
      │ ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
    2 │ 
  
  i Queries that compare enum values depend on their order.
  
  i Add BEFORE or AFTER to specify the position of the new value.
  

```

### Valid

```sql
alter type order_status add value 'in_review' before 'approved';
```

## How to configure
```json

{
  "linter": {
    "rules": {
      "requireEnumValueOrdering": "error"
    }
  }
}

```
## How to suppress

Suppress this diagnostic with a comment:

```sql
-- pgls-ignore requireEnumValueOrdering
```
