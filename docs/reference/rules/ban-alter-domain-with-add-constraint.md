# banAlterDomainWithAddConstraint

**Group** [`safety`](../rules.md#safety) · **Migrations only** · **Since** `Unreleased`  
**Diagnostic** `lint/banAlterDomainWithAddConstraint`

**Sources**: inspired by [`squawk/ban-alter-domain-with-add-constraint`](https://squawkhq.com/docs/ban-alter-domain-with-add-constraint)

## Description
Adding a constraint to a domain has poor support for online migrations.

`ALTER DOMAIN ... ADD CONSTRAINT` validates every column that uses the domain, across all
tables, while holding locks on them. Postgres can't reason about arbitrary domain
constraints the way it can about simple column types.

Put the constraint on the columns directly instead.

## Examples

### Invalid

```sql
alter domain positive_int add constraint positive_int_check check (value > 0);
```

```sh
code-block.sql:1:1 lint/banAlterDomainWithAddConstraint ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

  ! Adding a constraint to a domain has poor support for online migrations.
  
  > 1 │ alter domain positive_int add constraint positive_int_check check (value > 0);
      │ ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
    2 │ 
  
  i Postgres validates every column that uses the domain, across all tables.
  
  i Use table and column constraints instead.
  

```

## How to configure
```json

{
  "linter": {
    "rules": {
      "banAlterDomainWithAddConstraint": "error"
    }
  }
}

```
## How to suppress

Suppress this diagnostic with a comment:

```sql
-- pgls-ignore banAlterDomainWithAddConstraint
```
