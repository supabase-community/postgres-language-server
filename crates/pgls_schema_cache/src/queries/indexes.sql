SELECT
    c.oid::bigint as "id!",
    n.nspname as "schema!",
    c.relname as "name!",
    t.relname as "table_name!"
FROM pg_catalog.pg_class c
JOIN pg_catalog.pg_namespace n ON n.oid = c.relnamespace
JOIN pg_catalog.pg_index i ON i.indexrelid = c.oid
JOIN pg_catalog.pg_class t ON t.oid = i.indrelid
WHERE c.relkind = 'i'
  AND n.nspname NOT IN ('pg_catalog', 'information_schema', 'pg_toast')
