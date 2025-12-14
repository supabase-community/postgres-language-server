-- meta: name = duplicateIndex
-- meta: title = Duplicate Index
-- meta: severity = WARN
-- meta: category = PERFORMANCE
-- meta: description = Detects cases where two ore more identical indexes exist.
-- meta: remediation = https://supabase.com/docs/guides/database/database-linter?lint=0009_duplicate_index

(
select
    'duplicate_index' as "name!",
    'Duplicate Index' as "title!",
    'WARN' as "level!",
    'EXTERNAL' as "facing!",
    array['PERFORMANCE'] as "categories!",
    'Detects cases where two ore more identical indexes exist.' as "description!",
    format(
        'Table \`%s.%s\` has identical indexes %s. Drop all except one of them',
        n.nspname,
        c.relname,
        array_agg(pi.indexname order by pi.indexname)
    ) as "detail!",
    'https://supabase.com/docs/guides/database/database-linter?lint=0009_duplicate_index' as "remediation!",
    jsonb_build_object(
        'schema', n.nspname,
        'name', c.relname,
        'type', case
            when c.relkind = 'r' then 'table'
            when c.relkind = 'm' then 'materialized view'
            else 'ERROR'
        end,
        'indexes', array_agg(pi.indexname order by pi.indexname)
    ) as "metadata!",
    format(
        'duplicate_index_%s_%s_%s',
        n.nspname,
        c.relname,
        array_agg(pi.indexname order by pi.indexname)
    ) as "cache_key!"
from
    pg_catalog.pg_indexes pi
    join pg_catalog.pg_namespace n
        on n.nspname  = pi.schemaname
    join pg_catalog.pg_class c
        on pi.tablename = c.relname
        and n.oid = c.relnamespace
    left join pg_catalog.pg_depend dep
        on c.oid = dep.objid
        and dep.deptype = 'e'
where
    c.relkind in ('r', 'm') -- tables and materialized views
    and n.nspname not in (
        '_timescaledb_cache', '_timescaledb_catalog', '_timescaledb_config', '_timescaledb_internal', 'auth', 'cron', 'extensions', 'graphql', 'graphql_public', 'information_schema', 'net', 'pgmq', 'pgroonga', 'pgsodium', 'pgsodium_masks', 'pgtle', 'pgbouncer', 'pg_catalog', 'pgtle', 'realtime', 'repack', 'storage', 'supabase_functions', 'supabase_migrations', 'tiger', 'topology', 'vault'
    )
    and dep.objid is null -- exclude tables owned by extensions
group by
    n.nspname,
    c.relkind,
    c.relname,
    replace(pi.indexdef, pi.indexname, '')
having
    count(*) > 1)
