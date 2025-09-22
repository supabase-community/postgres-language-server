SELECT DISTINCT typtype, typreceive
FROM pg_type AS t1
WHERE t1.typtype not in ('b', 'p')
ORDER BY 1;
