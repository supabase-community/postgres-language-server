# invalidCast
**Diagnostic Category: `lint/invalidCast`**

**Group: `typecheck`**

**Since**: `vnext`

> [!NOTE]
> This rule is recommended. A diagnostic error will appear when linting your code.

## Description
An explicit cast is not permitted between the source and target types. The rule needs
a database connection to load the type and cast catalog.

Postgres reports SQLSTATE `42846` (`cannot_coerce`).

## Examples

### Invalid

```sql
select timestamp '2020-01-01'::integer;
```

```sh
```

### Valid

```sql
select 1::text;
```

## How to configure
```json

{
  "linter": {
    "rules": {
      "invalidCast": "error"
    }
  }
}

```
## How to suppress

Suppress this diagnostic with a comment:

```sql
-- pgls-ignore invalidCast
```
