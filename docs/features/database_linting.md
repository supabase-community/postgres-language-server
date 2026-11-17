# Database Linting

The database linter analyzes your live Postgres database schema to detect performance issues, security vulnerabilities, and configuration problems. Unlike the [file-based linter](./linting.md) which checks SQL migration files, the database linter connects directly to your database and inspects the actual schema state.

All database linting rules are powered by existing tools: [Splinter](https://github.com/supabase/splinter) and, if you turn it on, [pglinter](#pglinter).

## Rules

See the [Database Linter Rules Reference](../reference/database_rules.md) for the complete list of available rules and their descriptions.

## Configuration

Configure database linting behavior in your `postgres-language-server.jsonc`:

```json
{
  "splinter": {
    // Enable/disable the database linter entirely
    "enabled": true,
    "rules": {
      // Configure rule groups
      "performance": {
        // Individual rule configuration
        "noPrimaryKey": "warn",
        "unusedIndex": "info"
      },
      "security": {
        "rlsDisabledInPublic": "error",
        "authUsersExposed": "error"
      }
    }
  }
}
```

## Ignoring Database Objects

You can ignore specific database objects using glob patterns. Patterns use Unix-style globs where `*` matches any sequence of characters. Patterns should be in the format `schema.object_name`.

### Global Ignore

To ignore objects across all rules, use the top-level `ignore` field:

```json
{
  "splinter": {
    "ignore": [
      "audit.*",
      "temp_*"
    ],
    "rules": {
      // ...
    }
  }
}
```

This is useful for excluding entire schemas (like audit logs or temporary tables) from all database linting.

### Per-Rule Ignore

To ignore objects for a specific rule only, use the rule-level `ignore` option:

```json
{
  "splinter": {
    "rules": {
      "performance": {
        "noPrimaryKey": {
          "level": "warn",
          "options": {
            "ignore": [
              "public.temp_*",
              "staging.*"
            ]
          }
        }
      }
    }
  }
}
```

### Pattern Examples

| Pattern | Matches |
|---------|---------|
| `public.my_table` | Specific table in public schema |
| `audit.*` | All objects in the audit schema |
| `*.temp_*` | Objects with temp_ prefix in any schema |
| `public.log_*` | Tables starting with log_ in public schema |

## pglinter

[pglinter](https://github.com/pmpetit/pglinter) is a Postgres extension that checks a database for common design problems, such as tables without a primary key, foreign keys without an index, or objects with uppercase names. With `pglinter.enabled`, `dblint` runs its rules next to Splinter's and reports their violations:

```json
{
  "pglinter": {
    "enabled": true,
    "rules": {
      "base": {
        "howManyTableWithoutPrimaryKey": "error",
        "howManyTablesNeverSelected": "off"
      }
    }
  }
}
```

The rules are in the groups `base`, `schema` and `cluster`; see [pglinter's rules](https://github.com/pmpetit/pglinter#implemented-rules). pglinter is off by default.

pglinter runs its rules inside the database, so the extension must be installed there (see [pglinter's installation guide](https://github.com/pmpetit/pglinter#installation)), and created in the database you lint:

```sql
CREATE EXTENSION pglinter;
```

- Without the extension, `dblint` reports `pglinter/extensionNotInstalled`.
- A rule that the configuration enables but the extension disables, with `pglinter.disable_rule`, is reported as `pglinter/ruleDisabledInExtension`.

## Supabase-Specific Rules

Some rules are specifically designed for Supabase projects and will be automatically skipped if Supabase-specific database roles are not detected. These rules check for issues related to:

- Auth schema exposure
- RLS policy configuration
- API schema security
- Supabase-specific extensions

## CLI Usage

The database linter can be run via the CLI:

```bash
# Run database linting
postgres-language-server dblint
```

Which rules run is set in the configuration file.

See the [CLI Reference](../reference/cli.md) for more options.

## Database Connection

The database linter requires a database connection to analyze the schema. Configure your connection in `postgres-language-server.jsonc`:

```json
{
  "db": {
    "host": "127.0.0.1",
    "port": 5432,
    "database": "postgres",
    "username": "postgres",
    "password": "postgres"
  }
}
```

See the [database connection guide](../guides/configure_database.md) for more details.
