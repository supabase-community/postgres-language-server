# PL/pgSQL

Postgres Language Tools partially supports PL/pgSQL. We use `libpg_query` to parse the function body and show any syntax error. For a more sophisticated integration, make sure to enable the `plpgsql_check` extension in your development database.

```sql
CREATE EXTENSION IF NOT EXISTS plpgsql_check;
```

If the extension is detected, we leverage it to run more advanced checks against your PL/pgSQL functions.
