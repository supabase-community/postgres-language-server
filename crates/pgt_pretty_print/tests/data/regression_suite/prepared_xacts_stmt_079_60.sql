SELECT gid FROM pg_prepared_xacts WHERE gid ~ '^regress_' ORDER BY gid;
