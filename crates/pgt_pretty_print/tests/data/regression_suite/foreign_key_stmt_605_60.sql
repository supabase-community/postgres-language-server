SELECT conname, convalidated FROM pg_constraint
WHERE conrelid = 'fk_partitioned_fk_2'::regclass ORDER BY oid::regclass::text;
