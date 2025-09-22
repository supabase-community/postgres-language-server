SELECT p1.oid, p1.proname
FROM pg_proc as p1
WHERE p1.prorettype IN ('anyrange'::regtype, 'anymultirange'::regtype)
  AND NOT
    ('anyrange'::regtype = ANY (p1.proargtypes) OR
      'anymultirange'::regtype = ANY (p1.proargtypes))
ORDER BY 2;
