SELECT *
FROM pg_stats
WHERE schemaname = 'stats_import'
AND tablename = 'test'
AND inherited = false
AND attname = 'tags';
