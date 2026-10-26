# banDuplicateColumnAssignments

**Group** [`correctness`](../rules.md#correctness) · **Recommended** · **Unsafe fix** · **Since** `Unreleased`  
**Diagnostic** `lint/banDuplicateColumnAssignments` · **Postgres error** `42601`, `42701`

**Sources**: inspired by [`squawk/ban-duplicate-column-assignments`](https://squawkhq.com/docs/ban-duplicate-column-assignments)

## Description
A column is assigned or declared more than once.

Postgres rejects statements that assign the same column twice in a SET clause, like
`update t set a = 1, a = 2`, with `42601 syntax_error`. It rejects column lists that name
the same column twice, like `insert into t (a, a)` or `copy t (a, a)`, tables, views and
composite types that declare the same column twice, and `ALTER TABLE` commands that add
the same column twice with `42701 duplicate_column`. This includes view columns that get
the same name from their expressions, like `select a.id, b.id`.

Assigning different elements or fields of the same column, like `set a[1] = 1, a[2] = 2`,
is fine.

## Examples

### Invalid

```sql
update users set name = 'a', name = 'b';
```

```sh
code-block.sql:1:30 lint/banDuplicateColumnAssignments ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

  × Multiple assignments to column name.
  
  > 1 │ update users set name = 'a', name = 'b';
      │                              ^^^^
    2 │ 
  
  i Postgres rejects statements that assign a column more than once.
  
  i Remove all but one of the assignments.
  

```

```sql
create table users (id bigint, id text);
```

```sh
code-block.sql:1:32 lint/banDuplicateColumnAssignments ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

  × Column id is specified more than once.
  
  > 1 │ create table users (id bigint, id text);
      │                                ^^
    2 │ 
  
  i Postgres rejects relations with duplicate column names.
  
  i Give each column a distinct name.
  

```

```sql
create view user_ids as select 1 as id, 2 as id;
```

```sh
code-block.sql:1:41 lint/banDuplicateColumnAssignments ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

  × Column id is specified more than once.
  
  > 1 │ create view user_ids as select 1 as id, 2 as id;
      │                                         ^^^^^^^
    2 │ 
  
  i Postgres rejects relations with duplicate column names.
  
  i Give each column a distinct name.
  

```

### Valid

```sql
update users set name = 'b';
```

## How to configure
```json

{
  "linter": {
    "rules": {
      "banDuplicateColumnAssignments": "error"
    }
  }
}

```
## How to suppress

Suppress this diagnostic with a comment:

```sql
-- pgls-ignore banDuplicateColumnAssignments
```
