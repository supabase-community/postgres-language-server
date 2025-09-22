select x, pg_typeof(x) from anyctest(11, array[1, 2], point(1,2), point(3,4)) x;
