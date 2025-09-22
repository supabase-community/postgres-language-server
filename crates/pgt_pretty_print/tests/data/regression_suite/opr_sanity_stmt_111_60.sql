SELECT a1.oid, a1.amname, p1.oid, p1.proname
FROM pg_am AS a1, pg_proc AS p1
WHERE p1.oid = a1.amhandler AND a1.amtype = 't' AND
    (p1.prorettype != 'table_am_handler'::regtype
     OR p1.proretset
     OR p1.pronargs != 1
     OR p1.proargtypes[0] != 'internal'::regtype);
