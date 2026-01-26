# Formatting

> **Preview Feature**: The formatter is currently in preview. We'd love feedback from early adopters! Please report any issues or unexpected output at [GitHub Issues](https://github.com/supabase-community/postgres-language-server/issues).

## Known Limitations

!!! warning "Comments are not yet supported"
    SQL comments (`--` and `/* */`) will be removed during formatting. This is a temporary limitation that will be addressed in a future release. If your SQL files contain important comments, consider waiting for comment support before using the formatter on those files.

The language server provides SQL formatting that produces consistent, readable code. Built on Postgres' own parser, the formatter ensures 100% syntax compatibility with your SQL.

## Configuration

Configure formatting behavior in your `postgres-language-server.jsonc`:

```json
{
  "format": {
    "enabled": true,
    "lineWidth": 100,
    "indentSize": 2,
    "indentStyle": "spaces",
    "keywordCase": "lower",
    "constantCase": "lower",
    "typeCase": "lower"
  }
}
```

### Options

| Option | Default | Description |
|--------|---------|-------------|
| `enabled` | `true` | Enable or disable the formatter |
| `lineWidth` | `100` | Maximum line width before breaking |
| `indentSize` | `2` | Number of spaces (or tab width) for indentation |
| `indentStyle` | `"spaces"` | Use `"spaces"` or `"tabs"` for indentation |
| `keywordCase` | `"lower"` | Casing for SQL keywords: `"upper"` or `"lower"` |
| `constantCase` | `"lower"` | Casing for constants (NULL, TRUE, FALSE): `"upper"` or `"lower"` |
| `typeCase` | `"lower"` | Casing for data types (text, int, varchar): `"upper"` or `"lower"` |

### Example Output

With default settings (lowercase):

```sql
create table users (
  id serial primary key,
  name text not null,
  active boolean default true
);

select * from users where active = true;
```

With uppercase keywords and constants:

```sql
CREATE TABLE users (
  id serial PRIMARY KEY,
  name text NOT NULL,
  active boolean DEFAULT TRUE
);

SELECT * FROM users WHERE active = TRUE;
```

## CLI Usage

Format files using the CLI:

```bash
# Format and show diff
postgres-language-server format file.sql

# Format and write changes
postgres-language-server format file.sql --write

# Format entire directory
postgres-language-server format migrations/ --write
```

## Editor Integration

The formatter integrates with your editor via the Language Server Protocol. Use your editor's format document command (typically bound to a keyboard shortcut) to format SQL files.

## Ignoring Files

Use the `ignore` and `include` options to control which files are formatted:

```json
{
  "format": {
    "ignore": ["**/generated/**", "**/vendor/**"],
    "include": ["**/*.sql"]
  }
}
```
