SELECT relname, attname, atttypid::regtype
FROM pg_class c JOIN pg_attribute a ON c.oid = attrelid
WHERE c.oid < 16384 AND
      reltoastrelid = 0 AND
      relkind = 'r' AND
      attstorage != 'p'
ORDER BY 1, 2;
