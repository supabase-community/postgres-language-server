SELECT o1.oid, o1.oprname, p2.oid, p2.proname
FROM pg_operator AS o1, pg_proc AS p2
WHERE o1.oprjoin = p2.oid AND
    (o1.oprkind != 'b' OR o1.oprresult != 'bool'::regtype OR
     p2.prorettype != 'float8'::regtype OR p2.proretset OR
     p2.pronargs != 5 OR
     p2.proargtypes[0] != 'internal'::regtype OR
     p2.proargtypes[1] != 'oid'::regtype OR
     p2.proargtypes[2] != 'internal'::regtype OR
     p2.proargtypes[3] != 'int2'::regtype OR
     p2.proargtypes[4] != 'internal'::regtype);
