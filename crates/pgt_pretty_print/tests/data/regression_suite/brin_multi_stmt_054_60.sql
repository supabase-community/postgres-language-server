CREATE INDEX brin_test_multi_1_idx_2 ON brin_test_multi_1 USING brin (b int8_minmax_multi_ops) WITH (pages_per_range=5);
