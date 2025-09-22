SELECT seq_scan, idx_scan FROM pg_stat_all_tables WHERE relid = 'test_last_scan'::regclass;
