SELECT stats_reset > 'slru_commit_ts_reset_ts'::timestamptz FROM pg_stat_slru WHERE name = 'commit_timestamp';
