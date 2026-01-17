# unusedIndex

**Diagnostic Category: `splinter/performance/unusedIndex`**

**Severity**: Info

## Description

Detects if an index has never been used and may be a candidate for removal.

## Remediation

https://supabase.com/docs/guides/database/database-linter?lint=0005_unused_index

## SQL Query

```sql
(
select
    'unused_index' as "name!",
    'Unused Index' as "title!",
    'INFO' as "level!",
    'EXTERNAL' as "facing!",
    array['PERFORMANCE'] as "categories!",
    'Detects if an index has never been used and may be a candidate for removal.' as "description!",
    format(
        'Index \`%s\` on table \`%s.%s\` has not been used',
        psui.indexrelname,
        psui.schemaname,
        psui.relname
    ) as "detail!",
    'https://supabase.com/docs/guides/database/database-linter?lint=0005_unused_index' as "remediation!",
    jsonb_build_object(
        'schema', psui.schemaname,
        'name', psui.relname,
        'type', 'table'
    ) as "metadata!",
    format(
        'unused_index_%s_%s_%s',
        psui.schemaname,
        psui.relname,
        psui.indexrelname
    ) as "cache_key!"

from
    pg_catalog.pg_stat_user_indexes psui
    join pg_catalog.pg_index pi
        on psui.indexrelid = pi.indexrelid
    left join pg_catalog.pg_depend dep
        on psui.relid = dep.objid
        and dep.deptype = 'e'
where
    psui.idx_scan = 0
    and not pi.indisunique
    and not pi.indisprimary
    and dep.objid is null -- exclude tables owned by extensions
    and psui.schemaname not in (
        '_timescaledb_cache', '_timescaledb_catalog', '_timescaledb_config', '_timescaledb_internal', 'auth', 'cron', 'extensions', 'graphql', 'graphql_public', 'information_schema', 'net', 'pgmq', 'pgroonga', 'pgsodium', 'pgsodium_masks', 'pgtle', 'pgbouncer', 'pg_catalog', 'pgtle', 'realtime', 'repack', 'storage', 'supabase_functions', 'supabase_migrations', 'tiger', 'topology', 'vault'
    ))
```

## How to configure

```json
{
  "splinter": {
    "rules": {
      "performance": {
        "unusedIndex": "error"
      }
    }
  }
}
```
