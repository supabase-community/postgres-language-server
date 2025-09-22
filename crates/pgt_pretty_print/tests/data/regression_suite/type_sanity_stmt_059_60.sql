SELECT r.rngtypid, r.rngsubtype, p.proname
FROM pg_range r JOIN pg_proc p ON p.oid = r.rngsubdiff
WHERE pronargs != 2
    OR proargtypes[0] != rngsubtype OR proargtypes[1] != rngsubtype
    OR prorettype != 'pg_catalog.float8'::regtype;
