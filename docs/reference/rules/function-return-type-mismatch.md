# functionReturnTypeMismatch
**Diagnostic Category: `lint/functionReturnTypeMismatch`**

**Group: `typecheck`**

**Since**: `vnext`

> [!NOTE]
> This rule is recommended. A diagnostic error will appear when linting your code.

## Description
The final statement of a SQL function doesn't return what the function is declared to
return.

When it creates a `LANGUAGE sql` function, Postgres checks the final statement of the
body against the declared result. A scalar result needs exactly one column. A composite
result, like a table's row type or `RETURNS TABLE`, needs one column per attribute, or a
single column holding the whole row. Unless the function returns `void`, the final
statement must be a `SELECT`, or an `INSERT`, `UPDATE`, `DELETE`, or `MERGE` with
`RETURNING`.

Postgres raises `42P13 invalid_function_definition` for these functions, unless
`check_function_bodies` is off. The rule only compares the number of columns, not their
types.

## Examples

### Invalid

```sql
create table users (id int8, name text, email text);
create function find_user(user_id int8) returns users language sql as $$
    select id, name from users where id = user_id;
$$;
```

```sh
code-block.sql:2:49 lint/functionReturnTypeMismatch ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

  × Return type mismatch in function declared to return users.
  
    1 │ create table users (id int8, name text, email text);
  > 2 │ create function find_user(user_id int8) returns users language sql as $$
      │                                                 ^^^^^
    3 │     select id, name from users where id = user_id;
    4 │ $$;
  
  i Final statement returns too few columns: expected 3, found 2.
  

```

### Valid

```sql
create table users (id int8, name text, email text);
create function find_user(user_id int8) returns users language sql as $$
    select * from users where id = user_id;
$$;
```

## How to configure
```json

{
  "linter": {
    "rules": {
      "functionReturnTypeMismatch": "error"
    }
  }
}

```
## How to suppress

Suppress this diagnostic with a comment:

```sql
-- pgls-ignore functionReturnTypeMismatch
```
