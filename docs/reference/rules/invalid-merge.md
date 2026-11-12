# invalidMerge

**Group** [`typecheck`](../rules.md#typecheck) · **Recommended** · **Needs a database connection** · **Since** `Unreleased`  
**Diagnostic** `lint/invalidMerge` · **Postgres error** `42712`, `42601`

## Description
A `MERGE`, or `merge_action()`, that Postgres rejects.

Postgres rejects a `MERGE` with `WITH RECURSIVE`, a `WHEN` clause after an unconditional
`WHEN` clause of the same kind, which can never run, and a target and source with the same
name. `merge_action()` is only allowed in the `RETURNING` list of a `MERGE`.

Postgres raises `42712 duplicate_alias` for the same name, and `42601 syntax_error` for the
others.

## Examples

### Invalid

```sql
create table users (id int8, name text);
create table imports (id int8, name text);
merge into users using imports on imports.id = users.id
when matched then delete
when matched and imports.name is null then do nothing;
```

```sh
code-block.sql:3:1 lint/invalidMerge ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

  × unreachable WHEN clause specified after unconditional WHEN clause.
  
    1 │ create table users (id int8, name text);
    2 │ create table imports (id int8, name text);
  > 3 │ merge into users using imports on imports.id = users.id
      │ ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
  > 4 │ when matched then delete
  > 5 │ when matched and imports.name is null then do nothing;
      │ ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
    6 │ 
  
  i A WHEN clause without a condition matches every row of its kind first.
  

```

### Valid

```sql
create table users (id int8, name text);
create table imports (id int8, name text);
merge into users using imports on imports.id = users.id
when matched and imports.name is null then do nothing
when matched then delete;
```

## How to configure
```json

{
  "linter": {
    "rules": {
      "invalidMerge": "error"
    }
  }
}

```
## How to suppress

Suppress this diagnostic with a comment:

```sql
-- pgls-ignore invalidMerge
```
