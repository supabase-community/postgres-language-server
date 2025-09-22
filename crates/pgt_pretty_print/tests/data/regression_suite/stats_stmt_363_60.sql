SELECT current_setting('fsync') = 'off'
  OR 'io_sum_shared_after_fsyncs' > 'io_sum_shared_before_fsyncs';
