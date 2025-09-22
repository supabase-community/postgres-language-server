SELECT relname FROM pg_class WHERE 'PG_CLASS'::text = relname COLLATE case_insensitive;
