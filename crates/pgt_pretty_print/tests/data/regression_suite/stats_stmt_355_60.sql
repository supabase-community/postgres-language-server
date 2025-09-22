SELECT sum(extends) AS io_sum_shared_after_extends
  FROM pg_stat_io WHERE context = 'normal' AND object = 'relation' ;
