SELECT t1.oid, t1.typname
FROM pg_type as t1
WHERE t1.typtype not in ('p') AND t1.typname NOT LIKE E'\\_%'
    AND NOT EXISTS
    (SELECT 1 FROM pg_type as t2
     WHERE t2.typname = ('_' || t1.typname)::name AND
           t2.typelem = t1.oid and t1.typarray = t2.oid)
ORDER BY t1.oid;
