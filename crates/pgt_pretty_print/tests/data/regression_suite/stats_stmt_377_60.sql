SELECT sum(reads) AS io_sum_shared_after_reads
  FROM pg_stat_io WHERE context = 'normal' AND object = 'relation'  ;
