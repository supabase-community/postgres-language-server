# missingFromClauseEntry
**Diagnostic Category: `lint/missingFromClauseEntry`**

**Group: `typecheck`**

**Since**: `vnext`

> [!NOTE]
> This rule is recommended. A diagnostic error will appear when linting your code.

## Description
A column is qualified with a name that is not in the `FROM` clause.

The qualifier must be the name or alias of a relation in scope. After aliasing a table, its
original name can't be used anymore.

Postgres raises `42P01 undefined_table` for these statements.

## Examples

### Invalid

```sql
create table users (id int8);
select users.id from users u;
```

```sh
code-block.sql:2:8 lint/missingFromClauseEntry ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

  × Missing FROM-clause entry for users.
  
    1 │ create table users (id int8);
  > 2 │ select users.id from users u;
      │        ^^^^^^^^
    3 │ 
  

```

### Valid

```sql
create table users (id int8);
select u.id from users u;
```

## How to configure
```json

{
  "linter": {
    "rules": {
      "missingFromClauseEntry": "error"
    }
  }
}

```
## How to suppress

Suppress this diagnostic with a comment:

```sql
-- pgls-ignore missingFromClauseEntry
```
