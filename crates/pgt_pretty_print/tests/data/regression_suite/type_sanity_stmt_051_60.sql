SELECT c1.oid, c1.relname
FROM pg_class AS c1
WHERE c1.relnatts != (SELECT count(*) FROM pg_attribute AS a1
                      WHERE a1.attrelid = c1.oid AND a1.attnum > 0);
