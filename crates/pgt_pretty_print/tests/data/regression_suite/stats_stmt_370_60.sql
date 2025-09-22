SELECT current_setting('fsync') = 'off'
  OR current_setting('wal_sync_method') IN ('open_sync', 'open_datasync')
  OR 'io_sum_wal_normal_after_fsyncs' > 'io_sum_wal_normal_before_fsyncs';
