select
  c.oid :: int8 as "id!",
  nc.nspname as schema,
  c.relname as name,
  c.relkind as table_kind,
  c.relrowsecurity as rls_enabled,
  c.relforcerowsecurity as rls_forced,
  case
    when c.relreplident = 'd' then 'DEFAULT'
    when c.relreplident = 'i' then 'INDEX'
    when c.relreplident = 'f' then 'FULL'
    else 'NOTHING'
  end as "replica_identity!",
  relation_size:: int8 as "bytes!",
  pg_size_pretty(relation_size) as "size!",
  pg_stat_get_live_tuples(c.oid) as "live_rows_estimate!",
  pg_stat_get_dead_tuples(c.oid) as "dead_rows_estimate!",
  obj_description(c.oid) as comment
from
  pg_namespace nc
  join pg_class c on nc.oid = c.relnamespace
  cross join lateral pg_total_relation_size(c.oid) relation_size
where
  c.relkind in ('r', 'p', 'v', 'm')
  and not pg_is_other_temp_schema(nc.oid)
  and (
    pg_has_role(c.relowner, 'USAGE')
    or has_table_privilege(
      c.oid,
      'SELECT, INSERT, UPDATE, DELETE, TRUNCATE, REFERENCES, TRIGGER'
    )
    or has_any_column_privilege(c.oid, 'SELECT, INSERT, UPDATE, REFERENCES')
  )
