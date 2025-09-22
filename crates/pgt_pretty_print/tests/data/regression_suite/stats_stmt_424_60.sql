SELECT sum(evictions) + sum(reuses) + sum(extends) + sum(fsyncs) + sum(reads) + sum(writes) + sum(writebacks) + sum(hits) AS io_stats_pre_reset
  FROM pg_stat_io ;
