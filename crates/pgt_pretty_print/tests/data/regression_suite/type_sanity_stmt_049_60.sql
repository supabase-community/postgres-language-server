SELECT a1.attrelid, a1.attname
FROM pg_attribute as a1
WHERE a1.attrelid = 0 OR a1.atttypid = 0 OR a1.attnum = 0 OR
    a1.attinhcount < 0 OR (a1.attinhcount = 0 AND NOT a1.attislocal);
