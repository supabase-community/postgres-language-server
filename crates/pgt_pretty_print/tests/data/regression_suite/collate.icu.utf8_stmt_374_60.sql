SELECT typname FROM pg_type WHERE typname LIKE 'int_' AND typname <> 'INT2'::text
  COLLATE case_insensitive ORDER BY typname;
