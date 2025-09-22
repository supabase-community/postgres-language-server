SELECT c1.oid, c1.relname
FROM pg_class as c1
WHERE c1.relkind IN ('S', 'v', 'f', 'c', 'p') and
    c1.relam != 0;
