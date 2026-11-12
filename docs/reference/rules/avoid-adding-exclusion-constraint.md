# avoidAddingExclusionConstraint

**Group** [`safety`](../rules.md#safety) · **Recommended** · **Migrations only** · **Since** `0.25.0`  
**Diagnostic** `lint/avoidAddingExclusionConstraint`

**Sources**: inspired by [`pgfence/add-constraint-exclude`](https://github.com/flvmnt/pgfence)

## Description
Adding an exclusion constraint acquires an `ACCESS EXCLUSIVE` lock.

Exclusion constraints require a full table scan to validate and block all reads
and writes while held. Unlike other constraints, there is no concurrent alternative.
Use `SET lock_timeout` to limit the impact on concurrent operations.

This also applies to exclusion constraints defined inline in `CREATE TABLE`.

## Examples

### Invalid

```sql
alter table my_table add constraint my_excl exclude using gist (col with &&);
```

```sh
code-block.sql:1:1 lint/avoidAddingExclusionConstraint ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

  ! Adding an exclusion constraint acquires an ACCESS EXCLUSIVE lock.
  
  > 1 │ alter table my_table add constraint my_excl exclude using gist (col with &&);
      │ ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
    2 │ 
  
  i There is no concurrent alternative for exclusion constraints. Use SET lock_timeout to limit the impact on concurrent operations.
  

```

### Valid

```sql
alter table my_table add constraint my_check check (col > 0) not valid;
```

## How to configure
```json

{
  "linter": {
    "rules": {
      "avoidAddingExclusionConstraint": "error"
    }
  }
}

```
## How to suppress

Suppress this diagnostic with a comment:

```sql
-- pgls-ignore avoidAddingExclusionConstraint
```
