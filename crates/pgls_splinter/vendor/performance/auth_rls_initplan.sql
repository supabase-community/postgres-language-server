-- meta: name = authRlsInitplan
-- meta: title = Auth RLS Initialization Plan
-- meta: severity = WARN
-- meta: category = PERFORMANCE
-- meta: description = Detects if calls to \`current_setting()\` and \`auth.<function>()\` in RLS policies are being unnecessarily re-evaluated for each row
-- meta: remediation = https://supabase.com/docs/guides/database/database-linter?lint=0003_auth_rls_initplan
-- meta: requires_supabase = true

(
with policies as (
    select
        nsp.nspname as schema_name,
        pb.tablename as table_name,
        pc.relrowsecurity as is_rls_active,
        polname as policy_name,
        polpermissive as is_permissive, -- if not, then restrictive
        (select array_agg(r::regrole) from unnest(polroles) as x(r)) as roles,
        case polcmd
            when 'r' then 'SELECT'
            when 'a' then 'INSERT'
            when 'w' then 'UPDATE'
            when 'd' then 'DELETE'
            when '*' then 'ALL'
        end as command,
        qual,
        with_check
    from
        pg_catalog.pg_policy pa
        join pg_catalog.pg_class pc
            on pa.polrelid = pc.oid
        join pg_catalog.pg_namespace nsp
            on pc.relnamespace = nsp.oid
        join pg_catalog.pg_policies pb
            on pc.relname = pb.tablename
            and nsp.nspname = pb.schemaname
            and pa.polname = pb.policyname
)
select
    'auth_rls_initplan' as "name!",
    'Auth RLS Initialization Plan' as "title!",
    'WARN' as "level!",
    'EXTERNAL' as "facing!",
    array['PERFORMANCE'] as "categories!",
    'Detects if calls to \`current_setting()\` and \`auth.<function>()\` in RLS policies are being unnecessarily re-evaluated for each row' as "description!",
    format(
        'Table \`%s.%s\` has a row level security policy \`%s\` that re-evaluates current_setting() or auth.<function>() for each row. This produces suboptimal query performance at scale. Resolve the issue by replacing \`auth.<function>()\` with \`(select auth.<function>())\`. See [docs](https://supabase.com/docs/guides/database/postgres/row-level-security#call-functions-with-select) for more info.',
        schema_name,
        table_name,
        policy_name
    ) as "detail!",
    'https://supabase.com/docs/guides/database/database-linter?lint=0003_auth_rls_initplan' as "remediation!",
    jsonb_build_object(
        'schema', schema_name,
        'name', table_name,
        'type', 'table'
    ) as "metadata!",
    format('auth_rls_init_plan_%s_%s_%s', schema_name, table_name, policy_name) as "cache_key!"
from
    policies
where
    is_rls_active
    -- NOTE: does not include realtime in support of monitoring policies on realtime.messages
    and schema_name not in (
        '_timescaledb_cache', '_timescaledb_catalog', '_timescaledb_config', '_timescaledb_internal', 'auth', 'cron', 'extensions', 'graphql', 'graphql_public', 'information_schema', 'net', 'pgmq', 'pgroonga', 'pgsodium', 'pgsodium_masks', 'pgtle', 'pgbouncer', 'pg_catalog', 'pgtle', 'repack', 'storage', 'supabase_functions', 'supabase_migrations', 'tiger', 'topology', 'vault'
    )
    and (
        -- Example: auth.uid()
        (
            qual like '%auth.uid()%'
            and lower(qual) not like '%select auth.uid()%'
        )
        or (
            qual like '%auth.jwt()%'
            and lower(qual) not like '%select auth.jwt()%'
        )
        or (
            qual like '%auth.role()%'
            and lower(qual) not like '%select auth.role()%'
        )
        or (
            qual like '%auth.email()%'
            and lower(qual) not like '%select auth.email()%'
        )
        or (
            qual like '%current\_setting(%)%'
            and lower(qual) not like '%select current\_setting(%)%'
        )
        or (
            with_check like '%auth.uid()%'
            and lower(with_check) not like '%select auth.uid()%'
        )
        or (
            with_check like '%auth.jwt()%'
            and lower(with_check) not like '%select auth.jwt()%'
        )
        or (
            with_check like '%auth.role()%'
            and lower(with_check) not like '%select auth.role()%'
        )
        or (
            with_check like '%auth.email()%'
            and lower(with_check) not like '%select auth.email()%'
        )
        or (
            with_check like '%current\_setting(%)%'
            and lower(with_check) not like '%select current\_setting(%)%'
        )
    ))
