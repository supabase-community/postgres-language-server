
(
select
    'auth_users_exposed' as "name!",
    'Exposed Auth Users' as "title!",
    'ERROR' as "level!",
    'EXTERNAL' as "facing!",
    array['SECURITY'] as "categories!",
    'Detects if auth.users is exposed to anon or authenticated roles via a view or materialized view in schemas exposed to PostgREST, potentially compromising user data security.' as "description!",
    format(
        'View/Materialized View "%s" in the public schema may expose \`auth.users\` data to anon or authenticated roles.',
        c.relname
    ) as "detail!",
    'https://supabase.com/docs/guides/database/database-linter?lint=0002_auth_users_exposed' as "remediation!",
    jsonb_build_object(
        'schema', n.nspname,
        'name', c.relname,
        'type', 'view',
        'exposed_to', array_remove(array_agg(DISTINCT case when pg_catalog.has_table_privilege('anon', c.oid, 'SELECT') then 'anon' when pg_catalog.has_table_privilege('authenticated', c.oid, 'SELECT') then 'authenticated' end), null)
    ) as "metadata!",
    format('auth_users_exposed_%s_%s', n.nspname, c.relname) as "cache_key!"
from
    -- Identify the oid for auth.users
    pg_catalog.pg_class auth_users_pg_class
    join pg_catalog.pg_namespace auth_users_pg_namespace
        on auth_users_pg_class.relnamespace = auth_users_pg_namespace.oid
        and auth_users_pg_class.relname = 'users'
        and auth_users_pg_namespace.nspname = 'auth'
    -- Depends on auth.users
    join pg_catalog.pg_depend d
        on d.refobjid = auth_users_pg_class.oid
    join pg_catalog.pg_rewrite r
        on r.oid = d.objid
    join pg_catalog.pg_class c
        on c.oid = r.ev_class
    join pg_catalog.pg_namespace n
        on n.oid = c.relnamespace
    join pg_catalog.pg_class pg_class_auth_users
        on d.refobjid = pg_class_auth_users.oid
where
    d.deptype = 'n'
    and (
      pg_catalog.has_table_privilege('anon', c.oid, 'SELECT')
      or pg_catalog.has_table_privilege('authenticated', c.oid, 'SELECT')
    )
    and n.nspname = any(array(select trim(unnest(string_to_array(current_setting('pgrst.db_schemas', 't'), ',')))))
    -- Exclude self
    and c.relname <> '0002_auth_users_exposed'
    -- There are 3 insecure configurations
    and
    (
        -- Materialized views don't support RLS so this is insecure by default
        (c.relkind in ('m')) -- m for materialized view
        or
        -- Standard View, accessible to anon or authenticated that is security_definer
        (
            c.relkind = 'v' -- v for view
            -- Exclude security invoker views
            and not (
                lower(coalesce(c.reloptions::text,'{}'))::text[]
                && array[
                    'security_invoker=1',
                    'security_invoker=true',
                    'security_invoker=yes',
                    'security_invoker=on'
                ]
            )
        )
        or
        -- Standard View, security invoker, but no RLS enabled on auth.users
        (
            c.relkind in ('v') -- v for view
            -- is security invoker
            and (
                lower(coalesce(c.reloptions::text,'{}'))::text[]
                && array[
                    'security_invoker=1',
                    'security_invoker=true',
                    'security_invoker=yes',
                    'security_invoker=on'
                ]
            )
            and not pg_class_auth_users.relrowsecurity
        )
    )
group by
    n.nspname,
    c.relname,
    c.oid)
union all

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
union all

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
union all

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
union all

