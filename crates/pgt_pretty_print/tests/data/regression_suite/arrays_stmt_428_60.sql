select pg_typeof(array(select '11 22 33'::oidvector from generate_series(1,5)));
