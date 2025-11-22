
(
with foreign_keys as (
    select
        cl.relnamespace::regnamespace::text as schema_name,
        cl.relname as table_name,
        cl.oid as table_oid,
        ct.conname as fkey_name,
        ct.conkey as col_attnums
    from
        pg_catalog.pg_constraint ct
        join pg_catalog.pg_class cl -- fkey owning table
            on ct.conrelid = cl.oid
        left join pg_catalog.pg_depend d
            on d.objid = cl.oid
            and d.deptype = 'e'
    where
        ct.contype = 'f' -- foreign key constraints
        and d.objid is null -- exclude tables that are dependencies of extensions
        and cl.relnamespace::regnamespace::text not in (
            'pg_catalog', 'information_schema', 'auth', 'storage', 'vault', 'extensions'
        )
),
index_ as (
    select
        pi.indrelid as table_oid,
        indexrelid::regclass as index_,
        string_to_array(indkey::text, ' ')::smallint[] as col_attnums
    from
        pg_catalog.pg_index pi
    where
        indisvalid
)
select
    'unindexed_foreign_keys' as "name!",
    'Unindexed foreign keys' as "title!",
    'INFO' as "level!",
    'EXTERNAL' as "facing!",
    array['PERFORMANCE'] as "categories!",
    'Identifies foreign key constraints without a covering index, which can impact database performance.' as "description!",
    format(
        'Table \`%s.%s\` has a foreign key \`%s\` without a covering index. This can lead to suboptimal query performance.',
        fk.schema_name,
        fk.table_name,
        fk.fkey_name
    ) as "detail!",
    'https://supabase.com/docs/guides/database/database-linter?lint=0001_unindexed_foreign_keys' as "remediation!",
    jsonb_build_object(
        'schema', fk.schema_name,
        'name', fk.table_name,
        'type', 'table',
        'fkey_name', fk.fkey_name,
        'fkey_columns', fk.col_attnums
    ) as "metadata!",
    format('unindexed_foreign_keys_%s_%s_%s', fk.schema_name, fk.table_name, fk.fkey_name) as "cache_key!"
from
    foreign_keys fk
    left join index_ idx
        on fk.table_oid = idx.table_oid
        and fk.col_attnums = idx.col_attnums[1:array_length(fk.col_attnums, 1)]
    left join pg_catalog.pg_depend dep
        on idx.table_oid = dep.objid
        and dep.deptype = 'e'
where
    idx.index_ is null
    and fk.schema_name not in (
        '_timescaledb_cache', '_timescaledb_catalog', '_timescaledb_config', '_timescaledb_internal', 'auth', 'cron', 'extensions', 'graphql', 'graphql_public', 'information_schema', 'net', 'pgmq', 'pgroonga', 'pgsodium', 'pgsodium_masks', 'pgtle', 'pgbouncer', 'pg_catalog', 'pgtle', 'realtime', 'repack', 'storage', 'supabase_functions', 'supabase_migrations', 'tiger', 'topology', 'vault'
    )
    and dep.objid is null -- exclude tables owned by extensions
order by
    fk.schema_name,
    fk.table_name,
    fk.fkey_name)
union all

(
select
    'no_primary_key' as "name!",
    'No Primary Key' as "title!",
    'INFO' as "level!",
    'EXTERNAL' as "facing!",
    array['PERFORMANCE'] as "categories!",
    'Detects if a table does not have a primary key. Tables without a primary key can be inefficient to interact with at scale.' as "description!",
    format(
        'Table \`%s.%s\` does not have a primary key',
        pgns.nspname,
        pgc.relname
    ) as "detail!",
    'https://supabase.com/docs/guides/database/database-linter?lint=0004_no_primary_key' as "remediation!",
     jsonb_build_object(
        'schema', pgns.nspname,
        'name', pgc.relname,
        'type', 'table'
    ) as "metadata!",
    format(
        'no_primary_key_%s_%s',
        pgns.nspname,
        pgc.relname
    ) as "cache_key!"
from
    pg_catalog.pg_class pgc
    join pg_catalog.pg_namespace pgns
        on pgns.oid = pgc.relnamespace
    left join pg_catalog.pg_index pgi
        on pgi.indrelid = pgc.oid
    left join pg_catalog.pg_depend dep
        on pgc.oid = dep.objid
        and dep.deptype = 'e'
where
    pgc.relkind = 'r' -- regular tables
    and pgns.nspname not in (
        '_timescaledb_cache', '_timescaledb_catalog', '_timescaledb_config', '_timescaledb_internal', 'auth', 'cron', 'extensions', 'graphql', 'graphql_public', 'information_schema', 'net', 'pgmq', 'pgroonga', 'pgsodium', 'pgsodium_masks', 'pgtle', 'pgbouncer', 'pg_catalog', 'pgtle', 'realtime', 'repack', 'storage', 'supabase_functions', 'supabase_migrations', 'tiger', 'topology', 'vault'
    )
    and dep.objid is null -- exclude tables owned by extensions
group by
    pgc.oid,
    pgns.nspname,
    pgc.relname
having
    max(coalesce(pgi.indisprimary, false)::int) = 0)
