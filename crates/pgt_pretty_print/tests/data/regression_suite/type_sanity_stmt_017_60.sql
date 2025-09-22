SELECT DISTINCT typtype, typoutput
FROM pg_type AS t1
WHERE t1.typtype not in ('b', 'd', 'p')
ORDER BY 1;
