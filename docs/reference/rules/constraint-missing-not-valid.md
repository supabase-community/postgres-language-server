# constraintMissingNotValid
**Diagnostic Category: `lint/safety/constraintMissingNotValid`**

**Since**: `vnext`


**Sources**: 
- Inspired from: <a href="https://squawkhq.com/docs/constraint-missing-not-valid" target="_blank"><code>squawk/constraint-missing-not-valid</code></a>

## Description
Adding constraints without NOT VALID blocks all reads and writes.

When adding a CHECK or FOREIGN KEY constraint, PostgreSQL must validate all existing rows,
which requires a full table scan. This blocks reads and writes for the duration.

Instead, add the constraint with NOT VALID first, then VALIDATE CONSTRAINT in a separate
transaction. This allows reads and writes to continue while validation happens.

## Examples

### Invalid

```sql
ALTER TABLE distributors ADD CONSTRAINT distfk FOREIGN KEY (address) REFERENCES addresses (address);
```

```sh
code-block.sql:1:1 lint/safety/constraintMissingNotValid ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

  ! Adding a constraint without NOT VALID will block reads and writes while validating existing rows.
  
  > 1 │ ALTER TABLE distributors ADD CONSTRAINT distfk FOREIGN KEY (address) REFERENCES addresses (address);
      │ ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
    2 │ 
  
  i Add the constraint as NOT VALID in one transaction, then run VALIDATE CONSTRAINT in a separate transaction.
  

```

### Valid

```sql
ALTER TABLE distributors ADD CONSTRAINT distfk FOREIGN KEY (address) REFERENCES addresses (address) NOT VALID;
```

## How to configure
```json

{
  "linter": {
    "rules": {
      "safety": {
        "constraintMissingNotValid": "error"
      }
    }
  }
}

```
