# Configure database connection

The language server requires a database connection for schema-dependent features.

## Features requiring database connection

- Type checking
- Autocompletion for tables, columns, functions, and schemas
- Hover information for database objects
- PL/pgSQL analysis via `plpgsql_check` extension
- Code actions for executing statements under the cursor
- Schema-aware validation and object resolution

## Configuration

Configure database connection details in your `postgres-language-server.jsonc` file:

```json
{
  "db": {
    // Database host address (default: "127.0.0.1")
    "host": "localhost",
    // Database port (default: 5432)
    "port": 5432,
    // Database username (default: "postgres")
    "username": "postgres",
    // Database password (default: `PGPASSWORD`, then the password file, then "postgres")
    "password": "your_password",
    // Database name to connect to (default: "postgres")
    "database": "your_database_name",
    // Connection timeout in seconds (default: 10)
    "connTimeoutSecs": 10,
    // Glob patterns matched against `host/database`; code actions may execute
    // statements only when one of them matches (default: [], never)
    "allowStatementExecutionsAgainst": ["localhost/*", "127.0.0.1/*"],
    // Completely disable database features (default: false)
    "disableConnection": false
  }
}
```

## Supabase projects

When you run `postgres-language-server init` in a directory that contains a `supabase/config.toml`, the generated configuration connects to the local Supabase database started by `supabase start`: host `127.0.0.1`, the port from `[db] port` in `supabase/config.toml` (default: `54322`), and `postgres` as username, password, and database. If the port cannot be read from `supabase/config.toml`, `init` uses `54322` and prints a warning. It also sets `migrations.migrationsDir` to `supabase/migrations`, so that the [migration rules](checking_migrations.md) apply to your migrations. Executing statements stays disabled; allow it with `db.allowStatementExecutionsAgainst` if you want to.

In a monorepo, run `init` at the root: if the root has no `supabase/config.toml`, `init` looks for one in the folders one or two levels below it, such as `apps/web/supabase/config.toml`, and uses it if there is exactly one. With several, it leaves the connection alone and lists them, so run `init` in the folder of the project you want instead.

Run inside the `supabase/` folder itself (a folder named `supabase` with a `config.toml` that has a `[db]` section), `init` writes the configuration into the project folder above it, next to `supabase/`, so that it applies to the whole project and the editor finds it when it opens the project.

## Password file

When neither the configuration, the connection string, nor `PGPASSWORD` provides a password, the password is looked up in the [password file](https://www.postgresql.org/docs/current/libpq-pgpass.html), the same way `psql` and other libpq clients do:

- The file is `PGPASSFILE` if set, else `~/.pgpass` (`%APPDATA%\postgresql\pgpass.conf` on Windows).
- Each line is `hostname:port:database:username:password`. `*` matches any value, and `:` and `\` in a field are escaped with `\`. The first line that matches the host, port, database, and username of the connection wins.
- The host is compared literally: an entry for `localhost` does not match `"host": "127.0.0.1"`.
- On Unix-like systems such as Linux and macOS, the file is ignored if its group or others can access it. Restrict it with `chmod 0600 ~/.pgpass`.

If no entry matches, the connection uses the default password `postgres`, except with a connection string, which then connects without a password.

## Connection failures

When a database is configured (`host` or `connectionString`, or `PGHOST` or `DATABASE_URL` in the environment), `postgres-language-server check` treats a failed connection or login as an error: it reports a `database/connection` diagnostic, still runs the checks that do not need the database, and exits with a non-zero code. This way a CI run cannot pass with the database-backed checks silently skipped.

To check without a database, disable the connection with `--disable-db` or `"disableConnection": true` (see [Disabling Database Features](#disabling-database-features)).

The language server keeps working without the database when the connection fails, and retries it in the background.

An invalid `connectionString` (or `DATABASE_URL`), such as one with a non-numeric port, disables the database connection instead of stopping the language server, which shows a warning. `postgres-language-server check` reports it as a `database/connection` error, like a failed connection.

## Setting the connection from the editor

An editor extension can set the connection for a running language server session with the LSP request `pgls/set_configuration_overrides`, which takes any configuration and applies it on top of the configuration file:

```json
{
  "overrides": {
    "db": {
      "host": "localhost",
      "port": 5432,
      "username": "postgres",
      "password": "your_password",
      "database": "your_database_name"
    },
    "typecheck": {
      "searchPath": ["tenant", "public"]
    }
  }
}
```

The overrides stay in effect until they are replaced or cleared with `{ "overrides": null }`, which returns to whatever the configuration file and the environment define. See [Client configuration overrides](../configuration.md#client-configuration-overrides) for the precedence rules.

## Security Considerations

### Read-Only Access
The language server primarily needs read access to your database schema. Consider creating a dedicated user with limited permissions:

```sql
CREATE USER postgres_language_server WITH PASSWORD 'secure_password';
GRANT CONNECT ON DATABASE your_database TO postgres_language_server;
GRANT USAGE ON SCHEMA public TO postgres_language_server;
GRANT SELECT ON ALL TABLES IN SCHEMA public TO postgres_language_server;
GRANT SELECT ON ALL SEQUENCES IN SCHEMA public TO postgres_language_server;
GRANT EXECUTE ON ALL FUNCTIONS IN SCHEMA public TO postgres_language_server;
```

### Statement Execution Control
Code actions that execute the statement under the cursor are disabled by default. To enable them, list glob patterns in `allowStatementExecutionsAgainst`. Each pattern is matched against `host/database` of the connection (taken from the connection string when one is set), and statements can only be executed when one of them matches:

```json
{
  "db": {
    // any database on the local machine
    "allowStatementExecutionsAgainst": ["localhost/*", "127.0.0.1/*"]
  }
}
```

The patterns do not refer to schemas. To allow a single database only, name it, e.g. `"localhost/my_dev_db"`.

When no pattern matches, the code action is disabled and its reason names the setting. An invalid pattern is ignored with a warning in the log, and the reason lists it.

### Statement Execution Results
The language server runs the statement with the LSP command `pgls.executeStatement`, shows a short summary with `window/showMessage`, and returns the result of `workspace/executeCommand`, so an editor extension can render the rows as a table:

```json
{
  "message": "2 rows",
  "columns": ["id", "name"],
  "rows": [
    ["1", "Alice"],
    ["2", null]
  ],
  "truncated": false
}
```

- `columns` and `rows` are filled for statements that return rows, such as `select`, `values`, `show`, `explain`, and statements with `returning`.
- Each value is a string in its Postgres text representation, as `psql` shows it, or `null` for SQL `NULL`.
- At most 100 rows are returned. `truncated` is `true` if the statement returned more.
- `rowsAffected` is set instead of `columns` and `rows` for statements that return no rows, such as `insert` without `returning` or `create table`. For example: `{ "message": "Successfully executed statement. Rows affected: 2", "columns": [], "rows": [], "truncated": false, "rowsAffected": 2 }`.
- If the statement is not executed, for example because statement execution is disabled, only `message` is meaningful: `columns` and `rows` are empty and `rowsAffected` is not set.
- If Postgres rejects the statement, the command fails with a JSON-RPC error carrying the Postgres error message.

## Disabling Database Features

If you prefer to work without a database connection, you can disable all database-related features:

```json
{
  "db": {
    "disableConnection": true
  }
}
```

Or use the command line flag:

```bash
postgres-language-server check sql/ --disable-db
```

When disabled, you'll still get:

- Basic syntax highlighting
- Linting rules that don't require schema information
