SELECT relname, relpages, reltuples, relallvisible, relallfrozen
FROM pg_class
WHERE oid = 'pg_temp.stats_temp'::regclass
ORDER BY relname;
