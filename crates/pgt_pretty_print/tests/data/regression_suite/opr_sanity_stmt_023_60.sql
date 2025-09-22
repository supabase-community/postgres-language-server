SELECT p1.oid, p1.proname
FROM pg_proc as p1
WHERE p1.prorettype = 'anycompatiblerange'::regtype
  AND NOT
     'anycompatiblerange'::regtype = ANY (p1.proargtypes)
ORDER BY 2;
