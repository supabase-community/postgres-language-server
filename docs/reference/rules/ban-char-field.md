# banCharField
**Diagnostic Category: `lint/safety/banCharField`**

**Since**: `vnext`


**Sources**: 
- Inspired from: <a href="https://squawkhq.com/docs/ban-char-field" target="_blank"><code>squawk/ban-char-field</code></a>

## Description
Using CHAR(n) or CHARACTER(n) types is discouraged.

CHAR types are fixed-length and padded with spaces, which can lead to unexpected behavior
when comparing strings or concatenating values. They also waste storage space when values
are shorter than the declared length.

Use VARCHAR or TEXT instead for variable-length character data.

## Examples

### Invalid

```sql
CREATE TABLE "core_bar" (
    "id" serial NOT NULL PRIMARY KEY,
    "alpha" char(100) NOT NULL
);
```

```sh
code-block.sql:1:1 lint/safety/banCharField ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

  ! CHAR type is discouraged due to space padding behavior.
  
  > 1 │ CREATE TABLE "core_bar" (
      │ ^^^^^^^^^^^^^^^^^^^^^^^^^
  > 2 │     "id" serial NOT NULL PRIMARY KEY,
  > 3 │     "alpha" char(100) NOT NULL
  > 4 │ );
      │ ^^
    5 │ 
  
  i CHAR types are fixed-length and padded with spaces, which can lead to unexpected behavior.
  
  i Use VARCHAR or TEXT instead for variable-length character data.
  

```

### Valid

```sql
CREATE TABLE "core_bar" (
    "id" serial NOT NULL PRIMARY KEY,
    "alpha" varchar(100) NOT NULL
);
```

## How to configure
```json

{
  "linter": {
    "rules": {
      "safety": {
        "banCharField": "error"
      }
    }
  }
}

```
