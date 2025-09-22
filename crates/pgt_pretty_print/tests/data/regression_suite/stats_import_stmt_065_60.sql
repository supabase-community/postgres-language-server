SELECT pg_catalog.pg_restore_attribute_stats(
    'schemaname', 'stats_import',
    'relname', 'test',
    'attnum', 1::smallint,
    'inherited', false::boolean,
    'null_frac', 0.4::real);
