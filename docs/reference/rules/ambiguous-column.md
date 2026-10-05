# ambiguousColumn

**Group** [`typecheck`](../rules.md#typecheck) · **Recommended** · **Needs a database connection** · **Since** `0.27.0`  
**Diagnostic** `lint/ambiguousColumn` · **Postgres error** `42702`

## Description
An unqualified column name matches columns of more than one relation in scope.

Qualify the column with the name or alias of the relation it belongs to.

Postgres raises `42702 ambiguous_column` for these statements.

## Examples

### Invalid

```sql
create table users (id int8, name text);
create table posts (id int8, user_id int8);
select id from users join posts on posts.user_id = users.id;
```

```sh
code-block.sql:3:8 lint/ambiguousColumn ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

  × Column reference id is ambiguous.
  
    1 │ create table users (id int8, name text);
    2 │ create table posts (id int8, user_id int8);
  > 3 │ select id from users join posts on posts.user_id = users.id;
      │        ^^
    4 │ 
  
  i It matches columns of users, posts.
  

```

### Valid

```sql
create table users (id int8, name text);
create table posts (id int8, user_id int8);
select users.id from users join posts on posts.user_id = users.id;
```

## How to configure
```json

{
  "linter": {
    "rules": {
      "ambiguousColumn": "error"
    }
  }
}

```
## How to suppress

Suppress this diagnostic with a comment:

```sql
-- pgls-ignore ambiguousColumn
```
