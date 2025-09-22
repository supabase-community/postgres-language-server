SELECT sum(evictions) + sum(reuses) + sum(extends) + sum(fsyncs) + sum(reads) + sum(writes) + sum(writebacks) + sum(hits) AS my_io_stats_post_reset
  FROM pg_stat_get_backend_io(pg_backend_pid()) ;
