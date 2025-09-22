SELECT conname, convalidated, conrelid::regclass FROM pg_constraint
WHERE conrelid::regclass::text like 'fk_partitioned_fk%' ORDER BY oid::regclass::text;
