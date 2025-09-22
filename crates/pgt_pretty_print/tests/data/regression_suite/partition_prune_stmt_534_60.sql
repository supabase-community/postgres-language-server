select * from ma_test where a >= (select min(b) from ma_test_p2) order by b;
