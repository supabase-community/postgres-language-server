SELECT attname, atthasmissing, attmissingval FROM pg_attribute
  WHERE attrelid = 't'::regclass AND attnum > 0
  ORDER BY attnum;
