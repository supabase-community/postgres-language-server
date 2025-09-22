select pg_typeof(unnest(array['11 22 33'::int2vector]));
