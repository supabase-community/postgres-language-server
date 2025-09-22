SELECT t1.oid, t1.typname, p1.oid, p1.proname
FROM pg_type AS t1, pg_proc AS p1
WHERE t1.typsend = p1.oid AND t1.typtype in ('b', 'p') AND NOT
    (p1.pronargs = 1 AND
     (p1.proargtypes[0] = t1.oid OR
      (p1.oid = 'array_send'::regproc AND
       t1.typelem != 0 AND t1.typlen = -1)))
ORDER BY 1;
