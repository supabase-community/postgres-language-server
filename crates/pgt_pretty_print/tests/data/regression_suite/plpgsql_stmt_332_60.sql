select x, pg_typeof(x), y, pg_typeof(y)
  from f1(11, array[1, 2], point(1,2), point(3,4));
