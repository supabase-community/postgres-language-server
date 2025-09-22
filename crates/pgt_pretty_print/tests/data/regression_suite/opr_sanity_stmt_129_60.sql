SELECT relname, attname, attcollation
FROM pg_class c, pg_attribute a
WHERE c.oid = attrelid AND c.oid < 16384 AND
    c.relkind != 'v' AND  -- we don't care about columns in views
    attcollation != 0 AND
    attcollation != (SELECT oid FROM pg_collation WHERE collname = 'C');
