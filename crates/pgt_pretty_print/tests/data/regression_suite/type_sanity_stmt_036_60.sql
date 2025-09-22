SELECT t1.oid, t1.typname, t1.typalign, t2.typname, t2.typalign
FROM pg_type AS t1, pg_type AS t2
WHERE t1.typarray = t2.oid AND
    t2.typalign != (CASE WHEN t1.typalign = 'd' THEN 'd'::"char"
                         ELSE 'i'::"char" END);
