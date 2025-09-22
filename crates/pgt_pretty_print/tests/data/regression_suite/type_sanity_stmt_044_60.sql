SELECT c1.oid, c1.relname
FROM pg_class as c1
WHERE relkind NOT IN ('r', 'i', 'S', 't', 'v', 'm', 'c', 'f', 'p', 'I') OR
    relpersistence NOT IN ('p', 'u', 't') OR
    relreplident NOT IN ('d', 'n', 'f', 'i');
