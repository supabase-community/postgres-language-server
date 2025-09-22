insert into gin_test_tbl select array[1, 3, g] from generate_series(1, 1000) g;
