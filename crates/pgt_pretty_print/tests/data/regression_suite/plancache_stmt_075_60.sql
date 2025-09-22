insert into test_mode select 1 from generate_series(1,1000) union all select 2;