(
with policies as (
    select
        nsp.nspname as schema_name,
        pb.tablename as table_name,
        polname as policy_name,
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
    'rls_references_user_metadata' as "name!",
    'RLS references user metadata' as "title!",
    'ERROR' as "level!",
    'EXTERNAL' as "facing!",
    array['SECURITY'] as "categories!",
    'Detects when Supabase Auth user_metadata is referenced insecurely in a row level security (RLS) policy.' as "description!",
    format(
        'Table \`%s.%s\` has a row level security policy \`%s\` that references Supabase Auth \`user_metadata\`. \`user_metadata\` is editable by end users and should never be used in a security context.',
        schema_name,
        table_name,
        policy_name
    ) as "detail!",
    'https://supabase.com/docs/guides/database/database-linter?lint=0015_rls_references_user_metadata' as "remediation!",
    jsonb_build_object(
        'schema', schema_name,
        'name', table_name,
        'type', 'table'
    ) as "metadata!",
    format('rls_references_user_metadata_%s_%s_%s', schema_name, table_name, policy_name) as "cache_key!"
from
    policies
where
    schema_name not in (
        '_timescaledb_cache', '_timescaledb_catalog', '_timescaledb_config', '_timescaledb_internal', 'auth', 'cron', 'extensions', 'graphql', 'graphql_public', 'information_schema', 'net', 'pgmq', 'pgroonga', 'pgsodium', 'pgsodium_masks', 'pgtle', 'pgbouncer', 'pg_catalog', 'pgtle', 'realtime', 'repack', 'storage', 'supabase_functions', 'supabase_migrations', 'tiger', 'topology', 'vault'
    )
    and (
        -- Example: auth.jwt() -> 'user_metadata'
        -- False positives are possible, but it isn't practical to string match
        -- If false positive rate is too high, this expression can iterate
        qual like '%auth.jwt()%user_metadata%'
        or qual like '%current_setting(%request.jwt.claims%)%user_metadata%'
        or with_check like '%auth.jwt()%user_metadata%'
        or with_check like '%current_setting(%request.jwt.claims%)%user_metadata%'
    ))
union all

(
select
    'materialized_view_in_api' as "name!",
    'Materialized View in API' as "title!",
    'WARN' as "level!",
    'EXTERNAL' as "facing!",
    array['SECURITY'] as "categories!",
    'Detects materialized views that are accessible over the Data APIs.' as "description!",
    format(
        'Materialized view \`%s.%s\` is selectable by anon or authenticated roles',
        n.nspname,
        c.relname
    ) as "detail!",
    'https://supabase.com/docs/guides/database/database-linter?lint=0016_materialized_view_in_api' as "remediation!",
    jsonb_build_object(
        'schema', n.nspname,
        'name', c.relname,
        'type', 'materialized view'
    ) as "metadata!",
    format(
        'materialized_view_in_api_%s_%s',
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
    c.relkind = 'm'
    and (
        pg_catalog.has_table_privilege('anon', c.oid, 'SELECT')
        or pg_catalog.has_table_privilege('authenticated', c.oid, 'SELECT')
    )
    and n.nspname = any(array(select trim(unnest(string_to_array(current_setting('pgrst.db_schemas', 't'), ',')))))
    and n.nspname not in (
        '_timescaledb_cache', '_timescaledb_catalog', '_timescaledb_config', '_timescaledb_internal', 'auth', 'cron', 'extensions', 'graphql', 'graphql_public', 'information_schema', 'net', 'pgmq', 'pgroonga', 'pgsodium', 'pgsodium_masks', 'pgtle', 'pgbouncer', 'pg_catalog', 'pgtle', 'realtime', 'repack', 'storage', 'supabase_functions', 'supabase_migrations', 'tiger', 'topology', 'vault'
    )
    and dep.objid is null)
union all

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
union all

(
select
    'insecure_queue_exposed_in_api' as "name!",
    'Insecure Queue Exposed in API' as "title!",
    'ERROR' as "level!",
    'EXTERNAL' as "facing!",
    array['SECURITY'] as "categories!",
    'Detects cases where an insecure Queue is exposed over Data APIs' as "description!",
    format(
        'Table \`%s.%s\` is public, but RLS has not been enabled.',
        n.nspname,
        c.relname
    ) as "detail!",
    'https://supabase.com/docs/guides/database/database-linter?lint=0019_insecure_queue_exposed_in_api' as "remediation!",
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
    c.relkind in ('r', 'I') -- regular or partitioned tables
    and not c.relrowsecurity -- RLS is disabled
    and (
        pg_catalog.has_table_privilege('anon', c.oid, 'SELECT')
        or pg_catalog.has_table_privilege('authenticated', c.oid, 'SELECT')
    )
    and n.nspname = 'pgmq' -- tables in the pgmq schema
    and c.relname like 'q_%' -- only queue tables
    -- Constant requirements
    and 'pgmq_public' = any(array(select trim(unnest(string_to_array(current_setting('pgrst.db_schemas', 't'), ','))))))
union all

(
select
    'fkey_to_auth_unique' as "name!",
    'Foreign Key to Auth Unique Constraint' as "title!",
    'ERROR' as "level!",
    'EXTERNAL' as "facing!",
    array['SECURITY'] as "categories!",
    'Detects user defined foreign keys to unique constraints in the auth schema.' as "description!",
    format(
        'Table `%s`.`%s` has a foreign key `%s` referencing an auth unique constraint',
        n.nspname, -- referencing schema
        c_rel.relname, -- referencing table
        c.conname -- fkey name
    ) as "detail!",
    'Drop the foreign key constraint that references the auth schema.' as "remediation!",
    jsonb_build_object(
        'schema', n.nspname,
        'name', c_rel.relname,
        'foreign_key', c.conname
    ) as "metadata!",
    format(
        'fkey_to_auth_unique_%s_%s_%s',
        n.nspname, -- referencing schema
        c_rel.relname, -- referencing table
        c.conname
    ) as "cache_key!"
from
    pg_catalog.pg_constraint c
    join pg_catalog.pg_class c_rel
        on c.conrelid = c_rel.oid
    join pg_catalog.pg_namespace n
        on c_rel.relnamespace = n.oid
    join pg_catalog.pg_class ref_rel
        on c.confrelid = ref_rel.oid
    join pg_catalog.pg_namespace cn
        on ref_rel.relnamespace = cn.oid
    join pg_catalog.pg_index i
        on c.conindid = i.indexrelid
where c.contype = 'f'
    and cn.nspname = 'auth'
    and i.indisunique
    and not i.indisprimary)
