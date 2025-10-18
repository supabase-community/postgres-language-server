VACUUM;

SELECT relname, nspname
 FROM pg_class c LEFT JOIN pg_namespace n ON n.oid = relnamespace JOIN pg_attribute a ON (attrelid = c.oid AND attname = 'oid')
 WHERE relkind = 'r' and c.oid < 16384
     AND ((nspname ~ '^pg_') IS NOT FALSE)
     AND NOT EXISTS (SELECT 1 FROM pg_index i WHERE indrelid = c.oid
                     AND indkey[0] = a.attnum AND indnatts = 1
                     AND indisunique AND indimmediate);

SELECT relname, relkind
  FROM pg_class
 WHERE relkind IN ('v', 'c', 'f', 'p', 'I')
       AND relfilenode <> 0;
