INSERT INTO brin_test_bloom SELECT x/100,x%100 FROM generate_series(1,10000) x(x);
