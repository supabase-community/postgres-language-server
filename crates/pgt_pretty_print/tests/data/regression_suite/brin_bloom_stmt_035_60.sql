CREATE INDEX brin_test_bloom_a_idx ON brin_test_bloom USING brin (a) WITH (pages_per_range = 2);
