SELECT
    c.oid::bigint as "id!",
    n.nspname as "schema!",
    c.relname as "name!"
FROM pg_catalog.pg_class c
JOIN pg_catalog.pg_namespace n ON n.oid = c.relnamespace
WHERE c.relkind = 'S'
  AND n.nspname NOT IN ('pg_catalog', 'information_schema', 'pg_toast')
