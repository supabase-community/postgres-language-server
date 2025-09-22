SELECT t1.oid, t1.typname, t2.oid, t2.typname
FROM pg_type AS t1 LEFT JOIN pg_type AS t2 ON t1.typbasetype = t2.oid
WHERE t1.typtype = 'd' AND t1.typsend IS DISTINCT FROM t2.typsend;
