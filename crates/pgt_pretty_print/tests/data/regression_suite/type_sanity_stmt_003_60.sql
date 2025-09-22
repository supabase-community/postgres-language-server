SELECT t1.oid, t1.typname
FROM pg_type as t1
WHERE (t1.typtype = 'c' AND t1.typrelid = 0) OR
    (t1.typtype != 'c' AND t1.typrelid != 0);
