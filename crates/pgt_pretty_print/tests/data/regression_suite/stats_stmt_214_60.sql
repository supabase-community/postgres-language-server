SELECT seq_scan, 'test_last_seq' < last_seq_scan AS seq_ok, idx_scan, 'test_last_idx' = last_idx_scan AS idx_ok
FROM pg_stat_all_tables WHERE relid = 'test_last_scan'::regclass;
