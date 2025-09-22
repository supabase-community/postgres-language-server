select size > 20, isdir from pg_stat_file('postmaster.pid');
