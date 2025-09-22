SELECT r.rngtypid, r.rngsubtype, r.rngcollation, t.typcollation
FROM pg_range r JOIN pg_type t ON t.oid = r.rngsubtype
WHERE (rngcollation = 0) != (typcollation = 0);
