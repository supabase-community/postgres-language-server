SELECT t1.oid, t1.typname, p1.oid, p1.proname
FROM pg_type AS t1, pg_proc AS p1
WHERE t1.typmodout = p1.oid AND p1.provolatile NOT IN ('i', 's');
