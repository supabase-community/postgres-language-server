insert into test_multirange_gist select int4multirange(int4range(g, g+10000)) from generate_series(1,1000) g;
