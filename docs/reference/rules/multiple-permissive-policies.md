# multiplePermissivePolicies

**Diagnostic Category: `splinter/performance/multiplePermissivePolicies`**

**Severity**: Warning

## Description

Detects if multiple permissive row level security policies are present on a table for the same \`role\` and \`action\` (e.g. insert). Multiple permissive policies are suboptimal for performance as each policy must be executed for every relevant query.

[Learn More](https://supabase.com/docs/guides/database/database-linter?lint=0006_multiple_permissive_policies)

## SQL Query

```sql
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
```

## How to configure

```json
{
  "splinter": {
    "rules": {
      "performance": {
        "multiplePermissivePolicies": "error"
      }
    }
  }
}
```
