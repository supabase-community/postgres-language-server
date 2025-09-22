SELECT pg_restore_relation_stats(
        'schemaname', 'stats_import',
        'relname', 'test',
        'reltuples', '500'::real);
