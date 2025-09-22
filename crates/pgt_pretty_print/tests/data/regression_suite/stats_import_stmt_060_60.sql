SELECT pg_catalog.pg_restore_attribute_stats(
    'schemaname', 'stats_import',
    'relname', 'test',
    'inherited', false::boolean,
    'null_frac', 0.1::real);
