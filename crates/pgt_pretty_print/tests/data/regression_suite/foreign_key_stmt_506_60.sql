select conname from pg_constraint where conrelid = 'fktable2'::regclass order by conname;
