SELECT pg_catalog.pg_restore_attribute_stats(
    'schemaname', 'stats_import',
    'relname', 'test',
    'attname', 'id',
    'inherited', false::boolean,
    'null_frac', 0.36::real,
    'elem_count_histogram', '{1,1,1,1,1,1,1,1,1,1}'::real[]
    );
