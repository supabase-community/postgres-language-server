SELECT current_setting('fsync') = 'off'
  OR 'my_io_sum_shared_after_fsyncs' >= 'my_io_sum_shared_before_fsyncs';
