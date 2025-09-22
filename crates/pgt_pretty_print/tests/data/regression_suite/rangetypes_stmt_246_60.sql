insert into test_range_spgist select int4range(NULL,g*10,'(]') from generate_series(1,100) g;
