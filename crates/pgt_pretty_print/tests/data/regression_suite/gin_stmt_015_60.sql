select count(*) from gin_test_tbl where i @> array[1, 999];
