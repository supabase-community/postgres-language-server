SELECT pg_catalog.pg_clear_attribute_stats(
    schemaname => 'stats_import',
    relname => 'test',
    attname => 'arange',
    inherited => false);
