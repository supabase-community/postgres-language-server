SELECT sum(evictions) AS evictions,
       sum(reads) AS reads,
       sum(writes) AS writes,
       sum(extends) AS extends
  FROM pg_stat_io
  WHERE context = 'normal' AND object = 'temp relation'  ;
