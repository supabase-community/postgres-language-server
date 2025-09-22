CREATE INDEX brin_test_multi_2_idx ON brin_test_multi_2 USING brin (a uuid_minmax_multi_ops) WITH (pages_per_range=5);
