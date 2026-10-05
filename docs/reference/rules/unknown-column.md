# unknownColumn

**Group** [`typecheck`](../rules.md#typecheck) · **Recommended** · **Needs a database connection** · **Since** `0.27.0`  
**Diagnostic** `lint/unknownColumn` · **Postgres error** `42703`

## Description
A column does not exist on the relation or record it is taken from.

Columns are resolved against the relations in scope: tables, views, subqueries, CTEs,
functions in `FROM`, and the parameters of SQL functions.

Postgres raises `42703 undefined_column` for these statements.

## Examples

### Invalid

```sql
create table users (id int8, name text);
select email from users;
```

```sh
code-block.sql:2:8 lint/unknownColumn ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

  × Column email does not exist.
  
    1 │ create table users (id int8, name text);
  > 2 │ select email from users;
      │        ^^^^^
    3 │ 
  

```

### Valid

```sql
create table users (id int8, name text);
select name from users;
```

## How to configure
```json

{
  "linter": {
    "rules": {
      "unknownColumn": "error"
    }
  }
}

```
## How to suppress

Suppress this diagnostic with a comment:

```sql
-- pgls-ignore unknownColumn
```
