# Configuration

This guide will help you to understand how to configure the Postgres Language Server. It explains the structure of the configuration file and how the configuration is resolved.

The Postgres Language Server allows you to customize its behavior using CLI options or a configuration file named `postgres-language-server.jsonc`. We recommend that you create a configuration file for each project. This ensures that each team member has the same configuration in the CLI and in any editor that allows Biome integration. Many of the options available in a configuration file are also available in the CLI.

## Configuration file structure

A configuration file is usually placed in your project’s root folder. It is organized around the tools that are provided. All tools are enabled by default, but some require additional setup like a database connection or the `plpgsql_check` extension.

```json
{
  "$schema": "https://pg-language-server.com/latest/schema.json",
  "linter": {
    "enabled": true,
    "rules": {
      "recommended": true
    }
  },
  "typecheck": {
    "enabled": true
  }
  "plpgsqlCheck": {
    "enabled" : true
  }
}
```

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

The configuration file can be used to refine which files are processed. You can explicitly list the files to be processed using the `files.includes` field. `files.includes` accepts glob patterns such as sql/**/*.sql. Negated patterns starting with `!` can be used to exclude files.

Paths and globs inside the configuration file are resolved relative to the folder the configuration file is in. An exception to this is when a configuration file is extended by another.

#### Include files via configuration
Let’s take the following configuration, where we want to include only SQL files (`.sql`) that are inside the `sql/` folder:

```json
{
  "files": {
    "includes": ["sql/**/*.sql"]
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
You can ignore files ignored by your [VCS](/guides/vcs_integration.md).

