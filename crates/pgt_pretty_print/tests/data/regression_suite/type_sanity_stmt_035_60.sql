SELECT t1.oid, t1.typname, t2.oid, t2.typname
FROM pg_type AS t1, pg_type AS t2
WHERE t1.typarray = t2.oid AND NOT (t1.typdelim = t2.typdelim);
