# foreignTableInApi

**Diagnostic Category: `splinter/security/foreignTableInApi`**

**Severity**: Warning

> [!NOTE]
> This rule requires a Supabase database/project and will be automatically skipped if not detected.

## Description

Detects foreign tables that are accessible over APIs. Foreign tables do not respect row level security policies.

## Remediation

https://supabase.com/docs/guides/database/database-linter?lint=0017_foreign_table_in_api

## SQL Query

```sql
(
select
    'foreign_table_in_api' as "name!",
    'Foreign Table in API' as "title!",
    'WARN' as "level!",
    'EXTERNAL' as "facing!",
    array['SECURITY'] as "categories!",
    'Detects foreign tables that are accessible over APIs. Foreign tables do not respect row level security policies.' as "description!",
    format(
        'Foreign table \`%s.%s\` is accessible over APIs',
        n.nspname,
        c.relname
    ) as "detail!",
    'https://supabase.com/docs/guides/database/database-linter?lint=0017_foreign_table_in_api' as "remediation!",
    jsonb_build_object(
        'schema', n.nspname,
        'name', c.relname,
        'type', 'foreign table'
    ) as "metadata!",
    format(
        'foreign_table_in_api_%s_%s',
        n.nspname,
        c.relname
    ) as "cache_key!"
from
    pg_catalog.pg_class c
    join pg_catalog.pg_namespace n
        on n.oid = c.relnamespace
    left join pg_catalog.pg_depend dep
        on c.oid = dep.objid
        and dep.deptype = 'e'
where
    c.relkind = 'f'
    and (
        pg_catalog.has_table_privilege('anon', c.oid, 'SELECT')
        or pg_catalog.has_table_privilege('authenticated', c.oid, 'SELECT')
    )
    and n.nspname = any(array(select trim(unnest(string_to_array(current_setting('pgrst.db_schemas', 't'), ',')))))
    and n.nspname not in (
        '_timescaledb_cache', '_timescaledb_catalog', '_timescaledb_config', '_timescaledb_internal', 'auth', 'cron', 'extensions', 'graphql', 'graphql_public', 'information_schema', 'net', 'pgmq', 'pgroonga', 'pgsodium', 'pgsodium_masks', 'pgtle', 'pgbouncer', 'pg_catalog', 'pgtle', 'realtime', 'repack', 'storage', 'supabase_functions', 'supabase_migrations', 'tiger', 'topology', 'vault'
    )
    and dep.objid is null)
```

## How to configure

```json
{
  "splinter": {
    "rules": {
      "security": {
        "foreignTableInApi": "error"
      }
    }
  }
}
```
