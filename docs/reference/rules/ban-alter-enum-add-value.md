# banAlterEnumAddValue
**Diagnostic Category: `lint/safety/banAlterEnumAddValue`**

**Since**: `vnext`


**Sources**: 
- Inspired from: <a href="https://github.com/flvmnt/pgfence" target="_blank"><code>pgfence/alter-enum-add-value</code></a>

## Description
`ALTER TYPE ... ADD VALUE` cannot run inside a transaction block in older Postgres versions.

Adding a value to an enum type acquires an `ACCESS EXCLUSIVE` lock on the enum type.
In Postgres versions before 12, `ALTER TYPE ... ADD VALUE` cannot be executed inside a
transaction block. Even in newer versions, the new value cannot be used in the same
transaction until it is committed.

## Examples

### Invalid

```sql
alter type my_enum add value 'new_value';
```

```sh
code-block.sql:1:1 lint/safety/banAlterEnumAddValue ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

  ! ALTER TYPE ... ADD VALUE acquires an ACCESS EXCLUSIVE lock on the enum type.
  
  > 1 │ alter type my_enum add value 'new_value';
      │ ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
    2 │ 
  
  i The new enum value cannot be used in the same transaction. In Postgres versions before 12, this statement cannot run inside a transaction block at all.
  

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
