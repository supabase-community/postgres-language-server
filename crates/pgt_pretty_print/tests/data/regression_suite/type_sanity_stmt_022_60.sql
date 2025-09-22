SELECT t1.oid, t1.typname, p1.oid, p1.proname, p2.oid, p2.proname
FROM pg_type AS t1, pg_proc AS p1, pg_proc AS p2
WHERE t1.typinput = p1.oid AND t1.typreceive = p2.oid AND
    p1.pronargs != p2.pronargs;
