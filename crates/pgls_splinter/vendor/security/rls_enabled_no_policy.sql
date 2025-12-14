-- meta: name = rlsEnabledNoPolicy
-- meta: title = RLS Enabled No Policy
-- meta: severity = INFO
-- meta: category = SECURITY
-- meta: description = Detects cases where row level security (RLS) has been enabled on a table but no RLS policies have been created.
-- meta: remediation = https://supabase.com/docs/guides/database/database-linter?lint=0008_rls_enabled_no_policy

(
select
    'rls_enabled_no_policy' as "name!",
    'RLS Enabled No Policy' as "title!",
    'INFO' as "level!",
    'EXTERNAL' as "facing!",
    array['SECURITY'] as "categories!",
    'Detects cases where row level security (RLS) has been enabled on a table but no RLS policies have been created.' as "description!",
    format(
        'Table \`%s.%s\` has RLS enabled, but no policies exist',
        n.nspname,
        c.relname
    ) as "detail!",
    'https://supabase.com/docs/guides/database/database-linter?lint=0008_rls_enabled_no_policy' as "remediation!",
    jsonb_build_object(
        'schema', n.nspname,
        'name', c.relname,
        'type', 'table'
    ) as "metadata!",
    format(
        'rls_enabled_no_policy_%s_%s',
        n.nspname,
        c.relname
    ) as "cache_key!"
from
    pg_catalog.pg_class c
    left join pg_catalog.pg_policy p
        on p.polrelid = c.oid
    join pg_catalog.pg_namespace n
        on c.relnamespace = n.oid
    left join pg_catalog.pg_depend dep
        on c.oid = dep.objid
        and dep.deptype = 'e'
where
    c.relkind = 'r' -- regular tables
    and n.nspname not in (
        '_timescaledb_cache', '_timescaledb_catalog', '_timescaledb_config', '_timescaledb_internal', 'auth', 'cron', 'extensions', 'graphql', 'graphql_public', 'information_schema', 'net', 'pgmq', 'pgroonga', 'pgsodium', 'pgsodium_masks', 'pgtle', 'pgbouncer', 'pg_catalog', 'pgtle', 'realtime', 'repack', 'storage', 'supabase_functions', 'supabase_migrations', 'tiger', 'topology', 'vault'
    )
    -- RLS is enabled
    and c.relrowsecurity
    and p.polname is null
    and dep.objid is null -- exclude tables owned by extensions
group by
    n.nspname,
    c.relname)
