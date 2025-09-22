SELECT sum(reads) AS io_sum_local_before_reads
  FROM pg_stat_io WHERE context = 'normal' AND object = 'temp relation' ;
