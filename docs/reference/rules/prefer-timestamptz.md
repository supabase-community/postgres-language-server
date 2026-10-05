# preferTimestamptz

**Group** [`style`](../rules.md#style) · **Since** `0.15.0`  
**Diagnostic** `lint/preferTimestamptz`

**Sources**: inspired by [`squawk/prefer-timestamptz`](https://squawkhq.com/docs/prefer-timestamptz)

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
code-block.sql:1:1 lint/preferTimestamptz ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

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
code-block.sql:1:1 lint/preferTimestamptz ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

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
code-block.sql:1:1 lint/preferTimestamptz ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

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
      "preferTimestamptz": "error"
    }
  }
}

```
## How to suppress

Suppress this diagnostic with a comment:

```sql
-- pgls-ignore preferTimestamptz
```
