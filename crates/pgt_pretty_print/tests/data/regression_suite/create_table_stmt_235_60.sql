SELECT conname, conislocal, coninhcount FROM pg_constraint WHERE conrelid = 'part_b'::regclass ORDER BY coninhcount DESC, conname;
