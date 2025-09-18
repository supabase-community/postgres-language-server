# preferBigintOverSmallint
**Diagnostic Category: `lint/safety/preferBigintOverSmallint`**

**Since**: `vnext`


**Sources**: 
- Inspired from: <a href="https://squawkhq.com/docs/prefer-bigint-over-smallint" target="_blank"><code>squawk/prefer-bigint-over-smallint</code></a>

## Description
Prefer BIGINT over SMALLINT types.

SMALLINT has a very limited range (-32,768 to 32,767) that is easily exceeded.
Even for values that seem small initially, using SMALLINT can lead to problems
as your application grows.

The storage savings of SMALLINT (2 bytes) vs BIGINT (8 bytes) are negligible
on modern systems, while the cost of migrating when you exceed the limit is high.

## Examples

### Invalid

```sql
CREATE TABLE users (
    age smallint
);
```

```sh
code-block.sql:1:1 lint/safety/preferBigintOverSmallint ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

  ! SMALLINT has a very limited range that is easily exceeded.
  
  > 1 │ CREATE TABLE users (
      │ ^^^^^^^^^^^^^^^^^^^^
  > 2 │     age smallint
  > 3 │ );
      │ ^^
    4 │ 
  
  i SMALLINT can only store values from -32,768 to 32,767. This range is often insufficient.
  
  i Consider using INTEGER or BIGINT for better range and future-proofing.
  

```

```sql
CREATE TABLE products (
    quantity smallserial
);
```

```sh
code-block.sql:1:1 lint/safety/preferBigintOverSmallint ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

  ! SMALLINT has a very limited range that is easily exceeded.
  
  > 1 │ CREATE TABLE products (
      │ ^^^^^^^^^^^^^^^^^^^^^^^
  > 2 │     quantity smallserial
  > 3 │ );
      │ ^^
    4 │ 
  
  i SMALLINT can only store values from -32,768 to 32,767. This range is often insufficient.
  
  i Consider using INTEGER or BIGINT for better range and future-proofing.
  

```

### Valid

```sql
CREATE TABLE users (
    age integer
);
```

```sql
CREATE TABLE products (
    quantity bigint
);
```

## How to configure
```json

{
  "linter": {
    "rules": {
      "safety": {
        "preferBigintOverSmallint": "error"
      }
    }
  }
}

```
