SELECT t1.oid, t1.typname
FROM pg_type as t1
WHERE (t1.typinput = 0 OR t1.typoutput = 0);
