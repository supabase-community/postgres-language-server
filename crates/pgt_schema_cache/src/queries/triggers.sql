-- we need to join tables from the pg_catalog since "TRUNCATE" triggers are 
-- not available in the information_schema.trigger table.
select 
  t.tgname as "name!",
  c.relname as "table_name!",
  p.proname as "proc_name!",
  n.nspname as "schema_name!",
  t.tgtype as "details_bitmask!"
from 
  pg_catalog.pg_trigger t 
  left join pg_catalog.pg_proc p on t.tgfoid = p.oid
  left join pg_catalog.pg_class c on t.tgrelid = c.oid
  left join pg_catalog.pg_namespace n on c.relnamespace = n.oid
where 
  -- triggers enforcing constraints (e.g. unique fields) should not be included.
  t.tgisinternal = false and 
  t.tgconstraint = 0;
