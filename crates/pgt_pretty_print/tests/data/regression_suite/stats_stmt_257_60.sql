SELECT (current_schemas(true))[1] = ('pg_temp_' || beid::text) AS match
FROM pg_stat_get_backend_idset() beid
WHERE pg_stat_get_backend_pid(beid) = pg_backend_pid();
