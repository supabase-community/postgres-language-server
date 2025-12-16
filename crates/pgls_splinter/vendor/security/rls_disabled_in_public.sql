-- meta: name = rlsDisabledInPublic
-- meta: title = RLS Disabled in Public
-- meta: severity = ERROR
-- meta: category = SECURITY
-- meta: description = Detects cases where row level security (RLS) has not been enabled on tables in schemas exposed to PostgREST
-- meta: remediation = https://supabase.com/docs/guides/database/database-linter?lint=0013_rls_disabled_in_public

(
select
    'rls_disabled_in_public' as "name!",
    'RLS Disabled in Public' as "title!",
    'ERROR' as "level!",
    'EXTERNAL' as "facing!",
    array['SECURITY'] as "categories!",
    'Detects cases where row level security (RLS) has not been enabled on tables in schemas exposed to PostgREST' as "description!",
    format(
        'Table \`%s.%s\` is public, but RLS has not been enabled.',
        n.nspname,
        c.relname
    ) as "detail!",
    'https://supabase.com/docs/guides/database/database-linter?lint=0013_rls_disabled_in_public' as "remediation!",
    jsonb_build_object(
        'schema', n.nspname,
        'name', c.relname,
        'type', 'table'
    ) as "metadata!",
    format(
        'rls_disabled_in_public_%s_%s',
        n.nspname,
        c.relname
    ) as "cache_key!"
from
    pg_catalog.pg_class c
    join pg_catalog.pg_namespace n
        on c.relnamespace = n.oid
where
    c.relkind = 'r' -- regular tables
    -- RLS is disabled
    and not c.relrowsecurity
    and (
        pg_catalog.has_table_privilege('anon', c.oid, 'SELECT')
        or pg_catalog.has_table_privilege('authenticated', c.oid, 'SELECT')
    )
    and n.nspname = any(array(select trim(unnest(string_to_array(current_setting('pgrst.db_schemas', 't'), ',')))))
    and n.nspname not in (
        '_timescaledb_cache', '_timescaledb_catalog', '_timescaledb_config', '_timescaledb_internal', 'auth', 'cron', 'extensions', 'graphql', 'graphql_public', 'information_schema', 'net', 'pgmq', 'pgroonga', 'pgsodium', 'pgsodium_masks', 'pgtle', 'pgbouncer', 'pg_catalog', 'pgtle', 'realtime', 'repack', 'storage', 'supabase_functions', 'supabase_migrations', 'tiger', 'topology', 'vault'
    ))
