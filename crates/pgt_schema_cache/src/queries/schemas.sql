select
  n.oid :: int8 as "id!",
  n.nspname as name,
  u.rolname as "owner!",
  obj_description(n.oid, 'pg_namespace') as "comment",

  coalesce((
    select array_agg(grantee::regrole::text)
    from aclexplode(n.nspacl)
    where privilege_type = 'USAGE'
    and grantee::regrole::text <> ''
    and grantee::regrole::text <> '-'
  ), ARRAY[]::text[]) as "allowed_users!",

  coalesce((
    select array_agg(grantee::regrole::text)
    from aclexplode(n.nspacl)
    where privilege_type = 'CREATE'
    and grantee::regrole::text <> ''
    and grantee::regrole::text <> '-'
  ), ARRAY[]::text[]) as "allowed_creators!",

  (select count(*) from pg_class c where c.relnamespace = n.oid and c.relkind = 'r') as "table_count!",
  (select count(*) from pg_class c where c.relnamespace = n.oid and c.relkind = 'v') as "view_count!",
  (select count(*) from pg_proc p where p.pronamespace = n.oid) as "function_count!",

  coalesce(
  (select pg_size_pretty(sum(pg_total_relation_size(c.oid)))
    from pg_class c 
    where c.relnamespace = n.oid and c.relkind in ('r', 'i', 'm')),
  '0 bytes'
) as "total_size!"
from
  pg_namespace n,
  pg_roles u
where
  n.nspowner = u.oid
  and (
    pg_has_role(n.nspowner, 'USAGE')
    or has_schema_privilege(n.oid, 'CREATE, USAGE')
  )
  and not pg_catalog.starts_with(n.nspname, 'pg_temp_')
  and not pg_catalog.starts_with(n.nspname, 'pg_toast_temp_');