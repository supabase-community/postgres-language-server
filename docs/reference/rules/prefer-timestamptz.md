# preferTimestamptz
**Diagnostic Category: `lint/safety/preferTimestamptz`**

**Since**: `vnext`


**Sources**: 
- Inspired from: <a href="https://squawkhq.com/docs/prefer-timestamptz" target="_blank"><code>squawk/prefer-timestamptz</code></a>

## Description
Prefer TIMESTAMPTZ over TIMESTAMP types.

Using TIMESTAMP WITHOUT TIME ZONE can lead to issues when dealing with time zones.
TIMESTAMPTZ (TIMESTAMP WITH TIME ZONE) stores timestamps with time zone information,
making it safer for applications that handle multiple time zones or need to track
when events occurred in absolute time.

## Examples

### Invalid

```sql
CREATE TABLE app.users (
    created_ts timestamp
);
```

```sh
code-block.sql:1:1 lint/safety/preferTimestamptz ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

  ! Prefer TIMESTAMPTZ over TIMESTAMP for better timezone handling.
  
  > 1 │ CREATE TABLE app.users (
      │ ^^^^^^^^^^^^^^^^^^^^^^^^
  > 2 │     created_ts timestamp
  > 3 │ );
      │ ^^
    4 │ 
  
  i TIMESTAMP WITHOUT TIME ZONE can lead to issues when dealing with time zones.
  
  i Use TIMESTAMPTZ (TIMESTAMP WITH TIME ZONE) instead.
  

```

```sql
CREATE TABLE app.accounts (
    created_ts timestamp without time zone
);
```

```sh
code-block.sql:1:1 lint/safety/preferTimestamptz ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

  ! Prefer TIMESTAMPTZ over TIMESTAMP for better timezone handling.
  
  > 1 │ CREATE TABLE app.accounts (
      │ ^^^^^^^^^^^^^^^^^^^^^^^^^^^
  > 2 │     created_ts timestamp without time zone
  > 3 │ );
      │ ^^
    4 │ 
  
  i TIMESTAMP WITHOUT TIME ZONE can lead to issues when dealing with time zones.
  
  i Use TIMESTAMPTZ (TIMESTAMP WITH TIME ZONE) instead.
  

```

```sql
ALTER TABLE app.users ALTER COLUMN created_ts TYPE timestamp;
```

```sh
code-block.sql:1:1 lint/safety/preferTimestamptz ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

  ! Prefer TIMESTAMPTZ over TIMESTAMP for better timezone handling.
  
  > 1 │ ALTER TABLE app.users ALTER COLUMN created_ts TYPE timestamp;
      │ ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
    2 │ 
  
  i TIMESTAMP WITHOUT TIME ZONE can lead to issues when dealing with time zones.
  
  i Use TIMESTAMPTZ (TIMESTAMP WITH TIME ZONE) instead.
  

```

### Valid

```sql
CREATE TABLE app.users (
    created_ts timestamptz
);
```

```sql
CREATE TABLE app.accounts (
    created_ts timestamp with time zone
);
```

```sql
ALTER TABLE app.users ALTER COLUMN created_ts TYPE timestamptz;
```

## How to configure
```json

{
  "linter": {
    "rules": {
      "safety": {
        "preferTimestamptz": "error"
      }
    }
  }
}

```
