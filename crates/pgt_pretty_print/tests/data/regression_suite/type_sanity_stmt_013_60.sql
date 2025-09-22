SELECT DISTINCT typtype, typinput
FROM pg_type AS t1
WHERE t1.typtype not in ('b', 'p')
ORDER BY 1;
