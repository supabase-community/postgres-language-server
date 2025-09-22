select pg_typeof(array(select '11 22 33'::int2vector from generate_series(1,5)));
