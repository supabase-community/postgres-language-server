SELECT p1.oid, p1.proname
FROM pg_proc as p1
WHERE (prosrc = '' OR prosrc = '-') AND prosqlbody IS NULL;
