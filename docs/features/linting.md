# Linting

The language server provides static analysis through linting rules that detect potential issues in your SQL code. The linter analyses SQL statements for safety issues, best practices violations, and problems that could break existing applications.

## Rules

Rules are organized into categories like Safety, Performance, and Style. Each rule can be configured individually or disabled entirely.

See the [Rules Reference](../reference/rules.md) for the complete list of available rules and their descriptions.

## Configuration

Configure linting behavior in your `postgres-language-server.jsonc`:

```json
{
  "linter": {
    // Enable/disable the linter entirely
    "enabled": true,
    "rules": {
      // Configure rule groups
      "safety": {
        // Individual rule configuration
        "banDropColumn": "error",    // error, warn, info, hint, off
        "banDropTable": "warn",
        "addingRequiredField": "off"
      }
    }
  }
}
```

## Suppressing Diagnostics

You can suppress specific diagnostics using comments:

```sql
-- pgt-ignore-next-line safety/banDropColumn: Intentionally dropping deprecated column
ALTER TABLE users DROP COLUMN deprecated_field;

-- pgt-ignore safety/banDropTable: Cleanup during migration
DROP TABLE temp_migration_table;
```

For more details on suppressions check out [our guide]('../guides/suppressions.md').

## Schema-Aware Analysis

Some rules require a database connection to perform schema-aware analysis. If no connection is configured, they are skipped.

## CLI Usage

The linter can also be used via the CLI for CI integration:

```bash
# Lint specific files
postgres-language-server check migrations/

# With specific rules
postgres-language-server check migrations/ --only safety/banDropColumn

# Skip certain rules
postgres-language-server check migrations/ --skip safety/banDropTable
```

See the [CLI Reference](../reference/cli.md) for more options, and check the guide on [linting migrations]('../guides/checking_migrations.md').
