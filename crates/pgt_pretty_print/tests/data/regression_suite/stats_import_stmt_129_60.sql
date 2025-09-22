SELECT tablename, null_frac
FROM pg_stats
WHERE schemaname like 'pg_temp%'
AND tablename = 'stats_temp'
AND inherited = false
AND attname = 'i';
