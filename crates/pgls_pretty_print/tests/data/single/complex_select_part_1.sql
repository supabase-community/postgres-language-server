  select
    c.oid       as view_id,
    n.nspname   as view_schema,
    c.relname   as view_name,
    r.ev_action as view_definition
  from pg_class c
  join pg_namespace n on n.oid = c.relnamespace
  join pg_rewrite r on r.ev_class = c.oid
  where c.relkind in ('v', 'm')
