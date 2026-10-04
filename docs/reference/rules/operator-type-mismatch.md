# operatorTypeMismatch
**Diagnostic Category: `lint/operatorTypeMismatch`**

**Group: `typecheck`**

**Since**: `vnext`

> [!NOTE]
> This rule is recommended. A diagnostic error will appear when linting your code.

## Description
An operator exists by name but cannot be resolved for the operand types, or has
multiple equally suitable candidates.

The rule needs a database connection to load the operator and type catalog.
Postgres reports SQLSTATE `42883` when no operator matches and `42725` when an
operator is ambiguous.

## Examples

### Invalid

```sql
select 1 + timestamp '2020-01-01';
```

```sh
```

### Valid

```sql
select 1 + 2;
```

## How to configure
```json

{
  "linter": {
    "rules": {
      "operatorTypeMismatch": "error"
    }
  }
}

```
## How to suppress

Suppress this diagnostic with a comment:

```sql
-- pgls-ignore operatorTypeMismatch
```
