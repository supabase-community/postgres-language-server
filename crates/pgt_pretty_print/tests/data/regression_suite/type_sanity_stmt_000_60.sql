SELECT t1.oid, t1.typname
FROM pg_type as t1
WHERE t1.typnamespace = 0 OR
    (t1.typlen <= 0 AND t1.typlen != -1 AND t1.typlen != -2) OR
    (t1.typtype not in ('b', 'c', 'd', 'e', 'm', 'p', 'r')) OR
    NOT t1.typisdefined OR
    (t1.typalign not in ('c', 's', 'i', 'd')) OR
    (t1.typstorage not in ('p', 'x', 'e', 'm'));
