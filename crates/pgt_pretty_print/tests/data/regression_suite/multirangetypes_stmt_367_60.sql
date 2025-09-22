insert into test_multirange_gist select '{}'::int4multirange from generate_series(1,500) g;
