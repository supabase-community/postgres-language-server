# functionArgumentMismatch
**Diagnostic Category: `lint/functionArgumentMismatch`**

**Group: `typecheck`**

**Since**: `vnext`

> [!NOTE]
> This rule is recommended. A diagnostic error will appear when linting your code.

## Description
A function name and argument count exist, but its argument types do not select exactly
one overload. The rule needs a database connection to load the function and type catalog.

Postgres reports SQLSTATE `42883` when no function matches and `42725` when a function
call is ambiguous.

## Examples

### Invalid

```sql
create function example_fn(value integer) returns integer language sql as 'select value';
select example_fn(timestamp '2020-01-01');
```

```sh
```

### Valid

```sql
create function example_fn(value integer) returns integer language sql as 'select value';
select example_fn(1);
```

## How to configure
```json

{
  "linter": {
    "rules": {
      "functionArgumentMismatch": "error"
    }
  }
}

```
## How to suppress

Suppress this diagnostic with a comment:

```sql
-- pgls-ignore functionArgumentMismatch
```
