SELECT sum(writes) AS writes, sum(fsyncs) AS fsyncs
  FROM pg_stat_get_backend_io(pg_backend_pid())
  WHERE object = 'relation' ;
