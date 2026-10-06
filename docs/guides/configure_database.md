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
    // Schemas where code action statement execution is allowed (default: [])
    "allowStatementExecutionsAgainst": ["public", "testing"],
    // Completely disable database features (default: false)
    "disableConnection": false
  }
}
```

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
You can control which schemas allow code action statement execution (executing statements under the cursor):

```json
{
  "db": {
    "allowStatementExecutionsAgainst": ["public", "testing"]
  }
}
```

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
