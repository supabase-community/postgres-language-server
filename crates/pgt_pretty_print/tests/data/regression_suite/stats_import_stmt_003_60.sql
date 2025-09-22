SELECT
    pg_catalog.pg_restore_relation_stats(
        'schemaname', 'stats_import',
        'relname', 'test',
        'relpages', 18::integer,
        'reltuples', 21::real,
        'relallvisible', 24::integer,
	'relallfrozen', 27::integer);
