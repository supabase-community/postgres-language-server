# preferBigintOverInt
**Diagnostic Category: `lint/safety/preferBigintOverInt`**

**Since**: `vnext`


**Sources**: 
- Inspired from: <a href="https://squawkhq.com/docs/prefer-bigint-over-int" target="_blank"><code>squawk/prefer-bigint-over-int</code></a>

## Description
Prefer BIGINT over INT/INTEGER types.

Using INTEGER (INT4) can lead to overflow issues, especially for ID columns.
While SMALLINT might be acceptable for certain use cases with known small ranges,
INTEGER often becomes a limiting factor as applications grow.

The storage difference between INTEGER (4 bytes) and BIGINT (8 bytes) is minimal,
but the cost of migrating when you hit the 2.1 billion limit can be significant.

## Examples

### Invalid

```sql
CREATE TABLE users (
    id integer
);
```

```sh
code-block.sql:1:1 lint/safety/preferBigintOverInt ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

  ! INTEGER type may lead to overflow issues.
  
  > 1 │ CREATE TABLE users (
      │ ^^^^^^^^^^^^^^^^^^^^
  > 2 │     id integer
  > 3 │ );
      │ ^^
    4 │ 
  
  i INTEGER has a maximum value of 2,147,483,647 which can be exceeded by ID columns and counters.
  
  i Consider using BIGINT instead for better future-proofing.
  

```

```sql
CREATE TABLE users (
    id serial
);
```

```sh
code-block.sql:1:1 lint/safety/preferBigintOverInt ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

  ! INTEGER type may lead to overflow issues.
  
  > 1 │ CREATE TABLE users (
      │ ^^^^^^^^^^^^^^^^^^^^
  > 2 │     id serial
  > 3 │ );
      │ ^^
    4 │ 
  
  i INTEGER has a maximum value of 2,147,483,647 which can be exceeded by ID columns and counters.
  
  i Consider using BIGINT instead for better future-proofing.
  

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

```sql
CREATE TABLE users (
    id smallint
);
```

## How to configure
```json

{
  "linter": {
    "rules": {
      "safety": {
        "preferBigintOverInt": "error"
      }
    }
  }
}

```
