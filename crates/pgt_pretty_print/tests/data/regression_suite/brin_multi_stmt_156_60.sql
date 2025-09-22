CREATE INDEX ON brin_interval_test USING brin (a interval_minmax_multi_ops) WITH (pages_per_range=1);
