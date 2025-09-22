INSERT INTO t1
SELECT i,i,'t1' FROM generate_series(1,10) g(i);
