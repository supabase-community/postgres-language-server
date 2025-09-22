CREATE INDEX brinidx_multi ON brintest_multi USING brin (
	int8col int8_minmax_multi_ops(values_per_range = 7)
);
