# Linting

The language server provides static analysis through linting rules that detect potential issues in your SQL code. The linter analyses SQL statements for safety issues, best practices violations, and problems that could break existing applications.

## Rules

Every rule has a flat name, like `banDropColumn`, and belongs to a group: `correctness`, `safety`, `destructive`, `style`, `typecheck`, or `nursery`. Rules can be configured individually, and groups in bulk.

See the [Rules Reference](../reference/rules.md) for the complete list of available rules and their descriptions.

## Configuration

Configure linting behavior in your `postgres-language-server.jsonc`:

```json
{
  "linter": {
    // Enable/disable the linter entirely
    "enabled": true,
    "rules": {
      // Individual rule configuration: error, warn, info, hint, off
      "banDropColumn": "error",
      "banDropTable": "warn",
      "addingRequiredField": "off"
    },
    // Configure whole groups
    "groups": {
      "style": "off"
    }
  }
}
```

The former nested form `"rules": { "safety": { ... } }` still works, but prints a deprecation warning. See [Configuration](../configuration.md) for the details.

## Suppressing Diagnostics

You can suppress specific diagnostics using comments:

```sql
-- pgls-ignore banDropColumn: Intentionally dropping deprecated column
ALTER TABLE users DROP COLUMN deprecated_field;

-- pgls-ignore banDropTable: Cleanup during migration
DROP TABLE temp_migration_table;
```

For more details on suppressions check out [our guide](../guides/suppressions.md).

## Schema-Aware Analysis

Some rules require a database connection to perform schema-aware analysis, like the rules of the `typecheck` group described in [Type Checking](type_checking.md). If no connection is configured, they are skipped.

## CLI Usage

The linter can also be used via the CLI for CI integration:

```bash
# Lint specific files
postgres-language-server check migrations/
```

Which rules run is set in the configuration file, see above.

See the [CLI Reference](../reference/cli.md) for more options, and check the guide on [linting migrations](../guides/checking_migrations.md).
