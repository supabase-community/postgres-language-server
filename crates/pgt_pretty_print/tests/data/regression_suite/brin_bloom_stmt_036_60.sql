CREATE INDEX brin_test_bloom_b_idx ON brin_test_bloom USING brin (b) WITH (pages_per_range = 2);
