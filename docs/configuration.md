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

Available groups: `correctness`, `safety`, `destructive`, `style`, `typecheck`, `nursery`.

### Presets

`linter.rules.recommended` (default `true`) enables all rules marked as recommended. `linter.rules.all` enables every rule. Nursery rules are never enabled by presets.

### Typecheck group

The rules in the `typecheck` group need a database connection. Two settings affect them:

- `typecheck.enabled: false` turns typechecking off entirely: the `typecheck` rules and the [legacy `EXPLAIN`-based check](features/type_checking.md#legacy-explain-based-checking).
- `linter.groups.typecheck` sets the level of the `typecheck` rules, like any other group. Setting it to `"off"` turns the rules off, but leaves the legacy `EXPLAIN`-based check on.

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

## Where the configuration is loaded from

The configuration is resolved in this order, and the first match wins:

1. The path passed with `--config-path` or `PGLS_CONFIG_PATH`, or the configuration path set in your editor. Not finding a configuration file there is an error.
2. A `postgres-language-server.jsonc` (or the deprecated `postgrestools.jsonc`) in the project directory or any of its parent directories. The project directory is the working directory of the CLI, or the workspace folder in an editor.
3. The user configuration, a `postgres-language-server.jsonc` in your configuration directory:

   | Platform | Location |
   | --- | --- |
   | Linux | `$XDG_CONFIG_HOME/postgres-language-server/`, or `~/.config/postgres-language-server/` if `XDG_CONFIG_HOME` is not set |
   | macOS | `$XDG_CONFIG_HOME/postgres-language-server/` or `~/.config/postgres-language-server/`, then `~/Library/Application Support/postgres-language-server/` |
   | Windows | `%APPDATA%\postgres-language-server\` |

4. The built-in defaults.

The user configuration is only a fallback for projects without a configuration file. It is never merged into a project configuration. To share settings between projects and the user configuration, extend it explicitly from the project configuration (see below).

The user configuration applies to the project as if it were placed in the project directory: globs in `files` and the VCS settings are resolved against the project. Relative paths in its `extends` are resolved against its own directory.

## Sharing configuration with `extends`

`extends` lists configuration files to inherit from. They are applied from left to right, and the settings of the file that declares `extends` take precedence over all of them.

```jsonc
{
  "extends": [
    // relative to this file
    "./shared/base.jsonc",
    // relative to your home directory
    "~/.config/postgres-language-server/postgres-language-server.jsonc",
    // a package in node_modules
    "@acme/pgls-config"
  ]
}
```

- Entries starting with `~/` (or `~\` on Windows) are resolved against your home directory. This works on every platform, so a project can extend your user configuration with the same path on macOS and Linux.
- Entries starting with `.` or ending in `.jsonc` are resolved relative to the directory of the configuration file that declares them. Absolute paths are used as they are.
- Any other entry is resolved as a package, like `require` would in Node.js.

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

## Nested configurations / monorepos

A repository can hold more than one configuration file. Each file uses the nearest `postgres-language-server.jsonc` in its own folder or above it, so different folders can point at different databases or use different linter and formatter settings:

```
my-repo/
├── postgres-language-server.jsonc      # applies to everything else
└── src/
    ├── db1/
    │   ├── postgres-language-server.jsonc  # applies to src/db1/**
    │   └── migrations/0001_init.sql
    └── db2/
        ├── postgres-language-server.jsonc  # applies to src/db2/**
        └── schema.sql
```

The language server finds the configuration files below each workspace folder when it starts, and reloads them when one is created, changed, or deleted, or when a workspace folder is added or removed. Each workspace folder is a project of its own, so a file belongs to the deepest workspace folder that contains it. The `check` and `format` commands use the configuration files below the working directory: the ones in the folders between the working directory and each path they check, and the ones below the folders they check.

A nested configuration file inherits nothing from the configuration files above it: settings it does not set have their defaults. To share settings, extend the parent configuration explicitly; the nested file's own settings take precedence:

```json
{
  "extends": ["../../postgres-language-server.jsonc"],
  "db": {
    "database": "db1"
  }
}
```

Paths in a nested configuration file, such as `migrations.migrationsDir`, are relative to its folder.

The [user configuration](#where-the-configuration-is-loaded-from) is a fallback for the workspace root only. When there is no configuration file at or above the root, it applies to the files that no nested configuration file covers. It is never merged into a nested configuration file. To use it there, extend it explicitly, for example with `"extends": ["~/.config/postgres-language-server/postgres-language-server.jsonc"]`.

The database environment variables (`DATABASE_URL`, `PGHOST`, `PGPORT`, `PGUSER`, `PGPASSWORD`, `PGDATABASE`) override the connection of the root configuration, as without nested configurations. In a nested configuration file, they only fill in the connection options that the file doesn't set itself, so that each folder keeps its own database even when, for example, `DATABASE_URL` is set in your shell or CI:

- An environment variable is used for an option (`host`, `port`, `username`, `password`, `database`) that the nested file doesn't set itself. An option that the file only gets through `extends` is overridden like in the root configuration.
- `DATABASE_URL` is only used when the nested file sets none of `connectionString`, `host`, `port`, `username`, `password`, and `database` itself, because a connection string takes precedence over the other options.
- The database options passed to the CLI, such as `--connection-string`, and the editor's client overrides apply to every configuration file.

Each workspace folder uses its own settings. When the configuration file of a workspace folder can't be loaded, the error is reported and the folder keeps the settings it had, or the defaults if it is new, until the file is fixed; its nested configuration files are not loaded meanwhile. Files outside every workspace folder use the default settings. A nested configuration file that can't be loaded gives its folder the default settings, and `check` reports it as an error but still checks the other files.

Things to keep in mind:

- Configuration files in hidden folders, `node_modules`, `target`, folders ignored by `.gitignore`, folders that the configuration above them ignores through `files.ignore` or `files.include`, and the user configuration directory are not loaded as nested configurations. Ignoring the large folders that hold no SQL files also makes the search faster.
- When the workspace folder (or the working directory of the CLI) is your home directory or a filesystem root, nested configuration files are not looked up, because those folders hold unrelated projects.
- The search stops after 100,000 files and folders, or 12 levels of folders below the workspace folder, and logs a warning. Configuration files beyond that are not used.
- The database connection of a project is only opened when a file of that project needs it. Projects that point at the same database share its connection and schema cache.
- When a configuration file is passed explicitly with `--config-path`, it applies to all files and nested configuration files are not looked up.
- `dblint` uses the configuration of the working directory.

## Sharing settings with `extends`

`extends` lists other configuration files whose settings apply to this one, for example to share the linter setup between several projects of a monorepo:

```json
{
  "$schema": "https://pg-language-server.com/latest/schema.json",
  "extends": ["../postgres-language-server.base.jsonc"],
  "db": {
    "database": "billing"
  }
}
```

Paths that start with `.` or end in `.jsonc` are resolved relative to the configuration file. The files are applied from left to right, so a later file overrides an earlier one, and the settings of the configuration file itself take precedence over all of them.

## Specifying files to process

You can control the files/folders to process using different strategies, either CLI, configuration and VCS.

### Include files via CLI
The first way to control which files and folders are processed is to list them in the CLI. In the following command, we only check `file1.sql` and all the files in the `src` folder, because folders are recursively traversed.

```shell
postgres-language-server check file1.js src/
```

### Control files via configuration

The configuration file can be used to refine which files are processed. You can explicitly list the files to be processed using the `files.include` field. `files.include` accepts glob patterns such as sql/**/*.sql. Negated patterns starting with `!` can be used to exclude files.

Paths and globs inside the configuration file are resolved relative to the folder the configuration file is in. An exception to this is when a configuration file is extended by another. The [user configuration](#where-the-configuration-is-loaded-from) is resolved relative to the project directory.

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

