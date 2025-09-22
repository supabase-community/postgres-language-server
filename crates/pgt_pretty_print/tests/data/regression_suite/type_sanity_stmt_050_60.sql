SELECT a1.attrelid, a1.attname, c1.oid, c1.relname
FROM pg_attribute AS a1, pg_class AS c1
WHERE a1.attrelid = c1.oid AND a1.attnum > c1.relnatts;
