select max(histogram_bounds) from pg_stats where tablename = 'pg_am';
