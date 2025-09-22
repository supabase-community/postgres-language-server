SELECT p1.oid, p1.proname
FROM pg_proc as p1
WHERE p1.prorettype IN
    ('anyelement'::regtype, 'anyarray'::regtype, 'anynonarray'::regtype,
     'anyenum'::regtype)
  AND NOT
    ('anyelement'::regtype = ANY (p1.proargtypes) OR
     'anyarray'::regtype = ANY (p1.proargtypes) OR
     'anynonarray'::regtype = ANY (p1.proargtypes) OR
     'anyenum'::regtype = ANY (p1.proargtypes) OR
     'anyrange'::regtype = ANY (p1.proargtypes) OR
     'anymultirange'::regtype = ANY (p1.proargtypes))
ORDER BY 2;
