# Formatting

> **Preview Feature**: The formatter is currently in preview. We'd love feedback from early adopters! Please report any issues or unexpected output at [GitHub Issues](https://github.com/supabase-community/postgres-language-server/issues).

The language server provides SQL formatting that produces consistent, readable code. Built on Postgres' own parser, the formatter ensures 100% syntax compatibility with your SQL.

## What the formatter changes

The formatter changes whitespace, line breaks, the case of keywords, types and constants, and quotes that an identifier does not need. It does not change what is written:

- Schema qualifiers stay as written, `pg_catalog.` included.
- Function and `DO` bodies are kept byte for byte, and always dollar-quoted with `$$` (or `$body$` when the body contains `$$`), whatever quotes they were written with. Only `language sql` bodies are reformatted, unless `skipFnBodies` is set.
- String literals are kept as written.
- Function signatures keep their argument names and modes, in `ALTER FUNCTION`, `GRANT` and `DROP` as well.
- Identifiers that need quotes keep them: mixed case, reserved words such as `"user"`, and special characters.
- Comments stay where they were written, including comments at the end of a line.

The formatter checks every statement: if the formatted statement would not parse to the same tree as the original, the statement is left as written and a diagnostic is reported.

Between statements, the formatter keeps at most one blank line, and removes blank lines at the start and the end of the file.

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
    "typeCase": "lower",
    "functionArgumentGroups": {
      "my_pair_func": 2
    }
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
| `commaStyle` | `"trailing"` | Where a comma sits when a list breaks: `"trailing"` or `"leading"` |
| `logicalOperatorPlacement` | `"trailing"` | Where `AND` and `OR` sit when a condition breaks: `"trailing"` or `"leading"` |
| `layout` | `"fit"` | How a statement is laid out: `"fit"` breaks only past the line width, `"expanded"` always breaks between clauses |
| `valuesRowLayout` | `"fit"` | How `VALUES` rows are laid out: `"fit"` breaks only past the line width, `"expanded"` puts each tuple on its own indented line. `commaStyle` controls commas between rows as well as inside tuples |
| `castStyle` | `"cast"` | How an explicit cast is spelled: `"cast"` for `CAST(x AS t)`, `"operator"` for `x::t` |
| `clauseBodyStyle` | `"break"` | Where a clause body starts: `"break"` for a new line, `"compact"` to keep the first element on the keyword line. Compact also keeps a select list of a single item on the `SELECT` line |
| `isolateSemicolon` | `false` | Put the terminating semicolon on its own line when the statement spans several lines |
| `functionArgumentGroups` | `{}` (plus built-in defaults, see below) | Function names mapped to the number of adjacent arguments that should stay together when wrapping |

With `valuesRowLayout: "expanded"` and `commaStyle: "leading"`, each `VALUES`
tuple starts on its own indented line, even when the rows fit within `lineWidth`:

```sql
values
  ('MONTHLY', 1)
  , ('QUARTERLY', 3);
```

The expressions inside each tuple stay on one line when they fit. This option
applies to `INSERT`, standalone `VALUES`, and `VALUES` in subqueries and CTEs,
independently of `layout` and `clauseBodyStyle`.

With `clauseBodyStyle: "compact"`, the first relation stays on the `FROM` line
and the condition on the `WHERE`, `HAVING`, or `ON` line. A select list of a
single item stays on the `SELECT` line, while a list of several items still
puts one item per line:

```sql
select count(*)
from memberships
where memberships.user_id = $1 and memberships.role = 'admin' and memberships.active;

select
  memberships.user_id,
  memberships.role
from memberships
where memberships.organization_id = $1 and memberships.active;
```

`functionArgumentGroups` matches the final, unqualified function name
case-insensitively. A group size of `2` is useful for functions whose arguments
form key/value pairs:

```sql
select jsonb_build_object(
  'amountTTC', to_amount(amount_ttc, invoices.currency),
  'amountVAT', to_amount(amount_vat, invoices.currency)
);
```

A group stays on one line when it fits; when it does not, the formatter breaks
between its arguments rather than exceeding the line width.

`json_build_object` and `jsonb_build_object` are grouped in pairs by default,
without configuration. Configure a function explicitly to use a different group
size; a size of `1` opts out of the built-in default:

```json
{
  "format": {
    "functionArgumentGroups": {
      "jsonb_build_object": 1
    }
  }
}
```

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
# Check formatting and show a diff for every statement that would change
postgres-language-server format file.sql

# Format and write changes
postgres-language-server format file.sql --write

# Format entire directory
postgres-language-server format migrations/ --write
```

### Check mode

Without `--write`, `format` does not modify any file. For every statement that would be reformatted it prints a diagnostic with the file path and a diff to stderr, followed by a summary (`Processed N files ... Found N errors.`). The command exits with a non-zero code if any file is not formatted or could not be formatted (e.g. because of a syntax error), and with `0` otherwise. This makes it suitable for CI:

```bash
postgres-language-server format migrations/
```

`--staged` and `--changed` (optionally with `--since=<REF>`) limit the run to the files staged for commit or changed compared to your default branch.

### Formatting stdin

Use `--stdin-file-path=<PATH>` to format SQL piped through stdin, for example from an editor or a script. Only the formatted SQL is printed to stdout:

```bash
echo "SELECT a,b FROM t;" | postgres-language-server format --stdin-file-path=query.sql
# select a, b from t;
```

The file doesn't need to exist. Its path is used to apply the project configuration (for example, a path matching `files.ignore` is printed back unchanged) and to label diagnostics. If a statement cannot be formatted, it is printed unchanged, its diagnostics are written to stderr, and the command exits with a non-zero code. When the formatter is disabled, the input is printed unchanged. `--stdin-file-path` cannot be combined with file paths, `--write`, `--staged` or `--changed`.

## Editor Integration

The formatter integrates with your editor via the Language Server Protocol. Use your editor's format document command (typically bound to a keyboard shortcut) to format SQL files.

## JavaScript API

The `@postgres-language-server/wasm` package formats SQL in-process, with the same options as the `format` section of the configuration file:

```typescript
import { format } from "@postgres-language-server/wasm";

const sql = await format('CREATE TABLE "public"."t" ("id" "uuid" NOT NULL)', {
  keywordCase: "lower",
});
```

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
