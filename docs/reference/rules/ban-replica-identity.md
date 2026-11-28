# banReplicaIdentity

**Group** [`destructive`](../rules.md#destructive) · **Migrations only** · **Since** `Unreleased`  
**Diagnostic** `lint/banReplicaIdentity`

**Sources**: inspired by [`squawk/ban-replica-identity`](https://squawkhq.com/docs/ban-replica-identity)

## Description
Changing the replica identity of a table or a publication may break logical replication
consumers.

The replica identity decides which old values of a row logical replication sends for
updates and deletes. Dropping a publication, dropping or replacing its objects, or
changing its options stops or changes what its consumers receive.

Adding objects to a publication is not reported.

## Examples

### Invalid

```sql
alter table orders replica identity full;
```

```sh
code-block.sql:1:1 lint/banReplicaIdentity ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

  ! Changing the replica identity of a table may break logical replication consumers.
  
  > 1 │ alter table orders replica identity full;
      │ ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
    2 │ 
  
  i Consumers receive different old values of the rows that are updated or deleted.
  
  i Update the replication consumers before the change.
  

```

```sql
alter publication analytics drop table orders;
```

```sh
code-block.sql:1:1 lint/banReplicaIdentity ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

  ! Changing the objects or options of a publication changes what its consumers receive.
  
  > 1 │ alter publication analytics drop table orders;
      │ ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
    2 │ 
  
  i Consumers may miss changes or receive changes they don't expect.
  
  i Update the replication consumers before the change.
  

```

### Valid

```sql
alter publication analytics add table orders;
```

## How to configure
```json

{
  "linter": {
    "rules": {
      "banReplicaIdentity": "error"
    }
  }
}

```
## How to suppress

Suppress this diagnostic with a comment:

```sql
-- pgls-ignore banReplicaIdentity
```
