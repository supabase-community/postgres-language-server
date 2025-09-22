SELECT relname FROM pg_class WHERE relname = 'PG_CLASS'::text COLLATE case_insensitive;
