# Autocompletion & Hover

The language server provides autocompletion and hover information when connected to a database.

## Autocompletion

As you type SQL, the language server suggests relevant database objects based on your current context:

- **Tables**: Available tables from your database schema
- **Columns**: Columns from tables referenced in your query
- **Functions**: Database functions and procedures
- **Schemas**: Available schemas in your database
- **Keywords**: SQL keywords and syntax

The suggestions are context-aware - for example, when typing after `FROM`, you'll see table suggestions, and when typing after `SELECT`, you'll see column suggestions from relevant tables.

## Hover Information

Hovering over database objects in your SQL shows detailed information:

- **Tables**: Schema, column list with data types
- **Columns**: Data type, nullable status, table location
- **Functions**: Return type, parameter information

The hover information is pulled from your database schema.

## Requirements

Both features require:
- A configured database connection
- The language server must be able to read schema information from your database

Without a database connection, these features are not available.

## Configuration

These features work automatically when you have a database connection configured. See the [database configuration guide](../guides/configure_database.md) for setup instructions.

The language server caches schema information on startup.
