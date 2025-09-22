SELECT sessions AS db_stat_sessions FROM pg_stat_database WHERE datname = (SELECT current_database()) ;
