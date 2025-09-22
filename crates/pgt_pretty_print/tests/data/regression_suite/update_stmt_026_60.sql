UPDATE update_test SET (b,a) = (select a+1,b from update_test);
