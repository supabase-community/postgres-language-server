SELECT stats_reset > 'db_reset_ts'::timestamptz FROM pg_stat_database WHERE datname = (SELECT current_database());
