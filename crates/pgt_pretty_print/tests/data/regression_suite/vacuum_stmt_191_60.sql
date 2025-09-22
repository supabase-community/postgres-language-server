SELECT relname, last_analyze IS NOT NULL AS analyzed, last_vacuum IS NOT NULL AS vacuumed
  FROM pg_stat_user_tables
  WHERE relid IN ('only_parted'::regclass, 'only_parted1'::regclass)
  ORDER BY relname;
