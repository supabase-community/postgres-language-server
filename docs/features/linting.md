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

- **Safe** fixes only do what the rule asks for, e.g. add `CONCURRENTLY` to `REINDEX` (`requireConcurrentReindex`). They are left out where Postgres would reject them. `CONCURRENTLY` can't run in a transaction block or in a function body, so the safe fix only adds it where the statement surely runs outside a transaction: in a migration section without a transaction (`-- migrate:up transaction:false` for dbmate, if the statement is the only one of the section, or `-- +goose NO TRANSACTION` for goose, see [Checking migrations](../guides/checking_migrations.md)). Without section markers, we don't know whether the migration tool runs a file in a transaction. Migration tools usually do, so a file with more than one statement gets no fix, and adding `CONCURRENTLY` to a file with a single statement is an unsafe fix. It is never added on partitioned tables and system catalogs, which Postgres can't process concurrently.
- **Unsafe** fixes may change what a statement does, e.g. swap the type of a column or add `NOT VALID` to a constraint. Review them before applying them.

In the editor, the fixes are offered as quick fixes on the diagnostic, next to a quick fix that suppresses the rule for the statement with a `-- pgls-ignore <rule>` comment.

The `source.fixAll.pgls` code action applies all safe fixes of a file. To run it on save in VS Code:

```json
{
  "editor.codeActionsOnSave": {
    "source.fixAll.pgls": "explicit"
  }
}
```

On the command line, `check --fix` applies the safe fixes and writes the files. Add `--unsafe` to also apply the unsafe fixes:

```sh
postgres-language-server check --fix migrations/
postgres-language-server check --fix --unsafe migrations/
```

The [rules reference](../reference/rules.md) marks the rules with a fix with 🔧, and the page of each rule says whether its fix is safe. Most rules with fixes, like `requireConcurrentIndexCreation`, `preferRobustStmts` or `preferTimestamptz`, aren't in the recommended preset (✅), so enable them first:

```json
{
  "linter": {
    "rules": {
      "requireConcurrentIndexCreation": "warn",
      "requireConcurrentIndexDeletion": "warn"
    }
  }
}
```

Rules marked 🛠 in the reference, like the `CONCURRENTLY` rules, only run on files in `migrations.migrationsDir` if it is set.
