SELECT sum(extends) AS my_io_sum_shared_before_extends
  FROM pg_stat_get_backend_io(pg_backend_pid())
  WHERE context = 'normal' AND object = 'relation' ;
