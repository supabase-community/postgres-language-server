SELECT sum(extends) AS io_sum_bulkwrite_strategy_extends_before
  FROM pg_stat_io WHERE context = 'bulkwrite' ;
