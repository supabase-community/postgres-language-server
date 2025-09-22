SELECT pg_restore_relation_stats(
        'schemaname', 'stats_import',
        'relname', 'test',
        'version', 150000::integer,
        'relpages', '-17'::integer,
        'reltuples', 400::real,
        'relallvisible', 4::integer,
        'relallfrozen', 2::integer);