union all

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
union all

(
select
    'multiple_permissive_policies' as "name!",
    'Multiple Permissive Policies' as "title!",
    'WARN' as "level!",
    'EXTERNAL' as "facing!",
    array['PERFORMANCE'] as "categories!",
    'Detects if multiple permissive row level security policies are present on a table for the same \`role\` and \`action\` (e.g. insert). Multiple permissive policies are suboptimal for performance as each policy must be executed for every relevant query.' as "description!",
    format(
        'Table \`%s.%s\` has multiple permissive policies for role \`%s\` for action \`%s\`. Policies include \`%s\`',
        n.nspname,
        c.relname,
        r.rolname,
        act.cmd,
        array_agg(p.polname order by p.polname)
    ) as "detail!",
    'https://supabase.com/docs/guides/database/database-linter?lint=0006_multiple_permissive_policies' as "remediation!",
    jsonb_build_object(
        'schema', n.nspname,
        'name', c.relname,
        'type', 'table'
    ) as "metadata!",
    format(
        'multiple_permissive_policies_%s_%s_%s_%s',
        n.nspname,
        c.relname,
        r.rolname,
        act.cmd
    ) as "cache_key!"
from
    pg_catalog.pg_policy p
    join pg_catalog.pg_class c
        on p.polrelid = c.oid
    join pg_catalog.pg_namespace n
        on c.relnamespace = n.oid
    join pg_catalog.pg_roles r
        on p.polroles @> array[r.oid]
        or p.polroles = array[0::oid]
    left join pg_catalog.pg_depend dep
        on c.oid = dep.objid
        and dep.deptype = 'e',
    lateral (
        select x.cmd
        from unnest((
            select
                case p.polcmd
                    when 'r' then array['SELECT']
                    when 'a' then array['INSERT']
                    when 'w' then array['UPDATE']
                    when 'd' then array['DELETE']
                    when '*' then array['SELECT', 'INSERT', 'UPDATE', 'DELETE']
                    else array['ERROR']
                end as actions
        )) x(cmd)
    ) act(cmd)
where
    c.relkind = 'r' -- regular tables
    and p.polpermissive -- policy is permissive
    and n.nspname not in (
        '_timescaledb_cache', '_timescaledb_catalog', '_timescaledb_config', '_timescaledb_internal', 'auth', 'cron', 'extensions', 'graphql', 'graphql_public', 'information_schema', 'net', 'pgmq', 'pgroonga', 'pgsodium', 'pgsodium_masks', 'pgtle', 'pgbouncer', 'pg_catalog', 'pgtle', 'realtime', 'repack', 'storage', 'supabase_functions', 'supabase_migrations', 'tiger', 'topology', 'vault'
    )
    and r.rolname not like 'pg_%'
    and r.rolname not like 'supabase%admin'
    and not r.rolbypassrls
    and dep.objid is null -- exclude tables owned by extensions
group by
    n.nspname,
    c.relname,
    r.rolname,
    act.cmd
having
    count(1) > 1)
union all

