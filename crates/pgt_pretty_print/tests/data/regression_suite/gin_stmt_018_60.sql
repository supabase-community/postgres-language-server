select count(*) > 0 as ok from gin_test_tbl where i @> array[1];
