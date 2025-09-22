CREATE INDEX brinidx_bloom ON brintest_bloom USING brin (
	byteacol bytea_bloom_ops(n_distinct_per_range = -1.1)
);
