SELECT c.relname, COUNT(*) AS num_stats
FROM pg_class AS c
JOIN pg_statistic s ON s.starelid = c.oid
WHERE c.relnamespace = 'stats_import'::regnamespace
AND c.relname IN ('test', 'test_clone', 'is_odd', 'is_odd_clone')
GROUP BY c.relname
ORDER BY c.relname;
