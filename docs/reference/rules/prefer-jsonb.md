# preferJsonb
**Diagnostic Category: `lint/safety/preferJsonb`**

**Since**: `vnext`


**Sources**: 
- Inspired from: <a href="https://kaveland.no/eugene/hints/E3/index.html" target="_blank"><code>eugene/E3</code></a>

## Description
Prefer JSONB over JSON types.

JSONB is the binary JSON data type in PostgreSQL that is more efficient for most use cases.
Unlike JSON, JSONB stores data in a decomposed binary format which provides several advantages:

- Significantly faster query performance for operations like indexing and searching
- Support for indexing (GIN indexes)
- Duplicate keys are automatically removed
- Keys are sorted

The only reasons to use JSON instead of JSONB are:

- You need to preserve exact input formatting (whitespace, key order)
- You need to preserve duplicate keys
- You have very specific performance requirements where JSON's lack of parsing overhead matters

## Examples

### Invalid

```sql
CREATE TABLE users (
    data json
);
```

```sh
code-block.sql:1:1 lint/safety/preferJsonb ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

  ! Prefer JSONB over JSON for better performance and functionality.
  
  > 1 │ CREATE TABLE users (
      │ ^^^^^^^^^^^^^^^^^^^^
  > 2 │     data json
  > 3 │ );
      │ ^^
    4 │ 
  
  i JSON stores exact text representation while JSONB stores parsed binary format. JSONB is faster for queries, supports indexing, and removes duplicate keys.
  
  i Consider using JSONB instead unless you specifically need to preserve formatting or duplicate keys.
  

```

```sql
ALTER TABLE users ADD COLUMN metadata json;
```

```sh
code-block.sql:1:1 lint/safety/preferJsonb ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

  ! Prefer JSONB over JSON for better performance and functionality.
  
  > 1 │ ALTER TABLE users ADD COLUMN metadata json;
      │ ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
    2 │ 
  
  i JSON stores exact text representation while JSONB stores parsed binary format. JSONB is faster for queries, supports indexing, and removes duplicate keys.
  
  i Consider using JSONB instead unless you specifically need to preserve formatting or duplicate keys.
  

```

```sql
ALTER TABLE users ALTER COLUMN data TYPE json;
```

```sh
code-block.sql:1:1 lint/safety/preferJsonb ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

  ! Prefer JSONB over JSON for better performance and functionality.
  
  > 1 │ ALTER TABLE users ALTER COLUMN data TYPE json;
      │ ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
    2 │ 
  
  i JSON stores exact text representation while JSONB stores parsed binary format. JSONB is faster for queries, supports indexing, and removes duplicate keys.
  
  i Consider using JSONB instead unless you specifically need to preserve formatting or duplicate keys.
  

```

### Valid

```sql
CREATE TABLE users (
    data jsonb
);
```

```sql
ALTER TABLE users ADD COLUMN metadata jsonb;
```

```sql
ALTER TABLE users ALTER COLUMN data TYPE jsonb;
```

## How to configure
```json

{
  "linter": {
    "rules": {
      "safety": {
        "preferJsonb": "error"
      }
    }
  }
}

```
