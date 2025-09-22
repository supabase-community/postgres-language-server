insert into test_range_gist select int4range(g, g+10) from generate_series(1,2000) g;
