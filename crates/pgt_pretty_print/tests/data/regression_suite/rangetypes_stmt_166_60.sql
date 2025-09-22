insert into test_range_gist select int4range(g, g+10000) from generate_series(1,1000) g;
