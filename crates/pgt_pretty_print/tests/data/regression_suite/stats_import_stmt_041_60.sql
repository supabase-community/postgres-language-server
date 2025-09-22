SELECT pg_restore_relation_stats(
        'schemaname', 'stats_import',
        'relname', 'test',
        'relpages', 'nope'::text,
        'reltuples', 400.0::real,
        'relallvisible', 4::integer,
        'relallfrozen', 3::integer);
