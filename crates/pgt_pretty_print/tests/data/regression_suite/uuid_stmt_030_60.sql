SELECT count(*) FROM pg_class WHERE relkind='i' AND relname LIKE 'guid%';
