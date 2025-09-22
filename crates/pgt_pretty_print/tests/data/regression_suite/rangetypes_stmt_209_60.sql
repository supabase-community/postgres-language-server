select count(*) from test_range_gist where ir @> '{}'::int4multirange;
