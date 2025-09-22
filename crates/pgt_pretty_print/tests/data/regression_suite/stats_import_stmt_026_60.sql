SELECT pg_catalog.pg_restore_relation_stats(
        'schemaname', 'stats_import',
        'relname', 'part_parent_i',
        'relpages', 2::integer);
