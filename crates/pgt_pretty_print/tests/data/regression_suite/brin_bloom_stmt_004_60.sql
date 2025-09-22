CREATE INDEX brinidx_bloom ON brintest_bloom USING brin (
	byteacol bytea_bloom_ops(false_positive_rate = 0.00009)
);
