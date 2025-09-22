SELECT wal_bytes > 'backend_wal_bytes_before' FROM pg_stat_get_backend_wal(pg_backend_pid());
