SELECT relpages, reltuples, relallvisible, relallfrozen
FROM pg_class
WHERE oid = 'stats_import.test_i'::regclass;
