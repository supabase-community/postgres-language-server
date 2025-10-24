# creatingEnum
**Diagnostic Category: `lint/safety/creatingEnum`**

**Since**: `vnext`


**Sources**: 
- Inspired from: <a href="https://kaveland.no/eugene/hints/W13/index.html" target="_blank"><code>eugene/W13</code></a>

## Description
Creating enum types is not recommended for new applications.

Enumerated types have several limitations that make them difficult to work with in production:

- Removing values from an enum requires complex migrations and is not supported directly
- Adding values to an enum requires an ACCESS EXCLUSIVE lock in some PostgreSQL versions
- Associating additional data with enum values is impossible without restructuring
- Renaming enum values requires careful migration planning

A lookup table with a foreign key constraint provides more flexibility and is easier to maintain.

## Examples

### Invalid

```sql
CREATE TYPE document_type AS ENUM ('invoice', 'receipt', 'other');
```

```sh
code-block.sql:1:1 lint/safety/creatingEnum ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

  ! Creating enum type document_type is not recommended.
  
  > 1 │ CREATE TYPE document_type AS ENUM ('invoice', 'receipt', 'other');
      │ ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
    2 │ 
  
  i Enum types are difficult to modify: removing values requires complex migrations, and associating additional data with values is not possible.
  
  i Consider using a lookup table with a foreign key constraint instead, which provides more flexibility and easier maintenance.
  

```

### Valid

```sql
-- Use a lookup table instead
CREATE TABLE document_type (
    type_name TEXT PRIMARY KEY
);
INSERT INTO document_type VALUES ('invoice'), ('receipt'), ('other');
```

## How to configure
```json

{
  "linter": {
    "rules": {
      "safety": {
        "creatingEnum": "error"
      }
    }
  }
}

```
