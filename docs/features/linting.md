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

## Autofix

Some rules can fix the issues they report. The fixes come in two kinds:

- **Safe** fixes only do what the rule asks for, e.g. add `CONCURRENTLY` to `CREATE INDEX`. They are left out where Postgres would reject them, e.g. inside a transaction block.
- **Unsafe** fixes may change what a statement does, e.g. swap the type of a column or add `NOT VALID` to a constraint. Review them before applying them.

In the editor, the fixes are offered as quick fixes on the diagnostic, next to a quick fix that suppresses the rule for the statement with a `-- pgls-ignore <rule>` comment.

On the command line, `check --fix` applies the safe fixes and writes the files. Add `--unsafe` to also apply the unsafe fixes:

```sh
postgres-language-server check --fix migrations/
postgres-language-server check --fix --unsafe migrations/
```

The documentation of each rule says whether it has a fix and whether the fix is safe.
