insert into t_gin_test_tbl select array[1, g, g/10], array[2, g, g/10]
  from generate_series(1, 20000) g;
