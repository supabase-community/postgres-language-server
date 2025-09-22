SELECT r.rngtypid, r.rngsubtype, p.proname
FROM pg_range r JOIN pg_proc p ON p.oid = r.rngcanonical
WHERE pronargs != 1 OR proargtypes[0] != rngtypid OR prorettype != rngtypid;
