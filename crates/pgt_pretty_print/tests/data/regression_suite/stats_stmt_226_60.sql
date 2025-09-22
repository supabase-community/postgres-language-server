SELECT last_seq_scan AS test_last_seq, last_idx_scan AS test_last_idx
FROM pg_stat_all_tables WHERE relid = 'test_last_scan'::regclass ;
