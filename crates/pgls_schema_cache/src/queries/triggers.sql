-- we need to join tables from the pg_catalog since "TRUNCATE" triggers are
-- not available in the information_schema.trigger table.
select
 t.tgname as "name!",
 c.relname as "table_name!",
 p.proname as "proc_name!",
 proc_ns.nspname as "proc_schema!",
 table_ns.nspname as "table_schema!",
 t.tgtype as "details_bitmask!"
from
 pg_catalog.pg_trigger t
left join pg_catalog.pg_proc p on t.tgfoid = p.oid
left join pg_catalog.pg_class c on t.tgrelid = c.oid
left join pg_catalog.pg_namespace table_ns on c.relnamespace = table_ns.oid
left join pg_catalog.pg_namespace proc_ns on p.pronamespace = proc_ns.oid
where
 t.tgisinternal = false and
 t.tgconstraint = 0;
