SELECT attname, attcompression FROM pg_attribute
  WHERE attrelid = 'ctl_foreign_table2'::regclass and attnum > 0 ORDER BY attnum;
