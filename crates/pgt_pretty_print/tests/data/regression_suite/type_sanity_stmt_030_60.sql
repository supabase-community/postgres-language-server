SELECT t1.oid, t1.typname, p1.oid, p1.proname
FROM pg_type AS t1, pg_proc AS p1
WHERE t1.typmodin = p1.oid AND NOT
    (p1.pronargs = 1 AND
     p1.proargtypes[0] = 'cstring[]'::regtype AND
     p1.prorettype = 'int4'::regtype AND NOT p1.proretset);
