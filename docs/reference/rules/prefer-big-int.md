# preferBigInt
**Diagnostic Category: `lint/safety/preferBigInt`**

**Since**: `vnext`


**Sources**: 
- Inspired from: <a href="https://squawkhq.com/docs/prefer-big-int" target="_blank"><code>squawk/prefer-big-int</code></a>

## Description
Prefer BIGINT over smaller integer types.

Using smaller integer types like SMALLINT, INTEGER, or their aliases can lead to overflow
issues as your application grows. BIGINT provides a much larger range and helps avoid
future migration issues when values exceed the limits of smaller types.

The storage difference between INTEGER (4 bytes) and BIGINT (8 bytes) is minimal on
modern systems, while the cost of migrating to a larger type later can be significant.

## Examples

### Invalid

```sql
CREATE TABLE users (
    id integer
);
```

```sh
code-block.sql:1:1 lint/safety/preferBigInt ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

  ! Using smaller integer types can lead to overflow issues.
  
  > 1 │ CREATE TABLE users (
      │ ^^^^^^^^^^^^^^^^^^^^
  > 2 │     id integer
  > 3 │ );
      │ ^^
    4 │ 
  
  i The 'int4' type has a limited range that may be exceeded as your data grows.
  
  i Consider using BIGINT for integer columns to avoid future migration issues.
  

```

```sql
CREATE TABLE users (
    id serial
);
```

```sh
code-block.sql:1:1 lint/safety/preferBigInt ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

  ! Using smaller integer types can lead to overflow issues.
  
  > 1 │ CREATE TABLE users (
      │ ^^^^^^^^^^^^^^^^^^^^
  > 2 │     id serial
  > 3 │ );
      │ ^^
    4 │ 
  
  i The 'serial' type has a limited range that may be exceeded as your data grows.
  
  i Consider using BIGINT for integer columns to avoid future migration issues.
  

```

### Valid

```sql
CREATE TABLE users (
    id bigint
);
```

```sql
CREATE TABLE users (
    id bigserial
);
```

## How to configure
```json

{
  "linter": {
    "rules": {
      "safety": {
        "preferBigInt": "error"
      }
    }
  }
}

```
