select * from t_gin_test_tbl where array[0] <@ i and '{}'::int4[] <@ j;
