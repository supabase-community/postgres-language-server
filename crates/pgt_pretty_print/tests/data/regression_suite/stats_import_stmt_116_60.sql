SELECT s.schemaname, s.tablename, s.attname, s.inherited, r.*
FROM pg_catalog.pg_stats AS s
CROSS JOIN LATERAL
    pg_catalog.pg_restore_attribute_stats(
        'schemaname', 'stats_import',
        'relname', s.tablename::text || '_clone',
        'attname', s.attname::text,
        'inherited', s.inherited,
        'version', 150000,
        'null_frac', s.null_frac,
        'avg_width', s.avg_width,
        'n_distinct', s.n_distinct,
        'most_common_vals', s.most_common_vals::text,
        'most_common_freqs', s.most_common_freqs,
        'histogram_bounds', s.histogram_bounds::text,
        'correlation', s.correlation,
        'most_common_elems', s.most_common_elems::text,
        'most_common_elem_freqs', s.most_common_elem_freqs,
        'elem_count_histogram', s.elem_count_histogram,
        'range_bounds_histogram', s.range_bounds_histogram::text,
        'range_empty_frac', s.range_empty_frac,
        'range_length_histogram', s.range_length_histogram::text) AS r
WHERE s.schemaname = 'stats_import'
AND s.tablename IN ('test', 'is_odd')
ORDER BY s.tablename, s.attname, s.inherited;
