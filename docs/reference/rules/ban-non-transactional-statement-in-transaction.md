# banNonTransactionalStatementInTransaction

**Group** [`correctness`](../rules.md#correctness) · **Recommended** · **Since** `Unreleased`  
**Diagnostic** `lint/banNonTransactionalStatementInTransaction`

## Description
Statements that Postgres can't run inside a transaction block.

`VACUUM`, `CREATE DATABASE`, `DROP DATABASE`, `ALTER DATABASE ... SET TABLESPACE`,
`CREATE TABLESPACE`, `DROP TABLESPACE`, `ALTER SYSTEM`, `CLUSTER` without a table or on a
partitioned table, `REINDEX SCHEMA`, `REINDEX SYSTEM`, `REINDEX DATABASE`, `REINDEX TABLE`
on a partitioned table, `DISCARD ALL`, `COMMIT PREPARED`, `ROLLBACK PREPARED`,
`CREATE SUBSCRIPTION` that creates a replication slot, and
`ALTER SUBSCRIPTION ... REFRESH PUBLICATION` fail in a transaction block and in the body
of a function. The statements with `CONCURRENTLY` are covered by
`banConcurrentIndexCreationInTransaction`.

The rule only reports statements that surely run in a transaction block: between `BEGIN`
and `COMMIT`, in a function body, or in a migration section that runs in a transaction
(see `migrations.tool`). Without a migration tool, we don't know whether the
migration tool runs the file in a transaction, so the other statements aren't reported.

## Examples

### Invalid

```sql
begin;
vacuum users;
commit;
```

```sh
code-block.sql:2:1 lint/banNonTransactionalStatementInTransaction ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

  × VACUUM cannot run inside a transaction block.
  
    1 │ begin;
  > 2 │ vacuum users;
      │ ^^^^^^^^^^^^^
    3 │ commit;
    4 │ 
  
  i Run VACUUM outside of a transaction, e.g. in a migration of its own that runs without a transaction.
  

```

### Valid

```sql
vacuum users;
```

## How to configure
```json

{
  "linter": {
    "rules": {
      "banNonTransactionalStatementInTransaction": "error"
    }
  }
}

```
## How to suppress

Suppress this diagnostic with a comment:

```sql
-- pgls-ignore banNonTransactionalStatementInTransaction
```
