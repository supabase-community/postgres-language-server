SELECT pg_catalog.pg_restore_attribute_stats(
    'schemaname', 'pg_temp',
    'relname', 'stats_temp',
    'attname', 'i',
    'inherited', false::boolean,
    'null_frac', 0.0123::real
    );
