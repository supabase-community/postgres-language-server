-- meta: name = authUsersExposed
-- meta: title = Exposed Auth Users
-- meta: severity = ERROR
-- meta: category = SECURITY
-- meta: description = Detects if auth.users is exposed to anon or authenticated roles via a view or materialized view in schemas exposed to PostgREST, potentially compromising user data security.
-- meta: remediation = https://supabase.com/docs/guides/database/database-linter?lint=0002_auth_users_exposed

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
