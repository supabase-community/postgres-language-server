SELECT a1.oid, a1.amname
FROM pg_am AS a1
WHERE a1.amhandler = 0;
