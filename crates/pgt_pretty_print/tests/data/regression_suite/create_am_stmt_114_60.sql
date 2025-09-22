SELECT c.relname, a.amname FROM pg_class c, pg_am a
  WHERE c.relam = a.oid AND
        c.relname LIKE 'am_partitioned%'
UNION ALL
SELECT c.relname, 'default' FROM pg_class c
  WHERE c.relam = 0
        AND c.relname LIKE 'am_partitioned%' ORDER BY 1;
