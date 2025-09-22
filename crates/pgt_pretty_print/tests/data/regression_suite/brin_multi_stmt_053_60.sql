CREATE INDEX brin_test_multi_1_idx_1 ON brin_test_multi_1 USING brin (a int4_minmax_multi_ops) WITH (pages_per_range=5);
