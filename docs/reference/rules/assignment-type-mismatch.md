# assignmentTypeMismatch
**Diagnostic Category: `lint/assignmentTypeMismatch`**

**Group: `typecheck`**

**Since**: `vnext`

> [!NOTE]
> This rule is recommended. A diagnostic error will appear when linting your code.

## Description
An expression assigned to a column cannot be coerced to that column's type. The rule
needs a database connection to load the table and type catalog.

Postgres reports SQLSTATE `42804` (`datatype_mismatch`).

## Examples

### Invalid

```sql
create table typecheck_assignment (qty integer);
insert into typecheck_assignment values (timestamp '2020-01-01');
```

```sh
```

### Valid

```sql
create table typecheck_assignment (qty integer);
insert into typecheck_assignment values (1);
```

## How to configure
```json

{
  "linter": {
    "rules": {
      "assignmentTypeMismatch": "error"
    }
  }
}

```
## How to suppress

Suppress this diagnostic with a comment:

```sql
-- pgls-ignore assignmentTypeMismatch
```
