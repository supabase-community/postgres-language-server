SELECT *
FROM pg_depend as d1
WHERE refclassid = 0 OR refobjid = 0 OR
      classid = 0 OR objid = 0 OR
      deptype NOT IN ('a', 'e', 'i', 'n', 'x', 'P', 'S');

SELECT *
FROM pg_shdepend as d1
WHERE refclassid = 0 OR refobjid = 0 OR
      classid = 0 OR objid = 0 OR
      deptype NOT IN ('a', 'i', 'o', 'r', 't');

SELECT relname, attname, atttypid::regtype
FROM pg_class c JOIN pg_attribute a ON c.oid = attrelid
WHERE c.oid < 16384 AND
      reltoastrelid = 0 AND
      relkind = 'r' AND
      attstorage != 'p'
ORDER BY 1, 2;

SELECT relname
FROM pg_class
WHERE relnamespace = 'pg_catalog'::regnamespace AND relkind = 'r'
      AND pg_class.oid NOT IN (SELECT indrelid FROM pg_index WHERE indisprimary)
ORDER BY 1;

SELECT relname
FROM pg_class c JOIN pg_index i ON c.oid = i.indexrelid
WHERE relnamespace = 'pg_catalog'::regnamespace AND relkind = 'i'
      AND i.indisunique
      AND c.oid NOT IN (SELECT conindid FROM pg_constraint)
ORDER BY 1;
