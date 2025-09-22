SELECT *, pg_typeof(f3), pg_typeof(f4) FROM dup(22, array[44::bigint]);
