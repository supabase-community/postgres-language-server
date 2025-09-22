CREATE INDEX ON brin_timestamp_test USING brin (a timestamptz_minmax_multi_ops) WITH (pages_per_range=1);
