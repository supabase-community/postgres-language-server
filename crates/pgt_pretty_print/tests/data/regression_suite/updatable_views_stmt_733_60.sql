INSERT INTO t111
SELECT i,i,'t111','t111d','{1,1,1}'::int[] FROM generate_series(1,10) g(i);
