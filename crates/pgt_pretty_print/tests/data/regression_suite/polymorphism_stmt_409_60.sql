select x, pg_typeof(x) from anyctest(11.2, multirange(int4range(4,7))) x;
