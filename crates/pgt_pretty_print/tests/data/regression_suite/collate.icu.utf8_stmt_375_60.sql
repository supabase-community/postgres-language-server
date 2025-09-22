SELECT typname FROM pg_type WHERE typname LIKE 'int_' AND 'INT2'::text <> typname
  COLLATE case_insensitive ORDER BY typname;
