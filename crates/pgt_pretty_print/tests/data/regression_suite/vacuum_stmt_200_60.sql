SELECT relname, last_analyze IS NOT NULL AS analyzed, last_vacuum IS NOT NULL AS vacuumed
  FROM pg_stat_user_tables
  WHERE relid IN ('only_inh_parent'::regclass, 'only_inh_child'::regclass)
  ORDER BY relname;
