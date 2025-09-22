SELECT stats_reset > 'slru_notify_reset_ts'::timestamptz FROM pg_stat_slru WHERE name = 'notify';