(
select
    'policy_exists_rls_disabled' as "name!",
    'Policy Exists RLS Disabled' as "title!",
    'ERROR' as "level!",
    'EXTERNAL' as "facing!",
    array['SECURITY'] as "categories!",
    'Detects cases where row level security (RLS) policies have been created, but RLS has not been enabled for the underlying table.' as "description!",
    format(
        'Table \`%s.%s\` has RLS policies but RLS is not enabled on the table. Policies include %s.',
        n.nspname,
        c.relname,
        array_agg(p.polname order by p.polname)
    ) as "detail!",
    'https://supabase.com/docs/guides/database/database-linter?lint=0007_policy_exists_rls_disabled' as "remediation!",
    jsonb_build_object(
        'schema', n.nspname,
        'name', c.relname,
        'type', 'table'
    ) as "metadata!",
    format(
        'policy_exists_rls_disabled_%s_%s',
        n.nspname,
        c.relname
    ) as "cache_key!"
from
    pg_catalog.pg_policy p
    join pg_catalog.pg_class c
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
    -- RLS is disabled
    and not c.relrowsecurity
    and dep.objid is null -- exclude tables owned by extensions
group by
    n.nspname,
    c.relname)
union all

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
union all

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
union all

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
union all

(
select
    'extension_in_public' as "name!",
    'Extension in Public' as "title!",
    'WARN' as "level!",
    'EXTERNAL' as "facing!",
    array['SECURITY'] as "categories!",
    'Detects extensions installed in the \`public\` schema.' as "description!",
    format(
        'Extension \`%s\` is installed in the public schema. Move it to another schema.',
        pe.extname
    ) as "detail!",
    'https://supabase.com/docs/guides/database/database-linter?lint=0014_extension_in_public' as "remediation!",
    jsonb_build_object(
        'schema', pe.extnamespace::regnamespace,
        'name', pe.extname,
        'type', 'extension'
    ) as "metadata!",
    format(
        'extension_in_public_%s',
        pe.extname
    ) as "cache_key!"
from
    pg_catalog.pg_extension pe
where
    -- plpgsql is installed by default in public and outside user control
    -- confirmed safe
    pe.extname not in ('plpgsql')
    -- Scoping this to public is not optimal. Ideally we would use the postgres
    -- search path. That currently isn't available via SQL. In other lints
    -- we have used has_schema_privilege('anon', 'extensions', 'USAGE') but that
    -- is not appropriate here as it would evaluate true for the extensions schema
    and pe.extnamespace::regnamespace::text = 'public')
union all

(
select
    'unsupported_reg_types' as "name!",
    'Unsupported reg types' as "title!",
    'WARN' as "level!",
    'EXTERNAL' as "facing!",
    array['SECURITY'] as "categories!",
    'Identifies columns using unsupported reg* types outside pg_catalog schema, which prevents database upgrades using pg_upgrade.' as "description!",
    format(
        'Table \`%s.%s\` has a column \`%s\` with unsupported reg* type \`%s\`.',
        n.nspname,
        c.relname,
        a.attname,
        t.typname
    ) as "detail!",
    'https://supabase.com/docs/guides/database/database-linter?lint=unsupported_reg_types' as "remediation!",
    jsonb_build_object(
        'schema', n.nspname,
        'name', c.relname,
        'column', a.attname,
        'type', 'table'
    ) as "metadata!",
    format(
        'unsupported_reg_types_%s_%s_%s',
        n.nspname,
        c.relname,
        a.attname
    ) AS cache_key
from
    pg_catalog.pg_attribute a
    join pg_catalog.pg_class c
        on a.attrelid = c.oid
    join pg_catalog.pg_namespace n
        on c.relnamespace = n.oid
    join pg_catalog.pg_type t
        on a.atttypid = t.oid
    join pg_catalog.pg_namespace tn
        on t.typnamespace = tn.oid
where
    tn.nspname = 'pg_catalog'
    and t.typname in ('regcollation', 'regconfig', 'regdictionary', 'regnamespace', 'regoper', 'regoperator', 'regproc', 'regprocedure')
    and n.nspname not in ('pg_catalog', 'information_schema', 'pgsodium'))
union all

