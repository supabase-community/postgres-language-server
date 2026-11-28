# banAlterPolicyRoles

**Group** [`destructive`](../rules.md#destructive) · **Migrations only** · **Since** `Unreleased`  
**Diagnostic** `lint/banAlterPolicyRoles`

**Sources**: inspired by [`squawk/ban-alter-policy-roles`](https://squawkhq.com/docs/ban-alter-policy-roles)

## Description
Changing the roles of a policy may change access for existing clients.

A row level security policy only applies to the roles in its `TO` clause. Changing them
may take access away from the roles of existing clients, or apply the policy to roles it
didn't apply to before.

Changing the condition of a policy is reported by `banAlterPolicyCondition`.

## Examples

### Invalid

```sql
alter policy tenant_isolation on accounts to authenticated;
```

```sh
code-block.sql:1:1 lint/banAlterPolicyRoles ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

  ! Changing the roles of policy tenant_isolation may change access for existing clients.
  
  > 1 │ alter policy tenant_isolation on accounts to authenticated;
      │ ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
    2 │ 
  
  i The policy no longer applies to the roles it applied to before, or applies to new ones.
  
  i Check which roles existing clients use before changing the roles of the policy.
  

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
      "banAlterPolicyRoles": "error"
    }
  }
}

```
## How to suppress

Suppress this diagnostic with a comment:

```sql
-- pgls-ignore banAlterPolicyRoles
```
