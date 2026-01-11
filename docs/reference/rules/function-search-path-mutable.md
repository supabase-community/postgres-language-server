# functionSearchPathMutable

**Diagnostic Category: `splinter/security/functionSearchPathMutable`**

**Severity**: Warning

## Description

Detects functions where the search_path parameter is not set.

## Remediation

https://supabase.com/docs/guides/database/database-linter?lint=0011_function_search_path_mutable

## SQL Query

```sql
(
select
    'function_search_path_mutable' as "name!",
    'Function Search Path Mutable' as "title!",
    'WARN' as "level!",
    'EXTERNAL' as "facing!",
    array['SECURITY'] as "categories!",
    'Detects functions where the search_path parameter is not set.' as "description!",
    format(
        'Function \`%s.%s\` has a role mutable search_path',
        n.nspname,
        p.proname
    ) as "detail!",
    'https://supabase.com/docs/guides/database/database-linter?lint=0011_function_search_path_mutable' as "remediation!",
    jsonb_build_object(
        'schema', n.nspname,
        'name', p.proname,
        'type', 'function'
    ) as "metadata!",
    format(
        'function_search_path_mutable_%s_%s_%s',
        n.nspname,
        p.proname,
        md5(p.prosrc) -- required when function is polymorphic
    ) as "cache_key!"
from
    pg_catalog.pg_proc p
    join pg_catalog.pg_namespace n
        on p.pronamespace = n.oid
    left join pg_catalog.pg_depend dep
        on p.oid = dep.objid
        and dep.deptype = 'e'
where
    n.nspname not in (
        '_timescaledb_cache', '_timescaledb_catalog', '_timescaledb_config', '_timescaledb_internal', 'auth', 'cron', 'extensions', 'graphql', 'graphql_public', 'information_schema', 'net', 'pgmq', 'pgroonga', 'pgsodium', 'pgsodium_masks', 'pgtle', 'pgbouncer', 'pg_catalog', 'pgtle', 'realtime', 'repack', 'storage', 'supabase_functions', 'supabase_migrations', 'tiger', 'topology', 'vault'
    )
    and dep.objid is null -- exclude functions owned by extensions
    -- Search path not set
    and not exists (
        select 1
        from unnest(coalesce(p.proconfig, '{}')) as config
        where config like 'search_path=%'
    ))
```

## How to configure

```json
{
  "splinter": {
    "rules": {
      "security": {
        "functionSearchPathMutable": "error"
      }
    }
  }
}
```