(
with constants as (
    select current_setting('block_size')::numeric as bs, 23 as hdr, 4 as ma
),

bloat_info as (
    select
        ma,
        bs,
        schemaname,
        tablename,
        (datawidth + (hdr + ma - (case when hdr % ma = 0 then ma else hdr % ma end)))::numeric as datahdr,
        (maxfracsum * (nullhdr + ma - (case when nullhdr % ma = 0 then ma else nullhdr % ma end))) as nullhdr2
    from (
        select
            schemaname,
            tablename,
            hdr,
            ma,
            bs,
            sum((1 - null_frac) * avg_width) as datawidth,
            max(null_frac) as maxfracsum,
            hdr + (
                select 1 + count(*) / 8
                from pg_stats s2
                where
                    null_frac <> 0
                    and s2.schemaname = s.schemaname
                    and s2.tablename = s.tablename
            ) as nullhdr
        from pg_stats s, constants
        group by 1, 2, 3, 4, 5
    ) as foo
),

table_bloat as (
    select
        schemaname,
        tablename,
        cc.relpages,
        bs,
        ceil((cc.reltuples * ((datahdr + ma -
          (case when datahdr % ma = 0 then ma else datahdr % ma end)) + nullhdr2 + 4)) / (bs - 20::float)) as otta
    from
        bloat_info
        join pg_class cc
            on cc.relname = bloat_info.tablename
        join pg_namespace nn
            on cc.relnamespace = nn.oid
            and nn.nspname = bloat_info.schemaname
            and nn.nspname <> 'information_schema'
        where
            cc.relkind = 'r'
            and cc.relam = (select oid from pg_am where amname = 'heap')
),

bloat_data as (
    select
        'table' as type,
        schemaname,
        tablename as object_name,
        round(case when otta = 0 then 0.0 else table_bloat.relpages / otta::numeric end, 1) as bloat,
        case when relpages < otta then 0 else (bs * (table_bloat.relpages - otta)::bigint)::bigint end as raw_waste
    from
        table_bloat
)

select
    'table_bloat' as "name!",
    'Table Bloat' as "title!",
    'INFO' as "level!",
    'EXTERNAL' as "facing!",
    array['PERFORMANCE'] as "categories!",
    'Detects if a table has excess bloat and may benefit from maintenance operations like vacuum full or cluster.' as "description!",
    format(
        'Table `%s`.`%s` has excessive bloat',
        bloat_data.schemaname,
        bloat_data.object_name
    ) as "detail!",
    'Consider running vacuum full (WARNING: incurs downtime) and tweaking autovacuum settings to reduce bloat.' as "remediation!",
    jsonb_build_object(
        'schema', bloat_data.schemaname,
        'name', bloat_data.object_name,
        'type', bloat_data.type
    ) as "metadata!",
    format(
        'table_bloat_%s_%s',
        bloat_data.schemaname,
        bloat_data.object_name
    ) as "cache_key!"
from
    bloat_data
where
    bloat > 70.0
    and raw_waste > (20 * 1024 * 1024) -- filter for waste > 200 MB
order by
    schemaname,
    object_name)
union all

(
select
    'extension_versions_outdated' as "name!",
    'Extension Versions Outdated' as "title!",
    'WARN' as "level!",
    'EXTERNAL' as "facing!",
    array['SECURITY'] as "categories!",
    'Detects extensions that are not using the default (recommended) version.' as "description!",
    format(
        'Extension `%s` is using version `%s` but version `%s` is available. Using outdated extension versions may expose the database to security vulnerabilities.',
        ext.name,
        ext.installed_version,
        ext.default_version
    ) as "detail!",
    'https://supabase.com/docs/guides/database/database-linter?lint=0022_extension_versions_outdated' as "remediation!",
    jsonb_build_object(
        'extension_name', ext.name,
        'installed_version', ext.installed_version,
        'default_version', ext.default_version
    ) as "metadata!",
    format(
        'extension_versions_outdated_%s_%s',
        ext.name,
        ext.installed_version
    ) as "cache_key!"
from
    pg_catalog.pg_available_extensions ext
join
    -- ignore versions not in pg_available_extension_versions
    -- e.g. residue of pg_upgrade
    pg_catalog.pg_available_extension_versions extv
    on extv.name = ext.name and extv.installed
where
    ext.installed_version is not null
    and ext.default_version is not null
    and ext.installed_version != ext.default_version
order by
    ext.name)