SELECT wal_bytes AS backend_wal_bytes_before from pg_stat_get_backend_wal(pg_backend_pid()) ;
