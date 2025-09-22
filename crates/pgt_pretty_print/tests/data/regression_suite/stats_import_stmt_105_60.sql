SELECT pg_catalog.pg_restore_attribute_stats(
    'schemaname', 'stats_import',
    'relname', 'test',
    'attname', 'tags',
    'inherited', false::boolean,
    'null_frac', 0.35::real,
    'most_common_elem_freqs', '{0.3,0.2,0.2,0.3}'::real[]
    );
