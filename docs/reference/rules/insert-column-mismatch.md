# insertColumnMismatch
**Diagnostic Category: `lint/insertColumnMismatch`**

**Group: `typecheck`**

**Since**: `vnext`

> [!NOTE]
> This rule is recommended. A diagnostic error will appear when linting your code.

## Description
An `INSERT` has a different number of target columns than values.

With an explicit column list, every row of `VALUES` (or the `SELECT` list) must have
exactly one value per column. Without one, there can't be more values than the table has
columns.

Postgres raises `42601 syntax_error` for these statements.

## Examples

### Invalid

```sql
create table users (id int8, name text);
insert into users (id, name) values (1);
```

```sh
code-block.sql:2:1 lint/insertColumnMismatch ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

  × INSERT has more target columns than expressions.
  
    1 │ create table users (id int8, name text);
  > 2 │ insert into users (id, name) values (1);
      │ ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
    3 │ 
  
  i Expected 2 values, found 1.
  

```

### Valid

```sql
create table users (id int8, name text);
insert into users (id, name) values (1, 'Ada');
```

## How to configure
```json

{
  "linter": {
    "rules": {
      "insertColumnMismatch": "error"
    }
  }
}

```
## How to suppress

Suppress this diagnostic with a comment:

```sql
-- pgls-ignore insertColumnMismatch
```
