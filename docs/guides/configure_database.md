# Configure database connection

The language server requires a database connection for schema-dependent features.

## Features requiring database connection

- Type checking using `EXPLAIN`
- Autocompletion for tables, columns, functions, and schemas
- Hover information for database objects
- PL/pgSQL analysis via `plpgsql_check` extension
- Code actions for executing statements under the cursor
- Schema-aware validation and object resolution

## Configuration

Configure database connection details in your `postgres-language-server.jsonc` file:

```json
{
  "database": {
    // Database host address (default: "127.0.0.1")
    "host": "localhost",
    // Database port (default: 5432)
    "port": 5432,
    // Database username (default: "postgres")
    "username": "postgres",
    // Database password (default: "postgres")
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
  "database": {
    "allowStatementExecutionsAgainst": ["public", "testing"]
  }
}
```

## Disabling Database Features

If you prefer to work without a database connection, you can disable all database-related features:

```json
{
  "database": {
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
