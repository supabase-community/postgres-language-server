select x, pg_typeof(x) from anyctest(11, array[1, 2.2], 42, 34.5) x;
