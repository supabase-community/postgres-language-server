# assignmentTypeMismatch

**Group** [`typecheck`](../rules.md#typecheck) · **Recommended** · **Needs a database connection** · **Since** `0.27.0`  
**Diagnostic** `lint/assignmentTypeMismatch` · **Postgres error** `42804`

## Description
An expression assigned to a column cannot be coerced to that column's type.

Postgres raises `42804 datatype_mismatch` for these statements.

## Examples

### Invalid

```sql
create table typecheck_assignment (qty integer);
insert into typecheck_assignment values (timestamp '2020-01-01');
```

```sh
code-block.sql:2:1 lint/assignmentTypeMismatch ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

  × Column "qty" is of type integer but expression is of type timestamp without time zone.
  
    1 │ create table typecheck_assignment (qty integer);
  > 2 │ insert into typecheck_assignment values (timestamp '2020-01-01');
      │ ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
    3 │ 
  
  i You will need to rewrite or cast the expression.
  

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
