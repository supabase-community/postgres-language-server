SELECT t1.oid, t1.typname, p1.oid, p1.proname
FROM pg_type AS t1, pg_proc AS p1
WHERE t1.typinput = p1.oid AND NOT
    ((p1.pronargs = 1 AND p1.proargtypes[0] = 'cstring'::regtype) OR
     (p1.pronargs = 2 AND p1.proargtypes[0] = 'cstring'::regtype AND
      p1.proargtypes[1] = 'oid'::regtype) OR
     (p1.pronargs = 3 AND p1.proargtypes[0] = 'cstring'::regtype AND
      p1.proargtypes[1] = 'oid'::regtype AND
      p1.proargtypes[2] = 'int4'::regtype));
