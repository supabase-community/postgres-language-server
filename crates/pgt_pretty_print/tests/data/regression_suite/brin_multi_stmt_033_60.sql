CREATE INDEX brin_large_range_idx ON brin_large_range USING brin (a int4_minmax_multi_ops);
