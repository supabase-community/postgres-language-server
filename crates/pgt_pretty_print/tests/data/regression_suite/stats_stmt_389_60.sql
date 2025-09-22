SELECT sum(hits) AS io_sum_shared_after_hits
  FROM pg_stat_io WHERE context = 'normal' AND object = 'relation' ;
