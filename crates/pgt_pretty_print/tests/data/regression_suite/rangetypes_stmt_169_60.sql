insert into test_range_gist select int4range(g*10,NULL,'(]') from generate_series(1,100) g;
