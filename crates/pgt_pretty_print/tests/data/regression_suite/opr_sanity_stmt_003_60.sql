SELECT p1.oid, p1.proname
FROM pg_proc AS p1
WHERE proretset AND prokind != 'f';
