SELECT sum(extends) AS extends, sum(evictions) AS evictions, sum(writes) AS writes
  FROM pg_stat_io
  WHERE context = 'normal' AND object = 'temp relation' ;
