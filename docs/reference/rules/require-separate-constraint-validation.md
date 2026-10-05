# requireSeparateConstraintValidation

**Group** [`safety`](../rules.md#safety) · **Recommended** · **Migrations only** · **Since** `0.25.0`  
**Diagnostic** `lint/requireSeparateConstraintValidation`

**Sources**: inspired by [`pgfence/not-valid-validate-same-tx`](https://github.com/flvmnt/pgfence)

## Description
Validating a constraint in the same transaction it was added as `NOT VALID` defeats the purpose.

Adding a constraint with `NOT VALID` avoids a full table scan and lock during creation.
But if you immediately `VALIDATE CONSTRAINT` in the same transaction, the validation
still holds the lock from the `ADD CONSTRAINT`, blocking reads and writes.

Run `VALIDATE CONSTRAINT` in a separate transaction to get the benefit of `NOT VALID`.

## Examples

### Invalid

Adding a NOT VALID constraint and validating it in the same transaction:

```sql
ALTER TABLE orders ADD CONSTRAINT orders_check CHECK (total > 0) NOT VALID;
ALTER TABLE orders VALIDATE CONSTRAINT orders_check;
```

```sh
code-block.sql:2:1 lint/requireSeparateConstraintValidation ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

  × Constraint orders_check was added as NOT VALID and validated in the same transaction.
  
    1 │ ALTER TABLE orders ADD CONSTRAINT orders_check CHECK (total > 0) NOT VALID;
  > 2 │ ALTER TABLE orders VALIDATE CONSTRAINT orders_check;
      │ ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
    3 │ 
  
  i Run VALIDATE CONSTRAINT in a separate transaction to avoid holding locks during validation.
  

```

### Valid

```sql
select 1;
```

## How to configure
```json

{
  "linter": {
    "rules": {
      "requireSeparateConstraintValidation": "error"
    }
  }
}

```
## How to suppress

Suppress this diagnostic with a comment:

```sql
-- pgls-ignore requireSeparateConstraintValidation
```
