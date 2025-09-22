SELECT relname
FROM pg_class c JOIN pg_index i ON c.oid = i.indexrelid
WHERE relnamespace = 'pg_catalog'::regnamespace AND relkind = 'i'
      AND i.indisunique
      AND c.oid NOT IN (SELECT conindid FROM pg_constraint)
ORDER BY 1;
