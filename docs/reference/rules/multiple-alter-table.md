# multipleAlterTable
**Diagnostic Category: `lint/safety/multipleAlterTable`**

**Since**: `vnext`

> [!NOTE]
> This rule is recommended. A diagnostic error will appear when linting your code.

**Sources**: 
- Inspired from: <a href="https://kaveland.no/eugene/hints/W12/index.html" target="_blank"><code>eugene/W12</code></a>

## Description
Multiple ALTER TABLE statements on the same table should be combined into a single statement.

When you run multiple ALTER TABLE statements on the same table, PostgreSQL must scan and potentially
rewrite the table multiple times. Each ALTER TABLE command requires acquiring locks and performing
table operations that can be expensive, especially on large tables.

Combining multiple ALTER TABLE operations into a single statement with comma-separated actions
allows PostgreSQL to scan and modify the table only once, improving performance and reducing
the time locks are held.

## Examples

### Invalid

```sql
ALTER TABLE authors ALTER COLUMN name SET NOT NULL;
ALTER TABLE authors ALTER COLUMN email SET NOT NULL;
```

```sh
```

### Valid

```sql
ALTER TABLE authors
  ALTER COLUMN name SET NOT NULL,
  ALTER COLUMN email SET NOT NULL;
```

## How to configure
```json

{
  "linter": {
    "rules": {
      "safety": {
        "multipleAlterTable": "error"
      }
    }
  }
}

```
