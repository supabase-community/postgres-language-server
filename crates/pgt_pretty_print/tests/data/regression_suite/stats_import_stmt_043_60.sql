SELECT pg_restore_relation_stats(
        'schemaname', 'stats_import',
        'relname', 'test',
        'relpages', '171'::integer,
        'nope', 10::integer);
