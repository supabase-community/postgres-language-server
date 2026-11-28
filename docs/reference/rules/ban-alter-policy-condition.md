# banAlterPolicyCondition

**Group** [`destructive`](../rules.md#destructive) · **Migrations only** · **Since** `Unreleased`  
**Diagnostic** `lint/banAlterPolicyCondition`

**Sources**: inspired by [`squawk/ban-alter-policy-condition`](https://squawkhq.com/docs/ban-alter-policy-condition)

## Description
Changing the condition of a policy may change access for existing clients.

The `USING` and `WITH CHECK` expressions of a row level security policy decide which rows
clients can read and write. Changing them may hide rows from existing clients, reject
their writes, or give them access to rows they couldn't see before.

Each changed clause is reported. Changing the roles of a policy is reported by
`banAlterPolicyRoles`.

## Examples

### Invalid

```sql
alter policy tenant_isolation on accounts using (tenant_id = current_setting('app.tenant_id')::int);
```

```sh
code-block.sql:1:1 lint/banAlterPolicyCondition ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

  ! Changing the USING condition of policy tenant_isolation may change access for existing clients.
  
  > 1 │ alter policy tenant_isolation on accounts using (tenant_id = current_setting('app.tenant_id')::int);
      │ ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
    2 │ 
  
  i Existing clients may no longer see or write rows they could before, or see and write rows they couldn't.
  
  i Check which rows existing clients need before changing the condition.
  

```

### Valid

```sql
alter policy tenant_isolation on accounts rename to accounts_tenant_isolation;
```

## How to configure
```json

{
  "linter": {
    "rules": {
      "banAlterPolicyCondition": "error"
    }
  }
}

```
## How to suppress

Suppress this diagnostic with a comment:

```sql
-- pgls-ignore banAlterPolicyCondition
```
