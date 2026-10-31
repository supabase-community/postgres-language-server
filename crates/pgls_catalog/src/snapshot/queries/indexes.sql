SELECT
    c.oid::bigint as "id!",
    n.nspname as "schema!",
    c.relname as "name!",
    t.relname as "table_name!",
    -- The columns `DROP COLUMN` drops the index for: those of its keys and `INCLUDE` list, and
    -- those its expressions and predicate depend on. The index of a constraint depends on the
    -- constraint instead of its key columns, so these come from `indkey`.
    array(
        SELECT a.attname::text
        FROM pg_catalog.pg_attribute a
        WHERE a.attrelid = i.indrelid
          AND a.attnum > 0
          AND (
            a.attnum = ANY (i.indkey)
            OR EXISTS (
              SELECT 1
              FROM pg_catalog.pg_depend d
              WHERE d.classid = 'pg_catalog.pg_class'::regclass
                AND d.objid = i.indexrelid
                AND d.refclassid = 'pg_catalog.pg_class'::regclass
                AND d.refobjid = i.indrelid
                AND d.refobjsubid = a.attnum
            )
          )
        ORDER BY a.attnum
    ) as "columns?"
FROM pg_catalog.pg_class c
JOIN pg_catalog.pg_namespace n ON n.oid = c.relnamespace
JOIN pg_catalog.pg_index i ON i.indexrelid = c.oid
JOIN pg_catalog.pg_class t ON t.oid = i.indrelid
-- i: indexes, I: partitioned indexes
WHERE c.relkind IN ('i', 'I')
  AND n.nspname NOT IN ('pg_catalog', 'information_schema', 'pg_toast')
