SELECT sum(writes) AS writes, sum(fsyncs) AS fsyncs
  FROM pg_stat_io
  WHERE object = 'relation' ;
