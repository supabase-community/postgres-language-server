SELECT backend_type, object, context FROM pg_stat_io
  ORDER BY backend_type COLLATE "C", object COLLATE "C", context COLLATE "C";
