SELECT sum(writes) AS io_sum_local_new_tblspc_writes
  FROM pg_stat_io WHERE context = 'normal' AND object = 'temp relation'  ;
