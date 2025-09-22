INSERT INTO vacparted_i SELECT i, 'test_'|| i from generate_series(1,10) i;
