# banCreateDomainWithConstraint

**Group** [`safety`](../rules.md#safety) · **Migrations only** · **Since** `Unreleased`  
**Diagnostic** `lint/banCreateDomainWithConstraint`

**Sources**: inspired by [`squawk/ban-create-domain-with-constraint`](https://squawkhq.com/docs/ban-create-domain-with-constraint)

## Description
Domains with constraints have poor support for online migrations.

Any later change to the constraint of a domain revalidates every column that uses the
domain, and coercing a column to a domain with constraints rewrites the table. Postgres
can't reason about arbitrary domain constraints the way it can about simple column types.

Put the constraint on the columns directly instead.

## Examples

### Invalid

```sql
create domain positive_int as int check (value > 0);
```

```sh
code-block.sql:1:1 lint/banCreateDomainWithConstraint ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

  ! Domains with constraints have poor support for online migrations.
  
  > 1 │ create domain positive_int as int check (value > 0);
      │ ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
    2 │ 
  
  i Changing the constraint later revalidates every column that uses the domain.
  
  i Use table and column constraints instead.
  

```

### Valid

```sql
create domain name_text as text collate "C";
```

## How to configure
```json

{
  "linter": {
    "rules": {
      "banCreateDomainWithConstraint": "error"
    }
  }
}

```
## How to suppress

Suppress this diagnostic with a comment:

```sql
-- pgls-ignore banCreateDomainWithConstraint
```
