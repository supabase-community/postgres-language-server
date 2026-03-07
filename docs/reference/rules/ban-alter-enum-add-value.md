# banAlterEnumAddValue
**Diagnostic Category: `lint/safety/banAlterEnumAddValue`**

**Since**: `vnext`


**Sources**: 
- Inspired from: <a href="https://github.com/flvmnt/pgfence" target="_blank"><code>pgfence/alter-enum-add-value</code></a>

## Description
`ALTER TYPE ... ADD VALUE` cannot run inside a transaction block in older Postgres versions.

In Postgres versions before 12, `ALTER TYPE ... ADD VALUE` cannot be executed inside a
transaction block at all. On Postgres 12+, the operation is fast (metadata-only), but the
new enum value cannot be used in the same transaction until it is committed.

## Examples

### Invalid

```sql
alter type my_enum add value 'new_value';
```

```sh
code-block.sql:1:1 lint/safety/banAlterEnumAddValue ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

  ! ALTER TYPE ... ADD VALUE cannot be used in a transaction block before Postgres 12.
  
  > 1 │ alter type my_enum add value 'new_value';
      │ ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
    2 │ 
  
  i On Postgres 12+, the operation is fast but the new value cannot be used in the same transaction until committed.
  

```

### Valid

```sql
select 1;
```

## How to configure
```json

{
  "linter": {
    "rules": {
      "safety": {
        "banAlterEnumAddValue": "error"
      }
    }
  }
}

```
