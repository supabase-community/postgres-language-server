SELECT max(stats_reset) > 'slru_reset_ts'::timestamptz FROM pg_stat_slru;
