# Configuration

This guide will help you to understand how to configure the Postgres Language Server. It explains the structure of the configuration file and how the configuration is resolved.

The Postgres Language Server allows you to customize its behavior using CLI options or a configuration file named `postgres-language-server.jsonc`. We recommend that you create a configuration file for each project. This ensures that each team member has the same configuration in the CLI and in any editor that allows Biome integration. Many of the options available in a configuration file are also available in the CLI.

## Client configuration overrides

Clients can issue the LSP request `pgls/set_configuration_overrides` with parameters `{ "overrides": { ... } }` to apply runtime configuration, or `{ "overrides": null }` to clear it. The `overrides` value replaces the previous client override layer; it does not merge with the previous request. The layer is sticky across configuration file reloads. `workspace/didChangeConfiguration` also writes this same sticky layer.

Configuration precedence, from lowest to highest, is: built-in defaults, the configuration file on disk, environment configuration (`DATABASE_URL`, `PGHOST`, and related variables) captured when the server starts, then the sticky client override. The client deliberately has higher precedence than the environment: it is an explicit live instruction from the editor for this session, while the environment describes the shell where the server started. Environment configuration continues to override the file. Clearing the overrides restores the file-plus-environment baseline.

## Configuration file structure

A configuration file is usually placed in your project’s root folder. It is organized around the tools that are provided. All tools are enabled by default, but some require additional setup like a database connection or the `plpgsql_check` extension.

```json
{
  "$schema": "https://pg-language-server.com/latest/schema.json",
  "linter": {
    "enabled": true,
    "rules": {
      "recommended": true,
      "banDropColumn": "off",
      "preferBigInt": { "level": "warn", "options": { "checkSmallint": false } }
    },
    "groups": {
      "safety": "warn",
      "style": "off"
    }
  },
  "typecheck": {
    "enabled": true
  },
  "plpgsqlCheck": {
    "enabled": true
  },
  "format": {
    "enabled": true,
    "keywordCase": "lower"
  }
}
```

### Linter rules

Rule names are flat and unique across all groups. Configure each rule directly under `linter.rules`:

```jsonc
"linter": {
  "rules": {
    "banDropColumn": "off",                // simple level: "off" | "info" | "warn" | "error"
    "preferBigInt": {                        // or with options
      "level": "warn",
      "options": { "checkInt": true, "checkSmallint": false }
    }
  }
}
```

The `preferBigInt` rule accepts the options `checkInt` (default `true`) and `checkSmallint` (default `true`).

### Linter groups

Groups let you set a level for all rules in the group at once:

```jsonc
"linter": {
  "groups": {
    "style": "off",
    "destructive": "error"
  }
}
```

Available groups: `correctness`, `safety`, `destructive`, `style`, `security`, `typecheck`, `nursery`.

### Presets

`linter.rules.recommended` (default `true`) enables all rules marked as recommended. `linter.rules.all` enables every rule. Nursery rules are never enabled by presets.

### Typecheck group

`typecheck.enabled` switches the `typecheck` group on or off. Rules in this group require a database connection.

### Precedence

From highest to lowest:

1. `linter.rules.<rule>` — explicit rule setting
2. `linter.rules.safety.<rule>` — deprecated per-rule setting (see below)
3. `linter.groups.<group>` — group level
4. `linter.rules.recommended` / `linter.rules.all` preset (nursery never enabled)

### Deprecated `linter.rules.safety` form

The former nested form `linter.rules.safety.<rule>` is still accepted but prints a deprecation warning. Use `linter.rules.<rule>` instead.

Removed rules:

| Old rule | Replacement |
| --- | --- |
| `preferBigintOverInt` | `preferBigInt` with `{ "checkInt": true }` |
| `preferBigintOverSmallint` | `preferBigInt` with `{ "checkSmallint": true }` |
| `concurrentRefreshMatviewLock` | Dropped; `requireConcurrentRefreshMatview` covers it |

## Configuring a database connection

Some tools that the Postgres Language Server provides are implemented as mere interfaces on top of functionality that is provided by the database itself. This ensures correctness, but requires an active connection to a Postgres database. We strongly recommend to only connect to a local development database.

```json
{
  "$schema": "https://pg-language-server.com/latest/schema.json",
  "db": {
    "host": "127.0.0.1",
    "port": 5432,
    "username": "postgres",
    "password": "postgres",
    "database": "postgres",
    "connTimeoutSecs": 10,
    "allowStatementExecutionsAgainst": ["127.0.0.1/*", "localhost/*"]
  }
}
```

When you need to pass additional Postgres settings (e.g. `sslmode`, `options`,
`application_name`) you can provide a connection string instead of the
individual fields. The URI takes precedence over any other connection fields.

```json
{
  "db": {
    "connectionString": "postgres://postgres:postgres@localhost:5432/postgres?sslmode=disable",
    "allowStatementExecutionsAgainst": ["localhost/*"]
  }
}
```


## Specifying files to process

You can control the files/folders to process using different strategies, either CLI, configuration and VCS.

### Include files via CLI
The first way to control which files and folders are processed is to list them in the CLI. In the following command, we only check `file1.sql` and all the files in the `src` folder, because folders are recursively traversed.

```shell
postgres-language-server check file1.js src/
```

### Control files via configuration

The configuration file can be used to refine which files are processed. You can explicitly list the files to be processed using the `files.include` field. `files.include` accepts glob patterns such as sql/**/*.sql. Negated patterns starting with `!` can be used to exclude files.

Paths and globs inside the configuration file are resolved relative to the folder the configuration file is in. An exception to this is when a configuration file is extended by another.

#### Include files via configuration
Let’s take the following configuration, where we want to include only SQL files (`.sql`) that are inside the `sql/` folder:

```json
{
  "files": {
    "include": ["sql/**/*.sql"]
  }
}
```

#### Exclude files via configuration
If you want to exclude files and folders from being processed, you can use the `files.ignore` .

In the following example, we include all files, except those in any test/ folder:

```json
{
  "files": {
    "ignore": [
      "**/test",
    ]
  }
}
```

#### Control files via VCS
You can ignore files ignored by your [VCS](guides/vcs_integration.md).

