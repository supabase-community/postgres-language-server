SELECT p1.oid, p1.proname
FROM pg_proc as p1
WHERE p1.prorettype IN
    ('anycompatible'::regtype, 'anycompatiblearray'::regtype,
     'anycompatiblenonarray'::regtype)
  AND NOT
    ('anycompatible'::regtype = ANY (p1.proargtypes) OR
     'anycompatiblearray'::regtype = ANY (p1.proargtypes) OR
     'anycompatiblenonarray'::regtype = ANY (p1.proargtypes) OR
     'anycompatiblerange'::regtype = ANY (p1.proargtypes))
ORDER BY 2;
