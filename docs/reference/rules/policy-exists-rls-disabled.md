# policyExistsRlsDisabled

**Diagnostic Category: `splinter/security/policyExistsRlsDisabled`**

**Severity**: Error

## Description

Detects cases where row level security (RLS) policies have been created, but RLS has not been enabled for the underlying table.

## Remediation

https://supabase.com/docs/guides/database/database-linter?lint=0007_policy_exists_rls_disabled

## SQL Query

```sql
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
```

## How to configure

```json
{
  "splinter": {
    "rules": {
      "security": {
        "policyExistsRlsDisabled": "error"
      }
    }
  }
}
```
