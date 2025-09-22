SELECT p.oid, p.proname, c.oid, c.conname
FROM pg_proc p, pg_conversion c
WHERE p.oid = c.conproc AND
    (p.prorettype != 'int4'::regtype OR p.proretset OR
     p.pronargs != 6 OR
     p.proargtypes[0] != 'int4'::regtype OR
     p.proargtypes[1] != 'int4'::regtype OR
     p.proargtypes[2] != 'cstring'::regtype OR
     p.proargtypes[3] != 'internal'::regtype OR
     p.proargtypes[4] != 'int4'::regtype OR
     p.proargtypes[5] != 'bool'::regtype);
