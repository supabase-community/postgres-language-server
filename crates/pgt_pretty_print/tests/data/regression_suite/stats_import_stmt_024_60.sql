SELECT relpages
FROM pg_class
WHERE oid = 'stats_import.part_parent'::regclass;
