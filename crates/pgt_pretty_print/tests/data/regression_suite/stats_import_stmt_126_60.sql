SELECT pg_restore_relation_stats(
        'schemaname', 'pg_temp',
        'relname', 'stats_temp',
        'relpages', '-19'::integer,
        'reltuples', 401::real,
        'relallvisible', 5::integer,
        'relallfrozen', 3::integer);
