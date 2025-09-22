SELECT count(*) FROM pg_locks WHERE locktype = 'advisory' AND database = 'datoid';
