CREATE INDEX ON brin_timestamp_test USING brin (a timestamp_minmax_multi_ops) WITH (pages_per_range=1);
