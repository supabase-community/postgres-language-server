# rlsPolicyAlwaysTrue

**Diagnostic Category: `splinter/security/rlsPolicyAlwaysTrue`**

**Severity**: Warning

> [!NOTE]
> This rule requires a Supabase database/project and will be automatically skipped if not detected.

## Description

Detects RLS policies that use overly permissive expressions like USING (true) or WITH CHECK (true) for UPDATE, DELETE, or INSERT operations. SELECT policies with USING (true) are intentionally excluded as this pattern is often used deliberately for public read access.

## Remediation

https://supabase.com/docs/guides/database/database-linter?lint=0024_permissive_rls_policy

## SQL Query

```sql
(
with policies as (
    select
        nsp.nspname as schema_name,
        pb.tablename as table_name,
        pc.relrowsecurity as is_rls_active,
        pa.polname as policy_name,
        pa.polpermissive as is_permissive,
        pa.polroles as role_oids,
        (select array_agg(r::regrole::text) from unnest(pa.polroles) as x(r)) as roles,
        case pa.polcmd
            when 'r' then 'SELECT'
            when 'a' then 'INSERT'
            when 'w' then 'UPDATE'
            when 'd' then 'DELETE'
            when '*' then 'ALL'
        end as command,
        pb.qual,
        pb.with_check,
        -- Normalize expressions by removing whitespace and lowercasing
        replace(replace(replace(lower(coalesce(pb.qual, '')), ' ', ''), E'\n', ''), E'\t', '') as normalized_qual,
        replace(replace(replace(lower(coalesce(pb.with_check, '')), ' ', ''), E'\n', ''), E'\t', '') as normalized_with_check
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
    where
        pc.relkind = 'r' -- regular tables
        and nsp.nspname not in (
            '_timescaledb_cache', '_timescaledb_catalog', '_timescaledb_config', '_timescaledb_internal', 'auth', 'cron', 'extensions', 'graphql', 'graphql_public', 'information_schema', 'net', 'pgmq', 'pgroonga', 'pgsodium', 'pgsodium_masks', 'pgtle', 'pgbouncer', 'pg_catalog', 'pgtle', 'realtime', 'repack', 'storage', 'supabase_functions', 'supabase_migrations', 'tiger', 'topology', 'vault'
        )
),
permissive_patterns as (
    select
        p.*,
        -- Check for always-true USING clause patterns
        -- Note: SELECT with (true) is often intentional and documented, so we only flag UPDATE/DELETE
        case when (
            command in ('UPDATE', 'DELETE', 'ALL')
            and (
                normalized_qual in ('true', '(true)', '1=1', '(1=1)')
                -- Empty or null qual on permissive policy means allow all
                or (qual is null and is_permissive)
            )
        ) then true else false end as has_permissive_using,
        -- Check for always-true WITH CHECK clause patterns
        case when (
            normalized_with_check in ('true', '(true)', '1=1', '(1=1)')
            -- Empty with_check on INSERT means allow all (INSERT has no USING to fall back on)
            or (with_check is null and is_permissive and command = 'INSERT')
            -- Empty with_check on UPDATE/ALL with permissive USING means allow all writes
            or (with_check is null and is_permissive and command in ('UPDATE', 'ALL')
                and normalized_qual in ('true', '(true)', '1=1', '(1=1)'))
        ) then true else false end as has_permissive_with_check
    from
        policies p
    where
        -- Only check tables with RLS enabled (otherwise it's a different lint)
        is_rls_active
        -- Only check permissive policies (restrictive policies with true are less dangerous)
        and is_permissive
        -- Only flag policies that apply to anon or authenticated roles (or public/all roles)
        and (
            role_oids = array[0::oid] -- public (all roles)
            or exists (
                select 1
                from unnest(role_oids) as r
                where r::regrole::text in ('anon', 'authenticated')
            )
        )
)
select
    'rls_policy_always_true' as "name!",
    'RLS Policy Always True' as "title!",
    'WARN' as "level!",
    'EXTERNAL' as "facing!",
    array['SECURITY'] as "categories!",
    'Detects RLS policies that use overly permissive expressions like USING (true) or WITH CHECK (true) for UPDATE, DELETE, or INSERT operations. SELECT policies with USING (true) are intentionally excluded as this pattern is often used deliberately for public read access.' as "description!",
    format(
        'Table `%s.%s` has an RLS policy `%s` for `%s` that allows unrestricted access%s. This effectively bypasses row-level security for %s.',
        schema_name,
        table_name,
        policy_name,
        command,
        case
            when has_permissive_using and has_permissive_with_check then ' (both USING and WITH CHECK are always true)'
            when has_permissive_using then ' (USING clause is always true)'
            when has_permissive_with_check then ' (WITH CHECK clause is always true)'
            else ''
        end,
        array_to_string(roles, ', ')
    ) as "detail!",
    'https://supabase.com/docs/guides/database/database-linter?lint=0024_permissive_rls_policy' as "remediation!",
    jsonb_build_object(
        'schema', schema_name,
        'name', table_name,
        'type', 'table',
        'policy_name', policy_name,
        'command', command,
        'roles', roles,
        'qual', qual,
        'with_check', with_check,
        'permissive_using', has_permissive_using,
        'permissive_with_check', has_permissive_with_check
    ) as "metadata!",
    format(
        'rls_policy_always_true_%s_%s_%s',
        schema_name,
        table_name,
        policy_name
    ) as "cache_key!"
from
    permissive_patterns
where
    has_permissive_using or has_permissive_with_check
order by
    schema_name,
    table_name,
    policy_name)
```

## How to configure

```json
{
  "splinter": {
    "rules": {
      "security": {
        "rlsPolicyAlwaysTrue": "error"
      }
    }
  }
}
```
