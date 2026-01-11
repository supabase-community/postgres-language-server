# securityDefinerView

**Diagnostic Category: `splinter/security/securityDefinerView`**

**Severity**: Error

> [!NOTE]
> This rule requires a Supabase database/project and will be automatically skipped if not detected.

## Description

Detects views defined with the SECURITY DEFINER property. These views enforce Postgres permissions and row level security policies (RLS) of the view creator, rather than that of the querying user

## Remediation

https://supabase.com/docs/guides/database/database-linter?lint=0010_security_definer_view

## SQL Query

```sql
(
select
    'security_definer_view' as "name!",
    'Security Definer View' as "title!",
    'ERROR' as "level!",
    'EXTERNAL' as "facing!",
    array['SECURITY'] as "categories!",
    'Detects views defined with the SECURITY DEFINER property. These views enforce Postgres permissions and row level security policies (RLS) of the view creator, rather than that of the querying user' as "description!",
    format(
        'View \`%s.%s\` is defined with the SECURITY DEFINER property',
        n.nspname,
        c.relname
    ) as "detail!",
    'https://supabase.com/docs/guides/database/database-linter?lint=0010_security_definer_view' as "remediation!",
    jsonb_build_object(
        'schema', n.nspname,
        'name', c.relname,
        'type', 'view'
    ) as "metadata!",
    format(
        'security_definer_view_%s_%s',
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
    c.relkind = 'v'
    and (
        pg_catalog.has_table_privilege('anon', c.oid, 'SELECT')
        or pg_catalog.has_table_privilege('authenticated', c.oid, 'SELECT')
    )
    and substring(pg_catalog.version() from 'PostgreSQL ([0-9]+)') >= '15' -- security invoker was added in pg15
    and n.nspname = any(array(select trim(unnest(string_to_array(current_setting('pgrst.db_schemas', 't'), ',')))))
    and n.nspname not in (
        '_timescaledb_cache', '_timescaledb_catalog', '_timescaledb_config', '_timescaledb_internal', 'auth', 'cron', 'extensions', 'graphql', 'graphql_public', 'information_schema', 'net', 'pgmq', 'pgroonga', 'pgsodium', 'pgsodium_masks', 'pgtle', 'pgbouncer', 'pg_catalog', 'pgtle', 'realtime', 'repack', 'storage', 'supabase_functions', 'supabase_migrations', 'tiger', 'topology', 'vault'
    )
    and dep.objid is null -- exclude views owned by extensions
    and not (
        lower(coalesce(c.reloptions::text,'{}'))::text[]
        && array[
            'security_invoker=1',
            'security_invoker=true',
            'security_invoker=yes',
            'security_invoker=on'
        ]
    ))
```

## How to configure

```json
{
  "splinter": {
    "rules": {
      "security": {
        "securityDefinerView": "error"
      }
    }
  }
}
```
